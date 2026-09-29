---
type: validation_receipt
schema_version: "1.0"
spec: "specs/059_publishready_shared_crates.md"
epic: "tdrace-aips"
candidate_commit: "191a52bb64f430c46ec535ef4847c443b01f2bb2"
verifier: "local-user"
evaluated_at: "2026-09-29T15:00:15Z"
command: "cargo test -p cabinet --test app_id_tests && cargo test -p race-kit --doc && cargo build -p race-ui --example minimal_race && cargo test -p race-kit --test golden_world"
exit_code: 0
duration_ms: 16989
status: passed
---

# 🧾 Validation Receipt: Spec 059

- **Candidate Commit**: `191a52bb64f430c46ec535ef4847c443b01f2bb2`
- **Spec**: `specs/059_publishready_shared_crates.md`
- **Command**: `cargo test -p cabinet --test app_id_tests && cargo test -p race-kit --doc && cargo build -p race-ui --example minimal_race && cargo test -p race-kit --test golden_world`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 1 test
test profile_paths_follow_the_app_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test crates/race-kit/src/lib.rs - ReadmeDoctests (line 39) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.47s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running tests/app_id_tests.rs (target/debug/deps/app_id_tests-6eb3bc18370c236b)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
   Doc-tests race_kit
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/golden_world.rs (target/debug/deps/golden_world-0d88c7283aaceb24)

```
