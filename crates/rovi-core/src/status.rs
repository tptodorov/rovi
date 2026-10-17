//! Canonical owner status and the version-1 binary / ROS encodings.

use crate::arbiter::{Error, State, StopReason};

pub const CDR_MAX: usize = 80;

/// Current state, persistent owner control error, and session (zero while open).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    pub error: Option<Error>,
    pub session: u32,
}

/// One ROS connection's change/heartbeat schedule.
#[derive(Default)]
pub struct Publication {
    last: Option<(u64, Status)>,
}

impl Publication {
    pub fn due(&mut self, now: u64, status: Status) -> bool {
        if self
            .last
            .is_some_and(|(time, s)| s == status && now.saturating_sub(time) < 900)
        {
            return false;
        }
        self.last = Some((now, status));
        true
    }
}

impl Status {
    /// Version-1 state/reason/error bytes; numeric codes are unchanged.
    pub fn encode(self) -> [u8; 3] {
        let (state, reason) = match self.state {
            State::Open => (0, 0),
            State::Claimed => (1, 0),
            State::Armed => (2, 0),
            State::Stopped(r) => (
                3,
                match r {
                    StopReason::Stop => 1,
                    StopReason::LeaseExpired => 2,
                    StopReason::OwnerLost => 3,
                    StopReason::OutOfLimits => 4,
                },
            ),
        };
        let error = match self.error {
            None => 0,
            Some(Error::Busy) => 1,
            Some(Error::NotOwner) => 2,
            Some(Error::NotReady) => 3,
            Some(Error::Invalid) => 4,
        };
        [state, reason, error]
    }

    /// Four `key=value` fields as `std_msgs/String` CDR, with explicit none values.
    pub fn encode_cdr(self, out: &mut [u8]) -> Option<usize> {
        use core::fmt::Write;
        let state = match self.state {
            State::Open => "open",
            State::Claimed => "claimed",
            State::Armed => "armed",
            State::Stopped(_) => "stopped",
        };
        let reason = match self.state {
            State::Stopped(StopReason::Stop) => "stop",
            State::Stopped(StopReason::LeaseExpired) => "lease-expired",
            State::Stopped(StopReason::OwnerLost) => "owner-lost",
            State::Stopped(StopReason::OutOfLimits) => "out-of-limits",
            _ => "none",
        };
        let error = match self.error {
            None => "none",
            Some(Error::Busy) => "busy",
            Some(Error::NotOwner) => "not-owner",
            Some(Error::NotReady) => "not-ready",
            Some(Error::Invalid) => "invalid",
        };
        let mut text = crate::rmw::Key::<{ CDR_MAX - 9 }>::new();
        write!(
            text,
            "state={state} reason={reason} error={error} session={}",
            self.session
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_and_ros_fields_cover_all_reasons_and_errors() {
        for (reason, code, name) in [
            (StopReason::Stop, 1, "stop"),
            (StopReason::LeaseExpired, 2, "lease-expired"),
            (StopReason::OwnerLost, 3, "owner-lost"),
            (StopReason::OutOfLimits, 4, "out-of-limits"),
        ] {
            for (error, ec, en) in [
                (None, 0, "none"),
                (Some(Error::Busy), 1, "busy"),
                (Some(Error::NotOwner), 2, "not-owner"),
                (Some(Error::NotReady), 3, "not-ready"),
                (Some(Error::Invalid), 4, "invalid"),
            ] {
                let s = Status {
                    state: State::Stopped(reason),
                    error,
                    session: u32::MAX,
                };
                assert_eq!(s.encode(), [3, code, ec]);
                let mut out = [0; CDR_MAX];
                let n = s.encode_cdr(&mut out).unwrap();
                assert_eq!(&out[..4], &[0, 1, 0, 0]);
                assert_eq!(
                    u32::from_le_bytes(out[4..8].try_into().unwrap()) as usize,
                    n - 8
                );
                assert_eq!(out[n - 1], 0);
                let text = core::str::from_utf8(&out[8..n - 1]).unwrap();
                assert_eq!(
                    text,
                    std::format!("state=stopped reason={name} error={en} session=4294967295")
                );
            }
        }
    }

    #[test]
    fn status_cdr_short_buffers_are_left_unchanged() {
        let status = Status {
            state: State::Stopped(StopReason::LeaseExpired),
            error: Some(Error::NotReady),
            session: u32::MAX,
        };
        let mut out = [0xaa; CDR_MAX];
        assert_eq!(status.encode_cdr(&mut out[..CDR_MAX - 3]), None);
        assert_eq!(out, [0xaa; CDR_MAX]);
        assert!(status.encode_cdr(&mut out).is_some());
    }

    #[test]
    fn status_publication_tracks_each_field_and_heartbeat() {
        let open = Status {
            state: State::Open,
            error: None,
            session: 0,
        };
        let mut p = Publication::default();
        assert!(p.due(0, open));
        assert!(!p.due(899, open));
        assert!(p.due(900, open));
        let mut s = Status {
            state: State::Claimed,
            session: 1,
            ..open
        };
        assert!(p.due(910, s));
        assert!(!p.due(911, s));
        s.error = Some(Error::NotReady);
        assert!(p.due(920, s));
        s.error = None;
        assert!(p.due(930, s));
        s.session = 2;
        assert!(p.due(940, s));
        s.state = State::Stopped(StopReason::Stop);
        assert!(p.due(950, s));
        s.state = State::Stopped(StopReason::OutOfLimits);
        assert!(p.due(960, s));
        assert!(p.due(970, open), "release publishes all cleared fields");
    }
}
