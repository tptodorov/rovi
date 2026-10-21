# Verification map

| Contract | Host evidence | Required phone/physical evidence |
| --- | --- | --- |
| Default Buttons; permission only on tap; no automatic ARM | Browser and Session regressions | Bluefy mode/permission behavior |
| Three relative angles, wrap, dead zone, per-axis clamps | Physical unit tests | Phone signs, held twist, 60 s neutral drift; stand mixed axes |
| Independent sample/UI deadlines and no old input replay | Fake-clock input and Session tests, browser stalls | Phone stalls and observed manual recovery |
| Mode changes supersede ARM, clear/disarm/STOP | Fake-GATT race tests; real page switch while held | Phone and separately authorized stand stop |
| Lifecycle/reference/posture/invalid sample cleanup | Unit/Session/browser tests | Phone lock/app switch/orientation and stand interruption |
| Existing stream/Status/cleanup limits | Existing and new Session regressions | Actual sensor-plus-BLE gaps, STOP/readback; physical observation |

Pre-implementation sensor-only [phone result](../../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
supports the selected interaction, not these product acceptance gates. No task 3
or sensor-driven rover result is claimed yet.

Host evidence: 42 Node tests and 20 Chromium tests pass (source and built site). Physical unit tests
cover signs/wrap/caps/invalid/reference/posture and separate deadlines. Session
tests cover confirmed ARM/new hold, all-three wire values, mode/ARM races,
independent expiry and cleanup. Real-page tests cover permission denial/late
completion, switching while held, sensor/UI loss, screen rotation and manual
recovery. An outside-first-touch regression failed with wz=0.30 before the
primary/contact gate and passes at zero afterward. Original button tests remain
green, including STOP touch-down and layout with larger text.

The build regression failed on unversioned imports before the publication fix.
It now checks every HTML/script/style/module edge uses one revisioned filename
and records that revision in the loaded HTML. The controller footer and copied
diagnostic report identify the loaded build, even after a newer deployment.
CI runs browser scenarios against the built site. These checks do not claim
phone/BLE scheduling, sensor-driven motors or a physical stopping bound.
