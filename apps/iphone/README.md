# iPhone BLE controller

Static HTTPS controller for Bluefy on iPhone, with Buttons and Physical modes plus
an isolated phone-sensor diagnostic. Physical mode is accepted for supervised
use on the reported iPhone setup, with explicit qualification deferrals in the
[scoped acceptance review](../../openspec/changes/archive/2026-10-22-add-iphone-physical-control/verification.md#scoped-acceptance-review).
Implementation is authorized by the [proposal acceptance record](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/proposal.md#proposal-acceptance).
Buttons implementation and scoped phone/stand acceptance completed on 2026-10-21:
six directions/stops confirmed, ATT MTU 247 on iPhone 11 Pro/Bluefy, and
STOP-while-holding fixed and rechecked. [Acceptance and limits](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/tasks.md#4-phone-and-physical-acceptance)
retain the unexplained early drops, P2 closure decision and skipped VoiceOver check.

## Commands

From this directory:

* `rtk npm test` — codec, hold, transport and session tests (Node). Python 3 compares
  bytes with the existing product wire reference.
* `rtk npm run test:browser` — Chromium tests against a local HTTPS server with a fake
  Web Bluetooth peer. Needs `openssl`; on NixOS:
  `rtk nix shell nixpkgs#openssl --command env ROVI_CHROMIUM=/run/current-system/sw/bin/chromium npm run test:browser`.
* `rtk npm run build` — copies runtime assets and `version.txt` into `dist/iphone/`.
  It gives entry assets and every imported module revisioned filenames to avoid
  mixing cached releases. The loaded HTML records its own build revision; it does
  not infer that from a newer server deployment. It deploys nothing.
  Add `ROVI_IPHONE_DIST=1` to the browser-test
  environment to test the built site after building it.
* `./publish.sh` — builds a clean commit and pushes it to the `gh-pages` branch, served at
  <https://tptodorov.github.io/rovi/iphone/>. The page footer and `version.txt` show the
  revision. Rollback: `git push --force-with-lease origin <older-gh-pages-commit>:gh-pages`.
  For candidate validation, open the client URL with `?v=<revision>` to fetch fresh
  HTML. Existing tabs keep their loaded code; they do not migrate live sessions.

Open the URL in **Bluefy**, not Safari: Safari has no Web Bluetooth.

No test touches a BLE device. Runtime assets use no third-party scripts or command server.

## Layout

| File | Role |
| --- | --- |
| `wire.js` | 72-byte setpoint, capabilities and Status codecs |
| `hold.js` | Input interpreter: press-and-hold intent with 250 ms expiry |
| `physical.js`, `angles.js` | Calibrated three-axis input, posture guards and independent sensor/hold deadlines |
| `transport.js` | Web Bluetooth picker, GATT validation and reads/writes |
| `session.js` | One owner of claim, zero+ARM, STOP latch, stream, deadlines and cleanup |
| `ui.js` | Pointer/lifecycle wiring and rendering |

Physical input is an interpreter beside `hold.js`, selected by the control-mode
switch. It supplies intent to the existing Session and cannot arm or write GATT.

## Physical mode

The [accepted implementation scope](../../openspec/changes/archive/2026-10-22-add-iphone-physical-control/proposal.md#proposal-acceptance)
follows the [sensor-only phone result](../../firmware/bench/2026-10-21-iphone-sensors/notes.md).
Its [phone/stand acceptance tasks](../../openspec/changes/archive/2026-10-22-add-iphone-physical-control/tasks.md#3-phone-and-physical-acceptance)
closed for the supervised scope on 2026-10-22; requirements are
[synced](../../openspec/specs/iphone-ble-teleoperation/spec.md). For a new setup,
check requested motion with motor power off first. Each new motor batch still
needs its own announced stand setup and fresh go.

1. Select **Physical** in the segmented control and allow motion access if prompted.
   That user action prepares sensors; no separate Enable button is needed. Choosing
   a mode or granting permission does not connect or ARM. Buttons remain the default
   after reload. Missing/denied readings expose **Retry motion access**.
2. Tap **Connect** and choose the car. If a permission dialog interrupts an existing
   connection, use **Retry motion access**, then connect manually again.
3. Hold the phone in portrait, screen up like a tray (within 30° of flat), and tap
   **SET NEUTRAL & ARM** while disarmed. That single action captures neutral and
   arms at zero. Each ARM captures the current neutral; there is no separate
   calibration tap. Connecting clears any earlier calibration.
4. At calibrated neutral (all three offsets within ±3°), tap **START TILT CONTROL**.
   Put both hands where comfortable; no touch needs to stay on the screen.
   Tip forward/back to drive; sideways to strafe; turn the whole phone left/right
   to rotate. Holding a twist sustains rotation; all three components can combine.
5. Return to neutral for zero while control keeps running, or tap **PAUSE TILT
   CONTROL** for zero with driving paused. Restart requires a fresh START at neutral.
   **STOP** disarms and requires fresh ARM then START. Mode changes also clear
   calibration and disarm; confirmed STOP may leave the connection open. Selecting Physical
   prepares sensors again; then use Set neutral & ARM and START.

Each axis uses a 3° dead zone and scales to full gentle reference at 20°:
up to 0.1 m/s per translation axis or 0.3 rad/s rotation, clipped by advertised
limits. These are open-loop references, not measured velocities. Neutral beta/gamma
must be within ±30°; current beta/gamma must stay within ±60° and heading offset
within ±90° to drive. Posture faults identify the offending angle/value/limit.
While paused, sensor gaps and reading posture block START at zero without
losing arm/neutral. Returning readings cannot replay a rejected START. Scroll
pauses Physical control. While running, sensor/posture loss or foreground-renewal
expiry ends the session. Invalid values or changed reference also close an armed
session while paused. No automatic recentering, reconnection or restart occurs.

Only fresh orientation callbacks renew the 250 ms sample deadline; only UI frames
renew the separate 250 ms running-input deadline. Cached angles and stream ticks
cannot keep either alive. The compact controller keeps requested motion and ARM/START/PAUSE with STOP
in a persistent thumb dock. A sticky status area keeps the session/fault reason
visible. Raw sensor angles, Status age, stream counters and limits are in collapsed
**Connection & diagnostics**; opening it pauses input. Lock/app switch/screen
rotation require manual motion setup/retry, connect, Set neutral & ARM and START. Existing
stream/Status/cleanup deadlines are unchanged. No compass heading, declared
motion interval or integrated gyro rate controls the car.

Earlier [motor-power-off phone results](../../firmware/bench/2026-10-21-iphone-physical/notes.md)
cover held-pad build `780f65eff594`: signs, neutral/release, STOP/mode changes,
corrected scrolling/calibration and a 60 s requested-zero hold on reported iPhone 11
(1,366 writes, max gap 180 ms, slowest write 19 ms). Those session counters do not
define an exact 60 s trace or radio-delivery timing. Todor subsequently found
holding the pad while tilting awkward and reported posture-triggered disarming.
Todor reported the redesigned controller and tap START/PAUSE procedure worked
in the motor-power-off Bluefy retest of build `2b5104400362` on 2026-10-22
([evidence](../../firmware/bench/2026-10-21-iphone-physical/notes.md#redesigned-controller-phone-retest--2026-10-22)).
Todor also confirmed the five-step motor-power-off PAUSE/restart, lock,
app-switch, posture cleanup/manual recovery and 60 s neutral procedure on that
build ([results and remaining limits](../../firmware/bench/2026-10-21-iphone-physical/notes.md#interruption-and-recovery-phone-check--2026-10-22)).
Current-build Stream counters and additional phone fault cases remain unrecorded;
the original rotation/drop-of-touch cause remains unverified. The subsequent
[stand record](../../firmware/bench/2026-10-22-iphone-physical-stand/notes.md) contains
reported direction/blend exercise and eight command periods returning to zero;
Todor separately confirmed STOP while tilted, re-ARM stillness until START and
mode-switch stopping. Those final checks are operator evidence; the capture does
not independently show them. Todor accepted the tested supervised scope and
explicitly deferred numeric phone cap/Stream records, native rotation/permission outcomes and phone fault injection.
Those measurements and broader qualification remain unverified, not passed checks.

## Phone test delivery

When requesting a new iPhone client test from Todor, also send the exact versioned
build URL and concise setup/test instructions to his Telegram destination so he
can open it on the phone. Confirm delivery; if the destination is unavailable or
ambiguous, resolve that before relying on Telegram delivery. Keep credentials and
private destination identifiers out of the repository. A test link does not
authorize a motor run; announce each motor batch and obtain a fresh go.

## Phone sensor test

Open <https://tptodorov.github.io/rovi/iphone/sensor-test.html> in Bluefy.
The car can stay off. This separate page under the client site imports no BLE,
session or command modules. It does not change the button controller.

Todor selected simultaneous three-axis input: forward/back tilt → forward/back,
side tilt → left/right strafe, and heading offset from neutral → car rotation.
Holding a twist keeps the rotation reference nonzero; returning to neutral zeros it.
The diagnostic previews percentages only, with 3°/20° dead-zone/full-scale
angles. The [reported phone run](../../firmware/bench/2026-10-21-iphone-sensors/notes.md)
confirmed directions, held heading and neutral/release behavior. It contains only
13.1 s of quiet evidence; raw full-minute drift remains unmeasured. Product
phone/stand acceptance is separate and limited by the review linked above.

1. Enter the actual phone, iOS and Bluefy versions. Tap **Enable sensors** and allow
   motion/orientation access. If a permission dialog interrupts collection, tap
   Enable again. Granted permission without finite readings does not pass.
2. Hold the phone in portrait, screen up like a tray, top edge away. Tap
   **Calibrate neutral**. Stay still for 60 s, then tap **Save stationary result**
   before tilting; this preserves neutral jitter and drift separately from movement.
3. Hold **Hold preview**. Tilt forward/back and sideways, then turn the whole phone
   left/right while keeping it screen up. Hold each angle for 10 s, returning to
   neutral between moves. Check the signs and a stable rotation preview at a held
   heading. Release must zero all three axes.
4. Combine all three axes. Record unwanted cross-axis responses, jumps near heading
   wrap, and whether holding the phone/control is comfortable. These Euler deltas
   are a candidate mapping, not a verified physical controller.
5. Tap a simulated-stall button, then hold the preview before its 2 s countdown ends.
   The 50 ms consumer keeps running: after at most 250 ms without sensor input or
   hold renewal, it must clear calibration and zero. Resume, recalibrate and repeat
   for the other stall. New samples alone must never restore the old hold.
6. Check app switch, lock/unlock and screen rotation. On return, collection must
   require Enable and calibration again. Repeat permission denial/retry and reload
   as needed; note any missing lifecycle event or continued inactive activity.
7. Tap **Finish & report**, enter observations, then **Copy report** and send it back.
   The report stays local until shared. Counts/max gaps include interruptions;
   median/p95 use up to 4,096 recent gaps. Only the last 120 raw events and 80
   diagnostic/lifecycle events and six stationary results are retained. Times measure callback arrival,
   not hardware acquisition or rover stopping.

Host checks cover wrap, all three signs, null fields, independent freshness,
late-sample recovery, permissions, lifecycle and BLE isolation. They cannot prove
Bluefy sensor support by themselves. The [feasibility note](../../docs/research/iphone-motion-control-feasibility.md)
records the scoped phone evidence and remaining gates. This isolated diagnostic
does not exercise the product Session or establish physical acceptance.

## Evidence limits

Linux browser and fake-clock tests prove software decisions only. They do not prove
Bluefy behaviour, iOS scheduling, radio delivery, MTU or physical stopping. Those are
the phone checks P1–P4 in the [verification map](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/verification.md).

If the page is suspended or killed, cleanup may never run. The firmware's lease and
claim windows remain the stop fallback. A resolved write or local disconnect does not
prove the wheels stopped.
