# Read-only ROS board batch

Run only after an announced test and Todor's go. Disconnect motor power; leave USB
connected. This batch flashes the product ROS variant and checks delivery before
any ROS motion tests. Existing non-ROS firmware remains available for rollback.

1. Flash `firmware/target/ros/xtensa-esp32s3-none-elf/release/rovi-firmware` and capture
   boot/serial output in a new dated evidence directory. The build's router endpoint
   must be `tcp/192.168.4.2:7447`.
2. Join the existing `rovi-m8` Wi-Fi profile: laptop `192.168.4.2/24`, rover
   `192.168.4.1`. Preserve the Ethernet internet route and remember the prior Wi-Fi
   connection for restoration. This profile disables IPv6 and has no default IPv4 route.
3. Permit only the rover's router connection for this run. Record whether this exact
   rule already exists; insert it only if absent and remove it afterward only if added:

   ```sh
   sudo -n iptables -I nixos-fw 1 -i wlp1s0f0 -s 192.168.4.1/32 -d 192.168.4.2/32 \
     -p tcp --dport 7447 -m comment --comment rovi-ros-bench -j nixos-fw-accept
   ```

4. With the serial capture running, execute from the repository root:

   ```sh
   ROVI_TARGET=192.168.4.1:7777 ROVI_CARLOG=<live-car.log> \
     nix-shell --run 'bash firmware/bench/ros_readonly.sh'
   ```

The script owns a dedicated Docker router and removes it on exit. It reuses the
product `ros_capabilities.py` and `ros_status.py` probes before/after one router
restart: late receipt/heartbeat within one second, UDP/ROS capability parity,
claim/release updates, persistent errors and cleared open status. It checks fresh
serial link-up events and rejects any Armed state. Zero setpoints and refused ARM
commands exercise UDP status; no successful ARM or `/cmd_vel` publication is sent.

Always remove the firewall rule added for the run (same specification with `-D
nixos-fw` instead of `-I nixos-fw 1`), restore the prior Wi-Fi connection and stop
serial capture, including on failure. Record which firmware remains installed.
This batch does not establish motor output, ROS auto-arm, stop/recovery under
motion, owner loss while driving or numeric stop latency.

CI runs the identical script against the product simulator using loopback ports
17449/17796. Local results are preparation; actual board delivery remains separate.
