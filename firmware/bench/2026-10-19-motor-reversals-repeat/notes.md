# 2026-10-19: ten full-scale motor reversals — passed

Todor separately authorized this repeat with “go”, including the bench flash,
ten direct reversals at the 70% product cap and normal-image restoration.
Rover restrained on stand, wheels clear, motor bank awake and USB connected.
[Procedure](../MOTOR-REVERSALS.md). The [first attempt](../2026-10-19-motor-reversals/notes.md)
remains recorded as an abort at zero; it did not perform any reversal.

## Identity and commands

Corrected probe source: `205e17e1adb35a451e63e9f6dbd8a9104d1c826d` (PR #85).
The checker fix changed no firmware or motor policy. Both image hashes were
checked before flashing and before restoration:

| Image | Source / features | ELF SHA-256 |
| --- | --- | --- |
| Bench | `5cee76b`, `ros,motor-bench` | `be93c50c4c07deb0060c98a1c0626e957f6e6db2364b0617f7438c1969213073` |
| Normal ROS, restored | `b12cc21`, `ros` | `514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3` |

[identity.json](identity.json) is the preflight snapshot (`flashed=false` is
its initial state); [bench flash](flash-bench.txt) and [normal restoration](flash-restore.txt)
completed successfully. Opening each capture caused an authorized reset.

After the five-second lead-in, one UDP session armed at zero for about a
second. The fixed ±0.1 markers then requested ±1.0 on all four motor-module
channels, using the unchanged product profile (700-permille drive cap).
These are requested commands, not measured wheel speed or electrical duty.

| Hold | All four wheel demands (permille) | Serial at_ms |
| --- | ---: | ---: |
| 1 | +1000 | 20259 |
| 2 | −1000 | 21287 |
| 3 | +1000 | 22327 |
| 4 | −1000 | 23340 |
| 5 | +1000 | 24356 |
| 6 | −1000 | 25376 |
| 7 | +1000 | 26413 |
| 8 | −1000 | 27457 |
| 9 | +1000 | 28486 |
| 10 | −1000 | 29502 |
| 11 | +1000 | 30527 |

All ten sign changes were consecutive: no intervening zero or stopped state.
Zero followed at 31,607 ms, STOP at 32,620 ms, release at 32,626 ms and Open
at 32,631 ms. Repeated HELLO ACKs retained the same UDP owner session throughout
all holds. [probe.txt](probe.txt) reports success, `rc=0` and cleanup release.

## Observation and cleanup

**Todor confirmed “yes”: all four wheels followed the forward/reverse sequence
through all ten reversals, stopped at the end, and stayed still through both
flashes and boots.** This physical reversal subset passes on the recorded
bench image and stand setup; thermal qualification is separate.

The [bench capture](car.log) contains one boot, one UDP claim, the eleven
alternating full-scale states, STOP/release and final Open. No product fault,
dropped event, panic or unplanned reboot was logged. [batch.txt](batch.txt)
records the independent serial audit and completed restoration.

The exact normal ROS image was restored. Its [capture](normal-restored.log)
contains one application boot, no bench banner and no claim/arming during a
five-second idle watch. Both captures retain the bootloader's existing
multiple-DROM-segments diagnostic, also present in the earlier normal-image
[ROS timing capture](../2026-10-19-ros-device-lease-timing/car.log); it was not investigated here.
[network.txt](network.txt) confirms prior Wi-Fi restoration and an unchanged
Ethernet internet route. No router or firewall rule was used. Both captures
are stopped. The rover is back on normal instrumented ROS firmware.

Raw [result.json](result.json) was written before Todor's physical reply;
its software verdict is passed and the subsequent observation is recorded
above. Coast/STBY pin levels and selected-profile thermal qualification remain
pending; task 5.2 stays open. Loaded/stall operation was not tested.
