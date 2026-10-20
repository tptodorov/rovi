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

The client streams every 50 ms, checks Echo reachability every 250 ms and aborts
on a 500 ms reply deadline or a send-loop gap of 200 ms. These are host policies,
not measured wheel-stop bounds. A lease below 200 ms is unsupported. A failed run
ends rather than reconnecting/re-arming. Restarting is an operator action and
may initially receive BUSY until the old ownership window ends.

Output separates the local arm gate and requested vx/vy/wz from **last-reported**
device Status and its age. Echo age is separate: a responsive rebooted car can
reject the old session, so Echo proves neither ownership nor arming. There is no
automatic HELLO after the initial claim. The car has no encoders; the
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
This accepts the application on the stand; loaded/floor use and electrical or
physical timing measurements remain outside this evidence.
