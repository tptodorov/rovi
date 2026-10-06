# S1 spike: Rust/embassy firmware as a micro-ROS node (nano-ros)

**Spike date:** 2026-10-06, on the laptop (`blade`); no board.

**Status:** spike findings. The spike code was throwaway and is not in the repo. This is not a decision; the M8 design consumes it.

**Question:** can Rovi's ESP32-S3 Rust/embassy firmware be a micro-ROS node over UDP, for M8's optional `ros` feature?

**Answer:** yes, with conditions. nano-ros's XRCE-DDS backend cross-compiles for the S3, links next to Rovi's stack, and interoperates with stock ROS 2 through the official micro-ROS agent. Rovi would own the S3 port (13 platform symbols). Footprint and on-board behaviour are untested.

## Setup

* Host: NixOS, Linux 7.2.8, Docker 29.8.0.
* [nano-ros](https://github.com/NEWSLabNTU/nano-ros) at `6095183` (2026-10-06). XRCE submodules: Micro-XRCE-DDS-Client `bdfa280`, Micro-CDR `9967249`.
* Embedded toolchain: espup `esp` Rust (1.97 nightly-based), `xtensa-esp32s3-elf-gcc` esp-15.2.0_20250920, built inside the repo's `nix-shell`.
* Containers: `microros/micro-ros-agent:jazzy` (`sha256:76b7a438…`) and `ros:jazzy-ros-base` (`sha256:066420e0…`), both with `--net host`.
* Rovi baseline: `experiments/rovi-m3-wifi-queue` (esp-hal 1.2.2, esp-radio 1.0.0-beta.1, esp-rtos 0.4, embassy-net 0.8, trouble-host 0.8).

## Findings

| # | Check | Result |
| --- | --- | --- |
| 1 | Upstream ESP32-S3 support | **None.** The docs say: "ESP32-S3 (Xtensa) is NOT supported today… the in-tree board crate is RISC-V only" ([ESP32 guide](https://github.com/NEWSLabNTU/nano-ros/blob/6095183/book/src/getting-started/esp32.md)). The only ESP32 path is the C3 bare-metal under QEMU OpenETH, with no Wi-Fi. |
| 2 | Core crates (`nros`, features `alloc,rmw-cffi,ros-jazzy`) for `xtensa-esp32s3-none-elf` with `-Z build-std=core,alloc` | **Pass,** unmodified. |
| 3 | XRCE backend (`nros-rmw-xrce-cffi`, C) with `CC=xtensa-esp32s3-elf-gcc` | **Pass.** 36 objects, all `ELF 32-bit Tensilica Xtensa`, ~464 KiB unstripped (not a flash size). |
| 4 | M3 firmware plus nano-ros dependencies | **Pass.** Builds and links, so no version conflicts with Rovi's stack. Caveat: nothing was called, so the linker dropped nano-ros (dependency compatibility only). |
| 5 | M3 firmware calling `nros_rmw_xrce_cffi::register()` | **Link fails, as expected,** with 13 undefined platform symbols (below). This is the glue Rovi must write. |
| 6 | Host interop: ROS 2 Jazzy `ros2 topic pub /chatter` (Fast DDS) → micro-ROS agent (UDP 8888) → nano-ros XRCE listener (native) | **Pass,** 25/25 messages. The agent log shows session, participant, topic, subscriber and datareader created. |
| 7 | Host interop: nano-ros XRCE talker → agent → `ros2 topic echo /chatter` | **Pass.** 10 messages arrived in the echo window; it started ~3 s after the talker. |

### Platform glue Rovi must provide (from finding 5)

| Symbol(s) | Rovi implementation |
| --- | --- |
| `nros_platform_alloc`, `nros_platform_dealloc` | esp-alloc heap |
| `nros_platform_clock_ns`, `nros_platform_sleep_ms` | embassy-time / esp-hal timer |
| `nros_platform_log_write` | esp-println |
| `strtol` | ~20-line C or Rust shim (bare metal has no libc) |
| `nros_platform_udp_{create_endpoint,free_endpoint,open,close,send,read,set_recv_timeout}` | Bridge to embassy-net UDP (see below) |

* **The blocking-vs-async mismatch is the main design issue.** The UDP calls are synchronous and blocking, with a receive timeout; embassy-net is async. Candidates:
  * **Dual core:** nano-ros spins blocking on core 1, while an embassy task on core 0 owns the `UdpSocket` and exchanges datagrams through lock-free queues.
  * **Polling:** non-blocking reads (timeout 0) from a queue, through XRCE's custom-transport hook (`transport_custom.c`: open/close/write/read callbacks).
* **nano-ros's ESP32 platform crate isn't reusable.** It is ~860 lines and brings its own smoltcp interface (`nros-smoltcp`), which would conflict with embassy-net over the single Wi-Fi device. It also pins `esp-hal ~1.0` / `esp-backtrace ~0.18` for the C3, while Rovi uses 1.2.2 / 0.20.

### Observations about nano-ros

* It's a framework, not just a library. `system.toml`, `nros sync` code generation, generated message crates, `nros::main!` / `node!` macros and board crates own the executor. Rovi would use the core plus backend and skip the board/entry layer.
* It's very fast-moving: ~13k commits, with features tracked by "phase" numbers. Pin a commit.
* It needs nightly Rust; Rovi's `esp` toolchain already qualifies.
* Build gotchas seen on the host:
  * `nros build` ignored `rmw = "xrce"` and still built the default `rmw-zenoh` Cargo feature, which needs the zenoh-pico submodule. Workaround: `cargo build --config build/native/nros-cargo.toml --no-default-features --features rmw-xrce,ros-jazzy`.
  * `nros sync`'s sizing probe failed under the nix rustc (no `rust-src` Cargo.lock), so pools fell back to defaults.
  * `scripts/bootstrap.sh` proposes appending to `~/.bashrc` (it skipped this without a TTY).
* The XRCE locator is `udp/<host>:<port>` via `NROS_LOCATOR`; the domain comes from `ROS_DOMAIN_ID`.

## Not tested

* Anything on the ESP32-S3 itself: Wi-Fi AP transport, timing, coexistence with BLE and the control loop.
* Flash, RAM, heap and CPU footprint of a real node on the S3.
* `geometry_msgs/TwistStamped` (only `std_msgs/String` was used), and best-effort QoS.
* The fallback, calling eProsima's Micro XRCE-DDS client directly over FFI.

## Recommendation for M8

1. The `ros` feature is feasible, so keep it in the M8 design.
2. Rovi needs only one subscription (velocity setpoint) and one publication (status). nano-ros's backend wraps the same eProsima client, so calling that client directly may be leaner: a 4-callback custom transport plus hand-written CDR for one message, with no code-generation framework.
3. Decide **nano-ros vs direct XRCE client** in the M8 design. Optionally run a short follow-up spike that builds both minimal S3 nodes (one `TwistStamped` subscriber, one status publisher) and compares flash/RAM and glue size.
4. Whichever is chosen, bridge blocking UDP to embassy-net through a queue owned by an embassy task, and evaluate the second-core option.

## Reproduce

```sh
git clone https://github.com/NEWSLabNTU/nano-ros && cd nano-ros && git checkout 6095183
git submodule update --init --depth 1 packages/rmw/xrce/xrce-sys/{micro-xrce-dds-client,micro-cdr}

# S3 cross-build (inside the repo's nix-shell, after `. ~/export-esp.sh`), probe crate depending on
# nros {alloc,rmw-cffi,ros-jazzy} + nros-rmw-xrce-cffi:
CC_xtensa_esp32s3_none_elf=xtensa-esp32s3-elf-gcc \
  cargo +esp build --release --target xtensa-esp32s3-none-elf -Z build-std=core,alloc

# Host interop
bash scripts/bootstrap.sh && source activate.sh
cd examples/native/rust/listener && sed -i 's/^rmw = "zenoh"/rmw = "xrce"/' system.toml && nros sync
cargo build --config build/native/nros-cargo.toml --no-default-features --features rmw-xrce,ros-jazzy
docker run -d --name agent --net host microros/micro-ros-agent:jazzy udp4 --port 8888
NROS_LOCATOR=udp/127.0.0.1:8888 build/native/target/debug/listener &
docker run --rm --net host ros:jazzy-ros-base bash -c \
  'source /opt/ros/jazzy/setup.bash && ros2 topic pub -r 5 -t 25 /chatter std_msgs/msg/String "{data: hi}"'
```
