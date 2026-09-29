---
type: validation_receipt
schema_version: "1.0"
spec: "specs/050_body2d_trait_for_vehiclegeneric_collision_and_progress.md"
epic: "tdrace-r73j"
candidate_commit: "66ddb61630d67010afa41697aff0ff7b1e1b15f2"
verifier: "local-user"
evaluated_at: "2026-09-29T07:33:15Z"
command: "cargo test -p arcade-race-core --test body2d_tests --test golden_sim && cargo test -p tdrace-app --test golden_session"
exit_code: 0
duration_ms: 19041
status: passed
---

# 🧾 Validation Receipt: Spec 050

- **Candidate Commit**: `66ddb61630d67010afa41697aff0ff7b1e1b15f2`
- **Spec**: `specs/050_body2d_trait_for_vehiclegeneric_collision_and_progress.md`
- **Command**: `cargo test -p arcade-race-core --test body2d_tests --test golden_sim && cargo test -p tdrace-app --test golden_session`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 3 tests
test long_body_hits_wall_beyond_old_reach ... ok
test two_bodies_collide_and_conserve_momentum ... ok
test tracker_follows_a_non_car_body ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.76s


running 1 test
test golden_session ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.98s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/body2d_tests.rs (target/debug/deps/body2d_tests-ff5cd0ec0e29746d)
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-e957cda816cb8377)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/golden_session.rs (target/debug/deps/golden_session-055f65bcce72b3fa)

```
