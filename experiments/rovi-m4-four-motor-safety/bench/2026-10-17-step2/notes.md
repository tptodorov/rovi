# 2026-10-17 M4 step 2: channel mapping and direction, after the motor rework

Todor tuned the motors (after the 2026-10-16 FR start problem) and reconnected all four. Setup as 2026-10-16, motor bank woken before the runs, wheels not mounted (watching the shafts). Firmware `main` 28854d7. Log: `car.log`. Setup photo: [`setup.jpg`](setup.jpg) (chassis with FRONT/BACK marked, ESP32 and both drivers, C1/C2 next to the drivers).

Five 30 s runs; commanded duty per channel from the car log (FL, FR, RR, RL; ≤ 24 % doesn't start a motor here):

| Run | Command | FL | FR | RR | RL | Expected | Observed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | forward | 40 % | 40 % | 40 % | 40 % | all forward | (not counted) |
| 2 | diagonal fwd-left (`vx = vy = 0.5`) | 0 | 40 % | 0 | 40 % | FR, RL forward | FR, RL forward ✔ |
| 3 | diagonal fwd-right (`vx = 0.5, vy = −0.5`) | 40 % | 0 | 40 % | 0 | FL, RR forward | FL, RR forward ✔ |
| 4 | `vx = vy = 0.5, wz = −3` | 24 % | 16 % | −24 % | 64 % | RL only | RL only ✔ |
| 5 | `vx = 0.5, vy = −0.5, wz = 3` | 16 % | 24 % | 64 % | −24 % | RR only | RR only ✔ |

**Result:** every motor is on its intended channel and turns forward for a forward command; no firmware inversion needed (directions were set at the connectors). Mapping and direction (O8, M4 part) confirmed. Mecanum roller orientation is checked in M5 with the wheels on.

## Minimum-duty firmware (PR #42), same day

Duty now runs from 40 % (any command outside a 2 % deadband) to 100 % at 1.0. Flashed, booted cleanly (`car-min-duty.log`). Bank woken, all four motors connected:

| Run | Wheel command | Duty | Observed |
| --- | --- | --- | --- |
| Gentle forward, `vx` 0.1 | 5 % each | 43 % | all four start and turn forward ✔ |
| Rotate left, `wz` 3.0 | −30, +30, +30, −30 % (FL, FR, RR, RL) | 58 % | left backward, right forward ✔ |
| Forward, `vx` 1.0 | 50 % each | 70 % | all four, faster than before ✔ |

The dead zone (O6) is handled in the motor layer: the gentlest command starts every motor, rotation works.

## Reversal, lease expiry, thermal (same session, `car-min-duty.log`)

| Test | Log | Observed (Todor) |
| --- | --- | --- |
| Reversal: one session, +70 % ⇄ −70 % every 2 s, 10 changes | No `ev=boot` (no brownout reset), no BLE drop mid-run, STOP at the end | All four reverse cleanly; motor bank stays on ✔ |
| Lease expiry with four turning: 3 s at 70 %, then silent | Armed 190.0 s → `LeaseExpired` 194.1 s (1 s after the stream) | All four stop by themselves ✔ |
| Thermal: 5 min at 70 % | 300 s, 6000 setpoints, clean STOP, no reset | Drivers and motors at most warm ✔ |

STOP with four turning is shown by every run above (each ends with a client STOP, all wheels stop). Left for step 2: RST button and ESP32 power loss with four turning.

## Stops with four turning: RST and ESP32 power loss

Two 30 s runs at 70 %. Run 1: Todor pressed RST while all four turned; the BLE link dropped, the board rebooted, and run 2 then connected to the fresh boot (`Armed` at 14.4 s after the reset). Run 2: Todor unplugged the ESP32's USB while all four turned (motor bank on); the serial log ends with "Broken pipe".

Observed (Todor): **all four stopped at once on RST and on ESP32 power loss, and nothing restarted until a client armed again** ✔. With STOP, lease expiry and controller loss above, every stop trigger is shown with four motors.
