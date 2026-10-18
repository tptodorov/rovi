# 2026-10-18: BLE stop/recovery batch — partial evidence

Product firmware `7c2e2df` (non-ROS), already flashed in the
[Status/Echo run](../2026-10-18-ble-status-echo/notes.md); no new flash.
Probe source `bd2f317`, PR #69. Todor authorized the announced four-scenario
batch with “go”. Requested setup: stand, all wheels clear, motor bank awake.
Opening the serial monitor reset the existing firmware before the batch.

The [client output](probe.txt) reports a pass, but review of [car.log](car.log)
invalidates its disconnect result. Overall acceptance is **partial**. Physical
wheel observations for the other three scenarios are pending Todor's confirmation.

| Scenario | Serial/client evidence | Physical result |
| --- | --- | --- |
| STOP | Armed at 16,591 ms; STOP at 19,733 ms. Immediate ARM refused. Fresh stream stayed stopped with NotReady; explicit ARM at 21,673 ms, STOP at 24,818 ms. | Pending confirmation |
| Lease | Armed at 34,904 ms; LeaseExpired at 39,003 ms. Client received the stop notification 1,054 ms after its stream returned. Immediate ARM refused; fresh stream stayed stopped; explicit ARM at 40,981 ms, STOP at 44,123 ms. | Pending confirmation |
| Disconnect | Armed at 54,299 ms; **LeaseExpired** at 58,399 ms; BLE down/release only at 59,288 ms. Reconnect at 61,288 ms was claimed/disarmed with no error. | Owner-loss stop not exercised; repeat required |
| Out-of-limits | Armed at 71,405 ms; `vx=2.0` (limit 1.0) caused OutOfLimits at 74,543 ms. Immediate ARM refused; fresh stream stayed stopped with NotReady; explicit ARM at 76,485 ms, STOP at 79,628 ms. | Pending confirmation |

All seven armed intervals logged `83,83,83,83` permille wheel commands
(about 42% requested duty). STOP/lease/out-of-limits produced zero wheel commands.
No reset, panic or `ev=error` occurred during the batch. Final state at 82,297 ms:
Open, zero wheel commands. Serial capture stopped; existing firmware remains installed.

Client receipt timing includes BLE/host delay and the stream's final pacing interval;
it does not establish the lease+50 ms device-detection bound. Zero commands and
status alone do not establish physical standby pin levels or actual wheel stops.

## Disconnect probe correction

`Car.close()` unsubscribes from Status before disconnecting. In this run the
connection remained up long enough for the one-second lease to expire first.
The original probe only checked reconnection and therefore falsely accepted the
wrong stop trigger. The new regression test reproduced that false pass first
(`AssertionError not raised`).

The corrected probe disconnects the peer directly and requires the **first fresh
serial stop** after that operation to be `OwnerLost`. It rejects LeaseExpired
and ignores old OwnerLost events before its capture offset. Five laptop tests
pass, including the positive owner-loss case and cleanup/profile guards.
This correction has **not** been rerun on the board.

A single-scenario repeat is prepared, after a new announced go, with the same
stand/motor-bank setup: five-second lead-in, `vx=0.1` for three seconds, direct
BLE disconnect, require OwnerLost in serial, reconnect disarmed. No recovery
motor run or flash is included in that repeat.

```sh
nix-shell --run 'python3 firmware/bench/ble_stop_checks.py DISCONNECT --carlog <live-car.log>'
```

The full hardware acceptance matrix remains open: this batch does not cover
individual wheel mapping, pin levels, powered reset/power loss, actual radio
exclusion, UDP/ROS stop delivery, reversals or heat. Task 5.2 remains unchecked.
