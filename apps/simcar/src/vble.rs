//! Virtual BLE for scenarios: the sim car has no Bluetooth, so one UDP socket stands in for the
//! GATT service. Each client address is one central. It drives the same `Device` BLE calls the
//! firmware does, so BLE rules are tested in the sim. Development evidence only (ADR-0002): no
//! radio, MTU, connection interval or coexistence behaviour is modelled.
//!
//! The datagram format is `rovi_protocol::ble::sim` (`Request`/`Reply`); the car sends
//! `Reply::Dropped` when it drops a central, as the firmware disconnects a BLE owner that lost
//! the car.

use crate::{log, log_effects, ms, Shared};
use rovi_core::arbiter::Source;
use rovi_core::device::{BleConn, BLE_SLOTS};
use rovi_core::events::Event;
use rovi_protocol::ble::sim::{Characteristic, Reply, Request, ATT_VALUE_NOT_ALLOWED};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

struct Central {
    conn: BleConn,
    /// It has owned the car: losing ownership disconnects it.
    owned: bool,
}

pub fn run(dev: Shared, t0: Instant, port: u16) -> std::io::Result<()> {
    let sock = UdpSocket::bind(("0.0.0.0", port))?;
    sock.set_read_timeout(Some(Duration::from_millis(10)))?;
    println!("simcar virtual ble on udp/{port}");
    let mut centrals: HashMap<SocketAddr, Central> = HashMap::new();
    let mut handle = 0u16;
    let mut buf = [0u8; 512];
    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, from)) => {
                let reply = request(&dev, t0, &mut centrals, &mut handle, from, &buf[..n]);
                if let Some(r) = reply {
                    sock.send_to(&r, from)?;
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e),
        }
        // A BLE owner that lost the car is disconnected, as on the board.
        let dropped: Vec<SocketAddr> = {
            let d = dev.lock().unwrap();
            centrals
                .iter()
                .filter(|(_, c)| c.owned && !d.ble_owns(c.conn))
                .map(|(a, _)| *a)
                .collect()
        };
        for addr in dropped {
            if let Some(c) = centrals.remove(&addr) {
                let fx = dev.lock().unwrap().on_ble_disconnect(ms(t0), c.conn);
                log(
                    Event::Ble {
                        id: c.conn.handle,
                        mtu: None,
                    },
                    t0,
                );
                log_effects(fx, t0);
                sock.send_to(&encode(Reply::Dropped).unwrap_or_default(), addr)?;
            }
        }
    }
}

fn request(
    dev: &Shared,
    t0: Instant,
    centrals: &mut HashMap<SocketAddr, Central>,
    handle: &mut u16,
    from: SocketAddr,
    data: &[u8],
) -> Option<Vec<u8>> {
    let now = ms(t0);
    let request = Request::decode(data).ok()?;
    if request == Request::Connect {
        let mut d = dev.lock().unwrap();
        let ok = !centrals.contains_key(&from)
            && centrals.len() < BLE_SLOTS
            && d.radio_allowed(Source::Ble);
        if ok {
            *handle = handle.wrapping_add(1);
            let conn = d.on_ble_attach(now, *handle);
            centrals.insert(from, Central { conn, owned: false });
            log(
                Event::Ble {
                    id: *handle,
                    mtu: Some(247),
                },
                t0,
            );
        }
        return encode(Reply::Connected(ok));
    }
    let conn = centrals.get(&from)?.conn;
    let mut d = dev.lock().unwrap();
    let reply = match request {
        Request::Connect => unreachable!("handled above"),
        Request::Setpoint(value) => {
            let fx = d.on_ble_setpoint(now, conn, value);
            log_effects(fx, t0);
            None
        }
        Request::Control(code) => {
            let result = d.on_ble_control(now, conn, code);
            if let Some((source, cmd, error)) = d.take_mode_log() {
                log(Event::ModeCmd { source, cmd, error }, t0);
            }
            let att = match result {
                Ok(fx) => {
                    log_effects(fx, t0);
                    0
                }
                Err(_) => ATT_VALUE_NOT_ALLOWED,
            };
            encode(Reply::Control(att))
        }
        Request::Read(c) => {
            let value: Vec<u8> = match c {
                Characteristic::Status => d.ble_status().to_vec(),
                Characteristic::Capabilities => d.capabilities().to_vec(),
                Characteristic::Network => d.network_info().to_vec(),
            };
            encode(Reply::Read(c, &value))
        }
        Request::Echo(body) => rovi_core::device::echo(body).and_then(|e| encode(Reply::Echo(e))),
        Request::Disconnect => {
            let fx = d.on_ble_disconnect(now, conn);
            log(
                Event::Ble {
                    id: conn.handle,
                    mtu: None,
                },
                t0,
            );
            log_effects(fx, t0);
            drop(d);
            centrals.remove(&from);
            return encode(Reply::Disconnected);
        }
    };
    if d.ble_owns(conn) {
        if let Some(c) = centrals.get_mut(&from) {
            c.owned = true;
        }
    }
    reply
}

fn encode(reply: Reply) -> Option<Vec<u8>> {
    let mut b = [0u8; 64];
    reply.encode(&mut b).ok().map(|n| b[..n].to_vec())
}
