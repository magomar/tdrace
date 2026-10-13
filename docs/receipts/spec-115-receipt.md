---
type: validation_receipt
schema_version: "1.0"
spec: "specs/115_lowgrip_runoff_bot_recovery_and_wall_scraping_resistance_calibration.md"
epic: "tdrace-mbfz4"
candidate_commit: "2ad9903bab5d316bdceee411ab605e09398c3b19"
verifier: "local-user"
evaluated_at: "2026-10-13T12:37:24Z"
command: "cargo test -p tdrace-app --test wall_contact_escape_tests && cargo test -p tdrace-app --test launch_chute_race_tests test_every_bot_finishes_three_laps_on_kouvola"
exit_code: 0
duration_ms: 85718
status: passed
---

# 🧾 Validation Receipt: Spec 115

- **Candidate Commit**: `2ad9903bab5d316bdceee411ab605e09398c3b19`
- **Spec**: `specs/115_lowgrip_runoff_bot_recovery_and_wall_scraping_resistance_calibration.md`
- **Command**: `cargo test -p tdrace-app --test wall_contact_escape_tests && cargo test -p tdrace-app --test launch_chute_race_tests test_every_bot_finishes_three_laps_on_kouvola`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 2 tests
test test_car_against_the_kouvola_joker_tyre_wall_drives_away ... ok
test test_bot_at_kouvola_joker_wall ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.06s


running 1 test
test test_every_bot_finishes_three_laps_on_kouvola has been running for over 60 seconds
test test_every_bot_finishes_three_laps_on_kouvola ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 82.53s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/wall_contact_escape_tests.rs (target/debug/deps/wall_contact_escape_tests-9e32a58b48d80612)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/launch_chute_race_tests.rs (target/debug/deps/launch_chute_race_tests-38326a1401d0d19f)

```
