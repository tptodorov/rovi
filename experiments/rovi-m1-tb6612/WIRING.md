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

## Verified against the actual boards received

Full pin catalogs, board photos, and sources for both boards: [`docs/reference/esp32-s3-n16r8/`](../../docs/reference/esp32-s3-n16r8/) and [`docs/reference/tb6612fng/`](../../docs/reference/tb6612fng/). Confirmed: GPIO4/5/6/7 exist and don't conflict with strapping/USB/RGB-LED pins; the TB6612FNG board's pin names match this doc exactly except the logic-supply pin, silkscreened `VCC` here (fixed in `wiring.kicad_sch`, still the same net as the ESP's `3V3`); the physical IC is a genuine TB6612FNG (marked `TB717A3`/`6612FNG`), not the DRV8833 the listing's title ambiguously suggested.

## Quick checklist

1. Headers soldered on the ESP32-S3 CORE board if your unit shipped unsoldered.
2. Common GND between ESP, driver, and motor supply.
3. `STBY` driven high in firmware (GPIO4).
4. Start with low PWM duty (~30–40%) and a current-limited motor supply.

## References

* Toshiba TB6612FNG datasheet: https://toshiba.semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG
* Espressif ESP32-S3 module datasheet: https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
* Board-specific pinouts and sources: [`docs/reference/`](../../docs/reference/)
