# 2026-10-19: UDP/ROS shared-AP exclusion on the board — physically confirmed

Todor authorized the announced batch with “go”. Rover on stand, wheels clear,
motor bank awake. Installed ROS product firmware from the
[read-only batch](../2026-10-18-ros-readonly/notes.md); no flash. Probe/runner
source `d7f7ce7` (PR #78), run as `ros_stops.sh ros_udp_sharing.py`.
[probe.txt](probe.txt) is the output; [car.log](car.log) is raw serial.

| Check | Evidence (car ms) | Physical result |
| --- | --- | --- |
| UDP owner, ROS intruder | UDP claim 10,176, Armed at zero 10,198. UDP silent while ROS streamed `vx=0.1` for 2 s: no ROS claim, no nonzero wheel command, no `wifi ap=off`. LeaseExpired 11,247 (1,049 ms after arming, so ROS did not renew it). BYE released 12,757. | Passed: all four wheels stayed still |
| ROS owner, UDP intruder | ROS claim 12,813, auto-armed at zero 12,818. UDP HELLO got BUSY; its `vx=0.1` setpoints and ARM were ignored: no UDP claim, no nonzero wheel command. LeaseExpired 13,821 (1,003 ms). ROS STOP 14,884; quiet owner released 18,829. | Passed: all four wheels stayed still |

Every `wheels_pm` in the log is `0,0,0,0`. No errors, drops or panics. The
final `link=down` (18,904) is the runner removing its router. Firewall rule
removed, prior Wi-Fi restored, Ethernet internet route unchanged, capture stopped.

On 2026-10-19 Todor confirmed all four wheels stayed completely still throughout
(“yes, confirmed”).
