//! M4's log lines, in the shared format of docs/LOGGING.md: `ev=<name> [key=value ...] at_ms=<n>`.
//! Integers only. `boot`, `error` and `dropped` keep their meaning from LOGGING.md; `motor`, `power`
//! and `cmd` are M4's own (README, "Events").

use crate::command::Level;
use crate::motor::{Mode, Reason, ZeroMode, WHEELS};
use crate::power::Power;
use core::fmt::{self, Write};

/// Bench commands worth a line. Wheel commands are not echoed; `motor` shows their effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Enable,
    Disable,
    Zero(ZeroMode),
    Hang(Level),
    Bad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The INA226 did not answer or gave the wrong ID: no rail current, so no over-current trip.
    Ina226,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// `reset` is the chip's reset cause (`SocResetReason` name, or `Unknown`).
    Boot {
        reset: &'static str,
    },
    /// Applied wheel commands (FL, FR, RR, RL) in permille of full scale.
    Motor {
        mode: Mode,
        wheels_pm: [i16; WHEELS],
    },
    Power(Power),
    Cmd(Op),
    Fault(Fault),
    Dropped(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub at_ms: u32,
    pub ev: Event,
}

/// Wheel commands -1.0..=1.0 as permille (saturating).
pub fn permille(w: [f32; WHEELS]) -> [i16; WHEELS] {
    w.map(|v| libm::roundf(v * 1000.0) as i16)
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ev {
            Event::Boot { reset } => {
                write!(f, "ev=boot ver={} reset={reset}", env!("CARGO_PKG_VERSION"))?
            }
            Event::Motor { mode, wheels_pm: w } => {
                match mode {
                    Mode::Enabled => f.write_str("ev=motor state=Enabled")?,
                    Mode::Standby(r) => write!(f, "ev=motor state=Standby reason={}", reason(r))?,
                }
                write!(f, " wheels_pm={},{},{},{}", w[0], w[1], w[2], w[3])?
            }
            Event::Power(p) => write!(
                f,
                "ev=power bus_mv={} bus_min_mv={} ma={} ma_max={}",
                p.bus_mv, p.bus_min_mv, p.ma, p.ma_max
            )?,
            Event::Cmd(op) => match op {
                Op::Enable => f.write_str("ev=cmd op=Enable")?,
                Op::Disable => f.write_str("ev=cmd op=Disable")?,
                Op::Zero(ZeroMode::Brake) => f.write_str("ev=cmd op=Zero mode=Brake")?,
                Op::Zero(ZeroMode::Coast) => f.write_str("ev=cmd op=Zero mode=Coast")?,
                Op::Hang(Level::High) => f.write_str("ev=cmd op=Hang level=High")?,
                Op::Hang(Level::Low) => f.write_str("ev=cmd op=Hang level=Low")?,
                Op::Bad => f.write_str("ev=cmd op=Bad")?,
            },
            Event::Fault(Fault::Ina226) => f.write_str("ev=error src=ina226")?,
            Event::Dropped(n) => write!(f, "ev=dropped n={n}")?,
        }
        write!(f, " at_ms={}", self.at_ms)
    }
}

fn reason(r: Reason) -> &'static str {
    match r {
        Reason::Boot => "Boot",
        Reason::Disabled => "Disabled",
        Reason::Watchdog => "Watchdog",
        Reason::NonFinite => "NonFinite",
        Reason::OverCurrent => "OverCurrent",
    }
}

/// Decides when the control tick logs `ev=motor`: on every mode change, and while the wheels
/// change at most every `min_ms`, plus once when a ramp settles, so the final value is in the log.
pub struct Reporter {
    min_ms: u64,
    last: Option<(Mode, [i16; WHEELS], u64)>,
}

impl Reporter {
    pub const fn new(min_ms: u64) -> Self {
        Reporter { min_ms, last: None }
    }

    pub fn report(
        &mut self,
        mode: Mode,
        wheels: [f32; WHEELS],
        target: [f32; WHEELS],
        now_ms: u64,
    ) -> Option<Event> {
        let pm = permille(wheels);
        let due = match self.last {
            None => true,
            Some((m, w, at)) => {
                m != mode
                    || (w != pm
                        && (now_ms.saturating_sub(at) >= self.min_ms || pm == permille(target)))
            }
        };
        if !due {
            return None;
        }
        self.last = Some((mode, pm, now_ms));
        Some(Event::Motor {
            mode,
            wheels_pm: pm,
        })
    }
}

/// Room for the longest line (see the test) plus its newline.
pub const MAX_LINE: usize = 128;

/// A fixed buffer for one line. A line that does not fit is cut and so loses its `at_ms`, which is
/// how parsers recognise it as incomplete.
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

    fn line(ev: Event) -> String {
        Record { at_ms: 42, ev }.to_string()
    }

    #[test]
    fn every_event_has_one_stable_line() {
        let cases = [
            (
                Event::Boot {
                    reset: "ChipPowerOn",
                },
                "ev=boot ver=0.1.0 reset=ChipPowerOn at_ms=42",
            ),
            (
                Event::Motor {
                    mode: Mode::Standby(Reason::Watchdog),
                    wheels_pm: [0; 4],
                },
                "ev=motor state=Standby reason=Watchdog wheels_pm=0,0,0,0 at_ms=42",
            ),
            (
                Event::Motor {
                    mode: Mode::Enabled,
                    wheels_pm: [300, -1000, 0, 5],
                },
                "ev=motor state=Enabled wheels_pm=300,-1000,0,5 at_ms=42",
            ),
            (
                Event::Power(Power {
                    bus_mv: 5012,
                    bus_min_mv: 4890,
                    ma: 310,
                    ma_max: -20,
                }),
                "ev=power bus_mv=5012 bus_min_mv=4890 ma=310 ma_max=-20 at_ms=42",
            ),
            (Event::Cmd(Op::Enable), "ev=cmd op=Enable at_ms=42"),
            (Event::Cmd(Op::Disable), "ev=cmd op=Disable at_ms=42"),
            (
                Event::Cmd(Op::Zero(ZeroMode::Brake)),
                "ev=cmd op=Zero mode=Brake at_ms=42",
            ),
            (
                Event::Cmd(Op::Hang(Level::Low)),
                "ev=cmd op=Hang level=Low at_ms=42",
            ),
            (Event::Cmd(Op::Bad), "ev=cmd op=Bad at_ms=42"),
            (Event::Fault(Fault::Ina226), "ev=error src=ina226 at_ms=42"),
            (Event::Dropped(3), "ev=dropped n=3 at_ms=42"),
        ];
        for (ev, want) in cases {
            assert_eq!(line(ev), want);
        }
    }

    #[test]
    fn every_reason_has_a_name() {
        for (r, name) in [
            (Reason::Boot, "Boot"),
            (Reason::Disabled, "Disabled"),
            (Reason::Watchdog, "Watchdog"),
            (Reason::NonFinite, "NonFinite"),
            (Reason::OverCurrent, "OverCurrent"),
        ] {
            let l = line(Event::Motor {
                mode: Mode::Standby(r),
                wheels_pm: [0; 4],
            });
            assert!(l.contains(&std::format!(" reason={name} ")), "{l}");
        }
    }

    #[test]
    fn longest_line_fits() {
        let mut buf = LineBuf::<MAX_LINE>::new();
        let rec = Record {
            at_ms: u32::MAX,
            ev: Event::Power(Power {
                bus_mv: u32::MAX,
                bus_min_mv: u32::MAX,
                ma: i32::MIN,
                ma_max: i32::MIN,
            }),
        };
        assert!(writeln!(buf, "{rec}").is_ok());
        let rec = Record {
            at_ms: u32::MAX,
            ev: Event::Motor {
                mode: Mode::Standby(Reason::OverCurrent),
                wheels_pm: [i16::MIN; 4],
            },
        };
        let mut buf = LineBuf::<MAX_LINE>::new();
        assert!(writeln!(buf, "{rec}").is_ok());
    }

    #[test]
    fn reporter_logs_mode_changes_at_once_and_rate_limits_ramps() {
        let mut r = Reporter::new(50);
        let standby = Mode::Standby(Reason::Boot);
        assert!(r.report(standby, [0.0; 4], [0.0; 4], 0).is_some()); // first: always
        assert!(r.report(standby, [0.0; 4], [0.0; 4], 10).is_none()); // nothing changed
        assert_eq!(
            r.report(Mode::Enabled, [0.0; 4], [0.0; 4], 20),
            Some(Event::Motor {
                mode: Mode::Enabled,
                wheels_pm: [0; 4]
            })
        );
        let target = [1.0, 0.0, 0.0, 0.0];
        assert!(r
            .report(Mode::Enabled, [0.1, 0.0, 0.0, 0.0], target, 30)
            .is_none()); // < 50 ms
        assert!(r
            .report(Mode::Enabled, [0.5, 0.0, 0.0, 0.0], target, 70)
            .is_some());
        assert!(r
            .report(Mode::Enabled, [0.6, 0.0, 0.0, 0.0], target, 80)
            .is_none());
        // Settled on the target: logged even inside the 50 ms.
        assert!(r.report(Mode::Enabled, target, target, 90).is_some());
        // Standby is a mode change: logged at once.
        let stop = Mode::Standby(Reason::Watchdog);
        assert!(r.report(stop, [0.0; 4], [0.0; 4], 91).is_some());
    }

    #[test]
    fn permille_rounds_and_saturates() {
        assert_eq!(
            permille([0.2504, -1.0, 0.0, 40.0]),
            [250, -1000, 0, i16::MAX]
        );
    }
}
