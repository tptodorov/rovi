# Rovi M5: mecanum motion and feedback characterization

**Status:** proposed; no firmware package yet. Create the independent Cargo package when this experiment starts, following [`../README.md`](../README.md).

Reuse the assembled chassis, motor drivers, power system, and wiring from M4. Do not add encoders or other sensors unless inspection confirms they are already present or a later product requirement calls for them.

## Measuring motion without new hardware

A printed ArUco marker on the car and the laptop camera give measured motion to set against the commanded motion in the car log: `scripts/cam.sh` records, `scripts/track_marker.py` tracks the marker, `sim/compare_track.py` (in M8) prints distance and heading drift and draws both paths. Commands, marker print size and placement, and the limits of an oblique laptop view: [feedback loops, rank 11](../../docs/FEEDBACK-LOOPS.md). So far verified on synthetic frames only, never on the real car.

## Firmware and results

Firmware, wheel mapping, and build/flash commands will be added when implementation begins. Record wheel behavior, calibration, and any feedback hardware in [`MILESTONE.md`](MILESTONE.md#results).

Bench procedure: [TESTPLAN.md](TESTPLAN.md) (untethered, BLE control, tape and video).
