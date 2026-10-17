# M5 F4: first floor strafe — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Command over BLE: `drive --vx 0 --vy 0.05 --seconds 0.5`, after a 5 s lead-in (approximately 42.5% duty per wheel). Setup requested: car centred at HOME, FRONT unchanged, motor bank awake.

[Client output](strafe-left-005-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error. A second "go" arrived while this attempt was in progress; no second drive was started.

Todor reported **6.5 cm leftward travel**, then clarified that the **rear moved 1–2 mm farther left than the front**. This indicates a small heading change during the strafe; it is not a report of rearward translation. The angle and reference points were not measured. Physical stopping behavior was not separately confirmed.

Displacement divided by the 0.5 s command duration is approximately 0.13 m/s, including any unseparated coast after STOP; this is not steady-state lateral velocity. One run only; repeatability and opposite-direction behavior remain untested. Floor surface, bank charge and measurement uncertainty were not recorded.
