# Rovi M3: direct Wi-Fi and shared command ingress

**Status:** proposed; no firmware package yet. Create the independent Cargo package when this experiment starts, following [`../README.md`](../README.md).

This board-only experiment validates direct Wi-Fi control and shared command processing while keeping motors disconnected. BLE operation is already proven; use the existing BLE command source only to exercise the shared queue with Wi-Fi.

## Hardware setup

* ESP32-S3-N16R8 board powered over USB-C.
* The established BLE command source for shared-queue traffic. BLE client compatibility and command coverage are considered proven and are not retested here.
* Laptop or phone as a direct Wi-Fi client; use a separate test client if needed to generate concurrent traffic.
* No motor drivers or motors connected.

Do not store Wi-Fi credentials in source control. Select AP or station mode as an experiment variable and record the setup used for each run.

## Firmware and results

The firmware, wiring (if any), and build/flash commands will be added when implementation begins. Record measurements, client versions, configuration, and pass/fail evidence in [`MILESTONE.md`](MILESTONE.md#results).
