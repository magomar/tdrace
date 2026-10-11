---
type: validation_receipt
schema_version: "1.0"
spec: "specs/111_racing_number_placement_by_car_type.md"
epic: "tdrace-9lbqd"
candidate_commit: "aafbb472c0c49e69ce16b41481d83862419311f9"
verifier: "agent:claude-opus-5.5"
evaluated_at: "2026-10-11T21:31:53Z"
command: "cargo test -p tdrace-app --test racing_number_render_tests --test racing_number_tests > /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt111.log 2>&1 && grep '^test result' /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt111.log"
exit_code: 0
duration_ms: 52692
status: passed
---

# 🧾 Validation Receipt: Spec 111

- **Candidate Commit**: `aafbb472c0c49e69ce16b41481d83862419311f9`
- **Spec**: `specs/111_racing_number_placement_by_car_type.md`
- **Command**: `cargo test -p tdrace-app --test racing_number_render_tests --test racing_number_tests > /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt111.log 2>&1 && grep '^test result' /var/folders/15/hxtsrdp976v59cpvrmt_1db40000gn/T//receipt111.log`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.63s

stderr:

```
