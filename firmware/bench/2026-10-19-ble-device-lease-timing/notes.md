# 2026-10-19: BLE device-clock lease timing — passed

Todor authorized the announced instrumented-ROS flash and five zero-wheel BLE
trials with “go”. Rover on stand, wheels clear, USB connected, motor bank awake.
Firmware and probe source: `b12cc21` (PR #82, stacked on #81).
[Procedure](../BLE-LEASE-TIMING.md).

Flashed the prepared ROS ELF from
`firmware/target/ros/xtensa-esp32s3-none-elf/release/rovi-firmware`.
SHA-256: `514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`.
The [digest record](firmware.sha256) and [flash output](flash.log) identify the
image and completed flash. The AP credential matches the existing laptop
profile. The instrumented ROS firmware remains installed. Capture opening
resets the board, so [car.log](car.log) starts at boot.

One BLE connection, one accepted zero setpoint and explicit ARM per trial,
then silence until expiry. The arbiter records `lease_late_ms` as stop-decision
time minus the actual stored lease deadline, before clearing it. The value is
independent of queued logging, BLE/serial delivery and laptop observation.
Each trial required an integer from 0 through 50, fresh Armed/zero-wheel
serial evidence and a fresh BLE lease-expired Status notification.
The advertised lease was 1,000 ms.

| Trial | Device lateness after deadline (ms) | Limit (ms) | Host upper bound (ms), diagnostic only |
| --- | ---: | ---: | ---: |
| 1 | 7 | 50 | 1,058.148 |
| 2 | 4 | 50 | 1,049.366 |
| 3 | 4 | 50 | 1,050.000 |
| 4 | 4 | 50 | 1,048.756 |
| 5 | 2 | 50 | 1,050.856 |

**All five device-clock timing trials passed.** [probe.txt](probe.txt) contains
the assertions' successful result (`rc=0`). The host values include outbound
BLE delivery, serial capture, polling and scheduling. This run demonstrates
that a host upper bound above 1,050 ms can coexist with device detection only
2–7 ms after the deadline. It does not distinguish individual transport/host
delays or retroactively identify the cause of the
[first attempt](../2026-10-19-ble-lease-timing/notes.md), whose failed probe result
is preserved for its original firmware and measurement method.

**Physical observation passed:** Todor confirmed all four wheels stayed still
through flashing, boot, all five trials and cleanup: “Yes, all four stayed
still throughout”.

Audit: one boot; five Armed states with zero wheel commands and five lease
stops; cleanup STOP, BLE disconnect and one release; final Open and AP on.
Every wheel command was zero. No errors, dropped events or panics.
[network.txt](network.txt): laptop Wi-Fi and Ethernet internet route unchanged;
no firewall change. Capture stopped.

This accepts BLE lease detection timing for these five zero-wheel trials on
this instrumented build. It does not measure STBY propagation or physical
wheel stopping time. UDP has separate passing timing evidence; ROS timing,
coast/STBY levels and selected-profile reversal/thermal checks remain pending.
