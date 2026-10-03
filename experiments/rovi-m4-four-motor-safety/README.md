# Rovi M4: four-motor power and safety qualification

**Status:** proposed; no firmware package yet. Create the independent Cargo package when this experiment starts, following [`../README.md`](../README.md).

This experiment uses one integrated bench setup to characterize the received chassis motors, driver channels, power system, electrical noise, and standby fail-safe. Resolve unknown ratings before sustained or stall testing.

## Hardware setup

* ESP32-S3-N16R8 and four TB6612FNG channels (two dual-channel boards).
* Received four-motor mecanum chassis, secured with wheels clear of the bench for initial tests.
* Intended battery, protection, connectors, and logic-power arrangement.
* Current-limited supply, fused leads, multimeter, and oscilloscope or voltage logger; temperature measurement for drivers and motors.

Use short, current-limited tests for unknown motors. Do not hold a motor stalled or exceed verified IC, breakout-board, battery, wiring, or connector limits.

## Firmware and results

Firmware, pin mapping, wiring, and build/flash commands will be added when implementation begins. Record measured motor/driver/power data and safety observations in [`MILESTONE.md`](MILESTONE.md#results).
