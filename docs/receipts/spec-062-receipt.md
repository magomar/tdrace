---
type: validation_receipt
schema_version: "1.0"
spec: "specs/062_circuit_pit_lanes_and_interactive_pit_stop_procedures.md"
epic: "tdrace-q01g"
candidate_commit: "b2a295302ffa15973b41405cdbe3b1f437285295"
verifier: "local-user"
evaluated_at: "2026-10-02T18:25:28Z"
command: "cargo test -p arcade-race-core test_pit_lane && cargo test -p race-kit --test world_tests -- test_pit_service_state_machine && cargo test -p race-kit --test bot_vehicle_tests -- test_bot_ai_pit_tactics_and_stall_stopping && cargo test -p tdrace-app --test pit_lane_integration_tests -- test_pit_limiter_speed_clamping && cargo test -p tdrace-app --test track_editor_tests -- test_editor_pit_lane_tool_spline_and_box_placement"
exit_code: 0
duration_ms: 1195
status: passed
---

# 🧾 Validation Receipt: Spec 062

- **Candidate Commit**: `b2a295302ffa15973b41405cdbe3b1f437285295`
- **Spec**: `specs/062_circuit_pit_lanes_and_interactive_pit_stop_procedures.md`
- **Command**: `cargo test -p arcade-race-core test_pit_lane && cargo test -p race-kit --test world_tests -- test_pit_service_state_machine && cargo test -p race-kit --test bot_vehicle_tests -- test_bot_ai_pit_tactics_and_stall_stopping && cargo test -p tdrace-app --test pit_lane_integration_tests -- test_pit_limiter_speed_clamping && cargo test -p tdrace-app --test track_editor_tests -- test_editor_pit_lane_tool_spline_and_box_placement`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 2 tests
test track::checkpoint::tests::test_pit_lane_anti_cut_stops_only_on_service ... ok
test track::geometry::tests::test_pit_lane_geometry ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s


running 1 test
test test_pit_service_state_machine ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.30s


running 1 test
test test_bot_ai_pit_tactics_and_stall_stopping ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.21s


running 1 test
test test_pit_limiter_speed_clamping ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s


running 1 test
test test_editor_pit_lane_tool_spline_and_box_placement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.37s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/arcade_race_core-bd2d3fb9efdf928f)
     Running tests/body2d_tests.rs (target/debug/deps/body2d_tests-b44fffeaa1d11b10)
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-262de59c3d016fc3)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-4f6b512e9b8ff5d1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/world_tests.rs (target/debug/deps/world_tests-04ba45d71dcccb27)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/bot_vehicle_tests.rs (target/debug/deps/bot_vehicle_tests-8c477fe464b35a37)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/pit_lane_integration_tests.rs (target/debug/deps/pit_lane_integration_tests-6002ba664b7cad4b)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/track_editor_tests.rs (target/debug/deps/track_editor_tests-e0e357154a23109e)

```
