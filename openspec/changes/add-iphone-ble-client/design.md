# iPhone BLE web controller design (draft)

## Context

See [proposal](proposal.md) for the selected scope and acceptance gate. The accepted [car API](../../specs/car-control-api/spec.md) remains unchanged. The [laptop client](../../specs/laptop-teleoperation/spec.md) supplies the gentle-direction/explicit-arm precedent; UDP handshake/correlation does not apply to BLE. [Q9/Q18/Q19/Q21](../../../docs/PLAN.md#open-questions-and-decisions) remain the roadmap's home.

### Recorded user decisions (2026-10-20)

* Linux only; no Mac or paid Apple Developer membership; avoid a subscription.
* Target a web controller opened in Bluefy on iPhone.
* Phone supports the 72-byte payload (Todor's confirmation, not a new measured app/board result).
* Six press-and-hold buttons first; allow a later accelerometer-based input without changing the device API or bypassing safety policy. Full proposal implementation acceptance is still pending.

### Existing protocol evidence

Inspected at main `1be50b3`; no protocol change is proposed.

| Concern | Existing evidence and client consequence |
| --- | --- |
| Advertisement/claim | [Firmware BLE loop](../../../firmware/src/fw.rs#L365): flags and `Rovi M8` in scan response, no service UUID. Use a name filter plus optional service access, then validate GATT. Connecting claims immediately and switches off the AP. Name/device ID is not authentication. |
| GATT | [RoviService](../../../firmware/src/fw.rs#L67): service `d47a0010-45b2-4d19-8db0-6bd87e9c0001`; same suffix with `0011` Setpoint (write without response), `0012` Control (write with response), `0013` Capabilities (read), `0014` Status (read/notify), `0015` Echo (write without response/notify). Require the first four characteristics/properties; Echo is unused. |
| Setpoint | [Python wire reference](../../../experiments/rovi-m8-setpoint-ros/sim/wire.py#L14) and [Rust decoder](../../../crates/rovi-core/src/device.rs): u32 LE sequence + 68-byte little-endian TwistStamped CDR = 72 bytes. Empty frame means base_link; zero timestamp is valid; only vx/vy/wz vary. No UDP header, BLE numeric session, HELLO or BYE. |
| Capabilities | [Encoder](../../../crates/rovi-core/src/capabilities.rs#L42): u8 version, five f32 LE (radius, centre projection sum, max_vx/max_vy/max_wz), three u16 LE (lease/claim/reclaim ms), 27 bytes. Widen f32 values exactly; never round limits upward. |
| Status/control | [Encoder](../../../crates/rovi-core/src/status.rs#L35): states 0 Open/1 Claimed/2 Armed/3 Stopped; reasons 0 none/1 Stop/2 LeaseExpired/3 OwnerLost/4 OutOfLimits; errors 0 none/1 Busy/2 NotOwner/3 NotReady/4 Invalid. Require reason=0 except Stopped, whose reason must be nonzero. ARM=1/STOP=2; refused writes use ATT Value Not Allowed. |
| Windows | [Defaults](../../../crates/rovi-core/src/arbiter.rs#L32): lease 1000, claim 10000, reclaim 5000 ms. Read advertised values. Disarmed zero setpoints retain ownership; STOP retains it; disconnect releases it. Firmware disconnects on ownership expiry. |
| Prior evidence | [Bleak client](../../../experiments/rovi-m8-setpoint-ros/sim/ble_client.py), [product Status/Echo probe](../../../firmware/bench/ble_status_echo.py) and [BLE lease results](../../../firmware/bench/2026-10-19-ble-device-lease-timing/notes.md) inform the wire contract, not Bluefy performance. |

## Goals / Non-Goals

**Goals:** Linux-authorable static controller, one foreground phone session, button-first operation, testable codecs/session policy and separate browser/radio/wheel evidence.

**Non-Goals:** native app/scaffold/signing, sensor driving in v1, firmware/security changes, station mode, background driving, automatic recovery, calibration or measured speed. This draft creates no runtime or hosted site.

## Decisions

### Bluefy web delivery from Linux

Use static HTML/CSS/JavaScript under `apps/iphone/` with no third-party runtime scripts. Load a versioned page over HTTPS in Bluefy. No command backend, account or paid developer membership is needed for our web code. Propose a repository GitHub Pages deployment at `https://tptodorov.github.io/rovi/iphone/` (not a live URL claimed by this draft). [GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site) supports static sites in public repositories on GitHub Free. Prepare only the app assets, preserving any existing site configuration; confirm the final origin/HTTPS during delivery. The page is trusted controller code: no third-party runtime scripts or secrets. This spec PR performs no deployment.

Why this route: [Apple's Xcode requirements](https://developer.apple.com/support/xcode/) require macOS; [free Personal Team provisioning](https://developer.apple.com/help/account/basics/about-your-developer-account) uses Xcode and expires after seven days. A remote Mac can compile a native app, but compilation alone does not solve free signing/install from this Linux setup. [Bluefy's developer guide](https://bluefy.app/guide.html) documents HTTPS-page BLE control; its [App Store listing](https://apps.apple.com/us/app/bluefy-web-ble-browser/id1492822055) lists it as free. This replaces the earlier SwiftUI/CoreBluetooth proposal rather than adding a second client.

Bluefy is a required installed host, not proof that every Web Bluetooth function or inactive-state signal works on the selected phone. Feature checks and P1/P3 must fail closed where support is absent. Safari/Home Screen web apps are not promised substitutes; [WebKit's tracking issue](https://bugs.webkit.org/show_bug.cgi?id=238049) records the Web Bluetooth limitation. No Expo/React Native wrapper is needed.

### Selection and setup

The page's Choose & Connect action explains the ownership/AP effect, then invokes `requestDevice` with `filters: [{name: 'Rovi M8'}]` and `optionalServices: [serviceUUID]`. Selecting in the browser picker is the explicit target choice; only then connect while the page is active. Picker presentation/dismissal is browser-controlled: do not promise a custom scan list, RSSI, hardware addresses, a cancellable scan API or a ten-second picker timeout. Same-name candidates are not authenticated identities; display the selected name and opaque browser ID when available.

Start the ten-second setup deadline at GATT connect invocation after selection, not while the person considers the picker. Discover/validate required properties, exact capabilities, valid Claimed/Stopped Status and notifications; submit an initial 72-byte zero setpoint without response, with no ARM. Claim starts before discovery completes; a firmware timeout fails setup without automatic reconnect. Reject lease<500 ms, claim<1000 ms or invalid/nonpositive windows/geometry. Current defaults exceed these supported-profile floors; these are proposed client bounds, not iOS measurements.

The phone payload capability is user-confirmed. Web Bluetooth does not expose CoreBluetooth's MTU/write-capacity/readiness APIs. Show packet size 72 and MTU/capacity as unavailable unless actually measured externally. A rejected initial zero submission is incompatibility; promise resolution is not proof of firmware receipt. An explicit later zero+ARM/Status transaction and P1 establish the browser/firmware path. Never fragment or switch protocols automatically.

### Button input with a later sensor option

Separate the input interpreter from the session/transmitter: buttons produce only latest planar intent and its expiry; one shared policy owns arming, caps, STOP, lifecycle and connection generations. No generic plugin framework or unused sensor implementation is needed. A future accelerometer/tilt interpreter would feed that same intent boundary; it cannot arm or write GATT directly. It needs a separate reviewed proposal for axis mapping, calibration, dead zone, enable/deadman gesture, permission refusal, sensor loss and input switching. The [motion/orientation standard](https://www.w3.org/TR/orientation-event/) defines sensor APIs, not a guarantee that Bluefy exposes them. V1 does not request motion permission or consume sensor data.

The v1 page has Choose & Connect, local/reported state, ARM, prominent STOP, six labelled hold buttons, requested values and Disconnect. Buttons use pointer tracking/capture with explicit bounds checks; pointerup/cancel, lost capture, leaving bounds or another direction invalidates the hold. Renew the held input only from active visible UI animation frames, never the transmit timer; a stalled UI expires after 250 ms even if BLE work continues. Re-entry requires a new press. Text/accessibility labels, large targets, text sizing and cues beyond color apply; accessibility activation never creates a sticky nonzero toggle. Actual hold accessibility/usability remains a P3 acceptance check.

### Stream, controls and Status

One session generation and operation epoch own all asynchronous work. Serialize GATT procedures; keep at most one pending setpoint write promise and one coalesced latest intent, never a motion queue. Every 50 ms, check lifecycle, stops and input expiry before choosing a sample. Do not start another write while the prior procedure is pending. A scheduler/submission gap>=200 ms fails the run; do not catch up. A write-without-response promise is host progress, not an acknowledgement of applied motion. Fair scheduling must leave space for Status/control within these bounds; P2 measures the complete workload.

ARM invalidates touches, submits fresh zero, then performs one `writeValueWithResponse` ARM. Only its successful current-generation promise plus the subsequently requested `readValue` result Armed/none/none grants local motion permission; one second covers the whole action. Copy the returned bytes immediately. Notifications never grant arm. Unlike native delegate callbacks, Web Bluetooth provides an explicit read promise; there is no need to repeatedly disable notifications to identify read results. Ignore stale promise completions and never use cached characteristic.value to confirm controls.

STOP immediately invalidates pending local ARM and all holds, takes scheduling priority, and requires successful Control=2 write plus a subsequent read Stopped/Stop/none within one second. An in-flight GATT procedure cannot be assumed cancellable; keep intent zero, attempt STOP when possible, and disconnect on timeout. Confirmed STOP retains zero streaming/ownership; only a new explicit ARM and new press can resume. Notifications can revoke permission; repeated reports of the same stopped state while recovering do not create a new stop, but a new stop transition during ARM invalidates it.

Schedule an explicit Status read every 500 ms, at most one pending transaction; a control confirmation read also counts. A transaction lasting>=1000 ms, malformed result, GATT error, Open or unexpected Armed ends the run. Status notifications/reads do not renew the firmware lease. Treat disconnection events and observed service/permission loss as failure. Web APIs need not expose native powered-off/resetting/permission distinctions: use specific errors when known and honest unavailable/disconnected wording otherwise.

### Lifecycle and bounded shutdown

During setup/connection, use page visibility, blur, pagehide/navigation and any verified Bluefy host inactivity signal to clear holds and end the session. Before GATT connection, a browser picker can itself change focus; that focus change cannot drive anything. Recheck page activity after selection and never connect from an inactive page or a cancelled attempt. Gate every send on the current active/visible state; foreground return, page reload or a remembered device grant cannot reconnect or re-arm automatically. Do not opt into Bluefy background peripheral access or prevent screen locking. Browser/native lifecycle events are not interchangeable: P3 must verify actual lock, app switch, browser UI and interruption behavior; if an undetected inactive state allows renewed nonzero submissions, this browser/version is unsupported for driving.

Disconnect/failure first clears intent and stops the normal stream. Under one 500 ms deadline while runnable, independently attempt zero and STOP where possible, then invoke local `gatt.disconnect()` even if a promise fails/hangs. Cancel generations and ignore late completions; a connect promise resolving after shutdown is disconnected again. A browser picker cannot be forcibly dismissed by an invented API; a late selection never connects after its local attempt was cancelled.

Suspension/termination may prevent any cleanup. Firmware lease/window handling remains the independent fallback; local disconnect or resolved writes do not establish physical link teardown, ownership release or wheel stop. Show requested, device-reported and unconfirmed states separately, retaining the original Status observation age.

## Platform sources checked 2026-10-20

[Web Bluetooth specification](https://webbluetoothcg.github.io/web-bluetooth/) defines secure-context user selection, service access, characteristic properties, separate write modes, read promises and notification events. These are API contracts, not evidence that Bluefy implements every method. [Bluefy's official site](https://bluefy.app/) and [guide](https://bluefy.app/guide.html) document its GATT bridge and HTTPS requirement. The [App Store release notes](https://apps.apple.com/us/app/bluefy-web-ble-browser/id1492822055) mention a host app-state hook; exact hook registration/semantics must be verified instead of guessed. Apple's [foreground/background guide](https://developer.apple.com/library/archive/documentation/NetworkingInternetWeb/Conceptual/CoreBluetooth_concepts/CoreBluetoothBackgroundProcessingForIOSApps/PerformingTasksWhileYourAppIsInTheBackground.html) explains why suspension cannot be treated as a runnable cleanup opportunity.

## OPEN questions and acceptance gates

| Item | How to settle | Blocks |
| --- | --- | --- |
| I1 — browser/environment and delivery checks | Record iPhone model/iOS/Bluefy version. Confirm split write methods, read/notify, name-only picker and lifecycle signals. Verify the proposed GitHub Pages origin/HTTPS during delivery, preserving existing site configuration; publication remains a future authorized task. Linux-only/no-membership, Bluefy and the proposed free hosting route are recorded choices. | Deployment and phone acceptance; independent Linux codec/UI/session work can proceed after task 1.1. |
| I2 — browser payload integration, phone capability confirmed | Todor confirms phone 72-byte support. P1 checks the actual Bluefy path with zero+explicit ARM/Status at zero; keep runtime rejection handling. Record measured ATT MTU only if instrumentation provides it; do not fabricate capacity from Web Bluetooth or mark new board evidence passed. | Browser compatibility acceptance, not a repeated question about whether the phone supports 72 bytes. |
| I3 — Q19 full workload | P2 measures 60 s at 20 Hz with Status reads and zero ARM/STOP: no lease expiry, >=200 ms submission gap or one-second control/read timeout. Record connection interval only if measured. Failure requires reviewed changes, not silently relaxed bounds. | Phone acceptance and motor batch; BlueZ results do not pass it. |
| I4 — lifecycle/accessibility verification | Button-first interaction is selected. P3 verifies actual touch, text/accessibility, blur/hidden/lock/interruption behavior and no background renewal of motion. Missing host signals must still produce expiry/lease fallback; persistent hidden motion fails support. | UI/phone acceptance, not the selected button layout. |

Full revised proposal acceptance is pending. Unmeasured browser/hardware results cannot count as passed acceptance; see [verification](verification.md).

## Risks / Trade-offs

* Browser dependency and opaque picker/state APIs → feature gates and actual-phone checks; no native feature assumptions or guessed host hooks.
* Browser scheduling/buffered writes and suspension → expiring UI intent, bounded application work and firmware lease fallback. Proposed software deadlines are not physical stopping bounds.
* Public static controller code and same-name peers → controlled HTTPS origin, no remote runtime scripts, explicit selection and GATT validation; these do not add authenticated identity to the existing API.
* Zero streaming deliberately retains ownership after STOP → visible ownership effect and Disconnect. Open-loop gentle references can overcome motor deadband; they are not measured physical speeds.

## Migration Plan

After explicit acceptance, implement/test the static page from Linux, prepare a reviewable HTTPS deployment, and publish only within an authorized delivery task. Then run separately authorized phone checks and a separately announced supervised wheels-off-ground batch. Rollback is disconnecting/closing the page and withdrawing its deployment; no firmware migration. Archive after product acceptance, never after this planning PR alone.
