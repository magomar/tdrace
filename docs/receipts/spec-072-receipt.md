---
type: validation_receipt
schema_version: "1.0"
spec: "specs/072_progressive_drift_dynamics_lowspeed_steering_authority_and_assist_differentiation.md"
epic: "tdrace-epbk"
candidate_commit: "62f85dd6d033c9f4b5e2dc9bf65c72414d6904fb"
verifier: "local-user"
evaluated_at: "2026-10-02T07:02:38Z"
command: "TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test handling_calibration_tests --no-fail-fast && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test decoupled_tire_physics_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test differential_dynamics_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test handling_presets_tests -- --test-threads=1 && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test input_smoothing_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test keyboard_simulation_tests -- --test-threads=1 && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo check -p tdrace-app -p cabinet --all-targets"
exit_code: 0
duration_ms: 6188
status: passed
---

# 🧾 Validation Receipt: Spec 072

- **Candidate Commit**: `62f85dd6d033c9f4b5e2dc9bf65c72414d6904fb`
- **Spec**: `specs/072_progressive_drift_dynamics_lowspeed_steering_authority_and_assist_differentiation.md`
- **Command**: `TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test handling_calibration_tests --no-fail-fast && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test decoupled_tire_physics_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p wheelbase --test differential_dynamics_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test handling_presets_tests -- --test-threads=1 && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test input_smoothing_tests && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo test -p tdrace-app --test keyboard_simulation_tests -- --test-threads=1 && TDRACE_GIT_TRACKS_DIR=/home/mario/workspace/games/tdrace/tracks cargo check -p tdrace-app -p cabinet --all-targets`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 16 tests
test test_assist_intervention_telemetry_separates_sources_and_resets_each_step ... ok
test test_digital_pro_recovery_does_not_leak_to_analog ... ok
test test_low_speed_authority_is_higher_and_fades_to_stock_at_speed ... ok
test test_digital_flick_headroom_does_not_rearm_on_holds_or_feathering ... ok
test test_digital_flick_opens_low_speed_steering_headroom_once ... ok
test assist_telemetry_roundtrips_in_car_state_while_source_latch_stays_runtime_only ... ok
test test_corner_exit_with_throttle_held_keeps_drive ... ok
test test_lift_off_is_progressive ... ok
test test_traction_help_catches_power_oversteer ... ok
test test_roll_balance_flips_which_axle_saturates_first ... ok
test test_small_steer_does_not_weaken_brakes ... ok
test test_traction_help_does_not_ease_straight_line_drive ... ok
test test_grip_knob_moves_lateral_g ... ok
test test_low_speed_monotonic_curvature_and_braking_remain_calibrated ... ok
test test_steering_response_is_monotonic_at_every_speed ... ok
test test_random_input_fuzz_is_numerically_stable ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s


running 9 tests
test test_staggered_tire_dimensions_on_open_wheel_kart ... ok
test test_independent_front_wheel_brake_lockup_under_trail_braking ... ok
test test_legacy_configuration_backward_compatibility ... ok
test test_thermal_grip_degradation_under_prolonged_power_drifting ... ok
test test_kart_caster_jacking_inside_rear_wheel_unloading ... ok
test test_kart_cornering_under_throttle_preserves_drive_and_prevents_runaway_wheelspin ... ok
test test_kart_high_speed_tight_turning_radius_and_lateral_grip ... ok
test test_kart_low_speed_geometric_turning_circle ... ok
test test_headless_simulation_throughput_sla ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s


running 7 tests
test test_open_differential_equalizes_torque_and_limits_power_on_split_mu ... ok
test test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle ... ok
test test_spool_differential_locks_wheel_rotational_velocities ... ok
test test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor ... ok
test test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded ... ok
test test_lsd_power_lock_changes_corner_exit_yaw ... ok
test test_spool_produces_understeer_moment_versus_open ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 12 tests
test preset_feel::test_peak_yaw_at_speed_is_ordered_by_preset ... ok
test preset_feel::test_safe_presets_do_not_spin_on_held_key ... ok
test preset_feel::test_turn_in_time_is_ordered_by_preset ... ok
test test_catalog_random_input_fuzz_is_numerically_stable ... ok
test test_classic_drift_chassis_behavior_at_hairpin_speed ... ok
test test_classic_drift_chassis_calibration_is_applied ... ok
test test_custom_traction_preference_is_preserved_when_mode_changes ... ok
test test_input_config_round_trips_through_filter_config ... ok
test test_legacy_settings_file_loads_as_sharp ... ok
test test_mode_and_input_response_form_twelve_combinations ... ok
test test_settings_apply_to_live_player_car_both_directions ... ok
test test_split_screen_live_mode_change_keeps_player_handling_independent ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.19s


running 8 tests
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_non_linear_center_micro_corrections ... ok
test test_steering_profiles_configuration_and_cycling ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_steering_speed_setting_changes_time_to_full_input ... ok
test test_player_car_physics_receives_unattenuated_steering_when_direct_or_raw ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok
test test_steering_source_latch_prefers_analog_and_resets ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 9 tests
test test_assist_mode_matrix_is_independent_and_reports_twelve_pairs ... ok
test test_assist_mode_response_pair_keeps_help_and_torque_channels_distinct ... ok
test test_key_styles_do_not_spin_on_safe_presets ... ok
test test_keyboard_chicane_reversal_latency_measurement ... ok
test test_keyboard_filter_profiles_direct_vs_balanced_cornering ... ok
test test_keyboard_kart_caster_jacking_scrub_differential ... ok
test test_keyboard_slide_catch_recovery_on_dirt ... ok
test test_keyboard_sweeper_feathering_preserves_speed_vs_holding ... ok
test test_presets_change_key_style_spread ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/handling_calibration_tests.rs (target/debug/deps/handling_calibration_tests-2fdb89fbbc34a48e)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/decoupled_tire_physics_tests.rs (target/debug/deps/decoupled_tire_physics_tests-de34819b9da1eddb)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/differential_dynamics_tests.rs (target/debug/deps/differential_dynamics_tests-5aa6a01bf48f5b6a)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/handling_presets_tests.rs (target/debug/deps/handling_presets_tests-091e55e57f0df471)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-dd8254cba089bc74)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/keyboard_simulation_tests.rs (target/debug/deps/keyboard_simulation_tests-93ac0c34d29f32ac)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s

```
