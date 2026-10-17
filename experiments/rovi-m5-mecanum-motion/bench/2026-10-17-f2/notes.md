# M5 F2: first floor pulses — 2026-10-17

Setup requested: untethered car on its two onboard power banks, BLE control, measured-geometry firmware (radius 0.030 m, centre projection sum 0.1275 m). Floor surface, bank charge and total loaded mass for this run were not separately recorded.

Command: `drive --vx 0.05 --seconds 0.5` (approximately 42.5% PWM duty with the current normalization and motor mapping).

| Attempt | Client evidence | Observation |
| --- | --- | --- |
| Initial | [Output](forward-005-client.txt): BLE discovery failed before connecting | No setpoint or ARM sent; not a motor test. |
| Retry | [Output](forward-005-retry-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | No physical result supplied; Todor requested another run. |
| Repeat after a 5 s lead-in | [Output](forward-005-repeat-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor confirmed movement. Distance, sideways drift, heading change and physical stopping behavior were not separately quantified or confirmed. |
| Distance observation after a 5 s lead-in | [Output](forward-005-distance-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor reported approximately 10 cm forward "in good line": no obvious drift, not quantitatively measured. Physical stop timing and coast distance were not separately measured. |

The car moves at the lowest floor command tested so far (`vx 0.05`). This is not an exact starting threshold or calibrated speed. The observed 0.10 m displacement divided by the 0.5 s command duration is approximately 0.20 m/s, but may include coast after STOP and acceleration, so it is not steady-state driving speed. Repeatability, quantified drift and stopping distance remain pending. The numeric `vx 0.05` command does not currently mean an actual 0.05 m/s.
