---
type: validation_receipt
schema_version: "1.0"
spec: "specs/124_mega_jump_dual_wall_gap_trap_and_minimum_gap_sizing.md"
epic: "tdrace-sp124-oc0mh"
candidate_commit: "3dc73b12924325e2f3066f53ab7db02323ac4ec6"
verifier: "local-user"
evaluated_at: "2026-10-18T18:43:15Z"
command: "cargo test -p arcade-race-core --lib track::geometry::tests"
exit_code: 0
duration_ms: 970
status: passed
---

# 🧾 Validation Receipt: Spec 124

- **Candidate Commit**: `3dc73b12924325e2f3066f53ab7db02323ac4ec6`
- **Spec**: `specs/124_mega_jump_dual_wall_gap_trap_and_minimum_gap_sizing.md`
- **Command**: `cargo test -p arcade-race-core --lib track::geometry::tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 10 tests
test track::geometry::tests::test_barrier_type_virtual_properties ... ok
test track::geometry::tests::test_line_segment_intersection ... ok
test track::geometry::tests::test_jump_ramp_angle_and_size_manipulation ... ok
test track::geometry::tests::test_line_segment_projection_and_distance ... ok
test track::geometry::tests::test_ray_segment_intersection ... ok
test track::geometry::tests::test_jump_ramp_surface_and_flat_portion_fit ... ok
test track::geometry::tests::test_surface_shapes ... ok
test track::geometry::tests::test_whoops_moguls_and_mega_ramp_profiles_and_vehicle_interaction ... ok
test track::geometry::tests::test_pit_lane_geometry ... ok
test track::geometry::tests::test_mega_ramp_failed_gap_behaves_like_wall_collision ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out; finished in 0.00s


stderr:
   Compiling arcade-race-core v0.1.0 (/home/mario/workspace/games/tdrace/crates/arcade-race-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.97s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-bd2d3fb9efdf928f)

```
