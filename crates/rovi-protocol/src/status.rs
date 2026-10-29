//! Status: shared state, stop reason, last refused owner command and (ROS only) session.
//! Binary: BLE Status characteristic and UDP `STATUS`. Text: ROS `/rovi/status`.

use crate::text::{self, Fields, Text};
use crate::{DecodeError, EncodeError};
use core::fmt::Write;

pub const LEN: usize = 3;
/// Upper bound for the ROS `std_msgs/String` CDR, including encapsulation and NUL.
pub const CDR_MAX: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    /// Owner claimed, not armed.
    Claimed,
    Armed,
    /// Safety stop latched, owner kept for the reclaim window.
    Stopped(StopReason),
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

const REASONS: [StopReason; 4] = [
    StopReason::Stop,
    StopReason::LeaseExpired,
    StopReason::OwnerLost,
    StopReason::OutOfLimits,
];
const ERRORS: [Error; 4] = [
    Error::Busy,
    Error::NotOwner,
    Error::NotReady,
    Error::Invalid,
];

impl State {
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Claimed => "claimed",
            Self::Armed => "armed",
            Self::Stopped(_) => "stopped",
        }
    }
}

impl StopReason {
    pub fn code(self) -> u8 {
        self as u8 + 1
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::LeaseExpired => "lease-expired",
            Self::OwnerLost => "owner-lost",
            Self::OutOfLimits => "out-of-limits",
        }
    }
}

impl Error {
    pub fn code(self) -> u8 {
        self as u8 + 1
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Busy => "busy",
            Self::NotOwner => "not-owner",
            Self::NotReady => "not-ready",
            Self::Invalid => "invalid",
        }
    }
}

fn reason_name(state: State) -> &'static str {
    match state {
        State::Stopped(r) => r.name(),
        _ => "none",
    }
}

fn error_name(e: Option<Error>) -> &'static str {
    e.map_or("none", Error::name)
}

/// Current state, persistent owner control error, and session (zero while open).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    pub error: Option<Error>,
    /// Only on ROS (design T6); binary Status decodes with session 0.
    pub session: u32,
}

impl Status {
    /// Version-1 state/reason/error bytes.
    pub fn encode(self) -> [u8; LEN] {
        let (state, reason) = match self.state {
            State::Open => (0, 0),
            State::Claimed => (1, 0),
            State::Armed => (2, 0),
            State::Stopped(r) => (3, r.code()),
        };
        [state, reason, self.error.map_or(0, Error::code)]
    }

    pub fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        let [state, reason, error] = *b else {
            return Err(DecodeError::Length);
        };
        let reason = match reason {
            0 => None,
            r => Some(
                *REASONS
                    .get(r as usize - 1)
                    .ok_or(DecodeError::UnknownCode)?,
            ),
        };
        let error = match error {
            0 => None,
            e => Some(*ERRORS.get(e as usize - 1).ok_or(DecodeError::UnknownCode)?),
        };
        let state = match (state, reason) {
            (0, None) => State::Open,
            (1, None) => State::Claimed,
            (2, None) => State::Armed,
            (3, Some(r)) => State::Stopped(r),
            (0..=3, _) => return Err(DecodeError::Inconsistent),
            _ => return Err(DecodeError::UnknownCode),
        };
        Ok(Self {
            state,
            error,
            session: 0,
        })
    }

    /// `state=… reason=… error=… session=…` as `std_msgs/String` CDR.
    pub fn encode_cdr(self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let mut t = Text::<{ CDR_MAX - 9 }>::default();
        write!(
            t,
            "state={} reason={} error={} session={}",
            self.state.name(),
            reason_name(self.state),
            error_name(self.error),
            self.session
        )
        .map_err(|_| EncodeError::BufferTooSmall)?;
        text::encode_string(t.as_str(), out)
    }

    pub fn decode_cdr(b: &[u8]) -> Result<Self, DecodeError> {
        let mut f = Fields::new(text::decode_string(b)?);
        let (state, reason, error) = (f.next("state")?, f.next("reason")?, f.next("error")?);
        let session = f.parse("session")?;
        f.end()?;
        let reason = match reason {
            "none" => None,
            r => Some(
                *REASONS
                    .iter()
                    .find(|x| x.name() == r)
                    .ok_or(DecodeError::UnknownCode)?,
            ),
        };
        let state = match (state, reason) {
            ("open", None) => State::Open,
            ("claimed", None) => State::Claimed,
            ("armed", None) => State::Armed,
            ("stopped", Some(r)) => State::Stopped(r),
            ("open" | "claimed" | "armed" | "stopped", _) => return Err(DecodeError::Inconsistent),
            _ => return Err(DecodeError::UnknownCode),
        };
        let error = match error {
            "none" => None,
            e => Some(
                *ERRORS
                    .iter()
                    .find(|x| x.name() == e)
                    .ok_or(DecodeError::UnknownCode)?,
            ),
        };
        Ok(Self {
            state,
            error,
            session,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> impl Iterator<Item = Status> {
        let states = [State::Open, State::Claimed, State::Armed]
            .into_iter()
            .chain(REASONS.map(State::Stopped));
        states.flat_map(|state| {
            [None]
                .into_iter()
                .chain(ERRORS.map(Some))
                .map(move |error| Status {
                    state,
                    error,
                    session: 0,
                })
        })
    }

    #[test]
    fn every_binary_status_round_trips_and_every_other_byte_triple_is_refused() {
        let mut valid = 0;
        for b in 0..=u8::MAX {
            for r in 0..=5 {
                for e in 0..=5 {
                    let bytes = [b, r, e];
                    if let Ok(s) = Status::decode(&bytes) {
                        assert_eq!(s.encode(), bytes);
                        valid += 1;
                    }
                }
            }
        }
        assert_eq!(valid, all().count());
        for s in all() {
            assert_eq!(Status::decode(&s.encode()), Ok(s));
        }
        assert_eq!(Status::decode(&[3, 0, 0]), Err(DecodeError::Inconsistent));
        assert_eq!(Status::decode(&[2, 1, 0]), Err(DecodeError::Inconsistent));
        assert_eq!(Status::decode(&[4, 0, 0]), Err(DecodeError::UnknownCode));
        assert_eq!(Status::decode(&[0, 0]), Err(DecodeError::Length));
    }

    #[test]
    fn ros_text_round_trips_with_the_session() {
        for s in all() {
            for session in [0, 1, u32::MAX] {
                let s = Status { session, ..s };
                let mut out = [0; CDR_MAX];
                let n = s.encode_cdr(&mut out).unwrap();
                assert_eq!(Status::decode_cdr(&out[..n]), Ok(s));
            }
        }
        let mut out = [0; CDR_MAX];
        let n = text::encode_string("state=stopped reason=none error=none session=1", &mut out)
            .unwrap();
        assert_eq!(
            Status::decode_cdr(&out[..n]),
            Err(DecodeError::Inconsistent)
        );
        let n = text::encode_string("state=open reason=none error=none", &mut out).unwrap();
        assert_eq!(Status::decode_cdr(&out[..n]), Err(DecodeError::BadText));
    }

    #[test]
    fn short_buffers_are_left_unchanged() {
        let s = Status {
            state: State::Stopped(StopReason::LeaseExpired),
            error: Some(Error::NotReady),
            session: u32::MAX,
        };
        let mut out = [0xaa; CDR_MAX];
        assert_eq!(
            s.encode_cdr(&mut out[..CDR_MAX - 3]),
            Err(EncodeError::BufferTooSmall)
        );
        assert_eq!(out, [0xaa; CDR_MAX]);
        assert!(s.encode_cdr(&mut out).is_ok());
    }
}
