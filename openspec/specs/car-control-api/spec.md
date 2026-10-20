# car-control-api Specification

## Purpose
Defines how controllers request car motion, acquire ownership, arm, stop and inspect capabilities and status consistently across the supported transports.

## Requirements

### Requirement: Setpoint

The car's device control API SHALL accept a setpoint as a `geometry_msgs/TwistStamped`, using `linear.x` and `linear.y` in m/s and `angular.z` in rad/s, in `base_link`. A setpoint with any other Twist component non-zero SHALL be rejected and not applied. An owner's fresh, finite planar setpoint whose absolute `linear.x`, `linear.y` or `angular.z` exceeds its capability limit SHALL raise a safety stop.

#### Scenario: Out-of-plane setpoint is rejected

- **GIVEN** the car is armed
- **WHEN** the owner sends a setpoint with `linear.z` = 0.1
- **THEN** the setpoint is not applied and does not renew the lease
- **AND** no safety stop is raised

#### Scenario: Out-of-limits setpoint stops the car

- **GIVEN** the car is armed
- **WHEN** the owner sends a setpoint with `linear.x` above the capabilities' maximum
- **THEN** a safety stop is raised with reason out-of-limits

### Requirement: Mecanum kinematics

The car SHALL use `mecanum_drive_controller` inverse kinematics and parameter names, divide wheel-speed references by configured `max_wheel_speed`, and clamp to −1.0..=1.0. The current geometry SHALL use radius 0.030 m and centre projection sum 0.1275 m, with zero frame offset. The profile's 40 rad/s normalization and 1/1/3 input limits are experimental values, not measured physical maxima; speed tracking is approximate (O2).

#### Scenario: Same result as mecanum_drive_controller

- **GIVEN** a `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis` and `max_wheel_speed`
- **WHEN** the car is armed and applies a setpoint
- **THEN** each wheel command (FL, FR, RR, RL) equals its inverse-kinematic wheel speed divided by `max_wheel_speed`, clamped independently to −1.0..=1.0

#### Scenario: Same input on every source gives the same wheel commands

- **GIVEN** the same setpoint
- **WHEN** it is sent by an armed owner over BLE, UDP and ROS 2 in turn
- **THEN** the wheel commands are identical

#### Scenario: Combined inputs saturate independently

- **GIVEN** radius 0.03 m, zero frame offset, normalization 40 rad/s and input limits allowing vx=1 and vy=1
- **WHEN** the armed owner sends vx=1, vy=1, wz=0
- **THEN** normalized commands are (0, 1, 0, 1), not (0, 1.667, 0, 1.667)

### Requirement: Open-loop motion references

Twist fields SHALL retain their standard m/s and rad/s meaning as requested motion references. Product documentation SHALL describe current open-loop tracking as approximate and link the M5 observations. The car SHALL NOT present command-derived speed as measured velocity or promise velocity accuracy without supporting evidence.

#### Scenario: An estimate is not measured feedback

- **GIVEN** the current motors have no encoders and only short-pulse observations are available
- **WHEN** the product describes speed or publishes status
- **THEN** observed motion estimates are identified as approximate
- **AND** commanded velocity is not presented as measured feedback

### Requirement: Latest setpoint wins

The car SHALL apply only the owner's latest setpoint, with no queue. A setpoint whose sequence is not newer than the last accepted one SHALL NOT be applied.

#### Scenario: Older sequence is ignored

- **GIVEN** the current BLE/UDP session, or one ROS publisher, last accepted setpoint sequence 10
- **WHEN** a setpoint with sequence 9 or 10 arrives
- **THEN** it is not applied and does not renew the lease

### Requirement: Single owner

The first controller to claim SHALL become the owner. Other controllers' claims SHALL be refused and their control input ignored. Repeated UDP HELLO from the current owner SHALL return its existing session without renewing deadlines. BLE ownership SHALL disable the AP; UDP or ROS ownership SHALL stop BLE advertising. UDP and ROS share the AP and SHALL exclude one another through ownership checks, not radio shutdown.

#### Scenario: Second claimant is refused

- **GIVEN** a UDP controller owns the car
- **WHEN** a second UDP controller sends a hello
- **THEN** it receives `BUSY`
- **AND** the owner is unchanged

#### Scenario: Other radio is off while owned

- **GIVEN** the car is open
- **WHEN** a BLE controller connects
- **THEN** it becomes the owner and the car AP goes off
- **AND** when ownership is released, BLE advertising and the car AP are both up

#### Scenario: Non-owner ROS node is ignored

- **GIVEN** a ROS 2 node owns the car
- **WHEN** another node publishes a setpoint or an arm
- **THEN** it is ignored

#### Scenario: UDP and ROS share the Wi-Fi radio

- **GIVEN** a UDP owner and a connected ROS router on the car AP
- **WHEN** a ROS node sends a setpoint
- **THEN** the AP stays up, the setpoint cannot claim or drive, and the UDP lease is unchanged
- **AND** the same exclusion applies to UDP input while a ROS node owns the car

### Requirement: Claim window

If the owner sends no valid setpoint within the claim window, ownership SHALL be released.

#### Scenario: Silent owner loses the car

- **GIVEN** a controller has just claimed the car
- **WHEN** the claim window passes with no valid setpoint from it
- **THEN** ownership is released and every radio reopens

### Requirement: Explicit arming

New owners SHALL start disarmed. BLE/UDP SHALL require the owner's explicit arm and a valid lease. ROS SHALL auto-arm on an owner's valid setpoint only after a claim or lease expiry ([ADR-0011](../../../docs/adr/0011-ros-auto-arm.md)). After STOP or out-of-limits, all sources SHALL require a fresh setpoint followed by explicit arm. Disarmed setpoints SHALL renew the lease while wheel commands remain zero.

#### Scenario: Setpoints alone do not move the car

- **GIVEN** a new BLE or UDP owner that has not armed
- **WHEN** it streams non-zero setpoints
- **THEN** every wheel command is zero

#### Scenario: Arm without a lease is refused

- **GIVEN** a new BLE or UDP owner that has sent no setpoint
- **WHEN** it sends arm
- **THEN** arm is refused with not-ready and the car stays disarmed

#### Scenario: A ROS 2 stream arms the car

- **GIVEN** the car is open
- **WHEN** a known ROS 2 node streams valid non-zero setpoints on `/cmd_vel` and nothing on `/rovi/arm`
- **THEN** the first setpoint claims and arms the car and the wheel commands follow it
- **AND** after a lease expiry, the resumed stream arms the car again

#### Scenario: A ROS 2 stop stays latched while streaming

- **GIVEN** a ROS 2 owner that is streaming and armed
- **WHEN** it publishes `/rovi/arm` false and keeps streaming
- **THEN** the car stops and every wheel command stays zero until the owner publishes `/rovi/arm` true

#### Scenario: Arm then drive

- **GIVEN** an owner with a fresh valid setpoint, a valid lease and valid motor configuration
- **WHEN** it sends arm, then a non-zero setpoint
- **THEN** the car is armed and the wheel commands follow the setpoint

#### Scenario: Arm after a stop needs a new setpoint

- **GIVEN** the owner sent stop while its lease was valid
- **WHEN** it sends arm before any new setpoint
- **THEN** arm is refused with not-ready and the car stays in safety stop
- **AND** after one valid setpoint, the next arm succeeds

### Requirement: Lease and watchdog timeout

Only the owner's valid setpoints SHALL renew the lease. When the lease expires, the car SHALL raise a watchdog timeout and a safety stop within the lease length plus 50 ms (M8's acceptance bound), checked by the control tick independently of incoming traffic.

#### Scenario: Silent armed owner is stopped

- **GIVEN** the car is armed
- **WHEN** no valid setpoint arrives for longer than the lease
- **THEN** a safety stop is raised with reason lease-expired, within the lease plus 50 ms
- **AND** every wheel command is zero

### Requirement: Safety stop

STOP, lease expiry, owner loss and out-of-limits SHALL zero wheel commands, put both drivers in standby and disarm. Ownership SHALL be retained except on owner loss or window expiry. STOP/out-of-limits SHALL require a fresh valid setpoint then explicit ARM. Lease expiry SHALL allow ROS auto-arm on a fresh valid setpoint; BLE/UDP SHALL still require explicit ARM. Release SHALL clear session state while keeping drivers disabled.

#### Scenario: Stop keeps the owner

- **GIVEN** the car is armed
- **WHEN** the owner sends stop
- **THEN** the car is in safety stop with reason stop, disarmed, with every driver in standby
- **AND** the controller is still the owner

#### Scenario: Owner loss while armed

- **GIVEN** the car is armed
- **WHEN** the owner disconnects or sends bye
- **THEN** a safety stop is raised with reason owner-lost
- **AND** ownership is released

#### Scenario: Further setpoints do not clear the latch

- **GIVEN** the car stopped because of STOP or out-of-limits
- **WHEN** the owner streams valid setpoints without explicit ARM
- **THEN** every wheel command stays zero

### Requirement: Reclaim window

After a stop or an out-of-limits setpoint, the owner SHALL be kept for as long as its valid setpoints keep the lease valid. The reclaim window SHALL start when the lease lapses: at once on a lease expiry, or when the owner goes quiet after a stop or an out-of-limits setpoint. Within the window, the owner SHALL be able to resume in the same session using the explicit/automatic arming rules. When the window ends, ownership SHALL be released.

#### Scenario: Streaming owner is kept after a stop

- **GIVEN** the owner sent stop
- **WHEN** it keeps streaming valid setpoints for longer than the reclaim window
- **THEN** it is still the owner and the car stays in safety stop

#### Scenario: Quiet owner after a stop is released

- **GIVEN** the owner sent stop
- **WHEN** it sends no setpoint after that
- **THEN** the reclaim window starts when its lease lapses
- **AND** ownership is released when the window ends, and every radio reopens

#### Scenario: Out-of-limits keeps a streaming owner

- **GIVEN** an out-of-limits setpoint raised a safety stop
- **WHEN** the owner keeps streaming valid setpoints and then sends arm
- **THEN** the car is armed in the same session

#### Scenario: Resume within the window

- **GIVEN** the car stopped on lease expiry
- **WHEN** the same owner sends a valid setpoint and arm within the reclaim window
- **THEN** the car is armed in the same session

#### Scenario: Window ends

- **GIVEN** the car stopped on lease expiry
- **WHEN** the reclaim window passes without explicit or permitted automatic arming
- **THEN** ownership is released and every radio reopens

### Requirement: Boot state

After power-up or reset, the car SHALL be open, with no owner, disarmed, every wheel command zero and every driver in standby.

#### Scenario: Fresh boot

- **GIVEN** the car has just booted
- **WHEN** a controller reads the status
- **THEN** the state is open
- **AND** the car log reports every wheel command as zero

### Requirement: Capabilities

The car SHALL give its capabilities on every source: protocol version, `wheels_radius`, `sum_of_robot_center_projection_on_X_Y_axis`, maximum `linear.x`, `linear.y` and `angular.z`, lease length, claim window and reclaim window. BLE (Capabilities characteristic) and UDP (`HELLO_ACK`) give them to the owner on claim; ROS 2 gives them before the claim, as below. Each value SHALL be the one the car enforces, so a controller needs no hardcoded limits.

#### Scenario: Limits match enforcement

- **GIVEN** a controller has read the capabilities
- **WHEN** it sends a setpoint at exactly the maximum `linear.x`
- **THEN** the setpoint is applied
- **AND** a setpoint above it raises a safety stop

#### Scenario: Same capabilities on every source

- **GIVEN** the same car
- **WHEN** a BLE, a UDP and a ROS 2 controller each read the capabilities
- **THEN** every value is identical

### Requirement: ROS 2 capabilities topic

A ROS 2 controller claims with its first setpoint, so ROS 2 SHALL give the capabilities without a claim. The car SHALL publish them on `/rovi/capabilities` as `std_msgs/String` of space-separated `key=value` pairs, latched: reliable, transient local, depth 1. O9 selects this QoS or the specified volatile fallback; the published values are identical in both modes.

#### Scenario: Late subscriber gets the capabilities

- **GIVEN** the car is open and connected to the ROS 2 router
- **WHEN** a node starts and subscribes to `/rovi/capabilities`
- **THEN** it receives the capabilities without sending anything to the car

#### Scenario: One pair per field

- **GIVEN** a node has received `/rovi/capabilities`
- **WHEN** it splits the string on spaces and `=`
- **THEN** it gets exactly one value for every key in the capabilities encoding requirement

#### Scenario: Reading the capabilities is not a claim

- **GIVEN** the car is open
- **WHEN** a ROS 2 node reads `/rovi/capabilities`
- **THEN** the car stays open and every radio stays up

### Requirement: ROS 2 capabilities republish fallback

If O9 shows that zenoh-nostd cannot serve transient-local late joiners, the car SHALL use reliable, volatile, depth-1 capabilities and republish at least once per second and on every claim, release and router reconnect. Controllers SHALL subscribe with volatile durability in fallback mode. Documentation SHALL identify the chosen mode; the car SHALL NOT advertise unsupported transient-local durability.

#### Scenario: Late subscriber gets the capabilities without latching

- **GIVEN** the car uses the republish fallback
- **WHEN** a node with compatible volatile QoS subscribes while the car is open and the router is healthy
- **THEN** it receives the capabilities within one second

### Requirement: Status

The car SHALL report its status to the owner on every source (BLE Status characteristic, UDP `STATUS`, ROS 2 `/rovi/status`): state (open, claimed, armed or stopped), stop reason (none, stop, lease-expired, owner-lost or out-of-limits) and last owner control error (none, not-ready or invalid). Status SHALL be readable at once after a claim. Every status read or notification SHALL report the state at that moment.

#### Scenario: Status follows a stop

- **GIVEN** the car is armed and the owner reads status
- **WHEN** the lease expires
- **THEN** the next status reports safety stop with reason lease-expired

#### Scenario: Readable right after the claim

- **GIVEN** a BLE controller has just connected
- **WHEN** it reads Status before any other write
- **THEN** the status reports claimed with no error

### Requirement: ROS 2 status topic

The car SHALL publish `/rovi/status` as `std_msgs/String` of space-separated `key=value` pairs with the same content as the other sources' status: `state` (open, claimed, armed or stopped), `reason` (none except while stopped), `error` (none when clear) and `session` (decimal u32, 0 while open). It SHALL publish on every change of any of these and at least once a second, so a late subscriber sees the current status within a second.

#### Scenario: Stop reason on ROS 2

- **GIVEN** the car is armed under a ROS 2 owner
- **WHEN** the lease expires
- **THEN** the next `/rovi/status` has `state=stopped` and `reason=lease-expired`

#### Scenario: Refused arm on ROS 2

- **GIVEN** a ROS 2 owner stopped by its own STOP
- **WHEN** it publishes arm before any new setpoint
- **THEN** the next `/rovi/status` has `state=stopped` and `error=not-ready`

#### Scenario: Status on a change

- **GIVEN** a node subscribed to `/rovi/status`
- **WHEN** the state changes
- **THEN** it receives the new status without waiting for the next periodic publish

### Requirement: Last command error

On every source, a refused owner control command (ARM/STOP; not-ready or invalid) SHALL persist as the last error until the owner's next successful control command. Setpoints and echo SHALL NOT clear it. New sessions SHALL start with no error. Non-owner or old-session input SHALL NOT change it. BUSY is a response to a claimant, not an owner-status error.

#### Scenario: Refused arm shows on every source

- **GIVEN** a BLE, UDP or ROS owner that sent STOP and has sent no valid setpoint since
- **WHEN** it sends explicit ARM
- **THEN** the status reports the last command error not-ready
- **AND** a later setpoint does not clear it, and a successful arm does

#### Scenario: Non-owner commands do not change the owner's status

- **GIVEN** an owner whose status reports no error
- **WHEN** another controller sends arm
- **THEN** the owner's status still reports no error

### Requirement: Input validation before state changes

Malformed framing/CDR, non-finite Twist components, non-planar Twist, invalid Bool/control payloads, and unknown ROS publishers SHALL NOT claim, renew a lease, change outputs or consume a setpoint sequence. An identifiable owner's invalid control payload SHALL report invalid; invalid setpoint/framing SHALL be discarded. Owner/session and sequence checks SHALL precede limit enforcement, so an intruder or stale packet cannot stop the car.

#### Scenario: Invalid setpoint leaves the stream intact

- **GIVEN** an armed owner with last sequence 10
- **WHEN** sequence 11 contains NaN or truncated CDR
- **THEN** the previous output and lease deadline stay unchanged
- **AND** a later valid sequence 11 can be accepted

#### Scenario: Stale or non-owner oversized input cannot stop the owner

- **GIVEN** an armed owner with last sequence 10
- **WHEN** sequence 9 from that owner, or input from a non-owner, exceeds a velocity limit
- **THEN** it is ignored without changing the state or renewing the lease

#### Scenario: Bad first ROS input does not claim

- **GIVEN** the car is open and a known ROS publisher
- **WHEN** it sends a non-finite or out-of-limits setpoint
- **THEN** the car stays open and disabled

### Requirement: Setpoint frame and time interpretation

Empty `frame_id` SHALL mean `base_link`; `base_link` SHALL be accepted. Other frame names SHALL be rejected because the car does not transform frames. CDR strings SHALL be valid UTF-8, NUL-terminated and at most 64 bytes excluding the terminator. Receive time SHALL govern leases; the header stamp SHALL NOT substitute for a synchronized clock. A timestamp with nanoseconds outside 0..999999999 SHALL be rejected.

#### Scenario: Unsupported frame does not drive

- **GIVEN** an armed owner
- **WHEN** it sends an otherwise valid twist in `map`
- **THEN** the sample is discarded without lease renewal or sequence consumption

#### Scenario: Unsynchronized stamp is not a freshness clock

- **GIVEN** a valid owner and a newer setpoint sequence
- **WHEN** a supported-frame setpoint arrives with an old or zero header timestamp
- **THEN** it is accepted using the local receive time for its lease

### Requirement: Sequence domains

BLE/UDP setpoint sequences SHALL use u32 serial ordering: newer iff wrapping difference is in 1..2^31-1; equal and exactly-half-range values SHALL be dropped. ROS SHALL compare full nonnegative i64 attachment sequences strictly increasingly per publisher GID, independently for setpoint and arm publishers. Sequence history SHALL reset with the session; accepted messages from distinct owner publishers SHALL be ordered by receipt, never by comparing their counters.

#### Scenario: BLE or UDP counter wraps

- **GIVEN** last setpoint sequence 4294967295
- **WHEN** sequence 0 arrives in the same session
- **THEN** it is newer; a subsequent 4294967295 is stale

#### Scenario: Same-node publishers have independent counters

- **GIVEN** an owner node with setpoint publisher A at sequence 100 and publisher B with no accepted sample
- **WHEN** B sends sequence 1, then A repeats sequence 100
- **THEN** B's setpoint is accepted and A's duplicate is dropped
- **AND** an arm publisher's own sequence 1 is not compared with either setpoint counter

### Requirement: Session isolation and ROS identity

UDP identity SHALL include peer address/port and the current session ID; BLE identity SHALL include the connection lifetime, not a reusable handle alone. ROS ownership SHALL use graph-announced node identity (zid/nid), not publisher GID; unknown publishers SHALL be ignored. Release or adapter reconnect SHALL discard its queued old-generation input and ordering state. Stale UDP session IDs and callbacks from closed BLE connections SHALL be ignored.

#### Scenario: Old UDP traffic cannot affect a new claim

- **GIVEN** the same peer has released session S and claimed session T
- **WHEN** a delayed ARM, STOP or setpoint with session S arrives
- **THEN** ownership, state, lease and last error for T remain unchanged

#### Scenario: An arm publisher belongs to the owning node

- **GIVEN** cmd_vel and arm publishers announced under one node, with distinct GIDs
- **WHEN** that node owns the car and sends a valid explicit ARM with a fresh lease
- **THEN** ARM succeeds; an unannounced publisher or another node cannot arm it

#### Scenario: Router disconnect ends an owning ROS session

- **GIVEN** a ROS owner and queued adapter input
- **WHEN** the adapter detects router disconnection
- **THEN** owner loss stops and releases the car, old queued input is discarded, and reconnect re-declares endpoints and rebuilds graph identity
- **AND** before disconnection is detected, missing setpoints still trigger the independent lease timeout

### Requirement: Motor validation fault integration

A motor-layer validation fault SHALL cause a device out-of-limits safety stop and a diagnostic identifying the internal wheel fault. Both drivers SHALL remain in standby until the owner sends a fresh valid setpoint followed by explicit ARM, on every source including ROS. Repeated control ticks or new valid setpoints alone SHALL NOT re-enable motor control. Invalid boot configuration SHALL prevent motor enable and make ARM fail not-ready.

#### Scenario: Internal wheel fault cannot auto-recover

- **GIVEN** an armed ROS owner and a fault injected into one computed wheel command
- **WHEN** motor control reports the fault
- **THEN** the car reports stopped/out-of-limits and both drivers stay disabled
- **AND** continued setpoints alone do not resume; a fresh valid setpoint then explicit ARM can recover with a valid configuration

### Requirement: Capabilities encoding

ROS capabilities SHALL contain the nine keys in the scenario below exactly once, with space-separated `key=value` tokens and locale-independent decimal numbers. Geometry and input limits SHALL describe the same canonical f32 values encoded by BLE/UDP; integer fields SHALL be exact. Field order SHALL be insignificant. A session ID SHALL NOT be a capability; UDP carries it in the frame header.

#### Scenario: Keys and units are unambiguous

- **GIVEN** a capabilities message
- **WHEN** a controller parses it
- **THEN** `version` is an integer protocol version; `wheels_radius` and `sum_of_robot_center_projection_on_X_Y_axis` are metres
- **AND** `max_vx` and `max_vy` are m/s, `max_wz` is rad/s
- **AND** `lease_ms`, `claim_ms`, `reclaim_ms` are integer milliseconds
- **AND** floats use enough digits to round-trip their canonical f32 values

### Requirement: Status delivery and empty values

BLE SHALL update readable Status immediately on claim and every change, and notify subscribed centrals on changes. UDP SHALL reply STATUS to owner ARM/STOP, including refusals, and send STATUS on a stop transition to its owner; it provides no reliable delivery guarantee. ROS SHALL emit all four status keys on each publication, with `reason=none` outside stopped and `error=none` when clear. Release SHALL clear the status reason/error and report open.

#### Scenario: Release does not leave a stale reason

- **GIVEN** a stopped owner with reason stop and error not-ready
- **WHEN** ownership is released
- **THEN** readable/published status is open with no reason or error and ROS session=0
- **AND** the next claimant starts with no previous-session error

### Requirement: Valid operating profile

Geometry and normalization SHALL be finite and positive; frame offsets SHALL be finite. Input limits SHALL be finite nonnegative f32-representable values, enforced after canonicalizing to the advertised f32 values. Lease and window lengths SHALL be integers in 1..65535 ms. Invalid profiles SHALL fail initialization with drivers disabled rather than advertise truncated values or permit claims. Numerical overflow during valid-input kinematics SHALL trigger motor fault handling.

#### Scenario: Invalid normalization fails closed

- **GIVEN** max_wheel_speed is zero or NaN, or a lease exceeds 65535 ms
- **WHEN** the device initializes
- **THEN** initialization reports an error, motor enable is unavailable and control claims are not accepted

### Requirement: Deadline ordering

Before processing control input at time t, the device SHALL process any claim, lease or reclaim deadline at or before t using a monotonic clock. Expired sessions SHALL NOT be revived by queued ARM or setpoint input. After release, BLE/UDP SHALL require a new claim; ROS MAY create a new claim from an otherwise valid sample. Automatic ROS arming SHALL NOT clear an existing STOP/out-of-limits latch before release.

#### Scenario: Packet on the reclaim boundary

- **GIVEN** a UDP session with reclaim deadline t
- **WHEN** its ARM or setpoint arrives at t
- **THEN** the old ownership is released first, and that packet cannot drive or renew the expired session

### Requirement: Echo is observational

UDP Echo and the BLE Echo characteristic SHALL return payloads of at most 32 bytes unchanged and discard larger ones. Echo SHALL NOT claim, arm, change wheel commands, renew deadlines, consume setpoint sequences or clear errors. UDP echo SHALL be available to non-owners; BLE connection itself still claims as usual, so BLE echo does not provide a parallel non-owner connection.

#### Scenario: Echo does not keep an owner alive

- **GIVEN** an owner stops sending setpoints but continues echo probes
- **WHEN** the lease expires
- **THEN** the normal lease stop occurs and ownership eventually expires without ARM

### Requirement: New session allocation

Each new claim SHALL allocate a nonzero session ID; repeated HELLO by the current UDP owner SHALL keep that ID and its deadlines. UDP IDs SHALL be seeded randomly per boot and incremented per new claim, skipping zero on wrap. This SHALL be documented as stale-packet separation, not authentication or a guarantee against deliberate replay. BLE sessions SHALL use connection generations; ROS sessions SHALL use the identity and queue rules above.

#### Scenario: Repeated hello does not reset a claim

- **GIVEN** a UDP owner in session S with an existing claim or lease deadline
- **WHEN** the same peer repeats HELLO
- **THEN** HELLO_ACK carries S and the same capabilities, without extending either deadline

### Requirement: Bounded ROS ordering history

The adapter SHALL retain sequence high-water marks for every accepted owner publisher for the current session, including across publisher removal. Other graph announcements SHALL NOT evict those marks. If bounded storage cannot track another owner publisher, its input SHALL be discarded without lease renewal rather than accepted without replay checks. Ownership release SHALL clear these session marks.

#### Scenario: Unrelated publishers cannot reset ordering

- **GIVEN** an owner publisher whose last accepted sequence is 100
- **WHEN** unrelated graph announcements fill the table and that publisher repeats sequence 99
- **THEN** sequence 99 is rejected and cannot renew the lease
