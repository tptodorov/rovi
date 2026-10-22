# iphone-ble-teleoperation Specification

## Purpose
Lets an iPhone user control the normal product car over its existing BLE API with explicit arming, bounded touch intent and visible limits on connection and physical-state evidence.

## Requirements

### Requirement: Supported web delivery

The controller SHALL run as an HTTPS page with Bluefy as its supported iPhone BLE host. It SHALL check the picker API before selection and required GATT read/notify and explicit write modes before writes; missing support SHALL disable control with an explanation. Buttons SHALL be the default; only a user selection of Physical mode or explicit sensor retry may request sensor permission. Published assets SHALL contain no secrets or third-party runtime scripts.

#### Scenario: Unsupported host

- **WHEN** the page opens without required Web Bluetooth APIs, including in an unsupported Safari session, or without HTTPS
- **THEN** it explains that supported Bluefy/HTTPS is required and sends no ARM or nonzero setpoints

### Requirement: Explicit discovery and selection

The client SHALL invoke the browser device picker only from an explicit Choose & Connect action while active. It SHALL select the existing name-only advertisement without requiring an advertised service UUID, request access to the product service, and connect only to the chosen device. The picker SHALL NOT be described as a claim; GATT connection is the claim. Cancellation/no result SHALL NOT be described as proof that the car is busy.

#### Scenario: Browser selection precedes connection

- **WHEN** Choose & Connect opens the browser picker, including when same-name candidates exist
- **THEN** no GATT connection occurs until the user selects a candidate, and the page does not auto-select a remembered device
- **AND** the action explains that connecting claims the car and switches off its Wi-Fi AP; the page does not promise picker RSSI or distinguishable hardware addresses

#### Scenario: Picker cancelled or completed after local cancellation

- **WHEN** the picker is cancelled, has no selection, or returns after the local attempt was cancelled
- **THEN** no connection is started; a new manual action is required
- **AND** the page does not claim it can impose a scan timeout or forcibly dismiss the browser picker

### Requirement: Validate the connected car

Setup SHALL start locally disarmed at zero. Before regular streaming, the client SHALL validate GATT properties, 27-byte version-1 capabilities, positive finite geometry, finite nonnegative limits, nonzero windows, lease >=500 ms, claim >=1000 ms, valid Claimed/Stopped Status and an initial 72-byte zero write submission. Setup SHALL finish within 10 seconds of GATT connect invocation or end through cleanup. It SHALL send no ARM or nonzero setpoint.

#### Scenario: Unsupported payload or capabilities

- **WHEN** required characteristics/properties or capabilities are invalid, windows are unsupported, or the initial 72-byte submission is rejected
- **THEN** the client reports incompatibility and attempts disconnect without fragmentation, protocol substitution or automatic reconnect
- **AND** a resolved without-response write is not displayed as proof of device receipt or measured MTU

#### Scenario: Claim expires during setup

- **WHEN** discovery/subscription takes longer than the car's claim window and the car releases the connection
- **THEN** setup fails without automatic reconnect, even if the local 10-second deadline has not passed

### Requirement: Existing BLE wire contract

The client SHALL use the existing product UUIDs and write each setpoint as one 72-byte value: little-endian u32 sequence plus 68-byte little-endian TwistStamped CDR, empty frame, zero timestamp, and only vx/vy/wz nonzero. Sequences SHALL advance with wrapping u32 ordering per connection. ARM/STOP SHALL use one-byte values 1/2 with response. BLE SHALL have no UDP header, numeric session ID, HELLO or BYE.

#### Scenario: Wire values agree with the product

- **WHEN** zero, each direction and a sequence wrapping from 4294967295 to 0 are encoded
- **THEN** bytes match the existing Rust/Python product vectors and the unused Twist fields are zero
- **AND** a new connection starts a new sequence history

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
- **AND** a subsequent explicit Set neutral & ARM captures the current fresh screen-up pose without a separate calibration tap, sends zero then ARM and requires a separate post-arm START action
- **AND** missing/stale/invalid sensors or an invalid neutral posture refuse ARM locally without an ARM write

### Requirement: Gentle input modes

Buttons SHALL retain six single-axis hold directions; multiple touches yield zero until all release and a new press. Physical mode SHALL combine calibrated forward/back tilt as vx, side tilt as vy and heading offset as wz. Each axis SHALL remain within min(0.1 m/s, advertised translation maximum) or min(0.3 rad/s, advertised rotation maximum). Zero-limit axes SHALL stay zero; button velocities SHALL NOT combine.

#### Scenario: Direction mapping respects limits

- **WHEN** each direction is held after successful ARM
- **THEN** forward/back use +/-vx, left/right +/-vy, counter-clockwise/clockwise +/-wz, with other fields zero and advertised limits respected
- **AND** max_vx=0.05 yields forward vx=0.05, while max_vy=0 disables both strafe controls

#### Scenario: Physical movement combines without exceeding per-axis limits

- **WHEN** explicitly started Physical control supplies forward, left and counter-clockwise offsets after successful ARM
- **THEN** vx, vy and wz may be positive together, each within its gentle and advertised maximum
- **AND** a zero advertised axis stays zero without disabling other permitted components

### Requirement: Hold cancellation and input expiry

Buttons nonzero intent SHALL require one tracked press inside a direction control while active and locally armed. Release, bounds exit, cancellation, lost tracking or multiple touches SHALL clear intent before the next send. Only UI renewal may extend the 250 ms monotonic hold deadline. Buttons retain arm after expiry and require a new press. Physical control SHALL use the explicit START/PAUSE gate and independent foreground renewal deadline instead of a held pointer.

#### Scenario: Finger release or missing renewals

- **WHEN** a held direction ends, is cancelled, slides outside, or has no renewal for 250 ms
- **THEN** the next setpoint is zero and no buffered motion is played back
- **AND** sliding back inside does not resume motion without a new press

### Requirement: Periodic latest-intent stream

After setup the client SHALL attempt one latest setpoint every 50 ms, zero when locally disarmed, awaiting ARM, without a valid Buttons hold or while Physical control is paused. It SHALL check lifecycle/stops/expiry before sending, allow one pending setpoint write and retain no motion backlog. A scheduler gap or interval without submission of >=200 ms SHALL revoke local arm and end through cleanup, without catch-up motion. Write rejection SHALL end the session.

#### Scenario: Transport backpressure

- **GIVEN** nonzero intent and a pending write prevents another submission
- **WHEN** that procedure completes before the 200 ms failure bound
- **THEN** the next send uses only current unexpired intent, with no accumulated writes
- **AND** completion at or after the bound cannot resume motion; a promise resolving does not confirm applied wheel motion

### Requirement: Validate Status and link health

The client SHALL validate three-byte Status codes and state/reason consistency, subscribe to changes and schedule a Status read every 500 ms while ready, with at most one transaction pending. A transaction taking >=1000 ms, malformed Status, GATT error, Open Status or unexpected Armed Status SHALL revoke local motion and end through cleanup. Stopped Status SHALL revoke local arm. Callbacks from old connections or cancelled operations SHALL NOT change the current session.

#### Scenario: Silence and old callbacks

- **WHEN** Status is unchanged without notifications
- **THEN** reads refresh its observation age; missing a read deadline ends the session
- **AND** an Armed callback from an earlier connection cannot authorize a newly connected session

### Requirement: STOP and re-arm latch

STOP SHALL remain available throughout a connected session, clear intent immediately and request Control=2 with response. It SHALL override ARM and touch input. Confirmation SHALL require successful STOP write completion and a subsequent fresh Stopped/Stop/none Status read within one second. Confirmed STOP or reported safety stop SHALL require explicit ARM and a new press before motion; unconfirmed STOP SHALL end through cleanup. Zero alone SHALL NOT be described as STOP.

#### Scenario: STOP races with ARM and touch

- **WHEN** STOP occurs while ARM is pending or direction events are queued
- **THEN** pending local arming is invalidated, queued direction input is discarded, and any late Armed result cannot enable motion
- **AND** after confirmed STOP the client may keep streaming zero to retain ownership until manual re-arm or Disconnect

### Requirement: Active foreground and manual recovery

During a connected session, blur/hidden/pagehide, host inactivity, observed permission/service loss or Bluetooth failure SHALL clear holds, revoke local arm and end through cleanup. Return/reload/restoration SHALL require manual Choose & Connect and ARM, never automatic recovery/replay. A pending picker SHALL NOT connect while inactive. If inactive UI stops renewing input, expiry SHALL clear motion even while transmission runs. Sustained inactive motion SHALL fail host acceptance.

#### Scenario: Lock or Bluetooth loss while holding

- **WHEN** the phone locks, changes app, or loses Bluetooth while a direction is held
- **THEN** observed inactivity/loss clears motion before another send, and foreground return cannot resume the hold
- **AND** if execution or UI renewal stops without an event, input expiry or the independent firmware lease stops motion; runnable cleanup is not promised after suspension/termination
- **AND** phone checks must reject a host that keeps renewing nonzero motion while inactive

### Requirement: Bounded best-effort shutdown

Disconnect, setup failure, interruption and handled failure SHALL clear intent and stop normal writes, then independently attempt zero and STOP where supported, and local cancellation, within 500 ms total while runnable. Cancellation SHALL still be attempted if a write fails or never completes. The UI SHALL distinguish attempted, device-reported STOP and local disconnect; it SHALL NOT infer physical wheel stop or immediate ownership release from a write or cancellation.

#### Scenario: Shutdown cannot complete its writes

- **WHEN** zero is blocked or STOP receives no completion
- **THEN** local cancellation is requested by the shared 500 ms deadline without waiting indefinitely
- **AND** late callbacks are ignored and stop/release remain unconfirmed

### Requirement: Visible state and evidence limits

The UI SHALL show selected device name/available ID, phase, local arm, limits, packet size, requested vx/vy/wz and reported state/reason/error with age. Unexposed MTU/capacity SHALL be labelled unavailable. Controls SHALL have text/accessibility labels, adequate targets and cues beyond color. Buttons accessibility activation SHALL NOT latch motion; explicit Physical START/PAUSE activation SHALL match a tap. Docs SHALL separate approximate references, host tests, phone/radio and wheel evidence.

#### Scenario: Reported Armed does not mean moving wheels

- **WHEN** reported Status is Armed but no button direction is held and Physical control is paused, or interruption revoked local permission
- **THEN** requested motion is zero, Status retains its observation age, and the UI does not claim measured speed, live wheel state or certified emergency stopping

### Requirement: Visible control status

Local arm state, STOP, Physical START/PAUSE, requested motion and the latest session message SHALL remain visible while scrolling, including after fault disconnect. Physical mode SHALL show running/paused state and explain unavailable input. Scroll SHALL clear button holds and pause Physical control before another send, preserving arm/neutral. Real lifecycle and stream failures SHALL retain cleanup. Posture faults SHALL identify the angle, observed value and limit.

#### Scenario: Scroll while armed at zero

- **WHEN** the user scrolls while Physical control is paused
- **THEN** arm state, STOP and the session reason stay visible, intent remains zero and scrolling alone does not disconnect
- **AND** a genuine stream or lifecycle failure still disarms and displays its reason

### Requirement: Explicit physical sensor setup

Physical sensor setup SHALL begin only from an active user selection of Physical mode or explicit Retry action, with feature detection and permission handling. Missing/denied/failed permission or missing finite alpha/beta/gamma SHALL prevent physical motion. Late permission results or callbacks from an earlier mode, enable attempt or closed session SHALL NOT change current input. Sensor events SHALL NOT select a device, ARM or write BLE.

#### Scenario: Denial and delayed permission response

- **WHEN** permission is denied or resolves after switching to Buttons or leaving the page
- **THEN** no Physical running state or calibration becomes valid and no automatic connection or ARM occurs
- **AND** Buttons remain available without sensor permission

### Requirement: Physical calibration and angle mapping

Each explicit Physical ARM SHALL calibrate fresh finite orientation while locally disarmed with no pending control. Neutral beta/gamma SHALL be within ±30° with screen up. Shortest offsets SHALL map negative beta to forward, negative gamma to left strafe, positive alpha to left rotation, with 3° dead zones and linear scale to saturation at 20°. Held heading offset SHALL sustain rotation; neutral SHALL request zero. No silent recentering is allowed.

#### Scenario: Heading wrap and sustained twist

- **WHEN** calibrated alpha is 350° and current alpha is 10° while Physical control is explicitly running
- **THEN** the heading offset is +20°, requesting capped positive wz rather than a −340° turn
- **AND** fresh unchanged samples sustain that request, returning to neutral zeros it, and gyro rotation rate is not integrated

#### Scenario: Neutral posture and operating envelope

- **WHEN** calibration beta or gamma exceeds ±30°, or running Physical control has current beta/gamma beyond ±60° or heading offset beyond ±90°, or the screen orientation changes
- **THEN** calibration or motion is rejected with an explanation
- **AND** an armed or arming Physical session loses calibration and ends through cleanup, with manual recovery required

#### Scenario: Reading posture while paused

- **WHEN** Physical control is paused and the phone leaves the operating envelope
- **THEN** intent remains zero and START is unavailable, preserving arm and neutral
- **AND** returning to valid fresh input permits only a new explicit START at neutral; it neither recenters nor resumes driving

### Requirement: Independent physical sample freshness

Only new valid orientation events SHALL renew a 250 ms monotonic sample deadline. Expiry during running Physical control SHALL be checked before replacing a sample and end through cleanup before another nonzero send. Paused gaps SHALL preserve arm/neutral at zero and reject START until fresh input returns. Invalid input or changed reference SHALL revoke calibration and end armed/arming sessions. UI frames, stream ticks and cached values SHALL NOT renew samples or restart driving.

#### Scenario: Sensor stops while UI and stream continue

- **WHEN** no valid orientation arrives for 250 ms while Physical control and the transmitter continue running
- **THEN** requested motion becomes zero, calibration and local arm are revoked, and cleanup begins
- **AND** a late sample or resumed UI cannot restore motion without manual recovery

#### Scenario: Sensor pause while paused

- **WHEN** orientation pauses with Physical control paused, but the foreground and BLE stream remain healthy
- **THEN** no motion is permitted until new valid input returns, while arm and neutral remain
- **AND** an attempted START during the pause cannot start motion when readings return; a new explicit START at neutral is required

### Requirement: Explicit control mode switching

The same client SHALL expose Buttons and Physical modes, default Buttons without persistence. Every actual mode change SHALL clear both inputs and calibration, disable sensor collection and revoke local arm. A ready session SHALL supersede pending ARM with STOP; setup/picker attempts SHALL be cancelled. New mode selection SHALL NOT ARM or restore a hold; fresh prerequisites, ARM and post-arm input action are required.

#### Scenario: Switch while Physical is running or ARM is pending

- **WHEN** the user changes modes while Physical control is running or awaiting ARM completion
- **THEN** no later old running state, sample or ARM result can authorize motion in the new mode
- **AND** a ready connection may remain at zero after confirmed STOP until a fresh explicit ARM

### Requirement: Tap-based Physical driving

Physical control SHALL stay paused after ARM. START SHALL require active foreground, confirmed arm, fresh calibration and three offsets within ±3°. Running needs no held touch. PAUSE/scroll SHALL clear intent before the next send, retaining arm/neutral. ARM, STOP, mode changes and cleanup SHALL clear running state. Only UI frames may renew its 250 ms deadline; expiry SHALL revoke calibration and end through cleanup. Only a new explicit START may restart it.

#### Scenario: Start, tilt and pause without holding

- **WHEN** the user taps START at neutral after ARM, releases the screen and tilts the phone
- **THEN** fresh input supplies bounded motion without a held pointer and the persistent control reads PAUSE
- **AND** PAUSE yields zero while retaining local arm and neutral; returning sensor data or neutral alone does not restart driving

#### Scenario: Neutral and explicit restart are required

- **WHEN** START occurs before ARM confirmation, while sensors are stale, or outside ±3° of calibrated neutral
- **THEN** no running state or nonzero intent is granted, and later data/ARM completion cannot replay that START
- **AND** after pause/scroll or a new ARM, motion requires a new explicit START at neutral

#### Scenario: Foreground renewal stops independently of sensors

- **WHEN** Physical control is running and no UI frame renews its deadline for 250 ms while fresh sensors and the stream continue
- **THEN** intent becomes zero and local arm/calibration are revoked through cleanup
- **AND** resumed frames and samples cannot restart driving or reconnect

### Requirement: Mobile controller layout

The controller SHALL present labelled Buttons/Physical segmented radios, requested motion and connection/arm/driving states. At 320×568 through 390×844 CSS pixels, six Buttons directions and requested motion SHALL fit above a persistent thumb-reachable ARM/START/PAUSE and STOP dock. Sensor setup SHALL be implicit in choosing Physical; Retry appears only when needed. Protocol/raw sensor details SHALL be collapsed by default; opening diagnostics SHALL clear input without ARM or restart.

#### Scenario: Drive without scrolling through setup details

- **WHEN** the connected controller is viewed in either mode on the supported portrait sizes
- **THEN** the current primary action and STOP remain visible, and requested motion is readable without opening diagnostics
- **AND** Buttons keeps held directions while Physical keeps explicit neutral-gated START/PAUSE; choosing modes still disarms

#### Scenario: Larger text and keyboard access

- **WHEN** text is enlarged to 200%, a keyboard selects modes, or a narrow/landscape viewport is used
- **THEN** controls remain labelled, targets at least 44×44 CSS pixels, content can scroll without horizontal overflow and focus is not covered by the dock
- **AND** light/dark themes have readable text, motion is reduced when requested and no sensor or Bluetooth permission is requested by initial page load

#### Scenario: Diagnostics while Physical is running

- **WHEN** connection diagnostics are opened during Physical driving
- **THEN** input becomes zero and stays paused without losing neutral/arm or disconnecting solely because details opened
- **AND** closing details or returning readings cannot resume driving; a new explicit START at neutral is needed
