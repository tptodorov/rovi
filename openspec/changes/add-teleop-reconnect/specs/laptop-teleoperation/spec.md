## RENAMED Requirements

- FROM: `### Requirement: Link checks without automatic recovery`
- TO: `### Requirement: Link checks with disarmed recovery`

## MODIFIED Requirements

### Requirement: Explicit connection without motion

The client SHALL require an explicit IPv4 target/port and an interactive terminal. Each attempt SHALL claim with version-1 UDP HELLO and validate the nonzero session and capabilities. It SHALL start disarmed at zero. BUSY and a one-second handshake timeout SHALL retry without ARM or nonzero setpoints; invalid capabilities or leases below 200 ms SHALL fail. New attempts SHALL use different control sequences so stale ACKs cannot satisfy a later HELLO.

#### Scenario: Connection starts disarmed

- **WHEN** an operator connects to a valid target
- **THEN** the client displays negotiated limits and session and sends no ARM or nonzero setpoint

#### Scenario: Refused or unsupported connection

- **WHEN** the target returns BUSY or no valid handshake reply within one second
- **THEN** the client retries with no ARM or nonzero setpoint
- **AND** invalid capabilities or an unsupported lease terminate through cleanup

#### Scenario: Old handshake cannot reconnect

- **WHEN** an ACK from an earlier attempt arrives during a new handshake
- **THEN** it cannot establish the new session

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

Space SHALL clear motion and request STOP; quit, Ctrl-C, EOF and fatal errors SHALL exit through cleanup. Recoverable link errors SHALL clean up before retrying. Cleanup SHALL attempt zero, STOP and BYE within 500 ms total even if an earlier send fails. Exit SHALL restore terminal settings. Successful STOP acknowledgement SHALL require matching stopped/Stop Status. The client SHALL label unacknowledged stops and BYE as unconfirmed attempts, never verified wheel stop or release.

#### Scenario: Cleanup tolerates a failed send

- **WHEN** sending zero or STOP fails during cleanup
- **THEN** BYE and terminal restoration are still attempted within the cleanup deadline

## ADDED Requirements

### Requirement: Audible connection loss

On a connected session's first link failure the client SHALL issue one brief laptop audio alert and display loss/retry state. Repeated failures before a fresh claim and matched Echo SHALL not repeat the alert. A later outage SHALL alert again. Audio failure SHALL not prevent recovery or quit; unavailable direct audio SHALL be reported with a terminal-bell fallback. Audio playback SHALL be bounded and occur after motion is disabled.

#### Scenario: One alert for an outage

- **WHEN** a connected link fails and repeated reconnection attempts fail
- **THEN** one alert is attempted for that outage
- **AND** a new failure after confirmed recovery triggers a new alert

#### Scenario: Audio unavailable

- **WHEN** direct audio cannot play
- **THEN** the client reports the limitation, attempts a terminal bell and continues disarmed recovery
