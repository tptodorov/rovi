//! Golden protocol vectors (add-rovi-protocol task 2.1): every car message built by today's
//! encoders, plus malformed inputs, frozen in `protocol-vectors.json` before the wire code moves.
//! The Python and phone codecs are checked against the same file. Regenerate deliberately with
//! `ROVI_BLESS_VECTORS=1 cargo test -p rovi-core --test protocol_vectors`; any byte change needs
//! a recorded compatibility decision.

use rovi_core::arbiter::{Error, State, StopReason};
use rovi_core::capabilities::Capabilities;
use rovi_core::network::{Command, Info, Mode, Reason, Step};
use rovi_core::setpoint::{DecodeError, Setpoint};
use rovi_core::status::Status;
use rovi_core::udp::{self, Header, Kind};
use std::fmt::Write as _;
use std::path::PathBuf;

const PATH: &str = "tests/protocol-vectors.json";

/// ROS spellings (design D7) for every name in the file.
fn state(s: State) -> (&'static str, &'static str) {
    match s {
        State::Open => ("open", "none"),
        State::Claimed => ("claimed", "none"),
        State::Armed => ("armed", "none"),
        State::Stopped(r) => (
            "stopped",
            match r {
                StopReason::Stop => "stop",
                StopReason::LeaseExpired => "lease-expired",
                StopReason::OwnerLost => "owner-lost",
                StopReason::OutOfLimits => "out-of-limits",
            },
        ),
    }
}

fn error(e: Option<Error>) -> &'static str {
    match e {
        None => "none",
        Some(Error::Busy) => "busy",
        Some(Error::NotOwner) => "not-owner",
        Some(Error::NotReady) => "not-ready",
        Some(Error::Invalid) => "invalid",
    }
}

fn mode(m: Mode) -> &'static str {
    match m {
        Mode::Fallback => "fallback",
        Mode::Attempting(_) => "attempting",
        Mode::Home => "home",
    }
}

fn pending(p: Option<Command>) -> &'static str {
    match p {
        None => "none",
        Some(Command::Home) => "home",
        Some(Command::CarNetwork) => "car-network",
    }
}

const REASONS: [(Reason, &str); 8] = [
    (Reason::None, "none"),
    (Reason::NoSettings, "no-settings"),
    (Reason::Scan, "scan"),
    (Reason::Join, "join"),
    (Reason::Dhcp, "dhcp"),
    (Reason::Router, "router"),
    (Reason::LinkLost, "link-lost"),
    (Reason::Requested, "requested"),
];

fn hex(b: &[u8]) -> String {
    b.iter().fold(String::new(), |mut s, x| {
        write!(s, "{x:02x}").unwrap();
        s
    })
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// One JSON object per line keeps diffs reviewable.
struct Out {
    valid: Vec<String>,
    invalid: Vec<String>,
}

impl Out {
    fn valid(&mut self, message: &str, encoding: &str, name: &str, value: &str, bytes: &[u8]) {
        self.valid.push(format!(
            r#"{{"message":"{message}","encoding":"{encoding}","name":"{name}","value":{value},"hex":"{}"}}"#,
            hex(bytes)
        ));
    }

    /// `error` is the Rust decoder's error where one exists today, else the kind clients must
    /// report once `rovi-protocol` owns the decoder.
    fn invalid(&mut self, message: &str, encoding: &str, name: &str, error: &str, bytes: &[u8]) {
        self.invalid.push(format!(
            r#"{{"message":"{message}","encoding":"{encoding}","name":"{name}","error":"{error}","hex":"{}"}}"#,
            hex(bytes)
        ));
    }
}

const STATES: [State; 7] = [
    State::Open,
    State::Claimed,
    State::Armed,
    State::Stopped(StopReason::Stop),
    State::Stopped(StopReason::LeaseExpired),
    State::Stopped(StopReason::OwnerLost),
    State::Stopped(StopReason::OutOfLimits),
];
const ERRORS: [Option<Error>; 5] = [
    None,
    Some(Error::Busy),
    Some(Error::NotOwner),
    Some(Error::NotReady),
    Some(Error::Invalid),
];

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

fn caps_value(c: &Capabilities) -> String {
    format!(
        r#"{{"version":{},"wheels_radius":{:?},"center_projection_sum":{:?},"max_vx":{:?},"max_vy":{:?},"max_wz":{:?},"lease_ms":{},"claim_ms":{},"reclaim_ms":{}}}"#,
        c.version,
        c.wheels_radius,
        c.center_projection_sum,
        c.max_vx,
        c.max_vy,
        c.max_wz,
        c.lease_ms,
        c.claim_ms,
        c.reclaim_ms
    )
}

fn status_value(s: &Status) -> String {
    let (st, reason) = state(s.state);
    format!(
        r#"{{"state":"{st}","reason":"{reason}","error":"{}","session":{}}}"#,
        error(s.error),
        s.session
    )
}

fn network_value(i: &Info) -> String {
    let reason = REASONS.iter().find(|r| r.0 == i.reason).unwrap().1;
    let [a, b, c, d] = i.ip;
    format!(
        r#"{{"mode":"{}","pending":"{}","reason":"{reason}","ip":"{a}.{b}.{c}.{d}"}}"#,
        mode(i.mode),
        pending(i.pending)
    )
}

fn setpoint_value(s: &Setpoint) -> String {
    format!(
        r#"{{"vx":{:?},"vy":{:?},"wz":{:?},"sec":{},"nanosec":{}}}"#,
        s.vx, s.vy, s.wz, s.sec, s.nanosec
    )
}

fn cdr(s: &Setpoint) -> Vec<u8> {
    let mut b = [0u8; 68];
    let n = s.encode(&mut b).unwrap();
    b[..n].to_vec()
}

fn frame(kind: Kind, session: u32, sequence: u32, payload: &[u8]) -> Vec<u8> {
    let mut b = [0u8; 128];
    let n = udp::write(
        &mut b,
        Header {
            kind,
            session,
            seq: sequence,
        },
        payload,
    )
    .unwrap();
    b[..n].to_vec()
}

fn generate() -> String {
    let mut o = Out {
        valid: vec![],
        invalid: vec![],
    };

    // Status: every state x error, binary (BLE characteristic, UDP STATUS) and ROS text.
    for st in STATES {
        for e in ERRORS {
            let s = Status {
                state: st,
                error: e,
                session: if st == State::Open { 0 } else { 42 },
            };
            let (a, b) = state(st);
            let name = format!("{a}-{b}-{}", error(e));
            let mut ros = [0u8; 80];
            let n = s.encode_cdr(&mut ros).unwrap();
            // Binary Status carries no session (design T6).
            let bin_value = status_value(&Status { session: 0, ..s });
            o.valid("status", "binary", &name, &bin_value, &s.encode());
            o.valid("status", "ros", &name, &status_value(&s), &ros[..n]);
        }
    }
    for (name, b) in [
        ("short", &[0u8, 0][..]),
        ("long", &[0, 0, 0, 0]),
        ("unknown-state", &[4, 0, 0]),
        ("unknown-reason", &[3, 5, 0]),
        ("unknown-error", &[0, 0, 5]),
        ("stopped-without-reason", &[3, 0, 0]),
        ("reason-while-not-stopped", &[2, 1, 0]),
    ] {
        o.invalid("status", "binary", name, "invalid", b);
    }

    // Capabilities.
    let mut ros = [0u8; 384];
    let n = CAPS.encode_cdr(&mut ros).unwrap();
    o.valid(
        "capabilities",
        "binary",
        "default",
        &caps_value(&CAPS),
        &CAPS.encode(),
    );
    o.valid(
        "capabilities",
        "ros",
        "default",
        &caps_value(&CAPS),
        &ros[..n],
    );
    let zero_limits = Capabilities {
        max_vx: 0.0,
        max_vy: 0.0,
        max_wz: 0.0,
        ..CAPS
    };
    o.valid(
        "capabilities",
        "binary",
        "zero-limits",
        &caps_value(&zero_limits),
        &zero_limits.encode(),
    );
    let bad = |f: &dyn Fn(&mut Capabilities)| {
        let mut c = CAPS;
        f(&mut c);
        c.encode().to_vec()
    };
    let caps = CAPS.encode();
    o.invalid("capabilities", "binary", "short", "invalid", &caps[..26]);
    o.invalid(
        "capabilities",
        "binary",
        "long",
        "invalid",
        &[&caps[..], &[0]].concat(),
    );
    o.invalid(
        "capabilities",
        "binary",
        "version-2",
        "unsupported-version",
        &bad(&|c| c.version = 2),
    );
    o.invalid(
        "capabilities",
        "binary",
        "nan-radius",
        "invalid",
        &bad(&|c| c.wheels_radius = f32::NAN),
    );
    o.invalid(
        "capabilities",
        "binary",
        "zero-projection",
        "invalid",
        &bad(&|c| c.center_projection_sum = 0.0),
    );
    o.invalid(
        "capabilities",
        "binary",
        "negative-limit",
        "invalid",
        &bad(&|c| c.max_vy = -1.0),
    );
    o.invalid(
        "capabilities",
        "binary",
        "infinite-limit",
        "invalid",
        &bad(&|c| c.max_wz = f32::INFINITY),
    );
    o.invalid(
        "capabilities",
        "binary",
        "zero-lease",
        "invalid",
        &bad(&|c| c.lease_ms = 0),
    );
    o.invalid(
        "capabilities",
        "binary",
        "zero-claim",
        "invalid",
        &bad(&|c| c.claim_ms = 0),
    );
    o.invalid(
        "capabilities",
        "binary",
        "zero-reclaim",
        "invalid",
        &bad(&|c| c.reclaim_ms = 0),
    );

    // Network information: modes x pending, and every fallback reason.
    let ip = [192, 168, 1, 42];
    for (m, r, addr) in [
        (Mode::Home, Reason::None, ip),
        (Mode::Attempting(Step::Scan), Reason::Requested, [0; 4]),
        (Mode::Fallback, Reason::None, [0; 4]),
    ] {
        for p in [None, Some(Command::Home), Some(Command::CarNetwork)] {
            let i = Info {
                mode: m,
                pending: p,
                reason: r,
                ip: addr,
            };
            let name = format!("{}-{}", mode(m), pending(p));
            let mut ros = [0u8; 128];
            let n = i.encode_cdr(&mut ros).unwrap();
            o.valid("network", "binary", &name, &network_value(&i), &i.encode());
            o.valid("network", "ros", &name, &network_value(&i), &ros[..n]);
        }
    }
    for (r, rname) in REASONS {
        let i = Info {
            mode: Mode::Fallback,
            pending: None,
            reason: r,
            ip: [0; 4],
        };
        o.valid(
            "network",
            "binary",
            &format!("fallback-{rname}"),
            &network_value(&i),
            &i.encode(),
        );
    }
    for (name, b) in [
        ("short", &[2u8, 0, 0, 0, 1, 2, 3][..]),
        ("long", &[2, 0, 0, 0, 1, 2, 3, 4, 5]),
        ("unknown-mode", &[3, 0, 0, 0, 0, 0, 0, 0]),
        ("unknown-pending", &[2, 3, 0, 0, 0, 0, 0, 0]),
        ("unknown-reason", &[0, 0, 8, 0, 0, 0, 0, 0]),
        ("reserved-nonzero", &[2, 0, 0, 1, 0, 0, 0, 0]),
    ] {
        o.invalid("network", "binary", name, "invalid", b);
    }

    // Setpoint: CDR TwistStamped (UDP SETPOINT, ROS /cmd_vel) and BLE seq + CDR.
    let sp = |vx, vy, wz| Setpoint {
        sec: 0,
        nanosec: 0,
        vx,
        vy,
        wz,
    };
    let setpoints = [
        ("zero", sp(0.0, 0.0, 0.0)),
        ("forward", sp(0.1, 0.0, 0.0)),
        ("backward", sp(-0.1, 0.0, 0.0)),
        ("left", sp(0.0, 0.1, 0.0)),
        ("right", sp(0.0, -0.1, 0.0)),
        ("ccw", sp(0.0, 0.0, 0.3)),
        ("cw", sp(0.0, 0.0, -0.3)),
        ("combined", sp(0.5, -0.25, 1.5)),
        (
            "stamped",
            Setpoint {
                sec: 1,
                nanosec: 2,
                ..sp(0.5, -0.25, 1.5)
            },
        ),
    ];
    for (name, s) in &setpoints {
        let b = cdr(s);
        assert_eq!(Setpoint::decode(&b).as_ref(), Ok(s), "{name}");
        o.valid("setpoint", "cdr", name, &setpoint_value(s), &b);
    }
    for seq in [0u32, 1, 0xffff_ffff] {
        for (name, s) in &setpoints[..7] {
            let b = [&seq.to_le_bytes()[..], &cdr(s)].concat();
            let value = format!(r#"{{"seq":{seq},"setpoint":{}}}"#, setpoint_value(s));
            o.valid(
                "ble-setpoint",
                "binary",
                &format!("{name}-seq-{seq}"),
                &value,
                &b,
            );
        }
    }
    // Captured from ROS 2 Jazzy `ros2 topic pub` (src/setpoint.rs): non-zero padding, a frame id.
    for (name, h) in [
        ("ros-empty-frame", "00010000000000000000000001000000005f726f9a9999999999b93f00000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        ("ros-base-link", "0001000001000000020000000a000000626173655f6c696e6b003638000000000000e03f000000000000d0bf000000000000000000000000000000000000000000000000000000000000f83f"),
    ] {
        let b = unhex(h);
        let s = Setpoint::decode(&b).unwrap();
        o.valid("setpoint", "cdr-decode-only", name, &setpoint_value(&s), &b);
    }
    let forward = cdr(&setpoints[1].1);
    let mutate = |at: usize, bytes: &[u8]| {
        let mut b = forward.clone();
        b[at..at + bytes.len()].copy_from_slice(bytes);
        b
    };
    let base_link = unhex("0001000001000000020000000a000000626173655f6c696e6b003638000000000000e03f000000000000d0bf000000000000000000000000000000000000000000000000000000000000f83f");
    let mut other_frame = base_link.clone();
    other_frame[16] = b'B';
    for (name, b) in [
        ("truncated", forward[..67].to_vec()),
        ("big-endian", mutate(0, &[0, 0, 0, 0])),
        ("frame-id-without-nul", mutate(12, &2u32.to_le_bytes())),
        ("unsupported-frame", other_frame),
        (
            "nanosec-out-of-range",
            mutate(8, &1_000_000_000u32.to_le_bytes()),
        ),
        ("linear-z", mutate(36, &1.0f64.to_le_bytes())),
        ("nan-vx", mutate(20, &f64::NAN.to_le_bytes())),
    ] {
        let e = Setpoint::decode(&b).unwrap_err();
        let error = match e {
            DecodeError::Truncated => "truncated",
            DecodeError::BadEncapsulation => "bad-encapsulation",
            DecodeError::BadFrameId => "bad-frame-id",
            DecodeError::UnsupportedFrame => "unsupported-frame",
            DecodeError::BadStamp => "bad-stamp",
            DecodeError::OutOfPlane => "out-of-plane",
            DecodeError::NotFinite => "not-finite",
        };
        o.invalid("setpoint", "cdr", name, error, &b);
    }

    // UDP frames: one per kind, with the payloads the car and clients exchange.
    let st = Status {
        state: State::Claimed,
        error: None,
        session: 0,
    }
    .encode();
    let net = Info {
        mode: Mode::Home,
        pending: None,
        reason: Reason::None,
        ip,
    }
    .encode();
    let info = [&CAPS.encode()[..], &st, &net].concat();
    let nonce = [7u8; 12];
    for (name, kind, session, seq, payload) in [
        ("hello", Kind::Hello, 0, 1, &[][..]),
        ("hello-ack", Kind::HelloAck, 42, 1, &CAPS.encode()[..]),
        ("setpoint", Kind::Setpoint, 42, 2, &forward[..]),
        ("arm", Kind::Arm, 42, 3, &[]),
        ("stop", Kind::Stop, 42, 4, &[]),
        ("bye", Kind::Bye, 42, 5, &[]),
        ("status-reply", Kind::Status, 42, 3, &st[..]),
        ("status-unsolicited", Kind::Status, 42, 0, &st[..]),
        ("busy", Kind::Busy, 0, 1, &[]),
        ("echo", Kind::Echo, 42, 6, &nonce[..]),
        ("mode-home", Kind::Mode, 42, 7, &[1]),
        ("mode-car-network", Kind::Mode, 42, 8, &[2]),
        ("info-query", Kind::Info, 0, 9, &[]),
        ("info-reply", Kind::Info, 0, 9, &info[..]),
        ("sequence-wrap", Kind::Arm, 42, 0xffff_ffff, &[]),
    ] {
        let b = frame(kind, session, seq, payload);
        let value = format!(
            r#"{{"kind":{},"session":{session},"sequence":{seq},"payload":"{}"}}"#,
            kind as u8,
            hex(payload)
        );
        o.valid("udp-frame", "binary", name, &value, &b);
    }
    let hello = frame(Kind::Hello, 0, 1, &[]);
    for (name, b) in [
        ("short-header", hello[..11].to_vec()),
        ("bad-magic", [&b"RX"[..], &hello[2..]].concat()),
        ("version-2", [&hello[..2], &[2], &hello[3..]].concat()),
        ("unknown-kind", [&hello[..3], &[12], &hello[4..]].concat()),
        ("kind-zero", [&hello[..3], &[0], &hello[4..]].concat()),
    ] {
        assert!(udp::parse(&b).is_none(), "{name}");
        o.invalid("udp-frame", "binary", name, "invalid", &b);
    }
    // Payload lengths the receiver checks (car: rovi-core device; clients: their decoders).
    for (name, b) in [
        ("status-short", frame(Kind::Status, 42, 3, &st[..2])),
        (
            "hello-ack-short",
            frame(Kind::HelloAck, 42, 1, &CAPS.encode()[..26]),
        ),
        ("info-reply-short", frame(Kind::Info, 0, 9, &info[..37])),
        ("echo-too-long", frame(Kind::Echo, 42, 6, &[0; 33])),
        ("mode-empty", frame(Kind::Mode, 42, 7, &[])),
    ] {
        o.invalid("udp-frame", "binary", name, "invalid-length", &b);
    }

    // BLE Control characteristic values (one byte): T8 keeps these codes.
    for (name, code) in [("arm", 1u8), ("stop", 2), ("home", 3), ("car-network", 4)] {
        o.valid(
            "ble-control",
            "binary",
            name,
            &format!(r#""{name}""#),
            &[code],
        );
    }

    format!(
        "{{\n\"comment\":\"Generated by crates/rovi-core/tests/protocol_vectors.rs; do not edit.\",\n\"valid\":[\n{}\n],\n\"invalid\":[\n{}\n]\n}}\n",
        o.valid.join(",\n"),
        o.invalid.join(",\n")
    )
}

#[test]
fn committed_vectors_match_the_encoders() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PATH);
    let generated = generate();
    if std::env::var_os("ROVI_BLESS_VECTORS").is_some() {
        std::fs::write(&path, &generated).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == generated,
        "{PATH} is stale: a wire encoding changed. Record the compatibility decision, then rerun \
         with ROVI_BLESS_VECTORS=1"
    );
}
