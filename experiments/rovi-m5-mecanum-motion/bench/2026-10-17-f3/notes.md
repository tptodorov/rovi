# M5 F3: reverse floor pulses — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Each run used BLE command `drive --vx -0.05 --seconds 0.5` after a 5 s lead-in, with a request to reset at HOME and keep space behind clear. Approximate PWM duty: 42.5%.

| Attempt | Client evidence | Observation |
| --- | --- | --- |
| Initial | [Output](reverse-005-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | No physical observation supplied; repeat requested. |
| Second | [Output](reverse-005-repeat2-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | No physical observation supplied; repeat requested. |
| Third | [Output](reverse-005-repeat3-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor reported **10 cm backward, straight, clean stop**. |

Reverse motion and a clean physical stop are demonstrated at this command. The 10 cm distance is close to the three measured forward runs (10, 11, 10 cm), but only one reverse result was reported; do not count all three attempts as measured repeats. Drift and stopping distance were not quantified. Floor surface, bank charge and measurement uncertainty were not recorded.
