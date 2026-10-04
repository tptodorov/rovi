# Rovi plan

Current status, roadmap, and decisions deliberately deferred. For what Rovi is and why, see [`VISION.md`](VISION.md). For how work gets done, see the root [`README.md`](../README.md). This file is living — update it as milestones land, don't append a history; for *why* a specific hard-to-reverse decision was made, see [`adr/`](adr/).

## Current state (2026-10-04)

* Hardware parts package: received (2026-09-26) — see root [README.md](../README.md#hardware) for the part list and datasheets.
* Product software: the ESP32-S3 bring-up experiments exist; the low-level 4-wheel API and layered application are not implemented yet. Spec-driven product work starts after M1 as described in the root README; the API proposal can start now, while M3-M5 hardware evidence will constrain its final requirements and implementation.
* Preliminary current observation: one motor drew 120 mA from a 5 V power bank labeled 6500 mAh. Conditions are incomplete; see the [M4 results](../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results) and [mecanum reference](reference/mecanum-car/README.md#preliminary-measurement). It does not yet establish operating limits or runtime.

## Roadmap

* GitHub milestones: https://github.com/tptodorov/rovi/milestones
* **M1** (complete, bench-tested 2026-10-02) — prove one ESP32-S3 + one TB6612FNG channel drives one motor, on real hardware. See [`experiments/rovi-m1-tb6612/MILESTONE.md`](../experiments/rovi-m1-tb6612/MILESTONE.md).
* **M2** (complete, bench-tested 2026-10-02) — ESP32-S3 as a BLE peripheral: advertises a custom GATT service, a generic BLE client writes drive commands, firmware reacts observably (LED/serial). Bare board, no motor hardware — independent of M1. See [`experiments/rovi-m2-ble/`](../experiments/rovi-m2-ble/).
* **M3** (hardware-validated 2026-10-04; 5 of 6 acceptance items) — direct WPA2 car AP and shared BLE/Wi-Fi command ingress, board-only. Shared ingress, queue/stale/stop, watchdog-under-backlog and per-transport loss pass on the board. Range/distance runs, phone onboarding and a dedicated-radio latency run remain open. A 2 s TCP socket timeout caps Wi-Fi silence below the configured watchdog. See [`experiments/rovi-m3-wifi-queue/`](../experiments/rovi-m3-wifi-queue/).
* **M4** (proposed) — qualify four-motor electrical limits, power integrity, and standby fail-safe in one integrated bench setup. See [`experiments/rovi-m4-four-motor-safety/`](../experiments/rovi-m4-four-motor-safety/).
* **M5** (proposed) — characterize mecanum wheel mapping, open-loop behavior, and available feedback using the M4 setup. See [`experiments/rovi-m5-mecanum-motion/`](../experiments/rovi-m5-mecanum-motion/).
* **M6** (planned) — begin the low-level four-wheel API proposal now; finalize requirements and implement against M3-M5 evidence.
* **M7** (planned) — integrate direct teleoperation and the selected video path after the camera/compute decision. See [`experiments/rovi-m7-video-teleop/`](../experiments/rovi-m7-video-teleop/) and [next milestones and software options](research/next-milestones-and-software-options.md).

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
