//! Sim car: the device core on a laptop UDP socket (and, with `--features ros`, a ROS 2 node
//! over zenoh), for development against real clients. Development evidence only, never hardware
//! acceptance (ADR-0002).
//!
//! `simcar [port] [lease_ms claim_window_ms reclaim_window_ms]`. Logs one `ev=` line per event (`events.rs`).
//! Virtual BLE listens on `port + 1` (`vble.rs`); `ROVI_SIM_HOME=1` scripts home mode (`net.rs`).
//! With `ros`, the router endpoint is `ROVI_ZENOH_ROUTER` (default `tcp/127.0.0.1:7447`).

mod net;
#[cfg(feature = "ros")]
mod ros;
mod vble;

use rovi_core::arbiter::{Config, Effects};
use rovi_core::device::Device;
use rovi_core::events::{self, Event, Record};
use rovi_core::kinematics::Geometry;
use rovi_protocol::State;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type Shared = Arc<Mutex<Device>>;

/// Can Wi-Fi input reach the car? Not while BLE owns it (radio off) or while a home-mode attempt
/// is still joining (no network yet); the sim's sockets stay open, so this gate stands in.
pub fn wifi_up(d: &Device) -> bool {
    d.wifi_allowed() && !matches!(d.network().mode(), rovi_core::network::Mode::Attempting(_))
}

pub fn ms(t0: Instant) -> u64 {
    t0.elapsed().as_millis() as u64
}

/// One `ev=` line on stdout (spec: docs/LOGGING.md).
pub fn log(ev: Event, t0: Instant) {
    println!(
        "{}",
        Record {
            at_ms: ms(t0) as u32,
            ev
        }
    );
}

pub fn log_effects(fx: Effects, t0: Instant) {
    events::effects(fx, |e| log(e, t0));
}

fn peer_id(a: SocketAddr) -> u128 {
    match a {
        SocketAddr::V4(a) => (u32::from(*a.ip()) as u128) << 16 | a.port() as u128,
        SocketAddr::V6(a) => (u128::from(*a.ip()) << 16) | a.port() as u128,
    }
}

fn udp_loop(dev: Shared, t0: Instant, port: u16) -> std::io::Result<()> {
    let sock = UdpSocket::bind(("0.0.0.0", port))?;
    sock.set_read_timeout(Some(Duration::from_millis(10)))?;
    println!("simcar listening on udp/{port}");
    let (mut buf, mut out) = ([0u8; 1500], [0u8; 128]);
    let mut owner_addr = None;
    let mut peers = std::collections::HashMap::new();
    loop {
        match sock.recv_from(&mut buf) {
            // Wi-Fi is off while BLE owns the car: nothing arrives.
            Ok(_) if !wifi_up(&dev.lock().unwrap()) => {}
            Ok((n, from)) => {
                peers.insert(peer_id(from), from);
                let r = {
                    let mut d = dev.lock().unwrap();
                    let r = d.on_udp(ms(t0), peer_id(from), &buf[..n], &mut out);
                    if let Some((source, cmd, error)) = d.take_mode_log() {
                        log(Event::ModeCmd { source, cmd, error }, t0);
                    }
                    if d.arbiter().owner().is_some_and(|o| {
                        o.source == rovi_core::arbiter::Source::Udp && o.id == peer_id(from)
                    }) {
                        owner_addr = Some(from);
                    }
                    r
                };
                if r.len > 0 {
                    sock.send_to(&out[..r.len], from)?;
                }
                log_effects(r.effects, t0);
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e),
        }
        // A taken-over UDP owner hears it was stopped (spec: Takeover notice).
        let notice = dev
            .lock()
            .unwrap()
            .take_takeover_notice(rovi_core::arbiter::Source::Udp);
        if let Some(n) = notice {
            let len = n.udp_frame(&mut out);
            if let Some(to) = peers.get(&n.id) {
                sock.send_to(&out[..len], to)?;
            }
        }
        let pending = dev.lock().unwrap().udp_status_notification(&mut out);
        if let Some((peer, n)) = pending {
            if let Some(to) = owner_addr.filter(|a| peer_id(*a) == peer) {
                sock.send_to(&out[..n], to)?;
            }
        }
    }
}

fn control_loop(dev: Shared, t0: Instant, mut net: net::Net) {
    #[cfg(feature = "motor-bench")]
    let mut drive = {
        let motor = rovi_motor_control::MotorControl::new(
            NoPins,
            rovi_motor_control::Config::PRODUCT,
            rovi_motor_control::CAR_CHANNELS,
        )
        .expect("valid bench motor profile");
        rovi_drive::Drive::new(motor, &mut dev.lock().unwrap())
    };
    let mut last = (State::Open, [0.0f32; 4]);
    loop {
        std::thread::sleep(Duration::from_millis(10));
        net.tick(&dev, t0);
        let (fx, state, wheels) = {
            let mut d = dev.lock().unwrap();
            #[cfg(not(feature = "motor-bench"))]
            let fx = d.tick(ms(t0));
            #[cfg(feature = "motor-bench")]
            let fx = {
                let tick = drive.tick_motor_bench(&mut d, ms(t0));
                if let Some(fault) = tick.fault {
                    log(Event::Fault(fault), t0);
                }
                tick.effects
            };
            #[cfg(not(feature = "motor-bench"))]
            let wheels = d.wheels(ms(t0));
            #[cfg(feature = "motor-bench")]
            let wheels = rovi_drive::bench::wheels(&d, ms(t0));
            (fx, d.arbiter().state(), wheels)
        };
        log_effects(fx, t0);
        if (state, wheels) != last {
            log(
                Event::State {
                    state,
                    wheels_pm: events::permille(wheels),
                },
                t0,
            );
            last = (state, wheels);
        }
    }
}

/// Socket tests exercise the coordinator; no electrical simulation is claimed.
#[cfg(feature = "motor-bench")]
struct NoPins;
#[cfg(feature = "motor-bench")]
impl rovi_motor_control::Driver for NoPins {
    type Error = core::convert::Infallible;
    fn set_stby(&mut self, _: usize, _: bool) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_bridge(&mut self, _: usize, _: rovi_motor_control::Pins) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Session IDs start somewhere new each run, so a client's packets from an earlier run of the
/// sim car do not match the new session.
fn session_seed() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos() ^ d.as_secs() as u32)
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
    #[cfg(not(feature = "motor-bench"))]
    let geometry = Geometry {
        wheels_radius: 0.05,
        center_projection_sum: 0.2,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_theta: 0.0,
        max_wheel_speed: 40.0,
    };
    #[cfg(feature = "motor-bench")]
    let geometry = Geometry::CAR;
    let dev: Shared = Arc::new(Mutex::new(
        Device::new(cfg, geometry, session_seed()).expect("the sim profile is valid"),
    ));
    let t0 = Instant::now();
    log(Event::Boot { reset: None }, t0);
    #[cfg(feature = "motor-bench")]
    println!("{}", rovi_drive::bench::BANNER);

    let net = net::start(&dev, t0);
    let (d, u, b) = (dev.clone(), dev.clone(), dev.clone());
    std::thread::spawn(move || control_loop(d, t0, net));
    std::thread::spawn(move || udp_loop(u, t0, port).expect("udp"));
    std::thread::spawn(move || vble::run(b, t0, port + 1).expect("virtual ble"));

    #[cfg(feature = "ros")]
    ros::run(dev, t0);
    #[cfg(not(feature = "ros"))]
    std::thread::park();
    Ok(())
}
