# Milestones

## M5: mecanum motion and feedback characterization

**Status:** planned (2026-10-17), next after M4. Bench procedure: [TESTPLAN.md](TESTPLAN.md).
**Setup:** M4's car and carrier with the wheels mounted, untethered: ESP32 and motors on their own power banks, controlled over BLE from the laptop. Measured with tape and phone video (no car log off the cable).

### Goal

Measure whether the received chassis and motor setup can provide useful open-loop mecanum teleoperation, map each driver channel to its wheel and direction, and establish whether the available hardware supports closed-loop wheel control.

### Risks covered

* Motor-to-wheel mapping, direction, mecanum roller orientation, and chassis assembly assumptions.
* Whether normalized PWM/duty commands are adequate or wheel speed feedback is required.
* Wheel-to-wheel calibration and variation under changing load/battery voltage.
* Whether encoders are present; do not assume them from the listing.

### Known from M4 (2026-10-17)

* Motor-to-channel mapping and direction confirmed; the motor layer runs 40-100 % duty (start threshold 35 % with the wheels off the ground).
* **No encoders:** the TT motors have two wires only. M5 is open loop; closed-loop speed would need added sensors (out of scope now).

### Test plan

1. Inspect the assembled chassis and motors. Record motor-to-wheel mapping, wheel diameter, roller orientation, total mass, and whether each motor has an encoder or other speed sensor.
2. With the car restrained and then on a clear low-friction test surface, command each wheel independently at safe low duty. Confirm direction, stop, and mapping.
3. Apply the proposed mecanum combinations for forward/reverse, lateral, and rotation. Record command-to-motion results, wheel slip, dead zones, and any chassis asymmetry.
4. Repeat at multiple battery states and representative payloads. Measure actual wheel speed or distance only with available instrumentation; otherwise record the open-loop observations and their limitations.
5. If encoders exist, compare measured wheel speed against commanded targets at several loads and decide whether closed-loop velocity is feasible. If none exist, document open-loop limitations and do not add sensing speculatively.

### Acceptance

Real hardware only; reuse the verified M4 setup and stay within its recorded electrical limits.

- [x] Motor, driver, wheel, and direction mapping is documented. *(M4 step 2; roller orientation checked in F1.)*
- [ ] Four basic mecanum movement combinations and stop are demonstrated at safe speed.
- [ ] Repeatability and calibration variation are recorded across battery/load conditions.
- [ ] Encoder availability is confirmed; the open-loop versus closed-loop API decision has evidence and limitations. *(No encoders: confirmed in M4. The open-loop evidence comes from F3-F8.)*
- [ ] The API/spec can state units, bounds, stop behavior, update timing, and calibration needs without hidden assumptions.

### Results

#### Car measurements, 2026-10-17

Reported by Todor before the floor tests:

| Measurement | Value |
| --- | --- |
| Wheel outer diameter | 60 mm |
| Wheelbase (front to rear axle centres) | 115 mm |
| Track (left to right wheel centres) | 140 mm |
| Footprint, length × width | 255 × 170 mm |
| Mass with both power banks | 500 g |

Derived geometry: `wheels_radius = 0.030 m`; `center_projection_sum = (0.115 + 0.140) / 2 = 0.1275 m`.

These values are recorded here; firmware geometry has not yet been updated. Wheel speed, floor speed, drift and stopping distance remain unmeasured. The roller photo mentioned in chat was not available in the handoff, so the roller orientation is not yet verified. No M5 motor runs have been performed.

Record firmware revision, floor surface, battery state, observations and evidence for each subsequent run.
