# 2026-10-19: reversal batch aborted at zero speed

Todor authorized the announced bench-image flash, ten all-wheel reversals at
70% and normal-image restoration with “go”. Rover on stand, wheels clear,
motor bank awake and USB connected. [Procedure](../MOTOR-REVERSALS.md).

**Result: aborted before any full-duty command. Reversal acceptance remains
pending. No retry was performed.**

## Image identity and sequence

Probe/bench source: `5cee76b9fe1a47ab4ed964439d0c45e7c24463be` (PR #85).
Both ELF hashes were checked before flashing, and again before restoration:

| Image | Features/source | ELF SHA-256 |
| --- | --- | --- |
| Bench | `ros,motor-bench`, `5cee76b` | `be93c50c4c07deb0060c98a1c0626e957f6e6db2364b0617f7438c1969213073` |
| Normal ROS, restored | `ros`, recorded source `b12cc21` | `514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3` |

[identity.json](identity.json) is the start-of-batch preflight snapshot; its
`flashed=false` describes that initial state. Both actual flashes completed
successfully ([bench](flash-bench.txt), [normal restoration](flash-restore.txt)).
Opening each serial capture caused the separately authorized reset.

The bench banner and clean boot were observed before joining the AP. After
the announced five-second lead-in, UDP claimed and explicitly armed at zero.
The probe then failed during its initial zero-speed hold. It never reached
the forward/reverse loop. [probe.txt](probe.txt) has `rc=1` and the assertion;
[batch.txt](batch.txt) records the abort and unconditional restoration.

## Cause and correction

The real [serial trace](car.log) is valid:

```text
ev=state state=Claimed wheels_pm=0,0,0,0 at_ms=16509
ev=state state=Armed wheels_pm=0,0,0,0 at_ms=16521
```

Each poll rereads this same prefix. The checker retained its “Armed seen”
flag from the previous poll, then mistook the earlier Claimed entry for a
new Claimed-after-Armed transition. This is a host checker bug, not evidence
of a controller reset or lost session.

Before changing the checker, a deterministic three-poll replay of this
minimal board trace reproduced the same assertion. The fix recomputes prefix
ordering on each scan. That replay now passes, while a genuinely later
Claimed-after-Armed entry is still rejected. The full socket batch and its
existing wrong-demand, normal-image and dropped-cleanup-BYE checks also pass.
The fix changes no firmware or motor policy. A fresh announced go is needed
before any further physical batch.

## Cleanup and physical observation

The bench [serial log](car.log) contains one boot, one UDP claim, one zero-wheel
Armed state, STOP/release at 16,640 ms and Open/zero at 16,644 ms. Every wheel
command was zero; no product fault, dropped event or panic was logged. No
nonzero stage or unplanned reboot occurred in the captured session.

The normal ROS image was restored to the exact recorded ELF. Its
[restored capture](normal-restored.log) has one clean application boot, no
bench banner, and no claim/arming during the subsequent five-second watch.
Both captures retain the bootloader's multiple-DROM-segments diagnostic,
also present in the preceding [ROS timing capture](../2026-10-19-ros-device-lease-timing/car.log);
this attempt did not introduce or investigate it.

**Todor confirmed “yes”: all four wheels stayed completely still through both
flashes/boots, the initial zero-speed hold and cleanup.** The raw
[result.json](result.json) was written before that observation was supplied;
its failed software verdict is retained.

[network.txt](network.txt) records restoration to the previous Wi-Fi UUID,
with the Ethernet internet route unchanged. No router or firewall rule was
used. Both serial captures are stopped. The rover is on normal instrumented
ROS firmware, with the bench feature absent.

No full-scale reversal or thermal acceptance is claimed. Coast/STBY pin
measurements still need a meter/probe, currently unavailable.
