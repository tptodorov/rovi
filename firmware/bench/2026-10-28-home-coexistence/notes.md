# Home coexistence observation, 2026-10-28 — 6.3 remains open

Todor said “go on” after merging the router-name/stand work. Announced scope:
15 minutes of observational ROS Echo (20 Hz), stateless UDP Echo (~5 Hz), and
periodic BLE advertisement scans; no claims, ARM or motion commands. The normal
named-router firmware stayed installed, SHA-256
`b52dea02abd9560e848a476d325b1a6831c7685ffdc0cc6ac696e3fca9c28f97`
(the same image as 6.2). Recorder source: `a93ff19`.

The timed observation ran approximately **15:03:55–15:18:59 UTC**
(16:03:55–16:18:59 local), including final recorder snapshots. Each probe's
requested measurement interval was 900 seconds. A BLE Status/Network read
confirmed Home/open before timing; no BLE central connected during timing.
Blackbox's existing router was used throughout, with no router or firewall change.

## Recorded soak

| Measurement | Home observation | 2026-10-20 idle ROS/AP baseline |
| --- | --- | --- |
| UDP Echo | 4,182/4,184 replies; 2 misses (0.048%) | 4,363/4,363; zero misses |
| UDP RTT median | 6.710 ms | 4.901 ms |
| UDP RTT p95, nearest rank | 62.157 ms | 8.659 ms |
| UDP RTT max | 74.872 ms | 46.614 ms |
| ROS session | One link-up; no disconnect | No disconnect |
| BLE advertising | Seen in all 16 scans | Not measured |
| Serial | One boot; no ownership, arming, motion, stop, fault, panic or dropped log event | Same no-motion guards |

[Analysis](soak/analysis.json), [UDP summary](soak/udp/summary.json),
[ROS summary](soak/ros-summary.json), [serial](soak/car.log),
[advertisements](soak/advertising.jsonl.gz),
[identity](soak/identity.json), [cleanup](soak/cleanup.json).

This is not an identical-workload regression comparison: the baseline used the
car AP and an idle ROS session; this run used the home network, a wired router
host, active 20 Hz ROS Echo and BLE scanning. No network rejoin was requested
during the observation. #23's associated-but-silent stall was not reproduced;
two individual UDP misses do not establish that failure or a fix for it.

## ROS collector undercount

The original collector received **16,158/18,000** ROS Echo replies: 1,842 absent
from its output (10.2%). Recorded RTT median/p95/max were
34.973/107.172/309.783 ms. It used best-effort, keep-last **depth 1** for replies.
These are collector results, not proof of 10.2% packet loss on the wire.

A shorter hardware repro kept the same 20 Hz Echo rate and best-effort publisher,
with no concurrent UDP probe or BLE scans. The test asserts all 300 replies:

| Run | Receive depth | Replies | Test |
| --- | --- | --- | --- |
| [Original](probe-original/result.json) | 1 | 275/300 | Red: 25 missing |
| [Buffered variant](probe-buffered/result.json) | 128 | 300/300 | Green |
| [Fixed repository script](probe-fixed/result.json) | 128 | 300/300 | Green |
| [Original again](probe-original-again/result.json) | 1 | 270/300 | Red: 30 missing |

Only subscriber receive depth changed. Buffering replies prevents the observed
depth-1 undercount when samples arrive faster than callbacks consume them.
`apps/simcar/scenarios/ros_latency.py` now receives with depth 128; its publisher,
rate and output format remain unchanged. This fixes the measuring tool, not the
car firmware. It does not prove that every missing reply from the original
15-minute run was delivered, and **no corrected 15-minute repeat is claimed**.

[Executed repro](scripts/rovi-ros-loss-repro.py.txt),
[original probe](scripts/rovi-ros-latency-original.py.txt),
[buffered variant](scripts/rovi-ros-latency-depth128.py.txt).
Example: `python3 /tmp/rovi-ros-loss-repro.py /path/to/ros_latency.py unique-label`.
The helper requires the previously established `/tmp/rovi-61` bench helpers,
normal home firmware, a ROS container image and a new output label; it never
publishes a motion or ownership command. Each diagnostic started fresh serial
capture, intentionally resetting the application outside the timed soak.

## Physical range attempt: unverified

Todor confirmed returning with USB and board power continuous. The
[range capture](range-attempt/car.log) contains no station-loss or fallback
event: the car stayed Home, without a reset after initial capture startup.
The recorder was stopped once Todor returned; its
[KeyboardInterrupt](range-attempt/failure.json) is recorder cancellation, not a
firmware failure. There is no evidence establishing whether coverage was lost.

Todor then explicitly chose **“Leave the range check unverified.”** No
LinkLost fallback, fallback AP beacon or out-of-range recovery is claimed.
Task 6.3 remains unchecked; the home-mode change is not accepted/archived by this
observation. [Range recorder snapshot](scripts/rovi-home-range.py.txt).

## Integrity and cleanup

The [manifest](manifest.json) hashes both private raw inputs and published
derivatives. Actual `ROVI_HOME_PASSWORD` was checked in memory against every
published input; none contained it. SSIDs, MAC addresses and network UUIDs are
redacted in derivatives. CRLF/CR line endings are normalized to LF by the
[publishing script](scripts/rovi-coexistence-publish.py.txt); raw inputs are unchanged. JSONL streams are gzip-compressed without dropping rows;
uncompressed redacted hashes permit verification. Raw captures remain under
`/tmp/rovi-home-soak-2026-10-28`, `/tmp/rovi-home-range-2026-10-28` and
`/tmp/rovi-ros-loss-*`.

The soak's internet route was unchanged, its ROS client was removed and the
serial port was released. The range attempt and all short diagnostics also
released serial. No firmware flash, system configuration, Wi-Fi profile, power
saving or blackbox router policy was changed. Final car state is Home/open,
with the normal image retained. No loaded, floor, thermal, physical stopping or
numeric latency bound is qualified.
