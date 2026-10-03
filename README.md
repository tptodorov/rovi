## Rovi

A modular robotics hardware + software platform, currently building its first application: a remote-controlled 4 mecanum-wheel car. See [`docs/VISION.md`](docs/VISION.md) for what Rovi is and where it's going, [`docs/PLAN.md`](docs/PLAN.md) for current status and the roadmap, and [`AGENTS.md`](AGENTS.md) if you're a coding agent working in this repo.

### Hardware

#### Car

* 4x mecanum wheels
* 1x car frame
* 4x electric engines
* 2x 2-motor drivers
* 1-2 rechargeable batteries with USB ports
* Raspberry Pi Zero 2 W

#### ESP32-S3 platform

* 1x **ESP32-S3-N16R8** development board (CORE board, USB Type-C; 2.4 GHz WiFi + Bluetooth; 16 MB flash / 8 MB PSRAM; pin headers not soldered). Label part ID `FBBA0086-001`.
* 5x **TB6612FNG** dual motor driver boards (1 A module class; Arduino-compatible; often used instead of L298N). Pack quantity: 5 pcs.

#### Board references

For the pinout of the *specific boards received* (not just the chip family), see [`docs/reference/`](docs/reference/).

##### ESP32-S3-N16R8 (Espressif)

N16R8 means **16 MB Quad-SPI flash** and **8 MB Octal-SPI PSRAM** on an ESP32-S3 module (commonly ESP32-S3-WROOM-1 / WROOM-1U). Dual-core Xtensa LX7, 2.4 GHz Wi-Fi + Bluetooth LE, USB Type-C on this CORE board; headers not pre-soldered on the received unit.

* Module datasheet (WROOM-1 / WROOM-1U, includes N16R8): https://www.espressif.com/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
* Chip datasheet: https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf
* Technical reference manual: https://www.espressif.com/documentation/esp32-s3_technical_reference_manual_en.pdf

##### TB6612FNG (Toshiba)

Dual brushed-DC motor driver IC (two H-bridges). Official limits: **VM up to 15 V**, **1.2 A average / 3.2 A peak** per channel, low RON (~0.5 ohm typ.), CW / CCW / short-brake / stop, standby, thermal shutdown. Breakout boards labeled "1 A" are common retail packs of this IC.

* Product page: https://toshiba.semicon-storage.com/ap-en/semiconductor/product/motor-driver-ics/brushed-dc-motor-driver-ics/detail.TB6612FNG.html
* Datasheet (English): https://toshiba.semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG

### How we work

#### Contribution method

All changes land via pull request against `main` — no direct pushes to `main`. Work happens on a branch, gets its own PR, and is reviewed before merge.

#### Hardware-validation policy

Simulation never counts as proof for hardware a simulator doesn't model electrically. Wokwi, for example, simulates ESP32-S3 GPIO/PWM timing but not the TB6612FNG's electrical behavior, and doesn't simulate BLE radio at all (tracked upstream: https://github.com/wokwi/wokwi-features/issues/225). A milestone touching hardware isn't accepted until it's been physically bench-verified, whatever a simulator says. This applies to every hardware component (drivers, sensors, radios, etc.), not just one.

#### Hardware design assumptions

Never assume a wiring diagram, schematic, or PCB layout targets bare chips. Ask first: dev boards or bare chips, and if dev boards, which specific ones (get the exact board/listing, not just the chip family) — then design against [`docs/reference/`](docs/reference/) for those specific boards, not a generic chip datasheet. A bare-chip symbol with a handful of arbitrarily-numbered pins doesn't correspond to anything printed on a real dev board, and re-deriving the actual physical pinout after the fact means redoing the design. See [ADR 0003](docs/adr/0003-confirm-devboard-vs-chip-before-hw-design.md).

#### Two development tracks

* **Exploratory hardware bring-up** (`experiments/`) stays lightweight: each milestone is an independent Cargo package with colocated `README.md`/`MILESTONE.md`/`WIRING.md` (see [`experiments/README.md`](experiments/README.md)). Results get tracked, not just planned — after bench testing, the milestone's `MILESTONE.md` gets a `## Results` section (what was observed, pass/fail, date, links to logs/photos), and `experiments/README.md`'s index gets a status marker.
* **Product development** — the low-level 4-wheel API and anything built on top of it — follows spec-driven development: a change proposal, spec deltas, and a task list under `spec/changes/{change-id}/{proposal.md,tasks.md,specs/*.md}` (the OpenSpec convention, via the `openspec-implementation` skill). This starts once M1 proves the hardware chain works on real hardware; exploratory hardware bring-up remains outside that process.
