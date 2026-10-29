//! Network mode: home (station + ROS router) or fallback (the car AP, today's behaviour).
//!
//! Pure and clock-driven, like the arbiter. The firmware runs each home-mode step (scan, join,
//! DHCP, router) and reports its outcome; this module decides the mode, the budgets and what a
//! mode command or the end of an ownership does (`openspec/specs/network-mode`).

use rovi_protocol::network::{Command, NetMode, NetworkInfo, Reason};

/// Longest scan for the configured network before falling back.
pub const SCAN_BUDGET_MS: u64 = 1_500;
/// Join + DHCP + router, counted from the moment the network is found. 8 s: the board took
/// 4.5 s on good runs (join 0.66-2.8 s, DHCP 0.9 s, router 0.8 s; 6.1 on 2026-10-26), and a
/// router retry waits 2 s (Todor raised it from 5 s on 2026-10-26).
pub const JOIN_BUDGET_MS: u64 = 8_000;

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

/// Build-time home settings: all three or none (spec *Build-time home settings*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settings {
    None,
    Home,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsError {
    /// Some but not all of SSID, passphrase and router were given.
    Incomplete,
    /// SSID is not 1..=32 bytes.
    Ssid,
    /// Passphrase is not 8..=63 printable ASCII characters.
    Passphrase,
    /// Router is not `tcp/<IPv4 or host name>:<port 1..=65535>`.
    Router,
}

/// Checks the home settings; `const`, so the firmware refuses a bad set at build time.
pub const fn settings(
    ssid: Option<&str>,
    passphrase: Option<&str>,
    router: Option<&str>,
) -> Result<Settings, SettingsError> {
    let (ssid, passphrase, router) = match (ssid, passphrase, router) {
        (None, None, None) => return Ok(Settings::None),
        (Some(s), Some(p), Some(r)) => (s, p, r),
        _ => return Err(SettingsError::Incomplete),
    };
    if ssid.is_empty() || ssid.len() > 32 {
        return Err(SettingsError::Ssid);
    }
    let p = passphrase.as_bytes();
    if p.len() < 8 || p.len() > 63 {
        return Err(SettingsError::Passphrase);
    }
    let mut i = 0;
    while i < p.len() {
        if p[i] < 0x20 || p[i] > 0x7e {
            return Err(SettingsError::Passphrase);
        }
        i += 1;
    }
    if !tcp_endpoint(router.as_bytes()) {
        return Err(SettingsError::Router);
    }
    Ok(Settings::Home)
}

/// `tcp/<host>:<port>`: `host` is an IPv4 literal or a host name, `port` is 1..=65535. A name
/// ending in `.local` is resolved over mDNS by the car; any other name needs the DHCP-supplied DNS
/// server to know it.
const fn tcp_endpoint(b: &[u8]) -> bool {
    let prefix = b"tcp/";
    if b.len() <= prefix.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        if b[i] != prefix[i] {
            return false;
        }
        i += 1;
    }
    // The port follows the last ':'.
    let mut colon = b.len();
    let mut k = prefix.len();
    while k < b.len() {
        if b[k] == b':' {
            colon = k;
        }
        k += 1;
    }
    if colon == b.len() {
        return false;
    }
    let mut port: u32 = 0;
    let mut digits = 0;
    let mut k = colon + 1;
    while k < b.len() {
        if !b[k].is_ascii_digit() {
            return false;
        }
        port = port * 10 + (b[k] - b'0') as u32;
        digits += 1;
        if digits > 5 {
            return false;
        }
        k += 1;
    }
    if digits == 0 || port < 1 || port > 65535 {
        return false;
    }
    host_ok(b, prefix.len(), colon)
}

/// `b[start..end]` is an IPv4 literal or a host name.
const fn host_ok(b: &[u8], start: usize, end: usize) -> bool {
    if end <= start || end - start > 253 {
        return false;
    }
    let mut letter = false;
    let mut k = start;
    while k < end {
        if b[k].is_ascii_alphabetic() {
            letter = true;
        }
        k += 1;
    }
    // Digits and dots only: it must be a well-formed IPv4 address, not a mangled one.
    if !letter {
        return ipv4_ok(b, start, end);
    }
    // Dot-separated labels of letters, digits and hyphens, 1..=63 bytes, not hyphen-edged.
    let mut label_start = start;
    let mut k = start;
    while k <= end {
        if k == end || b[k] == b'.' {
            let len = k - label_start;
            if len == 0 || len > 63 || b[label_start] == b'-' || b[k - 1] == b'-' {
                return false;
            }
            label_start = k + 1;
        } else if !(b[k].is_ascii_alphanumeric() || b[k] == b'-') {
            return false;
        }
        k += 1;
    }
    true
}

const fn ipv4_ok(b: &[u8], start: usize, end: usize) -> bool {
    let mut part = 0;
    let mut value: u32 = 0;
    let mut digits = 0;
    let mut k = start;
    while k <= end {
        if k == end || b[k] == b'.' {
            if digits == 0 || value > 255 {
                return false;
            }
            part += 1;
            value = 0;
            digits = 0;
        } else if b[k].is_ascii_digit() {
            // Rust's socket parser rejects leading-zero octets too.
            if digits > 0 && value == 0 {
                return false;
            }
            value = value * 10 + (b[k] - b'0') as u32;
            digits += 1;
            if digits > 3 {
                return false;
            }
        } else {
            return false;
        }
        k += 1;
    }
    part == 4
}

/// Why a home-mode attempt started; decides the fallback reason when it fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cause {
    Boot,
    Requested,
    BleEnded,
    LinkLost,
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

    /// Boot straight into fallback after a mode switch reset with the stored reason (the
    /// firmware switches station <-> AP by resetting, design D2). Home commands still work.
    pub fn boot_fallback(home_supported: bool, reason: Reason) -> Self {
        Self {
            home_supported,
            mode: Mode::Fallback,
            reason,
            pending: None,
            cause: Cause::Boot,
            deadline: 0,
        }
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

    pub fn info(&self, ip: [u8; 4]) -> NetworkInfo {
        NetworkInfo {
            mode: match self.mode {
                Mode::Fallback => NetMode::Fallback,
                Mode::Attempting(_) => NetMode::Attempting,
                Mode::Home => NetMode::Home,
            },
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
    fn a_reason_code_round_trips_for_the_reset_flag() {
        for r in [
            Reason::None,
            Reason::NoSettings,
            Reason::Scan,
            Reason::Join,
            Reason::Dhcp,
            Reason::Router,
            Reason::LinkLost,
            Reason::Requested,
        ] {
            assert_eq!(Reason::from_code(r.code()), Some(r));
        }
        assert_eq!(Reason::from_code(8), None);
    }

    #[test]
    fn boot_after_a_switch_starts_in_fallback_with_the_stored_reason() {
        let n = Network::boot_fallback(true, Reason::LinkLost);
        assert_eq!((n.mode(), n.reason()), (Mode::Fallback, Reason::LinkLost));
        let mut n = Network::boot_fallback(true, Reason::Requested);
        n.command(Command::Home);
        n.on_ownership_end(10, false);
        assert_eq!(
            n.mode(),
            Mode::Attempting(Step::Scan),
            "Home still works from it"
        );
    }

    #[test]
    fn home_settings_are_all_or_none_and_valid() {
        use Settings as S;
        assert_eq!(settings(None, None, None), Ok(S::None));
        assert_eq!(
            settings(
                Some("devolo-899"),
                Some("12345678"),
                Some("tcp/192.168.1.244:7447")
            ),
            Ok(S::Home)
        );
        assert_eq!(
            settings(Some("x"), None, None),
            Err(SettingsError::Incomplete)
        );
        assert_eq!(
            settings(Some(""), Some("12345678"), Some("tcp/1.2.3.4:1")),
            Err(SettingsError::Ssid)
        );
        assert_eq!(
            settings(
                Some("0123456789012345678901234567890123"),
                Some("12345678"),
                Some("tcp/1.2.3.4:1")
            ),
            Err(SettingsError::Ssid)
        );
        assert_eq!(
            settings(Some("a"), Some("1234567"), Some("tcp/1.2.3.4:1")),
            Err(SettingsError::Passphrase)
        );
        assert_eq!(
            settings(Some("a"), Some("12345678é"), Some("tcp/1.2.3.4:1")),
            Err(SettingsError::Passphrase)
        );
        for bad in [
            "udp/1.2.3.4:1",
            "tcp/1.2.3:1",
            "tcp/1.2.3.4",
            "tcp/1.2.3.4:",
            "tcp/256.1.1.1:1",
            "tcp/01.2.3.4:7447",
            "tcp/1.002.3.4:7447",
            "tcp/1.2.3.4:70000",
            "tcp/1.2.3.4:0",
            // host names: letters/digits/hyphens in dot-separated labels, no empty or hyphen-edged
            // label, at least one letter (an all-digit name is a malformed address)
            "tcp/:7447",
            "tcp/.local:7447",
            "tcp/a..local:7447",
            "tcp/-a.local:7447",
            "tcp/a-.local:7447",
            "tcp/a_b.local:7447",
            "tcp/black box.local:7447",
            "tcp/1.2.3:1",
            "tcp/host.local:0",
            "tcp/host.local",
            "tcp/host.local:70000",
        ] {
            assert_eq!(
                settings(Some("a"), Some("12345678"), Some(bad)),
                Err(SettingsError::Router),
                "{bad}"
            );
        }
        for good in [
            "tcp/blackbox.local:7447",
            "tcp/Blackbox.LOCAL:7447",
            "tcp/blackbox:7447",
            "tcp/a-b.c1.local:1",
            "tcp/a.b.c.d:1",
            "tcp/192.168.1.175:7447",
        ] {
            assert_eq!(
                settings(Some("a"), Some("12345678"), Some(good)),
                Ok(S::Home),
                "{good}"
            );
        }
        let long_label =
            "tcp/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.local:1";
        assert_eq!(
            settings(Some("a"), Some("12345678"), Some(long_label)),
            Err(SettingsError::Router),
            "a label is at most 63 bytes"
        );
        const CHECKED: Result<Settings, SettingsError> =
            settings(Some("a"), Some("12345678"), Some("tcp/10.0.0.1:7447"));
        assert_eq!(CHECKED, Ok(S::Home), "usable in const context");
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
