//! `geometry_msgs/TwistStamped` as a Rovi setpoint, carried as its CDR encoding (ADR-0006).

/// Longest `frame_id` accepted, so a setpoint stays small and bounded.
pub const MAX_FRAME_ID: usize = 64;
/// CDR size with an empty `frame_id`: 4 encapsulation + 8 stamp + 8 string + 48 twist.
pub const MIN_LEN: usize = 68;
const ENCAPSULATION: [u8; 4] = [0x00, 0x01, 0x00, 0x00]; // CDR little-endian

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Setpoint {
    pub sec: i32,
    pub nanosec: u32,
    /// m/s, forward.
    pub vx: f64,
    /// m/s, left.
    pub vy: f64,
    /// rad/s, counter-clockwise.
    pub wz: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    BadEncapsulation,
    BadFrameId,
    /// `linear.z`, `angular.x` or `angular.y` is not zero.
    OutOfPlane,
    NotFinite,
}

impl Setpoint {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::Truncated);
        }
        if bytes[..4] != ENCAPSULATION {
            return Err(DecodeError::BadEncapsulation);
        }
        // Alignment is relative to the start of the data, after the encapsulation header.
        let body = &bytes[4..];
        if body.len() < 12 {
            return Err(DecodeError::Truncated);
        }
        let sec = i32::from_le_bytes(body[0..4].try_into().unwrap());
        let nanosec = u32::from_le_bytes(body[4..8].try_into().unwrap());
        // A CDR string carries its NUL in the length. Padding after it is not zeroed by ROS.
        let slen = u32::from_le_bytes(body[8..12].try_into().unwrap()) as usize;
        if slen == 0 || slen > MAX_FRAME_ID + 1 || body.len() < 12 + slen || body[11 + slen] != 0 {
            return Err(DecodeError::BadFrameId);
        }
        let twist = (12 + slen).next_multiple_of(8);
        if body.len() < twist + 48 {
            return Err(DecodeError::Truncated);
        }
        let f = |i: usize| f64::from_le_bytes(body[twist + 8 * i..twist + 8 * i + 8].try_into().unwrap());
        let [vx, vy, lz, ax, ay, wz] = [f(0), f(1), f(2), f(3), f(4), f(5)];
        if lz != 0.0 || ax != 0.0 || ay != 0.0 {
            return Err(DecodeError::OutOfPlane);
        }
        if !(vx.is_finite() && vy.is_finite() && wz.is_finite()) {
            return Err(DecodeError::NotFinite);
        }
        Ok(Self { sec, nanosec, vx, vy, wz })
    }

    /// Encodes with an empty `frame_id`. Returns the length, or `None` if `out` is too small.
    pub fn encode(&self, out: &mut [u8]) -> Option<usize> {
        let out = out.get_mut(..MIN_LEN)?;
        out.fill(0);
        out[..4].copy_from_slice(&ENCAPSULATION);
        out[4..8].copy_from_slice(&self.sec.to_le_bytes());
        out[8..12].copy_from_slice(&self.nanosec.to_le_bytes());
        out[12..16].copy_from_slice(&1u32.to_le_bytes()); // empty string: just the NUL
        // Twist at data offset 16 (abs 20): linear x, y, z, angular x, y, z.
        out[20..28].copy_from_slice(&self.vx.to_le_bytes());
        out[28..36].copy_from_slice(&self.vy.to_le_bytes());
        out[60..68].copy_from_slice(&self.wz.to_le_bytes());
        Some(MIN_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured from ROS 2 Jazzy (rmw_zenoh_cpp 0.2.10) `ros2 topic pub`. Padding bytes are not
    // zero in ROS's output, so the decoder must ignore them.
    const EMPTY_FRAME: &str = "00010000000000000000000001000000005f726f9a9999999999b93f00000000000000000000000000000000000000000000000000000000000000000000000000000000";
    const BASE_LINK: &str = "0001000001000000020000000a000000626173655f6c696e6b003638000000000000e03f000000000000d0bf000000000000000000000000000000000000000000000000000000000000f83f";
    const OUT_OF_PLANE: &str = "00010000000000000000000001000000005f726f000000000000000000000000000000000000000000000040000000000000084000000000000000000000000000000000";

    fn hex(s: &str) -> ([u8; 128], usize) {
        let mut out = [0u8; 128];
        let n = s.len() / 2;
        for i in 0..n {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        (out, n)
    }

    #[test]
    fn decodes_ros_empty_frame_id() {
        let (b, n) = hex(EMPTY_FRAME);
        assert_eq!(n, MIN_LEN);
        let sp = Setpoint::decode(&b[..n]).unwrap();
        assert_eq!(sp, Setpoint { sec: 0, nanosec: 0, vx: 0.1, vy: 0.0, wz: 0.0 });
    }

    #[test]
    fn decodes_ros_named_frame_with_stamp() {
        let (b, n) = hex(BASE_LINK);
        assert_eq!(n, 76);
        let sp = Setpoint::decode(&b[..n]).unwrap();
        assert_eq!(sp, Setpoint { sec: 1, nanosec: 2, vx: 0.5, vy: -0.25, wz: 1.5 });
    }

    #[test]
    fn rejects_out_of_plane_components() {
        let (b, n) = hex(OUT_OF_PLANE);
        assert_eq!(Setpoint::decode(&b[..n]), Err(DecodeError::OutOfPlane));
    }

    #[test]
    fn rejects_truncated_and_bad_encapsulation() {
        let (mut b, n) = hex(EMPTY_FRAME);
        assert_eq!(Setpoint::decode(&b[..n - 1]), Err(DecodeError::Truncated));
        assert_eq!(Setpoint::decode(&[]), Err(DecodeError::Truncated));
        b[1] = 0x00; // big-endian CDR is not supported
        assert_eq!(Setpoint::decode(&b[..n]), Err(DecodeError::BadEncapsulation));
    }

    #[test]
    fn rejects_unterminated_or_oversized_frame_id() {
        let (mut b, n) = hex(BASE_LINK);
        b[25] = 0x01; // the NUL terminator is now a character
        assert_eq!(Setpoint::decode(&b[..n]), Err(DecodeError::BadFrameId));
        let (mut b, n) = hex(BASE_LINK);
        b[12] = 200; // declared length past the buffer
        assert_eq!(Setpoint::decode(&b[..n]), Err(DecodeError::BadFrameId));
        b[12] = 0; // a CDR string always has at least its NUL
        assert_eq!(Setpoint::decode(&b[..n]), Err(DecodeError::BadFrameId));
    }

    #[test]
    fn rejects_non_finite_velocity() {
        let sp = Setpoint { sec: 0, nanosec: 0, vx: f64::NAN, vy: 0.0, wz: 0.0 };
        let mut b = [0u8; MIN_LEN];
        sp.encode(&mut b).unwrap();
        assert_eq!(Setpoint::decode(&b), Err(DecodeError::NotFinite));
    }

    #[test]
    fn encode_matches_ros_for_empty_frame_id_up_to_padding() {
        let sp = Setpoint { sec: 0, nanosec: 0, vx: 0.1, vy: 0.0, wz: 0.0 };
        let mut b = [0xAAu8; 80];
        assert_eq!(sp.encode(&mut b), Some(MIN_LEN));
        let (ros, _) = hex(EMPTY_FRAME);
        assert_eq!(&b[..17], &ros[..17]);
        assert_eq!(&b[20..MIN_LEN], &ros[20..MIN_LEN]);
        assert_eq!(&b[17..20], &[0, 0, 0], "padding is zeroed");
        assert_eq!(Setpoint::decode(&b[..MIN_LEN]), Ok(sp));
    }

    #[test]
    fn encode_rejects_short_buffer() {
        let sp = Setpoint { sec: 0, nanosec: 0, vx: 0.0, vy: 0.0, wz: 0.0 };
        assert_eq!(sp.encode(&mut [0u8; MIN_LEN - 1]), None);
    }
}
