---
type: validation_receipt
schema_version: "1.0"
spec: "specs/070_global_collinear_wall_optimization_and_circuit_rebake.md"
epic: "tdrace-ubkk"
candidate_commit: "e0457807bab44b6cd9f41596e94402e4579d25b8"
verifier: "local-user"
evaluated_at: "2026-09-30T14:26:00Z"
command: "cargo test -p tdrace-core --test official_catalog_tests"
exit_code: 0
duration_ms: 32000
status: passed
---

# 🧾 Validation Receipt: Spec 070

- **Candidate Commit**: `e0457807bab44b6cd9f41596e94402e4579d25b8`
- **Spec**: `specs/070_global_collinear_wall_optimization_and_circuit_rebake.md`
- **Command**: `cargo test -p tdrace-core --test official_catalog_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
warning: tdrace-core@0.1.0: embedded official circuits are 10.0 MB (target <= 8 MB)
   Compiling tdrace-core v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running tests/official_catalog_tests.rs (target/debug/deps/official_catalog_tests-92b19b2429c9fbe1)

running 5 tests
test test_catalog_counts_per_module ... ok
test test_aliases_and_module_hint ... ok
test test_every_embedded_circuit_equals_its_json_file ... ok
test test_no_embedded_circuit_has_validation_errors ... ok
test test_rebuild_reproduces_every_osm_circuit ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.89s
```
