# M4 bill of materials and shopping list

Reference designators are those in [`wiring.kicad_sch`](wiring.kicad_sch) / [`WIRING.md`](WIRING.md). Values marked *placeholder* are starting points the bench may change.

**Prices:** only where a retailer page showed one when checked (2026-10-11). They are snapshots in that retailer's currency, not quotes; Swiss shop prices were not checked. "—" means not verified.

## Already in the received kit

From the root [README](../../README.md#hardware) and the M1 bench (photo in `rovi-m1-tb6612/docs/`).

| Part | Qty in kit | M4 use | Note |
| --- | --- | --- | --- |
| ESP32-S3-N16R8 CORE board (U1) | 1 | controller | Headers soldered for M1 |
| TB6612FNG breakout (U2, U3) | 5 | 2 used, 3 spares | Genuine TB6612FNG (`TB717A3`) |
| Mecanum chassis with 4 TT motors (M1-M4) | 1 | the load | No motor datasheet |
| 5 V power bank, 6500 mAh (stated) | 1 | later supply candidate, not the M4 bench supply | Output rating unknown |
| Breadboard, jumper wires, USB-C cable | used in M1 | logic wiring | Count jumpers: M4 needs about 30 |
| Raspberry Pi Zero 2 W | 1 | not used | — |

## Must buy before M4

| # | Part | Qty | For | Spec | Price seen |
| --- | --- | --- | --- | --- | --- |
| 1 | INA226 breakout module (U5) | 2 (1 spare) | rail current and voltage, TESTPLAN T5+ | Check the shunt marking: **R010 (10 mΩ, ±8.2 A)** wanted; the common R100 (100 mΩ) reads only ±0.82 A | AU$3.30 ([Little Bird](https://littlebirdelectronics.com.au/products/cjmcu-226-ina226-voltage-current-power-monitor-alarm-module-36v-bi-directional-i2c.md)); listings seen all had R100 |
| 2 | 10 mΩ shunt, 2512, 1 %, ≥ 1 W | 2 | swap onto an R100 module if no R010 module is found | — | — |
| 3 | USB logic analyzer, 8 ch, 24 MHz, fx2lafw / sigrok compatible (J1) | 1 | STBY, heartbeat, PWM timing (T1-T4, O4) | 5 V max inputs | 299 SEK incl. VAT ([Electrokit](https://www.electrokit.com/en/logic-analyser-8-channel-24mhz-usb-minib)) |
| 4 | 74HC123 dual retriggerable monostable, **DIP-16** (U4), e.g. TI CD74HC123E | 2 (1 spare) | hardware dead-man | 2-6 V, HC (CMOS levels) | 9 SEK ([Electrokit](https://www.electrokit.com/en/74hc123-dip-16-dual-retriggerable-monostable-multivibrator)); €0.71 at DigiKey, quoted by a search snippet only |
| 5 | Resistors, 1/4 W: 10 kΩ ×5 (R1-R3), 220 kΩ ×2 (R4), plus 100 kΩ and 470 kΩ to tune `tw` | 1 set | pull-downs, Rext | — | — |
| 6 | Capacitors: 1 µF ceramic X7R ×2 (C7), 100 nF ceramic ×8 (C3-C6, C8), 470 µF ≥ 16 V electrolytic ×2 (C1, C2) | 1 set | Cext, motor suppression, decoupling, bulk (*placeholders*) | — | — |
| 7 | Current-limited bench supply, CV/CC (PS1) | 1 | the only M4 motor supply | ≥ 0-15 V, ≥ 3 A, adjustable current limit, output on/off button. **Skip if you already own one** | — |
| 8 | Inline fuse holder + fast fuses 0.5 A, 1 A, 2 A, 3 A (F1) | 1 + 2 each | second protection after the PSU limit | Values are *placeholders*; final value from T7 | — |
| 9 | Toggle or rocker switch, ≥ 5 A DC (SW1) | 1 | kill switch on VM, within reach | — | — |
| 10 | Power resistor 10 Ω, ≥ 10 W | 1 | test load (T5): 0.5 A / 2.5 W at 5 V, before any motor | — | — |
| 11 | Wire 0.5 mm² (20 AWG) red/black, plus lever connectors (WAGO-type) or screw terminals | 2 m each, ~6 connectors | VM and GND distribution, motor leads; breadboard rails are not for motor current | — | — |
| 12 | Blocks to lift the chassis (wood or a box) | 1 set | wheels-off-the-ground tests (T6-T9) | — | — |

## Should have (borrow or own)

| Part | For | Note |
| --- | --- | --- |
| Multimeter | continuity, VM, 3V3 | Probably owned; M1 reported a current reading |
| IR thermometer or thermocouple probe | driver and motor temperatures (T8) | Without one, temperatures stay "unknown" in the results |
| Safety glasses | every powered test | — |
| Oscilloscope | 3V3 sag during starts (O7) | Optional. Without it, O7 rests on the INA226 bus minimum, `ev=boot reset=SysBrownOut` and the logic analyzer; a scope gives the real sag waveform |

Not needed for M4: encoders, IMU, a car battery (the battery choice follows from the T8/T9 currents).
