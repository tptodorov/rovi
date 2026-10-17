# M5 F1: mounted wheels, car on a stand — 2026-10-17

Setup: all four wheels clear of the floor, FRONT marked on the chassis, red tape on the front-left wheel. Control over BLE; no firmware changes for this run.

| Check | Command / evidence | Result |
| --- | --- | --- |
| Gentle forward | `drive --vx 0.1 --seconds 5`; [client output](forward-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor confirmed all four started, turned forward and stopped, without rubbing or hesitation. Pass. |
| Gentle strafe left | `drive --vx 0 --vy 0.1 --seconds 5`; [client output](strafe-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor reported no movement. Fail; cause not established. Check the motor bank is awake before repeating. |
| Strafe-left retry | Same command; [client output](strafe-retry-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor confirmed all four moved with FL/RR backward and FR/RL forward. Pass. No video recorded. |
| Strafe-left repeat for video | Same command, after a recording lead-in; [client output](strafe-recording-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Run completed; video and physical observation for this repeat not yet supplied. |

Rotation and wheel-speed measurement remain pending. F1 is not complete. Client status alone does not establish motor movement or motor-supply voltage. The successful retry followed a request to wake the motor bank, but its voltage was not measured, so the first run's cause remains unconfirmed.

The connected firmware still reports placeholder geometry: wheel radius 0.050 m and centre projection sum 0.200 m. The measured car geometry has not been applied; this run checks direction and motion only, not speed calibration.
