# iphone-network-controls Specification

## Purpose
Allow the existing iPhone Bluefy controller to show the car's network state and request Home Wi-Fi or Car network without a laptop, using the existing BLE protocol.

## Requirements

### Requirement: Reported network information

The client SHALL show decoded network mode, pending request, fallback reason and available home IPv4 from the Network characteristic. Disconnected information SHALL be labelled last-reported. A zero address SHALL be shown as unavailable. Missing or malformed optional Network data SHALL disable network actions without preventing ordinary control on older firmware.

#### Scenario: Network changes while connected

- **WHEN** the client receives a valid Network notification
- **THEN** the displayed mode, pending request, reason and address update without changing ownership or arming

#### Scenario: Older firmware

- **WHEN** Network is absent
- **THEN** the client explains network information is unavailable, disables network actions and retains ordinary disarmed connection/control

### Requirement: Disarmed network request

While connected, active and disarmed with supported network information, the client SHALL offer Home Wi-Fi and Car network actions. It SHALL ignore armed, busy or invalid requests locally. A request SHALL send no ARM or nonzero setpoint and SHALL use the existing serialized control deadline and cleanup rules.

#### Scenario: Armed controls

- **GIVEN** the client or fresh reported Status is armed
- **WHEN** the user attempts a network action
- **THEN** no mode command is sent and the UI keeps network actions disabled

#### Scenario: Valid request

- **WHEN** a disarmed user requests either network
- **THEN** the client sends zero and the corresponding acknowledged Control code, then reads fresh Status and Network

### Requirement: Confirmation and release

The client SHALL confirm a network request only after fresh Status reports no error and disarmed state, and fresh Network reports the matching pending request. It SHALL then disconnect through normal cleanup and tell the user to reconnect to verify the resulting network. It SHALL distinguish refusal from failed or unconfirmed work and SHALL ignore old-session completions.

#### Scenario: Confirmed Home or Car network

- **WHEN** fresh reads confirm the requested pending mode
- **THEN** Bluetooth is disconnected without arming and the UI explains that the car applies the request after release

#### Scenario: Refused command

- **WHEN** fresh Status refuses a mode command while disarmed
- **THEN** the client explains the refusal and remains connected/disarmed

#### Scenario: Unconfirmed or stale operation

- **WHEN** confirmation stalls, fails, disagrees with the request or belongs to an ended session
- **THEN** no success or arming is reported, and active failed work closes using existing bounded cleanup
