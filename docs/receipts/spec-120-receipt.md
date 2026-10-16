---
type: validation_receipt
schema_version: "1.0"
spec: "specs/120_widened_pit_stop_side_bay_outer_stall_placement_and_unobstructed_pit_fast_lane.md"
epic: "tdrace-sp120-jyr2l"
candidate_commit: "23474eab41b758c95383e4404341db4378fd23da"
verifier: "local-user"
evaluated_at: "2026-10-16T10:31:25Z"
command: "cargo test -p arcade-race-core --test pit_kit_tests && cargo test -p tdrace-app --test pit_lane_integration_tests"
exit_code: 0
duration_ms: 24787
status: passed
---

# 🧾 Validation Receipt: Spec 120

- **Candidate Commit**: `23474eab41b758c95383e4404341db4378fd23da`
- **Spec**: `specs/120_widened_pit_stop_side_bay_outer_stall_placement_and_unobstructed_pit_fast_lane.md`
- **Command**: `cargo test -p arcade-race-core --test pit_kit_tests && cargo test -p tdrace-app --test pit_lane_integration_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 25 tests
test layout_json_uses_plain_variant_names ... ok
test entry_apex_is_where_the_pit_edge_leaves_the_track_edge ... ok
test guards_reject_bad_junctions ... ok
test turnoff_leaves_at_its_set_angle_toward_side ... ok
test taper_free_end_is_parallel_at_divider_gap ... ok
test turnoff_exit_rejoins_parallel_at_s ... ok
test taper_divergence_matches_analytic_bound ... ok
test layout_parameter_and_order_guards ... ok
test pit_lane_can_wrap_across_start_finish ... ok
test box_row_places_stalls_and_garages ... ok
test junction_markings_come_from_the_components ... ok
test pit_road_can_bend_away_from_the_track ... ok
test bake_fails_on_a_bad_layout_without_writing_a_lane ... ok
test valid_layout_compiles_and_passes_track_validation ... ok
test layout_guards_reject_bad_layouts_in_order ... ok
test junctions_follow_a_rescaled_main_spline ... ok
test pit_junction_wedges_are_not_paved_grip_islands ... ok
test bake_writes_the_compiled_lane_and_garages ... ok
test spec_120_validation_flags_stall_blocking_fast_lane ... ok
test spec_120_side_bay_surface_sampling_and_boundaries_taper_smoothly ... ok
test layout_free_tracks_keep_the_searched_junctions ... ok
test spec_120_compiled_stalls_sit_in_outer_side_bay_clear_of_fast_lane ... ok
test loaded_junctions_and_divider_wall_come_from_the_components ... ok
test baked_layout_lane_is_fully_enclosed ... ok
test bake_is_deterministic_and_does_not_duplicate_garages ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.77s


running 6 tests
test test_bots_do_not_stall_on_gt_circuits ... ignored
test test_pit_limiter_speed_clamping ... ok
test test_bot_pits_on_every_layout_circuit ... ok
test test_passing_car_clears_stopped_car_in_pit_bay ... ok
test test_gt_pit_lanes_are_fully_enclosed ... ok
test test_all_gt_pit_lanes_have_outer_bay_stalls_and_pass_validation ... ok

test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 21.90s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/pit_kit_tests.rs (target/debug/deps/pit_kit_tests-aa550375e3be0dab)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/pit_lane_integration_tests.rs (target/debug/deps/pit_lane_integration_tests-bd67b105c9d5fbae)

```
