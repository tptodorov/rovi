# iPhone P4: supervised wheels-off-ground run

2026-10-21, announced and run after Todor's go. iPhone 11 Pro, latest Bluefy,
controller build `a120de960836`, normal product firmware unchanged, car on its stand,
motor power on, USB serial capture ([car.log](car.log)); opening it reset the board once.

Todor's observation: "worked fine, correct" — all directions matched their labels and
the wheels stopped.

| Step | Serial evidence (ms since boot) |
| --- | --- |
| 1. Not armed | Claimed 4656, no wheel command until ARM at 28048 |
| 2. Six directions | `83,83,83,83` 31826→32314, `32,-32,-32,32` 40501→41102, `83,-83,83,-83` 41884→42364, `-83,-83,-83,-83` 43828→44642, `-83,83,-83,83` 45507→46291, `-32,32,32,-32` 47073→48091; each followed by zeros |
| 3. STOP while moving | **Not in serial.** No STOP with the link kept up was recorded |
| 4. Home swipe | Page cleanup STOP 61586, Stopped/Stop, link down 61705. Wheels were already zero since 48091 |
| 5. Bluetooth off | Second connection: rotation holds to 82683, Forward 83192→84484 zero, firmware link-loss stop `OwnerLost` 88496 |

* One boot; no reset, panic, fault or dropped event. Two claims, each released.

## Limits

* Steps 3–5 were not captured **while the wheels were commanded nonzero**. In step 5 the
  zero at 84484 came 4.0 s before link loss: most likely iOS cancelled the touch when
  Control Center opened, which itself stops motion. The manual STOP-while-moving and the
  STOP→re-ARM latch on the car are therefore user-observed only (P3 and Linux tests cover
  the page side).

## Acceptance

Todor accepted P4 as passed (2026-10-21). Known UI limits, accepted: STOP cannot be
pressed while a direction is held, and Bluetooth cannot be switched off while holding a
direction. In both cases releasing the direction already returns the command to zero
(step 2), so the stop paths were exercised at zero.
