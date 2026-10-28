# Home coexistence: corrected 15-minute collector

Todor authorized this no-motion repeat with “do it” on 2026-10-28. The timed interval began at 17:16:38 UTC (18:16:38 CET), lasted 900 seconds, and used the normal named-router firmware already flashed. No ARM, nonzero setpoint, mode command or ownership claim was sent. Initial BLE reads preceded timing; only advertising scans ran during timing.

## Results

| Observation | Result |
| --- | --- |
| ROS Echo, 20 Hz, reply depth 128 | **18,000 / 18,000**, zero loss, duplicates or foreign replies |
| ROS RTT, ms | min 7.751; median 42.864; p95 144.371; max 315.248 |
| UDP Echo | **4,026 / 4,035**, 9 missing replies (0.223%) |
| UDP RTT, ms | min 4.446; median 6.687; p95 62.682; max 78.797 |
| BLE advertising | Detected in **16 / 16** scans |
| Serial | One boot, one ROS link-up, no reset, disconnect, ownership, arming or fault event |
| ROS observations | Status 1,000; Network 887; Capabilities 1,000; largest gaps 1.111 / 1.228 / 1.127 s |

The laptop UDP observer recorded 18 link-state transitions; these are not car Wi-Fi-loss events. The car serial stayed in Home with its ROS connection. UDP loss remains an observed limitation; this is not a zero-loss result for every transport.

The [original depth-1 run](../2026-10-28-home-coexistence/notes.md) and its A/B/A diagnosis remain separate evidence. This repeat uses the corrected receive depth from PR #118; firmware and workload were unchanged. The [2026-10-20 baseline](../2026-10-20-ros-wifi-soak/notes.md) had 4,363/4,363 UDP replies, median 4.901 ms, p95 8.659 ms and max 46.614 ms. Its car-AP topology and idle ROS session differ from this home-network run with active ROS Echo, so this is descriptive comparison, not a controlled regression measurement.

## Setup and cleanup

[Identity](identity.json) records source `19e9679`, firmware SHA-256, authorization and the initial route. The car connected through `tcp/blackbox.local:7447`; the router ran on blackbox. No firmware flash, network profile, power policy, firewall or route change was made. Lightweight source edits and Node tests ran on the laptop; the full Chromium suite waited until after timing. [Cleanup](cleanup.json) records the unchanged default route, active router and remaining car session. The temporary ROS client was removed and USB serial released.

The retained controller and worker snapshots describe the exact workload. [Analysis](analysis.json) recomputes counts, nearest-rank p95 and serial observations from the capture. JSONL captures are gzip compressed. [Manifest](manifest.json) records raw, normalized and published SHA-256 hashes; SSID, hardware addresses and UUIDs are redacted, with line endings normalized. The home passphrase occurs in none of the published artifacts.

**6.3 remains open:** the physical range fallback check is unverified, as Todor explicitly requested. This repeat establishes the corrected soak result only; it adds no motion or range acceptance.
