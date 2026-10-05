---
type: validation_receipt
schema_version: "1.0"
spec: "specs/089_packed_and_deep_gravel_surfaces_and_asphaltcircuit_gravel_traps.md"
epic: "tdrace-fp0x"
candidate_commit: "7c67ecb8765c7c927b0970bfeb9b7ec8b11cc1ff"
verifier: "local-user"
evaluated_at: "2026-10-05T07:50:49Z"
command: "cargo test --release -p tdrace-app --test gravel_surfaces_tests"
exit_code: 0
duration_ms: 576
status: passed
---

# 🧾 Validation Receipt: Spec 089

- **Candidate Commit**: `7c67ecb8765c7c927b0970bfeb9b7ec8b11cc1ff`
- **Spec**: `specs/089_packed_and_deep_gravel_surfaces_and_asphaltcircuit_gravel_traps.md`
- **Command**: `cargo test --release -p tdrace-app --test gravel_surfaces_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
test test_old_gravel_name_loads_as_packed_gravel_and_saves_with_the_new_name ... ok
test test_packed_gravel_grip_is_85_to_95_percent_of_dirt ... ok
test test_deep_gravel_stops_a_coasting_car_in_60_percent_of_its_asphalt_distance ... ok
test test_deep_gravel_never_traps_a_stopped_car ... ok
test test_official_circuits_use_packed_and_deep_gravel ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 12.0 MB (target <= 8 MB)
    Finished `release` profile [optimized] target(s) in 0.12s
     Running tests/gravel_surfaces_tests.rs (target/release/deps/gravel_surfaces_tests-a1b12fbc458b9a5e)

```
