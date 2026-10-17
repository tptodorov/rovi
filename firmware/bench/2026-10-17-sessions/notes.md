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

## Requested repeat

Todor requested a repeat because physical confirmation of the original runs was unavailable.
The first repeat attempt did not connect: opening the monitor with `--no-reset` left the board
in its bootloader. No drive command reached the car ([retry-car.log](retry-car.log),
[retry-runs.txt](retry-runs.txt)). Restarting the same firmware with the normal monitor reset
restored advertising; no firmware was flashed during the repeat.

Both subsequent runs completed ([retry2-car.log](retry2-car.log), [retry2-runs.txt](retry2-runs.txt)):

| Run | Command | Serial/client evidence | Physical observation |
| --- | --- | --- | --- |
| 1 | `vx 0.1`, 3 s, STOP, disconnect | 60 setpoints; armed at 7,766 ms, STOP at 10,821 ms, released at 13,882 ms | Todor confirmed: all four forward, stopped cleanly |
| 2 | Reconnect, `vx 0.1`, 3 s, STOP, disconnect | 60 setpoints; armed at 21,090 ms, STOP at 24,142 ms, released at 26,887 ms | Todor confirmed: all four forward, stopped cleanly |

Both connections reused handle 1 and reported claimed → armed → stopped (STOP) with no error.
All four wheel commands were 83 ‰. The final state was open with all wheel commands zero,
and the serial capture was stopped.

Todor confirmed both repeated runs with “all worked” on 2026-10-17. The original runs remain
log-only evidence; this confirmation applies to the requested repeat.
