# TB6612FNG dual motor driver board

The specific breakout board in hand, from an AliExpress listing whose title ambiguously reads "TB6612 DRV8833" (as if the two chips were interchangeable — they're not: DRV8833 has no separate PWM pin and a `SLEEP` pin instead of `STBY`). **Confirmed genuine TB6612FNG**: the physical IC package is marked `TB717A3` / `6612FNG`. Pinout below is transcribed from this specific board's own product photo — see [`pinout.jpg`](pinout.jpg) — which shows the **bottom** of the board; left/right below are mirrored from that photo to describe the board as viewed from the **top** (component side), matching the convention used everywhere else in this repo.

## Docs by level

| Level | Files | Notes |
| --- | --- | --- |
| **Board** (this breakout) | [`pinout.jpg`](pinout.jpg), the pin tables below | Two 8-pin headers; the only docs for what is physically wired. The breakout merges `VM1–3` into one `VM`, `AO1`/`PGND1` pairs into single pads, etc. |
| **Chip** (Toshiba TB6612FNG, SSOP24) | [`datasheet.md`](datasheet.md) | Package pins 1–24 (e.g. pin 24 = VM1, 23 = PWMA) — **not** the board header order. Authoritative for limits/timing/truth table only. Breakout components (caps, any protection) are not described. |

## Pin header (left)

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

## Pin header (right)

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

## Used so far

* **M1** (`experiments/rovi-m1-tb6612/`): channel A only (`PWMA`/`AIN1`/`AIN2`/`STBY`/`VM`/`VCC`/`GND`/`AO1`/`AO2`). Channel B (`PWMB`/`BIN1`/`BIN2`/`BO1`/`BO2`) unused. See [`WIRING.md`](../../../experiments/rovi-m1-tb6612/WIRING.md) and [`wiring.kicad_sch`](../../../experiments/rovi-m1-tb6612/wiring.kicad_sch).

## Datasheet (chip level — not the breakout board)

[`datasheet.md`](datasheet.md) / [`datasheet.pdf`](datasheet.pdf) — Toshiba TB6612FNG (2007-06-30), searchable Markdown transcription (PDF authoritative). Key limits: VM 4.5–13.5 V, Vcc 2.7–5.5 V, 1.0 A continuous per channel (VM ≥ 5 V), **fPWM max 100 kHz**, input high ≥ 0.7·Vcc (3.3 V logic OK on 3 V–5 V Vcc).

## Source

AliExpress listing: https://de.aliexpress.com/item/1005009294983213.html (seller HHKFYD Module Store / ICGOICIC brand). `pinout.jpg` is the listing's own product photo.
Datasheet: SparkFun mirror `cdn.sparkfun.com/datasheets/Robotics/TB6612FNG.pdf`, fetched 2026-09-30.
