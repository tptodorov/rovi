## MODIFIED Requirements

### Requirement: BLE target and GATT validation

BLE SHALL discover/connect within five seconds each, select one advertising `Rovi` or an explicit MAC, and pin that address for retries. Ambiguous names SHALL fail. Before controls it SHALL validate the car service, characteristic properties, capabilities and 72-byte write-without-response capacity (ATT MTU at least 75). Incompatible GATT SHALL fail closed.

#### Scenario: Unsafe or ambiguous peer cannot move

- **WHEN** discovery is ambiguous, the service is missing, or setpoint capacity is below 72 bytes
- **THEN** the client reports the limitation, disconnects any connection and sends no ARM or nonzero setpoint

#### Scenario: Name discovery pins the device for recovery

- **GIVEN** name discovery selected one device
- **WHEN** its connection drops and another `Rovi` advertises
- **THEN** retries target the original device's MAC and cannot roam to the other car
