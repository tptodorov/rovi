# Rovi plan

Current status, roadmap, and decisions deliberately deferred. For what Rovi is and why, see [`VISION.md`](VISION.md). For how work gets done, see the root [`README.md`](../README.md). This file is living — update it as milestones land, don't append a history; for *why* a specific hard-to-reverse decision was made, see [`adr/`](adr/).

## Current state (2026-10-06)

* Hardware parts package: received (2026-09-26) — see root [README.md](../README.md#hardware) for the part list and datasheets. The bench setup is not currently available, so only software work is scheduled.
* Product software: the ESP32-S3 bring-up experiments exist; the low-level 4-wheel API and layered application are not implemented yet. Spec-driven product work starts after M1 as described in the root README; the API proposal can start now, while M3-M5 hardware evidence will constrain its final requirements and implementation.
* Preliminary current observation: one motor drew 120 mA from a 5 V power bank labeled 6500 mAh. Conditions are incomplete; see the [M4 results](../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results) and [mecanum reference](reference/mecanum-car/README.md#preliminary-measurement). It does not yet establish operating limits or runtime.

## Roadmap

* GitHub milestones: https://github.com/tptodorov/rovi/milestones
* Order (2026-10-06): the riskiest open decision comes first. No bench hardware is available right now, so only software work is scheduled. Each hardware item resumes when the bench setup is back. Milestone numbers are kept so links stay stable.

### Next: software only (laptop)

1. **S1 spike** (done 2026-10-06, laptop only) — yes, with conditions. nano-ros's XRCE backend cross-compiles for the S3, links next to Rovi's stack, and interoperates with ROS 2 Jazzy via the micro-ROS agent. Rovi would own the S3 port (13 platform symbols; blocking UDP must be bridged to embassy-net). Footprint and on-board behaviour are untested; M8 decides nano-ros vs the direct XRCE client. See [S1 findings](research/s1-micro-ros-nano-ros-spike.md).
   * **S2 spike** (done 2026-10-06, laptop only): native Rust/embassy [zenoh-nostd](https://github.com/eclipse-zenoh/zenoh-nostd) + `rmw_zenoh`. ROS 2 ↔ car data works in both directions over UDP, and a real session links into M3 at +101 KiB flash. The car is invisible to the ROS graph until liveliness tokens are added. The project is early-stage. It is the preferred path for M8's `ros` feature. See [S2 findings](research/s2-zenoh-nostd-spike.md).
2. **M8** (design approved 2026-10-06; not implemented) — setpoint protocol with a single owner and optional ROS 2. See [`experiments/rovi-m8-setpoint-ros/`](../experiments/rovi-m8-setpoint-ros/), [ADR-0004](adr/0004-single-owner-latest-setpoint.md), [ADR-0005](adr/0005-zenoh-nostd-ros-backend.md) and [ADR-0006](adr/0006-twiststamped-cdr-setpoint.md). Summary:
   * Setpoints are `TwistStamped`, CDR-encoded on BLE, UDP and ROS, with `mecanum_drive_controller`-compatible kinematics.
   * The latest setpoint wins; there is no queue. Each valid setpoint renews the lease.
   * The first controller to claim the car (BLE connect, UDP hello, or first ROS setpoint) owns it, and the other radio shuts down until release.
   * Motion needs an explicit arm, and every safety stop disarms. Claim and reclaim windows apply.
   * The car runs an AP only. ROS 2 runs over zenoh-nostd behind a compile-time `ros` feature; graph visibility is the first risk item.
   * The core and a sim car are tested on the laptop now; board acceptance waits for hardware.
3. **M6** (planned) — the low-level four-wheel API proposal (OpenSpec), using the M8 setpoint/ownership model and leaving the measured motor limits open until M4/M5.

### Hardware (resume when the bench setup is available)

* **M8 bench**: board-only acceptance of the above. **M3 remainder**: range/distance runs, phone onboarding, dedicated-radio latency.
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
| Q2 | Can the car be visible in the ROS 2 graph with zenoh-nostd? | Implement `rmw_zenoh` liveliness tokens: patch a fork or upstream. If impractical, revisit ADR-0005 (XRCE fallback) | M8 first risk item |
| Q6 | UDP access control | WPA2 on the car AP only (as M3) vs a per-session token or message signing | you, M8 design |
| Q7 | BLE security | M2/M3 unpaired bench access vs LE Secure Connections bonding | you; deferred until there is a product milestone |
| Q8 | When to retire M3's TCP protocol v1 and `client.py` | Retire once M8's UDP protocol passes on the bench (ADR-0004) | M8 bench |
| Q9 | iPhone client | M8 is accepted with laptop clients (BLE via bleak, a Python UDP client, ROS 2). Open: when and how to build the iPhone app | you, after M8 |
| Q11 | Open-loop vs closed-loop wheel control; ros2_control wheel-velocity mode | Depends on whether encoders exist | M5 |
| Q12 | Station mode (car joins an existing network) | Excluded for M8; revisit with an ADR if ROS on the lab network matters | you, after M8 |
| Q13 | Runtime vs compile-time configuration | Compile-time `ros` feature for M8 (agreed); runtime config later | after M8 |
| Q14 | M3 remainder (range, phone onboarding, dedicated-radio latency) | Run against M8's UDP firmware instead of M3, so effort isn't spent on the superseded TCP path | you |
| Q16 | zenoh-nostd patches | A pinned fork with optional `defmt`, `embassy-sync` 0.8 and a liveliness API (decided, ADR-0005). Open: whether upstream accepts them | M8 |
| Q17 | Clock sync for setpoint stamps | Without sync, staleness is judged by receive time and sequence. With sync, stamp-age checks could match `reference_timeout` | M8 bench, later |
| Q18 | BLE ATT MTU | CDR setpoints need MTU ≥ 75. Verify the negotiated value on iPhone and BlueZ with trouble-host; fall back to a compact 16-byte setpoint if it's unreliable | M8 bench |
| Q19 | BLE streaming rate | Check that iOS gives a connection interval suitable for 20 Hz setpoints | M8 bench, iPhone |

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **Video teleoperation, cameras and onboard Linux compute.** M7 was dropped on 2026-10-06 (not a priority), and its experiment folder was removed. The ESP32-S3 stays the car's only network endpoint. Recover it from git history if video returns.
* **An RTOS or ESP-IDF** is not used; firmware is bare-metal Rust on Embassy ([ADR-0007](adr/0007-rust-embassy-no-rtos.md)).
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
