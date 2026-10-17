# M5 command-response calibration on parquet — 2026-10-17

Surface: parquet, confirmed by Todor. Same measured-geometry firmware as the prior floor session (radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder). No new firmware flash for this phase. ESP32/motor bank charge indicators and current total mass have not been supplied for this phase; prior reported assembled mass was 500 g.

Forward command: `drive --vx 0.20 --seconds 0.5`, approximately 50% duty, with a 5 s lead-in and explicit STOP. Reset to START between trials.

| Trial | Client evidence | Observation |
| --- | --- | --- |
| 1 | [Output](forward-020-trial1-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor measured **13.5 cm forward travel**. Drift, heading change and physical stop behavior not separately reported. |
| 2 | [Output](forward-020-trial2-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor measured **14 cm forward travel**. Drift, heading change and physical stop behavior not separately reported. |
| 3 | [Output](forward-020-trial3-client.txt): 11 setpoints, Armed → Stopped (`Stop`), no reported error | Todor measured **14 cm forward travel**. Drift, heading change and physical stop behavior not separately reported. |

Three reported distances: **13.5, 14, 14 cm**, mean **13.8 cm**, spread **0.5 cm**. This is repeatability at one command under the current conditions, not across battery/load states. Drift, heading and physical stopping have not been separately reported for these trials.

Distances are whole-pulse travel, including acceleration and any coast. Do not interpret the numeric command as calibrated m/s or infer steady-state speed from travel divided by stream duration. Save trial observations locally, then commit the completed group.

## Scope decision after the three trials

Todor reported **"no performance changes noticed"** and asked to use the previous m/s and rad/s measurements. Accept the existing whole-pulse estimates as approximate open-loop references ([table](../../CALIBRATION.md)); stop the extended sweep. The proposed `vx 0.40` / 60%-duty run was not performed. No new firmware gains or calibration constants were applied. No controlled payload/charge comparison or velocity-accuracy guarantee is claimed.
