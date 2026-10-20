# laptop-teleoperation Specification

## Purpose
Lets a laptop operator control the existing car UDP API interactively with explicit arming, bounded motion input, visible state and predictable failure cleanup.

## Requirements

### Requirement: Explicit connection without motion

The client SHALL require an explicit IPv4 target/port and an interactive terminal. It SHALL claim once with version-1 UDP HELLO, validate the returned nonzero session and capabilities, and start locally disarmed with zero intent. BUSY, invalid capabilities or a one-second handshake timeout SHALL fail without ARM or nonzero setpoints. Leases shorter than 200 ms SHALL be reported unsupported.

#### Scenario: Connection starts disarmed

- **WHEN** an operator connects to a valid target
- **THEN** the client displays the negotiated limits and session and sends no ARM or nonzero setpoint

#### Scenario: Refused or unsupported connection

- **WHEN** the target returns BUSY, invalid capabilities, an unsupported lease, or no valid reply within one second
- **THEN** the client reports failure and sends no ARM or nonzero setpoint

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

The client SHALL accept only correctly framed/versioned replies from the selected endpoint, validating payload lengths and codes. After claim, Status SHALL match the current session and either a pending control sequence or the unsolicited sequence zero. Wrong-peer, malformed, stale-session and unmatched replies SHALL not authorize motion or extend reply deadlines. Unsolicited stopped Status SHALL disable local motion before the next send.

#### Scenario: Queued notification precedes an ARM reply

- **WHEN** a stopped notification arrives before the matching ARM reply
- **THEN** the client clears motion and does not confuse the notification with a successful ARM

#### Scenario: Intruder packet cannot enable movement

- **WHEN** another endpoint or session sends an Armed Status
- **THEN** local arming remains unchanged

### Requirement: Link checks without automatic recovery

The client SHALL check link responsiveness using nonce/sequence-matched Echo every 250 ms, allowing 500 ms for each reply. Echo SHALL not be described as proof of ownership or arming. A missed Echo/control deadline, socket error or terminal loss SHALL clear intent and terminate through cleanup. The client SHALL send no HELLO after its initial handshake and SHALL never reconnect or re-arm automatically.

#### Scenario: Lost link requires a new operator session

- **WHEN** an Echo or ARM reply deadline expires
- **THEN** local motion stops, cleanup runs, and no automatic HELLO or ARM is sent

### Requirement: Bounded shutdown and honest reporting

Space SHALL clear motion and request STOP; quit, Ctrl-C, EOF and handled errors SHALL also exit through cleanup. Cleanup SHALL attempt zero, STOP and BYE within 500 ms total without indefinite waits, even if an earlier send fails, and restore terminal settings. Successful STOP acknowledgement SHALL require matching stopped/Stop Status. The client SHALL label unacknowledged stops and BYE as unconfirmed attempts, never verified wheel stop or release.

#### Scenario: Cleanup tolerates a failed send

- **WHEN** sending zero or STOP fails during cleanup
- **THEN** BYE and terminal restoration are still attempted within the cleanup deadline

### Requirement: Observable commands and evidence limits

The client SHALL show connection/local-arm state, session, last valid device Status, reply age and requested vx/vy/wz. Last-reported Status SHALL be distinguished from live physical state; Echo SHALL not refresh Status age. Documentation SHALL identify open-loop speeds as approximate and UDP cleanup as best effort. Simulator passes SHALL not count as wheel, radio or electrical acceptance.

#### Scenario: A responsive rebooted device does not prove ownership

- **GIVEN** the car reboots and answers Echo but does not accept the old session
- **WHEN** the client displays its state
- **THEN** it shows Status as last reported with its original age, sends no new HELLO and does not claim ownership recovery
