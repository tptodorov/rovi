# M6: low-level four-wheel API

**Status:** archived/synced 2026-10-20 after [#87](https://github.com/tptodorov/rovi/pull/87) merged in `f5a2d8e`; proposed 2026-10-10; laptop implementation scope accepted 2026-10-17 by Todor's merge of [#48](https://github.com/tptodorov/rovi/pull/48). Open items O1–O8 closed or deferred 2026-10-17. Scoped product acceptance is recorded on 2026-10-19 in the [task 5.3 review](verification.md#acceptance-review-task-53), using Todor’s physical confirmations and explicit task 5.2 measurement deferrals. The product crates, sim car and firmware are implemented; laptop conformance and CI are tracked in tasks 1–3. Existing experiments remain the comparison/rollback reference; unverified physical limits below remain explicit. Terms are as defined in [`GLOSSARY.md`](../../../../GLOSSARY.md).

## Why

[VISION](../../../../docs/VISION.md) layer 2 needs a product API that drives the car's four wheels independently, stays independent of transport and mecanum math, and fails safe. M8 ([README](../../../../experiments/rovi-m8-setpoint-ros/README.md)) proved the transport and ownership model on the board. M4 added four physical motor outputs and verified stops, reversals and a thermal run; M5 demonstrated floor movement and recorded approximate motion references. This change turns that experimental behavior into a reusable product API, separating motor control from the device state machine. It reuses the working code and evidence rather than starting another firmware experiment.

## What Changes

* **`motor-control` (new):** the reusable layer that takes four wheel commands (FL, FR, RR, RL, normalised −1..1) and drives the motor drivers. It knows nothing of setpoints, kinematics or transports. It owns driver standby, the boot-safe state and the limits M4 measures.
* **`car-control-api` (new):** the car's device control API, extracted from M8 with its explicit transport differences (including ROS auto-arm). This proposal corrects validation, ordering, fault recovery and status gaps described in the design; it does not claim behavior-identical extraction.
* **Firmware integration follows the accepted laptop implementation tasks; flashing and motor runs require separate authorization.** BLE/UDP byte layouts and numeric codes stay as M8 defines them ([README](../../../../experiments/rovi-m8-setpoint-ros/README.md#transports)); this spec states behaviour, not wire formats.

## Capabilities

### New Capabilities

* `motor-control`: independent wheel commands, driver standby, duty limits and fault handling.
* `car-control-api`: setpoint processing, ownership, arming, lease, capabilities and status.

### Modified Capabilities

None; this is the first product specification change.

## Design decisions

* **The motor control boundary is normalised wheel commands, not SI wheel speeds.** M5 confirmed no encoders; the current car uses open-loop duty control (Q11/O3, 2026-10-17). A normalised command survives both: 1.0 means the configured maximum, whether that is a duty or a wheel speed. The device layer still converts setpoints to SI wheel speeds first, as `mecanum_drive_controller` does ([ADR-0006](../../../../docs/adr/0006-twiststamped-cdr-setpoint.md)).
* **Controllers cannot send wheel commands directly.** The device control API accepts setpoints only. Exposing four wheel velocities stays deferred, as ADR-0006 records; motor control is a firmware-internal boundary.
* **ROS 2 controllers read capabilities before claiming.** A ROS 2 controller claims with its first setpoint, so it needs the limits before it claims. The car publishes `/rovi/capabilities` (`std_msgs/String`, `key=value`) without a claim. O9 selected the specified reliable/volatile/depth-1 fallback: changes plus a 900 ms heartbeat; retained history was unavailable with the pinned publisher. Rejected: a custom message type (every controller must build a Rovi interface package) and ROS 2 parameters (they need services, which the car does not serve).
* **Safety is enforced twice.** The device layer raises safety stops ([ADR-0004](../../../../docs/adr/0004-single-owner-latest-setpoint.md)); motor control independently holds standby until it is explicitly enabled and on any invalid wheel command.

Implementation boundaries, migration and the reviewed contract changes are in [design.md](design.md). Existing ADRs remain authoritative; no new irreversible architectural decision is introduced.

## Evidence and remaining decisions

Use [M4 results](../../../../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results), [M5 results](../../../../experiments/rovi-m5-mecanum-motion/MILESTONE.md#results) and the [accepted motion references](../../../../experiments/rovi-m5-mecanum-motion/CALIBRATION.md). Todor stopped the extended measurement sweep; no new motor tests are queued by this proposal. Approximate references are not guaranteed velocity tracking.

| ID | Status | Answer |
| --- | --- | --- |
| O1 | **Closed** 2026-10-17 | Duty limits stay configurable. The product profile caps drive duty at **70%**, the envelope M4 verified on the bench (ten ±70% reversals, 5-minute thermal run, radio up). The experiment firmware's 100% is not a product value. **Deferred, needs a current monitor (INA226) or an oscilloscope with a shunt:** start and stall current. Needed only to raise the cap or to claim current protection; this change claims neither. |
| O2 | **Closed** 2026-10-17 | Measured geometry: radius 0.030 m, wheelbase 0.115 m, track 0.140 m, centre projection sum 0.1275 m. Open-loop with the accepted approximate motion references; no calibration sweep. The 40 rad/s normalization and 1/1/3 input limits stay in the profile, labelled experimental, not measured maxima. The references were recorded on the experiment's 40–100% mapping; under the 70% cap the same command gives less duty (`vx 0.20`: 45% instead of 50%). Maximum physical speeds are not claimed. |
| O3 | **Closed** | Open-loop duty control; no encoders, added sensing out of scope. |
| O4 | **Deferred, needs a logic analyzer or oscilloscope** | A numeric pin-level STBY disable bound. This change specifies and host-tests that a safety stop deasserts both STBY outputs in the control tick that processes it, before any further bridge update. M4 observed all four wheels stop on every trigger. No numeric bound is claimed. |
| O5 | **Closed** 2026-10-17 | Coast for an individual zero wheel (TB6612 IN1=IN2=low, PWM=high), standby for every safety stop. The experiment's all-low zero pins do not prove coast; task 2.1 corrects them test-first. Coast distance is not measured or claimed (tape and video, if a later requirement needs it). |
| O6 | **Closed** | Exact zero and magnitudes below 0.02 coast; other commands map linearly from 40% to the configured maximum. About 35% starting duty observed on the stand; no per-wheel threshold is claimed. |
| O7 | **Closed** 2026-10-17 | Direct reversals, no ramp, up to the 70% cap: M4's ten ±70% reversals caused no ESP32 reset or BLE loss. Raising the cap reopens O7 together with O1. |
| O8 | **Closed** | D1 A→FL, D1 B→RL, D2 A→FR, D2 B→RR; directions corrected at connectors, all software inversions false, reconfirmed after the motor rework. Each driver has its own STBY, both disabled on a safety stop. |
| O9 | **Closed** 2026-10-17 | The pinned ordinary publisher did not serve history to a late Jazzy transient-local subscriber (no sample within 3 s; its later live sample arrived). Selected the specified reliable/volatile/depth-1 fallback: every 900 ms (headroom under one second), on claim/release and reconnect. Late subscribers received in 0.413/0.551 s before/after router restart; [probe and fallback evidence](../../../../apps/simcar/scenarios/evidence/2026-10-17-capabilities/README.md). |

On 2026-10-19 Todor closed task 5.2 with the remaining equipment-dependent checks skipped because suitable measurement tools are unavailable. Coast/STBY pin levels, electrical duty/deadband measurements and the coast/driven-neighbour check remain physically unverified; they return in a later change with suitable tools. Completed board results are retained in the [verification map](verification.md#task-52-disposition); task 5.3 records [acceptance within that scope](verification.md#acceptance-review-task-53). Specified motor behavior is unchanged.

No unresolved product decisions remain. Deferred items are outside this change: it does not claim measured current, a numeric disable bound, coast distance, maximum speeds or operation above 70%. Equipment-gated items return as a new change when the equipment is available. Retain the experiment firmware for comparison; product acceptance is a separate checklist.

## First implementation scope

* Reuse M8's core, motor mapping and tests in a shared Cargo workspace; isolate motor control from the arbiter and transports.
* Add conformance tests for the specified deadband/duty rule, independent wheels, faults, standby and documented zero-output mode.
* Keep BLE, UDP, optional ROS and the accepted ownership/arming policies. Apply the reviewed corrections and implement the agreed ROS capabilities and extended status topics.
* No new encoder, camera, phone-app, station-mode or hardware-monitor work in this change.
* Do not flash or start a motor test as part of extraction or host validation. Each board run is announced separately and waits for Todor's go.

## Impact

* Planning: `openspec/changes/archive/2026-10-20-add-four-wheel-api/`, following the [product specification standard](../../../../docs/SPECIFICATION.md) and [ADR-0015](../../../../docs/adr/0015-native-openspec-workflow.md).
* Code: `crates/rovi-core`, `crates/rovi-motor-control` and `crates/rovi-drive` in the root workspace, with separate `apps/simcar` and `firmware` packages/lockfiles. The [tasks](tasks.md) and [verification map](verification.md) record software checks, confirmed board results and explicit physical deferrals.
* Evidence: host tests and the sim car are development evidence only ([ADR-0002](../../../../docs/adr/0002-simulation-never-proves-hardware.md)). The scoped acceptance review uses the product board runs linked in the verification map, preserving M4/M5 as references.

## Review status

The 2026-10-17 review findings are addressed: ROS latch exceptions, complete motor fault/configuration recovery, clamped kinematics, shared Wi-Fi exclusion, explicit transport edge cases and ROS field/QoS contracts. The design and [verification map](verification.md) accompany the requirements. Todor merged #48 on 2026-10-17, accepting the laptop implementation tasks (task 1.1). That does not establish product hardware acceptance or authorize a flash/motor run. The same day Todor agreed O1 and O2; O5 and O7 closed with them, and O4 and O1's current measurement are deferred as equipment-gated. O9 closed with the specified volatile fallback and recorded Jazzy evidence.

The 2026-10-19 [final acceptance review](verification.md#acceptance-review-task-53) completes task 5.3 for the recorded scope. It preserves the equipment-dependent checks Todor skipped as physically unverified; no new qualification claims or motor authorization follow from acceptance. The pinned CLI archived/synced this change on 2026-10-20 after the stack landed, creating the [car control API](../../../specs/car-control-api/spec.md) and [motor control](../../../specs/motor-control/spec.md) baselines without changing requirement behavior. Measurements explicitly deferred at acceptance remain unverified.
