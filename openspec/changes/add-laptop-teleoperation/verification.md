# Laptop teleoperation verification

**Host implementation verified, 2026-10-20; physical application acceptance pending.**
Todor selected laptop teleoperation after #89 presented the scope. The client,
shared codecs and tests now exist. Tasks 5.1/5.2 remain unchecked: no board run
or physical result is inferred from the laptop checks.

Run from the repository root:

```sh
python3 -m unittest discover -s apps/teleop/tests -t . -v
```

31 client tests cover protocol/policy assertions, real pseudo-terminals and
fake peers, and the real client against the product simulator. Existing codec
compatibility passed 23 Python tests, 40 bench tests and 14 UDP scenarios.
These are host results, not electrical or radio qualification.

| Requirement | Actual host evidence | Task |
| --- | --- | --- |
| [Explicit connection without motion](specs/laptop-teleoperation/spec.md#requirement-explicit-connection-without-motion) | `ProtocolTests.test_handshake_is_single_and_validates_caps`, `test_invalid_floats_and_window_bounds_are_refused`, `test_busy_and_zero_session_fail`, `test_invalid_frames_do_not_refresh_deadline`; `TerminalTests.test_non_tty_and_invalid_target_never_connect` checks no extra claim. The startup part of `test_real_cli_start_arm_expiry_space_rearm_and_restore` observes zero/no ARM before R. | 2.2, 4.1 |
| [Explicit arm and stop latch](specs/laptop-teleoperation/spec.md#requirement-explicit-arm-and-stop-latch) | `PolicyTests.test_no_implicit_arm_and_new_direction_after_arm`, `test_reported_stop_and_errors_cancel_motion`; real CLI Space latch/re-arm in `TerminalTests.test_real_cli_start_arm_expiry_space_rearm_and_restore`. | 3.1, 3.2, 4.1 |
| [Gentle planar directions](specs/laptop-teleoperation/spec.md#requirement-gentle-planar-directions) | `PolicyTests.test_all_six_signs_and_latest_only`, `test_zero_and_smaller_limits`; `SimulatorTests.test_six_directions_expiry_stop_latch_and_cleanup` checks all six actual simulator wheel-sign patterns. | 3.1, 4.1 |
| [Input expiry and latest intent](specs/laptop-teleoperation/spec.md#requirement-input-expiry-and-latest-intent) | `PolicyTests.test_expiry_boundary_repeat_and_unrelated_keys`, `test_stop_and_quit_dominate_batch_and_clear_arm`; simulator integration checks zero after every input expiry. | 3.1, 3.2 |
| [Periodic stream and scheduling failure](specs/laptop-teleoperation/spec.md#requirement-periodic-stream-and-scheduling-failure) | `PolicyTests.test_loop_gap_boundary_fails_without_catchup`, `ProtocolTests.test_sequence_wrap_and_control_timeout`; `TerminalTests.test_stalled_process_discards_buffered_motion_on_resume` suspends the actual process, queues W and verifies no motion replay. The client schedules 50 ms ticks; simulator expiry and session survival exercise the real stream. | 3.1, 3.2 |
| [Validate and correlate device input](specs/laptop-teleoperation/spec.md#requirement-validate-and-correlate-device-input) | `ProtocolTests.test_control_correlation_stop_cancels_pending_arm`, `test_bad_status_cannot_enable`; `RuntimeTests.test_wrong_endpoint_cannot_arm` and `TerminalTests.test_stop_before_arm_reply_needs_new_operator_arm`. | 2.2, 3.2 |
| [Link checks without automatic recovery](specs/laptop-teleoperation/spec.md#requirement-link-checks-without-automatic-recovery) | `ProtocolTests.test_echo_nonce_session_and_age_are_independent`; `TerminalTests.test_lost_echo_exits_and_restores_without_reclaim`, `test_missing_arm_reply_fails_without_nonzero`; `SimulatorTests.test_killed_simulator_ends_client_without_reconnect` kills an armed/driving simulator and observes client failure. | 3.2, 4.1 |
| [Bounded shutdown and honest reporting](specs/laptop-teleoperation/spec.md#requirement-bounded-shutdown-and-honest-reporting) | `RuntimeTests.test_cleanup_attempts_every_send_despite_failures`, `test_cleanup_only_accepts_matching_stop_ack`, `test_unconfirmed_handshake_still_attempts_session_zero_cleanup`; PTY tests verify normal/Ctrl-C/SIGTERM restoration and terminal-hangup STOP/BYE. The session-zero cleanup test failed before the early-return fix. | 3.2, 4.1 |
| [Observable commands and evidence limits](specs/laptop-teleoperation/spec.md#requirement-observable-commands-and-evidence-limits) | `RuntimeTests.test_echo_does_not_refresh_status_age`; output and the [usage guide](../../../apps/teleop/README.md) distinguish local versus last-reported state, approximate requested speed and unconfirmed BYE. | 3.3; physical 5.1/5.2 pending |

Test sources: [protocol](../../../apps/teleop/tests/test_protocol.py),
[policy](../../../apps/teleop/tests/test_policy.py),
[runtime](../../../apps/teleop/tests/test_runtime.py),
[terminal](../../../apps/teleop/tests/test_terminal.py),
[simulator](../../../apps/teleop/tests/test_simulator.py).

## Physical acceptance still pending

The [announced stand procedure](../../../firmware/bench/LAPTOP-TELEOP.md) is
prepared, not authorized or run. It checks startup stillness, actual directions,
expiry/Space/final stops and explicit re-arm on the installed normal firmware.
A fresh go and Todor’s physical observations are required. M6’s [measurement
deferrals](../archive/2026-10-20-add-four-wheel-api/verification.md#task-52-disposition)
remain unchanged; no electrical coast/STBY, current, temperature margin or
loaded/floor result is claimed.
