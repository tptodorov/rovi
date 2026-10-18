# 2026-10-18: product UDP stop/recovery batch — physically confirmed

Product firmware `7c2e2df` (non-ROS), unchanged since the recorded Status/Echo flash.
Probe source `9d166b7` (PR #71), run by its explicit `192.168.4.1:7777` target.
Todor authorized the announced four-scenario batch with “go”. Requested setup:
stand, all wheels clear, motor bank awake. No flash; opening the monitor reset the
existing firmware before capture.

| Scenario | Serial/client evidence | Physical result |
| --- | --- | --- |
| STOP | Armed 16,407 ms; STOP 19,415 ms. Immediate ARM refused; fresh stream alone remained stopped; explicit ARM 20,983 ms, STOP 23,993 ms. | Passed: all four stopped, remained still in pause, restarted only for recovery |
| Lease | Armed 29,068 ms; LeaseExpired 33,033 ms. Owner received unsolicited Status `[3,2,0]` with no request. Fresh stream alone remained stopped; explicit ARM 34,600 ms, STOP 37,614 ms. | Passed: all four stopped automatically; pause/recovery as expected |
| BYE | Armed 42,685 ms; OwnerLost/release at 45,692 ms. New claim at 45,743 ms has a distinct nonzero session; ARM without a setpoint refused. | Passed: all four stopped and stayed still in the new-session check |
| Out-of-limits | Armed 50,822 ms; `vx=2.0` (limit 1.0) raised OutOfLimits at 53,825 ms with unsolicited Status `[3,4,0]`. Fresh stream alone remained stopped; explicit ARM 55,395 ms, STOP 58,406 ms. | Passed: all four stopped; pause/recovery as expected |

On 2026-10-18, Todor answered “Yes, all as expected” to whether all four wheels
stopped for STOP, lease expiry, BYE and out-of-limits, stayed still during the
fresh-command pauses/new-session check, and restarted only during recovery.
These four UDP physical scenarios pass at the tested gentle setting.

[probe.txt](probe.txt) contains the protocol assertions; [car.log](car.log) records
seven armed intervals at `83,83,83,83` wheel permille, about 42% requested duty.
The five UDP claims include the disarmed new session after BYE. Every cleanup
required a fresh serial release event. No reset, panic, `ev=error` or dropped log
event occurred during the batch. Final state at 58,411 ms: Open/zero commands.
Serial capture stopped; no extra motor run followed.

The first scenario also checked a separate UDP endpoint: HELLO returned BUSY,
its STOP with the owner's session produced no reply, and the owner's next ARM
without a setpoint still reported Claimed/NotReady. This covers a second source
port on the same laptop/Wi-Fi station, not a second physical station or BLE/AP
radio exclusion.

## Network handling

The existing `rovi-m8` profile joined Rovi's AP on the laptop's Wi-Fi interface.
Its no-default IPv4 setting was retained; IPv6 no-default was temporarily enabled
and its previous setting restored. Ethernet remained the internet route throughout.
After the run, the wrapper restored the previous Wi-Fi connection and verified
its UUID and the unchanged Ethernet route. No network credentials were printed
or added to this evidence.

## Remaining acceptance

This does not establish the lease+50 ms receive-to-detection bound, STBY/PWM pin
levels or numeric disable latency. ROS, actual BLE/AP radio exclusion, individual
wheel directions, powered reset/power loss, coast and selected-profile reversal /
thermal checks remain open. Task 5.2 stays unchecked.
