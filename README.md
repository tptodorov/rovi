## Rovi

A modular robotics hardware + software platform, currently building its first application: a remote-controlled 4 mecanum-wheel car. See [`docs/VISION.md`](docs/VISION.md) for what Rovi is and where it's going, and [`AGENTS.md`](AGENTS.md) if you're a coding agent working in this repo.

### Hardware status

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

### Software status

* Working product today: the Python app in [`legacy-python/`](legacy-python/).
* The ESP32-S3/Rust hardware platform (see [`docs/VISION.md`](docs/VISION.md)) is being bootstrapped milestone by milestone under [`experiments/`](experiments/) — see [`experiments/README.md`](experiments/README.md) for the convention and current milestone status.
* GitHub milestones: https://github.com/tptodorov/rovi/milestones
