# Rovi protocol tasks

## 1. Approval

- [x] 1.1 Todor explicitly accepts the proposal; record the wording, the home-network byte contract it targets, and pass strict validation. The T10 rename (ROS `fallback=` → `reason=`) was approved on 2026-10-29; that approval alone does not authorize implementation. Accepted 2026-10-29: after merging the proposal (#122), Todor was told “To start, say you accept `add-rovi-protocol`” and replied “continue”; recorded as acceptance, open to correction.

## 2. Crate extraction

- [x] 2.1 Freeze golden vectors from current Rust tests and Python/JavaScript fixtures before moving code; add the generator and CI freshness check. Done: `crates/rovi-protocol/vectors.json` (150 valid, 40 malformed), generated and freshness-checked by `crates/rovi-protocol/tests/vectors.rs`; Python (`apps/teleop/tests/test_vectors.py`) and phone (`apps/iphone/tests/vectors.test.mjs`) codecs pass it, with gaps I2 and I3 asserted as known.
- [x] 2.2 Create `crates/rovi-protocol`; move wire types from `rovi-core`; add validating decoders, direction types and property tests; vectors byte-identical. Done 2026-10-29: Setpoint and UDP framing moved; Status, Capabilities, Network information, BLE Control/UUIDs and typed `udp::ToCar`/`FromCar` added with validating binary and ROS-text decoders and named errors (`DecodeError::name`). Every valid vector decodes back to its value and each UDP frame only in its own direction; invalid vectors now record the decoder's error. Bytes unchanged (verified against the pre-move file); one malformed setpoint case was replaced because it hit `unsupported-frame` instead of the frame-id check it was named for. The car still parses UDP with `udp::parse` + `Kind`; moving `device` onto `ToCar` is part of 2.4.
- [x] 2.3 Switch `rovi-core`, firmware (with and without ROS) and sim car; record firmware size before/after; sim conformance and all workspace tests pass unchanged. Done 2026-10-29: `rovi-core`, `rovi-drive`, firmware and sim car import wire types from `rovi-protocol` (no re-exports). Flash image (`espflash save-image`, vs `main` 7fa8f31): plain 738064 → 738272 bytes (+208), ROS 932464 → 931456 (−1008). Workspace 282 tests, motor-bench 14, sim UDP 14 and network 14 scenarios, Python and phone suites, firmware and sim clippy pass.
- [x] 2.4 Remove implementation variants I1–I10 (design D8): one model per concept with per-transport encodings and ROS text decoders; crate UUIDs in the firmware GATT service; one ROS topic table and a network `Publication` used by firmware and sim; typed sim BLE test transport. Add cross-transport equality tests. Done 2026-10-30 for the car side: I7: firmware GATT uses `ble::id` UUIDs and protocol lengths. I8: one ROS endpoint table (`ros::ENDPOINTS`, `rmw` moved into `rovi-protocol::ros`) used by firmware and sim. I9: a `rovi-core` network `Publication` replaces the two hand-written loops. I10: typed `ble::sim` transport used by the sim car. The car decodes UDP as `udp::ToCar`. Behaviour change: a HELLO or INFO with stray payload bytes is now discarded instead of claiming/answering (spec *Input validation before state changes*); an owner's malformed ARM/STOP/MODE still reports invalid. A device test checks Status, network information and Capabilities agree across BLE, UDP INFO and ROS. I1–I6 live in the Python and phone codecs and close when 3.2 and 4.2 replace them with bindings. Flash image vs 2.3: plain 738272 → 740032 (+1760), ROS 931456 → 933424 (+1968). GATT UUID byte order is checked on the host only (the `u128` and string forms agree); a read-only Bluetooth check on the car is pending.
- [ ] 2.5 Rename the ROS network key `fallback=` to `reason=` (T10, approved) with a recorded vector change; update teleop, bench and conformance readers.
- [ ] 2.6 Apply the single client minimum (I11, approved): crate constant used by laptop and phone; first check that no sim scenario or test starts the car with a lease below 500 ms or a claim window below 1000 ms; update teleop tests and README.
- [ ] 2.7 Generate the transport-difference table in `docs/reference/protocol.md` from the crate's list of intentional differences.

## 3. Python binding

- [ ] 3.1 Add the PyO3 extension and its Nix/CI build; pass the vector suite through it.
- [ ] 3.2 Switch conformance, `apps/client`, teleop and bench scripts to the extension; remove hand-written codecs; sim conformance and teleop tests pass.
- [ ] 3.3 Add the `device` conformance target; announce, get Todor's go, run observational rules with motor power off over UDP and BLE; record evidence.
- [ ] 3.4 Announce and get a separate go for the motion rules stand batch; record results.

## 4. Phone binding

- [ ] 4.1 Add the WebAssembly binding with pinned tooling; pass vector parity, Node and built-page tests including missing-module behaviour; measure bundle size.
- [ ] 4.2 Switch the phone codec; with motor power off, check Bluefy load, connect, Status/Network and both network actions; record evidence. Motion recheck only in an announced stand batch.

## 5. Close

- [ ] 5.1 Confirm no hand-written codec callers remain; update `docs/reference/protocol.md`, README commands and ADR-0017 status; strict spec/reader checks pass.
