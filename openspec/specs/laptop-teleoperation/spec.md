# laptop-teleoperation Specification

## Purpose
Lets a laptop operator control the existing car UDP API interactively with explicit arming, bounded motion input, visible state and predictable failure cleanup.

## Requirements

### Requirement: Explicit connection without motion

The client SHALL require an interactive terminal and exactly one transport: explicit UDP IPv4:port or BLE discovery with optional MAC. Each UDP attempt SHALL validate HELLO's nonzero session/capabilities and use fresh control sequences. BLE SHALL validate GATT/capabilities before controls. All connections SHALL start disarmed at zero. BUSY/unavailable targets SHALL retry without motion; invalid capabilities or leases below 200 ms SHALL fail.

#### Scenario: Connection starts disarmed

- **WHEN** an operator connects to a valid UDP or BLE target
- **THEN** the client displays negotiated limits and connection identity and sends no ARM or nonzero setpoint

#### Scenario: Refused or unsupported connection

- **WHEN** UDP returns BUSY, its one-second handshake expires, or BLE discovery/connect fails
- **THEN** the client retries with no ARM or nonzero setpoint
- **AND** invalid capabilities or an unsupported lease terminate through cleanup

#### Scenario: Old handshake cannot reconnect

- **WHEN** a UDP ACK from an earlier attempt arrives during a new handshake
- **THEN** it cannot establish the new session

### Requirement: Explicit arm and stop latch

Only an operator R key SHALL request ARM, after clearing motion and sending a fresh zero setpoint. Motion SHALL remain disabled until a matching successful Armed Status reply arrives. A refused/timed-out ARM, STOP or reported safety stop SHALL clear intent and require a new R key before further motion. Held or repeated motion keys SHALL never arm the car.

#### Scenario: Keys cannot arm

- **GIVEN** a connected but locally disarmed client
- **WHEN** motion keys arrive
- **THEN** no nonzero setpoint is sent

#### Scenario: Arm does not replay a direction

- **GIVEN** previous motion intent exists
- **WHEN** R is pressed and ARM succeeds
- **THEN** the client starts armed at zero and requires a subsequent motion key

### Requirement: Gentle planar directions

W/S SHALL request forward/backward, A/D left/right strafe, and Q/E counter-clockwise/clockwise rotation in base_link. Only one direction SHALL be active at a time. Translation magnitude SHALL be min(0.1 m/s, the advertised axis maximum), rotation min(0.3 rad/s, advertised maximum). Disabled axes SHALL remain zero. Other Twist components SHALL be zero; simultaneous keys SHALL not combine velocities.

#### Scenario: Limits below the defaults are respected

- **GIVEN** an armed client whose advertised max_vx is 0.05 m/s
- **WHEN** W is received
- **THEN** vx is 0.05 and vy/wz are zero

#### Scenario: New direction replaces the previous one

- **GIVEN** W was the current direction
- **WHEN** E is received
- **THEN** vx/vy become zero and only clockwise wz is requested

### Requirement: Input expiry and latest intent

Each recognized motion key SHALL expire 250 ms after its monotonic receipt time. The next send after expiry SHALL be zero. Key repeats SHALL refresh that window; unrelated keys SHALL not. Buffered motion SHALL not form a playback queue. STOP, quit, disarm and failure SHALL clear buffered motion. The client SHALL describe key repeat as renewal, not claim it receives physical key-release events.

#### Scenario: Motion input expires

- **GIVEN** W was received at t and no subsequent motion key arrives
- **WHEN** a send occurs at or after t+250 ms
- **THEN** the transmitted setpoint is zero

#### Scenario: Stop wins over buffered movement

- **WHEN** a read batch contains movement and Space or quit
- **THEN** movement is discarded and no later queued key replays it

### Requirement: Periodic stream and scheduling failure

While connected, the client SHALL send the current zero or armed intent every 50 ms with wrapping u32 setpoint sequences. It SHALL process received stops and input expiry before each send. A send-loop gap of at least 200 ms SHALL clear intent and end the run through cleanup; catching up SHALL not send a backlog of motion. Host scheduling SHALL not be described as a guaranteed physical stopping bound.

#### Scenario: Delayed loop cannot replay motion

- **GIVEN** nonzero intent and a 200 ms gap since the last send-loop iteration
- **WHEN** the client resumes
- **THEN** it sends no further nonzero setpoint and enters cleanup

### Requirement: Validate and correlate device input

The client SHALL validate device payload lengths and codes. UDP replies SHALL match the selected endpoint, version and current session, with Status matching a pending control sequence or unsolicited zero. BLE input SHALL belong to the current connection generation and its native GATT operation. Wrong-peer, malformed, stale or unmatched input SHALL not authorize motion or extend deadlines. Unsolicited stopped Status SHALL disable local motion before the next send.

#### Scenario: Queued notification precedes an ARM reply

- **WHEN** a stopped notification arrives before the matching ARM reply/readback
- **THEN** the client clears motion and does not confuse the notification with successful ARM

#### Scenario: Intruder packet cannot enable movement

- **WHEN** another endpoint, session or closed BLE connection sends Armed Status
- **THEN** local arming remains unchanged

### Requirement: Link checks with disarmed recovery

The client SHALL send matched Echo every 250 ms with a 500 ms reply deadline. Missed Echo/control deadlines or socket failures SHALL clear motion, attempt bounded cleanup and retry after one second until quit. Recovery SHALL require a fresh claim and matched Echo, discard outage input, and remain disarmed until a new R and subsequent direction. Echo SHALL not prove ownership or arming. Terminal loss and scheduler failures SHALL remain fatal.

#### Scenario: Lost link requires a new operator session

- **WHEN** an Echo or ARM reply deadline expires
- **THEN** the client clears intent, reports reconnecting and attempts zero/STOP/BYE
- **AND** it retries HELLO with at least one second between attempts, handling quit during the wait

#### Scenario: Recovery cannot replay buffered keys

- **GIVEN** R and direction keys were received while disconnected or verifying recovery
- **WHEN** a fresh claim and Echo succeed
- **THEN** the client reports reconnected and remains disarmed at zero
- **AND** only a new R followed by a new direction can move

#### Scenario: Quit during outage

- **WHEN** Esc, Ctrl-C or Ctrl-D is received while reconnecting
- **THEN** the client attempts bounded cleanup, restores the terminal and exits without another claim

### Requirement: Bounded shutdown and honest reporting

Space SHALL clear motion and request STOP; quit, Ctrl-C, EOF and fatal errors SHALL exit through cleanup. Recoverable link errors SHALL clean up before retrying. UDP cleanup SHALL attempt zero, STOP and BYE within 500 ms total even if an earlier send fails. Exit SHALL restore terminal settings. Matching stopped/Stop Status SHALL be required for STOP acknowledgement. Unacknowledged STOP and BYE SHALL be labelled attempts, never verified wheel stop or release.

#### Scenario: Cleanup tolerates a failed send

- **WHEN** sending zero or STOP fails during UDP cleanup
- **THEN** BYE and terminal restoration are still attempted within the cleanup deadline

### Requirement: Observable commands and evidence limits

The client SHALL show connection/local-arm state, UDP session or BLE connection generation, last valid device Status/reply age and requested vx/vy/wz. Last-reported Status SHALL be distinguished from live physical state; Echo SHALL not refresh Status age. Documentation SHALL identify open-loop speeds as approximate and cleanup as best effort. Simulator/fake-GATT passes SHALL not count as wheel, radio or electrical acceptance.

#### Scenario: A responsive rebooted device does not prove ownership

- **GIVEN** a UDP car reboots and answers Echo but does not accept the old session
- **WHEN** the client displays its state
- **THEN** it shows Status as last reported with its original age, sends no new HELLO merely because Echo replies, and does not claim ownership recovery

### Requirement: Audible connection loss

On a connected session's first link failure the client SHALL issue one brief laptop audio alert and display loss/retry state. Repeated failures before a fresh claim and matched Echo SHALL not repeat the alert. A later outage SHALL alert again. Audio failure SHALL not prevent recovery or quit; unavailable direct audio SHALL be reported with a terminal-bell fallback. Audio playback SHALL be bounded and occur after motion is disabled.

#### Scenario: One alert for an outage

- **WHEN** a connected link fails and repeated reconnection attempts fail
- **THEN** one alert is attempted for that outage
- **AND** a new failure after confirmed recovery triggers a new alert

#### Scenario: Audio unavailable

- **WHEN** direct audio cannot play
- **THEN** the client reports the limitation, attempts a terminal bell and continues disarmed recovery

### Requirement: BLE target and GATT validation

BLE SHALL discover/connect within five seconds each, select one advertising Rovi M8 or an explicit MAC, and pin that address for retries. Ambiguous names SHALL fail. Before controls it SHALL validate the car service, characteristic properties, capabilities and 72-byte write-without-response capacity (ATT MTU at least 75). Incompatible GATT SHALL fail closed.

#### Scenario: Unsafe or ambiguous peer cannot move

- **WHEN** discovery is ambiguous, the service is missing, or setpoint capacity is below 72 bytes
- **THEN** the client reports the limitation, disconnects any connection and sends no ARM or nonzero setpoint

#### Scenario: Name discovery pins the device for recovery

- **GIVEN** name discovery selected one device
- **WHEN** its connection drops and another Rovi M8 advertises
- **THEN** retries target the original device's MAC and cannot roam to the other car

### Requirement: BLE control confirmation and callback lifetime

BLE SHALL use native u32-sequenced setpoints and ARM/STOP bytes. ARM SHALL send zero first, succeed at ATT level and obtain fresh Armed/clear Status before enabling local motion. Notifications alone SHALL not authorize ARM. STOP, a safety notification or newer intent SHALL invalidate a pending ARM confirmation. Callbacks/Echo from an earlier connection generation SHALL not affect current state. Malformed Status SHALL not authorize motion.

#### Scenario: Stop wins over an in-flight ARM

- **WHEN** Space or a safety stop arrives while an ARM write/readback is pending
- **THEN** the eventual Armed readback cannot re-enable local motion

#### Scenario: Closed connection callback arrives late

- **WHEN** an old connection's Armed Status or Echo notification arrives after reconnect
- **THEN** it cannot arm, satisfy current Echo or change current Status

### Requirement: BLE loss and recovery

BLE SHALL check matched Echo every 250 ms with a 500 ms deadline. Disconnect, GATT timeout or missing Echo SHALL clear intent, attempt bounded cleanup, alert once per outage and retry after one second. Initial and recovered connections SHALL require a fresh matching Echo before accepting R. Recovery SHALL discard outage input, start disarmed and require a new R and subsequent direction. Quit SHALL work during discovery, setup and retry.

#### Scenario: Link returns with old keys pending

- **WHEN** BLE reconnects after R/motion were typed during loss
- **THEN** it reports reconnected disarmed only after matching Echo
- **AND** it stays zero until fresh R and direction; failed retries do not repeat the loss beep

### Requirement: BLE ownership and shutdown

BLE connection SHALL be treated as the ownership claim and its lifetime as identity, without invented UDP session/release acknowledgements. Only the latest intent and a single pending control action SHALL be retained. Cleanup SHALL attempt zero, STOP/readback and disconnect within 500 ms total, even when earlier operations fail. Successful disconnect SHALL be reported separately from physical stop. The client SHALL not manage Wi-Fi policy or switch transports automatically.

#### Scenario: Write fails during cleanup

- **WHEN** zero or STOP fails or times out
- **THEN** disconnect and terminal restoration are still attempted within their bounds
- **AND** missing confirmation is reported honestly
