---
type: validation_receipt
schema_version: "1.0"
spec: "specs/046_humanlike_bot_driving_with_tiered_mistakes_and_varied_lines.md"
epic: "tdrace-y3k5"
candidate_commit: "bc77e8d42d44b3603996d059fd931dc481223ce6"
verifier: "local-user"
evaluated_at: "2026-09-28T12:51:01Z"
command: "cargo test -p wheelbase && cargo test -p tdrace-app --test bot_humanlike_driving_tests --test ai_tests --test adversarial_piece4_tests --test orthogonal_ai_styles_and_tiers_tests --test predefined_driver_behaviours_tests --test driver_character_tests --test dynamic_roster_and_tier_tests --test handling_presets_tests --test keyboard_simulation_tests --test input_smoothing_tests --test config_tests"
exit_code: 0
duration_ms: 134434
status: passed
---

# 🧾 Validation Receipt: Spec 046

- **Candidate Commit**: `bc77e8d42d44b3603996d059fd931dc481223ce6`
- **Spec**: `specs/046_humanlike_bot_driving_with_tiered_mistakes_and_varied_lines.md`
- **Command**: `cargo test -p wheelbase && cargo test -p tdrace-app --test bot_humanlike_driving_tests --test ai_tests --test adversarial_piece4_tests --test orthogonal_ai_styles_and_tiers_tests --test predefined_driver_behaviours_tests --test driver_character_tests --test dynamic_roster_and_tier_tests --test handling_presets_tests --test keyboard_simulation_tests --test input_smoothing_tests --test config_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 56 tests
test bike::tests::test_motorbike_initialization ... ok
test bike::tests::test_motorbike_acceleration_and_wheelie ... ok
test bike::tests::test_motorbike_lean_and_camber_cornering ... ok
test car::tests::test_ackermann_angles ... ok
test config::tests::test_config_presets ... ok
test config::tests::test_finalize_derives_wheels_from_axle_settings ... ok
test car::tests::test_car_initialization ... ok
test config::tests::test_pacejka_peak_slip_angle_migration ... ok
test car::tests::test_reverse_drive_bias_distribution ... ok
test car::tests::test_crest_unloading ... ok
test config::tests::test_staggered_kart_wheel_assemblies ... ok
test sim::playground::tests::test_composite_gauntlet_structure ... ok
test sim::playground::tests::test_parametric_turn_generators ... ok
test car::tests::test_stopped_car_on_superelevated_segment_remains_static ... ok
test config::tests::test_legacy_config_deserialization_backward_compatibility ... ok
test car::tests::test_straight_line_step ... ok
test car::tests::test_step_with_sampler ... ok
test sim::playground::tests::test_protocol_g_wall_contact_hierarchy_and_anti_wall_riding ... ok
test car::tests::test_state_save_restore ... ok
test car::tests::test_reverse_heading_stability_and_steering_symmetry ... ok
test car::tests::test_grade_slope_resistance ... ok
test sim::tests::test_headless_harness_straight_line ... ok
test sim::tests::test_braking_in_turn_simulation ... ok
test car::tests::test_terrain_interaction_flotation_and_ice_studs ... ok
test car::tests::test_is_braking_state_off_throttle_vs_braking ... ok
test car::tests::test_reverse_handbrake ... ok
test car::tests::test_reverse_straight_line_neutral_steer ... ok
test sim::tests::test_braking_split_mu_simulation ... ok
test surface::tests::test_surface_properties ... ok
test surface::tests::test_surface_taxonomy_and_properties ... ok
test surface::tests::test_uniform_surface_sampler ... ok
test tire::tests::test_combined_slip_peak_and_symmetry ... ok
test tire::tests::test_curve_constant_places_peak_at_one ... ok
test tire::tests::test_friction_circle_clamping ... ok
test tire::tests::test_implicit_wheel_step_holds_peak_traction_under_moderate_drive ... ok
test tire::tests::test_load_sensitivity_reduces_grip_per_newton ... ok
test tire::tests::test_normalized_curve_peaks_at_one_and_falls_to_slide_grip ... ok
test tire::tests::test_pacejka_lateral_force ... ok
test sim::tests::test_braking_cadence_simulation ... ok
test tire::tests::test_wheel_assembly_numerical_stability ... ok
test tire::tests::test_wheel_assembly_rotational_inertia_and_lockup ... ok
test tire::tests::test_wheel_assembly_thermal_fade_and_wear ... ok
test tire::tests::test_wheelspin_erodes_lateral_grip_and_resultant_stays_in_envelope ... ok
test sim::tests::test_reverse_straight_line_simulation ... ok
test sim::tests::test_protocol_a_acceleration ... ok
test sim::playground::tests::test_playground_runner_execution ... ok
test sim::tests::test_reverse_step_steer_simulation ... ok
test car::tests::test_reverse_simulation_extended ... ok
test sim::tests::test_systematic_steering_calibration_speed_sweep ... ok
test sim::tests::test_braking_straight_line_simulation ... ok
test sim::tests::test_protocol_b_braking ... ok
test sim::tests::test_protocol_c_skidpad ... ok
test sim::tests::test_reverse_simulation_battery_fleet ... ok
test sim::tests::test_protocol_e_coast_down ... ok
test sim::tests::test_path_simulation_hypothetical_circuit_surfaces ... ok
test sim::tests::test_path_simulation_straight_with_turns_surfaces ... ok

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_deterministic_reproducibility ... ok
test test_infeasible_constraint_detection_and_graceful_fallback ... ok
test test_throughput_sla_performance ... ok
test test_spool_constrained_optimization_satisfies_turning_diameter_limit ... ok
test test_salisbury_rwd_optimization_enforces_power_coast_delta_and_stability ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s


running 9 tests
test test_thermal_grip_degradation_under_prolonged_power_drifting ... ok
test test_staggered_tire_dimensions_on_open_wheel_kart ... ok
test test_independent_front_wheel_brake_lockup_under_trail_braking ... ok
test test_legacy_configuration_backward_compatibility ... ok
test test_kart_caster_jacking_inside_rear_wheel_unloading ... ok
test test_kart_cornering_under_throttle_preserves_drive_and_prevents_runaway_wheelspin ... ok
test test_kart_high_speed_tight_turning_radius_and_lateral_grip ... ok
test test_kart_low_speed_geometric_turning_circle ... ok
test test_headless_simulation_throughput_sla ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s


running 7 tests
test test_open_differential_equalizes_torque_and_limits_power_on_split_mu ... ok
test test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle ... ok
test test_spool_differential_locks_wheel_rotational_velocities ... ok
test test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor ... ok
test test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded ... ok
test test_lsd_power_lock_changes_corner_exit_yaw ... ok
test test_spool_produces_understeer_moment_versus_open ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
test test_corner_exit_with_throttle_held_keeps_drive ... ok
test test_lift_off_is_progressive ... ok
test test_traction_help_catches_power_oversteer ... ok
test test_small_steer_does_not_weaken_brakes ... ok
test test_roll_balance_flips_which_axle_saturates_first ... ok
test test_traction_help_does_not_ease_straight_line_drive ... ok
test test_grip_knob_moves_lateral_g ... ok
test test_steering_response_is_monotonic_at_every_speed ... ok
test test_random_input_fuzz_is_numerically_stable ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_ui_and_hud_formatting_corner_cases ... ok
test test_camera_extreme_edge_cases_and_teleportation ... ok
test test_long_race_fx_memory_boundedness ... ok
test test_bot_ai_multi_track_lap_progression ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.32s


running 6 tests
test test_bot_profiles_creation ... ok
test test_bot_ai_collision_avoidance ... ok
test test_bot_ai_steering_and_throttle_on_straight ... ok
test test_bot_ai_cornering_slowdown ... ok
test test_bot_ai_slipstream_drafting_and_slingshot_pack_racing ... ok
test test_bot_ai_on_kart_grid_positions ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s


running 8 tests
test test_same_seed_gives_the_same_race ... ok
test test_human_layer_off_equals_pre_046_controller ... ok
test test_bots_do_not_drive_the_same_path_every_lap ... ok
test test_mistakes_follow_the_tier ... ok
test test_tier_1_is_relatively_easy_to_beat ... ok
test test_no_bot_gets_stuck ... ok
test test_bots_keep_their_driving_style ... ok
test test_keyboard_reference_driver_beats_tier_1 has been running for over 60 seconds
test test_keyboard_reference_driver_beats_tier_1 ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 88.03s


running 17 tests
test test_deep_merge_toml_partial_overrides ... ok
test test_config_load_invalid_toml_fallback ... ok
test test_in_file_module_override_merging ... ok
test test_external_module_files_and_hierarchy_precedence ... ok
test test_display_config_vehicle_shadows_setting ... ok
test test_display_resolution_and_window_config_roundtrip ... ok
test test_default_config_roundtrip_toml ... ok
test test_custom_car_specs_override ... ok
test test_custom_camera_zoom_levels_configuration ... ok
test test_config_save_and_load_from_path ... ok
test test_display_config_surface_texture_quality_roundtrip ... ok
test test_surface_texture_settings_ui_lifecycle ... ok
test test_session_initialization_with_custom_config ... ok
test test_session_module_switching_applies_effective_config ... ok
test test_user_config_installation_and_project_file_protection ... ok
test test_user_module_config_overrides_default_template ... ok
test test_default_gameplay_pilot_count_and_toml_override ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s


running 8 tests
test test_driver_roster_integrity_and_distinct_properties ... ok
test test_driver_roster_sampling_uniqueness ... ok
test test_no_preset_character_uses_player_default_colors ... ok
test test_all_six_modules_driving_style_distribution ... ok
test test_all_roster_bios_wrap_within_dossier_width ... ok
test test_driver_cards_navigation_state ... ok
test test_starting_grid_flow_and_roster_presentation ... ok
test test_race_session_driver_spawning_and_names ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s


running 19 tests
test test_spec024_scenario_bell_curve_tier_distribution_contender ... ok
test test_bell_curve_weights_specification_and_sampling ... ok
test test_career_roster_initialization_and_tier1_centering ... ok
test test_spec024_scenario_72_characters_alignment_and_no_baked_in_tiers ... ok
test test_spec024_scenario_boundary_tier_distributions_rookie_and_legend ... ok
test test_uniform_style_roster_sampling_determinism ... ok
test test_career_roster_evolution_end_to_end_multitier_progression ... ok
test test_module_career_progress_advance_tier_evolution ... ok
test test_spec024_scenario_career_10_car_grid_90_10_churn ... ok
test test_career_roster_evolution_90_10_churn_and_replacements ... ok
test test_championship_session_trigger_roster_evolution ... ok
test test_sqlite_persistence_of_career_rivals ... ok
test test_uniform_style_roster_sampling_quotas_and_uniqueness ... ok
test test_career_roster_probabilistic_skill_progression_distribution ... ok
test test_uniform_distribution_across_styles_over_many_trials ... ok
test test_career_roster_uniform_replacement_style_distribution ... ok
test test_career_roster_rank_dependent_skill_advancement ... ok
test test_spec024_scenario_uniform_sampling_12_driver_rosters_across_1000_seeds ... ok
test test_session_casual_race_dynamic_difficulty_and_bell_curve ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.00s


running 7 tests
test test_input_config_round_trips_through_filter_config ... ok
test test_legacy_settings_file_loads_as_sharp ... ok
test preset_feel::test_safe_presets_do_not_spin_on_held_key ... ok
test preset_feel::test_peak_yaw_at_speed_is_ordered_by_preset ... ok
test preset_feel::test_turn_in_time_is_ordered_by_preset ... ok
test test_catalog_random_input_fuzz_is_numerically_stable ... ok
test test_settings_apply_to_live_player_car_both_directions ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.11s


running 7 tests
test test_non_linear_center_micro_corrections ... ok
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_steering_profiles_configuration_and_cycling ... ok
test test_steering_speed_setting_changes_time_to_full_input ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_player_car_physics_receives_unattenuated_steering_when_direct_or_raw ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test test_keyboard_kart_caster_jacking_scrub_differential ... ok
test test_keyboard_chicane_reversal_latency_measurement ... ok
test test_keyboard_filter_profiles_direct_vs_balanced_cornering ... ok
test test_keyboard_slide_catch_recovery_on_dirt ... ok
test test_keyboard_sweeper_feathering_preserves_speed_vs_holding ... ok
test test_key_styles_do_not_spin_on_safe_presets ... ok
test test_presets_change_key_style_spread ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s


running 7 tests
test test_monotonic_braking_safety_padding_across_tiers ... ok
test test_monotonic_avoidance_distance_padding_across_tiers ... ok
test test_monotonic_pace_scaling_across_tiers ... ok
test test_all_30_combinations_generate_valid_bounded_profiles ... ok
test test_driver_character_classification ... ok
test test_compound_and_legacy_archetype_resolution ... ok
test test_series_toml_with_orthogonal_style_and_tier ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s


running 7 tests
test test_index_based_archetypes ... ok
test test_behavior_archetypes_diversity ... ok
test test_seeded_sampling_variation_and_determinism ... ok
test test_find_global_resolves_all_modules ... ok
test test_all_across_modules_integrity_and_distinct_ids ... ok
test test_series_ai_character_propagation_and_distinct_profiles ... ok
test test_all_modules_standard_race_instantiates_predefined_characters ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.11s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/deps/wheelbase-803e088669f423cb)
     Running unittests src/bin/auto_tune.rs (target/debug/deps/auto_tune-72ec9574f31cd182)
     Running tests/auto_calibration_tests.rs (target/debug/deps/auto_calibration_tests-f8d5d5fb626fa03c)
     Running tests/decoupled_tire_physics_tests.rs (target/debug/deps/decoupled_tire_physics_tests-16b86e1223464c35)
     Running tests/differential_dynamics_tests.rs (target/debug/deps/differential_dynamics_tests-401438944c103383)
     Running tests/handling_calibration_tests.rs (target/debug/deps/handling_calibration_tests-56637bf6000dc95f)
   Doc-tests wheelbase
   Compiling tdrace-core v0.1.0 (/Users/mario.gomez/workspace/games/tdrace/.claude/worktrees/tdrace-vehicle-dynamics-83fc42/crates/tdrace-core)
   Compiling cabinet v0.1.0 (/Users/mario.gomez/workspace/games/tdrace/.claude/worktrees/tdrace-vehicle-dynamics-83fc42/crates/cabinet)
   Compiling tdrace-app v0.1.0 (/Users/mario.gomez/workspace/games/tdrace/.claude/worktrees/tdrace-vehicle-dynamics-83fc42/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.91s
     Running tests/adversarial_piece4_tests.rs (target/debug/deps/adversarial_piece4_tests-4f6f0ca7e51f6c4b)
     Running tests/ai_tests.rs (target/debug/deps/ai_tests-28006b9857ed0ed7)
     Running tests/bot_humanlike_driving_tests.rs (target/debug/deps/bot_humanlike_driving_tests-3cc3ca984176912c)
     Running tests/config_tests.rs (target/debug/deps/config_tests-17d73a05371599dd)
     Running tests/driver_character_tests.rs (target/debug/deps/driver_character_tests-39609d31e6a1d437)
     Running tests/dynamic_roster_and_tier_tests.rs (target/debug/deps/dynamic_roster_and_tier_tests-e297145f84aa2b7a)
     Running tests/handling_presets_tests.rs (target/debug/deps/handling_presets_tests-2ef47fe3794716ce)
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-e9f89b364db7b9ae)
     Running tests/keyboard_simulation_tests.rs (target/debug/deps/keyboard_simulation_tests-01e3341db3fbc2eb)
     Running tests/orthogonal_ai_styles_and_tiers_tests.rs (target/debug/deps/orthogonal_ai_styles_and_tiers_tests-a594cf3059f8d981)
     Running tests/predefined_driver_behaviours_tests.rs (target/debug/deps/predefined_driver_behaviours_tests-c064bf5b7c4e8b0f)

```
