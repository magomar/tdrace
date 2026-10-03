---
type: validation_receipt
schema_version: "1.0"
spec: "specs/006_road_split_and_branching_tracks.md"
epic: "tdrace-road-split-branching-tracks-kjl6"
candidate_commit: "36a86910e29d313d1223a812e65fab2c3be6b0ac"
verifier: "local-user"
evaluated_at: "2026-10-03T14:19:57Z"
command: "cargo test -p arcade-race-core track::network && cargo test -p arcade-race-core --test multi_route_progress_tests && cargo test -p tdrace-app --test track_editor_tests test_road_split"
exit_code: 0
duration_ms: 1992
status: passed
---

# 🧾 Validation Receipt: Spec 006

- **Candidate Commit**: `36a86910e29d313d1223a812e65fab2c3be6b0ac`
- **Spec**: `specs/006_road_split_and_branching_tracks.md`
- **Command**: `cargo test -p arcade-race-core track::network && cargo test -p arcade-race-core --test multi_route_progress_tests && cargo test -p tdrace-app --test track_editor_tests test_road_split`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
test track::network::tests::test_spline_socket_ghost_point_and_c1_continuity ... ok
test track::network::tests::test_split_width_envelope_hermite_widening ... ok
test track::network::tests::test_road_segment_resampling_and_projection ... ok
test track::network::tests::test_c1_tangent_alignment_at_branch_split_socket ... ok
test track::network::tests::test_track_network_auto_promotion_from_single_spline ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s


running 3 tests
test test_track_progress_tracker_update_network_sync ... ok
test test_wrong_way_detection_on_divergent_branch ... ok
test test_multi_car_free_choice_routes_and_joker_counting ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_road_split_insert_split_junction_and_branch_seeding ... ok
test test_road_split_snap_to_merge_and_layout_generation ... ok
test test_road_split_append_waypoints_and_extend_branch ... ok
test test_road_split_wall_trimming_and_zero_collision ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 1.84s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-d6affea463a145ec)
     Running tests/body2d_tests.rs (target/debug/deps/body2d_tests-4e6ecb04cdaac332)
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-76c8531f0ba4e71f)
     Running tests/multi_route_progress_tests.rs (target/debug/deps/multi_route_progress_tests-98dd4ba329b2d5d6)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-387c71aed6dc27e1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/multi_route_progress_tests.rs (target/debug/deps/multi_route_progress_tests-98dd4ba329b2d5d6)
warning: tdrace-core@0.1.0: embedded official circuits are 8.7 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/track_editor_tests.rs (target/debug/deps/track_editor_tests-27bdb0108758b047)

```
