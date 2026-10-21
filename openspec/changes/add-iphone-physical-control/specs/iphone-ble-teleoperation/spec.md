## RENAMED Requirements

- FROM: `### Requirement: Gentle single-axis directions`
- TO: `### Requirement: Gentle input modes`

## MODIFIED Requirements

### Requirement: Supported web delivery

The controller SHALL run as an HTTPS page with Bluefy as its supported iPhone BLE host. It SHALL check the picker API before selection and required GATT read/notify and explicit write modes before writes; missing support SHALL disable control with an explanation. Buttons SHALL be the default; only an explicit Enable Sensors action in Physical mode may request sensor permission. Published assets SHALL contain no secrets or third-party runtime scripts.

#### Scenario: Unsupported host

- **WHEN** the page opens without required Web Bluetooth APIs, including in an unsupported Safari session, or without HTTPS
- **THEN** it explains that supported Bluefy/HTTPS is required and sends no ARM or nonzero setpoints

### Requirement: Gentle input modes

Buttons SHALL retain six single-axis hold directions; multiple touches yield zero until all release and a new press. Physical mode SHALL combine calibrated forward/back tilt as vx, side tilt as vy and heading offset as wz. Each axis SHALL remain within min(0.1 m/s, advertised translation maximum) or min(0.3 rad/s, advertised rotation maximum). Zero-limit axes SHALL stay zero; button velocities SHALL NOT combine.

#### Scenario: Direction mapping respects limits

- **WHEN** each direction is held after successful ARM
- **THEN** forward/back use +/-vx, left/right +/-vy, counter-clockwise/clockwise +/-wz, with other fields zero and advertised limits respected
- **AND** max_vx=0.05 yields forward vx=0.05, while max_vy=0 disables both strafe controls

#### Scenario: Physical movement combines without exceeding per-axis limits

- **WHEN** a fresh physical hold supplies forward, left and counter-clockwise offsets after successful ARM
- **THEN** vx, vy and wz may be positive together, each within its gentle and advertised maximum
- **AND** a zero advertised axis stays zero without disabling other permitted components

### Requirement: Explicit zero then ARM

ARM SHALL require a new user action; Physical ARM SHALL capture neutral from fresh valid sensors in that action. It SHALL invalidate touches, stream fresh zero and send ARM once. Local motion permission SHALL require the ARM write and a subsequent fresh Status read reporting Armed/none/none within one second. A notification alone SHALL NOT grant motion. Failure, refusal or timeout SHALL clear intent and end through cleanup; late callbacks SHALL NOT arm locally.

#### Scenario: Touch predates ARM

- **GIVEN** a direction is held while locally disarmed
- **WHEN** ARM succeeds
- **THEN** only zero is sent until that touch is released and a new direction press begins

#### Scenario: ARM is refused or its confirmation is late

- **WHEN** ARM returns an ATT error or its write/Status confirmation misses the one-second deadline
- **THEN** the client remains locally disarmed and requires a new connection and explicit ARM before motion

#### Scenario: One explicit Physical action captures neutral and arms

- **WHEN** sensors are enabled while disarmed
- **THEN** no ARM or nonzero setpoint is sent
- **AND** a subsequent explicit Set neutral & ARM captures the current fresh screen-up pose without a separate calibration tap, sends zero then ARM and requires a new post-arm press
- **AND** missing/stale/invalid sensors or an invalid neutral posture refuse ARM locally without an ARM write

### Requirement: Hold cancellation and input expiry

Nonzero intent SHALL require one tracked press inside the selected direction control or Physical drive pad while active and locally armed. Release, bounds exit, cancellation, lost tracking or multiple touches SHALL clear intent before the next send. Only UI renewal may extend the 250 ms monotonic hold deadline. Buttons retain arm after hold expiry; Physical expiry revokes calibration and ends an armed/arming session through cleanup. New motion always requires a new press.

#### Scenario: Finger release or missing renewals

- **WHEN** a held direction ends, is cancelled, slides outside, or has no renewal for 250 ms
- **THEN** the next setpoint is zero and no buffered motion is played back
- **AND** sliding back inside does not resume motion without a new press

## ADDED Requirements

### Requirement: Visible control status

Local arm state, STOP and the latest session message SHALL remain visible while scrolling the connected controller, including after a fault disconnects it. Physical mode SHALL explain when input is temporarily unavailable while released. Scroll SHALL clear any hold before another send, without itself revoking arm or calibration. Real lifecycle and stream failures SHALL retain their existing cleanup behavior.

#### Scenario: Scroll while armed at zero

- **WHEN** the user scrolls with the Physical drive pad released
- **THEN** arm state, STOP and the session reason stay visible, intent remains zero and scrolling alone does not disconnect
- **AND** a genuine stream or lifecycle failure still disarms and displays its reason


### Requirement: Explicit physical sensor setup

Physical sensor access SHALL require an active user Enable Sensors action, feature detection and permission handling. Missing/denied/failed permission or missing finite alpha/beta/gamma SHALL prevent physical motion. Late permission results or callbacks from an earlier mode, enable attempt or closed session SHALL NOT change current input. Sensor events SHALL NOT select a device, ARM or write BLE.

#### Scenario: Denial and delayed permission response

- **WHEN** permission is denied or resolves after switching to Buttons or leaving the page
- **THEN** no physical hold or calibration becomes valid and no automatic connection or ARM occurs
- **AND** Buttons remain available without sensor permission

### Requirement: Physical calibration and angle mapping

Each explicit Physical ARM SHALL calibrate fresh finite orientation while locally disarmed with no pending control. Neutral beta/gamma SHALL be within ±30° with screen up. Shortest offsets SHALL map negative beta to forward, negative gamma to left strafe, positive alpha to left rotation, with 3° dead zones and linear scale to saturation at 20°. Held heading offset SHALL sustain rotation; neutral SHALL request zero. No silent recentering is allowed.

#### Scenario: Heading wrap and sustained twist

- **WHEN** calibrated alpha is 350° and current alpha is 10° while a valid drive hold is active
- **THEN** the heading offset is +20°, requesting capped positive wz rather than a −340° turn
- **AND** fresh unchanged samples sustain that request, returning to neutral zeros it, and gyro rotation rate is not integrated

#### Scenario: Neutral posture and operating envelope

- **WHEN** calibration beta or gamma exceeds ±30°, or a held drive has current beta/gamma beyond ±60° or heading offset beyond ±90°, or the screen orientation changes
- **THEN** calibration or motion is rejected with an explanation
- **AND** an armed or arming Physical session loses calibration and ends through cleanup, with manual recovery required

#### Scenario: Reading posture while released

- **WHEN** the pad is released and the phone leaves the operating envelope
- **THEN** intent remains zero and the pad is unavailable, preserving arm and neutral
- **AND** returning to valid fresh input permits only a new press; it neither recenters nor replays a rejected press

### Requirement: Independent physical sample freshness

Only new valid orientation events SHALL renew a 250 ms monotonic sample deadline. Expiry during a held drive SHALL be checked before replacing a sample and end through cleanup before another nonzero send. A released-pad gap SHALL preserve arm/neutral at zero and reject presses until fresh input returns. Invalid input or changed reference SHALL revoke calibration and end armed/arming sessions. UI frames, stream ticks and cached values SHALL NOT renew sample freshness or recover a lost hold.

#### Scenario: Sensor stops while UI and stream continue

- **WHEN** no valid orientation arrives for 250 ms while the held pad and transmitter continue running
- **THEN** requested motion becomes zero, calibration and local arm are revoked, and cleanup begins
- **AND** a late sample or resumed UI cannot restore motion without manual recovery

#### Scenario: Sensor pause while released

- **WHEN** orientation pauses with no drive hold, but the foreground and BLE stream remain healthy
- **THEN** no motion is permitted until new valid input returns, while arm and neutral remain
- **AND** an attempted press during the pause cannot start motion when readings return; all touches must release before a new press

### Requirement: Explicit control mode switching

The same client SHALL expose Buttons and Physical modes, default Buttons without persistence. Every actual mode change SHALL clear both inputs and calibration, disable sensor collection and revoke local arm. A ready session SHALL supersede pending ARM with STOP; setup/picker attempts SHALL be cancelled. New mode selection SHALL NOT ARM or restore a hold; fresh prerequisites, ARM and post-arm press are required.

#### Scenario: Switch while Physical is held or ARM is pending

- **WHEN** the user changes modes while holding the physical pad or awaiting ARM completion
- **THEN** no later old hold, sample or ARM result can authorize motion in the new mode
- **AND** a ready connection may remain at zero after confirmed STOP until a fresh explicit ARM
