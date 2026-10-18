# 2026-10-18: ROS stop/recovery batch — physically confirmed

Todor authorized the announced three-scenario batch with “go”. Requested setup:
rover on stand, all wheels clear, USB connected, motor bank connected and awake.
No flash: the installed ROS product firmware is from the
[read-only batch](../2026-10-18-ros-readonly/notes.md), ELF SHA-256
`e987f9f75f7f7de20da70803266eff0efe7a601e40e920f19cc88b6c7fa2b00c`.
Firmware Rust source is unchanged since `7c2e2df`. Probe/runner source `59a7d10`
(PR #74). Opening the serial monitor reset the existing firmware before capture.

The [runner](../ros_stops.sh) used one ROS node with cmd_vel/arm publishers.
[probe.txt](probe.txt) contains the assertions; [car.log](car.log) is raw serial.
All six driving intervals commanded `83,83,83,83` wheel permille at `vx=0.1`,
about 42% requested duty.

| Scenario | Serial/protocol evidence | Physical result |
| --- | --- | --- |
| STOP | Armed at 32,055 ms; STOP at 35,089 ms. Premature ARM reported not-ready. Fresh streaming stayed stopped; explicit ARM at 36,647 ms, final STOP 39,687 ms, release 45,649 ms. | Passed: all four stopped, stayed still during pause, restarted only for explicit recovery |
| Lease | Armed at 50,698 ms; LeaseExpired at 54,703 ms, about one second after the three-second stream ended. Silent pause stayed stopped; resumed commands auto-armed at 56,238 ms, STOP 59,274 ms, release 65,226 ms. | Passed: all four stopped automatically, stayed still during silence, restarted only when announced commands resumed |
| Out-of-limits | Armed at 70,295 ms; `vx=2.0` raised OutOfLimits at 73,355 ms. Premature ARM reported not-ready. Fresh streaming stayed stopped; explicit ARM at 74,912 ms, final STOP 77,954 ms, release 83,913 ms. | Passed: all four stopped, stayed still during pause, restarted only for explicit recovery |

On 2026-10-18 Todor answered “Yes, all as expected” to whether all four wheels
turned forward, stopped for all three triggers, stayed still during pauses, and
restarted only for the announced recoveries (explicit ARM after STOP/limits;
resumed commands after lease expiry). These three physical ROS scenarios pass
at the tested gentle setting. Automatic recovery after lease expiry is the
specified ROS exception; STOP/out-of-limits stay latched during valid streaming.

Every transition/release required fresh serial and matching ROS Status evidence.
The log checker found one application boot, three ROS claims/releases, six armed
intervals and six stops, with no application errors, panic or dropped events.
Bootloader output retains the multiple-DROM-segments diagnostic also present in
the earlier read-only capture. Final state: Open/zero commands at 83,913 ms.

The runner removed its Docker router (serial link-down 84,091 ms). The network
wrapper removed only its added source/interface/address-scoped TCP/7447 rule,
restored the prior Wi-Fi connection, and verified its UUID and unchanged Ethernet
internet route. Serial capture stopped. The rover remains on ROS firmware and
will retry its router while disconnected. No additional motor run followed.

## Remaining acceptance

This batch does not establish the numeric lease+50 ms detection bound or GPIO
disable latency. ROS publisher/owner/router loss while driving, radio exclusion,
individual wheel directions, coast/STBY pin levels, powered reset/power loss and
selected-profile reversal/thermal behavior remain pending. Task 5.2 stays open.
