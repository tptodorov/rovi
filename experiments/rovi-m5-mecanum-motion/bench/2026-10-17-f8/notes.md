# M5 F8: end-of-session forward repeat — 2026-10-17

Same measured-geometry firmware and command as the earlier forward baseline: `drive --vx 0.05 --seconds 0.5`, after a 5 s lead-in and a request to reset to the same starting mark. BLE control, explicit STOP at the end.

[Client output](forward-005-end-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor reported **10 cm total forward travel**, within the earlier **10, 11, 10 cm** baseline. No reduction in displacement was observed in this one end-of-session run. Drift and physical stopping were not separately described for this repeat.

Bank charge/voltage, elapsed driving time, surface and payload were not controlled or recorded for comparison. This is a session-end consistency observation, **not a measured battery-discharge curve, runtime result or validation across battery/load conditions**. Commanded m/s remains uncalibrated.
