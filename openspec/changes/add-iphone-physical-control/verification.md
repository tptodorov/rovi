# Verification map

| Contract | Host evidence | Required phone/physical evidence |
| --- | --- | --- |
| Default Buttons; permission only on tap; no automatic ARM | Browser and Session regressions | Bluefy mode/permission behavior |
| Three relative angles, wrap, dead zone, per-axis clamps | Physical unit tests | Phone signs, held twist, 60 s neutral drift; stand mixed axes |
| Independent sample/UI deadlines and no old input replay | Fake-clock input and Session tests, browser stalls | Phone stalls and observed manual recovery |
| Mode changes supersede ARM, clear/disarm/STOP | Fake-GATT race tests; real page switch while held | Phone and separately authorized stand stop |
| Lifecycle/reference/invalid sample cleanup; held posture failure | Unit/Session/browser tests | Phone lock/app switch/orientation and stand interruption |
| One-tap neutral/ARM; released-pad gaps/posture; visible status/STOP | Red/green Session and scroll browser regressions | Todor confirmed corrected UX on build `780f65eff594`, motor power off |
| Existing stream/Status/cleanup limits | Existing and new Session regressions | Actual sensor-plus-BLE gaps, STOP/readback; physical observation |

Pre-implementation sensor-only [phone result](../../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
supports the selected interaction, not full product acceptance. Task 3.0 has a
scoped phone UX pass; sensor-driven motor acceptance is not claimed. The first integrated
[motor-power-off response](../../../firmware/bench/2026-10-21-iphone-physical/notes.md)
confirmed signs/neutral/release/STOP/mode changes, but reported scrolling disarm,
hidden messages and the extra calibration tap. Phone model and Stream metrics
were not supplied. Todor subsequently confirmed the corrected phone UX on build
`780f65eff594`; the original scroll trigger remains unclassified.

Host evidence before the UX correction: 42 Node tests and 21 Chromium tests passed (source and built site). Physical unit tests
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

Candidate published at <https://tptodorov.github.io/rovi/iphone/?v=8fb8fd174d30>
from clean build `8fb8fd174d30`. All 15 live files matched the built bytes. A
Linux browser loaded that public site, checked its own revision, selected
Physical while disconnected and remained disarmed with zero BLE picker calls.
A detached old sensor callback cannot replace a newly calibrated/armed input
(browser regression). This publication is not phone/BLE or motor acceptance.

UX regression reproduction: targeted Node command selecting `captures neutral`
and `released Physical` failed both tests before the change; browser command
selecting `scrolling while released` failed with disconnected rather than ready.
The browser injects a 350 ms sensor pause during scrolling; this reproduces a
possible trigger rather than proving what Bluefy did. Corrected tests cover
neutral capture/new press, rejected stale/invalid/posture ARM, preserved idle
neutral/arm, rejected-press no replay, scroll cancellation and sticky fault reason.
Real stream/lifecycle deadlines and held-drive expiry remain covered separately.
After correction, all 46 Node tests and 23 source and built-page Chromium scenarios pass.
The connected Buttons layout still fits the 390×844 viewport; larger text,
STOP under a held finger and existing lifecycle behavior remain covered.

Corrected candidate: <https://tptodorov.github.io/rovi/iphone/?v=780f65eff594>.
All 15 public files match clean build `780f65eff594`. A public-page Chromium
check confirmed Buttons default, the combined Physical ARM label, no separate
calibration button, disarmed state and zero picker calls/page errors. Pages
deployment completed successfully. Todor confirmed the announced motor-power-off
UX retest works, completing task 3.0. This supplies no Stream metrics, full-minute
neutral measurement or motor-on result; tasks 3.1–3.3 remain open.
