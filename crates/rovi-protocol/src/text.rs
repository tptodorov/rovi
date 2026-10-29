//! ROS `std_msgs/String` CDR and the canonical logfmt text Rovi publishes on it.

use crate::{DecodeError, EncodeError};
use core::fmt::{self, Write};

const ENCAPSULATION: [u8; 4] = [0x00, 0x01, 0x00, 0x00]; // CDR little-endian

/// A fixed-capacity string for building text without an allocator.
pub struct Text<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for Text<N> {
    fn default() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> Text<N> {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Write for Text<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        self.buf
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// `text` as `std_msgs/String` CDR. Writes nothing if `out` is too small.
pub fn encode_string(text: &str, out: &mut [u8]) -> Result<usize, EncodeError> {
    let bytes = text.as_bytes();
    let n = 9 + bytes.len();
    let out = out.get_mut(..n).ok_or(EncodeError::BufferTooSmall)?;
    out[..4].copy_from_slice(&ENCAPSULATION);
    out[4..8].copy_from_slice(&(bytes.len() as u32 + 1).to_le_bytes());
    out[8..n - 1].copy_from_slice(bytes);
    out[n - 1] = 0;
    Ok(n)
}

/// The text of a `std_msgs/String` CDR sample.
pub fn decode_string(b: &[u8]) -> Result<&str, DecodeError> {
    if b.len() < 9 {
        return Err(DecodeError::Length);
    }
    if b[..4] != ENCAPSULATION {
        return Err(DecodeError::BadText);
    }
    let len = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
    if len == 0 || b.len() != 8 + len || b[b.len() - 1] != 0 {
        return Err(DecodeError::Length);
    }
    core::str::from_utf8(&b[8..b.len() - 1]).map_err(|_| DecodeError::BadText)
}

/// Reads `key=value` fields in their canonical order; anything else is [`DecodeError::BadText`].
pub struct Fields<'a> {
    rest: core::str::Split<'a, char>,
}

impl<'a> Fields<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            rest: text.split(' '),
        }
    }

    /// The value of the next field, which must be named `key`.
    pub fn next(&mut self, key: &str) -> Result<&'a str, DecodeError> {
        let (k, v) = self
            .rest
            .next()
            .and_then(|t| t.split_once('='))
            .ok_or(DecodeError::BadText)?;
        if k != key || v.is_empty() {
            return Err(DecodeError::BadText);
        }
        Ok(v)
    }

    pub fn parse<T: core::str::FromStr>(&mut self, key: &str) -> Result<T, DecodeError> {
        self.next(key)?.parse().map_err(|_| DecodeError::BadText)
    }

    pub fn end(mut self) -> Result<(), DecodeError> {
        self.rest
            .next()
            .map_or(Ok(()), |_| Err(DecodeError::BadText))
    }
}
