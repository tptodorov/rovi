//! Wheel commands to TB6612FNG pins. Pure: the firmware passes the device state and the four
//! normalised wheel commands (FL, FR, RR, RL, -1..1) every control tick and writes the result.
//! Both drivers share one `STBY` line, high only while armed (ADR-0014).

use crate::arbiter::State;

/// One H-bridge channel: which wheel it drives and whether the motor is wired reversed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    /// 0 = FL, 1 = FR, 2 = RR, 3 = RL (the kinematics order).
    pub wheel: usize,
    pub invert: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// PWM duty in permille for the smallest command outside the deadband: the motors' starting
    /// duty (M4 step 2: about 35 % on the bench), so a gentle command still turns the wheel.
    pub min_duty_pm: u16,
    /// PWM duty in permille for a wheel command of 1.0. Caps speed and current.
    pub max_duty_pm: u16,
    /// Commands with a smaller magnitude coast (joystick drift, rounding).
    pub deadband: f32,
}

/// The pins of one channel: `IN1`, `IN2` and the PWM duty in permille.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bridge {
    pub in1: bool,
    pub in2: bool,
    pub duty_pm: u16,
}

impl Bridge {
    /// `IN1 = IN2 = L`, no PWM: outputs off, the wheel coasts.
    pub const OFF: Bridge = Bridge {
        in1: false,
        in2: false,
        duty_pm: 0,
    };
}

/// `STBY` and every channel's pins for this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output<const N: usize> {
    pub standby_high: bool,
    pub bridges: [Bridge; N],
}

/// Drivers are enabled only while armed with finite commands; otherwise `STBY` is low and every
/// bridge is off. A command inside the deadband coasts; outside it, duty runs linearly from
/// `min_duty_pm` (just above the deadband) to `max_duty_pm` (at 1.0).
pub fn output<const N: usize>(
    cfg: &Config,
    channels: &[Channel; N],
    state: State,
    wheels: [f32; 4],
) -> Output<N> {
    let off = Output {
        standby_high: false,
        bridges: [Bridge::OFF; N],
    };
    if state != State::Armed || wheels.iter().any(|w| !w.is_finite()) {
        return off;
    }
    let bridges = channels.map(|ch| {
        let w = wheels
            .get(ch.wheel)
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0);
        let w = if ch.invert { -w } else { w };
        let max = cfg.max_duty_pm.min(1000) as f32;
        let min = (cfg.min_duty_pm as f32).min(max);
        let duty_pm = if w == 0.0 || w.abs() < cfg.deadband {
            0
        } else {
            (min + w.abs() * (max - min) + 0.5) as u16
        };
        match duty_pm {
            0 => Bridge::OFF,
            _ => Bridge {
                in1: w > 0.0,
                in2: w < 0.0,
                duty_pm,
            },
        }
    });
    Output {
        standby_high: true,
        bridges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arbiter::StopReason;

    const CFG: Config = Config {
        min_duty_pm: 0,
        max_duty_pm: 400,
        deadband: 0.0,
    };
    /// The bench setting: motors need about 35 % to start (M4 step 2), full scale is 5 V.
    const BENCH: Config = Config {
        min_duty_pm: 400,
        max_duty_pm: 1000,
        deadband: 0.02,
    };
    const FL: [Channel; 1] = [Channel {
        wheel: 0,
        invert: false,
    }];

    #[test]
    fn standby_unless_armed() {
        for state in [
            State::Open,
            State::Claimed,
            State::Stopped(StopReason::Stop),
            State::Stopped(StopReason::LeaseExpired),
        ] {
            let out = output(&CFG, &FL, state, [1.0; 4]);
            assert!(!out.standby_high, "{state:?}");
            assert_eq!(out.bridges, [Bridge::OFF]);
        }
    }

    #[test]
    fn armed_forward_and_reverse_scale_to_the_duty_cap() {
        let out = output(&CFG, &FL, State::Armed, [0.5, 0.0, 0.0, 0.0]);
        assert!(out.standby_high);
        assert_eq!(
            out.bridges,
            [Bridge {
                in1: true,
                in2: false,
                duty_pm: 200
            }]
        );
        let out = output(&CFG, &FL, State::Armed, [-1.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            out.bridges,
            [Bridge {
                in1: false,
                in2: true,
                duty_pm: 400
            }]
        );
    }

    #[test]
    fn zero_coasts_with_the_driver_enabled() {
        let out = output(&CFG, &FL, State::Armed, [0.0; 4]);
        assert!(out.standby_high);
        assert_eq!(out.bridges, [Bridge::OFF]);
    }

    #[test]
    fn mapping_and_inversion_pick_the_wheel_and_sign() {
        let chans = [
            Channel {
                wheel: 2,
                invert: true,
            },
            Channel {
                wheel: 3,
                invert: false,
            },
        ];
        let out = output(&CFG, &chans, State::Armed, [0.0, 0.0, 0.5, -0.25]);
        assert_eq!(
            out.bridges,
            [
                Bridge {
                    in1: false,
                    in2: true,
                    duty_pm: 200
                },
                Bridge {
                    in1: false,
                    in2: true,
                    duty_pm: 100
                }
            ]
        );
    }

    #[test]
    fn out_of_range_is_clamped_and_non_finite_disables_everything() {
        let out = output(&CFG, &FL, State::Armed, [7.0, 0.0, 0.0, 0.0]);
        assert_eq!(out.bridges[0].duty_pm, 400);
        let out = output(&CFG, &FL, State::Armed, [0.5, f32::NAN, 0.0, 0.0]);
        assert!(!out.standby_high, "any NaN puts the drivers in standby");
        assert_eq!(out.bridges, [Bridge::OFF]);
    }

    #[test]
    fn duty_cap_never_exceeds_full_scale() {
        let cfg = Config {
            max_duty_pm: 5000,
            ..CFG
        };
        let out = output(&cfg, &FL, State::Armed, [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(out.bridges[0].duty_pm, 1000);
    }

    #[test]
    fn a_small_command_starts_at_the_minimum_duty() {
        let out = output(&BENCH, &FL, State::Armed, [0.05, 0.0, 0.0, 0.0]);
        assert_eq!(out.bridges[0].duty_pm, 430, "400 + 0.05 × 600");
        let out = output(&BENCH, &FL, State::Armed, [-0.05, 0.0, 0.0, 0.0]);
        assert_eq!((out.bridges[0].in2, out.bridges[0].duty_pm), (true, 430));
    }

    #[test]
    fn duty_scales_linearly_from_the_minimum_to_the_maximum() {
        let duty = |w| output(&BENCH, &FL, State::Armed, [w, 0.0, 0.0, 0.0]).bridges[0].duty_pm;
        assert_eq!(duty(0.5), 700);
        assert_eq!(duty(1.0), 1000);
    }

    #[test]
    fn commands_inside_the_deadband_coast() {
        let out = output(&BENCH, &FL, State::Armed, [0.01, 0.0, 0.0, 0.0]);
        assert!(out.standby_high);
        assert_eq!(out.bridges, [Bridge::OFF]);
    }

    #[test]
    fn a_minimum_above_the_maximum_is_capped_by_the_maximum() {
        let cfg = Config {
            min_duty_pm: 900,
            max_duty_pm: 500,
            deadband: 0.0,
        };
        let out = output(&cfg, &FL, State::Armed, [0.1, 0.0, 0.0, 0.0]);
        assert_eq!(out.bridges[0].duty_pm, 500);
    }

    #[test]
    fn zero_coasts_even_with_a_minimum_and_no_deadband() {
        let cfg = Config {
            deadband: 0.0,
            ..BENCH
        };
        let out = output(&cfg, &FL, State::Armed, [0.0; 4]);
        assert_eq!(out.bridges, [Bridge::OFF]);
    }
}
