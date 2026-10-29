# Shared client tasks

## 1. Approval and contract

- [ ] 1.1 Todor explicitly accepts the proposal and current home-network integration contract; record the wording, keep range unverified, and pass strict validation. Drafting authorization is not implementation authorization.
- [ ] 1.2 Map every new requirement/scenario to existing client tests or a new trace/physical check; record exact profile differences and migration/rollback commands before code changes.

## 2. Shared protocol and session module

- [ ] 2.1 Build on accepted `rovi-protocol` messages and vectors; add client-side correlation and u32-wrap handling with tests; document the library's protocol interface.
- [ ] 2.2 Implement profile-based session decisions test-first; verify fresh zero/ARM, STOP races, old callbacks, expiration, deadline equality, cleanup failures and network refusal using deterministic traces; document host action/result obligations.
- [ ] 2.3 Build the library natively and for WebAssembly without OS/transport dependencies; verify equivalent decisions for identical traces on both targets and measure build/bundle overhead.

## 3. Native laptop migration

- [ ] 3.1 Add native terminal/UDP adapters and user documentation; pass simulator, fake-clock/peer and PTY parity checks, including key batching, beep, retry, terminal restoration and `--mode`/network behavior.
- [ ] 3.2 Announce and obtain Todor's go for the native UDP stand batch; record motion, STOP/loss, disarmed recovery, takeover and mode results before promoting that replacement. Keep the accepted Python client available on failure.
- [ ] 3.3 Qualify a native BLE backend for discovery, properties, payload capacity and fresh operations; implement the adapter with fake-GATT/error tests and documented dependencies. Block promotion if the backend cannot satisfy the existing contract.
- [ ] 3.4 Announce and obtain a separate go for the native BLE stand/recovery batch; record measured results and operator confirmation before promotion, retaining a documented Python rollback.

## 4. Phone migration

- [ ] 4.1 Add the locally bundled WebAssembly binding and connect it to existing input/transport/UI adapters; pass Node, built-page, missing-module and revision-cache checks, plus native/WebAssembly trace parity. Document loading failure and rollback.
- [ ] 4.2 With motor power off, verify Bluefy module loading, connect, refusal, lifecycle/manual recovery, both network actions and fresh reconnect information; record phone/browser/build and serial evidence separately from host tests.
- [ ] 4.3 Announce and obtain Todor's go for the migrated phone stand batch; recheck Buttons/Physical directions, neutral/START/PAUSE, STOP and interruptions before promoting the release. Record any explicit scoped deferrals without claiming their measurements.

## 5. Integration and retirement

- [ ] 5.1 Check all old implementation callers before retirement; preserve independent fixtures and needed bench helpers, update entry points/docs and verify CLI, sim, browser, firmware build and strict spec/reader CI checks.
- [ ] 5.2 Review requirement-to-evidence coverage and per-client rollback/promotion records with Todor; mark accepted only when every required migration gate passes. Physical range fallback remains outside this change's acceptance claims.

## Workflow follow-up

- Archive/sync only after accepted implementation and migration checks; close #108 then.
- The home-network change retains its own acceptance and archive gates.
