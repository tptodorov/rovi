//! When to publish Status on ROS: on change, else every 900 ms. The message is
//! [`rovi_protocol::Status`].

use rovi_protocol::Status;

/// One ROS connection's change/heartbeat schedule.
#[derive(Default)]
pub struct Publication {
    last: Option<(u64, Status)>,
}

impl Publication {
    pub fn due(&mut self, now: u64, status: Status) -> bool {
        if self
            .last
            .is_some_and(|(time, s)| s == status && now.saturating_sub(time) < 900)
        {
            return false;
        }
        self.last = Some((now, status));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rovi_protocol::status::{Error, State, StopReason};

    #[test]
    fn status_publication_tracks_each_field_and_heartbeat() {
        let open = Status {
            state: State::Open,
            error: None,
            session: 0,
        };
        let mut p = Publication::default();
        assert!(p.due(0, open));
        assert!(!p.due(899, open));
        assert!(p.due(900, open));
        let mut s = Status {
            state: State::Claimed,
            session: 1,
            ..open
        };
        assert!(p.due(910, s));
        assert!(!p.due(911, s));
        s.error = Some(Error::NotReady);
        assert!(p.due(920, s));
        s.error = None;
        assert!(p.due(930, s));
        s.session = 2;
        assert!(p.due(940, s));
        s.state = State::Stopped(StopReason::Stop);
        assert!(p.due(950, s));
        s.state = State::Stopped(StopReason::OutOfLimits);
        assert!(p.due(960, s));
        assert!(p.due(970, open), "release publishes all cleared fields");
    }
}
