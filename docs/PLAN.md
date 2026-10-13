# Rovi plan

Current status, roadmap, and decisions deliberately deferred. For what Rovi is and why, see [`VISION.md`](VISION.md). For how work gets done, see the root [`README.md`](../README.md). This file is living — update it as milestones land, don't append a history; for *why* a specific hard-to-reverse decision was made, see [`adr/`](adr/).

## Current state (2026-10-13)

* M8 ran on the board for the first time on 2026-10-13 and passes most of its acceptance: BLE (MTU 247, 45 ms interval, 10 of 10 scenarios), UDP over Wi-Fi (8 of 8), the Wi-Fi AP switching off and on around BLE (10 clean cycles), lease and window timing, and ROS 2 (graph visibility, controller scenario, reconnect in 3 to 4 s). The board found six defects, all fixed ([results](../experiments/rovi-m8-setpoint-ros/MILESTONE.md#results)). The bench has no motors yet: M4 is next.


* Hardware parts package: received (2026-09-26) — see root [README.md](../README.md#hardware) for the part list and datasheets. The bench setup is not currently available, so only software work is scheduled.
* Product software: the ESP32-S3 bring-up experiments exist; the low-level 4-wheel API is proposed ([M6](../spec/changes/add-four-wheel-api/proposal.md)) but not implemented, and the layered application is not started. Spec-driven product work starts after M1 as described in the root README; the API proposal can start now, while M3-M5 hardware evidence will constrain its final requirements and implementation.
* Preliminary current observation: one motor drew 120 mA from a 5 V power bank labeled 6500 mAh. Conditions are incomplete; see the [M4 results](../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results) and [mecanum reference](reference/mecanum-car/README.md#preliminary-measurement). It does not yet establish operating limits or runtime.

## Roadmap

* GitHub milestones: https://github.com/tptodorov/rovi/milestones
* Order (2026-10-06): the riskiest open decision comes first. No bench hardware is available right now, so only software work is scheduled. Each hardware item resumes when the bench setup is back. Milestone numbers are kept so links stay stable.

### Next: software only (laptop)

1. **S1 spike** (done 2026-10-06, laptop only) — yes, with conditions. nano-ros's XRCE backend cross-compiles for the S3, links next to Rovi's stack, and interoperates with ROS 2 Jazzy via the micro-ROS agent. Rovi would own the S3 port (13 platform symbols; blocking UDP must be bridged to embassy-net). Footprint and on-board behaviour are untested; M8 decides nano-ros vs the direct XRCE client. See [S1 findings](research/s1-micro-ros-nano-ros-spike.md).
   * **S2 spike** (done 2026-10-06, laptop only): native Rust/embassy [zenoh-nostd](https://github.com/eclipse-zenoh/zenoh-nostd) + `rmw_zenoh`. ROS 2 ↔ car data works in both directions over UDP, and a real session links into M3 at +101 KiB flash. The car is invisible to the ROS graph until liveliness tokens are added. The project is early-stage. It is the preferred path for M8's `ros` feature. See [S2 findings](research/s2-zenoh-nostd-spike.md).
2. **M8** (board-verified 2026-10-13, except ROS latency, stamped teleop, a second BLE central and the iPhone side of Q18/Q19) — setpoint protocol with a single owner and optional ROS 2. See [`experiments/rovi-m8-setpoint-ros/`](../experiments/rovi-m8-setpoint-ros/), [ADR-0004](adr/0004-single-owner-latest-setpoint.md), [ADR-0005](adr/0005-zenoh-nostd-ros-backend.md) and [ADR-0006](adr/0006-twiststamped-cdr-setpoint.md). Summary:
   * Setpoints are `TwistStamped`, CDR-encoded on BLE, UDP and ROS, with `mecanum_drive_controller`-compatible kinematics.
   * The latest setpoint wins; there is no queue. Each valid setpoint renews the lease.
   * The first controller to claim the car (BLE connect, UDP hello, or first ROS setpoint) owns it, and the other radio shuts down until release.
   * Motion needs an explicit arm, and every safety stop disarms. Claim and reclaim windows apply.
   * The car runs an AP only. ROS 2 runs over zenoh-nostd behind a compile-time `ros` feature; graph visibility is the first risk item.
   * The core and a sim car are tested on the laptop now; board acceptance waits for hardware. Graph visibility (the first risk item) passed on the laptop on 2026-10-07.
3. **M6** (proposal written 2026-10-10; not accepted, not implemented) — the low-level four-wheel API as an OpenSpec change: [`spec/changes/add-four-wheel-api/`](../spec/changes/add-four-wheel-api/proposal.md). It adds a `motor-control` layer (four normalised wheel commands, standby, limits) under M8's setpoint/ownership model. Motor limits, maximum wheel speed, open- vs closed-loop, brake mode and standby latency are OPEN (O1-O8) until M4/M5.

### Hardware (resume when the bench setup is available)

* **M8 bench remainder**: ROS round-trip latency ([#16](https://github.com/tptodorov/rovi/issues/16)), stamped teleop, a second BLE central (needs a second adapter), the iPhone side of Q18 and Q19. **M3 remainder**: range/distance runs and phone onboarding (the dedicated-radio latency is done: UDP median 3.0 ms). Then M4 ([`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/)), the first step that needs wiring.
* **M4** (proposed) — qualify four-motor electrical limits, power integrity, and standby fail-safe in one integrated bench setup. Highest physical risk; gates all motion. See [`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/).
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
| Q16 | zenoh-nostd patches | Rovi pins the fork by commit ([ADR-0010](adr/0010-zenoh-nostd-fork-pin.md)); nine draft PRs to upstream (#28 to #36) are separate and do not block it. The three fixes the board found are [#35](https://github.com/eclipse-zenoh/zenoh-nostd/pull/35) (embassy link buffer sizes) and [#36](https://github.com/eclipse-zenoh/zenoh-nostd/pull/36) (end the session on a peer `Close`). A pinned fork with optional `defmt`, `embassy-sync` 0.8 and a liveliness API (decided, ADR-0005). The fork at `tptodorov/zenoh-nostd@rovi/liveliness-token` carries liveliness tokens, a liveliness subscriber, sample attachments and the zid accessor. The optional-`defmt` and `embassy-sync` 0.8 patches are in too, and the board added three more: a separate packet-metadata size and receive buffer for the embassy link, and ending the session on a peer `Close` (see the M8 results). Per-priority/reliability sequence numbers are in too. Open: whether upstream accepts the patches | M8 |
| Q17 | Clock sync for setpoint stamps | Without sync, staleness is judged by receive time and sequence. With sync, stamp-age checks could match `reference_timeout` | M8 bench, later |
| Q18 | BLE ATT MTU | **BlueZ half closed (2026-10-13):** negotiated 247 on all 12 connections (HCI trace: client 517, server 247), well above the 75 needed. Open: the iPhone's value (fall back to a compact 16-byte setpoint only if it is unreliable) | M8 bench, iPhone |
| Q19 | BLE streaming rate | **BlueZ half closed (2026-10-13):** connection interval 45 ms (BlueZ asked 30 to 50 ms), a 20 Hz stream ran for 22 s with no lease expiry, BLE echo median 70.5 ms, p95 94.3 ms. Open: the interval iOS grants. ROS round-trip latency is tracked in [#16](https://github.com/tptodorov/rovi/issues/16) | M8 bench, iPhone |

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **Video teleoperation, cameras and onboard Linux compute.** M7 was dropped on 2026-10-06 (not a priority), and its experiment folder was removed. The ESP32-S3 stays the car's only network endpoint. Recover it from git history if video returns.
* **An RTOS or ESP-IDF** is not used; firmware is bare-metal Rust on Embassy ([ADR-0007](adr/0007-rust-embassy-no-rtos.md)).
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
