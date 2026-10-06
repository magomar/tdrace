---
type: validation_receipt
schema_version: "1.0"
spec: "specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md"
epic: "tdrace-b3u4"
candidate_commit: "f306de6d06fb59ed0761ff3ea916901b571380e5"
verifier: "local-user"
evaluated_at: "2026-10-05T07:32:07Z"
command: "cargo test -p tdrace-app --test codex_export_tests"
exit_code: 0
duration_ms: 12355
status: passed
---

# 🧾 Validation Receipt: Spec 087

- **Candidate Commit**: `f306de6d06fb59ed0761ff3ea916901b571380e5`
- **Spec**: `specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md`
- **Command**: `cargo test -p tdrace-app --test codex_export_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 11 tests
test cars_on_one_base_preset_share_their_platform ... ok
test launch_scope_holds_only_launch_modules ... ok
test controls_matches_game_input_presets ... ok
test racing_data_matches_series_formats_and_academy ... ok
test all_scope_exports_every_playable_car_and_no_vault_content ... ok
test driving_matches_game_steering_and_assists ... ok
test hud_matches_game_elements_and_cameras ... ok
test rivals_data_matches_driver_roster_and_ai_traits ... ok
test committed_codex_data_is_fresh ... ok
test classic_ax_brawler_power_comes_from_the_catalogue ... ok
test every_platform_part_points_at_an_exported_table_row ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.29s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 12.0 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/codex_export_tests.rs (target/debug/deps/codex_export_tests-e38a2999eb2fa515)

```
