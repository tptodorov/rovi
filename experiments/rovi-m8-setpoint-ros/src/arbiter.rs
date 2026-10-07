//! One owner, latest setpoint wins, explicit arming, lease and windows (ADR-0004).
//! Pure state machine: the caller passes the time in milliseconds and acts on the returned effects.

use crate::setpoint::Setpoint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Ble,
    Udp,
    Ros,
}

/// Who is talking: the source plus an id the adapter keeps stable for one controller
/// (BLE connection, UDP address, ROS publisher gid).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Claimant {
    pub source: Source,
    pub id: u128,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub lease_ms: u64,
    pub claim_window_ms: u64,
    pub reclaim_window_ms: u64,
    pub max_vx: f64,
    pub max_vy: f64,
    pub max_wz: f64,
}

impl Config {
    pub const DEFAULT: Config = Config {
        lease_ms: 1_000,
        claim_window_ms: 10_000,
        reclaim_window_ms: 5_000,
        max_vx: 1.0,
        max_vy: 1.0,
        max_wz: 3.0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Stop,
    LeaseExpired,
    OwnerLost,
    OutOfLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Someone else owns the device.
    Busy,
    NotOwner,
    /// Arming needs a valid lease, so a setpoint must have arrived recently.
    NotReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    Applied,
    /// The sequence was not newer than the last one; nothing changed.
    Stale,
}

/// What the firmware shell must do after a call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    /// Ownership was claimed: stop the other radios.
    pub claimed: Option<Source>,
    /// Ownership was released: reopen all radios.
    pub released: bool,
    /// A safety stop happened: wheels to zero, drivers to standby.
    pub stop: Option<StopReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    /// Owner claimed, not armed.
    Claimed,
    Armed,
    /// Safety stop latched, owner kept for the reclaim window.
    Stopped(StopReason),
}

pub struct Arbiter {
    cfg: Config,
    state: State,
    owner: Option<Claimant>,
    session: u32,
    last_seq: Option<u32>,
    latest: Option<Setpoint>,
    /// The lease is valid until this time; `None` before the first valid setpoint.
    lease_until: Option<u64>,
    /// Claim window end while there has been no setpoint or arm; reclaim window end when stopped.
    deadline: u64,
}

impl Arbiter {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            state: State::Open,
            owner: None,
            session: 0,
            last_seq: None,
            latest: None,
            lease_until: None,
            deadline: 0,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// The current owner's session, incremented on every claim. 0 before the first claim.
    pub fn session(&self) -> u32 {
        self.session
    }

    pub fn claim(&mut self, now: u64, who: Claimant) -> Result<Effects, Error> {
        match self.owner {
            Some(owner) if owner == who => Ok(Effects::default()),
            Some(_) => Err(Error::Busy),
            None => {
                self.session += 1;
                self.owner = Some(who);
                self.state = State::Claimed;
                self.last_seq = None;
                self.latest = None;
                self.lease_until = None;
                self.deadline = now + self.cfg.claim_window_ms;
                Ok(Effects {
                    claimed: Some(who.source),
                    ..Effects::default()
                })
            }
        }
    }

    pub fn arm(&mut self, now: u64, who: Claimant) -> Result<(), Error> {
        self.check_owner(who)?;
        if !self.lease_valid(now) {
            return Err(Error::NotReady);
        }
        self.state = State::Armed;
        Ok(())
    }

    pub fn setpoint(
        &mut self,
        now: u64,
        who: Claimant,
        seq: u32,
        sp: Setpoint,
    ) -> Result<(Applied, Effects), Error> {
        self.check_owner(who)?;
        if self.last_seq.is_some_and(|last| seq <= last) {
            return Ok((Applied::Stale, Effects::default()));
        }
        self.last_seq = Some(seq);
        let within = sp.vx.abs() <= self.cfg.max_vx
            && sp.vy.abs() <= self.cfg.max_vy
            && sp.wz.abs() <= self.cfg.max_wz;
        let fx = if !within {
            self.stop_with(now, StopReason::OutOfLimits)
        } else {
            self.latest = Some(sp);
            self.lease_until = Some(now + self.cfg.lease_ms);
            Effects::default()
        };
        Ok((Applied::Applied, fx))
    }

    pub fn stop(&mut self, now: u64, who: Claimant) -> Result<Effects, Error> {
        self.check_owner(who)?;
        Ok(self.stop_with(now, StopReason::Stop))
    }

    /// BYE or disconnect.
    pub fn release(&mut self, _now: u64, who: Claimant) -> Result<Effects, Error> {
        self.check_owner(who)?;
        let stop = (self.state == State::Armed).then_some(StopReason::OwnerLost);
        self.open();
        Ok(Effects {
            claimed: None,
            released: true,
            stop,
        })
    }

    /// Call every control tick (10 ms).
    pub fn tick(&mut self, now: u64) -> Effects {
        match self.state {
            State::Open => Effects::default(),
            State::Claimed if self.lease_until.is_none() => self.release_at(now),
            State::Claimed | State::Armed => match self.lease_until {
                Some(t) if now >= t => self.stop_with(now, StopReason::LeaseExpired),
                _ => Effects::default(),
            },
            State::Stopped(_) => self.release_at(now),
        }
    }

    /// The setpoint to apply now: only while armed with a valid lease.
    pub fn drive(&self, now: u64) -> Option<Setpoint> {
        (self.state == State::Armed && self.lease_valid(now))
            .then_some(self.latest)
            .flatten()
    }

    fn check_owner(&self, who: Claimant) -> Result<(), Error> {
        if self.owner == Some(who) {
            Ok(())
        } else {
            Err(Error::NotOwner)
        }
    }

    fn lease_valid(&self, now: u64) -> bool {
        self.lease_until.is_some_and(|t| now < t)
    }

    /// Latches a safety stop and keeps the owner for the reclaim window. No effect if already stopped.
    fn stop_with(&mut self, now: u64, reason: StopReason) -> Effects {
        if matches!(self.state, State::Stopped(_)) {
            return Effects::default();
        }
        self.state = State::Stopped(reason);
        self.latest = None;
        self.lease_until = None;
        self.deadline = now + self.cfg.reclaim_window_ms;
        Effects {
            stop: Some(reason),
            ..Effects::default()
        }
    }

    /// Ends the claim window or the reclaim window once its deadline passed.
    fn release_at(&mut self, now: u64) -> Effects {
        if now < self.deadline {
            return Effects::default();
        }
        self.open();
        Effects {
            released: true,
            ..Effects::default()
        }
    }

    fn open(&mut self) {
        self.state = State::Open;
        self.owner = None;
        self.latest = None;
        self.lease_until = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: Config = Config::DEFAULT;
    const PHONE: Claimant = Claimant {
        source: Source::Ble,
        id: 1,
    };
    const LAPTOP: Claimant = Claimant {
        source: Source::Udp,
        id: 2,
    };

    fn sp(vx: f64) -> Setpoint {
        Setpoint {
            sec: 0,
            nanosec: 0,
            vx,
            vy: 0.0,
            wz: 0.0,
        }
    }

    /// Claimed by PHONE at t=0, one setpoint at t=0, armed.
    fn armed() -> Arbiter {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        a.setpoint(0, PHONE, 1, sp(0.5)).unwrap();
        a.arm(0, PHONE).unwrap();
        a
    }

    #[test]
    fn claim_makes_owner_and_stops_other_radios() {
        let mut a = Arbiter::new(CFG);
        assert_eq!(a.state(), State::Open);
        let fx = a.claim(0, PHONE).unwrap();
        assert_eq!(
            fx,
            Effects {
                claimed: Some(Source::Ble),
                ..Default::default()
            }
        );
        assert_eq!(a.state(), State::Claimed);
        assert_eq!(a.session(), 1);
    }

    #[test]
    fn second_claimant_is_busy_and_same_claimant_is_idempotent() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        assert_eq!(a.claim(1, LAPTOP), Err(Error::Busy));
        assert_eq!(a.claim(1, PHONE).unwrap(), Effects::default());
        assert_eq!(a.session(), 1);
    }

    #[test]
    fn claim_window_expiry_releases() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        assert_eq!(a.tick(9_999), Effects::default());
        assert_eq!(
            a.tick(10_000),
            Effects {
                released: true,
                ..Default::default()
            }
        );
        assert_eq!(a.state(), State::Open);
    }

    #[test]
    fn setpoint_before_arm_renews_lease_but_does_not_drive() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        assert_eq!(
            a.setpoint(5, PHONE, 1, sp(0.5)).unwrap().0,
            Applied::Applied
        );
        assert_eq!(a.drive(5), None);
        a.arm(6, PHONE).unwrap();
        assert_eq!(a.state(), State::Armed);
        assert_eq!(a.drive(6), Some(sp(0.5)));
    }

    #[test]
    fn arm_needs_a_valid_lease() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        assert_eq!(a.arm(1, PHONE), Err(Error::NotReady));
        a.setpoint(1, PHONE, 1, sp(0.1)).unwrap();
        assert_eq!(
            a.arm(1_001, PHONE),
            Err(Error::NotReady),
            "lease of 1000 ms has lapsed"
        );
    }

    #[test]
    fn non_owner_cannot_arm_drive_or_stop() {
        let mut a = armed();
        assert_eq!(a.arm(1, LAPTOP), Err(Error::NotOwner));
        assert_eq!(
            a.setpoint(1, LAPTOP, 9, sp(1.0)).unwrap_err(),
            Error::NotOwner
        );
        assert_eq!(a.stop(1, LAPTOP), Err(Error::NotOwner));
        assert_eq!(a.drive(1), Some(sp(0.5)), "owner's setpoint is untouched");
    }

    #[test]
    fn latest_wins_and_old_sequences_never_apply() {
        let mut a = armed();
        assert_eq!(
            a.setpoint(10, PHONE, 2, sp(0.7)).unwrap().0,
            Applied::Applied
        );
        assert_eq!(a.setpoint(20, PHONE, 2, sp(0.9)).unwrap().0, Applied::Stale);
        assert_eq!(a.setpoint(20, PHONE, 1, sp(0.9)).unwrap().0, Applied::Stale);
        assert_eq!(a.drive(20), Some(sp(0.7)));
    }

    #[test]
    fn stale_setpoints_do_not_renew_the_lease() {
        let mut a = armed();
        a.setpoint(900, PHONE, 1, sp(0.9)).unwrap(); // stale seq
        assert_eq!(a.tick(1_000).stop, Some(StopReason::LeaseExpired));
    }

    #[test]
    fn valid_setpoints_renew_the_lease() {
        let mut a = armed();
        a.setpoint(900, PHONE, 2, sp(0.6)).unwrap();
        assert_eq!(a.tick(1_000), Effects::default());
        assert_eq!(a.drive(1_899), Some(sp(0.6)));
        assert_eq!(
            a.drive(1_900),
            None,
            "no motion on an expired lease, even before the tick"
        );
        assert_eq!(a.tick(1_900).stop, Some(StopReason::LeaseExpired));
    }

    #[test]
    fn lease_expiry_disarms_and_keeps_owner_for_reclaim_window() {
        let mut a = armed();
        assert_eq!(a.tick(1_000).stop, Some(StopReason::LeaseExpired));
        assert_eq!(a.state(), State::Stopped(StopReason::LeaseExpired));
        assert_eq!(a.drive(1_000), None);
        assert_eq!(a.claim(1_001, LAPTOP), Err(Error::Busy));
        // Reclaim: fresh setpoint, then arm again, without reconnecting.
        a.setpoint(2_000, PHONE, 2, sp(0.3)).unwrap();
        a.arm(2_000, PHONE).unwrap();
        assert_eq!(a.drive(2_000), Some(sp(0.3)));
    }

    #[test]
    fn reclaim_window_expiry_releases_ownership() {
        let mut a = armed();
        a.tick(1_000);
        assert_eq!(a.tick(5_999), Effects::default());
        assert_eq!(
            a.tick(6_000),
            Effects {
                released: true,
                ..Default::default()
            }
        );
        let fx = a.claim(6_001, LAPTOP).unwrap();
        assert_eq!(fx.claimed, Some(Source::Udp));
        assert_eq!(a.session(), 2, "a new session, so old input is never valid");
    }

    #[test]
    fn out_of_limits_setpoint_is_a_safety_stop() {
        let mut a = armed();
        let (_, fx) = a.setpoint(10, PHONE, 2, sp(CFG.max_vx + 0.1)).unwrap();
        assert_eq!(fx.stop, Some(StopReason::OutOfLimits));
        assert_eq!(a.state(), State::Stopped(StopReason::OutOfLimits));
        assert_eq!(a.drive(10), None);
    }

    #[test]
    fn stop_disarms_and_arm_again_needs_a_fresh_setpoint() {
        let mut a = armed();
        assert_eq!(a.stop(10, PHONE).unwrap().stop, Some(StopReason::Stop));
        assert_eq!(a.drive(10), None);
        assert_eq!(a.arm(11, PHONE), Err(Error::NotReady));
        a.setpoint(12, PHONE, 2, sp(0.2)).unwrap();
        a.arm(12, PHONE).unwrap();
        assert_eq!(a.drive(12), Some(sp(0.2)));
    }

    #[test]
    fn owner_release_while_armed_stops_then_opens() {
        let mut a = armed();
        let fx = a.release(10, PHONE).unwrap();
        assert_eq!(
            fx,
            Effects {
                claimed: None,
                released: true,
                stop: Some(StopReason::OwnerLost)
            }
        );
        assert_eq!(a.state(), State::Open);
        assert_eq!(a.release(11, PHONE), Err(Error::NotOwner));
    }

    #[test]
    fn silent_disarmed_owner_does_not_hold_the_device_forever() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        a.setpoint(100, PHONE, 1, sp(0.1)).unwrap();
        assert_eq!(a.tick(1_100).stop, Some(StopReason::LeaseExpired));
        assert!(a.tick(6_100).released);
    }
}
