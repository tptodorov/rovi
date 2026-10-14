# 2026-10-14 M4 step 1: one driver, one motor

Setup: M1's wiring (WIRING.md step 1), power bank on VM, ESP32 on laptop USB, motor loose on the table. Firmware: M8 non-ROS build with motor outputs (PR #39), duty cap 800 ‰ (`vx` 1.0 = 40 % duty). Client: `sim/ble_client.py` over BLE. Logs: `car.log` (S2, S3 up to the unplug), `car-after-replug.log`, `car-s4-s5.log`.

| Test | Result |
| --- | --- |
| S1 `STBY` on a meter | Skipped (Todor): `STBY` behaviour is shown by S3 |
| S2 speed and direction | **Pass.** `vx` 0.25/0.5/1.0 (10/20/40 % duty) and -0.5: the motor ran in both directions and stopped after each run. Smallest turning duty not observed (left to M5, O6) |
| S3 STOP | **Pass.** `ev=stop reason=Stop`, motor stopped |
| S3 lease expiry | **Pass.** Stopped 1.0 s after the last setpoint (`LeaseExpired`) |
| S3 controller dies | **Pass.** Client killed: the BLE link stayed up at first, the lease stopped the car 1 s later |
| S3 RST button | **Pass.** Rebooted (`reset=ChipPowerOn`; the RST button is a chip enable reset), motor stopped and stayed off |
| S3 ESP32 unplugged | **Pass.** Motor stopped; after the replug the board booted idle, motor off |
| S4 radio while running | **Pass.** 300 s at 40 %, 6000 setpoints, no unplanned `ev=boot`, no BLE drop, status read afterwards |
| S5 5 min thermal | **Pass.** Driver cold after the run; the motor ran well throughout |

Motor stops in S3 were observed by Todor. An earlier S4/S5 attempt was cut short by a deliberate USB unplug, not a fault.
