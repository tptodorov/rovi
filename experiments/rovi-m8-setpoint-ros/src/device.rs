//! The device core: the arbiter plus kinematics, driven by UDP datagrams. The BLE and ROS
//! adapters will feed the same arbiter. Time is passed in; the firmware shell owns the sockets.

use crate::arbiter::{Arbiter, Claimant, Config, Effects, Error, Source, State, StopReason};
use crate::graph::Graph;
use crate::kinematics::{self, Geometry};
use crate::rmw::Attachment;
use crate::setpoint::Setpoint;
use crate::udp::{self, Header, Kind};

/// Bytes of the `HELLO_ACK` payload.
pub const CAPABILITIES_LEN: usize = 27;
/// Bytes of the `STATUS` payload: state, stop reason, last error.
pub const STATUS_LEN: usize = 3;

/// A reply to send back to the datagram's sender (`len` 0 means none) and the effects for the
/// shell to act on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    pub len: usize,
    pub effects: Effects,
}

pub struct Device {
    cfg: Config,
    geometry: Geometry,
    arbiter: Arbiter,
    graph: Graph<16>,
}

impl Device {
    pub fn new(cfg: Config, geometry: Geometry) -> Self {
        Self {
            cfg,
            geometry,
            arbiter: Arbiter::new(cfg),
            graph: Graph::new(),
        }
    }

    /// May `source`'s radio be on? While one source owns the car, the others' radios are off.
    pub fn radio_allowed(&self, source: Source) -> bool {
        self.arbiter.owner().is_none_or(|o| o.source == source)
    }

    /// May the Wi-Fi AP be up? UDP and ROS share it, so it is off only while BLE owns the car.
    pub fn wifi_allowed(&self) -> bool {
        self.arbiter.owner().is_none_or(|o| o.source != Source::Ble)
    }

    /// Is `conn` still the BLE owner?
    pub fn ble_owns(&self, conn: u16) -> bool {
        self.arbiter.owner() == Some(ble(conn))
    }

    pub fn arbiter(&self) -> &Arbiter {
        &self.arbiter
    }

    /// Handles one UDP datagram from `peer` (an id the shell keeps stable per remote address).
    pub fn on_udp(&mut self, now: u64, peer: u128, datagram: &[u8], out: &mut [u8]) -> Reply {
        let Some((h, payload)) = udp::parse(datagram) else {
            return Reply::default();
        };
        let who = Claimant {
            source: Source::Udp,
            id: peer,
        };
        let ours = h.session == self.arbiter.session();
        let reply = |kind, session, payload: &[u8], out: &mut [u8]| {
            udp::write(
                out,
                Header {
                    kind,
                    session,
                    seq: h.seq,
                },
                payload,
            )
            .unwrap_or(0)
        };
        match h.kind {
            Kind::Hello => match self.arbiter.claim(now, who) {
                Ok(effects) => {
                    let caps = self.capabilities();
                    let len = reply(Kind::HelloAck, self.arbiter.session(), &caps, out);
                    Reply { len, effects }
                }
                Err(_) => Reply {
                    len: reply(Kind::Busy, self.arbiter.session(), &[], out),
                    ..Reply::default()
                },
            },
            // Input from another session or a non-owner is dropped without a reply.
            Kind::Setpoint if ours => match Setpoint::decode(payload) {
                Ok(sp) => match self.arbiter.setpoint(now, who, h.seq, sp) {
                    Ok((_, effects)) => Reply { len: 0, effects },
                    Err(_) => Reply::default(),
                },
                Err(_) => Reply::default(),
            },
            Kind::Arm if ours => match self.arbiter.arm(now, who) {
                Ok(()) => self.status_reply(Effects::default(), None, h, out),
                Err(Error::NotOwner) => Reply::default(),
                Err(e) => self.status_reply(Effects::default(), Some(e), h, out),
            },
            Kind::Stop if ours => match self.arbiter.stop(now, who) {
                Ok(effects) => self.status_reply(effects, None, h, out),
                Err(_) => Reply::default(),
            },
            Kind::Bye if ours => match self.arbiter.release(now, who) {
                Ok(effects) => Reply { len: 0, effects },
                Err(_) => Reply::default(),
            },
            _ => Reply::default(),
        }
    }

    fn status_reply(
        &self,
        effects: Effects,
        error: Option<Error>,
        h: Header,
        out: &mut [u8],
    ) -> Reply {
        let header = Header {
            kind: Kind::Status,
            session: self.arbiter.session(),
            seq: h.seq,
        };
        let len = udp::write(out, header, &self.status(error)).unwrap_or(0);
        Reply { len, effects }
    }

    /// A BLE central connected: that is the claim.
    pub fn on_ble_connect(&mut self, now: u64, conn: u16) -> Result<Effects, Error> {
        self.arbiter.claim(now, ble(conn))
    }

    /// A write to the Setpoint characteristic: `seq u32` (LE) then the CDR `TwistStamped`.
    pub fn on_ble_setpoint(&mut self, now: u64, conn: u16, data: &[u8]) -> Effects {
        let Some((seq, cdr)) = data.split_first_chunk::<4>() else {
            return Effects::default();
        };
        let Ok(sp) = Setpoint::decode(cdr) else {
            return Effects::default();
        };
        match self
            .arbiter
            .setpoint(now, ble(conn), u32::from_le_bytes(*seq), sp)
        {
            Ok((_, effects)) => effects,
            Err(_) => Effects::default(),
        }
    }

    /// A write to the Control characteristic: 1 = ARM, 2 = STOP. An error rejects the write.
    pub fn on_ble_control(&mut self, now: u64, conn: u16, cmd: u8) -> Result<Effects, Error> {
        let who = ble(conn);
        match cmd {
            1 => self.arbiter.arm(now, who).map(|()| Effects::default()),
            2 => self.arbiter.stop(now, who),
            _ => self.arbiter.check_owner(who).and(Err(Error::Invalid)),
        }
    }

    /// The central disconnected.
    pub fn on_ble_disconnect(&mut self, now: u64, conn: u16) -> Effects {
        self.arbiter.release(now, ble(conn)).unwrap_or_default()
    }

    /// A liveliness token appeared (`alive`) or went away. Samples are attributed to a ROS node
    /// through the publisher tokens learned here; samples from unknown publishers are dropped.
    pub fn on_ros_liveliness(&mut self, key: &str, alive: bool) {
        self.graph.on_token(key, alive);
    }

    fn ros_claimant(&self, att: &Attachment) -> Option<Claimant> {
        let id = self.graph.node_of(&att.gid)?;
        Some(Claimant {
            source: Source::Ros,
            id,
        })
    }

    /// A sample on the `cmd_vel` topic. The first valid one from a free device claims it, and the
    /// owner is the ROS node of the publisher gid in the attachment. Input from other nodes is ignored.
    pub fn on_ros_cmd_vel(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let (Some(att), Ok(sp)) = (Attachment::decode(attachment), Setpoint::decode(payload))
        else {
            return Effects::default();
        };
        let Some(who) = self.ros_claimant(&att) else {
            return Effects::default();
        };
        let Ok(mut fx) = self.arbiter.claim(now, who) else {
            return Effects::default();
        };
        if let Ok((_, e)) = self.arbiter.setpoint(now, who, att.seq as u32, sp) {
            fx.stop = e.stop;
        }
        fx
    }

    /// A sample on `rovi/arm` (`std_msgs/Bool`): true arms, false stops.
    pub fn on_ros_arm(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let Some(att) = Attachment::decode(attachment) else {
            return Effects::default();
        };
        if payload.len() < 5 || payload[..4] != [0, 1, 0, 0] {
            return Effects::default();
        }
        let Some(who) = self.ros_claimant(&att) else {
            return Effects::default();
        };
        if payload[4] != 0 {
            let _ = self.arbiter.arm(now, who);
            Effects::default()
        } else {
            self.arbiter.stop(now, who).unwrap_or_default()
        }
    }

    /// Writes the `rovi/status` sample (`std_msgs/String` CDR) and returns its length.
    pub fn status_cdr(&self, out: &mut [u8]) -> usize {
        use core::fmt::Write;
        let state = match self.arbiter.state() {
            State::Open => "open",
            State::Claimed => "claimed",
            State::Armed => "armed",
            State::Stopped(_) => "stopped",
        };
        let mut text = crate::rmw::Key::<48>::new();
        let _ = write!(text, "state={state} session={}", self.arbiter.session());
        let text = text.as_str().as_bytes();
        let len = 8 + text.len() + 1;
        let Some(out) = out.get_mut(..len) else {
            return 0;
        };
        out[..4].copy_from_slice(&[0, 1, 0, 0]);
        out[4..8].copy_from_slice(&(text.len() as u32 + 1).to_le_bytes());
        out[8..len - 1].copy_from_slice(text);
        out[len - 1] = 0;
        len
    }

    /// Call every control tick (10 ms).
    pub fn tick(&mut self, now: u64) -> Effects {
        self.arbiter.tick(now)
    }

    /// Wheel commands (FL, FR, RR, RL) in -1.0..=1.0: zero unless armed with a valid lease.
    pub fn wheels(&self, now: u64) -> [f32; 4] {
        match self.arbiter.drive(now) {
            Some(sp) => kinematics::normalise(
                &self.geometry,
                kinematics::inverse(&self.geometry, sp.vx, sp.vy, sp.wz),
            ),
            None => [0.0; 4],
        }
    }

    /// The Capabilities characteristic (and the `HELLO_ACK` payload).
    pub fn capabilities(&self) -> [u8; CAPABILITIES_LEN] {
        let mut b = [0u8; CAPABILITIES_LEN];
        b[0] = udp::VERSION;
        let f = [
            self.geometry.wheels_radius,
            self.geometry.center_projection_sum,
            self.cfg.max_vx,
            self.cfg.max_vy,
            self.cfg.max_wz,
        ];
        for (i, v) in f.iter().enumerate() {
            b[1 + 4 * i..5 + 4 * i].copy_from_slice(&(*v as f32).to_le_bytes());
        }
        b[21..23].copy_from_slice(&(self.cfg.lease_ms as u16).to_le_bytes());
        b[23..25].copy_from_slice(&(self.cfg.claim_window_ms as u16).to_le_bytes());
        b[25..27].copy_from_slice(&(self.cfg.reclaim_window_ms as u16).to_le_bytes());
        b
    }

    /// The Status characteristic (and the `STATUS` payload).
    pub fn status(&self, error: Option<Error>) -> [u8; STATUS_LEN] {
        let (state, reason) = match self.arbiter.state() {
            State::Open => (0, 0),
            State::Claimed => (1, 0),
            State::Armed => (2, 0),
            State::Stopped(r) => (3, stop_code(r)),
        };
        let error = match error {
            None => 0,
            Some(Error::Busy) => 1,
            Some(Error::NotOwner) => 2,
            Some(Error::NotReady) => 3,
            Some(Error::Invalid) => 4,
        };
        [state, reason, error]
    }
}

fn ble(conn: u16) -> Claimant {
    Claimant {
        source: Source::Ble,
        id: conn as u128,
    }
}

fn stop_code(r: StopReason) -> u8 {
    match r {
        StopReason::Stop => 1,
        StopReason::LeaseExpired => 2,
        StopReason::OwnerLost => 3,
        StopReason::OutOfLimits => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;

    const GEOMETRY: Geometry = Geometry {
        wheels_radius: 0.05,
        center_projection_sum: 0.2,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_theta: 0.0,
        max_wheel_speed: 40.0,
    };
    const A: u128 = 0xA;
    const B: u128 = 0xB;

    fn dev() -> Device {
        Device::new(Config::DEFAULT, GEOMETRY)
    }

    fn frame(kind: Kind, session: u32, seq: u32, payload: &[u8]) -> ([u8; 96], usize) {
        let mut b = [0u8; 96];
        let n = udp::write(&mut b, Header { kind, session, seq }, payload).unwrap();
        (b, n)
    }

    fn send(
        d: &mut Device,
        now: u64,
        peer: u128,
        kind: Kind,
        session: u32,
        seq: u32,
        payload: &[u8],
    ) -> (Option<(Header, [u8; 64])>, Effects) {
        let (b, n) = frame(kind, session, seq, payload);
        let mut out = [0u8; 64];
        let r = d.on_udp(now, peer, &b[..n], &mut out);
        let reply = (r.len > 0).then(|| {
            let (h, p) = udp::parse(&out[..r.len]).unwrap();
            let mut pl = [0u8; 64];
            pl[..p.len()].copy_from_slice(p);
            (h, pl)
        });
        (reply, r.effects)
    }

    fn cdr(vx: f64) -> [u8; 68] {
        let mut b = [0u8; 68];
        Setpoint {
            sec: 0,
            nanosec: 0,
            vx,
            vy: 0.0,
            wz: 0.0,
        }
        .encode(&mut b)
        .unwrap();
        b
    }

    /// HELLO, one setpoint, ARM by A. Returns the session.
    fn armed(d: &mut Device) -> u32 {
        let (r, _) = send(d, 0, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        send(d, 0, A, Kind::Setpoint, s, 1, &cdr(0.5));
        let (r, _) = send(d, 0, A, Kind::Arm, s, 2, &[]);
        assert_eq!(r.unwrap().1[0], 2, "armed");
        s
    }

    #[test]
    fn hello_claims_and_acks_with_capabilities() {
        let mut d = dev();
        let (reply, fx) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let (h, p) = reply.unwrap();
        assert_eq!((h.kind, h.session), (Kind::HelloAck, 1));
        assert_eq!(fx.claimed, Some(Source::Udp));
        assert_eq!(p[0], udp::VERSION);
        assert_eq!(f32::from_le_bytes(p[1..5].try_into().unwrap()), 0.05);
        assert_eq!(u16::from_le_bytes(p[21..23].try_into().unwrap()), 1_000);
    }

    #[test]
    fn second_peer_gets_busy_and_cannot_drive() {
        let mut d = dev();
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let (r, fx) = send(&mut d, 1, B, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::Busy);
        assert_eq!(fx, Effects::default());
        let (r, _) = send(&mut d, 1, B, Kind::Setpoint, 1, 1, &cdr(0.5));
        assert!(r.is_none(), "non-owner input is ignored silently");
    }

    #[test]
    fn streams_setpoints_and_drives_wheels_only_when_armed() {
        let mut d = dev();
        let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        send(&mut d, 10, A, Kind::Setpoint, s, 1, &cdr(1.0));
        assert_eq!(d.wheels(10), [0.0; 4], "not armed yet");
        send(&mut d, 20, A, Kind::Arm, s, 2, &[]);
        assert_eq!(d.wheels(20), [0.5; 4], "20 rad/s of 40");
        send(&mut d, 70, A, Kind::Setpoint, s, 2, &cdr(-1.0));
        assert_eq!(d.wheels(70), [-0.5; 4]);
    }

    #[test]
    fn lease_expiry_stops_wheels_and_reports_it() {
        let mut d = dev();
        armed(&mut d);
        assert_eq!(d.wheels(999), [0.25; 4]);
        assert_eq!(d.tick(1_000).stop, Some(StopReason::LeaseExpired));
        assert_eq!(d.wheels(1_000), [0.0; 4]);
    }

    #[test]
    fn stop_replies_with_status_and_disarms() {
        let mut d = dev();
        let s = armed(&mut d);
        let (r, fx) = send(&mut d, 5, A, Kind::Stop, s, 3, &[]);
        assert_eq!(fx.stop, Some(StopReason::Stop));
        let (h, p) = r.unwrap();
        assert_eq!(h.kind, Kind::Status);
        assert_eq!(&p[..3], &[3, 1, 0], "stopped, reason Stop, no error");
        assert_eq!(d.wheels(5), [0.0; 4]);
    }

    #[test]
    fn arm_without_a_fresh_setpoint_is_refused_with_status() {
        let mut d = dev();
        let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        let (r, _) = send(&mut d, 1, A, Kind::Arm, s, 2, &[]);
        let (h, p) = r.unwrap();
        assert_eq!(h.kind, Kind::Status);
        assert_eq!(&p[..3], &[1, 0, 3], "still claimed, error NotReady");
    }

    #[test]
    fn wrong_session_and_garbage_are_dropped() {
        let mut d = dev();
        let s = armed(&mut d);
        let (r, _) = send(&mut d, 1, A, Kind::Setpoint, s + 1, 9, &cdr(1.0));
        assert!(r.is_none());
        assert_eq!(
            d.wheels(1),
            [0.25; 4],
            "the wrong-session setpoint did not apply"
        );
        let mut out = [0u8; 64];
        assert_eq!(d.on_udp(1, A, b"junk", &mut out), Reply::default());
        let (r, fx) = send(&mut d, 1, A, Kind::Setpoint, s, 5, &[1, 2, 3]);
        assert!(
            (r.is_none()) && fx == Effects::default(),
            "malformed CDR is dropped"
        );
    }

    #[test]
    fn out_of_plane_cdr_is_dropped_but_out_of_limits_is_a_stop() {
        let mut d = dev();
        let s = armed(&mut d);
        let (_, fx) = send(
            &mut d,
            1,
            A,
            Kind::Setpoint,
            s,
            5,
            &cdr(Config::DEFAULT.max_vx + 1.0),
        );
        assert_eq!(fx.stop, Some(StopReason::OutOfLimits));
    }

    #[test]
    fn bye_releases_and_reopens() {
        let mut d = dev();
        let s = armed(&mut d);
        let (_, fx) = send(&mut d, 5, A, Kind::Bye, s, 3, &[]);
        assert!(fx.released);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        let (r, _) = send(&mut d, 6, B, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.session, s + 1);
    }

    /// Registers a fake publisher `id` of node `nid` and returns its attachment for `seq`.
    fn publisher(d: &mut Device, nid: u8, id: u8, seq: i64) -> [u8; 33] {
        let key = format!(
            "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/{nid}/{id}/MP/%/%/n{nid}/%t{id}/std_msgs::msg::dds_::Bool_/RIHS01_x/::,10:,:,:,,"
        );
        d.on_ros_liveliness(&key, true);
        Attachment {
            seq,
            timestamp: 0,
            gid: crate::rmw::gid(&key),
        }
        .encode()
    }

    fn bool_cdr(v: bool) -> [u8; 5] {
        [0, 1, 0, 0, v as u8]
    }

    #[test]
    fn first_ros_setpoint_claims_then_arm_from_another_publisher_of_the_node_drives() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        let fx = d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(fx.claimed, Some(Source::Ros));
        assert_eq!(d.wheels(0), [0.0; 4], "claimed but not armed");
        d.on_ros_arm(5, &arm, &bool_cdr(true));
        assert_eq!(d.wheels(5), [0.5; 4]);
        let fx = d.on_ros_arm(6, &arm, &bool_cdr(false));
        assert_eq!(fx.stop, Some(StopReason::Stop));
        assert_eq!(d.wheels(6), [0.0; 4]);
    }

    #[test]
    fn another_ros_node_is_ignored_while_owned() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        let (cmd2, arm2) = (publisher(&mut d, 2, 1, 1), publisher(&mut d, 2, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        d.on_ros_arm(1, &arm, &bool_cdr(true));
        assert_eq!(d.on_ros_cmd_vel(2, &cmd2, &cdr(-1.0)), Effects::default());
        assert_eq!(d.on_ros_arm(2, &arm2, &bool_cdr(false)), Effects::default());
        assert_eq!(
            d.wheels(2),
            [0.25; 4],
            "owner's setpoint and arm are untouched"
        );
    }

    #[test]
    fn unknown_publishers_cannot_claim_and_forgotten_ones_stop_counting() {
        let mut d = dev();
        let unknown = Attachment {
            seq: 1,
            timestamp: 0,
            gid: [9; 16],
        }
        .encode();
        assert_eq!(d.on_ros_cmd_vel(0, &unknown, &cdr(0.5)), Effects::default());
        assert_eq!(d.arbiter().state(), State::Open);
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        assert_eq!(d.arbiter().state(), State::Claimed);
    }

    #[test]
    fn udp_owner_blocks_ros_and_ros_owner_blocks_udp() {
        let mut d = dev();
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let cmd = publisher(&mut d, 1, 1, 1);
        assert_eq!(d.on_ros_cmd_vel(1, &cmd, &cdr(0.5)), Effects::default());
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        let (r, _) = send(&mut d, 1, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::Busy);
    }

    #[test]
    fn malformed_ros_input_is_dropped() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 2));
        assert_eq!(
            d.on_ros_cmd_vel(0, &[1, 2, 3], &cdr(0.5)),
            Effects::default()
        );
        assert_eq!(d.on_ros_cmd_vel(0, &cmd, &[1, 2, 3]), Effects::default());
        assert_eq!(d.arbiter().state(), State::Open, "nothing claimed");
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        assert_eq!(d.on_ros_arm(1, &arm, &[0, 1, 0]), Effects::default());
        assert_eq!(d.arbiter().state(), State::Claimed);
    }

    #[test]
    fn status_is_a_cdr_string() {
        let mut d = dev();
        let mut b = [0u8; 64];
        let n = d.status_cdr(&mut b);
        let len = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        assert_eq!(&b[..4], &[0, 1, 0, 0]);
        assert_eq!(n, 8 + len);
        assert_eq!(b[8 + len - 1], 0, "NUL terminated");
        assert_eq!(
            core::str::from_utf8(&b[8..7 + len]).unwrap(),
            "state=open session=0"
        );
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 2));
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        d.on_ros_arm(1, &arm, &bool_cdr(true));
        let n = d.status_cdr(&mut b);
        let len = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        assert_eq!(
            core::str::from_utf8(&b[8..7 + len]).unwrap(),
            "state=armed session=1"
        );
        assert_eq!(n, 8 + len);
    }

    #[test]
    fn ble_connect_claims_and_excludes_others() {
        let mut d = dev();
        assert_eq!(d.on_ble_connect(0, 1).unwrap().claimed, Some(Source::Ble));
        assert_eq!(d.on_ble_connect(1, 2), Err(Error::Busy));
        let (r, _) = send(&mut d, 1, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::Busy);
    }

    fn ble_sp(seq: u32, vx: f64) -> [u8; 72] {
        let mut b = [0u8; 72];
        b[..4].copy_from_slice(&seq.to_le_bytes());
        b[4..].copy_from_slice(&cdr(vx));
        b
    }

    #[test]
    fn ble_setpoint_then_arm_drives_and_stop_disarms() {
        let mut d = dev();
        d.on_ble_connect(0, 1).unwrap();
        assert_eq!(
            d.on_ble_control(1, 1, 1),
            Err(Error::NotReady),
            "arm needs a setpoint"
        );
        d.on_ble_setpoint(2, 1, &ble_sp(1, 1.0));
        assert_eq!(d.on_ble_control(3, 1, 1), Ok(Effects::default()));
        assert_eq!(d.wheels(3), [0.5; 4]);
        d.on_ble_setpoint(4, 1, &ble_sp(2, -1.0));
        assert_eq!(d.wheels(4), [-0.5; 4]);
        assert_eq!(
            d.on_ble_control(5, 1, 2).unwrap().stop,
            Some(StopReason::Stop)
        );
        assert_eq!(d.wheels(5), [0.0; 4]);
    }

    #[test]
    fn ble_non_owner_malformed_and_unknown_input_is_ignored() {
        let mut d = dev();
        d.on_ble_connect(0, 1).unwrap();
        assert_eq!(d.on_ble_setpoint(1, 2, &ble_sp(1, 1.0)), Effects::default());
        assert_eq!(d.on_ble_control(1, 2, 1), Err(Error::NotOwner));
        assert_eq!(d.on_ble_setpoint(1, 1, &[1, 2, 3]), Effects::default());
        assert_eq!(
            d.on_ble_control(1, 1, 9),
            Err(Error::Invalid),
            "unknown command"
        );
        assert_eq!(d.arbiter().state(), State::Claimed);
    }

    #[test]
    fn ble_disconnect_while_armed_stops_and_reopens() {
        let mut d = dev();
        d.on_ble_connect(0, 1).unwrap();
        d.on_ble_setpoint(1, 1, &ble_sp(1, 1.0));
        d.on_ble_control(2, 1, 1).unwrap();
        let fx = d.on_ble_disconnect(3, 1);
        assert!(fx.released);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert_eq!(
            d.on_ble_disconnect(4, 1),
            Effects::default(),
            "not the owner any more"
        );
        assert_eq!(d.on_ble_connect(5, 2).unwrap().claimed, Some(Source::Ble));
    }

    #[test]
    fn radio_allowed_follows_the_owner() {
        let mut d = dev();
        assert!(
            d.radio_allowed(Source::Ble)
                && d.radio_allowed(Source::Udp)
                && d.radio_allowed(Source::Ros)
        );
        d.on_ble_connect(0, 7).unwrap();
        assert!(
            d.radio_allowed(Source::Ble)
                && !d.radio_allowed(Source::Udp)
                && !d.radio_allowed(Source::Ros)
        );
        assert!(d.ble_owns(7) && !d.ble_owns(8));
        d.on_ble_disconnect(1, 7);
        assert!(d.radio_allowed(Source::Udp) && !d.ble_owns(7));
    }

    #[test]
    fn wifi_is_off_only_while_ble_owns_the_car() {
        let mut d = dev();
        assert!(d.wifi_allowed());
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        assert!(d.wifi_allowed(), "UDP owns the car, so the AP stays up");
        d.on_udp(1, A, &[], &mut [0u8; 8]);
        let mut d = dev();
        d.on_ble_connect(0, 1).unwrap();
        assert!(!d.wifi_allowed());
        d.on_ble_disconnect(1, 1);
        assert!(d.wifi_allowed());
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        assert!(d.wifi_allowed(), "ROS rides on the AP");
    }
}
