# 2026-10-16 M4 step 2: four motors on the soldered carrier (in progress)

Setup: perfboard carrier per the build guide (C1/C2 fitted, motor capacitors C3-C6 not yet), ESP32 on its own power bank / laptop USB, motors on the motor power bank, wheels not mounted (watching the shafts). Firmware: `main` at 28854d7, four channels, duty cap 800 ‰ (`vx` 1.0 = 40 %). Client: `sim/ble_client.py` over BLE. Logs: `car*.log` (the capture dropped each time the board's USB was disturbed: "Broken pipe").

| Check | Result |
| --- | --- |
| Flash and boot, motors and motor power unplugged | Pass: `ev=boot reset=ChipPowerOn`, both drivers in standby |
| First drive (20 %) | Nothing moved: the power lines were faulty (fixed by Todor) |
| Direction, 40 % forward | All four turn; the reversed ones were fixed **at the connectors** (no firmware inversion) |
| Strafe left, 40 % | Pass: pattern as expected |
| Rotate left (24 %, the wz cap) | Did not start: below the four-motor start duty (see below); the log shows the correct command `-300,300,300,-300` (FL, FR, RR, RL) |
| **Motor power bank sleeps** | Idle it sits at **2.7 V** (below the TB6612's 4.5 V VM minimum) until its button is pressed; active, VM = **4.7 V** during a run |
| Start duty, single motor (bank awake) | Starts at 30 % |
| **Start duty, all four connected** | FL, RL, RR start at **35 %**; FR does not start by itself even at 40 % |
| FR fault isolation (FR ↔ RR plugs swapped) | **The fault follows the FR motor** (motor, gearbox or its harness); the carrier and driver 2 are fine |

## Findings

1. **The motor power bank auto-sleeps.** It must be woken before a run, or replaced/kept awake (bank with an always-on mode, a dummy load, sharing the ESP32 bank, or a battery without auto-off for the car).
2. **Mounted motors need about 35 % duty to start** with all four connected (a loose motor in step 1 started at 9 %). Input to O6: the firmware needs a start kick or a minimum duty before M5 floor driving.
3. **FR motor fault:** next, spin it by hand (stiffness) and measure its resistance at the plug versus another motor (harness/crimp).

Still to do for step 2: FR fix, stops with four turning (S3), reversal without brownout, 5 min thermal run, motor capacitors.
