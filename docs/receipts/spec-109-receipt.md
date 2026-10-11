---
type: validation_receipt
schema_version: "1.0"
spec: "specs/109_classic_academy_tier_1_medal_target_recalibration.md"
epic: "tdrace-qe6mb"
candidate_commit: "5466070cdcbc6c2c80b5b6a97e7b5a9ea8f63337"
verifier: "agent:claude-sonnet-5-5/cargo"
evaluated_at: "2026-10-11T15:47:52Z"
command: "cargo test -q -p arcade-race-core -p tdrace-app --test profile_tests --test career_onboarding_tests --test academy_calibration_tests --test codex_export_tests --test modality_flow_tests > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt109.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt109.log | tail -20; exit $rc"
exit_code: 0
duration_ms: 134941
status: passed
---

# 🧾 Validation Receipt: Spec 109

- **Candidate Commit**: `5466070cdcbc6c2c80b5b6a97e7b5a9ea8f63337`
- **Spec**: `specs/109_classic_academy_tier_1_medal_target_recalibration.md`
- **Command**: `cargo test -q -p arcade-race-core -p tdrace-app --test profile_tests --test career_onboarding_tests --test academy_calibration_tests --test codex_export_tests --test modality_flow_tests > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt109.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt109.log | tail -20; exit $rc`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 13.42s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.04s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 57.92s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.98s
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.37s

stderr:

```
