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
Corrected phone UX and sensor-driven motor behavior still need acceptance.
