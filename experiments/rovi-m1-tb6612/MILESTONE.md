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

1. Wiring documented in [`wiring.kicad_sch`](wiring.kicad_sch) (schematic) and [`WIRING.md`](WIRING.md) (pin table, notes)
2. Rust experiment in [`experiments/rovi-m1-tb6612/`](experiments/rovi-m1-tb6612/)
3. Build + flash instructions in that crate README
4. Manual test plan below

### Test plan

1. Wire per [`wiring.kicad_sch`](wiring.kicad_sch) / [`WIRING.md`](WIRING.md) (logic 3.3 V, separate motor supply, common GND). Keep motor unloaded or lightly loaded at first.
2. Install Rust ESP toolchain (`espup`) and `espflash` (see [`experiments/rovi-m1-tb6612/README.md`](experiments/rovi-m1-tb6612/README.md)).
3. `cd experiments/rovi-m1-tb6612 && cargo build --release`
4. Connect Type-C USB, put board in download mode if needed, then `cargo run --release` (or `espflash flash ...`).
5. Observe serial log: firmware should cycle **forward (2 s) → stop (1 s) → reverse (2 s) → stop (1 s)**.
6. Confirm motor direction matches log; if reversed, swap motor leads or swap AIN1/AIN2.
7. Power-cycle with STBY expected high in firmware; motor must not free-run unexpectedly when stopped (short-brake/stop modes).

### Acceptance

Real hardware only — no simulator is used for this experiment (see [`../../README.md`](../../README.md#hardware-validation-policy)).

- [ ] Wiring matches the doc and was verified on the bench
- [ ] Firmware flashes via USB Type-C
- [ ] Forward / reverse / stop all work under firmware control

### Results

**2026-09-29** — Wiring verified on paper against the actual boards' vendor documentation (see [`WIRING.md`](WIRING.md#verified-against-the-actual-boards-received-2026-09-29)): GPIO4/5/6/7 confirmed present and conflict-free on this ESP32-S3-N16R8 board, TB6612FNG pin names confirmed against this specific AliExpress board's photos (one naming fix: the logic-supply pin is silkscreened `VCC` on this board, not `3V3` — `wiring.kicad_sch` corrected). Also confirmed the physical IC is genuinely a TB6612FNG (marked `TB717A3` / `6612FNG`), not the DRV8833 the listing title ambiguously suggested. This is paper verification only, not bench verification — it doesn't satisfy any acceptance item above, which all require the physical bench test.

_Bench test still pending. Fill in after running the test plan on real hardware: date, what was observed, pass/fail per acceptance item, links to logs/photos._
