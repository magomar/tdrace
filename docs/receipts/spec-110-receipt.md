---
type: validation_receipt
schema_version: "1.0"
spec: "specs/110_racing_number_placement_by_car_type.md"
epic: "tdrace-9lbqd"
candidate_commit: "776369a82937d33f038f4ec158c63e2feb83acd8"
verifier: "agent:claude-opus-5.5"
evaluated_at: "2026-10-11T20:39:08Z"
command: "cargo test -p tdrace-app --test racing_number_render_tests --test racing_number_tests > /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt110.log 2>&1 && grep '^test result' /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt110.log"
exit_code: 0
duration_ms: 23488
status: passed
---

# 🧾 Validation Receipt: Spec 110

- **Candidate Commit**: `776369a82937d33f038f4ec158c63e2feb83acd8`
- **Spec**: `specs/110_racing_number_placement_by_car_type.md`
- **Command**: `cargo test -p tdrace-app --test racing_number_render_tests --test racing_number_tests > /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt110.log 2>&1 && grep '^test result' /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt110.log`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.26s

stderr:

```
