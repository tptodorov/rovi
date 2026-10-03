# rovi-m2-ble

`no_std` firmware for **Milestone 2**: an ESP32-S3 BLE peripheral with a custom GATT service. A generic BLE client writes one-byte drive actions; the board reports accepted actions through serial output and its onboard RGB LED. No motor hardware is connected.

## Stack

* BLE controller: [`esp-radio`](https://crates.io/crates/esp-radio) `1.0.0-beta.1`
* BLE host/GATT server: [`trouble-host`](https://crates.io/crates/trouble-host) `0.8.0`
* Runtime: `esp-hal` `1.2.2` + `esp-rtos` `0.4.0`

The BLE API is experimental in this stack.

## BLE service

* Service UUID: `d47a0001-45b2-4d19-8db0-6bd87e9c0001`
* Drive-command characteristic UUID: `d47a0002-45b2-4d19-8db0-6bd87e9c0001` (write)

## Build and flash

Install the Espressif Rust toolchain and `espflash` once (see the [Rust on ESP book](https://docs.espressif.com/projects/rust/)):

```bash
cargo install espup espflash
espup install
# Source the export-esp.sh path printed by espup.
```

Flash the bare board over its USB Type-C connector:

```bash
cd experiments/rovi-m2-ble
cargo build --release
cargo run --release
```

If the port is not detected, hold **BOOT**, tap **RESET**, release **BOOT**, then retry.

## Try it

Use a generic BLE client such as nRF Connect. Scan for **Rovi M2**, connect, discover the custom service, and write one byte to its drive-command characteristic:

| Byte | Action | LED |
|---:|---|---|
| `0` | Stop | Off |
| `1` | Forward | Green |
| `2` | Reverse | Red |

The LED is blue while advertising or after disconnect. The serial monitor logs each accepted command. Other values and payload lengths are rejected. These actions only exercise the BLE command path; they do not drive a motor.
