//! The `rmw_zenoh` (Jazzy) wire contract the car implements for ROS 2: topic keys, liveliness
//! tokens, the entity gid and the 33-byte sample attachment (S2 spike, M8 design).

use core::fmt::{self, Write};

/// A fixed-capacity key expression, built without an allocator.
#[derive(Clone, Copy)]
pub struct Key<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for Key<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Key<N> {
    pub fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Write for Key<N> {
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

/// A ROS 2 topic with its type, as `rmw_zenoh` names it.
#[derive(Clone, Copy, Debug)]
pub struct Topic {
    /// Without the leading slash, e.g. `cmd_vel` or `rovi/arm`.
    pub name: &'static str,
    pub ty: &'static str,
    /// `RIHS01_...` type hash; fixed per message type and ROS distro (Jazzy).
    pub hash: &'static str,
}

pub const TWIST_STAMPED: (&str, &str) = (
    "geometry_msgs::msg::dds_::TwistStamped_",
    "RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64",
);
pub const BOOL: (&str, &str) = (
    "std_msgs::msg::dds_::Bool_",
    "RIHS01_feb91e995ff9ebd09c0cb3d2aed18b11077585839fb5db80193b62d74528f6c9",
);
pub const STRING: (&str, &str) = (
    "std_msgs::msg::dds_::String_",
    "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
);

/// QoS as `rmw_zenoh` encodes it in liveliness keys: best effort, keep last 1 / 5, and the
/// default (reliable, keep last 10).
pub const QOS_BEST_EFFORT_1: &str = "2::,1:,:,:,,";
pub const QOS_BEST_EFFORT_5: &str = "2::,5:,:,:,,";
pub const QOS_DEFAULT_10: &str = "::,10:,:,:,,";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Publisher,
    Subscription,
}

/// The data key: `<domain>/<topic>/<type>/<hash>`.
pub fn topic_key(domain: u32, topic: &Topic) -> Option<Key<192>> {
    let mut k = Key::new();
    write!(k, "{domain}/{}/{}/{}", topic.name, topic.ty, topic.hash).ok()?;
    Some(k)
}

/// The liveliness token of node `name` (the node entity has id 0).
pub fn node_token(domain: u32, zid: &str, name: &str) -> Option<Key<128>> {
    let mut k = Key::new();
    write!(k, "@ros2_lv/{domain}/{zid}/0/0/NN/%/%/{name}").ok()?;
    Some(k)
}

/// The liveliness token of a publisher or subscription of `node`. `id` is unique per entity
/// within the session (1, 2, ...). Slashes in the topic name are mangled to `%`.
pub fn endpoint_token(
    domain: u32,
    zid: &str,
    id: u32,
    endpoint: Endpoint,
    node: &str,
    topic: &Topic,
    qos: &str,
) -> Option<Key<256>> {
    let mut k = Key::new();
    let kind = match endpoint {
        Endpoint::Publisher => "MP",
        Endpoint::Subscription => "MS",
    };
    write!(k, "@ros2_lv/{domain}/{zid}/0/{id}/{kind}/%/%/{node}/%").ok()?;
    for c in topic.name.chars() {
        k.write_char(if c == '/' { '%' } else { c }).ok()?;
    }
    write!(k, "/{}/{}/{qos}", topic.ty, topic.hash).ok()?;
    Some(k)
}

/// The entity gid: XXH3-128 of its liveliness key, low 64 bits first. Publishers put it in
/// their sample attachment, and ROS tools show it as `GID`.
pub fn gid(liveliness_key: &str) -> [u8; 16] {
    let h = xxhash_rust::xxh3::xxh3_128(liveliness_key.as_bytes());
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&(h as u64).to_le_bytes());
    out[8..].copy_from_slice(&((h >> 64) as u64).to_le_bytes());
    out
}

pub const ATTACHMENT_LEN: usize = 33;

/// The `rmw_zenoh` sample attachment: zenoh `ext::Serializer` of (seq i64, timestamp i64,
/// gid as a LEB128-length-prefixed byte array).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub seq: i64,
    /// Nanoseconds since the Unix epoch, or 0 if unknown.
    pub timestamp: i64,
    pub gid: [u8; 16],
}

impl Attachment {
    pub fn encode(&self) -> [u8; ATTACHMENT_LEN] {
        let mut out = [0u8; ATTACHMENT_LEN];
        out[..8].copy_from_slice(&self.seq.to_le_bytes());
        out[8..16].copy_from_slice(&self.timestamp.to_le_bytes());
        out[16] = 16;
        out[17..].copy_from_slice(&self.gid);
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != ATTACHMENT_LEN || bytes[16] != 16 {
            return None;
        }
        Some(Self {
            seq: i64::from_le_bytes(bytes[..8].try_into().ok()?),
            timestamp: i64::from_le_bytes(bytes[8..16].try_into().ok()?),
            gid: bytes[17..].try_into().ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured from a real Jazzy `rmw_zenoh_cpp` graph (see MILESTONE.md results).
    const ZID: &str = "9ad20db4c929c8189ca587de2f90c2e2";
    const CMD_VEL: Topic = Topic {
        name: "cmd_vel",
        ty: TWIST_STAMPED.0,
        hash: TWIST_STAMPED.1,
    };
    const CMD_VEL_SUB: &str = "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/0/1/MS/%/%/rovi/%cmd_vel/geometry_msgs::msg::dds_::TwistStamped_/RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64/2::,1:,:,:,,";

    #[test]
    fn node_and_subscription_tokens_match_ros() {
        assert_eq!(
            node_token(0, ZID, "rovi").unwrap().as_str(),
            "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/0/0/NN/%/%/rovi"
        );
        let k = endpoint_token(
            0,
            ZID,
            1,
            Endpoint::Subscription,
            "rovi",
            &CMD_VEL,
            QOS_BEST_EFFORT_1,
        )
        .unwrap();
        assert_eq!(k.as_str(), CMD_VEL_SUB);
    }

    #[test]
    fn nested_topic_names_are_mangled_in_tokens_but_not_in_data_keys() {
        let arm = Topic {
            name: "rovi/arm",
            ty: BOOL.0,
            hash: BOOL.1,
        };
        let t = endpoint_token(
            0,
            ZID,
            2,
            Endpoint::Subscription,
            "rovi",
            &arm,
            QOS_BEST_EFFORT_5,
        )
        .unwrap();
        assert!(t.as_str().ends_with("/MS/%/%/rovi/%rovi%arm/std_msgs::msg::dds_::Bool_/RIHS01_feb91e995ff9ebd09c0cb3d2aed18b11077585839fb5db80193b62d74528f6c9/2::,5:,:,:,,"));
        assert!(topic_key(0, &arm)
            .unwrap()
            .as_str()
            .starts_with("0/rovi/arm/std_msgs::msg::dds_::Bool_/RIHS01_feb9"));
    }

    #[test]
    fn publisher_token_uses_mp() {
        let st = Topic {
            name: "rovi/status",
            ty: STRING.0,
            hash: STRING.1,
        };
        let t =
            endpoint_token(0, ZID, 3, Endpoint::Publisher, "rovi", &st, QOS_DEFAULT_10).unwrap();
        assert!(t
            .as_str()
            .contains("/0/3/MP/%/%/rovi/%rovi%status/std_msgs::msg::dds_::String_/"));
        assert!(t.as_str().ends_with("/::,10:,:,:,,"));
    }

    #[test]
    fn data_key_matches_ros() {
        assert_eq!(
            topic_key(0, &CMD_VEL).unwrap().as_str(),
            "0/cmd_vel/geometry_msgs::msg::dds_::TwistStamped_/RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64"
        );
    }

    #[test]
    fn gid_matches_ros_topic_info() {
        // `ros2 topic info -v /cmd_vel` printed GID 1b.8f.09.d3.c8.5e.e5.5f.09.71.e4.08.ef.57.47.8a
        assert_eq!(
            gid(CMD_VEL_SUB),
            [
                0x1b, 0x8f, 0x09, 0xd3, 0xc8, 0x5e, 0xe5, 0x5f, 0x09, 0x71, 0xe4, 0x08, 0xef, 0x57,
                0x47, 0x8a
            ]
        );
    }

    #[test]
    fn key_overflow_is_an_error_not_a_panic() {
        let long = "x".repeat(300);
        assert!(node_token(0, ZID, &long).is_none());
    }

    #[test]
    fn attachment_round_trip_and_layout() {
        let a = Attachment {
            seq: 7,
            timestamp: 0,
            gid: [9; 16],
        };
        let b = a.encode();
        assert_eq!(b[..8], 7i64.to_le_bytes());
        assert_eq!(b[16], 16, "LEB128 length of the gid");
        assert_eq!(Attachment::decode(&b), Some(a));
        assert_eq!(Attachment::decode(&b[..32]), None);
        let mut bad = b;
        bad[16] = 15;
        assert_eq!(Attachment::decode(&bad), None);
    }
}
