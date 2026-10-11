# Rovi M4: four-motor power and safety qualification

**Status: prepared 2026-10-11, nothing run on hardware.** Wiring, parts list, test plan and a firmware draft are ready for the day the bench is back. The firmware cross-builds and passes clippy for the ESP32-S3; it has **never been flashed or run**. Host tests are development evidence only ([ADR-0002](../../docs/adr/0002-simulation-never-proves-hardware.md)). Every limit in it is a placeholder.

This experiment uses one integrated bench setup to characterize the received chassis motors, driver channels, power system, electrical noise, and standby fail-safe. Goal, risks and acceptance: [MILESTONE.md](MILESTONE.md).

| File | What |
| --- | --- |
| [WIRING.md](WIRING.md), [wiring.kicad_sch](wiring.kicad_sch) | Pin map, power path, INA226, fail-safe options and the recommended 74HC123 dead-man, logic-analyzer probe list |
| [BOM.md](BOM.md) | What is in the kit, what to buy before M4 |
| [TESTPLAN.md](TESTPLAN.md) | T0-T11, least to most dangerous; safety rules; O1, O4, O5, O7 mapped to tests |
| `src/` | Host-tested `no_std` core: all motor decisions |
| `firmware/` | ESP32-S3 shell (esp-hal, embassy via esp-rtos): pins, MCPWM, INA226, serial commands |
| `tools/m4ctl.py` | Laptop side: streams bench commands, saves the car log |
| `tools/gen_wiring.py` | Generates the schematic (`--check` compares KiCad's netlist with the intended nets) |

## Hardware setup

ESP32-S3-N16R8, two TB6612FNG boards (four channels), the four TT motors on the chassis (on blocks), a current-limited bench supply through a fuse, a kill switch and an INA226, and a 74HC123 that holds both drivers in standby unless the control tick keeps toggling a heartbeat. Full detail in [WIRING.md](WIRING.md). Do not hold a motor stalled or exceed verified IC, breakout-board, battery, wiring, or connector limits.

## Firmware

The core (`src/`) decides; the shell only applies its output. The control tick (10 ms) is the only code that drives pins.

* `motor::Core`: four wheel commands (FL, FR, RR, RL, −1..1) → one `Drive` per driver channel.
  * Standby from boot until `en`; `stby`, watchdog, non-finite command and over-current all return to standby, latched until the next `en`. Standby is immediate, never ramped, and clears the targets.
  * Finite commands are clamped; any NaN or infinity puts every driver in standby.
  * Watchdog: standby when no `w` command arrived for `command_timeout_ms`.
  * Slew limit on every change, including stops and reversals.
  * Wheel → channel mapping and inversion are configuration (O8).
* `command`: parses the serial lines below. `power`: INA226 register decoding and a min/max window. `events`: the log lines.
* Shell (`firmware/src/fw.rs`): STBY_EN low first thing at boot; standby drops STBY_EN before touching the bridges, enable raises it last. The tick toggles the heartbeat and feeds the RTC watchdog (placeholder 1 s, resets the chip). Coast = IN1 = IN2 = L, PWM 0; brake = IN1 = IN2 = H.

### Placeholders (`Config::BENCH`, `fw.rs`)

| Value | Placeholder | Measured by |
| --- | --- | --- |
| `max_duty_pm` (duty at 1.0) | 400 ‰ (M1's 40 % ramp top) | T5-T7, T11 (O1) |
| `slew_per_s` | 2.0 per s (0 → 1 in 0.5 s) | T8 (O7) |
| `command_timeout_ms` | 200 ms | T1, T9 (O4) |
| `trip_ma` (rail over-current) | 1500 mA; needs an R010 shunt, an R100 module reads only 819 mA | T6, T7 (O1) |
| `zero` | Coast | T10 (O5) |
| wheel mapping | FL = D1A, RL = D1B, FR = D2A, RR = D2B, none inverted | T5, then M5 (O8) |
| `SHUNT_MILLIOHM` | 100 (R100 module) | T0: read the marking |
| `RWDT_MS` | 1000 ms | T2 |
| 74HC123 `tw` (hardware) | ~110 ms (220 kΩ, 1 µF) | T2 |

### Bench commands (serial, one per line, 115200 on the COM port)

```text
en                      leave standby (wheels start at 0; the watchdog starts now)
stby                    standby
w <fl> <fr> <rr> <rl>   wheel commands -1.0..1.0 (clamped); renews the watchdog; "nan" and "inf" parse
zero brake|coast        zero-command mode
hang high|low           deliberate total hang with the heartbeat pin held high or low (fail-safe test)
```

`tools/m4ctl.py PORT LOG STEP...` sends them; `"w ... for MS"` streams at 20 Hz, `"wait MS"` sends nothing. Example: `tools/m4ctl.py /dev/ttyACM0 bench/x/car.log "en" "w 0.3 0 0 0 for 2000" "wait 500"`.

### Events

Same line format as [LOGGING.md](../../docs/LOGGING.md). `boot` gains a `reset` key (the chip's reset cause, e.g. `ChipPowerOn`, `SysBrownOut`, `SysRtcWdt`); `error` and `dropped` keep their meaning. M4's own events (not in M8's `sim/carlog.py` catalog, which will flag them as unknown):

| `ev` | Fields | When |
| --- | --- | --- |
| `motor` | `state` = `Standby`, `Enabled`; `reason` (Standby: `Boot`, `Disabled`, `Watchdog`, `NonFinite`, `OverCurrent`); `wheels_pm` = applied FL,FR,RR,RL | Mode change at once; while ramping at most every 50 ms, plus once when it settles |
| `power` | `bus_mv` (last), `bus_min_mv`, `ma` (mean), `ma_max` | Every 100 ms, over 10 ms samples |
| `cmd` | `op` = `Enable`, `Disable`, `Zero` (`mode`), `Hang` (`level`), `Bad` | A bench command other than `w`, or a malformed line |
| `error` | `src=ina226` | INA226 missing or failed: no current, so no over-current trip |

### Build and test

From this directory; the core and the firmware have separate lockfiles, as in M8.

```sh
cargo test --lib                                   # core host tests
cargo clippy --lib --tests -- -D warnings
python3 -m unittest discover -s tools -p 'test_*.py'   # m4ctl over a pseudo-terminal
python3 tools/gen_wiring.py --check                # regenerate the schematic, compare the netlist

source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH
cd firmware
cargo build --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc
cargo clippy --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc -- -D warnings
```

Flash (Todor, at the bench, per TESTPLAN T1): `cargo run --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc` from `firmware/`, as M1.

### Untried

Everything on the board. In particular: UART0 RX through esp-hal while esp-println writes TX on the same UART; whether opening the serial port resets the board; the RTC watchdog actually resetting on `hang`; the 74HC123 `tw` at 3.3 V; INA226 reads on this I²C setup; the MCPWM pin outputs on GPIO8/11/15. Not in this draft: Wi-Fi/BLE (MILESTONE test 7), acting on `INA_ALERT` or the `STBY` readback.

## Results

Record measured motor/driver/power data and safety observations in [`MILESTONE.md`](MILESTONE.md#results).
