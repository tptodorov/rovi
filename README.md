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
* **Product development** — the low-level 4-wheel API and anything built on top of it — follows the [product specification standard](docs/SPECIFICATION.md): native `openspec/`, a pinned CLI, proposal/specs/design/tasks, and separate implementation and physical acceptance gates. Exploratory hardware bring-up remains outside that process.
  For a human-friendly view, open the generated [specification reader](openspec/index.html) locally in your browser.

#### Product code

The M6 layout follows its [design](openspec/changes/add-four-wheel-api/design.md#package-boundaries). Host builds and the sim car are development evidence only ([ADR-0002](docs/adr/0002-simulation-never-proves-hardware.md)). The M8 experiment stays the frozen bench reference until the product firmware is accepted.

| Path | What | Build and test |
| --- | --- | --- |
| `crates/rovi-core` | Device core (`no_std`): codecs, arbiter, kinematics, graph, device | From the repo root: `cargo test --locked`, `cargo clippy --locked --workspace --all-targets -- -D warnings` (all three crates) |
| `crates/rovi-motor-control` | Motor control (`no_std`, no dependencies): wheel commands to TB6612 pins, standby, faults | As above, or `cargo test -p rovi-motor-control` |
| `crates/rovi-drive` | Coordinator (`no_std`): device state to motor control, motor faults to a device stop | As above, or `cargo test -p rovi-drive` |
| `apps/simcar` | Sim car on laptop sockets, optional ROS 2 node; own lockfile | `cargo build --locked [--features ros]` |
| `apps/simcar/scenarios` | UDP and ROS scenario harness (Python, Docker) | From the repo root: `python3 apps/simcar/scenarios/udp_scenarios.py`; with `--features ros`: `apps/simcar/scenarios/ros-scenarios.sh` (also `ros-router-gone.sh`, `ros-reconnect.sh`, `ros-refresh.sh`, `ros-stale-session.sh`, `ros-capabilities.sh`) |
| `firmware` | ESP32-S3 firmware; own lockfile, `esp` toolchain | `ROVI_WIFI_PASSWORD=… cargo build --locked --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc`; add `--features ros` with `CARGO_TARGET_DIR=target/ros` to keep both builds |

##### ROS capabilities

The product publishes `/rovi/capabilities` (`std_msgs/String`) before any claim, with the
[nine keys and units](openspec/changes/add-four-wheel-api/specs/car-control-api/spec.md#requirement-capabilities-encoding).
Use **reliable, volatile, keep-last depth 1**; retained history is unavailable with the pinned
client ([O9 evidence](apps/simcar/scenarios/evidence/2026-10-17-capabilities/README.md)).
Capabilities repeat every 900 ms and on claim, release and router reconnect. Reading them does not claim the car.

```sh
ros2 topic echo /rovi/capabilities std_msgs/msg/String --once --full-length \
  --qos-reliability reliable --qos-durability volatile --qos-history keep_last --qos-depth 1
```

Parse the space-separated `key=value` tokens as numbers; preserve their precision when using
limits. Scientific notation represents the exact canonical f32 values shared with BLE/UDP.
Check the product sim car with `apps/simcar/scenarios/ros-capabilities.sh` after a ROS-enabled build.
