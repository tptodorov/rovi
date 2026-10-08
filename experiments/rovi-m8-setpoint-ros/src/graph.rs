//! Which ROS node does a publisher gid belong to? `rmw_zenoh` sample attachments carry only the
//! publisher's gid, a hash of its liveliness key, so the car learns the graph from liveliness
//! tokens and maps each gid back to its node. Controllers are then told apart by node, not by
//! entity: `/cmd_vel` and `/rovi/arm` are separate publishers of one node.

use crate::rmw::gid;

/// A node identity: XXH3-128 of `<zid>/<nid>` from the liveliness key.
pub type NodeId = u128;

/// A fixed-size table of publisher gids; when full the oldest entry is evicted.
pub struct Graph<const N: usize> {
    entries: [Option<([u8; 16], NodeId)>; N],
    next: usize,
}

impl<const N: usize> Default for Graph<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Graph<N> {
    pub fn new() -> Self {
        Self {
            entries: [None; N],
            next: 0,
        }
    }

    /// Learns from a liveliness token key (`alive`) or forgets it (a dropped token). Only
    /// publisher (`MP`) tokens matter, since only publishers' samples are attributed.
    pub fn on_token(&mut self, key: &str, alive: bool) {
        let Some(node) = publisher_node(key) else {
            return;
        };
        let g = gid(key);
        let slot = self
            .entries
            .iter()
            .position(|e| e.is_some_and(|(k, _)| k == g));
        match (alive, slot) {
            (true, Some(_)) => {}
            (true, None) => {
                self.entries[self.next % N] = Some((g, node));
                self.next = (self.next + 1) % N;
            }
            (false, Some(i)) => self.entries[i] = None,
            (false, None) => {}
        }
    }

    pub fn node_of(&self, publisher_gid: &[u8; 16]) -> Option<NodeId> {
        self.entries
            .iter()
            .flatten()
            .find(|(g, _)| g == publisher_gid)
            .map(|(_, n)| *n)
    }
}

/// `@ros2_lv/<domain>/<zid>/<nid>/<id>/MP/...` gives the node `<zid>/<nid>`.
fn publisher_node(key: &str) -> Option<NodeId> {
    let mut p = key.splitn(7, '/');
    let (admin, _domain, zid, nid, _id, entity) = (
        p.next()?,
        p.next()?,
        p.next()?,
        p.next()?,
        p.next()?,
        p.next()?,
    );
    if admin != "@ros2_lv" || entity != "MP" {
        return None;
    }
    // zid and nid are adjacent in the key, so hash that slice.
    let start = key.find(zid)?;
    let node = &key[start..start + zid.len() + 1 + nid.len()];
    Some(xxhash_rust::xxh3::xxh3_128(node.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Publisher tokens of two entities of one ROS node, and of another node, captured from a
    // real Jazzy graph.
    const PARAM_EVENTS: &str = "@ros2_lv/0/943f9ebc32acd3e072bfda12ce8c7ed2/0/2/MP/%/%/_ros2cli_31/%parameter_events/rcl_interfaces::msg::dds_::ParameterEvent_/RIHS01_043e627780fcad87a22d225bc2a037361dba713fca6a6b9f4b869a5aa0393204/::,1000:,:,:,,";
    const ROSOUT: &str = "@ros2_lv/0/943f9ebc32acd3e072bfda12ce8c7ed2/0/1/MP/%/%/_ros2cli_31/%rosout/rcl_interfaces::msg::dds_::Log_/RIHS01_e28ce254ca8abc06abf92773b74602cdbf116ed34fbaf294fb9f81da9f318eac/:1:,1000:,:10,0:,,";
    const OTHER_NODE: &str = "@ros2_lv/0/c602d439ce8bbe5c350b96bc4c79a5c7/0/2/MP/%/%/_ros2cli_daemon_0_4a78e255bb8846f4bfe6a3ec278dbff1/%parameter_events/rcl_interfaces::msg::dds_::ParameterEvent_/RIHS01_043e627780fcad87a22d225bc2a037361dba713fca6a6b9f4b869a5aa0393204/::,1000:,:,:,,";
    const SUBSCRIPTION: &str = "@ros2_lv/0/943f9ebc32acd3e072bfda12ce8c7ed2/0/4/MS/%/%/_ros2cli_31/%cmd_vel/geometry_msgs::msg::dds_::TwistStamped_/RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64/2::,1:,:,:,,";
    const NODE: &str = "@ros2_lv/0/943f9ebc32acd3e072bfda12ce8c7ed2/0/0/NN/%/%/_ros2cli_31";

    #[test]
    fn publishers_of_one_node_share_a_node_id() {
        let mut g = Graph::<8>::new();
        g.on_token(PARAM_EVENTS, true);
        g.on_token(ROSOUT, true);
        g.on_token(OTHER_NODE, true);
        let (a, b, c) = (
            g.node_of(&gid(PARAM_EVENTS)).unwrap(),
            g.node_of(&gid(ROSOUT)).unwrap(),
            g.node_of(&gid(OTHER_NODE)).unwrap(),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn unknown_gids_and_non_publisher_tokens_are_not_mapped() {
        let mut g = Graph::<8>::new();
        g.on_token(SUBSCRIPTION, true);
        g.on_token(NODE, true);
        g.on_token("garbage", true);
        assert_eq!(g.node_of(&gid(SUBSCRIPTION)), None);
        assert_eq!(g.node_of(&gid(NODE)), None);
        assert_eq!(g.node_of(&[7; 16]), None);
    }

    #[test]
    fn a_dropped_token_is_forgotten() {
        let mut g = Graph::<8>::new();
        g.on_token(ROSOUT, true);
        g.on_token(ROSOUT, false);
        assert_eq!(g.node_of(&gid(ROSOUT)), None);
    }

    #[test]
    fn the_oldest_entry_is_evicted_when_full() {
        let mut g = Graph::<2>::new();
        g.on_token(ROSOUT, true);
        g.on_token(PARAM_EVENTS, true);
        g.on_token(OTHER_NODE, true);
        assert_eq!(g.node_of(&gid(ROSOUT)), None);
        assert!(g.node_of(&gid(PARAM_EVENTS)).is_some() && g.node_of(&gid(OTHER_NODE)).is_some());
        g.on_token(OTHER_NODE, true);
        assert!(
            g.node_of(&gid(PARAM_EVENTS)).is_some(),
            "re-announcing does not evict"
        );
    }
}
