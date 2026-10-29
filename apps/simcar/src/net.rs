//! Simulated home-mode networking for scenarios: the sim car has no Wi-Fi, so the steps of each
//! home-mode attempt are scripted. Development evidence only (ADR-0002).
//!
//! With `ROVI_SIM_HOME=1` the device starts as if built with home settings, and every step of
//! every attempt succeeds unless stdin scripted otherwise, one command per line:
//!
//! * `fail scan|join|dhcp|router` - the next attempt fails at that step
//! * `lost` - the station loses the home network
//!
//! In home mode the station address is 127.0.0.1.

use crate::{log, ms, Shared};
use rovi_core::events::Event;
use rovi_core::network::{Mode, Step};
use rovi_protocol::Reason;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Default)]
struct Script {
    fail_at: Option<Step>,
    lost: bool,
}

pub struct Net {
    script: Arc<Mutex<Script>>,
    last: Option<(Mode, Reason)>,
}

pub fn start(dev: &Shared, t0: Instant) -> Net {
    let home = std::env::var("ROVI_SIM_HOME").is_ok_and(|v| v == "1");
    dev.lock().unwrap().start_network(ms(t0), home);
    let script = Arc::new(Mutex::new(Script::default()));
    let input = script.clone();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            let mut s = input.lock().unwrap();
            match line.split_whitespace().collect::<Vec<_>>()[..] {
                ["fail", step] => s.fail_at = parse(step),
                ["lost"] => s.lost = true,
                _ => eprintln!("simcar: unknown network command {line:?}"),
            }
        }
    });
    Net { script, last: None }
}

fn parse(step: &str) -> Option<Step> {
    Some(match step {
        "scan" => Step::Scan,
        "join" => Step::Join,
        "dhcp" => Step::Dhcp,
        "router" => Step::Router,
        _ => return None,
    })
}

impl Net {
    /// One control tick: run the current step, apply a scripted loss, log mode changes.
    pub fn tick(&mut self, dev: &Shared, t0: Instant) {
        let now = ms(t0);
        let mut d = dev.lock().unwrap();
        let mut s = self.script.lock().unwrap();
        if std::mem::take(&mut s.lost) {
            d.on_station_lost(now);
        }
        if let Mode::Attempting(step) = d.network().mode() {
            let ok = s.fail_at != Some(step);
            if !ok {
                s.fail_at = None;
            }
            d.on_network_step(now, step, ok);
            log(
                Event::NetStep {
                    step,
                    ok,
                    took_ms: 0,
                },
                t0,
            );
        }
        let now_mode = (d.network().mode(), d.network().reason());
        if self.last != Some(now_mode) {
            d.set_station_ip((now_mode.0 == Mode::Home).then_some([127, 0, 0, 1]));
            log(
                Event::Net {
                    mode: now_mode.0,
                    reason: now_mode.1,
                },
                t0,
            );
            self.last = Some(now_mode);
        }
    }
}
