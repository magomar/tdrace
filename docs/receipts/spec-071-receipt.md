---
type: validation_receipt
schema_version: "1.0"
spec: "specs/071_centripetal_catmullrom_and_variable_density_track_splines.md"
epic: "tdrace-z914"
candidate_commit: "76ef68a330d9621d91831bede693e8a32ec6f017"
verifier: "local-user"
evaluated_at: "2026-10-01T22:11:34Z"
command: "cargo test -p arcade-race-core track:: && cargo test -p race-ui"
exit_code: 0
duration_ms: 13075
status: passed
---

# 🧾 Validation Receipt: Spec 071

- **Candidate Commit**: `76ef68a330d9621d91831bede693e8a32ec6f017`
- **Spec**: `specs/071_centripetal_catmullrom_and_variable_density_track_splines.md`
- **Command**: `cargo test -p arcade-race-core track:: && cargo test -p race-ui`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 56 tests
test track::checkpoint::tests::test_checkpoint_crossing ... ok
test track::geometry::tests::test_barrier_type_virtual_properties ... ok
test track::geometry::tests::test_jump_ramp_angle_and_size_manipulation ... ok
test track::geometry::tests::test_jump_ramp_surface_and_flat_portion_fit ... ok
test track::geometry::tests::test_line_segment_projection_and_distance ... ok
test track::geometry::tests::test_line_segment_intersection ... ok
test track::geometry::tests::test_ray_segment_intersection ... ok
test track::geometry::tests::test_surface_shapes ... ok
test track::presets::tests::test_merge_collinear_walls ... ok
test track::scenery::tests::test_building_properties_and_obstacle ... ok
test track::scenery::tests::test_grandstand_geometry_and_surface ... ok
test track::scenery::tests::test_rock_properties_and_obstacle ... ok
test track::scenery::tests::test_tree_trunk_and_canopy_containment ... ok
test track::scenery::tests::test_tree_type_properties ... ok
test track::spline::tests::test_centripetal_catmull_rom_equidistant_equivalence ... ok
test track::spline::tests::test_centripetal_catmull_rom_suppresses_extreme_overshoot ... ok
test track::spline::tests::test_swallowtail_loop_trimming_on_tight_hairpin ... ok
test track::spline::tests::test_adaptive_spline_sampling_density ... ok
test track::validation::tests::test_arena_track_validation_clean ... ok
test track::tests::test_segment_runoff_corridor_surface_resolution ... ok
test track::spline::tests::test_figure_eight_bridge_detection ... ok
test track::checkpoint::tests::test_lap_counting_and_sequence_enforcement ... ok
test track::spline::tests::test_spline_projection_on_track_and_curb ... ok
test track::spline::tests::test_natural_hill_elevation_no_bridge ... ok
test track::spline::tests::test_track_waypoint_and_spline_runoff_surfaces ... ok
test track::spline::tests::test_track_spline_creation_and_sampling ... ok
test track::tests::test_default_runoff_surfaces_across_disciplines ... ok
test track::bake::tests::test_bake_rejects_too_few_waypoints ... ok
test track::tests::test_short_inner_walls_of_tight_turns_are_kept ... ok
test track::presets::tests::test_speed_weighted_checkpoint_distribution_on_straight_vs_corner ... ok
test track::spline::tests::test_silverstone_loop_has_zero_self_intersections ... ok
test track::tests::test_track_surface_sampling ... ok
test track::validation::tests::test_insufficient_waypoints_detected ... ok
test track::tests::test_surface_layer_precedence_and_shapes ... ok
test track::presets::tests::test_adaptive_checkpoint_count_scaling ... ok
test track::bake::tests::test_adaptive_checkpoint_bake_on_short_circuit ... ok
test track::tests::test_runoff_corridor_width_set_where_wall_is_absent ... ok
test track::tests::test_single_waypoint_wall_distance_override_blends_without_step ... ok
test track::tests::test_grandstand_and_tree_scenery_sampling_and_serialization ... ok
test track::tests::test_track_presets_creation ... ok
test track::validation::tests::test_obstacle_on_track_detected ... ok
test track::validation::tests::test_preset_track_validation_passes_cleanly ... ok
test track::validation::tests::test_overlapping_grid_slots_detected ... ok
test track::validation::tests::test_missing_finish_line_detected ... ok
test track::validation::tests::test_wall_self_intersection_detected ... ok
test track::validation::tests::test_wall_elevated_overpass_bridge_allowed ... ok
test track::tests::test_track_rebuild_geometry ... ok
test track::tests::test_track_surface_breakdown ... ok
test track::validation::tests::test_invalid_wall_distance_detected ... ok
test track::validation::tests::test_wall_crossing_track_centerline_detected ... ok
test track::tests::test_track_json_serialization_roundtrip ... ok
test track::validation::tests::test_scenery_clearance_validation ... ok
test track::bake::tests::test_bake_keeps_existing_parts_unless_rebuild ... ok
test track::bake::tests::test_bake_fills_a_source_circuit ... ok
test track::bake::tests::test_rebuild_keeps_checkpoint_and_grid_counts ... ok
test track::bake::tests::test_rebuild_keeps_the_wall_setup_of_the_track ... ok

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 3.99s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s


running 6 tests
test render::track::tests::test_batch_mesh_builder_quad_and_triangle_indices ... ok
test render::track::tests::test_empty_and_insufficient_spline_rendering_does_not_panic ... ok
test render::barrier::tests::test_virtual_barrier_rendering_bypass_logic ... ok
test render::track::tests::test_render_surface_shape_textured_all_geometries ... ok
test render::track::tests::test_raised_ground_embankment_shade_pass ... ok
test render::track::tests::test_textured_rendering_across_quality_levels ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.13s


running 1 test
test asset_root_is_searched_first ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test camera_follows_a_non_car_body_like_a_car ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test effects_follow_a_non_car_vehicle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.51s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-bd2d3fb9efdf928f)
     Running tests/body2d_tests.rs (target/debug/deps/body2d_tests-b44fffeaa1d11b10)
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-262de59c3d016fc3)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-4f6b512e9b8ff5d1)
   Compiling race-ui v0.1.0 (/home/mario/workspace/games/tdrace/crates/race-ui)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.35s
     Running unittests src/lib.rs (target/debug/deps/race_ui-cdb83ad1f192b222)
     Running tests/asset_root_tests.rs (target/debug/deps/asset_root_tests-2756425a7e2a1e7f)
     Running tests/camera_body_tests.rs (target/debug/deps/camera_body_tests-c37388fb23b6150b)
     Running tests/fx_vehicle_tests.rs (target/debug/deps/fx_vehicle_tests-1aac3b168723ec1f)
   Doc-tests race_ui

```
