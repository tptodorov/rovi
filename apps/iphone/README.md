# iPhone BLE controller

Static HTTPS controller for Bluefy on iPhone, with Buttons and Physical modes plus
an isolated phone-sensor diagnostic. Physical mode is a host-tested candidate;
combined phone/BLE and stand acceptance remain pending.
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

## Physical mode candidate

The [accepted implementation scope](../../openspec/changes/add-iphone-physical-control/proposal.md#proposal-acceptance)
follows the [sensor-only phone result](../../firmware/bench/2026-10-21-iphone-sensors/notes.md).
Its product [phone/stand acceptance tasks](../../openspec/changes/add-iphone-physical-control/tasks.md#3-phone-and-physical-acceptance)
remain open. The first combined BLE check should use motor power off; a motor
batch still needs its own announced stand setup and fresh go.

1. Select **Physical**, tap **Enable sensors** and allow access. Choosing a mode or
   granting permission does not connect or ARM. Buttons remain the default after reload.
2. Choose and connect the car. If a permission dialog interrupts an existing
   connection, enable sensors first and then connect manually again.
3. Hold the phone in portrait, screen up like a tray (within 30° of flat), and tap
   **Calibrate neutral** while disarmed. Connecting clears any earlier calibration.
4. Tap **ARM AT ZERO**, then use a **new** hold on **Hold to drive with phone**.
   Tip forward/back to drive; sideways to strafe; turn the whole phone left/right
   to rotate. Holding a twist sustains rotation; all three components can combine.
5. Return to neutral or release to zero. **STOP** disarms and requires fresh ARM
   and a new hold. Changing modes also clears calibration and disarms; an open
   connection may remain after confirmed STOP. Sensors must be enabled/calibrated
   again when selecting Physical.

Each axis uses a 3° dead zone and scales to full gentle reference at 20°:
up to 0.1 m/s per translation axis or 0.3 rad/s rotation, further clipped by the
car's advertised limits. These are open-loop references, not measured velocities.
Neutral beta/gamma must be within ±30°; current beta/gamma must remain within
±60° and heading offset within ±90°. Leaving this posture, changed sensor
reference, invalid/stale samples or hold expiry closes an armed Physical session.
No automatic recentering, reconnection or re-arming occurs.

Only fresh orientation callbacks renew the 250 ms sample deadline; only UI frames
renew the separate 250 ms hold deadline. Cached angles and transmitter ticks
cannot keep either alive. Lock/app switch/screen rotation require manual Enable,
connection, calibration, ARM and a new press. Existing stream/Status/cleanup
deadlines remain unchanged. No compass heading, declared motion interval or
integrated gyro rate is used for control.

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
13.1 s of quiet evidence; full-minute drift and product BLE/stand checks remain open.

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
