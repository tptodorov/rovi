# Tasks

- [x] 1.1 Record Todor's implementation authorization (“do it”, 2026-10-28), define the observable behavior and pass strict proposal validation.
- [x] 2.1 Decode Network and support its optional BLE read/notifications, with legacy and malformed-data tests.
- [x] 2.2 Add serialized disarmed-only mode requests, fresh confirmation and disconnect; test success, refusal, timeout, stale callbacks and no ARM/nonzero input.
- [x] 2.3 Add the network panel and browser checks for status, controls, refusal, armed guards and reconnect guidance.
- [x] 3.1 Run Node, built-page Chromium and spec checks; document supported firmware, behavior and limits; publish a reviewable PR. **Done:** 63 Node, 43 built-page Chromium, strict specs/reader checks; [PR #119](https://github.com/tptodorov/rovi/pull/119).
- [x] 3.2 Verify the new controls on a real iPhone/Bluefy without arming: both modes, information and disconnect/reconnect. Record separately from host tests; no physical acceptance before this passes. **Done 2026-10-28:** Todor confirmed both network steps, fresh Home/Pending none after reconnect and clean final disconnect; [serial and operator evidence](../../../../firmware/bench/2026-10-28-iphone-network-controls/notes.md). The network-command phase is disarmed; earlier user motion is separate. Phone/browser versions are unrecorded.

## Workflow follow-up

Archive only after the required physical phone check. Home-mode range acceptance stays in its own change.
