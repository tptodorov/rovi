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

At the time these dimensions were recorded, firmware geometry was still the placeholder. Subsequent stand measurements are recorded below; floor speed, drift and stopping distance remain unmeasured.

#### Top-view inspection, 2026-10-17

![Assembled car, viewed from above, with FRONT pointing to the top of the image](bench/2026-10-17-setup/top-front.jpg)

Source: Todor's `IMG_0522.HEIC`, supplied from `experiments/rovi-m4-four-motor-safety/`; the JPEG is resized and stripped of metadata. The [earlier unmarked photo](bench/2026-10-17-setup/top.jpg) is retained as setup evidence.

* All four mecanum wheels and both power banks are visible.
* The visible roller axes appear to form the expected X pattern ([WPILib reference](https://docs.wpilib.org/en/stable/docs/software/hardware-apis/motors/wpi-drive-classes.html)). This is photo evidence only; the F1 motion check remains pending.
* The FRONT arrow points toward the top of the image, beside the black power bank. Viewed this way: FL is upper-left, FR upper-right, RL (BL) lower-left and RR (BR) lower-right. Left and right are the car's own sides when facing forward.
* Wiring continuity, wheel clearance through a full turn and secure mounting cannot be established from this view.

Record firmware revision, floor surface, battery state, observations and evidence for each subsequent run.

#### F1 mounted-wheel direction checks, 2026-10-17

With the car on a stand, Todor confirmed forward, strafe-left and rotate-left wheel directions, including all four starting and stopping. See [run notes and BLE client output](bench/2026-10-17-f1/notes.md). One initial strafe run produced no movement; its cause remains unconfirmed. These stand tests do not establish actual chassis motion.

At `vx 0.1` for 5 s, Todor counted approximately 4.75 FL wheel rotations: **0.95 rev/s = 57 RPM = 5.97 rad/s**. With a 60 mm wheel diameter, nominal rim speed is approximately **0.179 m/s**, unloaded. This whole-run average is not floor speed; acceleration, stopping, slip and load are not resolved. Full-duty wheel-speed measurement and all floor tests remain pending.

At `vx 0.5` for 5 s, Todor counted approximately 6.5 FL rotations: **1.30 rev/s = 78 RPM = 8.17 rad/s**, nominal unloaded rim speed **0.245 m/s**. The same whole-run-average limitations apply; this is a second calibration point, not maximum wheel speed.

At `vx 1.0` for 5 s (a repeated run), Todor counted approximately 8.75 FL rotations: **1.75 rev/s = 105 RPM = 11.00 rad/s**, nominal unloaded rim speed **0.330 m/s**. This command used about 70% duty with the placeholder geometry and motor mapping then flashed; it was not a 100%-duty measurement. The three points describe one unloaded wheel, not calibrated chassis velocity or wheel-to-wheel variation.

#### Measured-geometry firmware prepared, 2026-10-17

Firmware source now uses radius 0.030 m and centre projection sum 0.1275 m. The speed-normalization parameter remains 40 rad/s as a placeholder; the 105 RPM observation at 70% duty is not a full-duty measurement. ESP32-S3 release build and clippy passed. **Flashed successfully; boot and BLE capabilities verified on the board** ([evidence](bench/2026-10-17-geometry/notes.md)). A separately authorized gentle forward run (`vx 0.1`, 5 s, approximately 45% duty) passed on the stand: Todor confirmed all four wheels started, turned forward and stopped cleanly. Floor tests remain pending.

The geometry change affects duty for the same command: forward `vx 0.1` is about 45%, `vx 0.5` about 65%, and `vx 1.0` about 90% (previously 43%, 55% and 70%). Repeat a gentle stand check after flashing; announce and agree each new motor run. Floor tests start with a short, low-command pulse, not a repeat of the five-second stand runs.

#### First floor movement, 2026-10-17

At `vx 0.05` for 0.5 s (approximately 42.5% duty), Todor confirmed the car moved. This is the lowest floor command tested, not an exact start threshold. The client reported a clean STOP; travel distance, drift, heading change and physical stopping behavior were not separately quantified or confirmed. See [F2 notes and client output](bench/2026-10-17-f2/notes.md). Loaded speed and repeatability remain unmeasured.

A subsequent identical pulse produced approximately **10 cm forward travel with no obvious drift**, reported by Todor. Displacement divided by command duration is approximately **0.20 m/s**, not steady-state speed: acceleration and coast after STOP were not separated. Repeatability, quantified drift, heading change and stopping distance remain pending. Commanded `vx` is not calibrated to actual chassis velocity.

A second distance observation with the same command was **11 cm**. The two measured runs span 10–11 cm (mean 10.5 cm); a third repeat is pending. Drift and stopping were not separately reported for the second run.
