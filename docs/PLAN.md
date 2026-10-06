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
2. **M8** (design in progress) — direct UDP setpoint protocol with exclusive single ownership and optional micro-ROS:
   * Setpoints are body velocity shaped like `TwistStamped`.
   * The latest setpoint wins; there is no queue. Each valid setpoint renews the watchdog lease.
   * The first client on BLE, UDP or the micro-ROS agent owns the car, and the other radio is shut down. When ownership is released, both reopen.
   * The car runs an AP only. ROS 2 is enabled by a compile-time `ros` feature (zenoh-nostd preferred per S2; XRCE fallback per S1).
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

## Open questions and decisions

Living list. Close an item when it is decided, move hard-to-reverse outcomes into an [ADR](adr/), and delete it here. Owner: **you** = project decision; **M8** = settled by the M8 design or bench.

| # | Question / decision | Options / current lean | Settled by |
| --- | --- | --- | --- |
| Q1 | ROS 2 backend for M8's `ros` feature | zenoh-nostd + `rmw_zenoh` (lean, [S2](research/s2-zenoh-nostd-spike.md)) vs XRCE via nano-ros or direct FFI ([S1](research/s1-micro-ros-nano-ros-spike.md)) | M8 design, after Q2 |
| Q2 | Can the car be visible in the ROS 2 graph with zenoh-nostd? | Implement `rmw_zenoh` liveliness tokens: patch a fork or upstream. If impractical, choose XRCE | M8 first risk item |
| Q3 | Re-arm after a safety stop | Explicit operator arm (research recommendation) vs auto-clear on the next valid setpoint (M3 behaviour) | you, M8 design |
| Q4 | Ownership handover | First-connect owner; on loss, both radios reopen (agreed). Open: should an expired BLE lease keep the connection (stop and disarm only) to avoid slow BlueZ rediscovery? Is there a grace period before the other radio shuts down? | M8 design |
| Q5 | UDP setpoint protocol details | Framing, session handshake, sequence, deadline vs relative TTL, setpoint rate, lease length, discovery of max speeds. TwistStamped-shaped body velocity in SI units (agreed) | M8 design |
| Q6 | UDP access control | WPA2 on the car AP only (as M3) vs a per-session token or message signing | you, M8 design |
| Q7 | BLE security | M2/M3 unpaired bench access vs LE Secure Connections bonding | you; deferred until there is a product milestone |
| Q8 | Fate of M3's TCP protocol v1 and `client.py` | Retire once M8 UDP lands (lean) vs keep for comparison | M8 |
| Q9 | iPhone client for M8 acceptance | Build an iOS app, use a generic UDP/BLE tool, or accept with a laptop client first and add the iPhone later | you |
| Q10 | Max speeds and units before M5 calibration | Treat max speed as full duty (placeholder) until M5 measures it | M8 design / M5 |
| Q11 | Open-loop vs closed-loop wheel control; ros2_control wheel-velocity mode | Depends on whether encoders exist | M5 |
| Q12 | Station mode (car joins an existing network) | Excluded for M8; revisit with an ADR if ROS on the lab network matters | you, after M8 |
| Q13 | Runtime vs compile-time configuration | Compile-time `ros` feature for M8 (agreed); runtime config later | after M8 |
| Q14 | M3 remainder (range, phone onboarding, dedicated-radio latency) | Run against M8's UDP firmware instead of M3, so effort isn't spent on the superseded TCP path | you |
| Q15 | `VISION.md` drift | Still says "shared command queue" and "teleoperation with a video stream". Update once the M8 design is approved | after M8 design |
| Q16 | zenoh-nostd patches | Carry a fork vs upstream: optional `defmt`, `embassy-sync` 0.8, larger sample storage | M8 |

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **Video teleoperation, cameras and onboard Linux compute.** M7 was dropped on 2026-10-06 (not a priority), and its experiment folder was removed. The ESP32-S3 stays the car's only network endpoint. Recover it from git history if video returns.
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
