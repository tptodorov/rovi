# iPhone BLE teleoperation delta

## Purpose

Lets an iPhone user control the normal product car over its existing BLE API with explicit arming, bounded touch intent and visible limits on connection and physical-state evidence.

## ADDED Requirements

### Requirement: Explicit discovery and selection

The client SHALL scan only after a user action while active with Bluetooth powered on and authorized, for at most 10 seconds. It SHALL discover the existing name-only advertisement without requiring an advertised service UUID, list distinct peripheral identifiers, and connect only to the selected entry. Scanning SHALL NOT claim the car. No result SHALL be described as proof that the car is busy.

#### Scenario: Two cars share the advertised name

- **WHEN** two peripherals advertise `Rovi M8` during a scan
- **THEN** both appear with distinguishable identifiers and signal strength, and neither is connected automatically
- **AND** Connect explains that connection claims the car and switches off its Wi-Fi AP

#### Scenario: No advertisement is found

- **WHEN** the scan ends without a candidate
- **THEN** the client offers a new manual scan and describes power, range and another owner as possible causes

### Requirement: Validate the connected car

Connection SHALL start locally disarmed at zero. Before streaming, the client SHALL validate product GATT properties, 27-byte version-1 capabilities, positive finite geometry, finite nonnegative limits, nonzero windows, lease >=500 ms, claim >=1000 ms, write-without-response capacity >=72 bytes and a valid Claimed or Stopped Status. Setup SHALL finish within 10 seconds of Connect or end through cleanup; rejected setup SHALL send no ARM or nonzero setpoint.

#### Scenario: Unsupported payload or capabilities

- **WHEN** the selected peripheral lacks required characteristics/properties, has malformed/unsupported capabilities, short windows, or permits fewer than 72 bytes per setpoint write
- **THEN** the client explains incompatibility, attempts disconnect and never fragments a setpoint or substitutes a compact protocol

#### Scenario: Claim expires during setup

- **WHEN** discovery/subscription takes longer than the car's claim window and the car releases the connection
- **THEN** setup fails without reconnecting automatically, even if the local 10-second deadline has not passed

### Requirement: Existing BLE wire contract

The client SHALL use the existing product UUIDs and write each setpoint as one 72-byte value: little-endian u32 sequence plus 68-byte little-endian TwistStamped CDR, empty frame, zero timestamp, and only vx/vy/wz nonzero. Sequences SHALL advance with wrapping u32 ordering per connection. ARM/STOP SHALL use one-byte values 1/2 with response. BLE SHALL have no UDP header, numeric session ID, HELLO or BYE.

#### Scenario: Wire values agree with the product

- **WHEN** zero, each direction and a sequence wrapping from 4294967295 to 0 are encoded
- **THEN** bytes match the existing Rust/Python product vectors and the unused Twist fields are zero
- **AND** a new connection starts a new sequence history

### Requirement: Explicit zero then ARM

Only a new user ARM action SHALL initiate arming. It SHALL invalidate all touches, stream fresh zero and send ARM once. Local motion permission SHALL require both successful completion of that ARM write and a subsequent fresh Status read reporting Armed/none/none within one second of the action. A notification alone SHALL NOT grant motion. Failure, refusal or timeout SHALL clear intent and end through cleanup; late callbacks SHALL NOT arm locally.

#### Scenario: Touch predates ARM

- **GIVEN** a direction is held while locally disarmed
- **WHEN** ARM succeeds
- **THEN** only zero is sent until that touch is released and a new direction press begins

#### Scenario: ARM is refused or its confirmation is late

- **WHEN** ARM returns an ATT error or its write/Status confirmation misses the one-second deadline
- **THEN** the client remains locally disarmed and requires a new connection and explicit ARM before motion

### Requirement: Gentle single-axis directions

The UI SHALL provide forward/back, left/right strafe and counter-clockwise/clockwise rotation in base_link. Each press SHALL select one axis only: translation magnitude min(0.1 m/s, advertised axis maximum), rotation min(0.3 rad/s, advertised maximum). Zero-limit axes SHALL be disabled. Multiple simultaneous direction touches SHALL request zero and require all direction touches released before a new press; velocities SHALL NOT combine.

#### Scenario: Direction mapping respects limits

- **WHEN** each direction is held after successful ARM
- **THEN** forward/back use +/-vx, left/right +/-vy, counter-clockwise/clockwise +/-wz, with other fields zero and advertised limits respected
- **AND** max_vx=0.05 yields forward vx=0.05, while max_vy=0 disables both strafe controls

### Requirement: Hold cancellation and input expiry

Nonzero intent SHALL require a tracked press inside one direction control while active and locally armed. Release, leaving its bounds, cancellation or lost tracking SHALL clear intent before the next send. Hold renewal SHALL expire after 250 ms on a monotonic clock; the transmitter SHALL NOT renew input itself. Release/expiry SHALL send zero and require a new press, retaining local arm; STOP/interruption SHALL require re-arm and a new press.

#### Scenario: Finger release or missing renewals

- **WHEN** a held direction ends, is cancelled, slides outside, or has no renewal for 250 ms
- **THEN** the next setpoint is zero and no buffered motion is played back
- **AND** sliding back inside does not resume motion without a new press

### Requirement: Periodic latest-intent stream

After setup the client SHALL attempt one latest setpoint every 50 ms, zero when locally disarmed, awaiting ARM or without valid hold intent. It SHALL check stop/lifecycle events and input expiry before each send and obey BLE write-without-response readiness. It SHALL retain no motion backlog. A scheduler gap or interval without submitting a setpoint of >=200 ms SHALL revoke local arm and end through cleanup, without catch-up motion.

#### Scenario: Transport backpressure

- **GIVEN** a nonzero hold and BLE is not ready for writes
- **WHEN** readiness returns before the 200 ms failure bound
- **THEN** only current unexpired intent is submitted, with no accumulated writes
- **AND** readiness returning at or after the bound cannot resume motion

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

Leaving the active foreground, including lock or system interruption, SHALL revoke local arm, clear touches, stop scanning/normal streaming and end through cleanup. Bluetooth unavailable/resetting, permission denial/revocation, disconnect or service invalidation SHALL do the same. Return to active, permission restoration, Bluetooth restoration and app relaunch SHALL remain disconnected/disarmed until manual Connect and ARM; no restoration or automatic reconnect SHALL replay control.

#### Scenario: Lock or Bluetooth loss while holding

- **WHEN** the phone locks, the app becomes inactive/backgrounded, Bluetooth becomes unavailable or permission is lost
- **THEN** no further nonzero intent is generated, cleanup is attempted while execution remains available, and foreground return cannot resume the hold
- **AND** suspension/termination without callbacks relies on the firmware lease rather than a promised phone cleanup

### Requirement: Bounded best-effort shutdown

Disconnect, setup failure, interruption and handled failure SHALL clear intent and stop normal writes, then independently attempt zero and STOP where supported, and local cancellation, within 500 ms total while runnable. Cancellation SHALL still be attempted if a write fails or never completes. The UI SHALL distinguish attempted, device-reported STOP and local disconnect; it SHALL NOT infer physical wheel stop or immediate ownership release from a write or cancellation.

#### Scenario: Shutdown cannot complete its writes

- **WHEN** zero is blocked or STOP receives no completion
- **THEN** local cancellation is requested by the shared 500 ms deadline without waiting indefinitely
- **AND** late callbacks are ignored and stop/release remain unconfirmed

### Requirement: Visible state and evidence limits

The UI SHALL show selected peripheral, connection phase, local motion permission, limits, payload capacity, requested vx/vy/wz and last-reported device state/reason/error with observation age. Controls SHALL have text/accessibility labels, adequate touch targets and state cues beyond color; accessibility activation SHALL NOT latch nonzero motion. Documentation SHALL identify approximate open-loop references and distinguish unit/UI simulation from phone/radio/wheel evidence.

#### Scenario: Reported Armed does not mean moving wheels

- **WHEN** the last reported Status is Armed but no direction is held, or a lifecycle failure has revoked local permission
- **THEN** requested motion is zero, Status retains its observation age, and the UI does not claim measured speed, live wheel state or certified emergency stopping
