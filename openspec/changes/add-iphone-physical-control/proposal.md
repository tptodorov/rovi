# iPhone physical control

## Why

Todor wants the existing iPhone client to offer button and three-axis physical
control. The isolated phone test confirms sensor access and the intended local
interaction; it does not establish sensor-plus-BLE or motor acceptance.

## What Changes

- Add a Buttons / Physical selector, defaulting to Buttons without persistence.
- In Physical mode, explicitly enable orientation sensors, calibrate neutral while
  disarmed, then ARM and hold a drive pad. Combine forward/back tilt, strafe tilt
  and heading offset into gentle vx/vy/wz. Holding a twist sustains rotation.
- Invalidate motion on mode changes, stale/invalid sensor input, posture changes
  or interruptions. Preserve existing STOP, stream, cleanup and manual recovery.
- Keep the isolated diagnostic available and separate phone/radio/motor evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `iphone-ble-teleoperation`: selectable button/physical input, calibration,
  bounded blended velocities, independent sample/hold expiry and mode switching.

## Impact

`apps/iphone/` input, session and UI code, tests, static build and documentation.
No new dependency, wire format, firmware or native app. Physical mode is supported
initially for portrait, screen-up use near the phone posture tested by Todor.

## Proposal acceptance

Todor explicitly requested the same-client mode switch and all three mappings,
then chose angle-from-neutral rotation on 2026-10-21. After the sensor-only run,
he confirmed the three previews, held twist and neutral/release behavior, with
expiry only during deliberate interruptions. This authorizes the selected host
implementation. [Phone evidence](../../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
records the partial export, 13.1 s quiet sample and timing limits. Motor tests
still require a separately announced batch and fresh go; acceptance/archive are
not implied by this implementation authorization.
