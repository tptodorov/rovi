# ROS stop and recovery board batch

Run only after announcing this batch and Todor's go. Rover on its stand, all
wheels clear, motor bank connected and awake, USB connected, hand near the motor
power cable. Use the already installed ROS firmware from the read-only batch;
no flash is needed. This batch has six gentle forward runs at `vx=0.1`, about
42% requested duty, each lasting approximately three seconds.

The [probe](ros_stop_checks.py), in one ROS node with cmd_vel and arm publishers:

1. Five-second lead-in, valid first sample claims/auto-arms, forward three seconds.
   Publish ARM false (STOP). ARM true without fresh input must report not-ready.
   Stream valid commands for 1.5 seconds: wheels must stay still. Explicit ARM,
   forward three seconds, STOP; wait for release.
2. Five-second lead-in, auto-arm and forward three seconds, then go silent.
   Lease expiry should stop all wheels about one second later. Stay silent another
   1.5 seconds, then resume commands: ROS auto-arms, drives three seconds, then
   explicit STOP/release. This permitted auto-recovery differs from STOP/limits.
3. Five-second lead-in, auto-arm and forward three seconds. Send `vx=2.0`, above
   the 1.0 limit: all wheels must stop. Repeat the same inhibition/recovery checks.

Each stop/release needs fresh serial evidence and matching ROS Status. The STOP/limits pause
rejects any Armed state in serial or Status; the lease pause stays silent until
the announced automatic recovery. A second ROS node/intruder and router
loss while driving are separate future checks. Observer/client timings do not
establish the lease+50 ms device-detection bound or pin-level disable latency.

## Execution and cleanup

Start a live serial capture in a new dated directory. Reuse only the Wi-Fi and
scoped firewall setup from steps 2–3 of [ROS-READONLY.md](ROS-READONLY.md), preserving
the prior connection and Ethernet route. The motor-power setup above supersedes
the disconnected setup of that earlier batch. Confirm no other publisher is
running, then execute from the repository root:

```sh
ROVI_CARLOG=<live-car.log> nix-shell --run 'bash firmware/bench/ros_stops.sh'
```

The [runner](ros_stops.sh) refuses an existing container/router listener, starts
its own router, waits for a fresh rover link and invokes the probe. On failure,
the probe attempts STOP and removes its publishers; the runner removes its router,
so firmware lease/owner-loss handling also applies. Always remove only the firewall
rule added for the batch, restore prior Wi-Fi and stop serial capture. Record logs
and Todor's observations separately; assertions alone do not prove wheel behavior.

CI executes the identical runner on the product ROS simulator, using loopback
ports 17450/17797 and its stdout as the serial log. No hardware is involved there.
