# BLE keyboard verification

## Host evidence

`apps/teleop/tests/test_ble.py` uses a fake GATT peripheral and nonblocking input,
plus actual CLI subprocesses with pseudo-terminals. No Bluetooth device is used.

| Contract | Test |
| --- | --- |
| Startup stillness, six directions, expiry, STOP latch, quit | `test_startup_six_directions_expiry_stop_and_quit` |
| Acknowledged ARM/readback and Space priority | `test_space_during_arm_prevents_late_readback_enabling_motion`, `test_refused_arm_cannot_enable_motion` |
| Closed-peer callbacks/outage input cannot resume motion | `test_disconnect_reconnect_ignores_old_callbacks_and_outage_keys` |
| Echo required before ready; no startup ARM replay | `test_missing_echo_retries_and_stays_disarmed_until_fresh_echo` |
| Late Echo cannot erase an expired deadline | `test_late_echo_cannot_cancel_the_expired_reply_deadline` |
| One beep per outage, new beep after recovered loss | `test_repeated_failures_alert_once_then_new_loss_alerts_again` |
| MTU/capabilities/service failures refuse motion | `test_small_mtu_bad_caps_and_missing_service_fail_closed` |
| BlueZ's reported MTU=23 can use verified characteristic capacity | `test_bluez_write_capacity_can_establish_mtu_when_reported_view_is_23` |
| Valid sequential GATT replies do not cause false scheduler failure | `test_valid_control_roundtrips_do_not_look_like_a_scheduler_stall` |
| Malformed/unsolicited Status cannot arm | `test_malformed_or_unsolicited_status_never_arms` |
| Quit cancels discovery/connect and disconnects | `test_quit_during_discovery_cancels_and_returns`, `test_quit_during_connect_still_attempts_disconnect` |
| Failed/hung writes still allow bounded disconnect | `test_failed_or_stalled_cleanup_still_disconnects_within_budget` |
| Measured rover GATT/disconnect latencies still confirm STOP within 500 ms | `test_measured_rover_latencies_confirm_stop_within_budget` |
| Ambiguous discovery versus pinned MAC | `test_ambiguous_name_fails_and_mac_selects_exactly_one_device` |
| Retries cannot roam after name discovery | `test_name_discovery_pins_the_first_address_for_retries` |
| Optional dependency error explains Nix shell | `test_missing_bleak_has_actionable_error` |
| Actual CLI/PTY controls, signal cleanup and terminal restoration | `BleTerminalTests` |

The late-Echo test first failed (`LinkError not raised`) before the callback was
changed to retain expired probes; the same command now passes. The full client
suite also includes a test with three valid 110 ms GATT round trips that first
failed as a scheduler stall; loop checks now run between operations. The
suite also covers UDP protocol/runtime and the real product simulator. BLE
tests run both without Bleak installed and with the project's Bleak 2.1.1 in
`nix-shell`; optional BLE imports do not make UDP depend on Bleak.

Final host results: **59 client tests pass** (29.236 s), including all 20 BLE
tests; the same **20 BLE tests pass with Bleak 2.1.1** in Nix shell (9.581 s).
Strict active/baseline specs and all six generated-reader checks pass.

## Physical acceptance — passed 2026-10-22

Todor authorized the announced stand batch and confirmed **“yes, all good”**
([evidence](../../../../firmware/bench/2026-10-22-teleop-recovery-ble/notes.md)): ATT MTU 247, car AP off while connected and back on after
release, no motion before R, all six directions followed by zero on expiry,
Space/no-motion without R, a ~5 s laptop Bluetooth outage with no motion,
one beep, `Reconnected disarmed`, discarded outage input and a fresh R/direction.

The first quit failed: cleanup reported STOP and disconnect unconfirmed (exit 1).
A no-motion probe measured STOP write 75–124 ms, Status read 89–93 ms and BlueZ
disconnect 2.1–2.6 s against 80/80/250 ms deadlines. The new latency test failed
first; cleanup now shares one 500 ms budget and labels an unfinished disconnect
`requested (unconfirmed)`, which no longer fails the exit. The authorized quit
re-run acknowledged STOP, exited 0 in 0.57 s, and serial showed release, link
down and AP on ~2.3 s after STOP. Final host results: **60 client tests pass**,
including 21 BLE tests with Bleak 2.1.1.

Floor use, electrical/thermal measurements, physical stop timing and a bound on
BLE recovery time (~13 s observed) remain outside scope.
