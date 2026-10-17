//! The ROS 2 sim adapter: node `/rovi` over zenoh-nostd and `rmw_zenohd`, feeding the shared
//! device. Subscribes `/cmd_vel` (TwistStamped) and `/rovi/arm` (Bool), publishes `/rovi/status`.

use crate::{log, log_effects, ms, Shared};
use embassy_futures::select::{select, select5, Either, Either5};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use rovi_core::events::{Event, Link};
use rovi_core::rmw::{self, Attachment, Endpoint as Kind, Topic};
use static_cell::StaticCell;
use std::cell::Cell;
use std::time::Instant;
use zenoh_nostd::session::*;
use zenoh_std::StdLinkManager as LinkManager;

const DOMAIN: u32 = 0;
const NODE: &str = "rovi";

struct RosConfig {
    transports: TransportLinkManager<LinkManager>,
}

impl ZSessionConfig for RosConfig {
    type LinkManager = LinkManager;
    type Buff = Vec<u8>;
    type SubCallbacks<'res> = AllocSubCallbacks<'res, zenoh::storage::Box, zenoh::storage::Box>;
    type GetCallbacks<'res> = AllocGetCallbacks<'res, zenoh::storage::Box, zenoh::storage::Box>;
    type QueryableCallbacks<'res> =
        AllocQueryableCallbacks<'res, Self, zenoh::storage::Box, zenoh::storage::Box>;

    fn buff(&self) -> Self::Buff {
        vec![0; u16::MAX as usize]
    }

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager> {
        &self.transports
    }
}

type Chan = Channel<NoopRawMutex, FixedCapacitySample<192, 160>, 8>;
type TokenChan = Channel<NoopRawMutex, FixedCapacitySample<384, 8>, 16>;

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn run(dev: Shared, t0: Instant) {
    static EXECUTOR: StaticCell<embassy_executor::Executor> = StaticCell::new();
    let executor = EXECUTOR.init(embassy_executor::Executor::new());
    executor.run(|spawner| spawner.spawn(task(dev.clone(), t0)).unwrap());
}

#[embassy_executor::task]
async fn task(dev: Shared, t0: Instant) {
    env_logger::init();
    if let Err(e) = entry(dev, t0).await {
        println!("ros adapter stopped: {e}");
    }
}

async fn entry(dev: Shared, t0: Instant) -> zenoh::ZResult<()> {
    static CMD: StaticCell<Chan> = StaticCell::new();
    static ARM: StaticCell<Chan> = StaticCell::new();
    static TOKENS: StaticCell<TokenChan> = StaticCell::new();
    static ECHO: StaticCell<Chan> = StaticCell::new();
    static CONFIG: StaticCell<RosConfig> = StaticCell::new();
    let cmd: &'static Chan = CMD.init(Channel::new());
    let arm: &'static Chan = ARM.init(Channel::new());
    let lv_chan: &'static TokenChan = TOKENS.init(Channel::new());
    let echo: &'static Chan = ECHO.init(Channel::new());
    let router =
        leak(std::env::var("ROVI_ZENOH_ROUTER").unwrap_or_else(|_| "tcp/127.0.0.1:7447".into()));
    let config: &'static RosConfig = CONFIG.init(RosConfig {
        transports: TransportLinkManager::from(LinkManager),
    });
    let zid = format!("{:?}", config.transports().zid());

    // One session per connection. Each gets heap storage that is reclaimed once it has ended, so
    // the adapter can reconnect for as long as the car runs.
    loop {
        let resources: *mut Resources<'static, RosConfig> =
            Box::into_raw(Box::new(Resources::default()));
        // SAFETY: `resources` is a live allocation that nothing else references.
        let connected = zenoh::connect(
            unsafe { &mut *resources },
            config,
            Endpoint::try_from(router)?,
        )
        .await;
        let Ok(session) = connected else {
            // SAFETY: a failed connect did not keep the resources.
            drop(unsafe { Box::from_raw(resources) });
            println!("ros: router {router} not reachable, retrying");
            log(Event::Ros(Link::Retry), t0);
            embassy_time::Timer::after_secs(2).await;
            continue;
        };
        let session: *mut Session<'static, RosConfig> = Box::into_raw(Box::new(session));
        // SAFETY: the session lives until reclaimed below.
        let serving = serve(
            &dev,
            t0,
            unsafe { &*session },
            &zid,
            router,
            cmd,
            arm,
            lv_chan,
            echo,
        );
        // Test hook: abandon a healthy session after N seconds, as a silent link loss would
        // (over UDP the router keeps the old session until its lease runs out).
        let result = match std::env::var("ROVI_SIM_ABANDON_AFTER_S")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            Some(n) => {
                match embassy_futures::select::select(serving, embassy_time::Timer::after_secs(n))
                    .await
                {
                    embassy_futures::select::Either::First(r) => r,
                    embassy_futures::select::Either::Second(()) => Ok(()),
                }
            }
            None => serving.await,
        };
        println!("ros session ended ({:?}), reconnecting", result.err());
        log(Event::Ros(Link::Down), t0);
        // SAFETY: `serve` returned, so every borrow of the session (tokens, subscribers,
        // publisher) is gone. The session points into the resources, so it goes first.
        drop(unsafe { Box::from_raw(session) });
        drop(unsafe { Box::from_raw(resources) });
        // Samples queued by the old session are stale.
        while cmd.try_receive().is_ok() {}
        while arm.try_receive().is_ok() {}
        while lv_chan.try_receive().is_ok() {}
        while echo.try_receive().is_ok() {}
        embassy_time::Timer::after_secs(2).await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn serve(
    dev: &Shared,
    t0: Instant,
    session: &'static Session<'static, RosConfig>,
    zid: &str,
    router: &str,
    cmd: &'static Chan,
    arm: &'static Chan,
    lv_chan: &'static TokenChan,
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

    // Liveliness: the node, then each endpoint, so the car shows up in the ROS graph.
    let node = rmw::node_token(DOMAIN, zid, NODE).unwrap();
    let _node = session
        .declare_token(zenoh::keyexpr::new(node.as_str())?)
        .await?;
    let tokens = [
        (1, Kind::Subscription, &cmd_vel, rmw::QOS_BEST_EFFORT_1),
        (2, Kind::Subscription, &arm_t, rmw::QOS_BEST_EFFORT_5),
        (3, Kind::Publisher, &status_t, rmw::QOS_DEFAULT_10),
        (4, Kind::Subscription, &echo_t, rmw::QOS_BEST_EFFORT_1),
        (5, Kind::Publisher, &echo_reply_t, rmw::QOS_BEST_EFFORT_1),
    ]
    .map(|(id, kind, topic, qos)| {
        rmw::endpoint_token(DOMAIN, zid, id, kind, NODE, topic, qos).unwrap()
    });
    let mut _held = Vec::new();
    for t in &tokens {
        _held.push(
            session
                .declare_token(zenoh::keyexpr::new(t.as_str())?)
                .await?,
        );
    }
    let status_gid = rmw::gid(tokens[2].as_str());
    let echo_gid = rmw::gid(tokens[4].as_str());

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
    let lv_ke = zenoh::keyexpr::new(leak(format!("@ros2_lv/{DOMAIN}/*/*/*/MP/**")))?;
    let lv_sub = session
        .declare_liveliness_subscriber(lv_ke)
        .channel(lv_chan.dyn_sender(), lv_chan.dyn_receiver())
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
    println!("ros node /{NODE} up, zid {zid}, router {router}");
    log(Event::Ros(Link::Up), t0);

    let (d1, d2, d3, d4) = (dev.clone(), dev.clone(), dev.clone(), dev.clone());
    let last_refresh = Cell::new(None::<u64>);
    // Test hook: drop token events for the first N seconds after the link comes up, as lost UDP
    // datagrams would, to check that the car asks for the tokens again.
    let ignore_tokens_until = std::env::var("ROVI_SIM_IGNORE_TOKENS_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|n| ms(t0) + n * 1000);
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
    let ended = select(
        select5(
            session.run(),
            async {
                while let Some(s) = cmd_sub.recv().await {
                    maybe_refresh(
                        session,
                        lv_ke,
                        &d1,
                        s.attachment().unwrap_or(&[]),
                        t0,
                        &last_refresh,
                    )
                    .await;
                    let fx = d1.lock().unwrap().on_ros_cmd_vel(
                        ms(t0),
                        s.attachment().unwrap_or(&[]),
                        s.payload(),
                    );
                    log_effects(fx, t0);
                }
            },
            async {
                while let Some(s) = arm_sub.recv().await {
                    maybe_refresh(
                        session,
                        lv_ke,
                        &d2,
                        s.attachment().unwrap_or(&[]),
                        t0,
                        &last_refresh,
                    )
                    .await;
                    let fx = d2.lock().unwrap().on_ros_arm(
                        ms(t0),
                        s.attachment().unwrap_or(&[]),
                        s.payload(),
                    );
                    log_effects(fx, t0);
                }
            },
            async {
                while let Some(s) = lv_sub.recv().await {
                    if ignore_tokens_until.is_some_and(|t| ms(t0) < t) {
                        continue;
                    }
                    let alive = s.payload() == [1];
                    d4.lock()
                        .unwrap()
                        .on_ros_liveliness(s.keyexpr().as_str(), alive);
                }
            },
            async {
                let mut seq = 0i64;
                loop {
                    let mut buf = [0u8; 80];
                    let n = d3.lock().unwrap().status_cdr(&mut buf);
                    seq += 1;
                    let att = Attachment {
                        seq,
                        timestamp: 0,
                        gid: status_gid,
                    }
                    .encode();
                    let _ = status_pub.put(&buf[..n]).attachment(&att).finish().await;
                    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                }
            },
        ),
        echo_loop,
    )
    .await;
    // Only the session loop ends on its own; say why it did.
    if let Either::First(Either5::First(Err(e))) = ended {
        return Err(e.into());
    }
    Ok(())
}

/// A sample from a publisher no token announced means a token was lost (UDP does not resend):
/// ask the router for the current tokens again, at most every 2 s.
async fn maybe_refresh(
    session: &Session<'static, RosConfig>,
    lv_ke: &zenoh::keyexpr,
    dev: &Shared,
    attachment: &[u8],
    t0: Instant,
    last: &Cell<Option<u64>>,
) {
    let now = ms(t0);
    if std::env::var_os("ROVI_SIM_NO_REFRESH").is_some() // test hook: the behaviour before this fix
        || dev.lock().unwrap().ros_publisher_known(attachment) != Some(false)
        || last.get().is_some_and(|t| now < t + 2000)
    {
        return;
    }
    last.set(Some(now));
    log(Event::Ros(Link::Refresh), t0);
    let _ = session.liveliness_refresh(lv_ke).await;
}
