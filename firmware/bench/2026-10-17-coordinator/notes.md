# 2026-10-17: first board run of the product firmware (M6 tasks 2.1-2.3)

Product firmware at `5a51254` (branch `m6/coordinator`, PRs #55/#56): motor pins driven through the
`rovi-drive` coordinator, product profile 40-70 % duty, PWM-high coast. Car on its stand, wheels clear,
two power banks, BLE from the laptop (`experiments/rovi-m8-setpoint-ros/sim/ble_client.py`).
Serial log: [car.log](car.log). Development evidence for task 5.2, not product acceptance.

| Run | Command | Car log | Observed (Todor) |
| --- | --- | --- | --- |
| 1 | Flash, boot, BLE info | `ev=boot reset=ChipPowerOn`, no `ev=error`; MTU 247; capabilities radius 0.030, sum 0.1275, limits 1/1/3, lease 1000 ms ([info](info.txt)) | - |
| 2 | `vx 0.1`, 5 s, STOP | Armed 5.1 s at 83 ‰ (about 42 % duty), `ev=stop reason=Stop` | All four started, forward, stopped cleanly |
| 3 | `vx 1.0`, 5 s, STOP | Armed 5.1 s at 833 ‰ (about 65 % duty, cap 70 %), `ev=stop reason=Stop` | All four forward, stopped cleanly; turns not counted |
| 4 | `vx 0`, 15 s armed (every wheel coasts: IN1 = IN2 = low, PWM high), STOP | Armed 15.1 s at 0 ‰ | Worked as expected: a hand-spun wheel turned freely, no wheel moved on its own |
| 5 | `vx 0.5` 3 s, then silent 3 s | Armed at 417 ‰ (about 52 %), `ev=stop reason=LeaseExpired` 4.0 s after arming (about 1 s after the stream) | All four stopped by themselves |

No `ev=error` and no unplanned reboot across the five runs ([runs 2-3](runs-1.txt), [runs 4-5](runs-2.txt)). Not
checked here: STBY and PWM pin levels (a multimeter or logic analyzer), STOP/RST/power-loss with this firmware,
reversal and thermal at the 70 % cap, ROS. The coast check is a feel test, not a pin measurement.
