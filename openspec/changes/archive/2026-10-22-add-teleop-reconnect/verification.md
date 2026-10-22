# Reconnect verification

## Reproduction and host evidence

Before production edits, running
`python3 -m unittest apps.teleop.tests.test_terminal.TerminalTests.test_lost_echo_reconnects_disarmed_without_replaying_motion -v`
failed with `teleop: Echo reply timeout` and client exit. The same test now passes.

| Contract | Evidence |
| --- | --- |
| Recover after Echo/control loss, no motion replay | `test_lost_echo_reconnects_disarmed_without_replaying_motion`, `test_missing_arm_reply_reconnects_without_nonzero` |
| Retry unavailable/BUSY startup without arming | `test_unavailable_then_busy_target_recovers_without_arming` |
| One-second pacing, one alert per outage and reset after matched Echo | `test_one_alert_per_outage_until_echo_confirms_recovery` |
| Fresh correlation; old ACK/Status rejected | `test_new_attempt_rejects_old_handshake_and_old_session_status` |
| Claim without Echo cannot enable controls | `test_repeated_hello_success_without_echo_cannot_enable_motion` |
| Real product simulator restart and explicit new R/direction | `test_restarted_simulator_reconnects_disarmed_until_new_arm_and_direction` |
| Socket errors recover, failed cleanup still attempts every send | `test_socket_failures_are_recoverable_link_errors`, existing cleanup tests |
| Quit during outage restores terminal without another claim | `test_quit_during_retry_wait_does_not_start_another_attempt` |
| Audio is bounded; unavailable/failed/hung player returns fallback | `AudioTests`; outage test checks BEL warning |
| Fatal scheduling/terminal paths preserved | Existing stalled-process, hangup and termination-signal tests |

The full client suite has 39 tests including the real simulator and PTYs.
The local `beep()` invocation completed successfully through PipeWire on
2026-10-20; this proves playback success, not that Todor heard it.

## Physical acceptance — passed 2026-10-22

Todor authorized the announced stand batch and confirmed **“yes, all good”**
([evidence](../../../../firmware/bench/2026-10-22-teleop-recovery-ble/notes.md)). The laptop Wi-Fi was dropped for ~5 s during an armed
session: the car stopped on lease expiry and released, R/W typed during the
outage were discarded, the client reported `Reconnected disarmed`, one second
of W produced no motion, and only a fresh R plus W moved the wheels. One beep
played; Space and quit stopped and released the car (exit 0); laptop Wi-Fi and
route were restored.

No firmware change or flash was needed. Wi-Fi issue #23, floor/loaded operation,
electrical measurements and physical stopping bounds remain outside this change.
