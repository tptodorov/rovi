//! One capabilities snapshot, encoded identically for BLE/UDP and ROS controllers.

/// Upper bound for the CDR `std_msgs/String`, including encapsulation and NUL.
pub const CDR_MAX: usize = 384;

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
    /// Version-1 BLE characteristic / UDP HELLO_ACK payload.
    pub fn encode(&self) -> [u8; 27] {
        let mut out = [0; 27];
        out[0] = self.version;
        for (i, value) in [
            self.wheels_radius,
            self.center_projection_sum,
            self.max_vx,
            self.max_vy,
            self.max_wz,
        ]
        .iter()
        .enumerate()
        {
            out[1 + i * 4..5 + i * 4].copy_from_slice(&value.to_le_bytes());
        }
        out[21..23].copy_from_slice(&self.lease_ms.to_le_bytes());
        out[23..25].copy_from_slice(&self.claim_ms.to_le_bytes());
        out[25..27].copy_from_slice(&self.reclaim_ms.to_le_bytes());
        out
    }

    /// ROS `std_msgs/String` CDR. Returns `None` without changing an undersized output.
    pub fn encode_cdr(&self, out: &mut [u8]) -> Option<usize> {
        use core::fmt::Write;
        let mut text = crate::rmw::Key::<{ CDR_MAX - 9 }>::new();
        // Print the exact f32 as an f64: "0.7" would exceed the enforced f32 limit when a
        // controller parses it as f64. Scientific notation also bounds extreme finite values.
        write!(text, "version={} wheels_radius={:e} sum_of_robot_center_projection_on_X_Y_axis={:e} max_vx={:e} max_vy={:e} max_wz={:e} lease_ms={} claim_ms={} reclaim_ms={}",
            self.version, f64::from(self.wheels_radius), f64::from(self.center_projection_sum),
            f64::from(self.max_vx), f64::from(self.max_vy), f64::from(self.max_wz),
            self.lease_ms, self.claim_ms, self.reclaim_ms).ok()?;
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
    use std::collections::BTreeMap;
    use std::string::String;

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

    fn fields(caps: Capabilities) -> BTreeMap<String, String> {
        let mut out = [0; CDR_MAX];
        let n = caps.encode_cdr(&mut out).expect("bounded capabilities CDR");
        assert_eq!(out[..4], [0, 1, 0, 0]);
        assert_eq!(
            u32::from_le_bytes(out[4..8].try_into().unwrap()) as usize,
            n - 8
        );
        assert_eq!(out[n - 1], 0);
        let text = core::str::from_utf8(&out[8..n - 1]).unwrap();
        let mut fields = BTreeMap::new();
        for token in text.split(' ') {
            let (key, value) = token.split_once('=').unwrap();
            assert!(
                fields.insert(key.into(), value.into()).is_none(),
                "duplicate key"
            );
        }
        fields
    }

    #[test]
    fn binary_capabilities_keep_the_version_one_golden_bytes() {
        // Independently packed as <B5f3H by the existing Python wire codec.
        let golden = [
            0x01, 0x8f, 0xc2, 0xf5, 0x3c, 0x5c, 0x8f, 0x02, 0x3e, 0x33, 0x33, 0x33, 0x3f, 0x9a,
            0x99, 0x99, 0x3e, 0x00, 0x00, 0x40, 0x40, 0xe8, 0x03, 0x10, 0x27, 0x88, 0x13,
        ];
        assert_eq!(CAPS.encode(), golden);
    }

    #[test]
    fn ros_capabilities_have_exactly_the_nine_keys_and_units() {
        let f = fields(CAPS);
        assert_eq!(f.len(), 9);
        for (key, value) in [
            ("version", 1),
            ("lease_ms", 1000),
            ("claim_ms", 10000),
            ("reclaim_ms", 5000),
        ] {
            assert_eq!(f[key].parse::<u32>().unwrap(), value);
        }
        for (key, value) in [
            ("wheels_radius", CAPS.wheels_radius),
            (
                "sum_of_robot_center_projection_on_X_Y_axis",
                CAPS.center_projection_sum,
            ),
            ("max_vx", CAPS.max_vx),
            ("max_vy", CAPS.max_vy),
            ("max_wz", CAPS.max_wz),
        ] {
            assert_eq!(f[key].parse::<f64>().unwrap(), f64::from(value), "{key}");
        }
        assert!(!f.contains_key("session"));
    }

    #[test]
    fn ros_floats_round_trip_even_when_a_short_decimal_would_exceed_the_limit() {
        for value in [
            0.0,
            f32::from_bits(1),
            f32::MIN_POSITIVE,
            0.1,
            0.3,
            0.7,
            f32::MAX,
        ] {
            let caps = Capabilities {
                wheels_radius: value,
                center_projection_sum: value,
                max_vx: value,
                max_vy: value,
                max_wz: value,
                ..CAPS
            };
            let f = fields(caps);
            for key in [
                "wheels_radius",
                "sum_of_robot_center_projection_on_X_Y_axis",
                "max_vx",
                "max_vy",
                "max_wz",
            ] {
                assert_eq!(f[key].parse::<f32>().unwrap().to_bits(), value.to_bits());
                assert_eq!(f[key].parse::<f64>().unwrap(), f64::from(value));
            }
        }
    }

    #[test]
    fn capabilities_cdr_rejects_small_buffers_without_partial_output() {
        let mut full = [0; CDR_MAX];
        let n = CAPS.encode_cdr(&mut full).unwrap();
        for size in [0, 8, n - 1] {
            let mut short = [0x55; CDR_MAX];
            assert_eq!(CAPS.encode_cdr(&mut short[..size]), None);
            assert!(short.iter().all(|b| *b == 0x55));
        }
        assert_eq!(CAPS.encode_cdr(&mut full[..n]), Some(n));
    }
}
