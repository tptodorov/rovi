# Rovi protocol tasks

## 1. Approval

- [ ] 1.1 Todor explicitly accepts the proposal; record the wording and the home-network byte contract it targets, and pass strict validation.

## 2. Crate extraction

- [ ] 2.1 Freeze golden vectors from current Rust tests and Python/JavaScript fixtures before moving code; add the generator and CI freshness check.
- [ ] 2.2 Create `crates/rovi-protocol`; move wire types from `rovi-core`; add validating decoders, direction types and property tests; vectors byte-identical.
- [ ] 2.3 Switch `rovi-core`, firmware (with and without ROS) and sim car; record firmware size before/after; sim conformance and all workspace tests pass unchanged.

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
