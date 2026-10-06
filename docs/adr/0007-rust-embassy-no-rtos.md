---
status: accepted
---

# Firmware is bare-metal Rust on Embassy, without FreeRTOS or ESP-IDF

On an ESP32, the obvious choice would be ESP-IDF with FreeRTOS, in C or in Rust via `esp-idf-svc`. Rovi's firmware instead uses Rust `no_std` on esp-hal, with Embassy's async executor and esp-rtos (Espressif's small scheduler for the radio drivers). It uses esp-radio for Wi-Fi/BLE and trouble-host for BLE GATT. There is no general-purpose OS on the car. We decided this on 2026-10-06, recording the stack M1–M3 used and S2 built on:

* **Bench evidence.** M1–M3 passed on the real board with this stack.
* **ROS 2 path.** The chosen backend (zenoh-nostd, [ADR-0005](0005-zenoh-nostd-ros-backend.md)) is async Rust on embassy-net.
* **Lean low-level layer.** Cooperative async with no RTOS keeps most of the CPU free for higher layers, as `VISION.md` asks.

## Considered options

* **ESP-IDF + FreeRTOS, Rust via `esp-idf-svc` or C.** Rejected for now. It has the most mature Wi-Fi/BLE (NimBLE), OTA and the official micro-ROS component. But it means C or FFI and a heavier runtime, and we have no bench evidence for it.
* **Zephyr.** Rejected: its Rust support is immature, and we have no evidence for it on this board.

## Consequences

* We depend on esp-radio, which is still a beta with unstable APIs. We pin versions and accept upgrade churn.
* Concurrency is cooperative. A long-running task can delay others; time-critical work belongs in hardware peripherals (PWM/MCPWM) or an interrupt-priority executor.
* Revisit this decision if:
  * esp-radio's beta APIs block a milestone;
  * the product needs OTA updates or a BLE feature that only ESP-IDF provides;
  * motor control needs preemption beyond what interrupt executors give.
