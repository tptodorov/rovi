//! UDP framing: `RoviHeader { magic "RV", version u8, kind u8, session u32, seq u32 }` plus an
//! optional payload (M8 design).
//! `ECHO` is answered with the same header and payload, from any peer.
//!
//! [`parse`]/[`write`] handle the header only; [`ToCar`] and [`FromCar`] are the typed messages
//! each side sends (design D3), with payload validation.

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
    /// Round-trip probe: the payload comes back unchanged (`device::echo`).
    Echo = 9,
    /// Mode command from the owner: payload 1 = Home, 2 = Car network (ADR-0016).
    Mode = 10,
    /// Observation for any peer: capabilities, status and network information (ADR-0016).
    Info = 11,
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
            9 => Self::Echo,
            10 => Self::Mode,
            11 => Self::Info,
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

use crate::capabilities::{self, Capabilities};
use crate::network::{self, Command, NetworkInfo};
use crate::setpoint::Setpoint;
use crate::status::{self, Status};
use crate::{DecodeError, EncodeError, ECHO_MAX};

/// `INFO` reply payload: capabilities, Status and network information.
pub const INFO_LEN: usize = capabilities::LEN + status::LEN + network::LEN;
/// Room for the largest frame either side sends.
pub const MAX_FRAME: usize = HEADER_LEN + crate::setpoint::MIN_LEN + crate::setpoint::MAX_FRAME_ID;

/// A frame's session and sequence with its typed message.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame<M> {
    pub session: u32,
    pub seq: u32,
    pub message: M,
}

/// Messages a client sends to the car.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToCar<'a> {
    Hello,
    Setpoint(Setpoint),
    Arm,
    Stop,
    Bye,
    Echo(&'a [u8]),
    Mode(Command),
    /// Observation query; any peer may send it.
    Info,
}

/// The `INFO` answer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Info {
    pub capabilities: Capabilities,
    pub status: Status,
    pub network: NetworkInfo,
}

/// Messages the car sends to a client.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FromCar<'a> {
    HelloAck(Capabilities),
    /// A reply (the request's sequence) or unsolicited (sequence 0).
    Status(Status),
    Busy,
    Echo(&'a [u8]),
    Info(Info),
}

fn header(datagram: &[u8]) -> Result<(Header, &[u8]), DecodeError> {
    if datagram.len() < HEADER_LEN {
        return Err(DecodeError::Length);
    }
    if datagram[..2] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    if datagram[2] != VERSION {
        return Err(DecodeError::UnsupportedVersion);
    }
    parse(datagram).ok_or(DecodeError::UnknownCode)
}

fn empty(p: &[u8]) -> Result<(), DecodeError> {
    p.is_empty().then_some(()).ok_or(DecodeError::Length)
}

fn echo(p: &[u8]) -> Result<&[u8], DecodeError> {
    (p.len() <= ECHO_MAX)
        .then_some(p)
        .ok_or(DecodeError::Length)
}

fn put(
    out: &mut [u8],
    kind: Kind,
    session: u32,
    seq: u32,
    payload: &[u8],
) -> Result<usize, EncodeError> {
    write(out, Header { kind, session, seq }, payload).ok_or(EncodeError::BufferTooSmall)
}

impl<'a> Frame<ToCar<'a>> {
    pub fn decode(datagram: &'a [u8]) -> Result<Self, DecodeError> {
        let (h, p) = header(datagram)?;
        let message = match h.kind {
            Kind::Hello => empty(p).map(|_| ToCar::Hello)?,
            Kind::Setpoint => ToCar::Setpoint(Setpoint::decode(p)?),
            Kind::Arm => empty(p).map(|_| ToCar::Arm)?,
            Kind::Stop => empty(p).map(|_| ToCar::Stop)?,
            Kind::Bye => empty(p).map(|_| ToCar::Bye)?,
            Kind::Echo => ToCar::Echo(echo(p)?),
            Kind::Mode => match *p {
                [code] => ToCar::Mode(Command::from_code(code).ok_or(DecodeError::UnknownCode)?),
                _ => return Err(DecodeError::Length),
            },
            Kind::Info => empty(p).map(|_| ToCar::Info)?,
            Kind::HelloAck | Kind::Status | Kind::Busy => return Err(DecodeError::UnknownCode),
        };
        Ok(Self {
            session: h.session,
            seq: h.seq,
            message,
        })
    }

    pub fn encode(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let (s, q) = (self.session, self.seq);
        match self.message {
            ToCar::Hello => put(out, Kind::Hello, s, q, &[]),
            ToCar::Setpoint(sp) => {
                let mut b = [0; crate::setpoint::MIN_LEN];
                let n = sp.encode(&mut b)?;
                put(out, Kind::Setpoint, s, q, &b[..n])
            }
            ToCar::Arm => put(out, Kind::Arm, s, q, &[]),
            ToCar::Stop => put(out, Kind::Stop, s, q, &[]),
            ToCar::Bye => put(out, Kind::Bye, s, q, &[]),
            ToCar::Echo(p) => put(
                out,
                Kind::Echo,
                s,
                q,
                echo(p).map_err(|_| EncodeError::Invalid)?,
            ),
            ToCar::Mode(c) => put(out, Kind::Mode, s, q, &[c.code()]),
            ToCar::Info => put(out, Kind::Info, s, q, &[]),
        }
    }
}

impl<'a> Frame<FromCar<'a>> {
    pub fn decode(datagram: &'a [u8]) -> Result<Self, DecodeError> {
        let (h, p) = header(datagram)?;
        let message = match h.kind {
            Kind::HelloAck => FromCar::HelloAck(Capabilities::decode(p)?),
            Kind::Status => FromCar::Status(Status::decode(p)?),
            Kind::Busy => empty(p).map(|_| FromCar::Busy)?,
            Kind::Echo => FromCar::Echo(echo(p)?),
            Kind::Info => {
                if p.len() != INFO_LEN {
                    return Err(DecodeError::Length);
                }
                let (c, rest) = p.split_at(capabilities::LEN);
                let (st, net) = rest.split_at(status::LEN);
                FromCar::Info(Info {
                    capabilities: Capabilities::decode(c)?,
                    status: Status::decode(st)?,
                    network: NetworkInfo::decode(net)?,
                })
            }
            Kind::Hello | Kind::Setpoint | Kind::Arm | Kind::Stop | Kind::Bye | Kind::Mode => {
                return Err(DecodeError::UnknownCode)
            }
        };
        Ok(Self {
            session: h.session,
            seq: h.seq,
            message,
        })
    }

    pub fn encode(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let (s, q) = (self.session, self.seq);
        match self.message {
            FromCar::HelloAck(c) => put(out, Kind::HelloAck, s, q, &c.encode()),
            FromCar::Status(st) => put(out, Kind::Status, s, q, &st.encode()),
            FromCar::Busy => put(out, Kind::Busy, s, q, &[]),
            FromCar::Echo(p) => put(
                out,
                Kind::Echo,
                s,
                q,
                echo(p).map_err(|_| EncodeError::Invalid)?,
            ),
            FromCar::Info(i) => {
                let mut b = [0; INFO_LEN];
                b[..capabilities::LEN].copy_from_slice(&i.capabilities.encode());
                b[capabilities::LEN..capabilities::LEN + status::LEN]
                    .copy_from_slice(&i.status.encode());
                b[capabilities::LEN + status::LEN..].copy_from_slice(&i.network.encode());
                put(out, Kind::Info, s, q, &b)
            }
        }
    }
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

    #[test]
    fn typed_frames_round_trip_and_keep_direction() {
        let sp = Setpoint {
            sec: 0,
            nanosec: 0,
            vx: 0.1,
            vy: 0.0,
            wz: -0.3,
        };
        let nonce = [7u8; 12];
        let mut b = [0u8; MAX_FRAME];
        for m in [
            ToCar::Hello,
            ToCar::Setpoint(sp),
            ToCar::Arm,
            ToCar::Stop,
            ToCar::Bye,
            ToCar::Echo(&nonce),
            ToCar::Mode(Command::CarNetwork),
            ToCar::Info,
        ] {
            let f = Frame {
                session: 42,
                seq: u32::MAX,
                message: m,
            };
            let n = f.encode(&mut b).unwrap();
            assert_eq!(Frame::<ToCar>::decode(&b[..n]), Ok(f));
            assert!(
                Frame::<FromCar>::decode(&b[..n]).is_err() || matches!(m, ToCar::Echo(_)),
                "{m:?} is not a car message"
            );
        }
        let caps = Capabilities {
            version: 1,
            wheels_radius: 0.03,
            center_projection_sum: 0.1275,
            max_vx: 0.7,
            max_vy: 0.3,
            max_wz: 3.0,
            lease_ms: 1000,
            claim_ms: 10000,
            reclaim_ms: 5000,
        };
        let status = Status {
            state: crate::State::Claimed,
            error: None,
            session: 0,
        };
        let network = NetworkInfo {
            mode: crate::NetMode::Home,
            pending: None,
            reason: crate::Reason::None,
            ip: [1, 2, 3, 4],
        };
        for m in [
            FromCar::HelloAck(caps),
            FromCar::Status(status),
            FromCar::Busy,
            FromCar::Echo(&nonce),
            FromCar::Info(Info {
                capabilities: caps,
                status,
                network,
            }),
        ] {
            let f = Frame {
                session: 42,
                seq: 0,
                message: m,
            };
            let n = f.encode(&mut b).unwrap();
            assert_eq!(Frame::<FromCar>::decode(&b[..n]), Ok(f));
        }
    }

    #[test]
    fn typed_decoding_names_the_problem() {
        let mut b = [0u8; MAX_FRAME];
        let n = Frame {
            session: 1,
            seq: 1,
            message: ToCar::Hello,
        }
        .encode(&mut b)
        .unwrap();
        assert_eq!(Frame::<ToCar>::decode(&b[..11]), Err(DecodeError::Length));
        let mut x = b;
        x[0] = b'X';
        assert_eq!(Frame::<ToCar>::decode(&x[..n]), Err(DecodeError::BadMagic));
        x = b;
        x[2] = 2;
        assert_eq!(
            Frame::<ToCar>::decode(&x[..n]),
            Err(DecodeError::UnsupportedVersion)
        );
        x = b;
        x[3] = 12;
        assert_eq!(
            Frame::<ToCar>::decode(&x[..n]),
            Err(DecodeError::UnknownCode)
        );
        assert_eq!(
            Frame {
                session: 1,
                seq: 1,
                message: ToCar::Echo(&[0; 33])
            }
            .encode(&mut b),
            Err(EncodeError::Invalid)
        );
    }
}
