## MODIFIED Requirements

### Requirement: Single owner

The first controller to claim SHALL own the car; other claims SHALL be refused and their input ignored, except that UDP SHALL preempt a ROS owner, and in home mode BLE SHALL preempt a UDP or ROS owner. Preemption SHALL raise an owner-lost stop and leave the new owner disarmed. Repeated UDP HELLO from the owner SHALL return its session. BLE ownership SHALL turn off the AP or station. In fallback mode UDP or ROS ownership SHALL stop BLE advertising.

#### Scenario: Second claimant is refused

- **GIVEN** a UDP controller owns the car
- **WHEN** a second UDP controller sends a hello
- **THEN** it receives `BUSY`
- **AND** the owner is unchanged

#### Scenario: Other radio is off while owned

- **GIVEN** the car is open in fallback mode
- **WHEN** a BLE controller connects and writes its first setpoint or control command
- **THEN** it becomes the owner and the car AP goes off
- **AND** when ownership is released with no Home command pending, BLE advertising and the car AP are both up

#### Scenario: Non-owner ROS node is ignored

- **GIVEN** a ROS 2 node owns the car
- **WHEN** another node publishes a setpoint or an arm
- **THEN** it is ignored

#### Scenario: UDP and ROS share the Wi-Fi radio

- **GIVEN** a UDP owner and a connected ROS router on the same Wi-Fi network
- **WHEN** a ROS node sends a setpoint
- **THEN** the Wi-Fi stays up, the setpoint cannot claim or drive, and the UDP lease is unchanged
- **AND** the reverse is preemption, not exclusion (next scenario)

#### Scenario: UDP takes the car from ROS

- **GIVEN** an armed ROS owner, in either mode
- **WHEN** a UDP controller sends a hello
- **THEN** the car raises an owner-lost stop, the UDP controller gets a session disarmed, and ROS setpoints are ignored
- **AND** the UDP owner must arm explicitly before any wheel moves

#### Scenario: BLE takes the car from a Wi-Fi owner in home mode

- **GIVEN** home mode and an armed ROS or UDP owner
- **WHEN** a BLE controller connects
- **THEN** the car raises an owner-lost stop, the BLE controller becomes the owner disarmed, and the station goes off
- **AND** the BLE owner must arm explicitly before any wheel moves

#### Scenario: UDP cannot take the car from BLE

- **GIVEN** home mode and a BLE owner
- **WHEN** a UDP controller sends a hello
- **THEN** no UDP reply is possible because the station is off, and the BLE owner is unchanged

#### Scenario: No BLE preemption in fallback mode

- **GIVEN** fallback mode and a UDP or ROS owner
- **WHEN** a BLE central scans
- **THEN** the car is not advertising, as before this change

### Requirement: Explicit arming

New owners SHALL start disarmed. BLE/UDP SHALL require the owner's explicit arm and a valid lease. ROS SHALL auto-arm on an owner's valid setpoint only after a claim or lease expiry ([ADR-0011](../../../docs/adr/0011-ros-auto-arm.md)), and not while the hand-back latch of network-mode is set. After STOP or out-of-limits, all sources SHALL require a fresh setpoint followed by explicit arm. Disarmed setpoints SHALL renew the lease while wheels stay zero.

#### Scenario: Setpoints alone do not move the car

- **GIVEN** a new BLE or UDP owner that has not armed
- **WHEN** it streams non-zero setpoints
- **THEN** every wheel command is zero

#### Scenario: Arm without a lease is refused

- **GIVEN** a new BLE or UDP owner that has sent no setpoint
- **WHEN** it sends arm
- **THEN** arm is refused with not-ready and the car stays disarmed

#### Scenario: A ROS 2 stream arms the car

- **GIVEN** the car is open and the hand-back latch is clear
- **WHEN** a known ROS 2 node streams valid non-zero setpoints on `/cmd_vel` and nothing on `/rovi/arm`
- **THEN** the first setpoint claims and arms the car and the wheel commands follow it
- **AND** after a lease expiry, the resumed stream arms the car again

#### Scenario: Hand-back latch needs an explicit ROS arm

- **GIVEN** the hand-back latch is set
- **WHEN** a ROS 2 node streams valid non-zero setpoints, then publishes `/rovi/arm` true and keeps streaming
- **THEN** its first setpoint claims the car with every wheel command zero, and the wheels follow only after the arm
- **AND** the latch is then clear

#### Scenario: A ROS 2 stop stays latched while streaming

- **GIVEN** a ROS 2 owner that is streaming and armed
- **WHEN** it publishes `/rovi/arm` false and keeps streaming
- **THEN** the car stops and every wheel command stays zero until the owner publishes `/rovi/arm` true

#### Scenario: Arm then drive

- **GIVEN** an owner with a fresh valid setpoint, a valid lease and valid motor configuration
- **WHEN** it sends arm, then a non-zero setpoint
- **THEN** the car is armed and the wheel commands follow the setpoint

#### Scenario: Arm after a stop needs a new setpoint

- **GIVEN** the owner sent stop while its lease was valid
- **WHEN** it sends arm before any new setpoint
- **THEN** arm is refused with not-ready and the car stays in safety stop
- **AND** after one valid setpoint, the next arm succeeds

### Requirement: Capabilities

The car SHALL give its capabilities on every source, to owners and non-owners, before any claim: protocol version, `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis`, maximum `linear.x`, `linear.y` and `angular.z`, lease length, claim window and reclaim window. BLE: Capabilities characteristic; UDP: `INFO` to any peer and `HELLO_ACK`; ROS 2: as below. Each value SHALL be the one the car enforces.

#### Scenario: Limits match enforcement

- **GIVEN** a controller has read the capabilities
- **WHEN** it sends a setpoint at exactly the maximum `linear.x`
- **THEN** the setpoint is applied
- **AND** a setpoint above it raises a safety stop

#### Scenario: Same capabilities on every source

- **GIVEN** the same car
- **WHEN** a BLE, a UDP and a ROS 2 controller each read the capabilities
- **THEN** every value is identical

#### Scenario: Capabilities before claiming

- **GIVEN** another controller owns the car
- **WHEN** a connected unclaimed BLE central reads Capabilities, or a UDP peer sends `INFO`
- **THEN** it gets the capabilities and the owner, state and deadlines are unchanged

### Requirement: Status

The car SHALL report its status on every source to owners and non-owners (BLE Status characteristic, UDP `STATUS` to the owner and `INFO` to any peer, ROS 2 `/rovi/status`): state (open, claimed, armed or stopped), stop reason (none, stop, lease-expired, owner-lost or out-of-limits) and last owner control error (none, not-ready or invalid). Every status read or notification SHALL report the state at that moment.

#### Scenario: Status follows a stop

- **GIVEN** the car is armed and the owner reads status
- **WHEN** the lease expires
- **THEN** the next status reports safety stop with reason lease-expired

#### Scenario: Readable right after the claim

- **GIVEN** a BLE central has just connected to an open car
- **WHEN** it reads Status before any write, then writes a setpoint and reads Status again
- **THEN** the first read reports open and the second reports claimed with no error

### Requirement: Echo is observational

UDP Echo and the BLE Echo characteristic SHALL return payloads of at most 32 bytes unchanged and discard larger ones. Echo SHALL NOT claim, arm, change wheel commands, renew deadlines, consume setpoint sequences or clear errors. Echo SHALL be available to non-owners: any UDP peer and any connected BLE central, claimed or not.

#### Scenario: Echo does not keep an owner alive

- **GIVEN** an owner stops sending setpoints but continues echo probes
- **WHEN** the lease expires
- **THEN** the normal lease stop occurs and ownership eventually expires without ARM

#### Scenario: Echo does not claim over BLE

- **GIVEN** an open car and a connected BLE central that has written nothing else
- **WHEN** it writes Echo probes
- **THEN** each is returned and the car stays open

## ADDED Requirements

### Requirement: BLE claim on first command

The car SHALL accept up to two BLE connections. A connection SHALL claim on its first Setpoint or Control write, not on connect; reads, notifications and Echo SHALL NOT claim. A non-owner's Setpoint or Control write that cannot claim SHALL be rejected with an ATT error and change nothing. When BLE ownership ends, the car SHALL disconnect that central; other BLE connections stay.

#### Scenario: Watching without claiming

- **GIVEN** a UDP owner driving in home mode
- **WHEN** a BLE central connects, subscribes to Status and Network and reads Capabilities
- **THEN** it receives them, the UDP owner keeps driving and the station stays up

#### Scenario: Second central cannot claim while BLE owns

- **GIVEN** a BLE owner and a second connected BLE central
- **WHEN** the second central writes a setpoint or ARM
- **THEN** the write is rejected with an ATT error and the owner, state and lease are unchanged

#### Scenario: Owner released, watcher stays

- **GIVEN** a BLE owner and a second, unclaimed BLE central
- **WHEN** the owner's claim window expires
- **THEN** the owner is disconnected and the second central stays connected and unclaimed

### Requirement: Observation without ownership

Any client SHALL be able to read capabilities, Status and network information without claiming: a connected BLE central through its characteristics, any UDP peer through one `INFO` request (kind 11) answered with all three, and any ROS node through the topics. Observation SHALL NOT claim, renew deadlines, change state or clear errors.

#### Scenario: UDP INFO from a non-owner

- **GIVEN** a ROS owner and a UDP peer that has never sent HELLO
- **WHEN** the peer sends `INFO`
- **THEN** it receives capabilities, Status (claimed or armed) and network information, and the ROS owner is unchanged

### Requirement: ROS release

A ROS owner node SHALL be able to release the car by publishing on `/rovi/release` (`std_msgs/Empty`), with the same effects as a UDP BYE: an owner-lost stop if armed, release at once, and any pending mode command taking effect. Publications from non-owner nodes SHALL be ignored.

#### Scenario: Release applies a pending mode at once

- **GIVEN** a disarmed ROS owner that published Home on `/rovi/mode` in fallback mode
- **WHEN** it publishes on `/rovi/release`
- **THEN** the car is released and starts a home-mode attempt without waiting for the lease or reclaim window

#### Scenario: Non-owner release is ignored

- **GIVEN** a ROS node owns the car
- **WHEN** another node publishes on `/rovi/release`
- **THEN** the owner and state are unchanged

### Requirement: Takeover notice

When a controller is preempted, the car SHALL first send it a final Status (stopped, owner-lost) on its own transport: a UDP `STATUS` to the old session's endpoint, or `/rovi/status` for ROS. Later input from the preempted session SHALL be ignored; its HELLO SHALL receive `BUSY` while the new owner holds the car.

#### Scenario: UDP owner learns it was overridden

- **GIVEN** home mode and a UDP owner
- **WHEN** a BLE central claims the car
- **THEN** the UDP owner's endpoint receives a `STATUS` with stopped and owner-lost before the station goes off

#### Scenario: ROS owner learns it was overridden

- **GIVEN** a ROS owner
- **WHEN** a UDP controller sends HELLO
- **THEN** `/rovi/status` publishes stopped with owner-lost before the UDP `HELLO_ACK`
