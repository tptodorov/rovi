# 2026-10-17: BLE reconnect after task 3.2

Product firmware `33e2472` (branch `m6/sessions`, PR #60). Two separately connected BLE runs,
authorized by Todor's go, using `experiments/rovi-m8-setpoint-ros/sim/ble_client.py`.
Setup requested: car on its stand, wheels clear, motor bank awake.
Development evidence, not product acceptance.

| Run | Command | Serial/client evidence | Physical observation |
| --- | --- | --- | --- |
| 1 | `vx 0.1`, 3 s, STOP, disconnect | 60 setpoints; armed at 10,251 ms, STOP at 13,308 ms, released at 15,604 ms | Pending Todor's confirmation |
| 2 | Reconnect, `vx 0.1`, 3 s, STOP, disconnect | 60 setpoints; armed at 20,835 ms, STOP at 23,883 ms, released at 26,629 ms | Pending Todor's confirmation |

Both connections reused BLE handle 1 and successfully claimed, armed and released the car.
Wheel commands were 83 ‰ on all four wheels (about 42% duty). Boot completed and no `ev=error`
appears in [car.log](car.log); [runs.txt](runs.txt) contains both client results.
No serial capture remains running. The board remains on this firmware.

Not exercised physically: stale callback injection, sequence rollover, UDP or ROS sessions.
