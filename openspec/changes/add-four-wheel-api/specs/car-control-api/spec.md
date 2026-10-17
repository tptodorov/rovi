## Purpose

Defines how controllers request car motion, acquire ownership, arm, stop and inspect capabilities and status consistently across the supported transports.

## ADDED Requirements

### Requirement: Setpoint

The car's device control API SHALL accept a setpoint as a `geometry_msgs/TwistStamped`, using `linear.x` and `linear.y` in m/s and `angular.z` in rad/s, in `base_link`. A setpoint with any other Twist component non-zero SHALL be rejected and not applied. A setpoint whose `linear.x`, `linear.y` or `angular.z` exceeds the limit in the capabilities SHALL raise a safety stop.

#### Scenario: Out-of-plane setpoint is rejected

- **GIVEN** the car is armed
- **WHEN** the owner sends a setpoint with `linear.z` = 0.1
- **THEN** the setpoint is not applied and does not renew the lease
- **AND** no safety stop is raised

#### Scenario: Out-of-limits setpoint stops the car

- **GIVEN** the car is armed
- **WHEN** the owner sends a setpoint with `linear.x` above the capabilities' maximum
- **THEN** a safety stop is raised with reason out-of-limits

### Requirement: Mecanum kinematics

The car SHALL use `mecanum_drive_controller` inverse kinematics and parameter names, divide wheel-speed references by configured `max_wheel_speed`, and clamp to −1.0..=1.0. The current geometry SHALL use radius 0.030 m and centre projection sum 0.1275 m, with zero frame offset. **OPEN (O2):** qualified profile normalization and input limits; the experiment's 40 rad/s and 1/1/3 limits are not measured physical maxima.

#### Scenario: Same result as mecanum_drive_controller

- **GIVEN** a `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis` and `max_wheel_speed`
- **WHEN** the car is armed and applies a setpoint
- **THEN** the wheel commands (FL, FR, RR, RL) equal `mecanum_drive_controller`'s wheel speeds for that setpoint, divided by `max_wheel_speed`

#### Scenario: Same input on every source gives the same wheel commands

- **GIVEN** the same setpoint
- **WHEN** it is sent by an armed owner over BLE, UDP and ROS 2 in turn
- **THEN** the wheel commands are identical

### Requirement: Open-loop motion references

Twist fields SHALL retain their standard m/s and rad/s meaning as requested motion references. Product documentation SHALL describe current open-loop tracking as approximate and link the M5 observations. The car SHALL NOT present command-derived speed as measured velocity or promise velocity accuracy without supporting evidence.

#### Scenario: An estimate is not measured feedback

- **GIVEN** the current motors have no encoders and only short-pulse observations are available
- **WHEN** the product describes speed or publishes status
- **THEN** observed motion estimates are identified as approximate
- **AND** commanded velocity is not presented as measured feedback

### Requirement: Latest setpoint wins

The car SHALL apply only the owner's latest setpoint, with no queue. A setpoint whose sequence is not newer than the last accepted one SHALL NOT be applied.

#### Scenario: Older sequence is ignored

- **GIVEN** the owner's last accepted setpoint has sequence 10
- **WHEN** a setpoint with sequence 9 or 10 arrives
- **THEN** it is not applied and does not renew the lease

### Requirement: Single owner

The first controller to make a claim SHALL become the owner. While an owner exists, every other claim SHALL be refused, input from any other controller SHALL be ignored, and the radios of the other sources SHALL be off. Each claim SHALL start a new session.

#### Scenario: Second claimant is refused

- **GIVEN** a UDP controller owns the car
- **WHEN** a second UDP controller sends a hello
- **THEN** it receives `BUSY`
- **AND** the owner is unchanged

#### Scenario: Other radio is off while owned

- **GIVEN** the car is open
- **WHEN** a BLE controller connects
- **THEN** it becomes the owner and the car AP goes off
- **AND** when ownership is released, BLE advertising and the car AP are both up

#### Scenario: Non-owner ROS node is ignored

- **GIVEN** a ROS 2 node owns the car
- **WHEN** another node publishes a setpoint or an arm
- **THEN** it is ignored

### Requirement: Claim window

If the owner sends no valid setpoint within the claim window, ownership SHALL be released.

#### Scenario: Silent owner loses the car

- **GIVEN** a controller has just claimed the car
- **WHEN** the claim window passes with no valid setpoint from it
- **THEN** ownership is released and every radio reopens

### Requirement: Explicit arming

New owners SHALL start disarmed. BLE/UDP SHALL require the owner's explicit arm and a valid lease. ROS SHALL auto-arm on an owner's valid setpoint only after a claim or lease expiry ([ADR-0011](../../../../../docs/adr/0011-ros-auto-arm.md)). After STOP or out-of-limits, all sources SHALL require a fresh setpoint followed by explicit arm. Disarmed setpoints SHALL renew the lease while wheel commands remain zero.

#### Scenario: Setpoints alone do not move the car

- **GIVEN** a new BLE or UDP owner that has not armed
- **WHEN** it streams non-zero setpoints
- **THEN** every wheel command is zero

#### Scenario: Arm without a lease is refused

- **GIVEN** a new owner that has sent no setpoint
- **WHEN** it sends arm
- **THEN** arm is refused with not-ready and the car stays disarmed

#### Scenario: A ROS 2 stream arms the car

- **GIVEN** the car is open
- **WHEN** a ROS 2 node streams non-zero setpoints on `/cmd_vel` and nothing on `/rovi/arm`
- **THEN** the first setpoint claims and arms the car and the wheel commands follow it
- **AND** after a lease expiry, the resumed stream arms the car again

#### Scenario: A ROS 2 stop stays latched while streaming

- **GIVEN** a ROS 2 owner that is streaming and armed
- **WHEN** it publishes `/rovi/arm` false and keeps streaming
- **THEN** the car stops and every wheel command stays zero until the owner publishes `/rovi/arm` true

#### Scenario: Arm then drive

- **GIVEN** an owner with a valid lease
- **WHEN** it sends arm, then a non-zero setpoint
- **THEN** the car is armed and the wheel commands follow the setpoint

#### Scenario: Arm after a stop needs a new setpoint

- **GIVEN** the owner sent stop while its lease was valid
- **WHEN** it sends arm before any new setpoint
- **THEN** arm is refused with not-ready and the car stays in safety stop
- **AND** after one valid setpoint, the next arm succeeds

### Requirement: Lease and watchdog timeout

Only the owner's valid setpoints SHALL renew the lease. When the lease expires, the car SHALL raise a watchdog timeout and a safety stop within the lease length plus 50 ms (M8's acceptance bound), checked by the control tick independently of incoming traffic.

#### Scenario: Silent armed owner is stopped

- **GIVEN** the car is armed
- **WHEN** no valid setpoint arrives for longer than the lease
- **THEN** a safety stop is raised with reason lease-expired, within the lease plus 50 ms
- **AND** every wheel command is zero

### Requirement: Safety stop

The car SHALL enter a latched safety stop on the owner's stop command, lease expiry, owner loss, or an out-of-limits setpoint. A safety stop SHALL set every wheel command to zero, put every driver in standby and disarm the car. The owner SHALL be kept after every safety stop except owner loss, as the reclaim window requirement states. The latch SHALL clear only when the owner arms again or ownership is released.

#### Scenario: Stop keeps the owner

- **GIVEN** the car is armed
- **WHEN** the owner sends stop
- **THEN** the car is in safety stop with reason stop, disarmed, with every driver in standby
- **AND** the controller is still the owner

#### Scenario: Owner loss while armed

- **GIVEN** the car is armed
- **WHEN** the owner disconnects or sends bye
- **THEN** a safety stop is raised with reason owner-lost
- **AND** ownership is released

#### Scenario: Further setpoints do not clear the latch

- **GIVEN** the car is in safety stop
- **WHEN** the owner streams valid setpoints without arming
- **THEN** every wheel command stays zero

### Requirement: Reclaim window

After a stop or an out-of-limits setpoint, the owner SHALL be kept for as long as its valid setpoints keep the lease valid. The reclaim window SHALL start when the lease lapses: at once on a lease expiry, or when the owner goes quiet after a stop or an out-of-limits setpoint. Within the reclaim window, the owner SHALL be able to resume by sending a valid setpoint and then arming, without a new claim. When the window ends, ownership SHALL be released.

#### Scenario: Streaming owner is kept after a stop

- **GIVEN** the owner sent stop
- **WHEN** it keeps streaming valid setpoints for longer than the reclaim window
- **THEN** it is still the owner and the car stays in safety stop

#### Scenario: Quiet owner after a stop is released

- **GIVEN** the owner sent stop
- **WHEN** it sends no setpoint after that
- **THEN** the reclaim window starts when its lease lapses
- **AND** ownership is released when the window ends, and every radio reopens

#### Scenario: Out-of-limits keeps a streaming owner

- **GIVEN** an out-of-limits setpoint raised a safety stop
- **WHEN** the owner keeps streaming valid setpoints and then sends arm
- **THEN** the car is armed in the same session

#### Scenario: Resume within the window

- **GIVEN** the car stopped on lease expiry
- **WHEN** the same owner sends a valid setpoint and arm within the reclaim window
- **THEN** the car is armed in the same session

#### Scenario: Window ends

- **GIVEN** the car stopped on lease expiry
- **WHEN** the reclaim window passes without arm
- **THEN** ownership is released and every radio reopens

### Requirement: Boot state

After power-up or reset, the car SHALL be open, with no owner, disarmed, every wheel command zero and every driver in standby.

#### Scenario: Fresh boot

- **GIVEN** the car has just booted
- **WHEN** a controller reads the status
- **THEN** the state is open
- **AND** the car log reports every wheel command as zero

### Requirement: Capabilities

The car SHALL give its capabilities on every source: protocol version, `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis`, maximum `linear.x`, `linear.y` and `angular.z`, lease length, claim window and reclaim window. BLE (Capabilities characteristic) and UDP (`HELLO_ACK`) give them to the owner on claim; ROS 2 gives them before the claim, as below. Each value SHALL be the one the car enforces, so a controller needs no hardcoded limits.

#### Scenario: Limits match enforcement

- **GIVEN** a controller has read the capabilities
- **WHEN** it sends a setpoint at exactly the maximum `linear.x`
- **THEN** the setpoint is applied
- **AND** a setpoint above it raises a safety stop

#### Scenario: Same capabilities on every source

- **GIVEN** the same car
- **WHEN** a BLE, a UDP and a ROS 2 controller each read the capabilities
- **THEN** every value is identical

### Requirement: ROS 2 capabilities topic

A ROS 2 controller claims with its first setpoint, so ROS 2 SHALL give the capabilities without a claim. The car SHALL publish them on `/rovi/capabilities` as `std_msgs/String` of space-separated `key=value` pairs, latched: reliable, transient local, depth 1. Whether zenoh-nostd can serve the latching is OPEN (O9); the fallback below applies if it cannot.

#### Scenario: Late subscriber gets the capabilities

- **GIVEN** the car is open and connected to the ROS 2 router
- **WHEN** a node starts and subscribes to `/rovi/capabilities`
- **THEN** it receives the capabilities without sending anything to the car

#### Scenario: One pair per field

- **GIVEN** a node has received `/rovi/capabilities`
- **WHEN** it splits the string on spaces and `=`
- **THEN** it gets one value for each capabilities field, keyed by the field name above (for example `wheels_radius`)

#### Scenario: Reading the capabilities is not a claim

- **GIVEN** the car is open
- **WHEN** a ROS 2 node reads `/rovi/capabilities`
- **THEN** the car stays open and every radio stays up

### Requirement: ROS 2 capabilities republish fallback

If O9 shows that zenoh-nostd cannot serve transient-local late joiners, the car SHALL instead republish `/rovi/capabilities` periodically and on every claim and release. **OPEN (O9):** the period.

#### Scenario: Late subscriber gets the capabilities without latching

- **GIVEN** the car uses the republish fallback
- **WHEN** a node subscribes to `/rovi/capabilities` while the car is open
- **THEN** it receives the capabilities within one republish period

### Requirement: Status

The car SHALL report its status to the owner on every source (BLE Status characteristic, UDP `STATUS`, ROS 2 `/rovi/status`): state (open, claimed, armed or safety stop), the stop reason (stop, lease-expired, owner-lost or out-of-limits) and the last command error (busy, not-owner, not-ready or invalid). Status SHALL be readable at once after a claim. Every status read or notification SHALL report the state at that moment.

#### Scenario: Status follows a stop

- **GIVEN** the car is armed and the owner reads status
- **WHEN** the lease expires
- **THEN** the next status reports safety stop with reason lease-expired

#### Scenario: Readable right after the claim

- **GIVEN** a BLE controller has just connected
- **WHEN** it reads Status before any other write
- **THEN** the status reports claimed with no error

### Requirement: ROS 2 status topic

The car SHALL publish `/rovi/status` as `std_msgs/String` of space-separated `key=value` pairs with the same content as the other sources' status: `state` (open, claimed, armed or stopped), `reason` (the stop reason, while stopped), `error` (the last command error, if any) and `session`. It SHALL publish on every change of any of these and at least once a second, so a late subscriber sees the current status within a second.

#### Scenario: Stop reason on ROS 2

- **GIVEN** the car is armed under a ROS 2 owner
- **WHEN** the lease expires
- **THEN** the next `/rovi/status` has `state=stopped` and `reason=lease-expired`

#### Scenario: Refused arm on ROS 2

- **GIVEN** a ROS 2 owner stopped by its own STOP
- **WHEN** it publishes arm before any new setpoint
- **THEN** the next `/rovi/status` has `state=stopped` and `error=not-ready`

#### Scenario: Status on a change

- **GIVEN** a node subscribed to `/rovi/status`
- **WHEN** the state changes
- **THEN** it receives the new status without waiting for the next periodic publish

### Requirement: Last command error

On every source, a refused device command from the owner (not-ready, invalid) SHALL show as the last command error until the owner's next device command succeeds. Setpoints SHALL NOT clear it. Commands from controllers that are not the owner SHALL NOT change the owner's status.

#### Scenario: Refused arm shows on every source

- **GIVEN** a new owner on BLE, UDP or ROS 2 that has sent no setpoint
- **WHEN** it sends arm
- **THEN** the status reports the last command error not-ready
- **AND** a later setpoint does not clear it, and a successful arm does

#### Scenario: Non-owner commands do not change the owner's status

- **GIVEN** an owner whose status reports no error
- **WHEN** another controller sends arm
- **THEN** the owner's status still reports no error
