# 2026-10-18: repeated BLE stop/recovery batch — owner loss still open

Todor requested a full repeat to observe the wheels, then authorized the announced
STOP/lease/disconnect/out-of-limits batch with “go”. Setup requested: stand, wheels
clear, motor bank awake. Product firmware remains `7c2e2df`, non-ROS; no new flash.
Probe source `7840078`. The serial monitor reset the existing firmware before capture.

| Scenario | Serial/client result | Physical result |
| --- | --- | --- |
| STOP | Armed 17,150 ms; STOP 20,295 ms. ARM without fresh input refused; fresh stream remained stopped with NotReady; explicit ARM 22,231 ms, STOP 25,380 ms. | Passed: Todor confirmed all four wheels behaved as expected |
| Lease | Armed 34,929 ms; LeaseExpired 39,029 ms. Notification received 1,098 ms after client stream return. ARM refused until fresh input; stream alone stayed stopped; explicit ARM 41,047 ms, STOP 44,190 ms. | Passed: Todor confirmed all four wheels behaved as expected |
| Disconnect | Armed 54,094 ms; **LeaseExpired 58,204 ms**, release/link-down 59,670 ms. The corrected probe rejected this trigger and halted before LIMITS. | Owner-loss stop not exercised |
| Out-of-limits | Completed separately within the already-announced batch: armed 102,294 ms; `vx=2.0` raised OutOfLimits 105,436 ms. Fresh stream alone stayed stopped; explicit ARM 107,375 ms, STOP 110,520 ms. | Passed: Todor confirmed all four wheels behaved as expected |

On 2026-10-18, Todor answered “Yes, all as expected” to whether all four wheels
stopped for STOP, lease expiry and out-of-limits, stayed still during the fresh-command
pauses, and restarted only during the announced recovery runs. These three BLE
physical scenarios pass at the tested gentle setting. Disconnect owner loss remains
unexercised; the confirmation does not close the full task 5.2 acceptance matrix.

[probe.txt](probe.txt) contains STOP/lease and the failed owner-loss assertion;
[probe-limits.txt](probe-limits.txt) contains the remaining announced LIMITS check.
[car.log](car.log) shows seven armed intervals at `83,83,83,83` wheel permille
(about 42% requested duty). No reset, panic or motor fault occurred during the batch.
Final state: Open with zero wheel commands at 112,696 ms. Both serial and packet
captures are stopped; no subsequent motor run was made.

The client notification times include host/BLE delay and pacing; they do not prove
the lease+50 ms detection bound. Commands/status alone do not prove pin levels or
physical wheel behavior. Task 5.2 remains unchecked.

## Why direct Bleak disconnect still missed the trigger

Removing unsubscribe-first close did not bypass BlueZ's Device.Disconnect policy.
The installed BlueZ 5.86 backend calls that D-Bus method. Its
[source](https://github.com/bluez/bluez/blob/5.86/src/device.c#L2100) schedules
`disconnect_all` using a [two-second timer](https://github.com/bluez/bluez/blob/5.86/src/device.c#L71),
which exceeds the rover's one-second lease.

The [Rovi-only trace excerpt](disconnect-timing.txt), captured during the final
LIMITS cleanup with motors already stopped, shows the delay before the management
command and then prompt controller completion. Other devices' Bluetooth traffic
is excluded; the raw trace remains in `/tmp/rovi-ble-stops-repeat.btsnoop`.
This trace supports the host-delay explanation; it does not independently capture
the earlier failed DISCONNECT attempt.

## Next owner-loss probe prepared, not run

The probe now uses passwordless `sudo -n btmgmt --timeout 3 --index 0 disconnect
--type 2 <connected-peer-address>`, targeting only Rovi's LE-random address on hci0.
It bypasses the BlueZ service's timer, retains the fresh serial OwnerLost assertion,
and reconnects disarmed. No adapter power toggle or firmware change is involved.
The command choice follows the installed tool's help and BlueZ 5.86
[management client](https://github.com/bluez/bluez/blob/5.86/client/mgmt.c#L2829).

A laptop test first failed because the targeted command was not invoked; all five
probe tests now pass, including target address, wrong-trigger rejection and cleanup.
Actual management-command delivery with the rover has **not** been tested.
A new separately announced go is needed before a three-second gentle forward /
management-disconnect / disarmed-reconnect run (about 15 seconds).

Other acceptance work remains: individual wheel directions, coast/STBY pin levels,
powered reset and power loss, radio exclusion, UDP/ROS stop delivery, reversals/heat,
and the lease detection timing measurement.
