# Home network mode

**Status:** accepted for implementation on 2026-10-22 (Todor: “merge them and continue with next steps”, PR #107). No flash or motor run is authorized; each board step is announced separately.

## Why

Rovi can only be driven from a device on its own access point (AP) or over BLE, so a ROS 2 controller must run on a laptop that has given up its normal Wi-Fi. Todor wants a controller running as a service on the home network to drive the car around the house and take part in the other ROS 2 activity there, while he can still take over by hand over BLE. This closes open question Q21 in [`PLAN.md`](../../../docs/PLAN.md).

## What Changes

* **Home mode.** At boot, if the firmware was built with home Wi-Fi settings, the car joins the home Wi-Fi as a station (DHCP) and connects to the configured ROS router. When both succeed it serves ROS 2 (and UDP) there ([ADR-0011](../../../docs/adr/0011-ros-auto-arm.md) auto-arm unchanged).
* **Fallback.** If no settings were built in, the network is not seen in a ~1.5 s scan, or join + address + ROS router do not succeed within ~8 s, the car runs exactly today's AP behaviour. It stays there until reboot or a Home mode command. AP and station never run at the same time.
* **Mode command on every transport.** A disarmed owner on BLE, UDP or ROS can choose Home or Car network; it takes effect when that ownership ends. So every mode is reachable from any one type of client, without reflashing or touching the car.
* **Network information on every transport.** Mode, pending command, last fallback reason and the car's home IP, readable by owners and non-owners (BLE characteristic, UDP kind, ROS topic).
* **Takeover priority: BLE > UDP > ROS.** UDP preempts a ROS owner in both modes; in home mode BLE (which keeps advertising) preempts UDP or ROS. Preemption stops the car and leaves the new owner disarmed. **BREAKING** for [ADR-0004](../../../docs/adr/0004-single-owner-latest-setpoint.md)'s first-claim rule.
* **Safe hand-back.** When a manual takeover ends without a Home command (e.g. an accidental BLE drop), the next ROS owner must publish `/rovi/arm` true once before moving. With Home sent first, ROS takes over and auto-arms as today.
* **Protocol alignment.** Every transport gets the same operations (todor, 2026-10-22):
  * BLE claims on its first Setpoint/Control write, not on connect, with up to two connections, so a BLE central can watch without claiming.
  * Capabilities, Status and network information are readable before claiming everywhere (UDP `INFO`, kind 11).
  * ROS gets an explicit release (`/rovi/release`) like UDP BYE and BLE disconnect.
  * A preempted owner receives a final Status (stopped, owner-lost) on its own transport.
  * One transport-independent scenario suite runs the shared rules over BLE, UDP and ROS in the simulator.
* **Settings at build time.** SSID, passphrase and the home ROS router endpoint are build variables, like today's `ROVI_WIFI_PASSWORD`, flashed over USB. Nothing about the home network is sent over the air.
* **Trust.** Any device on the home network may control the car, with no new authentication ([ADR-0008](../../../docs/adr/0008-local-link-security-baseline.md) station-mode exclusion superseded).
* **Observability.** New log events report mode changes, mode commands and each join step with its duration.

Deferred: a shared Rust client crate for all clients ([#108](https://github.com/tptodorov/rovi/issues/108)), the iPhone mode/network UI ([#109](https://github.com/tptodorov/rovi/issues/109)) and resolving the router by name via mDNS ([#115](https://github.com/tptodorov/rovi/issues/115)). Out of scope: a ROS echo (router liveliness covers it), running AP and station together, UDP authentication, mDNS/zenoh scouting for the router, NTP/OTA, changing home settings without a reflash, automatic retry of home Wi-Fi from fallback, BLE preemption in fallback mode.

## Capabilities

### New Capabilities

* `network-mode`: home/fallback selection, join budget, mode command and its errors on all transports, network information, hand-back latch, BLE availability in home mode, link loss, build-time settings and mode events.

### Modified Capabilities

* `car-control-api`: modified *Single owner* (BLE > UDP > ROS preemption), *Explicit arming* (hand-back latch), *Capabilities* and *Status* (readable before claiming), *Echo is observational* (BLE echo without claiming); added *BLE claim on first command*, *Observation without ownership*, *ROS release* and *Takeover notice*. Claim/reclaim windows, lease, stop rules and existing message encodings are unchanged.

## Impact

* Firmware: Wi-Fi station and DHCP (embassy-net `dhcpv4`), build variables `ROVI_HOME_SSID`, `ROVI_HOME_PASSWORD`, `ROVI_HOME_ROUTER`, a per-mode ROS router endpoint, one new GATT characteristic (Network), ROS topics `/rovi/mode` and `/rovi/network`. No new dependency or flash storage.
* `rovi-core`: mode supervisor, arbiter preemption, hand-back latch, mode command and network-information codecs on all three transports, new log events; simulator and host tests.
* Clients: laptop teleop gains `--mode home|car` and shows network information; the iPhone client keeps working unchanged (mode UI optional later).
* Docs: new ADR superseding parts of ADR-0004 and ADR-0008; `PLAN.md` Q21 closed; glossary STA entry updated.
* Acceptance needs board checks on a real home network, including BLE/Wi-Fi coexistence while ROS drives; each motor batch needs its own announced go.
