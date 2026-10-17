# M5 F1: mounted wheels, car on a stand — 2026-10-17

Setup: all four wheels clear of the floor, FRONT marked on the chassis, red tape on the front-left wheel. Control over BLE; no firmware changes for this run.

| Check | Command / evidence | Result |
| --- | --- | --- |
| Gentle forward | `drive --vx 0.1 --seconds 5`; [client output](forward-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor confirmed all four started, turned forward and stopped, without rubbing or hesitation. Pass. |
| Gentle strafe left | `drive --vx 0 --vy 0.1 --seconds 5`; [client output](strafe-client.txt): 101 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error | Todor reported no movement. Fail; cause not established. Check the motor bank is awake before repeating. |
| Strafe-left retry | Same command; [client output](strafe-retry-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor confirmed all four moved with FL/RR backward and FR/RL forward. Pass. No video recorded. |
| Strafe-left repeat for video | Same command, after a recording lead-in; [client output](strafe-recording-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor confirmed video captured; video not yet supplied for inspection. |
| Rotate left | `drive --vx 0 --wz 1.5 --seconds 5`, after a recording lead-in; [client output](rotate-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor confirmed both left wheels backward, both right wheels forward, all four starting and stopping properly. Pass. |
| Gentle-forward wheel speed | `drive --vx 0.1 --seconds 5`; [client output](wheel-speed-gentle-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor counted approximately 4.75 rotations of the red tape on FL in 5 s: 0.95 rev/s, 57 RPM, 5.97 rad/s. Video not supplied for independent inspection. |
| Mid-command forward wheel speed | `drive --vx 0.5 --seconds 5`; [client output](wheel-speed-mid-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor counted approximately 6.5 FL rotations in 5 s: 1.30 rev/s, 78 RPM, 8.17 rad/s; nominal rim speed 0.245 m/s at 60 mm diameter. Video not supplied for independent inspection. |
| High-command forward, first attempt | `drive --vx 1.0 --seconds 5`; [client output](wheel-speed-high-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | No rotation count supplied; repeated at Todor's request. |
| High-command forward, retry | Same command; [client output](wheel-speed-high-retry-client.txt): 101 setpoints, Armed → Stopped (`Stop`), no reported error | Todor counted approximately 8.75 FL rotations in 5 s: 1.75 rev/s, 105 RPM, 11.00 rad/s; nominal rim speed 0.330 m/s at 60 mm diameter. Video not supplied for independent inspection. |

The mounted-wheel direction checks (forward, strafe left and rotate left) pass. Three wheel-speed points are recorded; the full-duty measurement requested by F1 remains pending. Client status alone does not establish motor movement or motor-supply voltage. The successful strafe retry followed a request to wake the motor bank, but its voltage was not measured, so the first run's cause remains unconfirmed.

At the measured 60 mm wheel diameter, 0.95 rev/s corresponds to a nominal rim speed of approximately 0.179 m/s. This is an unloaded, whole-run average that may include acceleration and stopping; it is not measured floor speed or a maximum-wheel-speed calibration. The numeric `vx 0.1` command remains uncalibrated.

The connected firmware still reports placeholder geometry: wheel radius 0.050 m and centre projection sum 0.200 m. The measured car geometry has not been applied; this run checks direction and motion only, not speed calibration.
