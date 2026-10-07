---
type: validation_receipt
schema_version: "1.0"
spec: "specs/094_modality_chassis_platforms_architecture_expansion_and_arcade_alignment.md"
epic: "tdrace-wpn0"
candidate_commit: "a8122ab165f39b052a4209215f99254f404abf26"
verifier: "local-user"
evaluated_at: "2026-10-06T10:49:20Z"
command: "cargo test -q -p wheelbase && cargo test -q -p tdrace-app --test handling_presets_tests --test random_car_assignment_tests --test roster_race_setup_tests --test render_tests --test codex_export_tests"
exit_code: 0
duration_ms: 34575
status: passed
---

# 🧾 Validation Receipt: Spec 094

- **Candidate Commit**: `a8122ab165f39b052a4209215f99254f404abf26`
- **Spec**: `specs/094_modality_chassis_platforms_architecture_expansion_and_arcade_alignment.md`
- **Command**: `cargo test -q -p wheelbase && cargo test -q -p tdrace-app --test handling_presets_tests --test random_car_assignment_tests --test roster_race_setup_tests --test render_tests --test codex_export_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 62 tests
..............................................................
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 13 tests
.............
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 16 tests
................
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.39s


running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.21s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s


running 51 tests
...................................................
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.70s


running 18 tests
..................
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.61s


stderr:

```
