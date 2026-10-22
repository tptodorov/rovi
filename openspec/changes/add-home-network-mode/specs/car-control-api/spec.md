## MODIFIED Requirements

### Requirement: Single owner

The first controller to claim SHALL become the owner. Other claims SHALL be refused and their input ignored, except that in home mode a BLE connection SHALL preempt a Wi-Fi owner. Repeated UDP HELLO from the owner SHALL return its session without renewing deadlines. BLE ownership SHALL turn off the AP (fallback) or the station (home). In fallback mode UDP or ROS ownership SHALL stop BLE advertising; in home mode it SHALL NOT. UDP and ROS SHALL exclude one another through ownership checks.

#### Scenario: Second claimant is refused

- **GIVEN** a UDP controller owns the car
- **WHEN** a second UDP controller sends a hello
- **THEN** it receives `BUSY`
- **AND** the owner is unchanged

#### Scenario: Other radio is off while owned

- **GIVEN** the car is open in fallback mode
- **WHEN** a BLE controller connects
- **THEN** it becomes the owner and the car AP goes off
- **AND** when ownership is released, BLE advertising and the car AP are both up

#### Scenario: Non-owner ROS node is ignored

- **GIVEN** a ROS 2 node owns the car
- **WHEN** another node publishes a setpoint or an arm
- **THEN** it is ignored

#### Scenario: UDP and ROS share the Wi-Fi radio

- **GIVEN** a UDP owner and a connected ROS router on the car AP
- **WHEN** a ROS node sends a setpoint
- **THEN** the AP stays up, the setpoint cannot claim or drive, and the UDP lease is unchanged
- **AND** the same exclusion applies to UDP input while a ROS node owns the car

#### Scenario: BLE advertises while a ROS node drives in home mode

- **GIVEN** home mode and an armed ROS owner
- **WHEN** a BLE central scans
- **THEN** the car is advertising and connectable

#### Scenario: BLE preempts a Wi-Fi owner in home mode

- **GIVEN** home mode and an armed ROS or UDP owner
- **WHEN** a BLE controller connects
- **THEN** the car raises an owner-lost safety stop, the BLE controller becomes the owner disarmed, and the station goes off
- **AND** the BLE owner must arm explicitly before any wheel moves

#### Scenario: No preemption in fallback mode

- **GIVEN** fallback mode and a UDP or ROS owner
- **WHEN** a BLE central scans
- **THEN** the car is not advertising, as before this change
