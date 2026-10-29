//! Capabilities: geometry, limits and ownership windows the car advertises.
//! Binary: BLE Capabilities characteristic, UDP `HELLO_ACK` and `INFO`. Text: ROS
//! `/rovi/capabilities`.

use crate::text::{self, Fields, Text};
use crate::{DecodeError, EncodeError};
use core::fmt::Write;

pub const LEN: usize = 27;
pub const VERSION: u8 = 1;
/// Upper bound for the ROS `std_msgs/String` CDR, including encapsulation and NUL.
pub const CDR_MAX: usize = 384;

/// The smallest windows any client accepts (design I11): shorter ones cannot be kept alive
/// reliably by the phone over Bluetooth, so no client relies on them.
pub const MIN_LEASE_MS: u16 = 500;
pub const MIN_CLAIM_MS: u16 = 1000;

/// Canonical f32 values and exact integers advertised by a validated device profile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capabilities {
    pub version: u8,
    pub wheels_radius: f32,
    pub center_projection_sum: f32,
    pub max_vx: f32,
    pub max_vy: f32,
    pub max_wz: f32,
    pub lease_ms: u16,
    pub claim_ms: u16,
    pub reclaim_ms: u16,
}

impl Capabilities {
    pub fn encode(&self) -> [u8; LEN] {
        let mut out = [0; LEN];
        out[0] = self.version;
        for (i, v) in self.floats().iter().enumerate() {
            out[1 + i * 4..5 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        out[21..23].copy_from_slice(&self.lease_ms.to_le_bytes());
        out[23..25].copy_from_slice(&self.claim_ms.to_le_bytes());
        out[25..27].copy_from_slice(&self.reclaim_ms.to_le_bytes());
        out
    }

    pub fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        let b: &[u8; LEN] = b.try_into().map_err(|_| DecodeError::Length)?;
        let f = |i: usize| f32::from_le_bytes(b[1 + i * 4..5 + i * 4].try_into().unwrap());
        let u = |at: usize| u16::from_le_bytes([b[at], b[at + 1]]);
        let caps = Self {
            version: b[0],
            wheels_radius: f(0),
            center_projection_sum: f(1),
            max_vx: f(2),
            max_vy: f(3),
            max_wz: f(4),
            lease_ms: u(21),
            claim_ms: u(23),
            reclaim_ms: u(25),
        };
        caps.validate()?;
        Ok(caps)
    }

    /// Version 1, finite positive geometry, finite non-negative limits, non-zero windows.
    pub fn validate(&self) -> Result<(), DecodeError> {
        if self.version != VERSION {
            return Err(DecodeError::UnsupportedVersion);
        }
        if !self.floats().iter().all(|v| v.is_finite()) {
            return Err(DecodeError::NotFinite);
        }
        let geometry = self.wheels_radius > 0.0 && self.center_projection_sum > 0.0;
        let limits = [self.max_vx, self.max_vy, self.max_wz]
            .iter()
            .all(|v| *v >= 0.0);
        let windows = self.lease_ms > 0 && self.claim_ms > 0 && self.reclaim_ms > 0;
        if !(geometry && limits && windows) {
            return Err(DecodeError::OutOfRange);
        }
        Ok(())
    }

    /// Whether the windows meet the single client minimum (design I11).
    pub fn meets_client_minimum(&self) -> bool {
        self.lease_ms >= MIN_LEASE_MS && self.claim_ms >= MIN_CLAIM_MS && self.reclaim_ms > 0
    }

    fn floats(&self) -> [f32; 5] {
        [
            self.wheels_radius,
            self.center_projection_sum,
            self.max_vx,
            self.max_vy,
            self.max_wz,
        ]
    }

    /// ROS `std_msgs/String` CDR. Writes nothing if `out` is too small.
    pub fn encode_cdr(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let mut t = Text::<{ CDR_MAX - 9 }>::default();
        // Print the exact f32 as an f64: "0.7" would exceed the enforced f32 limit when a
        // controller parses it as f64. Scientific notation also bounds extreme finite values.
        write!(t, "version={} wheels_radius={:e} sum_of_robot_center_projection_on_X_Y_axis={:e} max_vx={:e} max_vy={:e} max_wz={:e} lease_ms={} claim_ms={} reclaim_ms={}",
            self.version, f64::from(self.wheels_radius), f64::from(self.center_projection_sum),
            f64::from(self.max_vx), f64::from(self.max_vy), f64::from(self.max_wz),
            self.lease_ms, self.claim_ms, self.reclaim_ms).map_err(|_| EncodeError::BufferTooSmall)?;
        text::encode_string(t.as_str(), out)
    }

    pub fn decode_cdr(b: &[u8]) -> Result<Self, DecodeError> {
        let mut f = Fields::new(text::decode_string(b)?);
        let version = f.parse("version")?;
        let mut float = |key| -> Result<f32, DecodeError> {
            let v: f64 = f.parse(key)?;
            // Only exact f32 values are canonical.
            let narrow = v as f32;
            if f64::from(narrow) != v && !v.is_nan() {
                return Err(DecodeError::BadText);
            }
            Ok(narrow)
        };
        let wheels_radius = float("wheels_radius")?;
        let center_projection_sum = float("sum_of_robot_center_projection_on_X_Y_axis")?;
        let (max_vx, max_vy, max_wz) = (float("max_vx")?, float("max_vy")?, float("max_wz")?);
        let caps = Self {
            version,
            wheels_radius,
            center_projection_sum,
            max_vx,
            max_vy,
            max_wz,
            lease_ms: f.parse("lease_ms")?,
            claim_ms: f.parse("claim_ms")?,
            reclaim_ms: f.parse("reclaim_ms")?,
        };
        f.end()?;
        caps.validate()?;
        Ok(caps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPS: Capabilities = Capabilities {
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

    #[test]
    fn binary_capabilities_keep_the_version_one_golden_bytes() {
        // Independently packed as <B5f3H by the Python wire codec.
        let golden = [
            0x01, 0x8f, 0xc2, 0xf5, 0x3c, 0x5c, 0x8f, 0x02, 0x3e, 0x33, 0x33, 0x33, 0x3f, 0x9a,
            0x99, 0x99, 0x3e, 0x00, 0x00, 0x40, 0x40, 0xe8, 0x03, 0x10, 0x27, 0x88, 0x13,
        ];
        assert_eq!(CAPS.encode(), golden);
        assert_eq!(Capabilities::decode(&golden), Ok(CAPS));
    }

    #[test]
    fn exact_floats_round_trip_through_binary_and_ros_text() {
        for v in [
            f32::from_bits(1),
            f32::MIN_POSITIVE,
            0.1,
            0.3,
            0.7,
            f32::MAX,
        ] {
            let caps = Capabilities {
                wheels_radius: v,
                center_projection_sum: v,
                max_vx: v,
                max_vy: v,
                max_wz: v,
                lease_ms: u16::MAX,
                claim_ms: u16::MAX,
                reclaim_ms: u16::MAX,
                ..CAPS
            };
            let mut out = [0; CDR_MAX];
            let n = caps.encode_cdr(&mut out).unwrap();
            let back = Capabilities::decode_cdr(&out[..n]).unwrap();
            assert_eq!(back.max_vx.to_bits(), v.to_bits());
            assert_eq!(back, caps);
            assert_eq!(Capabilities::decode(&caps.encode()), Ok(caps));
        }
    }

    #[test]
    fn invalid_profiles_are_refused() {
        let bad = |f: fn(&mut Capabilities)| {
            let mut c = CAPS;
            f(&mut c);
            Capabilities::decode(&c.encode())
        };
        assert_eq!(bad(|c| c.version = 2), Err(DecodeError::UnsupportedVersion));
        assert_eq!(bad(|c| c.max_wz = f32::NAN), Err(DecodeError::NotFinite));
        assert_eq!(bad(|c| c.wheels_radius = 0.0), Err(DecodeError::OutOfRange));
        assert_eq!(bad(|c| c.max_vy = -1.0), Err(DecodeError::OutOfRange));
        assert_eq!(bad(|c| c.reclaim_ms = 0), Err(DecodeError::OutOfRange));
        assert_eq!(
            Capabilities::decode(&CAPS.encode()[..26]),
            Err(DecodeError::Length)
        );
    }

    #[test]
    fn client_minimum_is_one_rule() {
        assert!(CAPS.meets_client_minimum());
        assert!(!Capabilities {
            lease_ms: 499,
            ..CAPS
        }
        .meets_client_minimum());
        assert!(!Capabilities {
            claim_ms: 999,
            ..CAPS
        }
        .meets_client_minimum());
        assert!(Capabilities {
            lease_ms: 500,
            claim_ms: 1000,
            ..CAPS
        }
        .meets_client_minimum());
    }

    #[test]
    fn cdr_short_buffers_get_no_partial_output() {
        let mut full = [0; CDR_MAX];
        let n = CAPS.encode_cdr(&mut full).unwrap();
        for size in [0, 8, n - 1] {
            let mut short = [0x55; CDR_MAX];
            assert_eq!(
                CAPS.encode_cdr(&mut short[..size]),
                Err(EncodeError::BufferTooSmall)
            );
            assert!(short.iter().all(|b| *b == 0x55));
        }
    }
}
