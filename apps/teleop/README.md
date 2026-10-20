# Laptop UDP teleoperation

Linux, Python 3.10+ and the normal M6 firmware. No installation or extra Python
package is needed. From the repository root, after manually joining the car AP:

```sh
python3 -m apps.teleop --target 192.168.4.1:7777
```

Use this only for a separately authorized, supervised motor session. The client
requires a real terminal and an explicit IPv4 target/port. It does not change
Wi-Fi, firewall rules or firmware. Do not use the opt-in `motor-bench` image.

| Key | Action |
| --- | --- |
| R | Clear motion, send zero, then request ARM; wait for `local=armed` |
| W / S | Forward / backward, at most 0.1 m/s requested |
| A / D | Strafe left / right, at most 0.1 m/s requested |
| Q / E | Rotate left / right, at most 0.3 rad/s requested |
| Space | Clear motion and request STOP; retain the session, disarmed |
| Esc / Ctrl-C / Ctrl-D | Quit through cleanup |

Connection starts disarmed. Pressing directions cannot arm it. Each direction
replaces the previous one and expires after **250 ms without another direction
key**. Keyboard repeat renews it; terminal input has no physical key-up event,
so there may be an initial pause before the keyboard starts repeating. Unrelated
keys do not renew motion. An expired direction sends zero, while the local arm
gate stays enabled. Space or a device stop requires R and then a new direction.
Speeds are clamped to the advertised axis limits; zero-limited axes stay zero.

The client streams every 50 ms and checks Echo reachability every 250 ms.
A 500 ms reply timeout or socket failure clears motion, attempts zero/STOP/BYE,
then retries the connection after a one-second pause. It keeps trying through
an unavailable or BUSY car until you quit. The old ownership window may need
to expire before a new claim succeeds. It does not change Wi-Fi association;
the laptop must still be joined to the car AP. A send-loop gap of 200 ms,
terminal failure, invalid capabilities or a lease below 200 ms still ends the run.
These are host policies, not measured wheel-stop bounds.

After a fresh claim and matching Echo, the client reports `Reconnected disarmed`.
R and directions typed during reconnection are discarded. Press **R again**,
then a new direction; previous motion never resumes automatically. Esc/Ctrl-C/
Ctrl-D also work while retrying. Startup timeout/BUSY retries never arm.

A brief laptop beep announces each connection drop. Repeated failed attempts
stay quiet until a claim and Echo confirm recovery. Direct playback uses the
installed PipeWire `pw-play` utility; no Python package is needed. Keep laptop
audio unmuted. Missing/failed playback prints `Audio alert unavailable` and
attempts the terminal bell, whose audibility depends on terminal configuration.

Output separates the local arm gate and requested vx/vy/wz from **last-reported**
device Status and its age. Echo age is separate: a responsive rebooted car can
reject the old session, so Echo proves neither ownership nor arming. There is no
claim based solely on Echo; recovery negotiates a fresh session. The car has no
encoders; the
[M5 motion references](../../experiments/rovi-m5-mecanum-motion/MILESTONE.md) are
approximate, and requested speeds are not measured feedback.

Cleanup attempts zero, STOP and BYE within a total 500 ms without waiting
indefinitely for a reply; individual send failures do not skip later attempts.
`STOP acknowledged` means a matching device reply, not a verified physical stop.
BYE/release is always labelled unconfirmed. Missing acknowledgement or cleanup
errors give a nonzero exit. Terminal settings are restored on ordinary exit and
handled errors/signals; the device watchdog remains independent of the client.

## Laptop validation

```sh
cargo build --locked --manifest-path apps/simcar/Cargo.toml
python3 -m unittest discover -s apps/teleop/tests -t . -v
```

Tests use loopback UDP peers, pseudo-terminals and the product simulator. They do
not contact the rover. The separately authorized [stand batch](../../firmware/bench/2026-10-20-laptop-teleop/notes.md)
passed on 2026-10-20 with Todor’s physical confirmation: six directions,
input-expiry/Space/quit stops, explicit re-arm and startup/unarmed stillness.
This accepts the original client on the stand; loaded/floor use and electrical
or physical timing measurements remain outside this evidence. Automatic
recovery and audio are the [follow-up](../../openspec/changes/add-teleop-reconnect/proposal.md):
host tests include dropped Echo/ARM replies, initial unavailable/BUSY targets,
stale replies, simulator restart, buffered keys, retry quit and audio failures.
Its separate board/audio acceptance is pending; it does not establish a Wi-Fi fix.

The [accepted requirements](../../openspec/specs/laptop-teleoperation/spec.md)
and [archived verification map](../../openspec/changes/archive/2026-10-20-add-laptop-teleoperation/verification.md)
record the host tests and scoped physical acceptance.
