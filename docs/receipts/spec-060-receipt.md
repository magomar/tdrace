---
type: validation_receipt
schema_version: "1.0"
spec: "specs/060_classic_academy_and_grassroots_career_onboarding.md"
epic: "tdrace-2sou"
candidate_commit: "f866c56006bf211a2b0fcbc00dd1eeae2c294682"
verifier: "local-user"
evaluated_at: "2026-10-01T14:56:13Z"
command: "cargo test -p tdrace-app --test career_onboarding_tests"
exit_code: 0
duration_ms: 1293
status: passed
---

# 🧾 Validation Receipt: Spec 060

- **Candidate Commit**: `f866c56006bf211a2b0fcbc00dd1eeae2c294682`
- **Spec**: `specs/060_classic_academy_and_grassroots_career_onboarding.md`
- **Command**: `cargo test -p tdrace-app --test career_onboarding_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 17 tests
test test_academy_clean_attempt_and_disqualification_rules ... ok
test test_academy_all_bronze_progression_purse_and_unlocks ... ok
test test_academy_curriculum_graduation_grants_license ... ok
test test_academy_attempt_idempotency_on_replay ... ok
test test_academy_all_silver_and_gold_curriculum_purses ... ok
test test_academy_pace_status_real_time_splits ... ok
test test_career_mode_access_requires_both_license_and_owned_vehicle ... ok
test test_classic_academy_selectable_in_modality_menu ... ok
test test_feeder_ladder_and_discipline_license_gating ... ok
test test_fresh_rookie_profile_zero_start_invariants ... ok
test test_grassroots_starter_models_in_catalog ... ok
test test_grassroots_pricing_calibration_and_affordability ... ok
test test_rookie_profile_database_persistence ... ok
test test_starter_car_purchase_adds_to_owned_cars_and_unlocks_career ... ok
test test_career_mode_locked_modal_when_unlicensed ... ok
test test_graduation_ceremony_ui_and_showroom_navigation ... ok
test test_setup_academy_session_and_execution ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 10.3 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/career_onboarding_tests.rs (target/debug/deps/career_onboarding_tests-28af41fc2c91cab7)

```
