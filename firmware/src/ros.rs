//! The ROS 2 adapter: node `/rovi` over zenoh-nostd and `rmw_zenohd`, the same wiring as the
//! sim car. The car is a zenoh client of the router on the laptop that joined the car AP.

extern crate alloc;

use super::{log, logger::emit, now};
use core::cell::{Cell, RefCell};
use embassy_futures::select::{select, select5, Either, Either5};
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use embassy_time::Timer;
use esp_println::println;
use rovi_core::device::Device;
use rovi_core::events::{Event, Link as RosLink};
use rovi_core::rmw::{self, Attachment, Endpoint as Kind, Topic};
use static_cell::StaticCell;
use zenoh_embassy::EmbassyLinkManager;
use zenoh_nostd::session::*;

const DOMAIN: u32 = 0;
const NODE: &str = "rovi";
/// TCP, which a stock `rmw_zenohd` listens on. Over UDP the router's listener can stop accepting
/// every peer after a stall (eclipse-zenoh/zenoh#2849), so the car could not rejoin (ADR-0012).
const ROUTER: &str = match option_env!("ROVI_ZENOH_ROUTER") {
    Some(v) => v,
    None => "tcp/192.168.4.2:7447",
};

/// Link buffers (TCP uses the same 512-byte transmit and 8192-byte receive buffers). A router answers the liveliness interest with a burst of datagrams, and UDP
/// does not resend what a full receive queue drops: 8 slots and 512 bytes (the defaults) lost the
/// controller's publisher tokens on the board, so the car never learned its node. 32 slots and
/// 8192 bytes held a three-node graph; SOCKS 2 covers a reconnect while the old link is dropped.
type Link = EmbassyLinkManager<'static, 512, 2, 32, 8192>;

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
    static ECHO: StaticCell<Chan> = StaticCell::new();

    let config: &'static RosConfig = CONFIG.init(RosConfig {
        transports: TransportLinkManager::from(Link::new(stack)),
    });
    let zid = alloc::format!("{:?}", config.transports().zid());
    let (cmd, arm, tokens, echo) = (
        CMD.init(Channel::new()),
        ARM.init(Channel::new()),
        TOKENS.init(Channel::new()),
        ECHO.init(Channel::new()),
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
                    emit(Event::Ros(RosLink::Retry));
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
        let result =
            declare_and_serve(dev, unsafe { &*session }, &zid, cmd, arm, tokens, echo).await;
        println!("ros session ended ({:?}), reconnecting", result.err());
        emit(Event::Ros(RosLink::Down));
        // The router is gone: a ROS owner is released now, not only by its lease.
        log(dev.borrow_mut().on_ros_disconnect(now()));
        // SAFETY: `declare_and_serve` returned, so every borrow of the session (tokens,
        // subscribers, publisher) is gone. The session points into the resources, so it goes
        // first.
        drop(unsafe { alloc::boxed::Box::from_raw(session) });
        drop(unsafe { alloc::boxed::Box::from_raw(resources) });
        // Samples queued by the old session are stale.
        while cmd.try_receive().is_ok() {}
        while arm.try_receive().is_ok() {}
        while tokens.try_receive().is_ok() {}
        while echo.try_receive().is_ok() {}
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
    echo: &'static Chan,
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
    let echo_t = Topic {
        name: "rovi/echo",
        ty: rmw::INT64.0,
        hash: rmw::INT64.1,
    };
    let echo_reply_t = Topic {
        name: "rovi/echo_reply",
        ty: rmw::INT64.0,
        hash: rmw::INT64.1,
    };
    let status_t = Topic {
        name: "rovi/status",
        ty: rmw::STRING.0,
        hash: rmw::STRING.1,
    };

    let capabilities_t = Topic {
        name: "rovi/capabilities",
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
        (4, Kind::Subscription, &echo_t, rmw::QOS_BEST_EFFORT_1),
        (5, Kind::Publisher, &echo_reply_t, rmw::QOS_BEST_EFFORT_1),
        (
            6,
            Kind::Publisher,
            &capabilities_t,
            rmw::QOS_RELIABLE_VOLATILE_1,
        ),
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
    let echo_gid = rmw::gid(eps[4].as_str());
    let capabilities_gid = rmw::gid(eps[5].as_str());

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
    let lv_ke = zenoh::keyexpr::new(leak(alloc::format!("@ros2_lv/{DOMAIN}/*/*/*/MP/**")))?;
    let lv_sub = session
        .declare_liveliness_subscriber(lv_ke)
        .channel(tokens.dyn_sender(), tokens.dyn_receiver())
        .finish()
        .await?;
    let echo_sub = session
        .declare_subscriber(zenoh::keyexpr::new(key(&echo_t))?)
        .channel(echo.dyn_sender(), echo.dyn_receiver())
        .finish()
        .await?;
    let echo_pub = session
        .declare_publisher(zenoh::keyexpr::new(key(&echo_reply_t))?)
        .finish()
        .await?;
    let status_pub = session
        .declare_publisher(zenoh::keyexpr::new(key(&status_t))?)
        .finish()
        .await?;
    let capabilities_pub = session
        .declare_publisher(zenoh::keyexpr::new(key(&capabilities_t))?)
        .finish()
        .await?;
    println!("ros node /{} up, zid {}, router {}", NODE, zid, ROUTER);
    emit(Event::Ros(RosLink::Up));

    let last_refresh = Cell::new(None::<u64>);
    let echo_loop = async {
        // Latency probe: any ROS node may use it, no claim; the payload comes back unchanged.
        let mut seq = 0i64;
        while let Some(s) = echo_sub.recv().await {
            if let Some(bytes) = rovi_core::device::echo(s.payload()) {
                seq += 1;
                let att = Attachment {
                    seq,
                    timestamp: 0,
                    gid: echo_gid,
                }
                .encode();
                let _ = echo_pub.put(bytes).attachment(&att).finish().await;
            }
        }
    };
    let capabilities_loop = async {
        let mut schedule = rovi_core::capabilities::Publication::default();
        let mut seq = 0i64;
        let mut buf = [0; rovi_core::capabilities::CDR_MAX];
        loop {
            let pending = {
                let d = dev.borrow();
                schedule
                    .due(now(), d.arbiter().session(), d.arbiter().owner().is_some())
                    .then(|| d.capabilities_snapshot().encode_cdr(&mut buf))
                    .flatten()
            };
            if let Some(n) = pending {
                seq += 1;
                let att = Attachment {
                    seq,
                    timestamp: 0,
                    gid: capabilities_gid,
                }
                .encode();
                if let Err(e) = capabilities_pub
                    .put(&buf[..n])
                    .attachment(&att)
                    .finish()
                    .await
                {
                    break Err::<(), _>(e);
                }
            }
            Timer::after_millis(10).await;
        }
    };
    let ended = select(
        select5(
            session.run(),
            async {
                while let Some(s) = cmd_sub.recv().await {
                    maybe_refresh(
                        session,
                        lv_ke,
                        dev,
                        s.attachment().unwrap_or(&[]),
                        &last_refresh,
                    )
                    .await;
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
                    maybe_refresh(
                        session,
                        lv_ke,
                        dev,
                        s.attachment().unwrap_or(&[]),
                        &last_refresh,
                    )
                    .await;
                    let fx = dev.borrow_mut().on_ros_arm(
                        now(),
                        s.attachment().unwrap_or(&[]),
                        s.payload(),
                    );
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
                let mut schedule = rovi_core::status::Publication::default();
                let mut buf = [0u8; rovi_core::status::CDR_MAX];
                loop {
                    let snapshot = dev.borrow().status_snapshot();
                    if schedule.due(now(), snapshot) {
                        if let Some(n) = snapshot.encode_cdr(&mut buf) {
                            seq += 1;
                            let att = Attachment {
                                seq,
                                timestamp: 0,
                                gid: status_gid,
                            }
                            .encode();
                            let _ = status_pub.put(&buf[..n]).attachment(&att).finish().await;
                        }
                    }
                    Timer::after_millis(10).await;
                }
            },
        ),
        select(echo_loop, capabilities_loop),
    )
    .await;
    // Report the session or publication failure before the adapter reconnects.
    match ended {
        Either::First(Either5::First(Err(e))) => return Err(e.into()),
        Either::Second(Either::Second(Err(e))) => return Err(e.into()),
        _ => {}
    }
    Ok(())
}

/// A sample from a publisher no token announced means a token was lost (UDP does not resend):
/// ask the router for the current tokens again, at most every 2 s.
async fn maybe_refresh(
    session: &Session<'static, RosConfig>,
    lv_ke: &zenoh::keyexpr,
    dev: &RefCell<Device>,
    attachment: &[u8],
    last: &Cell<Option<u64>>,
) {
    let at = now();
    if dev.borrow().ros_publisher_known(attachment) != Some(false)
        || last.get().is_some_and(|t| at < t + 2000)
    {
        return;
    }
    last.set(Some(at));
    emit(Event::Ros(RosLink::Refresh));
    let _ = session.liveliness_refresh(lv_ke).await;
}
