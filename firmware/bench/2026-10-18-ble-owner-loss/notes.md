# 2026-10-18: BLE owner-loss stop — controller-management disconnect

Product firmware `7c2e2df` (non-ROS), unchanged since the recorded Status/Echo flash.
Probe source `c17df69` (merged main). Todor authorized the announced single-scenario
run with “go”: gentle forward for three seconds, targeted controller disconnect,
require OwnerLost, then reconnect disarmed. Requested setup: stand, wheels clear,
motor bank awake. No firmware was flashed; opening the monitor reset the existing
firmware before capture.

| Check | Evidence |
| --- | --- |
| Boot / initial claim | Clean boot; BLE claim at 16,286 ms; claimed/clear Status; MTU 247 |
| Gentle forward | Armed at 17,970 ms, all four wheel commands 83 permille (about 42% duty) |
| Disconnect stop | `ev=stop reason=OwnerLost` at 21,160 ms, then release and BLE link-down at the same logged millisecond |
| Disarmed output | Open with all four wheel commands zero at 21,163 ms; no LeaseExpired event |
| Reconnect | Claim at 25,950 ms; immediate Status claimed/clear, no setpoint or successful ARM sent |
| Cleanup | STOP while disarmed; release at 29,440 ms, final Open/zero at 29,447 ms |
| Physical wheel observation | Not observed: Todor answered “I couldn’t observe the run”; physical stop/reconnect confirmation requires a repeat |

[probe.txt](probe.txt) contains the passing client assertion and disarmed reconnect.
[car.log](car.log) records the actual OwnerLost stop, unlike the two earlier attempts
that stopped by lease expiry. No reset, panic or `ev=error` occurred during this run.
Only one armed interval occurred. The serial capture is stopped; the board remains
on the same product firmware. No extra motor run followed.

The probe used `sudo -n btmgmt --timeout 3 --index 0 disconnect --type 2
<connected-peer-address>`, targeting only Rovi's LE-random connection on hci0.
This bypassed the BlueZ service's two-second D-Bus disconnect timer. No adapter
power toggle was issued. The software assertion passed on the
board. Todor could not observe this run, so it remains log-only evidence and the
physical wheel-stop/reconnect check needs a separately authorized repeat.

This does not measure STBY/PWM pin levels, numeric disable latency or the lease
receive-to-detection bound. It does not close the remaining UDP/ROS, powered reset /
power-loss, mapping/coast or reversal/thermal checks. Task 5.2 remains unchecked.
