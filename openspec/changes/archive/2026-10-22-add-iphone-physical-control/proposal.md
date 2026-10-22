# iPhone physical control

## Why

Todor wants the existing iPhone client to offer button and three-axis physical
control. The isolated phone test confirms sensor access and the intended local
interaction; it does not establish sensor-plus-BLE or motor acceptance.

## What Changes

- Add a Buttons / Physical selector, defaulting to Buttons without persistence.
- In Physical mode, explicitly enable orientation sensors, then capture neutral
  and ARM in one tap while disarmed, followed by an explicit START at neutral. Combine forward/back tilt, strafe tilt
  and heading offset into gentle vx/vy/wz. Holding a twist sustains rotation.
- Block motion on unavailable input; sensor/posture loss while driving ends the
  session. Paused sensor gaps or reading posture retain neutral and arm at
  zero. Preserve STOP, stream, cleanup and lifecycle recovery. Keep safety status
  and STOP visible while scrolling.
- Keep the isolated diagnostic available and separate phone/radio/motor evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `iphone-ble-teleoperation`: selectable button/physical input, calibration,
  bounded blended velocities, independent sample/foreground expiry and mode switching.

## Impact

`apps/iphone/` input, session and UI code, tests, static build and documentation.
No new dependency, wire format, firmware or native app. Physical mode is supported
initially for portrait, screen-up use near the phone posture tested by Todor.

## Proposal acceptance

Todor explicitly requested the same-client mode switch and all three mappings,
then chose angle-from-neutral rotation on 2026-10-21. After the sensor-only run,
he confirmed the three previews, held twist and neutral/release behavior, with
expiry only during deliberate interruptions. This authorizes the selected host
implementation. [Phone evidence](../../../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
records the partial export, 13.1 s quiet sample and timing limits. Motor tests
still require a separately announced batch and fresh go; acceptance/archive are
not implied by this implementation authorization.

The first motor-power-off integrated test passed directions, neutral/release,
STOP and switching. Todor then reported unwanted disarming during scrolling,
hidden status and the unnecessary separate calibration tap. This authorizes the
one-tap neutral/ARM and released-pad UX correction. Todor subsequently confirmed
the announced corrected phone retest on build `780f65eff594` works with motor
power off. He subsequently confirmed a 60 s neutral hold at requested zero on
iPhone 11: 1,366 writes, max gap 180 ms, slowest write 19 ms. The original
Bluefy trigger remains unclassified; remaining phone interruption/recovery and
motor-on acceptance remain open.


Todor then reported posture-triggered disarming and difficulty keeping the drive
pad pressed while moving the phone, explicitly requesting a different interface.
Replace only Physical mode's held pad with tap START/PAUSE, a neutral prerequisite
for START and a separate frame-renewal deadline. Buttons retain press-and-hold.
Keep the existing posture limits and identify the exact failing angle in messages;
the reported rotation/drop-of-touch cause is not yet established. This UI change
requires another motor-power-off retest; earlier held-pad results remain historical.


Todor additionally requested a production-quality iPhone controller UI for both
modes using design skills. This authorizes a compact responsive redesign, native
segmented mode selection, clear state/feedback, a persistent action/STOP dock,
collapsible diagnostics and sensor setup on explicit Physical selection. Preserve
control safety and existing hosting; validate new usability on phone before acceptance.


## Product acceptance — 2026-10-22

After accepting the redesigned UI and confirming phone/stand behavior, Todor
explicitly selected “Accept tested scope; defer remaining checks”. Physical mode
is accepted for supervised use on the reported iPhone setup, calibrated in the
tested portrait/screen-up posture. The [acceptance review](verification.md#scoped-acceptance-review)
records completed functional checks and deferred phone cap/Stream numbers, native
rotation/permission outcomes and phone-only fault injection. Historical timing,
capture discrepancies, duration overruns and unmeasured stopping remain explicit;
this does not claim wider device, floor-use or measured stopping qualification.
