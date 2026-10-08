//! Sim car: the device core on a laptop UDP socket (and, with `--features ros`, a ROS 2 node
//! over zenoh), for development against real clients. Development evidence only, never hardware
//! acceptance (ADR-0002).
//!
//! `simcar [port] [lease_ms claim_window_ms reclaim_window_ms]`. Logs one line per event.
//! With `ros`, the router endpoint is `ROVI_ZENOH_ROUTER` (default `tcp/127.0.0.1:7447`).

#[cfg(feature = "ros")]
mod ros;

use rovi_m8_setpoint_ros::arbiter::{Config, Effects};
use rovi_m8_setpoint_ros::device::Device;
use rovi_m8_setpoint_ros::kinematics::Geometry;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type Shared = Arc<Mutex<Device>>;

pub fn ms(t0: Instant) -> u64 {
    t0.elapsed().as_millis() as u64
}

pub fn log_effects(fx: Effects) {
    if let Some(s) = fx.claimed {
        println!("claimed {s:?}");
    }
    if let Some(r) = fx.stop {
        println!("stop {r:?}");
    }
    if fx.released {
        println!("released");
    }
}

fn peer_id(a: SocketAddr) -> u128 {
    match a {
        SocketAddr::V4(a) => (u32::from(*a.ip()) as u128) << 16 | a.port() as u128,
        SocketAddr::V6(a) => (u128::from(*a.ip()) << 16) | a.port() as u128,
    }
}

fn udp_loop(dev: Shared, t0: Instant, port: u16) -> std::io::Result<()> {
    let sock = UdpSocket::bind(("0.0.0.0", port))?;
    println!("simcar listening on udp/{port}");
    let (mut buf, mut out) = ([0u8; 1500], [0u8; 128]);
    loop {
        let (n, from) = sock.recv_from(&mut buf)?;
        let r = dev
            .lock()
            .unwrap()
            .on_udp(ms(t0), peer_id(from), &buf[..n], &mut out);
        if r.len > 0 {
            sock.send_to(&out[..r.len], from)?;
        }
        log_effects(r.effects);
    }
}

fn control_loop(dev: Shared, t0: Instant) {
    let mut last = [0.0f32; 4];
    loop {
        std::thread::sleep(Duration::from_millis(10));
        let (fx, wheels) = {
            let mut d = dev.lock().unwrap();
            let fx = d.tick(ms(t0));
            (fx, d.wheels(ms(t0)))
        };
        log_effects(fx);
        if wheels != last {
            println!("wheels {wheels:?}");
            last = wheels;
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let num = |i: usize, default: u64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(default);
    let port = num(0, 7777) as u16;
    let cfg = Config {
        lease_ms: num(1, Config::DEFAULT.lease_ms),
        claim_window_ms: num(2, Config::DEFAULT.claim_window_ms),
        reclaim_window_ms: num(3, Config::DEFAULT.reclaim_window_ms),
        ..Config::DEFAULT
    };
    let geometry = Geometry {
        wheels_radius: 0.05,
        center_projection_sum: 0.2,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_theta: 0.0,
        max_wheel_speed: 40.0,
    };
    let dev: Shared = Arc::new(Mutex::new(Device::new(cfg, geometry)));
    let t0 = Instant::now();

    let (d, u) = (dev.clone(), dev.clone());
    std::thread::spawn(move || control_loop(d, t0));
    std::thread::spawn(move || udp_loop(u, t0, port).expect("udp"));

    #[cfg(feature = "ros")]
    ros::run(dev, t0);
    #[cfg(not(feature = "ros"))]
    std::thread::park();
    Ok(())
}
