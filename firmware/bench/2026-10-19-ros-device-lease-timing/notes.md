# 2026-10-19: ROS device-clock lease timing — passed

Todor authorized the announced five zero-speed ROS trials with “go”. Rover
on stand, wheels clear, USB connected and motor bank awake. Probe source
`8fa6ddd` (PR #84). [Procedure](../ROS-LEASE-TIMING.md).

No flash: the instrumented ROS firmware from the
[BLE device-clock batch](../2026-10-19-ble-device-lease-timing/notes.md) remains
installed (source `b12cc21`, ELF SHA-256
`514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`).
Opening serial capture resets the board; [car.log](car.log) begins at boot.

One ROS node/session, five zero-speed auto-arm/lease/recovery trials. Each
required fresh serial Armed evidence with all four wheel commands zero,
LeaseExpired with integer `lease_late_ms` in 0–50 ms, and matching ROS Armed
then lease-expired Status in the same session. The advertised lease was
1,000 ms. Every trial needed just one zero publish attempt; the probe went
silent after observing auto-arming. No explicit ARM true or nonzero command
was sent. Final cleanup sent STOP, destroyed the node and observed release.

| Trial | Device lateness after deadline (ms) | Limit (ms) | Host upper bound (ms), diagnostic only |
| --- | ---: | ---: | ---: |
| 1 | 4 | 50 | 1,015.278 |
| 2 | 9 | 50 | 1,023.038 |
| 3 | 2 | 50 | 1,021.835 |
| 4 | 5 | 50 | 1,019.408 |
| 5 | 4 | 50 | 1,022.557 |

**All five device-clock timing trials passed**, including matching ROS Status
and cleanup release ([probe.txt](probe.txt), `rc=0`). Device lateness is captured
at the stop decision against the actual accepted setpoint's deadline, before
it is cleared; queued logging and network/serial/laptop delay do not enter it.
Host values span the first zero publish attempt to serial observation, using
a separate laptop clock. No timestamps from different clocks were subtracted.

**Physical observation passed:** Todor confirmed all four wheels stayed still
through the serial reset, all five trials and cleanup: “Yes, all four stayed
still throughout”.

Serial audit: one boot, one ROS claim, five zero-wheel Armed states, five lease
stops and one release; final Open. Every wheel command was zero. No errors,
dropped events or panics. Capture stopped; router container removed.
[network.txt](network.txt) records joining the AP, adding/removing only the
scoped rule, restoring the previous Wi-Fi UUID and retaining Ethernet internet
routing. A final rule check confirmed absence. The initial iptables “Bad rule”
line is the expected missing-rule check before insertion; removal returned 0.

This accepts ROS lease detection timing for these five zero-wheel trials on
this instrumented build. BLE and UDP retain their separately recorded passing
timing evidence. STBY propagation and physical wheel stopping time are not
measured here. Coast/STBY levels and selected-profile reversal/thermal checks
remain pending, so task 5.2 stays open.
