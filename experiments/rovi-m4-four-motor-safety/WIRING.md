# M4 wiring: four motors, two TB6612FNG drivers

Starts from M1's wiring ([`../rovi-m1-tb6612/WIRING.md`](../rovi-m1-tb6612/WIRING.md), [photo](../rovi-m1-tb6612/docs/breadboard-setup.jpg)), bench-proven in M1 and in M4 step 1. Everything new is on the ESP32's **left header**, next to M1's wires. No extra parts ([ADR-0014](../../docs/adr/0014-m4-minimal-wiring-software-fail-safe.md)).

Header positions: ESP32 `L1`-`L22` top to bottom on the left header (3V3, 3V3, RST, 4, 5, 6, 7, 15, 16, 17, 18, 8, 3, 46, 9, 10, 11, 12, 13, 14, 5Vin, GND). TB6612 board: left header `VM, VCC, GND, AO1, AO2, BO2, BO1, GND`; right header `PWMA, AIN2, AIN1, STBY, BIN1, BIN2, PWMB, GND` ([board reference](../../docs/reference/tb6612fng/README.md)).

## Signals: ESP32 → drivers

| ESP32 GPIO (header) | Driver 1 (D1) | Driver 2 (D2) | Wheel | Status |
| --- | --- | --- | --- | --- |
| **4** (L4) | `STBY` | `STBY` | both drivers | D1 wired (M1); **add a jumper D1 `STBY` → D2 `STBY`** |
| **5** (L5) | `AIN1` | | front-left | wired (M1) |
| **6** (L6) | `AIN2` | | front-left | wired (M1) |
| **7** (L7) | `PWMA` | | front-left | wired (M1) |
| **15** (L8) | `PWMB` | | rear-left | **new** |
| **16** (L9) | `BIN1` | | rear-left | **new** |
| **17** (L10) | `BIN2` | | rear-left | **new** |
| 18 (L11) | — | — | | **skip**: high glitch at power-up |
| **8** (L12) | | `PWMA` | front-right | **new** |
| 3, 46 (L13, L14) | — | — | | **skip**: boot strapping pins |
| **9** (L15) | | `AIN1` | front-right | **new** |
| **10** (L16) | | `AIN2` | front-right | **new** |
| **11** (L17) | | `PWMB` | rear-right | **new** |
| **12** (L18) | | `BIN1` | rear-right | **new** |
| **13** (L19) | | `BIN2` | rear-right | **new** |

## Power and motors

| From | To | Status |
| --- | --- | --- |
| ESP32 `3V3` (L1) | D1 `VCC` | wired (M1) |
| ESP32 `3V3` (L2) | D2 `VCC` | **new** |
| ESP32 `GND` (L22) | D1 `GND` and D2 `GND` | D1 wired; **add D2** |
| Power bank + (5 V) | D1 `VM` and D2 `VM` | D1 wired; **add D2** |
| Power bank − | the same ground as above | wired |
| D1 `AO1`/`AO2` | front-left motor | wired (M1's motor) |
| D1 `BO1`/`BO2` | rear-left motor | **new** |
| D2 `AO1`/`AO2` | front-right motor | **new** |
| D2 `BO1`/`BO2` | rear-right motor | **new** |

```text
                 ESP32-S3 (left header)                       power bank 5 V
                 L1 3V3 ───────────── D1 VCC                      +  ── D1 VM ── D2 VM
                 L2 3V3 ───────────── D2 VCC                      −  ── GND (common)
                 L4  4 ── STBY ─┬──── D1 STBY
                                └──── D2 STBY
   D1 (left)     L5  5 ── AIN1        L8  15 ── PWMB    D1 AO ── front-left motor
                 L6  6 ── AIN2        L9  16 ── BIN1    D1 BO ── rear-left motor
                 L7  7 ── PWMA        L10 17 ── BIN2
   D2 (right)    L12  8 ── PWMA       L17 11 ── PWMB    D2 AO ── front-right motor
                 L15  9 ── AIN1       L18 12 ── BIN1    D2 BO ── rear-right motor
                 L16 10 ── AIN2       L19 13 ── BIN2
                 L22 GND ──────────── D1 GND, D2 GND, power bank −
```

## Notes

* **Why these pins:** consecutive free pins on the left header. GPIO15/16/17 and 8-13 only show a 60 µs *low* glitch at power-up, which cannot enable a driver; GPIO18 shows a *high* glitch and is skipped; GPIO3/46 are strapping pins ([SoC datasheet](../../docs/reference/esp32-s3-n16r8/soc-datasheet.md) Table 2-2). GPIO14 (L20) stays free.
* **PWM:** MCPWM0 at 20 kHz: operator 0 drives `D1 PWMA` (GPIO7) and `D1 PWMB` (GPIO15), operator 1 drives `D2 PWMA` (GPIO8) and `D2 PWMB` (GPIO11).
* **Firmware mapping** (`CHANNELS` in `../rovi-m8-setpoint-ros/firmware/src/fw.rs`): D1 A = front-left, D1 B = rear-left, D2 A = front-right, D2 B = rear-right; no motor inverted yet. The motors on the right side face the other way, so they will likely turn backwards for a forward command; step 2's first test records that, and the fix is one `invert: true` per motor.
* **Motor wire polarity** doesn't matter for safety: a reversed motor just turns the other way, which the inversion flag corrects.
* **Kill switch:** unplugging the power bank cable stops every motor. With four motors the power bank supplies about 0.6 A running; its own protection is the limit.
* **Fail-safe:** one `STBY` line for both drivers: low from boot, low unless armed, and during reset or with the ESP32 unpowered the TB6612's internal pull-down holds it low. The 1 s RTC watchdog resets the chip if the control tick stops.
