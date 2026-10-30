//! BLE GATT service: UUIDs, characteristic sizes, Control values and the Setpoint value.

use crate::network::Command;
use crate::setpoint::{self, Setpoint};
use crate::{DecodeError, EncodeError};

pub const SERVICE: &str = "d47a0010-45b2-4d19-8db0-6bd87e9c0001";
/// Write without response: [`encode_setpoint`].
pub const SETPOINT: &str = "d47a0011-45b2-4d19-8db0-6bd87e9c0001";
/// Write: one [`Control`] byte.
pub const CONTROL: &str = "d47a0012-45b2-4d19-8db0-6bd87e9c0001";
/// Read: [`crate::Capabilities`].
pub const CAPABILITIES: &str = "d47a0013-45b2-4d19-8db0-6bd87e9c0001";
/// Read, notify: [`crate::Status`].
pub const STATUS: &str = "d47a0014-45b2-4d19-8db0-6bd87e9c0001";
/// Write without response, notify: up to [`crate::ECHO_MAX`] bytes, returned unchanged.
pub const ECHO: &str = "d47a0015-45b2-4d19-8db0-6bd87e9c0001";
/// Read, notify: [`crate::NetworkInfo`].
pub const NETWORK: &str = "d47a0016-45b2-4d19-8db0-6bd87e9c0001";

/// The same UUIDs as numbers, for GATT servers (`trouble-host` takes `u128`).
pub mod id {
    pub const SERVICE: u128 = 0xd47a0010_45b2_4d19_8db0_6bd87e9c0001;
    pub const SETPOINT: u128 = 0xd47a0011_45b2_4d19_8db0_6bd87e9c0001;
    pub const CONTROL: u128 = 0xd47a0012_45b2_4d19_8db0_6bd87e9c0001;
    pub const CAPABILITIES: u128 = 0xd47a0013_45b2_4d19_8db0_6bd87e9c0001;
    pub const STATUS: u128 = 0xd47a0014_45b2_4d19_8db0_6bd87e9c0001;
    pub const ECHO: u128 = 0xd47a0015_45b2_4d19_8db0_6bd87e9c0001;
    pub const NETWORK: u128 = 0xd47a0016_45b2_4d19_8db0_6bd87e9c0001;
}

/// `seq u32` LE + CDR `TwistStamped` with an empty frame id.
pub const SETPOINT_LEN: usize = 4 + setpoint::MIN_LEN;

/// Control characteristic values (design T8: BLE keeps its own codes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Arm,
    Stop,
    Mode(Command),
}

impl Control {
    pub fn code(self) -> u8 {
        match self {
            Self::Arm => 1,
            Self::Stop => 2,
            Self::Mode(Command::Home) => 3,
            Self::Mode(Command::CarNetwork) => 4,
        }
    }

    pub fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        match *b {
            [1] => Ok(Self::Arm),
            [2] => Ok(Self::Stop),
            [3] => Ok(Self::Mode(Command::Home)),
            [4] => Ok(Self::Mode(Command::CarNetwork)),
            [_] => Err(DecodeError::UnknownCode),
            _ => Err(DecodeError::Length),
        }
    }
}

pub fn encode_setpoint(seq: u32, sp: &Setpoint) -> Result<[u8; SETPOINT_LEN], EncodeError> {
    let mut out = [0; SETPOINT_LEN];
    out[..4].copy_from_slice(&seq.to_le_bytes());
    sp.encode(&mut out[4..])?;
    Ok(out)
}

/// The sequence and setpoint of a Setpoint write. The car accepts a frame id here too.
pub fn decode_setpoint(b: &[u8]) -> Result<(u32, Setpoint), DecodeError> {
    let seq = b.get(..4).ok_or(DecodeError::Length)?;
    Ok((
        u32::from_le_bytes(seq.try_into().unwrap()),
        Setpoint::decode(&b[4..])?,
    ))
}

/// The sim car's stand-in for this GATT service: one UDP datagram per GATT operation
/// (`apps/simcar` virtual BLE). Development evidence only (ADR-0002): no radio, MTU or
/// connection timing is modelled.
pub mod sim {
    use crate::{DecodeError, EncodeError};

    /// ATT "Value Not Allowed", the firmware's answer to a refused Control write.
    pub const ATT_VALUE_NOT_ALLOWED: u8 = 0x13;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Characteristic {
        Status = 1,
        Capabilities = 2,
        Network = 3,
    }

    /// Central to car.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Request<'a> {
        Connect,
        /// Write without response: the Setpoint value.
        Setpoint(&'a [u8]),
        /// Write: one Control byte, passed on unchecked so the car decides.
        Control(u8),
        Read(Characteristic),
        Echo(&'a [u8]),
        Disconnect,
    }

    /// Car to central.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Reply<'a> {
        /// `false` when the car is not advertising or both slots are taken.
        Connected(bool),
        /// The ATT result of a Control write: 0 accepted, else an ATT error.
        Control(u8),
        Read(Characteristic, &'a [u8]),
        Echo(&'a [u8]),
        Disconnected,
        /// The car dropped this central (an owner that lost the car).
        Dropped,
    }

    fn characteristic(b: u8) -> Result<Characteristic, DecodeError> {
        match b {
            1 => Ok(Characteristic::Status),
            2 => Ok(Characteristic::Capabilities),
            3 => Ok(Characteristic::Network),
            _ => Err(DecodeError::UnknownCode),
        }
    }

    fn put(out: &mut [u8], op: u8, body: &[u8]) -> Result<usize, EncodeError> {
        let out = out
            .get_mut(..1 + body.len())
            .ok_or(EncodeError::BufferTooSmall)?;
        out[0] = op;
        out[1..].copy_from_slice(body);
        Ok(out.len())
    }

    impl<'a> Request<'a> {
        pub fn decode(b: &'a [u8]) -> Result<Self, DecodeError> {
            let (&op, body) = b.split_first().ok_or(DecodeError::Length)?;
            Ok(match (op, body) {
                (1, []) => Self::Connect,
                (2, _) => Self::Setpoint(body),
                (3, &[c]) => Self::Control(c),
                (4, &[c]) => Self::Read(characteristic(c)?),
                (5, _) => Self::Echo(body),
                (6, []) => Self::Disconnect,
                (1..=6, _) => return Err(DecodeError::Length),
                _ => return Err(DecodeError::UnknownCode),
            })
        }

        pub fn encode(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
            match *self {
                Self::Connect => put(out, 1, &[]),
                Self::Setpoint(v) => put(out, 2, v),
                Self::Control(c) => put(out, 3, &[c]),
                Self::Read(c) => put(out, 4, &[c as u8]),
                Self::Echo(v) => put(out, 5, v),
                Self::Disconnect => put(out, 6, &[]),
            }
        }
    }

    impl<'a> Reply<'a> {
        pub fn decode(b: &'a [u8]) -> Result<Self, DecodeError> {
            let (&op, body) = b.split_first().ok_or(DecodeError::Length)?;
            Ok(match (op, body) {
                (0x81, &[ok]) => Self::Connected(ok != 0),
                (0x83, &[att]) => Self::Control(att),
                (0x84, [c, value @ ..]) => Self::Read(characteristic(*c)?, value),
                (0x85, _) => Self::Echo(body),
                (0x86, []) => Self::Disconnected,
                (0x87, []) => Self::Dropped,
                (0x81 | 0x83 | 0x84 | 0x86 | 0x87, _) => return Err(DecodeError::Length),
                _ => return Err(DecodeError::UnknownCode),
            })
        }

        pub fn encode(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
            match *self {
                Self::Connected(ok) => put(out, 0x81, &[ok as u8]),
                Self::Control(att) => put(out, 0x83, &[att]),
                Self::Read(c, value) => {
                    let n = put(out, 0x84, &[c as u8])?;
                    let rest = out
                        .get_mut(n..n + value.len())
                        .ok_or(EncodeError::BufferTooSmall)?;
                    rest.copy_from_slice(value);
                    Ok(n + value.len())
                }
                Self::Echo(v) => put(out, 0x85, v),
                Self::Disconnected => put(out, 0x86, &[]),
                Self::Dropped => put(out, 0x87, &[]),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_codes_round_trip() {
        for c in [
            Control::Arm,
            Control::Stop,
            Control::Mode(Command::Home),
            Control::Mode(Command::CarNetwork),
        ] {
            assert_eq!(Control::decode(&[c.code()]), Ok(c));
        }
        assert_eq!(Control::decode(&[0]), Err(DecodeError::UnknownCode));
        assert_eq!(Control::decode(&[5]), Err(DecodeError::UnknownCode));
        assert_eq!(Control::decode(&[]), Err(DecodeError::Length));
    }

    #[test]
    fn setpoint_round_trips_with_wrapping_sequences() {
        let sp = Setpoint {
            sec: 0,
            nanosec: 0,
            vx: -0.1,
            vy: 0.1,
            wz: 0.3,
        };
        for seq in [0, 1, u32::MAX] {
            assert_eq!(
                decode_setpoint(&encode_setpoint(seq, &sp).unwrap()),
                Ok((seq, sp))
            );
        }
        assert_eq!(decode_setpoint(&[0; 3]), Err(DecodeError::Length));
    }

    #[test]
    fn numeric_uuids_match_the_strings() {
        for (text, n) in [
            (SERVICE, id::SERVICE),
            (SETPOINT, id::SETPOINT),
            (CONTROL, id::CONTROL),
            (CAPABILITIES, id::CAPABILITIES),
            (STATUS, id::STATUS),
            (ECHO, id::ECHO),
            (NETWORK, id::NETWORK),
        ] {
            let hex: std::string::String = text.chars().filter(|c| *c != '-').collect();
            assert_eq!(u128::from_str_radix(&hex, 16), Ok(n), "{text}");
        }
    }

    #[test]
    fn sim_transport_round_trips() {
        use sim::{Characteristic, Reply, Request};
        let mut b = [0u8; 96];
        for r in [
            Request::Connect,
            Request::Setpoint(&[1, 2, 3]),
            Request::Control(9),
            Request::Read(Characteristic::Network),
            Request::Echo(&[]),
            Request::Disconnect,
        ] {
            let n = r.encode(&mut b).unwrap();
            assert_eq!(Request::decode(&b[..n]), Ok(r));
        }
        for r in [
            Reply::Connected(true),
            Reply::Control(sim::ATT_VALUE_NOT_ALLOWED),
            Reply::Read(Characteristic::Status, &[3, 1, 0]),
            Reply::Echo(&[7; 4]),
            Reply::Disconnected,
            Reply::Dropped,
        ] {
            let n = r.encode(&mut b).unwrap();
            assert_eq!(Reply::decode(&b[..n]), Ok(r));
        }
        assert_eq!(Request::decode(&[3, 1, 2]), Err(DecodeError::Length));
        assert_eq!(Request::decode(&[4, 9]), Err(DecodeError::UnknownCode));
        assert_eq!(Request::decode(&[7]), Err(DecodeError::UnknownCode));
    }
}
