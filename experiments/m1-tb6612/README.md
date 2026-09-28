# rovi-m1-tb6612

`no_std` firmware for **Milestone 1**: ESP32-S3-N16R8 + one TB6612FNG channel.

## Crate choice

Use **[`tb6612fng`](https://crates.io/crates/tb6612fng) `1.0.0`** — dedicated `no_std` driver for this IC (`Motor` / `Tb6612fng`, `DriveCommand::{Forward, Backward, Stop, Brake}`), built on `embedded-hal` 1.0. Pair with **`esp-hal`** (`esp32s3`, `unstable` for LEDC).

## Wokwi simulation

The project includes a Wokwi ESP32-S3 diagram with a logic analyzer on STBY, AIN1, AIN2, and PWMA. Wokwi does not model the TB6612FNG itself, so this checks firmware timing and GPIO/PWM signals rather than motor electrical behavior.

Install [`wokwi-cli`](https://docs.wokwi.com/wokwi-ci/cli-installation), then run `direnv allow` once from the repo root. Direnv loads the Wokwi token from `pass` and the Rust toolchain environment.

Run the simulation from `experiments/m1-tb6612`:

```bash
cargo build --release --features wokwi
wokwi-cli . --timeout 700 --fail-text 'PANIC' --expect-text 'reverse' \
  --vcd-file target/m1-signals.vcd
```

The run exits non-zero if the firmware panics, never reaches `reverse`, or times out.

The simulator can also be started through the Wokwi MCP configured in `.codex/config.toml`.

## Hardware setup

```bash
# once
cargo install espup
espup install
# follow the printed `. ./export-esp.sh` (or source it in your shell)
cargo install espflash
```

Wire the board per [`WIRING.md`](WIRING.md), then plug in **USB Type-C**.

## Build & flash

```bash
cd experiments/m1-tb6612
cargo build --release
cargo run --release   # uses espflash runner + serial monitor
```

If the port is not detected, hold **BOOT**, tap **RESET**, release **BOOT**, then retry `espflash flash --monitor target/xtensa-esp32s3-none-elf/release/rovi-m1-tb6612`.

## Behavior

After boot, the firmware loops: forward 2 s → stop 1 s → reverse 2 s → stop 1 s at ~40% duty on channel A (GPIO7 PWM, GPIO5/6 direction, GPIO4 STBY high).
