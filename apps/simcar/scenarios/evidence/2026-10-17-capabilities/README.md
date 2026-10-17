# O9: ROS capabilities publication (2026-10-17)

Laptop evidence only; no board access or motor run. Client: pinned zenoh-nostd `761f03c`.
Router/subscriber: Jazzy `rmw_zenoh_cpp` package `0.2.10-1noble.20260902.013525` in `rovi-ros-sim`.

## Retained-history probe

Source commit `1d7b7f4` adds an ordinary zenoh publisher advertising reliable/transient-local/depth-1
ROS QoS. It publishes at startup and again 12 seconds later. The QoS token was checked against
a real Jazzy transient-local publisher (`:1:,1:,:,:,,`).

After the initial publication, [history_probe.py](history_probe.py) creates a late subscriber
with matching QoS. Its assertions verify publisher QoS, no retained message for 3 seconds,
and receipt of the later live control from the same publisher. Results:

* [history-sim.log](history-sim.log): publications at 1 ms and 12,001 ms.
* [history-probe.log](history-probe.log): no history within 3 s; the live control arrived 9.589 s after subscribing.

The pinned ordinary publisher does not serve ROS retained history. Implementing a custom ROS
publication cache/queryable was not attempted; the approved specification provides the volatile fallback.

## Selected fallback

Reliable, volatile, keep-last depth 1. Publish every 900 ms (headroom under the one-second
requirement), on observed claim/release changes, and when a new router connection is declared.
Both adapters use the same typed values, CDR encoder and publication schedule.

[fallback-probe.log](fallback-probe.log) and [fallback-sim.log](fallback-sim.log) record
`ros-capabilities.sh` against the product sim car, before and after router restart:

| Check | Before restart | After restart |
| --- | --- | --- |
| Late subscriber receives capabilities | 0.413 s | 0.551 s |
| Longest observed periodic gap | 0.907 s | 0.906 s |
| Publication after claim | 0.010 s | 0.010 s |
| Publication after release | 0.010 s | 0.010 s |

The script asserts all nine values equal the UDP HELLO_ACK exactly, publisher QoS matches
the selected mode, and reading capabilities causes no claim or arming. Ownership checks use
HELLO/BYE only. It runs in CI. Repeat from the repository root after a ROS-enabled sim build:

```sh
apps/simcar/scenarios/ros-capabilities.sh
```

This closes O9 for the chosen fallback; it does not establish ESP32 radio delivery or product acceptance.
