---
type: validation_receipt
schema_version: "1.0"
spec: "specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md"
epic: "tdrace-ylew"
candidate_commit: "a17537ea2e46acfd19671d812d1bd64793542a90"
verifier: "agent:claude-opus-5-5"
evaluated_at: "2026-09-28T01:05:26Z"
command: "cargo test -q -p tdrace-core --test official_catalog_tests && cargo test -q -p tdrace-app --test official_catalog_tests && cargo test -q -p arcade-race-core bake && .venv/bin/pytest -q tests/python/test_official_tracks.py tests/python/test_osm_importer.py"
exit_code: 0
duration_ms: 25344
status: passed
---

# 🧾 Validation Receipt: Spec 042

- **Candidate Commit**: `a17537ea2e46acfd19671d812d1bd64793542a90`
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
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.59s


running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.30s


running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.47s

......................                                                   [100%]
22 passed in 0.61s

stderr:

```
