# Milestones

## M1: TB6612FNG + ESP32-S3 motor test

**Status:** complete — bench-tested on real hardware  
**GitHub:** https://github.com/tptodorov/rovi/milestone/1

### Goal

Prove the new hardware stack: one **ESP32-S3-N16R8** CORE board drives one **TB6612FNG** channel and spins a single brushed DC motor forward, reverse, and stop.

### Recommended stack

* HAL: [`esp-hal`](https://crates.io/crates/esp-hal) (`esp32s3` + `unstable` for MCPWM — the chip's native motor-control PWM peripheral)
* Motor driver: [`tb6612fng`](https://crates.io/crates/tb6612fng) `1.0.0` (`no_std`, `embedded-hal` 1.0)
* Flash tool: [`espflash`](https://crates.io/crates/espflash) over the board USB Type-C port

### Deliverables

1. Wiring documented in [`wiring.kicad_sch`](wiring.kicad_sch) (schematic) and [`WIRING.md`](WIRING.md) (pin table, notes)
2. Rust experiment in [`experiments/rovi-m1-tb6612/`](experiments/rovi-m1-tb6612/)
3. Build + flash instructions in that crate README
4. Manual test plan below

### Test plan

1. Wire per [`wiring.kicad_sch`](wiring.kicad_sch) / [`WIRING.md`](WIRING.md) (logic 3.3 V, separate motor supply, common GND). Keep motor unloaded or lightly loaded at first.
2. Install Rust ESP toolchain (`espup`) and `espflash` (see [`experiments/rovi-m1-tb6612/README.md`](experiments/rovi-m1-tb6612/README.md)).
3. `cd experiments/rovi-m1-tb6612 && cargo build --release`
4. Connect Type-C USB, put board in download mode if needed, then `cargo run --release` (or `espflash flash ...`).
5. Observe the serial log and onboard LED while firmware ramps forward and reverse from 5% to 40%, then back down, with a stop between directions.
6. Confirm the LED identifies commanded direction and that the motor direction matches it.
7. Confirm the motor stops when STBY is disabled and resumes when STBY is enabled.

### Acceptance

Real hardware only — no simulator is used for this experiment (see [`../../README.md`](../../README.md#hardware-validation-policy)).

- [x] Wiring was assembled and the documented motor control path was verified on the bench
- [x] Firmware builds and flashes via the board's COM USB-C connector
- [x] Forward / reverse / stop work under firmware control
- [x] Speed ramps up and down smoothly in both directions
- [x] STBY disables and re-enables the motor as expected
- [x] Onboard LED distinguishes forward, reverse, and standby

### Results

**2026-09-29** — Wiring verified on paper against the actual boards' vendor documentation (see [`WIRING.md`](WIRING.md#verified-against-the-actual-boards-received-2026-09-29)): GPIO4/5/6/7 confirmed present and conflict-free on this ESP32-S3-N16R8 board, TB6612FNG pin names confirmed against this specific AliExpress board's photos (one naming fix: the logic-supply pin is silkscreened `VCC` on this board, not `3V3` — `wiring.kicad_sch` corrected). Also confirmed the physical IC is genuinely a TB6612FNG (marked `TB717A3` / `6612FNG`), not the DRV8833 the listing title ambiguously suggested. This was paper verification; the bench test below completed the hardware validation.

**2026-10-02 — PASS, real-hardware bench test.** Firmware built and flashed over the board's COM USB-C connector; the ESP32-S3 booted and its serial output showed the expected forward/reverse ramps and standby transitions. The user confirmed the motor runs clockwise on the green forward indication and in reverse on red, the onboard LED shows blue while standby is disabled, speed increases and decreases smoothly, and direction and standby controls behave as expected. The documented wiring path, firmware flashing, forward/reverse/stop control, speed ramp, standby, and direction indication acceptance criteria are all satisfied.
