# S2 spike: native Rust/embassy ROS 2 node via zenoh-nostd + rmw_zenoh

**Spike date:** 2026-10-06, on the laptop (`blade`); no board.

**Status:** spike findings. The spike code was throwaway and is not in the repo. This is not a decision; the M8 design consumes it, alongside [S1](s1-micro-ros-nano-ros-spike.md).

**Question:** can the car be a ROS 2 node using pure Rust, embassy-native [zenoh-nostd](https://github.com/eclipse-zenoh/zenoh-nostd), talking to stock ROS 2 through `rmw_zenoh` over UDP, without the C/XRCE stack S1 needed?

**Answer:** yes for data in both directions, and it links into Rovi's actual firmware stack at +101 KiB flash. But the car is **invisible to the ROS graph** until we add `rmw_zenoh` liveliness tokens (missing from zenoh-nostd's API). zenoh-nostd is early-stage and quiet. Small integration gaps: it pins `embassy-sync` 0.7 and hard-enables `defmt`.

## Setup

* zenoh-nostd `e88f73a` (2026-06-26, the last push; one PR updated 2026-08-15). EPL-2.0 OR Apache-2.0; README: "⚠️ This project is in early development".
* ROS 2: `ros:jazzy-ros-base` + `ros-jazzy-rmw-zenoh-cpp` 0.2.10 (zenoh-cpp-vendor 0.2.10), `RMW_IMPLEMENTATION=rmw_zenoh_cpp`. The router `rmw_zenohd` runs in Docker with `--net host`.
* Embedded toolchain: espup `esp` Rust, `xtensa-esp32s3-elf-gcc` esp-15.2.0_20250920, in the repo's `nix-shell`.
* Rovi baseline: `experiments/rovi-m3-wifi-queue` (esp-hal 1.2.2, esp-radio 1.0.0-beta.1, esp-rtos 0.4, embassy-net 0.8, embassy-sync 0.8, trouble-host 0.8).

## Findings

| # | Check | Result |
| --- | --- | --- |
| 1 | Upstream ESP32-S3 example (`z_sub`, `--features esp32s3,defmt`, Wi-Fi STA) | **Builds** on its own pinned stack (esp-hal 1.0, esp-rtos 0.2, esp-radio 0.17). App image 613,392 B, `.bss` 76 KB. With `log` instead of `defmt` it fails (453 errors in `zenoh-proto`), so only `defmt` logging works on the S3. |
| 2 | `zenoh-nostd` + `zenoh-embassy` added to M3 (Rovi's newer stack) | **Link fails** until `defmt` is supplied: `zenoh-embassy` hard-enables `defmt` on `embassy-net` / `embedded-io-async` (undefined `_defmt_*`). With those features removed, it builds. |
| 3 | A real session in M3: `connect!` over M3's AP `embassy-net` stack, `udp/192.168.4.2:7448`, one subscriber, one publisher with attachment | **Builds and links** for `xtensa-esp32s3-none-elf`. Needed: `StackResources` made `'static` (the session config is `'static`); an `embassy-sync` **0.7** alias for zenoh's channel types (Rovi uses 0.8, and the types don't unify); a `getrandom` custom backend on esp `Rng`; and callback-table capacities ≥ 2 (heapless). |
| 4 | Footprint, M3 vs M3 + zenoh session (release) | **+103,744 B app image (+101 KiB, 685,904 → 789,648)**; `.text` +103,656 B. `.data`+`.bss` unchanged (±0.1 KiB). Runtime stack and heap use is not measured. |
| 5 | ROS 2 `ros2 topic pub /chatter std_msgs/String` → `rmw_zenohd` (TCP) → host zenoh-nostd `z_sub` on `**` | **Pass, 20/20.** Key `0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c…1a18`; payload plain CDR (`00 01 00 00`, u32 len, bytes, NUL). |
| 6 | Same over UDP (router `listen/endpoints` += `udp/0.0.0.0:7448`, client `udp/127.0.0.1:7448`) | **Pass, 20/20.** |
| 7 | zenoh-nostd `z_pub` → `ros2 topic echo /chatter std_msgs/msg/String` | **Pass, 15/15,** only with the `rmw_zenoh` attachment. With no attachment: `rmw_zenoh_cpp: Unable to obtain attachment` and nothing delivered. A 32-byte attachment fails: "Failed to read the sequence length". |
| 8 | ROS graph: `ros2 node list` / `topic list` / `topic info /chatter` while zenoh-nostd publishes | **Invisible:** no node, no `/chatter`, "Unknown topic". `ros2 topic pub` (default) waits forever for a matching subscriber; `-w 0` bypasses this. |
| 9 | Receive buffer limits | The default `FixedCapacitySample<128,128>` rejected some samples on `**` ("Couldn't convert to a transferable sample"), e.g. long ROS keys or payloads. Callback storage is a fixed `RawOrBox<600>`, so key+payload is capped at ~256 B per sample (`<160,96>` fits; `<192,128>` does not). Larger needs a bigger `RawOrBox` or `alloc`. |

### rmw_zenoh wire contract the car must implement

From `rmw_zenoh` Jazzy ([attachment_helpers.cpp](https://github.com/ros2/rmw_zenoh/blob/jazzy/rmw_zenoh_cpp/src/detail/attachment_helpers.cpp)) and the observations above:

* **Key:** `<domain>/<topic without leading slash>/<pkg>::msg::dds_::<Type>_/<RIHS01_type-hash>`.
* **Payload:** CDR little-endian with the 4-byte encapsulation header `00 01 00 00`.
* **Attachment, 33 bytes,** in zenoh `ext::Serializer` format:
  * `sequence_number`: i64 LE.
  * `source_timestamp`: i64 LE.
  * `source_gid`: a 16-byte array, prefixed with its LEB128 length `0x10`.
* **Graph discovery (not done in this spike):** `rmw_zenoh` liveliness tokens. zenoh-nostd's protocol layer has `DeclareToken`/`UndeclareToken` messages, but the session API doesn't expose them, so this needs a patch or an upstream contribution.

## Integration notes for Rovi

* It's async and embassy-native, using the same `embassy-net` 0.8 as Rovi. There is no C, no code generation and no agent, but a zenoh router is required (`rmw_zenohd` on the ROS host, or any `zenohd`).
* UDP works (`udp/<host>:<port>`). In the car-AP topology the car connects to the router on the client, e.g. `udp/192.168.4.2:7448`, which fits M8's "agent host joins the car AP".
* Dependency gaps, to patch or upstream:
  * `defmt` is mandatory in `zenoh-embassy`, while Rovi logs with esp-println.
  * `embassy-sync` is 0.7 (Rovi: 0.8).
  * The upstream examples pin an older esp stack.
* Messages need hand-written CDR (`TwistStamped` is a header plus 6 × f64). Type hashes are fixed per message type and ROS distro, and can be compile-time constants.
* Project risk: it's early-stage, with no commits since 2026-06-26. Expect to carry patches.

## Not tested

* Anything on the ESP32-S3 itself: the radio, AP-mode UDP to the router, latency, reconnection, BLE coexistence.
* Runtime RAM (stack/heap) of the session.
* Liveliness tokens and ROS graph visibility; `geometry_msgs/TwistStamped`; QoS behaviour.

## Comparison with S1 (nano-ros / XRCE)

| | S1 nano-ros XRCE | S2 zenoh-nostd + rmw_zenoh |
| --- | --- | --- |
| Language / model | C client + Rust framework, blocking | Pure Rust, async embassy |
| S3 support | none upstream; we write 13 platform symbols and a blocking→async bridge | upstream S3 example; links into Rovi with small patches |
| ROS side | micro-ROS agent | `rmw_zenohd` router (`ros-jazzy-rmw-zenoh-cpp` binary package) |
| ROS graph visibility | yes (the agent creates DDS entities) | **no** until liveliness tokens are added |
| Flash cost measured | not measured | +101 KiB in M3 |
| Maturity | very active, fast-moving | early, quiet since June |

## Recommendation for M8

1. **Prefer zenoh-nostd for the `ros` feature.** It's the only option that is native async Rust on embassy. Data interop in both directions is proven, and the flash cost is moderate.
2. **Make graph visibility the first M8 risk item.** Implement `rmw_zenoh` liveliness tokens, by patching zenoh-nostd's session API or upstreaming it. Then verify `ros2 node list` / `topic info` and normal `ros2 topic pub` without `-w 0`. If that proves impractical, fall back to S1's XRCE path.
3. Carry three small patches, or upstream them: optional `defmt` in `zenoh-embassy`, an `embassy-sync` 0.8 bump, and current esp-hal/esp-radio examples.
4. Keep the `rmw_zenoh` wire contract above as the spec for Rovi's ROS adapter: key builder, CDR codec for `TwistStamped` and status, and the attachment encoder. All of it is host-testable.

## Reproduce

```sh
git clone https://github.com/eclipse-zenoh/zenoh-nostd && cd zenoh-nostd && git checkout e88f73a
# ROS 2 + rmw_zenoh image
printf 'FROM ros:jazzy-ros-base\nRUN apt-get update && apt-get install -y ros-jazzy-rmw-zenoh-cpp\nENV RMW_IMPLEMENTATION=rmw_zenoh_cpp\n' | docker build -t ros-zenoh -
docker run -d --name router --net host -e ZENOH_CONFIG_OVERRIDE='listen/endpoints=["tcp/[::]:7447","udp/0.0.0.0:7448"]' \
  ros-zenoh bash -c 'source /opt/ros/jazzy/setup.bash && ros2 run rmw_zenoh_cpp rmw_zenohd'
# host subscriber (examples/z_sub.rs patched: keyexpr "**", hex payload, FixedCapacitySample<160, 96>)
ENDPOINT=udp/127.0.0.1:7448 cargo run --release --example z_sub --no-default-features --features std,log
docker run --rm --net host ros-zenoh bash -c 'source /opt/ros/jazzy/setup.bash && ros2 topic pub -w 0 -r 5 -t 20 /chatter std_msgs/msg/String "{data: hi}"'
# S3 upstream example
WIFI_PASSWORD=x cargo +esp --config .cargo/config.esp32s3.toml build --release --example z_sub --no-default-features --features esp32s3,defmt
```
