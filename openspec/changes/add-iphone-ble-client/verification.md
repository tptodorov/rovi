# iPhone BLE web controller verification map

**Linux evidence recorded (2026-10-20); phone evidence pending.** H1–H5 pass on Linux under [`apps/iphone`](../../../apps/iphone/README.md) and in CI. No Bluefy/phone, deployment or wheel check has run. Todor's phone 72-byte confirmation is user evidence, not a measurement. [Tasks](tasks.md) record which gates are closed.

## Linux host and browser evidence

| ID | Test boundary and completion evidence |
| --- | --- |
| H1 | Codec tests: 72-byte zero/six-direction vectors, endian/CDR layout, u32 wrap, 27-byte capabilities/f32 limits, finite/geometry/window/version/length checks and all valid/invalid Status combinations. Compare [wire vectors](../../../experiments/rovi-m8-setpoint-ros/sim/test_wire.py) and [product encoder](../../../crates/rovi-core/src/capabilities.rs). |
| H2 | Fake Web Bluetooth and Linux browser: HTTPS/API gates, explicit user picker/name filter/service access, cancelled/no/late selection, no remembered auto-connect, GATT properties, zero-write rejection, setup/claim timeouts and late connect results. No ARM/nonzero on setup failure; no invented picker cancellation/MTU/readiness methods. |
| H3 | Fake clock/promise/event traces: fresh zero before one ARM; only current successful write plus subsequent read promise confirms; notifications/cached value cannot grant motion. STOP over pending ARM, safety-stop latch, malformed/stale results, read/control deadlines, serial GATT work, one pending setpoint and no backlog. Inject 199/200 ms gaps and pending-write completion after release. |
| H4 | Linux browser/pointer tests: six signs/capped values, zero axes, multi-touch, release/outside/re-entry/cancel/lost capture; 249/250 ms expiry; UI frame renewal stalls while transmitter continues; pre-arm/post-STOP touches never replay. No sensor input or motion permission in v1. |
| H5 | Browser/session tests: blur/hidden/pagehide/host-inactivity/disconnect/errors, reload/return manual recovery; zero/STOP failures/hangs independently; disconnect by shared 500 ms deadline while runnable. Check text sizing, labels, targets, non-color state and no sticky accessibility activation. Verify static output/HTTPS deployment instructions contain no secrets/remote runtime scripts. |

Linux browser tests and fake clocks establish software decisions, not Bluefy/native bridge behavior, iOS scheduling, radio delivery or physical stopping. No Mac/Xcode/Simulator evidence is required or claimed for this web route.

## Requirement and scenario coverage

Each row covers every scenario under the named [requirement](specs/iphone-ble-teleoperation/spec.md); H IDs are implemented Linux tests; P IDs are planned phone/board evidence.

| Requirement | Linux host/browser | Phone/board evidence |
| --- | --- | --- |
| Supported web delivery | H2/H5 (HTTPS/API refusal/assets) | P1 actual API support/deployed revision |
| Explicit discovery and selection | H2 (browser picker/cancel/late result) | P1 name-only picker/no automatic claim |
| Validate the connected car | H1/H2 (incompatibility/claim loss) | P1 actual zero/ARM/Status path and I2 |
| Existing BLE wire contract | H1 (directions/wrap) | P1 zero path; P4 wheel directions |
| Explicit zero then ARM | H3/H4 (old hold/refusal/late confirmation) | P2 controls; P4 no replay |
| Gentle single-axis directions | H1/H4 (signs/caps/disabled axes) | P3 touch; P4 six directions |
| Hold cancellation and input expiry | H4 (release/cancel/expiry/frame stall) | P3 actual pointer/lifecycle; P4 release |
| Periodic latest-intent stream | H3 (pending write/backpressure/gap) | P2 complete workload and I3 |
| Validate Status and link health | H1/H3 (read deadline/old event) | P2 read/notify/error behavior; P3 loss |
| STOP and re-arm latch | H3/H4 (STOP/ARM/touch race) | P2 confirmation; P4 latch observation |
| Active foreground and manual recovery | H2/H5 (inactive/return/reload) | P3 actual host events/frame expiry/termination |
| Bounded best-effort shutdown | H5 (blocked writes/disconnect) | P3 attempt/detection delay; P4 loss fallback |
| Visible state and evidence limits | H4/H5 (Armed without motion/unavailable MTU) | P3 usability/labels; P4 separate wheel observation |

## Bluefy / normal product firmware evidence planned

These are proposed checks, **not authorization to run them**. No hosted page, phone, network, router, serial or motor activity occurred while revising this draft. Use normal default or `ros` firmware; never `motor-bench`. Do not change firmware to make Bluefy pass.

* **P1 — browser/claim/payload integration (I1/I2):** motor power disabled, separately authorized batch. Record phone/iOS/Bluefy/web/firmware revisions, HTTPS origin, split with/without-response write APIs, explicit read promises/notifications, name-only picker and error behavior. Across 10 manual connections, verify validated setup and explicit ARM/STOP at zero with the expected fresh Status. The phone's 72-byte capability is user-confirmed; this checks our browser/firmware path. Record ATT MTU/capacity only if actually measured externally; unavailable MTU is not a browser failure or fabricated API result. Record claim timing and release observations separately.
* **P2 — full workload (I3/Q19):** motor power disabled, separately authorized batch. Run 60 s at 20 Hz zero including Status polling and explicit ARM/STOP/re-arm at zero. Require no lease expiry, submission gap>=200 ms or one-second control/read timeout. Log pending-operation delays, promise durations, submission-gap distribution and read/control latency. Only actual instrumentation can establish connection interval; timer frequency/BlueZ cannot. Failing performance requires reviewed contract/approach changes before motor checks.
* **P3 — host lifecycle and usability (I1/I4):** motor power disabled, separately authorized batch. Verify real pointer/hold/cancel/multi-touch, accessibility/text sizing, Bluefy browser controls, blur/page visibility, app switch/Home, lock/unlock, interruptions, Bluetooth off/on, permission changes, navigation, force termination/reload and unavailable peer. Record detected events, UI renewal expiry, last nonzero submission, cleanup attempts and device-reported state. P3 fails if an inactive host keeps renewing nonzero intent/streaming it; unsupported signals must not silently become a safety promise. Distinguish observed expiry/firmware fallback from runnable cleanup. Record non-inducible cases as unverified.
* **P4 — supervised wheel acceptance:** only after P1–P3 and a separately announced batch with Todor's go, wheels off ground. Observe six directions briefly, release-to-zero, STOP during hold, new ARM/new press, lock/background and link loss. Record Todor's physical observations separately from browser/protocol/firmware logs. Confirm independent lease fallback; app deadlines are not physical stop bounds. No electrical/coast limits, floor speed accuracy, range or unattended-safety claim.

Record date, exact steps, hardware/browser/web/firmware identities, logs, observations and failures. [Hardware policy](../../../README.md#hardware-validation-policy) and [acceptance gates](../../../docs/SPECIFICATION.md#workflow-and-gates) apply. Missing/failed evidence leaves its task unchecked; revised behavior returns to proposal review.

## Draft checks

Web revision authoring checks on 2026-10-20 (documentation evidence only):

* `rtk npm run spec:check`: strict active draft plus three baseline specs pass (4/4).
* `rtk npm run openspec -- validate --archived --strict --no-interactive`: both archived changes pass (2/2).
* `rtk npm run spec:build` regenerates the reader; `rtk npm run spec:reader:check` passes six tests and freshness checking.
* All 34 local links/anchors resolve, and all 13 proposed requirements have scenarios and verification mappings. Reviewed picker timing/focus, capability checks, read-promise confirmation, serialized writes, lifecycle expiry, cleanup and deferred sensor scope for consistency.
* `rtk git diff --check` passes. Accepted specs, roadmap, runtime/firmware and Wi-Fi files remain unchanged. No deployment, phone/board result or implementation acceptance is implied.
