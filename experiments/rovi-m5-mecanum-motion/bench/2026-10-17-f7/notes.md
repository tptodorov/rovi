# M5 F7: automatic lease stop on the floor — 2026-10-17

Measured-geometry firmware, BLE control, `lease_ms=1000`. Setup requested: at least 60 cm clear ahead and motor bank awake.

After a 5 s lead-in, ran `drive --vx 0.05 --seconds 0.3 --quiet 2`: 0.3 s of setpoints, then 2 s connected but silent. [Client output](lease-forward-005-client.txt): 7 setpoints, Claimed → Armed → Stopped (`LeaseExpired`), no reported error. The client's later STOP did not replace the observed lease-expiry reason.

Todor confirmed the car moved forward and **physically stopped by itself**. Total distance was not known. No video, time of physical stop or position at stream cessation was supplied.

Automatic lease stopping is demonstrated on the floor at this command. **Total travel, distance after the last setpoint, mechanical coast distance and exact stop timing remain unmeasured.** Do not infer them from the configured 1 s lease or the whole-run speeds measured in other tests. No high-speed stopping test has been performed.

## Measured repeat

Repeated the same command after a 5 s lead-in, with a request to mark the car's front edge before starting and measure to the same edge after stopping. [Client output](lease-forward-005-repeat2-client.txt): 7 setpoints, Armed → Stopped (`LeaseExpired`), no reported error.

Todor reported **22.5 cm total forward travel**. This includes travel during the 0.3 s stream, the remaining lease interval, and any coast. It is **not** a measured distance from the final setpoint or from motor-output disable. Those positions and exact stop timing remain unknown. A matching 0.3 s run ending in explicit STOP has not yet been measured for comparison.
