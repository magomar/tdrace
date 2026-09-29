---
type: validation_receipt
schema_version: "1.0"
spec: "specs/049_reusable_racing_platform_layers.md"
epic: "tdrace-c36o"
candidate_commit: "3bbb3c4af29ddb70ef251ad5285bbf6ea7ae7d5b"
verifier: "local-user"
evaluated_at: "2026-09-28T20:48:49Z"
command: "cargo test -p arcade-race-core --test golden_sim && cargo test -p tdrace-app --test golden_session"
exit_code: 0
duration_ms: 18614
status: passed
---

# 🧾 Validation Receipt: Spec 049

- **Candidate Commit**: `3bbb3c4af29ddb70ef251ad5285bbf6ea7ae7d5b`
- **Spec**: `specs/049_reusable_racing_platform_layers.md`
- **Command**: `cargo test -p arcade-race-core --test golden_sim && cargo test -p tdrace-app --test golden_session`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.56s


running 1 test
test golden_session ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-e957cda816cb8377)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/golden_session.rs (target/debug/deps/golden_session-055f65bcce72b3fa)

```
