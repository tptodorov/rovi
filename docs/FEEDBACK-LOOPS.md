# Feedback loops

How an agent (or you) sees Rovi's work run and iterates on it, ranked by impact and ease. Status as of 2026-10-09; update it as loops land. Simulation never counts as hardware acceptance ([ADR-0002](adr/0002-simulation-never-proves-hardware.md)); the sim loops speed up iteration, the board loops produce the evidence.

Status: **DONE** means built and verified on the laptop. Nothing here counts as board evidence yet.

Impact is the judgment of how many real bugs or open decisions a loop catches (Q18-Q20, M4's fail-safe, M5's wheel mapping). Ease covers setup effort and purchases.

| Rank | Channel | How it works | Impact | Ease | Needs | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | **One scenario suite for sim and board** (`sim/udp_scenarios.py`) | Seven `unittest` scenarios talk UDP over real sockets. By default each test starts a fresh `simcar` with short lease/claim/reclaim windows. With `ROVI_TARGET=host:port` they use a running car instead: wait until it is free, send `BYE` afterwards, take the windows from `ROVI_*_MS`. Each scenario also asserts events in the car's log (rank 2) | High | Easy | Laptop | **DONE.** Sim and running-car modes verified; board untested |
| 2 | **`ev=` event lines from the firmware and sim car, plus `sim/carlog.py`** (logfmt, not JSONL: one shared formatter, no serializer on the S3) | `src/events.rs` defines the events (see [LOGGING.md](LOGGING.md)); the sim car prints them, and the firmware queues them in a ring that one paced task formats and writes. `carlog.parse()` reads a sim stdout or a serial capture and ignores other lines and cut-off ones. Scenarios poll it for up to 1 s (`saw()`); `python3 sim/carlog.py FILE` checks that claim and release alternate per boot, that every event is in the catalog, and that there is no panic, fault or dropped event | High | Easy-medium | Laptop | **DONE.** Firmware builds and passes clippy; its serial output is not yet seen on a board |
| 3 | **Bench script** (`sim/bench.sh`) | Makes `bench/<name>/`, runs the scenarios into `scenarios.txt`, keeps the car log as `car.log` (sim: written by the sim car; board: copied from `ROVI_CARLOG`), runs the `carlog.py` check into `carlog.txt`, and exits non-zero if either fails. `BENCH_CAM=<s>` records camera frames alongside (rank 4) | High | Medium | Board | **DONE for the sim.** The board run is the same script with `ROVI_TARGET`; flashing and joining the AP stay manual |
| 4 | **Laptop camera frames** (`scripts/cam.sh`) | `ffmpeg` copies the camera's 1280x720 30 fps MJPEG frames, without re-encoding, into `frames/<epoch µs>.jpg` (the name is the wall-clock capture time), then tiles 12 evenly spaced frames into `sheet.jpg` for a quick look | Medium-high | Easy | Laptop camera | **DONE.** Frame rate and naming verified; the car is not in view yet |
| 5 | **Native USB port** (USB-Serial-JTAG) for logs and button-free reset | `esp-println` `auto` switches to the native port once it sees USB traffic (SOF), otherwise it uses UART0; `espflash` resets and enters download mode over it | Medium | Easy | Board, cable | **BLOCKED** on the bench. Confirm which connector it is (`303a:1001`) |
| 6 | **Round-trip latency echo** over BLE and UDP (Q18, Q19, ADR-0008) | Planned: the firmware echoes a client timestamp and the client times the round trip | Medium-high | Easy-medium | Board | Not started |
| 7 | **Current and voltage logging** (INA226 on the rail, or a USB meter with serial output) | Planned: log rail current and voltage next to the `ev=` lines | High | Medium | About €5-15 | Buy before M4 |
| 8 | **Logic analyzer with `sigrok-cli`** (PWM, STBY, stop-to-brake latency) | Planned: probe the TB6612 inputs and decode captures on the laptop | High | Medium | About €10-20 | Buy before M4 |
| 9 | **Commanded-motion plot** (`sim/plot_car.py`) | Takes the last boot in a car log, reads the `ev=state` events, holds each wheel command until the next event, and integrates the mecanum forward kinematics (wheel order FL, FR, RR, RL) into a pose. The PNG shows wheel commands over time and the path. It is commanded motion, not measured motion | Low-medium | Easy | Laptop | **DONE.** Checked against a forward, rotate, strafe drive (1.5 rad/s for 1 s gave 87 degrees) |
| 10 | **probe-rs JTAG debugging** | `probe-rs run --chip esp32s3` over the native USB port for breakpoints, memory reads and a GDB server | Medium | Medium | Board | Tools in `shell.nix` (`probe-rs` 0.31.0 lists `esp32s3`). The udev rule in `~/mycfg` and RTT on the board are untested |
| 11 | **Marker tracking on the laptop camera** (OpenCV ArUco) for relative motion. An overhead webcam only if the laptop view is not enough | Planned: a marker on the car, detected in the frames from rank 4 | High | Medium-hard | Marker, calibration | Needed for M5; not started |
| 12 | **Per-port USB power switch** (`uhubctl`) for power-cycle tests | Planned: switch the board's hub port off and on from the script | Low-medium | Easy-medium | A hub with per-port switching | Later |
| 13 | **Wokwi** (PWM and watchdog timing only; no BLE or TB6612; needs a token and a config) | - | Low | Medium | Token | Skip |
| 14 | **IMU or encoders** | - | High if M5 picks closed-loop | Hard | Hardware change | Defer until Q11 |

## Running the done loops

Run from `experiments/rovi-m8-setpoint-ros/` inside the repo's nix shell (`direnv` or `nix-shell ../../shell.nix`).

```sh
task m8:test                                  # core tests + car-log parser tests
task m8:bench                                 # sim: scenarios + log check -> bench/sim-<time>/
python3 sim/plot_car.py bench/sim-<time>/car.log plot.png

# camera, 5 s, frames named <epoch µs>.jpg + sheet.jpg
../../scripts/cam.sh bench/sim-cam 5
BENCH_CAM=20 sim/bench.sh                     # scenarios with the camera recording meanwhile
```

Board run (expected flow once the bench is back; **not yet tried**): flash the firmware, join the car AP (`192.168.4.2/24`, see the M8 README), capture the serial output to a file, then:

```sh
ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=<serial capture> BENCH_NAME=board-<date> sim/bench.sh
```

The scenario windows default to `Config::DEFAULT` for a board (`ROVI_LEASE_MS`, `ROVI_CLAIM_MS`, `ROVI_RECLAIM_MS` override them). Wi-Fi packet loss can fail a scenario that passes on the sim; read the car log before assuming a firmware bug.

## Notes

* **Log format.** One standard for sim and board: [LOGGING.md](LOGGING.md). `at_ms` is the car's clock; the camera and client use the laptop's epoch clock, so correlating them needs one shared reference event, such as the first setpoint.
* **S3 build.** `cargo` and `rustc` from the nix profile cannot build Xtensa. Use the esp toolchain: `source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH`, then the build command in the M8 README.
* **Camera privacy.** The camera shows the room. `bench/*/cam/frames/` and `bench/sim-*/` are gitignored; look at frames only while a test runs with the car in view. Only one program can hold the camera at a time.
* **Motion safety.** Wheels turn only with the car on blocks or you present. M4 gates all motion.
* **Both USB ports.** `esp-println` `auto` keeps printing to the native port once it has seen USB traffic, so the CH343 port can go silent. Pick one log channel per run. Check the board schematic for backfeed before running motors from a battery with both cables attached.
