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
- **WHEN** a BLE controller connects
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
