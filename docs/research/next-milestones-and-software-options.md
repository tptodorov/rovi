# Rovi: next milestones and software options

**Research snapshot:** 2026-10-03

**Status:** exploration, not an approved roadmap or architecture decision

This note joins the current project state with a proposed sequence, questions to settle, and software options checked against first-party documentation. Update [`PLAN.md`](../PLAN.md) and record an ADR only after the relevant choices are made.

## Where the project is

* M1 is complete on real hardware: the ESP32-S3-N16R8 and one TB6612FNG channel drove one brushed DC motor forward, reverse, and stop. That confirms one motor-control path, not the 4-wheel platform or application ([M1 results](../../experiments/rovi-m1-tb6612/MILESTONE.md)).
* The first board is an ESP32-S3. It supports BLE, Wi-Fi, USB OTG, and dual Xtensa cores; it does not support Bluetooth Classic ([Espressif ESP32-S3 overview](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/get-started/index.html), [Bluetooth capability table](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/bt-architecture/overview.html)).
* The chassis-kit listing identifies four mecanum wheels and four TT motors, but provides no motor electrical or mechanical ratings in the accessible listing details. A preliminary 120 mA single-motor reading exists, but its load and measurement conditions are unknown, so it does not establish a rating or driver limit ([car reference](../reference/mecanum-car/README.md#preliminary-measurement), [M4 results](../../experiments/rovi-m4-four-motor-safety/MILESTONE.md#results)).
* M2 is complete: the ESP32-S3 served a custom GATT peripheral to the laptop's BlueZ client and reported commands through LED/serial, with no motor hardware connected ([M2 results](../../experiments/rovi-m2-ble/MILESTONE.md)). BLE operation is considered proven; no further BLE bring-up experiment is needed.
* The process is aligned: spec-driven product work starts after M1. The low-level API proposal can start now; M3-M5 evidence will refine requirements and gate implementation ([README](../../README.md#two-development-tracks), [PLAN](../PLAN.md#roadmap)).

## Confirmed control direction

The system must provide both BLE and Wi-Fi network interfaces for control. Remote controls connect directly to the car over BLE or the car's Wi-Fi network. Initial BLE clients are an iPhone and the development laptop, and each must be able to issue every command supported by the current device. Command definitions are device-specific and independent of transport: a mecanum car's command set need not be the command set of another Rovi device.

BLE and Wi-Fi are input/signal sources. Each receiver translates its transport input into a device-specific command and enqueues it in a shared command channel for system processing. For the car, a valid command or controller ping must arrive within a configurable period. If it does not, firmware raises a timeout event and deasserts the standby signal on every motor driver.

## Suggested next milestones

This ordering keeps each milestone independently useful and makes the hardware and controller risks visible early. It is a proposal; acceptance criteria should be written in each milestone's `MILESTONE.md` before work begins.

| Order | Proposed milestone | Evidence required to complete it |
|---|---|---|
| M3 | Direct Wi-Fi and shared BLE/Wi-Fi command ingress on one ESP32-S3; motors disconnected. BLE bring-up and client command coverage are complete. | Use the already-established BLE input only as a source to the shared queue while validating Wi-Fi, command ordering, overload/staleness, latency, disconnect/recovery, and queue-independent watchdog behavior. Do not repeat BLE compatibility testing. Experiment: [`rovi-m3-wifi-queue`](../../experiments/rovi-m3-wifi-queue/MILESTONE.md). |
| M4 | Four-motor electrical qualification, power integrity, and standby fail-safe in one integrated chassis bench setup. | Establish motor/driver/battery/wiring limits; measure individual and combined load, rail sag, temperature, noise/reset effects, and safe standby behavior on boot/reset/watchdog. Experiment: [`rovi-m4-four-motor-safety`](../../experiments/rovi-m4-four-motor-safety/MILESTONE.md). |
| M5 | Mecanum motion and feedback characterization using the M4 setup, with no added sensors unless the received kit has them or requirements call for them. | Document wheel mapping/direction, movement combinations, calibration variation, and whether encoders exist; use evidence to choose open-loop or closed-loop API behavior. Experiment: [`rovi-m5-mecanum-motion`](../../experiments/rovi-m5-mecanum-motion/MILESTONE.md). |
| M6 | Start the low-level four-wheel API proposal now, then refine and implement it using M3-M5 results. Keep wheel outputs independent of mecanum math and input transport. | OpenSpec proposal accepted; API demonstrated on four physical channels. Define units, bounds, stop/rearm semantics, update timing, calibration, and fault behavior from measured limits. |
| M7 | Teleoperation and video integration after the camera/compute and Wi-Fi topology decisions. | End-to-end direct control with latency/loss/disconnect evidence; selected camera, video format/rate, and traffic behavior documented. Reuse the M4 car setup; add only the selected camera/Pi hardware. Experiment: [`rovi-m7-video-teleop`](../../experiments/rovi-m7-video-teleop/MILESTONE.md). |

**Setup grouping:** M3 is a board-only setup with controllers; M4 uses one integrated four-motor bench setup for power, electrical noise, and fail-safe checks; M5 reuses M4's setup for motion and feedback. This keeps the planned work to two primary setups through M5. M3 and M4 can proceed independently; M5's powered motion tests depend on M4's safe limits. M7 reuses the assembled car and adds the chosen video hardware only after that choice is made.

## Questions to close

These open questions should be resolved in the relevant product specifications. Measure hardware limits before treating them as fixed specification values.

### Hardware and control contract

1. What are the exact motor models and their rated voltage, no-load current, stall current, and gearbox ratio? Is the 5 V, 6500 mAh power bank the intended car supply, and what are its rated output current and usable capacity at 5 V? The preliminary 120 mA observation is not enough to answer these questions or establish whether the TB6612FNG modules can safely drive the motors together. Toshiba gives IC ratings, but the motor and breakout-board limits still govern ([TB6612FNG datasheet](https://toshiba-semicon-storage.com/info/TB6612FNG_datasheet_en_20141001.pdf?did=10660&prodName=TB6612FNG), [measurement record](../reference/mecanum-car/README.md#preliminary-measurement)).
2. Can the battery, protection circuitry, connectors, wiring, and regulators supply startup and simultaneous motor loads while keeping the ESP32 logic rail stable? Measure voltage sag and current during starts, reversals, and safe stall tests; confirm brownout and recovery behavior.
3. How will motor switching noise and current return paths be contained so they do not reset the controller or disrupt control links? Check grounding, decoupling, and any suppression needed on the received motor/driver setup under load.
4. Are all motor-driver standby inputs held inactive by hardware while the ESP32 is reset, booting, or unpowered? Confirm GPIO boot states and wiring make the motor outputs safe before firmware takes control.
5. Are wheel commands open-loop PWM/duty, target speed, or target velocity? Are wheel encoders present or planned? A closed-loop velocity promise needs sensing and calibration that the current hardware list does not identify.
6. The car fail-safe is a configurable watchdog: a valid command or controller ping must arrive within the interval; expiry raises a timeout event and deasserts every driver's standby signal. What should count as a valid command/ping, what interval range/default is suitable, and how should control be re-armed after timeout or reboot? Test that BLE and Wi-Fi traffic follow the same rule.
7. Does the first API expose four signed normalized wheel commands, physical units, or motor-driver primitives? Where should mecanum inverse kinematics and wheel calibration live: car application or a reusable motion layer?

### Radio and application boundary

8. How do iPhone and laptop clients learn the current device's command set: static per-device client code, a capability/service description, or a UI supplied by the device? The commands are device-specific; discovery/versioning still needs a mechanism.
9. BLE and Wi-Fi inputs will feed a shared command queue. What ordering applies when commands from different clients arrive close together? What happens when the queue is full or a queued command becomes stale? Does a valid command or ping from any authorized interface refresh the car's watchdog? Should stop or timeout events bypass the regular queue so a backlog cannot delay motor shutdown?
10. Which Wi-Fi mode should the car use (access point or joining an existing network), and which direct connection protocol, onboarding, and authentication should the controller use? What range, latency, and loss tolerance matter? Remote controls connect directly over the car's Wi-Fi or BLE.
11. What camera and video path should the first release use? Choose the host and define format, frame rate, latency, and bandwidth requirements from the new product needs; no prior software layout determines this choice.
12. What direct Wi-Fi command latency, connection-loss, and recovery behavior should the product guarantee? If camera/video shares the 2.4 GHz radio, what concurrent traffic must the system support?

### Scope and project process

13. What belongs in the first product release: direct manual control only, or direct control plus video? Set this scope from the Rovi product vision.
14. Is autonomy still explicitly deferred? If yes, keep it as a future consumer of the API and avoid designing its control stack now.

## Software options

### ESP32-S3 firmware

| Option | Fit | Trade-off |
|---|---|---|
| **Current Rust direction: `esp-hal` + `esp-radio` + Embassy `trouble-host`** | Best continuity with M1 and keeps control in Rust/`no_std`. M2 proved the BLE peripheral path; the current stack also provides the Wi-Fi path to evaluate. TrouBLE implements BLE central/peripheral roles and GATT server/client. | The Rust MCPWM API is unstable and requires the feature already enabled in M1. Validate four-channel PWM and direct Wi-Fi control on the board before choosing them for product firmware ([esp-radio docs](https://docs.espressif.com/projects/rust/esp-radio/1.0.0-beta.1/esp32s3/esp_radio/index.html), [TrouBLE](https://github.com/embassy-rs/trouble), [esp-hal MCPWM](https://docs.espressif.com/projects/rust/esp-hal/1.1.0/esp32s3/esp_hal/mcpwm/index.html)). |
| **Rust with ESP-IDF: `esp-idf-svc` and NimBLE** | Rust application with Espressif's ESP-IDF runtime and GATT server/client wrappers; `esp32-nimble` is another Rust wrapper built on ESP-IDF ([GATT server](https://github.com/esp-rs/esp-idf-svc/blob/master/src/bt/ble/gatt/server.rs), [GATT client](https://github.com/esp-rs/esp-idf-svc/blob/master/src/bt/ble/gatt/client.rs), [crate docs](https://docs.rs/crate/esp32-nimble/latest)). | Switches away from M1's bare-metal `esp-hal` runtime. Validate build, latency, PWM timing, and current crate maintenance before adopting it for all firmware. |
| **ESP-IDF C/C++** | Espressif's first-party framework and networking APIs, including Wi-Fi and BLE support ([BLE stack guidance](https://docs.espressif.com/projects/esp-idf/en/v5.4.1/esp32s3/api-reference/bluetooth/index.html)). | Most direct vendor path, but introduces a second implementation language after a successful Rust M1. Consider if Rust Wi-Fi or motor integration blocks progress. |

**Recommendation:** retain Rust for motor control and the proven BLE path. Focus remaining transport work on direct Wi-Fi control and shared command-queue behavior. BLE operation is not an outstanding project risk.

For the motor layer, continue `esp-hal` PWM/MCPWM and the existing `tb6612fng` crate where it fits. Hardware-timed PWM leaves the CPU to handle setpoint updates, safety, and communications; keep mecanum mapping above a transport-neutral, motor-driver-independent four-wheel API. Keep each exploratory milestone in its independent Cargo package until the shared product API exists, consistent with the repo's current convention.

### Client and video software

* Keep command definitions device-specific and transport-independent. M3 will settle the direct Wi-Fi interface; choose the control-client implementation against the supported iPhone and laptop requirements after the command schema and authentication policy are defined.
* No camera or video-host choice is made. Select candidate hardware and streaming software from the first-release requirements for frame rate, latency, bandwidth, power, and direct connectivity. Validate the selected path in M7; do not preserve an earlier application architecture by default.

## Immediate next actions

1. Start the low-level API proposal under the existing spec-driven process; capture known commands and leave measured motor limits as open parameters until M4.
2. Resolve motor and power-bank ratings that are absent from the chassis listing before setting driver limits or estimating runtime.
3. Start M3 for direct Wi-Fi and shared queue behavior with motors disconnected.
4. Run M4 to qualify the four-motor power/safety setup before sustained drive tests.
5. Run M5 on the same chassis to settle wheel mapping, calibration, and feedback scope.
6. Refine the M6 low-level API with M3-M5 evidence before finalizing implementation limits.
7. Decide the camera and compute placement before scheduling M7 video coexistence/integration tests.

## Concern-to-evidence map

| Concern | Planned evidence |
|---|---|
| Direct Wi-Fi mode, protocol, client setup, authentication, latency, and recovery | M3 compares viable AP/station arrangements and records the selected product behavior. |
| Device-specific command discovery and versioning | M3 records how clients obtain the current command set; settle the mechanism in the control-interface spec. |
| Shared queue ordering, overload, stale commands, and stop priority | M3 exercises the already-proven BLE input and Wi-Fi source concurrently with a fake output consumer; BLE compatibility is not retested. |
| Watchdog definition, refresh sources, rearm, and timeout event | M3 validates queue-independent software behavior; M4 verifies all physical STBY outputs under motor load and reset/disconnect conditions. |
| Motor, driver, battery, wiring, connector, regulator, thermal, brownout, and noise limits | M4 uses one integrated four-motor bench setup and records unknowns as unresolved rather than guessing ratings. |
| Safe driver state during boot/reset/unpowered conditions | M4 inspects hardware bias and verifies each driver output through reset and power transitions. |
| Mecanum mapping, API units, calibration, and feedback/encoder need | M5 characterizes motion and available sensors on the same setup as M4; M6 converts results into the API/spec. |
| Camera/compute placement and video/control coexistence | Resolve camera hardware and placement first; then test together in [`rovi-m7-video-teleop`](../../experiments/rovi-m7-video-teleop/MILESTONE.md) using the selected equipment. |
| Product scope and autonomy | Decide in PLAN/spec work; these are project decisions, not bench experiments. Spec work starts after M1; hardware results gate API implementation details. |

## Sources

Project facts are linked inline to the repo's vision, plan, M1 results, and process guide. External implementation claims link to primary vendor, framework, or project documentation above. Software capabilities and API maturity can change; recheck linked docs when creating the implementation proposal.
