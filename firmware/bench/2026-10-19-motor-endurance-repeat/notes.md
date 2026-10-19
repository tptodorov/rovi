# 2026-10-19: five-minute stand endurance — serial checks passed

Todor separately authorized this repeat with “go”: two flashes, five minutes
forward at the configured 70% cap, STOP/release and normal-image restoration.
Rover restrained on stand, wheels clear, motor bank awake and USB connected.
[Procedure](../MOTOR-ENDURANCE.md). The [earlier attempt](../2026-10-19-motor-endurance/notes.md)
remains an abort at the unarmed announcement gate, with no probe launched.

## Identity and observed commands

Probe/controller source: `0cec6e707a45099658610f2d7a3980df5c219fa8` (PR #86).
Bench firmware source `5cee76b`, features `ros,motor-bench`, ELF SHA-256:
`be93c50c4c07deb0060c98a1c0626e957f6e6db2364b0617f7438c1969213073`.
Restored normal ROS source `b12cc21`, features `ros`, ELF SHA-256:
`514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`.
Both hashes were checked before flashing and again before restoration.
[identity.json](identity.json) is the initial preflight snapshot; its
`flashed=false` describes that initial state. Both actual flashes completed
([bench](flash-bench.txt), [normal restoration](flash-restore.txt)).

The unarmed gate was opened after the motion announcement. After the
five-second lead-in, one UDP owner explicitly armed at zero. The fixed
forward marker requested `[1.0; 4]` through the product motor module at its
700-permille drive cap. The 300-second laptop clock started after fresh
full-scale serial evidence; the independent device interval was 300,093 ms.
No clock domains were subtracted from each other.

| Event | Serial at_ms | All four wheel demands (permille) |
| --- | ---: | --- |
| Armed at zero | 16253 | 0,0,0,0 |
| Full forward | 17268 | 1000,1000,1000,1000 |
| Zero after hold | 317361 | 0,0,0,0 |
| STOP | 318365 | zero |
| Released | 318371 | zero |
| Open | 318379 | 0,0,0,0 |

Repeated HELLO ACKs preserved the same UDP session throughout. The serial
stream had no reversal, intervening zero/stop, product fault, dropped event,
panic or unplanned reboot. [probe.txt](probe.txt) reports `rc=0`, progress at
30-second intervals and fresh cleanup release. The independent audit in
[batch.txt](batch.txt) passed. These are requested commands, not measured
wheel speed, electrical duty, current or temperature.

## Physical and heating observations

Todor reported: “test run was fine, no temperature increase in the drivers
at all.” This is retained as a qualitative user observation; no numeric
temperature measurement or instrument was available.

**Pending explicit confirmation:** continuous motion of all four wheels,
final stop and still wheels through both flashes/boots. Whether the warming
observation also covers the motors is pending. Do not infer motor temperature
or mark thermal acceptance from the serial result or driver-only report.

## Cleanup and limits

The exact normal ROS image was restored. Its [capture](normal-restored.log)
has one application boot, no bench banner, and no claim/arming during a
five-second idle watch. The [bench capture](car.log) has one boot, one claim,
one sustained full-scale state, STOP/release and final Open. The bootloader's
existing multiple-DROM-segments diagnostic remains as in prior image runs.
No product fault, panic or dropped event occurred in either capture.

[network.txt](network.txt) verifies prior Wi-Fi restoration and the unchanged
Ethernet internet route. No router or firewall rule was used. Both captures
are stopped; normal instrumented ROS firmware is installed. Raw
[result.json](result.json) was written before physical confirmation and is
retained unchanged. Coast/STBY measurements and thermal acceptance remain
open; no loaded/stall, current, temperature-bound or thermal-margin claim.
