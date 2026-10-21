# iPhone BLE drop diagnosis (D1)

2026-10-21, announced and run after Todor's go. iPhone 11 Pro, latest Bluefy,
controller build `a120de960836`, normal product firmware unchanged, motor power off,
USB serial capture ([car.log](car.log)). Opening the capture reset the board once.

Goal: explain the "Bluetooth connection lost" drops Todor saw at the start of P2.

## Result

**Not reproduced.** Five connect → ARM at zero → STOP → disconnect cycles, all clean:

| Cycle | link up (ms) | MTU 23→247 | Armed | Stopped/Stop | link down |
| --- | --- | --- | --- | --- | --- |
| 1 | 37170 | +600 | 47079 | 67361 | 68432 |
| 2 | 71758 | +501 | 77740 | 90517 | 91322 |
| 3 | 93821 | +501 | 96338 | 117067 | 118773 |
| 4 | 121097 | +600 | 123578 | 143197 | 145053 |
| 5 | 147884 | +500 | 149825 | 161167 | 161403 |

* One boot, no reset, panic, fault or dropped event. Five claim/release pairs.
* Wi-Fi AP went off 7–25 ms after each claim, before the MTU exchange; no link loss followed.
* Every release follows the phone's STOP and disconnect; the car never ended a link itself.
* All wheel commands stayed 0.

## Measured

* **iPhone ATT MTU: 247** after negotiation (~0.5 s after connect), every cycle. The 72-byte
  setpoint fits in one write. Resolves Q18 for this phone/Bluefy.

## Open

The earlier drops remain unexplained. Candidates still standing: radio conditions at the
time, the Wi-Fi AP switch racing link setup, iOS/Bluefy link management. Next time a drop
happens, capture serial in the same way before reconnecting.
