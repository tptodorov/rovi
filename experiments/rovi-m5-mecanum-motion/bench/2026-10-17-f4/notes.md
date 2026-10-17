# M5 F4: first floor strafe — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Command over BLE: `drive --vx 0 --vy 0.05 --seconds 0.5`, after a 5 s lead-in (approximately 42.5% duty per wheel). Setup requested: car centred at HOME, FRONT unchanged, motor bank awake.

[Client output](strafe-left-005-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error. A second "go" arrived while this attempt was in progress; no second drive was started.

Todor reported **6.5 cm leftward travel**, then clarified that the **rear moved 1–2 mm farther left than the front**. This indicates a small heading change during the strafe; it is not a report of rearward translation. The angle and reference points were not measured. Physical stopping behavior was not separately confirmed.

For the first run, displacement divided by the 0.5 s command duration is approximately 0.13 m/s, including any unseparated coast after STOP; this is not steady-state lateral velocity. Floor surface, bank charge and measurement uncertainty were not recorded.

## Second leftward run

Same command and a 5 s lead-in. [Client output](strafe-left-005-repeat2-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor measured **5.5 cm left at the front and 6.0 cm left at the rear**. The rear travelled **5 mm farther left**, compared with the first run's reported 1–2 mm difference. Reference-point positions and heading angle were not measured; do not combine the first run's single distance with these two distances as equivalent measurements. Physical stopping was not separately reported.

## Third leftward run

Same command and a 5 s lead-in. [Client output](strafe-left-005-repeat3-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor measured **7 cm left at both front and rear**, using the same requested reference points. No front/rear difference was detected at the reported precision; physical stop timing was not separately reported.

| Leftward run | Front travel | Rear travel | Rear minus front |
| --- | --- | --- | --- |
| 1 | Single reported distance 6.5 cm; exact reference unspecified | Rear travelled 1–2 mm farther than front | 1–2 mm |
| 2 | 5.5 cm | 6.0 cm | 5 mm |
| 3 | 7.0 cm | 7.0 cm | No detected difference |

Leftward translation is demonstrated at this command. Travel varies between runs and rear swing was not consistent across all three; no compensation has been applied. Bank charge, surface variation and measurement uncertainty were not controlled or quantified.

## Rightward runs

Command: `drive --vx 0 --vy -0.05 --seconds 0.5`, with a 5 s lead-in for each attempt.

| Attempt | Client evidence | Observation |
| --- | --- | --- |
| Initial rightward pulse | [Output](strafe-right-005-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | No distance or physical result supplied; Todor requested a repeat. |
| Repeat | [Output](strafe-right-005-repeat2-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor measured **6.5 cm right at the front, 7.5 cm at the rear** and said it seemed to slip. |
| Second measured rightward run | [Output](strafe-right-005-repeat3-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor measured **7.0 cm right at the front, 7.5 cm at the rear**. No separate slip observation or physical-stop confirmation supplied. |

The rear travelled **1 cm farther right than the front** in the first measured run and **5 mm farther** in the second, indicating heading change of differing magnitude. Slip is an observer impression from the first measured run, not an independently established cause. Stop timing was not measured and physical stopping was not separately confirmed. Two measured runs at one command and condition are insufficient for general compensation; firmware remains unchanged.
