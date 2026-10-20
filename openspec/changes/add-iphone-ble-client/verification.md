# iPhone BLE client verification map

**Planning evidence only (2026-10-20).** No iPhone app exists in this change; no unit/UI/device/wheel acceptance has passed. The [requirements](specs/iphone-ble-teleoperation/spec.md) are proposed. All [tasks](tasks.md) remain unchecked. Existing laptop results explain the protocol and do not close iPhone Q18/Q19. This file is the future evidence ledger; do not infer acceptance from artifact completeness or a green documentation check.

## Host and UI evidence planned

| ID | Test boundary and completion evidence |
| --- | --- |
| H1 | Codec unit tests: 72-byte golden setpoints (zero/six directions), endian/CDR layout, u32 wrap, 27-byte capabilities with f32 limits, finite/geometry/window/version/length checks and all Status codes/invalid combinations. Compare with existing [wire vectors](../../../experiments/rovi-m8-setpoint-ros/sim/test_wire.py) and [product encoding](../../../crates/rovi-core/src/capabilities.rs). |
| H2 | Fake central/peripheral: no scan before user action; same-name candidates; manual selection, properties and payload<72; every setup timeout/claim-loss/error; denied/off/resetting/unsupported states; successful connect received after cancellation. Capture writes and show no ARM/nonzero on setup failure. |
| H3 | Fake clock and delegate event traces: zero-before-ARM, no notification-only confirmation, subscription/read separation, current-operation write success plus fresh Status, control refusal/deadlines, queued STOP over pending ARM, safety-stop latch, malformed/old-generation callbacks, read deadline and no automatic recovery. Inject 199/200 ms submission gaps and backpressure with release during blockage; no catch-up writes. |
| H4 | Touch/state and UI tests: all six signs and capped magnitudes; zero axes; multiple touches; release, outside/re-entry, cancellation, lost tracking and 249/250 ms expiry; pre-arm and post-STOP stale touches; local/requested/reported state and age. Repeating the transmitted value must not renew input. |
| H5 | Lifecycle/cleanup fakes plus UI tests: inactive/lock signals, background, permission/power/service loss, disconnect and termination/relaunch state; zero and STOP failures/hangs independently; cancellation by the shared 500 ms deadline while runnable. Text/accessibility/Dynamic Type checks; accessibility activation cannot latch movement. |

Host/fake clocks establish software decisions, not scheduling guarantees, iOS radio delivery or physical stopping. UI Simulator evidence establishes layout/interaction logic only; it cannot resolve phone/firmware compatibility.

## Requirement and scenario coverage

Each row covers all scenarios under the named requirement; evidence IDs describe planned checks, not tests already implemented.

| Proposed requirement | Host/UI | Required phone/board evidence |
| --- | --- | --- |
| Explicit discovery and selection | H2 (two names/no results) | P1 discovery/selection; absence is inconclusive |
| Validate the connected car | H1/H2 (incompatibility/claim expiry) | P1 capacity/GATT/setup and I2 |
| Existing BLE wire contract | H1 (directions/wrap) | P1 accepted zero stream; P4 direction observations |
| Explicit zero then ARM | H3/H4 (old hold/refused or late ARM) | P2 controls; P4 no replay |
| Gentle single-axis directions | H1/H4 (signs/caps/disabled axes) | P3 interaction; P4 six directions |
| Hold cancellation and input expiry | H4 (release/cancel/expiry/outside) | P3 real touch; P4 release stop |
| Periodic latest-intent stream | H3 (backpressure/200 ms) | P2 stream workload and I3 |
| Validate Status and link health | H1/H3 (unchanged Status/read timeout/old callbacks) | P2 read/notify ordering; P3 loss |
| STOP and re-arm latch | H3/H4 (STOP/ARM/touch race) | P2 confirmation; P4 observed latch |
| Active foreground and manual recovery | H2/H5 (lock/off/loss/resume) | P3 interruptions/termination |
| Bounded best-effort shutdown | H5 (blocked zero/STOP/cancel) | P3 observed attempt/detection delays; P4 loss fallback |
| Visible state and evidence limits | H4/H5 (Armed without motion/accessibility) | P3 usability/labels; P4 distinguish wheel observation |

## Real phone / normal product firmware evidence planned

These are proposed procedures, **not authorization to run them**. No device, network, router, serial capture or motor activity occurred while writing this draft. Use the current normal product default or `ros` image; never `motor-bench`. Do not change firmware to make a phone pass.

* **P1 — payload/claim (I2, Q18):** with motor power disabled and a separately authorized batch, record phone model/iOS, Xcode/app/firmware revisions, capacity from `.withoutResponse`, raw capabilities/Status and, where authorized instrumentation is available, board ATT MTU for 10 manual connect/disconnect cycles. Require capacity>=72 and observed MTU>=75 every time; record setup latency versus claim_ms, claimed-disarmed state, rejection paths and release observation separately. Missing board MTU evidence leaves that measurement open even if zero writes work.
* **P2 — full streaming/control workload (I3, Q19):** with motor power disabled, run 60 s of 20 Hz zero setpoints, including Status polling and explicit ARM/STOP/re-arm at zero. Require no lease expiry, no submission gap>=200 ms and no one-second control/read-transaction timeout. Log gap distribution, readiness stalls, notification subscription/read ordering and control completion/read latency. Record connection interval only from actual authorized measurement. A phone that fails is unsupported pending reviewed changes; laptop results and a timer frequency do not establish iOS throughput or interval.
* **P3 — phone lifecycle and UI (I4):** with motor power disabled, exercise actual hold/release/slide/multiple-touch, app switcher, Home/background, lock/unlock, interruption, Bluetooth off/on, permission deny/change, service invalidation/disconnect, force termination/relaunch and a deliberately unavailable peer. Record callback and last-submission times, cleanup attempts and any device-reported stop. Capture UI evidence of manual recovery, ageing Status and no sticky VoiceOver activation. Never require cleanup to run after suspension/kill; verify firmware fallback separately. If a case cannot be induced, record it as unverified rather than a fake-test pass.
* **P4 — supervised physical acceptance:** only after P1–P3 and a newly announced batch with Todor's go, wheels off ground, test each gentle direction briefly, release-to-zero, STOP while holding, fresh ARM/new-press recovery, lock/background and link loss. Record Todor's wheel observations separately from app/GATT/firmware logs. Confirm the firmware's lease fallback under loss; host deadlines are not physical stopping bounds. Do not claim electrical/coast bounds, floor speed accuracy, range or unattended safety from this batch.

Each run records date, exact procedure, hardware/app/firmware identity, raw logs and observed failures. A physical run requires its own safe setup/cleanup announcement and Todor's confirmation. Existing [hardware policy](../../../README.md#hardware-validation-policy) and [product acceptance gates](../../../docs/SPECIFICATION.md#workflow-and-gates) apply. Failed or missing evidence leaves its task unchecked; changes to the contract return to proposal review.

## Draft checks

Authoring checks on 2026-10-20 (these validate documentation only):

* `rtk npm run spec:check`: active draft plus three baseline capabilities pass strict validation (4/4).
* `rtk npm run openspec -- validate --archived --strict --no-interactive`: both archived changes pass (2/2).
* `rtk npm run spec:build` regenerates `openspec/index.html`; `rtk npm run spec:reader:check` passes all six tests and freshness checking.
* Local Markdown review/check resolves all 34 relative links/anchors across the five authored documents; each of 12 added requirements has scenarios and an H/P mapping. Reviewed zero/ARM ordering, read/notify ambiguity, stop/input priority, deadlines, lifecycle recovery and explicit OPEN gates for consistency.
* `rtk git diff --check` passes. Accepted specs, roadmap, runtime/firmware and Wi-Fi files remain unchanged. No implementation or physical acceptance is implied; the artifact status “4/4 complete” means planning files exist only.
