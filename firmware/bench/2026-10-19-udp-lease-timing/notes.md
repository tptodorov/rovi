# 2026-10-19: UDP lease timing at zero wheels

Todor authorized the announced five-trial batch with “go”. Rover on stand,
wheels clear, USB connected and motor bank awake. Installed ROS product
firmware from the [read-only batch](../2026-10-18-ros-readonly/notes.md);
no flash. Probe source `b6571ef` (PR #80), following the
[procedure](../UDP-LEASE-TIMING.md).

Each trial accepted one zero setpoint, armed at zero wheels and sent no more
commands until the lease expired. The laptop measured from immediately before
sending that setpoint to receipt of the fresh serial `LeaseExpired` event,
using its monotonic clock. Successful ARM proved acceptance of the sole
setpoint. These conservative upper bounds include UDP delivery, serial
capture, polling and laptop scheduling; ESP32 and laptop clocks were not
mixed. The advertised lease was 1,000 ms; every trial had to be at most
1,050 ms. Every trial also received the unsolicited lease-expired UDP Status
and observed cleanup release.

| Trial | Observed upper bound (ms) | Limit (ms) | Result |
| --- | ---: | ---: | --- |
| 1 | 1,011.059 | 1,050 | Pass |
| 2 | 1,015.314 | 1,050 | Pass |
| 3 | 1,013.882 | 1,050 | Pass |
| 4 | 1,009.014 | 1,050 | Pass |
| 5 | 1,006.483 | 1,050 | Pass |

Evidence: [probe.txt](probe.txt), [car.log](car.log),
[network.txt](network.txt). Log audit: one boot, five zero-wheel Armed states,
five lease stops, five releases, final state Open. Every wheel command was
zero; no errors, dropped events or panics. Capture stopped, previous Wi-Fi
restored and Ethernet internet routing unchanged. No firewall change.

**Physical observation:** pending Todor's confirmation that all four wheels
stayed still throughout. UDP lease-detection timing passed on the board;
BLE/ROS timing remains pending. This does not measure STBY propagation or
physical wheel stopping time.
