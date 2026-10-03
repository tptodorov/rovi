# Milestones

## M5: mecanum motion and feedback characterization

**Status:** proposed — not yet implemented or bench-tested
**Setup count:** reuse M4's integrated four-motor chassis setup; no additional electrical setup

### Goal

Measure whether the received chassis and motor setup can provide useful open-loop mecanum teleoperation, map each driver channel to its wheel and direction, and establish whether the available hardware supports closed-loop wheel control.

### Risks covered

* Motor-to-wheel mapping, direction, mecanum roller orientation, and chassis assembly assumptions.
* Whether normalized PWM/duty commands are adequate or wheel speed feedback is required.
* Wheel-to-wheel calibration and variation under changing load/battery voltage.
* Whether encoders are present; do not assume them from the listing.

### Test plan

1. Inspect the assembled chassis and motors. Record motor-to-wheel mapping, wheel diameter, roller orientation, total mass, and whether each motor has an encoder or other speed sensor.
2. With the car restrained and then on a clear low-friction test surface, command each wheel independently at safe low duty. Confirm direction, stop, and mapping.
3. Apply the proposed mecanum combinations for forward/reverse, lateral, and rotation. Record command-to-motion results, wheel slip, dead zones, and any chassis asymmetry.
4. Repeat at multiple battery states and representative payloads. Measure actual wheel speed or distance only with available instrumentation; otherwise record the open-loop observations and their limitations.
5. If encoders exist, compare measured wheel speed against commanded targets at several loads and decide whether closed-loop velocity is feasible. If none exist, document open-loop limitations and do not add sensing speculatively.

### Acceptance

Real hardware only; reuse the verified M4 setup and stay within its recorded electrical limits.

- [ ] Motor, driver, wheel, and direction mapping is documented.
- [ ] Four basic mecanum movement combinations and stop are demonstrated at safe speed.
- [ ] Repeatability and calibration variation are recorded across battery/load conditions.
- [ ] Encoder availability is confirmed; the open-loop versus closed-loop API decision has evidence and limitations.
- [ ] The API/spec can state units, bounds, stop behavior, update timing, and calibration needs without hidden assumptions.

### Results

Add date, firmware revision, test surface, battery/load conditions, measurements, and pass/fail observations after bench testing.
