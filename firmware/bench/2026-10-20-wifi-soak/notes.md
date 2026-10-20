# 2026-10-20: Wi-Fi Echo baseline — #23 not reproduced

Todor authorized the [announced observation](../WIFI-SOAK.md) with “go”.
Announced setup: ESP32 USB connected, motor power disconnected. One reset opened
serial capture; no flash, BLE connection, router, firewall rule or motor command.
Recorder source: `5a6952fe54669110604a78677299c4058b9c751d` ([identity](identity.json)).
The previously restored normal ROS image remained installed, source `b12cc21`,
local ELF SHA-256
`514ba816d3b6b8b3b2c40ee63e156fd4f07ca9be104cef8246d6e848602c73f3`.

## Observation

The complete 900-second probe interval ran from approximately **11:19:35 to
11:34:35 UTC** (13:19–13:34 local), after joining the AP. Initial/final snapshots
add about four seconds. [Probe summary](soak/summary.json), [transcript](probe.txt)
and [derived analysis](analysis.json) record:

| Evidence | Result |
| --- | --- |
| Correlated UDP Echo | **4,385 replies / 4,385 attempts; zero loss** |
| Echo RTT | min 3.246 ms, median 4.269 ms, p95 5.900 ms (nearest rank), max 29.389 ms |
| ICMP | 3/3 replies at each of start, initial-up and end: 9/9 total |
| NetworkManager snapshots | Connected to `rovi-m8`, IPv4 `192.168.4.2/24`; target route through `wlp1s0f0` at each snapshot |
| Service journal | No entries during the probe-only query; controller query shows AP join and deliberate final Wi-Fi restoration, with no logged reconnect between them |
| Serial audit | One boot event only: no claim, release, stop, Armed/nonzero state, product fault, dropped event or panic |
| Final fresh Echo | Replied in 3.652 ms; conditional recovery rejoin was unnecessary |

The recorder sent stateless Echo with session zero; it never claimed or armed.
The [serial capture](car.log) retains the existing bootloader multiple-DROM-
segments diagnostic seen on prior normal-image runs; no new product fault event
was logged. [Cleanup](cleanup.json) confirms capture stopped, the previous Wi-Fi
UUID restored and the internet route unchanged. The [controller transcript](controller.txt)
and [executed controller](controller.py.txt) preserve the setup and cleanup.

## Evidence integrity and coverage

The full per-probe stream is preserved losslessly as [echo.jsonl.gz](soak/echo.jsonl.gz).
`analysis.json` records the uncompressed SHA-256 and RTT statistics. Validation
confirmed contiguous sequences and matching counts. [Start](soak/start.json),
[initial-up](soak/transition-001.json), [end](soak/end.json),
[probe journal](soak/journal.json) and [final Echo](after-soak-echo.json) are
unchanged tool outputs.

`iw` was unavailable: both association and power-save commands recorded
`FileNotFoundError` at every snapshot. Effective wireless power saving therefore
remains unknown. The service queries succeeded, but only wpa_supplicant emitted
entries in the full controller interval; no NetworkManager entries appeared.
The [published controller journal](journal-excerpt.md) is an explicitly redacted
derivative: laptop hostname, MACs and prior Wi-Fi SSID replaced; all 18 lines
and timestamps retained. Its unchanged raw source stays in the private local
`/tmp/rovi-wifi-soak-2026-10-20/` directory. No raw capture was edited.

## Disposition

**#23 remains open.** This no-ROS Echo baseline did not reproduce the sustained
stall; it does not establish a fix. The original ROS keepalive workload was not
replayed, active probes may affect inactivity behavior, and snapshots pause
probing. No recovery rejoin or ten-second recovery observation was exercised.
No motor/physical-motion acceptance or new electrical measurement is claimed.
