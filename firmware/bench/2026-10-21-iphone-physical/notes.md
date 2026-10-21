# 2026-10-21: Physical mode, motor power off

Build tested: `8fb8fd174d30`. Motor power bank off, ESP32 powered by USB;
Todor used the published iPhone client in Bluefy. No motor result is claimed.

Todor confirmed the requested motion stayed zero at neutral/release, followed
all three axes, and remained disarmed after STOP and mode changes. He then
reported unwanted disarming when scrolling an already armed page, an unnecessary
separate calibration tap before ARM, and status/messages hidden while scrolling.
He did not notice the fault message or report whether the drive pad was held.
The exact scroll-triggered failure remains unclassified.

Phone model and Stream write count/max gap/slowest write were not supplied.
The response does not establish a recorded 60 s sensor-plus-BLE timing result.
Task 3.1 remains open; this is a partial functional pass with UX defects.

## Host correction

Before the fix, two Session tests failed: fresh valid sensor input could not ARM
without separate calibration, and a released-pad sensor gap revoked arm. A real
page browser test also disconnected after a simulated 350 ms sensor pause during
scrolling, reproducing a possible trigger, not identifying Bluefy's actual cause.

The correction captures neutral within the explicit Set neutral & ARM action.
Released-pad sample gaps/reading posture keep arm and neutral at zero, with new
presses blocked until input is valid. Scroll cancels holds without recentering.
Sensor/posture loss during a held drive, invalid values, reference changes and
existing stream/lifecycle failures retain cleanup. Status/STOP stay visible.
At publication, corrected phone UX and sensor-driven motor behavior still needed acceptance.

Correction published as build `780f65eff594`: 46 Node tests and 23 source/built
browser tests pass; all 15 public assets match the clean build. The live-page
smoke check starts disarmed with no picker call. This is host/publication
evidence, not another phone or motor run.


## Corrected phone retest

Todor tested build `780f65eff594` in the announced motor-power-off procedure:
Physical → Enable sensors → connect → Set neutral & ARM; scroll with the pad
released, tilt to reading posture and return to neutral; check visible status/STOP,
then STOP and Set neutral & ARM again without separate calibration.

He replied “ok, seems to work wekk”. Record this as an operator-confirmed pass
for that UX retest (task 3.0), with no fault reported. This is a functional report,
not a captured event trace or identification of the original Bluefy trigger.
Phone model, 60 s neutral observations and Stream write count/max gap/slowest
write were not supplied. Timing/lifecycle task 3.1 and motor-on tasks remain open.


## 60-second neutral and stream check

On the announced motor-power-off check of build `780f65eff594`, Todor reported
an iPhone 11 and confirmed that requested motion stayed zero throughout the
60-second neutral hold, with no disconnect or stale-sensor message reported.
The procedure ended with release and STOP.

| Reported Stream value | Result |
| --- | --- |
| Write count | 1,366 |
| Maximum submission gap | 180 ms |
| Slowest completed write | 19 ms |

The observed maximum gap is 20 ms below the unchanged 200 ms client cutoff.
These are client-reported session counters, not a captured 60-second trace;
setup and other activity may contribute to the write count. No exact send rate,
radio delivery latency, physical stop time or raw angle-drift bound is inferred.
This confirms the requested-motion display stayed at zero during the announced
hold; the display's rounded values do not establish an exact raw-angle bound. The earlier sensor-only report identified iOS 18.7/Bluefy 3.9.3; their
versions were not re-entered in this response. No motor power was enabled.

Task 3.1's neutral/stream subcheck passes. Permission retry, held-input interruption,
reference/posture loss, lock/app-switch and fresh manual recovery on the actual
phone remain to be recorded; task 3.1 as a whole stays open. Motor-on acceptance
also remains open and needs its own announced batch and fresh go.
