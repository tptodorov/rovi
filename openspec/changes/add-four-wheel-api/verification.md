# M6 verification map

All checks below are **planned product checks**, not claims that the new implementation exists. Reuse the M8 unit tests and BLE/UDP/ROS harnesses where applicable, then record concrete product test names and evidence as tasks complete. Each row covers every scenario under that requirement. Task numbers refer to [tasks.md](tasks.md).

Host: deterministic tests/recording driver. Integration: real sockets or Jazzy on the laptop. Physical: separately authorized product firmware bench evidence; old M4/M5 evidence is a reference only.

## car-control-api

| Requirement | Completion check |
| --- | --- |
| [Setpoint](specs/car-control-api/spec.md#requirement-setpoint) | Host codec/device tests: positive and negative limit boundaries and invalid planar input; socket conformance (3.1, 3.3). |
| [Mecanum kinematics](specs/car-control-api/spec.md#requirement-mecanum-kinematics) | Host golden kinematics and combined-input saturation tests; transport parity (3.1, 3.3). |
| [Open-loop motion references](specs/car-control-api/spec.md#requirement-open-loop-motion-references) | Review product docs/status fields against M5 evidence; no command-derived measurement claims (3.5, 5.3). |
| [Latest setpoint wins](specs/car-control-api/spec.md#requirement-latest-setpoint-wins) | Host ordering tests and duplicate socket input; lease unchanged on stale data (3.2, 3.3). |
| [Single owner](specs/car-control-api/spec.md#requirement-single-owner) | Host claim tests and UDP/ROS intruders; physical AP/BLE exclusion on board (3.3, 5.2). |
| [Claim window](specs/car-control-api/spec.md#requirement-claim-window) | Injected-clock host tests, including refused ARM and no valid setpoint (3.1). |
| [Explicit arming](specs/car-control-api/spec.md#requirement-explicit-arming) | BLE/UDP scenarios plus real Jazzy auto-arm and STOP-latch scenarios (3.3). |
| [Lease and watchdog timeout](specs/car-control-api/spec.md#requirement-lease-and-watchdog-timeout) | Injected-time detection tests and physical log timing; detection only, no numeric disable bound (O4 deferred) (3.1, 5.2). |
| [Safety stop](specs/car-control-api/spec.md#requirement-safety-stop) | Host transition tests; every physical stop trigger with all four channels (3.3, 5.2). |
| [Reclaim window](specs/car-control-api/spec.md#requirement-reclaim-window) | Injected-clock tests for retained streaming owner, quiet owner, boundary and explicit/automatic recovery (3.1, 3.3). |
| [Boot state](specs/car-control-api/spec.md#requirement-boot-state) | Host initial state plus actual reset/power-loss observations on product firmware (3.3, 5.2). |
| [Capabilities](specs/car-control-api/spec.md#requirement-capabilities) | Golden BLE/UDP/ROS comparison and exact advertised-limit acceptance (3.4). |
| [ROS 2 capabilities topic](specs/car-control-api/spec.md#requirement-ros-2-capabilities-topic) | Real Jazzy late subscriber and no-claim checks; retained-history path if supported (3.4/O9). |
| [ROS 2 capabilities republish fallback](specs/car-control-api/spec.md#requirement-ros-2-capabilities-republish-fallback) | Real Jazzy volatile subscriber timing plus claim/release/reconnect; record selected mode (3.4/O9). |
| [Status](specs/car-control-api/spec.md#requirement-status) | Host snapshots and fake-BLE immediate read; validate actual BLE notification on board (3.5, 5.2). |
| [ROS 2 status topic](specs/car-control-api/spec.md#requirement-ros-2-status-topic) | Jazzy subscriber observes changes and heartbeat, reason/error/session fields (3.5). |
| [Last command error](specs/car-control-api/spec.md#requirement-last-command-error) | Cross-transport test sequence STOP, refused ARM, setpoint, successful ARM, non-owner input (3.5). |
| [Input validation before state changes](specs/car-control-api/spec.md#requirement-input-validation-before-state-changes) | Malformed corpus and non-owner/stale oversized samples; assert state, sequence and lease snapshots (3.1). |
| [Setpoint frame and time interpretation](specs/car-control-api/spec.md#requirement-setpoint-frame-and-time-interpretation) | Host CDR cases for empty/base_link/map, string bounds, nanoseconds and stale header stamp (3.1). |
| [Sequence domains](specs/car-control-api/spec.md#requirement-sequence-domains) | Host u32 wrap/half-range and full-i64 per-GID tests; same-node multiple-publisher ROS scenario (3.2). |
| [Session isolation and ROS identity](specs/car-control-api/spec.md#requirement-session-isolation-and-ros-identity) | Old UDP/BLE generation injection and real router reconnect; distinct GID same-node ARM (3.2). |
| [Motor validation fault integration](specs/car-control-api/spec.md#requirement-motor-validation-fault-integration) | Coordinator fault injection under continuing ROS stream; no drive until fresh setpoint and explicit ARM (2.3). |
| [Capabilities encoding](specs/car-control-api/spec.md#requirement-capabilities-encoding) | Exact key/unit golden tests and f32 round-trip values across three encodings (3.4). |
| [Status delivery and empty values](specs/car-control-api/spec.md#requirement-status-delivery-and-empty-values) | BLE readable/notify tests, UDP reply and stop-frame capture, ROS release snapshot (3.5). |
| [Valid operating profile](specs/car-control-api/spec.md#requirement-valid-operating-profile) | Constructor boundary tests for finite geometry, f32 limits and u16 time ranges; no enable on failure (3.1). |
| [Deadline ordering](specs/car-control-api/spec.md#requirement-deadline-ordering) | Injected-time tests at deadline-1, deadline and deadline+1 for each window (3.1). |
| [Echo is observational](specs/car-control-api/spec.md#requirement-echo-is-observational) | Existing UDP/fake-BLE echo patterns with state/lease/error snapshots and oversized payloads (3.3). |
| [New session allocation](specs/car-control-api/spec.md#requirement-new-session-allocation) | Injected boot seed, repeated HELLO, counter wrap/skip-zero and stale-session test (3.2). |
| [Bounded ROS ordering history](specs/car-control-api/spec.md#requirement-bounded-ros-ordering-history) | Fill/churn graph and ordering tables, remove/reannounce owner GID, reject replay and capacity overflow (3.2). |

## motor-control

Host test names are `rovi-motor-control` unit tests (`cargo test -p rovi-motor-control`).

| Requirement | Completion check |
| --- | --- |
| [Four independent wheel commands](specs/motor-control/spec.md#requirement-four-independent-wheel-commands) | Recording driver verifies one wheel at a time; physical direction on each wheel (2.1, 5.2). Host: `each_wheel_is_driven_on_its_own`, `sign_sets_direction_and_inversion_flips_it`. |
| [Independent of transport and kinematics](specs/motor-control/spec.md#requirement-independent-of-transport-and-kinematics) | Standalone no_std crate build/test and dependency inspection (2.2). Host: `cargo tree -p rovi-motor-control` shows no dependencies; `cargo build -p rovi-motor-control --target xtensa-esp32s3-none-elf -Zbuild-std=core` builds against `core` alone. |
| [Invalid wheel commands stop the wheels](specs/motor-control/spec.md#requirement-invalid-wheel-commands-stop-the-wheels) | Clamp both signs; inject NaN and both infinities at each wheel, inspect all outputs (2.2). Host: `every_non_finite_value_at_every_wheel_latches_standby`, `clamping_works_in_both_directions`, `the_product_profile_caps_drive_duty_at_70_percent`. |
| [Standby until explicitly enabled](specs/motor-control/spec.md#requirement-standby-until-explicitly-enabled) | Recording driver checks boot/disable; physical reset and power-loss STBY verification (2.1, 5.2). Host: `new_starts_in_standby`, `updates_need_an_explicit_enable`. |
| [Standby on safety stop](specs/motor-control/spec.md#requirement-standby-on-safety-stop) | Recording driver checks same-tick disable before any bridge update; physical: every wheel stops on each trigger. Numeric pin bound deferred (O4, needs a logic analyzer) (2.1, 2.3, 5.2). Coordinator (`cargo test -p rovi-drive`): `a_lease_expiry_puts_both_drivers_in_standby_in_the_same_tick`, `a_stop_and_rearm_between_ticks_still_drops_stby_first`, `a_release_puts_both_drivers_in_standby`. |
| [Full-scale maps to the configured limit](specs/motor-control/spec.md#requirement-full-scale-maps-to-the-configured-limit) | Host duty-cap tests including the 70% product profile; current is not measured (O1 deferred) (2.1, 5.1). Host: `the_product_profile_caps_drive_duty_at_70_percent`. |
| [Zero command behaviour](specs/motor-control/spec.md#requirement-zero-command-behaviour) | Failing coast-pin reproduction first, neighbor independence; coast distance not claimed (O5) (2.1, 5.2). Host: `zero_requests_coast_with_both_direction_pins_low_and_pwm_high`, `coast_keeps_pwm_high_under_a_70_percent_drive_cap`. |
| [Dead zone handling](specs/motor-control/spec.md#requirement-dead-zone-handling) | Exact zero, deadband-minus/at/plus, both signs and rounding table (2.1). Host: `deadband_and_duty_follow_the_rule`. |
| [Starts and reversals do not reset the controller](specs/motor-control/spec.md#requirement-starts-and-reversals-do-not-reset-the-controller) | Host signed transitions; product reversal/thermal/radio run at the 70% cap (2.1, 5.2). Host: `full_reversal_is_direct_with_the_drivers_kept_enabled`. |
| [Calibrated wheel mapping](specs/motor-control/spec.md#requirement-calibrated-wheel-mapping) | Host mapping/inversion tests; compare schematic and confirm all four physical channels (2.1, 5.1, 5.2). Host: `the_car_mapping_puts_each_wheel_on_its_bridge`, `sign_sets_direction_and_inversion_flips_it`. |
| [Configuration validation](specs/motor-control/spec.md#requirement-configuration-validation) | Rejected min/max/deadband/mapping cases and config replacement while disabled (2.2). Host: `invalid_duty_limits_cannot_enable`, `a_mapping_that_is_not_one_to_one_cannot_enable`, `an_invalid_deadband_is_reported_before_any_enable`, `boundary_configurations_are_valid`, `replacing_the_configuration_disables_first`, `an_invalid_replacement_latches_and_a_valid_one_clears_it`. |
| [Fault recovery requires explicit enable](specs/motor-control/spec.md#requirement-fault-recovery-requires-explicit-enable) | Host stateful fault/update/enable sequence; stale vector never replayed (2.2). Host: `finite_updates_do_not_clear_a_fault_and_enable_applies_only_the_new_vector`, `standby_works_in_every_fault_state`, `nan_while_driving_puts_both_drivers_in_standby`. Coordinator: `a_non_finite_wheel_demand_stops_the_device_and_no_tick_recovers_it`, `a_driver_failure_stops_and_only_a_fresh_setpoint_and_arm_recover`, `a_ros_stream_does_not_recover_a_motor_fault_but_an_explicit_arm_does`, `an_invalid_motor_configuration_never_enables`. |
| [Complete vector validation before output](specs/motor-control/spec.md#requirement-complete-vector-validation-before-output) | Recording driver verifies no partial writes and standby/prepare/enable ordering (2.1, 2.2). Host: `a_non_finite_fourth_wheel_cannot_drive_the_first_three`, `every_non_finite_value_at_every_wheel_latches_standby`, `enable_prepares_every_bridge_before_asserting_stby`, `standby_deasserts_both_stby_outputs_first`, `driver_failures_reach_the_caller`. |
| [Coast is independent of the drive duty cap](specs/motor-control/spec.md#requirement-coast-is-independent-of-the-drive-duty-cap) | 70%-cap neighbor test checks PWM-high coast with both direction pins low (2.1). Host: `coast_keeps_pwm_high_under_a_70_percent_drive_cap`. |
