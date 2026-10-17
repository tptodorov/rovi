# M5 empirical motion references

On 2026-10-17 Todor reported **no performance changes noticed** and chose to use the existing m/s and rad/s estimates. Stop the extended command sweep; the proposed 60%-duty runs were not performed. These references are sufficient for the current open-loop bring-up scope. No encoder work or further motor run is authorized by this record.

## Accepted approximate references

Values below divide reported floor displacement/angle by the 0.5 s command duration. They include acceleration and any coast after STOP, so they are **whole-pulse estimates, not steady-state speed measurements or guaranteed velocity tracking**. The numeric command and observed result are deliberately shown separately.

| Command | Observed travel / angle | Approximate motion reference |
| --- | --- | --- |
| Forward `vx 0.05` | 10, 11, 10 cm; end repeat 10 cm | 0.21 m/s from the initial three-run mean |
| Forward `vx 0.20` | 13.5, 14, 14 cm | 0.28 m/s |
| Reverse `vx -0.05` | 10 cm | 0.20 m/s backward |
| Strafe `vy ±0.05` | Reported front/rear travel spans 5.5–7.5 cm | 0.11–0.15 m/s at the observed reference points; some heading change |
| Rotate left `wz 0.5` | 40° twice | 1.4 rad/s left |
| Rotate right `wz -0.5` | 40–45° | 1.4–1.6 rad/s right |
| Diagonal `vx 0.05`, `vy ±0.05` | 4.5 cm forward and 4.5 cm sideways, each direction | 0.09 m/s per axis; magnitude about 0.13 m/s |

Geometry is measured and flashed: wheel radius 0.030 m, wheelbase 0.115 m, track 0.140 m, centre projection sum 0.1275 m. The motors have no encoders; closed-loop sensing remains out of scope. The current motor duty mapping and the placeholder 40 rad/s normalization are unchanged. These observations do not justify replacing that parameter with 105 RPM: that unloaded wheel count was at 70% duty under the older geometry, not a full-duty measurement.

## Consistency and stopping

* Todor noticed no performance change. Forward baseline 10, 11, 10 cm and session-end repeat 10 cm support qualitative session consistency. No controlled battery-charge or payload comparison was performed; none is claimed.
* The 50%-duty forward group spans only 0.5 cm (13.5, 14, 14 cm). Parquet was confirmed for this group. Charge indicators and current load were not separately recorded.
* A 0.3 s stream at `vx 0.05` gave 6.5 cm total travel with explicit STOP and 22.5 cm with lease expiry. The extra 16 cm is a difference between whole-run distances, not an isolated mechanical stopping distance. Physical automatic stopping was confirmed.

## Use and limits

Use the table as empirical references for manual teleoperation and M6 requirements discussions. Keep the standard Twist command fields in m/s and rad/s as requested references; do not claim that current firmware makes the car track them accurately. No interpolation, new gains, limits or compensation have been implemented from these estimates.

Detailed evidence: [forward baseline](bench/2026-10-17-f2/notes.md), [50%-duty trials](bench/2026-10-17-calibration/notes.md), [reverse](bench/2026-10-17-f3/notes.md), [strafe](bench/2026-10-17-f4/notes.md), [rotation](bench/2026-10-17-f5/notes.md), [diagonals](bench/2026-10-17-f6/notes.md), [stops](bench/2026-10-17-f7/notes.md), [session-end repeat](bench/2026-10-17-f8/notes.md).

Full-duty performance, minimum steady usable speed, accuracy bounds, controlled battery/load effects and worst-case stopping distance remain unverified. Resume measurement only for a concrete requirement that needs one of those answers, with each motor test announced and started on Todor's go. No 60%-duty test is queued.
