# Verification map

| Contract | Host evidence | Required phone/physical evidence |
| --- | --- | --- |
| Default Buttons; permission only on tap; no automatic ARM | Browser and Session regressions | Bluefy mode/permission behavior |
| Three relative angles, wrap, dead zone, per-axis clamps | Physical unit tests | Phone signs/held twist confirmed; 60 s requested-zero hold confirmed, raw drift unmeasured; stand mixed axes pending |
| Independent sample/foreground deadlines and no input replay | Fake-clock input and Session tests, browser stalls | Phone stalls and observed manual recovery |
| Mode changes supersede ARM, clear/disarm/STOP | Fake-GATT race tests; real page switch while held | Phone and separately authorized stand stop |
| Lifecycle/reference/invalid sample cleanup; running posture failure | Unit/Session/browser tests | Phone lock/app switch/orientation and stand interruption |
| One-tap neutral/ARM; released-pad gaps/posture; visible status/STOP | Red/green Session and scroll browser regressions | Todor confirmed corrected UX on build `780f65eff594`, motor power off |
| Existing stream/Status/cleanup limits | Existing and new Session regressions | Reported iPhone 11: 1,366 writes, max gap 180 ms, slowest write 19 ms; physical observation pending |

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
UX retest works, completing task 3.0. A subsequent 60 s neutral hold on reported
iPhone 11 stayed at requested zero without disconnect/stale-sensor messages, with
1,366 writes, max gap 180 ms and slowest write 19 ms. These session counters do
not define an exact 60 s trace, send rate, delivery timing or raw drift bound.
The 200 ms client cutoff is unchanged. Task 3.1 remains open for phone
interruption/recovery checks; tasks 3.2–3.3 still require motor-on evidence/review.


Tap-based interface correction: a real-page regression failed before the change
because a tap followed by tilt stayed at zero after finger release. It now checks
hands-free continuous input, PAUSE, neutral-only restart and STOP/new ARM remaining
paused. Tests also cover early START during pending ARM, source-specific posture
faults, independent sample/foreground expiry, keyboard START/PAUSE and no restart
after blur/hidden/screen orientation loss. Previous held-pad phone UX/timing results
are historical evidence; the new interaction requires a motor-power-off retest.

Current tap-interface host checks: 48 Node tests and 26 source and built-page
Chromium scenarios pass, including the original Buttons layout and STOP/lifecycle checks.
The sticky Physical panel includes requested motion so scrolling to read it does
not interfere with a running test.


Tap-interface candidate: <https://tptodorov.github.io/rovi/iphone/?v=cef9e9bca8fe>.
Pages deployment succeeded and all 15 live files match that clean build. A public
Chromium check confirmed default Buttons, Physical START paused/disabled while
disconnected, requested zero, sticky controls visible, local disarm and zero
picker calls/page errors. Tap-based phone usability/interruptions and motor-on
acceptance remain open; no phone result is inferred from this publication.


Controller redesign: the 320×568 layout regression failed before the redraw
(direction bottom 696 px); mode-triggered setup failed with zero permission calls
until a separate Enable tap. New tests cover 320×568, 375×667 and 390×844 control
visibility above the dock, native radio keyboard/focus, dark mode, 200% text,
landscape, diagnostics pausing and permission interrupted by blur/STOP. Enlarged
text additionally exposed focused controls hidden behind sticky status; scroll
padding now keeps focus between status and dock. Representative enabled text
contrast is checked at 4.5:1 in both palettes. These are host checks, not VoiceOver,
Bluefy native-permission or motor acceptance. STOP remains immediate and modes
remain unpersisted, deliberately following the control contract over generic UI
recommendations for confirmations or URL state.

Controller-redesign host result: 48 Node tests and 37 source and built-page Chromium
scenarios pass. Reviewed screenshots show both modes at 390×844 and compact Buttons at
320×568, plus landscape and dark 200% text.


Redesigned controller candidate:
<https://tptodorov.github.io/rovi/iphone/?v=2b5104400362>, [PR #104](https://github.com/tptodorov/rovi/pull/104)
stacked on #102. Pages deployment succeeded; all 15 live files match the clean
build. A live Chromium smoke check confirmed default Buttons with no sensor prompt,
explicit Physical setup, Retry hidden after fresh readings, local disarm/zero,
and no BLE picker calls/page errors. A preview tab is open in this workspace.
On 2026-10-22, Todor replied “seems ok” to the announced motor-power-off,
USB-powered Bluefy retest of both modes, mode-triggered sensor setup, visible
requested motion/dock controls, PAUSE and STOP. Record this as a scoped usability
pass with no issue reported ([phone evidence](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#redesigned-controller-phone-retest--2026-10-22)).
No new timing trace or motor-on result is inferred. Full phone interruption/recovery,
VoiceOver and motor-on acceptance remain open.


UI acceptance: Todor explicitly stated “i accept the UI” on 2026-10-22 for the
published redesigned Buttons/Physical interface (`2b5104400362`). The UI design
review is complete. This does not close task 3.1's remaining interruption/recovery
checks or tasks 3.2–3.3's stand evidence and overall acceptance review.


Phone interruption/recovery update (2026-10-22): Todor reported “works as expected”
for the five-step motor-power-off procedure on `2b5104400362`: PAUSE/neutral-only
restart, lock, app switching, running posture failure/manual recovery and a 60 s
neutral run ending in PAUSE/STOP. See the [recorded procedure and limits](../../../firmware/bench/2026-10-21-iphone-physical/notes.md#interruption-and-recovery-phone-check--2026-10-22).
Current-build Stream counters were not supplied. Independent sensor/UI stalls,
invalid/reference changes, screen-orientation changes and permission denial/retry
still have host coverage only; task 3.1 remains partial. Motor-on tasks 3.2–3.3
remain open.

## Next stand batch

Pending a fresh go: rover on its stand, all wheels clear, motor bank awake and
USB connected; use the same published `2b5104400362` build. No flash is needed.
Capture serial output before enabling motor power. In Physical mode, capture
neutral with Set neutral & ARM and use explicit START at neutral for each move.

- Six single-axis moves: forward, backward, strafe left/right, rotate left/right.
- Two blended moves: forward + left strafe; then forward + left strafe + left turn.
- One forward move ending with STOP; verify a new ARM stays paused until START.
- One forward move ending by switching to Buttons; verify disarm and stillness.

Each move lasts at most 2 s, with at least 3 s at zero between moves (at most 20 s
of requested motion in total). For the first eight moves return to neutral,
observe zero, then PAUSE. Use offsets up to 20° to check the displayed gentle caps
of 0.1 m/s per translation axis and 0.3 rad/s rotation, subject to advertised limits.
The final two moves exercise STOP and mode-change stopping while tilted. Finish
with STOP/disconnect and motor power off. Any unexpected motion or failure to
stop aborts the batch. Record serial commands separately from Todor's physical
wheel confirmation; do not infer measured velocity or stopping time.
