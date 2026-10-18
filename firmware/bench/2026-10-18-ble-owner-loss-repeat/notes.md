# 2026-10-18: repeated BLE owner-loss check

Todor could not observe the first successful [owner-loss run](../2026-10-18-ble-owner-loss/notes.md),
then authorized the announced repeat with “go”. Requested setup: stand, wheels clear,
motor bank awake. Product firmware remains `7c2e2df`, non-ROS; no flash or code change.
Probe source `aff98d7`. Opening the serial monitor reset the existing firmware.

| Check | Result |
| --- | --- |
| Gentle forward | Armed at 17,771 ms; all four wheel commands 83 permille, about 42% duty |
| Disconnect | OwnerLost, release and BLE link-down at 20,959 ms; Open/zero commands at 20,965 ms |
| Reconnect | Claim at 25,579 ms; immediate claimed/clear Status; no new setpoint or successful ARM |
| Final cleanup | STOP while disarmed; release at 28,294 ms; Open/zero at 28,297 ms |
| Wheel observation | Passed: Todor confirmed all four forward, stopped at disconnect and still after reconnect |

[probe.txt](probe.txt) passes the fresh serial OwnerLost assertion and disarmed
reconnect; [car.log](car.log) shows exactly one armed interval, no LeaseExpired,
panic, reset or `ev=error` during the run. The targeted management disconnect reached
Rovi before lease expiry. Capture stopped; the board remains on the same firmware.
No further motor run followed.

The first run remains log-only evidence. On 2026-10-18 Todor confirmed this repeat
with “Yes, all as expected”: all four wheels ran forward, stopped at disconnect
and stayed still after reconnect. This BLE owner-loss wheel-stop/reconnect scenario
passes physically at the tested gentle setting. Other task 5.2 checks remain open; no pin levels or numeric
stop latency are inferred from these logs.
