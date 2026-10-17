# Tasks

Host tests establish software behavior only. Physical acceptance uses [ADR-0002](../../../docs/adr/0002-simulation-never-proves-hardware.md). [Design](design.md) defines package boundaries; [verification](verification.md) maps every requirement to its check. Existing experiments remain the rollback reference.

## 1. Accept and extract (laptop)

- [ ] 1.1 Record Todor's explicit acceptance or the merge of this acceptance PR in the proposal; verify the approved scope excludes automatic physical tests.
- [x] 1.2 ROS capabilities selected as product work (2026-10-11); exact keys, QoS and fallback are now specified.
- [ ] 1.3 Extract the M8 core/tests into the reusable Cargo workspace and create separate product simulator/firmware manifests as designed. Verify all carried-over core tests, simulator scenarios and both ESP32 feature builds before behavior changes; preserve experiment builds and document new commands.

## 2. Independent motor control (laptop, test-first)

- [ ] 2.1 Introduce the standalone motor module and recording driver. Reproduce all-low versus documented coast with a failing test before correction; verify per-wheel independence, safe pin ordering, zero/deadband boundaries, clamping and duty caps, including coast under a 70% drive cap.
- [ ] 2.2 Implement configuration validation and latched command faults; test every invalid numeric class, mapping collision, non-finite fourth wheel, update-after-fault and explicit enable with a new vector. Verify the crate builds without device, transport or std dependencies.
- [ ] 2.3 Connect device outputs to motor control through the coordinator. Fault-injection tests must prove stopped/out-of-limits, both standby outputs inactive, no repeated-tick or ROS-stream recovery, and explicit fresh-setpoint/ARM recovery. Document the fault diagnostic and integration API.

## 3. Device and transport conformance (laptop)

- [ ] 3.1 Implement input/profile validation, supported frame handling, boundary timing and initial ROS claim validation. Add failing reproductions for changed behavior, then verify the validation/clock scenarios in the map with deterministic time; retain CDR golden vectors and version-1 byte layouts.
- [ ] 3.2 Implement wrapping BLE/UDP and full per-GID ROS sequence domains, session generations and reconnect cleanup. Test rollover, half-range, repeated HELLO, old-session input, multiple same-node publishers, unknown publishers and router disconnect/reconnect; update adapter docs.
- [ ] 3.3 Preserve explicit/automatic arming and shared-radio policy. Run product UDP, fake-BLE and ROS scenarios covering latch reasons, claim/reclaim boundaries, owner isolation and echo; map actual test names to every applicable verification row.
- [ ] 3.4 Implement canonical capabilities and ROS publication. Verify field/units/round-trip golden tests and a late Jazzy subscriber. Close O9 with evidence of working transient-local history or the specified one-second volatile fallback; document chosen QoS and reconnect behavior.
- [ ] 3.5 Implement shared owner-error persistence and status delivery for BLE/UDP/ROS. Test immediate BLE reads, UDP transition/reply frames, ROS change/periodic publication, none values, release and non-owner isolation; update client/runbook documentation without changing numeric wire codes.
- [ ] 3.6 Add product CI alongside experiment checks. Verify format, clippy, core/motor tests, socket/ROS scenarios, and ESP32 builds with/without ROS; complete the verification map with test names and clearly label remaining physical checks.

## 4. Qualify the physical profile (bench, separate go for each run)

- [ ] 4.1 Record O1/O4/O7 qualified limits and pin-level disable timing for the selected profile. Distinguish configured 100% from 70% bench evidence; acceptance requires the recorded bounds and corresponding physical results, not inferred current protection.
- [ ] 4.2 Review O2 profile suitability and O5 coast-distance qualification using accepted approximate references. Record any remaining physical bounds explicitly; no extended calibration sweep is authorized by this task.
- [x] 4.3 Q11/O3 is settled (2026-10-17): open-loop duty, no added sensors. O6 deadband and O8 mapping are recorded in the requirements.

## 5. Product hardware acceptance

- [ ] 5.1 Bind the product motor driver to all four TB6612 channels and the qualified profile. Cross-build both firmware variants and check pin mapping against the carrier schematic before proposing a flash.
- [ ] 5.2 After separately announced/authorized board runs, record physical scenario results for outputs, stop triggers, radio operation and selected-profile reversal/thermal behavior across BLE/UDP/ROS. Use the verification map to distinguish software-only scenarios from physical claims.
- [ ] 5.3 Verify each requirement has its specified evidence, all blocking open items are resolved, and product acceptance is recorded; update PLAN and product instructions.

## Workflow follow-up

* After acceptance, run archive and validation per [the specification standard](../../../docs/SPECIFICATION.md). Review resulting baseline purposes/scenarios and repair relative links in both baseline and archived documents. Archive is not an implementation task checkbox.
