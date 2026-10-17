# M5 F4: first floor strafe — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Command over BLE: `drive --vx 0 --vy 0.05 --seconds 0.5`, after a 5 s lead-in (approximately 42.5% duty per wheel). Setup requested: car centred at HOME, FRONT unchanged, motor bank awake.

[Client output](strafe-left-005-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error. A second "go" arrived while this attempt was in progress; no second drive was started.

Todor reported **6.5 cm leftward travel**, then clarified that the **rear moved 1–2 mm farther left than the front**. This indicates a small heading change during the strafe; it is not a report of rearward translation. The angle and reference points were not measured. Physical stopping behavior was not separately confirmed.

For the first run, displacement divided by the 0.5 s command duration is approximately 0.13 m/s, including any unseparated coast after STOP; this is not steady-state lateral velocity. Floor surface, bank charge and measurement uncertainty were not recorded.

## Second leftward run

Same command and a 5 s lead-in. [Client output](strafe-left-005-repeat2-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor measured **5.5 cm left at the front and 6.0 cm left at the rear**. The rear travelled **5 mm farther left**, compared with the first run's reported 1–2 mm difference. Reference-point positions and heading angle were not measured; do not combine the first run's single distance with these two distances as equivalent measurements. Physical stopping was not separately reported.

Two leftward runs show rear swing, with differing magnitude. More repeat measurements and rightward motion remain pending; no compensation has been applied from these observations.
