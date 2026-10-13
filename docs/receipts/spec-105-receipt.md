---
type: validation_receipt
schema_version: "1.0"
spec: "specs/105_redrawing_the_23_official_rallycross_jokers_to_fit_junction_components.md"
epic: "tdrace-dic8l"
candidate_commit: "257e3b0733ec097f08b037807d54b8ef8c0f09d3"
verifier: "local-user"
evaluated_at: "2026-10-13T09:46:32Z"
command: "cargo test -p tdrace-app --test branch_junction_circuit_tests"
exit_code: 0
duration_ms: 75753
status: passed
---

# 🧾 Validation Receipt: Spec 105

- **Candidate Commit**: `257e3b0733ec097f08b037807d54b8ef8c0f09d3`
- **Spec**: `specs/105_redrawing_the_23_official_rallycross_jokers_to_fit_junction_components.md`
- **Command**: `cargo test -p tdrace-app --test branch_junction_circuit_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 7 tests
test test_branch_layout_survives_a_json_round_trip_and_track_studio ... ok
test test_migration_report_matches_the_circuits ... ok
test test_bots_take_the_joker_once_and_do_not_stall ... ok
test test_branch_circuits_keep_the_main_route has been running for over 60 seconds
test test_branch_circuits_validate_without_error_or_junction_warning has been running for over 60 seconds
test test_car_driven_at_the_nose_stops_on_it has been running for over 60 seconds
test test_car_driving_the_branch_route_gets_its_lap_and_its_joker has been running for over 60 seconds
test test_car_driven_at_the_nose_stops_on_it ... ok
test test_branch_circuits_keep_the_main_route ... ok
test test_branch_circuits_validate_without_error_or_junction_warning ... ok
test test_car_driving_the_branch_route_gets_its_lap_and_its_joker ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 75.69s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/branch_junction_circuit_tests.rs (target/debug/deps/branch_junction_circuit_tests-e595209e45c93993)

```
