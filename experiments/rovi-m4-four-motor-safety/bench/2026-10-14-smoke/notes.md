# 2026-10-14 smoke test: motor firmware on the bare board

Board only, nothing on GPIO4-7 (no driver, no motor). M8 firmware with motor outputs (non-ROS build), BLE client from the laptop.

* `ev=boot ver=0.1.0 reset=ChipPowerOn`: the reset cause is logged.
* `ble_client.py drive --vx 1.0 --seconds 3 --quiet 3`: claimed, armed at `wheels_pm=500` (wheel command 0.5, so 40 % duty with the 800 ‰ cap), `LeaseExpired` 4.0 s after arming (1 s after the last setpoint), released at disconnect.
* `drive --vx -0.5 --seconds 2`: armed at `wheels_pm=-250` (20 % duty, reverse), stopped by STOP.
* Two Wi-Fi AP off/on switches around the BLE sessions and no unplanned `ev=boot`: the 1 s RTC watchdog did not fire in normal operation.

Not checked here (needs the wiring): the `STBY` and PWM pin levels, and the watchdog actually resetting a hung chip.
