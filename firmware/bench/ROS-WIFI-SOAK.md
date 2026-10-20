# Wi-Fi observation with an idle ROS session

Follow-up to the [Echo-only baseline](2026-10-20-wifi-soak/notes.md) for
[#23](https://github.com/tptodorov/rovi/issues/23). It adds a connected ROS router
and `iw` snapshots, with the current normal firmware. A clean run leaves #23 open.

## Scope and setup

Fifteen minutes of observation, plus setup/cleanup. Motor power disconnected,
ESP32 USB connected. No flash, BLE connection, ROS application publisher, claim,
ARM or setpoint. Opening serial capture resets once. The firmware's usual ROS
discovery/publication/keepalive traffic remains; the observer sends UDP Echo.

Use the project's declared Nix shell: `iw` is included there, without changing
the running system or wireless power-saving policy. The installed normal ROS
image connects over TCP to `192.168.4.2:7447`. The old October 13 UDP-router
transport is not replayed; no firmware rollback belongs to this check.

1. Save the previous Wi-Fi UUID, internet route and exact scoped firewall-rule
   presence. Start fresh serial capture and verify a normal boot/free car.
2. Join `rovi-m8`; add the temporary firewall accept rule only if absent:
   interface `wlp1s0f0`, source `192.168.4.1/32`, destination `192.168.4.2/32`,
   TCP destination port 7447. Do not replace an existing router or listener.
3. Start the ROS runner below. It waits for fresh `ros link=up` before beginning
   the 900-second Echo observation. Do not reconnect or restart during it.
4. Preserve Echo, NetworkManager, association/power-save, ICMP, route, service
   journal, ROS link and router configuration/log evidence. A ROS link-down is
   diagnostic evidence, not automatically proof of a motor or firmware fault.
5. The runner stops its router after recording the interval. If a fresh Echo
   remains unreachable, rejoin once and record a separate ten-second Echo
   recovery observation. This recovery phase has no ROS router.
6. Stop capture, remove only a firewall rule added by this batch, restore the
   prior Wi-Fi and verify the internet route and prior firewall state. Preserve
   failure evidence; do not automatically repeat the full observation.

```sh
ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=/tmp/live-car.log \
  nix-shell --run 'bash firmware/bench/ros_wifi_soak.sh wlp1s0f0 900 /tmp/rovi-ros-wifi-soak'
```

The output directory must be new. The [underlying recorder](WIFI-SOAK.md)
documents sample correlation, snapshot pauses, exit codes and journal limits.
The runner additionally saves `ros-link.json`, `router.log` and the exact
`router.json5`. Serial must show no unexpected boot, ownership, Armed/nonzero
state, stop, product fault or dropped event in the observation interval.
Inspect router/link transitions and logging coverage separately from Echo loss.

## Laptop validation

The runner passes against the real ROS-enabled product simulator over loopback.
A separate injected UDP claim makes its serial guard fail without arming the
simulator. CI exercises the positive loopback runner; the existing seven Echo
recorder tests cover matching, timeouts, loss/recovery and read-only snapshots.
The Nix shell resolves `iw` and its power-save query runs on the laptop. These
checks establish the harness, not radio behavior or an issue fix.
