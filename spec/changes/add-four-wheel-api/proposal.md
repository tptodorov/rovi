# M6: low-level four-wheel API

**Status:** proposed 2026-10-10. Requirements marked **OPEN** wait for bench evidence; nothing here is implemented beyond M8's board-only core. Terms are as defined in [`GLOSSARY.md`](../../../GLOSSARY.md).

## Why

[VISION](../../../docs/VISION.md) layer 2 needs a product API that drives the car's four wheels independently, stays independent of transport and mecanum math, and fails safe. M8 ([README](../../../experiments/rovi-m8-setpoint-ros/README.md)) proved the control model on the laptop: `TwistStamped` setpoints, one owner, explicit arming, lease, claim and reclaim windows, safety stop. M8 stops at the `WheelOutput` boundary with no motors. This change turns that model into product requirements and adds the motor control layer below it, so implementation can start once M4 and M5 supply the hardware numbers.

## What changes

* **`motor-control` (new):** the reusable layer that takes four wheel commands (FL, FR, RR, RL, normalised −1..1) and drives the motor drivers. It knows nothing of setpoints, kinematics or transports. It owns driver standby, the boot-safe state and the limits M4 measures.
* **`car-control-api` (new):** the car's device control API, carried over from M8 unchanged in behaviour: setpoint, ownership, arming, lease, windows, safety stop, capabilities and status, identical on BLE, UDP and ROS 2.
* **No firmware changes** in this proposal. Byte layouts stay as M8 defines them ([README](../../../experiments/rovi-m8-setpoint-ros/README.md#transports)); this spec states behaviour, not wire formats.

## Design decisions

* **The motor control boundary is normalised wheel commands, not SI wheel speeds.** M5 confirmed no encoders; the current car uses open-loop duty control (Q11/O3, 2026-10-17). A normalised command survives both: 1.0 means the configured maximum, whether that is a duty or a wheel speed. The device layer still converts setpoints to SI wheel speeds first, as `mecanum_drive_controller` does ([ADR-0006](../../../docs/adr/0006-twiststamped-cdr-setpoint.md)).
* **Controllers cannot send wheel commands directly.** The device control API accepts setpoints only. Exposing four wheel velocities stays deferred, as ADR-0006 records; motor control is a firmware-internal boundary.
* **ROS 2 controllers read capabilities from a latched topic.** A ROS 2 controller claims with its first setpoint, so it needs the limits before it claims. The car publishes `/rovi/capabilities` (`std_msgs/String`, `key=value`, transient local), readable without a claim; O9 decides whether zenoh-nostd can serve it as latched. Rejected: a custom message type (every controller must build a Rovi interface package) and ROS 2 parameters (they need services, which the car does not serve).
* **Safety is enforced twice.** The device layer raises safety stops ([ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md)); motor control independently holds standby until it is explicitly enabled and on any invalid wheel command.

No new ADR: none of these is hard to reverse before implementation starts.

## Open questions

Each **OPEN** requirement names its question here. O1–O8 are hardware numbers; O9 is a software question. No numbers are invented; M8's defaults (`max_wheel_speed` 40 rad/s, `max_vx`/`max_vy` 1 m/s, `max_wz` 3 rad/s) are placeholders, not requirements.

| ID | Question | Settled by |
| --- | --- | --- |
| O1 | Per-channel and combined motor current limits, and the maximum duty that wheel command 1.0 may map to | M4 |
| O2 | Maximum wheel speed (`max_wheel_speed`) and the resulting `max_vx`, `max_vy`, `max_wz` | M5 |
| O4 | Bound from safety stop to every driver in standby, under four-motor load | M4 |
| O5 | Zero wheel command: short brake or coast, and stopping distance at maximum speed | M4, M5 |
| O6 | Dead zone: the smallest wheel command that turns each wheel, and whether to compensate for it | M5 |
| O7 | Whether starts and reversals need a slew limit to keep the logic rail above brownout | M4 |
| O8 | Driver channel to wheel mapping and per-wheel direction | M5 |
| O9 | Can zenoh-nostd serve `rmw_zenoh`'s transient-local late joiners for `/rovi/capabilities`, or must the car republish it | M6 implementation (laptop, then board) |

**Resolved O3 (M5, 2026-10-17):** open-loop duty control; motors have no encoders and adding sensors is out of scope. Todor accepts [existing approximate motion references](../../../experiments/rovi-m5-mecanum-motion/CALIBRATION.md); these do not guarantee accurate velocity tracking or settle O2's full-duty maximum.

## Impact

* New: `spec/changes/add-four-wheel-api/`. First use of `spec/` ([ADR-0001](../../../docs/adr/0001-openspec-for-product-development.md)).
* Code: none now. Implementation (see [tasks](tasks.md)) reuses M8's core and adds a motor control module; it starts the shared Cargo workspace PLAN defers until this point.
* Evidence: host tests and the sim car are development evidence only ([ADR-0002](../../../docs/adr/0002-simulation-never-proves-hardware.md)). Acceptance needs the M4/M5 bench setup.
