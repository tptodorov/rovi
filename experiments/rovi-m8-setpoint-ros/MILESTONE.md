# Milestones

## M8: setpoint protocol, single owner, optional ROS 2

**Status:** design approved 2026-10-06; not implemented
**Setup count:** one, ESP32-S3 board only; motor hardware disconnected

### Goal

Prove that a laptop or phone over BLE or UDP, or a ROS 2 graph over zenoh, can each become the single owner of the car. The owner arms it and drives it with `TwistStamped`-shaped setpoints that produce the same wheel commands as `mecanum_drive_controller`. Losing the owner, or any stop, must reliably disarm the car. See [README](README.md).

### Risks covered

* ROS 2 graph visibility from zenoh-nostd (liveliness tokens): the first risk item.
* Exclusive ownership and radio shutdown/reopen on real radios.
* Arming, lease, claim-window and reclaim-window behaviour under loss and reconnect.
* Setpoint codec and kinematics compatibility with ROS 2.
* BLE ATT MTU and connection interval for 20 Hz CDR setpoints (Q18, Q19).
* Footprint and latency of the three transports on the S3.

### Test plan

1. **Liveliness tokens first.** Patch the zenoh-nostd fork. Then, with the sim car, check that `ros2 node list` shows `/rovi`, that `ros2 topic info -v /cmd_vel` lists it, and that plain `ros2 topic pub` works without `-w 0`. If this fails, stop and revisit [ADR-0005](../../docs/adr/0005-zenoh-nostd-ros-backend.md).
2. **Core host tests:** CDR golden vectors, kinematics vectors, the state machine, and `rmw_zenoh` keys and attachments.
3. **Sim car scenarios** with a Python UDP client and ROS 2 (CLI and stamped teleop): claim, busy, arm, stream, stop, lease expiry, claim and reclaim windows, owner loss, non-owner gid.
4. **S3 firmware:** cross-build, clippy, footprint.
5. **Bench (board-only).** Repeat step 3 on real radios with BLE (bleak), UDP and ROS. Confirm the other radio stops and reopens. Measure lease-expiry timing and latency for each transport (median, p95, max), and record the negotiated ATT MTU and connection interval.

### Acceptance

Real hardware only; no motor hardware is connected for this milestone.

- [ ] A BLE, UDP or ROS claim makes that client the owner, and the other radio shuts down and reopens on release.
- [ ] A second claimant is refused, and ROS input from a non-owner gid is ignored.
- [ ] A new owner starts disarmed. Only ARM enables motion, and every safety stop disarms.
- [ ] The latest setpoint wins, and non-increasing sequences are never applied.
- [ ] Lease expiry raises a safety stop within lease + 50 ms. Claim and reclaim windows behave as specified.
- [ ] For identical `TwistStamped` input, wheel commands match `mecanum_drive_controller`'s formulas.
- [ ] `ros2 node list` and `topic info` show `/rovi`. Plain `ros2 topic pub` and stamped teleop drive it.
- [ ] The BLE setpoint characteristic works at the negotiated MTU, streaming at 20 Hz.
- [ ] Latency for BLE, UDP and ROS (median, p95, max) and the flash/RAM footprint are recorded.

### Results

Add after bench testing: date, commit, board, clients, settings, logs, pass/fail observations.
