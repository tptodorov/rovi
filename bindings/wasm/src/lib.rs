//! `rovi_protocol` for the phone page: the rovi-protocol codecs with values as plain objects, using
//! the same field names and spellings as `crates/rovi-protocol/vectors.json`. Thin by design
//! (design D5): typed values in, typed values or a named error out; no session policy.
//!
//! Errors are JavaScript `Error`s named `DecodeError` or `EncodeError`, whose `kind` is the crate's
//! stable name (`length`, `unknown-code`, `inconsistent`, ...).

use js_sys::{Error as JsError, Reflect};
use protocol::ble::{self, Control};
use protocol::network::Command;
use protocol::udp::{self, Frame, FromCar, Info, ToCar};
use protocol::{capabilities, network, status};
use protocol::{Capabilities, NetworkInfo, Setpoint, Status};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use wasm_bindgen::prelude::*;

type Result<T> = core::result::Result<T, JsValue>;

fn named(name: &str, kind: &str) -> JsValue {
    let e = JsError::new(kind);
    e.set_name(name);
    let _ = Reflect::set(&e, &"kind".into(), &kind.into());
    e.into()
}

fn decode_err(e: protocol::DecodeError) -> JsValue {
    named("DecodeError", e.name())
}

/// A value the caller passed that the protocol would refuse.
fn refused(e: protocol::DecodeError) -> JsValue {
    named("EncodeError", e.name())
}

fn encode_err(e: protocol::EncodeError) -> JsValue {
    named(
        "EncodeError",
        match e {
            protocol::EncodeError::BufferTooSmall => "buffer-too-small",
            protocol::EncodeError::Invalid => "invalid",
        },
    )
}

fn to_js<T: Serialize>(v: &T) -> Result<JsValue> {
    serde_wasm_bindgen::to_value(v).map_err(|e| named("EncodeError", &e.to_string()))
}

fn from_js<T: DeserializeOwned>(v: JsValue) -> Result<T> {
    serde_wasm_bindgen::from_value(v).map_err(|e| named("EncodeError", &e.to_string()))
}

#[derive(Serialize, Deserialize)]
struct StatusV {
    state: String,
    reason: String,
    error: String,
    #[serde(default)]
    session: u32,
}

impl From<Status> for StatusV {
    fn from(s: Status) -> Self {
        let (state, reason, error) = s.names();
        Self {
            state: state.into(),
            reason: reason.into(),
            error: error.into(),
            session: s.session,
        }
    }
}

impl StatusV {
    fn value(&self) -> Result<Status> {
        Status::from_names(&self.state, &self.reason, &self.error, self.session).map_err(refused)
    }
}

#[derive(Serialize, Deserialize)]
struct CapsV {
    version: u8,
    wheels_radius: f32,
    center_projection_sum: f32,
    max_vx: f32,
    max_vy: f32,
    max_wz: f32,
    lease_ms: u16,
    claim_ms: u16,
    reclaim_ms: u16,
}

impl From<Capabilities> for CapsV {
    fn from(c: Capabilities) -> Self {
        Self {
            version: c.version,
            wheels_radius: c.wheels_radius,
            center_projection_sum: c.center_projection_sum,
            max_vx: c.max_vx,
            max_vy: c.max_vy,
            max_wz: c.max_wz,
            lease_ms: c.lease_ms,
            claim_ms: c.claim_ms,
            reclaim_ms: c.reclaim_ms,
        }
    }
}

impl CapsV {
    fn value(&self) -> Result<Capabilities> {
        let c = Capabilities {
            version: self.version,
            wheels_radius: self.wheels_radius,
            center_projection_sum: self.center_projection_sum,
            max_vx: self.max_vx,
            max_vy: self.max_vy,
            max_wz: self.max_wz,
            lease_ms: self.lease_ms,
            claim_ms: self.claim_ms,
            reclaim_ms: self.reclaim_ms,
        };
        c.validate().map_err(refused)?;
        Ok(c)
    }
}

#[derive(Serialize, Deserialize)]
struct NetworkV {
    mode: String,
    pending: String,
    reason: String,
    ip: String,
}

impl From<NetworkInfo> for NetworkV {
    fn from(i: NetworkInfo) -> Self {
        let [a, b, c, d] = i.ip;
        Self {
            mode: i.mode.name().into(),
            pending: i.pending.map_or("none", Command::name).into(),
            reason: i.reason.name().into(),
            ip: format!("{a}.{b}.{c}.{d}"),
        }
    }
}

impl NetworkV {
    fn value(&self) -> Result<NetworkInfo> {
        NetworkInfo::from_names(&self.mode, &self.pending, &self.reason, &self.ip).map_err(refused)
    }
}

/// Missing fields are zero, so `{vx: 0.1}` is a forward setpoint.
#[derive(Serialize, Deserialize)]
struct SetpointV {
    #[serde(default)]
    vx: f64,
    #[serde(default)]
    vy: f64,
    #[serde(default)]
    wz: f64,
    #[serde(default)]
    sec: i32,
    #[serde(default)]
    nanosec: u32,
}

impl From<Setpoint> for SetpointV {
    fn from(s: Setpoint) -> Self {
        Self {
            vx: s.vx,
            vy: s.vy,
            wz: s.wz,
            sec: s.sec,
            nanosec: s.nanosec,
        }
    }
}

impl SetpointV {
    fn value(&self) -> Setpoint {
        Setpoint {
            vx: self.vx,
            vy: self.vy,
            wz: self.wz,
            sec: self.sec,
            nanosec: self.nanosec,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ToCarV {
    Hello,
    Setpoint { setpoint: SetpointV },
    Arm,
    Stop,
    Bye,
    Echo { data: ByteBuf },
    Mode { command: String },
    Info,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum FromCarV {
    HelloAck {
        capabilities: CapsV,
    },
    Status {
        status: StatusV,
    },
    Busy,
    Echo {
        data: ByteBuf,
    },
    Info {
        capabilities: CapsV,
        status: StatusV,
        network: NetworkV,
    },
}

#[derive(Serialize, Deserialize)]
struct FrameV<M> {
    session: u32,
    seq: u32,
    message: M,
}

// Status

#[wasm_bindgen(js_name = statusDecode)]
pub fn status_decode(b: &[u8]) -> Result<JsValue> {
    to_js(&StatusV::from(Status::decode(b).map_err(decode_err)?))
}

#[wasm_bindgen(js_name = statusEncode)]
pub fn status_encode(v: JsValue) -> Result<Vec<u8>> {
    Ok(from_js::<StatusV>(v)?.value()?.encode().to_vec())
}

#[wasm_bindgen(js_name = statusDecodeRos)]
pub fn status_decode_ros(b: &[u8]) -> Result<JsValue> {
    to_js(&StatusV::from(Status::decode_cdr(b).map_err(decode_err)?))
}

#[wasm_bindgen(js_name = statusEncodeRos)]
pub fn status_encode_ros(v: JsValue) -> Result<Vec<u8>> {
    let mut out = [0u8; status::CDR_MAX];
    let n = from_js::<StatusV>(v)?
        .value()?
        .encode_cdr(&mut out)
        .map_err(encode_err)?;
    Ok(out[..n].to_vec())
}

// Capabilities

#[wasm_bindgen(js_name = capabilitiesDecode)]
pub fn capabilities_decode(b: &[u8]) -> Result<JsValue> {
    to_js(&CapsV::from(Capabilities::decode(b).map_err(decode_err)?))
}

#[wasm_bindgen(js_name = capabilitiesEncode)]
pub fn capabilities_encode(v: JsValue) -> Result<Vec<u8>> {
    Ok(from_js::<CapsV>(v)?.value()?.encode().to_vec())
}

#[wasm_bindgen(js_name = capabilitiesDecodeRos)]
pub fn capabilities_decode_ros(b: &[u8]) -> Result<JsValue> {
    to_js(&CapsV::from(
        Capabilities::decode_cdr(b).map_err(decode_err)?,
    ))
}

#[wasm_bindgen(js_name = capabilitiesEncodeRos)]
pub fn capabilities_encode_ros(v: JsValue) -> Result<Vec<u8>> {
    let mut out = [0u8; capabilities::CDR_MAX];
    let n = from_js::<CapsV>(v)?
        .value()?
        .encode_cdr(&mut out)
        .map_err(encode_err)?;
    Ok(out[..n].to_vec())
}

/// The single client minimum for advertised windows (design I11).
#[wasm_bindgen(js_name = meetsClientMinimum)]
pub fn meets_client_minimum(v: JsValue) -> Result<bool> {
    Ok(from_js::<CapsV>(v)?.value()?.meets_client_minimum())
}

// Network information

#[wasm_bindgen(js_name = networkDecode)]
pub fn network_decode(b: &[u8]) -> Result<JsValue> {
    to_js(&NetworkV::from(NetworkInfo::decode(b).map_err(decode_err)?))
}

#[wasm_bindgen(js_name = networkEncode)]
pub fn network_encode(v: JsValue) -> Result<Vec<u8>> {
    Ok(from_js::<NetworkV>(v)?.value()?.encode().to_vec())
}

#[wasm_bindgen(js_name = networkDecodeRos)]
pub fn network_decode_ros(b: &[u8]) -> Result<JsValue> {
    to_js(&NetworkV::from(
        NetworkInfo::decode_cdr(b).map_err(decode_err)?,
    ))
}

#[wasm_bindgen(js_name = networkEncodeRos)]
pub fn network_encode_ros(v: JsValue) -> Result<Vec<u8>> {
    let mut out = [0u8; network::CDR_MAX];
    let n = from_js::<NetworkV>(v)?
        .value()?
        .encode_cdr(&mut out)
        .map_err(encode_err)?;
    Ok(out[..n].to_vec())
}

// Setpoints

#[wasm_bindgen(js_name = setpointDecode)]
pub fn setpoint_decode(b: &[u8]) -> Result<JsValue> {
    to_js(&SetpointV::from(
        Setpoint::decode(b).map_err(|e| decode_err(e.into()))?,
    ))
}

#[wasm_bindgen(js_name = setpointEncode)]
pub fn setpoint_encode(v: JsValue) -> Result<Vec<u8>> {
    let mut out = [0u8; protocol::setpoint::MIN_LEN];
    let n = from_js::<SetpointV>(v)?
        .value()
        .encode(&mut out)
        .map_err(encode_err)?;
    Ok(out[..n].to_vec())
}

/// The BLE Setpoint value as `{seq, setpoint}`.
#[wasm_bindgen(js_name = bleSetpointDecode)]
pub fn ble_setpoint_decode(b: &[u8]) -> Result<JsValue> {
    #[derive(Serialize)]
    struct V {
        seq: u32,
        setpoint: SetpointV,
    }
    let (seq, sp) = ble::decode_setpoint(b).map_err(decode_err)?;
    to_js(&V {
        seq,
        setpoint: sp.into(),
    })
}

/// `seq` is a JavaScript number: anything but a whole u32 is refused, not wrapped.
#[wasm_bindgen(js_name = bleSetpointEncode)]
pub fn ble_setpoint_encode(seq: f64, v: JsValue) -> Result<Vec<u8>> {
    if seq.fract() != 0.0 || !(0.0..=f64::from(u32::MAX)).contains(&seq) {
        return Err(named("EncodeError", "invalid"));
    }
    let sp = from_js::<SetpointV>(v)?.value();
    Ok(ble::encode_setpoint(seq as u32, &sp)
        .map_err(encode_err)?
        .to_vec())
}

// BLE Control

/// One Control byte as `arm`, `stop`, `home` or `car-network`.
#[wasm_bindgen(js_name = controlDecode)]
pub fn control_decode(b: &[u8]) -> Result<String> {
    Control::decode(b)
        .map(|c| c.name().into())
        .map_err(decode_err)
}

#[wasm_bindgen(js_name = controlEncode)]
pub fn control_encode(name: &str) -> Result<Vec<u8>> {
    let c = Control::from_name(name).ok_or_else(|| named("EncodeError", "unknown-code"))?;
    Ok(vec![c.code()])
}

// UDP frames: {session, seq, message: {kind, ...fields}}

fn frame_bytes(
    f: impl Fn(&mut [u8]) -> core::result::Result<usize, protocol::EncodeError>,
) -> Result<Vec<u8>> {
    let mut out = [0u8; udp::MAX_FRAME];
    let n = f(&mut out).map_err(encode_err)?;
    Ok(out[..n].to_vec())
}

#[wasm_bindgen(js_name = udpDecodeToCar)]
pub fn udp_decode_to_car(b: &[u8]) -> Result<JsValue> {
    let f = Frame::<ToCar>::decode(b).map_err(decode_err)?;
    let message = match f.message {
        ToCar::Hello => ToCarV::Hello,
        ToCar::Setpoint(s) => ToCarV::Setpoint { setpoint: s.into() },
        ToCar::Arm => ToCarV::Arm,
        ToCar::Stop => ToCarV::Stop,
        ToCar::Bye => ToCarV::Bye,
        ToCar::Echo(p) => ToCarV::Echo {
            data: ByteBuf::from(p),
        },
        ToCar::Mode(c) => ToCarV::Mode {
            command: c.name().into(),
        },
        ToCar::Info => ToCarV::Info,
    };
    to_js(&FrameV {
        session: f.session,
        seq: f.seq,
        message,
    })
}

#[wasm_bindgen(js_name = udpEncodeToCar)]
pub fn udp_encode_to_car(v: JsValue) -> Result<Vec<u8>> {
    let f: FrameV<ToCarV> = from_js(v)?;
    let (session, seq) = (f.session, f.seq);
    let message = match &f.message {
        ToCarV::Hello => ToCar::Hello,
        ToCarV::Setpoint { setpoint } => ToCar::Setpoint(setpoint.value()),
        ToCarV::Arm => ToCar::Arm,
        ToCarV::Stop => ToCar::Stop,
        ToCarV::Bye => ToCar::Bye,
        ToCarV::Echo { data } => ToCar::Echo(data),
        ToCarV::Mode { command } => ToCar::Mode(
            Command::from_name(command).ok_or_else(|| named("EncodeError", "unknown-code"))?,
        ),
        ToCarV::Info => ToCar::Info,
    };
    frame_bytes(|out| {
        Frame {
            session,
            seq,
            message,
        }
        .encode(out)
    })
}

#[wasm_bindgen(js_name = udpDecodeFromCar)]
pub fn udp_decode_from_car(b: &[u8]) -> Result<JsValue> {
    let f = Frame::<FromCar>::decode(b).map_err(decode_err)?;
    let message = match f.message {
        FromCar::HelloAck(c) => FromCarV::HelloAck {
            capabilities: c.into(),
        },
        FromCar::Status(s) => FromCarV::Status { status: s.into() },
        FromCar::Busy => FromCarV::Busy,
        FromCar::Echo(p) => FromCarV::Echo {
            data: ByteBuf::from(p),
        },
        FromCar::Info(i) => FromCarV::Info {
            capabilities: i.capabilities.into(),
            status: i.status.into(),
            network: i.network.into(),
        },
    };
    to_js(&FrameV {
        session: f.session,
        seq: f.seq,
        message,
    })
}

#[wasm_bindgen(js_name = udpEncodeFromCar)]
pub fn udp_encode_from_car(v: JsValue) -> Result<Vec<u8>> {
    let f: FrameV<FromCarV> = from_js(v)?;
    let (session, seq) = (f.session, f.seq);
    let message = match &f.message {
        FromCarV::HelloAck { capabilities } => FromCar::HelloAck(capabilities.value()?),
        FromCarV::Status { status } => FromCar::Status(status.value()?),
        FromCarV::Busy => FromCar::Busy,
        FromCarV::Echo { data } => FromCar::Echo(data),
        FromCarV::Info {
            capabilities,
            status,
            network,
        } => FromCar::Info(Info {
            capabilities: capabilities.value()?,
            status: status.value()?,
            network: network.value()?,
        }),
    };
    frame_bytes(|out| {
        Frame {
            session,
            seq,
            message,
        }
        .encode(out)
    })
}

// Constants

#[wasm_bindgen(js_name = constants)]
pub fn constants() -> Result<JsValue> {
    #[derive(Serialize)]
    struct Uuids {
        service: &'static str,
        setpoint: &'static str,
        control: &'static str,
        capabilities: &'static str,
        status: &'static str,
        echo: &'static str,
        network: &'static str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Constants {
        ble_uuids: Uuids,
        ble_setpoint_len: usize,
        echo_max: usize,
        min_lease_ms: u16,
        min_claim_ms: u16,
        udp_port: u16,
    }
    to_js(&Constants {
        ble_uuids: Uuids {
            service: ble::SERVICE,
            setpoint: ble::SETPOINT,
            control: ble::CONTROL,
            capabilities: ble::CAPABILITIES,
            status: ble::STATUS,
            echo: ble::ECHO,
            network: ble::NETWORK,
        },
        ble_setpoint_len: ble::SETPOINT_LEN,
        echo_max: protocol::ECHO_MAX,
        min_lease_ms: capabilities::MIN_LEASE_MS,
        min_claim_ms: capabilities::MIN_CLAIM_MS,
        udp_port: udp::PORT,
    })
}
