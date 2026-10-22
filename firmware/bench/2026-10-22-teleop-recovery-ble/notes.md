# 2026-10-22: laptop reconnect and BLE keyboard stand acceptance

Todor authorized the announced batches A (UDP reconnect) and B (BLE keyboard)
with “go” and confirmed afterwards: **“yes, all good”**. This covers wheels moving
only in announced runs, stillness through both link drops, all four wheels
following all six BLE directions, and one loss beep per drop. A short BLE quit
re-run after the cleanup fix had its own “go”.

Rover on stand, wheels clear, motor bank awake. No flash: the already-flashed
normal image (`ver=0.1.0`). The actual `apps.teleop` process received terminal
bytes through a pseudo-terminal ([driver](driver.py.txt), [controller](controller.py.txt)).
Opening capture caused one reset. Wheel order FL, FR, RR, RL; timestamps are
firmware `at_ms`, not measured physical stopping times.

## A: UDP reconnect ([client](client-udp.txt), [result](result-udp.json))

| Stage | Serial/client evidence | Physical |
| --- | --- | --- |
| Startup, unarmed W | Claim 6,668 ms, zero; no nonzero before R | Still |
| R, forward 2 s | Armed zero 10,687; `83×4` 14,990; zero at expiry 17,204 | Correct, stopped |
| Laptop Wi-Fi off ~5 s, R/W sent during outage | LeaseExpired 19,066; released 24,068; no nonzero | Still, one beep |
| Recovery | Fresh claim 24,377; client `Reconnected disarmed`; 1 s W only: no nonzero | Still |
| R, fresh W, Space, quit | Armed 25,505; forward 26,775; Stop 27,758; released 27,804; exit 0 | Correct, stopped |

Audio played through `pw-play` (no bell fallback).

## B: BLE keyboard ([client](client-ble.txt), [result](result-ble.json))

| Stage | Serial/client evidence | Physical |
| --- | --- | --- |
| Connect | `ble link=up`, `wifi ap=off`, ATT MTU **247** | Still |
| Unarmed W, R | No nonzero; Armed zero 45,989 | Still |
| Six directions, 2 s each, 1 s expiry | `83×4`, `-83×4`, `-83,83,-83,83`, `83,-83,83,-83`, `-32,32,32,-32`, `32,-32,-32,32`, each followed by zero | All correct, stopped |
| Space, W without R, R + W | Stop 69,336; no nonzero; Armed 70,509; forward 71,728 | Correct |
| Laptop Bluetooth off ~5 s, R/W sent during outage | OwnerLost 73,790; released, `ap=on`; no nonzero | Still, one beep |
| Recovery | Reclaim 91,826 (~13 s, several retries); `Reconnected disarmed`; 1 s W only: no nonzero | Still |
| R, fresh W, Space | Armed 93,848; forward 95,066; Stop 96,049 | Correct, stopped |
| Quit | **Fail:** `Cleanup: STOP unconfirmed; BLE disconnect unconfirmed`, exit 1; no `link=down` within ~1 s | Already stopped |

### Cleanup defect and fix

A no-motion [timing probe](timing/ble_timing.py.txt) (three connects, never ARM;
[result](timing/result.json), [serial](timing/car.log)) measured STOP write with
response 75–124 ms, Status read 89–93 ms and BlueZ disconnect 2.1–2.6 s. The
client's 80/80/250 ms deadlines could not succeed. The spec requires
*attempting* cleanup within 500 ms and reporting disconnect separately, so the
[fix](quit-rerun/client-fix.diff) shares one budget (STOP ≤170 ms, read
≤150 ms, disconnect gets the rest) and labels an unfinished disconnect
`requested (unconfirmed)` without failing the exit. A host test with the
measured latencies failed before the fix and passes after.

[Quit re-run](quit-rerun/ble_quit.py.txt) (R, 1 s forward, Space, Ctrl-D):
`Cleanup: STOP acknowledged; BLE disconnect requested (unconfirmed)`, exit 0,
quit 0.57 s. Serial ([events](quit-rerun/car-events.log)): forward 313,577,
Stop 314,565, `released` / `ble link=down` 316,858, `wifi ap=on` 316,885:
BlueZ completed the disconnect ~2.3 s after the STOP. The stale probe monitor
held the port, so this run had no reset and its events came from
`timing/car.log`.

[Cleanup](cleanup.json): laptop Wi-Fi UUID and route restored, Bluetooth on.
`carlog.check` reports no fault, drop or panic.

## Scope

Accepts reconnect/audio and BLE keyboard control on the stand with the normal
firmware. No floor/loaded result, measured speed, current, physical stopping
bound or Wi-Fi #23 conclusion is claimed. BLE recovery took ~13 s after the
adapter returned; it is not bounded by this evidence.
