---
type: validation_receipt
schema_version: "1.0"
spec: "specs/080_curvatureaware_track_boundary_geometry_swallowtail_pinch_elimination_and_global_circuit_validation.md"
epic: "tdrace-vi1t"
candidate_commit: "bc6a1b4fedc6822dbbee49d3d257f8cbb36a0d01"
verifier: "local-user"
evaluated_at: "2026-10-03T16:09:08Z"
command: "cargo test -p tdrace-core --test official_catalog_tests test_no_embedded_circuit_has_validation_errors"
exit_code: 0
duration_ms: 33161
status: passed
---

# 🧾 Validation Receipt: Spec 080

- **Candidate Commit**: `bc6a1b4fedc6822dbbee49d3d257f8cbb36a0d01`
- **Spec**: `specs/080_curvatureaware_track_boundary_geometry_swallowtail_pinch_elimination_and_global_circuit_validation.md`
- **Command**: `cargo test -p tdrace-core --test official_catalog_tests test_no_embedded_circuit_has_validation_errors`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 1 test
test test_no_embedded_circuit_has_validation_errors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 33.12s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 11.0 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/official_catalog_tests.rs (target/debug/deps/official_catalog_tests-92b19b2429c9fbe1)

```
