---
type: validation_receipt
schema_version: "1.0"
spec: "specs/097_turnpreserving_nonlinear_circuit_rescaling_075x_scale_expansion_and_global_apex_smoothing.md"
epic: "tdrace-j9i0"
candidate_commit: "7fd0cee8698dd82384f7ed9543e4e92ff54c00a4"
verifier: "agent:claude-opus-5-5"
evaluated_at: "2026-10-08T14:13:03Z"
command: "cargo test -q --release -p tdrace-app --test gt_circuit_geometry_tests --test nascar_circuit_scale_tests --test track_manager_tests --test track_editor_tests --test kart_tracks_tests --test codex_export_tests -p tdrace-core --test gt_pit_lane_tests --test official_catalog_tests -p arcade-race-core --lib > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-circuit-building-approach-c6d2de/b4369581-83d1-4131-b76a-f3cad141f36a/scratchpad/receipt_run.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-circuit-building-approach-c6d2de/b4369581-83d1-4131-b76a-f3cad141f36a/scratchpad/receipt_run.log; exit $rc"
exit_code: 0
duration_ms: 89299
status: passed
---

# 🧾 Validation Receipt: Spec 097

- **Candidate Commit**: `7fd0cee8698dd82384f7ed9543e4e92ff54c00a4`
- **Spec**: `specs/097_turnpreserving_nonlinear_circuit_rescaling_075x_scale_expansion_and_global_apex_smoothing.md`
- **Command**: `cargo test -q --release -p tdrace-app --test gt_circuit_geometry_tests --test nascar_circuit_scale_tests --test track_manager_tests --test track_editor_tests --test kart_tracks_tests --test codex_export_tests -p tdrace-core --test gt_pit_lane_tests --test official_catalog_tests -p arcade-race-core --lib > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-circuit-building-approach-c6d2de/b4369581-83d1-4131-b76a-f3cad141f36a/scratchpad/receipt_run.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-circuit-building-approach-c6d2de/b4369581-83d1-4131-b76a-f3cad141f36a/scratchpad/receipt_run.log; exit $rc`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.87s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.94s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 58.34s

stderr:

```
