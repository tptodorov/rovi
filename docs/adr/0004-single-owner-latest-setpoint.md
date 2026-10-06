---
status: accepted
---

# One owner, latest setpoint wins, with no shared command queue

M3 fed BLE and Wi-Fi motion into one shared FIFO, as `VISION.md` described. Every system we reviewed (ExpressLRS/CRSF, MAVLink, PX4 Offboard, ROS 2 `cmd_vel` with `twist_mux` and `mecanum_drive_controller`) instead streams a full setpoint, lets one controller drive at a time, and requires explicit arming ([research](../research/rc-car-command-communication-best-practices.md)). We decided (2026-10-06, M8):

* **One owner.** The first controller to claim the device on BLE, UDP or ROS 2 owns it, and the other radio shuts down until ownership is released.
* **Latest wins, no queue.** The owner streams setpoints, and each valid one renews the lease.
* **Explicit arming.** Motion needs an explicit arm, and every safety stop disarms.
* **ROS-shaped setpoint.** The setpoint is a `geometry_msgs/TwistStamped`-shaped body velocity in SI units. On every transport it is carried as its CDR encoding, so it matches `mecanum_drive_controller`'s input and kinematics.

This supersedes M3's shared-queue design.

## Considered options

* **Keep M3's shared FIFO with cross-source arbitration.** Rejected. It interleaves two operators' motion, and queued motion is stale by definition.
* **Priority mux of simultaneous sources (`twist_mux` style).** Rejected for now. It is more complex than one owner and not needed for a single-operator car.

## Consequences

* While one radio owns the device, the other is off, so concurrent BLE and Wi-Fi traffic (M3's coexistence case) no longer happens in normal use.
* There is one CDR codec for BLE, UDP and ROS 2, so BLE writes need a negotiated ATT MTU of at least 75.
