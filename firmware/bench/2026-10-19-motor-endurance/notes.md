# 2026-10-19: endurance attempt aborted before probe launch

Todor authorized the announced two flashes and five-minute forward endurance
batch with “go”. Rover restrained on stand, wheels clear, motor bank awake,
USB connected; visual observation only. [Procedure](../MOTOR-ENDURANCE.md).

**Aborted before drive. No UDP claim, ARM, setpoint or endurance probe was
launched. No endurance or thermal acceptance is claimed. No retry occurred.**

## Identity and failure

Probe preparation source: `fb44708a740ef2b364202946155759cf987e9599` (PR #86).
Bench firmware source `5cee76b`, features `ros,motor-bench`, ELF SHA-256:
`be93c50c4c07deb0060c98a1c0626e957f6e6db2364b0617f7438c1969213073`.
Restored normal ROS source `b12cc21`, features `ros`, ELF SHA-256:
`514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`.
Both hashes were checked before flashing and before restoration.
[identity.json](identity.json) is the initial preflight snapshot; its
`flashed=false` describes that initial state, not the final outcome.

The [bench flash](flash-bench.txt) completed and the [capture](car.log) showed
its bench banner and one application boot. The laptop joined Rovi's AP. The
controller then waited for the assistant's motion-announcement gate. Its
120-second unarmed waiting window expired before the gate was opened, so the
controller aborted and restored normal firmware automatically. The assistant's
subsequent gate-open command correctly refused the already-finished state.
This was an orchestration timeout; the motor probe was never started and
there is no `probe.txt`. [batch.txt](batch.txt) preserves the traceback and
restoration sequence; [result.json](result.json) retains the failed verdict.

The next controller attempt permits up to ten minutes at the **unarmed**
announcement gate, with the same abort-and-restore behavior on expiry. The
five-minute motor hold, one-second command lease, 330-second outer probe
limit, image checks and STOP/release paths are unchanged. A new announced go
is required; the gate does not infer consent from elapsed time.

## Cleanup and physical observation

[Normal-image restoration](flash-restore.txt) completed using the exact saved
ELF. The [restored capture](normal-restored.log) has one application boot, no
bench banner and no claim/arming during its five-second idle watch. Both
captures contain only the product boot event: no claim, arming, stop, release,
product fault, dropped event or panic. The bootloader's existing multiple-DROM
segments diagnostic remains, as in prior normal-image runs.

**Todor confirmed “yes”: all four wheels stayed completely still through both
flashes, boots and cleanup.** The raw result was written before that physical
reply and is retained unchanged.

[network.txt](network.txt) verifies prior Wi-Fi restoration and an unchanged
Ethernet internet route. No router or firewall rule was used. Both captures
are stopped; the rover is on normal instrumented ROS firmware. Endurance is
still untested. Thermal acceptance and coast/STBY measurements remain open.
