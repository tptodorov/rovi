# Milestones

## M8: setpoint protocol, single owner, optional ROS 2

**Status:** design approved 2026-10-06; implementation started 2026-10-07 (laptop only; first risk item cleared)
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

Bench results (date, commit, board, clients, settings, logs, pass/fail) are added after bench testing. Laptop results are development evidence only ([ADR-0002](../../docs/adr/0002-simulation-never-proves-hardware.md)).

#### Laptop: core and UDP scenarios, 2026-10-07 — pass

* **Host tests:** 48 pass (`cargo test --lib`): CDR codec against real ROS bytes, kinematics against the `mecanum_drive_controller` formulas, the arbiter (ownership, lease, arm, claim and reclaim windows, limits), UDP framing and the device core, and the `rmw_zenoh` keys, liveliness tokens, entity gid and attachment against a live Jazzy graph (gid `1b8f09d3…` reproduced).
* **Sim car over real UDP sockets:** 6 scenarios pass (`sim/udp_scenarios.py`).
* **Not yet:** the ROS adapter in the sim, the BLE adapter, the S3 firmware shell, and everything on the board.

#### Laptop: ROS graph visibility (test-plan step 1), 2026-10-07 — pass

* **Setup:** zenoh-nostd fork [`tptodorov/zenoh-nostd@rovi/liveliness-token`](https://github.com/tptodorov/zenoh-nostd/tree/rovi/liveliness-token) (`ac96994`) on the std platform, as a client of `rmw_zenohd` (ROS 2 Jazzy, `rmw_zenoh_cpp` 0.2.10 in Docker). The fork adds `Session::declare_token` and `TransportLinkManager::zid()`. The example `z_ros_node` declares node `/rovi` and a best-effort `/cmd_vel` `TwistStamped` subscription. Reproduce with [`sim/graph-check.sh`](sim/graph-check.sh).
* **Result:**
  * `ros2 node list` shows `/rovi`.
  * `ros2 topic info -v /cmd_vel` shows one subscription from node `rovi`, type hash `RIHS01_5f0fcd4f…`, QoS best-effort, depth 1.
  * Plain `ros2 topic pub -r 5 /cmd_vel geometry_msgs/msg/TwistStamped` (no `-w 0`) starts at once, and the node receives 68-byte CDR payloads, matching the design.
* **Liveliness key** (verified against a real ROS 2 node): `@ros2_lv/<domain>/<zid>/<nid>/<id>/<NN|MS|MP>/%/%/<node>[/<%topic>/<type>/<hash>/<qos>]`. The `source_gid` in a publisher's attachment is the XXH3-128 hash of its own liveliness key, which the car must reproduce for non-owner gid filtering.
* **Not yet checked:** the S3 build and footprint with the patch, the board's own zid (it must be stable, so it comes from the MAC), reconnect re-declaration of tokens, and the publisher tokens for `/rovi/status`.
