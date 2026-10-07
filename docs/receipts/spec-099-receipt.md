---
type: validation_receipt
schema_version: "1.0"
spec: "specs/099_packed_and_deep_gravel_surfaces_and_asphaltcircuit_gravel_traps.md"
epic: "tdrace-fp0x"
candidate_commit: "0d3e47fbbc58c06164211bc841612a160dc6d714"
verifier: "local-user"
evaluated_at: "2026-10-07T18:39:31Z"
command: "cargo test --release -p tdrace-app --test gravel_surfaces_tests"
exit_code: 0
duration_ms: 702
status: passed
---

# 🧾 Validation Receipt: Spec 099

- **Candidate Commit**: `0d3e47fbbc58c06164211bc841612a160dc6d714`
- **Spec**: `specs/099_packed_and_deep_gravel_surfaces_and_asphaltcircuit_gravel_traps.md`
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

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 14.3 MB (target <= 8 MB)
    Finished `release` profile [optimized] target(s) in 0.15s
     Running tests/gravel_surfaces_tests.rs (target/release/deps/gravel_surfaces_tests-635e2cd500eb2b5f)

```
