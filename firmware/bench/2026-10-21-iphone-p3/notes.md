# iPhone P3: touch and lifecycle checks

2026-10-21, announced and run after Todor's go. iPhone 11 Pro, latest Bluefy,
controller build `a120de960836`, normal product firmware unchanged, motor power off,
USB serial capture ([car.log](car.log)); opening it reset the board once. Wheel values
are commanded duty in permille (`wheels_pm`), not observed motion.

Todor ran steps 1–7 and reported them complete; step 8 (VoiceOver) was not run.

| Step | Serial evidence (ms since boot) |
| --- | --- |
| 1. Six directions | Six distinct patterns, each followed by zeros: `83,83,83,83` 30322→32094, `32,-32,-32,32` 33436→34729, `83,-83,83,-83` 36830→38450, `-83,-83,-83,-83` 39376→41209, `-83,83,-83,83` 42112→44150, `-32,32,32,-32` 45379→47211 |
| 2. Slide off | Forward 54828 → zero 55878 |
| 3. Second finger | Forward 68238 → zero 68962 |
| 4. Home swipe while holding | Forward 80814 → zero 85219; page cleanup STOP 87975, Stopped/Stop, link down 88094 |
| 5. Lock while armed | Page cleanup STOP 104144, Stopped/Stop, link down 104294 |
| 6. Bluetooth off while armed | No STOP reached the car; firmware detected link loss: `stop reason=OwnerLost`, link down 120735 |
| 7. Reload while armed | Page cleanup STOP 137326, Stopped/Stop, link down 137385 |

* One boot; no reset, panic, fault or dropped event. Four claims, each with its release.
* No nonzero command after any lifecycle event: the backgrounded page never renewed motion.
* The page's best-effort STOP reached the car on home swipe, lock and reload. With
  Bluetooth off, the firmware's own link-loss stop covered it, as designed.
* No automatic reconnect: each new claim came about 7–11 s after the previous release (manual).

## Limits

* Serial has no phone-side timestamps, so release-to-zero latency and the delay from
  home swipe/lock/Bluetooth-off to the stop are not measured. In step 4 the zero came
  2.8 s before STOP; whether the finger lifted first or iOS cancelled the touch is unknown.
* VoiceOver/accessibility not checked on the phone.
* Wheels were unpowered: this proves commands, not physical motion.
