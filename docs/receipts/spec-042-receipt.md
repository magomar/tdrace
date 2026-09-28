---
type: validation_receipt
schema_version: "1.0"
spec: "specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md"
epic: "tdrace-ylew"
candidate_commit: "5c8556e6b04834f10fdb7f68e128b71dd182fb78"
verifier: "agent:claude-opus-5-5"
evaluated_at: "2026-09-28T06:01:05Z"
command: "cargo test -q -p tdrace-core --test official_catalog_tests && cargo test -q -p tdrace-app --test official_catalog_tests && cargo test -q -p arcade-race-core bake && .venv/bin/pytest -q tests/python/test_official_tracks.py tests/python/test_osm_importer.py"
exit_code: 0
duration_ms: 28078
status: passed
---

# 🧾 Validation Receipt: Spec 042

- **Candidate Commit**: `5c8556e6b04834f10fdb7f68e128b71dd182fb78`
- **Spec**: `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md`
- **Command**: `cargo test -q -p tdrace-core --test official_catalog_tests && cargo test -q -p tdrace-app --test official_catalog_tests && cargo test -q -p arcade-race-core bake && .venv/bin/pytest -q tests/python/test_official_tracks.py tests/python/test_osm_importer.py`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.80s


running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.37s


running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.47s

......................                                                   [100%]
22 passed in 0.73s

stderr:

```
