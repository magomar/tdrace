---
type: validation_receipt
schema_version: "1.0"
spec: "specs/083_evorl_bot_driving_style_and_performance_optimization.md"
epic: "tdrace-eow3"
candidate_commit: "740aabb8e6154eea21a8398986b01a39f500e2dd"
verifier: "local-user"
evaluated_at: "2026-10-16T16:20:07Z"
command: "cargo test -p tdrace-app --release --test bot_tier_ladder_tests && cargo test -p tdrace-app --test bot_humanlike_driving_tests && cargo test -p tdrace-app --test classic_circuits_bot_tests -- test_a_bot_with_its_nose_on_a_wall_reverses_off_it"
exit_code: 0
duration_ms: 60088
status: passed
---

# 🧾 Validation Receipt: Spec 083

- **Candidate Commit**: `740aabb8e6154eea21a8398986b01a39f500e2dd`
- **Spec**: `specs/083_evorl_bot_driving_style_and_performance_optimization.md`
- **Command**: `cargo test -p tdrace-app --release --test bot_tier_ladder_tests && cargo test -p tdrace-app --test bot_humanlike_driving_tests && cargo test -p tdrace-app --test classic_circuits_bot_tests -- test_a_bot_with_its_nose_on_a_wall_reverses_off_it`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 1 test
test test_tier_pace_ladder_holds_on_every_sample_track ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s


running 8 tests
test test_same_seed_gives_the_same_race ... ok
test test_bots_do_not_drive_the_same_path_every_lap ... ok
test test_human_layer_off_equals_pre_046_controller ... ok
test test_mistakes_follow_the_tier ... ok
test test_no_bot_gets_stuck ... ok
test test_tier_1_is_relatively_easy_to_beat ... ok
test test_bots_keep_their_driving_style ... ok
test test_keyboard_reference_driver_beats_tier_1 ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 57.48s


running 1 test
test test_a_bot_with_its_nose_on_a_wall_reverses_off_it ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.02s


stderr:
    Finished `release` profile [optimized] target(s) in 0.05s
     Running tests/bot_tier_ladder_tests.rs (target/release/deps/bot_tier_ladder_tests-b7b8c5f407fa4986)
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.40s
     Running tests/bot_humanlike_driving_tests.rs (target/debug/deps/bot_humanlike_driving_tests-8f883892ad128efe)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/classic_circuits_bot_tests.rs (target/debug/deps/classic_circuits_bot_tests-4a692c5d6898b82b)

```
