---
type: validation_receipt
schema_version: "1.0"
spec: "specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md"
epic: "tdrace-zwja"
candidate_commit: "72b70f8cfd98ce1218a5a5c98cf42b34918cffc9"
verifier: "local-user"
evaluated_at: "2026-09-28T07:56:41Z"
command: "cargo test -p wheelbase && cargo test -p tdrace-app --test handling_presets_tests --test keyboard_simulation_tests --test input_smoothing_tests --test config_tests && cargo test -p cabinet --test cabinet_integration_tests"
exit_code: 0
duration_ms: 5970
status: passed
---

# 🧾 Validation Receipt: Spec 043

- **Candidate Commit**: `72b70f8cfd98ce1218a5a5c98cf42b34918cffc9`
- **Spec**: `specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md`
- **Command**: `cargo test -p wheelbase && cargo test -p tdrace-app --test handling_presets_tests --test keyboard_simulation_tests --test input_smoothing_tests --test config_tests && cargo test -p cabinet --test cabinet_integration_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 56 tests
test bike::tests::test_motorbike_initialization ... ok
test bike::tests::test_motorbike_lean_and_camber_cornering ... ok
test bike::tests::test_motorbike_acceleration_and_wheelie ... ok
test car::tests::test_ackermann_angles ... ok
test car::tests::test_car_initialization ... ok
test car::tests::test_crest_unloading ... ok
test car::tests::test_reverse_drive_bias_distribution ... ok
test config::tests::test_config_presets ... ok
test config::tests::test_finalize_derives_wheels_from_axle_settings ... ok
test config::tests::test_pacejka_peak_slip_angle_migration ... ok
test config::tests::test_staggered_kart_wheel_assemblies ... ok
test sim::playground::tests::test_composite_gauntlet_structure ... ok
test car::tests::test_stopped_car_on_superelevated_segment_remains_static ... ok
test config::tests::test_legacy_config_deserialization_backward_compatibility ... ok
test car::tests::test_straight_line_step ... ok
test sim::playground::tests::test_parametric_turn_generators ... ok
test car::tests::test_step_with_sampler ... ok
test car::tests::test_state_save_restore ... ok
test car::tests::test_reverse_heading_stability_and_steering_symmetry ... ok
test car::tests::test_grade_slope_resistance ... ok
test sim::playground::tests::test_protocol_g_wall_contact_hierarchy_and_anti_wall_riding ... ok
test sim::tests::test_braking_in_turn_simulation ... ok
test sim::tests::test_headless_harness_straight_line ... ok
test car::tests::test_is_braking_state_off_throttle_vs_braking ... ok
test car::tests::test_reverse_handbrake ... ok
test car::tests::test_reverse_straight_line_neutral_steer ... ok
test sim::tests::test_braking_split_mu_simulation ... ok
test surface::tests::test_surface_properties ... ok
test surface::tests::test_surface_taxonomy_and_properties ... ok
test surface::tests::test_uniform_surface_sampler ... ok
test tire::tests::test_combined_slip_peak_and_symmetry ... ok
test tire::tests::test_curve_constant_places_peak_at_one ... ok
test car::tests::test_terrain_interaction_flotation_and_ice_studs ... ok
test tire::tests::test_friction_circle_clamping ... ok
test sim::tests::test_braking_cadence_simulation ... ok
test tire::tests::test_implicit_wheel_step_holds_peak_traction_under_moderate_drive ... ok
test tire::tests::test_load_sensitivity_reduces_grip_per_newton ... ok
test tire::tests::test_normalized_curve_peaks_at_one_and_falls_to_slide_grip ... ok
test tire::tests::test_pacejka_lateral_force ... ok
test tire::tests::test_wheel_assembly_thermal_fade_and_wear ... ok
test tire::tests::test_wheel_assembly_rotational_inertia_and_lockup ... ok
test tire::tests::test_wheel_assembly_numerical_stability ... ok
test tire::tests::test_wheelspin_erodes_lateral_grip_and_resultant_stays_in_envelope ... ok
test sim::tests::test_reverse_straight_line_simulation ... ok
test sim::tests::test_protocol_a_acceleration ... ok
test sim::playground::tests::test_playground_runner_execution ... ok
test sim::tests::test_reverse_step_steer_simulation ... ok
test car::tests::test_reverse_simulation_extended ... ok
test sim::tests::test_systematic_steering_calibration_speed_sweep ... ok
test sim::tests::test_protocol_b_braking ... ok
test sim::tests::test_braking_straight_line_simulation ... ok
test sim::tests::test_protocol_c_skidpad ... ok
test sim::tests::test_reverse_simulation_battery_fleet ... ok
test sim::tests::test_protocol_e_coast_down ... ok
test sim::tests::test_path_simulation_hypothetical_circuit_surfaces ... ok
test sim::tests::test_path_simulation_straight_with_turns_surfaces ... ok

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_deterministic_reproducibility ... ok
test test_infeasible_constraint_detection_and_graceful_fallback ... ok
test test_throughput_sla_performance ... ok
test test_spool_constrained_optimization_satisfies_turning_diameter_limit ... ok
test test_salisbury_rwd_optimization_enforces_power_coast_delta_and_stability ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s


running 9 tests
test test_staggered_tire_dimensions_on_open_wheel_kart ... ok
test test_thermal_grip_degradation_under_prolonged_power_drifting ... ok
test test_independent_front_wheel_brake_lockup_under_trail_braking ... ok
test test_legacy_configuration_backward_compatibility ... ok
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
test test_spool_produces_understeer_moment_versus_open ... ok
test test_lsd_power_lock_changes_corner_exit_yaw ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test test_corner_exit_with_throttle_held_keeps_drive ... ok
test test_lift_off_is_progressive ... ok
test test_small_steer_does_not_weaken_brakes ... ok
test test_roll_balance_flips_which_axle_saturates_first ... ok
test test_grip_knob_moves_lateral_g ... ok
test test_steering_response_is_monotonic_at_every_speed ... ok
test test_random_input_fuzz_is_numerically_stable ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 17 tests
test test_deep_merge_toml_partial_overrides ... ok
test test_in_file_module_override_merging ... ok
test test_config_load_invalid_toml_fallback ... ok
test test_external_module_files_and_hierarchy_precedence ... ok
test test_custom_camera_zoom_levels_configuration ... ok
test test_custom_car_specs_override ... ok
test test_default_config_roundtrip_toml ... ok
test test_display_resolution_and_window_config_roundtrip ... ok
test test_display_config_vehicle_shadows_setting ... ok
test test_config_save_and_load_from_path ... ok
test test_display_config_surface_texture_quality_roundtrip ... ok
test test_surface_texture_settings_ui_lifecycle ... ok
test test_session_initialization_with_custom_config ... ok
test test_session_module_switching_applies_effective_config ... ok
test test_user_config_installation_and_project_file_protection ... ok
test test_user_module_config_overrides_default_template ... ok
test test_default_gameplay_pilot_count_and_toml_override ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s


running 7 tests
test test_input_config_round_trips_through_filter_config ... ok
test test_legacy_settings_file_loads_as_sharp ... ok
test preset_feel::test_safe_presets_do_not_spin_on_held_key ... ok
test preset_feel::test_peak_yaw_at_speed_is_ordered_by_preset ... ok
test preset_feel::test_turn_in_time_is_ordered_by_preset ... ok
test test_settings_apply_to_live_player_car_both_directions ... ok
test test_catalog_random_input_fuzz_is_numerically_stable ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s


running 7 tests
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_non_linear_center_micro_corrections ... ok
test test_steering_profiles_configuration_and_cycling ... ok
test test_steering_speed_setting_changes_time_to_full_input ... ok
test test_player_car_physics_receives_unattenuated_steering_when_direct_or_raw ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test test_keyboard_kart_caster_jacking_scrub_differential ... ok
test test_keyboard_chicane_reversal_latency_measurement ... ok
test test_keyboard_slide_catch_recovery_on_dirt ... ok
test test_keyboard_filter_profiles_direct_vs_balanced_cornering ... ok
test test_keyboard_sweeper_feathering_preserves_speed_vs_holding ... ok
test test_key_styles_do_not_spin_on_safe_presets ... ok
test test_presets_change_key_style_spread ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s


running 25 tests
test test_audio_mixer_buses ... ok
test test_arcade_settings_modal_controls_subtabs_navigation_and_profile_presets ... ok
test test_arcade_settings_modal_controls_tab_widgets_and_rollback ... ok
test test_arcade_settings_modal_helpers_tab_integration ... ok
test test_digital_input_filter_progressive_ramp ... ok
test test_display_resolutions_and_window_modes ... ok
test test_crt_scanlines_and_settings_integration ... ok
test test_arcade_settings_modal_display_tab_integration ... ok
test test_arcade_settings_modal_lifecycle_and_bindings ... ok
test test_floating_text_popups_and_decay ... ok
test test_cabinet_settings_widgets_interaction ... ok
test test_juice_fx_mechanics ... ok
test test_modal_screen_stack ... ok
test test_nav_grid_2d_orthogonal_navigation ... ok
test test_profile_manager_lifecycle ... ok
test test_screen_stack_transitions_lifecycle ... ok
test test_ui_scaler_responsive_math ... ok
test test_input_mapping_action_system ... ok
test test_leaderboard_modal_rendering_and_scrolling ... ok
test test_profile_select_modal_slot_and_customization ... ok
test test_universal_confirm_modal_lifecycle ... ok
test test_cabinet_context_audio_wiring_and_tactile_feedback ... ok
test test_arcade_settings_dirty_tracking_and_exit_modal ... ok
test test_arcade_settings_modal_gameplay_subtab_navigation ... ok
test test_arcade_settings_modal_arrow_category_navigation ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/wheelbase-803e088669f423cb)
     Running unittests src/bin/auto_tune.rs (target/debug/deps/auto_tune-72ec9574f31cd182)
     Running tests/auto_calibration_tests.rs (target/debug/deps/auto_calibration_tests-f8d5d5fb626fa03c)
     Running tests/decoupled_tire_physics_tests.rs (target/debug/deps/decoupled_tire_physics_tests-16b86e1223464c35)
     Running tests/differential_dynamics_tests.rs (target/debug/deps/differential_dynamics_tests-401438944c103383)
     Running tests/handling_calibration_tests.rs (target/debug/deps/handling_calibration_tests-56637bf6000dc95f)
   Doc-tests wheelbase
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/config_tests.rs (target/debug/deps/config_tests-fcad2df1042e6460)
     Running tests/handling_presets_tests.rs (target/debug/deps/handling_presets_tests-42cb79ea13f17503)
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-2de0f2202abe3dd2)
     Running tests/keyboard_simulation_tests.rs (target/debug/deps/keyboard_simulation_tests-b2178e36f26d560f)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/cabinet_integration_tests.rs (target/debug/deps/cabinet_integration_tests-5ffbcaf2a70aae8f)

```
