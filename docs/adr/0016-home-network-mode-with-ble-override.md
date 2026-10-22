---
status: proposed
---

# Home network mode with a BLE override; the car AP becomes the fallback

A ROS 2 controller running as a service on the home network should drive the car around the house, and Todor should still be able to take over by hand ([Q21](../PLAN.md#open-questions-and-decisions)). We decided (2026-10-22):

* **Home mode first.** With home settings built in, the car joins the home Wi-Fi as a station and connects to the configured ROS router. Without settings, or if the network is not found (~1 s scan) or join + DHCP + router take longer than ~5 s, it runs exactly today's AP behaviour until reboot. AP and station never run together.
* **BLE overrides in home mode.** BLE keeps advertising while a Wi-Fi controller drives. A BLE connection stops the car, takes ownership disarmed and turns Wi-Fi off; any BLE disconnect restarts the home-mode attempt. This supersedes [ADR-0004](0004-single-owner-latest-setpoint.md)'s first-claim rule in home mode only.
* **The home network is trusted.** Any device on it may control the car, with no new authentication. This supersedes [ADR-0008](0008-local-link-security-baseline.md)'s "no station mode".
* **Home settings are baked in at build time** (SSID, passphrase, router endpoint) and flashed over USB, like the AP password. Nothing about the home network goes over the air.

Details: [`add-home-network-mode`](../../openspec/changes/add-home-network-mode/design.md).

## Considered options

* **AP and station at once.** Rejected: channel sharing and two networks for no use Todor needs; the AP is only a fallback.
* **Keep first-claim ownership everywhere.** Rejected: the operator could not take the car back from a misbehaving home controller without reaching it.
* **Hand back only on a deliberate BLE release.** Rejected by Todor: any BLE disconnect returns to home mode.
* **Provisioning over BLE.** Rejected: the home passphrase would cross unencrypted BLE (ADR-0008). Pairing or a USB settings protocol would avoid that but add storage and protocol for a rare operation.

## Consequences

* An accidental BLE drop hands the car back to any ROS node still streaming, which then drives without an operator step ([ADR-0011](0011-ros-auto-arm.md)).
* Anyone in BLE range can stop the car in home mode (ADR-0008's BLE risk, now with more reach).
* Changing the home network needs a rebuild and USB flash.
* BLE advertising runs alongside Wi-Fi traffic in home mode; coexistence must be measured on the board.
* Fallback mode is unchanged, so existing clients, tests and bench evidence keep holding.
