//! Sim car: the device core on a laptop UDP socket, for development against real clients.
//! Development evidence only, never hardware acceptance (ADR-0002).
//!
//! `simcar [port] [lease_ms claim_window_ms reclaim_window_ms]`. Logs one line per event.

use rovi_m8_setpoint_ros::arbiter::Config;
use rovi_m8_setpoint_ros::device::Device;
use rovi_m8_setpoint_ros::kinematics::Geometry;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

fn peer_id(a: SocketAddr) -> u128 {
    match a {
        SocketAddr::V4(a) => (u32::from(*a.ip()) as u128) << 16 | a.port() as u128,
        SocketAddr::V6(a) => (u128::from(*a.ip()) << 16) | a.port() as u128,
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let num = |i: usize, default: u64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(default);
    let port = num(0, 7777);
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
    let mut dev = Device::new(cfg, geometry);
    let sock = UdpSocket::bind(("0.0.0.0", port as u16))?;
    sock.set_read_timeout(Some(Duration::from_millis(10)))?;
    println!("simcar listening on udp/{port}");
    let t0 = Instant::now();
    let (mut buf, mut out) = ([0u8; 1500], [0u8; 128]);
    let mut last_wheels = [0.0f32; 4];
    loop {
        let now = t0.elapsed().as_millis() as u64;
        let mut fx = dev.tick(now);
        if let Ok((n, from)) = sock.recv_from(&mut buf) {
            let now = t0.elapsed().as_millis() as u64;
            let r = dev.on_udp(now, peer_id(from), &buf[..n], &mut out);
            if r.len > 0 {
                sock.send_to(&out[..r.len], from)?;
            }
            fx.claimed = fx.claimed.or(r.effects.claimed);
            fx.released |= r.effects.released;
            fx.stop = fx.stop.or(r.effects.stop);
        }
        if let Some(s) = fx.claimed {
            println!("claimed {s:?}");
        }
        if let Some(r) = fx.stop {
            println!("stop {r:?}");
        }
        if fx.released {
            println!("released");
        }
        let wheels = dev.wheels(t0.elapsed().as_millis() as u64);
        if wheels != last_wheels {
            println!("wheels {wheels:?}");
            last_wheels = wheels;
        }
    }
}
