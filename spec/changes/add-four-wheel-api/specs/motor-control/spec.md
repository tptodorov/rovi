## ADDED Requirements

### Requirement: Four independent wheel commands

Motor control SHALL accept one set of four wheel commands per control tick, in the order FL, FR, RR, RL. Each wheel command SHALL be a signed value in −1.0..=1.0, where the sign is the direction that moves the car forward when positive, and the magnitude is the fraction of the configured maximum. Motor control SHALL apply each wheel command to its own wheel only, with no kinematic mixing.

#### Scenario: Each wheel is driven on its own

- **GIVEN** motor control is enabled
- **WHEN** it receives wheel commands (0.5, 0, 0, 0)
- **THEN** only the FL wheel is driven, forward, at half the configured maximum
- **AND** FR, RR and RL receive a zero wheel command

#### Scenario: Sign sets direction

- **GIVEN** motor control is enabled
- **WHEN** it receives wheel commands (−0.5, −0.5, −0.5, −0.5)
- **THEN** every wheel turns in the direction that moves the car backward

### Requirement: Independent of transport and kinematics

Motor control SHALL expose only wheel commands, enable and standby. It SHALL NOT depend on setpoints, mecanum kinematics, the source or the session.

#### Scenario: Built without the device layer

- **GIVEN** the motor control module
- **WHEN** it is compiled and host-tested on its own
- **THEN** it builds without the setpoint, kinematics, arbiter or transport modules

### Requirement: Invalid wheel commands stop the wheels

Motor control SHALL clamp a finite wheel command outside −1.0..=1.0 to the nearest bound. A non-finite wheel command (NaN or infinity) SHALL put every driver in standby and report a fault, even when the other three are valid.

#### Scenario: Out-of-range value is clamped

- **GIVEN** motor control is enabled
- **WHEN** it receives wheel commands (1.5, 0, 0, 0)
- **THEN** FL is driven as for 1.0

#### Scenario: NaN stops every wheel

- **GIVEN** motor control is enabled and all four wheels are driven
- **WHEN** it receives wheel commands (NaN, 0.2, 0.2, 0.2)
- **THEN** every driver is put in standby
- **AND** a fault is reported

### Requirement: Standby until explicitly enabled

Every motor driver SHALL be in standby from power-up, through reset and firmware recovery, until motor control is explicitly enabled. Disabling motor control SHALL put every driver in standby.

#### Scenario: Boot does not move a wheel

- **GIVEN** the car is powered off with motors connected
- **WHEN** it powers up, resets, or recovers from a reset
- **THEN** every driver's STBY input stays inactive until motor control is enabled
- **AND** no wheel turns

### Requirement: Standby on safety stop within a bound

When the device raises a safety stop, motor control SHALL set every wheel command to zero and put every driver in standby within a measured bound. **OPEN (O4, M4):** the bound, under four-motor load.

#### Scenario: Watchdog timeout puts every driver in standby

- **GIVEN** the car is armed and all four wheels are driven
- **WHEN** a watchdog timeout raises a safety stop
- **THEN** every driver's STBY input is inactive within the O4 bound

### Requirement: Full-scale maps to a measured limit

Wheel command 1.0 SHALL map to a configured maximum that keeps each motor, driver channel and the supply within the limits M4 records, with all four wheels at 1.0 at once. **OPEN (O1, M4):** the current limits and the maximum duty. **OPEN (O3, M5):** whether the maximum is a duty (open-loop) or a wheel speed (closed-loop).

#### Scenario: All wheels at full scale stay in limits

- **GIVEN** the car on the M4 bench with its intended supply
- **WHEN** all four wheels are commanded at 1.0
- **THEN** measured per-channel and combined current stay within the O1 limits
- **AND** the logic rail stays above brownout

### Requirement: Zero command behaviour

A zero wheel command SHALL stop that wheel using one documented method. **OPEN (O5, M4/M5):** short brake or coast.

#### Scenario: Zero stops a turning wheel

- **GIVEN** a wheel driven at 0.5
- **WHEN** it receives a zero wheel command
- **THEN** it is stopped by the documented method

### Requirement: Dead zone handling

Motor control SHALL document the smallest wheel command that turns each wheel. **OPEN (O6, M5):** the value, and whether commands below it are raised to it or left as is.

#### Scenario: Small command behaviour is predictable

- **GIVEN** the O6 dead zone is known
- **WHEN** a wheel receives a non-zero wheel command below it
- **THEN** the wheel behaves as the documented dead-zone rule states

### Requirement: Starts and reversals do not reset the controller

Motor control SHALL change wheel commands in a way that does not reset the ESP32-S3 or drop a control link. **OPEN (O7, M4):** whether a slew limit is needed, and its rate.

#### Scenario: Full reversal on all wheels

- **GIVEN** all four wheels at 1.0
- **WHEN** all four are commanded to −1.0 in one control tick
- **THEN** the ESP32-S3 does not reset and the owner's session survives

### Requirement: Calibrated wheel mapping

Motor control SHALL map each wheel position (FL, FR, RR, RL) to its driver channel and direction through configuration, not code. **OPEN (O8, M5):** the mapping for the car.

#### Scenario: Positive command moves each wheel forward

- **GIVEN** the car restrained with wheels clear
- **WHEN** each wheel in turn receives 0.3 alone
- **THEN** the wheel at that position turns, in the direction that moves the car forward
