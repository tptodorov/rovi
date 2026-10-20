# iPhone BLE client design (draft)

## Context

See [proposal](proposal.md) for scope and acceptance status. The [car API](../../specs/car-control-api/spec.md) is accepted; this application consumes it. The [laptop client](../../specs/laptop-teleoperation/spec.md) supplies the gentle direction/explicit-arm precedent, but its UDP handshake and reply correlation do not apply to BLE. [Q9/Q18/Q19/Q21](../../../docs/PLAN.md#open-questions-and-decisions) remain the roadmap's source of truth.

### Existing protocol evidence

Inspected at main `1be50b3`; no protocol change is proposed.

| Concern | Normal product evidence and consequence |
| --- | --- |
| Advertisement/claim | [Firmware BLE loop](../../../firmware/src/fw.rs#L365) advertises flags and `Rovi M8` in scan response, without service UUID. Scan foreground with services=nil, filter local name, then validate the service after explicit Connect. `on_ble_connect` claims immediately; AP goes off. Name, RSSI and peripheral identifier distinguish candidates, not authenticate them. |
| GATT | [RoviService](../../../firmware/src/fw.rs#L57): service `d47a0010-45b2-4d19-8db0-6bd87e9c0001`; same suffix with `0011` Setpoint (write without response), `0012` Control (write with response), `0013` Capabilities (read), `0014` Status (read/notify), `0015` Echo (write without response/notify). Require the first four characteristics and properties; Echo is optional and unused by this design. |
| Setpoint | [Python wire reference](../../../experiments/rovi-m8-setpoint-ros/sim/wire.py#L14) and [Rust decoder](../../../crates/rovi-core/src/device.rs): u32 LE sequence + 68-byte CDR = 72 bytes. CDR is `00 01 00 00`, zero seconds/nanoseconds, string length 1, NUL and padding, six f64 LE Twist values. Empty frame means base_link. No stamp synchronization, UDP envelope or BLE numeric session. |
| Capabilities | [Canonical encoder](../../../crates/rovi-core/src/capabilities.rs#L42): `u8 version`, five f32 LE (radius, centre projection sum, max_vx, max_vy, max_wz), three u16 LE (lease/claim/reclaim ms), 27 bytes. Preserve f32 values when widening to Double; do not round limits upward. |
| Status/control | [Status encoder](../../../crates/rovi-core/src/status.rs#L35): state 0 Open/1 Claimed/2 Armed/3 Stopped; reason 0 none/1 Stop/2 LeaseExpired/3 OwnerLost/4 OutOfLimits; error 0 none/1 Busy/2 NotOwner/3 NotReady/4 Invalid. Non-Stopped requires reason=0; Stopped requires a nonzero reason. Control ARM=1/STOP=2; refused writes return ATT Value Not Allowed and retain owner error in Status. |
| Windows | [Arbiter defaults](../../../crates/rovi-core/src/arbiter.rs#L32): lease 1000, claim 10000, reclaim 5000 ms. Valid disarmed zero setpoints retain ownership. STOP does not release; BLE disconnect does. After expiry firmware disconnects. Use advertised windows, not these defaults as constants. |
| Prior clients/results | [Bleak client](../../../experiments/rovi-m8-setpoint-ros/sim/ble_client.py), [normal-firmware Status/Echo probe](../../../firmware/bench/ble_status_echo.py), and [BLE lease results](../../../firmware/bench/2026-10-19-ble-device-lease-timing/notes.md) inform the contract. They are laptop/BlueZ evidence, not iPhone validation. |

## Goals / Non-Goals

**Goals:** small native UI; testable codec/session logic; one foreground controller; fail closed on unsupported payload/limits and stale operations; phone evidence collected separately from fake-transport evidence.

**Non-Goals:** changes to firmware/API/security, background control, automatic reconnect, persistent commands, calibration, measured speed, mixed-axis joystick, Wi-Fi/ROS configuration or distribution infrastructure. No code or device activity is part of this draft.

## Decisions

### Native UI and one session owner

Propose SwiftUI + CoreBluetooth with no third-party runtime dependency. Future `apps/iphone/` would contain the app, codec/session unit tests and UI tests. Keep codecs and a deterministic session state machine separate from the CoreBluetooth adapter and touch UI so fake-clock/fake-peripheral tests need no radio. One serial executor owns BLE operations, deadlines and session generations; the UI provides timestamped touch intent and immediately invalidates it on inactivity. Avoid sharing mutable arming state between independent timers.

The proposed screen has Scan/results/Connect, then connection/local-arm and last-reported Status, ARM, prominent STOP, six labelled hold controls, requested values and Disconnect. No numeric tuning or settings screen in v1. Text, VoiceOver labels, Dynamic Type and usable hit targets apply to all controls. Standard accessibility activation never becomes a sticky motion toggle; device usability review must verify a deliberate hold interaction or keep nonzero controls unavailable for unsupported activation modes (OPEN I4).

Compared with a web client or a joystick, this keeps BLE/lifecycle integration native and requests one bounded axis at a time. The six-button choice is proposed for Todor's review, not a prior accepted product decision.

### Connection, validation and streaming

No connection from scan, app launch or a remembered identifier. Stop scan on selection/timeout/inactivity; retain the selected CBPeripheral through that connection only. Check authorization and manager poweredOn before discovery/connect; show denied/off/resetting/unsupported distinctly. Include a specific `NSBluetoothAlwaysUsageDescription` explaining discovery and control. A settings change permits manual retry, not automatic connection.

Within the single 10-second Connect deadline, discover the exact service/characteristics, validate properties/capabilities, check `maximumWriteValueLength(for: .withoutResponse) >= 72`, read valid Claimed or Stopped Status and enable notifications. Claim is already running during discovery; if firmware times it out, offer manual retry. Only after validation start the 50 ms zero stream. The lease>=500 ms and claim>=1000 ms checks are this client's supported-profile floor; current product defaults exceed them. They are not measured iOS timing guarantees.

UI hold tracking renews intent while the same physical touch remains inside the control; release, cancellation, a second direction or lifecycle change invalidates it. The transmitter never extends the 250 ms input deadline. Expiry and STOP win before any send. Check `canSendWriteWithoutResponse`; on readiness send current intent at the next due send, never drain old samples. A 200 ms scheduler/submission gap ends the session. CoreBluetooth's own queued writes cannot be recalled: keep application submissions to one per 50 ms and never promise immediate physical cessation from the app's deadline.

### Control confirmation without invented BLE sequence replies

BLE Status has no request ID. Serialize Control writes, allow only one pending control transaction, and associate it with a local connection generation and operation epoch. ARM clears holds, submits fresh zero before ARM and continues zero throughout confirmation. Write submission is not proof that zero arrived; a refused ARM ends the session rather than being retried automatically.

Only the current successful with-response Control completion plus a fresh explicit Status read can confirm ARM/STOP. CoreBluetooth delivers reads and notifications through the same delegate method; a pending read flag alone cannot correlate them. For an unambiguous read, pause Status notifications, await the subscription-state completion and drain prior callbacks on the serial executor, perform the sole Status read, then restore notifications and await completion before enabling motion. Use this serialized read procedure for periodic health reads too; continue latest zero/armed-intent streaming during normal health reads. All control steps share the one-second control deadline. Any ambiguity/error/deadline failure closes the session. Validate this ordering with fakes and the selected real phone before acceptance.

Notifications can revoke local arm at any time but never grant it. Ignore duplicate Stopped reports while explicitly recovering from that same stop; a new stop transition during ARM invalidates the attempt. STOP invalidates pending local ARM immediately, discards touch input and takes priority over further application writes. If an in-flight ARM cannot be cancelled, keep intent zero, send STOP when possible and cancel the connection on timeout. Never treat the late ARM completion as permission to move.

Schedule a Status read every 500 ms with at most one outstanding transaction; if one remains pending at the next period, do not overlap it. Its 1000 ms deadline includes disabling/restoring notifications and the read response. Coordinate polling with controls; a control confirmation read also satisfies that period. Status polling does not renew firmware leases. This uses existing readable Status rather than adding Echo traffic or implying Echo proves ownership.

### Lifecycle and shutdown

Control is active-foreground only. Use scene/activity notifications and touch cancellation to revoke permission immediately on inactive, including app switcher, Control Center, incoming interruption or lock. Disable background Bluetooth modes, state restoration and automatic reconnect. Return from any interruption requires a new manual connection and ARM.

Disconnect/failure/interruption first clears local intent and ends the normal stream. Under one 500 ms runnable deadline, attempt zero if permitted, attempt STOP with response independently, then call `cancelPeripheralConnection` even if earlier operations fail or hang. Do not wait for normal one-second control confirmation during cleanup; any obtained Status can be reported as last observed. Cancel pending connect as well as an established connection. Invalidate the generation so delayed connect/write/read/notification callbacks cannot affect another session; a late successful connect after cancellation is cancelled again.

This bounds local attempts only. iOS may suspend or terminate without allowing cleanup. Apple also documents that cancelling the app's local connection need not immediately tear down a shared physical link. Firmware lease/claim/reclaim behavior remains the independent fallback; zero/STOP/cancellation is best effort. UI says “STOP requested”, “device reported Stopped/Stop”, or “local connection ended”, never “wheels verified stopped” or “ownership definitely released”.

## Apple platform facts checked 2026-10-20

* [`maximumWriteValueLength(for:)`](https://developer.apple.com/documentation/corebluetooth/cbperipheral/maximumwritevaluelength%28for%3A%29) gives a per-write-type byte capacity. Use it rather than inventing a public iOS MTU setting; capacity>=72 is the client gate, not proof of actual ATT MTU. Firmware needs MTU>=75 for its 72-byte packet; Q18 still needs phone/board evidence.
* [`writeValue`](https://developer.apple.com/documentation/corebluetooth/cbperipheral/writevalue%28_%3Afor%3Atype%3A%29) provides write completion only with response; without-response delivery is not guaranteed. [`canSendWriteWithoutResponse`](https://developer.apple.com/documentation/corebluetooth/cbperipheral/cansendwritewithoutresponse) and [readiness callback](https://developer.apple.com/documentation/corebluetooth/cbperipheraldelegate/peripheralisready%28tosendwritewithoutresponse%3A%29) support flow control, not lease acknowledgements. [Value callbacks](https://developer.apple.com/documentation/corebluetooth/cbperipheraldelegate/peripheral%28_%3Adidupdatevaluefor%3Aerror%3A%29-1xyna) serve both reads and notifications.
* [Scanning](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/scanforperipherals%28withservices%3Aoptions%3A%29) normally benefits from a service filter, but the existing advertisement requires the foreground unfiltered exception. [Bluetooth manager states](https://developer.apple.com/documentation/corebluetooth/cbmanagerstate) and the required [usage description](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothalwaysusagedescription) define availability and permission handling.
* [Scene phase](https://developer.apple.com/documentation/swiftui/scenephase) exposes activity changes. Apple's [background guide](https://developer.apple.com/library/archive/documentation/NetworkingInternetWeb/Conceptual/CoreBluetooth_concepts/CoreBluetoothBackgroundProcessingForIOSApps/PerformingTasksWhileYourAppIsInTheBackground.html) explains that suspended foreground-only apps cannot process Bluetooth work; background support would not guarantee indefinite execution either. [Cancellation](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/cancelperipheralconnection%28_%3A%29) is nonblocking and does not guarantee immediate physical link teardown.

## Prerequisites and OPEN questions

This Linux worktree does not establish that a Mac, compatible Xcode/macOS pair, signing team or test iPhone is available. [Apple's device workflow](https://developer.apple.com/documentation/xcode/running-your-app-on-simulated-or-physical-devices) requires Xcode/platform support on a Mac and device signing/provisioning. Local device deployment needs a trusted/paired phone, Apple Account/team and unique bundle ID, with [Developer Mode](https://developer.apple.com/documentation/xcode/enabling-developer-mode-on-a-device) where required. A [Personal Team](https://developer.apple.com/help/account/basics/about-your-developer-account) can support personal device development, with seven-day provisioning expiry; do not assume paid membership or credentials. Simulator/fake tests do not establish radio behavior.

| OPEN | How to settle | Blocks |
| --- | --- | --- |
| I1 — execution environment and supported OS | Todor identifies Mac access, installed Xcode/macOS, iPhone model/iOS, signing team and bundle ID; record a supported minimum iOS/toolchain pair before scaffolding. Proposed deployment is local development signing; TestFlight/App Store need separate scope. | App scaffold/build/signing tasks 2.x/3.x; independent protocol review can proceed after acceptance. |
| I2 — Q18 on iPhone | On normal firmware, record phone-reported without-response capacity and board-observed ATT MTU over 10 manual connections; all must fit the unchanged 72-byte packet. A low/unreliable capacity fails compatibility, not triggers a compact fallback here. | Real-phone compatibility acceptance (4.1); cannot close from BlueZ MTU 247. |
| I3 — Q19 stream/control responsiveness | Measure 20 Hz for 60 seconds at zero, submission-gap distribution, backpressure, Status reads/subscription transitions and ARM/STOP confirmation against normal firmware. No lease expiry, no >=200 ms submission gap and no control/read timeout are the proposed gate. Record actual interval only if captured by instrumentation; never infer iOS interval from the 50 ms timer or BlueZ's 45 ms. A failing result requires revised proposal/review before loosening bounds. | Real-phone acceptance (4.2) and motor batch (4.4); not pure host tests. |
| I4 — touch/accessibility/lifecycle usability | Todor reviews six labelled hold controls; verify hold/release/cancel, VoiceOver behavior, Dynamic Type and inactive/lock transitions on the selected phone. Resolve a usable bounded accessible hold interaction before claiming accessible motion; any alternate interaction changing the contract needs review. | UI acceptance (3.2/4.3); six-direction state/codec work can be tested independently. |

All choices above remain proposed until task 1.1 records Todor's acceptance. Unmeasured I2/I3 can remain open during independent host implementation after that gate, but never count as passed phone acceptance. [Verification](verification.md) specifies evidence tiers and stop conditions.

## Risks / Trade-offs

* Same-name or spoofed peripherals → validate GATT/capabilities and require selection; this is compatibility, not authenticated identity. Existing API security is unchanged.
* Failed/slow setup consumes the claim window; zero streams intentionally retain ownership even after STOP → show ownership effects and explicit Disconnect; never quietly reconnect to hide them.
* SDK/runtime timing, buffered radio writes and suspension can exceed local intentions → firmware lease fallback, honest state labels and mandatory physical evidence. Gentle references are approximate and can overcome the motor deadband; they are not measured slow speeds.
* Pausing notifications for unambiguous reads adds ATT overhead → test the complete workload under I3. Do not claim the laptop's throughput transfers to iOS.

## Migration Plan

After explicit proposal acceptance and I1 resolution, implement in `apps/iphone/` without modifying existing transports. Run host/UI checks, then separately authorized normal-firmware phone checks, then a separately announced supervised wheels-off-ground batch. Record phone/OS/app/firmware revisions and Todor's observations. Rollback is closing/uninstalling the app and restoring manual controller use; no firmware migration or credentials are introduced. Archive only after the acceptance review, not after this documentation PR.
