---
status: accepted
---

# The car reaches the ROS 2 router over TCP, not UDP

M8 connected the car to `rmw_zenohd` over UDP (`udp/192.168.4.2:7448`), which also needed an extra UDP listener in the router config. On 2026-10-14 we found that after the router stalls for longer than the car's lease (a frozen process, a suspended laptop), the router's UDP listener stops accepting every peer: the car never rejoined, and neither did a fresh client ([zenoh#2849](https://github.com/eclipse-zenoh/zenoh/issues/2849); evidence in `experiments/rovi-m8-setpoint-ros/bench/`). Over TCP the car rejoins 2.0 s after the router resumes, on the sim and on the board, with a 10 s and with the stock 60 s router lease. We decided (2026-10-14):

* **The car's default router endpoint is `tcp/192.168.4.2:7447`**, which a stock `rmw_zenohd` listens on with no config change.
* UDP stays available through `ROVI_ZENOH_ROUTER` at build time, for tests and in case upstream fixes the listener.

## Considered options

* **Stay on UDP and work around it on the car.** Rejected: the router stops answering everyone, so nothing the car does can recover; only a router restart helps.
* **Stay on UDP and wait for an upstream fix.** Rejected as the default: the bug is on `main`, and every ROS user's router would need the fix.

## Consequences

* The board's ROS round trip went from median 7.7 to 8.0 ms (p95 about 11.5 ms) over UDP to median 9.1 ms (p95 14.3 ms) over TCP.
* A lost TCP segment delays the setpoints queued behind it (head-of-line blocking), so a burst of slightly old setpoints can arrive at once. Latest-wins and the 1 s lease bound the effect; staleness is still judged by receive time (Q17).
* A dead router is still noticed after the router's lease (`rmw_zenohd` default 60 s, 10 s with `lease: 10000` in the router config).
* The router config no longer needs a UDP listener for the car.
