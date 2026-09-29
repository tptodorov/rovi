# M1 wiring: ESP32-S3-N16R8 ↔ one TB6612FNG

Schematic: [`wiring.kicad_sch`](wiring.kicad_sch) — open in KiCad for the visual diagram. This file is the quick-reference pin table and bring-up notes; the two should always agree.

Easiest bring-up: **one motor, one TB6612FNG channel A**, PWM + direction + STBY from the ESP32-S3.

## Power (do this first)

| Net | Connection | Notes |
| --- | --- | --- |
| ESP 3V3 | TB6612FNG `VCC` (logic) | Keep logic at 3.3 V |
| ESP GND | TB6612FNG `GND` | **Common ground required** |
| Motor battery / PSU (+) | TB6612FNG `VM` | Separate motor supply; max **15 V** per Toshiba datasheet |
| Motor battery / PSU (−) | TB6612FNG `GND` / ESP GND | Same ground as logic |
| Motor | TB6612FNG `AO1` / `AO2` | Channel A outputs |

Do **not** power the motor from the ESP 3.3 V rail.

## Signal map (channel A only)

| TB6612FNG pin | ESP32-S3 GPIO | Role |
| --- | --- | --- |
| `PWMA` | **GPIO7** | Speed (LEDC PWM, 20 kHz) |
| `AIN1` | **GPIO5** | Direction bit 1 |
| `AIN2` | **GPIO6** | Direction bit 2 |
| `STBY` | **GPIO4** | Standby (hold **HIGH** to enable) |
| `PWMB`, `BIN1`, `BIN2` | — | Unused for M1 (tie inactive or leave for later) |

These GPIOs are free on a typical ESP32-S3 DEV/CORE board and match the firmware defaults in this experiment.

## Verified against the actual boards received (2026-09-29)

* **ESP32-S3-N16R8 board**: vendor datasheet PDF's pinout diagram confirms GPIO4/5/6/7 exist on this specific board (left header) and aren't boot-strapping (GPIO0/3/45/46), USB (GPIO19/20/21), or RGB-LED (GPIO48) pins — no conflicts with M1's use of them. The logic-supply pin is silkscreened `3V3` (two of them; either works).
* **TB6612FNG board** (AliExpress, seller HHKFYD/ICGOICIC): product photos confirm the physical IC package is marked `TB6612FNG` and the breakout's silkscreen pin names exactly match the table above (`PWMA`/`AIN1`/`AIN2`/`STBY`/`VM`/`GND`/`AO1`/`AO2`, plus unused `PWMB`/`BIN1`/`BIN2`/`BO1`/`BO2`) — **except** the logic-supply pin, which this board silkscreens `VCC`, not `3V3` (fixed in `wiring.kicad_sch`; it's still the same net as the ESP's `3V3`, just named differently on each side).
* ⚠️ **The listing's title reads "TB6612 DRV8833"** — these are different, incompatible chips (DRV8833 has no separate PWM pin and a `SLEEP` pin instead of `STBY`). The listing's own photos are TB6612FNG, but confirm the marking on your *actual* received IC (`TB6612FNG` printed on the black package) before wiring — a marketplace listing mixing chip names in the title is a real risk of receiving the wrong part.

## Quick checklist

1. Headers soldered on the ESP32-S3 CORE board if your unit shipped unsoldered.
2. Common GND between ESP, driver, and motor supply.
3. `STBY` driven high in firmware (GPIO4).
4. Start with low PWM duty (~30–40%) and a current-limited motor supply.

## References

* Toshiba TB6612FNG datasheet: https://toshiba.semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG
* Espressif ESP32-S3 module datasheet: https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
* This specific ESP32-S3-N16R8 board's vendor datasheet (pinout diagram, page 32): user-supplied PDF, not in this repo
* This specific TB6612FNG board: https://de.aliexpress.com/item/1005009294983213.html (seller HHKFYD Module Store / ICGOICIC brand)
