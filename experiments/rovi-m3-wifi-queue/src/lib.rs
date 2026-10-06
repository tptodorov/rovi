#![no_std]

pub const CAPACITY: usize = 8;
pub const VERSION: u8 = 1;
pub const FRAME_LEN: usize = 22;
pub const COMMANDS: &str = "0=stop,1=forward,2=reverse,3=ping";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    Stop = 0,
    Forward = 1,
    Reverse = 2,
    Ping = 3,
}
impl Command {
    pub fn parse(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Stop),
            1 => Some(Self::Forward),
            2 => Some(Self::Reverse),
            3 => Some(Self::Ping),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Wifi = 0,
    Ble = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub source: Source,
    pub session: u64,
    pub seq: u32,
    pub expires_ms: u64,
    pub command: Command,
}

/// Network integers are little-endian. Exact length and version are mandatory.
pub fn decode(bytes: &[u8]) -> Option<Input> {
    if bytes.len() != FRAME_LEN || bytes[0] != VERSION {
        return None;
    }
    Some(Input {
        command: Command::parse(bytes[1])?,
        source: Source::Wifi,
        session: u64::from_le_bytes(bytes[2..10].try_into().ok()?),
        seq: u32::from_le_bytes(bytes[10..14].try_into().ok()?),
        expires_ms: u64::from_le_bytes(bytes[14..22].try_into().ok()?),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub input: Input,
    pub order: u64,
    pub received_ms: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Queued,
    Stop,
    Ping,
    Session,
    Sequence,
    Stale,
    Full,
    Stopping,
}
impl Status {
    pub fn accepted(self) -> bool {
        matches!(self, Self::Queued | Self::Stop | Self::Ping)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Stop,
    Disconnect(Source),
    Timeout(Source),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Safety {
        reason: Reason,
        at_ms: u64,
        purged: usize,
        entry: Option<Entry>,
    },
    Apply(Entry),
    Stale(Entry),
}
#[derive(Clone, Copy)]
struct Session {
    id: u64,
    seq: u32,
    refreshed_ms: u64,
}

/// One owner, two ingress adapters. No I/O, allocation, or waiting in this core.
pub struct Ingress {
    queue: [Option<Entry>; CAPACITY],
    depth: usize,
    sessions: [Option<Session>; 2],
    safety: Option<Output>,
    order: u64,
    next_session: u64,
    pub watchdog_ms: u64,
    pub max_age_ms: u64,
}
impl Ingress {
    pub const fn new(watchdog_ms: u64, max_age_ms: u64, seed: u64) -> Self {
        Self {
            queue: [None; CAPACITY],
            depth: 0,
            sessions: [None; 2],
            safety: None,
            order: 0,
            next_session: seed,
            watchdog_ms,
            max_age_ms,
        }
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn live(&self, source: Source, id: u64) -> bool {
        self.sessions[source as usize].is_some_and(|s| s.id == id)
    }
    pub fn connect(&mut self, source: Source, now: u64) -> u64 {
        self.disconnect(source, now);
        self.next_session = self.next_session.checked_add(1).expect("session exhausted");
        let id = self.next_session;
        // Connection grants one initial lease, but never refreshes another source.
        self.sessions[source as usize] = Some(Session {
            id,
            seq: 0,
            refreshed_ms: now,
        });
        id
    }
    pub fn disconnect(&mut self, source: Source, now: u64) {
        if self.sessions[source as usize].take().is_some() {
            self.stop(Reason::Disconnect(source), now, None);
        }
    }
    fn stop(&mut self, reason: Reason, now: u64, entry: Option<Entry>) {
        let purged = self.depth;
        self.queue = [None; CAPACITY];
        self.depth = 0;
        // Keep the first safety cause until the consumer observes it.
        if self.safety.is_none() {
            self.safety = Some(Output::Safety {
                reason,
                at_ms: now,
                purged,
                entry,
            });
        }
    }
    /// Runs before ingress and on a 10 ms consumer tick, never behind FIFO work.
    pub fn expire(&mut self, now: u64) {
        for source in [Source::Wifi, Source::Ble] {
            if self.sessions[source as usize]
                .is_some_and(|s| now.saturating_sub(s.refreshed_ms) >= self.watchdog_ms)
            {
                self.sessions[source as usize] = None;
                self.stop(Reason::Timeout(source), now, None);
            }
        }
    }
    pub fn submit(&mut self, input: Input, now: u64) -> (Status, Option<Entry>) {
        self.expire(now);
        let Some(session) = self.sessions[input.source as usize].as_mut() else {
            return (Status::Session, None);
        };
        if session.id != input.session {
            return (Status::Session, None);
        }
        if input.seq <= session.seq {
            return (Status::Sequence, None);
        }
        // Consume sequence even on rejection: retries need fresh intent and a new sequence.
        session.seq = input.seq;
        if input.expires_ms <= now || input.expires_ms - now > self.max_age_ms {
            return (Status::Stale, None);
        }
        if matches!(input.command, Command::Forward | Command::Reverse) {
            if self.safety.is_some() {
                return (Status::Stopping, None);
            }
            if self.depth == CAPACITY {
                return (Status::Full, None);
            }
        }
        session.refreshed_ms = now;
        self.order += 1;
        let entry = Entry {
            input,
            order: self.order,
            received_ms: now,
        };
        match input.command {
            Command::Stop => {
                self.stop(Reason::Stop, now, Some(entry));
                (Status::Stop, Some(entry))
            }
            Command::Ping => (Status::Ping, Some(entry)),
            _ => {
                self.queue[self.depth] = Some(entry);
                self.depth += 1;
                (Status::Queued, Some(entry))
            }
        }
    }
    pub fn poll(&mut self, now: u64, drain: bool) -> Option<Output> {
        self.expire(now);
        if let Some(safety) = self.safety.take() {
            return Some(safety);
        }
        if !drain || self.depth == 0 {
            return None;
        }
        let entry = self.queue[0].take().unwrap();
        self.queue.copy_within(1..self.depth, 0);
        self.depth -= 1;
        self.queue[self.depth] = None;
        Some(if now >= entry.input.expires_ms {
            Output::Stale(entry)
        } else {
            Output::Apply(entry)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(source: Source, session: u64, seq: u32, command: Command, expires_ms: u64) -> Input {
        Input {
            source,
            session,
            seq,
            command,
            expires_ms,
        }
    }
    #[test]
    fn interleaved_opposing_commands_share_fifo() {
        let mut q = Ingress::new(1000, 250, 0);
        let w = q.connect(Source::Wifi, 0);
        let b = q.connect(Source::Ble, 0);
        for (source, session, seq, cmd) in [
            (Source::Wifi, w, 1, Command::Forward),
            (Source::Ble, b, 1, Command::Reverse),
            (Source::Wifi, w, 2, Command::Reverse),
        ] {
            assert_eq!(
                q.submit(input(source, session, seq, cmd, 250), 1).0,
                Status::Queued
            );
        }
        for order in 1..=3 {
            let Some(Output::Apply(e)) = q.poll(2, true) else {
                panic!()
            };
            assert_eq!(e.order, order);
        }
    }
    #[test]
    fn full_queue_rejects_newest_but_stop_bypasses_and_purges() {
        let mut q = Ingress::new(1000, 250, 0);
        let s = q.connect(Source::Wifi, 0);
        for seq in 1..=8 {
            assert_eq!(
                q.submit(input(Source::Wifi, s, seq, Command::Forward, 250), 0)
                    .0,
                Status::Queued
            );
        }
        assert_eq!(
            q.submit(input(Source::Wifi, s, 9, Command::Reverse, 250), 0)
                .0,
            Status::Full
        );
        assert_eq!(
            q.submit(input(Source::Wifi, s, 10, Command::Stop, 250), 0)
                .0,
            Status::Stop
        );
        assert_eq!(
            q.submit(input(Source::Wifi, s, 11, Command::Forward, 250), 0)
                .0,
            Status::Stopping
        );
        assert!(matches!(
            q.poll(1, false),
            Some(Output::Safety {
                purged: 8,
                reason: Reason::Stop,
                ..
            })
        ));
        assert_eq!(q.poll(2, true), None);
    }
    #[test]
    fn watchdog_bypasses_paused_consumer_and_invalidates_session() {
        let mut q = Ingress::new(100, 250, 0);
        let s = q.connect(Source::Wifi, 0);
        for seq in 1..=8 {
            q.submit(input(Source::Wifi, s, seq, Command::Forward, 250), 0);
        }
        assert_eq!(
            q.submit(input(Source::Wifi, s, 9, Command::Reverse, 250), 99)
                .0,
            Status::Full
        );
        assert!(matches!(
            q.poll(100, false),
            Some(Output::Safety {
                reason: Reason::Timeout(Source::Wifi),
                at_ms: 100,
                purged: 8,
                ..
            })
        ));
        assert_eq!(
            q.submit(input(Source::Wifi, s, 10, Command::Forward, 251), 101)
                .0,
            Status::Session
        );
    }
    #[test]
    fn only_valid_admitted_commands_and_pings_refresh_each_source() {
        let mut q = Ingress::new(100, 250, 0);
        let w = q.connect(Source::Wifi, 0);
        let b = q.connect(Source::Ble, 0);
        assert_eq!(
            q.submit(input(Source::Wifi, w, 1, Command::Ping, 300), 90)
                .0,
            Status::Ping
        );
        assert_eq!(
            q.submit(input(Source::Wifi, w, 1, Command::Ping, 300), 99)
                .0,
            Status::Sequence
        );
        assert_eq!(
            q.submit(input(Source::Ble, b, 1, Command::Forward, 99), 99)
                .0,
            Status::Stale
        );
        assert!(matches!(
            q.poll(100, false),
            Some(Output::Safety {
                reason: Reason::Timeout(Source::Ble),
                ..
            })
        ));
        assert!(q.live(Source::Wifi, w));
        assert!(matches!(
            q.poll(190, false),
            Some(Output::Safety {
                reason: Reason::Timeout(Source::Wifi),
                ..
            })
        ));
    }
    #[test]
    fn stale_at_ingress_or_dequeue_and_future_deadlines_rejected() {
        let mut q = Ingress::new(1000, 250, 0);
        let s = q.connect(Source::Wifi, 0);
        assert_eq!(
            q.submit(input(Source::Wifi, s, 1, Command::Forward, 500), 0)
                .0,
            Status::Stale
        );
        q.submit(input(Source::Wifi, s, 2, Command::Reverse, 50), 0);
        assert!(matches!(q.poll(50, true), Some(Output::Stale(_))));
    }
    #[test]
    fn disconnect_clears_both_sources_and_reconnect_rejects_old_session() {
        let mut q = Ingress::new(1000, 250, 0);
        let old = q.connect(Source::Wifi, 0);
        let b = q.connect(Source::Ble, 0);
        q.submit(input(Source::Ble, b, 1, Command::Reverse, 250), 0);
        q.disconnect(Source::Wifi, 1);
        let new = q.connect(Source::Wifi, 2);
        assert_ne!(old, new);
        assert!(matches!(
            q.poll(2, true),
            Some(Output::Safety { purged: 1, .. })
        ));
        assert_eq!(
            q.submit(input(Source::Wifi, old, 1, Command::Forward, 250), 2)
                .0,
            Status::Session
        );
        assert_eq!(
            q.submit(input(Source::Wifi, new, 1, Command::Forward, 250), 2)
                .0,
            Status::Queued
        );
    }
    #[test]
    fn expired_lease_cannot_be_revived_by_input_at_deadline() {
        let mut q = Ingress::new(100, 250, 0);
        let s = q.connect(Source::Ble, 0);
        assert_eq!(
            q.submit(input(Source::Ble, s, 1, Command::Ping, 200), 100)
                .0,
            Status::Session
        );
        assert!(matches!(
            q.poll(100, false),
            Some(Output::Safety {
                reason: Reason::Timeout(Source::Ble),
                ..
            })
        ));
    }
    #[test]
    fn ping_refreshes_full_queue_and_ble_stop_purges_wifi_work() {
        let mut q = Ingress::new(100, 250, 0);
        let w = q.connect(Source::Wifi, 0);
        let b = q.connect(Source::Ble, 0);
        for seq in 1..=8 {
            q.submit(input(Source::Wifi, w, seq, Command::Forward, 250), 0);
        }
        assert_eq!(
            q.submit(input(Source::Wifi, w, 9, Command::Ping, 300), 90)
                .0,
            Status::Ping
        );
        assert_eq!(q.depth(), 8);
        assert_eq!(
            q.submit(input(Source::Ble, b, 1, Command::Stop, 300), 90).0,
            Status::Stop
        );
        assert!(matches!(
            q.poll(100, false),
            Some(Output::Safety {
                reason: Reason::Stop,
                purged: 8,
                ..
            })
        ));
        assert!(q.live(Source::Wifi, w));
        assert!(q.live(Source::Ble, b));
        assert_eq!(q.poll(100, true), None);
        // Neither dequeuing nor rejected replay renews the 90 ms refresh.
        assert_eq!(
            q.submit(input(Source::Wifi, w, 9, Command::Ping, 300), 180)
                .0,
            Status::Sequence
        );
        assert!(matches!(
            q.poll(190, false),
            Some(Output::Safety {
                reason: Reason::Timeout(_),
                ..
            })
        ));
    }

    #[test]
    fn decoder_rejects_unknown_version_command_and_length() {
        let mut frame = [0; FRAME_LEN];
        frame[0] = VERSION;
        for command in 0..=3 {
            frame[1] = command;
            assert_eq!(
                decode(&frame).unwrap().command,
                Command::parse(command).unwrap()
            );
        }
        frame[1] = 4;
        assert!(decode(&frame).is_none());
        frame[1] = 0;
        frame[0] = 2;
        assert!(decode(&frame).is_none());
        assert!(decode(&frame[..21]).is_none());
    }
}
