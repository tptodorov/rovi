//! The Rovi wire contract (ADR-0017): every message on BLE, UDP and ROS as one typed value per
//! concept, with its encodings beside it (design D7). Encoders and validating decoders are
//! symmetric: `decode(encode(v)) == v` and, for canonical bytes, `encode(decode(b)) == b`.
//! No I/O, no allocator, no device behaviour; that lives in `rovi-core`.
#![no_std]

#[cfg(test)]
extern crate std;

pub mod ble;
pub mod capabilities;
pub mod network;
pub mod setpoint;
pub mod status;
pub mod text;
pub mod udp;

pub use capabilities::Capabilities;
pub use network::{Command, NetMode, NetworkInfo, Reason};
pub use setpoint::Setpoint;
pub use status::{Error, State, Status, StopReason};

/// Largest Echo payload (UDP `ECHO`, BLE Echo characteristic).
pub const ECHO_MAX: usize = 32;

/// Why bytes were refused. Shared by every message so all bindings report the same kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Wrong length for the message.
    Length,
    /// An enum code outside the defined set.
    UnknownCode,
    /// A reserved byte is not zero.
    Reserved,
    /// Fields that contradict each other (a reason without a stop, a stop without a reason).
    Inconsistent,
    NotFinite,
    /// A finite value outside its allowed range (non-positive geometry, negative limit, zero window).
    OutOfRange,
    UnsupportedVersion,
    /// Not a Rovi UDP frame (magic).
    BadMagic,
    /// ROS text that is not exactly the canonical `key=value` form.
    BadText,
    Setpoint(setpoint::DecodeError),
}

impl DecodeError {
    /// Stable kebab-case name, as written in `vectors.json` and reported by the bindings.
    pub fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::UnknownCode => "unknown-code",
            Self::Reserved => "reserved",
            Self::Inconsistent => "inconsistent",
            Self::NotFinite => "not-finite",
            Self::OutOfRange => "out-of-range",
            Self::UnsupportedVersion => "unsupported-version",
            Self::BadMagic => "bad-magic",
            Self::BadText => "bad-text",
            Self::Setpoint(e) => e.name(),
        }
    }
}

impl From<setpoint::DecodeError> for DecodeError {
    fn from(e: setpoint::DecodeError) -> Self {
        Self::Setpoint(e)
    }
}

/// Why a value could not be encoded. Nothing is written on error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    BufferTooSmall,
    /// A value the decoder would refuse (non-finite velocity, oversized Echo).
    Invalid,
}
