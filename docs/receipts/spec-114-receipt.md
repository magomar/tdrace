---
type: validation_receipt
schema_version: "1.0"
spec: "specs/114_realworld_homologation_telemetry_calibration_for_autocross_and_rallycross_highpower_4wd_drivetrains_competition_braking_and_mechanical_grip.md"
epic: "tdrace-ibjhr"
candidate_commit: "3b472ec469edcc2087c91d5c6c30809ce19f8012"
verifier: "local-user"
evaluated_at: "2026-10-13T09:06:03Z"
command: "cargo test -p tdrace-app --test autocross_catalog_tests && cargo test -p tdrace-app --test rallycross_and_autocross_simulation_tests -- test_calibrated"
exit_code: 0
duration_ms: 65510
status: passed
---

# 🧾 Validation Receipt: Spec 114

- **Candidate Commit**: `3b472ec469edcc2087c91d5c6c30809ce19f8012`
- **Spec**: `specs/114_realworld_homologation_telemetry_calibration_for_autocross_and_rallycross_highpower_4wd_drivetrains_competition_braking_and_mechanical_grip.md`
- **Command**: `cargo test -p tdrace-app --test autocross_catalog_tests && cargo test -p tdrace-app --test rallycross_and_autocross_simulation_tests -- test_calibrated`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 7 tests
test test_autocross_4wd_loose_surface_balance_and_differentials ... ok
test test_autocross_catalog_has_three_vehicles_per_tier ... ok
test test_autocross_catalog_has_fifteen_vehicles ... ok
test test_autocross_specific_vehicle_identities ... ok
test test_carchoice_direct_instantiation_parity ... ok
test test_autocross_buggy1600_turning_circle_diameter_and_differential_action ... ok
test test_autocross_tier3_buggy1600_sweeper_turnin_and_understeer_elimination ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test test_calibrated_autocross_multi_circuit ... ok
test test_calibrated_rallycross_multi_circuit has been running for over 60 seconds
test test_calibrated_rallycross_multi_circuit ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 65.38s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/autocross_catalog_tests.rs (target/debug/deps/autocross_catalog_tests-6bc6471398b6b09f)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/rallycross_and_autocross_simulation_tests.rs (target/debug/deps/rallycross_and_autocross_simulation_tests-4d9d1dfaca4535c3)

```
