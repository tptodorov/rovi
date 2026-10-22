## Purpose

Lets the car join a home Wi-Fi network as a ROS 2 participant when one is configured, fall back to its own access point otherwise, and stay overridable by hand over BLE.

## ADDED Requirements

### Requirement: Mode selection at boot

At boot the car SHALL attempt home mode when the firmware was built with home settings and ROS 2, and SHALL otherwise enter fallback mode at once. Home mode SHALL be entered only when the station has joined the configured network, holds a DHCP address and has a ROS router session. Any failed step SHALL enter fallback mode. The AP and the station SHALL never be up at the same time.

#### Scenario: No home settings

- **GIVEN** firmware built without home settings
- **WHEN** the car boots
- **THEN** it enters fallback mode without scanning, and behaves as before this change

#### Scenario: Home network and router available

- **GIVEN** built-in settings for a network that is in range and a reachable ROS router
- **WHEN** the car boots
- **THEN** it enters home mode, the car AP is never started, and a ROS 2 node on the home network can claim and drive the car as it does on the AP

#### Scenario: Router unreachable

- **GIVEN** built-in settings, the network in range, and no ROS router at the configured endpoint
- **WHEN** the join budget ends
- **THEN** the station is stopped and the car enters fallback mode

### Requirement: Join budget

The car SHALL fall back when the configured network is not found by a scan lasting at most 1500 ms. Once the network is found, joining, DHCP and the ROS router session together SHALL complete within 5000 ms or the car SHALL fall back. Each step's outcome and duration SHALL be logged.

#### Scenario: Network not in range

- **GIVEN** built-in settings and the network out of range
- **WHEN** the car boots
- **THEN** the car AP is up within 2500 ms of the scan starting

#### Scenario: Wrong passphrase

- **GIVEN** built-in settings with a wrong passphrase
- **WHEN** the car boots
- **THEN** the join step is logged as failed and the car enters fallback mode within 5000 ms of the network being found

### Requirement: Fallback is sticky

Fallback mode SHALL last until reboot.

#### Scenario: Home network appears later

- **GIVEN** the car is in fallback mode
- **WHEN** the home network and router become available
- **THEN** the car stays in fallback mode until reboot

### Requirement: BLE override in home mode

In home mode the car SHALL advertise BLE whenever no BLE controller is connected. A BLE connection SHALL take ownership as defined in car-control-api; losing that ownership SHALL end the connection, as today. When the connection ends for any reason, the car SHALL start a new home-mode attempt with the boot join budget and SHALL fall back if it fails.

#### Scenario: Hand-back to a streaming ROS controller

- **GIVEN** home mode, a BLE owner that took over from a ROS node, and that node still streaming setpoints
- **WHEN** the BLE controller disarms and disconnects
- **THEN** the car rejoins the home network and router, and the node's stream claims and arms the car without operator action

#### Scenario: Accidental BLE loss

- **GIVEN** home mode and an armed BLE owner
- **WHEN** the BLE link drops without a disarm
- **THEN** the car raises an owner-lost safety stop, then starts a home-mode attempt as for a deliberate disconnect

### Requirement: Home link loss

In home mode, loss of the station link SHALL stop the car as the lease rules require, then run a new home-mode attempt with the boot join budget (scan, join, DHCP and router); failure SHALL enter fallback mode. Loss of only the ROS router session SHALL use the existing router reconnect and SHALL NOT change the mode.

#### Scenario: Car drives out of Wi-Fi range

- **GIVEN** home mode and an armed ROS owner
- **WHEN** the station link is lost and not regained within 5000 ms
- **THEN** the lease stop has zeroed the wheels and the car enters fallback mode with its AP up

#### Scenario: Router restarts

- **GIVEN** home mode with the station joined
- **WHEN** the ROS router stops and later restarts
- **THEN** the car stays in home mode and the ROS session reconnects as it does on the AP

### Requirement: Build-time home settings

Home settings SHALL be supplied only at build time: SSID (1–32 bytes), WPA2 passphrase (8–63 ASCII) and ROS router endpoint (`tcp/<IPv4>:<port>`), all three or none. An incomplete or invalid set SHALL fail the build. The passphrase SHALL never be sent or reported over any interface or log.

#### Scenario: Partial settings fail the build

- **GIVEN** a build with an SSID but no passphrase, or a malformed router endpoint
- **WHEN** the firmware is built
- **THEN** the build fails with a message naming the missing or invalid value

#### Scenario: Passphrase is not disclosed

- **GIVEN** firmware built with home settings
- **WHEN** any controller reads every readable characteristic, Status, ROS topic and the car log
- **THEN** no output contains the passphrase

### Requirement: Mode events

The car log SHALL report each mode change (home or fallback, with the reason for fallback) and each home-mode step (scan, join, DHCP, router) with its result and duration in milliseconds, following the logging standard.

#### Scenario: Boot into fallback is explained

- **GIVEN** built-in settings and the network out of range
- **WHEN** the car boots
- **THEN** the log shows the failed scan with its duration and a fallback mode event naming the scan as the reason
