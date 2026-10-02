---
type: validation_receipt
schema_version: "1.0"
spec: "specs/074_decoupled_wheel_geometry_and_data_driven_tire_compounds.md"
epic: "tdrace-recg"
candidate_commit: "29a195f105a605042b7d23c569b4a9ba33e86526"
verifier: "local-user"
evaluated_at: "2026-10-02T17:25:37Z"
command: "cargo test -p wheelbase -p race-ui && cargo test -p tdrace-app --test render_tests"
exit_code: 0
duration_ms: 12159
status: passed
---

# 🧾 Validation Receipt: Spec 074

- **Candidate Commit**: `29a195f105a605042b7d23c569b4a9ba33e86526`
- **Spec**: `specs/074_decoupled_wheel_geometry_and_data_driven_tire_compounds.md`
- **Command**: `cargo test -p wheelbase -p race-ui && cargo test -p tdrace-app --test render_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 6 tests
test render::track::tests::test_batch_mesh_builder_quad_and_triangle_indices ... ok
test render::track::tests::test_empty_and_insufficient_spline_rendering_does_not_panic ... ok
test render::barrier::tests::test_virtual_barrier_rendering_bypass_logic ... ok
test render::track::tests::test_raised_ground_embankment_shade_pass ... ok
test render::track::tests::test_render_surface_shape_textured_all_geometries ... ok
test render::track::tests::test_textured_rendering_across_quality_levels ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.14s


running 1 test
test asset_root_is_searched_first ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test camera_follows_a_non_car_body_like_a_car ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test effects_follow_a_non_car_vehicle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 59 tests
test bike::tests::test_motorbike_initialization ... ok
test bike::tests::test_motorbike_acceleration_and_wheelie ... ok
test bike::tests::test_motorbike_lean_and_camber_cornering ... ok
test car::tests::test_car_initialization ... ok
test car::tests::low_speed_authority_expands_progressively_and_preserves_profile_order ... ok
test car::tests::test_ackermann_angles ... ok
test car::tests::test_crest_unloading ... ok
test car::tests::test_reverse_drive_bias_distribution ... ok
test config::tests::test_config_presets ... ok
test config::tests::test_finalize_derives_wheels_from_axle_settings ... ok
test config::tests::test_pacejka_peak_slip_angle_migration ... ok
test config::tests::test_staggered_kart_wheel_assemblies ... ok
test config::tests::test_wheel_assembly_inertia_derivation ... ok
test config::tests::test_legacy_config_deserialization_backward_compatibility ... ok
test sim::playground::tests::test_composite_gauntlet_structure ... ok
test sim::playground::tests::test_parametric_turn_generators ... ok
test car::tests::test_stopped_car_on_superelevated_segment_remains_static ... ok
test car::tests::test_straight_line_step ... ok
test surface::tests::test_compound_surface_affinities ... ok
test surface::tests::test_surface_properties ... ok
test surface::tests::test_uniform_surface_sampler ... ok
test tire::tests::test_combined_slip_peak_and_symmetry ... ok
test car::tests::test_step_with_sampler ... ok
test tire::tests::test_friction_circle_clamping ... ok
test tire::tests::test_load_sensitivity_reduces_grip_per_newton ... ok
test car::tests::test_state_save_restore ... ok
test tire::tests::test_implicit_wheel_step_holds_peak_traction_under_moderate_drive ... ok
test tire::tests::test_normalized_curve_peaks_at_one_and_falls_to_slide_grip ... ok
test sim::playground::tests::test_protocol_g_wall_contact_hierarchy_and_anti_wall_riding ... ok
test tire::tests::test_pacejka_lateral_force ... ok
test surface::tests::test_surface_taxonomy_and_properties ... ok
test tire::tests::test_wheel_assembly_rotational_inertia_and_lockup ... ok
test tire::tests::test_wheelspin_erodes_lateral_grip_and_resultant_stays_in_envelope ... ok
test tire::tests::test_curve_constant_places_peak_at_one ... ok
test car::tests::test_grade_slope_resistance ... ok
test tire::tests::test_wheel_assembly_thermal_fade_and_wear ... ok
test sim::tests::test_headless_harness_straight_line ... ok
test car::tests::test_reverse_heading_stability_and_steering_symmetry ... ok
test tire::tests::test_wheel_assembly_numerical_stability ... ok
test car::tests::test_is_braking_state_off_throttle_vs_braking ... ok
test car::tests::test_terrain_interaction_flotation_and_ice_studs ... ok
test sim::tests::test_braking_in_turn_simulation ... ok
test car::tests::test_reverse_handbrake ... ok
test sim::tests::test_reverse_straight_line_simulation ... ok
test car::tests::test_reverse_straight_line_neutral_steer ... ok
test sim::tests::test_braking_split_mu_simulation ... ok
test sim::tests::test_braking_cadence_simulation ... ok
test sim::tests::test_protocol_a_acceleration ... ok
test sim::tests::test_reverse_step_steer_simulation ... ok
test sim::playground::tests::test_playground_runner_execution ... ok
test car::tests::test_reverse_simulation_extended ... ok
test sim::tests::test_systematic_steering_calibration_speed_sweep ... ok
test sim::tests::test_protocol_b_braking ... ok
test sim::tests::test_protocol_c_skidpad ... ok
test sim::tests::test_braking_straight_line_simulation ... ok
test sim::tests::test_reverse_simulation_battery_fleet ... ok
test sim::tests::test_protocol_e_coast_down ... ok
test sim::tests::test_path_simulation_hypothetical_circuit_surfaces ... ok
test sim::tests::test_path_simulation_straight_with_turns_surfaces ... ok

test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_deterministic_reproducibility ... ok
test test_infeasible_constraint_detection_and_graceful_fallback ... ok
test test_throughput_sla_performance ... ok
test test_spool_constrained_optimization_satisfies_turning_diameter_limit ... ok
test test_salisbury_rwd_optimization_enforces_power_coast_delta_and_stability ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s


running 5 tests
test test_to_body_hull ... ok
test test_geometric_center_offset_from_cg ... ok
test test_all_presets_have_positive_chassis_bounds ... ok
test test_light_positions_world ... ok
test test_serde_synthesis_when_chassis_omitted ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 10 tests
test test_kart_caster_jacking_inside_rear_wheel_unloading ... ok
test test_legacy_configuration_backward_compatibility ... ok
test test_compound_surface_affinity_mud_tractive_force_ratio ... ok
test test_staggered_tire_dimensions_on_open_wheel_kart ... ok
test test_kart_cornering_under_throttle_preserves_drive_and_prevents_runaway_wheelspin ... ok
test test_thermal_grip_degradation_under_prolonged_power_drifting ... ok
test test_kart_high_speed_tight_turning_radius_and_lateral_grip ... ok
test test_independent_front_wheel_brake_lockup_under_trail_braking ... ok
test test_kart_low_speed_geometric_turning_circle ... ok
test test_headless_simulation_throughput_sla ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s


running 7 tests
test test_open_differential_equalizes_torque_and_limits_power_on_split_mu ... ok
test test_kart_spool_with_caster_jacking_maintains_drive_and_turning_circle ... ok
test test_lsd_transfers_torque_to_gripping_wheel_proportional_to_locking_factor ... ok
test test_spool_differential_transfers_100_percent_torque_when_one_wheel_unloaded ... ok
test test_spool_differential_locks_wheel_rotational_velocities ... ok
test test_lsd_power_lock_changes_corner_exit_yaw ... ok
test test_spool_produces_understeer_moment_versus_open ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 16 tests
test test_assist_intervention_telemetry_separates_sources_and_resets_each_step ... ok
test test_digital_pro_recovery_does_not_leak_to_analog ... ok
test test_digital_flick_headroom_does_not_rearm_on_holds_or_feathering ... ok
test test_digital_flick_opens_low_speed_steering_headroom_once ... ok
test test_low_speed_authority_is_higher_and_fades_to_stock_at_speed ... ok
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

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s


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


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 46 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_car_body_roll_and_geometry ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_palette_and_car_color_schemes ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_spec_075_chassis_skeleton_render_geometry_and_fixture_alignment ... ok
test test_track_wear_state_phase_2_hooks ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_surface_material_quality_and_properties ... ok
test test_track_backdrop_colors ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_surface_asset_files_exist_and_are_valid_png ... ok
test test_procedural_surface_image_generators_all_15_surfaces ... ok
test test_seamless_periodic_grass_and_asphalt_generators ... ok
test test_wheel_texture_cache_memory_bounds ... ok
test test_spec_026_standalone_wheel_texture_asset_integrity ... ok
test test_peugeot_208_rally4_topdown_sprite_orientation ... ok
test test_tony_kart_topdown_sprite_orientation ... ok
test test_vortex_dune_crusher_topdown_sprite_orientation ... ok
test test_classic_kart_topdown_sprite_orientation ... ok
test test_spec_026_colorway_tinting_consistency_on_decomposed_kart ... ok
test test_spline_ribbon_and_world_space_uv_mappings ... ok
test test_segment_curvature_and_apex_rubbering_lateral_distribution ... ok
test test_track_presets_geometry_for_rendering ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_spec_073_classic_12_vehicle_harmonization_and_steered_wheels ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_backdrop_ground_pass_execution ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.59s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src/lib.rs (target/debug/deps/race_ui-cdb83ad1f192b222)
     Running tests/asset_root_tests.rs (target/debug/deps/asset_root_tests-2756425a7e2a1e7f)
     Running tests/camera_body_tests.rs (target/debug/deps/camera_body_tests-c37388fb23b6150b)
     Running tests/fx_vehicle_tests.rs (target/debug/deps/fx_vehicle_tests-1aac3b168723ec1f)
     Running unittests src/lib.rs (target/debug/deps/wheelbase-1547161ff291be56)
     Running unittests src/bin/auto_tune.rs (target/debug/deps/auto_tune-03f74640c5c6768b)
     Running tests/auto_calibration_tests.rs (target/debug/deps/auto_calibration_tests-89b703532b05fd97)
     Running tests/chassis_skeleton_tests.rs (target/debug/deps/chassis_skeleton_tests-385cfc04b7e93150)
     Running tests/decoupled_tire_physics_tests.rs (target/debug/deps/decoupled_tire_physics_tests-7e3c0a5457f2929d)
     Running tests/differential_dynamics_tests.rs (target/debug/deps/differential_dynamics_tests-7c87a6f85e9f0883)
     Running tests/handling_calibration_tests.rs (target/debug/deps/handling_calibration_tests-34230ff10f0d851a)
     Running tests/suspension_dynamics_tests.rs (target/debug/deps/suspension_dynamics_tests-205215c3c40ef4f2)
   Doc-tests race_ui
   Doc-tests wheelbase
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/render_tests.rs (target/debug/deps/render_tests-20de0586ba9dacb9)

```
