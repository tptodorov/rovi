# 2026-10-26: home network mode, bare board (task 6.1)

Todor authorized the announced 6.1 batch with “go”. Bare board on USB, motor power off, no ARM sent in any phase. Images built from `feat/network-mode-firmware` with `--features ros` ([build](build.py.txt): AP password from NetworkManager, home settings from gopass `env/rovi/home`, never printed). Serial captured from a fresh boot per phase ([helpers](bench.py.txt)); the home SSID is redacted to `<home-ssid>` in the logs. The home passphrase appears in **no** log (checked by script over every log).

## Results

Times are firmware `at_ms`; step durations are `took_ms` from `ev=net_step`. Budgets: scan ≤ 1500 ms, join + DHCP + router ≤ 5000 ms after the network is found, car AP up ≤ 2500 ms after the scan starts when the network is missing.

| Phase | Image / condition | Outcome | Log |
| --- | --- | --- | --- |
| A | No home settings | `net mode=Fallback reason=NoSettings` 6 ms after boot; `Rovi M8` visible from the laptop | [a](a-nosettings.log) |
| B (first) | Home settings, router on laptop | **Fail:** scan timed out at 1501 ms though the network was in range; reset into fallback (`reason=Scan`) worked | [b](b-home.log) (start) |
| diag | 8 s scan budget (uncommitted) | Scan **2686 ms**, join 651, DHCP 933; router step failed (laptop firewall, below) | [diag](diag-scan.log) |
| diag | + BLE quiet during the attempt | Scan **160 ms**, join 2795, DHCP 859, router 805 → `mode=Home` | [diag-bleoff](diag-scan-bleoff.log) |
| B | Fixed image (below), real budgets | Scan 172, join 2791, DHCP 895, router 804 ms → `mode=Home` at 5091 ms; join + DHCP + router **4490 ms of 5000** | [b](b-home.log) |
| B2 | BLE watcher, UDP `INFO`, ROS graph, BLE takeover | Watcher read Status `open` and `mode=home ip=192.168.1.193` without claiming; UDP `INFO` to that address from a non-owner; `/rovi` in the home router's graph; BLE STOP claimed (`claimed source=Ble`, stopped), UDP unanswered while BLE owned; after release the car rejoined (scan 386, join 59, DHCP 886, router 0 ms) | [b](b-home.log), [script](b2.py.txt) |
| B3 | UDP `MODE` Car network, then BYE | `mode_cmd Udp CarNetwork Ok`, released, reset (`reset=Sw`), `mode=Fallback reason=Requested` 1024 ms after boot; about 1.25 s from release to the car AP | [b](b-home.log) |
| B4 | UDP `MODE` Home from the car AP, then BYE | `mode_cmd Udp Home Ok`, reset into a home attempt; router step cut at the 5 s budget (`reason=Router`) while the laptop was still rejoining the home Wi-Fi; mechanism passes, home not reached in this run | [b](b-home.log) |
| C | Router stopped | Router step cut at the budget (5617 ms), `reason=Router`, car AP after a 1 s reset | [c](c-norouter.log) |
| D | SSID that does not exist | Scan failed in 161 ms, `reason=Scan`; car AP about **1.4 s** after the scan started (budget 2.5 s) | [d](d-nossid.log) |
| E | Wrong passphrase | Join timed out at 5000 ms, `reason=Join` | [e](e-wrongpass.log) |
| End | Home image, router, host route (below) | Join 2790, DHCP 833, router 826 ms → `mode=Home`, `/rovi` in the graph | [end-3](end-home-3.log) |

## Defects found and fixed (test-first where host-testable)

1. **BLE advertising during a home attempt stretched the Wi-Fi scan from 160 ms to 2.7 s** (coexistence), past the scan budget. The core now keeps BLE quiet while an attempt runs (`Device::radio_allowed`, test `ble_is_quiet_while_a_home_attempt_runs`); the spec only requires BLE in home mode. Sim and conformance suites rerun green.
2. **A step outlived the core's budget:** the core ended the attempt at 8117 ms, the firmware noticed only at its own step timeout (9701 ms). Each step now also ends when the core leaves it; the board then cut router steps exactly at the budget (phases B4, C).
3. The scan's strongest access point (BSSID, channel) is pinned for the join. It did **not** shorten the join (0.66–2.8 s either way); it is kept so the join uses the access point the scan saw.

## Open findings (not fixed here)

* **Join budget is tight.** Join takes 0.66–2.8 s at boot; with DHCP and router, 4.45–4.5 s of the 5 s budget on a good run, and no room when the router is slow to answer. Raising the join budget is Todor's decision (budgets were set in the design conversation).
* **Laptop as router host:** the NixOS firewall drops TCP 7447 from the LAN, and the laptop is on 192.168.1.0/24 twice (USB Ethernet 192.168.1.148 and Wi-Fi 192.168.1.244), so after a reconnect replies to the car left by the wired interface. For this run a temporary rule (TCP 7447 from 192.168.1.0/24 on Wi-Fi) and a host route to the car were added and **removed** afterwards. A permanent setup needs a router host reachable on the address built into the image, and a declarative firewall rule.
* After a BLE takeover the router step can pass in 0 ms because the previous ROS session survived the short station outage (`LINK_UP` still set); if that session is in fact dead, the normal ROS reconnect recovers it.

## Scope

Accepts 6.1 for boot-time selection, scan/join/router failures, the reset-based switch both ways, watching, UDP `INFO`, BLE takeover at home and mode commands over UDP, on this home network. Mode commands over BLE and ROS, ROS takeover and hand-back with wheels (6.2) and coexistence under load (6.3) are not covered.
