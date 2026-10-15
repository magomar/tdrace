---
type: validation_receipt
schema_version: "1.0"
spec: "specs/118_chassis_guided_wheel_surface_sampling_pyengine_raceworld_unification_and_fastrasterizer_culling.md"
epic: "tdrace-sp118-g1o43"
candidate_commit: "c8cf176facae39dc3fe06684e02fb1dd6e4f604c"
verifier: "local-user"
evaluated_at: "2026-10-15T22:45:31Z"
command: "cargo test -p arcade-race-core --lib && cargo test -p race-kit && cargo test -p tdrace-py --lib"
exit_code: 0
duration_ms: 29324
status: passed
---

# 🧾 Validation Receipt: Spec 118

- **Candidate Commit**: `c8cf176facae39dc3fe06684e02fb1dd6e4f604c`
- **Spec**: `specs/118_chassis_guided_wheel_surface_sampling_pyengine_raceworld_unification_and_fastrasterizer_culling.md`
- **Command**: `cargo test -p arcade-race-core --lib && cargo test -p race-kit && cargo test -p tdrace-py --lib`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 87 tests
test collision::car_collision::tests::test_head_on_car_car_elastic_collision ... ok
test collision::sat::tests::test_obb_obb_overlap_and_separation ... ok
test collision::car_collision::tests::test_multi_car_pileup_non_tunneling ... ok
test collision::wall::tests::test_corner_impact_angular_impulse ... ok
test collision::wall::tests::test_low_speed_wall_impact_damage_deadzone ... ok
test collision::wall::tests::test_head_on_wall_collision_restitution ... ok
test collision::wall::tests::test_virtual_barrier_collision_bypass ... ok
test collision::sat::tests::test_rotated_obb_collision ... ok
test track::checkpoint::tests::test_checkpoint_crossing ... ok
test track::checkpoint::tests::test_pit_lane_anti_cut_stops_only_on_service ... ok
test track::geometry::tests::test_barrier_type_virtual_properties ... ok
test track::geometry::tests::test_jump_ramp_surface_and_flat_portion_fit ... ok
test track::geometry::tests::test_line_segment_projection_and_distance ... ok
test track::geometry::tests::test_jump_ramp_angle_and_size_manipulation ... ok
test track::geometry::tests::test_ray_segment_intersection ... ok
test track::geometry::tests::test_surface_shapes ... ok
test track::geometry::tests::test_line_segment_intersection ... ok
test track::geometry::tests::test_pit_lane_geometry ... ok
test track::network::tests::test_clearance_gap_parallel_and_diverging ... ok
test track::network::tests::test_gore_triangle_keeps_the_ingress_surface ... ok
test track::network::tests::test_c1_tangent_alignment_at_branch_split_socket ... ok
test track::network::tests::test_degenerate_junction_triangle_contains_no_point ... ok
test track::network::tests::test_road_segment_resampling_and_projection ... ok
test track::network::tests::test_spline_socket_ghost_point_and_c1_continuity ... ok
test track::network::tests::test_split_width_envelope_hermite_widening ... ok
test track::presets::tests::test_merge_collinear_walls ... ok
test track::scenery::tests::test_building_properties_and_obstacle ... ok
test track::scenery::tests::test_grandstand_geometry_and_surface ... ok
test track::scenery::tests::test_tree_trunk_and_canopy_containment ... ok
test track::scenery::tests::test_rock_properties_and_obstacle ... ok
test track::network::tests::test_track_network_branch_segments_filtering ... ok
test track::spline::tests::test_centripetal_catmull_rom_equidistant_equivalence ... ok
test track::spline::tests::test_centripetal_catmull_rom_suppresses_extreme_overshoot ... ok
test track::scenery::tests::test_tree_type_properties ... ok
test track::spline::tests::test_adaptive_spline_sampling_density ... ok
test track::tests::test_segment_runoff_corridor_surface_resolution ... ok
test track::checkpoint::tests::test_lap_counting_and_sequence_enforcement ... ok
test track::spline::tests::test_swallowtail_loop_trimming_on_tight_hairpin ... ok
test track::spline::tests::test_synchronized_road_and_curb_untangling_on_hairpin ... ok
test track::spline::tests::test_figure_eight_bridge_detection ... ok
test track::tests::test_default_runoff_surfaces_across_disciplines ... ok
test track::spline::tests::test_elevation_stays_between_segment_waypoints ... ok
test track::spline::tests::test_track_spline_creation_and_sampling ... ok
test track::validation::tests::test_arena_track_validation_clean ... ok
test track::spline::tests::test_track_waypoint_and_spline_runoff_surfaces ... ok
test track::spline::tests::test_natural_hill_elevation_no_bridge ... ok
test track::spline::tests::test_spline_projection_on_track_and_curb ... ok
test track::network::tests::test_track_network_auto_promotion_from_single_spline ... ok
test track::presets::tests::test_speed_weighted_checkpoint_distribution_on_straight_vs_corner ... ok
test track::tests::test_pit_lane_surface_sampling_and_walls ... ok
test track::tests::test_short_inner_walls_of_tight_turns_are_kept ... ok
test lidar::tests::test_lidar_ignores_virtual_barrier ... ok
test lidar::tests::test_lidar_scanner_basic ... ok
test lidar::tests::test_lidar_opponent_detection ... ok
test track::tests::test_surface_layer_precedence_and_shapes ... ok
test track::validation::tests::test_insufficient_waypoints_detected ... ok
test track::spline::tests::test_project_point_ignores_stale_blocks ... ok
test track::tests::test_track_surface_sampling ... ok
test track::bake::tests::test_adaptive_checkpoint_bake_on_short_circuit ... ok
test track::validation::tests::test_curb_boundary_self_intersection_detected ... ok
test track::tests::test_grandstand_and_tree_scenery_sampling_and_serialization ... ok
test track::validation::tests::test_road_boundary_self_intersection_detected ... ok
test track::presets::tests::test_adaptive_checkpoint_count_scaling ... ok
test track::validation::tests::test_missing_finish_line_detected ... ok
test track::validation::tests::test_overlapping_grid_slots_detected ... ok
test track::tests::test_runoff_corridor_width_set_where_wall_is_absent ... ok
test track::validation::tests::test_invalid_wall_distance_detected ... ok
test track::validation::tests::test_preset_track_validation_passes_cleanly ... ok
test track::validation::tests::test_wall_elevated_overpass_bridge_allowed ... ok
test track::tests::test_single_waypoint_wall_distance_override_blends_without_step ... ok
test track::validation::tests::test_obstacle_on_track_detected ... ok
test track::spline::tests::test_continuity_window_covers_every_candidate_in_order ... ok
test track::validation::tests::test_wall_crossing_track_centerline_detected ... ok
test track::validation::tests::test_wall_self_intersection_detected ... ok
test track::tests::test_spec_118_chassis_guided_surface_sampling_parity ... ok
test track::spline::tests::test_silverstone_loop_has_zero_self_intersections ... ok
test track::spline::tests::test_project_point_blocks_match_full_scan ... ok
test track::tests::test_track_presets_creation ... ok
test track::validation::tests::test_scenery_clearance_validation ... ok
test track::tests::test_track_rebuild_geometry ... ok
test track::tests::test_track_surface_breakdown ... ok
test track::bake::tests::test_bake_rejects_too_few_waypoints ... ok
test track::bake::tests::test_rebuild_keeps_the_wall_setup_of_the_track ... ok
test track::bake::tests::test_rebuild_keeps_checkpoint_and_grid_counts ... ok
test track::bake::tests::test_bake_fills_a_source_circuit ... ok
test track::bake::tests::test_bake_keeps_existing_parts_unless_rebuild ... ok
test track::tests::test_track_json_serialization_roundtrip ... ok

test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.92s


running 1 test
test ai::tests::test_driver_tier_indexing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_bot_ai_pit_tactics_and_stall_stopping ... ok
test test_bot_parallel_straight_immunity_and_throat_lateral_repulsion ... ok
test test_bot_tactical_pit_stop_entry_service_and_rejoin_pipeline ... ok
test test_bot_ai_lap1_pit_gating_stall_assignment_and_cooldown ... ok
test bot_drives_a_non_car_vehicle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.74s


running 3 tests
test driver_who_took_the_joker_gets_no_penalty ... ok
test default_rules_never_penalise ... ok
test missed_joker_adds_penalty_and_drops_the_driver_behind_a_slower_finisher ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s


running 4 tests
test a_stage_read_mid_race_is_incomplete ... ok
test a_heat_is_deterministic ... ok
test a_heat_of_eight_completes_with_real_times ... ok
test a_wrecked_car_is_last_and_has_no_time ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.69s


running 4 tests
test test_tree_canopy_ground_drag_distinction_bushes_vs_trees ... ok
test wreck_gives_dnf_and_stays_collidable ... ok
test test_pit_service_state_machine ... ok
test laps_race_finishes_with_real_times ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s


running 1 test
test crates/race-kit/src/lib.rs - ReadmeDoctests (line 43) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s


running 2 tests
test engine::tests::test_spec_118_pyengine_road_projection_and_zero_copy_lidar ... ok
test rasterizer::tests::test_spec_118_fast_rasterizer_culling_and_scanline ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-bd2d3fb9efdf928f)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/race_kit-626c8643462c7548)
     Running tests/bot_vehicle_tests.rs (target/debug/deps/bot_vehicle_tests-8c477fe464b35a37)
     Running tests/golden_world.rs (target/debug/deps/golden_world-f6ac92bdd6ea4394)
     Running tests/joker_rule_tests.rs (target/debug/deps/joker_rule_tests-903d8a33845d3638)
     Running tests/tournament_weekend_tests.rs (target/debug/deps/tournament_weekend_tests-d4164d40e4ac2653)
     Running tests/world_tests.rs (target/debug/deps/world_tests-04ba45d71dcccb27)
   Doc-tests race_kit
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src/lib.rs (target/debug/deps/_tdrace-be83eed1e56a88a7)

```
