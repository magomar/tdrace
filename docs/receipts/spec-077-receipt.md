---
type: validation_receipt
schema_version: "1.0"
spec: "specs/077_procedural_gt_circuit_pit_lanes_from_openstreetmap_survey_data.md"
epic: "tdrace-dkgt"
candidate_commit: "ef095e66f3abaa19b36c72fc7c6b0cb6064c6e44"
verifier: "local-user"
evaluated_at: "2026-10-03T20:27:49Z"
command: "cargo test -p tdrace-core --test gt_pit_lane_tests"
exit_code: 0
duration_ms: 1703
status: passed
---

# 🧾 Validation Receipt: Spec 077

- **Candidate Commit**: `ef095e66f3abaa19b36c72fc7c6b0cb6064c6e44`
- **Spec**: `specs/077_procedural_gt_circuit_pit_lanes_from_openstreetmap_survey_data.md`
- **Command**: `cargo test -p tdrace-core --test gt_pit_lane_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 1 test
test test_pilot_circuits_have_valid_baked_pit_lanes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 11.0 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/gt_pit_lane_tests.rs (target/debug/deps/gt_pit_lane_tests-ed72ccae9a7514c8)

```
