//! When to publish capabilities on ROS. The message is [`rovi_protocol::Capabilities`].

/// Leaves room for the adapter's 10 ms poll and subscription setup within the one-second bound.
const REPUBLISH_MS: u64 = 900;

/// Per-router-connection publication schedule. Claims/releases are identified by session/owner.
#[derive(Default)]
pub struct Publication {
    last: Option<(u64, u32, bool)>,
}

impl Publication {
    /// Records a publication attempt when connection, ownership or periodic timing requires it.
    pub fn due(&mut self, now: u64, session: u32, owned: bool) -> bool {
        if self.last.is_some_and(|(time, s, o)| {
            session == s && owned == o && now.saturating_sub(time) < REPUBLISH_MS
        }) {
            return false;
        }
        self.last = Some((now, session, owned));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_publish_on_connect_and_before_each_second_elapses() {
        let mut p = Publication::default();
        assert!(p.due(0, 0, false));
        assert!(!p.due(REPUBLISH_MS - 1, 0, false));
        assert!(p.due(REPUBLISH_MS, 0, false));
        assert!(!p.due(2 * REPUBLISH_MS - 1, 0, false));
        assert!(p.due(2 * REPUBLISH_MS, 0, false));
    }

    #[test]
    fn claim_release_and_new_sessions_publish_without_waiting_for_the_period() {
        let mut p = Publication::default();
        p.due(0, 0, false);
        assert!(p.due(10, 1, true), "claim");
        assert!(!p.due(11, 1, true), "an unchanged owner does not flood");
        assert!(p.due(20, 1, false), "release");
        assert!(
            p.due(30, 2, true),
            "new session even with the same controller"
        );
    }

    #[test]
    fn router_reconnect_starts_a_fresh_publication_schedule() {
        let mut old = Publication::default();
        assert!(old.due(1000, 7, true));
        assert!(!old.due(1001, 7, true));
        let mut new = Publication::default();
        assert!(new.due(1001, 7, true));
    }
}
