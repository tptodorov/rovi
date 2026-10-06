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
2. **M8** (design in progress) — direct UDP setpoint protocol with exclusive single ownership and optional micro-ROS:
   * Setpoints are body velocity shaped like `TwistStamped`.
   * The latest setpoint wins; there is no queue. Each valid setpoint renews the watchdog lease.
   * The first client on BLE, UDP or the micro-ROS agent owns the car, and the other radio is shut down. When ownership is released, both reopen.
   * The car runs an AP only. micro-ROS is enabled by a compile-time `ros` feature.
   * Mecanum kinematics feed a `WheelOutput` trait.
   * The core is host-tested now; board acceptance waits for hardware.
3. **M6** (planned) — the low-level four-wheel API proposal (OpenSpec), using the M8 setpoint/ownership model and leaving the measured motor limits open until M4/M5.

### Hardware (resume when the bench setup is available)

* **M8 bench**: board-only acceptance of the above. **M3 remainder**: range/distance runs, phone onboarding, dedicated-radio latency.
* **M4** (proposed) — qualify four-motor electrical limits, power integrity, and standby fail-safe in one integrated bench setup. Highest physical risk; gates all motion. See [`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/).
* **M5** (proposed) — characterize mecanum wheel mapping, open-loop behavior, and available feedback (encoders decide open- vs closed-loop) using the M4 setup. See [`experiments/rovi-m5-mecanum-motion/`](../experiments/rovi-m5-mecanum-motion/).

### Done

* **M1** (complete, bench-tested 2026-10-02) — prove one ESP32-S3 + one TB6612FNG channel drives one motor, on real hardware. See [`experiments/rovi-m1-tb6612/MILESTONE.md`](../experiments/rovi-m1-tb6612/MILESTONE.md).
* **M2** (complete, bench-tested 2026-10-02) — ESP32-S3 as a BLE peripheral: advertises a custom GATT service, a generic BLE client writes drive commands, firmware reacts observably (LED/serial). Bare board, no motor hardware — independent of M1. See [`experiments/rovi-m2-ble/`](../experiments/rovi-m2-ble/).
* **M3** (hardware-validated 2026-10-04; 5 of 6 acceptance items) — direct WPA2 car AP and shared BLE/Wi-Fi command ingress, board-only. Shared ingress, queue/stale/stop, watchdog-under-backlog and per-transport loss pass on the board. A 2 s TCP socket timeout caps Wi-Fi silence below the configured watchdog. M8 supersedes its TCP+FIFO design. See [`experiments/rovi-m3-wifi-queue/`](../experiments/rovi-m3-wifi-queue/).

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **M7: video teleoperation, and any camera or onboard Linux compute** (decided 2026-10-06: none planned at this stage). The ESP32-S3 stays the car's only network endpoint. [`experiments/rovi-m7-video-teleop/`](../experiments/rovi-m7-video-teleop/) is kept for when this changes.
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
