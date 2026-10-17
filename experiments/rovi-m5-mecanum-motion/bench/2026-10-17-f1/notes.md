# M5 F1: mounted wheels, car on a stand — 2026-10-17

Setup: all four wheels clear of the floor, FRONT marked on the chassis, red tape on the front-left wheel. Control over BLE; no firmware changes for this run.

| Check | Command / evidence | Result |
| --- | --- | --- |
| Gentle forward | `drive --vx 0.1 --seconds 5`; [client output](forward-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor confirmed all four started, turned forward and stopped, without rubbing or hesitation. Pass. |

Strafe, rotation and wheel-speed measurement remain pending. F1 is not complete.

The connected firmware still reports placeholder geometry: wheel radius 0.050 m and centre projection sum 0.200 m. The measured car geometry has not been applied; this run checks direction and motion only, not speed calibration.
