# Rovi M5: mecanum motion and feedback characterization

**Status:** in progress. The 2026-10-17 session demonstrated low-command forward/reverse, strafe, rotation, diagonals and automatic lease stopping on the floor. Calibration and controlled battery/load variation remain open; see [MILESTONE.md](MILESTONE.md#results).

M5 reuses M4's assembled car and the [M8 firmware and BLE client](../rovi-m8-setpoint-ros/README.md); no separate firmware package is needed. The motors have no encoders, and adding sensors is out of scope. Control is untethered over BLE, with distances from tape and video.

* [TESTPLAN.md](TESTPLAN.md): overall procedure and the 1 × 1 m test area.
* [CALIBRATION.md](CALIBRATION.md): next command-response and battery/load measurements, on parquet.
* [MILESTONE.md](MILESTONE.md): measurements, acceptance and remaining limits.

Every motor run is announced and waits for Todor's "go". Keep command values distinct from measured m/s and rad/s: the geometry is measured, but the motor/velocity mapping remains uncalibrated.

## Optional camera measurements

A printed ArUco marker and the laptop camera can compare observed motion with the commanded path using `scripts/cam.sh`, `scripts/track_marker.py` and M8's `sim/compare_track.py`. These tools have synthetic-frame evidence only, not validation on this car. Setup and camera limits: [feedback loops, rank 11](../../docs/FEEDBACK-LOOPS.md). They are optional for the tape/video procedure.
