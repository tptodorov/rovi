# Milestones

## M2: ESP32-S3 BLE peripheral

**Status:** complete — bench-tested 2026-10-02
**GitHub:** https://github.com/tptodorov/rovi/milestone/2

### Goal

Prove the ESP32-S3 can advertise a custom GATT service, accept drive-action writes from a generic BLE client, and report them through serial output and the onboard RGB LED. Use the bare board only; no motor hardware is connected. This experiment is independent of M1.

### Stack

* BLE controller: [`esp-radio`](https://crates.io/crates/esp-radio) `1.0.0-beta.1`
* BLE host: [`trouble-host`](https://crates.io/crates/trouble-host) `0.8.0`
* HAL/runtime: `esp-hal` `1.2.2` + `esp-rtos` `0.4.0`

### Command characteristic

The custom GATT service (`d47a0001-45b2-4d19-8db0-6bd87e9c0001`) exposes one write-only command characteristic (`d47a0002-45b2-4d19-8db0-6bd87e9c0001`). Each write must contain exactly one byte:

| Byte | Action | LED |
|---:|---|---|
| `0` | Stop | Off |
| `1` | Forward | Green |
| `2` | Reverse | Red |

Unknown values and malformed payloads are rejected. This is an experiment protocol, not the future low-level drive API.

### Deliverables

1. Independent Rust experiment in [`experiments/rovi-m2-ble/`](./)
2. Build, flash, and generic-client instructions in its README
3. Manual real-hardware test plan and acceptance checklist below

### Test plan

1. Connect the bare ESP32-S3 board over USB Type-C; do not connect motor hardware.
2. Build and flash with `cd experiments/rovi-m2-ble && cargo run --release`.
3. Confirm serial output reports that M2 started and is advertising as **Rovi M2**.
4. Scan with a generic BLE client, connect, and discover the custom GATT service and write characteristic.
5. Write `0`, `1`, and `2`; confirm serial output and LED state match the table.
6. Write an unknown byte and a payload with an invalid length; confirm they are rejected and the LED state does not change.
7. Disconnect and reconnect; confirm the peripheral resumes advertising and accepts writes again.

### Acceptance

Real hardware only; simulation does not validate BLE radio behavior (see [hardware-validation policy](../../README.md#hardware-validation-policy)).

- [x] Firmware builds for ESP32-S3
- [x] Firmware flashes and starts on the bare board
- [x] A generic BLE client discovers the advertised device and custom GATT service
- [x] Stop, forward, and reverse writes produce the expected LED and serial response
- [x] Invalid writes are rejected without changing the current indication
- [x] The board resumes advertising after disconnect
- [x] No motor hardware is required or connected

### Results

#### 2026-10-02 bench test

* **Board:** ESP32-S3 revision v0.2, 16 MB flash; connected over USB Type-C at `/dev/ttyACM0`.
* **Client:** laptop's BlueZ `bluetoothctl` GATT client; no motors connected.
* **Pass:** release build and flash completed. Serial boot output reported `Rovi M2 BLE peripheral started` and `Advertising`. The client discovered `Rovi M2`, service `d47a0001-45b2-4d19-8db0-6bd87e9c0001`, and write characteristic `d47a0002-45b2-4d19-8db0-6bd87e9c0001`.
* **Pass:** writes `00`, `01`, and `02` logged `stop (0)`, `forward (1)`, and `reverse (2)`. Write `03` was rejected and logged `Rejected unknown drive command: 3`. A malformed/empty write attempt was logged as rejected.
* **Pass:** after disconnect, serial output reported the disconnect and `Advertising`; reconnect succeeded and another stop command was accepted.
* **Pass:** Todor confirmed the onboard LED showed off for stop, green for forward, and red for reverse. Rejected writes leave the current indication unchanged.
