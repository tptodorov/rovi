# Rovi protocol tasks

## 1. Approval

- [x] 1.1 Todor explicitly accepts the proposal; record the wording, the home-network byte contract it targets, and pass strict validation. The T10 rename (ROS `fallback=` → `reason=`) was approved on 2026-10-29; that approval alone does not authorize implementation. Accepted 2026-10-29: after merging the proposal (#122), Todor was told “To start, say you accept `add-rovi-protocol`” and replied “continue”; recorded as acceptance, open to correction.

## 2. Crate extraction

- [x] 2.1 Freeze golden vectors from current Rust tests and Python/JavaScript fixtures before moving code; add the generator and CI freshness check. Done: `crates/rovi-core/tests/protocol-vectors.json` (150 valid, 40 malformed), generated and freshness-checked by `crates/rovi-core/tests/protocol_vectors.rs`; Python (`apps/teleop/tests/test_vectors.py`) and phone (`apps/iphone/tests/vectors.test.mjs`) codecs pass it, with gaps I2 and I3 asserted as known. Moves to `crates/rovi-protocol` in 2.2.
- [ ] 2.2 Create `crates/rovi-protocol`; move wire types from `rovi-core`; add validating decoders, direction types and property tests; vectors byte-identical.
- [ ] 2.3 Switch `rovi-core`, firmware (with and without ROS) and sim car; record firmware size before/after; sim conformance and all workspace tests pass unchanged.
- [ ] 2.4 Remove implementation variants I1–I10 (design D8): one model per concept with per-transport encodings and ROS text decoders; crate UUIDs in the firmware GATT service; one ROS topic table and a network `Publication` used by firmware and sim; typed sim BLE test transport. Add cross-transport equality tests.
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
