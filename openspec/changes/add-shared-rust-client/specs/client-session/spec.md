## Purpose

Let native and browser applications share validated car protocol exchange and session decisions while preserving each accepted client's input, transport and recovery behavior.

## ADDED Requirements

### Requirement: Portable validated protocol exchange

The client library SHALL provide the same typed protocol values, encodings and validation results on native and WebAssembly targets. It SHALL preserve existing BLE, UDP and ROS message layouts, canonical advertised limits and wrapping u32 ordering. Malformed, unsupported or non-finite device data SHALL NOT authorize motion. Invalid optional Network information SHALL disable network actions while preserving otherwise valid control.

#### Scenario: Cross-target protocol agreement

- **WHEN** native and WebAssembly consumers process the same valid protocol vectors, including zero, all directions, Network and sequence wrap
- **THEN** they produce identical wire bytes and typed values matching the accepted device contract

#### Scenario: Invalid device input

- **WHEN** required capabilities or Status are malformed, inconsistent or unsupported
- **THEN** setup fails or the active session closes without granting motion, following the selected client profile
- **AND** malformed optional Network data alone cannot arm or select a network

### Requirement: Deterministic session decisions

Given the same client profile, monotonic times, user intent, device input and operation results, the library SHALL produce the same session state and ordered requested operations on both targets. Old connection or operation results SHALL NOT alter a new session. One normal GATT procedure SHALL be pending at a time; bounded shutdown SHALL retain its independent best-effort behavior. The library SHALL NOT execute host operations itself.

#### Scenario: Replay native and browser traces

- **WHEN** the same profile's trace includes delayed writes, STOP during ARM and a reconnect with an old Armed callback
- **THEN** native and WebAssembly runs produce equivalent decisions and the old callback grants no motion

### Requirement: Explicit manual motion permission

Manual-control sessions SHALL start disarmed at zero. ARM SHALL require a new operator action, fresh zero and the profile's matching successful device confirmation. STOP, safety stops, failure and ended sessions SHALL clear intent and invalidate pending ARM. A new ARM SHALL NOT replay pre-arm input. Connection, Echo, metadata and network requests SHALL NOT grant motion. Reported state SHALL remain distinct from local permission.

#### Scenario: Stale motion after recovery

- **WHEN** input predates ARM, arrives during disconnection or belongs to an ended session
- **THEN** it cannot produce nonzero output after a fresh connection or ARM; a new permitted input action is required

### Requirement: Fresh intent and bounded work

The library SHALL enforce the selected profile's accepted deadlines before normal output and after asynchronous completion. Intent expires after 250 ms without its eligible renewal; periodic transmission or cached input SHALL NOT renew it. Latest intent SHALL replace previous intent without a motion backlog. A stream gap of at least 200 ms SHALL end normal control without catch-up motion. Runnable cleanup SHALL retain its 500 ms total budget and honest confirmation reporting.

#### Scenario: Completion at the failure bound

- **WHEN** an operation completes at or after the stream or control deadline
- **THEN** the library clears motion and requests cleanup rather than issuing another nonzero setpoint

#### Scenario: Cleanup cannot finish its writes

- **WHEN** zero or STOP fails or never completes
- **THEN** remaining cancellation/release attempts remain bounded by the original cleanup deadline and unconfirmed physical effects stay unconfirmed

### Requirement: Preserve distinct client profiles

The library SHALL preserve the accepted laptop and phone behavior instead of imposing one recovery policy. Laptop recovery SHALL retain target pinning, matched Echo, outage notification and disarmed retry. Phone recovery SHALL require manual selection/connect and retain foreground, physical-input and confirmation gates. BLE identity SHALL remain a connection generation; UDP replies SHALL retain endpoint/session/sequence correlation. No migration SHALL switch transports automatically.

#### Scenario: Same loss on different hosts

- **WHEN** an established laptop session and an established phone session each lose their transport
- **THEN** both revoke motion, the laptop follows its disarmed retry policy and the phone waits for manual recovery

### Requirement: Network request and observation semantics

Shared sessions SHALL retain each client's current mode-command confirmation and release procedure. A phone request SHALL require an acknowledged write, fresh disarmed error-free Status and matching pending Network before cleanup/disconnect. Rejected writes SHALL obtain fresh Status to distinguish refusal from unconfirmed work. Disconnected information SHALL remain last-reported; pending mode SHALL NOT prove the requested network was joined.

#### Scenario: Phone changes network

- **WHEN** a disarmed phone requests Home or Car network and fresh reads confirm it
- **THEN** the session disconnects without ARM or nonzero output and requires reconnect to obtain the resulting mode

#### Scenario: Failed write with old pending metadata

- **WHEN** a write fails but retained Network happens to contain the requested pending mode
- **THEN** the session cannot report that request as successfully confirmed

### Requirement: Migration preserves accepted operation

Each replacement SHALL preserve its client's accepted controls, limits, failure behavior and observable state before becoming the normal entry point. The accepted implementation SHALL remain available until that migration's host and separately authorized physical checks pass. Missing or failed WebAssembly initialization SHALL leave phone connection/control disabled with an explanation. Library or host tests SHALL NOT count as phone, radio or wheel acceptance.

#### Scenario: Replacement fails its acceptance gate

- **WHEN** native BLE, browser loading or a required physical check fails
- **THEN** that replacement is not promoted and the accepted client remains available
