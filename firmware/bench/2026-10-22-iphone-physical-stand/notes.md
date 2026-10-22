# 2026-10-22: iPhone Physical stand batch

Status: direction/blend exercise reported performed; final STOP/re-ARM/mode-switch checks need clarification. Capture stopped.

Todor authorized the ten-move, at-most-20-second batch with “go”. The
[procedure](../../../openspec/changes/add-iphone-physical-control/verification.md#next-stand-batch)
uses published UI build `2b5104400362` and the existing firmware without flashing.
Drive instructions were delivered via Telegram. The operator controls the phone;
no host ARM or nonzero setpoint was sent.

## Preflight

The USB serial monitor was initially suspended by its terminal/process-group
setup. Those attempts produced no firmware events; their output is retained in
`monitor-preflight.txt` and `monitor-no-stub-preflight.txt`. Relaunching without
a PTY and with `--no-stub` produced the boot log in `car.log`. Capture was bounded
to ten minutes by `timeout` and stopped early with SIGINT after the operator
reported completion; earlier attempts were stopped. No reset was sent at cleanup.

After sending the procedure, the agent briefly paused the batch because the
banner said “M8 board-only”. This was a false alarm: the product firmware still
contains that literal after constructing `MotorControl` with the product profile.
The seven boot image segment descriptors match the prior iPhone P3 capture;
this is a layout comparison, not a complete binary identity check. The operator
was informed and the same batch resumed, counting any already attempted moves
toward the original limit. No additional motion or flash was authorized.

## Operator report

Todor replied “done. all moves performewd” to the result question for the ten-move
procedure. Record the direction/blend exercise as reported performed, with no
mismatch or fault reported. The response did not separately identify each move,
STOP/re-ARM, mode-switch stopping, startup stillness or final motor-power state.
Clarification of the final two checks was requested because they do not appear
in the capture. No additional drive is requested or authorized by this record.

No Stream write count, maximum gap or slowest write was supplied. Approximate
requested velocities, physical stopping times and measured speed are not proved.

## Serial evidence

`car.log` contains one boot, one BLE claim, MTU 247, 343 state events, one explicit
STOP, release and BLE disconnect. The final state is Open with all four wheel
commands zero; Wi-Fi returns on. No `ev=error`, `ev=fault`, `ev=dropped` or panic
appears. The ROM boot warning about multiple DROM segments is present as in the
previous iPhone P3 boot; this is not an assertion that the entire log is error-free.

| Event | Device at_ms |
| --- | ---: |
| BLE claim | 198525 |
| Armed at zero | 208286 |
| Last nonzero command returns to zero | 315328 |
| Explicit STOP | 327139 |
| Stopped at zero | 327144 |
| Release / BLE down | 334669 |
| Open at zero | 334684 |

The eight nonzero command periods below are reconstructed by holding each logged
wheel vector until the next state event; they are command durations, not measured
wheel-motion durations. All periods end at an all-zero command.

| Period | Start at_ms | Zero at_ms | Command duration ms |
| --- | ---: | ---: | ---: |
| 1 | 220013 | 221276 | 1263 |
| 2 | 239872 | 241492 | 1620 |
| 3 | 252981 | 255053 | 2072 |
| 4 | 263394 | 264862 | 1468 |
| 5 | 274288 | 277045 | 2757 |
| 6 | 281516 | 283528 | 2012 |
| 7 | 293363 | 298644 | 5281 |
| 8 | 307196 | 315328 | 8132 |

Total held nonzero-command time is 24,605 ms. Several periods exceed the announced
2 s target and the total exceeds 20 s; compliance with those batch bounds is not
claimed. The log has no re-ARM after STOP and no later nonzero period. It therefore
does not establish the intended STOP-while-moving/re-ARM or mode-switch-while-moving
checks, and the final STOP occurred after commands had already returned to zero.
Raw requested twists, phone mode and PAUSE actions are not logged, so the capture
cannot independently map periods to all named moves or prove client per-axis caps.
Task 3.2 remains partial pending clarification and remaining evidence.

