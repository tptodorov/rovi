## MODIFIED Requirements

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

### Requirement: Bounded shutdown and honest reporting

Space SHALL clear motion and request STOP; quit, Ctrl-C, EOF and fatal errors SHALL exit through cleanup. Recoverable link errors SHALL clean up before retrying. UDP cleanup SHALL attempt zero, STOP and BYE within 500 ms total even if an earlier send fails. Exit SHALL restore terminal settings. Matching stopped/Stop Status SHALL be required for STOP acknowledgement. Unacknowledged STOP and BYE SHALL be labelled attempts, never verified wheel stop or release.

#### Scenario: Cleanup tolerates a failed send

- **WHEN** sending zero or STOP fails during UDP cleanup
- **THEN** BYE and terminal restoration are still attempted within the cleanup deadline

### Requirement: Validate and correlate device input

The client SHALL validate device payload lengths and codes. UDP replies SHALL match the selected endpoint, version and current session, with Status matching a pending control sequence or unsolicited zero. BLE input SHALL belong to the current connection generation and its native GATT operation. Wrong-peer, malformed, stale or unmatched input SHALL not authorize motion or extend deadlines. Unsolicited stopped Status SHALL disable local motion before the next send.

#### Scenario: Queued notification precedes an ARM reply

- **WHEN** a stopped notification arrives before the matching ARM reply/readback
- **THEN** the client clears motion and does not confuse the notification with successful ARM

#### Scenario: Intruder packet cannot enable movement

- **WHEN** another endpoint, session or closed BLE connection sends Armed Status
- **THEN** local arming remains unchanged

### Requirement: Observable commands and evidence limits

The client SHALL show connection/local-arm state, UDP session or BLE connection generation, last valid device Status/reply age and requested vx/vy/wz. Last-reported Status SHALL be distinguished from live physical state; Echo SHALL not refresh Status age. Documentation SHALL identify open-loop speeds as approximate and cleanup as best effort. Simulator/fake-GATT passes SHALL not count as wheel, radio or electrical acceptance.

#### Scenario: A responsive rebooted device does not prove ownership

- **GIVEN** a UDP car reboots and answers Echo but does not accept the old session
- **WHEN** the client displays its state
- **THEN** it shows Status as last reported with its original age, sends no new HELLO merely because Echo replies, and does not claim ownership recovery

## ADDED Requirements

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
