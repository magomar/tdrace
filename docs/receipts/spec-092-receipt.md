---
type: validation_receipt
schema_version: "1.0"
spec: "specs/092_player_racing_number_customization_and_persistent_nonconflicting_bot_rosters.md"
epic: "tdrace-xa7q"
candidate_commit: "ffb41a56c721ccc364df84fe6c6f929df60ac401"
verifier: "agent:claude-sonnet-5.5"
evaluated_at: "2026-10-11T09:20:39Z"
command: "cargo test -p tdrace-app --lib racing_number > /private/tmp/claude-501/receipt092.log 2>&1 && cargo test -p tdrace-app --test racing_number_tests --test racing_number_render_tests >> /private/tmp/claude-501/receipt092.log 2>&1 && grep '^test result' /private/tmp/claude-501/receipt092.log"
exit_code: 0
duration_ms: 51695
status: passed
---

# 🧾 Validation Receipt: Spec 092

- **Candidate Commit**: `ffb41a56c721ccc364df84fe6c6f929df60ac401`
- **Spec**: `specs/092_player_racing_number_customization_and_persistent_nonconflicting_bot_rosters.md`
- **Command**: `cargo test -p tdrace-app --lib racing_number > /private/tmp/claude-501/receipt092.log 2>&1 && cargo test -p tdrace-app --test racing_number_tests --test racing_number_render_tests >> /private/tmp/claude-501/receipt092.log 2>&1 && grep '^test result' /private/tmp/claude-501/receipt092.log`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 84 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.50s

stderr:

```
