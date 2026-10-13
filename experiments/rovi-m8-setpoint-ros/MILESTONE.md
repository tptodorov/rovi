# Milestones

## M8: setpoint protocol, single owner, optional ROS 2

**Status:** implemented; board-verified 2026-10-13 on the bare ESP32-S3, except stamped teleop, a second BLE central and the iPhone side of Q18/Q19. Acceptance boxes below are ticked only for what was observed on the board.
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

- [x] A BLE, UDP or ROS claim makes that client the owner, and the other radio shuts down and reopens on release. *(observed 2026-10-13: BLE claim switches the Wi-Fi AP off and back, a UDP claim stops BLE advertising and `BYE` restores it; a ROS claim stopping BLE was not observed separately)*
- [x] A second claimant is refused, and ROS input from a non-owner gid is ignored. *(UDP `BUSY` and the ROS intruder node observed; a second BLE central needs a second adapter and is untried)*
- [x] A new owner starts disarmed. Only ARM enables motion, and every safety stop disarms.
- [x] The latest setpoint wins, and non-increasing sequences are never applied.
- [x] Lease expiry raises a safety stop within lease + 50 ms. Claim and reclaim windows behave as specified. *(lease 995 to 1006 ms, reclaim 5002 to 5003 ms, claim 10007 ms)*
- [x] For identical `TwistStamped` input, wheel commands match `mecanum_drive_controller`'s formulas.
- [x] `ros2 node list` and `topic info` show `/rovi`, and plain `ros2 topic pub` claims the car. *(Board: phase 6 `q2.txt`.)*
- [x] One ROS node can arm and drive the car (`/rovi/arm` and `/cmd_vel` from the same node). *(Board: the controller scenario, `ros_controller.py`. Plain CLI pub cannot do this by design: a second `ros2 topic pub` is another node, which the car ignores as a non-owner. Reworded from "plain pub and stamped teleop drive it", [#24](https://github.com/tptodorov/rovi/issues/24).)*
- [ ] A stock stamped teleop (`teleop_twist_keyboard` with `stamped:=true`) drives the car. *(Untried; it publishes only `/cmd_vel`, so arming needs the single-node tool or a product teleop. Not an M8 gate.)*
- [x] The BLE setpoint characteristic works at the negotiated MTU, streaming at 20 Hz.
- [x] Latency for BLE, UDP and ROS (median, p95, max) and the flash/RAM footprint are recorded.

### Results

Bench results (date, commit, board, clients, settings, logs, pass/fail) are added after bench testing. Laptop results are development evidence only ([ADR-0002](../../docs/adr/0002-simulation-never-proves-hardware.md)).

#### Board, 2026-10-13: UDP, timing, ROS, Q20 (first runs on hardware)

Setup: laptop Wi-Fi radio on the car AP (`192.168.4.2/24`, never a default route), internet over USB Ethernet. Evidence in `bench/board-2026-10-13-*`.

* **UDP over Wi-Fi** (`board-…-udp`): **8 of 8 scenarios pass**; echo 200 of 200, median 3.0 ms, p95 6.3 ms, max 31 ms, on a **dedicated radio**, which M3 could not measure. The car log is clean.
* **Timing on the car's clock** (`board-…-udp/windows.txt`, 10 probes): lease 995 to 1006 ms, reclaim window 5002 to 5003 ms, claim window 10007 ms.
* **Sequence rule** (`board-…-q20/sequence.txt`): wheels 250‰, then 150‰; the older and the repeated sequence number were never applied.
* **Q20 (AP off and on around BLE)**, 10 cycles (`board-…-q20`): every BLE session completed without a BLE error, so the switch does not drop the BLE link; the car logged 10 `ap=off` and 10 `ap=on`, no `ev=error`; the laptop rejoined the AP in 1 to 2 s each time (4 times 1 s, 6 times 2 s) and every UDP echo run was 20 of 20. A Wi-Fi client is dropped when BLE claims, and NetworkManager did not rejoin by itself within 25 s even with autoconnect on (`board-…-ap-client/`).
* **BLE off while UDP owns, and back after `BYE`** (`board-…-q20/ble-off.txt`): pass.
* **ROS on the board** (`board-…-ros`, build with `--features ros`): `/rovi` is in `ros2 node list`; `topic info -v /cmd_vel` shows one subscription from node `rovi`, hash `RIHS01_5f0fcd4f…`, BEST_EFFORT, depth 1; `/rovi/status` has one publisher; plain `ros2 topic pub` (no `-w 0`) claimed the car (`state=claimed session=1`, `ev=claimed source=Ros`). **Q2: yes.** The controller scenario (one node with `/cmd_vel` and `/rovi/arm`, plus an intruder node) passes: claimed, armed at `wheels_pm=500`, stopped on `Stop`, intruder ignored. **Router restarts: `/rovi` back after 3, 4 and 3 s.** The stale-session check (laptop off the AP for 5 s) was too short to break the session (back after 0 s), so it proves little.
* **ROS round trip** (`board-…-ros-latency/latency-ros.txt`, `/rovi/echo` to `/rovi/echo_reply`, 600 probes at 20 Hz, two runs, laptop on the car AP): 600 of 600 both times, loss 0 %, median 7.97 and 7.67 ms, p95 11.73 and 11.48 ms, max 47.0 and 23.8 ms. The echo claimed the car zero times. The path is rclpy, rmw_zenoh, the router, one Wi-Fi hop each way and the car; the sim over loopback measures 1.4 ms for the tool and the router alone. Closes [#16](https://github.com/tptodorov/rovi/issues/16).
* **Lost liveliness tokens, refresh** ([#22](https://github.com/tptodorov/rovi/issues/22), `board-…-ros-refresh/`): the car asks the router for the tokens again when a sample comes from an unknown publisher. On the sim, `sim/ros-refresh.sh`: with token events dropped and the refresh disabled the car never claims; with it, it recovers. On the board, built with a deliberately tiny receive queue (2 slots, 1 KB) to force the loss: **3 of 3 controller runs claimed, each after exactly one `ev=ros link=refresh`**. The shipped build keeps the 32-slot, 8192-byte queue.
* **Footprint (final builds):** text 653,537 B, data 18,804 B, bss 519,560 B without ROS; text 818,849 B, data 19,040 B, bss 519,356 B with ROS.

**Defects the board found** (each reproduced on the board, fixed, and re-run; the failing runs stay in `bench/` as evidence):

| Run | Symptom | Cause | Fix |
| --- | --- | --- | --- |
| `board-…-ble` | Status never notified while streaming | The 100 ms notify timer was recreated on every loop pass and never fired under 20 Hz writes | A `Ticker` per connection |
| `board-…-ble` | The car logged `mtu=23` | Logged at connect, before the central's exchange | `ev=mtu` after the exchange |
| `latency.py --ble` | Link dropped after 10 s | Echo is not activity; the claim window released the owner | The tool streams zero setpoints too |
| `board-…-ros-panic` | Stack overflow at boot (ROS build) | `EmbassyLinkManager` held 3072 `PacketMetadata` (about 120 KB), built by value on the stack | Separate `META` size (fork) |
| `board-…-ros-firstfail` | The controller never claimed the car | The router's burst of token datagrams overflowed a one-datagram receive queue; UDP does not resend, so the publishers' tokens were lost | `RX` and `META` consts (fork); queue of 32 slots and 8192 bytes; the car subscribes to publisher tokens only |
| `board-…-ros-before-close-fix` | `/rovi` never came back after a router restart | The restarted router answers every datagram with a 2-byte `Close`, which the receiver dropped | A `Close` from the peer ends the session (fork) |

**Follow-up issues:** [#22](https://github.com/tptodorov/rovi/issues/22) (re-request liveliness for an unknown publisher), [#25](https://github.com/tptodorov/rovi/issues/25) (about 60 s to notice a vanished router), [#24](https://github.com/tptodorov/rovi/issues/24) (acceptance wording for plain `ros2 topic pub`).

**Not observed yet:** a second BLE central (one adapter), stamped teleop, the iPhone side of Q18 and Q19, Wi-Fi range, phone onboarding. **Unexplained ([#23](https://github.com/tptodorov/rovi/issues/23)):** twice, after about 10 minutes, the laptop's Wi-Fi link to the car silently stopped passing traffic while still showing "connected" (re-association fixed it, and the car's ROS session returned by itself).

#### Board, 2026-10-13: boot and BLE (first runs on hardware)

Setup: ESP32-S3 (rev v0.2, 16 MB, MAC `14:c1:9f:cd:71:78`) on the CH343 UART port, no motors or other wiring; laptop BlueZ central; non-ROS build (text 653,409 B at first, 672,448 B image after the fix). Evidence in `bench/board-2026-10-13-*`.

* **Boot** (`board-…-boot`): `M8 board-only AP=Rovi M8 udp=192.168.4.1:7777 …` then `ev=boot ver=0.1.0`; no panic, no `ev=error`; `sim/carlog.py` passes.
* **BLE scenarios, first run** (`board-…-ble`): 8 pass, 1 skipped (second central: one adapter), **2 fail**, and both failures were firmware or log findings:
  * **Status notifications were starved while streaming.** The 100 ms notify timer was recreated on every loop pass, so it never fired while the central wrote setpoints every 50 ms. `drive.txt` showed `Armed` was never notified. Fixed with a `Ticker` that keeps its schedule.
  * **The car logged `mtu=23` at connect,** before the central's exchange. Added `ev=mtu id=N mtu=M` when the MTU changes (LOGGING.md catalog, `carlog.py`, tests updated). `mtu` in `ble link=up` stays as the value at connect.
* **BLE scenarios after the fix** (`board-…-ble-fix`): **10 pass, 1 skipped, 0 fail**, car log clean (145 events in one boot).
* **Q18 (ATT MTU), BlueZ half:** the client reads `mtu=247`; the HCI trace (`hci.btsnoop`, 12 connections) shows Client RX MTU 517 and Server RX MTU 247 on every connection. 247 is well above the 75 needed. The iPhone value is untested.
* **Q19 (connection interval), BlueZ half:** requested 30 to 50 ms; all 12 connections ran at **45 ms**, latency 0, with no connection update. A 20 Hz stream (201 setpoints in 10 s, 441 in 22 s) produced no `LeaseExpired`. The iPhone value is untested.
* **Q20 (AP off and on around BLE):** each BLE claim logged `ev=wifi ap=off` 15 to 17 ms later and `ap=on` 12 to 26 ms after the release, over 24 cycles with no `ev=error`. Observed from outside (`board-…-ap-switch/timeline-iw.txt`, flushed active scans of the SSID on channel 1): visible before, **absent in all 10 probes during the session**, visible again after. Still untried: a Wi-Fi client already associated when BLE claims (phase 5 with the laptop on the AP).
* **BLE echo latency** (`board-…-ble-fix/latency-ble.txt`, 600 probes at 20 Hz with a zero-setpoint stream alongside): 600 of 600 returned, loss 0 %, median 70.5 ms, p95 94.3 ms, max 159.5 ms, at a 45 ms connection interval. The first attempt failed: echoes alone are not activity, the claim window (10 s) released the owner and the car dropped the link, so `sim/latency.py --ble` now streams zero setpoints too.
* **Kinematics on the board:** `vx = 0.2 m/s` gave `wheels_pm=100,100,100,100`, as the host tests predict.

#### Laptop: core and UDP scenarios, 2026-10-07 — pass

* **Host tests:** 48 pass (`cargo test --lib`): CDR codec against real ROS bytes, kinematics against the `mecanum_drive_controller` formulas, the arbiter (ownership, lease, arm, claim and reclaim windows, limits), UDP framing and the device core, and the `rmw_zenoh` keys, liveliness tokens, entity gid and attachment against a live Jazzy graph (gid `1b8f09d3…` reproduced).
* **Sim car over real UDP sockets:** 6 scenarios pass (`sim/udp_scenarios.py`).
* **ROS 2 sim car (real Jazzy controller in Docker, `sim/ros-scenarios.sh`):** pass. The first `/cmd_vel` claims the car, an arm from a second publisher of the same node arms it, an intruder node's setpoint and stop are ignored, arm false stops, and `/rovi/status` reaches `ros2`. 58 host tests pass.
* **Design change found by this test:** an owner identified by publisher gid cannot work, because `/cmd_vel` and `/rovi/arm` are different publishers with different gids. The car now learns publisher gid to ROS node from liveliness tokens and treats the node as the controller. This needed a liveliness subscriber in the zenoh-nostd fork.
* **S3 firmware, cross-build (test-plan step 4):** pass for AP + UDP + BLE GATT + control tick + LED. `cargo build --release` and `cargo clippy -D warnings` are clean. Footprint: text 663,197 B, data 18,668 B, bss 519,688 B (static buffers). Not flashed, because no board was available.
* **S3 firmware with ROS (`--features ros`):** cross-builds and passes clippy. text 817,481 B (+154 KB over the build without ROS), data 18,928 B, bss 519,468 B. The fork needed optional `defmt` in `zenoh-embassy` and `embassy-sync` 0.8. Not flashed.
* **ROS reconnect, 2026-10-08:** reproduced first (the car never rejoined after a router restart: `session.run()` returned `TransportClosed`, the adapter ignored it, and the static session cells could not be reopened). Fixed with a reconnect loop and one heap-allocated session per connection. `sim/ros-reconnect.sh` passes three router restarts in a row, and the ROS scenario still passes. The firmware has the same loop (flash +3.7 KB, text 821,145 B); on the board it relies on the UDP link freeing its buffer slot when dropped, which is read from the code, not run.
* **Open issues worked, 2026-10-08:**
  * Sequence numbers: fixed in the fork (per priority and reliability, with a wrap-around test). The adapters no longer ignore the check, and the ROS, reconnect and stale-session scenarios show no "Inconsistent SN" errors.
  * Why a session ended: now logged (`LinkTxFailed` after a router kill).
  * Duplicate zid: `sim/ros-stale-session.sh` drops a UDP session silently and rejoins with the same zid. The first attempt failed with the router still holding the old session, and a retry succeeded.
* **Release rules, 2026-10-11 (decision):** the reclaim window now follows the glossary: it starts when the lease lapses. STOP and an out-of-limits setpoint keep the owner while it keeps streaming, and arming again needs a setpoint sent after the stop. The code had opened the window after every stop. Host tests added first (73 to 76 pass with the BLE status tests).
* **BLE Status (decision):** Status is set at connect, so a read right after connecting is correct, and its last-error byte reports a refused control write (`NotReady`, `Invalid`) like UDP, until the next control write that succeeds.
* **Wi-Fi radio-off:** `esp-radio` 1.0.0-beta.1 (the latest) has no public AP stop, so the firmware uses a `set_config` mode switch (AP to an idle station config and back) while BLE owns the car. Core rule `Device::wifi_allowed` is tested (UDP and ROS share the AP). The firmware builds and passes clippy (text 664,677 B without ROS, 823,025 B with). Whether the switch is clean on the board, and next to a live BLE link, is a bench item. Espressif rates SoftAP plus BLE connected as unstable, so this is the reason to switch the AP off.
* **Not yet:**  token-drop handling (dropped tokens without a key are skipped, so stale gid entries age out by eviction), per-reliability sequence numbers in zenoh-nostd, and everything on the board.

#### Laptop: latency echo, 2026-10-11 — pass (UDP on the sim only)

* **Core:** UDP `ECHO` (kind 9) and the BLE Echo characteristic (`…0015`) send back up to 32 bytes unchanged. One rule in the core (`device::echo`), used by both. Host tests: a non-owner's echo works, the arbiter state and lease are untouched (the lease still expires), and oversize payloads are dropped. 79 host tests pass (re-run after the reclaim-window change, where reclaim starts when the lease lapses).
* **Sim car:** a UDP scenario checks echo while open (not a claim), a non-owner's 32-byte echo while another client is armed, the oversize drop, and 20 probes through `sim/latency.py`. 8 scenarios pass.
* **Client:** `sim/latency.py` against the sim car over loopback, 100 probes at 50 Hz: count 100, loss 0 %, median 0.28 ms, p95 0.39 ms, max 0.49 ms. This only checks the tool; the bench numbers for acceptance come from the board.
* **S3 firmware:** builds and passes clippy with and without `ros`. Text 653,465 B, 816,989 B with `ros` (the echo adds about 2.1 KB). Not flashed.
* **Not yet:** the BLE variant has not been run (no board), and no UDP, BLE or ROS latency on hardware. ROS latency needs a separate echo and is tracked in [#16](https://github.com/tptodorov/rovi/issues/16). BLE echo runs only while the car is free (connecting claims it); accepted.

#### Laptop: ROS graph visibility (test-plan step 1), 2026-10-07 — pass

* **Setup:** zenoh-nostd fork [`tptodorov/zenoh-nostd@rovi/liveliness-token`](https://github.com/tptodorov/zenoh-nostd/tree/rovi/liveliness-token) (`ac96994`) on the std platform, as a client of `rmw_zenohd` (ROS 2 Jazzy, `rmw_zenoh_cpp` 0.2.10 in Docker). The fork adds `Session::declare_token` and `TransportLinkManager::zid()`. The example `z_ros_node` declares node `/rovi` and a best-effort `/cmd_vel` `TwistStamped` subscription. Reproduce with [`sim/graph-check.sh`](sim/graph-check.sh).
* **Result:**
  * `ros2 node list` shows `/rovi`.
  * `ros2 topic info -v /cmd_vel` shows one subscription from node `rovi`, type hash `RIHS01_5f0fcd4f…`, QoS best-effort, depth 1.
  * Plain `ros2 topic pub -r 5 /cmd_vel geometry_msgs/msg/TwistStamped` (no `-w 0`) starts at once, and the node receives 68-byte CDR payloads, matching the design.
* **Liveliness key** (verified against a real ROS 2 node): `@ros2_lv/<domain>/<zid>/<nid>/<id>/<NN|MS|MP>/%/%/<node>[/<%topic>/<type>/<hash>/<qos>]`. The `source_gid` in a publisher's attachment is the XXH3-128 hash of its own liveliness key, which the car must reproduce for non-owner gid filtering.
* **Not yet checked:** the S3 build and footprint with the patch, the board's own zid (it must be stable, so it comes from the MAC), reconnect re-declaration of tokens, and the publisher tokens for `/rovi/status`.
