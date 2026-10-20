# iPhone BLE client tasks

**Draft only.** These are future implementation/acceptance tasks, not permission to begin them. Nothing below is complete. [Proposal acceptance](proposal.md#proposal-acceptance) controls task 1.1; [verification map](verification.md) defines H/P evidence. Resolve changes to the proposed contract through review before implementation.

## 1. Acceptance and prerequisites

- [ ] 1.1 Record Todor's explicit acceptance of the concrete proposal and selected scope; completion check: link the exact acceptance message/PR comment in proposal.md and here. Documentation merge alone is insufficient. Do not start 2.x/3.x beforehand.
- [ ] 1.2 Resolve OPEN I1 and review I4; completion check: record Mac/Xcode/macOS access, target phone/iOS, supported deployment minimum, signing team/bundle ID and reviewed six-control/hold approach in design.md, without credentials. No scaffold until I1 is resolved; any interaction contract change updates the spec first.

## 2. App foundation and BLE session

- [ ] 2.1 Create the minimal native app/test targets under apps/iphone using the agreed environment and no third-party runtime dependencies; completion check: reproducible xcodebuild build/test commands pass on the selected Mac with a fake transport, and README records prerequisites, signing and simulator evidence limits.
- [ ] 2.2 Implement version-1 codecs and capability/Status validation; completion check: H1 golden, malformed, boundary and wrapping-sequence tests pass against existing Rust/Python vectors, with no firmware/API changes.
- [ ] 2.3 Implement explicit scan/select/connect, GATT validation, payload/window checks and bounded setup; completion check: H2 fake-central tests cover same-name candidates, unavailable Bluetooth/denied permission, incompatible peer, claim loss, connect timeout and late callbacks, without automatic connection or motion.
- [ ] 2.4 Implement serialized controls/Status reads and the 50 ms latest-intent stream with local generations/deadlines; completion check: H3 verifies fresh-zero ARM, read/notification separation, backpressure, stop priority, 200 ms failure, 1000 ms read deadline, refusal and stale callbacks. Document which confirmations remain device-reported.

## 3. Touch UI and interruption handling

- [ ] 3.1 Implement six gentle hold controls and visible local/reported state with ARM, STOP and Disconnect; completion check: H4 verifies all direction signs/limits, disabled axes, multi-touch, release/slide/cancel, 250 ms input expiry, pre-arm touch invalidation and no motion replay.
- [ ] 3.2 Implement active-foreground cancellation, permission/Bluetooth/service-loss handling and bounded cleanup; completion check: H5 injects every interruption and a failed/hung write, verifies cancellation attempted within 500 ms and no late re-arm/reconnect. UI tests check labels, Dynamic Type, non-color state cues and no sticky accessibility activation. Document the approved I4 hold interaction and physical checks still pending.

## 4. Real-phone verification and scoped acceptance

- [ ] 4.1 After separate authorization, run P1 on the selected iPhone and unchanged normal product firmware with motor power disabled; completion check: record 10 manual connection trials, phone capacity and board ATT MTU, claim/setup behavior, revisions and logs. Resolve I2/Q18 for that phone only; fakes/BlueZ do not pass this task.
- [ ] 4.2 After separate authorization, run P2's complete zero-stream/read/control workload; completion check: record 60 s at 20 Hz, gap/backpressure/control timing, zero lease expiries/timeouts, and resolve I3/Q19 for that phone only. A failed gate requires proposal revision before motor checks; do not quietly relax bounds.
- [ ] 4.3 After separate authorization, run P3 with motor power disabled; completion check: real touch and accessibility review plus inactive/background/lock, permission/off/reset, disconnect, termination and relaunch evidence show cleared intent/manual recovery, with unconfirmed cleanup clearly labelled. Resolve I4 and distinguish measured app behavior from firmware fallback.
- [ ] 4.4 Announce the exact P4 supervised wheels-off-ground batch and obtain Todor's go before any motor run; completion check: record Todor's physical confirmation for six directions, release, STOP/re-arm and loss stops on normal firmware, plus phone/app/firmware identities and separate observation vs protocol evidence.
- [ ] 4.5 Review every requirement/scenario against H1–H5 and P1–P4, with all implementation gates resolved; completion check: publish dated evidence and Todor's scoped product acceptance, run strict active/baseline/archived validation and reader checks, and leave any unmeasured physical bound explicitly unclaimed. Draft-authoring checks alone cannot close this task.

## Workflow follow-up

* After implementation and product acceptance, archive with the pinned CLI under [the specification standard](../../../docs/SPECIFICATION.md#workflow-and-gates); never move deltas manually.
* Review generated baseline purpose, requirement/scenario coverage and links in baseline/archive, regenerate the reader, and repeat strict and archived validation plus reader checks in the archive PR. Parent owns merge coordination; do not auto-merge this draft.
