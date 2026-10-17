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
    /// A malformed or unknown command.
    Invalid,
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
    /// Claim window end while there has been no setpoint or arm.
    deadline: u64,
    /// Reclaim window end. It starts when the lease lapses: at once for a lease expiry, and
    /// after the owner goes quiet for a STOP or an out-of-limits setpoint, which keep the owner
    /// for as long as it keeps streaming.
    reclaim_until: Option<u64>,
    /// After a STOP or an out-of-limits setpoint, arming needs a setpoint sent after it.
    fresh_needed: bool,
    /// Transitions into `Armed`, wrapping.
    arms: u32,
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
            reclaim_until: None,
            fresh_needed: false,
            arms: 0,
        }
    }

    /// How many times the device has been armed (wrapping). A repeated ARM while armed does not
    /// count, so a change means a new arming even if a stop came and went between two reads.
    pub fn arm_count(&self) -> u32 {
        self.arms
    }

    /// A fault below the device (motor control): latches an out-of-limits stop while owned, so
    /// resuming needs a fresh setpoint and an explicit arm, and ROS does not arm it on its own.
    pub fn fault_stop(&mut self, now: u64) -> Effects {
        match self.state {
            State::Open | State::Stopped(StopReason::OutOfLimits) => Effects::default(),
            // Already stopped for another reason: keep its lease and reclaim timing, but latch
            // the fault so no stream can arm it again.
            State::Stopped(_) => {
                self.state = State::Stopped(StopReason::OutOfLimits);
                self.fresh_needed = true;
                Effects {
                    stop: Some(StopReason::OutOfLimits),
                    ..Effects::default()
                }
            }
            State::Claimed | State::Armed => self.stop_with(now, StopReason::OutOfLimits),
        }
    }

    pub fn owner(&self) -> Option<Claimant> {
        self.owner
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
                self.reclaim_until = None;
                self.fresh_needed = false;
                Ok(Effects {
                    claimed: Some(who.source),
                    ..Effects::default()
                })
            }
        }
    }

    pub fn arm(&mut self, now: u64, who: Claimant) -> Result<(), Error> {
        self.check_owner(who)?;
        if !self.lease_valid(now) || self.fresh_needed {
            return Err(Error::NotReady);
        }
        self.reclaim_until = None;
        if self.state != State::Armed {
            self.arms = self.arms.wrapping_add(1);
        }
        self.state = State::Armed;
        Ok(())
    }

    /// A BLE/UDP setpoint with the session's u32 sequence: applied only if `seq` is newer than
    /// the last one in serial order (ahead by 1 to 2^31 - 1, so it survives the wrap).
    pub fn setpoint(
        &mut self,
        now: u64,
        who: Claimant,
        seq: u32,
        sp: Setpoint,
    ) -> Result<(Applied, Effects), Error> {
        self.check_owner(who)?;
        if self.last_seq.is_some_and(|last| !newer(seq, last)) {
            return Ok((Applied::Stale, Effects::default()));
        }
        self.last_seq = Some(seq);
        Ok((Applied::Applied, self.apply(now, sp)))
    }

    /// A setpoint the caller has already ordered (ROS: per publisher, `Device`). Samples from
    /// different publishers are taken in the order they arrive.
    pub fn setpoint_ordered(
        &mut self,
        now: u64,
        who: Claimant,
        sp: Setpoint,
    ) -> Result<Effects, Error> {
        self.check_owner(who)?;
        Ok(self.apply(now, sp))
    }

    fn apply(&mut self, now: u64, sp: Setpoint) -> Effects {
        if !self.within_limits(&sp) {
            self.stop_with(now, StopReason::OutOfLimits)
        } else {
            self.latest = Some(sp);
            self.lease_until = Some(now + self.cfg.lease_ms);
            self.fresh_needed = false;
            Effects::default()
        }
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
            State::Stopped(_) => self.reclaim_tick(now),
        }
    }

    /// The setpoint to apply now: only while armed with a valid lease.
    pub fn drive(&self, now: u64) -> Option<Setpoint> {
        (self.state == State::Armed && self.lease_valid(now))
            .then_some(self.latest)
            .flatten()
    }

    /// Is every component within its capability limit?
    pub fn within_limits(&self, sp: &Setpoint) -> bool {
        sp.vx.abs() <= self.cfg.max_vx
            && sp.vy.abs() <= self.cfg.max_vy
            && sp.wz.abs() <= self.cfg.max_wz
    }

    pub fn check_owner(&self, who: Claimant) -> Result<(), Error> {
        if self.owner == Some(who) {
            Ok(())
        } else {
            Err(Error::NotOwner)
        }
    }

    fn lease_valid(&self, now: u64) -> bool {
        self.lease_until.is_some_and(|t| now < t)
    }

    /// Latches a safety stop and keeps the owner. No effect if already stopped.
    fn stop_with(&mut self, now: u64, reason: StopReason) -> Effects {
        if matches!(self.state, State::Stopped(_)) {
            return Effects::default();
        }
        self.state = State::Stopped(reason);
        self.latest = None;
        match reason {
            // The lease is gone, so the owner has the reclaim window from now.
            StopReason::LeaseExpired | StopReason::OwnerLost => {
                self.lease_until = None;
                self.reclaim_until = Some(now + self.cfg.reclaim_window_ms);
            }
            // The owner is still there: the window starts only if its lease lapses.
            StopReason::Stop | StopReason::OutOfLimits => {
                self.reclaim_until = None;
                self.fresh_needed = true;
            }
        }
        Effects {
            stop: Some(reason),
            ..Effects::default()
        }
    }

    /// While stopped: release at the end of the reclaim window, and start the window once the
    /// lease has lapsed.
    fn reclaim_tick(&mut self, now: u64) -> Effects {
        match self.reclaim_until {
            Some(end) if now >= end => {
                self.open();
                Effects {
                    released: true,
                    ..Effects::default()
                }
            }
            Some(_) => Effects::default(),
            None => {
                if !self.lease_valid(now) {
                    self.reclaim_until = Some(now + self.cfg.reclaim_window_ms);
                }
                Effects::default()
            }
        }
    }

    /// Ends the claim window once its deadline passed.
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
        self.reclaim_until = None;
        self.fresh_needed = false;
    }
}

/// Serial number order for u32: `seq` is newer than `last` if it is ahead by 1 to 2^31 - 1.
/// Equal and exactly half the range away are not newer.
fn newer(seq: u32, last: u32) -> bool {
    (1..1 << 31).contains(&seq.wrapping_sub(last))
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

    #[test]
    fn stop_keeps_the_owner_while_it_keeps_streaming() {
        let mut a = armed();
        a.stop(10, PHONE).unwrap();
        // The owner streams on, so no reclaim window runs and it is never released.
        for (seq, t) in (2..).zip((500..60_000).step_by(500)) {
            a.setpoint(t, PHONE, seq, sp(0.1)).unwrap();
            assert_eq!(a.tick(t), Effects::default());
        }
        assert_eq!(a.state(), State::Stopped(StopReason::Stop));
        assert_eq!(a.claim(60_000, LAPTOP), Err(Error::Busy));
    }

    #[test]
    fn reclaim_window_starts_when_the_lease_lapses_after_a_stop() {
        let mut a = armed(); // the lease is valid until 1000
        a.stop(10, PHONE).unwrap();
        assert_eq!(a.tick(999), Effects::default());
        assert_eq!(
            a.tick(1_000),
            Effects::default(),
            "the lease lapses here and the window starts"
        );
        assert_eq!(a.tick(5_999), Effects::default());
        assert_eq!(
            a.tick(6_000),
            Effects {
                released: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn out_of_limits_behaves_like_stop() {
        let mut a = armed();
        a.setpoint(10, PHONE, 2, sp(CFG.max_vx + 0.1)).unwrap();
        assert_eq!(a.state(), State::Stopped(StopReason::OutOfLimits));
        a.setpoint(900, PHONE, 3, sp(0.1)).unwrap(); // a valid setpoint renews the lease
        assert_eq!(a.tick(1_000), Effects::default());
        assert_eq!(
            a.tick(1_900),
            Effects::default(),
            "the lease lapses, the window starts"
        );
        assert_eq!(a.tick(6_899), Effects::default());
        assert_eq!(
            a.tick(6_900),
            Effects {
                released: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn arming_again_after_a_stop_needs_a_setpoint_sent_after_it() {
        let mut a = armed(); // an older setpoint still holds the lease
        a.stop(10, PHONE).unwrap();
        assert_eq!(a.arm(11, PHONE), Err(Error::NotReady));
        a.setpoint(12, PHONE, 2, sp(0.2)).unwrap();
        a.arm(12, PHONE).unwrap();
        assert_eq!(a.drive(12), Some(sp(0.2)));
    }

    #[test]
    fn arm_count_counts_each_arming_once() {
        let mut a = armed();
        assert_eq!(a.arm_count(), 1);
        a.arm(1, PHONE).unwrap();
        assert_eq!(
            a.arm_count(),
            1,
            "a repeated ARM while armed is not a new arming"
        );
        a.stop(2, PHONE).unwrap();
        a.setpoint(3, PHONE, 2, sp(0.5)).unwrap();
        a.arm(3, PHONE).unwrap();
        assert_eq!(a.arm_count(), 2, "stop and re-arm is a new arming");
    }

    #[test]
    fn a_fault_stop_latches_out_of_limits_until_a_fresh_setpoint_and_arm() {
        let mut a = armed();
        assert_eq!(a.fault_stop(10).stop, Some(StopReason::OutOfLimits));
        assert_eq!(a.state(), State::Stopped(StopReason::OutOfLimits));
        assert_eq!(a.drive(10), None);
        assert_eq!(
            a.arm(20, PHONE),
            Err(Error::NotReady),
            "needs a fresh setpoint"
        );
        a.setpoint(30, PHONE, 2, sp(0.5)).unwrap();
        assert_eq!(a.arm(30, PHONE), Ok(()));
        assert_eq!(a.fault_stop(40).stop, Some(StopReason::OutOfLimits));
        assert_eq!(a.fault_stop(50), Effects::default(), "already latched");
    }

    #[test]
    fn a_fault_stop_overrides_an_earlier_stop_reason() {
        let mut a = armed();
        assert_eq!(a.tick(1_000).stop, Some(StopReason::LeaseExpired));
        assert_eq!(a.fault_stop(1_010).stop, Some(StopReason::OutOfLimits));
        assert_eq!(a.state(), State::Stopped(StopReason::OutOfLimits));
    }

    #[test]
    fn a_fault_stop_without_an_owner_does_nothing() {
        let mut a = Arbiter::new(CFG);
        assert_eq!(a.fault_stop(0), Effects::default());
        assert_eq!(a.state(), State::Open);
    }

    // Deadline boundaries: the deadline itself counts as passed (`now >= deadline`), and a tick
    // after it changes nothing more. With the 10 ms control tick that is within lease + 10 ms.

    #[test]
    fn claim_window_boundary() {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        assert!(!a.tick(9_999).released, "deadline - 1");
        assert!(a.tick(10_000).released, "deadline");
        assert_eq!(a.tick(10_001), Effects::default(), "deadline + 1");
        assert_eq!(a.state(), State::Open);
    }

    #[test]
    fn lease_boundary() {
        let mut a = armed(); // setpoint at 0, lease 1000 ms
        assert_eq!(a.tick(999).stop, None, "deadline - 1");
        assert_eq!(a.drive(999).map(|s| s.vx), Some(0.5));
        assert_eq!(
            a.tick(1_000).stop,
            Some(StopReason::LeaseExpired),
            "deadline"
        );
        assert_eq!(a.drive(1_000), None);
        assert_eq!(a.tick(1_001), Effects::default(), "deadline + 1");
    }

    #[test]
    fn reclaim_window_boundary_after_a_lease_expiry() {
        let mut a = armed();
        a.tick(1_000); // the window starts at once: until 6000
        assert!(!a.tick(5_999).released, "deadline - 1");
        assert!(a.tick(6_000).released, "deadline");
        assert_eq!(a.tick(6_001), Effects::default(), "deadline + 1");
    }

    #[test]
    fn reclaim_window_boundary_after_a_quiet_owners_stop() {
        let mut a = armed();
        a.stop(10, PHONE).unwrap(); // the owner is quiet; its lease lapses at 1000
        assert!(!a.tick(999).released);
        a.tick(1_000); // the window starts when the lease lapses: until 6000
        assert!(!a.tick(5_999).released, "deadline - 1");
        assert!(a.tick(6_000).released, "deadline");
        assert_eq!(a.tick(6_001), Effects::default(), "deadline + 1");
    }

    // Sequence domains: BLE/UDP use u32 serial ordering (RFC 1982 style).

    /// Is `seq` applied after `last`? The arbiter is armed, so an applied setpoint drives.
    fn applies(last: u32, seq: u32) -> bool {
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        a.setpoint(0, PHONE, last, sp(0.1)).unwrap();
        a.setpoint(1, PHONE, seq, sp(0.2)).unwrap().0 == Applied::Applied
    }

    #[test]
    fn a_wrapped_sequence_is_newer() {
        assert!(applies(u32::MAX, 0), "0 follows 4294967295");
        let mut a = Arbiter::new(CFG);
        a.claim(0, PHONE).unwrap();
        a.setpoint(0, PHONE, u32::MAX, sp(0.1)).unwrap();
        a.setpoint(1, PHONE, 0, sp(0.2)).unwrap();
        assert_eq!(
            a.setpoint(2, PHONE, u32::MAX, sp(0.3)).unwrap().0,
            Applied::Stale
        );
    }

    #[test]
    fn newer_means_up_to_half_the_range_ahead() {
        assert!(applies(0, 1));
        assert!(applies(0, (1 << 31) - 1), "half range - 1 is newer");
        assert!(!applies(0, 1 << 31), "exactly half range is dropped");
        assert!(!applies(0, u32::MAX), "behind by one");
        assert!(!applies(7, 7), "equal");
        assert!(applies(u32::MAX - 5, 10), "across the wrap");
    }
}
