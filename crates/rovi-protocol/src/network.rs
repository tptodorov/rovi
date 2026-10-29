//! Network information and the mode command (ADR-0016).
//! Binary: BLE Network characteristic and the tail of UDP `INFO`. Text: ROS `/rovi/network`.

use crate::text::{self, Fields, Text};
use crate::{DecodeError, EncodeError};
use core::fmt::Write;

pub const LEN: usize = 8;
pub const CDR_MAX: usize = 105;

/// The network mode as reported; the car's attempt step is not on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetMode {
    Fallback,
    Attempting,
    Home,
}

/// A mode command: pending until the commanding owner's ownership ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Home,
    CarNetwork,
}

/// Why the car is in fallback (or why the current attempt started).
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

const MODES: [NetMode; 3] = [NetMode::Fallback, NetMode::Attempting, NetMode::Home];
const COMMANDS: [Command; 2] = [Command::Home, Command::CarNetwork];
const REASONS: [Reason; 8] = [
    Reason::None,
    Reason::NoSettings,
    Reason::Scan,
    Reason::Join,
    Reason::Dhcp,
    Reason::Router,
    Reason::LinkLost,
    Reason::Requested,
];

impl NetMode {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Fallback => "fallback",
            Self::Attempting => "attempting",
            Self::Home => "home",
        }
    }
}

impl Command {
    /// UDP `MODE` payload and ROS `/rovi/mode` value (BLE Control uses its own codes, T8).
    pub fn code(self) -> u8 {
        self as u8 + 1
    }

    pub fn from_code(code: u8) -> Option<Self> {
        COMMANDS.get((code as usize).wrapping_sub(1)).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::CarNetwork => "car-network",
        }
    }
}

impl Reason {
    /// The network-information code (also the firmware's reset flag).
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<Self> {
        REASONS.get(code as usize).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::NoSettings => "no-settings",
            Self::Scan => "scan",
            Self::Join => "join",
            Self::Dhcp => "dhcp",
            Self::Router => "router",
            Self::LinkLost => "link-lost",
            Self::Requested => "requested",
        }
    }
}

/// What every transport reports about the network.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkInfo {
    pub mode: NetMode,
    pub pending: Option<Command>,
    pub reason: Reason,
    /// Station IPv4; 0.0.0.0 without one.
    pub ip: [u8; 4],
}

fn pending_name(p: Option<Command>) -> &'static str {
    p.map_or("none", Command::name)
}

impl NetworkInfo {
    /// `[mode, pending, reason, 0, ip0, ip1, ip2, ip3]`.
    pub fn encode(self) -> [u8; LEN] {
        let [a, b, c, d] = self.ip;
        [
            self.mode.code(),
            self.pending.map_or(0, Command::code),
            self.reason.code(),
            0,
            a,
            b,
            c,
            d,
        ]
    }

    pub fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        let [mode, pending, reason, reserved, a, b, c, d] = *b else {
            return Err(DecodeError::Length);
        };
        let mode = *MODES.get(mode as usize).ok_or(DecodeError::UnknownCode)?;
        let pending = match pending {
            0 => None,
            p => Some(Command::from_code(p).ok_or(DecodeError::UnknownCode)?),
        };
        let reason = Reason::from_code(reason).ok_or(DecodeError::UnknownCode)?;
        if reserved != 0 {
            return Err(DecodeError::Reserved);
        }
        Ok(Self {
            mode,
            pending,
            reason,
            ip: [a, b, c, d],
        })
    }

    /// `mode=… pending=… fallback=… ip=…` as `std_msgs/String` CDR.
    pub fn encode_cdr(self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let mut t = Text::<{ CDR_MAX - 9 }>::default();
        let [a, b, c, d] = self.ip;
        write!(
            t,
            "mode={} pending={} fallback={} ip={a}.{b}.{c}.{d}",
            self.mode.name(),
            pending_name(self.pending),
            self.reason.name()
        )
        .map_err(|_| EncodeError::BufferTooSmall)?;
        text::encode_string(t.as_str(), out)
    }

    pub fn decode_cdr(b: &[u8]) -> Result<Self, DecodeError> {
        let mut f = Fields::new(text::decode_string(b)?);
        let (mode, pending, reason, ip) = (
            f.next("mode")?,
            f.next("pending")?,
            f.next("fallback")?,
            f.next("ip")?,
        );
        f.end()?;
        let mode = *MODES
            .iter()
            .find(|m| m.name() == mode)
            .ok_or(DecodeError::UnknownCode)?;
        let pending = match pending {
            "none" => None,
            p => Some(
                *COMMANDS
                    .iter()
                    .find(|c| c.name() == p)
                    .ok_or(DecodeError::UnknownCode)?,
            ),
        };
        let reason = *REASONS
            .iter()
            .find(|r| r.name() == reason)
            .ok_or(DecodeError::UnknownCode)?;
        let mut octets = ip.split('.');
        let mut ip = [0u8; 4];
        for o in &mut ip {
            let s = octets.next().ok_or(DecodeError::BadText)?;
            // Canonical decimal only: no leading zeros, no sign.
            if s.is_empty()
                || (s.len() > 1 && s.starts_with('0'))
                || !s.bytes().all(|c| c.is_ascii_digit())
            {
                return Err(DecodeError::BadText);
            }
            *o = s.parse().map_err(|_| DecodeError::BadText)?;
        }
        if octets.next().is_some() {
            return Err(DecodeError::BadText);
        }
        Ok(Self {
            mode,
            pending,
            reason,
            ip,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> impl Iterator<Item = NetworkInfo> {
        MODES.into_iter().flat_map(|mode| {
            [None, Some(Command::Home), Some(Command::CarNetwork)]
                .into_iter()
                .flat_map(move |pending| {
                    REASONS.into_iter().map(move |reason| NetworkInfo {
                        mode,
                        pending,
                        reason,
                        ip: [192, 168, 1, 50],
                    })
                })
        })
    }

    #[test]
    fn every_value_round_trips_binary_and_text() {
        for i in all() {
            assert_eq!(NetworkInfo::decode(&i.encode()), Ok(i));
            let mut out = [0; CDR_MAX];
            let n = i.encode_cdr(&mut out).unwrap();
            assert_eq!(NetworkInfo::decode_cdr(&out[..n]), Ok(i));
        }
    }

    #[test]
    fn malformed_information_is_refused() {
        assert_eq!(
            NetworkInfo::decode(&[2, 0, 0, 1, 0, 0, 0, 0]),
            Err(DecodeError::Reserved)
        );
        assert_eq!(
            NetworkInfo::decode(&[3, 0, 0, 0, 0, 0, 0, 0]),
            Err(DecodeError::UnknownCode)
        );
        assert_eq!(
            NetworkInfo::decode(&[2, 3, 0, 0, 0, 0, 0, 0]),
            Err(DecodeError::UnknownCode)
        );
        assert_eq!(
            NetworkInfo::decode(&[0, 0, 8, 0, 0, 0, 0, 0]),
            Err(DecodeError::UnknownCode)
        );
        assert_eq!(NetworkInfo::decode(&[0; 7]), Err(DecodeError::Length));
        let mut out = [0; CDR_MAX];
        for bad in [
            "mode=home pending=none fallback=none ip=1.2.3",
            "mode=home pending=none fallback=none ip=01.2.3.4",
            "mode=home pending=none reason=none ip=1.2.3.4",
        ] {
            let n = text::encode_string(bad, &mut out).unwrap();
            assert_eq!(
                NetworkInfo::decode_cdr(&out[..n]),
                Err(DecodeError::BadText),
                "{bad}"
            );
        }
    }

    #[test]
    fn longest_text_fits() {
        let i = NetworkInfo {
            mode: NetMode::Attempting,
            pending: Some(Command::CarNetwork),
            reason: Reason::NoSettings,
            ip: [255; 4],
        };
        assert!(i.encode_cdr(&mut [0; CDR_MAX]).is_ok());
    }

    #[test]
    fn command_codes_are_one_and_two() {
        assert_eq!((Command::Home.code(), Command::CarNetwork.code()), (1, 2));
        assert_eq!(Command::from_code(0), None);
        assert_eq!(Command::from_code(3), None);
    }
}
