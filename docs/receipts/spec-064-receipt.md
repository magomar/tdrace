---
type: validation_receipt
schema_version: "1.0"
spec: "specs/064_classic_module_multitier_academy_missions_and_degradation_curriculum.md"
epic: "tdrace-xyyf"
candidate_commit: "1d061884c77af5cf5e49c1c5fd5de44256b83351"
verifier: "agent:claude-sonnet-5-5/cargo"
evaluated_at: "2026-10-11T09:36:28Z"
command: "cargo test -q -p wheelbase -p arcade-race-core -p tdrace-app --test decoupled_tire_physics_tests --test profile_tests --test career_onboarding_tests --test academy_calibration_tests --test car_damage_settings_tests --test codex_export_tests > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-nervous-easley-bfef60/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt-run.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-nervous-easley-bfef60/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt-run.log | tail -20; exit $rc"
exit_code: 0
duration_ms: 121035
status: passed
---

# 🧾 Validation Receipt: Spec 064

- **Candidate Commit**: `1d061884c77af5cf5e49c1c5fd5de44256b83351`
- **Spec**: `specs/064_classic_module_multitier_academy_missions_and_degradation_curriculum.md`
- **Command**: `cargo test -q -p wheelbase -p arcade-race-core -p tdrace-app --test decoupled_tire_physics_tests --test profile_tests --test career_onboarding_tests --test academy_calibration_tests --test car_damage_settings_tests --test codex_export_tests > /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-nervous-easley-bfef60/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt-run.log 2>&1; rc=$?; grep -E 'test result|FAILED|panicked' /private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-nervous-easley-bfef60/e462500d-1223-48d3-8fc2-f4f2698455b4/scratchpad/receipt-run.log | tail -20; exit $rc`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 13.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.12s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 57.88s
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.36s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

stderr:

```
