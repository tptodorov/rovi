# Rovi plan

Current status, roadmap, and decisions deliberately deferred. For what Rovi is and why, see [`VISION.md`](VISION.md). For how work gets done, see the root [`README.md`](../README.md). This file is living — update it as milestones land, don't append a history; for *why* a specific hard-to-reverse decision was made, see [`adr/`](adr/).

## Current state (2026-09-29)

* Hardware parts package: received (2026-09-26) — see root [README.md](../README.md#hardware) for the part list and datasheets.
* Software: [`legacy-python/`](../legacy-python/) is the current working product — a flat, car-specific Raspberry Pi app. It predates the layered vision and stays the working product until the ESP32-S3 platform reaches parity.
* No code yet implements the low-level 4-wheel API or any layered application — that starts once M1 proves the basic hardware chain works.

## Roadmap

* GitHub milestones: https://github.com/tptodorov/rovi/milestones
* **M1** (in progress) — prove one ESP32-S3 + one TB6612FNG channel drives one motor, on real hardware, not just simulated. See [`experiments/rovi-m1-wokwi/MILESTONE.md`](../experiments/rovi-m1-wokwi/MILESTONE.md).
* **M2** (planned) — ESP32-S3 as a BLE peripheral: advertises a custom GATT service, a generic BLE client writes drive commands, firmware reacts observably (LED/serial). Bare board, no motor hardware — independent of M1. Not yet created.
* **M3** (placeholder) — ESP32-S3 as BLE central, pairing with a real BLE gamepad. Not yet designed; comes after M2.
* Beyond M1-M3: the low-level 4-wheel API becomes the first spec-driven product-development effort — see root [README.md](../README.md#how-we-work).

## Outstanding

* The OpenVPN client config that used to be committed at repo root (`tptodorov.homeservice.ovpn`) contained certificate/private-key material and was pushed to this public repo before being untracked — the key is still exposed in git history. Needs a credential rotation and a history rewrite (destructive, requires explicit sign-off) — not yet done.

## Deferred decisions (non-goals for now)

Considered and deliberately set aside — revisit only if the premise changes:

* **Autonomy** is not a committed near-term goal. The low-level API shouldn't preclude it, but nothing is architected around it yet.
* **Splitting the platform into its own repo** is deferred until a second real hardware application (boat, drone, etc.) needs to consume it independently of this car.
* **A Cargo workspace** for shared firmware code is deferred until the low-level API work actually starts — `experiments/` stays independent packages until then.
