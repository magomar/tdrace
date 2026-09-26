---
type: validation_receipt
schema_version: "1.0"
spec: "specs/038_keyboard_steering_smoothing_and_drivetrain_telemetry_isolation.md"
epic: "tdrace-whkv"
candidate_commit: "9f99f54ced5a14479989260e9937111475216ce9"
verifier: "local-user"
evaluated_at: "2026-09-26T15:45:47Z"
command: "cargo test -p tdrace-app --test kart_steering_stability_tests && cargo test -p tdrace-app --test input_smoothing_tests"
exit_code: 0
duration_ms: 158
status: passed
---

# 🧾 Validation Receipt: Spec 038

- **Candidate Commit**: `9f99f54ced5a14479989260e9937111475216ce9`
- **Spec**: `specs/038_keyboard_steering_smoothing_and_drivetrain_telemetry_isolation.md`
- **Command**: `cargo test -p tdrace-app --test kart_steering_stability_tests && cargo test -p tdrace-app --test input_smoothing_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 4 tests
test test_classic_sprint_kart_parameter_alignment ... ok
test test_digital_keyboard_progressive_steering_modulation ... ok
test test_front_steer_slip_angle_does_not_induce_engine_rev_flare ... ok
test test_top_speed_governor_preserves_cornering_drive_thrust ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_non_linear_center_micro_corrections ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_speed_sensitive_steering_scaling ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/kart_steering_stability_tests.rs (target/debug/deps/kart_steering_stability_tests-28d2ec7e94aff595)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-f3d53b7ee5156480)

```
