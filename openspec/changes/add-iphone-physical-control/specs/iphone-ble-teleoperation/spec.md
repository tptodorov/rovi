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

ARM SHALL require a new user action and, in Physical mode, enabled fresh calibrated sensor input. It SHALL invalidate all touches, stream fresh zero and send ARM once. Local motion permission SHALL require both successful completion of that ARM write and a subsequent fresh Status read reporting Armed/none/none within one second of the action. A notification alone SHALL NOT grant motion. Failure, refusal or timeout SHALL clear intent and end through cleanup; late callbacks SHALL NOT arm locally.

#### Scenario: Touch predates ARM

- **GIVEN** a direction is held while locally disarmed
- **WHEN** ARM succeeds
- **THEN** only zero is sent until that touch is released and a new direction press begins

#### Scenario: ARM is refused or its confirmation is late

- **WHEN** ARM returns an ATT error or its write/Status confirmation misses the one-second deadline
- **THEN** the client remains locally disarmed and requires a new connection and explicit ARM before motion

#### Scenario: Physical calibration cannot arm

- **WHEN** sensors are enabled or neutral is calibrated while disarmed
- **THEN** no ARM or nonzero setpoint is sent
- **AND** an ARM request without valid physical prerequisites is refused locally; successful ARM clears any old hold and requires a new press

### Requirement: Hold cancellation and input expiry

Nonzero intent SHALL require one tracked press inside the selected direction control or Physical drive pad while active and locally armed. Release, bounds exit, cancellation, lost tracking or multiple touches SHALL clear intent before the next send. Only UI renewal may extend the 250 ms monotonic hold deadline. Buttons retain arm after hold expiry; Physical expiry revokes calibration and ends an armed/arming session through cleanup. New motion always requires a new press.

#### Scenario: Finger release or missing renewals

- **WHEN** a held direction ends, is cancelled, slides outside, or has no renewal for 250 ms
- **THEN** the next setpoint is zero and no buffered motion is played back
- **AND** sliding back inside does not resume motion without a new press

## ADDED Requirements

### Requirement: Explicit physical sensor setup

Physical sensor access SHALL require an active user Enable Sensors action, feature detection and permission handling. Missing/denied/failed permission or missing finite alpha/beta/gamma SHALL prevent physical motion. Late permission results or callbacks from an earlier mode, enable attempt or closed session SHALL NOT change current input. Sensor events SHALL NOT select a device, ARM or write BLE.

#### Scenario: Denial and delayed permission response

- **WHEN** permission is denied or resolves after switching to Buttons or leaving the page
- **THEN** no physical hold or calibration becomes valid and no automatic connection or ARM occurs
- **AND** Buttons remain available without sensor permission

### Requirement: Physical calibration and angle mapping

Calibration SHALL require fresh finite orientation, local disarm and no pending control. Neutral beta/gamma SHALL be within ±30° with screen up. Signed shortest offsets SHALL map negative beta to forward, negative gamma to left strafe, positive alpha to left rotation, with 3° dead zones, linear scale to 20° and saturation. Held heading offset SHALL sustain rotation; neutral SHALL request zero. No silent recentering is allowed.

#### Scenario: Heading wrap and sustained twist

- **WHEN** calibrated alpha is 350° and current alpha is 10° while a valid drive hold is active
- **THEN** the heading offset is +20°, requesting capped positive wz rather than a −340° turn
- **AND** fresh unchanged samples sustain that request, returning to neutral zeros it, and gyro rotation rate is not integrated

#### Scenario: Neutral posture and operating envelope

- **WHEN** calibration beta or gamma exceeds ±30°, or a current beta/gamma exceeds ±60°, or heading offset exceeds ±90°, or the screen orientation changes
- **THEN** calibration or motion is rejected with an explanation
- **AND** an armed or arming Physical session loses calibration and ends through cleanup, with manual recovery required

### Requirement: Independent physical sample freshness

Only newly received valid orientation events SHALL renew a 250 ms monotonic sample deadline. Expiry SHALL be checked before replacing an old sample. Stale/invalid input or changed orientation reference SHALL revoke calibration; if locally armed or arming, it SHALL end through cleanup before another nonzero send. UI frames, stream ticks, cached values and declared motion interval SHALL NOT renew sample freshness or silently recover motion.

#### Scenario: Sensor stops while UI and stream continue

- **WHEN** no valid orientation arrives for 250 ms while the held pad and transmitter continue running
- **THEN** requested motion becomes zero, calibration and local arm are revoked, and cleanup begins
- **AND** a late sample or resumed UI cannot restore motion without manual recovery

### Requirement: Explicit control mode switching

The same client SHALL expose Buttons and Physical modes, default Buttons without persistence. Every actual mode change SHALL clear both inputs and calibration, disable sensor collection and revoke local arm. A ready session SHALL supersede pending ARM with STOP; setup/picker attempts SHALL be cancelled. New mode selection SHALL NOT ARM or restore a hold; fresh prerequisites, ARM and post-arm press are required.

#### Scenario: Switch while Physical is held or ARM is pending

- **WHEN** the user changes modes while holding the physical pad or awaiting ARM completion
- **THEN** no later old hold, sample or ARM result can authorize motion in the new mode
- **AND** a ready connection may remain at zero after confirmed STOP until a fresh explicit ARM
