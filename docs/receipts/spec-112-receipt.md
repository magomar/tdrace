---
type: validation_receipt
schema_version: "1.0"
spec: "specs/112_junction_throat_wall_diagnostics_adaptive_median_barriers_and_runoff_consistency.md"
epic: "tdrace-fofx9"
candidate_commit: "cd04980d0d21dffa3486cc8cd2c1fd33bf977001"
verifier: "local-user"
evaluated_at: "2026-10-12T19:22:13Z"
command: "cargo test --package arcade-race-core --test track_geometry_tests && cargo test --package race-ui --test track_render_tests && cargo test --package tdrace-app --test track_editor_tests test_wall_suppression_diagnostics_on_road_conflict && cargo test --package tdrace-app --test rally_tracks_tests test_rx_main_road_is_walled_up_to_the_joker"
exit_code: 0
duration_ms: 30057
status: passed
---

# 🧾 Validation Receipt: Spec 112

- **Candidate Commit**: `cd04980d0d21dffa3486cc8cd2c1fd33bf977001`
- **Spec**: `specs/112_junction_throat_wall_diagnostics_adaptive_median_barriers_and_runoff_consistency.md`
- **Command**: `cargo test --package arcade-race-core --test track_geometry_tests && cargo test --package race-ui --test track_render_tests && cargo test --package tdrace-app --test track_editor_tests test_wall_suppression_diagnostics_on_road_conflict && cargo test --package tdrace-app --test rally_tracks_tests test_rx_main_road_is_walled_up_to_the_joker`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 3 tests
test test_adaptive_median_barrier_bypassed_if_angle_exceeds_60_degrees ... ok
test test_wall_suppression_record_properties ... ok
test test_adaptive_median_barrier_in_diverging_gore ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_runoff_shoulder_tapers_where_wall_suppressed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_wall_suppression_diagnostics_on_road_conflict ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 57 filtered out; finished in 2.40s


running 1 test
test test_rx_main_road_is_walled_up_to_the_joker ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 27.45s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/track_geometry_tests.rs (target/debug/deps/track_geometry_tests-c4fc2870beb4b8cb)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/track_render_tests.rs (target/debug/deps/track_render_tests-18f7c226bb457789)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/track_editor_tests.rs (target/debug/deps/track_editor_tests-906bde3eb5d3be34)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/rally_tracks_tests.rs (target/debug/deps/rally_tracks_tests-90b0909abd297b9e)

```
