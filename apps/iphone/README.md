# iPhone BLE controller

Static HTTPS controller for Bluefy on iPhone; button control first, sensors deferred.
Implementation is authorized by the [proposal acceptance record](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/proposal.md#proposal-acceptance).
Phone/radio/wheel acceptance is separate and remains pending.

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

A later accelerometer input would be another interpreter beside `hold.js`. It feeds
only intent into `session.js`; it cannot arm or write GATT. It needs its own proposal.

## Evidence limits

Linux browser and fake-clock tests prove software decisions only. They do not prove
Bluefy behaviour, iOS scheduling, radio delivery, MTU or physical stopping. Those are
the phone checks P1–P4 in the [verification map](../../openspec/changes/archive/2026-10-21-add-iphone-ble-client/verification.md).

If the page is suspended or killed, cleanup may never run. The firmware's lease and
claim windows remain the stop fallback. A resolved write or local disconnect does not
prove the wheels stopped.
