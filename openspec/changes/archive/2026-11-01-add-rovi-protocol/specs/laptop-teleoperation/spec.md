## MODIFIED Requirements

### Requirement: Explicit connection without motion

The client SHALL require an interactive terminal and exactly one transport: explicit UDP IPv4:port or BLE discovery with optional MAC. Each UDP attempt SHALL validate HELLO's nonzero session/capabilities and use fresh control sequences. BLE SHALL validate GATT/capabilities before controls. All connections SHALL start disarmed at zero. BUSY/unavailable targets SHALL retry without motion; invalid capabilities or windows below the shared client minimum (lease 500 ms, claim 1000 ms, nonzero reclaim) SHALL fail.

#### Scenario: Connection starts disarmed

- **WHEN** an operator connects to a valid UDP or BLE target
- **THEN** the client displays negotiated limits and connection identity and sends no ARM or nonzero setpoint

#### Scenario: Refused or unsupported connection

- **WHEN** UDP returns BUSY, its one-second handshake expires, or BLE discovery/connect fails
- **THEN** the client retries with no ARM or nonzero setpoint
- **AND** invalid capabilities or windows below the shared client minimum terminate through cleanup

#### Scenario: Old handshake cannot reconnect

- **WHEN** a UDP ACK from an earlier attempt arrives during a new handshake
- **THEN** it cannot establish the new session
