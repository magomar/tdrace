---
type: validation_receipt
schema_version: "1.0"
spec: "specs/083_evorl_bot_driving_style_and_performance_optimization.md"
epic: "tdrace-eow3"
candidate_commit: "8cdf64c183ca11bbe90af01b07f3d56c0f5525ce"
verifier: "local-user"
evaluated_at: "2026-10-17T20:28:56Z"
command: "cargo test --release -p tdrace-app --test bot_tier_ladder_tests --test bot_eval_worker_tests"
exit_code: 0
duration_ms: 76456
status: passed
---

# 🧾 Validation Receipt: Spec 083

- **Candidate Commit**: `8cdf64c183ca11bbe90af01b07f3d56c0f5525ce`
- **Spec**: `specs/083_evorl_bot_driving_style_and_performance_optimization.md`
- **Command**: `cargo test --release -p tdrace-app --test bot_tier_ladder_tests --test bot_eval_worker_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 8 tests
test test_genome_values_are_clamped_and_non_finite_values_fail ... ok
test test_empty_genome_gives_the_game_profile ... ok
test test_stage_a_genome_moves_every_style_by_the_same_change ... ok
test test_stage_b_genome_sets_the_tier_quality ... ok
test test_launch_circuits_cover_the_four_launch_modules ... ok
test test_worker_answers_bad_input_with_an_error_and_keeps_running ... ok
test test_worker_matches_the_rust_harness_and_is_deterministic ... ok
test test_all_modality_circuits_cover_all_seven_modalities ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.84s


running 1 test
test test_tier_pace_ladder_holds_on_every_sample_track ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s


stderr:
   Compiling race-kit v0.1.0 (/home/mario/workspace/games/tdrace/crates/race-kit)
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `release` profile [optimized] target(s) in 1m 12s
     Running tests/bot_eval_worker_tests.rs (target/release/deps/bot_eval_worker_tests-a593a1c2d71d55e0)
     Running tests/bot_tier_ladder_tests.rs (target/release/deps/bot_tier_ladder_tests-b7b8c5f407fa4986)

```
