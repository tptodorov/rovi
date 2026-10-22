## Purpose

Lets the car join a home Wi-Fi network as a ROS 2 participant when one is configured, fall back to its own access point otherwise, switch between the two from any client without reflashing or touching the car, and stay overridable by hand.

## ADDED Requirements

### Requirement: Mode selection at boot

At boot the car SHALL attempt home mode when the firmware was built with home settings and ROS 2, and SHALL otherwise enter fallback mode at once. Home mode SHALL be entered only when the station has joined the configured network, holds a DHCP address and has a ROS router session. Any failed step SHALL enter fallback mode. The AP and the station SHALL never be up at the same time.

#### Scenario: No home settings

- **GIVEN** firmware built without home settings, or without ROS 2
- **WHEN** the car boots
- **THEN** it enters fallback mode without scanning, and behaves as before this change

#### Scenario: Home network and router available

- **GIVEN** built-in settings for a network that is in range and a reachable ROS router
- **WHEN** the car boots
- **THEN** it enters home mode, the car AP is never started, and a ROS 2 node on the home network can claim and drive the car as it does on the AP

#### Scenario: Router unreachable

- **GIVEN** built-in settings, the network in range, and no ROS router at the configured endpoint
- **WHEN** the join budget ends
- **THEN** the station is stopped and the car enters fallback mode with reason router

### Requirement: Join budget

Every home-mode attempt (at boot, after a mode command, after a BLE override ends, after station loss) SHALL fall back when the network is not found by a scan lasting at most 1500 ms. Once found, joining, DHCP and the ROS router session together SHALL complete within 5000 ms or the car SHALL fall back. Each step's outcome and duration SHALL be logged.

#### Scenario: Network not in range

- **GIVEN** built-in settings and the network out of range
- **WHEN** the car boots
- **THEN** the car AP is up within 2500 ms of the scan starting, with fallback reason scan

#### Scenario: Wrong passphrase

- **GIVEN** built-in settings with a wrong passphrase
- **WHEN** the car boots
- **THEN** the join step is logged as failed and the car enters fallback mode with reason join within 5000 ms of the network being found

### Requirement: Fallback is sticky

Fallback mode SHALL last until reboot or until a Home mode command takes effect. The car SHALL NOT retry the home network on its own.

#### Scenario: Home network appears later

- **GIVEN** the car is in fallback mode and no mode command is pending
- **WHEN** the home network and router become available
- **THEN** the car stays in fallback mode

### Requirement: Mode command

Every transport SHALL accept a mode command, Home or Car network, from the owner while disarmed: BLE Control codes 3 and 4, UDP kind 10 with payload 1 or 2, ROS `/rovi/mode` (`std_msgs/UInt8`, 1 or 2). It SHALL become pending and take effect when that ownership ends. Home SHALL start a home-mode attempt; Car network SHALL enter fallback with reason requested. A later command SHALL replace a pending one.

#### Scenario: Back home from fallback over BLE

- **GIVEN** fallback mode and a disarmed BLE owner
- **WHEN** it writes Home, then disconnects
- **THEN** the car stops the AP and starts a home-mode attempt

#### Scenario: Back home from fallback over UDP

- **GIVEN** fallback mode and a disarmed UDP owner on the car AP
- **WHEN** it sends Home, then BYE
- **THEN** the car stops the AP and starts a home-mode attempt

#### Scenario: Car network while at home

- **GIVEN** home mode and a disarmed owner on any transport
- **WHEN** it sends Car network and its ownership ends
- **THEN** the car stops the station and enters fallback mode with reason requested

#### Scenario: Ownership ends without a command

- **GIVEN** an owner that sent no mode command
- **WHEN** its ownership ends
- **THEN** the mode follows the other requirements here; in fallback it stays fallback

### Requirement: Mode command errors

A mode command SHALL change nothing and SHALL be refused: with not-ready when the owner is armed; with invalid for an unknown value, a wrong payload length, or Home in firmware without home settings or ROS 2. Commands from non-owners SHALL be ignored as other control input is. Errors SHALL be reported as the existing last-command error on every transport.

#### Scenario: Armed owner

- **GIVEN** an armed owner
- **WHEN** it sends a mode command
- **THEN** it is refused with not-ready, the car stays armed and nothing is pending

#### Scenario: Home without settings

- **GIVEN** firmware built without home settings
- **WHEN** a disarmed owner sends Home
- **THEN** it is refused with invalid and the car stays in fallback after release

#### Scenario: Unknown value

- **GIVEN** a disarmed owner
- **WHEN** it sends mode value 0, 3 or a two-byte payload
- **THEN** it is refused with invalid and any pending command is unchanged

#### Scenario: Non-owner

- **GIVEN** a ROS node owns the car
- **WHEN** another node publishes on `/rovi/mode`
- **THEN** it is ignored and nothing is pending

### Requirement: Network information

The car SHALL report network information on every transport, to owners and non-owners: mode (home, fallback, attempting), pending command (none, home, car network), last fallback reason (none, no-settings, scan, join, dhcp, router, link-lost, requested) and station IPv4 (0.0.0.0 if none). BLE: a readable, notifying Network characteristic. UDP: kind 11, answered to any peer. ROS: `/rovi/network`, republished at least every second and on change.

#### Scenario: Finding the car's home address

- **GIVEN** home mode
- **WHEN** a BLE central reads the Network characteristic
- **THEN** it gets mode home and the car's DHCP address, which a UDP client on the home network can then use

#### Scenario: Pending command is visible

- **GIVEN** an owner sent Car network
- **WHEN** any client reads network information before the ownership ends
- **THEN** pending is car network

### Requirement: Hand-back after a manual takeover

When a BLE or UDP owner that preempted a ROS owner loses ownership, ROS auto-arm SHALL be latched off unless that owner sent Home: the next ROS owner SHALL need `/rovi/arm` true once before wheels move. A pending Home SHALL hand back with normal ROS auto-arm. Reboot SHALL clear the latch.

#### Scenario: Deliberate hand-back

- **GIVEN** a BLE owner that took over from a streaming ROS node
- **WHEN** it sends Home, then disconnects
- **THEN** the car rejoins home and the node's stream claims and arms the car without operator action

#### Scenario: Accidental BLE loss

- **GIVEN** an armed BLE owner that took over from a streaming ROS node
- **WHEN** the BLE link drops
- **THEN** the car raises an owner-lost stop and rejoins home; the node's stream claims the car but every wheel command stays zero until it publishes `/rovi/arm` true

#### Scenario: UDP takeover ends without Home

- **GIVEN** a UDP owner that took over from a ROS node
- **WHEN** it sends BYE without a mode command
- **THEN** the next ROS owner needs `/rovi/arm` true before moving

### Requirement: BLE availability in home mode

In home mode the car SHALL advertise BLE whenever no BLE controller is connected. When a BLE owner's connection ends for any reason in home mode, the car SHALL start a home-mode attempt, unless Car network is pending.

#### Scenario: BLE advertises while a Wi-Fi controller drives

- **GIVEN** home mode and an armed ROS or UDP owner
- **WHEN** a BLE central scans
- **THEN** the car is advertising and connectable

### Requirement: Home link loss

In home mode, loss of the station link SHALL stop the car as the lease rules require, then run a home-mode attempt; failure SHALL enter fallback with reason link-lost. Loss of only the ROS router session SHALL use the existing router reconnect and SHALL NOT change the mode.

#### Scenario: Car drives out of Wi-Fi range

- **GIVEN** home mode and an armed ROS owner
- **WHEN** the station link is lost and the attempt fails
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
- **WHEN** any controller reads every characteristic, Status, network information, ROS topic and the car log
- **THEN** no output contains the passphrase

### Requirement: Mode events

The car log SHALL report each mode change (home, fallback with reason, attempting), each accepted or refused mode command, and each home-mode step (scan, join, DHCP, router) with its result and duration in milliseconds, following the logging standard.

#### Scenario: Boot into fallback is explained

- **GIVEN** built-in settings and the network out of range
- **WHEN** the car boots
- **THEN** the log shows the failed scan with its duration and a fallback mode event naming the scan as the reason
