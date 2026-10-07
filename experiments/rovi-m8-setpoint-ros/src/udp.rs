//! UDP framing: `RoviHeader { magic "RV", version u8, kind u8, session u32, seq u32 }` plus an
//! optional payload (M8 design).

pub const VERSION: u8 = 1;
pub const HEADER_LEN: usize = 12;
const MAGIC: [u8; 2] = *b"RV";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Hello = 1,
    HelloAck = 2,
    Setpoint = 3,
    Arm = 4,
    Stop = 5,
    Bye = 6,
    Status = 7,
    Busy = 8,
}

impl Kind {
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Hello,
            2 => Self::HelloAck,
            3 => Self::Setpoint,
            4 => Self::Arm,
            5 => Self::Stop,
            6 => Self::Bye,
            7 => Self::Status,
            8 => Self::Busy,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub kind: Kind,
    pub session: u32,
    pub seq: u32,
}

/// Splits a datagram into its header and payload. `None` for anything that isn't a valid frame.
pub fn parse(datagram: &[u8]) -> Option<(Header, &[u8])> {
    if datagram.len() < HEADER_LEN || datagram[..2] != MAGIC || datagram[2] != VERSION {
        return None;
    }
    let header = Header {
        kind: Kind::from_u8(datagram[3])?,
        session: u32::from_le_bytes(datagram[4..8].try_into().ok()?),
        seq: u32::from_le_bytes(datagram[8..12].try_into().ok()?),
    };
    Some((header, &datagram[HEADER_LEN..]))
}

/// Writes a frame into `out` and returns its length, or `None` if it doesn't fit.
pub fn write(out: &mut [u8], header: Header, payload: &[u8]) -> Option<usize> {
    let len = HEADER_LEN + payload.len();
    let out = out.get_mut(..len)?;
    out[..2].copy_from_slice(&MAGIC);
    out[2] = VERSION;
    out[3] = header.kind as u8;
    out[4..8].copy_from_slice(&header.session.to_le_bytes());
    out[8..12].copy_from_slice(&header.seq.to_le_bytes());
    out[HEADER_LEN..].copy_from_slice(payload);
    Some(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut b = [0u8; 32];
        let h = Header {
            kind: Kind::Setpoint,
            session: 7,
            seq: 0x0102_0304,
        };
        let n = write(&mut b, h, &[9, 8, 7]).unwrap();
        assert_eq!(n, 15);
        assert_eq!(&b[..4], b"RV\x01\x03");
        assert_eq!(parse(&b[..n]), Some((h, &[9u8, 8, 7][..])));
    }

    #[test]
    fn rejects_bad_frames() {
        let mut b = [0u8; 12];
        let h = Header {
            kind: Kind::Hello,
            session: 0,
            seq: 1,
        };
        write(&mut b, h, &[]).unwrap();
        assert!(parse(&b).is_some());
        assert_eq!(parse(&b[..11]), None, "short");
        let mut x = b;
        x[0] = b'X';
        assert_eq!(parse(&x), None, "magic");
        let mut x = b;
        x[2] = 2;
        assert_eq!(parse(&x), None, "version");
        let mut x = b;
        x[3] = 99;
        assert_eq!(parse(&x), None, "kind");
    }

    #[test]
    fn write_rejects_small_buffer() {
        let h = Header {
            kind: Kind::Bye,
            session: 1,
            seq: 0,
        };
        assert_eq!(write(&mut [0u8; 11], h, &[]), None);
    }
}
