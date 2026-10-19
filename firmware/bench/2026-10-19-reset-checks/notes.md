# 2026-10-19: boot, reset and power loss with motor power present

Todor authorized the announced batch with “go”. Rover on stand, wheels clear,
motor bank (BT1) connected and awake. Installed ROS product firmware from the
[read-only batch](../2026-10-18-ros-readonly/notes.md); no flash. Probe source
`5e443dd` (PR #79). [probe.txt](probe.txt) is its output; each `*.log` is one
serial capture, opened with a hard reset so it starts at boot.

| Step | Serial evidence | Physical result |
| --- | --- | --- |
| Idle reset 1 | [idle-reset-1.log](idle-reset-1.log): boot, then 5 s with no claim, stop, fault or nonzero wheel command | Pending Todor's confirmation |
| Idle reset 2 | [idle-reset-2.log](idle-reset-2.log): same | Pending Todor's confirmation |
| Reset while driving | [drive.log](drive.log): UDP claim, Armed `83,83,83,83` at 9,759 ms; the capture was closed and the board reset while the UDP stream continued. [reset-while-driving.log](reset-while-driving.log): boot, then 5 s Open with zero wheels despite the old session's continuing setpoints | Pending Todor's confirmation |
| ESP32 power loss | Todor unplugged the ESP32 USB (its only supply; the motor bank stayed on) and reconnected it. [power-restored.log](power-restored.log): boot, then 5 s Open with zero wheels. The power-on boot itself precedes this capture, whose opening resets the board again | Pending Todor's confirmation |

No errors, drops or panics in any capture. Logs cannot show the wheel state
while the chip is in reset or unpowered; that is the physical observation.
Prior Wi-Fi restored, Ethernet internet route unchanged, capture stopped.
