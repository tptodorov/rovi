# M1 wiring: ESP32-S3-N16R8 ↔ one TB6612FNG

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

## Quick checklist

1. Headers soldered on the ESP32-S3 CORE board if your unit shipped unsoldered.
2. Common GND between ESP, driver, and motor supply.
3. `STBY` driven high in firmware (GPIO4).
4. Start with low PWM duty (~30–40%) and a current-limited motor supply.

## References

* Toshiba TB6612FNG datasheet: https://toshiba.semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG
* Espressif ESP32-S3 module datasheet: https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
