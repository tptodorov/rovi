# rovi-m1-tb6612

`no_std` firmware for **Milestone 1**: ESP32-S3-N16R8 + one TB6612FNG channel, real hardware only. Proves the [`tb6612fng`](https://crates.io/crates/tb6612fng) driver crate against real wiring — see [`MILESTONE.md`](MILESTONE.md) for the goal and acceptance checklist, [`wiring.kicad_sch`](wiring.kicad_sch) for the schematic, and [`WIRING.md`](WIRING.md) for the pin table and bring-up notes.

## Crate choice

Use **[`tb6612fng`](https://crates.io/crates/tb6612fng) `1.0.0`** — dedicated `no_std` driver for this IC (`Motor` / `Tb6612fng`, `DriveCommand::{Forward, Backward, Stop, Brake}`), built on `embedded-hal` 1.0. Pair with **`esp-hal`** (`esp32s3`, `unstable` for LEDC).

## Hardware setup

```bash
# once
cargo install espup
espup install
# follow the printed `. ./export-esp.sh` (or source it in your shell)
cargo install espflash
```

Wire the board per [`wiring.kicad_sch`](wiring.kicad_sch) / [`WIRING.md`](WIRING.md), then plug in **USB Type-C**.

## Build & flash

```bash
cd experiments/rovi-m1-tb6612
cargo build --release
cargo run --release   # uses espflash runner + serial monitor
```

If the port is not detected, hold **BOOT**, tap **RESET**, release **BOOT**, then retry `espflash flash --monitor target/xtensa-esp32s3-none-elf/release/rovi-m1-tb6612`.

## Behavior

After boot, the firmware loops: forward 2 s → stop 1 s → reverse 2 s → stop 1 s at ~40% duty on channel A (GPIO7 PWM, GPIO5/6 direction, GPIO4 STBY high).
