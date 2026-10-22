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

- [x] 3.0 Retest corrected one-tap neutral/ARM, released-pad scrolling and visible status/STOP on the phone with motor power off; record any fault reason (Todor confirmed build `780f65eff594` works in the announced retest; no fault reported, [evidence](../../../../firmware/bench/2026-10-21-iphone-physical/notes.md#corrected-phone-retest)).

- [x] 3.1 Close scoped phone acceptance: recorded UI, neutral, PAUSE/restart, scroll retaining arm/connection, lock/app-switch and posture cleanup/manual recovery. On 2026-10-22 Todor accepted the tested scope and explicitly deferred current-build cap/Stream numbers, exact version reconfirmation, native rotation/permission outcomes and phone-only fault injection. These remain unverified, with host coverage retained ([accepted review](verification.md#scoped-acceptance-review), [phone evidence](../../../../firmware/bench/2026-10-21-iphone-physical/notes.md)).
- [x] 3.2 Close scoped stand acceptance: fresh go obtained; directions/blends, neutral/PAUSE, STOP while tilted, re-ARM stillness and mode-switch stopping operator-confirmed. Capture independently shows eight zero-ending command periods and final STOP/release/Open; the final re-ARM/mode-switch sequence is not captured. Todor explicitly deferred numeric phone cap evidence in the accepted review. Duration overruns and unmeasured stopping remain limits, not passed checks ([stand evidence](../../../../firmware/bench/2026-10-22-iphone-physical-stand/notes.md), [accepted review](verification.md#scoped-acceptance-review)).
- [x] 3.3 Validate and finalize the recorded scoped acceptance before archive. Todor selected “Accept tested scope; defer remaining checks” on 2026-10-22; the [review](verification.md#scoped-acceptance-review) retains every explicit limit. Strict validation passes (7 items); reader tests (6) and freshness pass. Deferred measurements are not passed checks.

## Workflow follow-up

Archive/sync only after tracked acceptance tasks are complete. Keep unmeasured
drift/stopping and wider device qualification limits explicit.
