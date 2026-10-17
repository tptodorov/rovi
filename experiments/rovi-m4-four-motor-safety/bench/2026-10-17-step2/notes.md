# 2026-10-17 M4 step 2: channel mapping and direction, after the motor rework

Todor tuned the motors (after the 2026-10-16 FR start problem) and reconnected all four. Setup as 2026-10-16, motor bank woken before the runs, wheels not mounted (watching the shafts). Firmware `main` 28854d7. Log: `car.log`.

Five 30 s runs; commanded duty per channel from the car log (FL, FR, RR, RL; ≤ 24 % doesn't start a motor here):

| Run | Command | FL | FR | RR | RL | Expected | Observed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | forward | 40 % | 40 % | 40 % | 40 % | all forward | (not counted) |
| 2 | diagonal fwd-left (`vx = vy = 0.5`) | 0 | 40 % | 0 | 40 % | FR, RL forward | FR, RL forward ✔ |
| 3 | diagonal fwd-right (`vx = 0.5, vy = −0.5`) | 40 % | 0 | 40 % | 0 | FL, RR forward | FL, RR forward ✔ |
| 4 | `vx = vy = 0.5, wz = −3` | 24 % | 16 % | −24 % | 64 % | RL only | RL only ✔ |
| 5 | `vx = 0.5, vy = −0.5, wz = 3` | 16 % | 24 % | 64 % | −24 % | RR only | RR only ✔ |

**Result:** every motor is on its intended channel and turns forward for a forward command; no firmware inversion needed (directions were set at the connectors). Mapping and direction (O8, M4 part) confirmed. Mecanum roller orientation is checked in M5 with the wheels on.
