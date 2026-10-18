# 2026-10-18: physical wheel directions

Todor authorized the announced six-move batch with “go”. Rover on stand, wheels
clear, USB connected, motor bank awake. No flash: installed ROS product firmware
from the [read-only batch](../2026-10-18-ros-readonly/notes.md), driven over UDP.
Probe source `9ef7ac4` (PR #76). Opening the serial monitor reset the board.

[probe.txt](probe.txt) is the probe output; [car.log](car.log) is raw serial.
One UDP session (claimed 14,690 ms), each move 3 s then 3 s zero; STOP and
release at 50,825/50,830 ms, final Open/zero. No errors, drops or panics.

| Move | Serial `wheels_pm` FL,FR,RR,RL | Physical result |
| --- | --- | --- |
| forward | 83,83,83,83 | Pending Todor's confirmation |
| backward | -83,-83,-83,-83 | Pending Todor's confirmation |
| strafe left | -83,83,-83,83 | Pending Todor's confirmation |
| strafe right | 83,-83,83,-83 | Pending Todor's confirmation |
| rotate left (CCW) | -85,85,85,-85 | Pending Todor's confirmation |
| rotate right (CW) | 85,-85,-85,85 | Pending Todor's confirmation |

Every pause logged `0,0,0,0` while armed. Prior Wi-Fi restored, Ethernet internet
route unchanged, capture stopped.
