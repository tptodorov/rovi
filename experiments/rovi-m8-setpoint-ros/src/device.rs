//! The device core: the arbiter plus kinematics, driven by UDP datagrams. The BLE and ROS
//! adapters will feed the same arbiter. Time is passed in; the firmware shell owns the sockets.

use crate::arbiter::{Arbiter, Claimant, Config, Effects, Error, Source, State, StopReason};
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
}

impl Device {
    pub fn new(cfg: Config, geometry: Geometry) -> Self {
        Self {
            cfg,
            geometry,
            arbiter: Arbiter::new(cfg),
        }
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

    /// A sample on the `cmd_vel` topic. The first valid one from a free device claims it, and the
    /// owner is the publisher gid in the attachment. Input from other gids is ignored.
    pub fn on_ros_cmd_vel(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let (Some(att), Ok(sp)) = (Attachment::decode(attachment), Setpoint::decode(payload))
        else {
            return Effects::default();
        };
        let who = Claimant {
            source: Source::Ros,
            id: u128::from_le_bytes(att.gid),
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
        let who = Claimant {
            source: Source::Ros,
            id: u128::from_le_bytes(att.gid),
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

    fn capabilities(&self) -> [u8; CAPABILITIES_LEN] {
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

    fn status(&self, error: Option<Error>) -> [u8; STATUS_LEN] {
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
        };
        [state, reason, error]
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

    fn ros(gid: u8, seq: i64) -> [u8; 33] {
        Attachment {
            seq,
            timestamp: 0,
            gid: [gid; 16],
        }
        .encode()
    }

    fn bool_cdr(v: bool) -> [u8; 5] {
        [0, 1, 0, 0, v as u8]
    }

    #[test]
    fn first_ros_setpoint_claims_then_arm_drives() {
        let mut d = dev();
        let fx = d.on_ros_cmd_vel(0, &ros(1, 1), &cdr(1.0));
        assert_eq!(fx.claimed, Some(Source::Ros));
        assert_eq!(d.wheels(0), [0.0; 4], "claimed but not armed");
        d.on_ros_arm(5, &ros(1, 2), &bool_cdr(true));
        assert_eq!(d.wheels(5), [0.5; 4]);
        let fx = d.on_ros_arm(6, &ros(1, 3), &bool_cdr(false));
        assert_eq!(fx.stop, Some(StopReason::Stop));
        assert_eq!(d.wheels(6), [0.0; 4]);
    }

    #[test]
    fn other_ros_gid_is_ignored_while_owned() {
        let mut d = dev();
        d.on_ros_cmd_vel(0, &ros(1, 1), &cdr(0.5));
        d.on_ros_arm(1, &ros(1, 2), &bool_cdr(true));
        assert_eq!(
            d.on_ros_cmd_vel(2, &ros(2, 1), &cdr(-1.0)),
            Effects::default()
        );
        assert_eq!(
            d.on_ros_arm(2, &ros(2, 1), &bool_cdr(false)),
            Effects::default()
        );
        assert_eq!(
            d.wheels(2),
            [0.25; 4],
            "owner's setpoint and arm are untouched"
        );
    }

    #[test]
    fn udp_owner_blocks_ros_and_ros_owner_blocks_udp() {
        let mut d = dev();
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(
            d.on_ros_cmd_vel(1, &ros(1, 1), &cdr(0.5)),
            Effects::default()
        );
        let mut d = dev();
        d.on_ros_cmd_vel(0, &ros(1, 1), &cdr(0.5));
        let (r, _) = send(&mut d, 1, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::Busy);
    }

    #[test]
    fn malformed_ros_input_is_dropped() {
        let mut d = dev();
        assert_eq!(
            d.on_ros_cmd_vel(0, &[1, 2, 3], &cdr(0.5)),
            Effects::default()
        );
        assert_eq!(
            d.on_ros_cmd_vel(0, &ros(1, 1), &[1, 2, 3]),
            Effects::default()
        );
        assert_eq!(d.arbiter().state(), State::Open, "nothing claimed");
        d.on_ros_cmd_vel(0, &ros(1, 1), &cdr(0.5));
        assert_eq!(d.on_ros_arm(1, &ros(1, 2), &[0, 1, 0]), Effects::default());
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
        d.on_ros_cmd_vel(0, &ros(1, 1), &cdr(0.5));
        d.on_ros_arm(1, &ros(1, 2), &bool_cdr(true));
        let n = d.status_cdr(&mut b);
        let len = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        assert_eq!(
            core::str::from_utf8(&b[8..7 + len]).unwrap(),
            "state=armed session=1"
        );
        assert_eq!(n, 8 + len);
    }
}
