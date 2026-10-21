# iPhone BLE controller

Static HTTPS controller for Bluefy on iPhone: button control plus an isolated phone-sensor diagnostic. Physical driving is not implemented yet.
Implementation is authorized by the [proposal acceptance record](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/proposal.md#proposal-acceptance).
Implementation and scoped phone/stand acceptance completed on 2026-10-21:
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
  It deploys nothing.
* `./publish.sh` — builds a clean commit and pushes it to the `gh-pages` branch, served at
  <https://tptodorov.github.io/rovi/iphone/>. The page footer and `version.txt` show the
  revision. Rollback: `git push --force-with-lease origin <older-gh-pages-commit>:gh-pages`.

Open the URL in **Bluefy**, not Safari: Safari has no Web Bluetooth.

No test touches a BLE device. Runtime assets use no third-party scripts or command server.

## Layout

| File | Role |
| --- | --- |
| `wire.js` | 72-byte setpoint, capabilities and Status codecs |
| `hold.js` | Input interpreter: press-and-hold intent with 250 ms expiry |
| `transport.js` | Web Bluetooth picker, GATT validation and reads/writes |
| `session.js` | One owner of claim, zero+ARM, STOP latch, stream, deadlines and cleanup |
| `ui.js` | Pointer/lifecycle wiring and rendering |

Physical input will be another interpreter beside `hold.js`, selected by a
Buttons / Physical switch in this client. It feeds only intent into `session.js`;
it cannot arm or write GATT. The product change follows the sensor test below.

## Phone sensor test

Open <https://tptodorov.github.io/rovi/iphone/sensor-test.html> in Bluefy.
The car can stay off. This separate page under the client site imports no BLE,
session or command modules. It does not change the button controller.

Todor selected simultaneous three-axis input: forward/back tilt → forward/back,
side tilt → left/right strafe, and heading offset from neutral → car rotation.
Holding a twist keeps the rotation reference nonzero; returning to neutral zeros it.
The diagnostic previews percentages only, with provisional 3°/20° dead-zone/full-scale
angles. Signs, mixed-axis coupling and neutral posture still need phone evidence.

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
Bluefy sensor support. The [feasibility note](../../docs/research/iphone-motion-control-feasibility.md)
records the implementation gate. No sensor-to-car integration or physical acceptance
is claimed by this diagnostic.

## Evidence limits

Linux browser and fake-clock tests prove software decisions only. They do not prove
Bluefy behaviour, iOS scheduling, radio delivery, MTU or physical stopping. Those are
the phone checks P1–P4 in the [verification map](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/verification.md).

If the page is suspended or killed, cleanup may never run. The firmware's lease and
claim windows remain the stop fallback. A resolved write or local disconnect does not
prove the wheels stopped.
