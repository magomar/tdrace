---
type: validation_receipt
schema_version: "1.0"
spec: "specs/039_configurable_steering_smoothing_profiles_and_high_speed_turning_authority.md"
epic: "tdrace-x0kp"
candidate_commit: "f40bf665a6430d16961e8cf5a91c083378fcecc0"
verifier: "local-user"
evaluated_at: "2026-09-26T22:16:12Z"
command: "cargo test -p tdrace-app --test input_smoothing_tests && cargo test -p tdrace-app --test kart_steering_stability_tests"
exit_code: 0
duration_ms: 572
status: passed
---

# 🧾 Validation Receipt: Spec 039

- **Candidate Commit**: `f40bf665a6430d16961e8cf5a91c083378fcecc0`
- **Spec**: `specs/039_configurable_steering_smoothing_profiles_and_high_speed_turning_authority.md`
- **Command**: `cargo test -p tdrace-app --test input_smoothing_tests && cargo test -p tdrace-app --test kart_steering_stability_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 6 tests
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_non_linear_center_micro_corrections ... ok
test test_speed_sensitive_steering_scaling ... ok
test test_steering_profiles_configuration_and_cycling ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
test test_classic_sprint_kart_parameter_alignment ... ok
test test_digital_keyboard_progressive_steering_modulation ... ok
test test_front_steer_slip_angle_does_not_induce_engine_rev_flare ... ok
test test_high_speed_sustained_key_hold_achieves_full_turning_authority ... ok
test test_top_speed_governor_preserves_cornering_drive_thrust ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-f3d53b7ee5156480)
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running tests/kart_steering_stability_tests.rs (target/debug/deps/kart_steering_stability_tests-28d2ec7e94aff595)

```
