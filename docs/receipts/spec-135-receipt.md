---
type: validation_receipt
schema_version: "1.0"
spec: "specs/135_rallycross_tier_6_nitrocross_championship_isolation_world_rx_tier_4_5_realignment_and_tier_0_group_b_vault_archival.md"
epic: "tdrace-sp135-iyr2w"
candidate_commit: "a50c8c44c150db82086f522a32ffe07a72276671"
verifier: "local-user"
evaluated_at: "2026-10-22T09:57:36Z"
command: "cargo test -p tdrace-app --test module_track_unlock_tests && cargo test -p tdrace-app --test vault_module_tests"
exit_code: 0
duration_ms: 3819
status: passed
---

# 🧾 Validation Receipt: Spec 135

- **Candidate Commit**: `a50c8c44c150db82086f522a32ffe07a72276671`
- **Spec**: `specs/135_rallycross_tier_6_nitrocross_championship_isolation_world_rx_tier_4_5_realignment_and_tier_0_group_b_vault_archival.md`
- **Command**: `cargo test -p tdrace-app --test module_track_unlock_tests && cargo test -p tdrace-app --test vault_module_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
test test_gt_madring_unlocks_at_tier_five ... ok
test test_rally_six_tier_circuit_unlock_matrix ... ok
test test_every_module_locks_circuits_above_tier_one ... ok
test test_every_module_circuit_unlocks_by_max_level ... ok
test test_session_locks_circuits_in_every_module_but_classic ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.75s


running 4 tests
test test_vault_game_module_identity_and_traits ... ok
test test_vault_track_manager_filter_and_manifest_integrity ... ok
test test_vault_career_and_driver_isolation ... ok
test test_vault_grand_hub_dev_mode_gating_and_switching ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s


stderr:
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.39s
     Running tests/module_track_unlock_tests.rs (target/debug/deps/module_track_unlock_tests-720be127722fea27)
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.39s
     Running tests/vault_module_tests.rs (target/debug/deps/vault_module_tests-51bb3cc3d6f60f6b)

```
