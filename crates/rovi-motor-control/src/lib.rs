//! Four independent wheel commands to two TB6612FNG drivers (M6 `motor-control`).
//!
//! Knows nothing of setpoints, kinematics or transports: it takes four normalised wheel commands
//! (FL, FR, RR, RL, -1..=1) and writes STBY and bridge pins through a [`Driver`]. The drivers stay
//! in standby until [`MotorControl::enable`]; every change of state deasserts STBY before any
//! bridge pin is touched, and asserts it only after all four bridges are prepared.
#![no_std]

#[cfg(test)]
extern crate std;

/// Wheel commands in the kinematics order: FL, FR, RR, RL.
pub type Wheels = [f32; 4];

/// Bridges in driver order: D1 A, D1 B, D2 A, D2 B. Bridge `i` belongs to driver `i / 2`.
pub const BRIDGES: usize = 4;
/// Each driver has its own STBY output.
pub const DRIVERS: usize = 2;

/// Duty mapping for a wheel command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Duty in permille just outside the deadband: the motors' starting duty (about 35 % on the
    /// M4 bench), so a gentle command still turns the wheel.
    pub min_duty_pm: u16,
    /// Duty in permille for a wheel command of 1.0. A cap on drive duty, not current protection.
    pub max_duty_pm: u16,
    /// Commands with a smaller magnitude coast (joystick drift, rounding).
    pub deadband: f32,
}

impl Config {
    /// The product profile (M6 O1/O6): 40-70 % drive duty, the envelope M4 verified.
    pub const PRODUCT: Config = Config {
        min_duty_pm: 400,
        max_duty_pm: 700,
        deadband: 0.02,
    };
}

/// Which wheel one bridge drives, and whether its motor is wired reversed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    /// Index into [`Wheels`]: 0 = FL, 1 = FR, 2 = RR, 3 = RL.
    pub wheel: usize,
    pub invert: bool,
}

/// The current car (M4/M5, O8): D1 A→FL, D1 B→RL, D2 A→FR, D2 B→RR; directions are fixed at the
/// connectors, so nothing is inverted.
pub const CAR_CHANNELS: [Channel; BRIDGES] = [
    Channel {
        wheel: 0,
        invert: false,
    },
    Channel {
        wheel: 3,
        invert: false,
    },
    Channel {
        wheel: 1,
        invert: false,
    },
    Channel {
        wheel: 2,
        invert: false,
    },
];

/// One bridge's pins: `IN1`, `IN2` and the PWM duty in permille (1000 = always high).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pins {
    pub in1: bool,
    pub in2: bool,
    pub pwm_pm: u16,
}

impl Pins {
    /// `IN1 = IN2 = low`, PWM high: the TB6612 "stop" row (outputs high impedance), so the wheel
    /// coasts. With PWM low it would be outside that row (Toshiba H-SW table). PWM high here is
    /// part of the coast state, not drive duty, so the drive cap does not apply.
    pub const COAST: Pins = Pins {
        in1: false,
        in2: false,
        pwm_pm: 1000,
    };
}

/// The pin boundary: the ESP32 implementation or a recording test double.
pub trait Driver {
    type Error;
    /// Sets one driver's STBY output; `true` enables its two bridges.
    fn set_stby(&mut self, driver: usize, high: bool) -> Result<(), Self::Error>;
    /// Writes one bridge's pins.
    fn set_bridge(&mut self, bridge: usize, pins: Pins) -> Result<(), Self::Error>;
}

/// Why a configuration cannot drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The deadband is not a finite value in `0.0..1.0`.
    Deadband,
    /// Not `min_duty_pm <= max_duty_pm <= 1000`.
    DutyLimits,
    /// The bridges do not drive each of the four wheels exactly once.
    Mapping,
}

/// Why motor control is latched in standby.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A wheel command was NaN or infinite. Only [`MotorControl::enable`] with a finite vector
    /// clears it.
    NonFinite,
    /// The configuration is invalid. Only [`MotorControl::configure`] with a valid one clears it.
    Config(ConfigError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// Both drivers are in standby until the fault is cleared.
    Fault(Fault),
    /// Wheel updates need an explicit [`MotorControl::enable`] first.
    NotEnabled,
    /// The pin driver failed.
    Driver(E),
}

impl Config {
    /// Checks the duty mapping and that the bridges drive each wheel exactly once.
    pub fn validate(&self, channels: &[Channel; BRIDGES]) -> Result<(), ConfigError> {
        if !(self.deadband.is_finite() && (0.0..1.0).contains(&self.deadband)) {
            return Err(ConfigError::Deadband);
        }
        if self.min_duty_pm > self.max_duty_pm || self.max_duty_pm > 1000 {
            return Err(ConfigError::DutyLimits);
        }
        let mut seen = [false; 4];
        for ch in channels {
            match seen.get_mut(ch.wheel) {
                Some(s) if !*s => *s = true,
                _ => return Err(ConfigError::Mapping),
            }
        }
        Ok(())
    }
}

/// A bridge's pins for one wheel command: clamped to -1..=1, coast inside the deadband, otherwise
/// duty runs linearly from `min_duty_pm` to `max_duty_pm` and never above the cap.
pub fn pins(cfg: &Config, command: f32) -> Pins {
    let w = command.clamp(-1.0, 1.0);
    if w == 0.0 || w.abs() < cfg.deadband {
        return Pins::COAST;
    }
    let max = cfg.max_duty_pm.min(1000);
    let min = f32::from(cfg.min_duty_pm.min(max));
    let duty = (min + w.abs() * (f32::from(max) - min) + 0.5) as u16;
    Pins {
        in1: w > 0.0,
        in2: w < 0.0,
        pwm_pm: duty.min(max),
    }
}

/// Owns the driver; the only way to its pins.
pub struct MotorControl<D> {
    driver: D,
    cfg: Config,
    channels: [Channel; BRIDGES],
    enabled: bool,
    config_error: Option<ConfigError>,
    /// Set by a non-finite command; cleared only by a successful enable.
    latched: bool,
}

impl<D: Driver> MotorControl<D> {
    /// Takes the driver, puts both drivers in standby, then validates the configuration. An
    /// invalid one is reported by [`fault`](Self::fault) and keeps both drivers in standby.
    pub fn new(
        driver: D,
        cfg: Config,
        channels: [Channel; BRIDGES],
    ) -> Result<Self, Error<D::Error>> {
        let mut mc = MotorControl {
            driver,
            cfg,
            channels,
            enabled: false,
            config_error: cfg.validate(&channels).err(),
            latched: false,
        };
        mc.standby()?;
        Ok(mc)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The fault holding both drivers in standby, if any.
    pub fn fault(&self) -> Option<Fault> {
        match (self.config_error, self.latched) {
            (Some(e), _) => Some(Fault::Config(e)),
            (None, true) => Some(Fault::NonFinite),
            (None, false) => None,
        }
    }

    /// Replaces the configuration: standby first, then validation. Configuration is never
    /// changed while enabled; driving again needs [`enable`](Self::enable).
    pub fn configure(
        &mut self,
        cfg: Config,
        channels: [Channel; BRIDGES],
    ) -> Result<(), Error<D::Error>> {
        self.standby()?;
        self.cfg = cfg;
        self.channels = channels;
        self.config_error = cfg.validate(&channels).err();
        match self.config_error {
            Some(e) => Err(Error::Fault(Fault::Config(e))),
            None => Ok(()),
        }
    }

    /// Deasserts both STBY outputs. Always allowed, from any state.
    pub fn standby(&mut self) -> Result<(), Error<D::Error>> {
        self.enabled = false;
        for d in 0..DRIVERS {
            self.driver.set_stby(d, false).map_err(Error::Driver)?;
        }
        Ok(())
    }

    /// Prepares all four bridges for `wheels` while in standby, then asserts both STBY outputs.
    /// The only way to clear a [`Fault::NonFinite`]; it applies `wheels` alone, never an earlier
    /// vector. Fails with both drivers in standby on an invalid configuration or command.
    pub fn enable(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        self.standby()?;
        if let Some(e) = self.config_error {
            return Err(Error::Fault(Fault::Config(e)));
        }
        self.write_bridges(wheels)?;
        for d in 0..DRIVERS {
            self.driver.set_stby(d, true).map_err(Error::Driver)?;
        }
        self.latched = false;
        self.enabled = true;
        Ok(())
    }

    /// Applies new wheel commands while enabled. A non-finite command latches both drivers in
    /// standby; later updates fail with the fault and write nothing.
    pub fn update(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        if let Some(f) = self.fault() {
            return Err(Error::Fault(f));
        }
        if !self.enabled {
            return Err(Error::NotEnabled);
        }
        self.write_bridges(wheels)
    }

    /// Checks the whole vector before writing any bridge, so a bad fourth value cannot drive the
    /// first three.
    fn write_bridges(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        if wheels.iter().any(|w| !w.is_finite()) {
            self.latched = true;
            self.standby()?;
            return Err(Error::Fault(Fault::NonFinite));
        }
        for (bridge, ch) in self.channels.iter().enumerate() {
            let w = wheels.get(ch.wheel).copied().unwrap_or(0.0);
            let p = pins(&self.cfg, if ch.invert { -w } else { w });
            self.driver.set_bridge(bridge, p).map_err(Error::Driver)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Op {
        Stby(usize, bool),
        Bridge(usize, Pins),
    }

    /// Records every pin write in order; `fail_bridge` makes bridge writes fail.
    #[derive(Default)]
    struct Recording {
        ops: Vec<Op>,
        fail_bridge: bool,
    }

    impl Driver for Recording {
        type Error = &'static str;
        fn set_stby(&mut self, driver: usize, high: bool) -> Result<(), Self::Error> {
            self.ops.push(Op::Stby(driver, high));
            Ok(())
        }
        fn set_bridge(&mut self, bridge: usize, pins: Pins) -> Result<(), Self::Error> {
            if self.fail_bridge {
                return Err("bridge write failed");
            }
            self.ops.push(Op::Bridge(bridge, pins));
            Ok(())
        }
    }

    const STANDBY: [Op; 2] = [Op::Stby(0, false), Op::Stby(1, false)];
    const NON_FINITE: Error<&str> = Error::Fault(Fault::NonFinite);
    const ENABLE: [Op; 2] = [Op::Stby(0, true), Op::Stby(1, true)];
    /// IN1 = IN2 = low with PWM high: the TB6612 "stop" (high-impedance) row, i.e. coast.
    const COAST: Pins = Pins {
        in1: false,
        in2: false,
        pwm_pm: 1000,
    };

    fn fwd(pwm_pm: u16) -> Pins {
        Pins {
            in1: true,
            in2: false,
            pwm_pm,
        }
    }
    fn rev(pwm_pm: u16) -> Pins {
        Pins {
            in1: false,
            in2: true,
            pwm_pm,
        }
    }

    fn car(cfg: Config) -> MotorControl<Recording> {
        let mut mc = MotorControl::new(Recording::default(), cfg, CAR_CHANNELS).unwrap();
        mc.driver.ops.clear();
        mc
    }

    /// The bridge writes of the last enable or update, in bridge order.
    fn bridges(mc: &MotorControl<Recording>) -> Vec<Pins> {
        let mut out = Vec::new();
        for op in &mc.driver.ops {
            if let Op::Bridge(_, p) = op {
                out.push(*p);
            }
        }
        out.split_off(out.len().saturating_sub(BRIDGES))
    }

    #[test]
    fn zero_requests_coast_with_both_direction_pins_low_and_pwm_high() {
        assert_eq!(pins(&Config::PRODUCT, 0.0), COAST);
        assert_eq!(pins(&Config::PRODUCT, 0.01), COAST, "inside the deadband");
    }

    #[test]
    fn coast_keeps_pwm_high_under_a_70_percent_drive_cap() {
        // FL at zero, RL at 1.0: FL and RL share driver 1 (bridges 0 and 1).
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.0, 0.0, 0.0, 1.0]).unwrap();
        let b = bridges(&mc);
        assert_eq!(b[0], COAST, "FL coasts, not braked by PWM low");
        assert_eq!(b[1], fwd(700), "RL is driven at the 70 % cap");
        assert!(
            mc.is_enabled(),
            "a zero wheel does not disable its neighbour's driver"
        );
    }

    #[test]
    fn each_wheel_is_driven_on_its_own() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(bridges(&mc), [fwd(550), COAST, COAST, COAST]);
    }

    #[test]
    fn the_car_mapping_puts_each_wheel_on_its_bridge() {
        // FL, FR, RR, RL commands; bridges are D1 A (FL), D1 B (RL), D2 A (FR), D2 B (RR).
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.1, 0.2, 0.3, 0.4]).unwrap();
        assert_eq!(bridges(&mc), [fwd(430), fwd(520), fwd(460), fwd(490)]);
    }

    #[test]
    fn sign_sets_direction_and_inversion_flips_it() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([-0.5; 4]).unwrap();
        assert_eq!(bridges(&mc), [rev(550); 4]);

        let mut inverted = CAR_CHANNELS;
        inverted[0].invert = true;
        let mut mc = MotorControl::new(Recording::default(), Config::PRODUCT, inverted).unwrap();
        mc.enable([0.5, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(bridges(&mc)[0], rev(550));
    }

    #[test]
    fn deadband_and_duty_follow_the_rule() {
        // Spec scenario "Small command behaviour is predictable": deadband 0.02, 40-100 %.
        let cfg = Config {
            max_duty_pm: 1000,
            ..Config::PRODUCT
        };
        assert_eq!(pins(&cfg, 0.0), COAST);
        assert_eq!(pins(&cfg, 0.01), COAST);
        assert_eq!(pins(&cfg, 0.02), fwd(412));
        assert_eq!(pins(&cfg, 0.5), fwd(700));
        assert_eq!(pins(&cfg, 1.0), fwd(1000));
        assert_eq!(pins(&cfg, -0.02), rev(412));
        assert_eq!(pins(&cfg, -0.5), rev(700));
    }

    #[test]
    fn the_product_profile_caps_drive_duty_at_70_percent() {
        let cfg = Config::PRODUCT;
        assert_eq!(pins(&cfg, 1.0), fwd(700));
        assert_eq!(pins(&cfg, 0.5), fwd(550));
        assert_eq!(pins(&cfg, 1.5), fwd(700), "out of range is clamped to 1.0");
        assert_eq!(pins(&cfg, -7.0), rev(700));
        for i in 0..=200 {
            let w = -1.0 + i as f32 * 0.01;
            assert!(pins(&cfg, w).pwm_pm <= 700 || pins(&cfg, w) == COAST, "{w}");
        }
    }

    #[test]
    fn full_reversal_is_direct_with_the_drivers_kept_enabled() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([1.0; 4]).unwrap();
        mc.driver.ops.clear();
        mc.update([-1.0; 4]).unwrap();
        assert_eq!(bridges(&mc), [rev(700); 4], "no ramp");
        assert!(
            mc.driver.ops.iter().all(|op| matches!(op, Op::Bridge(..))),
            "no standby"
        );
    }

    #[test]
    fn new_starts_in_standby() {
        let mc = MotorControl::new(Recording::default(), Config::PRODUCT, CAR_CHANNELS).unwrap();
        assert_eq!(mc.driver.ops, STANDBY);
        assert!(!mc.is_enabled());
    }

    #[test]
    fn enable_prepares_every_bridge_before_asserting_stby() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        let ops = &mc.driver.ops;
        assert_eq!(ops[..2], STANDBY, "standby before any bridge is prepared");
        assert!(ops[2..6].iter().all(|op| matches!(op, Op::Bridge(..))));
        assert_eq!(
            ops[6..],
            ENABLE,
            "STBY goes high only after all four bridges"
        );
    }

    #[test]
    fn standby_deasserts_both_stby_outputs_first() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        mc.driver.ops.clear();
        mc.standby().unwrap();
        assert_eq!(mc.driver.ops, STANDBY);
        assert!(!mc.is_enabled());
    }

    #[test]
    fn a_non_finite_fourth_wheel_cannot_drive_the_first_three() {
        let mut mc = car(Config::PRODUCT);
        assert_eq!(mc.enable([0.5, 0.5, 0.5, f32::INFINITY]), Err(NON_FINITE));
        assert!(
            mc.driver
                .ops
                .iter()
                .all(|op| matches!(op, Op::Stby(_, false))),
            "no bridge write and no STBY high: {:?}",
            mc.driver.ops
        );
        assert!(!mc.is_enabled());
    }

    #[test]
    fn nan_while_driving_puts_both_drivers_in_standby() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.2; 4]).unwrap();
        mc.driver.ops.clear();
        assert_eq!(mc.update([f32::NAN, 0.2, 0.2, 0.2]), Err(NON_FINITE));
        assert_eq!(mc.driver.ops, STANDBY);
        assert_eq!(mc.fault(), Some(Fault::NonFinite));
    }

    #[test]
    fn updates_need_an_explicit_enable() {
        let mut mc = car(Config::PRODUCT);
        assert_eq!(mc.update([0.5; 4]), Err(Error::NotEnabled));
        assert!(mc.driver.ops.is_empty(), "nothing is written");
    }

    #[test]
    fn driver_failures_reach_the_caller() {
        let mut mc = car(Config::PRODUCT);
        mc.driver.fail_bridge = true;
        assert_eq!(
            mc.enable([0.5; 4]),
            Err(Error::Driver("bridge write failed"))
        );
        assert!(!mc.is_enabled());
        assert!(!mc.driver.ops.contains(&Op::Stby(0, true)));
    }

    fn ch(wheel: usize) -> Channel {
        Channel {
            wheel,
            invert: false,
        }
    }

    /// No STBY high and no bridge write: the drivers stayed off.
    fn stayed_off(mc: &MotorControl<Recording>) -> bool {
        mc.driver
            .ops
            .iter()
            .all(|op| matches!(op, Op::Stby(_, false)))
    }

    fn with(cfg: Config, channels: [Channel; BRIDGES]) -> MotorControl<Recording> {
        MotorControl::new(Recording::default(), cfg, channels).unwrap()
    }

    #[test]
    fn invalid_duty_limits_cannot_enable() {
        for (min, max) in [(800, 700), (400, 1001)] {
            let cfg = Config {
                min_duty_pm: min,
                max_duty_pm: max,
                ..Config::PRODUCT
            };
            let mut mc = with(cfg, CAR_CHANNELS);
            let fault = Fault::Config(ConfigError::DutyLimits);
            assert_eq!(mc.fault(), Some(fault), "{min}..{max}");
            assert_eq!(mc.enable([0.5; 4]), Err(Error::Fault(fault)));
            assert!(stayed_off(&mc), "{:?}", mc.driver.ops);
        }
    }

    #[test]
    fn a_mapping_that_is_not_one_to_one_cannot_enable() {
        for channels in [
            [ch(0), ch(0), ch(1), ch(2)], // FL on two bridges, RL on none
            [ch(0), ch(3), ch(1), ch(4)], // no such wheel
        ] {
            let mut mc = with(Config::PRODUCT, channels);
            let fault = Fault::Config(ConfigError::Mapping);
            assert_eq!(mc.fault(), Some(fault), "{channels:?}");
            assert_eq!(mc.enable([0.5; 4]), Err(Error::Fault(fault)));
            assert!(stayed_off(&mc));
        }
    }

    #[test]
    fn an_invalid_deadband_is_reported_before_any_enable() {
        for deadband in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.0, 1.5] {
            let cfg = Config {
                deadband,
                ..Config::PRODUCT
            };
            let mut mc = with(cfg, CAR_CHANNELS);
            let fault = Fault::Config(ConfigError::Deadband);
            assert_eq!(mc.fault(), Some(fault), "{deadband}");
            assert_eq!(mc.enable([0.5; 4]), Err(Error::Fault(fault)));
            assert!(stayed_off(&mc));
        }
    }

    #[test]
    fn boundary_configurations_are_valid() {
        for cfg in [
            Config {
                min_duty_pm: 700,
                max_duty_pm: 700,
                deadband: 0.0,
            },
            Config {
                min_duty_pm: 0,
                max_duty_pm: 1000,
                deadband: 0.999,
            },
            Config::PRODUCT,
        ] {
            assert_eq!(cfg.validate(&CAR_CHANNELS), Ok(()), "{cfg:?}");
        }
    }

    #[test]
    fn replacing_the_configuration_disables_first() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        mc.driver.ops.clear();
        let full = Config {
            max_duty_pm: 1000,
            ..Config::PRODUCT
        };
        mc.configure(full, CAR_CHANNELS).unwrap();
        assert_eq!(mc.driver.ops, STANDBY, "standby before validation");
        assert!(!mc.is_enabled());
        mc.enable([1.0; 4]).unwrap();
        assert_eq!(
            bridges(&mc),
            [fwd(1000); 4],
            "the new configuration applies"
        );
    }

    #[test]
    fn an_invalid_replacement_latches_and_a_valid_one_clears_it() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        let bad = Config {
            min_duty_pm: 900,
            ..Config::PRODUCT
        };
        let fault = Error::Fault(Fault::Config(ConfigError::DutyLimits));
        assert_eq!(mc.configure(bad, CAR_CHANNELS), Err(fault));
        assert!(!mc.is_enabled());
        assert_eq!(
            mc.enable([0.5; 4]),
            Err(fault),
            "enable cannot clear a bad configuration"
        );
        mc.configure(Config::PRODUCT, CAR_CHANNELS).unwrap();
        assert_eq!(mc.fault(), None);
        assert_eq!(mc.enable([0.5; 4]), Ok(()));
    }

    #[test]
    fn every_non_finite_value_at_every_wheel_latches_standby() {
        for wheel in 0..4 {
            for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut w = [0.3; 4];
                w[wheel] = bad;

                let mut mc = car(Config::PRODUCT);
                assert_eq!(mc.enable(w), Err(NON_FINITE), "enable {w:?}");
                assert!(stayed_off(&mc), "enable {w:?}: {:?}", mc.driver.ops);
                assert_eq!(mc.fault(), Some(Fault::NonFinite));

                let mut mc = car(Config::PRODUCT);
                mc.enable([0.3; 4]).unwrap();
                mc.driver.ops.clear();
                assert_eq!(mc.update(w), Err(NON_FINITE), "update {w:?}");
                assert_eq!(mc.driver.ops, STANDBY, "update {w:?}");
                assert_eq!(mc.fault(), Some(Fault::NonFinite));
            }
        }
    }

    #[test]
    fn finite_updates_do_not_clear_a_fault_and_enable_applies_only_the_new_vector() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        assert_eq!(mc.update([f32::NAN, 0.2, 0.2, 0.2]), Err(NON_FINITE));
        mc.driver.ops.clear();
        for _ in 0..3 {
            assert_eq!(mc.update([0.2; 4]), Err(NON_FINITE));
        }
        assert!(mc.driver.ops.is_empty(), "nothing is written while latched");
        assert!(!mc.is_enabled());

        mc.enable([0.1, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(mc.fault(), None);
        assert_eq!(
            bridges(&mc),
            [fwd(430), COAST, COAST, COAST],
            "no replay of 0.5 or 0.2"
        );
    }

    #[test]
    fn standby_works_in_every_fault_state() {
        let bad = Config {
            deadband: f32::NAN,
            ..Config::PRODUCT
        };
        let mut mc = with(bad, CAR_CHANNELS);
        mc.driver.ops.clear();
        assert_eq!(mc.standby(), Ok(()));
        assert_eq!(mc.driver.ops, STANDBY);

        let mut mc = car(Config::PRODUCT);
        mc.enable([0.5; 4]).unwrap();
        let _ = mc.update([f32::NAN; 4]);
        mc.driver.ops.clear();
        assert_eq!(mc.standby(), Ok(()));
        assert_eq!(mc.driver.ops, STANDBY);
    }

    #[test]
    fn clamping_works_in_both_directions() {
        let mut mc = car(Config::PRODUCT);
        mc.enable([1.5, -1.5, 0.0, 0.0]).unwrap();
        assert_eq!(bridges(&mc), [fwd(700), COAST, rev(700), COAST]);
    }
}
