---
type: validation_receipt
schema_version: "1.0"
spec: "specs/078_directional_impact_masking_engine_placement_damage_and_archetype_suspension_failure.md"
epic: "tdrace-kkzl"
candidate_commit: "3948cecf0fb154cbda7bec091b9e7003dcaf0b8a"
verifier: "local-user"
evaluated_at: "2026-10-03T07:14:09Z"
command: "cargo test -p wheelbase --test directional_damage_tests && cargo test -p race-kit --test world_tests && cargo test -p tdrace-app --test repair_invoice_tests"
exit_code: 0
duration_ms: 3037
status: passed
---

# 🧾 Validation Receipt: Spec 078

- **Candidate Commit**: `3948cecf0fb154cbda7bec091b9e7003dcaf0b8a`
- **Spec**: `specs/078_directional_impact_masking_engine_placement_damage_and_archetype_suspension_failure.md`
- **Command**: `cargo test -p wheelbase --test directional_damage_tests && cargo test -p race-kit --test world_tests && cargo test -p tdrace-app --test repair_invoice_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
test test_directional_impact_masking_zones ... ok
test test_jump_landing_damping_loss_roll_snap ... ok
test test_pushrod_fragility_drag_penalty ... ok
test test_rear_engine_headon_collision_immunity ... ok
test test_macpherson_curb_bottoming_steering_pull ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test wreck_gives_dnf_and_stays_collidable ... ok
test test_pit_service_state_machine ... ok
test laps_race_finishes_with_real_times ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.91s


running 1 test
test test_itemized_garage_repair_invoice_archetype_pricing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/directional_damage_tests.rs (target/debug/deps/directional_damage_tests-05916fda5b7189c3)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/world_tests.rs (target/debug/deps/world_tests-04ba45d71dcccb27)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/repair_invoice_tests.rs (target/debug/deps/repair_invoice_tests-c20aa2fd0d5e6551)

```
