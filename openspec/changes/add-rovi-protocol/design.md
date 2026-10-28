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
