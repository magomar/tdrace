---
type: validation_receipt
schema_version: "1.0"
spec: "specs/050_fia_autocross_championship_and_vehicle_roster.md"
epic: "tdrace-8sx9"
candidate_commit: "0b191580585e28ef4dbbb993b9a9006aba5cb140"
verifier: "local-user"
evaluated_at: "2026-09-29T22:31:16Z"
command: "cargo test -p tdrace-app --test autocross_rules_tests --test autocross_catalog_tests --test autocross_circuits_tests"
exit_code: 0
duration_ms: 7496
status: passed
---

# 🧾 Validation Receipt: Spec 050

- **Candidate Commit**: `0b191580585e28ef4dbbb993b9a9006aba5cb140`
- **Spec**: `specs/050_fia_autocross_championship_and_vehicle_roster.md`
- **Command**: `cargo test -p tdrace-app --test autocross_rules_tests --test autocross_catalog_tests --test autocross_circuits_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 3 tests
test test_autocross_catalog_has_three_vehicles_per_tier ... ok
test test_autocross_catalog_has_fifteen_vehicles ... ok
test test_autocross_specific_vehicle_identities ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_seventeen_autocross_circuits_embedded_in_catalog ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.79s


running 4 tests
test test_autocross_driver_roster ... ok
test test_autocross_game_module_identity_and_properties ... ok
test test_autocross_supported_game_modes ... ok
test test_autocross_tracks_count_and_zero_joker_ruleset ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.64s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 9.3 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/autocross_catalog_tests.rs (target/debug/deps/autocross_catalog_tests-c384f847d62e6786)
     Running tests/autocross_circuits_tests.rs (target/debug/deps/autocross_circuits_tests-cfc899dfea8f9095)
     Running tests/autocross_rules_tests.rs (target/debug/deps/autocross_rules_tests-d5bd7da98c797745)

```
