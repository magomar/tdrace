---
type: validation_receipt
schema_version: "1.0"
spec: "specs/134_nitrocross_circuit_pack_and_mega_jump_over_under_crossover_geometry.md"
epic: "tdrace-sp134-s4qjd"
candidate_commit: "2ab007d74eff8984c883e4e17204bcb9b816bd77"
verifier: "local-user"
evaluated_at: "2026-10-22T08:15:55Z"
command: "cargo test -p arcade-race-core --lib track && cargo test -p tdrace-app --test rally_tracks_tests"
exit_code: 0
duration_ms: 42145
status: passed
---

# 🧾 Validation Receipt: Spec 134

- **Candidate Commit**: `2ab007d74eff8984c883e4e17204bcb9b816bd77`
- **Spec**: `specs/134_nitrocross_circuit_pack_and_mega_jump_over_under_crossover_geometry.md`
- **Command**: `cargo test -p arcade-race-core --lib track && cargo test -p tdrace-app --test rally_tracks_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 82 tests
test track::checkpoint::tests::test_checkpoint_crossing ... ok
test track::geometry::tests::test_barrier_type_virtual_properties ... ok
test track::checkpoint::tests::test_pit_lane_anti_cut_stops_only_on_service ... ok
test track::geometry::tests::test_jump_ramp_angle_and_size_manipulation ... ok
test track::geometry::tests::test_line_segment_intersection ... ok
test track::geometry::tests::test_jump_ramp_surface_and_flat_portion_fit ... ok
test track::geometry::tests::test_line_segment_projection_and_distance ... ok
test track::geometry::tests::test_ray_segment_intersection ... ok
test track::geometry::tests::test_surface_shapes ... ok
test track::geometry::tests::test_whoops_moguls_and_mega_ramp_profiles_and_vehicle_interaction ... ok
test track::geometry::tests::test_pit_lane_geometry ... ok
test track::network::tests::test_c1_tangent_alignment_at_branch_split_socket ... ok
test track::checkpoint::tests::test_lap_counting_and_sequence_enforcement ... ok
test track::network::tests::test_degenerate_junction_triangle_contains_no_point ... ok
test track::network::tests::test_gore_triangle_keeps_the_ingress_surface ... ok
test track::geometry::tests::test_mega_ramp_failed_gap_behaves_like_wall_collision ... ok
test track::network::tests::test_clearance_gap_parallel_and_diverging ... ok
test track::network::tests::test_spline_socket_ghost_point_and_c1_continuity ... ok
test track::network::tests::test_split_width_envelope_hermite_widening ... ok
test track::network::tests::test_road_segment_resampling_and_projection ... ok
test track::network::tests::test_track_network_branch_segments_filtering ... ok
test track::presets::tests::test_merge_collinear_walls ... ok
test track::scenery::tests::test_building_properties_and_obstacle ... ok
test track::scenery::tests::test_tree_type_properties ... ok
test track::scenery::tests::test_grandstand_geometry_and_surface ... ok
test track::scenery::tests::test_rock_properties_and_obstacle ... ok
test track::spline::tests::test_centripetal_catmull_rom_suppresses_extreme_overshoot ... ok
test track::spline::tests::test_centripetal_catmull_rom_equidistant_equivalence ... ok
test track::scenery::tests::test_tree_trunk_and_canopy_containment ... ok
test track::spline::tests::test_adaptive_spline_sampling_density ... ok
test track::tests::test_segment_runoff_corridor_surface_resolution ... ok
test track::spline::tests::test_track_spline_creation_and_sampling ... ok
test track::spline::tests::test_synchronized_road_and_curb_untangling_on_hairpin ... ok
test track::network::tests::test_track_network_auto_promotion_from_single_spline ... ok
test track::spline::tests::test_swallowtail_loop_trimming_on_tight_hairpin ... ok
test track::presets::tests::test_speed_weighted_checkpoint_distribution_on_straight_vs_corner ... ok
test track::spline::tests::test_spline_projection_on_track_and_curb ... ok
test track::spline::tests::test_elevation_stays_between_segment_waypoints ... ok
test track::spline::tests::test_track_waypoint_and_spline_runoff_surfaces ... ok
test track::validation::tests::test_arena_track_validation_clean ... ok
test track::spline::tests::test_figure_eight_bridge_detection ... ok
test track::spline::tests::test_natural_hill_elevation_no_bridge ... ok
test track::validation::tests::test_mega_ramp_over_under_crossover_point_and_validation ... ok
test track::spline::tests::test_project_point_ignores_stale_blocks ... ok
test track::tests::test_short_inner_walls_of_tight_turns_are_kept ... ok
test track::presets::tests::test_adaptive_checkpoint_count_scaling ... ok
test track::tests::test_default_runoff_surfaces_across_disciplines ... ok
test track::tests::test_runoff_corridor_width_set_where_wall_is_absent ... ok
test track::tests::test_single_waypoint_wall_distance_override_blends_without_step ... ok
test track::tests::test_pit_lane_surface_sampling_and_walls ... ok
test track::bake::tests::test_adaptive_checkpoint_bake_on_short_circuit ... ok
test track::tests::test_track_surface_sampling ... ok
test track::validation::tests::test_insufficient_waypoints_detected ... ok
test track::tests::test_surface_layer_precedence_and_shapes ... ok
test track::validation::tests::test_curb_boundary_self_intersection_detected ... ok
test track::validation::tests::test_road_boundary_self_intersection_detected ... ok
test track::tests::test_track_rebuild_geometry ... ok
test track::tests::test_spec_118_chassis_guided_surface_sampling_parity ... ok
test track::bake::tests::test_rebuild_keeps_checkpoint_and_grid_counts ... ok
test track::bake::tests::test_rebuild_keeps_the_wall_setup_of_the_track ... ok
test track::tests::test_grandstand_and_tree_scenery_sampling_and_serialization ... ok
test track::validation::tests::test_invalid_wall_distance_detected ... ok
test track::validation::tests::test_wall_self_intersection_detected ... ok
test track::validation::tests::test_wall_crossing_track_centerline_detected ... ok
test track::validation::tests::test_wall_elevated_overpass_bridge_allowed ... ok
test track::validation::tests::test_obstacle_on_track_detected ... ok
test track::tests::test_track_surface_breakdown ... ok
test track::validation::tests::test_mega_ramp_gap_validation ... ok
test track::validation::tests::test_preset_track_validation_passes_cleanly ... ok
test track::validation::tests::test_missing_finish_line_detected ... ok
test track::validation::tests::test_overlapping_grid_slots_detected ... ok
test track::spline::tests::test_continuity_window_covers_every_candidate_in_order ... ok
test track::tests::test_track_presets_creation ... ok
test track::spline::tests::test_project_point_blocks_match_full_scan ... ok
test track::spline::tests::test_silverstone_loop_has_zero_self_intersections ... ok
test track::validation::tests::test_scenery_clearance_validation ... ok
test track::tests::test_lydden_hill_sync_network_main_segments_eliminates_spurious_wall_holes ... ok
test track::bake::tests::test_bake_rejects_too_few_waypoints ... ok
test track::tests::test_pit_lane_does_not_skew_effective_barrier_offset_or_runoff_width ... ok
test track::bake::tests::test_bake_fills_a_source_circuit ... ok
test track::bake::tests::test_bake_keeps_existing_parts_unless_rebuild ... ok
test track::tests::test_track_json_serialization_roundtrip ... ok

test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 5.62s


running 27 tests
test test_dirt_figure_eight_horizontal_flat_dirt_arena ... ok
test test_dirt_figure_eight_jump_ramps_proportional_trajectory ... ok
test test_spec134_scenario_3_calgary_stampede_snow_and_ice_surface_composition ... ok
test test_blyton_ids_resolve_to_croft ... ok
test test_holjes_rx_joker_lap_time_delta_simulation ... ok
test test_silverstone_and_yas_marina_ids_resolve_to_their_replacements ... ok
test test_spec134_scenario_2_grade_separated_mega_jump_over_under_joker_crossover_without_collision ... ok
test test_spec134_scenario_1_nitrocross_circuit_pack_integrity_and_validation ... ok
test test_rx_race_ignores_the_joker_layout_pick ... ok
test test_export_and_save_rally_tracks_to_disk ... ok
test test_world_rx_tracks_jump_ramps_and_mixed_surfaces ... ok
test test_world_rx_jump_ramps_dirt_surface_and_containment_landing ... ok
test test_all_20_world_rx_circuits_have_valid_joker_track_networks ... ok
test test_rally_tracks_centerline_driving_and_no_wall_obstructions ... ok
test test_famous_rally_tracks_in_track_manager_and_menu_resolution ... ok
test test_rx_joker_costs_lap_time ... ok
test test_rx_races_get_the_joker_rule_and_other_races_do_not ... ok
test test_rx_layout_checkpoints_lie_on_their_route_in_driving_order ... ok
test test_rx_gore_keeps_the_ingress_surface ... ok
test test_rx_main_road_is_walled_up_to_the_joker ... ok
test test_rally_module_tracks_integrity_and_validation ... ok
test test_rx_joker_branch_never_turns_tighter_than_3_m ... ok
test test_rx_joker_road_reads_as_its_own_surface_not_runoff ... ok
test test_rx_joker_branch_has_walls_that_stay_off_every_road ... ok
test test_rally_race_session_simulation_on_new_tracks ... ok
test test_rx_joker_walls_are_rebuilt_on_load_and_never_saved ... ok
test test_rx_car_driving_the_joker_route_gets_its_lap_and_its_joker ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.39s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-bd2d3fb9efdf928f)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/rally_tracks_tests.rs (target/debug/deps/rally_tracks_tests-5655daf2122ba85b)
Checking track: holjes_rx
Checking track: lydden_hill
Checking track: hell_rx
Checking track: loheac_rx
Checking track: estering_rx
Checking track: montalegre_rx
Checking track: nyirad_rx
Checking track: kouvola_rx
Checking track: catalunya_rx
Checking track: mettet_rx
Checking track: lavare_rx
Checking track: riga_rx
Checking track: killarney_rx
Checking track: lessay_rx
Checking track: essay_rx
Checking track: dreux_rx
Checking track: croft_rx
Checking track: spa_rx
Checking track: silverstone_rx
Checking track: erx_motor_park
Checking track: utah_umc_rx
Checking track: wild_horse_pass_rx
Checking track: glen_helen_rx
Checking track: midamerica_outdoors_rx
Checking track: calgary_stampede_rx

```
