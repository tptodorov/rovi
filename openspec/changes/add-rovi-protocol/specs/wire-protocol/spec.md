## Purpose

Define every car message once, as typed Rust values, and use that definition on the car, the sim car and every client, so the protocol is checked by the compiler and by one conformance suite on both sim and device.

## ADDED Requirements

### Requirement: Single protocol definition

All BLE, UDP and ROS message layouts, identifiers, codes, limits and the protocol version SHALL be defined in one `no_std`, allocation-free Rust crate. The firmware, the sim car and all clients and test tools SHALL encode and decode car messages only through that crate or its bindings. No participant SHALL keep a separate hand-written codec once its migration completes.

#### Scenario: A layout changes

- **WHEN** a field of a car message is changed in the protocol crate
- **THEN** the firmware, sim car, Python tools and phone module all pick up the change from that crate, with no other codec to edit

### Requirement: Validated symmetric codecs

Every message SHALL have an encoder and a validating decoder. Decoding SHALL reject wrong lengths, unknown codes, non-zero reserved bytes, non-finite floats and unsupported versions with a named error. For valid values, decoding an encoding SHALL return the identical value with bit-exact floats; for canonical bytes, encoding a decoded value SHALL return the identical bytes. Encoding into a short buffer SHALL fail without partial output.

#### Scenario: Malformed Status from a device

- **WHEN** a client decodes a Status with an unknown state code
- **THEN** decoding fails with a named error and no Status value is produced

#### Scenario: Round trip

- **WHEN** any valid message value is encoded and decoded on any supported target
- **THEN** the result equals the original value

### Requirement: Direction-typed messages

The crate SHALL separate messages a client sends to the car from messages the car sends. Client APIs SHALL only encode car-bound messages and decode client-bound messages; car APIs SHALL do the reverse.

#### Scenario: Wrong direction

- **WHEN** client code tries to send a Status frame
- **THEN** it does not compile against the client API, or the binding rejects it with a named error

### Requirement: One model per concept

Each protocol concept (Setpoint, Command, Status, Capabilities, Network information, Echo) SHALL be one type, with its BLE, UDP and ROS encodings defined beside it, including the ROS text form. Every encoding SHALL have a decoder and round-trip tests. Display and log names SHALL come from that type. Differences between transports SHALL be limited to those listed as intentional in the crate documentation, each with its reason.

#### Scenario: Same Status over three transports

- **WHEN** the car reports one Status over BLE, UDP and ROS
- **THEN** decoding each with the crate yields equal Status values, apart from fields that are documented as transport-specific

#### Scenario: Same validation in every client

- **WHEN** the laptop client and the phone receive the same malformed Capabilities, Status or Network bytes
- **THEN** both reject them with the same error kind

### Requirement: Single client minimum windows

The crate SHALL define one minimum for advertised windows that every client applies after decoding Capabilities: lease at least 500 ms, claim at least 1000 ms and nonzero reclaim. Clients SHALL NOT use their own minimums.

#### Scenario: Short lease

- **WHEN** a car advertises a 400 ms lease
- **THEN** the laptop and the phone both refuse the connection with the same error, without ARM or nonzero setpoints

### Requirement: Generated golden vectors

The crate SHALL generate a committed vector file of valid and malformed cases for every message. CI SHALL fail when the committed file differs from the generated one. Bytes for existing messages SHALL be identical to the protocol in force before extraction. A changed vector SHALL be accompanied by a recorded compatibility decision: compatible addition, or a protocol version change that decoders enforce.

#### Scenario: Unreviewed byte change

- **WHEN** a code change alters the encoding of an existing message without updating the vectors
- **THEN** CI fails

### Requirement: Same results on every binding

Rust, Python and WebAssembly consumers SHALL produce identical bytes, values and error kinds for every vector. Bindings SHALL contain no session policy. If the phone module cannot load, the phone SHALL keep connection disabled and explain why.

#### Scenario: Binding parity

- **WHEN** the vector suite runs natively, through the Python extension and through the WebAssembly module
- **THEN** all three agree on every case

#### Scenario: Phone module missing

- **WHEN** the phone page cannot load the protocol module
- **THEN** Connect stays disabled with an explanation and no fallback codec is used

### Requirement: Conformance on sim and device

One conformance suite, using only the shared protocol types, SHALL run against the sim car in CI and against the real car on the bench. Device runs of observational rules SHALL not arm and SHALL be done with motor power off. Motion rules on the device SHALL run only in a separately announced stand batch. Each run SHALL record target, firmware revision and vector hash. Sim results SHALL NOT count as device or radio acceptance.

#### Scenario: Device observational run

- **WHEN** the suite runs against the real car with motor power off
- **THEN** INFO, Echo, Capabilities, Status, Network, malformed-input rejection and busy/not-owner rules pass over UDP and BLE without any ARM
