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

- [ ] 3.1 Record phone model/iOS/Bluefy; confirm 60 s neutral stability, combined sensor/BLE stream timing, signs and UI behavior. Check denial/retry, stale sensor/UI, reference/posture changes, lock/app-switch, release and fresh manual recovery. Neutral/stream subcheck passed on reported iPhone 11: zero throughout 60 s, 1,366 writes, max gap 180 ms, slowest write 19 ms ([evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#60-second-neutral-and-stream-check)); redesigned Buttons/Physical UI, tap START/PAUSE and mode-triggered setup passed the operator-reported motor-power-off retest on `2b5104400362` ([evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#redesigned-controller-phone-retest--2026-10-22)). Todor explicitly accepted the redesigned UI on 2026-10-22 (“i accept the UI”). The five-step `2b5104400362` PAUSE/restart, lock, app-switch, posture cleanup/manual recovery and 60 s neutral procedure also passed by operator report ([evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#interruption-and-recovery-phone-check--2026-10-22)). Current-build Stream numbers, independent sensor/UI stalls, invalid/reference and screen-orientation changes, and permission denial/retry remain unrecorded on the phone; this task stays open.
- [ ] 3.2 Announce a bounded stand batch and obtain fresh go; verify all six signs, blended axes, caps, neutral/release, STOP and mode-switch stop with physical confirmation. Todor authorized and reported the moves performed on `2b5104400362`; the capture has eight nonzero periods ending at zero and final STOP/release/Open, but no re-ARM or subsequent move. Todor explicitly confirmed the final STOP-while-tilted/re-ARM and mode-switch checks in follow-up; these are operator evidence, with their absence from the capture unexplained. Client per-axis caps remain unconfirmed on the phone ([evidence and duration limits](../../../firmware/bench/2026-10-22-iphone-physical-stand/notes.md)); retain partial status.
- [ ] 3.3 Review actual phone/stand evidence and explicit limits; accept only the demonstrated scope and validate before archive.

## Workflow follow-up

Archive/sync only after tracked acceptance tasks are complete. Keep unmeasured
drift/stopping and wider device qualification limits explicit.
