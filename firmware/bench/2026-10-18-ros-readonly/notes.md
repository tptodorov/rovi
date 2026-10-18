# 2026-10-18: ROS capabilities and Status on the rover

Todor authorized the announced read-only batch with “go”. Requested setup:
motor power disconnected, USB connected. Flashed the ROS product firmware from
`4d8aa82` (PR #73), built for `tcp/192.168.4.2:7447` with locked dependencies.
ELF SHA-256: `e987f9f75f7f7de20da70803266eff0efe7a601e40e920f19cc88b6c7fa2b00c`.
Firmware Rust source is unchanged since `7c2e2df`; this run enables its ROS feature.

The [runner](../ros_readonly.sh) reused stock Jazzy observers before and after one
router restart. [probe.txt](probe.txt) records both passing capability/Status
checks; [car.log](car.log) is the raw flash/boot/serial capture.

| Measurement | Initial connection | After router restart |
| --- | --- | --- |
| Capabilities late receipt | 0.574 s | 0.667 s |
| Capabilities largest observed period | 0.901 s | 0.907 s |
| Capabilities claim/release update | 0.020 / 0.015 s | 0.019 / 0.013 s |
| Status late receipt | 0.766 s | 0.783 s |
| Status largest observed period | 0.908 s | 0.907 s |
| Status claim/release update | 0.011 / 0.023 s | 0.031 / 0.023 s |

Both passes verified reliable/volatile/depth-1 delivery, nine canonical ROS
capability keys matching the UDP payload, four canonical Status keys, retained
shared owner errors, transition updates, and cleared owner/session on release.
The measured receipt/period values were below the one-second fallback bound.
These are observed samples, not a worst-case latency guarantee.

Serial ROS link-up was at 23,102 ms. Restart produced link-down at 41,629 ms and
link-up at 43,712 ms (2.083 s apart). Final link-down at 52,219 ms followed router
cleanup. Zero UDP setpoints and refused ARM commands exercised shared Status;
there was no `/cmd_vel` publication or successful ARM. The complete application
capture has no Armed state, application `ev=error`, panic or dropped-log event.
Final device state was Open with zero commands at 52,159 ms. Bootloader output
includes “Image contains multiple DROM segments”; it is retained in the raw log,
and the application subsequently booted and completed all probes.

## Setup and cleanup

The first Wi-Fi attempt timed out while flashing was still in progress, before
any firewall rule or probe ran. The wrapper restored the prior Wi-Fi connection.
After flash/boot completed, the retry joined the existing `rovi-m8` profile and
added the temporary TCP/7447 rule restricted to the rover's source address and
laptop's Wi-Fi interface/address. Ethernet remained the internet route.

On completion, the runner removed its Docker router. The wrapper removed only
the firewall rule it had added, restored the previous Wi-Fi connection, and
verified its UUID and unchanged Ethernet route. Serial capture was stopped.
The rover remains on this ROS firmware; without its laptop router it will retry.
No credentials or home-network identifiers are included in this evidence.

## Acceptance scope

Actual ESP32 ROS capabilities/Status delivery and recovery after a router restart
pass this read-only batch. It does not establish ROS auto-arm, physical motor
outputs, stop/recovery under motion, ROS owner/router loss while driving, radio
exclusion or numeric stop latency. Task 5.2 remains unchecked.
