//! The ROS 2 adapter: node `/rovi` over zenoh-nostd and `rmw_zenohd`, the same wiring as the
//! sim car. The car is a zenoh client of the router on the laptop that joined the car AP.

extern crate alloc;

use super::{log, now};
use core::cell::RefCell;
use embassy_futures::select::select5;
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
/// endpoints, and feeds the samples to the device.
pub async fn run(dev: &RefCell<Device>, stack: Stack<'static>) -> ! {
    static CONFIG: StaticCell<RosConfig> = StaticCell::new();
    static RESOURCES: StaticCell<Resources<'static, RosConfig>> = StaticCell::new();
    static SESSION: StaticCell<Session<'static, RosConfig>> = StaticCell::new();
    static CMD: StaticCell<Chan> = StaticCell::new();
    static ARM: StaticCell<Chan> = StaticCell::new();
    static TOKENS: StaticCell<TokenChan> = StaticCell::new();

    let config: &'static RosConfig = CONFIG.init(RosConfig {
        transports: TransportLinkManager::from(Link::new(stack)),
    });
    let zid = alloc::format!("{:?}", config.transports().zid());
    let mut transport = loop {
        let endpoint = Endpoint::try_from(ROUTER).expect("ROVI_ZENOH_ROUTER endpoint");
        match config.transports().connect(endpoint, config.buff()).await {
            Ok(t) => break t,
            Err(_) => {
                println!("ros: router {} not reachable, retrying", ROUTER);
                Timer::after_secs(2).await;
            }
        }
    };
    // ROS mixes reliable and best-effort traffic, which zenoh sequences separately, but
    // zenoh-nostd tracks one sequence number and would drop frames. So skip the check.
    transport.transport_mut().rx.ignore_invalid_sn();
    let session: &'static Session<'static, RosConfig> = SESSION.init(Session::new(
        RESOURCES.init(Resources::default()).init(transport),
    ));
    let (cmd, arm, tokens) = (
        CMD.init(Channel::new()),
        ARM.init(Channel::new()),
        TOKENS.init(Channel::new()),
    );

    let err = declare_and_serve(dev, session, &zid, cmd, arm, tokens).await;
    println!("ros adapter stopped: {:?}", err.err());
    // The session cannot be reopened in place; a reboot reconnects.
    loop {
        Timer::after_secs(3600).await;
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

    select5(
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
    Ok(())
}
