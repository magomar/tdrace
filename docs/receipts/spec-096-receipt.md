---
type: validation_receipt
schema_version: "1.0"
spec: "specs/096_chassisconforming_dynamic_vehicle_ground_shadows.md"
epic: "tdrace-xam4"
candidate_commit: "c1515d169a978918099ee4cf32ce7531eb88bca4"
verifier: "local-user"
evaluated_at: "2026-10-10T23:51:26Z"
command: "cargo test -q -p tdrace-app --test render_tests --test config_tests && cargo test -q -p cabinet --test cabinet_integration_tests"
exit_code: 0
duration_ms: 25307
status: passed
---

# 🧾 Validation Receipt: Spec 096

- **Candidate Commit**: `c1515d169a978918099ee4cf32ce7531eb88bca4`
- **Spec**: `specs/096_chassisconforming_dynamic_vehicle_ground_shadows.md`
- **Command**: `cargo test -q -p tdrace-app --test render_tests --test config_tests && cargo test -q -p cabinet --test cabinet_integration_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 18 tests
..................
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.26s


running 62 tests
..............................................................
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.71s


running 26 tests
..........................
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


stderr:

```
