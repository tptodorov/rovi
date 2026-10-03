# Rovi M7: direct teleoperation with video

**Status:** proposed and gated on selecting the camera and compute placement; no firmware package yet. Create the independent Cargo package when experiment firmware implementation begins, following [`../README.md`](../README.md).

Reuse the M4 car bench setup and M3 connection mode. Add only the selected camera and video hardware. The video-host decision must come from product requirements; existing software history does not set it. This tests video and control together without reopening BLE bring-up.

## Firmware and results

Add firmware, wiring, and build/flash instructions after the camera and control topology are selected. Record video and control measurements in [`MILESTONE.md`](MILESTONE.md#results).
