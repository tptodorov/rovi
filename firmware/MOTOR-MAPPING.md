# Product motor binding — 2026-10-18

Task 5.1 check at source commit `83312bb` (firmware/core binding from `0056104`).
Result: all 14 control GPIOs and four motor routes match the recorded carrier schematic;
the firmware installs `MotorConfig::PRODUCT` and `CAR_CHANNELS` before transport input.
No firmware changes, flashing or motor runs were needed for this check.

## Sources and confidence

* [Firmware](src/fw.rs): `stby`, `dirs`, the MCPWM pin bindings, PWM dispatch and `MotorPins` driver.
* [Motor crate](../crates/rovi-motor-control/src/lib.rs): bridge order, `CAR_CHANNELS`, product profile and coast request.
* [Carrier schematic](../experiments/rovi-m4-four-motor-safety/wiring.kicad_sch): KiCad's own XML netlist export, independently compared with the schematic analyzer's connectivity.
* [Carrier layout](../experiments/rovi-m4-four-motor-safety/build/gen_build_guide.py): its trace check connects 25 carrier nets with 34 wires, with no splits/shorts or shared-hole problems. The separate schematic export check reports 26 intended nets, zero differences (including the off-carrier USB power connection).
* Recorded board header conventions: [ESP32 CORE](../docs/reference/esp32-s3-n16r8/README.md) and [TB6612 breakout](../docs/reference/tb6612fng/README.md). These are module headers, not chip package pin numbers.

Confidence is **deterministic** for source-to-net consistency; evidence sources are firmware
source and exported schematic topology. The analyzer reports DS-001 because the custom module
symbols lack MPN/datasheet metadata. This is a connectivity consistency check, not a new
component/electrical assessment. Actual pin signals and wheel positions remain bench checks.
No PCB/Gerber files represent this perfboard assembly; layout, EMC and thermal analyses are
outside this binding check. No SPICE simulator was installed.

## GPIO and wheel routes

`MotorPins` indexes bridges D1 A, D1 B, D2 A, D2 B. The device wheel vector is FL, FR, RR, RL.
The firmware's PWM closure dispatches bridge indices 0–3 to those same four MCPWM outputs.

| Bridge | Wheel index / position | Motor | PWM GPIO / ESP header | IN1 GPIO / ESP header | IN2 GPIO / ESP header | STBY GPIO / ESP header |
| --- | --- | --- | --- | --- | --- | --- |
| D1 A / U2 A | 0 / FL | M1 | 4 / L4 | 6 / L6 | 5 / L5 | 7 / L7 |
| D1 B / U2 B | 3 / RL | M2 | 17 / L10 | 15 / L8 | 16 / L9 | 7 / L7 |
| D2 A / U3 A | 1 / FR | M3 | 46 / L14 | 10 / L16 | 9 / L15 | 11 / L17 |
| D2 B / U3 B | 2 / RR | M4 | 14 / L20 | 12 / L18 | 13 / L19 | 11 / L17 |

On both driver headers: R1=PWMA, R2=AIN2, R3=AIN1, R4=STBY, R5=BIN1, R6=BIN2,
R7=PWMB. The netlist connects each to exactly one ESP32 header pin. Driver A outputs L4/L5
and B outputs L7/L6 connect to leads 1/2 of the motor listed above.
All four software inversions are false, using the recorded connector polarity correction;
this check does not newly demonstrate physical direction.

## Profile and PWM binding

* Deadband: 0.02; minimum driven duty: 400 permille; maximum: 700 permille (70%).
* Each driver has its own STBY; GPIO7/GPIO11 initialize low before other startup work.
* MCPWM0 uses operator 0 for D1 and operator 1 for D2, both on timer 0, active high at 20 kHz.
* Timer period is 999 (1000 count positions); the adapter passes permille compare values directly. Coast requests compare 1000 with both direction bits low; it is independent of the 700 drive cap. Physical PWM levels remain task 5.2.
* `Drive::new(motor, &mut device)` binds readiness before the transport tasks run. A bad motor configuration inhibits explicit ARM and ROS auto-arm; boot logs identify `motor_config`.

The pin/profile check found no routing change to make.

## Checks and remaining bench work

On 2026-10-18, both ESP32-S3 variants built with their locked dependencies and passed clippy
(`--release`, with/without `ros`, `-D warnings`). The existing RWX linker message remains;
it was already present in the frozen experiment. The 25 motor and 10 coordinator tests pass,
including wheel mapping, 70% cap, coast under that cap, standby ordering and boot readiness.

Useful reproduction commands, from the repo root:

```sh
kicad-cli sch export netlist experiments/rovi-m4-four-motor-safety/wiring.kicad_sch \
  --format kicadxml --output /tmp/rovi-motor-map.xml
python3 experiments/rovi-m4-four-motor-safety/tools/gen_wiring.py --check
python3 experiments/rovi-m4-four-motor-safety/build/gen_build_guide.py --check
cargo test --locked -p rovi-motor-control -p rovi-drive
```

Compare the XML GPIO pin names/nodes with the table and current firmware arrays/dispatch;
the schematic generator check alone does not compare the product firmware.
Build commands are in the [product README](../README.md#product-code).

Task 5.2 remains: individual wheels/directions, actual coast pin levels and neighbour behavior,
boot/reset/power loss, all stop triggers, radio/status delivery, and the selected-profile
reversal/thermal runs. The empty BLE Echo software fix is prepared; the
[Status/Echo probe](bench/ble_status_echo.py) still needs board delivery evidence. Earlier product BLE runs are development evidence on older firmware.
Every physical motor run is separately announced and waits for Todor's go.
