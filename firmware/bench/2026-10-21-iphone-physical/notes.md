# 2026-10-21: Physical mode, motor power off

Build tested: `8fb8fd174d30`. Motor power bank off, ESP32 powered by USB;
Todor used the published iPhone client in Bluefy. No motor result is claimed.

Todor confirmed the requested motion stayed zero at neutral/release, followed
all three axes, and remained disarmed after STOP and mode changes. He then
reported unwanted disarming when scrolling an already armed page, an unnecessary
separate calibration tap before ARM, and status/messages hidden while scrolling.
He did not notice the fault message or report whether the drive pad was held.
The exact scroll-triggered failure remains unclassified.

Phone model and Stream write count/max gap/slowest write were not supplied.
The response does not establish a recorded 60 s sensor-plus-BLE timing result.
Task 3.1 remains open; this is a partial functional pass with UX defects.

## Host correction

Before the fix, two Session tests failed: fresh valid sensor input could not ARM
without separate calibration, and a released-pad sensor gap revoked arm. A real
page browser test also disconnected after a simulated 350 ms sensor pause during
scrolling, reproducing a possible trigger, not identifying Bluefy's actual cause.

The correction captures neutral within the explicit Set neutral & ARM action.
Released-pad sample gaps/reading posture keep arm and neutral at zero, with new
presses blocked until input is valid. Scroll cancels holds without recentering.
Sensor/posture loss during a held drive, invalid values, reference changes and
existing stream/lifecycle failures retain cleanup. Status/STOP stay visible.
At publication, corrected phone UX and sensor-driven motor behavior still needed acceptance.

Correction published as build `780f65eff594`: 46 Node tests and 23 source/built
browser tests pass; all 15 public assets match the clean build. The live-page
smoke check starts disarmed with no picker call. This is host/publication
evidence, not another phone or motor run.


## Corrected phone retest

Todor tested build `780f65eff594` in the announced motor-power-off procedure:
Physical → Enable sensors → connect → Set neutral & ARM; scroll with the pad
released, tilt to reading posture and return to neutral; check visible status/STOP,
then STOP and Set neutral & ARM again without separate calibration.

He replied “ok, seems to work wekk”. Record this as an operator-confirmed pass
for that UX retest (task 3.0), with no fault reported. This is a functional report,
not a captured event trace or identification of the original Bluefy trigger.
Phone model, 60 s neutral observations and Stream write count/max gap/slowest
write were not supplied. Timing/lifecycle task 3.1 and motor-on tasks remain open.


## 60-second neutral and stream check

On the announced motor-power-off check of build `780f65eff594`, Todor reported
an iPhone 11 and confirmed that requested motion stayed zero throughout the
60-second neutral hold, with no disconnect or stale-sensor message reported.
The procedure ended with release and STOP.

| Reported Stream value | Result |
| --- | --- |
| Write count | 1,366 |
| Maximum submission gap | 180 ms |
| Slowest completed write | 19 ms |

The observed maximum gap is 20 ms below the unchanged 200 ms client cutoff.
These are client-reported session counters, not a captured 60-second trace;
setup and other activity may contribute to the write count. No exact send rate,
radio delivery latency, physical stop time or raw angle-drift bound is inferred.
This confirms the requested-motion display stayed at zero during the announced
hold; the display's rounded values do not establish an exact raw-angle bound. The earlier sensor-only report identified iOS 18.7/Bluefy 3.9.3; their
versions were not re-entered in this response. No motor power was enabled.

Task 3.1's neutral/stream subcheck passes. Permission retry, held-input interruption,
reference/posture loss, lock/app-switch and fresh manual recovery on the actual
phone remain to be recorded; task 3.1 as a whole stays open. Motor-on acceptance
also remains open and needs its own announced batch and fresh go.


## Posture report and interface change

During the requested interruption test, Todor reported disarming with the message
“Phone moved outside the calibrated operating posture”. He thought it might occur
during z-axis rotation or loss of the pad press, and said holding the pad while
tilting was difficult, requesting a different interface. The triggering sensor
values and timing relative to lock/app-switch were not captured; neither those
lifecycle checks nor a specific rotation defect are accepted by this response.

Physical mode is being changed to explicit tap START/PAUSE after neutral/ARM,
without a held touch. START requires neutral; frame/sample expiry and STOP remain.
Posture limits stay unchanged and faults will include the angle/value/limit.
The old build's reported neutral/stream result remains valid for that build;
tap-based phone UX and motor-on behavior need new acceptance.


Tap-interface build `cef9e9bca8fe` is published: 48 Node tests and 26 source/built
browser scenarios pass, and 15 live files match its clean build. Public-page boot
stays paused/disarmed without a BLE picker call. START/PAUSE and requested motion
are in the sticky panel. New phone usability and interruption checks remain pending;
this work enabled no motor power and ran no motor batch.


## Redesigned controller phone retest — 2026-10-22

Todor tested build `2b5104400362` in Bluefy using the announced motor-power-off,
USB-powered procedure: both Buttons and Physical, direction/phone visualization,
requested motion, visible bottom ARM/START/PAUSE/STOP controls, mode-triggered
sensor setup without a separate Enable tap, PAUSE and STOP.

He replied “seems ok”. Record this as an operator-reported usability pass for
that procedure, with no permission error, disconnect, clipping or awkward
interaction reported. No new stream counters, sensor trace, phone/version details
or motor-on evidence were supplied. Full interruption/recovery checks, VoiceOver
and motor-on acceptance remain open. Future iPhone test requests must also send
the exact versioned build link through Todor's Telegram destination, as documented
in the [client testing workflow](../../../apps/iphone/README.md#phone-test-delivery).


Todor subsequently stated “i accept the UI” on 2026-10-22, explicitly accepting
the redesigned Buttons and Physical interface in build `2b5104400362`. This closes
the UI design review; phone interruption/recovery and motor-on acceptance remain
separate, open checks.


## Interruption and recovery phone check — 2026-10-22

The five-step procedure for build `2b5104400362` was delivered through Telegram.
Todor replied “works as expected” to the combined result question, confirming
the following with motor power off and the ESP32 powered over USB:

| Check | Operator-reported result |
| --- | --- |
| PAUSE, reading posture and neutral-only explicit restart | Zero while paused, arm retained, no delayed START replay |
| Lock for 5 s during running input, then unlock | Disarmed/zero on return, no automatic reconnect or restart |
| Switch apps for 5 s during running input, then return | Disarmed/zero on return, manual recovery required |
| Slowly leave the running operating posture, then return flat | Zero and fault cleanup; no automatic restart |
| Recover manually, run at neutral for 60 s, then PAUSE and STOP | Requested motion stayed zero; no issue reported |

This is a combined functional confirmation, without an event trace or exact
posture-fault wording. No new Stream write count, maximum gap, slowest write or
phone/version details were supplied. The earlier `780f65eff594` counters are not
reused as timing evidence for this build. Independent sensor/UI stalls, invalid
samples/reference changes, screen-orientation changes and permission denial/retry
were not separately exercised by this procedure; their phone checks remain open.
No motor-on result or measured stopping bound is inferred.


## Remaining phone-check response — 2026-10-22

For build `2b5104400362`, the next motor-power-off procedure requested displayed
per-axis caps (including combined input), current Stream counters, phone/browser
versions, screen-rotation recovery, and permission denial/retry if prompted.
Todor replied “completed. when the page scrolls it stops the tilt control”.

Record the procedure as reported completed and scrolling as reported to stop
running tilt input. Exact caps, Stream counters, updated versions, rotation-fault
text and whether a fresh permission prompt appeared were not supplied; no numeric
or case-specific pass is inferred from “completed”.

The product contract and `ui.js` intentionally pause Physical control on scroll,
zeroing input and preserving arm/neutral. The existing browser regression checks
zero, retained Armed state and explicit restart at neutral. Clarification was
requested whether the observed phone behavior is that pause or a disarm/disconnect.
No runtime change, motor run or firmware operation was performed for this report.
