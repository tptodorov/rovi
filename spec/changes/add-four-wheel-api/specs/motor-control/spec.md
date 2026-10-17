## ADDED Requirements

### Requirement: Four independent wheel commands

Motor control SHALL accept four signed wheel commands in −1.0..=1.0, ordered FL, FR, RR, RL. Positive means forward for that wheel. Magnitude SHALL be converted by the documented deadband/duty rule, not assumed proportional to physical speed. Each command SHALL affect its own wheel only, with no kinematic mixing.

#### Scenario: Each wheel is driven on its own

- **GIVEN** motor control is enabled
- **WHEN** it receives wheel commands (0.5, 0, 0, 0)
- **THEN** only FL is driven forward at the duty produced by the deadband/duty rule
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

Wheel command 1.0 SHALL map to the configured maximum PWM duty. The car SHALL use open-loop duty control without encoder feedback. The configured duty cap SHALL be enforced independently for each wheel. **OPEN (O1):** the electrically qualified maximum; 70% reversal/thermal evidence does not validate 100% operation. Duty limiting SHALL NOT be advertised as measured current protection.

#### Scenario: All wheels at full scale stay in limits

- **GIVEN** the car on the M4 bench with its intended supply
- **WHEN** all four wheels are commanded at 1.0
- **THEN** measured per-channel and combined current stay within the O1 limits
- **AND** the logic rail stays above brownout

### Requirement: Zero command behaviour

A zero wheel command SHALL request coast for that wheel without disabling a driven neighbour on the same driver. On the TB6612, this SHALL use IN1=IN2=low and PWM=high while STBY is high. A safety stop SHALL instead deassert both drivers' STBY inputs. **OPEN (O5):** physical coast distance, not the selected mode. Source: Toshiba [H-SW table, page 4](../../../../../docs/reference/tb6612fng/datasheet.pdf).

#### Scenario: Zero stops a turning wheel

- **GIVEN** a wheel driven at 0.5
- **WHEN** it receives a zero wheel command
- **THEN** its outputs are high impedance and it is allowed to coast

#### Scenario: A zero wheel does not disable its neighbour

- **GIVEN** FL and RL share driver 1 and motor control is enabled
- **WHEN** FL receives zero and RL receives 0.5
- **THEN** FL requests coast with both direction pins low and PWM high
- **AND** RL retains its requested duty and driver 1 remains enabled

### Requirement: Dead zone handling

Exact zero or a magnitude below the configured deadband SHALL request coast. Otherwise duty SHALL be `min + abs(command) * (max - min)`, rounded to permille after input clamping. The initial deadband SHALL be 0.02 and minimum duty 40%; maximum duty SHALL be configured. A precision minimum physical speed SHALL NOT be inferred from this rule. The stand starting observation was about 35%, not a universal per-wheel threshold.

#### Scenario: Small command behaviour is predictable

- **GIVEN** deadband 0.02, minimum duty 40% and maximum duty 100%
- **WHEN** the wheel receives 0, 0.01, 0.02, 0.5 and 1.0 in turn
- **THEN** the requested modes/duties are coast, coast, 41.2%, 70% and 100%
- **AND** negative commands use the same duty magnitude with reversed direction

### Requirement: Starts and reversals do not reset the controller

Motor control SHALL support direct signed changes within the qualified operating envelope. M4 observed reversals at ±70% without reset or BLE loss; no slew limit is required by that evidence. The product SHALL NOT claim the same result above the qualified duty without validation. **OPEN (O7):** qualification at the selected product maximum.

#### Scenario: Full reversal on all wheels

- **GIVEN** a configured maximum within the qualified operating envelope and all four wheels at 1.0
- **WHEN** all four are commanded to −1.0 in one control tick
- **THEN** the ESP32-S3 does not reset and the owner's session survives

### Requirement: Calibrated wheel mapping

Motor control SHALL configure each wheel's driver channel and inversion. The current car mapping SHALL be D1 A→FL, D1 B→RL, D2 A→FR, D2 B→RR, with all inversions false because directions were corrected at connectors. Both separate STBY outputs SHALL follow motor enable/disable together.

#### Scenario: Positive command moves each wheel forward

- **GIVEN** the car restrained with wheels clear
- **WHEN** each wheel in turn receives 0.3 alone
- **THEN** the wheel at that position turns, in the direction that moves the car forward
