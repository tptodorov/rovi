## Purpose

Controls four motor channels independently with explicit enable, standby, duty mapping and fault behavior, without depending on device kinematics or transports.

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

Motor control SHALL expose configuration, wheel commands, enable/standby and typed validation faults. It SHALL NOT depend on setpoints, mecanum kinematics, the source or the session.

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

### Requirement: Standby on safety stop

When the device raises a safety stop, the coordinator SHALL request standby in the control tick that processes the stop, and motor control SHALL set every wheel command to zero and deassert both STBY outputs before any further bridge update. A measured pin-level bound is deferred (O4: needs a logic analyzer or oscilloscope); this change SHALL NOT claim one.

#### Scenario: Watchdog timeout puts every driver in standby

- **GIVEN** the car is armed and all four wheels are driven
- **WHEN** a watchdog timeout raises a safety stop
- **THEN** both STBY outputs are deasserted in that control tick, before any further bridge update
- **AND** on the bench every wheel stops

### Requirement: Full-scale maps to the configured limit

Wheel command 1.0 SHALL map to the configured maximum PWM duty, enforced independently for each wheel. The car SHALL use open-loop duty control without encoder feedback. The product profile SHALL cap drive duty at 70%, the envelope M4 verified (O1). Raising the cap SHALL require new qualification. Duty limiting SHALL NOT be advertised as current protection; start and stall current are not measured.

#### Scenario: Product profile full scale

- **GIVEN** the product profile: deadband 0.02, minimum duty 40%, maximum duty 70%
- **WHEN** a wheel receives 1.0, then 0.5, then 1.5
- **THEN** the requested duties are 70%, 55% and 70%
- **AND** no command drives any wheel above 70%

### Requirement: Zero command behaviour

A zero wheel command SHALL request coast for that wheel without disabling a driven neighbour on the same driver. On the TB6612, this SHALL use IN1=IN2=low and PWM=high while STBY is high. A safety stop SHALL instead deassert both drivers' STBY inputs. Coast distance is not measured or claimed (O5). Source: Toshiba [H-SW table, page 4](../../../../../docs/reference/tb6612fng/datasheet.pdf).

#### Scenario: Zero coasts a turning wheel

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

Motor control SHALL support direct signed changes, without a slew limit, up to the product cap. M4 reversed all four wheels between ±70% ten times without an ESP32 reset or BLE loss (O7). Operation above 70% SHALL NOT be claimed without new qualification.

#### Scenario: Full reversal on all wheels

- **GIVEN** the product profile (maximum 70%) and all four wheels at 1.0
- **WHEN** all four are commanded to −1.0 in one control tick
- **THEN** the ESP32-S3 does not reset and the owner's session survives

### Requirement: Calibrated wheel mapping

Motor control SHALL configure each wheel's driver channel and inversion. The current car mapping SHALL be D1 A→FL, D1 B→RL, D2 A→FR, D2 B→RR, with all inversions false because directions were corrected at connectors. Both separate STBY outputs SHALL follow motor enable/disable together.

#### Scenario: Positive command moves each wheel forward

- **GIVEN** the car restrained with wheels clear
- **WHEN** each wheel in turn receives 0.3 alone
- **THEN** the wheel at that position turns, in the direction that moves the car forward

### Requirement: Configuration validation

Before enabling, motor control SHALL validate finite deadband in [0,1), integer duty limits 0 ≤ min ≤ max ≤ 1000 permille, and a one-to-one mapping of four wheels to four bridge channels. Invalid configuration SHALL report a configuration fault and keep both drivers disabled. Configuration SHALL be immutable while enabled; replacing it SHALL disable and clear stored commands before validation.

#### Scenario: Invalid duty or mapping cannot enable

- **GIVEN** minimum duty 800 and maximum 700, or two wheels assigned to the same channel
- **WHEN** configuration is installed and enable requested
- **THEN** configuration is rejected and both STBY signals stay low

#### Scenario: Invalid floating-point configuration cannot drive

- **GIVEN** deadband NaN, infinity, negative or at least 1
- **WHEN** configuration is installed
- **THEN** a configuration fault is reported before any driver enable

### Requirement: Fault recovery requires explicit enable

A non-finite wheel vector SHALL invalidate stored commands and latch motor control disabled. Subsequent wheel updates SHALL NOT clear the latch. Explicit enable with a valid configuration and a complete finite wheel vector SHALL be required to recover; enabling SHALL NOT replay a previously stored vector. Standby SHALL always work, including in a fault state.

#### Scenario: Valid updates do not clear a fault

- **GIVEN** a NaN command caused both drivers to enter standby
- **WHEN** several finite wheel updates arrive without explicit enable
- **THEN** both drivers remain disabled
- **AND** a later explicit enable with a new finite vector clears the command fault and applies only that vector

### Requirement: Complete vector validation before output

Motor control SHALL validate the entire vector before applying any channel, so a bad fourth value cannot partially drive the first three. Standby SHALL deassert both STBY outputs before preparing any new direction/PWM state. On enable, valid bridge states SHALL be prepared while disabled before STBY is asserted. Driver faults SHALL be reported to the caller, not silently converted to successful commands.

#### Scenario: Fourth invalid wheel cannot cause partial motion

- **GIVEN** drivers are disabled and a recording host driver
- **WHEN** enable receives (0.5, 0.5, 0.5, infinity)
- **THEN** no enable-high or drive update is issued for any bridge

#### Scenario: Safe enable ordering

- **GIVEN** disabled motor control with a valid configuration
- **WHEN** it is explicitly enabled with a finite vector
- **THEN** both drivers remain disabled until all four bridge states are prepared

### Requirement: Coast is independent of the drive duty cap

The configured maximum SHALL limit driven PWM duty, not the PWM-high signal required by the documented coast mode. A coast request SHALL keep both direction inputs low while PWM is high; it SHALL NOT become forward drive or brake merely because the configured drive cap is below 100%.

#### Scenario: Coast with a reduced maximum

- **GIVEN** maximum drive duty 70%, FL at zero and RL at 1.0
- **WHEN** their shared driver is enabled
- **THEN** FL has both direction pins low and PWM high, while RL is driven at 70%
