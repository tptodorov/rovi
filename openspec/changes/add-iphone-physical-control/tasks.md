## 1. Planning

- [x] 1.1 Record authorization and scoped sensor evidence; review complete proposal/spec/design for contradictions, validate strictly and regenerate the reader.

## 2. Host implementation

- [x] 2.1 Implement the Physical interpreter and meaningful tests for three signs, wrap, clamps, posture/reference guards, invalid values and independent freshness; verify pure tests (36 Node tests pass).
- [x] 2.2 Integrate mode switching and invalidation into the single Session; verify fake-GATT tests including ARM/STOP/mode races, cleanup and late callbacks without regressing Buttons (41 Node tests pass).
- [x] 2.3 Add explicit sensor permission, calibration, mode UI and held pad; verify real-page browser tests for permissions, lifecycle, pointer cancellation, switching and no replay (21 Chromium tests pass, including red/green outside-first-touch and detached-callback regressions).
- [x] 2.4 Update build/docs/evidence map, run complete iPhone/strict reader checks, publish a clean reviewable candidate and verify served assets (build `8fb8fd174d30`; all 15 live files match; local live-page boot stays disarmed with zero BLE picker calls).

## 3. Phone and physical acceptance

- [ ] 3.1 Record phone model/iOS/Bluefy; confirm 60 s neutral stability, combined sensor/BLE stream timing, signs and UI behavior. Check denial/retry, stale sensor/UI, reference/posture changes, lock/app-switch, release and fresh manual recovery.
- [ ] 3.2 Announce a bounded stand batch and obtain fresh go; verify all six signs, blended axes, caps, neutral/release, STOP and mode-switch stop with physical confirmation.
- [ ] 3.3 Review actual phone/stand evidence and explicit limits; accept only the demonstrated scope and validate before archive.

## Workflow follow-up

Archive/sync only after tracked acceptance tasks are complete. Keep unmeasured
drift/stopping and wider device qualification limits explicit.
