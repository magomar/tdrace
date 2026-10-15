---
type: validation_receipt
schema_version: "1.0"
spec: "specs/117_wheelbase_4m_steps_sec_tire_solver_acceleration_and_transcendental_deduplication.md"
epic: "tdrace-sp117-mfdey"
candidate_commit: "04724bb3ca12ea76791006e7b75a742b9a9264b6"
verifier: "local-user"
evaluated_at: "2026-10-15T22:24:53Z"
command: "cargo test -p wheelbase"
exit_code: 0
duration_ms: 1529
status: passed
---

# 🧾 Validation Receipt: Spec 117

- **Candidate Commit**: `04724bb3ca12ea76791006e7b75a742b9a9264b6`
- **Spec**: `specs/117_wheelbase_4m_steps_sec_tire_solver_acceleration_and_transcendental_deduplication.md`
- **Command**: `cargo test -p wheelbase`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 68 tests
test bike::tests::test_motorbike_initialization ... ok
test bike::tests::test_motorbike_acceleration_and_wheelie ... ok
test bike::tests::test_motorbike_lean_and_camber_cornering ... ok
test car::tests::test_ackermann_angles ... ok
test car::tests::low_speed_authority_expands_progressively_and_preserves_profile_order ... ok
test car::tests::test_car_initialization ... ok
test car::tests::test_crest_unloading ... ok
test car::tests::test_directional_impact_classification_and_damage_partitioning ... ok
test car::tests::test_rally_balance_only_reshapes_all_wheel_drive ... ok
test car::tests::test_reverse_drive_bias_distribution ... ok
test config::tests::test_config_presets ... ok
test car::tests::test_grade_slope_resistance ... ok
test config::tests::test_finalize_derives_wheels_from_axle_settings ... ok
test config::tests::test_pacejka_peak_slip_angle_migration ... ok
test config::tests::test_set_slick_compound_gives_the_tire_the_compound_character ... ok
test config::tests::test_legacy_config_deserialization_backward_compatibility ... ok
test config::tests::test_staggered_kart_wheel_assemblies ... ok
test car::tests::test_straight_line_step ... ok
test config::tests::test_suspension_archetype_factors_and_engine_placement ... ok
test config::tests::test_wheel_assembly_inertia_derivation ... ok
test config::tests::test_engine_placement_serde_compatibility ... ok
test car::tests::test_is_braking_state_off_throttle_vs_braking ... ok
test car::tests::test_stopped_car_on_superelevated_segment_remains_static ... ok
test sim::playground::tests::test_composite_gauntlet_structure ... ok
test sim::playground::tests::test_parametric_turn_generators ... ok
test car::tests::test_step_with_sampler ... ok
test surface::tests::test_compound_surface_affinities ... ok
test car::tests::test_state_save_restore ... ok
test surface::tests::test_surface_taxonomy_and_properties ... ok
test surface::tests::test_surface_affinity_map_deserialization_backward_compatibility ... ok
test surface::tests::test_surface_properties ... ok
test tire::tests::test_combined_slip_peak_and_symmetry ... ok
test tire::tests::test_friction_circle_clamping ... ok
test surface::tests::test_uniform_surface_sampler ... ok
test sim::tests::test_headless_harness_straight_line ... ok
test tire::tests::test_curve_constant_places_peak_at_one ... ok
test tire::tests::test_normalized_curve_peaks_at_one_and_falls_to_slide_grip ... ok
test tire::tests::test_pacejka_lateral_force ... ok
test tire::tests::test_spec_117_curve_constant_and_peak_slip_tan_accuracy ... ok
test tire::tests::test_spec_117_fused_fx_and_stiffness ... ok
test tire::tests::test_load_sensitivity_reduces_grip_per_newton ... ok
test car::tests::test_rally_car_rotates_into_a_corner_under_throttle ... ok
test tire::tests::test_implicit_wheel_step_holds_peak_traction_under_moderate_drive ... ok
test tire::tests::test_wheel_assembly_rotational_inertia_and_lockup ... ok
test sim::playground::tests::test_protocol_g_wall_contact_hierarchy_and_anti_wall_riding ... ok
test tire::tests::test_wheel_assembly_thermal_fade_and_wear ... ok
test tire::tests::test_wheelspin_erodes_lateral_grip_and_resultant_stays_in_envelope ... ok
test tire::tests::test_wheel_assembly_numerical_stability ... ok
test car::tests::test_reverse_heading_stability_and_steering_symmetry ... ok
test sim::tests::test_braking_in_turn_simulation ... ok
test car::tests::test_reverse_handbrake ... ok
test car::tests::test_terrain_interaction_flotation_and_ice_studs ... ok
test sim::tests::test_braking_cadence_simulation ... ok
test sim::tests::test_reverse_straight_line_simulation ... ok
test car::tests::test_reverse_straight_line_neutral_steer ... ok
test sim::tests::test_protocol_a_acceleration ... ok
test sim::tests::test_braking_split_mu_simulation ... ok
test sim::tests::test_reverse_step_steer_simulation ... ok
test sim::playground::tests::test_playground_runner_execution ... ok
test car::tests::test_reverse_simulation_extended ... ok
test sim::tests::test_systematic_steering_calibration_speed_sweep ... ok
test sim::tests::test_protocol_b_braking ... ok
test sim::tests::test_braking_straight_line_simulation ... ok
test sim::tests::test_protocol_c_skidpad ... ok
test sim::tests::test_reverse_simulation_battery_fleet ... ok
test sim::tests::test_protocol_e_coast_down ... ok
test sim::tests::test_path_simulation_hypothetical_circuit_surfaces ... ok
test sim::tests::test_path_simulation_straight_with_turns_surfaces ... ok

test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_deterministic_reproducibility ... ok
test test_infeasible_constraint_detection_and_graceful_fallback ... ok
test test_throughput_sla_performance ... ok
test test_salisbury_rwd_optimization_enforces_power_coast_delta_and_stability ... ok
test test_spool_constrained_optimization_satisfies_turning_diameter_limit ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s


running 6 tests
test test_geometric_center_offset_from_cg ... ok
test test_light_positions_world ... ok
test test_to_body_hull ... ok
test test_modality_chassis_platforms_spec_094 ... ok
test test_all_presets_have_positive_chassis_bounds ... ok
test test_serde_synthesis_when_chassis_omitted ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 17 tests
test test_compound_surface_affinity_mud_tractive_force_ratio ... ok
test test_independent_front_wheel_brake_lockup_under_trail_braking ... ok
test test_staggered_tire_dimensions_on_open_wheel_kart ... ok
test test_legacy_configuration_backward_compatibility ... ok
test test_kart_cornering_under_throttle_preserves_drive_and_prevents_runaway_wheelspin ... ok
test test_thermal_grip_degradation_under_prolonged_power_drifting ... ok
test test_kart_caster_jacking_inside_rear_wheel_unloading ... ok
test test_wear_rate_scale_defaults_to_one ... ok
test test_grass_runoff_mobility_and_acceleration ... ok
test test_kart_low_speed_geometric_turning_circle ... ok
test test_kart_high_speed_tight_turning_radius_and_lateral_grip ... ok
test test_rally_car_runs_all_terrain_tyres ... ok
test test_curbs_move_load_but_add_none ... ok
test test_wear_rate_scale_is_separate_from_the_compound_wear_rate ... ok
test test_wear_rate_scale_multiplies_the_tread_wear_step ... ok
test test_a_car_with_one_axle_or_side_on_a_curb_drives_off ... ok
test test_headless_simulation_throughput_sla ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s


running 7 tests
test test_open_differential_equalizes_torque_and_limits_power_on_split_mu ... ok
test test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle ... ok
test test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor ... ok
test test_spool_differential_locks_wheel_rotational_velocities ... ok
test test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded ... ok
test test_lsd_power_lock_changes_corner_exit_yaw ... ok
test test_spool_produces_understeer_moment_versus_open ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
test test_directional_impact_masking_zones ... ok
test test_jump_landing_damping_loss_roll_snap ... ok
test test_minor_collision_and_deadzone_immunity ... ok
test test_pushrod_fragility_drag_penalty ... ok
test test_rear_engine_headon_collision_immunity ... ok
test test_macpherson_curb_bottoming_steering_pull ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 16 tests
test test_assist_intervention_telemetry_separates_sources_and_resets_each_step ... ok
test test_digital_pro_recovery_does_not_leak_to_analog ... ok
test test_digital_flick_opens_low_speed_steering_headroom_once ... ok
test test_low_speed_authority_is_higher_and_fades_to_stock_at_speed ... ok
test test_digital_flick_headroom_does_not_rearm_on_holds_or_feathering ... ok
test test_corner_exit_with_throttle_held_keeps_drive ... ok
test assist_telemetry_roundtrips_in_car_state_while_source_latch_stays_runtime_only ... ok
test test_lift_off_is_progressive ... ok
test test_traction_help_catches_power_oversteer ... ok
test test_small_steer_does_not_weaken_brakes ... ok
test test_roll_balance_flips_which_axle_saturates_first ... ok
test test_traction_help_does_not_ease_straight_line_drive ... ok
test test_grip_knob_moves_lateral_g ... ok
test test_low_speed_monotonic_curvature_and_braking_remain_calibrated ... ok
test test_steering_response_is_monotonic_at_every_speed ... ok
test test_random_input_fuzz_is_numerically_stable ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 6 tests
test test_extreme_offroad_jump_landing_absorption ... ok
test test_hypercar_bottoming_out_on_sausage_kerb ... ok
test test_kart_rigid_chassis_diagonal_jacking_on_kerb ... ok
test test_solid_live_axle_coupled_camber_kinematics ... ok
test test_gt3_double_wishbone_camber_preservation ... ok
test test_gt4_macpherson_camber_loss_and_grip_degradation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/wheelbase-1547161ff291be56)
     Running unittests src/bin/auto_tune.rs (target/debug/deps/auto_tune-03f74640c5c6768b)
     Running tests/auto_calibration_tests.rs (target/debug/deps/auto_calibration_tests-89b703532b05fd97)
     Running tests/chassis_skeleton_tests.rs (target/debug/deps/chassis_skeleton_tests-385cfc04b7e93150)
     Running tests/decoupled_tire_physics_tests.rs (target/debug/deps/decoupled_tire_physics_tests-7e3c0a5457f2929d)
     Running tests/differential_dynamics_tests.rs (target/debug/deps/differential_dynamics_tests-7c87a6f85e9f0883)
     Running tests/directional_damage_tests.rs (target/debug/deps/directional_damage_tests-05916fda5b7189c3)
     Running tests/handling_calibration_tests.rs (target/debug/deps/handling_calibration_tests-34230ff10f0d851a)
     Running tests/suspension_dynamics_tests.rs (target/debug/deps/suspension_dynamics_tests-205215c3c40ef4f2)
   Doc-tests wheelbase

```
