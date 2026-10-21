# iPhone STOP-while-holding recheck

2026-10-21, announced and run after Todor's go. iPhone 11 Pro, latest Bluefy, controller
build `002a888b467a` (STOP acts on touch-down), normal firmware unchanged, motor power off,
USB serial capture ([car.log](car.log)); opening it reset the board once.

Rechecks the P4 finding that STOP could not be pressed while a direction was held.
Todor reported it working.

| Check | Serial evidence (ms since boot) |
| --- | --- |
| Hold Forward, second finger on STOP | Forward `83,83,83,83` 34617; `stop reason=Stop` 35902; Stopped, wheels zero 35908; link stays up |
| Single STOP tap after re-ARM | ARM 50881; exactly one `stop reason=Stop` 55792 |
| Disconnect | Released, link down 65962 |

One boot; no reset, panic, fault or dropped event. Commands only; wheels were unpowered.
