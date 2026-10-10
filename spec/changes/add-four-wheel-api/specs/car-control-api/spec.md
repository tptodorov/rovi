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

The car SHALL convert a setpoint to four wheel commands with `mecanum_drive_controller`'s inverse kinematics (Jazzy) and parameter names, divide each wheel speed by `max_wheel_speed`, and clamp to −1.0..=1.0. **OPEN (O2, M5):** `max_wheel_speed` and the setpoint limits.

#### Scenario: Same result as mecanum_drive_controller

- **GIVEN** a `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis` and `max_wheel_speed`
- **WHEN** the car is armed and applies a setpoint
- **THEN** the wheel commands (FL, FR, RR, RL) equal `mecanum_drive_controller`'s wheel speeds for that setpoint, divided by `max_wheel_speed`

#### Scenario: Same input on every source gives the same wheel commands

- **GIVEN** the same setpoint
- **WHEN** it is sent by an armed owner over BLE, UDP and ROS 2 in turn
- **THEN** the wheel commands are identical

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

A new owner SHALL start disarmed. Only the owner's explicit arm command SHALL arm the car, and only while the lease is valid. While disarmed, setpoints SHALL renew the lease and every wheel command SHALL be zero.

#### Scenario: Setpoints alone do not move the car

- **GIVEN** a new owner that has not armed
- **WHEN** it streams non-zero setpoints
- **THEN** every wheel command is zero

#### Scenario: Arm without a lease is refused

- **GIVEN** a new owner that has sent no setpoint
- **WHEN** it sends arm
- **THEN** arm is refused with not-ready and the car stays disarmed

#### Scenario: Arm then drive

- **GIVEN** an owner with a valid lease
- **WHEN** it sends arm, then a non-zero setpoint
- **THEN** the car is armed and the wheel commands follow the setpoint

### Requirement: Lease and watchdog timeout

Only the owner's valid setpoints SHALL renew the lease. When the lease expires, the car SHALL raise a watchdog timeout and a safety stop within the lease length plus 50 ms (M8's acceptance bound), checked by the control tick independently of incoming traffic.

#### Scenario: Silent armed owner is stopped

- **GIVEN** the car is armed
- **WHEN** no valid setpoint arrives for longer than the lease
- **THEN** a safety stop is raised with reason lease-expired, within the lease plus 50 ms
- **AND** every wheel command is zero

### Requirement: Safety stop

The car SHALL enter a latched safety stop on the owner's stop command, lease expiry, owner loss, or an out-of-limits setpoint. A safety stop SHALL set every wheel command to zero, put every driver in standby and disarm the car. The owner SHALL be kept after every safety stop except owner loss. The latch SHALL clear only when the owner arms again or ownership is released.

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

After a safety stop that keeps the owner, the owner SHALL be able to resume within the reclaim window by sending a valid setpoint and then arming, without a new claim. When the window ends, ownership SHALL be released.

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

The car SHALL give its capabilities to the owner on claim, over BLE (Capabilities characteristic) and UDP (`HELLO_ACK`): protocol version, `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis`, maximum `linear.x`, `linear.y` and `angular.z`, lease length, claim window and reclaim window. Each value SHALL be the one the car enforces, so a controller needs no hardcoded limits.

#### Scenario: Limits match enforcement

- **GIVEN** a controller has read the capabilities
- **WHEN** it sends a setpoint at exactly the maximum `linear.x`
- **THEN** the setpoint is applied
- **AND** a setpoint above it raises a safety stop

### Requirement: Status

The car SHALL report its status to the owner on every source: state (open, claimed, armed or safety stop), the stop reason (stop, lease-expired, owner-lost or out-of-limits) and the last command error (busy, not-owner, not-ready or invalid). Every status read or notification SHALL report the state at that moment.

#### Scenario: Status follows a stop

- **GIVEN** the car is armed and the owner reads status
- **WHEN** the lease expires
- **THEN** the next status reports safety stop with reason lease-expired
