# ROS lease timing at zero wheels

Run only after the separately announced batch and Todor's go. Use the
instrumented ROS product firmware installed for the
[BLE device-clock batch](2026-10-19-ble-device-lease-timing/notes.md); no flash.
Rover on stand, wheels clear, motor bank awake, USB connected. Watch that all
four wheels stay still, including the reset when serial capture opens.

Start serial capture, join Rovi's AP and add the scoped router firewall rule
as in [ROS-READONLY.md](ROS-READONLY.md). Preserve the previous Wi-Fi connection
and Ethernet route. Confirm no other publisher, router listener or
`rovi-ros-stops` container is running, then:

```sh
ROVI_CARLOG=<live-car.log> bash firmware/bench/ros_stops.sh ros_lease_timing.py
```

Five zero-speed trials on one ROS node, about 30 seconds plus network setup:

1. Discover the car, receive Open Status and verify the advertised 1,000 ms
   lease before sending commands.
2. Publish zero `TwistStamped` samples until a fresh serial Armed event appears,
   then stop publishing. The first sample may be dropped during publisher
   discovery; every attempt is zero and its count is printed. ROS auto-arms
   without an ARM command.
3. Require a fresh serial LeaseExpired stop with integer `lease_late_ms` in
   **0–50 ms**, zero wheel commands and a matching ROS lease-expired Status
   after Armed Status. Reject faults, resets, drops, release, wrong/duplicate
   stops and missing timing. All five trials must keep the same session.
4. Repeat with zero input after each lease expiry; this is ROS's specified
   automatic recovery. All wheel commands remain zero.
5. STOP, destroy the node and require fresh serial release before success.
   Cleanup also runs on assertion failure; the runner removes its router.

The timing decision uses the device clock and the actual accepted setpoint's
deadline, as documented in [LOGGING.md](../../docs/LOGGING.md). The required
lateness remains at most 50 ms. `host_upper_bound_ms` spans the first zero
publish attempt through serial observation and includes discovery retries,
transport, serial and laptop delay; it is a separate diagnostic. It is not
substituted for device timing, and timestamps from different clocks are not
subtracted. This does not measure STBY propagation or wheel stopping time.

Remove only the firewall rule added for this batch, restore the prior Wi-Fi
connection and stop capture. Record firmware/probe identity, serial log, probe
output, cleanup and Todor's wheel observation in dated bench notes.

The full runner passed against the product ROS simulator on loopback ports
17453/17800 and runs identically in CI. Eight serial-evidence unit tests reject
invalid or stale/misordered evidence and pin the 50/51 ms boundary. These are
development checks. A local simulator fault injection also verified that a
51 ms value is rejected and failure cleanup releases ownership. No ROS board
timing result is claimed until the announced hardware batch runs.
