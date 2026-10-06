---
status: accepted
---

# One owner, latest setpoint wins, with no shared command queue

M3 fed BLE and Wi-Fi motion into one shared FIFO, as `VISION.md` described. Every system we reviewed (ExpressLRS/CRSF, MAVLink, PX4 Offboard, ROS 2 `cmd_vel` with `twist_mux`) instead streams a full setpoint, lets one controller drive at a time, and requires explicit arming ([research](../research/rc-car-command-communication-best-practices.md)). We decided (2026-10-06, M8):

* **One owner.** The first controller to claim the device on BLE, UDP or ROS 2 owns it, and the other radio shuts down until ownership is released.
* **Latest wins, no queue.** The owner streams setpoints, and each valid one renews the lease.
* **Explicit arming.** Motion needs an explicit arm, and every safety stop disarms.

This supersedes M3's shared-queue design. The setpoint's format is a separate decision ([ADR-0006](0006-twiststamped-cdr-setpoint.md)).

## Considered options

* **Keep M3's shared FIFO with cross-source arbitration.** Rejected. It interleaves two operators' motion, and queued motion is stale by definition.
* **Priority mux of simultaneous sources (`twist_mux` style).** Rejected for now. It is more complex than one owner and not needed for a single-operator car.

## Consequences

While one radio owns the device, the other is off, so concurrent BLE and Wi-Fi traffic (M3's coexistence case) no longer happens in normal use.
