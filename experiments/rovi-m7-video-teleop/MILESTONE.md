# Milestones

## M7: direct teleoperation with video

**Status:** deferred (2026-10-06) — no camera or onboard Linux planned at this stage; see [PLAN](../../docs/PLAN.md)
**Setup count:** reuse M3/M4 equipment and connection mode; add the selected camera/video hardware only

### Goal

Measure the selected camera/video path while a remote control drives the car directly over the chosen Wi-Fi topology, without control latency, command loss, or watchdog behavior exceeding product requirements. BLE client compatibility is already proven and is outside this milestone.

### Risks covered

* Camera and compute placement for the first end-to-end car.
* Video bandwidth, frame delay, and loss under the selected Wi-Fi topology.
* Control latency and watchdog behavior while video is active.
* App/client integration with the device-specific command set.

### Prerequisites

* M3 direct Wi-Fi mode and command protocol selected.
* M4 electrical limits and safe motor operation established.
* M5 command/API behavior specified sufficiently for the client.
* Camera, compute board, video transport, and test client selected. Do not run this milestone with assumed or placeholder camera hardware.

### Test plan

1. Assemble the selected camera and compute board with the M4 car setup. Record the selected network topology and which device hosts motor control and video.
2. Measure video-only startup time, sustained frame rate, bandwidth, latency, and frame loss at the expected operating distance.
3. Send representative drive commands directly to the car with video disabled, then repeat with video active and at expected peak load.
4. Record command round-trip latency, loss, disconnects, video latency/frame loss, and watchdog events across repeated runs.
5. Disconnect the control client while video remains active. Verify watchdog expiry disables all driver standby signals within the M4/product bound; reconnect and verify intentional re-arm behavior.

### Acceptance

Real hardware only. Define numeric latency, frame-rate, bandwidth, and timeout targets in the product spec before evaluating pass/fail.

- [ ] Camera/compute placement and network topology are recorded.
- [ ] Video meets the agreed frame-rate, latency, and loss targets at the intended range.
- [ ] Direct control meets its agreed latency/loss targets while video is active.
- [ ] Control loss still triggers the configured watchdog and disables all driver standby signals within the agreed bound.
- [ ] End-to-end client can issue the supported device command set and displays the selected video stream.

### Results

Add date, firmware/app revisions, camera/compute models, network topology, test range, measurements, and pass/fail observations after testing.
