//! The ROS 2 sim adapter: node `/rovi` over zenoh-nostd and `rmw_zenohd`, feeding the shared
//! device. Subscribes `/cmd_vel` (TwistStamped) and `/rovi/arm` (Bool), publishes `/rovi/status`.

use crate::{log_effects, ms, Shared};
use embassy_futures::select::select5;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use rovi_m8_setpoint_ros::rmw::{self, Attachment, Endpoint as Kind, Topic};
use static_cell::StaticCell;
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
    let cmd: &'static Chan = CMD.init(Channel::new());
    let arm: &'static Chan = ARM.init(Channel::new());
    let lv_chan: &'static TokenChan = TOKENS.init(Channel::new());

    let router =
        leak(std::env::var("ROVI_ZENOH_ROUTER").unwrap_or_else(|_| "tcp/127.0.0.1:7447".into()));
    static CONFIG: StaticCell<RosConfig> = StaticCell::new();
    static RESOURCES: StaticCell<Resources<'static, RosConfig>> = StaticCell::new();
    static SESSION: StaticCell<Session<'static, RosConfig>> = StaticCell::new();
    let config: &'static RosConfig = CONFIG.init(RosConfig {
        transports: TransportLinkManager::from(LinkManager),
    });
    let zid = format!("{:?}", config.transports().zid());
    // ROS mixes reliable and best-effort traffic, which zenoh sequences separately, but
    // zenoh-nostd tracks one sequence number and would drop frames. So skip the check.
    let session: &'static Session<'static, RosConfig> = SESSION.init(
        zenoh::connect_ignore_invalid_sn(
            RESOURCES.init(Resources::default()),
            config,
            Endpoint::try_from(router)?,
        )
        .await?,
    );

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

    // Liveliness: the node, then each endpoint, so the car shows up in the ROS graph.
    let node = rmw::node_token(DOMAIN, &zid, NODE).unwrap();
    let _node = session
        .declare_token(zenoh::keyexpr::new(node.as_str())?)
        .await?;
    let tokens = [
        (1, Kind::Subscription, &cmd_vel, rmw::QOS_BEST_EFFORT_1),
        (2, Kind::Subscription, &arm_t, rmw::QOS_BEST_EFFORT_5),
        (3, Kind::Publisher, &status_t, rmw::QOS_DEFAULT_10),
    ]
    .map(|(id, kind, topic, qos)| {
        rmw::endpoint_token(DOMAIN, &zid, id, kind, NODE, topic, qos).unwrap()
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
        .declare_liveliness_subscriber(zenoh::keyexpr::new(leak(format!("@ros2_lv/{DOMAIN}/**")))?)
        .channel(lv_chan.dyn_sender(), lv_chan.dyn_receiver())
        .finish()
        .await?;
    let status_pub = session
        .declare_publisher(zenoh::keyexpr::new(key(&status_t))?)
        .finish()
        .await?;
    println!("ros node /{NODE} up, zid {zid}, router {router}");

    let (d1, d2, d3, d4) = (dev.clone(), dev.clone(), dev.clone(), dev);
    select5(
        session.run(),
        async {
            while let Some(s) = cmd_sub.recv().await {
                let fx = d1.lock().unwrap().on_ros_cmd_vel(
                    ms(t0),
                    s.attachment().unwrap_or(&[]),
                    s.payload(),
                );
                log_effects(fx);
            }
        },
        async {
            while let Some(s) = arm_sub.recv().await {
                let fx = d2.lock().unwrap().on_ros_arm(
                    ms(t0),
                    s.attachment().unwrap_or(&[]),
                    s.payload(),
                );
                log_effects(fx);
            }
        },
        async {
            while let Some(s) = lv_sub.recv().await {
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
    )
    .await;
    Ok(())
}
