# M6: low-level four-wheel API

**Status:** proposed 2026-10-10; review corrections and implementation design completed 2026-10-17; proposed for acceptance of host implementation. Qualified product hardware acceptance remains pending. The product workspace is not implemented. Existing experiment code is the starting point; unverified limits below remain explicit. Terms are as defined in [`GLOSSARY.md`](../../../GLOSSARY.md).

## Why

[VISION](../../../docs/VISION.md) layer 2 needs a product API that drives the car's four wheels independently, stays independent of transport and mecanum math, and fails safe. M8 ([README](../../../experiments/rovi-m8-setpoint-ros/README.md)) proved the transport and ownership model on the board. M4 added four physical motor outputs and verified stops, reversals and a thermal run; M5 demonstrated floor movement and recorded approximate motion references. This change turns that experimental behavior into a reusable product API, separating motor control from the device state machine. It reuses the working code and evidence rather than starting another firmware experiment.

## What Changes

* **`motor-control` (new):** the reusable layer that takes four wheel commands (FL, FR, RR, RL, normalised −1..1) and drives the motor drivers. It knows nothing of setpoints, kinematics or transports. It owns driver standby, the boot-safe state and the limits M4 measures.
* **`car-control-api` (new):** the car's device control API, extracted from M8 with its explicit transport differences (including ROS auto-arm). This proposal corrects validation, ordering, fault recovery and status gaps described in the design; it does not claim behavior-identical extraction.
* **No firmware changes** in this proposal. BLE/UDP byte layouts and numeric codes stay as M8 defines them ([README](../../../experiments/rovi-m8-setpoint-ros/README.md#transports)); this spec states behaviour, not wire formats.

## Capabilities

### New Capabilities

* `motor-control`: independent wheel commands, driver standby, duty limits and fault handling.
* `car-control-api`: setpoint processing, ownership, arming, lease, capabilities and status.

### Modified Capabilities

None; this is the first product specification change.

## Design decisions

* **The motor control boundary is normalised wheel commands, not SI wheel speeds.** M5 confirmed no encoders; the current car uses open-loop duty control (Q11/O3, 2026-10-17). A normalised command survives both: 1.0 means the configured maximum, whether that is a duty or a wheel speed. The device layer still converts setpoints to SI wheel speeds first, as `mecanum_drive_controller` does ([ADR-0006](../../../docs/adr/0006-twiststamped-cdr-setpoint.md)).
* **Controllers cannot send wheel commands directly.** The device control API accepts setpoints only. Exposing four wheel velocities stays deferred, as ADR-0006 records; motor control is a firmware-internal boundary.
* **ROS 2 controllers read capabilities from a latched topic.** A ROS 2 controller claims with its first setpoint, so it needs the limits before it claims. The car publishes `/rovi/capabilities` (`std_msgs/String`, `key=value`, transient local), readable without a claim; O9 decides whether zenoh-nostd can serve it as latched. Rejected: a custom message type (every controller must build a Rovi interface package) and ROS 2 parameters (they need services, which the car does not serve).
* **Safety is enforced twice.** The device layer raises safety stops ([ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md)); motor control independently holds standby until it is explicitly enabled and on any invalid wheel command.

Implementation boundaries, migration and the reviewed contract changes are in [design.md](design.md). Existing ADRs remain authoritative; no new irreversible architectural decision is introduced.

## Evidence and remaining decisions

Use [M4 results](../../../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results), [M5 results](../../../experiments/rovi-m5-mecanum-motion/MILESTONE.md#results) and the [accepted motion references](../../../experiments/rovi-m5-mecanum-motion/CALIBRATION.md). Todor stopped the extended measurement sweep; no new motor tests are queued by this proposal. Approximate references are not guaranteed velocity tracking.

| ID | Current answer / remaining limit |
| --- | --- |
| O1 | Existing mapping is 40–100% PWM, 20 kHz; running current reported as 150 mA per motor. Four-motor reversal and thermal evidence covers 70% duty, not continuous full-scale 100% operation. Start/stall current and a qualified electrical maximum remain unknown. Implement configurable duty limits; do not describe them as measured current protection. |
| O2 | Geometry is radius 0.030 m, wheelbase 0.115 m, track 0.140 m, centre projection sum 0.1275 m. Use existing approximate motion references. Experiment normalization 40 rad/s and input limits 1/1/3 are operational placeholders, not measured maximum speeds. Profile values must stay explicit; no single gain fitted from short pulses is justified. |
| O3 | Resolved: open-loop duty control; no encoders, added sensing out of scope. |
| O4 | Physical stopping is observed; a numeric pin-level disable bound is not measured. Keep driver-disable validation separate from lease detection and mechanical stopping. Implement immediate standby when the motor layer receives disable; no invented physical timing claim. |
| O5 | Select coast for an individual zero wheel and standby for every safety stop. Encode the TB6612 documented coast pin state explicitly; the experimental all-low zero pins must not be assumed to prove coast. Whole-run STOP/lease travel is 6.5/22.5 cm at one low command; maximum-speed coast distance remains unknown. |
| O6 | Resolved rule: exact zero and magnitudes below 0.02 do not drive; other commands map linearly from 40% toward configured maximum duty. About 35% starting duty observed on the stand. No precise per-wheel minimum steady speed or floor threshold is claimed. |
| O7 | Direct reversals at ±70% did not reset the ESP32 or drop BLE; retain direct transitions within the qualified operating envelope, with no speculative ramping. Behavior beyond that evidence remains to be qualified before claiming full-scale reversal. |
| O8 | Resolved: D1 A→FL, D1 B→RL, D2 A→FR, D2 B→RR. Directions corrected at connectors, all software inversions false. Each driver has its own STBY, both disabled on a safety stop. |
| O9 | Software question: can zenoh-nostd serve transient-local late joiners for `/rovi/capabilities`? Test on the laptop; if unsupported, use reliable/volatile depth 1 and republish every second and on claim/release/reconnect, with compatible subscriber QoS. This blocks choosing/documenting the ROS publication mode, not other host work. |

The remaining electrical/physical bounds do not prevent implementing and host-testing configurable modules. They do prevent claiming qualified maxima, exact velocity tracking or completed product hardware acceptance. Retain the experiment firmware for comparison; product acceptance is a separate checklist.

## First implementation scope

* Reuse M8's core, motor mapping and tests in a shared Cargo workspace; isolate motor control from the arbiter and transports.
* Add conformance tests for the specified deadband/duty rule, independent wheels, faults, standby and documented zero-output mode.
* Keep BLE, UDP, optional ROS and the accepted ownership/arming policies. Apply the reviewed corrections and implement the agreed ROS capabilities and extended status topics.
* No new encoder, camera, phone-app, station-mode or hardware-monitor work in this change.
* Do not flash or start a motor test as part of extraction or host validation. Each board run is announced separately and waits for Todor's go.

## Impact

* Planning: `openspec/changes/add-four-wheel-api/`, following the [product specification standard](../../../docs/SPECIFICATION.md) and [ADR-0015](../../../docs/adr/0015-native-openspec-workflow.md).
* Code: none now. Implementation (see [tasks](tasks.md)) extracts M8's existing core and motor output behavior into product modules; it starts the shared Cargo workspace PLAN defers until this point.
* Evidence: host tests and the sim car are development evidence only ([ADR-0002](../../../docs/adr/0002-simulation-never-proves-hardware.md)). Acceptance needs the M4/M5 bench setup.

## Review status

The 2026-10-17 review findings are addressed: ROS latch exceptions, complete motor fault/configuration recovery, clamped kinematics, shared Wi-Fi exclusion, explicit transport edge cases and ROS field/QoS contracts. The design and [verification map](verification.md) accompany the requirements. This PR proposes acceptance for the laptop implementation tasks; merging it explicitly accepts that scope and allows task 1.1 to be recorded as accepted. It does not accept the unimplemented product or authorize a flash/motor run. O1/O2/O4/O5/O7 physical qualification remains open and blocks final product acceptance; O9 has a specified laptop test and fallback.
