# TB6612FNG dual motor driver board

The specific breakout board in hand, from an AliExpress listing whose title ambiguously reads "TB6612 DRV8833" (as if the two chips were interchangeable — they're not: DRV8833 has no separate PWM pin and a `SLEEP` pin instead of `STBY`). **Confirmed genuine TB6612FNG**: the physical IC package is marked `TB717A3` / `6612FNG`. Pinout below is transcribed from this specific board's own product photo — see [`pinout.jpg`](pinout.jpg).

## Pin header (left)

| Pin | Function |
| --- | --- |
| PWMA | Channel A speed (PWM) |
| AIN2 | Channel A direction bit 2 |
| AIN1 | Channel A direction bit 1 |
| STBY | Standby — hold **HIGH** to enable either channel |
| BIN1 | Channel B direction bit 1 |
| BIN2 | Channel B direction bit 2 |
| PWMB | Channel B speed (PWM) |
| GND | Ground |

## Pin header (right)

| Pin | Function |
| --- | --- |
| VM | Motor supply, separate from logic, ≤15 V |
| VCC | Logic supply, 3.3–5 V (silkscreened `VCC` on this board, not `3V3`) |
| GND | Ground |
| AO1 | Channel A output 1 |
| AO2 | Channel A output 2 |
| BO2 | Channel B output 2 |
| BO1 | Channel B output 1 |
| GND | Ground |

## Used so far

* **M1** (`experiments/rovi-m1-tb6612/`): channel A only (`PWMA`/`AIN1`/`AIN2`/`STBY`/`VM`/`VCC`/`GND`/`AO1`/`AO2`). Channel B (`PWMB`/`BIN1`/`BIN2`/`BO1`/`BO2`) unused. See [`WIRING.md`](../../../experiments/rovi-m1-tb6612/WIRING.md) and [`wiring.kicad_sch`](../../../experiments/rovi-m1-tb6612/wiring.kicad_sch).

## Source

AliExpress listing: https://de.aliexpress.com/item/1005009294983213.html (seller HHKFYD Module Store / ICGOICIC brand). `pinout.jpg` is the listing's own product photo.
