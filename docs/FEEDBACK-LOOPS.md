# Feedback loops

How an agent (or you) sees Rovi's work run and iterates on it, ranked by impact and ease. Status as of 2026-10-09; update it as loops land. Simulation never counts as hardware acceptance ([ADR-0002](adr/0002-simulation-never-proves-hardware.md)); the sim loops speed up iteration, the board loops produce the evidence.

Impact is the judgment of how many real bugs or open decisions a loop catches (Q18-Q20, M4's fail-safe, M5's wheel mapping). Ease covers setup effort and purchases.

| Rank | Channel | Impact | Ease | Needs | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | **One scenario suite for sim and board.** `sim/udp_scenarios.py` takes `ROVI_TARGET` | High | Easy | Laptop | **Done.** Sim and running-car modes verified on the laptop; board untested |
| 2 | **`ev=` event lines from the firmware and sim car, plus `sim/carlog.py`** (logfmt, not JSONL: one shared formatter, no serializer on the S3) | High | Easy-medium | Laptop | **Done.** Firmware builds and passes clippy; its serial output is not yet seen on a board |
| 3 | **Bench script** `sim/bench.sh`: scenarios, log check, optional camera, into `bench/<name>/` | High | Medium | Board | **Done for the sim.** The board run is the same script with `ROVI_TARGET`; flashing and joining the AP stay manual |
| 4 | **Laptop camera frames** `scripts/cam.sh`, named by wall-clock µs | Medium-high | Easy | Laptop camera | **Done.** 30 fps MJPEG verified; the car is not in view yet |
| 5 | **Native USB port** (USB-Serial-JTAG) for logs and button-free reset | Medium | Easy | Board, cable | Blocked on the bench. `esp-println` `auto` already prints there. Confirm which connector it is (`303a:1001`) |
| 6 | **Round-trip latency echo** over BLE and UDP (Q18, Q19, ADR-0008) | Medium-high | Easy-medium | Board | Not started |
| 7 | **Current and voltage logging** (INA226 on the rail, or a USB meter with serial output) | High | Medium | About €5-15 | Buy before M4 |
| 8 | **Logic analyzer with `sigrok-cli`** (PWM, STBY, stop-to-brake latency) | High | Medium | About €10-20 | Buy before M4 |
| 9 | **Commanded-motion plot** `sim/plot_car.py` (wheel commands and integrated path from a car log) | Low-medium | Easy | Laptop | **Done.** Checked against a forward, rotate, strafe drive |
| 10 | **probe-rs JTAG debugging** | Medium | Medium | Board | Tools in `shell.nix` (`probe-rs` 0.31.0 lists `esp32s3`). The udev rule in `~/mycfg` and RTT on the board are untested |
| 11 | **Marker tracking on the laptop camera** (OpenCV ArUco) for relative motion. An overhead webcam only if the laptop view is not enough | High | Medium-hard | Marker, calibration | Needed for M5; not started |
| 12 | **Per-port USB power switch** (`uhubctl`) for power-cycle tests | Low-medium | Easy-medium | A hub with per-port switching | Later |
| 13 | **Wokwi** (PWM and watchdog timing only; no BLE or TB6612; needs a token and a config) | Low | Medium | Token | Skip |
| 14 | **IMU or encoders** | High if M5 picks closed-loop | Hard | Hardware change | Defer until Q11 |

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

* **Log format.** `ev=claimed source=Udp at_ms=N`, `ev=stop reason=LeaseExpired at_ms=N`, `ev=released at_ms=N`, `ev=state state=Armed wheels=a,b,c,d at_ms=N` (`src/events.rs`). `at_ms` is the car's clock; the camera and client use the laptop's epoch clock, so correlating them needs one shared reference event, such as the first setpoint.
* **S3 build.** `cargo` and `rustc` from the nix profile cannot build Xtensa. Use the esp toolchain: `source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH`, then the build command in the M8 README.
* **Camera privacy.** The camera shows the room. `bench/*/cam/frames/` and `bench/sim-*/` are gitignored; look at frames only while a test runs with the car in view. Only one program can hold the camera at a time.
* **Motion safety.** Wheels turn only with the car on blocks or you present. M4 gates all motion.
* **Both USB ports.** `esp-println` `auto` keeps printing to the native port once it has seen USB traffic, so the CH343 port can go silent. Pick one log channel per run. Check the board schematic for backfeed before running motors from a battery with both cables attached.
