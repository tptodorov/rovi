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
}
