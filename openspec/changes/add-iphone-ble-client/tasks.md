# iPhone BLE web controller tasks

**Draft only.** These are future implementation/acceptance tasks, not permission to begin them. [Proposal acceptance](proposal.md#proposal-acceptance) controls task 1.1; [verification map](verification.md) defines H/P evidence. Bluefy delivery and button-first input are selected; the complete revised proposal is not accepted. All boxes remain unchecked.

## 1. Acceptance and environment

- [ ] 1.1 Record Todor's explicit acceptance of the revised concrete proposal; completion check: link the exact acceptance message/PR comment in proposal.md and here. Documentation merge alone is insufficient. Do not start implementation beforehand.
- [ ] 1.2 Record iPhone model/iOS/Bluefy version and required browser API/lifecycle feature checks; completion check: design.md records the proposed GitHub Pages route and identifies the delivery/phone observations still pending in I1–I4. No Mac, Xcode, signing credentials or membership is required for this web route.

## 2. Linux development and tests

- [ ] 2.1 Implement codecs/capability/Status validation under apps/iphone; completion check: H1 Linux tests match existing Rust/Python vectors for 72-byte zero/six-direction setpoints, f32 limits, sequence wrap and malformed/boundary inputs. No firmware/API changes.
- [ ] 2.2 Implement the minimal static button UI and expiring intent policy; completion check: H4/H5 Linux browser tests cover six signs/caps, disabled axes, simultaneous touches, release/slide/cancel/lost capture, stalled UI renewal, 250 ms expiry, text/accessibility and no sticky sensor/accessibility input. Sensor APIs remain unused.
- [ ] 2.3 Implement the Web Bluetooth adapter with explicit picker/connect, feature/GATT validation and bounded setup; completion check: H2 exercises missing APIs, insecure origin, cancelled/late picker selection, invalid peer/zero-write rejection, claim loss, connect timeout and late completion without automatic recovery. Do not implement native scan/MTU/readiness APIs.
- [ ] 2.4 Implement serialized GATT/control/Status work and periodic latest-intent transmission; completion check: H3 passes zero-before-ARM, read-promise-only confirmation, STOP priority, safety-stop latch, 200 ms gap/backpressure, read/control deadlines and stale-promise/event cases, with no queued motion.
- [ ] 2.5 Implement page/host interruption handling and bounded cleanup; completion check: H5 injects blur/hidden/pagehide/verified host inactivity, disconnect/error, reload and failed/hung writes, verifying disconnect attempted within 500 ms while runnable. README documents firmware fallback, browser evidence limits and the later input-interpreter boundary without implementing sensors.

## 3. Reviewable HTTPS delivery

- [ ] 3.1 Prepare deployment to the selected origin without publishing; completion check: Linux checks/build pass, static output contains only first-party runtime assets/no secrets, version/revision is visible and the documented deployment/rollback is reviewable. Host simulation is labelled as such.
- [ ] 3.2 Publish within an authorized implementation/delivery task; completion check: record HTTPS URL and deployed revision, verify required feature checks render correctly on iPhone, and document opening the page in Bluefy rather than Safari. This specification-authoring PR does not authorize publication.

## 4. Phone and physical acceptance

- [ ] 4.1 After separate authorization, run P1 on the selected Bluefy/iPhone and unchanged normal firmware with motor power disabled; completion check: record actual API/picker/GATT behavior and 10 manual connection cycles with zero+explicit ARM/Status at zero. Retain Todor's phone payload confirmation as user evidence; record MTU only if measured. No new sensor/native API assumptions.
- [ ] 4.2 After separate authorization, run P2's full 60 s/20 Hz zero-stream/read/control workload; completion check: no lease expiry, >=200 ms submission gap or one-second control/read timeout; record promise/backpressure/gap/control timing. Resolve I3/Q19 for this browser/phone only. Failure requires reviewed changes before motor checks.
- [ ] 4.3 After separate authorization, run P3 with motor power disabled; completion check: verify real touch/accessibility, browser UI, blur/background/lock, observed permission/Bluetooth loss, navigation and termination/reload. Any sustained inactive nonzero renewal fails host support; document unobservable signals and firmware fallback separately. Resolve I1/I4 compatibility, not merely browser layout tests.
- [ ] 4.4 Announce the exact P4 supervised wheels-off-ground batch and obtain Todor's go before any motor run; completion check: record Todor's physical confirmation of all directions, release, STOP/re-arm and interruption/link-loss stops with phone/Bluefy/web/firmware revisions.
- [ ] 4.5 Review all 13 requirements/scenarios against H1–H5 and P1–P4; completion check: dated evidence, resolved gates, Todor's scoped product acceptance, strict active/baseline/archived and reader checks. Unmeasured physical bounds remain unclaimed. Draft-authoring checks cannot close this task.

## Workflow follow-up

* After implementation and acceptance, archive with the pinned CLI under [the specification standard](../../../docs/SPECIFICATION.md#workflow-and-gates), review baseline/archive purposes, scenarios and links, regenerate the reader and repeat strict/archived/reader checks.
* Sensor input is later work: review permissions, mapping/calibration, expiry, explicit activation/deadman and input switching in its own proposal. No v1 motion-permission prompt or unused sensor/plugin framework.
* Parent owns merge coordination; do not auto-merge this draft.
