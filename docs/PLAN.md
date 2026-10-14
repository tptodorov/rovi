# Rovi plan

Current status, roadmap, and decisions deliberately deferred. For what Rovi is and why, see [`VISION.md`](VISION.md). For how work gets done, see the root [`README.md`](../README.md). This file is living — update it as milestones land, don't append a history; for *why* a specific hard-to-reverse decision was made, see [`adr/`](adr/).

## Current state (2026-10-14)

* M8 runs on the bare board (no motors) and passes its gating acceptance: BLE (MTU 247, 45 ms interval, 10 of 10 scenarios), UDP over Wi-Fi (8 of 8), the Wi-Fi AP switching off and on around BLE (10 clean cycles), lease and window timing, and ROS 2 (graph visibility, controller scenario, reconnect). The board found six defects, all fixed ([results](../experiments/rovi-m8-setpoint-ros/MILESTONE.md#results)). Since 2026-10-14 the car reaches the ROS router over TCP (ADR-0012) and ROS 2 arms on its own (ADR-0011). Motors are not wired yet: M4 is next.
* Hardware: the parts package arrived on 2026-09-26 ([README](../README.md#hardware)). The ESP32-S3 bench is available, without motor wiring.
* Product software: the low-level four-wheel API is proposed ([M6](../spec/changes/add-four-wheel-api/proposal.md)), not accepted or implemented; the layered application is not started. Motor limits, speeds and fail-safe timing wait for M4/M5.
* CI runs the host tests, the sim scenarios and the firmware builds on every PR (`.github/workflows/ci.yml`). Board and ROS-router checks stay manual.
* Motor current: 150 mA per motor running (2026-10-14), about 0.6 A for four; start and stall currents are not measured ([M4 results](../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results)).

## Roadmap

* GitHub milestones: https://github.com/tptodorov/rovi/milestones
* Order (2026-10-06): the riskiest open decision comes first. Milestone numbers are kept so links stay stable.

### Software

1. **S1 spike** (done 2026-10-06, laptop only) — yes, with conditions. nano-ros's XRCE backend cross-compiles for the S3, links next to Rovi's stack, and interoperates with ROS 2 Jazzy via the micro-ROS agent. Rovi would own the S3 port (13 platform symbols; blocking UDP must be bridged to embassy-net). Footprint and on-board behaviour are untested; M8 decides nano-ros vs the direct XRCE client. See [S1 findings](research/s1-micro-ros-nano-ros-spike.md).
   * **S2 spike** (done 2026-10-06, laptop only): native Rust/embassy [zenoh-nostd](https://github.com/eclipse-zenoh/zenoh-nostd) + `rmw_zenoh`. ROS 2 ↔ car data works in both directions over UDP, and a real session links into M3 at +101 KiB flash. The car is invisible to the ROS graph until liveliness tokens are added. The project is early-stage. It is the preferred path for M8's `ros` feature. See [S2 findings](research/s2-zenoh-nostd-spike.md).
2. **M8** (board-verified 2026-10-13/14; accepted except the deferred items below) — setpoint protocol with a single owner and optional ROS 2. See [`experiments/rovi-m8-setpoint-ros/`](../experiments/rovi-m8-setpoint-ros/), [ADR-0004](adr/0004-single-owner-latest-setpoint.md), [ADR-0005](adr/0005-zenoh-nostd-ros-backend.md), [ADR-0006](adr/0006-twiststamped-cdr-setpoint.md), [ADR-0011](adr/0011-ros-auto-arm.md), [ADR-0012](adr/0012-ros-router-over-tcp.md) and [ADR-0013](adr/0013-ros-owner-is-the-node.md). Summary:
   * Setpoints are `TwistStamped`, CDR-encoded on BLE, UDP and ROS, with `mecanum_drive_controller`-compatible kinematics.
   * The latest setpoint wins; there is no queue. Each valid setpoint renews the lease.
   * The first controller to claim the car (BLE connect, UDP hello, or first ROS setpoint) owns it, and the other radio shuts down until release.
   * BLE and UDP need an explicit arm; ROS 2 arms on its own stream. Every safety stop disarms, and a STOP or out-of-limits stop needs an explicit arm on every source. Claim and reclaim windows apply.
   * The car runs an AP only. ROS 2 runs over zenoh-nostd behind a compile-time `ros` feature, over TCP to `rmw_zenohd` on a laptop on the car AP.
   * **Gating (done):** every acceptance item observed on the board. **Deferred, not gating:** a second BLE central (needs a second adapter), the iPhone side of Q18/Q19, range and phone onboarding (M3 remainder), and the Wi-Fi stall ([#23](https://github.com/tptodorov/rovi/issues/23), open until it recurs).
3. **M6** (proposal written 2026-10-10; not accepted, not implemented) — the low-level four-wheel API as an OpenSpec change: [`spec/changes/add-four-wheel-api/`](../spec/changes/add-four-wheel-api/proposal.md). It adds a `motor-control` layer (four normalised wheel commands, standby, limits) under M8's setpoint/ownership model. Motor limits, maximum wheel speed, open- vs closed-loop, brake mode and standby latency are OPEN (O1-O8) until M4/M5.

### Hardware (resume when the bench setup is available)

* **M8 and M3 remainder** (deferred, see M8 above). Then M4 ([`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/)), the first step that needs wiring.
* **M4** (step 1 ready, 2026-10-14) — four motors on the M8 firmware, wheels off the ground, minimal wiring (M1's pins, the power bank) and a software fail-safe ([ADR-0014](adr/0014-m4-minimal-wiring-software-fail-safe.md)). Step 1 is one driver and one motor; step 2 adds the other three. Running current is 150 mA per motor. Gates all motion. See [`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/).
* **M5** (proposed) — characterize mecanum wheel mapping, open-loop behavior, and available feedback (encoders decide open- vs closed-loop) using the M4 setup. See [`experiments/rovi-m5-mecanum-motion/`](../experiments/rovi-m5-mecanum-motion/).

### Done

* **M1** (complete, bench-tested 2026-10-02) — prove one ESP32-S3 + one TB6612FNG channel drives one motor, on real hardware. See [`experiments/rovi-m1-tb6612/MILESTONE.md`](../experiments/rovi-m1-tb6612/MILESTONE.md).
* **M2** (complete, bench-tested 2026-10-02) — ESP32-S3 as a BLE peripheral: advertises a custom GATT service, a generic BLE client writes drive commands, firmware reacts observably (LED/serial). Bare board, no motor hardware — independent of M1. See [`experiments/rovi-m2-ble/`](../experiments/rovi-m2-ble/).
* **M3** (hardware-validated 2026-10-04; 5 of 6 acceptance items) — direct WPA2 car AP and shared BLE/Wi-Fi command ingress, board-only. Shared ingress, queue/stale/stop, watchdog-under-backlog and per-transport loss pass on the board. A 2 s TCP socket timeout caps Wi-Fi silence below the configured watchdog. M8 supersedes its TCP+FIFO design. See [`experiments/rovi-m3-wifi-queue/`](../experiments/rovi-m3-wifi-queue/).

## Open questions and decisions

Living list. Close an item when it is decided, move hard-to-reverse outcomes into an [ADR](adr/), and delete it here. Owner: **you** = project decision; **M8** = settled by the M8 design or bench.

| # | Question / decision | Options / current lean | Settled by |
| --- | --- | --- | --- |
| Q9 | iPhone client | M8 is accepted with laptop clients (BLE via bleak, a Python UDP client, ROS 2). Open: when and how to build the iPhone app. Decide after M8 bench results for Q18/Q19 | you, after M8 |
| Q11 | Open-loop vs closed-loop wheel control; ros2_control wheel-velocity mode | Depends on whether encoders exist. M6's normalised wheel command fits both (O3) | M5 |
| Q13 | Runtime vs compile-time configuration | Compile-time `ros` feature for M8 (agreed); runtime config later | after M8 |
| Q16 | zenoh-nostd patches | Rovi pins the fork by commit ([ADR-0010](adr/0010-zenoh-nostd-fork-pin.md)); nine PRs to upstream (#28 to #36; the four fixes #28, #31, #35, #36 marked ready on 2026-10-14) are separate and do not block it. Upstream has merged nothing since 2026-06-26, so we asked which patches they want in [zenoh-nostd#37](https://github.com/eclipse-zenoh/zenoh-nostd/issues/37). The router's UDP listener bug is [zenoh#2849](https://github.com/eclipse-zenoh/zenoh/issues/2849). The three fixes the board found are [#35](https://github.com/eclipse-zenoh/zenoh-nostd/pull/35) (embassy link buffer sizes) and [#36](https://github.com/eclipse-zenoh/zenoh-nostd/pull/36) (end the session on a peer `Close`). A pinned fork with optional `defmt`, `embassy-sync` 0.8 and a liveliness API (decided, ADR-0005). The fork at `tptodorov/zenoh-nostd@rovi/liveliness-token` carries liveliness tokens, a liveliness subscriber, sample attachments and the zid accessor. The optional-`defmt` and `embassy-sync` 0.8 patches are in too, and the board added three more: a separate packet-metadata size and receive buffer for the embassy link, and ending the session on a peer `Close` (see the M8 results). Per-priority/reliability sequence numbers are in too. Open: whether upstream accepts the patches | M8 |
| Q17 | Clock sync for setpoint stamps | Without sync, staleness is judged by receive time and sequence. With sync, stamp-age checks could match `reference_timeout` | M8 bench, later |
| Q18 | BLE ATT MTU | **BlueZ half closed (2026-10-13):** negotiated 247 on all 12 connections (HCI trace: client 517, server 247), well above the 75 needed. Open: the iPhone's value (fall back to a compact 16-byte setpoint only if it is unreliable) | M8 bench, iPhone |
| Q19 | BLE streaming rate | **BlueZ half closed (2026-10-13):** connection interval 45 ms (BlueZ asked 30 to 50 ms), a 20 Hz stream ran for 22 s with no lease expiry, BLE echo median 70.5 ms, p95 94.3 ms. Open: the interval iOS grants. The ROS round trip on the board is median 7.7 to 8.0 ms, p95 about 11.5 ms | M8 bench, iPhone |
| Q21 | Wi-Fi client (station) mode next to the car AP | ADR-0008 chose AP only. The AP is infrastructure-free, one hop (UDP median 3.0 ms) and access-controlled by WPA2 with one station. It costs: clients lose their normal Wi-Fi (phones may avoid a network without internet), one client at a time, no NTP (Q17) or OTA, and the car can't join a ROS 2 network. **Lean (you, 2026-10-14): AP and station at once.** The AP stays the default for direct control. Station mode is optional, with credentials provisioned over BLE, for joining existing ROS 2 networks (the main reason), NTP, OTA and phones that keep their internet. Open: the AP must share the station's channel; UDP needs its own authentication on a shared network (supersedes part of ADR-0008); how the router endpoint and BLE radio switching work in station mode | you, after M5 |

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **Video teleoperation, cameras and onboard Linux compute.** M7 was dropped on 2026-10-06 (not a priority), and its experiment folder was removed. The ESP32-S3 stays the car's only network endpoint. Recover it from git history if video returns.
* **An RTOS or ESP-IDF** is not used; firmware is bare-metal Rust on Embassy ([ADR-0007](adr/0007-rust-embassy-no-rtos.md)).
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
