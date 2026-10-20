# 2026-10-20: idle ROS Wi-Fi observation — #23 not reproduced

Todor authorized the [follow-up](../ROS-WIFI-SOAK.md), then asked to continue
the local task. Announced setup: USB connected, motor power disconnected.
Normal serial-monitor startup reset the application once; no firmware flash,
BLE connection, ROS application publisher, claim, ARM or setpoint was sent.
The installed normal ROS image remained unchanged (source `b12cc21`, local ELF
SHA-256 `514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`).
Recorder source: `9baabf4`; [identity](observation/identity.redacted.json).

## Observation

The complete 900-second interval ran approximately **14:29:02–14:44:02 UTC**
(16:29–16:44 local), after a fresh ROS link-up. Snapshots add about four seconds.
[Summary](observation/soak/summary.redacted.json), [analysis](analysis.json):

| Evidence | Result |
| --- | --- |
| Correlated UDP Echo | **4,363 replies / 4,363 attempts; zero loss** |
| Echo RTT | min 4.056 ms, median 4.901 ms, p95 8.659 ms (nearest rank), max 46.614 ms |
| ROS session | One link-up; no link-down during observation |
| ICMP | 3/3 replies at each snapshot: 9/9 total |
| Association | `Rovi M8`, 2412 MHz, −40 dBm at all three snapshots |
| Laptop power saving | `off` at all three snapshots; policy unchanged |
| Serial | One boot; no ownership, Armed/nonzero, stop, fault, dropped event or panic |
| Fresh Echo after router shutdown | Replied in 4.301 ms; no recovery rejoin needed |

[Start](observation/soak/start.redacted.json),
[initial-up](observation/soak/transition-001.redacted.json) and
[end](observation/soak/end.redacted.json) snapshots retain successful `iw`,
NetworkManager, route and ping queries. The [service journal](observation/controller-journal.redacted.json)
shows AP association and deliberate final restoration, with no intervening
reconnect. No NetworkManager service entries appeared; wpa_supplicant supplied
the entries. The [serial capture](observation/car.redacted.log) includes the
existing bootloader multiple-DROM-segments diagnostic, without a product fault.
Its final ROS link-down follows the runner's intentional router shutdown,
outside the timed interval; [interval link audit](observation/soak/ros-link.redacted.json).

[Cleanup](observation/cleanup.redacted.json) confirms capture stopped, the
temporary firewall rule removed, previous Wi-Fi restored and internet route
unchanged. The router was removed. The [executed controller](observation/controller.py.redacted.txt)
records setup, live guards and cleanup.

## Aborted setups

Neither attempt below reached the timed observer or sent motor commands.

- **14:20 UTC:** the controller appended the temporary firewall rule after
  NixOS's terminal reject, so ROS link-up timed out. Its subsequent missing-summary
  error masked that timeout in the cleanup failure field; the
  [runner transcript](setup-firewall/soak.redacted.txt) preserves the initial
  failure. [Cleanup](setup-firewall/cleanup.redacted.json) restored prior state.
  The corrected controller inserts the rule first and checks its ordering.
- **14:25 UTC:** `espflash monitor --no-reset` connected through the flash stub
  and left the application stopped. AP joining timed out before the router or
  observer started. [Capture](setup-monitor/car.redacted.log) and
  [cleanup](setup-monitor/cleanup.redacted.json) preserve this setup failure.
  Normal monitor startup resumed the application for the completed run.

These orchestration errors are not reproductions of #23's associated-but-silent
Wi-Fi stall. All three attempts restored their scoped network/capture state.

## Integrity and limits

The [full Echo stream](observation/soak/echo.jsonl.gz) is losslessly compressed.
Analysis verifies contiguous sequences, matching counts and its uncompressed
SHA-256. [Manifest](manifest.json) records source and published file hashes.
All other captures are explicitly named **redacted derivatives**: MACs, unrelated
IP addresses, prior Wi-Fi SSID/UUID, laptop hostname, home path and router ID
are replaced where present; JSON is reformatted. Timestamps and benchmark
measurements are retained. Unchanged raw files remain under the three private
`/tmp/rovi-ros-wifi-soak-2026-10-20*` directories; none was edited.

**#23 remains open.** This observation adds normal ROS discovery/publication/
keepalive traffic over the current **TCP** transport. It does not replay the
October 13 UDP-router workload or establish a fix. Active probes affect idle
traffic, snapshots pause Echo probing, and conditional recovery was not exercised.
No motor acceptance or electrical measurement is claimed.
