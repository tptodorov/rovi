---
status: accepted
---

# Home network mode with a BLE override; the car AP becomes the fallback

A ROS 2 controller running as a service on the home network should drive the car around the house, and Todor should still be able to take over by hand ([Q21](../PLAN.md#open-questions-and-decisions)). We decided (2026-10-22):

* **Home mode first.** With home settings built in, the car joins the home Wi-Fi as a station and connects to the configured ROS router. Without settings, or if the network is not found (~1 s scan) or join + DHCP + router take longer than ~5 s, it runs exactly today's AP behaviour until reboot or a Home mode command. AP and station never run together.
* **Takeover priority BLE > UDP > ROS.** UDP takes the car from ROS in both modes; in home mode BLE keeps advertising and takes it from UDP or ROS. A takeover stops the car and leaves the new owner disarmed. This supersedes [ADR-0004](0004-single-owner-latest-setpoint.md)'s first-claim rule for these cases.
* **Safe hand-back.** A takeover that ends without a Home command (e.g. an accidental BLE drop) leaves ROS needing `/rovi/arm` true once; with Home sent first, ROS resumes and auto-arms.
* **Mode command and network information on every transport,** so each mode is reachable from BLE alone or Wi-Fi alone, without reflashing or touching the car.
* **The home network is trusted.** Any device on it may control the car, with no new authentication. This supersedes [ADR-0008](0008-local-link-security-baseline.md)'s "no station mode".
* **Aligned protocols.** BLE claims on its first command (two connections, so a watcher never blocks a rescue); capabilities, Status and network information are readable before claiming everywhere; ROS gets an explicit release; a preempted owner gets a final owner-lost Status. One conformance suite runs the shared rules over all three transports.
* **Home settings are baked in at build time** (SSID, passphrase, router endpoint) and flashed over USB, like the AP password. Nothing about the home network goes over the air.

Details: [`add-home-network-mode`](../../openspec/changes/add-home-network-mode/design.md).

## Considered options

* **AP and station at once.** Rejected: channel sharing and two networks for no use Todor needs; the AP is only a fallback.
* **Keep first-claim ownership everywhere.** Rejected: the operator could not take the car back from a misbehaving home controller without reaching it.
* **Any BLE disconnect hands back with auto-arm.** Rejected on safety review: an accidental drop mid-rescue would let ROS drive at once.
* **Mode command on BLE only.** Rejected: a Wi-Fi-only operator could not leave fallback without a reboot.
* **Automatic home retries from fallback.** Rejected: each scan blinks the car AP off.
* **Provisioning over BLE.** Rejected: the home passphrase would cross unencrypted BLE (ADR-0008). Pairing or a USB settings protocol would avoid that but add storage and protocol for a rare operation.

## Consequences

* After an accidental takeover end, stock ROS tools that publish only `/cmd_vel` cannot resume until something publishes `/rovi/arm` true ([ADR-0011](0011-ros-auto-arm.md) narrowed).
* Any UDP client on the home network can take the car from a ROS controller.
* Anyone in BLE range can stop the car in home mode (ADR-0008's BLE risk, now with more reach).
* Changing the home network needs a rebuild and USB flash.
* BLE advertising runs alongside Wi-Fi traffic in home mode; coexistence must be measured on the board.
* Fallback mode is unchanged, so existing clients, tests and bench evidence keep holding.
