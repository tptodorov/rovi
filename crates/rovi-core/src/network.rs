//! Network mode: home (station + ROS router) or fallback (the car AP, today's behaviour).
//!
//! Pure and clock-driven, like the arbiter. The firmware runs each home-mode step (scan, join,
//! DHCP, router) and reports its outcome; this module decides the mode, the budgets and what a
//! mode command or the end of an ownership does (`openspec/specs/network-mode`).

/// Longest scan for the configured network before falling back.
pub const SCAN_BUDGET_MS: u64 = 1_500;
/// Join + DHCP + router, counted from the moment the network is found.
pub const JOIN_BUDGET_MS: u64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Attempting(Step),
    Home,
    Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Scan,
    Join,
    Dhcp,
    Router,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    None,
    NoSettings,
    Scan,
    Join,
    Dhcp,
    Router,
    LinkLost,
    Requested,
}

/// A mode command: pending until the commanding owner's ownership ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Home,
    CarNetwork,
}

/// Why a home-mode attempt started; decides the fallback reason when it fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cause {
    Boot,
    Requested,
    BleEnded,
    LinkLost,
}

/// What every transport reports about the network: BLE Network characteristic, UDP `INFO`,
/// ROS `/rovi/network`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub mode: Mode,
    pub pending: Option<Command>,
    pub reason: Reason,
    /// Station IPv4; 0.0.0.0 without one.
    pub ip: [u8; 4],
}

impl Info {
    /// `[mode, pending, reason, 0, ip0, ip1, ip2, ip3]`.
    pub fn encode(self) -> [u8; 8] {
        let mode = match self.mode {
            Mode::Fallback => 0,
            Mode::Attempting(_) => 1,
            Mode::Home => 2,
        };
        let pending = match self.pending {
            None => 0,
            Some(Command::Home) => 1,
            Some(Command::CarNetwork) => 2,
        };
        let [a, b, c, d] = self.ip;
        [mode, pending, reason_code(self.reason), 0, a, b, c, d]
    }

    /// The `/rovi/network` sample: a CDR `std_msgs/String` in logfmt.
    pub fn encode_cdr(self, out: &mut [u8]) -> Option<usize> {
        use core::fmt::Write;
        let mode = match self.mode {
            Mode::Fallback => "fallback",
            Mode::Attempting(_) => "attempting",
            Mode::Home => "home",
        };
        let pending = match self.pending {
            None => "none",
            Some(Command::Home) => "home",
            Some(Command::CarNetwork) => "car-network",
        };
        let [a, b, c, d] = self.ip;
        let mut text = crate::rmw::Key::<96>::new();
        write!(
            text,
            "mode={mode} pending={pending} fallback={} ip={a}.{b}.{c}.{d}",
            reason_name(self.reason)
        )
        .ok()?;
        let bytes = text.as_str().as_bytes();
        let n = 9 + bytes.len();
        let out = out.get_mut(..n)?;
        out[..4].copy_from_slice(&[0, 1, 0, 0]);
        out[4..8].copy_from_slice(&(bytes.len() as u32 + 1).to_le_bytes());
        out[8..n - 1].copy_from_slice(bytes);
        out[n - 1] = 0;
        Some(n)
    }
}

fn reason_code(r: Reason) -> u8 {
    match r {
        Reason::None => 0,
        Reason::NoSettings => 1,
        Reason::Scan => 2,
        Reason::Join => 3,
        Reason::Dhcp => 4,
        Reason::Router => 5,
        Reason::LinkLost => 6,
        Reason::Requested => 7,
    }
}

fn reason_name(r: Reason) -> &'static str {
    match r {
        Reason::None => "none",
        Reason::NoSettings => "no-settings",
        Reason::Scan => "scan",
        Reason::Join => "join",
        Reason::Dhcp => "dhcp",
        Reason::Router => "router",
        Reason::LinkLost => "link-lost",
        Reason::Requested => "requested",
    }
}

#[derive(Debug)]
pub struct Network {
    /// Firmware built with home settings and ROS 2.
    home_supported: bool,
    mode: Mode,
    reason: Reason,
    pending: Option<Command>,
    cause: Cause,
    deadline: u64,
}

impl Network {
    pub fn boot(now: u64, home_supported: bool) -> Self {
        let mut n = Self {
            home_supported,
            mode: Mode::Fallback,
            reason: Reason::NoSettings,
            pending: None,
            cause: Cause::Boot,
            deadline: 0,
        };
        n.attempt(now, Cause::Boot);
        n
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn reason(&self) -> Reason {
        self.reason
    }

    pub fn pending(&self) -> Option<Command> {
        self.pending
    }

    pub fn home_supported(&self) -> bool {
        self.home_supported
    }

    pub fn info(&self, ip: [u8; 4]) -> Info {
        Info {
            mode: self.mode,
            pending: self.pending,
            reason: self.reason,
            ip,
        }
    }

    /// The firmware finished a step of the current attempt. Reports for any other step (late,
    /// or from an earlier attempt) are ignored.
    pub fn on_step(&mut self, now: u64, step: Step, ok: bool) {
        if self.mode != Mode::Attempting(step) {
            return;
        }
        if !ok {
            self.fall_back(step_reason(step));
            return;
        }
        self.mode = match step {
            Step::Scan => {
                self.deadline = now + JOIN_BUDGET_MS;
                Mode::Attempting(Step::Join)
            }
            Step::Join => Mode::Attempting(Step::Dhcp),
            Step::Dhcp => Mode::Attempting(Step::Router),
            Step::Router => {
                self.reason = Reason::None;
                Mode::Home
            }
        };
    }

    /// Ends an attempt whose step overran its budget. Call from the control tick.
    pub fn tick(&mut self, now: u64) {
        if let Mode::Attempting(step) = self.mode {
            if now >= self.deadline {
                self.fall_back(step_reason(step));
            }
        }
    }

    /// An accepted mode command from the owner; replaces any pending one.
    pub fn command(&mut self, command: Command) {
        self.pending = Some(command);
    }

    /// The owner's ownership ended. `ble` is true for a BLE owner (whose claim turned the
    /// station off).
    pub fn on_ownership_end(&mut self, now: u64, ble: bool) {
        match self.pending.take() {
            // A Wi-Fi owner at home is already there; BLE turned the station off, so it rejoins.
            Some(Command::Home) if self.mode == Mode::Home && !ble => {}
            Some(Command::Home) => self.attempt(now, Cause::Requested),
            Some(Command::CarNetwork) => self.fall_back(Reason::Requested),
            None if ble && self.mode == Mode::Home => self.attempt(now, Cause::BleEnded),
            None => {}
        }
    }

    /// The commanding owner was taken over: it never released, so its command does not apply.
    pub fn discard_pending(&mut self) {
        self.pending = None;
    }

    /// The station lost the home network while in home mode.
    pub fn on_station_lost(&mut self, now: u64) {
        if self.mode == Mode::Home {
            self.attempt(now, Cause::LinkLost);
        }
    }

    fn attempt(&mut self, now: u64, cause: Cause) {
        if !self.home_supported {
            self.fall_back(Reason::NoSettings);
            return;
        }
        self.cause = cause;
        self.mode = Mode::Attempting(Step::Scan);
        self.deadline = now + SCAN_BUDGET_MS;
    }

    fn fall_back(&mut self, reason: Reason) {
        self.mode = Mode::Fallback;
        // A reattempt after losing the home network reports the loss, not the step it failed on.
        self.reason = if self.cause == Cause::LinkLost && reason != Reason::Requested {
            Reason::LinkLost
        } else {
            reason
        };
    }
}

fn step_reason(step: Step) -> Reason {
    match step {
        Step::Scan => Reason::Scan,
        Step::Join => Reason::Join,
        Step::Dhcp => Reason::Dhcp,
        Step::Router => Reason::Router,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(now: u64) -> Network {
        let mut n = Network::boot(now, true);
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            n.on_step(now, step, true);
        }
        assert_eq!(n.mode(), Mode::Home);
        n
    }

    fn fallback() -> Network {
        let mut n = Network::boot(0, true);
        n.on_step(100, Step::Scan, false);
        n
    }

    #[test]
    fn info_encodes_mode_pending_reason_and_ip() {
        let mut n = fallback();
        n.command(Command::Home);
        assert_eq!(n.info([0; 4]).encode(), [0, 1, 2, 0, 0, 0, 0, 0]);
        let n = home(0);
        assert_eq!(
            n.info([192, 168, 1, 50]).encode(),
            [2, 0, 0, 0, 192, 168, 1, 50]
        );
        let mut n = home(0);
        n.on_ownership_end(1, true);
        n.command(Command::CarNetwork);
        assert_eq!(n.info([0; 4]).encode()[..3], [1, 2, 0]);
    }

    #[test]
    fn info_cdr_is_a_logfmt_string() {
        let mut out = [0u8; 96];
        let n = home(0)
            .info([192, 168, 1, 50])
            .encode_cdr(&mut out)
            .unwrap();
        let text = b"mode=home pending=none fallback=none ip=192.168.1.50";
        assert_eq!(out[..4], [0, 1, 0, 0]);
        assert_eq!(out[4..8], (text.len() as u32 + 1).to_le_bytes());
        assert_eq!(&out[8..n - 1], text);
        assert_eq!(out[n - 1], 0);
        let mut n2 = fallback();
        n2.command(Command::CarNetwork);
        let len = n2.info([0; 4]).encode_cdr(&mut out).unwrap();
        assert_eq!(
            &out[8..len - 1],
            b"mode=fallback pending=car-network fallback=scan ip=0.0.0.0"
        );
    }

    #[test]
    fn no_home_settings_falls_back_without_scanning() {
        let n = Network::boot(0, false);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::NoSettings));
    }

    #[test]
    fn every_step_succeeding_enters_home() {
        let mut n = Network::boot(0, true);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
        n.on_step(800, Step::Scan, true);
        assert_eq!(n.mode(), Mode::Attempting(Step::Join));
        n.on_step(2_000, Step::Join, true);
        n.on_step(2_500, Step::Dhcp, true);
        assert_eq!(n.mode(), Mode::Attempting(Step::Router));
        n.on_step(3_000, Step::Router, true);
        assert_eq!((n.mode(), n.reason()), (Mode::Home, Reason::None));
    }

    #[test]
    fn a_failed_step_falls_back_with_that_step_as_reason() {
        for (failing, reason) in [
            (Step::Scan, Reason::Scan),
            (Step::Join, Reason::Join),
            (Step::Dhcp, Reason::Dhcp),
            (Step::Router, Reason::Router),
        ] {
            let mut n = Network::boot(0, true);
            for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
                n.on_step(10, step, step != failing);
                if step == failing {
                    break;
                }
            }
            assert_eq!(
                (n.mode(), n.reason()),
                (Mode::Fallback, reason),
                "{failing:?}"
            );
        }
    }

    #[test]
    fn scan_overrunning_its_budget_falls_back() {
        let mut n = Network::boot(1_000, true);
        n.tick(1_000 + SCAN_BUDGET_MS - 1);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
        n.tick(1_000 + SCAN_BUDGET_MS);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Scan));
    }

    #[test]
    fn join_budget_counts_from_the_network_being_found() {
        let mut n = Network::boot(0, true);
        n.on_step(1_200, Step::Scan, true);
        n.on_step(3_000, Step::Join, true);
        n.on_step(4_000, Step::Dhcp, true);
        n.tick(1_200 + JOIN_BUDGET_MS - 1);
        assert_eq!(n.mode(), Mode::Attempting(Step::Router));
        n.tick(1_200 + JOIN_BUDGET_MS);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Router));
    }

    #[test]
    fn late_step_report_after_fallback_is_ignored() {
        let mut n = Network::boot(0, true);
        n.tick(SCAN_BUDGET_MS);
        n.on_step(SCAN_BUDGET_MS + 10, Step::Scan, true);
        assert_eq!(n.mode(), Mode::Fallback);
    }

    #[test]
    fn out_of_order_step_report_is_ignored() {
        let mut n = Network::boot(0, true);
        n.on_step(10, Step::Router, true);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
    }

    #[test]
    fn fallback_is_sticky_without_a_command() {
        let mut n = fallback();
        n.tick(1_000_000);
        n.on_ownership_end(1_000_001, false);
        n.on_ownership_end(1_000_002, true);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Scan));
    }

    #[test]
    fn home_command_applies_at_ownership_end_from_fallback() {
        let mut n = fallback();
        n.command(Command::Home);
        assert_eq!(
            (n.mode(), n.pending()),
            (Mode::Fallback, Some(Command::Home))
        );
        n.on_ownership_end(5_000, false);
        assert_eq!(
            (n.mode(), n.pending()),
            (Mode::Attempting(Step::Scan), None)
        );
    }

    #[test]
    fn car_network_command_at_home_falls_back_as_requested() {
        let mut n = home(0);
        n.command(Command::CarNetwork);
        assert_eq!(n.mode(), Mode::Home);
        n.on_ownership_end(1_000, false);
        assert_eq!(
            (n.mode(), n.reason(), n.pending()),
            (Mode::Fallback, Reason::Requested, None)
        );
    }

    #[test]
    fn car_network_command_wins_over_ble_reattempt() {
        let mut n = home(0);
        n.command(Command::CarNetwork);
        n.on_ownership_end(1_000, true);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Requested));
    }

    #[test]
    fn car_network_command_in_fallback_records_the_request() {
        let mut n = fallback();
        n.command(Command::CarNetwork);
        n.on_ownership_end(1_000, false);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Requested));
    }

    #[test]
    fn later_command_replaces_the_pending_one() {
        let mut n = fallback();
        n.command(Command::CarNetwork);
        n.command(Command::Home);
        n.on_ownership_end(1_000, false);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
    }

    #[test]
    fn home_command_from_a_wifi_owner_at_home_keeps_home() {
        let mut n = home(0);
        n.command(Command::Home);
        n.on_ownership_end(1_000, false);
        assert_eq!((n.mode(), n.pending()), (Mode::Home, None));
    }

    #[test]
    fn discarded_command_does_not_apply() {
        let mut n = home(0);
        n.command(Command::CarNetwork);
        n.discard_pending();
        n.on_ownership_end(1_000, false);
        assert_eq!(n.mode(), Mode::Home);
    }

    #[test]
    fn ble_ownership_end_at_home_reattempts() {
        let mut n = home(0);
        n.on_ownership_end(1_000, true);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
    }

    #[test]
    fn wifi_ownership_end_at_home_keeps_home() {
        let mut n = home(0);
        n.on_ownership_end(1_000, false);
        assert_eq!(n.mode(), Mode::Home);
    }

    #[test]
    fn station_loss_reattempts_and_failure_reports_link_lost() {
        let mut n = home(0);
        n.on_station_lost(1_000);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
        n.on_step(1_500, Step::Join, false); // out of order: ignored
        n.on_step(1_600, Step::Scan, true);
        n.on_step(2_000, Step::Join, false);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::LinkLost));
    }

    #[test]
    fn station_loss_outside_home_is_ignored() {
        let mut n = fallback();
        n.on_station_lost(1_000);
        assert_eq!(n.mode(), Mode::Fallback);
    }

    #[test]
    fn reattempt_succeeding_returns_home_and_clears_the_reason() {
        let mut n = home(0);
        n.on_ownership_end(1_000, true);
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            n.on_step(1_100, step, true);
        }
        assert_eq!((n.mode(), n.reason()), (Mode::Home, Reason::None));
    }

    #[test]
    fn home_command_without_support_never_attempts() {
        let mut n = Network::boot(0, false);
        n.command(Command::Home);
        n.on_ownership_end(10, false);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::NoSettings));
    }

    #[test]
    fn reattempt_budgets_start_at_the_reattempt() {
        let mut n = home(0);
        n.on_ownership_end(100_000, true);
        n.tick(100_000 + SCAN_BUDGET_MS - 1);
        assert_eq!(n.mode(), Mode::Attempting(Step::Scan));
        n.tick(100_000 + SCAN_BUDGET_MS);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::Scan));
    }
}
