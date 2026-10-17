# M5 next session: low-command response and variation

Use the same firmware as the 2026-10-17 measured-geometry floor session: wheel radius 0.030 m, centre projection sum 0.1275 m, speed normalization still 40 rad/s, motor duty 40–100% above the deadband. No firmware change or flash is needed for this phase.

Surface: **parquet**, confirmed by Todor. Preserve this surface and note any seams or uneven patches under the wheels. Charge LED counts for both banks are not yet supplied. Record them before new runs where available; coarse LED bars are context, not a voltage measurement. Do not assign this surface or a charge state retroactively to earlier runs without confirmation.

## First: command response without changing firmware

Every motor run is separately announced and waits for Todor's "go". Use the 1 × 1 m area, a marked front edge at START, clear space ahead and wake the motor bank. Return to the same start position between runs. A 5 s lead-in allows filming and positioning; connection setup adds variable delay, so watch the car rather than relying on a wall-clock countdown.

| Forward command | Approximate wheel duty | Stream duration | Repeats | State |
| --- | --- | --- | --- | --- |
| `vx 0.05` | 42.5% | 0.5 s | 3 + session-end repeat | Earlier total travel 10, 11, 10 cm; end repeat 10 cm |
| `vx 0.20` | 50% | 0.5 s | First run, then up to 2 repeats after observation | Next; not run |
| `vx 0.40` | 60% | 0.5 s | Only after reviewing the 50% results | Pending; do not run automatically |

Each run ends with explicit STOP. Measure forward displacement, lateral displacement of the same front/rear reference points, heading change where visible, and whether the car stops cleanly. The first run at a new setting determines whether the duration and available space remain appropriate; cancel later runs if the car approaches the boundary or motion is abnormal. Do not combine this phase with lease-expiry tests or increase duty and duration together.

These are **whole-pulse response measurements**, including acceleration and any coast. They are useful for comparing settings and repeatability, not sufficient to label a command as accurate m/s. For steady speed, use original-time video with floor marks and measure position over a visible interval while motion is steady. Slow-motion playback duration is not capture duration. If 1 × 1 m does not give a steady interval, leave steady-speed calibration open rather than fitting a misleading scale factor.

## Then: controlled variation

Repeat the baseline and one characterized setting after a documented change in bank charge, keeping the same floor, firmware, geometry, start position and load. Record both banks' indicators and any supply-voltage measurement already available. Separate elapsed-session variation from a confirmed battery effect; regulated USB output may stay similar at different indicated charge levels.

For a load comparison, first agree and record a securely mounted payload and total mass. Do not add an unspecified load during a drive. Keep the motor commands and all other conditions the same. A single session-end baseline of 10 cm does not settle battery/load variation.

## Recording template

| Field | Value |
| --- | --- |
| Date, firmware source revision | |
| Surface and start position | parquet; |
| ESP32 bank / motor bank indicators | unknown / unknown |
| Total mass and secured payload | prior reported mass 500 g; current load to confirm |
| Command, stream duration, end condition | |
| Front/rear forward and lateral displacement | |
| Heading change, physical stop, slip observation | |
| Client output and original video path / capture frame rate | |

Store per-run observations and client output under `bench/<date>-calibration/`. Keep unreported fields unknown. Do not push a commit after every observation: save each result locally, then commit a coherent group of runs so CI has time to complete.

## Decisions after measurement

* Keep open-loop control within the current no-encoder scope. Do not infer that precise low-speed tracking is possible from successful short pulses.
* The 40% minimum means very small nonzero commands can already move the car appreciably. Establish a minimum usable speed or describe commands as uncalibrated references until a measured motor/velocity mapping exists.
* Do not use the earlier 105 RPM at 70% duty as full-duty `max_wheel_speed`, or set it as the new normalization value without reviewing the resulting duties. Those wheel counts used the earlier geometry.
* Full-duty behavior, per-wheel variation, actual stop timing/coast distance and a worst-case stopping bound remain separate work. The 6.5 vs 22.5 cm STOP/lease comparison is total-travel evidence at one low command.

M5 closes only when its remaining acceptance items are supported by evidence or explicitly deferred by Todor; these measurements do not by themselves accept M6 or change its speed contract.
