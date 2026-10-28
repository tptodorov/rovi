//! The car's log: one line per event, `ev=<name> [key=value ...] at_ms=<n>` (spec: docs/LOGGING.md).
//!
//! Emitters build a small `Record` (a copy, no formatting, no I/O); a sink formats it with `Display`
//! later and off the hot path. Integers only: no float formatting on the board. To add a property,
//! add a field to the `Event` variant and one `write!` below; parsers ignore keys they don't know.

use crate::arbiter::{Effects, Error, Source, State, StopReason};
use crate::network::{Command, Mode, Reason, Step};
use core::fmt::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Up,
    Down,
    Retry,
    /// A sample came from a publisher no token announced, so the tokens were asked for again.
    Refresh,
}

/// Faults: events that should never happen on a healthy run (`ev=error`, flagged by `sim/carlog.py`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    WifiAp,
    /// Motor control got a NaN or infinite wheel command.
    MotorNonFinite,
    /// The motor configuration is invalid.
    MotorConfig,
    /// A motor pin write failed.
    MotorDriver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// `reset`: the chip's reset cause (the firmware's `SocResetReason` name); none on the sim.
    Boot {
        reset: Option<&'static str>,
    },
    Claimed(Source),
    Released,
    Stop {
        reason: StopReason,
        /// Detection lateness captured by the arbiter, only on a lease stop.
        lease_late_ms: Option<u64>,
    },
    /// Wheel commands (FL, FR, RR, RL) in permille of full scale.
    State {
        state: State,
        wheels_pm: [i16; 4],
    },
    /// `mtu` is set when the link comes up: the ATT MTU at connect, before any exchange.
    Ble {
        id: u16,
        mtu: Option<u16>,
    },
    /// The ATT MTU after the central's exchange, once per connection.
    Mtu {
        id: u16,
        mtu: u16,
    },
    Wifi {
        ap_on: bool,
    },
    Ros(Link),
    /// The network mode changed (ADR-0016); `reason` is set for fallback.
    Net {
        mode: Mode,
        reason: Reason,
    },
    /// A home-mode step finished.
    NetStep {
        step: Step,
        ok: bool,
        took_ms: u32,
    },
    /// A mode command from an owner: `error` is `None` when accepted.
    ModeCmd {
        source: Source,
        cmd: Command,
        error: Option<Error>,
    },
    /// A higher-priority controller took the car from an owner of this source.
    Preempted(Source),
    /// The ROS router's host name was looked up (an IPv4 literal needs no lookup and logs nothing).
    Resolve {
        ok: bool,
        took_ms: u32,
        ip: [u8; 4],
    },
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
/// Order: stop, release, claim. One call never claims and stops; when a passed deadline
/// releases the car and the same input claims it again, the release happens first.
pub fn effects(fx: Effects, mut emit: impl FnMut(Event)) {
    if let Some(old) = fx.preempted {
        emit(Event::Preempted(old.source));
    }
    if let Some(r) = fx.stop {
        emit(Event::Stop {
            reason: r,
            lease_late_ms: fx.lease_late_ms,
        });
    }
    if fx.released {
        emit(Event::Released);
    }
    if let Some(s) = fx.claimed {
        emit(Event::Claimed(s));
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

fn step(s: Step) -> &'static str {
    match s {
        Step::Scan => "Scan",
        Step::Join => "Join",
        Step::Dhcp => "Dhcp",
        Step::Router => "Router",
    }
}

fn net_reason(r: Reason) -> &'static str {
    match r {
        Reason::None => "None",
        Reason::NoSettings => "NoSettings",
        Reason::Scan => "Scan",
        Reason::Join => "Join",
        Reason::Dhcp => "Dhcp",
        Reason::Router => "Router",
        Reason::LinkLost => "LinkLost",
        Reason::Requested => "Requested",
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
            Event::Boot { reset } => {
                write!(f, "ev=boot ver={}", env!("CARGO_PKG_VERSION"))?;
                if let Some(r) = reset {
                    write!(f, " reset={r}")?;
                }
            }
            Event::Claimed(s) => write!(f, "ev=claimed source={}", source(s))?,
            Event::Released => f.write_str("ev=released")?,
            Event::Stop {
                reason: r,
                lease_late_ms,
            } => {
                write!(f, "ev=stop reason={}", reason(r))?;
                if let Some(late) = lease_late_ms {
                    write!(f, " lease_late_ms={late}")?;
                }
            }
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
            Event::Mtu { id, mtu } => write!(f, "ev=mtu id={id} mtu={mtu}")?,
            Event::Wifi { ap_on } => write!(f, "ev=wifi ap={}", if ap_on { "on" } else { "off" })?,
            Event::Ros(l) => write!(
                f,
                "ev=ros link={}",
                match l {
                    Link::Up => "up",
                    Link::Down => "down",
                    Link::Retry => "retry",
                    Link::Refresh => "refresh",
                }
            )?,
            Event::Net { mode, reason } => match mode {
                Mode::Home => f.write_str("ev=net mode=Home")?,
                Mode::Attempting(s) => write!(f, "ev=net mode=Attempting step={}", step(s))?,
                Mode::Fallback => write!(f, "ev=net mode=Fallback reason={}", net_reason(reason))?,
            },
            Event::NetStep {
                step: s,
                ok,
                took_ms,
            } => write!(
                f,
                "ev=net_step step={} result={} took_ms={took_ms}",
                step(s),
                if ok { "Ok" } else { "Failed" }
            )?,
            Event::ModeCmd {
                source: s,
                cmd,
                error,
            } => write!(
                f,
                "ev=mode_cmd source={} cmd={} result={}",
                source(s),
                match cmd {
                    Command::Home => "Home",
                    Command::CarNetwork => "CarNetwork",
                },
                match error {
                    None => "Ok",
                    Some(Error::NotReady) => "NotReady",
                    Some(Error::Invalid) => "Invalid",
                    Some(Error::Busy) => "Busy",
                    Some(Error::NotOwner) => "NotOwner",
                }
            )?,
            Event::Preempted(s) => write!(f, "ev=preempted source={}", source(s))?,
            Event::Resolve { ok, took_ms, ip } => write!(
                f,
                "ev=resolve result={} took_ms={took_ms} ip={}.{}.{}.{}",
                if ok { "Ok" } else { "Failed" },
                ip[0],
                ip[1],
                ip[2],
                ip[3]
            )?,
            Event::Fault(Fault::WifiAp) => f.write_str("ev=error src=wifi_ap")?,
            Event::Fault(Fault::MotorNonFinite) => f.write_str("ev=error src=motor_nonfinite")?,
            Event::Fault(Fault::MotorConfig) => f.write_str("ev=error src=motor_config")?,
            Event::Fault(Fault::MotorDriver) => f.write_str("ev=error src=motor_driver")?,
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
    use crate::arbiter::Error;
    use crate::network::{Command, Mode, Reason, Step};
    use std::string::{String, ToString};
    use std::vec::Vec;

    fn line(ev: Event) -> String {
        Record { at_ms: 42, ev }.to_string()
    }

    #[test]
    fn lease_lateness_survives_effects_and_delayed_log_formatting() {
        let mut a = crate::arbiter::Arbiter::new(crate::arbiter::Config::DEFAULT);
        let who = crate::arbiter::Claimant {
            source: Source::Ble,
            id: 1,
        };
        a.claim(0, who).unwrap();
        a.setpoint(
            10,
            who,
            1,
            crate::setpoint::Setpoint {
                sec: 0,
                nanosec: 0,
                vx: 0.0,
                vy: 0.0,
                wz: 0.0,
            },
        )
        .unwrap();
        a.arm(11, who).unwrap();
        let fx = a.tick(1_061);
        let mut lines = Vec::new();
        effects(fx, |ev| lines.push(Record { at_ms: 9_999, ev }.to_string()));
        assert_eq!(
            lines,
            ["ev=stop reason=LeaseExpired lease_late_ms=51 at_ms=9999"]
        );
        let longest = Record {
            at_ms: u32::MAX,
            ev: Event::Stop {
                reason: StopReason::LeaseExpired,
                lease_late_ms: Some(u64::MAX),
            },
        }
        .to_string();
        assert!(longest.len() < MAX_LINE, "{longest}");
    }

    #[test]
    fn a_takeover_logs_the_preempted_owner_first() {
        let fx = Effects {
            claimed: Some(Source::Udp),
            released: true,
            stop: Some(StopReason::OwnerLost),
            preempted: Some(crate::arbiter::Claimant {
                source: Source::Ros,
                id: 7,
            }),
            ..Effects::default()
        };
        let mut lines = Vec::new();
        effects(fx, |ev| lines.push(line(ev)));
        assert_eq!(
            lines,
            [
                "ev=preempted source=Ros at_ms=42",
                "ev=stop reason=OwnerLost at_ms=42",
                "ev=released at_ms=42",
                "ev=claimed source=Udp at_ms=42",
            ]
        );
    }

    #[test]
    fn every_event_has_one_stable_line() {
        let state = |state, w| Event::State {
            state,
            wheels_pm: w,
        };
        let cases = [
            (Event::Boot { reset: None }, "ev=boot ver=0.1.0 at_ms=42"),
            (
                Event::Boot {
                    reset: Some("SysRtcWdt"),
                },
                "ev=boot ver=0.1.0 reset=SysRtcWdt at_ms=42",
            ),
            (
                Event::Claimed(Source::Udp),
                "ev=claimed source=Udp at_ms=42",
            ),
            (Event::Released, "ev=released at_ms=42"),
            (
                Event::Stop {
                    reason: StopReason::LeaseExpired,
                    lease_late_ms: None,
                },
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
            (
                Event::Mtu { id: 1, mtu: 247 },
                "ev=mtu id=1 mtu=247 at_ms=42",
            ),
            (Event::Wifi { ap_on: false }, "ev=wifi ap=off at_ms=42"),
            (
                Event::Net {
                    mode: Mode::Fallback,
                    reason: Reason::Scan,
                },
                "ev=net mode=Fallback reason=Scan at_ms=42",
            ),
            (
                Event::Net {
                    mode: Mode::Home,
                    reason: Reason::None,
                },
                "ev=net mode=Home at_ms=42",
            ),
            (
                Event::Net {
                    mode: Mode::Attempting(Step::Dhcp),
                    reason: Reason::None,
                },
                "ev=net mode=Attempting step=Dhcp at_ms=42",
            ),
            (
                Event::NetStep {
                    step: Step::Join,
                    ok: false,
                    took_ms: 4_210,
                },
                "ev=net_step step=Join result=Failed took_ms=4210 at_ms=42",
            ),
            (
                Event::ModeCmd {
                    source: Source::Udp,
                    cmd: Command::CarNetwork,
                    error: None,
                },
                "ev=mode_cmd source=Udp cmd=CarNetwork result=Ok at_ms=42",
            ),
            (
                Event::ModeCmd {
                    source: Source::Ble,
                    cmd: Command::Home,
                    error: Some(Error::NotReady),
                },
                "ev=mode_cmd source=Ble cmd=Home result=NotReady at_ms=42",
            ),
            (
                Event::Resolve {
                    ok: true,
                    took_ms: 112,
                    ip: [192, 168, 1, 175],
                },
                "ev=resolve result=Ok took_ms=112 ip=192.168.1.175 at_ms=42",
            ),
            (
                Event::Resolve {
                    ok: false,
                    took_ms: 3000,
                    ip: [0; 4],
                },
                "ev=resolve result=Failed took_ms=3000 ip=0.0.0.0 at_ms=42",
            ),
            (
                Event::Preempted(Source::Ros),
                "ev=preempted source=Ros at_ms=42",
            ),
            (Event::Ros(Link::Retry), "ev=ros link=retry at_ms=42"),
            (Event::Ros(Link::Refresh), "ev=ros link=refresh at_ms=42"),
            (Event::Fault(Fault::WifiAp), "ev=error src=wifi_ap at_ms=42"),
            (
                Event::Fault(Fault::MotorNonFinite),
                "ev=error src=motor_nonfinite at_ms=42",
            ),
            (
                Event::Fault(Fault::MotorConfig),
                "ev=error src=motor_config at_ms=42",
            ),
            (
                Event::Fault(Fault::MotorDriver),
                "ev=error src=motor_driver at_ms=42",
            ),
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
            ..Effects::default()
        };
        let mut evs = Vec::new();
        effects(fx, |e| evs.push(e));
        // The old session's stop and release come before the new claim (deadline ordering).
        assert_eq!(
            evs,
            [
                Event::Stop {
                    reason: StopReason::OwnerLost,
                    lease_late_ms: None,
                },
                Event::Released,
                Event::Claimed(Source::Ble)
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
                ev: Event::Stop {
                    reason: StopReason::LeaseExpired,
                    lease_late_ms: Some(0),
                }
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
