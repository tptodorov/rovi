# Home network mode

**Status:** draft for review (2026-10-22). Behaviour decided by Todor in conversation on 2026-10-22; no implementation, flash or motor run is authorized yet.

## Why

Rovi can only be driven from a device on its own access point (AP) or over BLE, so a ROS 2 controller must run on a laptop that has given up its normal Wi-Fi. Todor wants a controller running as a service on the home network to drive the car around the house and take part in the other ROS 2 activity there, while he can still take over by hand over BLE. This closes open question Q21 in [`PLAN.md`](../../../docs/PLAN.md).

## What Changes

* **Home mode.** At boot, if the firmware was built with home Wi-Fi settings, the car joins the home Wi-Fi as a station (DHCP) and connects to the configured ROS router. When both succeed it serves ROS 2 commands there as today ([ADR-0011](../../../docs/adr/0011-ros-auto-arm.md) auto-arm unchanged).
* **Fallback.** If no settings were built in, the network is not seen in a ~1 s scan, or join + address + ROS router do not succeed within ~5 s, the car runs exactly today's behaviour: its own AP with UDP, ROS and BLE, first claim wins. Fallback lasts until reboot. AP and station never run at the same time.
* **BLE override.** In home mode BLE keeps advertising. A BLE connection takes ownership from any Wi-Fi owner (safety stop, then a disarmed BLE owner using today's BLE protocol, including explicit ARM) and turns Wi-Fi off. When that BLE connection ends, for any reason, the car restarts the home-mode attempt; a streaming ROS controller then drives again without an operator step. **BREAKING** for [ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md)'s first-claim rule, in home mode only.
* **Settings at build time.** SSID, passphrase and the home ROS router endpoint are build variables, like today's `ROVI_WIFI_PASSWORD`, flashed over USB. Nothing about the home network is sent over the air.
* **Trust.** Any device on the home network may control the car, with no new authentication ([ADR-0008](../../../docs/adr/0008-local-link-security-baseline.md) station-mode exclusion superseded by a new ADR).
* **Observability.** New log events report the network mode and each join step with its duration.

Out of scope: running AP and station together, UDP authentication, mDNS/zenoh scouting for the router, NTP/OTA, changing home settings without a reflash, retrying home Wi-Fi from fallback without a reboot, client changes.

## Capabilities

### New Capabilities

* `network-mode`: choosing home or fallback networking at boot and after a BLE override, the join budget, home-mode link loss, build-time home settings and the mode events.

### Modified Capabilities

* `car-control-api`: the *Single owner* requirement gains home-mode BLE advertising and preemption; fallback mode keeps today's rules. Claim, reclaim, arming, lease and stop rules are unchanged.

## Impact

* Firmware: Wi-Fi station configuration and DHCP (embassy-net `dhcpv4`), a mode supervisor, build variables `ROVI_HOME_SSID`, `ROVI_HOME_PASSWORD` and `ROVI_HOME_ROUTER`, and a per-mode ROS router endpoint. No new dependency, GATT characteristic or flash storage.
* `rovi-core`: an arbiter preemption path for BLE in home mode and new log events; simulator and host tests.
* Clients: unchanged.
* Docs: new ADR superseding parts of ADR-0004 and ADR-0008; `PLAN.md` Q21 closed; glossary STA entry updated.
* Acceptance needs board checks on a real home network, including BLE/Wi-Fi coexistence while ROS drives; each motor batch needs its own announced go.
