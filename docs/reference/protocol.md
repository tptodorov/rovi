# Car control protocol: one operation, three transports

The same rules on every transport ([car-control-api](../../openspec/specs/car-control-api/spec.md), [network-mode](../../openspec/changes/add-home-network-mode/specs/network-mode/spec.md)); this page is the mapping. Byte layouts and codecs: [`rovi-protocol`](../../crates/rovi-protocol/src/) ([ADR-0017](../adr/0017-one-rust-protocol-crate.md)), with generated [vectors](../../crates/rovi-protocol/vectors.json). The [conformance suite](../../apps/simcar/scenarios/conformance.py) runs the shared rules over all three.

The product car's BLE device name and car AP SSID are both `Rovi`. BLE discovery matches that name and checks the service UUID after connecting; the service UUID is absent from the advertisement.

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

## Intentional differences

Everything else is the same on every transport. The list lives in [`transports.rs`](../../crates/rovi-protocol/src/transports.rs); `cargo test -p rovi-protocol --test docs` keeps this table in step (`ROVI_BLESS_DOCS=1` rewrites it).

<!-- generated: rovi-protocol transports::table -->
| # | Difference | Why it stays |
| --- | --- | --- |
| T1 | Claim: BLE connects and claims on its first write; UDP sends `HELLO`; ROS claims on the first `cmd_vel` from a node in the graph | Each transport's own session model (ADR-0013, ADR-0016) |
| T2 | Release: BLE disconnects; UDP sends `BYE`; ROS publishes `/rovi/release` | Same as T1 |
| T3 | ROS arms on its own stream; BLE and UDP need an explicit ARM | Stock ROS tools publish only `cmd_vel` (ADR-0011) |
| T4 | Sequences: u32 serial on BLE and UDP; the ROS attachment's i64, per publisher | ROS defines its attachment |
| T5 | No takeover notice on BLE | BLE is never taken over |
| T6 | ROS Status text carries `session=`; binary Status does not | ROS has no handshake that returns a session; bench tools use it |
| T7 | Echo: up to 32 bytes on BLE and UDP; `std_msgs/Int64` on ROS | The ROS-native type |
| T8 | Mode command: BLE Control 3 / 4; UDP `MODE` and ROS `UInt8` 1 / 2 | Renumbering would break the accepted phone/firmware pairing for no gain; one `Command` type maps both |
| T9 | Status, Capabilities and network information are logfmt text on ROS, binary elsewhere | Readable in ROS tools; both forms come from the same type |
| T11 | Delivery: BLE read/notify; UDP `INFO`/`HELLO_ACK` replies and unsolicited `STATUS`; ROS topics republished about once a second | Each transport's way to deliver state; the conformance suite checks the values agree |
<!-- end generated -->

## Takeover and modes

* Takeover rank: BLE > UDP > ROS. UDP takes the car from ROS in both modes; BLE takes it from UDP or ROS in home mode only. The new owner starts disarmed ([ADR-0016](../adr/0016-home-network-mode-with-ble-override.md)).
* A mode command needs a disarmed owner and applies when that ownership ends. Fallback mode (the car's own network) lasts until reboot or a Home command.
* Network information: `mode` 0 fallback / 1 attempting / 2 home; `pending` 0 none / 1 home / 2 car network; `reason` 0 none, 1 no-settings, 2 scan, 3 join, 4 dhcp, 5 router, 6 link-lost, 7 requested; byte 3 reserved; then the station IPv4.
* Errors on every transport: not-ready (armed, or ARM without a fresh setpoint), invalid (unknown value or length, Home without home settings).
