---
type: validation_receipt
schema_version: "1.0"
spec: "specs/060_classic_academy_and_grassroots_career_onboarding.md"
epic: "tdrace-2sou"
candidate_commit: "ec65b3dc66448d5645d84913b727aaf8ebfce4a9"
verifier: "local-user"
evaluated_at: "2026-10-01T02:38:45Z"
command: "cargo test -p arcade-race-core --test profile_tests && cargo test -p tdrace-app --test career_onboarding_tests"
exit_code: 0
duration_ms: 1279
status: passed
---

# 🧾 Validation Receipt: Spec 060

- **Candidate Commit**: `ec65b3dc66448d5645d84913b727aaf8ebfce4a9`
- **Spec**: `specs/060_classic_academy_and_grassroots_career_onboarding.md`
- **Command**: `cargo test -p arcade-race-core --test profile_tests && cargo test -p tdrace-app --test career_onboarding_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 6 tests
test test_all_bronze_curriculum_purse_totals_7000 ... ok
test test_all_gold_curriculum_purse_totals_14500 ... ok
test test_all_silver_curriculum_purse_totals_10750 ... ok
test test_default_curriculum_structure ... ok
test test_fresh_academy_initialization_and_lesson_unlock_progression ... ok
test test_idempotent_bounty_payouts_on_replay ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 16 tests
test test_academy_all_bronze_progression_purse_and_unlocks ... ok
test test_academy_all_silver_and_gold_curriculum_purses ... ok
test test_academy_attempt_idempotency_on_replay ... ok
test test_academy_curriculum_graduation_grants_license ... ok
test test_academy_clean_attempt_and_disqualification_rules ... ok
test test_academy_pace_status_real_time_splits ... ok
test test_career_mode_access_requires_both_license_and_owned_vehicle ... ok
test test_classic_academy_selectable_in_modality_menu ... ok
test test_fresh_rookie_profile_zero_start_invariants ... ok
test test_grassroots_starter_models_in_catalog ... ok
test test_grassroots_pricing_calibration_and_affordability ... ok
test test_rookie_profile_database_persistence ... ok
test test_graduation_ceremony_ui_and_showroom_navigation ... ok
test test_career_mode_locked_modal_when_unlicensed ... ok
test test_starter_car_purchase_adds_to_owned_cars_and_unlocks_career ... ok
test test_setup_academy_session_and_execution ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-4f6b512e9b8ff5d1)
warning: tdrace-core@0.1.0: embedded official circuits are 10.3 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/career_onboarding_tests.rs (target/debug/deps/career_onboarding_tests-28af41fc2c91cab7)

```
