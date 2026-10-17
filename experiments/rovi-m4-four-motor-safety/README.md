# Rovi M4: four motors on the M8 firmware

**Status: step 1 passed (2026-10-14); step 2 in progress (mapping and direction confirmed 2026-10-17).**

M4 drives the car's motors from the M8 firmware for the first time, wheels off the ground, with the minimal wiring of [ADR-0014](../../docs/adr/0014-m4-minimal-wiring-software-fail-safe.md): the ESP32-S3, TB6612FNG drivers, the motors and the 5 V power bank. Step 1 used M1's wiring; step 2 uses one 7-wire ribbon cable per driver ([WIRING](WIRING.md)). Goal, acceptance and results: [MILESTONE.md](MILESTONE.md).

| File | What |
| --- | --- |
| [WIRING.md](WIRING.md), [wiring.kicad_sch](wiring.kicad_sch) | Four motors and two drivers; one 7-wire ribbon cable per driver |
| [build/](build/README.md) | Build guide: parts dictionary, crimping, motor harnesses, the perfboard carrier (top and bottom views, numbered wire list), checks, first power-up |
| [TESTPLAN.md](TESTPLAN.md) | Bench steps S1-S5, safety rules, evidence |

## Firmware

There is no M4 firmware of its own. The M8 firmware ([`../rovi-m8-setpoint-ros/`](../rovi-m8-setpoint-ros/)) drives the motor pins from its 10 ms control tick:

* `motor::output` (host-tested core, `src/motor.rs`) turns the device state and the four wheel commands into `STBY` and each channel's `IN1`/`IN2`/duty. `STBY` is high only while armed with finite commands; a zero command coasts.
* Duty range: any wheel command outside a 2 % deadband starts at `min_duty_pm` 40 % (the mounted motors need about 35 % to start, step 2) and scales linearly to `max_duty_pm` 100 % (5 V) at 1.0. With the placeholder geometry `vx` 1.0 m/s is 70 % duty. A zero command always coasts.
* `CHANNELS` maps the four bridges to wheels: driver 1 A/B = front-left/rear-left, driver 2 A/B = front-right/rear-right ([WIRING](WIRING.md)); inversion per motor is set after step 2's first test.
* The control tick feeds the RTC watchdog (1 s, resets the chip); `ev=boot` logs the reset cause (`reset=SysRtcWdt`, `SysBrownOut`, …).

The motor is driven by any M8 client: `sim/ble_client.py drive` (BLE), the UDP client, or `ros2 topic pub /cmd_vel` (ROS build).

## Results

[MILESTONE.md](MILESTONE.md#results).
