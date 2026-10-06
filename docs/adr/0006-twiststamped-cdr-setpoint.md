---
status: accepted
---

# Setpoints are CDR-encoded TwistStamped on every transport

The car must be drivable both by direct clients (BLE, UDP) and by standard ROS 2 tooling. ros2_controllers' `mecanum_drive_controller` takes a `geometry_msgs/TwistStamped` body velocity: `linear.x`/`linear.y` in m/s and `angular.z` in rad/s, in `base_link`. We decided (2026-10-06, M8):

* **One message, one encoding.** A Rovi setpoint *is* that message. It is carried as its CDR encoding on BLE, UDP and ROS 2 alike, behind a small per-transport header (sequence, session).
* **The same kinematics.** The car applies `mecanum_drive_controller`'s inverse kinematics, with the same parameter names (`wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis`, `base_frame_offset`) and wheel order (FL, FR, RR, RL).

So the same input gives the same wheel speeds whether it comes from an iPhone or a ROS 2 graph, and one host-tested codec serves all three transports.

## Considered options

* **A compact custom setpoint** (e.g. 3 × f32, normalised −1…1). Rejected: it needs translation for ROS 2 and a second codec. A compact 16-byte BLE fallback stays available only if ATT MTU negotiation proves unreliable (PLAN Q18).
* **Four wheel velocities** (the `ros2_control` hardware-interface shape). Deferred: it moves kinematics to the client and needs encoders for the wheel state (PLAN Q11).

## Consequences

* BLE setpoint writes are 72 bytes, so they need a negotiated ATT MTU of at least 75.
* The header stamp is carried but not trusted for age without clock sync (PLAN Q17).
* Speeds are in SI units, so `max_wheel_speed` is a placeholder until M5 calibrates it.
