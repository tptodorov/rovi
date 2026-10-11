//! Motor control safety core: four wheel commands in, one drive per driver channel out.
//!
//! Rules (spec: `spec/changes/add-four-wheel-api/specs/motor-control`):
//! * Standby from boot until `enable`; `disable` and every fault return to standby, latched until
//!   the next `enable`. Standby is immediate, never ramped.
//! * A finite wheel command is clamped to -1.0..=1.0; any non-finite one puts every driver in standby.
//! * Without a fresh command for `command_timeout_ms`, the core drops to standby (watchdog).
//! * The applied command moves towards the target at most `slew_per_s` per second (O7).
//! * Rail current above `trip_ma` puts every driver in standby (O1).
//!
//! Every number in `Config::BENCH` is a placeholder until the M4 bench measures it.

/// Wheel order everywhere: FL, FR, RR, RL.
pub const WHEELS: usize = 4;

/// What a driver channel does with a zero command (O5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZeroMode {
    /// IN1 = IN2 = H: outputs shorted low (short brake).
    Brake,
    /// IN1 = IN2 = L: outputs off (Hi-Z), the wheel coasts.
    Coast,
}

/// Which driver channel (0 = D1A, 1 = D1B, 2 = D2A, 3 = D2B) drives a wheel, and whether its
/// forward is the channel's reverse (O8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    pub index: u8,
    pub invert: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Duty in permille at wheel command 1.0 (O1).
    pub max_duty_pm: u16,
    /// Largest change of a wheel command per second; `f32::INFINITY` turns the slew limit off (O7).
    pub slew_per_s: f32,
    /// Watchdog: standby when no fresh command arrived for longer than this.
    pub command_timeout_ms: u32,
    /// Rail current that trips standby (O1).
    pub trip_ma: i32,
    pub zero: ZeroMode,
    /// Indexed by wheel (FL, FR, RR, RL).
    pub wheels: [Channel; WHEELS],
}

impl Config {
    /// Bench defaults. PLACEHOLDERS, not measured: 40 % duty is what M1 ran one motor at; the rest
    /// are guesses to be replaced by the TESTPLAN measurements.
    pub const BENCH: Config = Config {
        max_duty_pm: 400,
        slew_per_s: 2.0,
        command_timeout_ms: 200,
        trip_ma: 1500,
        zero: ZeroMode::Coast,
        wheels: [
            Channel {
                index: 0,
                invert: false,
            }, // FL = D1A
            Channel {
                index: 2,
                invert: false,
            }, // FR = D2A
            Channel {
                index: 3,
                invert: false,
            }, // RR = D2B
            Channel {
                index: 1,
                invert: false,
            }, // RL = D1B
        ],
    };
}

/// Why the drivers are in standby.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Boot,
    Disabled,
    Watchdog,
    NonFinite,
    OverCurrent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Standby(Reason),
    Enabled,
}

/// One driver channel's pins: direction and duty in permille, or a zero mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drive {
    Forward(u16),
    Reverse(u16),
    Brake,
    Coast,
}

/// What the firmware writes to the pins after a tick. `enabled` is the STBY_EN line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output {
    pub enabled: bool,
    /// Indexed by driver channel (D1A, D1B, D2A, D2B).
    pub channels: [Drive; WHEELS],
}

pub struct Core {
    config: Config,
    mode: Mode,
    target: [f32; WHEELS],
    applied: [f32; WHEELS],
    fresh_ms: u64,
    last_tick_ms: Option<u64>,
}

impl Core {
    pub const fn new(config: Config) -> Self {
        Core {
            config,
            mode: Mode::Standby(Reason::Boot),
            target: [0.0; WHEELS],
            applied: [0.0; WHEELS],
            fresh_ms: 0,
            last_tick_ms: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The applied wheel commands (after clamping and slew), FL, FR, RR, RL.
    pub fn wheels(&self) -> [f32; WHEELS] {
        self.applied
    }

    pub fn target(&self) -> [f32; WHEELS] {
        self.target
    }

    pub fn set_zero(&mut self, zero: ZeroMode) {
        self.config.zero = zero;
    }

    /// Leaves standby with every wheel at zero; the watchdog starts now.
    pub fn enable(&mut self, now_ms: u64) {
        if self.mode != Mode::Enabled {
            self.target = [0.0; WHEELS];
            self.applied = [0.0; WHEELS];
            self.mode = Mode::Enabled;
        }
        self.fresh_ms = now_ms;
    }

    pub fn disable(&mut self) {
        self.standby(Reason::Disabled);
    }

    /// A new set of wheel commands. Ignored in standby, except that a non-finite value still
    /// re-latches standby.
    pub fn command(&mut self, wheels: [f32; WHEELS], now_ms: u64) {
        if wheels.iter().any(|w| !w.is_finite()) {
            self.standby(Reason::NonFinite);
            return;
        }
        if self.mode == Mode::Enabled {
            self.target = wheels.map(|w| w.clamp(-1.0, 1.0));
            self.fresh_ms = now_ms;
        }
    }

    /// One rail current sample in mA.
    pub fn current(&mut self, ma: i32) {
        if ma > self.config.trip_ma {
            self.standby(Reason::OverCurrent);
        }
    }

    /// Runs the watchdog and the slew limit, and returns what to write to the pins.
    pub fn tick(&mut self, now_ms: u64) -> Output {
        let dt_ms = self.last_tick_ms.map_or(0, |t| now_ms.saturating_sub(t));
        self.last_tick_ms = Some(now_ms);
        if self.mode == Mode::Enabled
            && now_ms.saturating_sub(self.fresh_ms) > u64::from(self.config.command_timeout_ms)
        {
            self.standby(Reason::Watchdog);
        }
        if self.mode != Mode::Enabled {
            return Output {
                enabled: false,
                channels: [Drive::Coast; WHEELS],
            };
        }
        let step = if self.config.slew_per_s.is_finite() {
            self.config.slew_per_s * dt_ms as f32 / 1000.0
        } else {
            f32::INFINITY // `INFINITY * 0 ms` would be NaN
        };
        for (a, t) in self.applied.iter_mut().zip(self.target) {
            // Snap onto the target when within one step, so rounding never leaves it 1e-7 short.
            *a = if (t - *a).abs() <= step {
                t
            } else {
                *a + (t - *a).clamp(-step, step)
            };
        }
        let mut channels = [Drive::Coast; WHEELS];
        for (w, ch) in self.applied.iter().zip(self.config.wheels) {
            channels[ch.index as usize] = self.drive(*w, ch.invert);
        }
        Output {
            enabled: true,
            channels,
        }
    }

    fn drive(&self, w: f32, invert: bool) -> Drive {
        let duty = libm::roundf(w.abs() * f32::from(self.config.max_duty_pm)) as u16;
        match (duty, (w < 0.0) != invert) {
            (0, _) => match self.config.zero {
                ZeroMode::Brake => Drive::Brake,
                ZeroMode::Coast => Drive::Coast,
            },
            (d, false) => Drive::Forward(d),
            (d, true) => Drive::Reverse(d),
        }
    }

    fn standby(&mut self, reason: Reason) {
        self.mode = Mode::Standby(reason);
        self.target = [0.0; WHEELS];
        self.applied = [0.0; WHEELS];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> Core {
        Core::new(Config {
            slew_per_s: f32::INFINITY,
            ..Config::BENCH
        })
    }

    const OFF: Output = Output {
        enabled: false,
        channels: [Drive::Coast; WHEELS],
    };

    #[test]
    fn boots_in_standby_and_ignores_commands_until_enabled() {
        let mut c = core();
        assert_eq!(c.mode(), Mode::Standby(Reason::Boot));
        c.command([0.5; 4], 0);
        assert_eq!(c.tick(10), OFF);
        assert_eq!(c.wheels(), [0.0; 4]);
    }

    #[test]
    fn enable_starts_at_zero_and_then_drives() {
        let mut c = core();
        c.command([0.5; 4], 0); // before enable: dropped, not remembered
        c.enable(10);
        let out = c.tick(10);
        assert!(out.enabled);
        assert_eq!(out.channels, [Drive::Coast; 4]);
        c.command([0.5, 0.0, 0.0, 0.0], 20);
        let out = c.tick(20);
        // FL is D1A: half of the 400 permille maximum.
        assert_eq!(out.channels[0], Drive::Forward(200));
        assert_eq!(out.channels[1..], [Drive::Coast; 3]);
    }

    #[test]
    fn sign_and_inversion_set_the_direction() {
        let mut cfg = Config::BENCH;
        cfg.slew_per_s = f32::INFINITY;
        cfg.wheels[1].invert = true; // FR on D2A
        let mut c = Core::new(cfg);
        c.enable(0);
        c.command([-0.5, 0.5, -1.0, 0.25], 0);
        let out = c.tick(0);
        assert_eq!(out.channels[0], Drive::Reverse(200)); // FL
        assert_eq!(out.channels[2], Drive::Reverse(200)); // FR inverted
        assert_eq!(out.channels[3], Drive::Reverse(400)); // RR
        assert_eq!(out.channels[1], Drive::Forward(100)); // RL
    }

    #[test]
    fn out_of_range_is_clamped() {
        let mut c = core();
        c.enable(0);
        c.command([1.5, -7.0, 0.0, 0.0], 0);
        let out = c.tick(0);
        assert_eq!(out.channels[0], Drive::Forward(400));
        assert_eq!(out.channels[2], Drive::Reverse(400));
        assert_eq!(c.wheels(), [1.0, -1.0, 0.0, 0.0]);
    }

    #[test]
    fn non_finite_puts_every_driver_in_standby_and_latches() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut c = core();
            c.enable(0);
            c.command([0.2; 4], 0);
            assert!(c.tick(0).enabled);
            c.command([bad, 0.2, 0.2, 0.2], 5);
            assert_eq!(c.mode(), Mode::Standby(Reason::NonFinite));
            assert_eq!(c.tick(5), OFF);
            c.command([0.2; 4], 6); // valid again, still latched
            assert_eq!(c.tick(6), OFF);
            c.enable(7);
            assert_eq!(c.tick(7).channels, [Drive::Coast; 4]);
        }
    }

    #[test]
    fn non_finite_in_standby_keeps_the_new_reason() {
        let mut c = core();
        c.command([f32::NAN, 0.0, 0.0, 0.0], 0);
        assert_eq!(c.mode(), Mode::Standby(Reason::NonFinite));
    }

    #[test]
    fn watchdog_drops_to_standby_without_fresh_commands() {
        let mut c = core();
        c.enable(0);
        c.command([0.3; 4], 0);
        assert!(c.tick(200).enabled); // exactly the timeout: still fresh
        assert_eq!(c.tick(201), OFF);
        assert_eq!(c.mode(), Mode::Standby(Reason::Watchdog));
        c.command([0.3; 4], 202); // late command does not re-enable
        assert_eq!(c.tick(203), OFF);
    }

    #[test]
    fn watchdog_runs_from_enable_even_without_any_command() {
        let mut c = core();
        c.enable(1000);
        assert!(c.tick(1200).enabled);
        assert_eq!(c.tick(1201), OFF);
        assert_eq!(c.mode(), Mode::Standby(Reason::Watchdog));
    }

    #[test]
    fn fresh_commands_keep_it_enabled() {
        let mut c = core();
        c.enable(0);
        for t in (0..2000).step_by(50) {
            c.command([0.1; 4], t);
            assert!(c.tick(t + 10).enabled, "at {t}");
        }
    }

    #[test]
    fn disable_is_immediate_and_needs_a_new_enable() {
        let mut c = core();
        c.enable(0);
        c.command([1.0; 4], 0);
        c.tick(0);
        c.disable();
        assert_eq!(c.mode(), Mode::Standby(Reason::Disabled));
        assert_eq!(c.tick(1), OFF);
        assert_eq!(c.wheels(), [0.0; 4]);
    }

    #[test]
    fn over_current_trips_standby() {
        let mut c = core();
        c.enable(0);
        c.command([1.0; 4], 0);
        c.current(1500); // at the limit: fine
        assert!(c.tick(0).enabled);
        c.current(1501);
        assert_eq!(c.mode(), Mode::Standby(Reason::OverCurrent));
        assert_eq!(c.tick(1), OFF);
    }

    #[test]
    fn zero_command_uses_the_zero_mode() {
        let mut c = core();
        c.set_zero(ZeroMode::Brake);
        c.enable(0);
        assert_eq!(c.tick(0).channels, [Drive::Brake; 4]);
        c.command([0.0004, 0.0, 0.0, 0.0], 1); // rounds to 0 permille
        assert_eq!(c.tick(1).channels[0], Drive::Brake);
        c.set_zero(ZeroMode::Coast);
        assert_eq!(c.tick(2).channels[0], Drive::Coast);
    }

    #[test]
    fn slew_limits_starts_and_reversals() {
        let mut c = Core::new(Config::BENCH); // 2.0 per second
        c.enable(0);
        c.tick(0);
        c.command([1.0, -1.0, 0.0, 0.0], 0);
        c.tick(100); // 0.2 per 100 ms
        assert_eq!(c.wheels(), [0.2, -0.2, 0.0, 0.0]);
        for t in (110..=520).step_by(10) {
            c.command([1.0, -1.0, 0.0, 0.0], t);
            c.tick(t);
        }
        assert_eq!(c.wheels()[0], 1.0);
        // Full reversal takes 1 s at 2.0/s and passes through zero.
        c.command([-1.0, 1.0, 0.0, 0.0], 520);
        c.tick(1020);
        assert!(c.wheels()[0].abs() < 1e-6);
        assert_eq!(c.tick(1020).channels[0], Drive::Coast);
    }

    #[test]
    fn standby_is_never_ramped() {
        let mut c = Core::new(Config::BENCH);
        c.enable(0);
        c.tick(0);
        for t in (10..=600).step_by(10) {
            c.command([1.0; 4], t);
            c.tick(t);
        }
        assert_eq!(c.wheels(), [1.0; 4]);
        c.disable();
        assert_eq!(c.tick(610), OFF);
        assert_eq!(c.wheels(), [0.0; 4]);
    }

    #[test]
    fn re_enable_starts_from_zero_not_the_old_target() {
        let mut c = core();
        c.enable(0);
        c.command([0.8; 4], 0);
        c.tick(0);
        c.disable();
        c.enable(10);
        assert_eq!(c.tick(10).channels, [Drive::Coast; 4]);
        assert_eq!(c.target(), [0.0; 4]);
    }

    #[test]
    fn clock_going_backwards_does_not_panic_or_unlock() {
        let mut c = Core::new(Config::BENCH);
        c.enable(100);
        c.tick(100);
        c.command([1.0; 4], 100);
        let _ = c.tick(50);
        assert!(c.wheels()[0] <= 1.0);
    }
}
