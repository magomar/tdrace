---
type: validation_receipt
schema_version: "1.0"
spec: "specs/040_interactive_controls_settings_for_steering_smoothing_and_hold_bleed.md"
epic: "tdrace-vgez"
candidate_commit: "aa9c1d8a312770e1c1a6e49c62d5ab544d09382d"
verifier: "local-user"
evaluated_at: "2026-09-27T17:22:03Z"
command: "cargo test -p cabinet --test cabinet_integration_tests && cargo test -p tdrace-app --test input_smoothing_tests"
exit_code: 0
duration_ms: 306
status: passed
---

# 🧾 Validation Receipt: Spec 040

- **Candidate Commit**: `aa9c1d8a312770e1c1a6e49c62d5ab544d09382d`
- **Spec**: `specs/040_interactive_controls_settings_for_steering_smoothing_and_hold_bleed.md`
- **Command**: `cargo test -p cabinet --test cabinet_integration_tests && cargo test -p tdrace-app --test input_smoothing_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 24 tests
test test_arcade_settings_modal_controls_tab_widgets_and_rollback ... ok
test test_arcade_settings_modal_display_tab_integration ... ok
test test_audio_mixer_buses ... ok
test test_arcade_settings_modal_helpers_tab_integration ... ok
test test_cabinet_settings_widgets_interaction ... ok
test test_arcade_settings_modal_lifecycle_and_bindings ... ok
test test_crt_scanlines_and_settings_integration ... ok
test test_digital_input_filter_progressive_ramp ... ok
test test_display_resolutions_and_window_modes ... ok
test test_floating_text_popups_and_decay ... ok
test test_juice_fx_mechanics ... ok
test test_modal_screen_stack ... ok
test test_nav_grid_2d_orthogonal_navigation ... ok
test test_profile_manager_lifecycle ... ok
test test_ui_scaler_responsive_math ... ok
test test_screen_stack_transitions_lifecycle ... ok
test test_input_mapping_action_system ... ok
test test_cabinet_context_audio_wiring_and_tactile_feedback ... ok
test test_leaderboard_modal_rendering_and_scrolling ... ok
test test_arcade_settings_dirty_tracking_and_exit_modal ... ok
test test_profile_select_modal_slot_and_customization ... ok
test test_universal_confirm_modal_lifecycle ... ok
test test_arcade_settings_modal_gameplay_subtab_navigation ... ok
test test_arcade_settings_modal_arrow_category_navigation ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test test_digital_input_filter_progressive_rise_and_centering ... ok
test test_interactive_controls_filter_sync_and_persistence ... ok
test test_speed_sensitive_steering_scaling ... ok
test test_non_linear_center_micro_corrections ... ok
test test_steering_profiles_configuration_and_cycling ... ok
test test_keyboard_progressive_brake_tap_vs_hold ... ok
test test_vehicle_high_speed_turn_stability_with_smoothed_input ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
   Compiling cabinet v0.1.0 (/home/mario/workspace/games/tdrace/crates/cabinet)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/cabinet_integration_tests.rs (target/debug/deps/cabinet_integration_tests-d5ac5d838e38a120)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/input_smoothing_tests.rs (target/debug/deps/input_smoothing_tests-f3d53b7ee5156480)

```
