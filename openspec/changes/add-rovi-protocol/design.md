# Rovi protocol design

## Context

See [proposal](proposal.md). Today's mapping is [protocol.md](../../../docs/reference/protocol.md); layouts live in `rovi-core` (`udp.rs`, `setpoint.rs`, `status.rs`, `capabilities.rs`, `network.rs`, `device.rs`, `rmw.rs`) next to device logic, with hand-written copies in Python, JavaScript and the conformance suite.

## Decisions

### D1. One crate is the contract

`crates/rovi-protocol` is `no_std`, has no allocator and no I/O, and depends only on `libm` if needed. It owns:

* identifiers: BLE service/characteristic UUIDs, UDP port, magic, version and `Kind`s, ROS topic names, message types and QoS;
* messages: `Setpoint`, `Control` (Arm, Stop, Home, CarNetwork), `Status` (state, reason, error), `Capabilities`, `NetworkInfo` (mode, pending, reason, IPv4), `Echo`, UDP `Frame` with typed payloads, ROS `String`/`UInt8`/`Bool`/`Empty` CDR bodies used by Rovi topics;
* sizes, limits and error codes as enums, never bare bytes.

Device behaviour (arbiter, network state machine, publication schedules, settings) stays in `rovi-core`, which depends on `rovi-protocol`. Callers import from `rovi-protocol` directly; no long-lived re-exports.

### D2. Symmetric, validated codecs

Every message has `encode(&self, out) -> Result<usize, EncodeError>` and `decode(&[u8]) -> Result<Self, DecodeError>`. Decoding rejects wrong length, unknown codes, reserved non-zero bytes, non-finite floats and unsupported versions. For canonical bytes `encode(decode(b)) == b`; for valid values `decode(encode(v)) == v`, floats bit-exact. Short buffers produce no partial output. Property tests cover all messages.

### D3. Direction is a type

`ToCar` and `FromCar` enums per transport group the messages each side may send. Clients get `encode(ToCar)` and `decode -> FromCar`; the car the reverse. A client cannot construct a `STATUS` frame or decode a `SETPOINT` as car output without going through test-only helpers.

### D4. Golden vectors are generated, not written

A small `rovi-protocol` binary emits `crates/rovi-protocol/vectors.json`: name, typed value, bytes (hex), and malformed cases with expected error. CI regenerates and fails on diff. Vectors are seeded from today's Rust tests and the current Python/JavaScript fixtures before extraction, so moving code cannot silently change bytes. A changed vector requires an explicit version decision in the PR: compatible addition, or `VERSION` bump with decoders rejecting the other version.

### D5. Bindings, not re-implementations

| Participant | Binding | Replaces |
| --- | --- | --- |
| Firmware, sim car | Rust crate | `rovi-core` wire modules |
| Laptop teleop, `apps/client`, sim scenarios, bench scripts, conformance | Python extension `rovi_protocol` (PyO3, built with maturin, provided by `shell.nix` and CI) | `wire.py`, `protocol.py`, byte slicing |
| iPhone page | WebAssembly module (`wasm-bindgen`, pinned), shipped in the revision-pinned build | codec parts of `wire.js` |
| ROS tools | ROS message types plus topic/QoS constants from the binding | literal topic strings |

Bindings are thin: typed values in, typed values or a named error out. No policy, retries or session state; those belong to the shared client ([#108](https://github.com/tptodorov/rovi/issues/108)). If the phone module fails to load, connection stays disabled with an explanation; there is no JavaScript fallback codec.

### D6. One conformance suite, two targets

`apps/simcar/scenarios/conformance.py` uses only `rovi_protocol` to build and read messages and gains a target: `sim` (today's CI run over UDP, virtual BLE and ROS) and `device` (real car over UDP and BLE via BlueZ, ROS via blackbox). On the device, observational rules (INFO, Echo, Capabilities, Status, Network, malformed rejection, BUSY, not-owner) run with motor power off and never arm. Motion rules run only in a separately announced stand batch. Results record firmware revision and vector hash.

### D7. One model per concept, encodings per transport

Each concept (Status, Capabilities, Network information, Command, Echo, Setpoint) is one Rust type. BLE, UDP and ROS are encodings of that type, defined next to it, each with an encoder, a decoder and round-trip tests, including the ROS logfmt text. Display and log names come from the same enum, using the ROS spelling (`lease-expired`, `car-network`). No client keeps its own name table.

### D8. Variant inventory and alignment

Inventory from 2026-10-28 code review (`rovi-core`, `firmware/src`, `apps/simcar/src`, `apps/client`, `apps/teleop`, `apps/iphone`, `conformance.py`).

**Implementation variants** (one message, several codes). All are removed by D1–D7 with **no byte change**:

| # | Difference | Today | Aligned |
| --- | --- | --- | --- |
| I1 | Capabilities validation | `apps/client`: none; teleop: lease ≥ 200, claim/reclaim > 0; phone: lease ≥ 500, claim ≥ 1000, reclaim ≠ 0 | Crate decoder checks structure (version 1, finite positive geometry, finite non-negative limits, non-zero windows). Client minimums are named per-profile constants in the crate, not inline literals |
| I2 | Status consistency (stopped ⇔ reason) | Phone checks at decode; teleop checks later in its session; `apps/client` never | Decoder rejects it |
| I3 | Network reserved byte 3 | Phone rejects non-zero; Python ignores it | Decoder rejects it |
| I4 | Names | Python `LeaseExpired`/`None`, phone `CarNetwork`/`'none'`, ROS `lease-expired`/`car-network` | One enum, ROS spelling (D7) |
| I5 | Setpoint encoding | Phone rejects non-finite values; Python does not | Encoder rejects non-finite values and `vz`/roll/pitch ≠ 0 |
| I6 | UDP payload sizes and INFO layout | Teleop length table; teleop and conformance slice `[27:30]`, `[30:38]` | Typed `Frame` / `Info` |
| I7 | BLE UUIDs | Firmware macro literals, `ble.py`, `wire.js` | Crate constants; firmware `gatt_service` uses them |
| I8 | ROS topic table, types and QoS | Duplicated in `firmware/src/ros.rs` and `apps/simcar/src/ros.rs` (~440 differing lines, endpoint indexes by position) | One topic table in the crate; endpoints by name |
| I9 | Network republish (on change, else every 1 s) | Hand-written in both ROS adapters; Status and Capabilities use `rovi-core` `Publication` | A `rovi-core` network `Publication`, like the others |
| I10 | Sim virtual BLE op codes | Ad-hoc framing in `vble.rs` and in the conformance suite | Test transport defined in the crate (sim-only, no radio, ADR-0002) |

After I8–I9, the firmware and sim ROS adapters differ only in runtime glue (embassy vs std). Shared handler code goes into a common module where the runtimes allow.

**Transport variants** (contract differences between BLE, UDP and ROS):

| # | Difference | Decision |
| --- | --- | --- |
| T1 | Claim: connect + first write / `HELLO` / first `cmd_vel` from a graph node | Keep: each transport's native session model (ADR-0013, ADR-0016) |
| T2 | Release: disconnect / `BYE` / `/rovi/release` | Keep, same reason |
| T3 | ROS auto-arms on its stream | Keep (ADR-0011) |
| T4 | Sequences: u32 serial (BLE, UDP) vs ROS attachment i64 per publisher | Keep: ROS defines its attachment |
| T5 | No takeover notice on BLE | Keep: BLE is never taken over |
| T6 | ROS Status text carries `session=`; binary Status does not | Keep: ROS has no handshake that returns a session, and bench tools use it |
| T7 | Echo: bytes ≤ 32 (BLE, UDP) vs `Int64` (ROS) | Keep: ROS-native type; one typed `Echo` with per-transport encoding |
| T8 | Mode command codes: BLE Control 3/4, UDP `MODE` 1/2, ROS `UInt8` 1/2 | Keep bytes: renumbering would break the accepted phone/firmware pairing for no behavioural gain. One `Command` enum; the BLE mapping lives in one place |
| T9 | Status/Capabilities/Network on ROS are logfmt text, binary elsewhere | Keep text for ROS tools; generated and decoded by the crate from the same type (D7) |
| T10 | ROS network key `fallback=` names the field the binary layout and code call `reason` | **Align** (protocol change, text only): rename to `reason=`. Only Rovi tools read it |
| T11 | Status, Capabilities and Network delivery: BLE read/notify, UDP `INFO`/`HELLO_ACK` replies and unsolicited `STATUS`, ROS periodic topics | Keep; the conformance suite checks the same observable values on all three |

T10 is the only contract change. Todor approved it on 2026-10-29 (“ok, i approve”, in reply to the T10 rename question); it ships as a recorded vector change. All other transport differences are documented in the crate's module docs and `docs/reference/protocol.md`, generated from one table.

## Alternatives

* **Schema language (protobuf, FlatBuffers, Cap'n Proto) with generated code.** Rejected: existing layouts are fixed BLE/UDP byte formats and ROS CDR; a schema would need custom encodings or a protocol change, plus a generator in the firmware build. Rust types already give a single source.
* **Keep per-language codecs, share only vectors.** Rejected: catches drift late, still four implementations of validation.
* **Rewrite every client in Rust first.** Rejected as the first step: bindings give all participants the shared types now; the native client migration stays separate.
* **Leave wire code inside `rovi-core`.** Rejected: clients would compile device logic, and the contract would not have a reviewable boundary.

## Risks

* Firmware size or stack growth from decoders → decoders are only linked where called; record firmware size before/after.
* Native build friction for Python → extension built by the Nix shell and CI; a missing extension fails loudly at import.
* Phone bundle size and load → measure size and cold start in Bluefy; keep the module in the same revision-pinned build.
* Behaviour drift during extraction → vectors frozen before moving code; firmware and sim conformance must pass unchanged.

## Rollout

1. Freeze vectors from current code; extract the crate; switch `rovi-core`, firmware and sim car. Byte-identical vectors and green sim conformance.
2. Python binding; switch conformance, `apps/client`, teleop and bench scripts. Sim conformance plus teleop tests.
3. Device conformance (motor power off), then announced stand batch for motion rules.
4. WebAssembly binding; switch the phone codec; Node/browser tests, then Bluefy check with motor power off.
5. Delete the hand-written codecs once no caller remains.

Record the decision as [ADR-0017](../../../docs/adr/0017-one-rust-protocol-crate.md).
