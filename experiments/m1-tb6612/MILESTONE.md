# Milestones

## M1: TB6612FNG + ESP32-S3 motor test

**Status:** in progress  
**GitHub:** https://github.com/tptodorov/rovi/milestone/1

### Goal

Prove the new hardware stack: one **ESP32-S3-N16R8** CORE board drives one **TB6612FNG** channel and spins a single brushed DC motor forward, reverse, and stop.

### Recommended stack

* HAL: [`esp-hal`](https://crates.io/crates/esp-hal) (`esp32s3` + `unstable` for LEDC PWM)
* Motor driver: [`tb6612fng`](https://crates.io/crates/tb6612fng) `1.0.0` (`no_std`, `embedded-hal` 1.0)
* Flash tool: [`espflash`](https://crates.io/crates/espflash) over the board USB Type-C port

### Deliverables

1. Minimal wiring documented in [`WIRING.md`](WIRING.md)
2. Rust experiment in [`experiments/m1-tb6612/`](experiments/m1-tb6612/)
3. Build + flash instructions in that crate README
4. Manual test plan below

### Test plan

1. Wire per [`WIRING.md`](WIRING.md) (logic 3.3 V, separate motor supply, common GND). Keep motor unloaded or lightly loaded at first.
2. Install Rust ESP toolchain (`espup`) and `espflash` (see [`experiments/m1-tb6612/README.md`](experiments/m1-tb6612/README.md)).
3. `cd experiments/m1-tb6612 && cargo build --release`
4. Connect Type-C USB, put board in download mode if needed, then `cargo run --release` (or `espflash flash ...`).
5. Observe serial log: firmware should cycle **forward (2 s) → stop (1 s) → reverse (2 s) → stop (1 s)**.
6. Confirm motor direction matches log; if reversed, swap motor leads or swap AIN1/AIN2.
7. Power-cycle with STBY expected high in firmware; motor must not free-run unexpectedly when stopped (short-brake/stop modes).

### Acceptance

- [ ] Wiring matches the doc and was verified on the bench
- [ ] Firmware flashes via USB Type-C
- [ ] Forward / reverse / stop all work under firmware control
