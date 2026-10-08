---
type: validation_receipt
schema_version: "1.0"
spec: "specs/097_turnpreserving_nonlinear_circuit_rescaling_075x_scale_expansion_and_global_apex_smoothing.md"
epic: "tdrace-j9i0"
candidate_commit: "8b099b7eec36a3d5f72b1c1c0d363474be49d309"
verifier: "local-user"
evaluated_at: "2026-10-06T22:02:28Z"
command: "cargo test -p tdrace-app --test gt_circuit_geometry_tests"
exit_code: 0
duration_ms: 2957
status: passed
---

# 🧾 Validation Receipt: Spec 097

- **Candidate Commit**: `8b099b7eec36a3d5f72b1c1c0d363474be49d309`
- **Spec**: `specs/097_turnpreserving_nonlinear_circuit_rescaling_075x_scale_expansion_and_global_apex_smoothing.md`
- **Command**: `cargo test -p tdrace-app --test gt_circuit_geometry_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 2 tests
test test_gt_tracks_declare_three_quarter_scale ... ok
test test_gt_kerbs_only_on_real_corners ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.88s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 16.1 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/gt_circuit_geometry_tests.rs (target/debug/deps/gt_circuit_geometry_tests-00c6f8d9891fcdd9)

```
