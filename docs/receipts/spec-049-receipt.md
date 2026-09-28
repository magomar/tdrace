---
type: validation_receipt
schema_version: "1.0"
spec: "specs/049_reusable_racing_platform_layers.md"
epic: "tdrace-c36o"
candidate_commit: "04a5347b8fd16d5e74cff3813926d898b3c76dd5"
verifier: "local-user"
evaluated_at: "2026-09-28T20:47:37Z"
command: "cargo test -p arcade-race-core --test golden_sim && cargo test -p tdrace-app --test golden_session"
exit_code: 0
duration_ms: 18960
status: passed
---

# 🧾 Validation Receipt: Spec 049

- **Candidate Commit**: `04a5347b8fd16d5e74cff3813926d898b3c76dd5`
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

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.90s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-e957cda816cb8377)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/golden_session.rs (target/debug/deps/golden_session-055f65bcce72b3fa)

```
