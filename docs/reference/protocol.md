# Car control protocol: one operation, three transports

The same rules on every transport ([car-control-api](../../openspec/specs/car-control-api/spec.md), [network-mode](../../openspec/changes/add-home-network-mode/specs/network-mode/spec.md)); this page is the mapping. Byte layouts: [`rovi-core`](../../crates/rovi-core/src/) (`udp.rs`, `device.rs`, `status.rs`, `network.rs`). The [conformance suite](../../apps/simcar/scenarios/conformance.py) runs the shared rules over all three.

| Operation | BLE (GATT service `d47a0010-…`) | UDP (port 7777, `RV` v1 frames) | ROS 2 (node `/rovi`) |
| --- | --- | --- | --- |
| Watch (no claim) | Connect; read/notify Status, Capabilities, Network; Echo | `INFO` (11) from any peer: capabilities + Status + network | Subscribe `/rovi/status`, `/rovi/capabilities`, `/rovi/network` |
| Claim | First Setpoint or Control write (clients send STOP, `0x02`, and use its response) | `HELLO` (1) → `HELLO_ACK` (2) with capabilities, or `BUSY` (8) | First valid `/cmd_vel` from a node announced in the graph |
| Setpoint | Setpoint `d47a0011` (write without response): `seq u32` LE + CDR `TwistStamped` | `SETPOINT` (3): CDR `TwistStamped` | `/cmd_vel` `geometry_msgs/TwistStamped` |
| Arm | Control `d47a0012` = 1 | `ARM` (4) → `STATUS` | Automatic on the stream ([ADR-0011](../adr/0011-ros-auto-arm.md)); `/rovi/arm` true after STOP, out-of-limits or a takeover without Home |
| Stop | Control = 2 | `STOP` (5) → `STATUS` | `/rovi/arm` false |
| Mode command | Control = 3 Home, 4 Car network | `MODE` (10), payload 1 / 2 → `STATUS` | `/rovi/mode` `std_msgs/UInt8` 1 / 2 |
| Release | Disconnect | `BYE` (6) | `/rovi/release` `std_msgs/Empty` |
| Status | Status `d47a0014` read/notify: state, reason, error | `STATUS` (7), replies and unsolicited (sequence 0) | `/rovi/status` `std_msgs/String` |
| Capabilities | Capabilities `d47a0013` | in `HELLO_ACK` and `INFO` | `/rovi/capabilities` |
| Network information | Network `d47a0016` read/notify (8 bytes) | in `INFO` | `/rovi/network` logfmt string |
| Takeover notice | — (BLE is never taken over) | unsolicited `STATUS` stopped/owner-lost to the old session | `/rovi/status` stopped/owner-lost |
| Echo (link probe) | Echo `d47a0015` | `ECHO` (9), any peer | `/rovi/echo` → `/rovi/echo_reply` |

## Takeover and modes

* Takeover rank: BLE > UDP > ROS. UDP takes the car from ROS in both modes; BLE takes it from UDP or ROS in home mode only. The new owner starts disarmed ([ADR-0016](../adr/0016-home-network-mode-with-ble-override.md)).
* A mode command needs a disarmed owner and applies when that ownership ends. Fallback mode (the car's own network) lasts until reboot or a Home command.
* Network information: `mode` 0 fallback / 1 attempting / 2 home; `pending` 0 none / 1 home / 2 car network; `reason` 0 none, 1 no-settings, 2 scan, 3 join, 4 dhcp, 5 router, 6 link-lost, 7 requested; byte 3 reserved; then the station IPv4.
* Errors on every transport: not-ready (armed, or ARM without a fresh setpoint), invalid (unknown value or length, Home without home settings).
