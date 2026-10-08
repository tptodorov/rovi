//! The ROS 2 adapter: node `/rovi` over zenoh-nostd and `rmw_zenohd`, the same wiring as the
//! sim car. The car is a zenoh client of the router on the laptop that joined the car AP.

extern crate alloc;

use super::{log, now};
use core::cell::RefCell;
use embassy_futures::select::{select5, Either5};
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use embassy_time::Timer;
use esp_println::println;
use rovi_m8_setpoint_ros::device::Device;
use rovi_m8_setpoint_ros::rmw::{self, Attachment, Endpoint as Kind, Topic};
use static_cell::StaticCell;
use zenoh_embassy::EmbassyLinkManager;
use zenoh_nostd::session::*;

const DOMAIN: u32 = 0;
const NODE: &str = "rovi";
/// `rmw_zenohd` must listen on UDP (see the README).
const ROUTER: &str = match option_env!("ROVI_ZENOH_ROUTER") {
    Some(v) => v,
    None => "udp/192.168.4.2:7448",
};

type Link = EmbassyLinkManager<'static, 512, 3>;

struct RosConfig {
    transports: TransportLinkManager<Link>,
}

impl ZSessionConfig for RosConfig {
    type LinkManager = Link;
    type Buff = alloc::vec::Vec<u8>;
    type SubCallbacks<'res> = AllocSubCallbacks<'res, zenoh::storage::Box, zenoh::storage::Box>;
    type GetCallbacks<'res> = AllocGetCallbacks<'res, zenoh::storage::Box, zenoh::storage::Box>;
    type QueryableCallbacks<'res> =
        AllocQueryableCallbacks<'res, Self, zenoh::storage::Box, zenoh::storage::Box>;

    fn buff(&self) -> Self::Buff {
        alloc::vec![0; 512]
    }

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager> {
        &self.transports
    }
}

type Chan = Channel<NoopRawMutex, FixedCapacitySample<192, 160>, 4>;
type TokenChan = Channel<NoopRawMutex, FixedCapacitySample<384, 8>, 8>;

fn leak(s: alloc::string::String) -> &'static str {
    alloc::boxed::Box::leak(s.into_boxed_str())
}

/// Runs forever: connects to the router (retrying until it is up), declares the node and its
/// endpoints, feeds the samples to the device, and starts over when the session ends.
pub async fn run(dev: &RefCell<Device>, stack: Stack<'static>) -> ! {
    static CONFIG: StaticCell<RosConfig> = StaticCell::new();
    static CMD: StaticCell<Chan> = StaticCell::new();
    static ARM: StaticCell<Chan> = StaticCell::new();
    static TOKENS: StaticCell<TokenChan> = StaticCell::new();

    let config: &'static RosConfig = CONFIG.init(RosConfig {
        transports: TransportLinkManager::from(Link::new(stack)),
    });
    let zid = alloc::format!("{:?}", config.transports().zid());
    let (cmd, arm, tokens) = (
        CMD.init(Channel::new()),
        ARM.init(Channel::new()),
        TOKENS.init(Channel::new()),
    );

    // One session per connection. Each gets heap storage that is reclaimed once it has ended
    // (the link frees its buffer slot when dropped), so the car can reconnect indefinitely.
    loop {
        let transport = loop {
            let endpoint = Endpoint::try_from(ROUTER).expect("ROVI_ZENOH_ROUTER endpoint");
            match config.transports().connect(endpoint, config.buff()).await {
                Ok(t) => break t,
                Err(_) => {
                    println!("ros: router {} not reachable, retrying", ROUTER);
                    Timer::after_secs(2).await;
                }
            }
        };
        let resources: *mut Resources<'static, RosConfig> =
            alloc::boxed::Box::into_raw(alloc::boxed::Box::new(Resources::default()));
        // SAFETY: `resources` is a live allocation that nothing else references; the session
        // below is dropped before it is freed.
        let link = unsafe { &mut *resources }.init(transport);
        let session: *mut Session<'static, RosConfig> =
            alloc::boxed::Box::into_raw(alloc::boxed::Box::new(Session::new(link)));
        // SAFETY: the session lives until it is reclaimed below.
        let result = declare_and_serve(dev, unsafe { &*session }, &zid, cmd, arm, tokens).await;
        println!("ros session ended ({:?}), reconnecting", result.err());
        // SAFETY: `declare_and_serve` returned, so every borrow of the session (tokens,
        // subscribers, publisher) is gone. The session points into the resources, so it goes
        // first.
        drop(unsafe { alloc::boxed::Box::from_raw(session) });
        drop(unsafe { alloc::boxed::Box::from_raw(resources) });
        // Samples queued by the old session are stale.
        while cmd.try_receive().is_ok() {}
        while arm.try_receive().is_ok() {}
        while tokens.try_receive().is_ok() {}
        Timer::after_secs(2).await;
    }
}

async fn declare_and_serve(
    dev: &RefCell<Device>,
    session: &'static Session<'static, RosConfig>,
    zid: &str,
    cmd: &'static Chan,
    arm: &'static Chan,
    tokens: &'static TokenChan,
) -> zenoh::ZResult<()> {
    let cmd_vel = Topic {
        name: "cmd_vel",
        ty: rmw::TWIST_STAMPED.0,
        hash: rmw::TWIST_STAMPED.1,
    };
    let arm_t = Topic {
        name: "rovi/arm",
        ty: rmw::BOOL.0,
        hash: rmw::BOOL.1,
    };
    let status_t = Topic {
        name: "rovi/status",
        ty: rmw::STRING.0,
        hash: rmw::STRING.1,
    };

    let node = rmw::node_token(DOMAIN, zid, NODE).unwrap();
    let _node = session
        .declare_token(zenoh::keyexpr::new(node.as_str())?)
        .await?;
    let eps = [
        (1, Kind::Subscription, &cmd_vel, rmw::QOS_BEST_EFFORT_1),
        (2, Kind::Subscription, &arm_t, rmw::QOS_BEST_EFFORT_5),
        (3, Kind::Publisher, &status_t, rmw::QOS_DEFAULT_10),
    ]
    .map(|(id, kind, topic, qos)| {
        rmw::endpoint_token(DOMAIN, zid, id, kind, NODE, topic, qos).unwrap()
    });
    for t in &eps {
        let _ = session
            .declare_token(zenoh::keyexpr::new(t.as_str())?)
            .await?;
    }
    let status_gid = rmw::gid(eps[2].as_str());

    let key = |t: &Topic| leak(rmw::topic_key(DOMAIN, t).unwrap().as_str().into());
    let cmd_sub = session
        .declare_subscriber(zenoh::keyexpr::new(key(&cmd_vel))?)
        .channel(cmd.dyn_sender(), cmd.dyn_receiver())
        .finish()
        .await?;
    let arm_sub = session
        .declare_subscriber(zenoh::keyexpr::new(key(&arm_t))?)
        .channel(arm.dyn_sender(), arm.dyn_receiver())
        .finish()
        .await?;
    // Learn which ROS node each publisher gid belongs to from the graph's liveliness tokens.
    let lv_sub = session
        .declare_liveliness_subscriber(zenoh::keyexpr::new(leak(alloc::format!(
            "@ros2_lv/{DOMAIN}/**"
        )))?)
        .channel(tokens.dyn_sender(), tokens.dyn_receiver())
        .finish()
        .await?;
    let status_pub = session
        .declare_publisher(zenoh::keyexpr::new(key(&status_t))?)
        .finish()
        .await?;
    println!("ros node /{} up, zid {}, router {}", NODE, zid, ROUTER);

    let ended = select5(
        session.run(),
        async {
            while let Some(s) = cmd_sub.recv().await {
                let fx = dev.borrow_mut().on_ros_cmd_vel(
                    now(),
                    s.attachment().unwrap_or(&[]),
                    s.payload(),
                );
                log(fx);
            }
        },
        async {
            while let Some(s) = arm_sub.recv().await {
                let fx =
                    dev.borrow_mut()
                        .on_ros_arm(now(), s.attachment().unwrap_or(&[]), s.payload());
                log(fx);
            }
        },
        async {
            while let Some(s) = lv_sub.recv().await {
                let alive = s.payload() == [1];
                dev.borrow_mut()
                    .on_ros_liveliness(s.keyexpr().as_str(), alive);
            }
        },
        async {
            let mut seq = 0i64;
            loop {
                let mut buf = [0u8; 80];
                let n = dev.borrow().status_cdr(&mut buf);
                seq += 1;
                let att = Attachment {
                    seq,
                    timestamp: 0,
                    gid: status_gid,
                }
                .encode();
                let _ = status_pub.put(&buf[..n]).attachment(&att).finish().await;
                Timer::after_secs(1).await;
            }
        },
    )
    .await;
    // Only the session loop ends on its own; say why it did.
    if let Either5::First(Err(e)) = ended {
        return Err(e.into());
    }
    Ok(())
}
