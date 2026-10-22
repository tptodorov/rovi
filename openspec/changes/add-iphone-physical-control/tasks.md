## 1. Planning

- [x] 1.1 Record authorization and scoped sensor evidence; review complete proposal/spec/design for contradictions, validate strictly and regenerate the reader.

## 2. Host implementation

- [x] 2.1 Implement the Physical interpreter and meaningful tests for three signs, wrap, clamps, posture/reference guards, invalid values and independent freshness; verify pure tests (36 Node tests pass).
- [x] 2.2 Integrate mode switching and invalidation into the single Session; verify fake-GATT tests including ARM/STOP/mode races, cleanup and late callbacks without regressing Buttons (41 Node tests pass).
- [x] 2.3 Add explicit sensor permission, calibration, mode UI and held pad; verify real-page browser tests for permissions, lifecycle, pointer cancellation, switching and no replay (21 Chromium tests pass, including red/green outside-first-touch and detached-callback regressions).
- [x] 2.4 Update build/docs/evidence map, run complete iPhone/strict reader checks, publish a clean reviewable candidate and verify served assets (build `8fb8fd174d30`; all 15 live files match; local live-page boot stays disarmed with zero BLE picker calls).
- [x] 2.5 Correct reported calibration/scroll/status UX with failing reproductions; verify one-tap neutral/ARM, released-pad safety and visible status, then publish a clean candidate and check served assets (46 Node/23 source and built-page Chromium tests pass; build `780f65eff594`, all 15 live files match, public boot stays disarmed with zero picker calls).

- [x] 2.6 Replace the Physical held pad with explicit neutral-gated START/PAUSE, independent frame expiry and precise posture faults; reproduce tap-only failure, verify host/browser safety paths, publish and verify assets (48 Node/26 source and built-page Chromium scenarios pass; clean `cef9e9bca8fe`, 15 live files match; public boot paused/disarmed with zero picker calls).

- [x] 2.7 Apply UX/design guidelines to both controller modes: document flows/states/accessibility, reproduce compact-layout and extra-enable issues, implement responsive/light/dark controller and mode-triggered sensor setup, verify safety/layout/keyboard/diagnostics and publish a reviewable candidate (48 Node/37 source and built browser checks pass; `2b5104400362`, 15 public files match, live boot disarmed with no BLE picker, [PR #104](https://github.com/tptodorov/rovi/pull/104)).

## 3. Phone and physical acceptance

- [x] 3.0 Retest corrected one-tap neutral/ARM, released-pad scrolling and visible status/STOP on the phone with motor power off; record any fault reason (Todor confirmed build `780f65eff594` works in the announced retest; no fault reported, [evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#corrected-phone-retest)).

- [ ] 3.1 Record phone/browser details; confirm 60 s neutral/stream timing, signs/UI, permissions, freshness, reference/posture, lifecycle and manual recovery. UI is accepted; neutral, PAUSE/restart, lock/app-switch and posture cleanup/recovery have operator confirmation. Current-build cap/Stream values, exact versions and rotation/permission outcomes remain unrecorded; scroll pause versus disarm/disconnect needs clarification. Independent sensor/UI stalls and invalid/reference changes remain host-only ([phone evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md)).
- [ ] 3.2 Announce a bounded stand batch and obtain fresh go; verify all six signs, blended axes, caps, neutral/release, STOP and mode-switch stop with physical confirmation. Todor authorized and reported the moves performed on `2b5104400362`; the capture has eight nonzero periods ending at zero and final STOP/release/Open, but no re-ARM or subsequent move. Todor explicitly confirmed the final STOP-while-tilted/re-ARM and mode-switch checks in follow-up; these are operator evidence, with their absence from the capture unexplained. Client per-axis caps remain unconfirmed on the phone ([evidence and duration limits](../../../firmware/bench/2026-10-22-iphone-physical-stand/notes.md)); retain partial status.
- [ ] 3.3 Review actual phone/stand evidence and explicit limits; accept only the demonstrated scope and validate before archive.

## Workflow follow-up

Archive/sync only after tracked acceptance tasks are complete. Keep unmeasured
drift/stopping and wider device qualification limits explicit.
