# Feedback loops

How an agent (or you) sees Rovi's work run and iterates on it, ranked by impact and ease. Status as of 2026-10-11; update it as loops land. Simulation never counts as hardware acceptance ([ADR-0002](adr/0002-simulation-never-proves-hardware.md)); the sim loops speed up iteration, the board loops produce the evidence.

Status: **DONE** means built and verified on the laptop. Nothing here counts as board evidence yet.

Impact is the judgment of how many real bugs or open decisions a loop catches (Q18-Q20, M4's fail-safe, M5's wheel mapping). Ease covers setup effort and purchases.

| Rank | Channel | How it works | Impact | Ease | Needs | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | **One scenario suite for sim and board** (`sim/udp_scenarios.py`) | Eight `unittest` scenarios talk UDP over real sockets. By default each test starts a fresh `simcar` with short lease/claim/reclaim windows. With `ROVI_TARGET=host:port` they use a running car instead: wait until it is free, send `BYE` afterwards, take the windows from `ROVI_*_MS`. Each scenario also asserts events in the car's log (rank 2) | High | Easy | Laptop | **DONE.** Sim and running-car modes verified; board untested |
| 2 | **`ev=` event lines from the firmware and sim car, plus `sim/carlog.py`** (logfmt, not JSONL: one shared formatter, no serializer on the S3) | `src/events.rs` defines the events (see [LOGGING.md](LOGGING.md)); the sim car prints them, and the firmware queues them in a ring that one paced task formats and writes. `carlog.parse()` reads a sim stdout or a serial capture and ignores other lines and cut-off ones. Scenarios poll it for up to 1 s (`saw()`); `python3 sim/carlog.py FILE` checks that claim and release alternate per boot, that every event is in the catalog, and that there is no panic, fault or dropped event | High | Easy-medium | Laptop | **DONE.** Firmware builds and passes clippy; its serial output is not yet seen on a board |
| 2b | **BLE bench client and scenarios** (`sim/ble_client.py`, `sim/ble_scenarios.py`, bleak) | The client scans for "Rovi M8" (or `ROVI_BLE_ADDR`), connects (the claim), reports the ATT MTU and warns below 75, reads Capabilities, subscribes to Status, streams setpoints at 20 Hz and writes ARM/STOP. Eleven `unittest` scenarios mirror the UDP suite: claim by connect (Status read at once), a refused ARM or unknown command shown as `NotReady`/`Invalid` in Status besides the rejected write, drive, stop (owner kept while it streams; re-arm needs a setpoint after the stop), lease expiry and re-arm, reclaim and claim windows, disconnect releases, wheel signs. A second-central scenario needs a second Bluetooth adapter; the bench has one, so it stays skipped with the same car-log checks; windows come from Capabilities. No BLE sim car: without `ROVI_BLE_ADDR` they skip. The client, payloads (`sim/wire.py`, Rust golden vectors) and the scenario code are tested against an in-memory fake GATT peer | High | Easy | Board | **Laptop only.** Fake-peer tests pass and the scan path runs on the laptop's BlueZ; never run against a board |
| 3 | **Bench script** (`sim/bench.sh`) | Makes `bench/<name>/`, runs the UDP scenarios into `scenarios.txt` (and with `ROVI_BLE_ADDR` the BLE scenarios into `ble-scenarios.txt`), keeps the car log as `car.log` (sim: written by the sim car; board: copied from `ROVI_CARLOG`), runs the `carlog.py` check into `carlog.txt`, and exits non-zero if either fails. `BENCH_CAM=<s>` records camera frames alongside (rank 4) | High | Medium | Board | **DONE for the sim.** The board run is the same script with `ROVI_TARGET`; flashing and joining the AP stay manual |
| 4 | **Laptop camera frames** (`scripts/cam.sh`) | `ffmpeg` copies the camera's 1280x720 30 fps MJPEG frames, without re-encoding, into `frames/<epoch µs>.jpg` (the name is the wall-clock capture time), then tiles 12 evenly spaced frames into `sheet.jpg` for a quick look | Medium-high | Easy | Laptop camera | **DONE.** Frame rate and naming verified; the car is not in view yet |
| 5 | **Native USB port** (USB-Serial-JTAG) for logs and button-free reset | `esp-println` `auto` switches to the native port once it sees USB traffic (SOF), otherwise it uses UART0; `espflash` resets and enters download mode over it | Medium | Easy | Board, cable | **BLOCKED** on the bench. Confirm which connector it is (`303a:1001`) |
| 6 | **Round-trip latency echo** over BLE, UDP and ROS 2 (Q18, Q19, ADR-0008) (`sim/latency.py`, `sim/ros_latency.py`) | The car sends back any payload of up to 32 bytes unchanged: UDP `ECHO` frame, BLE Echo characteristic (write without response, notify). The echo is stateless: any peer may use it and it never touches the owner or lease. It is not logged. **BLE echo works only while the car is free:** connecting over BLE is a claim, so a BLE run owns the car for its duration and cannot run during another client's session. Over ROS 2 the car subscribes to `/rovi/echo` and republishes the same `std_msgs/Int64` on `/rovi/echo_reply` (any node, no claim); `sim/ros_latency.py` runs inside the ROS container and prints the same summary (`ros sent=… median_ms=…`). The client sends N probes (probe number, send time in ns) at a fixed rate and prints `sent count loss_pct median_ms p95_ms max_ms`. `sim/bench.sh` writes it to `latency.txt` | Medium-high | Easy-medium | Board | **DONE for UDP on the sim** (200 probes at 20 Hz: median about 0.3 ms over loopback). The BLE variant, ROS ([#16](https://github.com/tptodorov/rovi/issues/16)) and every board number are untested |
| 7 | **Current and voltage logging** (INA226 on the rail, or a USB meter with serial output) | Not planned for M4 ([ADR-0014](adr/0014-m4-minimal-wiring-software-fail-safe.md)): current is read on a meter by hand. Revisit for M5 runtime and stall measurements | High | Medium | About €5-15 | Later |
| 8 | **Logic analyzer with `sigrok-cli`** (PWM, STBY, stop-to-brake latency) | Planned: probe the TB6612 inputs and decode captures on the laptop | High | Medium | About €10-20 | Buy before M4 |
| 9 | **Commanded-motion plot** (`sim/plot_car.py`) | Takes the last boot in a car log, reads the `ev=state` events, holds each wheel command until the next event, and integrates the mecanum forward kinematics (wheel order FL, FR, RR, RL) into a pose. The PNG shows wheel commands over time and the path. It is commanded motion, not measured motion | Low-medium | Easy | Laptop | **DONE.** Checked against a forward, rotate, strafe drive (1.5 rad/s for 1 s gave 87 degrees) |
| 10 | **probe-rs JTAG debugging** | `probe-rs run --chip esp32s3` over the native USB port for breakpoints, memory reads and a GDB server | Medium | Medium | Board | Tools in `shell.nix` (`probe-rs` 0.31.0 lists `esp32s3`). The udev rule in `~/mycfg` and RTT on the board are untested |
| 11 | **Marker tracking on the laptop camera** (`scripts/track_marker.py`, `sim/compare_track.py`) for measured motion. An overhead webcam only if the laptop view is not enough | `track_marker.py` finds one ArUco marker (DICT_4X4_50, id 0) in each rank 4 frame and writes a CSV: `epoch_us` (from the file name), centre `x_px`/`y_px`, `heading_rad`, `size_px`; with `--side-m` also `x_m`/`y_m`, with `--calib` (from `scripts/calibrate_camera.py`) corners are undistorted first. Frames without the whole marker give no row. `compare_track.py` aligns the first wheel command with the first marker motion, turns the track into the car's start frame, overlays it on `plot_car.commanded_path` and prints distance and heading drift | High | Medium-hard | Printed marker, camera looking down | **DONE on synthetic frames only.** Markers drawn at known poses come back within 1.5 px and 1.5 degrees; a synthetic 0.5 m drive through JPEG frames shows 5 mm and 0.3 degrees of drift. No real car or marker has been in view; the laptop camera's oblique view is untested (see Notes) |
| 12 | **Per-port USB power switch** (`uhubctl`) for power-cycle tests | Planned: switch the board's hub port off and on from the script | Low-medium | Easy-medium | A hub with per-port switching | Later |
| 13 | **Wokwi** (PWM and watchdog timing only; no BLE or TB6612; needs a token and a config) | - | Low | Medium | Token | Skip |
| 14 | **IMU or encoders** | - | High if M5 picks closed-loop | Hard | Hardware change | Defer until Q11 |

## Running the done loops

Run from `experiments/rovi-m8-setpoint-ros/` inside the repo's nix shell (`direnv` or `nix-shell ../../shell.nix`).

```sh
task m8:test                                  # core tests + car-log parser tests
task m8:bench                                 # sim: scenarios + latency + log check -> bench/sim-<time>/
python3 sim/latency.py                        # UDP RTT against a fresh sim car (ROVI_TARGET=host:port for a running car)
python3 sim/plot_car.py bench/sim-<time>/car.log plot.png

# camera, 5 s, frames named <epoch µs>.jpg + sheet.jpg
../../scripts/cam.sh bench/sim-cam 5
BENCH_CAM=20 sim/bench.sh                     # scenarios with the camera recording meanwhile
```

Marker tracking (from the repo root; `task scripts:test` runs its tests). Measured motion, but only as good as the view: see the notes below.

```sh
scripts/cam.sh bench/m5-fwd/cam 10            # record while the car drives
python3 scripts/track_marker.py bench/m5-fwd/cam/frames bench/m5-fwd/track.csv --side-m 0.08 [--calib calib.json]
python3 experiments/rovi-m8-setpoint-ros/sim/compare_track.py bench/m5-fwd/car.log bench/m5-fwd/track.csv bench/m5-fwd/compare.png
python3 scripts/calibrate_camera.py calib.json shots/*.jpg   # once per camera and resolution, 9x6 inner-corner board
```

Board run (expected flow once the bench is back; **not yet tried**): flash the firmware, join the car AP (`192.168.4.2/24`, see the M8 README), capture the serial output to a file, then:

```sh
ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=<serial capture> BENCH_NAME=board-<date> sim/bench.sh
python3 sim/latency.py --ble <address>        # BLE RTT (needs bleak); connecting claims the car, so run it while the car is free
```

Step by step for bench day (preflight, each phase, what to look for, results template): [M8 bench runbook](../experiments/rovi-m8-setpoint-ros/BENCH.md).

BLE leg (also **not yet tried**): find the car's address (`bluetoothctl scan on`, name "Rovi M8"), then add `ROVI_BLE_ADDR=<addr>` to the line above, or use it without `ROVI_TARGET` for BLE only. `python3 sim/ble_client.py info` shows MTU, capabilities and status; `drive --vx 0.2 --seconds 2` streams one drive. The second-central scenario stays skipped: it needs a second Bluetooth adapter (`ROVI_BLE_ADAPTER2`) and the bench has one. With one adapter, bleak reuses the open connection, so on the bench a second BLE claim is refused only in the host tests (`Busy`) and by the car not advertising while owned. Without bleak (outside the nix shell) the bench fails rather than skipping.

The UDP scenario windows default to `Config::DEFAULT` for a board (`ROVI_LEASE_MS`, `ROVI_CLAIM_MS`, `ROVI_RECLAIM_MS` override them). Wi-Fi packet loss can fail a scenario that passes on the sim; read the car log before assuming a firmware bug.

## Notes

* **Log format.** One standard for sim and board: [LOGGING.md](LOGGING.md). `at_ms` is the car's clock; the camera and client use the laptop's epoch clock, so correlating them needs one shared reference event, such as the first setpoint.
* **S3 build.** `cargo` and `rustc` from the nix profile cannot build Xtensa. Use the esp toolchain: `source ~/export-esp.sh; PATH=~/.rustup/toolchains/esp/bin:$PATH`, then the build command in the M8 README.
* **Camera privacy.** The camera shows the room. `bench/*/cam/frames/` and `bench/sim-*/` are gitignored; look at frames only while a test runs with the car in view. Only one program can hold the camera at a time. Track CSVs and comparison PNGs hold no imagery and can be committed; checkerboard shots also show the room, so keep them out of git.
* **Marker print and placement.** Print id 0, for example `python3 -c "import cv2; d=cv2.aruco.getPredefinedDictionary(cv2.aruco.DICT_4X4_50); cv2.imwrite('marker0.png', cv2.copyMakeBorder(cv2.aruco.generateImageMarker(d, 0, 600), 100, 100, 100, 100, cv2.BORDER_CONSTANT, value=255))"`. Make the black square 6-10 cm on matte paper, keep the white border, then measure the printed black square with a ruler and pass that as `--side-m` (printers rescale). Tape it flat on top of the car over the centre of rotation, its top edge (as printed) towards the car's front; the comparison assumes that. At 1280x720 it should span at least about 30 px in every frame.
* **Camera geometry.** `--side-m` uses one scale for the whole image, which is right only for a camera looking straight down at the floor. A laptop camera looking across the room sees the floor at an angle: distances shrink with depth and headings skew, so treat its numbers as relative (repeatability, sign, left/right asymmetry), not absolute. Raise and tilt the laptop as far as possible, or use an overhead webcam. Calibration fixes lens distortion only, not the angle.
* **Motion safety.** Wheels turn only with the car on blocks or you present. M4 gates all motion.
* **Both USB ports.** `esp-println` `auto` keeps printing to the native port once it has seen USB traffic, so the CH343 port can go silent. Pick one log channel per run. Check the board schematic for backfeed before running motors from a battery with both cables attached.
