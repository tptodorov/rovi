# ROS router-loss board batch

Run only after announcing this batch and Todor's go. Same setup as
[ROS-STOPS.md](ROS-STOPS.md): rover on its stand, wheels clear, motor bank awake,
USB connected, installed ROS firmware, no flash. Two gentle forward runs at
`vx=0.1` (about 42% requested duty), three seconds each, five-second lead-ins.

The [runner](ros_router_loss.sh) owns a dedicated Docker router (10 s router
lease) and runs a stock `ros2 topic pub` on `/cmd_vel` inside it, so the
publisher dies with the router and can never resume:

1. **KILLED:** auto-arm, drive three seconds, `docker kill` the router. Requires
   fresh serial `stop reason=OwnerLost`, `released` and `ros link=down`, and no
   lease stop (the car must detect the closed TCP link before its 1 s lease).
2. **FROZEN:** auto-arm, drive three seconds, `docker pause` the router (TCP
   stays open). Requires a lease stop, then `released` and the car's own
   `ros link=down` within 35 s, no OwnerLost; the paused router is then killed.

After each, a new router must bring a fresh link-up with no claim or arming for
three seconds, and `/rovi/status` must read `state=open reason=none error=none session=0`.

## Execution and cleanup

Start a live serial capture in a new dated directory. Reuse the Wi-Fi and scoped
firewall steps 2–3 of [ROS-READONLY.md](ROS-READONLY.md). Confirm no other
publisher or router runs, then from the repository root:

```sh
ROVI_CARLOG=<live-car.log> nix-shell --run 'bash firmware/bench/ros_router_loss.sh'
```

The runner removes its router on exit, including failure. Then remove only the
firewall rule added for the batch, restore the prior Wi-Fi and stop the capture.
Record logs and Todor's observations separately. Host timing does not establish
the lease+50 ms bound or pin-level disable latency.

CI runs the identical runner against the product simulator on loopback ports
17451/17798.
