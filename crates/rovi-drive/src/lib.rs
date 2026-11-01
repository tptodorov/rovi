//! The coordinator (M6 design): the only caller of motor control. Each control tick it runs the
//! device's timers, then drives the motors from the device state:
//!
//! * a new arming (the arbiter's [`arm_count`](rovi_core::arbiter::Arbiter::arm_count) changed)
//!   is the only thing that enables the motors, so a STOP and re-ARM between two ticks still
//!   drops STBY first;
//! * while armed, the wheel demands update the bridges;
//! * otherwise both drivers go to standby in that tick.
//!
//! Any motor error puts the drivers in standby and raises the device's out-of-limits stop: the
//! owner keeps the car, and only a fresh setpoint plus an explicit ARM (a new arming) enables the
//! motors again. Repeated ticks and a ROS stream do not.
#![no_std]

#[cfg(test)]
extern crate std;

use rovi_core::{arbiter::Effects, device::Device, events::Fault};
use rovi_motor_control::{Driver, Error, Fault as MotorFault, MotorControl};
use rovi_protocol::State;

#[cfg(feature = "motor-bench")]
pub mod bench;

/// What one control tick did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    /// The device's effects, including the out-of-limits stop a motor fault raises.
    pub effects: Effects,
    /// A motor fault in this tick, for the log (`ev=error src=motor_*`). The typed detail is in
    /// [`MotorControl::fault`] through [`Drive::motor`].
    pub fault: Option<Fault>,
}

pub struct Drive<D> {
    motor: MotorControl<D>,
    /// The arming the motors were last enabled for.
    armed_for: Option<u32>,
}

impl<D: Driver> Drive<D> {
    /// Bind before any transport input, so bad configuration refuses even the first ARM.
    pub fn new(motor: MotorControl<D>, dev: &mut Device) -> Self {
        dev.set_motor_ready(!matches!(motor.fault(), Some(MotorFault::Config(_))));
        Self {
            motor,
            armed_for: None,
        }
    }

    pub fn motor(&self) -> &MotorControl<D> {
        &self.motor
    }

    /// One control tick (every 10 ms): the device's timers first, so a lease expiry reaches the
    /// pins in the same tick, then the motors.
    pub fn tick(&mut self, dev: &mut Device, now: u64) -> Tick {
        self.tick_with(dev, now, Device::wheels)
    }

    /// Bench image only: fixed full-scale vectors, retaining the normal safety coordinator.
    #[cfg(feature = "motor-bench")]
    pub fn tick_motor_bench(&mut self, dev: &mut Device, now: u64) -> Tick {
        self.tick_with(dev, now, bench::wheels)
    }

    /// [`tick`](Self::tick) with the wheel demands from `wheels`: the tests inject faults the
    /// device's validated profile cannot produce.
    fn tick_with(
        &mut self,
        dev: &mut Device,
        now: u64,
        wheels: fn(&Device, u64) -> [f32; 4],
    ) -> Tick {
        let mut effects = dev.tick(now);
        let fault = self.apply(dev, now, wheels);
        if fault.is_some() {
            // Best effort: the drivers may already be in standby, and the fault is reported anyway.
            let _ = self.motor.standby();
            if let Some(reason) = dev.on_motor_fault(now).stop {
                effects.stop = Some(reason);
                effects.lease_late_ms = None;
            }
        }
        Tick { effects, fault }
    }

    fn apply(
        &mut self,
        dev: &Device,
        now: u64,
        wheels: fn(&Device, u64) -> [f32; 4],
    ) -> Option<Fault> {
        let arbiter = dev.arbiter();
        let result = if arbiter.state() != State::Armed {
            if !self.motor.is_enabled() {
                return None;
            }
            self.motor.standby()
        } else if self.armed_for != Some(arbiter.arm_count()) {
            // Recorded before enabling: a failed enable is not retried on the next tick.
            self.armed_for = Some(arbiter.arm_count());
            self.motor.enable(wheels(dev, now))
        } else if self.motor.is_enabled() {
            self.motor.update(wheels(dev, now))
        } else {
            // Armed, but a fault holds the motors off until the next arming.
            return None;
        };
        match result.err()? {
            Error::Fault(MotorFault::NonFinite) => Some(Fault::MotorNonFinite),
            Error::Fault(MotorFault::Config(_)) => Some(Fault::MotorConfig),
            Error::Driver(_) => Some(Fault::MotorDriver),
            // Only updated while enabled.
            Error::NotEnabled => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rovi_core::{arbiter::Config, kinematics::Geometry};
    use rovi_motor_control::{Config as MotorConfig, Pins, CAR_CHANNELS};
    use rovi_protocol::{
        ros::{gid, Attachment},
        setpoint::Setpoint,
        udp::{self, Header, Kind},
        StopReason,
    };
    use std::{cell::RefCell, format, rc::Rc, vec::Vec};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Op {
        Stby(usize, bool),
        Bridge(usize, Pins),
    }

    #[derive(Default)]
    struct Log {
        ops: Vec<Op>,
        fail_bridge: bool,
        fail_stby: bool,
    }

    /// Records every pin write into a log the test shares; `fail_bridge` makes bridge writes fail.
    #[derive(Clone, Default)]
    struct Recording(Rc<RefCell<Log>>);

    impl Driver for Recording {
        type Error = ();
        fn set_stby(&mut self, driver: usize, high: bool) -> Result<(), ()> {
            if self.0.borrow().fail_stby {
                return Err(());
            }
            self.0.borrow_mut().ops.push(Op::Stby(driver, high));
            Ok(())
        }
        fn set_bridge(&mut self, bridge: usize, pins: Pins) -> Result<(), ()> {
            let mut log = self.0.borrow_mut();
            if log.fail_bridge {
                return Err(());
            }
            log.ops.push(Op::Bridge(bridge, pins));
            Ok(())
        }
    }

    const STANDBY: [Op; 2] = [Op::Stby(0, false), Op::Stby(1, false)];
    const ENABLE: [Op; 2] = [Op::Stby(0, true), Op::Stby(1, true)];
    const CAR: Geometry = Geometry::CAR;
    const PEER: u128 = 0xA;

    struct Car {
        dev: Device,
        drive: Drive<Recording>,
        log: Recording,
        session: u32,
        seq: u32,
    }

    impl Car {
        fn new(geometry: Geometry, motor: MotorConfig) -> Self {
            let log = Recording::default();
            let mc = MotorControl::new(log.clone(), motor, CAR_CHANNELS).unwrap();
            let mut dev = Device::new(Config::DEFAULT, geometry, 0).unwrap();
            let drive = Drive::new(mc, &mut dev);
            let mut car = Car {
                dev,
                drive,
                log,
                session: 0,
                seq: 0,
            };
            car.take_ops();
            car
        }

        fn ops(&self) -> Vec<Op> {
            self.log.0.borrow().ops.clone()
        }

        fn take_ops(&mut self) -> Vec<Op> {
            core::mem::take(&mut self.log.0.borrow_mut().ops)
        }

        fn fail(&self, on: bool) {
            self.log.0.borrow_mut().fail_bridge = on;
        }

        fn udp(&mut self, now: u64, kind: Kind, payload: &[u8]) -> Effects {
            self.seq += 1;
            let mut b = [0u8; 96];
            let h = Header {
                kind,
                session: self.session,
                seq: self.seq,
            };
            let n = udp::write(&mut b, h, payload).unwrap();
            let mut out = [0u8; 64];
            let r = self.dev.on_udp(now, PEER, &b[..n], &mut out);
            if kind == Kind::Hello {
                self.session = udp::parse(&out[..r.len]).unwrap().0.session;
            }
            r.effects
        }

        fn setpoint(&mut self, now: u64, vx: f64) {
            self.udp(now, Kind::Setpoint, &cdr(vx));
        }

        /// HELLO, a setpoint and ARM over UDP at `now`.
        fn arm_udp(&mut self, now: u64) {
            self.udp(now, Kind::Hello, &[]);
            self.setpoint(now, 0.5);
            self.udp(now, Kind::Arm, &[]);
            assert_eq!(self.dev.arbiter().state(), State::Armed);
        }

        fn tick(&mut self, now: u64) -> Tick {
            self.drive.tick(&mut self.dev, now)
        }

        fn stby_high(&self) -> bool {
            self.ops().iter().any(|op| matches!(op, Op::Stby(_, true)))
        }

        fn bridge_writes(&self) -> usize {
            self.ops()
                .iter()
                .filter(|op| matches!(op, Op::Bridge(..)))
                .count()
        }
    }

    fn cdr(vx: f64) -> [u8; 68] {
        let mut b = [0u8; 68];
        Setpoint {
            sec: 0,
            nanosec: 0,
            vx,
            vy: 0.0,
            wz: 0.0,
        }
        .encode(&mut b)
        .unwrap();
        b
    }

    /// A ROS publisher of node 1 (`id` 1 = cmd_vel, 2 = arm) known from its liveliness token.
    fn publisher(dev: &mut Device, id: u8) -> [u8; 33] {
        let key = format!(
            "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/1/{id}/MP/%/%/n1/%t{id}/std_msgs::msg::dds_::Bool_/RIHS01_x/::,10:,:,:,,"
        );
        dev.on_ros_liveliness(&key, true);
        Attachment {
            seq: 1,
            timestamp: 0,
            gid: gid(&key),
        }
        .encode()
    }

    fn at(att: &[u8; 33], seq: i64) -> [u8; 33] {
        let mut a = *att;
        a[..8].copy_from_slice(&seq.to_le_bytes());
        a
    }

    #[test]
    fn an_arming_enables_the_motors_once() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        let ops = car.take_ops();
        assert_eq!(ops[..2], STANDBY, "standby before the bridges are prepared");
        assert_eq!(
            ops[2..6]
                .iter()
                .filter(|op| matches!(op, Op::Bridge(..)))
                .count(),
            4
        );
        assert_eq!(ops[6..], ENABLE);
        car.take_ops();
        car.tick(20);
        assert_eq!(car.bridge_writes(), 4, "later ticks update the bridges");
        assert!(
            car.ops().iter().all(|op| matches!(op, Op::Bridge(..))),
            "no STBY change"
        );
    }

    #[test]
    fn a_lease_expiry_puts_both_drivers_in_standby_in_the_same_tick() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        car.take_ops();
        let t = car.tick(1_000);
        assert_eq!(t.effects.stop, Some(StopReason::LeaseExpired));
        assert_eq!(car.ops(), STANDBY, "standby, and no bridge update after it");
        car.take_ops();
        car.tick(1_010);
        assert!(car.ops().is_empty(), "nothing more while stopped");
    }

    #[test]
    fn a_stop_and_rearm_between_ticks_still_drops_stby_first() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        car.udp(12, Kind::Stop, &[]);
        car.setpoint(14, 0.5);
        car.udp(16, Kind::Arm, &[]);
        car.take_ops();
        car.tick(20);
        let ops = car.take_ops();
        assert_eq!(
            ops[..2],
            STANDBY,
            "the stop reaches the pins even though it is armed again"
        );
        assert_eq!(ops[6..], ENABLE);
    }

    #[test]
    fn a_release_puts_both_drivers_in_standby() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        car.udp(12, Kind::Bye, &[]);
        car.take_ops();
        car.tick(20);
        assert_eq!(car.ops(), STANDBY);
    }

    #[test]
    fn a_standby_failure_overriding_a_lease_stop_does_not_report_lease_lateness() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        car.log.0.borrow_mut().fail_stby = true;
        let tick = car.tick(1_051);
        assert_eq!(tick.fault, Some(Fault::MotorDriver));
        assert_eq!(tick.effects.stop, Some(StopReason::OutOfLimits));
        assert_eq!(tick.effects.lease_late_ms, None);
    }

    #[test]
    fn a_non_finite_wheel_demand_stops_the_device_and_no_tick_recovers_it() {
        // Injected: a validated device profile cannot produce a NaN demand.
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        let t = car.drive.tick_with(&mut car.dev, 10, |_, _| [f32::NAN; 4]);
        assert_eq!(t.fault, Some(Fault::MotorNonFinite));
        assert_eq!(t.effects.stop, Some(StopReason::OutOfLimits));
        assert_eq!(
            car.dev.arbiter().state(),
            State::Stopped(StopReason::OutOfLimits)
        );
        assert!(
            !car.stby_high() && car.bridge_writes() == 0,
            "{:?}",
            car.ops()
        );
        for i in 2..100 {
            car.setpoint(i * 10, 0.5);
            assert_eq!(car.tick(i * 10 + 5).fault, None);
        }
        assert!(
            !car.stby_high() && car.bridge_writes() == 0,
            "no tick re-enables"
        );
    }

    #[test]
    fn an_invalid_motor_configuration_never_enables() {
        let bad = MotorConfig {
            min_duty_pm: 800,
            ..MotorConfig::PRODUCT
        };
        let mut car = Car::new(CAR, bad);
        car.udp(0, Kind::Hello, &[]);
        car.setpoint(0, 0.5);
        car.udp(0, Kind::Arm, &[]);
        assert_eq!(
            car.dev.status(),
            [1, 0, 3],
            "ARM must refuse before the first tick"
        );
        for i in 1..10 {
            car.setpoint(i * 10, 0.5);
            car.udp(i * 10, Kind::Arm, &[]);
            car.tick(i * 10 + 1);
            assert_eq!(car.dev.status(), [1, 0, 3]);
        }
        assert!(!car.stby_high() && car.bridge_writes() == 0);
        car.udp(101, Kind::Bye, &[]);
        car.udp(102, Kind::Hello, &[]);
        car.setpoint(102, 0.5);
        car.udp(102, Kind::Arm, &[]);
        assert_eq!(
            car.dev.status(),
            [1, 0, 3],
            "a new session cannot fix configuration"
        );
    }

    #[test]
    fn an_invalid_motor_configuration_refuses_ble_arm_before_any_tick() {
        let bad = MotorConfig {
            deadband: f32::NAN,
            ..MotorConfig::PRODUCT
        };
        let mut car = Car::new(CAR, bad);
        let c = car.dev.on_ble_connect(0, 1).unwrap().0;
        let mut sp = [0; 72];
        sp[..4].copy_from_slice(&1u32.to_le_bytes());
        sp[4..].copy_from_slice(&cdr(0.5));
        car.dev.on_ble_setpoint(1, c, &sp);
        assert_eq!(
            car.dev.on_ble_control(2, c, 1),
            Err(rovi_protocol::Error::NotReady)
        );
        assert_eq!(car.dev.ble_status(), [1, 0, 3]);
        car.tick(10);
        assert!(!car.stby_high() && car.bridge_writes() == 0);
    }

    #[test]
    fn an_invalid_motor_configuration_blocks_ros_auto_and_explicit_arm() {
        let bad = MotorConfig {
            max_duty_pm: 1_001,
            ..MotorConfig::PRODUCT
        };
        let mut car = Car::new(CAR, bad);
        let (cmd, arm) = (publisher(&mut car.dev, 1), publisher(&mut car.dev, 2));
        car.dev.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        assert_eq!(
            car.dev.status(),
            [1, 0, 0],
            "no automatic arm before the tick"
        );
        car.dev.on_ros_arm(1, &arm, &[0, 1, 0, 0, 1]);
        assert_eq!(car.dev.status(), [1, 0, 3]);
        for i in 2..10 {
            car.dev
                .on_ros_cmd_vel(i * 10, &at(&cmd, i as i64), &cdr(0.5));
            car.tick(i * 10 + 1);
            assert_eq!(car.dev.status(), [1, 0, 3]);
        }
        assert!(!car.stby_high() && car.bridge_writes() == 0);
    }

    #[test]
    fn a_driver_failure_stops_and_only_a_fresh_setpoint_and_arm_recover() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.arm_udp(0);
        car.tick(10);
        car.fail(true);
        let t = car.tick(20);
        assert_eq!(t.fault, Some(Fault::MotorDriver));
        assert_eq!(t.effects.stop, Some(StopReason::OutOfLimits));
        assert_eq!(
            car.ops()[car.ops().len() - 2..],
            STANDBY,
            "standby after the failure"
        );
        assert!(!car.drive.motor().is_enabled());

        car.fail(false);
        car.take_ops();
        for i in 3..20 {
            car.tick(i * 10);
        }
        car.udp(200, Kind::Arm, &[]);
        car.tick(210);
        assert!(
            car.ops().is_empty(),
            "ticks and an ARM without a fresh setpoint do nothing"
        );

        car.setpoint(220, 0.5);
        car.udp(220, Kind::Arm, &[]);
        car.tick(230);
        let ops = car.take_ops();
        assert_eq!(ops[..2], STANDBY);
        assert_eq!(ops[6..], ENABLE, "a fresh setpoint and ARM enable again");
    }

    #[test]
    fn a_ros_stream_does_not_recover_a_motor_fault_but_an_explicit_arm_does() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        let (cmd, arm) = (publisher(&mut car.dev, 1), publisher(&mut car.dev, 2));
        car.dev.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        car.tick(10);
        assert!(car.stby_high(), "ROS armed on its own");

        car.fail(true);
        assert_eq!(car.tick(20).fault, Some(Fault::MotorDriver));
        car.fail(false);
        car.take_ops();
        for i in 3..30 {
            car.dev
                .on_ros_cmd_vel(i * 10, &at(&cmd, i as i64), &cdr(0.5));
            car.tick(i * 10 + 5);
        }
        assert!(
            car.ops().is_empty(),
            "the stream keeps the lease but does not arm"
        );

        car.dev.on_ros_arm(300, &at(&arm, 2), &[0, 1, 0, 0, 1]);
        car.tick(305);
        assert_eq!(
            car.ops()[6..],
            ENABLE,
            "an explicit arm after a fresh setpoint resumes"
        );
    }

    #[cfg(feature = "motor-bench")]
    #[test]
    fn bench_reverses_all_four_at_700_without_standby_or_session_change() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.udp(0, Kind::Hello, &[]);
        car.setpoint(0, 0.1);
        car.udp(0, Kind::Arm, &[]);
        let session = car.session;
        assert_eq!(car.drive.tick_motor_bench(&mut car.dev, 0).fault, None);
        assert_eq!(bench::wheels(&car.dev, 0), [1.0; 4]);
        assert_eq!(car.ops()[6..], ENABLE);
        for i in 1..=10 {
            car.take_ops();
            let reverse = i % 2 == 1;
            let at = i * 10;
            car.setpoint(at, if reverse { -0.1 } else { 0.1 });
            let tick = car.drive.tick_motor_bench(&mut car.dev, at);
            assert_eq!(tick.fault, None);
            car.udp(at, Kind::Hello, &[]);
            assert_eq!(car.session, session);
            assert_eq!(car.dev.arbiter().state(), State::Armed);
            assert_eq!(
                car.ops(),
                (0..4)
                    .map(|b| Op::Bridge(
                        b,
                        Pins {
                            in1: !reverse,
                            in2: reverse,
                            pwm_pm: 700,
                        }
                    ))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[cfg(feature = "motor-bench")]
    #[test]
    fn bench_zero_and_other_demands_coast_and_normal_tick_does_not_scale() {
        let mut car = Car::new(CAR, MotorConfig::PRODUCT);
        car.udp(0, Kind::Hello, &[]);
        car.setpoint(0, 0.1);
        car.drive.tick_motor_bench(&mut car.dev, 0);
        assert!(
            car.ops().is_empty(),
            "a marker alone must not enable motors"
        );
        car.udp(0, Kind::Arm, &[]);
        car.tick(0);
        assert!(car.ops().contains(&Op::Bridge(
            0,
            Pins {
                in1: true,
                in2: false,
                pwm_pm: 425
            }
        )));
        for vx in [0.0, 0.5, -0.5, 1.0] {
            car.setpoint(10, vx);
            car.take_ops();
            car.drive.tick_motor_bench(&mut car.dev, 10);
            assert_eq!(bench::wheels(&car.dev, 10), [0.0; 4]);
            assert_eq!(
                car.ops(),
                (0..4)
                    .map(|b| Op::Bridge(b, Pins::COAST))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[cfg(feature = "motor-bench")]
    #[test]
    fn bench_keeps_stop_lease_release_and_motor_fault_safety_paths() {
        for mode in [0, 1, 2, 3] {
            let mut car = Car::new(CAR, MotorConfig::PRODUCT);
            car.udp(0, Kind::Hello, &[]);
            car.setpoint(0, 0.1);
            car.udp(0, Kind::Arm, &[]);
            car.drive.tick_motor_bench(&mut car.dev, 0);
            car.take_ops();
            match mode {
                0 => {
                    car.udp(10, Kind::Stop, &[]);
                }
                1 => {} // Expiry is processed in the bench tick, just as in the product tick.
                2 => {
                    car.udp(10, Kind::Bye, &[]);
                }
                _ => car.fail(true),
            }
            let tick = car
                .drive
                .tick_motor_bench(&mut car.dev, if mode == 3 { 10 } else { 1000 });
            assert!(!car.drive.motor().is_enabled(), "mode {mode}");
            assert_eq!(car.ops()[car.ops().len() - 2..], STANDBY);
            if mode == 1 {
                assert_eq!(tick.effects.stop, Some(StopReason::LeaseExpired));
            }
            if mode == 3 {
                assert_eq!(tick.fault, Some(Fault::MotorDriver));
            }
            car.fail(false);
            car.take_ops();
            car.setpoint(1010, 0.1);
            car.drive.tick_motor_bench(&mut car.dev, 1010);
            assert!(
                car.ops().is_empty(),
                "a fresh stream cannot re-enable mode {mode}"
            );
        }
    }
}
