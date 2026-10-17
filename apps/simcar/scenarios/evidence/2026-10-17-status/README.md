# 2026-10-17: shared status and owner errors

Development evidence from the product sim car and stock ROS 2 Jazzy in Docker, using the
pinned dependency lockfiles. No board access or motor runs; not physical acceptance.

Reproduce after `cd apps/simcar && cargo build --locked --features ros`:

```sh
apps/simcar/scenarios/ros-scenarios.sh
apps/simcar/scenarios/ros-capabilities.sh
```

* [ROS owner timeline](ros-status.txt): STOP at 3.0 s; ARM before fresh input at 3.1 s reports
  `error=not-ready`; resumed streaming and an intruder STOP preserve it. Explicit owner ARM
  scheduled at 4.5 s clears it. Lease-expired and out-of-limits reasons appear promptly; release at
  16.0 s restores `state=open reason=none error=none session=0`. All samples have four keys. [Full ROS run](ros-scenarios.txt).
* [Shared UDP/ROS checks](capabilities-status.txt), before and after router restart: late status
  receipt 0.461/0.420 s; longest observed heartbeat 0.905/0.906 s; claim/release about 0.010 s.
  `ros_status.py` checks UDP refusal persistence across setpoint/echo/non-owner input, successful
  STOP clearing, current-session stop notification and release cleanup. It never enables outputs.
  [Car log](shared-status-car.log) confirms no armed state during these metadata checks.

Host tests cover every numeric status code, reason/error string, small output buffers, BLE
immediate snapshots, malformed UDP controls/ROS Bool, replay isolation and UDP owner targeting.
The numeric version-1 wire codes are unchanged. A queued UDP stop notification initially broke
three socket scenarios; the client now matches direct replies by request sequence. That
regression and silent-owner stop delivery are explicit socket scenarios.

Actual ESP32 GATT read/notification delivery and UDP/ROS radio behavior remain task 5.2.
