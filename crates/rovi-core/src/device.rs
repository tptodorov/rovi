//! The device core: the arbiter plus kinematics, driven by UDP datagrams. The BLE and ROS
//! adapters will feed the same arbiter. Time is passed in; the firmware shell owns the sockets.

use crate::arbiter::{Arbiter, Claimant, Config, Effects, Source};
use crate::graph::Graph;
use crate::kinematics::{self, Geometry};
use crate::network::{Mode, Network, Step};
use rovi_protocol::capabilities::Capabilities;
use rovi_protocol::network::Command;
use rovi_protocol::ros::Attachment;
use rovi_protocol::setpoint::Setpoint;
use rovi_protocol::status::{Error, State, Status, StopReason};
use rovi_protocol::udp::{self, Frame, FromCar, Header, Kind, ToCar};

use rovi_protocol::capabilities::LEN as CAPABILITIES_LEN;
use rovi_protocol::status::LEN as STATUS_LEN;
use rovi_protocol::ECHO_MAX;

/// BLE connections the device tracks: one owner and one watcher (ADR-0016).
pub const BLE_SLOTS: usize = 2;
/// The latency echo: the payload to send back unchanged, or `None` if it is too long. Stateless:
/// any peer may echo, owner or not, and it never touches the arbiter or the lease.
pub fn echo(data: &[u8]) -> Option<&[u8]> {
    (data.len() <= ECHO_MAX).then_some(data)
}

/// Prepare an Echo notification: `Some(0)` is an accepted empty reply; `None` means discard.
pub fn ble_echo(offset: usize, data: &[u8], out: &mut [u8]) -> Option<usize> {
    if offset != 0 {
        return None;
    }
    let payload = echo(data)?;
    out.get_mut(..payload.len())?.copy_from_slice(payload);
    Some(payload.len())
}

/// A reply to send back to the datagram's sender (`len` 0 means none) and the effects for the
/// shell to act on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    pub len: usize,
    pub effects: Effects,
}

/// One BLE connection: the stack's handle, which it reuses, and a generation the device counts,
/// so a callback from a closed connection cannot act for a newer one with the same handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BleConn {
    pub handle: u16,
    generation: u32,
}

/// A controller was taken over: tell it on its own transport, once (spec *Takeover notice*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notice {
    pub source: Source,
    /// The UDP peer id, BLE token or ROS node of the old owner.
    pub id: u128,
    /// The old owner's session.
    pub session: u32,
}

impl Notice {
    /// What the old owner is told: stopped, owner-lost, in its own session.
    pub fn status(&self) -> Status {
        Status {
            state: State::Stopped(StopReason::OwnerLost),
            error: None,
            session: self.session,
        }
    }

    /// The unsolicited UDP `STATUS` for the old session (sequence 0); 0 if `out` is too small.
    pub fn udp_frame(&self, out: &mut [u8]) -> usize {
        let header = Header {
            kind: Kind::Status,
            session: self.session,
            seq: 0,
        };
        udp::write(out, header, &self.status().encode()).unwrap_or(0)
    }

    /// The `rovi/status` sample for the old ROS owner.
    pub fn ros_cdr(&self, out: &mut [u8]) -> usize {
        self.status().encode_cdr(out).unwrap_or(0)
    }
}

/// Why a device profile is refused. The device does not start, so nothing can claim it and the
/// motors are never enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    /// Radius, centre projection sum or maximum wheel speed is not a positive finite f32, or a
    /// frame offset is not finite.
    Geometry,
    /// A speed limit is not a finite, nonnegative f32.
    Limits,
    /// A lease or window length is outside `1..=65535` ms (the capabilities carry u16).
    Times,
}

pub struct Device {
    cfg: Config,
    geometry: Geometry,
    arbiter: Arbiter,
    graph: Graph<16>,
    /// The owner's last refused ARM/STOP, retained until its next successful control command.
    owner_error: Option<Error>,
    /// Coordinator readiness, installed before transports start (the sim has no motor driver).
    motor_ready: bool,
    /// Deadline effects an input processed but could not return (a refused BLE write); the
    /// next [`tick`](Self::tick) returns them.
    pending: Effects,
    /// ROS ordering for the current owner session, per publisher.
    marks: Marks,
    /// The last BLE connection's generation; 0 is never handed out.
    ble_generation: u32,
    /// Last delivered UDP status, including owner identity; avoids duplicating direct replies.
    last_udp_status: Option<(u128, Status)>,
    /// Home or fallback networking (ADR-0016). Fallback, today's behaviour, until the firmware
    /// starts it with home settings.
    network: Network,
    /// A takeover ended without a Home hand-back: the next ROS owner must arm explicitly.
    handback_latch: bool,
    /// The station's DHCP address; 0.0.0.0 without one.
    station_ip: [u8; 4],
    /// The last mode command (source, command, refusal) for the adapter to log.
    mode_log: Option<(Source, Command, Option<Error>)>,
    /// Open BLE connections; only these can claim with a write.
    ble_live: [Option<BleConn>; BLE_SLOTS],
    /// The last takeover, for the adapter to tell the old owner.
    notice: Option<Notice>,
}

/// Owner publishers the ROS ordering table tracks per session, as many as the graph holds (the
/// design's 16). A node usually has two (cmd_vel and arm); marks outlive a publisher's removal,
/// so input from one more is discarded rather than taken without a replay check.
const MARKS: usize = 16;

/// The highest accepted ROS sequence per owner publisher GID in one session. Kept apart from the
/// graph, so a publisher's removal or other announcements cannot reset it.
struct Marks {
    session: u32,
    gids: [[u8; 16]; MARKS],
    seqs: [i64; MARKS],
    len: usize,
}

impl Marks {
    fn new(session: u32) -> Self {
        Self {
            session,
            gids: [[0; 16]; MARKS],
            seqs: [0; MARKS],
            len: 0,
        }
    }

    fn is_new(&self, session: u32, gid: [u8; 16], seq: i64) -> bool {
        seq >= 0
            && (session != self.session
                || self.gids[..self.len]
                    .iter()
                    .position(|g| *g == gid)
                    .map_or(self.len < MARKS, |i| seq > self.seqs[i]))
    }

    /// Records `seq` from `gid` if it is above that publisher's mark in `session` (a new session
    /// starts empty). False for a negative, repeated or older sequence, or a full table.
    fn accept(&mut self, session: u32, gid: [u8; 16], seq: i64) -> bool {
        if !self.is_new(session, gid, seq) {
            return false;
        }
        if session != self.session {
            *self = Marks::new(session);
        }
        match self.gids[..self.len].iter().position(|g| *g == gid) {
            Some(i) if seq > self.seqs[i] => {
                self.seqs[i] = seq;
                true
            }
            Some(_) => false,
            None if self.len < MARKS => {
                self.gids[self.len] = gid;
                self.seqs[self.len] = seq;
                self.len += 1;
                true
            }
            None => false,
        }
    }
}

/// The profile with its speed limits as the advertised f32 values, or why it is refused.
fn validated(cfg: Config, g: &Geometry) -> Result<Config, ProfileError> {
    // Advertised as f32, so a positive f64 must stay positive and finite as an f32.
    let positive = |x: f64| x.is_finite() && (x as f32).is_finite() && x as f32 > 0.0;
    let offsets = [g.offset_x, g.offset_y, g.offset_theta];
    if !(positive(g.wheels_radius)
        && positive(g.center_projection_sum)
        && positive(g.max_wheel_speed)
        && offsets.iter().all(|o| o.is_finite()))
    {
        return Err(ProfileError::Geometry);
    }
    let limit = |x: f64| {
        let advertised = x as f32;
        (x.is_finite() && x >= 0.0 && advertised.is_finite()).then_some(f64::from(advertised))
    };
    let (Some(max_vx), Some(max_vy), Some(max_wz)) =
        (limit(cfg.max_vx), limit(cfg.max_vy), limit(cfg.max_wz))
    else {
        return Err(ProfileError::Limits);
    };
    let time = |ms: u64| (1..=u64::from(u16::MAX)).contains(&ms);
    if !(time(cfg.lease_ms) && time(cfg.claim_window_ms) && time(cfg.reclaim_window_ms)) {
        return Err(ProfileError::Times);
    }
    Ok(Config {
        max_vx,
        max_vy,
        max_wz,
        ..cfg
    })
}

/// Takeover priority (ADR-0016): a higher rank may take the car from a lower one.
fn rank(source: Source) -> u8 {
    match source {
        Source::Ros => 0,
        Source::Udp => 1,
        Source::Ble => 2,
    }
}

/// `a` then `b`, as one set of effects. `events::effects` logs a stop, then a release, then a
/// claim, which is also the order they happen in when a deadline releases the car and the same
/// input claims it again.
fn merge(a: Effects, b: Effects) -> Effects {
    Effects {
        preempted: b.preempted.or(a.preempted),
        claimed: b.claimed.or(a.claimed),
        released: a.released || b.released,
        stop: a.stop.or(b.stop),
        lease_late_ms: if a.stop.is_some() {
            a.lease_late_ms
        } else {
            b.lease_late_ms
        },
    }
}

impl Device {
    /// Starts the device with a validated profile. Speed limits are kept as the f32 values the
    /// capabilities advertise, so a limit a controller reads is one it can send. Session IDs start
    /// after `session_seed`, which the apps draw at random each boot.
    pub fn new(cfg: Config, geometry: Geometry, session_seed: u32) -> Result<Self, ProfileError> {
        let cfg = validated(cfg, &geometry)?;
        Ok(Self {
            cfg,
            geometry,
            arbiter: Arbiter::seeded(cfg, session_seed),
            graph: Graph::new(),
            owner_error: None,
            motor_ready: true,
            pending: Effects::default(),
            marks: Marks::new(session_seed),
            ble_generation: 0,
            last_udp_status: None,
            network: Network::boot(0, false),
            handback_latch: false,
            station_ip: [0; 4],
            mode_log: None,
            ble_live: [None; BLE_SLOTS],
            notice: None,
        })
    }

    /// Bind motor readiness before starting transports; invalid motor configuration inhibits ARM.
    /// The simulator uses the default ready state without a physical driver dependency.
    pub fn set_motor_ready(&mut self, ready: bool) {
        self.motor_ready = ready;
    }

    fn arm(&mut self, now: u64, who: Claimant) -> Result<(), Error> {
        self.arbiter.check_owner(who)?;
        if !self.motor_ready {
            return Err(Error::NotReady);
        }
        self.arbiter.arm(now, who)
    }

    fn expire(&mut self, now: u64) -> Effects {
        self.network.tick(now);
        let owner = self.arbiter.owner();
        let fx = self.arbiter.tick(now);
        self.ended(now, owner, fx)
    }

    /// Starts home or fallback networking at boot: home only with home settings and ROS 2.
    pub fn start_network(&mut self, now: u64, home_supported: bool) {
        self.network = Network::boot(now, home_supported);
    }

    /// Starts in fallback after a mode-switch reset, keeping the reason the switch had.
    pub fn start_network_fallback(
        &mut self,
        home_supported: bool,
        reason: rovi_protocol::network::Reason,
    ) {
        self.network = Network::boot_fallback(home_supported, reason);
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    /// The firmware finished a home-mode step (scan, join, DHCP, router).
    pub fn on_network_step(&mut self, now: u64, step: Step, ok: bool) {
        self.network.on_step(now, step, ok);
    }

    /// The station lost the home network.
    pub fn on_station_lost(&mut self, now: u64) {
        self.network.on_station_lost(now);
    }

    /// The station's DHCP address, or `None` when it has none.
    pub fn set_station_ip(&mut self, ip: Option<[u8; 4]>) {
        self.station_ip = ip.unwrap_or([0; 4]);
    }

    /// The BLE Network characteristic and the network part of UDP `INFO`.
    pub fn network_info(&self) -> [u8; 8] {
        self.network.info(self.station_ip).encode()
    }

    /// Writes the `rovi/network` sample (`std_msgs/String` CDR) and returns its length.
    pub fn network_cdr(&self, out: &mut [u8]) -> usize {
        self.network
            .info(self.station_ip)
            .encode_cdr(out)
            .unwrap_or(0)
    }

    /// The last takeover of a `source` owner, once: that transport's adapter sends the old owner
    /// its final status. Each adapter takes only its own notices.
    pub fn take_takeover_notice(&mut self, source: Source) -> Option<Notice> {
        self.notice
            .filter(|n| n.source == source)
            .and_then(|_| self.notice.take())
    }

    /// The last mode command, once, for the adapter's `ev=mode_cmd` line.
    pub fn take_mode_log(&mut self) -> Option<(Source, Command, Option<Error>)> {
        self.mode_log.take()
    }

    /// A mode command (`[1]` Home, `[2]` Car network) from the owner while disarmed; it applies
    /// when that ownership ends. Refusals are kept as the owner's last control error.
    fn mode_command(&mut self, who: Claimant, payload: &[u8]) -> Result<Effects, Error> {
        self.arbiter.check_owner(who)?;
        let cmd = match payload {
            [1] => Some(Command::Home),
            [2] => Some(Command::CarNetwork),
            _ => None,
        };
        let result = match cmd {
            None => Err(Error::Invalid),
            Some(Command::Home) if !self.network.home_supported() => Err(Error::Invalid),
            Some(_) if self.arbiter.state() == State::Armed => Err(Error::NotReady),
            Some(c) => {
                self.network.command(c);
                Ok(Effects::default())
            }
        };
        self.owner_error = result.err();
        if let Some(c) = cmd {
            self.mode_log = Some((who.source, c, result.err()));
        }
        result
    }

    /// Bookkeeping after `fx` from an input or a deadline, given the owner before it: a released
    /// owner's pending mode command applies, and a Home hand-back clears the ROS latch.
    fn ended(&mut self, now: u64, owner: Option<Claimant>, fx: Effects) -> Effects {
        if fx.released {
            self.owner_error = None;
            // A BLE owner that lost the car is being disconnected; it cannot claim again.
            if let Some(o) = owner.filter(|o| o.source == Source::Ble) {
                for slot in &mut self.ble_live {
                    if slot.is_some_and(|c| ble(c) == o) {
                        *slot = None;
                    }
                }
            }
            if let (Some(o), None) = (owner, fx.preempted) {
                if o.source != Source::Ros && self.network.pending() == Some(Command::Home) {
                    self.handback_latch = false;
                }
                self.network.on_ownership_end(now, o.source == Source::Ble);
            }
        }
        fx
    }

    /// Claims for `who`, taking the car from a lower-priority owner where ADR-0016 allows:
    /// UDP over ROS in both modes, BLE over UDP or ROS in home mode only.
    fn claim(&mut self, now: u64, who: Claimant) -> Result<Effects, Error> {
        match self.arbiter.claim(now, who) {
            Err(Error::Busy) => {
                let owner = self.arbiter.owner().ok_or(Error::Busy)?;
                let home = self.network.mode() != Mode::Fallback;
                let may =
                    rank(who.source) > rank(owner.source) && (who.source != Source::Ble || home);
                if !may {
                    return Err(Error::Busy);
                }
                self.network.discard_pending();
                self.notice = Some(Notice {
                    source: owner.source,
                    id: owner.id,
                    session: self.arbiter.session(),
                });
                if owner.source == Source::Ros {
                    self.handback_latch = true;
                }
                self.owner_error = None;
                Ok(self.arbiter.preempt(now, who))
            }
            other => other,
        }
    }

    fn control(&mut self, now: u64, who: Claimant, arm: Option<bool>) -> Result<Effects, Error> {
        let result = match arm {
            Some(true) => self.arm(now, who).map(|()| Effects::default()),
            Some(false) => self.arbiter.stop(now, who),
            None => self.arbiter.check_owner(who).and(Err(Error::Invalid)),
        };
        match result {
            Ok(_) => self.owner_error = None,
            Err(e @ (Error::NotReady | Error::Invalid)) => self.owner_error = Some(e),
            Err(_) => {}
        }
        result
    }

    /// May `source`'s radio be on? While one source owns the car, the others' radios are off,
    /// except that in home mode BLE always advertises, so it can take over (ADR-0016), and BLE is
    /// quiet while a home attempt runs.
    pub fn radio_allowed(&self, source: Source) -> bool {
        match (source, self.network.mode()) {
            // BLE advertising during a home attempt stretched the Wi-Fi scan from 160 ms to 2.7 s
            // on the board (coexistence, 2026-10-26); the car is not drivable then anyway.
            (Source::Ble, Mode::Attempting(_)) => false,
            (Source::Ble, Mode::Home) => true,
            _ => self.arbiter.owner().is_none_or(|o| o.source == source),
        }
    }

    /// May the Wi-Fi AP be up? UDP and ROS share it, so it is off only while BLE owns the car.
    pub fn wifi_allowed(&self) -> bool {
        self.arbiter.owner().is_none_or(|o| o.source != Source::Ble)
    }

    /// Is `conn` still the BLE owner?
    pub fn ble_owns(&self, conn: BleConn) -> bool {
        self.arbiter.owner() == Some(ble(conn))
    }

    pub fn arbiter(&self) -> &Arbiter {
        &self.arbiter
    }

    /// Handles one UDP datagram from `peer` (an id the shell keeps stable per remote address).
    pub fn on_udp(&mut self, now: u64, peer: u128, datagram: &[u8], out: &mut [u8]) -> Reply {
        let due = self.expire(now);
        let mut r = self.udp_input(now, peer, datagram, out);
        r.effects = merge(due, r.effects);
        r
    }

    fn udp_input(&mut self, now: u64, peer: u128, datagram: &[u8], out: &mut [u8]) -> Reply {
        let Some((h, payload)) = udp::parse(datagram) else {
            return Reply::default();
        };
        let who = Claimant {
            source: Source::Udp,
            id: peer,
        };
        let ours = h.session == self.arbiter.session();
        let Ok(frame) = Frame::<ToCar>::decode(datagram) else {
            // An owner's malformed ARM/STOP/MODE reports invalid; other malformed frames are
            // discarded (spec *Input validation before state changes*).
            return match h.kind {
                Kind::Arm | Kind::Stop if ours => self.control_reply(now, who, None, h, out),
                Kind::Mode if ours => {
                    let result = self.mode_command(who, payload);
                    self.result_reply(result, h, out)
                }
                _ => Reply::default(),
            };
        };
        let send = |message: FromCar, session, out: &mut [u8]| {
            Frame {
                session,
                seq: h.seq,
                message,
            }
            .encode(out)
            .unwrap_or(0)
        };
        match frame.message {
            ToCar::Echo(p) => Reply {
                len: send(FromCar::Echo(p), h.session, out),
                ..Reply::default()
            },
            ToCar::Info => {
                let info = udp::Info {
                    capabilities: self.capabilities_snapshot(),
                    status: self.status_snapshot(),
                    network: self.network.info(self.station_ip),
                };
                Reply {
                    len: send(FromCar::Info(info), 0, out),
                    ..Reply::default()
                }
            }
            ToCar::Hello => match self.claim(now, who) {
                Ok(effects) => {
                    if effects.claimed.is_some() {
                        self.owner_error = None;
                    }
                    let caps = FromCar::HelloAck(self.capabilities_snapshot());
                    let len = send(caps, self.arbiter.session(), out);
                    Reply { len, effects }
                }
                Err(_) => Reply {
                    len: send(FromCar::Busy, self.arbiter.session(), out),
                    ..Reply::default()
                },
            },
            // Input from another session or a non-owner is dropped without a reply.
            ToCar::Setpoint(sp) if ours => match self.arbiter.setpoint(now, who, h.seq, sp) {
                Ok((_, effects)) => Reply { len: 0, effects },
                Err(_) => Reply::default(),
            },
            ToCar::Arm if ours => self.control_reply(now, who, Some(true), h, out),
            ToCar::Stop if ours => self.control_reply(now, who, Some(false), h, out),
            ToCar::Mode(c) if ours => {
                let result = self.mode_command(who, &[c.code()]);
                self.result_reply(result, h, out)
            }
            ToCar::Bye if ours => match self.arbiter.release(now, who) {
                Ok(effects) => Reply {
                    len: 0,
                    effects: self.ended(now, Some(who), effects),
                },
                Err(_) => Reply::default(),
            },
            _ => Reply::default(),
        }
    }

    fn control_reply(
        &mut self,
        now: u64,
        who: Claimant,
        command: Option<bool>,
        h: Header,
        out: &mut [u8],
    ) -> Reply {
        let result = self.control(now, who, command);
        self.result_reply(result, h, out)
    }

    /// Owner commands get a Status reply; a non-owner gets none.
    fn result_reply(&mut self, result: Result<Effects, Error>, h: Header, out: &mut [u8]) -> Reply {
        match result {
            Ok(effects) => self.status_reply(effects, h, out),
            Err(Error::NotOwner) => Reply::default(),
            Err(_) => self.status_reply(Effects::default(), h, out),
        }
    }

    fn status_reply(&mut self, effects: Effects, h: Header, out: &mut [u8]) -> Reply {
        let header = Header {
            kind: Kind::Status,
            session: self.arbiter.session(),
            seq: h.seq,
        };
        let len = udp::write(out, header, &self.status()).unwrap_or(0);
        if len > 0 {
            if let Some(owner) = self.arbiter.owner() {
                self.last_udp_status = Some((owner.id, self.status_snapshot()));
            }
        }
        Reply { len, effects }
    }

    /// A BLE central connected and claims at once (M8 behaviour, until the firmware moves to
    /// [`on_ble_attach`](Self::on_ble_attach)). Returns the connection's token for every later
    /// call; a refused claim leaves the connection without one (the adapter disconnects it).
    pub fn on_ble_connect(&mut self, now: u64, handle: u16) -> Result<(BleConn, Effects), Error> {
        let conn = self.on_ble_attach(now, handle);
        let due = core::mem::take(&mut self.pending);
        let result = self.claim(now, ble(conn));
        if result.is_ok() {
            self.owner_error = None;
        } else {
            self.detach(conn);
        }
        self.returned(due, result).map(|fx| (conn, fx))
    }

    /// A BLE central connected. It can read and get notifications without claiming; its first
    /// Setpoint or Control write claims (ADR-0016). Returns the token for every later call.
    pub fn on_ble_attach(&mut self, now: u64, handle: u16) -> BleConn {
        self.ble_generation = match self.ble_generation.wrapping_add(1) {
            0 => 1,
            g => g,
        };
        let conn = BleConn {
            handle,
            generation: self.ble_generation,
        };
        let due = self.expire(now);
        self.pending = merge(self.pending, due);
        if let Some(slot) = self.ble_live.iter_mut().find(|s| s.is_none()) {
            *slot = Some(conn);
        }
        conn
    }

    fn detach(&mut self, conn: BleConn) {
        for slot in &mut self.ble_live {
            if *slot == Some(conn) {
                *slot = None;
            }
        }
    }

    /// Claims for a live BLE connection that does not own the car yet.
    fn ble_claim(&mut self, now: u64, conn: BleConn) -> Result<Effects, Error> {
        if self.ble_owns(conn) {
            return Ok(Effects::default());
        }
        if !self.ble_live.contains(&Some(conn)) {
            return Err(Error::NotOwner);
        }
        let fx = self.claim(now, ble(conn))?;
        self.owner_error = None;
        Ok(fx)
    }

    /// Returns `due` with an accepted input's effects, or keeps it for the next tick.
    fn returned(&mut self, due: Effects, result: Result<Effects, Error>) -> Result<Effects, Error> {
        match result {
            Ok(fx) => Ok(merge(due, fx)),
            Err(e) => {
                self.pending = merge(self.pending, due);
                Err(e)
            }
        }
    }

    /// A write to the Setpoint characteristic: `seq u32` (LE) then the CDR `TwistStamped`.
    pub fn on_ble_setpoint(&mut self, now: u64, conn: BleConn, data: &[u8]) -> Effects {
        let due = self.expire(now);
        // A setpoint that does not decode must not claim.
        let decodes = data
            .split_first_chunk::<4>()
            .is_some_and(|(_, cdr)| Setpoint::decode(cdr).is_ok());
        if !decodes {
            return due;
        }
        let claimed = match self.ble_claim(now, conn) {
            Ok(fx) => fx,
            Err(_) => return due,
        };
        merge(merge(due, claimed), self.ble_setpoint(now, conn, data))
    }

    fn ble_setpoint(&mut self, now: u64, conn: BleConn, data: &[u8]) -> Effects {
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

    /// A write to the Control characteristic: 1 = ARM, 2 = STOP, 3 = Home, 4 = Car network.
    /// An error rejects the write.
    pub fn on_ble_control(&mut self, now: u64, conn: BleConn, cmd: u8) -> Result<Effects, Error> {
        let due = self.expire(now);
        let result = self.ble_claim(now, conn).and_then(|claimed| {
            self.ble_control(now, conn, cmd)
                .map(|fx| merge(claimed, fx))
        });
        self.returned(due, result)
    }

    fn ble_control(&mut self, now: u64, conn: BleConn, cmd: u8) -> Result<Effects, Error> {
        match cmd {
            3 | 4 => self.mode_command(ble(conn), &[cmd - 2]),
            1 | 2 => self.control(now, ble(conn), Some(cmd == 1)),
            _ => self.control(now, ble(conn), None),
        }
    }

    /// The Status characteristic for the BLE owner: the state and the last control error.
    pub fn ble_status(&self) -> [u8; STATUS_LEN] {
        self.status()
    }

    /// The central disconnected.
    pub fn on_ble_disconnect(&mut self, now: u64, conn: BleConn) -> Effects {
        let due = self.expire(now);
        self.detach(conn);
        let fx = self.arbiter.release(now, ble(conn)).unwrap_or_default();
        let fx = self.ended(now, Some(ble(conn)), fx);
        merge(due, fx)
    }

    /// The ROS adapter lost its router session. A ROS owner is released at once (a stop if it was
    /// armed); without this, only its lease would stop the car. Other owners are not affected.
    pub fn on_ros_disconnect(&mut self, now: u64) -> Effects {
        let due = self.expire(now);
        let owner = self.arbiter.owner();
        let fx = match owner {
            Some(o) if o.source == Source::Ros => self.arbiter.release(now, o).unwrap_or_default(),
            _ => Effects::default(),
        };
        let fx = self.ended(now, owner, fx);
        merge(due, fx)
    }

    /// A liveliness token appeared (`alive`) or went away. Samples are attributed to a ROS node
    /// through the publisher tokens learned here; samples from unknown publishers are dropped.
    pub fn on_ros_liveliness(&mut self, key: &str, alive: bool) {
        self.graph.on_token(key, alive);
    }

    /// Is the publisher in this sample attachment one the car has seen announced? `None` for an
    /// attachment that does not parse. An unknown publisher usually means a liveliness token was
    /// lost (UDP does not resend), so the adapter asks the router for the tokens again.
    pub fn ros_publisher_known(&self, attachment: &[u8]) -> Option<bool> {
        let att = Attachment::decode(attachment)?;
        Some(self.graph.node_of(&att.gid).is_some())
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
    /// ROS arms on its own (ADR-0011): a setpoint arms a claimed car, and a stream resuming after
    /// a lease expiry arms it again, so stock tools that publish only `cmd_vel` can drive. A STOP
    /// or an out-of-limits setpoint stays latched while the owner streams, until `rovi/arm` true.
    pub fn on_ros_cmd_vel(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let due = self.expire(now);
        merge(due, self.ros_cmd_vel(now, attachment, payload))
    }

    fn ros_cmd_vel(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let (Some(att), Ok(sp)) = (Attachment::decode(attachment), Setpoint::decode(payload))
        else {
            return Effects::default();
        };
        let Some(who) = self.ros_claimant(&att) else {
            return Effects::default();
        };
        // An out-of-limits or negatively numbered sample must not claim a free car.
        if self.arbiter.owner().is_none() && (!self.arbiter.within_limits(&sp) || att.seq < 0) {
            return Effects::default();
        }
        let Ok(mut fx) = self.claim(now, who) else {
            return Effects::default();
        };
        if fx.claimed.is_some() {
            self.owner_error = None;
        }
        // Owner only from here: order the sample against its own publisher's mark.
        if !self.marks.accept(self.arbiter.session(), att.gid, att.seq) {
            return fx;
        }
        if let Ok(e) = self.arbiter.setpoint_ordered(now, who, sp) {
            fx.stop = e.stop;
            let auto_arm = matches!(
                self.arbiter.state(),
                State::Claimed | State::Stopped(StopReason::LeaseExpired)
            );
            if e.stop.is_none() && auto_arm && !self.handback_latch {
                let _ = self.arm(now, who);
            }
        }
        fx
    }

    /// A sample on `rovi/arm` (`std_msgs/Bool`): true arms, false stops.
    pub fn on_ros_arm(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let due = self.expire(now);
        merge(due, self.ros_arm(now, attachment, payload))
    }

    fn ros_arm(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let Some(att) = Attachment::decode(attachment) else {
            return Effects::default();
        };
        let Some(who) = self.ros_claimant(&att) else {
            return Effects::default();
        };
        if self.arbiter.owner() != Some(who)
            || !self.marks.is_new(self.arbiter.session(), att.gid, att.seq)
        {
            return Effects::default();
        }
        let valid = payload.len() == 5 && payload[..4] == [0, 1, 0, 0] && payload[4] <= 1;
        if !valid {
            let _ = self.control(now, who, None);
            return Effects::default();
        }
        self.marks.accept(self.arbiter.session(), att.gid, att.seq);
        let arm = payload[4] == 1;
        let result = self.control(now, who, Some(arm));
        if arm && result.is_ok() {
            self.handback_latch = false;
        }
        result.unwrap_or_default()
    }

    /// A sample on `rovi/release` (`std_msgs/Empty`) from the owner node: release at once, like a
    /// UDP BYE (an owner-lost stop if armed); a pending mode command applies.
    pub fn on_ros_release(&mut self, now: u64, attachment: &[u8], _payload: &[u8]) -> Effects {
        let due = self.expire(now);
        let Some(att) = Attachment::decode(attachment) else {
            return due;
        };
        let Some(who) = self.ros_claimant(&att) else {
            return due;
        };
        if self.arbiter.owner() != Some(who)
            || !self.marks.accept(self.arbiter.session(), att.gid, att.seq)
        {
            return due;
        }
        let fx = self.arbiter.release(now, who).unwrap_or_default();
        let fx = self.ended(now, Some(who), fx);
        merge(due, fx)
    }

    /// A sample on `rovi/mode` (`std_msgs/UInt8`): 1 = Home, 2 = Car network, from the owner node.
    pub fn on_ros_mode(&mut self, now: u64, attachment: &[u8], payload: &[u8]) -> Effects {
        let due = self.expire(now);
        let Some(att) = Attachment::decode(attachment) else {
            return due;
        };
        let Some(who) = self.ros_claimant(&att) else {
            return due;
        };
        if self.arbiter.owner() != Some(who)
            || !self.marks.accept(self.arbiter.session(), att.gid, att.seq)
        {
            return due;
        }
        let value = match payload {
            [0, 1, 0, 0, _] => &payload[4..],
            _ => &[],
        };
        let _ = self.mode_command(who, value);
        due
    }

    /// Writes the `rovi/status` sample (`std_msgs/String` CDR) and returns its length.
    pub fn status_cdr(&self, out: &mut [u8]) -> usize {
        self.status_snapshot().encode_cdr(out).unwrap_or(0)
    }

    /// Call every control tick (10 ms). Also returns deadline effects a refused input kept.
    pub fn tick(&mut self, now: u64) -> Effects {
        let kept = core::mem::take(&mut self.pending);
        merge(kept, self.expire(now))
    }

    /// Motor control failed (the coordinator reports it): an out-of-limits stop.
    pub fn on_motor_fault(&mut self, now: u64) -> Effects {
        self.arbiter.fault_stop(now)
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
        self.capabilities_snapshot().encode()
    }

    /// The same canonical values for every transport, readable without claiming the device.
    pub fn capabilities_snapshot(&self) -> Capabilities {
        Capabilities {
            version: rovi_protocol::capabilities::VERSION,
            wheels_radius: self.geometry.wheels_radius as f32,
            center_projection_sum: self.geometry.center_projection_sum as f32,
            max_vx: self.cfg.max_vx as f32,
            max_vy: self.cfg.max_vy as f32,
            max_wz: self.cfg.max_wz as f32,
            lease_ms: self.cfg.lease_ms as u16,
            claim_ms: self.cfg.claim_window_ms as u16,
            reclaim_ms: self.cfg.reclaim_window_ms as u16,
        }
    }

    /// The Status characteristic (and the `STATUS` payload).
    pub fn status(&self) -> [u8; STATUS_LEN] {
        self.status_snapshot().encode()
    }

    /// Current state/error/session for every source. Release exposes open with no stale error.
    pub fn status_snapshot(&self) -> Status {
        let owned = self.arbiter.owner().is_some();
        Status {
            state: self.arbiter.state(),
            error: if owned { self.owner_error } else { None },
            session: if owned { self.arbiter.session() } else { 0 },
        }
    }

    /// A stop-transition STATUS for the UDP owner. Call even while no datagrams arrive.
    /// Returns peer id / frame length; sequence 0 denotes an unsolicited transition.
    pub fn udp_status_notification(&mut self, out: &mut [u8]) -> Option<(u128, usize)> {
        let Some(owner) = self.arbiter.owner().filter(|o| o.source == Source::Udp) else {
            self.last_udp_status = None;
            return None;
        };
        let status = self.status_snapshot();
        let current = (owner.id, status);
        if self.last_udp_status == Some(current) {
            return None;
        }
        if !matches!(status.state, State::Stopped(_)) {
            self.last_udp_status = Some(current);
            return None;
        }
        let n = udp::write(
            out,
            Header {
                kind: Kind::Status,
                session: status.session,
                seq: 0,
            },
            &status.encode(),
        )?;
        self.last_udp_status = Some(current);
        Some((owner.id, n))
    }
}

fn ble(conn: BleConn) -> Claimant {
    Claimant {
        source: Source::Ble,
        id: u128::from(conn.generation) << 16 | u128::from(conn.handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ROS-style `TwistStamped` CDR with the given frame id (rovi-protocol only encodes empty).
    fn with_frame(frame: &[u8], nanosec: u32, vx: f64) -> std::vec::Vec<u8> {
        let mut b = std::vec::Vec::from([0u8, 1, 0, 0]);
        b.extend_from_slice(&7i32.to_le_bytes());
        b.extend_from_slice(&nanosec.to_le_bytes());
        b.extend_from_slice(&(frame.len() as u32 + 1).to_le_bytes());
        b.extend_from_slice(frame);
        b.push(0);
        while (b.len() - 4) % 8 != 0 {
            b.push(0);
        }
        for v in [vx, 0.0, 0.0, 0.0, 0.0, 0.0] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b
    }
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
        Device::new(Config::DEFAULT, GEOMETRY, 0).unwrap()
    }

    #[test]
    fn input_expiry_keeps_its_lateness_when_the_same_input_renews_the_lease() {
        let mut d = dev();
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        d.on_ble_setpoint(10, conn, &ble_sp(1, 0.0));
        d.on_ble_control(11, conn, 1).unwrap();
        let fx = d.on_ble_setpoint(1_061, conn, &ble_sp(2, 0.0));
        assert_eq!(fx.stop, Some(StopReason::LeaseExpired));
        assert_eq!(fx.lease_late_ms, Some(51));
        assert_eq!(d.tick(1_062).lease_late_ms, None);
    }

    #[test]
    fn refused_control_preserves_detection_time_until_pending_effects_are_drained() {
        let mut d = dev();
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        d.on_ble_setpoint(10, conn, &ble_sp(1, 0.0));
        d.on_ble_control(11, conn, 1).unwrap();
        assert!(d.on_ble_control(1_060, conn, 1).is_err());
        let fx = d.tick(1_200);
        assert_eq!(fx.stop, Some(StopReason::LeaseExpired));
        assert_eq!(fx.lease_late_ms, Some(50));
        assert_eq!(d.tick(1_201).lease_late_ms, None);
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

    /// Spec *One model per concept*: BLE, UDP `INFO` and ROS report equal values (ROS Status
    /// also carries the session, design T6).
    fn assert_transports_agree(d: &mut Device, now: u64) {
        // INFO is observational: a watcher's query changes nothing.
        let (r, _) = send(d, now, 0xFFFF, Kind::Info, 0, 77, &[]);
        let (h, p) = r.unwrap();
        let mut b = [0u8; 128];
        let n = udp::write(&mut b, h, &p[..udp::INFO_LEN]).unwrap();
        let info = match Frame::<FromCar>::decode(&b[..n]).unwrap().message {
            FromCar::Info(i) => i,
            m => panic!("{m:?}"),
        };
        let ble = Status::decode(&d.ble_status()).unwrap();
        let mut cdr = [0u8; 384];
        let n = d.status_cdr(&mut cdr);
        let ros = Status::decode_cdr(&cdr[..n]).unwrap();
        assert_eq!(info.status, ble);
        assert_eq!(Status { session: 0, ..ros }, ble);
        let net = rovi_protocol::NetworkInfo::decode(&d.network_info()).unwrap();
        let n = d.network_cdr(&mut cdr);
        let ros_net = rovi_protocol::NetworkInfo::decode_cdr(&cdr[..n]).unwrap();
        assert_eq!((info.network, ros_net), (net, net));
        let caps = Capabilities::decode(&d.capabilities()).unwrap();
        let n = d.capabilities_snapshot().encode_cdr(&mut cdr).unwrap();
        let ros_caps = Capabilities::decode_cdr(&cdr[..n]).unwrap();
        assert_eq!((info.capabilities, ros_caps), (caps, caps));
    }

    #[test]
    fn every_transport_reports_the_same_status_network_and_capabilities() {
        let mut d = dev();
        d.set_station_ip(Some([192, 168, 1, 7]));
        assert_transports_agree(&mut d, 0);
        let s = armed(&mut d);
        assert_transports_agree(&mut d, 0);
        send(&mut d, 1, A, Kind::Stop, s, 3, &[]);
        assert_transports_agree(&mut d, 1);
        send(&mut d, 2, A, Kind::Arm, s, 4, &[1]);
        assert_transports_agree(&mut d, 2);
    }

    #[test]
    fn malformed_hello_or_info_is_discarded_without_claiming() {
        // Spec *Input validation before state changes*: malformed framing SHALL NOT claim.
        let mut d = dev();
        let (r, fx) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[1]);
        assert!(r.is_none());
        assert_eq!(fx.claimed, None);
        assert_eq!(d.arbiter().owner(), None);
        let (r, _) = send(&mut d, 0, B, Kind::Info, 0, 2, &[1]);
        assert!(r.is_none());
    }

    #[test]
    fn an_owners_malformed_control_reports_invalid() {
        let mut d = dev();
        let s = armed(&mut d);
        for (kind, payload) in [
            (Kind::Arm, &[1u8][..]),
            (Kind::Stop, &[0]),
            (Kind::Mode, &[3]),
            (Kind::Mode, &[]),
        ] {
            let (r, _) = send(&mut d, 1, A, kind, s, 9, payload);
            let (h, p) = r.unwrap();
            assert_eq!((h.kind, h.seq), (Kind::Status, 9), "{kind:?}");
            assert_eq!(p[2], Error::Invalid.code(), "{kind:?}");
        }
        let (r, _) = send(&mut d, 1, B, Kind::Arm, s, 9, &[1]);
        assert!(r.is_none(), "a non-owner gets no reply");
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

    #[test]
    fn echo_answers_any_peer_with_the_same_bytes() {
        let mut d = dev();
        let s = armed(&mut d);
        let (r, fx) = send(
            &mut d,
            5,
            B,
            Kind::Echo,
            77,
            42,
            b"0123456789abcdef0123456789abcdef",
        );
        let (h, p) = r.unwrap();
        assert_eq!(
            (h.kind, h.session, h.seq),
            (Kind::Echo, 77, 42),
            "header echoed as sent"
        );
        assert_eq!(&p[..32], b"0123456789abcdef0123456789abcdef");
        assert_eq!(fx, Effects::default());
        let (r, _) = send(&mut d, 5, A, Kind::Echo, s, 3, &[]);
        assert_eq!(
            r.unwrap().0.kind,
            Kind::Echo,
            "empty payload, from the owner"
        );
    }

    #[test]
    fn echo_never_claims_nor_renews_the_lease() {
        let mut d = dev();
        send(&mut d, 0, A, Kind::Echo, 0, 1, b"t");
        assert_eq!(d.arbiter().state(), State::Open, "echo is not a claim");
        armed(&mut d);
        for t in (100..1_000).step_by(100) {
            send(&mut d, t, A, Kind::Echo, 0, t as u32, b"t");
            send(&mut d, t, B, Kind::Echo, 0, t as u32, b"t");
        }
        assert_eq!(d.arbiter().state(), State::Armed);
        assert_eq!(
            d.tick(1_000).stop,
            Some(StopReason::LeaseExpired),
            "lease not renewed"
        );
    }

    #[test]
    fn oversize_echo_is_dropped() {
        let mut d = dev();
        let (r, fx) = send(&mut d, 0, B, Kind::Echo, 0, 1, &[7; ECHO_MAX + 1]);
        assert!(r.is_none() && fx == Effects::default());
        assert_eq!(echo(&[7; ECHO_MAX + 1]), None, "BLE uses the same rule");
        assert_eq!(echo(&[7; ECHO_MAX]), Some(&[7u8; ECHO_MAX][..]));
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
            gid: rovi_protocol::ros::gid(&key),
        }
        .encode()
    }

    fn bool_cdr(v: bool) -> [u8; 5] {
        [0, 1, 0, 0, v as u8]
    }

    /// Attachment of publisher `cmd` with sequence `seq` (the same gid, a later sample).
    fn at(cmd: &[u8; 33], seq: i64) -> [u8; 33] {
        let mut a = *cmd;
        a[..8].copy_from_slice(&seq.to_le_bytes());
        a
    }

    #[test]
    fn first_ros_setpoint_claims_and_arms() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let fx = d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(fx.claimed, Some(Source::Ros));
        assert_eq!(d.arbiter().state(), State::Armed, "ROS arms on its own");
        assert_eq!(d.wheels(0), [0.5; 4]);
    }

    #[test]
    fn ros_stream_resuming_after_a_lease_expiry_arms_again() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(d.tick(1_000).stop, Some(StopReason::LeaseExpired));
        assert_eq!(d.wheels(1_000), [0.0; 4]);
        d.on_ros_cmd_vel(1_500, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.arbiter().state(), State::Armed);
        assert_eq!(d.wheels(1_500), [0.5; 4]);
    }

    #[test]
    fn ros_stop_latches_while_the_owner_streams_until_it_arms() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        let fx = d.on_ros_arm(5, &arm, &bool_cdr(false));
        assert_eq!(fx.stop, Some(StopReason::Stop));
        d.on_ros_cmd_vel(10, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.wheels(10), [0.0; 4], "a stop is not undone by the stream");
        d.on_ros_arm(15, &at(&arm, 2), &bool_cdr(true));
        assert_eq!(d.wheels(15), [0.5; 4], "an explicit arm resumes");
    }

    #[test]
    fn ros_out_of_limits_latches_while_the_owner_streams() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(
            d.on_ros_cmd_vel(10, &at(&cmd, 2), &cdr(9.0)).stop,
            Some(StopReason::OutOfLimits)
        );
        d.on_ros_cmd_vel(20, &at(&cmd, 3), &cdr(1.0));
        assert_eq!(d.wheels(20), [0.0; 4]);
    }

    #[test]
    fn ros_owner_that_goes_quiet_after_a_stop_is_released_and_a_new_stream_arms() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        d.on_ros_arm(5, &arm, &bool_cdr(false));
        d.tick(1_000); // lease lapses: the reclaim window starts
        assert!(d.tick(6_000).released);
        let fx = d.on_ros_cmd_vel(6_100, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(fx.claimed, Some(Source::Ros));
        assert_eq!(d.wheels(6_100), [0.5; 4]);
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
        assert_eq!(d.arbiter().state(), State::Armed);
    }

    #[test]
    fn udp_owner_blocks_ros_and_udp_takes_over_from_ros() {
        let mut d = dev();
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let cmd = publisher(&mut d, 1, 1, 1);
        assert_eq!(d.on_ros_cmd_vel(1, &cmd, &cdr(0.5)), Effects::default());
        // ADR-0016 replaced "a ROS owner blocks UDP": UDP now takes over from ROS.
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        let (r, _) = send(&mut d, 1, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::HelloAck);
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
        assert_eq!(
            d.arbiter().state(),
            State::Armed,
            "a short arm sample does not stop"
        );
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
            "state=open reason=none error=none session=0"
        );
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 2));
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        d.on_ros_arm(1, &arm, &bool_cdr(true));
        let n = d.status_cdr(&mut b);
        let len = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        assert_eq!(
            core::str::from_utf8(&b[8..7 + len]).unwrap(),
            "state=armed reason=none error=none session=1"
        );
        assert_eq!(n, 8 + len);
    }

    #[test]
    fn ble_connect_claims_and_excludes_others() {
        let mut d = dev();
        assert_eq!(d.on_ble_connect(0, 1).unwrap().1.claimed, Some(Source::Ble));
        assert_eq!(d.on_ble_connect(1, 2), Err(Error::Busy));
        let (r, _) = send(&mut d, 1, A, Kind::Hello, 0, 1, &[]);
        assert_eq!(r.unwrap().0.kind, Kind::Busy);
    }

    /// A BLE connection the device never handed out.
    fn fake(handle: u16) -> BleConn {
        BleConn {
            handle,
            generation: 0,
        }
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
        let c = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(
            d.on_ble_control(1, c, 1),
            Err(Error::NotReady),
            "arm needs a setpoint"
        );
        d.on_ble_setpoint(2, c, &ble_sp(1, 1.0));
        assert_eq!(d.on_ble_control(3, c, 1), Ok(Effects::default()));
        assert_eq!(d.wheels(3), [0.5; 4]);
        d.on_ble_setpoint(4, c, &ble_sp(2, -1.0));
        assert_eq!(d.wheels(4), [-0.5; 4]);
        assert_eq!(
            d.on_ble_control(5, c, 2).unwrap().stop,
            Some(StopReason::Stop)
        );
        assert_eq!(d.wheels(5), [0.0; 4]);
    }

    #[test]
    fn ble_non_owner_malformed_and_unknown_input_is_ignored() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(
            d.on_ble_setpoint(1, fake(2), &ble_sp(1, 1.0)),
            Effects::default()
        );
        assert_eq!(d.on_ble_control(1, fake(2), 1), Err(Error::NotOwner));
        assert_eq!(d.on_ble_setpoint(1, c, &[1, 2, 3]), Effects::default());
        assert_eq!(
            d.on_ble_control(1, c, 9),
            Err(Error::Invalid),
            "unknown command"
        );
        assert_eq!(d.arbiter().state(), State::Claimed);
    }

    #[test]
    fn ble_disconnect_while_armed_stops_and_reopens() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        d.on_ble_setpoint(1, c, &ble_sp(1, 1.0));
        d.on_ble_control(2, c, 1).unwrap();
        let fx = d.on_ble_disconnect(3, c);
        assert!(fx.released);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert_eq!(
            d.on_ble_disconnect(4, c),
            Effects::default(),
            "not the owner any more"
        );
        assert_eq!(d.on_ble_connect(5, 2).unwrap().1.claimed, Some(Source::Ble));
    }

    #[test]
    fn radio_allowed_follows_the_owner() {
        let mut d = dev();
        assert!(
            d.radio_allowed(Source::Ble)
                && d.radio_allowed(Source::Udp)
                && d.radio_allowed(Source::Ros)
        );
        let c = d.on_ble_connect(0, 7).unwrap().0;
        assert!(
            d.radio_allowed(Source::Ble)
                && !d.radio_allowed(Source::Udp)
                && !d.radio_allowed(Source::Ros)
        );
        assert!(d.ble_owns(c) && !d.ble_owns(fake(8)));
        d.on_ble_disconnect(1, c);
        assert!(d.radio_allowed(Source::Udp) && !d.ble_owns(c));
    }

    #[test]
    fn wifi_is_off_only_while_ble_owns_the_car() {
        let mut d = dev();
        assert!(d.wifi_allowed());
        send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        assert!(d.wifi_allowed(), "UDP owns the car, so the AP stays up");
        d.on_udp(1, A, &[], &mut [0u8; 8]);
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        assert!(!d.wifi_allowed());
        d.on_ble_disconnect(1, c);
        assert!(d.wifi_allowed());
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        assert!(d.wifi_allowed(), "ROS rides on the AP");
    }

    #[test]
    fn ble_status_carries_the_last_control_error_until_a_control_write_succeeds() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(
            d.ble_status(),
            [1, 0, 0],
            "claimed, no error, readable right after connect"
        );
        assert_eq!(d.on_ble_control(1, c, 1), Err(Error::NotReady));
        assert_eq!(
            d.ble_status(),
            [1, 0, 3],
            "a refused ARM shows as NotReady, as over UDP"
        );
        assert_eq!(d.on_ble_control(2, c, 9), Err(Error::Invalid));
        assert_eq!(d.ble_status(), [1, 0, 4]);
        d.on_ble_setpoint(3, c, &ble_sp(1, 1.0));
        assert_eq!(d.ble_status(), [1, 0, 4], "a setpoint does not clear it");
        d.on_ble_control(4, c, 1).unwrap();
        assert_eq!(
            d.ble_status(),
            [2, 0, 0],
            "a control write that succeeds clears it"
        );
    }

    #[test]
    fn a_new_ble_connection_starts_without_an_error() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        let _ = d.on_ble_control(1, c, 1);
        d.on_ble_disconnect(2, c);
        d.on_ble_connect(3, 2).unwrap();
        assert_eq!(d.ble_status(), [1, 0, 0]);
    }

    #[test]
    fn a_non_owner_write_does_not_change_the_owners_status() {
        let mut d = dev();
        d.on_ble_connect(0, 1).unwrap();
        assert_eq!(d.on_ble_control(1, fake(2), 1), Err(Error::NotOwner));
        assert_eq!(d.ble_status(), [1, 0, 0]);
    }

    #[test]
    fn an_unknown_publisher_is_told_apart_from_a_malformed_attachment() {
        let mut d = dev();
        let known = publisher(&mut d, 1, 1, 1);
        let unknown = Attachment {
            seq: 1,
            timestamp: 0,
            gid: [9; 16],
        }
        .encode();
        assert_eq!(d.ros_publisher_known(&known), Some(true));
        assert_eq!(
            d.ros_publisher_known(&unknown),
            Some(false),
            "its token has not been seen"
        );
        assert_eq!(
            d.ros_publisher_known(&[1, 2, 3]),
            None,
            "not an attachment at all"
        );
        // Once its token arrives (a refresh answered), it is known.
        let key = "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/3/3/MP/%/%/n3/%t3/std_msgs::msg::dds_::Bool_/RIHS01_x/::,10:,:,:,,";
        let late = Attachment {
            seq: 2,
            timestamp: 0,
            gid: rovi_protocol::ros::gid(key),
        }
        .encode();
        assert_eq!(d.ros_publisher_known(&late), Some(false));
        d.on_ros_liveliness(key, true);
        assert_eq!(d.ros_publisher_known(&late), Some(true));
    }

    #[test]
    fn a_motor_fault_stops_and_a_ros_stream_does_not_rearm_it() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(d.on_motor_fault(5).stop, Some(StopReason::OutOfLimits));
        d.on_ros_cmd_vel(10, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.arbiter().state(), State::Stopped(StopReason::OutOfLimits));
        assert_eq!(
            d.wheels(10),
            [0.0; 4],
            "the stream does not recover a motor fault"
        );
        d.on_ros_arm(15, &at(&arm, 2), &bool_cdr(true));
        assert_eq!(
            d.wheels(15),
            [0.5; 4],
            "an explicit arm after a fresh setpoint resumes"
        );
    }

    #[test]
    fn a_udp_arm_after_a_motor_fault_needs_a_fresh_setpoint() {
        let mut d = dev();
        let s = armed(&mut d);
        d.on_motor_fault(5);
        let (r, _) = send(&mut d, 10, A, Kind::Arm, s, 3, &[]);
        assert_ne!(r.unwrap().1[0], 2, "not armed without a fresh setpoint");
        send(&mut d, 20, A, Kind::Setpoint, s, 4, &cdr(0.5));
        let (r, _) = send(&mut d, 20, A, Kind::Arm, s, 5, &[]);
        assert_eq!(r.unwrap().1[0], 2, "armed");
    }

    #[test]
    fn an_unsupported_frame_neither_renews_the_lease_nor_uses_the_sequence() {
        let mut d = dev();
        let s = armed(&mut d); // setpoint seq 1 and ARM at t=0
        send(
            &mut d,
            900,
            A,
            Kind::Setpoint,
            s,
            3,
            &with_frame(b"map", 0, 0.5),
        );
        assert_eq!(
            d.tick(1_000).stop,
            Some(StopReason::LeaseExpired),
            "map did not renew"
        );

        let mut d = dev();
        let s = armed(&mut d);
        send(
            &mut d,
            900,
            A,
            Kind::Setpoint,
            s,
            3,
            &with_frame(b"map", 0, 0.5),
        );
        send(
            &mut d,
            950,
            A,
            Kind::Setpoint,
            s,
            3,
            &with_frame(b"base_link", 0, 0.5),
        );
        assert_eq!(
            d.tick(1_000).stop,
            None,
            "the same sequence in base_link is still accepted"
        );
    }

    #[test]
    fn an_old_stamp_is_accepted_and_the_lease_runs_from_receive_time() {
        let mut d = dev();
        let s = armed(&mut d);
        send(
            &mut d,
            900,
            A,
            Kind::Setpoint,
            s,
            3,
            &with_frame(b"", 0, 0.5),
        );
        assert_eq!(
            d.tick(1_000).stop,
            None,
            "renewed at 900 despite a zero stamp"
        );
        assert_eq!(d.tick(1_900).stop, Some(StopReason::LeaseExpired));
    }

    #[test]
    fn a_bad_first_ros_input_does_not_claim() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let fx = d.on_ros_cmd_vel(0, &cmd, &cdr(9.0));
        assert_eq!(fx, Effects::default(), "out of limits: no claim, no stop");
        assert_eq!(d.arbiter().state(), State::Open);
        let mut nan = cdr(0.5);
        nan[20..28].copy_from_slice(&f64::NAN.to_le_bytes());
        assert_eq!(d.on_ros_cmd_vel(10, &at(&cmd, 2), &nan), Effects::default());
        assert_eq!(d.arbiter().state(), State::Open);
        assert_eq!(
            d.on_ros_cmd_vel(20, &at(&cmd, 3), &cdr(1.0)).claimed,
            Some(Source::Ros)
        );
    }

    // Deadline ordering: input at time t first processes every deadline at or before t, even when
    // no control tick ran in between.

    #[test]
    fn a_setpoint_on_the_lease_deadline_cannot_renew_the_expired_lease() {
        let mut d = dev();
        let s = armed(&mut d); // lease until 1000
        let (_, fx) = send(&mut d, 1_000, A, Kind::Setpoint, s, 3, &cdr(0.5));
        assert_eq!(fx.stop, Some(StopReason::LeaseExpired));
        assert_eq!(
            d.arbiter().state(),
            State::Stopped(StopReason::LeaseExpired)
        );
        assert_eq!(d.wheels(1_000), [0.0; 4]);
    }

    #[test]
    fn a_setpoint_on_the_claim_deadline_cannot_revive_the_claim() {
        let mut d = dev();
        let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        let (_, fx) = send(&mut d, 10_000, A, Kind::Setpoint, s, 2, &cdr(0.5));
        assert!(fx.released, "the claim window ended first");
        assert_eq!(d.arbiter().state(), State::Open, "UDP needs a new claim");
    }

    #[test]
    fn arm_and_setpoint_on_the_reclaim_deadline_cannot_drive_the_expired_session() {
        let mut d = dev();
        let s = armed(&mut d);
        assert_eq!(d.tick(1_000).stop, Some(StopReason::LeaseExpired)); // reclaim until 6000
        let (_, fx) = send(&mut d, 6_000, A, Kind::Setpoint, s, 3, &cdr(0.5));
        assert!(fx.released, "released first");
        send(&mut d, 6_000, A, Kind::Arm, s, 4, &[]);
        assert_eq!(d.arbiter().state(), State::Open);
        assert_eq!(d.wheels(6_000), [0.0; 4]);
    }

    #[test]
    fn a_ros_sample_on_the_reclaim_deadline_releases_then_claims_anew() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        d.on_ros_arm(5, &arm, &bool_cdr(false)); // STOP latched
        d.tick(1_000); // lease lapses: reclaim until 6000
        let fx = d.on_ros_cmd_vel(6_000, &at(&cmd, 2), &cdr(1.0));
        assert!(fx.released && fx.claimed == Some(Source::Ros), "{fx:?}");
        let mut log = std::vec::Vec::new();
        crate::events::effects(fx, |e| log.push(e));
        assert_eq!(
            log,
            [
                crate::events::Event::Released,
                crate::events::Event::Claimed(Source::Ros)
            ],
            "release logged before the new claim"
        );
    }

    #[test]
    fn a_refused_ble_write_after_a_deadline_still_reports_the_release() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 7).unwrap().0;
        let mut w = [0u8; 4 + 68];
        w[4..].copy_from_slice(&cdr(0.5));
        d.on_ble_setpoint(0, c, &w);
        d.on_ble_control(0, c, 1).unwrap();
        d.tick(1_000); // lease expiry, reclaim until 6000
        assert_eq!(
            d.on_ble_control(6_000, c, 1),
            Err(Error::NotOwner),
            "the session is over"
        );
        assert!(
            d.tick(6_005).released,
            "the release reaches the log on the next tick"
        );
    }

    // Valid operating profile.

    #[test]
    fn the_car_profile_is_valid() {
        assert!(Device::new(Config::DEFAULT, Geometry::CAR, 0).is_ok());
    }

    #[test]
    fn an_invalid_geometry_fails_to_start() {
        for g in [
            Geometry {
                max_wheel_speed: 0.0,
                ..GEOMETRY
            },
            Geometry {
                max_wheel_speed: f64::NAN,
                ..GEOMETRY
            },
            Geometry {
                max_wheel_speed: -40.0,
                ..GEOMETRY
            },
            Geometry {
                wheels_radius: 0.0,
                ..GEOMETRY
            },
            Geometry {
                wheels_radius: f64::INFINITY,
                ..GEOMETRY
            },
            Geometry {
                wheels_radius: 1e-300,
                ..GEOMETRY
            }, // advertised as f32 0.0
            Geometry {
                center_projection_sum: -0.2,
                ..GEOMETRY
            },
            Geometry {
                offset_x: f64::NAN,
                ..GEOMETRY
            },
            Geometry {
                offset_theta: f64::INFINITY,
                ..GEOMETRY
            },
        ] {
            assert_eq!(
                Device::new(Config::DEFAULT, g, 0).err(),
                Some(ProfileError::Geometry),
                "{g:?}"
            );
        }
    }

    #[test]
    fn invalid_speed_limits_fail_to_start() {
        for cfg in [
            Config {
                max_vx: f64::NAN,
                ..Config::DEFAULT
            },
            Config {
                max_vy: -0.1,
                ..Config::DEFAULT
            },
            Config {
                max_wz: 1e39,
                ..Config::DEFAULT
            }, // not an f32
            Config {
                max_wz: f64::INFINITY,
                ..Config::DEFAULT
            },
        ] {
            assert_eq!(
                Device::new(cfg, GEOMETRY, 0).err(),
                Some(ProfileError::Limits),
                "{cfg:?}"
            );
        }
    }

    #[test]
    fn times_outside_one_to_65535_ms_fail_to_start() {
        for cfg in [
            Config {
                lease_ms: 0,
                ..Config::DEFAULT
            },
            Config {
                lease_ms: 65_536,
                ..Config::DEFAULT
            },
            Config {
                claim_window_ms: 70_000,
                ..Config::DEFAULT
            },
            Config {
                reclaim_window_ms: 0,
                ..Config::DEFAULT
            },
        ] {
            assert_eq!(
                Device::new(cfg, GEOMETRY, 0).err(),
                Some(ProfileError::Times),
                "{cfg:?}"
            );
        }
    }

    #[test]
    fn boundary_profiles_start() {
        let cfg = Config {
            lease_ms: 1,
            claim_window_ms: 65_535,
            reclaim_window_ms: 65_535,
            max_vx: 0.0,
            ..Config::DEFAULT
        };
        assert!(Device::new(cfg, GEOMETRY, 0).is_ok());
    }

    #[test]
    fn a_limit_read_from_the_capabilities_is_accepted() {
        // 0.3 is not an f32: the car advertises 0.30000001192..., and a controller that sends
        // exactly that must not be stopped for exceeding 0.3.
        let cfg = Config {
            max_vx: 0.3,
            ..Config::DEFAULT
        };
        let mut d = Device::new(cfg, GEOMETRY, 0).unwrap();
        let caps = d.capabilities();
        let max_vx = f64::from(f32::from_le_bytes(caps[9..13].try_into().unwrap()));
        let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        send(&mut d, 0, A, Kind::Setpoint, s, 1, &cdr(0.1));
        send(&mut d, 0, A, Kind::Arm, s, 2, &[]);
        let (_, fx) = send(&mut d, 10, A, Kind::Setpoint, s, 3, &cdr(max_vx));
        assert_eq!(fx.stop, None, "{max_vx} is the advertised limit");
        assert_eq!(d.arbiter().state(), State::Armed);
    }

    // Input validation before state changes.

    /// Setpoint payloads that must not renew a lease or use a sequence.
    fn malformed() -> std::vec::Vec<std::vec::Vec<u8>> {
        let mut nan = cdr(0.5);
        nan[20..28].copy_from_slice(&f64::NAN.to_le_bytes());
        let mut out_of_plane = cdr(0.5);
        out_of_plane[36..44].copy_from_slice(&0.1f64.to_le_bytes()); // linear.z
        std::vec![
            nan.to_vec(),
            cdr(0.5)[..40].to_vec(),
            std::vec::Vec::new(),
            out_of_plane.to_vec(),
            with_frame(b"map", 0, 0.5),
        ]
    }

    #[test]
    fn malformed_setpoints_neither_renew_the_lease_nor_use_the_sequence() {
        let mut d = dev();
        let s = armed(&mut d); // seq 1 at t=0: lease until 1000
        for p in malformed() {
            let (_, fx) = send(&mut d, 500, A, Kind::Setpoint, s, 3, &p);
            assert_eq!(fx, Effects::default());
        }
        assert_eq!(
            d.wheels(500),
            [0.25; 4],
            "the previous output (0.5 m/s) stays"
        );
        assert_eq!(
            d.tick(1_000).stop,
            Some(StopReason::LeaseExpired),
            "not renewed"
        );

        let mut d = dev();
        let s = armed(&mut d);
        for p in malformed() {
            send(&mut d, 500, A, Kind::Setpoint, s, 3, &p);
        }
        send(&mut d, 600, A, Kind::Setpoint, s, 3, &cdr(0.25));
        assert_eq!(
            d.wheels(600),
            [0.125; 4],
            "sequence 3 (0.25 m/s) was still free"
        );
    }

    #[test]
    fn stale_or_non_owner_oversized_input_cannot_stop_the_owner() {
        let mut d = dev();
        let s = armed(&mut d);
        send(&mut d, 10, A, Kind::Setpoint, s, 10, &cdr(0.5));
        let (_, fx) = send(&mut d, 20, A, Kind::Setpoint, s, 9, &cdr(9.0));
        assert_eq!(fx.stop, None, "stale: sequence 9 after 10");
        let (_, fx) = send(&mut d, 30, B, Kind::Setpoint, s, 11, &cdr(9.0));
        assert_eq!(fx.stop, None, "not the owner");
        assert_eq!(d.arbiter().state(), State::Armed);
        assert_eq!(
            d.tick(1_010).stop,
            Some(StopReason::LeaseExpired),
            "neither renewed it"
        );
    }

    #[test]
    fn another_ros_nodes_oversized_sample_cannot_stop_the_owner() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let other = publisher(&mut d, 2, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        assert_eq!(d.on_ros_cmd_vel(10, &other, &cdr(9.0)).stop, None);
        assert_eq!(d.arbiter().state(), State::Armed);
    }

    // ROS sequence domains: strictly increasing i64 per publisher GID, per session.

    fn lv_key(nid: u8, id: u8) -> std::string::String {
        format!(
            "@ros2_lv/0/9ad20db4c929c8189ca587de2f90c2e2/{nid}/{id}/MP/%/%/n{nid}/%t{id}/std_msgs::msg::dds_::Bool_/RIHS01_x/::,10:,:,:,,"
        )
    }

    #[test]
    fn same_node_publishers_have_independent_counters() {
        let mut d = dev();
        let (a, b, arm) = (
            publisher(&mut d, 1, 1, 1),
            publisher(&mut d, 1, 3, 1),
            publisher(&mut d, 1, 2, 1),
        );
        d.on_ros_cmd_vel(0, &at(&a, 100), &cdr(1.0));
        d.on_ros_cmd_vel(10, &at(&b, 1), &cdr(0.5));
        assert_eq!(d.wheels(10), [0.25; 4], "B's first sample is accepted");
        d.on_ros_cmd_vel(20, &at(&a, 100), &cdr(1.0));
        assert_eq!(d.wheels(20), [0.25; 4], "A's duplicate is dropped");
        let fx = d.on_ros_arm(30, &at(&arm, 1), &bool_cdr(false));
        assert_eq!(
            fx.stop,
            Some(StopReason::Stop),
            "the arm publisher's 1 is its own counter"
        );
    }

    #[test]
    fn ros_sequences_compare_the_full_i64() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &at(&cmd, u32::MAX as i64), &cdr(0.5));
        d.on_ros_cmd_vel(10, &at(&cmd, 1 << 32), &cdr(1.0));
        assert_eq!(d.wheels(10), [0.5; 4], "2^32 follows 2^32 - 1");
    }

    #[test]
    fn a_negative_ros_sequence_is_rejected() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        assert_eq!(
            d.on_ros_cmd_vel(0, &at(&cmd, -1), &cdr(0.5)),
            Effects::default()
        );
        assert_eq!(d.arbiter().state(), State::Open);
    }

    #[test]
    fn a_replayed_ros_arm_is_ignored() {
        let mut d = dev();
        let (cmd, arm) = (publisher(&mut d, 1, 1, 1), publisher(&mut d, 1, 2, 1));
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        d.on_ros_arm(5, &at(&arm, 5), &bool_cdr(false)); // STOP
        d.on_ros_cmd_vel(10, &at(&cmd, 2), &cdr(1.0)); // a fresh setpoint
        d.on_ros_arm(15, &at(&arm, 5), &bool_cdr(true));
        assert_eq!(
            d.wheels(15),
            [0.0; 4],
            "ARM with an old sequence is a replay"
        );
        d.on_ros_arm(20, &at(&arm, 6), &bool_cdr(true));
        assert_eq!(d.wheels(20), [0.5; 4]);
    }

    #[test]
    fn ordering_marks_survive_a_publishers_removal() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &at(&cmd, 100), &cdr(1.0));
        d.on_ros_liveliness(&lv_key(1, 1), false);
        d.on_ros_liveliness(&lv_key(1, 1), true);
        d.on_ros_cmd_vel(10, &at(&cmd, 99), &cdr(0.5));
        assert_eq!(
            d.wheels(10),
            [0.5; 4],
            "99 after 100 is rejected after re-announcement"
        );
    }

    #[test]
    fn unrelated_announcements_cannot_reset_ordering() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &at(&cmd, 100), &cdr(1.0));
        for nid in 2..40 {
            publisher(&mut d, nid, 1, 1);
        }
        d.on_ros_cmd_vel(10, &at(&cmd, 99), &cdr(0.5));
        assert_ne!(d.wheels(10), [0.25; 4], "99 did not drive");
        assert_eq!(
            d.tick(1_000).stop,
            Some(StopReason::LeaseExpired),
            "nor renew the lease"
        );
    }

    #[test]
    fn an_owner_publisher_beyond_the_ordering_table_is_discarded() {
        let mut d = dev();
        let pubs: std::vec::Vec<_> = (1..=16).map(|id| publisher(&mut d, 1, id, 1)).collect();
        for (i, p) in pubs.iter().enumerate() {
            d.on_ros_cmd_vel(i as u64, p, &cdr(0.05 * (i + 1) as f64));
        }
        assert_eq!(
            d.wheels(15),
            [0.4; 4],
            "all sixteen (the last at 0.8 m/s) are tracked"
        );
        // One leaves the graph but keeps its mark, so a seventeenth has no ordering slot.
        d.on_ros_liveliness(&lv_key(1, 1), false);
        let extra = publisher(&mut d, 1, 17, 1);
        d.on_ros_cmd_vel(16, &extra, &cdr(0.2));
        assert_eq!(d.wheels(16), [0.4; 4], "the seventeenth is discarded");
        assert_eq!(
            d.tick(1_015).stop,
            Some(StopReason::LeaseExpired),
            "and did not renew"
        );
    }

    #[test]
    fn a_new_ros_session_starts_fresh_marks() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &at(&cmd, 100), &cdr(1.0));
        d.tick(1_000); // lease expiry
        assert!(d.tick(6_000).released);
        let fx = d.on_ros_cmd_vel(6_100, &at(&cmd, 50), &cdr(0.5));
        assert_eq!(fx.claimed, Some(Source::Ros));
        assert_eq!(d.wheels(6_100), [0.25; 4], "50 is fine in the new session");
    }

    #[test]
    fn a_repeated_hello_keeps_the_session_and_its_deadlines() {
        let mut d = dev();
        let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
        let (h1, caps1) = r.unwrap();
        let (r, _) = send(&mut d, 5_000, A, Kind::Hello, 0, 2, &[]);
        let (h2, caps2) = r.unwrap();
        assert_eq!((h2.kind, h2.session), (Kind::HelloAck, h1.session));
        assert_eq!(caps1, caps2);
        assert!(
            d.tick(10_000).released,
            "the claim window still ends at 10 s"
        );
    }

    #[test]
    fn a_callback_from_a_closed_connection_cannot_act_for_a_new_one_with_the_same_handle() {
        let mut d = dev();
        let old = d.on_ble_connect(0, 0).unwrap().0;
        d.on_ble_disconnect(1, old);
        let new = d.on_ble_connect(2, 0).unwrap().0;
        assert_eq!(old.handle, new.handle, "the stack reused the handle");
        assert_eq!(
            d.on_ble_setpoint(3, old, &ble_sp(1, 1.0)),
            Effects::default()
        );
        assert_eq!(d.on_ble_control(4, old, 1), Err(Error::NotOwner));
        assert_eq!(d.on_ble_disconnect(5, old), Effects::default());
        assert!(d.ble_owns(new) && !d.ble_owns(old));
        d.on_ble_setpoint(6, new, &ble_sp(1, 1.0));
        assert_eq!(d.on_ble_control(7, new, 1), Ok(Effects::default()));
        assert_eq!(d.wheels(7), [0.5; 4]);
    }

    #[test]
    fn a_router_disconnect_releases_a_ros_owner() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(1.0));
        let fx = d.on_ros_disconnect(10);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert!(fx.released);
        assert_eq!(d.arbiter().state(), State::Open);
        assert_eq!(d.wheels(10), [0.0; 4]);
    }

    #[test]
    fn a_router_disconnect_leaves_other_owners_alone() {
        let mut d = dev();
        armed(&mut d); // UDP
        assert_eq!(d.on_ros_disconnect(10), Effects::default());
        assert_eq!(d.arbiter().state(), State::Armed);
    }

    #[test]
    fn ros_stop_at_the_lease_deadline_requires_explicit_rearm() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let arm = publisher(&mut d, 1, 2, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        let session = d.arbiter().session();
        d.on_ros_arm(1_000, &arm, &bool_cdr(false));
        assert_eq!(d.arbiter().state(), State::Stopped(StopReason::Stop));
        d.on_ros_arm(1_001, &at(&arm, 2), &bool_cdr(true));
        assert_eq!(d.wheels(1_001), [0.0; 4], "a new setpoint is required");
        d.on_ros_cmd_vel(1_002, &at(&cmd, 2), &cdr(0.5));
        assert_eq!(d.wheels(1_002), [0.0; 4], "the stream cannot clear STOP");
        d.on_ros_arm(1_003, &at(&arm, 3), &bool_cdr(true));
        assert_eq!(d.wheels(1_003), [0.25; 4]);
        assert_eq!(d.arbiter().session(), session);
    }

    #[test]
    fn ros_out_of_limits_after_lease_expiry_requires_explicit_rearm() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let arm = publisher(&mut d, 1, 2, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        d.tick(1_000);
        assert_eq!(
            d.on_ros_cmd_vel(1_001, &at(&cmd, 2), &cdr(9.0)).stop,
            Some(StopReason::OutOfLimits)
        );
        d.on_ros_cmd_vel(1_002, &at(&cmd, 3), &cdr(0.5));
        assert_eq!(d.arbiter().state(), State::Stopped(StopReason::OutOfLimits));
        assert_eq!(d.wheels(1_002), [0.0; 4]);
        d.on_ros_arm(1_003, &arm, &bool_cdr(true));
        assert_eq!(d.wheels(1_003), [0.25; 4]);
    }

    #[test]
    fn ble_setpoints_need_explicit_arm_after_claim_and_lease_expiry() {
        let mut d = dev();
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        d.on_ble_setpoint(1, conn, &ble_sp(1, 1.0));
        assert_eq!(d.wheels(1), [0.0; 4]);
        d.on_ble_control(2, conn, 1).unwrap();
        assert_eq!(d.wheels(2), [0.5; 4]);
        assert_eq!(d.tick(1_001).stop, Some(StopReason::LeaseExpired));
        let session = d.arbiter().session();
        d.on_ble_setpoint(1_002, conn, &ble_sp(2, 1.0));
        assert_eq!(d.wheels(1_002), [0.0; 4]);
        d.on_ble_control(1_003, conn, 1).unwrap();
        assert_eq!(d.wheels(1_003), [0.5; 4]);
        assert_eq!(d.arbiter().session(), session);
    }

    #[test]
    fn udp_setpoints_need_explicit_arm_after_lease_expiry() {
        let mut d = dev();
        let session = armed(&mut d);
        d.tick(1_000);
        send(&mut d, 1_001, A, Kind::Setpoint, session, 2, &cdr(0.5));
        assert_eq!(d.wheels(1_001), [0.0; 4]);
        let (r, _) = send(&mut d, 1_002, A, Kind::Arm, session, 3, &[]);
        assert_eq!(r.unwrap().1[..3], [2, 0, 0]);
        assert_eq!(d.wheels(1_002), [0.25; 4]);
        assert_eq!(d.arbiter().session(), session);
    }

    #[test]
    fn ble_stop_and_out_of_limits_need_a_new_setpoint_then_arm() {
        for reason in [StopReason::Stop, StopReason::OutOfLimits] {
            let mut d = dev();
            let conn = d.on_ble_connect(0, 1).unwrap().0;
            d.on_ble_setpoint(1, conn, &ble_sp(1, 1.0));
            d.on_ble_control(2, conn, 1).unwrap();
            if reason == StopReason::Stop {
                d.on_ble_control(3, conn, 2).unwrap();
            } else {
                d.on_ble_setpoint(3, conn, &ble_sp(2, 9.0));
            }
            assert_eq!(d.on_ble_control(4, conn, 1), Err(Error::NotReady));
            d.on_ble_setpoint(5, conn, &ble_sp(3, 0.5));
            assert_eq!(d.arbiter().state(), State::Stopped(reason));
            assert_eq!(d.wheels(5), [0.0; 4]);
            d.on_ble_control(6, conn, 1).unwrap();
            assert_eq!(d.wheels(6), [0.25; 4]);
        }
    }

    #[test]
    fn identical_setpoints_give_identical_wheels_on_every_transport() {
        let mut payload = [0; 68];
        Setpoint {
            sec: 0,
            nanosec: 0,
            vx: 0.4,
            vy: 0.2,
            wz: 0.5,
        }
        .encode(&mut payload)
        .unwrap();
        let mut udp = dev();
        let (r, _) = send(&mut udp, 0, A, Kind::Hello, 0, 1, &[]);
        let session = r.unwrap().0.session;
        send(&mut udp, 1, A, Kind::Setpoint, session, 1, &payload);
        send(&mut udp, 2, A, Kind::Arm, session, 2, &[]);

        let mut ble = dev();
        let conn = ble.on_ble_connect(0, 1).unwrap().0;
        let mut sample = [0; 72];
        sample[..4].copy_from_slice(&1u32.to_le_bytes());
        sample[4..].copy_from_slice(&payload);
        ble.on_ble_setpoint(1, conn, &sample);
        ble.on_ble_control(2, conn, 1).unwrap();

        let mut ros = dev();
        let cmd = publisher(&mut ros, 1, 1, 1);
        ros.on_ros_cmd_vel(1, &cmd, &payload);
        assert_ne!(udp.wheels(2), [0.0; 4]);
        assert_eq!(ble.wheels(2), udp.wheels(2));
        assert_eq!(ros.wheels(2), udp.wheels(2));
    }

    #[test]
    fn shared_wifi_intruders_cannot_renew_the_owners_lease() {
        let mut udp = dev();
        let session = armed(&mut udp);
        let cmd = publisher(&mut udp, 1, 1, 1);
        let arm = publisher(&mut udp, 1, 2, 1);
        assert_eq!(udp.on_ros_cmd_vel(999, &cmd, &cdr(0.5)), Effects::default());
        assert_eq!(
            udp.on_ros_arm(999, &arm, &bool_cdr(false)),
            Effects::default()
        );
        assert!(udp.wifi_allowed() && !udp.radio_allowed(Source::Ble));
        assert_eq!(udp.arbiter().session(), session);
        assert_eq!(udp.tick(1_000).stop, Some(StopReason::LeaseExpired));

        let mut ros = dev();
        let cmd = publisher(&mut ros, 1, 1, 1);
        ros.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        let session = ros.arbiter().session();
        // A UDP HELLO now takes over from ROS (ADR-0016, tested separately); input without a
        // claim still cannot touch the ROS owner's session.
        let setpoint = cdr(0.5);
        for kind in [Kind::Setpoint, Kind::Arm, Kind::Stop, Kind::Bye] {
            let payload = if kind == Kind::Setpoint {
                &setpoint[..]
            } else {
                &[]
            };
            let (r, fx) = send(&mut ros, 999, A, kind, session, 2, payload);
            assert!(r.is_none());
            assert_eq!(fx, Effects::default());
        }
        assert!(ros.wifi_allowed() && !ros.radio_allowed(Source::Ble));
        assert_eq!(ros.arbiter().session(), session);
        assert_eq!(ros.tick(1_000).stop, Some(StopReason::LeaseExpired));
    }

    #[test]
    fn ble_echo_preserves_state_error_and_setpoint_sequence() {
        let mut d = dev();
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(d.on_ble_control(1, conn, 1), Err(Error::NotReady));
        d.on_ble_setpoint(2, conn, &ble_sp(1, 0.5));
        let before = (
            d.ble_status(),
            d.arbiter().session(),
            d.arbiter().owner(),
            d.wheels(2),
        );
        for data in [&[][..], &[2; ECHO_MAX][..], &[2; ECHO_MAX + 1][..]] {
            assert_eq!(echo(data), (data.len() <= ECHO_MAX).then_some(data));
            assert_eq!(
                (
                    d.ble_status(),
                    d.arbiter().session(),
                    d.arbiter().owner(),
                    d.wheels(2)
                ),
                before
            );
        }
        // Echo payloads cannot consume this next setpoint's sequence or clear NotReady.
        d.on_ble_setpoint(3, conn, &ble_sp(2, 1.0));
        assert_eq!(d.ble_status()[2], before.0[2]);
        d.on_ble_control(4, conn, 1).unwrap();
        assert_eq!(d.wheels(4), [0.5; 4]);
        assert_eq!(
            echo(b"still only a probe"),
            Some(&b"still only a probe"[..])
        );
        assert_eq!(d.tick(1_003).stop, Some(StopReason::LeaseExpired));
    }

    #[test]
    fn boot_and_claim_expiry_leave_zero_wheels_and_reopen_radios() {
        let mut d = dev();
        assert_eq!(d.status(), [0, 0, 0]);
        assert!(d.arbiter().owner().is_none());
        assert_eq!(d.wheels(0), [0.0; 4]);
        assert!(d.wifi_allowed() && d.radio_allowed(Source::Ble));
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        assert!(!d.wifi_allowed());
        assert!(!d.tick(9_999).released);
        assert!(d.tick(10_000).released);
        assert_eq!(d.wheels(10_000), [0.0; 4]);
        assert!(d.wifi_allowed() && d.radio_allowed(Source::Ble));
        assert!(!d.ble_owns(conn));
    }

    #[test]
    fn capabilities_are_the_same_before_claim_and_on_every_transport() {
        let mut d = dev();
        let snapshot = d.capabilities_snapshot();
        let binary = snapshot.encode();
        let mut out = [0; rovi_protocol::capabilities::CDR_MAX];
        assert!(snapshot.encode_cdr(&mut out).is_ok());
        assert_eq!(d.arbiter().state(), State::Open);
        assert_eq!(d.wheels(0), [0.0; 4]);
        assert!(d.wifi_allowed() && d.radio_allowed(Source::Ble));
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(d.capabilities(), binary, "BLE characteristic");
        d.on_ble_disconnect(1, conn);
        let (r, _) = send(&mut d, 2, A, Kind::Hello, 0, 1, &[]);
        let (header, payload) = r.unwrap();
        assert_eq!(payload[..CAPABILITIES_LEN], binary, "UDP HELLO_ACK");
        send(&mut d, 3, A, Kind::Bye, header.session, 2, &[]);
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(4, &cmd, &cdr(0.0));
        assert_eq!(d.capabilities_snapshot(), snapshot, "ROS typed values");
    }

    #[test]
    fn a_limit_read_from_ros_capabilities_is_accepted_without_rounding_up() {
        let mut d = Device::new(
            Config {
                max_vx: 0.7,
                ..Config::DEFAULT
            },
            GEOMETRY,
            0,
        )
        .unwrap();
        let mut out = [0; rovi_protocol::capabilities::CDR_MAX];
        let n = d.capabilities_snapshot().encode_cdr(&mut out).unwrap();
        let text = core::str::from_utf8(&out[8..n - 1]).unwrap();
        let limit = text
            .split(' ')
            .find_map(|t| t.strip_prefix("max_vx="))
            .unwrap()
            .parse::<f64>()
            .unwrap();
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(limit));
        assert_eq!(d.arbiter().state(), State::Armed);
        assert_eq!(
            d.on_ros_cmd_vel(1, &at(&cmd, 2), &cdr(limit + f64::EPSILON))
                .stop,
            Some(StopReason::OutOfLimits)
        );
    }

    #[test]
    fn udp_owner_error_persists_through_setpoints_echo_and_intruders() {
        let mut d = dev();
        let s = armed(&mut d);
        send(&mut d, 1, A, Kind::Stop, s, 3, &[]);
        let (r, _) = send(&mut d, 2, A, Kind::Arm, s, 4, &[]);
        assert_eq!(r.unwrap().1[..3], [3, 1, 3]);
        assert_eq!(
            d.ble_status(),
            [3, 1, 3],
            "same persistent snapshot on all sources"
        );
        send(&mut d, 3, A, Kind::Setpoint, s, 2, &cdr(0.5));
        send(&mut d, 4, A, Kind::Echo, s, 99, b"probe");
        send(&mut d, 5, B, Kind::Stop, s, 5, &[]);
        send(&mut d, 6, A, Kind::Stop, s.wrapping_add(1), 5, &[]);
        send(&mut d, 7, B, Kind::Hello, 0, 6, &[]);
        assert_eq!(d.ble_status(), [3, 1, 3]);
        send(&mut d, 8, A, Kind::Arm, s, 5, &[]);
        assert_eq!(d.ble_status(), [2, 0, 0]);
    }

    #[test]
    fn udp_control_payloads_must_be_empty_and_report_invalid_without_acting() {
        for kind in [Kind::Arm, Kind::Stop] {
            let mut d = dev();
            let (r, _) = send(&mut d, 0, A, Kind::Hello, 0, 1, &[]);
            let s = r.unwrap().0.session;
            send(&mut d, 1, A, Kind::Setpoint, s, 1, &cdr(0.5));
            if kind == Kind::Stop {
                send(&mut d, 2, A, Kind::Arm, s, 2, &[]);
            }
            let before = d.arbiter().state();
            let (r, _) = send(&mut d, 3, A, kind, s, 3, &[1]);
            assert_eq!(r.unwrap().1[2], 4, "identifiable owner's malformed control");
            assert_eq!(d.arbiter().state(), before);
            assert_eq!(d.ble_status()[2], 4);
            send(&mut d, 4, A, kind, s, 3, &[]);
            assert_eq!(d.ble_status()[2], 0);
        }
    }

    #[test]
    fn ros_owner_error_persists_until_a_successful_fresh_control() {
        let mut d = dev();
        let cmd = publisher(&mut d, 1, 1, 1);
        let arm = publisher(&mut d, 1, 2, 1);
        let intruder = publisher(&mut d, 2, 1, 1);
        d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
        d.on_ros_arm(1, &arm, &bool_cdr(false));
        d.on_ros_arm(2, &at(&arm, 2), &bool_cdr(true));
        assert_eq!(d.ble_status(), [3, 1, 3]);
        d.on_ros_cmd_vel(3, &at(&cmd, 2), &cdr(0.5));
        assert_eq!(echo(b"probe"), Some(&b"probe"[..]));
        d.on_ros_arm(4, &intruder, &bool_cdr(false));
        d.on_ros_arm(5, &arm, &bool_cdr(false)); // replay cannot clear the error
        assert_eq!(d.ble_status(), [3, 1, 3]);
        d.on_ros_arm(6, &at(&arm, 3), &bool_cdr(true));
        assert_eq!(d.ble_status(), [2, 0, 0]);
    }

    #[test]
    fn malformed_ros_bool_reports_invalid_without_consuming_the_arm_sequence() {
        for bad in [
            &[0, 1, 0, 0, 2][..],
            &[0, 1, 0, 0, 255],
            &[0, 1, 0, 0],
            &[0, 0, 0, 0, 1],
            &[0, 1, 0, 0, 1, 99],
        ] {
            let mut d = dev();
            let cmd = publisher(&mut d, 1, 1, 1);
            let arm = publisher(&mut d, 1, 2, 1);
            d.on_ros_cmd_vel(0, &cmd, &cdr(0.5));
            d.on_ros_arm(1, &arm, bad);
            assert_eq!(d.ble_status(), [2, 0, 4], "{bad:?}");
            d.on_ros_arm(2, &arm, &bool_cdr(false));
            assert_eq!(
                d.ble_status(),
                [3, 1, 0],
                "invalid did not consume the sequence"
            );
            d.on_ros_arm(3, &arm, bad);
            assert_eq!(
                d.ble_status(),
                [3, 1, 0],
                "stale invalid cannot change the error"
            );
        }
    }

    #[test]
    fn release_and_new_transport_claim_cannot_retain_a_previous_owners_error() {
        let mut d = dev();
        let conn = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(d.on_ble_control(1, conn, 99), Err(Error::Invalid));
        d.on_ble_disconnect(2, conn);
        assert_eq!(d.ble_status(), [0, 0, 0]);
        let (r, _) = send(&mut d, 3, A, Kind::Hello, 0, 1, &[]);
        let s = r.unwrap().0.session;
        assert_eq!(d.ble_status(), [1, 0, 0]);
        send(&mut d, 4, A, Kind::Arm, s, 2, &[]);
        assert_eq!(d.ble_status(), [1, 0, 3]);
        assert!(d.tick(10_003).released);
        assert_eq!(d.ble_status(), [0, 0, 0]);
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(10_004, &cmd, &cdr(0.0));
        assert_eq!(d.ble_status(), [2, 0, 0]);
    }

    #[test]
    fn udp_lease_stop_notifies_the_owner_once_in_the_current_session() {
        let mut d = dev();
        let s = armed(&mut d);
        let mut out = [0; 32];
        assert_eq!(d.udp_status_notification(&mut out), None);
        d.tick(1_000);
        assert_eq!(
            d.udp_status_notification(&mut out[..14]),
            None,
            "retry when buffer fits"
        );
        let (peer, n) = d
            .udp_status_notification(&mut out)
            .expect("lease stop notification");
        let (h, bytes) = udp::parse(&out[..n]).unwrap();
        assert_eq!((peer, h.kind, h.session, h.seq), (A, Kind::Status, s, 0));
        assert_eq!(bytes, [3, 2, 0]);
        assert_eq!(d.udp_status_notification(&mut out), None);
    }

    #[test]
    fn direct_udp_control_replies_do_not_duplicate_stop_notifications() {
        let mut d = dev();
        let s = armed(&mut d);
        let mut out = [0; 32];
        send(&mut d, 1, A, Kind::Stop, s, 3, &[]);
        assert_eq!(d.udp_status_notification(&mut out), None);
        send(&mut d, 2, A, Kind::Arm, s, 4, &[]);
        assert_eq!(d.udp_status_notification(&mut out), None);
        send(&mut d, 3, A, Kind::Setpoint, s, 2, &cdr(0.5));
        send(&mut d, 4, A, Kind::Arm, s, 5, &[]);
        d.on_motor_fault(5);
        let (_, n) = d
            .udp_status_notification(&mut out)
            .expect("motor fault notification");
        assert_eq!(udp::parse(&out[..n]).unwrap().1, [3, 4, 0]);
    }

    #[test]
    fn udp_notifications_never_leak_a_released_owners_session() {
        let mut d = dev();
        let s = armed(&mut d);
        d.tick(1_000);
        send(&mut d, 1_001, A, Kind::Bye, s, 3, &[]);
        let mut out = [0; 32];
        assert_eq!(d.udp_status_notification(&mut out), None);
        let (r, _) = send(&mut d, 1_002, B, Kind::Hello, 0, 1, &[]);
        let new = r.unwrap().0.session;
        assert_eq!(d.udp_status_notification(&mut out), None);
        d.on_motor_fault(1_003);
        let (peer, n) = d.udp_status_notification(&mut out).unwrap();
        assert_eq!(peer, B);
        assert_eq!(udp::parse(&out[..n]).unwrap().0.session, new);
    }

    #[test]
    fn ble_reads_follow_claim_control_deadline_and_release_immediately() {
        let mut d = dev();
        let c = d.on_ble_connect(0, 1).unwrap().0;
        assert_eq!(d.ble_status(), [1, 0, 0]);
        assert_eq!(d.on_ble_control(1, c, 1), Err(Error::NotReady));
        assert_eq!(d.ble_status(), [1, 0, 3]);
        assert_eq!(d.on_ble_control(2, c, 99), Err(Error::Invalid));
        d.on_ble_setpoint(3, c, &ble_sp(1, 0.5));
        assert_eq!(d.ble_status(), [1, 0, 4]);
        d.on_ble_control(4, c, 1).unwrap();
        assert_eq!(d.ble_status(), [2, 0, 0]);
        d.tick(1_003);
        assert_eq!(d.ble_status(), [3, 2, 0]);
        d.on_ble_control(1_004, c, 1).unwrap_err();
        assert_eq!(d.ble_status(), [3, 2, 3]);
        d.on_ble_disconnect(1_005, c);
        assert_eq!(d.ble_status(), [0, 0, 0]);
        assert_eq!(d.status_snapshot().session, 0);
        let mut out = [0; rovi_protocol::status::CDR_MAX];
        let n = d.status_cdr(&mut out);
        assert_eq!(
            &out[8..n - 1],
            b"state=open reason=none error=none session=0"
        );
    }

    #[test]
    fn ble_echo_accepts_an_empty_notification() {
        assert_eq!(ble_echo(0, &[], &mut []), Some(0));
    }

    #[test]
    fn ble_echo_copies_short_and_maximum_notifications() {
        let mut out = [0xaa; ECHO_MAX];
        assert_eq!(ble_echo(0, b"abc", &mut out), Some(3));
        assert_eq!(&out[..3], b"abc");
        assert!(out[3..].iter().all(|b| *b == 0xaa));
        assert_eq!(ble_echo(0, &[0x55; ECHO_MAX], &mut out), Some(ECHO_MAX));
        assert_eq!(out, [0x55; ECHO_MAX]);
    }

    #[test]
    fn ble_echo_rejects_bad_offset_oversize_and_short_buffer_without_writes() {
        let mut out = [0xaa; ECHO_MAX];
        assert_eq!(ble_echo(1, b"abc", &mut out), None);
        assert_eq!(ble_echo(0, &[1; ECHO_MAX + 1], &mut out), None);
        assert_eq!(ble_echo(0, b"abc", &mut out[..2]), None);
        assert_eq!(out, [0xaa; ECHO_MAX]);
    }

    // --- Takeover priority and hand-back (network-mode, ADR-0016) ---

    use crate::network::{Mode, Step};

    fn home_dev() -> Device {
        let mut d = dev();
        d.start_network(0, true);
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            d.on_network_step(0, step, true);
        }
        assert_eq!(d.network().mode(), Mode::Home);
        d
    }

    /// A ROS node streaming 1 m/s, armed by its own stream. Returns its cmd and arm attachments.
    fn ros_armed(d: &mut Device, now: u64) -> ([u8; 33], [u8; 33]) {
        let (cmd, arm) = (publisher(d, 1, 1, 1), publisher(d, 1, 2, 1));
        d.on_ros_cmd_vel(now, &cmd, &cdr(1.0));
        assert_eq!(d.arbiter().state(), State::Armed);
        (cmd, arm)
    }

    fn hello(d: &mut Device, now: u64, peer: u128) -> (Kind, u32, Effects) {
        let (r, fx) = send(d, now, peer, Kind::Hello, 0, 1, &[]);
        let h = r.unwrap().0;
        (h.kind, h.session, fx)
    }

    #[test]
    fn udp_preempts_an_armed_ros_owner_and_starts_disarmed() {
        let mut d = dev();
        let (cmd, _) = ros_armed(&mut d, 0);
        let ros_owner = d.arbiter().owner().unwrap();
        let (kind, _, fx) = hello(&mut d, 100, A);
        assert_eq!(kind, Kind::HelloAck);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert!(fx.released);
        assert_eq!(fx.claimed, Some(Source::Udp));
        assert_eq!(fx.preempted, Some(ros_owner));
        assert_eq!(d.arbiter().state(), State::Claimed);
        assert_eq!(d.wheels(100), [0.0; 4]);
        d.on_ros_cmd_vel(150, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(
            d.arbiter().owner().map(|o| o.source),
            Some(Source::Udp),
            "ROS ignored"
        );
        assert_eq!(d.wheels(150), [0.0; 4]);
    }

    #[test]
    fn udp_does_not_preempt_udp_and_ros_does_not_preempt_udp() {
        let mut d = dev();
        armed(&mut d);
        assert_eq!(hello(&mut d, 10, B).0, Kind::Busy);
        let cmd = publisher(&mut d, 1, 1, 1);
        d.on_ros_cmd_vel(20, &cmd, &cdr(1.0));
        assert_eq!(d.arbiter().owner().map(|o| o.id), Some(A));
    }

    #[test]
    fn ble_preempts_a_wifi_owner_only_in_home_mode() {
        let mut fallback = dev();
        armed(&mut fallback);
        assert_eq!(fallback.on_ble_connect(10, 1).err(), Some(Error::Busy));

        let mut d = home_dev();
        armed(&mut d);
        let (_, fx) = d.on_ble_connect(10, 1).unwrap();
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert_eq!(fx.claimed, Some(Source::Ble));
        assert_eq!(fx.preempted.map(|o| o.source), Some(Source::Udp));
        assert_eq!(
            d.arbiter().state(),
            State::Claimed,
            "the BLE owner starts disarmed"
        );
        assert!(!d.wifi_allowed(), "the station goes off");
    }

    #[test]
    fn ble_advertises_while_wifi_owns_only_in_home_mode() {
        let mut fallback = dev();
        ros_armed(&mut fallback, 0);
        assert!(!fallback.radio_allowed(Source::Ble));
        let mut d = home_dev();
        ros_armed(&mut d, 0);
        assert!(d.radio_allowed(Source::Ble));
    }

    #[test]
    fn takeover_without_home_latches_ros_until_an_explicit_arm() {
        let mut d = dev();
        let (cmd, arm) = ros_armed(&mut d, 0);
        let (_, s, _) = hello(&mut d, 100, A);
        send(&mut d, 200, A, Kind::Bye, s, 2, &[]);
        d.on_ros_cmd_vel(300, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.arbiter().owner().map(|o| o.source), Some(Source::Ros));
        assert_eq!(d.wheels(300), [0.0; 4], "no auto-arm after a takeover");
        d.on_ros_arm(310, &at(&arm, 2), &bool_cdr(true));
        assert_eq!(d.wheels(310), [0.5; 4], "an explicit arm resumes");
        d.tick(1_310);
        d.on_ros_cmd_vel(1_400, &at(&cmd, 3), &cdr(1.0));
        assert_eq!(d.wheels(1_400), [0.5; 4], "latch cleared: auto-arm is back");
    }

    #[test]
    fn takeover_ending_with_home_hands_back_with_auto_arm() {
        let mut d = home_dev();
        let (cmd, _) = ros_armed(&mut d, 0);
        let (conn, _) = d.on_ble_connect(100, 1).unwrap();
        d.network.command(Command::Home);
        d.on_ble_disconnect(200, conn);
        assert_eq!(d.network().mode(), Mode::Attempting(Step::Scan));
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            d.on_network_step(300, step, true);
        }
        d.on_ros_cmd_vel(400, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.wheels(400), [0.5; 4]);
    }

    #[test]
    fn accidental_ble_loss_at_home_reattempts_with_ros_latched() {
        let mut d = home_dev();
        let (cmd, _) = ros_armed(&mut d, 0);
        let (conn, _) = d.on_ble_connect(100, 1).unwrap();
        let fx = d.on_ble_disconnect(200, conn);
        assert!(fx.released);
        assert_eq!(d.network().mode(), Mode::Attempting(Step::Scan));
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            d.on_network_step(300, step, true);
        }
        d.on_ros_cmd_vel(400, &at(&cmd, 2), &cdr(1.0));
        assert_eq!(d.wheels(400), [0.0; 4]);
    }

    #[test]
    fn preemption_discards_the_old_owners_pending_command() {
        let mut d = home_dev();
        ros_armed(&mut d, 0);
        d.network.command(Command::CarNetwork);
        let (_, s, _) = hello(&mut d, 100, A);
        assert_eq!(d.network().pending(), None);
        send(&mut d, 200, A, Kind::Bye, s, 2, &[]);
        assert_eq!(d.network().mode(), Mode::Home);
    }

    #[test]
    fn ble_claim_window_expiry_at_home_reattempts() {
        let mut d = home_dev();
        d.on_ble_connect(0, 1).unwrap();
        assert!(d.tick(Config::DEFAULT.claim_window_ms).released);
        assert_eq!(d.network().mode(), Mode::Attempting(Step::Scan));
    }

    #[test]
    fn wifi_owner_ending_with_home_at_home_stays_home() {
        let mut d = home_dev();
        let (_, s, _) = hello(&mut d, 0, A);
        d.network.command(Command::Home);
        send(&mut d, 10, A, Kind::Bye, s, 2, &[]);
        assert_eq!(d.network().mode(), Mode::Home);
    }

    #[test]
    fn network_budget_runs_on_the_device_tick() {
        let mut d = dev();
        d.start_network(0, true);
        d.tick(crate::network::SCAN_BUDGET_MS);
        assert_eq!(d.network().mode(), Mode::Fallback);
    }

    // --- Mode command and network information (task 2.4) ---

    fn uint8(v: u8) -> [u8; 5] {
        [0, 1, 0, 0, v]
    }

    #[test]
    fn ble_control_codes_3_and_4_set_a_pending_mode_command() {
        let mut d = home_dev();
        let (conn, _) = d.on_ble_connect(0, 1).unwrap();
        d.on_ble_control(10, conn, 4).unwrap();
        assert_eq!(d.network().pending(), Some(Command::CarNetwork));
        d.on_ble_control(20, conn, 3).unwrap();
        assert_eq!(d.network().pending(), Some(Command::Home));
        assert_eq!(
            d.take_mode_log(),
            Some((Source::Ble, Command::Home, None)),
            "the last command is logged"
        );
    }

    #[test]
    fn udp_mode_kind_sets_pending_and_replies_with_status() {
        let mut d = dev();
        let (_, s, _) = hello(&mut d, 0, A);
        let (r, _) = send(&mut d, 10, A, Kind::Mode, s, 2, &[2]);
        let (h, p) = r.unwrap();
        assert_eq!(h.kind, Kind::Status);
        assert_eq!(p[2], 0, "no error");
        assert_eq!(d.network().pending(), Some(Command::CarNetwork));
        send(&mut d, 20, A, Kind::Bye, s, 3, &[]);
        assert_eq!(
            d.network().reason(),
            rovi_protocol::network::Reason::Requested
        );
    }

    #[test]
    fn ros_mode_from_the_owner_sets_pending_and_others_are_ignored() {
        let mut d = home_dev();
        let (_, arm) = ros_armed(&mut d, 0);
        let other = publisher(&mut d, 2, 1, 1);
        d.on_ros_arm(10, &at(&arm, 2), &bool_cdr(false));
        d.on_ros_mode(20, &other, &uint8(2));
        assert_eq!(d.network().pending(), None, "non-owner ignored");
        let mode = publisher(&mut d, 1, 3, 1);
        d.on_ros_mode(30, &mode, &uint8(2));
        assert_eq!(d.network().pending(), Some(Command::CarNetwork));
    }

    #[test]
    fn armed_owner_mode_command_is_refused_not_ready() {
        let mut d = home_dev();
        let s = armed(&mut d);
        let (r, _) = send(&mut d, 10, A, Kind::Mode, s, 5, &[1]);
        assert_eq!(r.unwrap().1[2], 3, "STATUS carries not-ready");
        assert_eq!(d.arbiter().state(), State::Armed);
        assert_eq!(d.network().pending(), None);
        assert_eq!(
            d.take_mode_log(),
            Some((Source::Udp, Command::Home, Some(Error::NotReady)))
        );
    }

    #[test]
    fn unknown_mode_values_and_lengths_are_invalid_and_keep_the_pending_command() {
        let mut d = home_dev();
        let (_, s, _) = hello(&mut d, 0, A);
        send(&mut d, 5, A, Kind::Mode, s, 2, &[2]);
        for (seq, payload) in [(3, &[0u8][..]), (4, &[3]), (5, &[1, 1]), (6, &[])] {
            let (r, _) = send(&mut d, 10, A, Kind::Mode, s, seq, payload);
            assert_eq!(r.unwrap().1[2], 4, "invalid: {payload:?}");
        }
        assert_eq!(d.network().pending(), Some(Command::CarNetwork));
        let mut b = home_dev();
        let (conn, _) = b.on_ble_connect(0, 1).unwrap();
        assert_eq!(b.on_ble_control(10, conn, 9), Err(Error::Invalid));
    }

    #[test]
    fn home_without_home_support_is_invalid() {
        let mut d = dev();
        let (_, s, _) = hello(&mut d, 0, A);
        let (r, _) = send(&mut d, 10, A, Kind::Mode, s, 2, &[1]);
        assert_eq!(r.unwrap().1[2], 4, "invalid");
        assert_eq!(d.network().pending(), None);
    }

    #[test]
    fn network_information_reports_mode_and_station_ip() {
        let mut d = home_dev();
        d.set_station_ip(Some([192, 168, 1, 50]));
        assert_eq!(d.network_info(), [2, 0, 0, 0, 192, 168, 1, 50]);
        let mut out = [0u8; 96];
        let n = d.network_cdr(&mut out);
        assert!(core::str::from_utf8(&out[8..n - 1])
            .unwrap()
            .ends_with("ip=192.168.1.50"));
        d.set_station_ip(None);
        assert_eq!(d.network_info()[4..], [0; 4]);
    }

    // --- Protocol alignment (task 2.5) ---

    #[test]
    fn attached_ble_central_watches_without_claiming() {
        let mut d = home_dev();
        let s = armed(&mut d);
        let conn = d.on_ble_attach(10, 1);
        assert_eq!(
            d.ble_status(),
            [2, 0, 0],
            "it reads the UDP owner's armed state"
        );
        assert_eq!(
            d.on_ble_disconnect(20, conn),
            Effects::default(),
            "nothing to release"
        );
        assert_eq!(d.arbiter().owner().map(|o| o.id), Some(A));
        assert_eq!(d.arbiter().session(), s);
        assert!(d.wifi_allowed());
    }

    #[test]
    fn first_ble_setpoint_or_control_write_claims() {
        let mut d = dev();
        let conn = d.on_ble_attach(0, 1);
        assert_eq!(d.ble_status()[0], 0, "open before any write");
        let fx = d.on_ble_setpoint(10, conn, &ble_sp(1, 0.0));
        assert_eq!(fx.claimed, Some(Source::Ble));
        assert_eq!(d.ble_status()[0], 1, "claimed after the first write");

        let mut d = dev();
        let conn = d.on_ble_attach(0, 1);
        let fx = d.on_ble_control(10, conn, 2).unwrap();
        assert_eq!(fx.claimed, Some(Source::Ble), "STOP claims too");
    }

    #[test]
    fn second_ble_central_cannot_claim_while_ble_owns() {
        let mut d = dev();
        let owner = d.on_ble_attach(0, 1);
        d.on_ble_setpoint(10, owner, &ble_sp(1, 0.0));
        let other = d.on_ble_attach(20, 2);
        assert_eq!(
            d.on_ble_setpoint(30, other, &ble_sp(1, 0.5)),
            Effects::default()
        );
        assert_eq!(d.on_ble_control(40, other, 1), Err(Error::Busy));
        assert!(d.ble_owns(owner) && !d.ble_owns(other));
        assert_eq!(d.on_ble_disconnect(50, other), Effects::default());
        assert!(d.ble_owns(owner));
    }

    #[test]
    fn attached_ble_write_takes_over_a_wifi_owner_in_home_mode() {
        let mut d = home_dev();
        armed(&mut d);
        let conn = d.on_ble_attach(10, 1);
        let fx = d.on_ble_control(20, conn, 2).unwrap();
        assert_eq!(fx.preempted.map(|o| o.source), Some(Source::Udp));
        assert!(d.ble_owns(conn) && !d.wifi_allowed());
    }

    #[test]
    fn udp_info_answers_any_peer_with_capabilities_status_and_network() {
        let mut d = home_dev();
        d.set_station_ip(Some([10, 0, 0, 7]));
        let s = armed(&mut d);
        let (r, fx) = send(&mut d, 10, B, Kind::Info, 0, 9, &[]);
        let (h, p) = r.unwrap();
        assert_eq!((h.kind, h.session, h.seq), (Kind::Info, 0, 9));
        assert_eq!(p[..CAPABILITIES_LEN], d.capabilities()[..]);
        assert_eq!(
            p[CAPABILITIES_LEN..CAPABILITIES_LEN + STATUS_LEN],
            [2, 0, 0]
        );
        assert_eq!(
            p[CAPABILITIES_LEN + STATUS_LEN..udp::INFO_LEN],
            [2, 0, 0, 0, 10, 0, 0, 7]
        );
        assert_eq!(fx, Effects::default());
        assert_eq!(d.arbiter().session(), s, "observation changes nothing");
        assert_eq!(d.arbiter().state(), State::Armed);
    }

    #[test]
    fn ros_release_from_the_owner_applies_a_pending_mode_at_once() {
        let mut d = dev();
        d.start_network(0, true);
        d.on_network_step(0, Step::Scan, false); // fallback
        let (cmd, arm) = ros_armed(&mut d, 0);
        d.on_ros_arm(5, &at(&arm, 2), &bool_cdr(false));
        let mode = publisher(&mut d, 1, 3, 1);
        d.on_ros_mode(10, &mode, &uint8(1));
        let other = publisher(&mut d, 2, 1, 1);
        assert_eq!(
            d.on_ros_release(15, &other, &[0, 1, 0, 0, 0]),
            Effects::default()
        );
        let release = publisher(&mut d, 1, 4, 1);
        let fx = d.on_ros_release(20, &release, &[0, 1, 0, 0, 0]);
        assert!(fx.released);
        assert_eq!(d.network().mode(), Mode::Attempting(Step::Scan));
        let _ = cmd;
    }

    #[test]
    fn ros_release_while_armed_stops_owner_lost() {
        let mut d = dev();
        ros_armed(&mut d, 0);
        let release = publisher(&mut d, 1, 4, 1);
        let fx = d.on_ros_release(10, &release, &[0, 1, 0, 0, 0]);
        assert_eq!(fx.stop, Some(StopReason::OwnerLost));
        assert_eq!(d.arbiter().state(), State::Open);
    }

    #[test]
    fn takeover_notice_goes_to_the_old_udp_session() {
        let mut d = home_dev();
        let s = armed(&mut d);
        assert!(d.take_takeover_notice(Source::Udp).is_none());
        let conn = d.on_ble_attach(10, 1);
        d.on_ble_control(20, conn, 2).unwrap();
        assert!(d.take_takeover_notice(Source::Ros).is_none(), "not for ROS");
        let notice = d.take_takeover_notice(Source::Udp).unwrap();
        assert_eq!(
            (notice.source, notice.id, notice.session),
            (Source::Udp, A, s)
        );
        let mut out = [0u8; 64];
        let n = notice.udp_frame(&mut out);
        let (h, p) = udp::parse(&out[..n]).unwrap();
        assert_eq!((h.kind, h.session, h.seq), (Kind::Status, s, 0));
        assert_eq!(p, [3, 3, 0], "stopped, owner-lost, no error");
        assert!(d.take_takeover_notice(Source::Udp).is_none(), "given once");
    }

    #[test]
    fn takeover_notice_for_a_ros_owner_is_a_status_sample() {
        let mut d = dev();
        ros_armed(&mut d, 0);
        hello(&mut d, 10, A);
        let notice = d.take_takeover_notice(Source::Ros).unwrap();
        assert_eq!(notice.source, Source::Ros);
        let mut out = [0u8; 96];
        let n = notice.ros_cdr(&mut out);
        let text = core::str::from_utf8(&out[8..n - 1]).unwrap();
        assert!(
            text.starts_with("state=stopped reason=owner-lost"),
            "{text}"
        );
    }

    #[test]
    fn released_ble_owner_cannot_reclaim_but_the_watcher_can() {
        let mut d = dev();
        let owner = d.on_ble_attach(0, 1);
        d.on_ble_control(0, owner, 2).unwrap(); // claims, then the claim window runs out
        let watcher = d.on_ble_attach(10, 2);
        assert!(d.tick(Config::DEFAULT.claim_window_ms).released);
        assert_eq!(d.on_ble_control(10_001, owner, 2), Err(Error::NotOwner));
        let fx = d.on_ble_control(10_002, watcher, 2).unwrap();
        assert_eq!(fx.claimed, Some(Source::Ble));
    }

    #[test]
    fn ble_is_quiet_while_a_home_attempt_runs() {
        // Measured on the board (2026-10-26): advertising during a scan stretched it from
        // 160 ms to 2.7 s through coexistence, past the scan budget.
        let mut d = dev();
        d.start_network(0, true);
        assert!(!d.radio_allowed(Source::Ble), "attempting at boot");
        for step in [Step::Scan, Step::Join, Step::Dhcp, Step::Router] {
            d.on_network_step(0, step, true);
        }
        assert!(d.radio_allowed(Source::Ble), "home");
        let conn = d.on_ble_attach(10, 1);
        d.on_ble_control(20, conn, 2).unwrap();
        d.on_ble_disconnect(30, conn);
        assert_eq!(d.network().mode(), Mode::Attempting(Step::Scan));
        assert!(!d.radio_allowed(Source::Ble), "rejoining after BLE");
    }
}
