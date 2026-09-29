---
type: validation_receipt
schema_version: "1.0"
spec: "specs/056_racekit_headless_race_world.md"
epic: "tdrace-vw2o"
candidate_commit: "2eb709f9798c211bf4064f8d2947114da332d34d"
verifier: "local-user"
evaluated_at: "2026-09-29T09:28:51Z"
command: "cargo test -p race-kit && cargo test -p arcade-race-core --test golden_sim && cargo test -p tdrace-app --test golden_session --test race_results_tests --test hall_of_fame_tests --test bot_curb_cutting_tests"
exit_code: 0
duration_ms: 39507
status: passed
---

# 🧾 Validation Receipt: Spec 056

- **Candidate Commit**: `2eb709f9798c211bf4064f8d2947114da332d34d`
- **Spec**: `specs/056_racekit_headless_race_world.md`
- **Command**: `cargo test -p race-kit && cargo test -p arcade-race-core --test golden_sim && cargo test -p tdrace-app --test golden_session --test race_results_tests --test hall_of_fame_tests --test bot_curb_cutting_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.54s


running 2 tests
test wreck_gives_dnf_and_stays_collidable ... ok
test laps_race_finishes_with_real_times ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test golden_oval ... ok
test golden_figure_eight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.67s


running 2 tests
test test_no_weaving_on_straight ... ok
test test_tier_5_cuts_apex_curbs ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s


running 1 test
test golden_session ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s


running 10 tests
test test_hall_of_fame_in_memory_db_schema_and_empty_top10 ... ok
test test_hall_of_fame_clean_start_and_clear ... ok
test test_hall_of_fame_track_isolation ... ok
test test_hall_of_fame_insertion_and_ordering ... ok
test test_hall_of_fame_top10_cutoff_and_qualification ... ok
test test_hall_of_fame_clear_track_history_isolation ... ok
test test_clear_bot_hall_of_fame_preserves_human_records ... ok
test test_race_session_hof_automatic_logging_and_congratulations ... ok
test test_race_session_circuit_history_cleared_on_editor_modify ... ok
test test_race_session_save_new_circuit_does_not_clear_other_tracks ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s


running 1 test
test results_use_world_finish_and_projected_times ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/race_kit-9fed28fef58dadf9)
     Running tests/golden_world.rs (target/debug/deps/golden_world-0d88c7283aaceb24)
     Running tests/world_tests.rs (target/debug/deps/world_tests-95d12e6eb734e685)
   Doc-tests race_kit
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/golden_sim.rs (target/debug/deps/golden_sim-e957cda816cb8377)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running tests/bot_curb_cutting_tests.rs (target/debug/deps/bot_curb_cutting_tests-b4ea5a85e5415565)
     Running tests/golden_session.rs (target/debug/deps/golden_session-ddfe37e961063054)
     Running tests/hall_of_fame_tests.rs (target/debug/deps/hall_of_fame_tests-39de08a32e504be4)
     Running tests/race_results_tests.rs (target/debug/deps/race_results_tests-3a382d6f48bd8602)

```
