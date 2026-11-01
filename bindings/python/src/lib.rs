//! `rovi_protocol` for Python: the rovi-protocol codecs with values as plain dicts, using the
//! same field names and spellings as `crates/rovi-protocol/vectors.json`. Thin by design
//! (design D5): typed values in, typed values or a named error out; no session policy.
//!
//! Errors: `DecodeError(kind)` and `EncodeError(kind)`, both `ValueError`s whose single argument
//! is the crate's stable kind name (`length`, `unknown-code`, `inconsistent`, ...).

use protocol::ble::{self, Control};
use protocol::capabilities;
use protocol::network::{self, Command, NetMode, NetworkInfo, Reason};
use protocol::status::{self, Error, State, Status, StopReason};
use protocol::udp::{self, Frame, FromCar, Info, ToCar};
use protocol::{Capabilities, Setpoint};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyTuple};

create_exception!(rovi_protocol, DecodeError, PyValueError);
create_exception!(rovi_protocol, EncodeError, PyValueError);

fn decode_err(e: protocol::DecodeError) -> PyErr {
    DecodeError::new_err(e.name())
}

fn encode_err(e: protocol::EncodeError) -> PyErr {
    EncodeError::new_err(match e {
        protocol::EncodeError::BufferTooSmall => "buffer-too-small",
        protocol::EncodeError::Invalid => "invalid",
    })
}

/// A value the caller passed that names nothing the protocol defines.
fn bad(what: &str, value: &str) -> PyErr {
    EncodeError::new_err(format!("unknown {what}: {value}"))
}

fn bytes<'py>(py: Python<'py>, b: &[u8]) -> Bound<'py, PyBytes> {
    PyBytes::new(py, b)
}

fn get<'py, T: for<'a> FromPyObject<'a, 'py>>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<T>
where
    for<'a> <T as FromPyObject<'a, 'py>>::Error: Into<PyErr>,
{
    let item = d
        .get_item(key)?
        .ok_or_else(|| EncodeError::new_err(format!("missing {key}")))?;
    item.extract::<T>().map_err(Into::into)
}

// Names: the crate's spellings, matched back over every variant.

const STOP_REASONS: [StopReason; 4] = [
    StopReason::Stop,
    StopReason::LeaseExpired,
    StopReason::OwnerLost,
    StopReason::OutOfLimits,
];
const ERRORS: [Error; 4] = [
    Error::Busy,
    Error::NotOwner,
    Error::NotReady,
    Error::Invalid,
];
const MODES: [NetMode; 3] = [NetMode::Fallback, NetMode::Attempting, NetMode::Home];
const COMMANDS: [Command; 2] = [Command::Home, Command::CarNetwork];
const REASONS: [Reason; 8] = [
    Reason::None,
    Reason::NoSettings,
    Reason::Scan,
    Reason::Join,
    Reason::Dhcp,
    Reason::Router,
    Reason::LinkLost,
    Reason::Requested,
];

fn find<T: Copy>(all: &[T], name: fn(T) -> &'static str, what: &str, s: &str) -> PyResult<T> {
    all.iter()
        .copied()
        .find(|v| name(*v) == s)
        .ok_or_else(|| bad(what, s))
}

fn optional<T: Copy>(
    all: &[T],
    name: fn(T) -> &'static str,
    what: &str,
    s: &str,
) -> PyResult<Option<T>> {
    if s == "none" {
        Ok(None)
    } else {
        find(all, name, what, s).map(Some)
    }
}

// Status

fn status_dict<'py>(py: Python<'py>, s: Status) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("state", s.state.name())?;
    d.set_item(
        "reason",
        match s.state {
            State::Stopped(r) => r.name(),
            _ => "none",
        },
    )?;
    d.set_item("error", s.error.map_or("none", Error::name))?;
    d.set_item("session", s.session)?;
    Ok(d)
}

fn status_value(d: &Bound<'_, PyDict>) -> PyResult<Status> {
    let state: String = get(d, "state")?;
    let reason: String = get(d, "reason")?;
    let reason = optional(&STOP_REASONS, StopReason::name, "reason", &reason)?;
    let state = match (state.as_str(), reason) {
        ("open", None) => State::Open,
        ("claimed", None) => State::Claimed,
        ("armed", None) => State::Armed,
        ("stopped", Some(r)) => State::Stopped(r),
        _ => return Err(EncodeError::new_err("inconsistent state and reason")),
    };
    let error: String = get(d, "error")?;
    let session = match d.get_item("session")? {
        Some(v) => v.extract()?,
        None => 0,
    };
    Ok(Status {
        state,
        error: optional(&ERRORS, Error::name, "error", &error)?,
        session,
    })
}

/// Binary Status (BLE characteristic, UDP `STATUS`): session decodes as 0.
#[pyfunction]
fn status_decode<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    status_dict(py, Status::decode(b).map_err(decode_err)?)
}

#[pyfunction]
fn status_encode<'py>(py: Python<'py>, d: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyBytes>> {
    Ok(bytes(py, &status_value(d)?.encode()))
}

/// ROS `/rovi/status` sample (`std_msgs/String` CDR).
#[pyfunction]
fn status_decode_ros<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    status_dict(py, Status::decode_cdr(b).map_err(decode_err)?)
}

#[pyfunction]
fn status_encode_ros<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; status::CDR_MAX];
    let n = status_value(d)?.encode_cdr(&mut out).map_err(encode_err)?;
    Ok(bytes(py, &out[..n]))
}

// Capabilities

fn caps_dict<'py>(py: Python<'py>, c: Capabilities) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("version", c.version)?;
    d.set_item("wheels_radius", c.wheels_radius)?;
    d.set_item("center_projection_sum", c.center_projection_sum)?;
    d.set_item("max_vx", c.max_vx)?;
    d.set_item("max_vy", c.max_vy)?;
    d.set_item("max_wz", c.max_wz)?;
    d.set_item("lease_ms", c.lease_ms)?;
    d.set_item("claim_ms", c.claim_ms)?;
    d.set_item("reclaim_ms", c.reclaim_ms)?;
    Ok(d)
}

fn caps_value(d: &Bound<'_, PyDict>) -> PyResult<Capabilities> {
    let f = |k| get::<f64>(d, k).map(|v| v as f32);
    let c = Capabilities {
        version: get(d, "version")?,
        wheels_radius: f("wheels_radius")?,
        center_projection_sum: f("center_projection_sum")?,
        max_vx: f("max_vx")?,
        max_vy: f("max_vy")?,
        max_wz: f("max_wz")?,
        lease_ms: get(d, "lease_ms")?,
        claim_ms: get(d, "claim_ms")?,
        reclaim_ms: get(d, "reclaim_ms")?,
    };
    c.validate().map_err(|e| EncodeError::new_err(e.name()))?;
    Ok(c)
}

#[pyfunction]
fn capabilities_decode<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    caps_dict(py, Capabilities::decode(b).map_err(decode_err)?)
}

#[pyfunction]
fn capabilities_encode<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(bytes(py, &caps_value(d)?.encode()))
}

#[pyfunction]
fn capabilities_decode_ros<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    caps_dict(py, Capabilities::decode_cdr(b).map_err(decode_err)?)
}

#[pyfunction]
fn capabilities_encode_ros<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; capabilities::CDR_MAX];
    let n = caps_value(d)?.encode_cdr(&mut out).map_err(encode_err)?;
    Ok(bytes(py, &out[..n]))
}

/// The single client minimum for advertised windows (design I11).
#[pyfunction]
fn meets_client_minimum(d: &Bound<'_, PyDict>) -> PyResult<bool> {
    Ok(caps_value(d)?.meets_client_minimum())
}

// Network information

fn network_dict<'py>(py: Python<'py>, i: NetworkInfo) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    let [a, b, c, e] = i.ip;
    d.set_item("mode", i.mode.name())?;
    d.set_item("pending", i.pending.map_or("none", Command::name))?;
    d.set_item("reason", i.reason.name())?;
    d.set_item("ip", format!("{a}.{b}.{c}.{e}"))?;
    Ok(d)
}

fn network_value(d: &Bound<'_, PyDict>) -> PyResult<NetworkInfo> {
    let (mode, pending, reason, ip): (String, String, String, String) = (
        get(d, "mode")?,
        get(d, "pending")?,
        get(d, "reason")?,
        get(d, "ip")?,
    );
    let octets: Vec<u8> = ip
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| bad("ip", &ip))?;
    Ok(NetworkInfo {
        mode: find(&MODES, NetMode::name, "mode", &mode)?,
        pending: optional(&COMMANDS, Command::name, "pending", &pending)?,
        reason: find(&REASONS, Reason::name, "reason", &reason)?,
        ip: octets.try_into().map_err(|_| bad("ip", &ip))?,
    })
}

#[pyfunction]
fn network_decode<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    network_dict(py, NetworkInfo::decode(b).map_err(decode_err)?)
}

#[pyfunction]
fn network_encode<'py>(py: Python<'py>, d: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyBytes>> {
    Ok(bytes(py, &network_value(d)?.encode()))
}

#[pyfunction]
fn network_decode_ros<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    network_dict(py, NetworkInfo::decode_cdr(b).map_err(decode_err)?)
}

#[pyfunction]
fn network_encode_ros<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; network::CDR_MAX];
    let n = network_value(d)?.encode_cdr(&mut out).map_err(encode_err)?;
    Ok(bytes(py, &out[..n]))
}

// Setpoint

fn setpoint_dict<'py>(py: Python<'py>, s: Setpoint) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("vx", s.vx)?;
    d.set_item("vy", s.vy)?;
    d.set_item("wz", s.wz)?;
    d.set_item("sec", s.sec)?;
    d.set_item("nanosec", s.nanosec)?;
    Ok(d)
}

/// Missing fields are zero, so `{"vx": 0.1}` is a forward setpoint.
fn setpoint_value(d: &Bound<'_, PyDict>) -> PyResult<Setpoint> {
    let f = |k| -> PyResult<f64> {
        Ok(d.get_item(k)?
            .map(|v| v.extract())
            .transpose()?
            .unwrap_or(0.0))
    };
    Ok(Setpoint {
        vx: f("vx")?,
        vy: f("vy")?,
        wz: f("wz")?,
        sec: d
            .get_item("sec")?
            .map(|v| v.extract())
            .transpose()?
            .unwrap_or(0),
        nanosec: d
            .get_item("nanosec")?
            .map(|v| v.extract())
            .transpose()?
            .unwrap_or(0),
    })
}

/// `geometry_msgs/TwistStamped` CDR (UDP `SETPOINT` payload, ROS `/cmd_vel`).
#[pyfunction]
fn setpoint_decode<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    setpoint_dict(py, Setpoint::decode(b).map_err(|e| decode_err(e.into()))?)
}

#[pyfunction]
fn setpoint_encode<'py>(py: Python<'py>, d: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; protocol::setpoint::MIN_LEN];
    let n = setpoint_value(d)?.encode(&mut out).map_err(encode_err)?;
    Ok(bytes(py, &out[..n]))
}

/// The BLE Setpoint value: `(seq, setpoint)`.
#[pyfunction]
fn ble_setpoint_decode<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyTuple>> {
    let (seq, sp) = ble::decode_setpoint(b).map_err(decode_err)?;
    PyTuple::new(
        py,
        [
            seq.into_pyobject(py)?.into_any(),
            setpoint_dict(py, sp)?.into_any(),
        ],
    )
}

#[pyfunction]
fn ble_setpoint_encode<'py>(
    py: Python<'py>,
    seq: u32,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(bytes(
        py,
        &ble::encode_setpoint(seq, &setpoint_value(d)?).map_err(encode_err)?,
    ))
}

// BLE Control

fn control_name(c: Control) -> &'static str {
    match c {
        Control::Arm => "arm",
        Control::Stop => "stop",
        Control::Mode(c) => c.name(),
    }
}

/// One Control byte as `arm`, `stop`, `home` or `car-network`.
#[pyfunction]
fn control_decode(b: &[u8]) -> PyResult<&'static str> {
    Control::decode(b).map(control_name).map_err(decode_err)
}

#[pyfunction]
fn control_encode<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyBytes>> {
    let c = match name {
        "arm" => Control::Arm,
        "stop" => Control::Stop,
        other => Control::Mode(find(&COMMANDS, Command::name, "control", other)?),
    };
    Ok(bytes(py, &[c.code()]))
}

// UDP frames: {"session", "seq", "message": {"kind": ..., fields}}

fn frame_dict<'py>(
    py: Python<'py>,
    session: u32,
    seq: u32,
    message: Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("session", session)?;
    d.set_item("seq", seq)?;
    d.set_item("message", message)?;
    Ok(d)
}

fn message<'py>(py: Python<'py>, kind: &str) -> PyResult<Bound<'py, PyDict>> {
    let m = PyDict::new(py);
    m.set_item("kind", kind)?;
    Ok(m)
}

fn frame_parts<'py>(d: &Bound<'py, PyDict>) -> PyResult<(u32, u32, Bound<'py, PyDict>, String)> {
    let m: Bound<'py, PyDict> = get(d, "message")?;
    let kind: String = get(&m, "kind")?;
    Ok((get(d, "session")?, get(d, "seq")?, m, kind))
}

fn encode_frame<'py, F: Fn(&mut [u8]) -> Result<usize, protocol::EncodeError>>(
    py: Python<'py>,
    f: F,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut out = [0u8; udp::MAX_FRAME];
    let n = f(&mut out).map_err(encode_err)?;
    Ok(bytes(py, &out[..n]))
}

/// `(kind, session, seq, payload)` of any frame with a valid header; the payload is unchecked.
#[pyfunction]
fn udp_split<'py>(py: Python<'py>, b: &[u8]) -> PyResult<(u8, u32, u32, Bound<'py, PyBytes>)> {
    let (h, p) = udp::split(b).map_err(decode_err)?;
    Ok((h.kind as u8, h.session, h.seq, bytes(py, p)))
}

/// A datagram a client sends to the car.
#[pyfunction]
fn udp_decode_to_car<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    let f = Frame::<ToCar>::decode(b).map_err(decode_err)?;
    let m = match f.message {
        ToCar::Hello => message(py, "hello")?,
        ToCar::Setpoint(sp) => {
            let m = message(py, "setpoint")?;
            m.set_item("setpoint", setpoint_dict(py, sp)?)?;
            m
        }
        ToCar::Arm => message(py, "arm")?,
        ToCar::Stop => message(py, "stop")?,
        ToCar::Bye => message(py, "bye")?,
        ToCar::Echo(p) => {
            let m = message(py, "echo")?;
            m.set_item("data", bytes(py, p))?;
            m
        }
        ToCar::Mode(c) => {
            let m = message(py, "mode")?;
            m.set_item("command", c.name())?;
            m
        }
        ToCar::Info => message(py, "info")?,
    };
    frame_dict(py, f.session, f.seq, m)
}

#[pyfunction]
fn udp_encode_to_car<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    let (session, seq, m, kind) = frame_parts(d)?;
    let data: Option<Vec<u8>> = m.get_item("data")?.map(|v| v.extract()).transpose()?;
    let message = match kind.as_str() {
        "hello" => ToCar::Hello,
        "setpoint" => ToCar::Setpoint(setpoint_value(&get(&m, "setpoint")?)?),
        "arm" => ToCar::Arm,
        "stop" => ToCar::Stop,
        "bye" => ToCar::Bye,
        "echo" => ToCar::Echo(data.as_deref().unwrap_or(&[])),
        "mode" => ToCar::Mode(find(
            &COMMANDS,
            Command::name,
            "command",
            &get::<String>(&m, "command")?,
        )?),
        "info" => ToCar::Info,
        other => return Err(bad("kind", other)),
    };
    encode_frame(py, |out| {
        Frame {
            session,
            seq,
            message,
        }
        .encode(out)
    })
}

/// A datagram the car sends to a client.
#[pyfunction]
fn udp_decode_from_car<'py>(py: Python<'py>, b: &[u8]) -> PyResult<Bound<'py, PyDict>> {
    let f = Frame::<FromCar>::decode(b).map_err(decode_err)?;
    let m = match f.message {
        FromCar::HelloAck(c) => {
            let m = message(py, "hello-ack")?;
            m.set_item("capabilities", caps_dict(py, c)?)?;
            m
        }
        FromCar::Status(s) => {
            let m = message(py, "status")?;
            m.set_item("status", status_dict(py, s)?)?;
            m
        }
        FromCar::Busy => message(py, "busy")?,
        FromCar::Echo(p) => {
            let m = message(py, "echo")?;
            m.set_item("data", bytes(py, p))?;
            m
        }
        FromCar::Info(i) => {
            let m = message(py, "info")?;
            m.set_item("capabilities", caps_dict(py, i.capabilities)?)?;
            m.set_item("status", status_dict(py, i.status)?)?;
            m.set_item("network", network_dict(py, i.network)?)?;
            m
        }
    };
    frame_dict(py, f.session, f.seq, m)
}

#[pyfunction]
fn udp_encode_from_car<'py>(
    py: Python<'py>,
    d: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    let (session, seq, m, kind) = frame_parts(d)?;
    let data: Option<Vec<u8>> = m.get_item("data")?.map(|v| v.extract()).transpose()?;
    let message = match kind.as_str() {
        "hello-ack" => FromCar::HelloAck(caps_value(&get(&m, "capabilities")?)?),
        "status" => FromCar::Status(status_value(&get(&m, "status")?)?),
        "busy" => FromCar::Busy,
        "echo" => FromCar::Echo(data.as_deref().unwrap_or(&[])),
        "info" => FromCar::Info(Info {
            capabilities: caps_value(&get(&m, "capabilities")?)?,
            status: status_value(&get(&m, "status")?)?,
            network: network_value(&get(&m, "network")?)?,
        }),
        other => return Err(bad("kind", other)),
    };
    encode_frame(py, |out| {
        Frame {
            session,
            seq,
            message,
        }
        .encode(out)
    })
}

#[pymodule]
fn rovi_protocol(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("DecodeError", py.get_type::<DecodeError>())?;
    m.add("EncodeError", py.get_type::<EncodeError>())?;
    m.add("UDP_PORT", udp::PORT)?;
    m.add("UDP_VERSION", udp::VERSION)?;
    m.add("ECHO_MAX", protocol::ECHO_MAX)?;
    m.add("MIN_LEASE_MS", capabilities::MIN_LEASE_MS)?;
    m.add("MIN_CLAIM_MS", capabilities::MIN_CLAIM_MS)?;
    let uuids = PyDict::new(py);
    for (k, v) in [
        ("service", ble::SERVICE),
        ("setpoint", ble::SETPOINT),
        ("control", ble::CONTROL),
        ("capabilities", ble::CAPABILITIES),
        ("status", ble::STATUS),
        ("echo", ble::ECHO),
        ("network", ble::NETWORK),
    ] {
        uuids.set_item(k, v)?;
    }
    m.add("BLE_UUIDS", uuids)?;
    m.add("BLE_SETPOINT_LEN", ble::SETPOINT_LEN)?;
    m.add("STATUS_LEN", status::LEN)?;
    m.add("CAPABILITIES_LEN", capabilities::LEN)?;
    m.add("NETWORK_LEN", network::LEN)?;
    m.add("INFO_LEN", udp::INFO_LEN)?;
    m.add("UDP_HEADER_LEN", udp::HEADER_LEN)?;
    m.add_function(wrap_pyfunction!(status_decode, m)?)?;
    m.add_function(wrap_pyfunction!(status_encode, m)?)?;
    m.add_function(wrap_pyfunction!(status_decode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(status_encode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_decode, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_encode, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_decode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_encode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(meets_client_minimum, m)?)?;
    m.add_function(wrap_pyfunction!(network_decode, m)?)?;
    m.add_function(wrap_pyfunction!(network_encode, m)?)?;
    m.add_function(wrap_pyfunction!(network_decode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(network_encode_ros, m)?)?;
    m.add_function(wrap_pyfunction!(setpoint_decode, m)?)?;
    m.add_function(wrap_pyfunction!(setpoint_encode, m)?)?;
    m.add_function(wrap_pyfunction!(ble_setpoint_decode, m)?)?;
    m.add_function(wrap_pyfunction!(ble_setpoint_encode, m)?)?;
    m.add_function(wrap_pyfunction!(control_decode, m)?)?;
    m.add_function(wrap_pyfunction!(control_encode, m)?)?;
    m.add_function(wrap_pyfunction!(udp_split, m)?)?;
    m.add_function(wrap_pyfunction!(udp_decode_to_car, m)?)?;
    m.add_function(wrap_pyfunction!(udp_encode_to_car, m)?)?;
    m.add_function(wrap_pyfunction!(udp_decode_from_car, m)?)?;
    m.add_function(wrap_pyfunction!(udp_encode_from_car, m)?)?;
    Ok(())
}
