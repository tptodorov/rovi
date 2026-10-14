---
status: accepted
---

# ROS 2 arms on its own; BLE and UDP keep the explicit arm

[ADR-0004](0004-single-owner-latest-setpoint.md) requires an explicit arm for motion. Over ROS 2 the arm (`/rovi/arm`) had to come from the node that streams `/cmd_vel`, so stock tools that publish only `/cmd_vel` (`teleop_twist_keyboard`, Nav2, `ros2 topic pub`) could claim the car but never move it ([#24](https://github.com/tptodorov/rovi/issues/24)). We decided (2026-10-14) that **for ROS 2 only**:

* **A valid setpoint from the owner arms the car** when it is claimed (the first setpoint) or stopped by a lease expiry (the stream paused, then resumed).
* **A STOP (`/rovi/arm` false) and an out-of-limits setpoint stay latched** while the owner keeps streaming. Only `/rovi/arm` true from the owner resumes, or the owner goes quiet until the car is released and a new stream claims it.
* BLE and UDP keep ADR-0004's explicit arm. This supersedes ADR-0004's explicit arming for ROS 2 only.

## Considered options

* **Keep the same-node rule and ship a helper node** that arms and drives. Rejected: stock tools still could not drive.
* **Accept `/rovi/arm` from any node for the current owner.** Rejected: one extra manual step for every ROS user, and it weakens ownership for little gain.
* **An arm service (`std_srvs/SetBool`).** Rejected for now: it needs queryable support in zenoh-nostd.

## Consequences

* Over ROS 2 the car moves as soon as a node streams non-zero setpoints. The lease (1 s), the speed limits and the latched STOP remain the safety controls; there is no separate operator step before motion.
* After a lease expiry (a teleop pause or a Wi-Fi gap over 1 s), the car moves again as soon as the stream resumes.
* Revisit before motors run near people (M4/M5): auto-arm may need to be a build-time or runtime option.
