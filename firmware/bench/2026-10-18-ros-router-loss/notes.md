# 2026-10-18: ROS router-loss batch — physically confirmed

Todor authorized the announced two-scenario batch with “go”. Setup as in
[ROS-ROUTER-LOSS.md](../ROS-ROUTER-LOSS.md): rover on stand, wheels clear, USB
connected, motor bank awake. No flash: installed ROS firmware from the
[read-only batch](../2026-10-18-ros-readonly/notes.md). Runner source `5f87f2e` (PR #75).
Opening the serial monitor reset the board (power-on boot at 1,006 ms).

[probe.txt](probe.txt) is the runner output; [car.log](car.log) is raw serial.
Both drives commanded `83,83,83,83` wheel permille at `vx=0.1`.

| Scenario | Serial evidence (car ms) | Physical result |
| --- | --- | --- |
| KILLED | Claimed/Armed 19,104/19,110. Router killed after 3 s: `link=down`, `stop reason=OwnerLost`, `released` all at 22,189; Open/zero at 22,194. No lease stop. New router linked at 24,271; no claim/arm for 3 s; `/rovi/status` open. | Passed: all four stopped and stayed still |
| FROZEN | Claimed/Armed 34,697/34,699. Router paused after 3 s; `LeaseExpired` at 38,704 (Stopped/zero). Stopped-owner window released at 43,708; car's own `link=down` at 47,697 (about 10 s router lease after the freeze). No OwnerLost. Paused router killed; new router linked at 49,779; no claim/arm for 3 s; `/rovi/status` open. | Passed: all four stopped and stayed still |

No errors, dropped events or panics. The final `link=down` at 54,386 is the
runner removing its router. The temporary firewall rule was removed, the prior
Wi-Fi connection restored and the Ethernet internet route unchanged. Serial
capture stopped. The rover remains on ROS firmware.

On 2026-10-18 Todor answered “yes” to whether all four wheels drove forward,
stopped at once when the router was killed, stopped about 1 s after the freeze
and stayed still afterwards. Both physical ROS router-loss scenarios pass at the
tested gentle setting.
