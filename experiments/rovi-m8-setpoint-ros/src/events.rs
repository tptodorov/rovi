//! The car's log: one line per event, `ev=<name> [key=value ...] at_ms=<n>` (spec: docs/LOGGING.md).
//!
//! Emitters build a small `Record` (a copy, no formatting, no I/O); a sink formats it with `Display`
//! later and off the hot path. Integers only: no float formatting on the board. To add a property,
//! add a field to the `Event` variant and one `write!` below; parsers ignore keys they don't know.

use crate::arbiter::{Effects, Source, State, StopReason};
use core::fmt::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Up,
    Down,
    Retry,
}

/// Faults: events that should never happen on a healthy run (`ev=error`, flagged by `sim/carlog.py`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    WifiAp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Boot,
    Claimed(Source),
    Released,
    Stop(StopReason),
    /// Wheel commands (FL, FR, RR, RL) in permille of full scale.
    State {
        state: State,
        wheels_pm: [i16; 4],
    },
    /// `mtu` is set when the link comes up.
    Ble {
        id: u16,
        mtu: Option<u16>,
    },
    Wifi {
        ap_on: bool,
    },
    Ros(Link),
    Fault(Fault),
    /// Events lost because the log ring was full.
    Dropped(u32),
}

/// An event and when it happened: device uptime in ms, wrapping every 49.7 days.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub at_ms: u32,
    pub ev: Event,
}

/// Calls `emit` once per event in `fx`.
pub fn effects(fx: Effects, mut emit: impl FnMut(Event)) {
    if let Some(s) = fx.claimed {
        emit(Event::Claimed(s));
    }
    if let Some(r) = fx.stop {
        emit(Event::Stop(r));
    }
    if fx.released {
        emit(Event::Released);
    }
}

/// Wheel commands -1.0..=1.0 as permille (saturating).
pub fn permille(w: [f32; 4]) -> [i16; 4] {
    w.map(|v| libm::roundf(v * 1000.0) as i16)
}

fn source(s: Source) -> &'static str {
    match s {
        Source::Ble => "Ble",
        Source::Udp => "Udp",
        Source::Ros => "Ros",
    }
}

fn reason(r: StopReason) -> &'static str {
    match r {
        StopReason::Stop => "Stop",
        StopReason::LeaseExpired => "LeaseExpired",
        StopReason::OwnerLost => "OwnerLost",
        StopReason::OutOfLimits => "OutOfLimits",
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ev {
            Event::Boot => write!(f, "ev=boot ver={}", env!("CARGO_PKG_VERSION"))?,
            Event::Claimed(s) => write!(f, "ev=claimed source={}", source(s))?,
            Event::Released => f.write_str("ev=released")?,
            Event::Stop(r) => write!(f, "ev=stop reason={}", reason(r))?,
            Event::State {
                state,
                wheels_pm: w,
            } => {
                let (name, why) = match state {
                    State::Open => ("Open", None),
                    State::Claimed => ("Claimed", None),
                    State::Armed => ("Armed", None),
                    State::Stopped(r) => ("Stopped", Some(r)),
                };
                write!(f, "ev=state state={name}")?;
                if let Some(r) = why {
                    write!(f, " reason={}", reason(r))?;
                }
                write!(f, " wheels_pm={},{},{},{}", w[0], w[1], w[2], w[3])?
            }
            Event::Ble { id, mtu: Some(m) } => write!(f, "ev=ble link=up id={id} mtu={m}")?,
            Event::Ble { id, mtu: None } => write!(f, "ev=ble link=down id={id}")?,
            Event::Wifi { ap_on } => write!(f, "ev=wifi ap={}", if ap_on { "on" } else { "off" })?,
            Event::Ros(l) => write!(
                f,
                "ev=ros link={}",
                match l {
                    Link::Up => "up",
                    Link::Down => "down",
                    Link::Retry => "retry",
                }
            )?,
            Event::Fault(Fault::WifiAp) => f.write_str("ev=error src=wifi_ap")?,
            Event::Dropped(n) => write!(f, "ev=dropped n={n}")?,
        }
        write!(f, " at_ms={}", self.at_ms)
    }
}

/// Room for the longest line (see the test) plus its newline.
pub const MAX_LINE: usize = 128;

/// A fixed buffer for formatting one line off the heap. A line that does not fit is cut, and so
/// loses its trailing `at_ms`, which is how parsers recognise a line as incomplete.
pub struct LineBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> LineBuf<N> {
    pub const fn new() -> Self {
        LineBuf {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> Default for LineBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for LineBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let n = s.len().min(N - self.len);
        self.buf[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        if n < s.len() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::{String, ToString};
    use std::vec::Vec;

    fn line(ev: Event) -> String {
        Record { at_ms: 42, ev }.to_string()
    }

    #[test]
    fn every_event_has_one_stable_line() {
        let state = |state, w| Event::State {
            state,
            wheels_pm: w,
        };
        let cases = [
            (Event::Boot, "ev=boot ver=0.1.0 at_ms=42"),
            (
                Event::Claimed(Source::Udp),
                "ev=claimed source=Udp at_ms=42",
            ),
            (Event::Released, "ev=released at_ms=42"),
            (
                Event::Stop(StopReason::LeaseExpired),
                "ev=stop reason=LeaseExpired at_ms=42",
            ),
            (
                state(State::Armed, [250, -250, 0, 1000]),
                "ev=state state=Armed wheels_pm=250,-250,0,1000 at_ms=42",
            ),
            (
                state(State::Stopped(StopReason::Stop), [0; 4]),
                "ev=state state=Stopped reason=Stop wheels_pm=0,0,0,0 at_ms=42",
            ),
            (
                Event::Ble {
                    id: 1,
                    mtu: Some(75),
                },
                "ev=ble link=up id=1 mtu=75 at_ms=42",
            ),
            (
                Event::Ble { id: 1, mtu: None },
                "ev=ble link=down id=1 at_ms=42",
            ),
            (Event::Wifi { ap_on: false }, "ev=wifi ap=off at_ms=42"),
            (Event::Ros(Link::Retry), "ev=ros link=retry at_ms=42"),
            (Event::Fault(Fault::WifiAp), "ev=error src=wifi_ap at_ms=42"),
            (Event::Dropped(3), "ev=dropped n=3 at_ms=42"),
        ];
        for (ev, want) in cases {
            assert_eq!(line(ev), want);
            assert!(
                want.split(' ').all(|kv| kv.contains('=')),
                "no spaces inside values: {want}"
            );
        }
    }

    #[test]
    fn the_longest_line_fits_the_line_buffer() {
        let ev = Event::State {
            state: State::Stopped(StopReason::LeaseExpired),
            wheels_pm: [i16::MIN; 4],
        };
        let longest = Record {
            at_ms: u32::MAX,
            ev,
        }
        .to_string();
        assert!(
            longest.len() < MAX_LINE,
            "{} bytes: {longest}",
            longest.len()
        );
    }

    #[test]
    fn effects_come_out_in_order() {
        let fx = Effects {
            claimed: Some(Source::Ble),
            released: true,
            stop: Some(StopReason::OwnerLost),
        };
        let mut evs = Vec::new();
        effects(fx, |e| evs.push(e));
        assert_eq!(
            evs,
            [
                Event::Claimed(Source::Ble),
                Event::Stop(StopReason::OwnerLost),
                Event::Released
            ]
        );
    }

    #[test]
    fn permille_rounds_and_saturates() {
        assert_eq!(
            permille([0.25, -0.0004, 0.9996, 1000.0]),
            [250, 0, 1000, i16::MAX]
        );
    }

    #[test]
    fn a_line_that_does_not_fit_loses_its_at_ms() {
        let mut b = LineBuf::<20>::new();
        let r = write!(
            b,
            "{}",
            Record {
                at_ms: 7,
                ev: Event::Stop(StopReason::LeaseExpired)
            }
        );
        assert!(r.is_err());
        assert_eq!(b.as_bytes(), b"ev=stop reason=Lease");
        let mut b = LineBuf::<64>::new();
        writeln!(
            b,
            "{}",
            Record {
                at_ms: 7,
                ev: Event::Released
            }
        )
        .unwrap();
        assert_eq!(b.as_bytes(), b"ev=released at_ms=7\n");
    }
}
