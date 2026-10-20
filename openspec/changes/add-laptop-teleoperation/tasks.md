# Tasks

Todor selected this laptop scope on 2026-10-20. Nothing here authorizes a flash or motor run. [Verification map](verification.md).

## 1. Choose and accept the next scope

- [x] 1.1 Laptop scope accepted 2026-10-20: Todor selected “Laptop teleoperation client” after #89 presented the concrete option. Contract review preserves the existing car API and defines explicit arm, gentle limits, input expiry and failure cleanup. Strict OpenSpec/reader validation passes; no firmware or motor-run authorization.

## 2. Codec and client boundaries

- [ ] 2.1 Extract the existing Python payload codec into one shared location with a compatibility import at its current path. Run the existing 23 codec/log/latency tests, bench tests and UDP scenarios unchanged; document the shared import and confirm no runtime dependency is added.
- [ ] 2.2 Implement framed UDP parsing, capability validation, session/peer/request correlation and bounded handshake. Add tests for malformed/truncated payloads, BUSY, unsupported leases, wrong sources/sessions, unsolicited Status and request-sequence wrap; verify no input in these cases can arm or emit nonzero output.

## 3. Interactive motion policy

- [ ] 3.1 Implement the pure clock-driven intent/arming/stop policy test-first. Verify all six signs, smaller/zero limits, explicit R at zero, no queued motion after arm/STOP, exact 250 ms expiry, key repeats and the 200 ms scheduler-gap failure. Document input pulse/renewal semantics with the key map.
- [ ] 3.2 Add the Linux terminal/event loop, periodic zeros/setpoints and independent Echo/control deadlines. Use a pseudo-terminal and fake UDP peer to verify missing replies, socket/terminal loss, queued stopped-before-Armed replies and no HELLO/ARM retry. Verify cleanup deadline and terminal restoration even when individual sends fail.
- [ ] 3.3 Present local state, last-reported Status/reply age, session, limits and requested motion. Test output so Echo does not refresh Status age and neither measured speed nor confirmed stop/release is invented. Add a runnable command/help page.

## 4. Laptop integration and CI

- [ ] 4.1 Exercise the real client/PTY against the product simulator: startup idle, explicit arm, six directions, input expiry, Space latch/re-arm, quit and killed/lost peer. Confirm zero outputs before arm and after expiry/fault, one claim only, and no automatic recovery. Add client tests to existing CI without removing M6 checks.
- [ ] 4.2 Run relevant Python suites, UDP scenarios, strict OpenSpec/reader checks and authored-file whitespace checks. Replace every planned verification row with actual test/evidence names; review the user command against the README.

## 5. Separately authorized board acceptance

- [ ] 5.1 Prepare a concrete gentle stand sequence and await Todor’s separate go. Test the normal installed firmware without flashing: startup still, arm at zero, six short directions, input expiry, Space, explicit re-arm and quit; verify protocol/serial assertions and Todor’s wheel observations separately. Restore any changed laptop network/capture state. No board run belongs to task 4.
- [ ] 5.2 Reconcile product evidence with the requirements, record scoped acceptance and update PLAN/usage documentation. Preserve M6 measurement deferrals and do not claim loaded/floor performance or measured stop bounds.

## Workflow follow-up

* After implementation and scoped acceptance, archive/sync through the pinned CLI per [SPECIFICATION](../../../docs/SPECIFICATION.md); preserve evidence and repair relocated links.
