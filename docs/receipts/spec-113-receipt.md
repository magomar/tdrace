---
type: validation_receipt
schema_version: "1.0"
spec: "specs/113_4wd_loose_surface_torque_split_differential_upgrades_and_turnin_dynamics.md"
epic: "tdrace-yjnu0"
candidate_commit: "3257a36ab49079052677dc91edb53b65d782375f"
verifier: "local-user"
evaluated_at: "2026-10-13T06:45:43Z"
command: "cargo test --test autocross_catalog_tests"
exit_code: 0
duration_ms: 73
status: passed
---

# 🧾 Validation Receipt: Spec 113

- **Candidate Commit**: `3257a36ab49079052677dc91edb53b65d782375f`
- **Spec**: `specs/113_4wd_loose_surface_torque_split_differential_upgrades_and_turnin_dynamics.md`
- **Command**: `cargo test --test autocross_catalog_tests`
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


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/autocross_catalog_tests.rs (target/debug/deps/autocross_catalog_tests-fc9b8b99c6b32751)

```
