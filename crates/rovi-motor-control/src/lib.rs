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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// A wheel command was NaN or infinite; both drivers are in standby.
    NonFinite,
    /// Wheel updates need an explicit [`MotorControl::enable`] first.
    NotEnabled,
    /// The pin driver failed.
    Driver(E),
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
}

impl<D: Driver> MotorControl<D> {
    /// Takes the driver and puts both drivers in standby.
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
        };
        mc.standby()?;
        Ok(mc)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
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
    /// A non-finite command leaves both drivers in standby.
    pub fn enable(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        self.standby()?;
        self.write_bridges(wheels)?;
        for d in 0..DRIVERS {
            self.driver.set_stby(d, true).map_err(Error::Driver)?;
        }
        self.enabled = true;
        Ok(())
    }

    /// Applies new wheel commands while enabled. A non-finite command puts both drivers in standby.
    pub fn update(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        if !self.enabled {
            return Err(Error::NotEnabled);
        }
        self.write_bridges(wheels)
    }

    /// Checks the whole vector before writing any bridge, so a bad fourth value cannot drive the
    /// first three.
    fn write_bridges(&mut self, wheels: Wheels) -> Result<(), Error<D::Error>> {
        if wheels.iter().any(|w| !w.is_finite()) {
            self.standby()?;
            return Err(Error::NonFinite);
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
        assert_eq!(
            mc.enable([0.5, 0.5, 0.5, f32::INFINITY]),
            Err(Error::NonFinite)
        );
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
        assert_eq!(mc.update([f32::NAN, 0.2, 0.2, 0.2]), Err(Error::NonFinite));
        assert_eq!(mc.driver.ops, STANDBY);
        assert_eq!(
            mc.update([0.2; 4]),
            Err(Error::NotEnabled),
            "only enable resumes"
        );
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
}
