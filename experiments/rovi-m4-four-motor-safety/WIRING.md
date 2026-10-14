# M4 wiring

M1's wiring ([`../rovi-m1-tb6612/WIRING.md`](../rovi-m1-tb6612/WIRING.md), [schematic](../rovi-m1-tb6612/wiring.kicad_sch), [photo](../rovi-m1-tb6612/docs/breadboard-setup.jpg)), bench-proven on 2026-10-02, driven by the M8 firmware. No extra parts: no current monitor, no dead-man chip, no logic analyzer ([ADR-0014](../../docs/adr/0014-m4-minimal-wiring-software-fail-safe.md)). The earlier M4 wiring draft (INA226, 74HC123, four-channel pin map) is retired.

## Step 1: one driver, one motor (now)

| TB6612FNG pin | Connects to | Role |
| --- | --- | --- |
| `VCC` (L2) | ESP32 `3V3` (L1) | Driver logic, 3.3 V |
| `GND` (L3) | ESP32 `GND` (L22) and power bank − | **Common ground** |
| `VM` (L1) | Power bank + (5 V) | Motor supply |
| `AO1` (L4) / `AO2` (L5) | Motor | Channel A |
| `PWMA` (R1) | **GPIO7** (L7) | Speed, MCPWM 20 kHz |
| `AIN1` (R3) | **GPIO5** (L5) | Direction |
| `AIN2` (R2) | **GPIO6** (L6) | Direction |
| `STBY` (R4) | **GPIO4** (L4) | High only while the car is armed |
| `PWMB`, `BIN1`, `BIN2`, `BO1`, `BO2` | — | Unused in step 1 |

* **ESP32 power:** USB-C from the laptop on the COM (CH343) port, for flashing and the log. Nothing on `5Vin`, and never 5 V into `3V3`.
* **Motor power:** the 5 V power bank on `VM`, as in M1. Its own over-current shutdown is the current limit, and **unplugging its cable is the kill switch**: keep it within reach.
* **Firmware mapping:** channel A drives the front-left wheel (`CHANNELS` in `../rovi-m8-setpoint-ros/firmware/src/fw.rs`). A forward `vx` turns it forward.
* **Fail-safe without parts:** `STBY` is low from boot, low unless armed, and the chip's RTC watchdog resets the board 1 s after the control tick stops. During reset and while unpowered, the TB6612's internal pull-down holds `STBY` low.

## Step 2: four motors, two drivers (later)

The second channel of driver 1 and both channels of driver 2, with `STBY` of both drivers on GPIO4. The pins are chosen from [`docs/reference/esp32-s3-n16r8/`](../../docs/reference/esp32-s3-n16r8/) and checked on the board before step 2; the firmware change is the `CHANNELS` table and three more PWM pins.
