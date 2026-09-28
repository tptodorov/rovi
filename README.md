## Remote Controlled Car with 4 mecanum-wheels

### Status

* Hardware parts package: **received** (2026-09-26)

### Car

* 4x mecanum wheels
* 1x car frame
* 4x electric engines
* 2x 2-motor drivers
* 1-2 rechargeable batteries with USB ports
* Raspberry Pi Zero 2 W

### New parts received (2026-09-26)

* 1x **ESP32-S3-N16R8** development board (CORE board, USB Type-C; 2.4 GHz WiFi + Bluetooth; 16 MB flash / 8 MB PSRAM; pin headers not soldered). Label part ID `FBBA0086-001`.
* 5x **TB6612FNG** dual motor driver boards (1 A module class; Arduino-compatible; often used instead of L298N). Pack quantity: 5 pcs.

### Board references

#### ESP32-S3-N16R8 (Espressif)

N16R8 means **16 MB Quad-SPI flash** and **8 MB Octal-SPI PSRAM** on an ESP32-S3 module (commonly ESP32-S3-WROOM-1 / WROOM-1U). Dual-core Xtensa LX7, 2.4 GHz Wi-Fi + Bluetooth LE, USB Type-C on this CORE board; headers not pre-soldered on the received unit.

* Module datasheet (WROOM-1 / WROOM-1U, includes N16R8): https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
* Chip datasheet: https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf
* Technical reference manual: https://www.espressif.com/documentation/esp32-s3_technical_reference_manual_en.pdf

#### TB6612FNG (Toshiba)

Dual brushed-DC motor driver IC (two H-bridges). Official limits: **VM up to 15 V**, **1.2 A average / 3.2 A peak** per channel, low RON (~0.5 ohm typ.), CW / CCW / short-brake / stop, standby, thermal shutdown. Breakout boards labeled "1 A" are common retail packs of this IC.

* Product page: https://toshiba.semicon-storage.com/ap-en/semiconductor/product/motor-driver-ics/brushed-dc-motor-driver-ics/detail.TB6612FNG.html
* Datasheet (English): https://toshiba.semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG

### Control Software

The main Rovi product is the Raspberry Pi application, meant to run with Python 3.11. Its service entrypoint is [`roviservice.py`](roviservice.py).

The 4 motors are controlled via PWM pins connected to the motor drivers.
For each motor, one pin is used to control the direction and another pin to control the speed.

Implemented control methods:

* over Bluetooth PS4 controller directly connected to the car
* network via websockets from any internet host using a web page
* network via Zenoh from the local network using a keyboard

### Milestone 1

First bring-up goal: drive one DC motor with the ESP32-S3-N16R8 + one TB6612FNG (Rust `no_std`).

* GitHub milestone: https://github.com/tptodorov/rovi/milestone/1
* M1 plan, wiring, and experiment: [`experiments/m1-tb6612/`](experiments/m1-tb6612/)

Rust hardware experiments live under `experiments/`. Each is an independent Cargo package with its own `target/` build directory; the Rovi product remains a separate application.
