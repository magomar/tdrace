---
type: validation_receipt
schema_version: "1.0"
spec: "specs/050_fia_autocross_championship_and_vehicle_roster.md"
epic: "tdrace-8sx9"
candidate_commit: "0eb6b213fa98ec72bf83d017563756287143b676"
verifier: "local-user"
evaluated_at: "2026-09-29T21:06:32Z"
command: "cargo test -p tdrace-app --test autocross_rules_tests --test autocross_catalog_tests --test autocross_circuits_tests --test series_tests --test garage_tests --test track_manager_tests"
exit_code: 0
duration_ms: 17454
status: passed
---

# 🧾 Validation Receipt: Spec 050

- **Candidate Commit**: `0eb6b213fa98ec72bf83d017563756287143b676`
- **Spec**: `specs/050_fia_autocross_championship_and_vehicle_roster.md`
- **Command**: `cargo test -p tdrace-app --test autocross_rules_tests --test autocross_catalog_tests --test autocross_circuits_tests --test series_tests --test garage_tests --test track_manager_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 3 tests
test test_autocross_catalog_has_three_vehicles_per_tier ... ok
test test_autocross_catalog_has_fifteen_vehicles ... ok
test test_autocross_specific_vehicle_identities ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_seventeen_autocross_circuits_embedded_in_catalog ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.84s


running 4 tests
test test_autocross_driver_roster ... ok
test test_autocross_game_module_identity_and_properties ... ok
test test_autocross_supported_game_modes ... ok
test test_autocross_tracks_count_and_zero_joker_ruleset ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.59s


running 22 tests
test test_fleet_gallery_filter_conversions ... ok
test test_fleet_gallery_tab_and_card_geometry ... ok
test test_fleet_gallery_module_tabs_only_and_no_all_tab ... ok
test test_all_80_real_cars_attributes_and_data_integrity ... ok
test test_garage_geometry_and_car_card_carousel_layout ... ok
test test_garage_shows_all_module_models_across_tiers ... ok
test test_garage_view_modes ... ok
test test_starting_grid_footer_prompt_space_reserved_for_launch ... ok
test test_roster_featured_cars_have_valid_lateral_assets ... ok
test test_starting_grid_garage_button_rect_geometry ... ok
test test_fleet_gallery_session_navigation_by_module ... ok
test test_menu_direct_garage_shortcut ... ok
test test_starting_grid_card_0_enters_garage ... ok
test test_starting_grid_car_card_1_direct_garage_transition ... ok
test test_starting_grid_card_0_enter_vs_space_reservation ... ok
test test_garage_stops_music_and_plays_engine ... ok
test leaving_the_garage_by_any_path_restores_the_discipline ... ok
test test_garage_engine_sound_config_cached_and_not_reset_per_frame ... ok
test browsing_another_discipline_then_esc_restores_the_session_discipline ... ok
test picking_another_disciplines_car_from_the_circuit_menu_switches_fully ... ok
test a_set_up_race_refuses_a_car_from_another_discipline ... ok
test test_career_tier_gating_in_garage ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s


running 28 tests
test test_modality_select_championship_editor_option ... ok
test test_series_toml_syntax_and_session ... ok
test test_to_session_runtime_bridging ... ok
test test_toml_deserialization_and_validation ... ok
test test_validation_rejects_invalid ... ok
test test_toml_roundtrip_fidelity ... ok
test test_all_embedded_presets_are_valid ... ok
test test_championship_manager_discovery_and_saving ... ok
test test_gt_tiers_1_to_5_specifications ... ok
test test_all_motorsport_modules_tiers_1_to_5_specifications ... ok
test test_championship_editor_open_and_load_modal ... ok
test test_primary_confirm_action_does_not_engage_handbrake ... ok
test test_nascar_and_kart_module_switch_defaults_to_tier_1_starter ... ok
test test_game_session_championship_editor_integration ... ok
test test_championship_cancel_latest_round_rolls_back_points_and_history ... ok
test test_player_throttle_in_kart_championship ... ok
test test_post_race_rerun_cancels_uncommitted_results_and_restarts_round ... ok
test test_rally_tier_1_championship_starting_grid_eligibility ... ok
test test_championship_standings_screen_rerun_rolls_back_and_restarts_round ... ok
test test_reset_championship_clears_session_and_database_history ... ok
test test_kart_and_gt_championship_rosters_match_modules ... ok
test test_rally_championship_points_awarded_to_all_drivers_and_persisted_across_rounds ... ok
test test_kart_career_tiers_1_to_5_launch_eligibility ... ok
test test_nascar_career_tiers_1_to_5_launch_eligibility ... ok
test test_kart_championship_first_round_bots_move ... ok
test test_all_modules_tier_1_championship_starters_are_eligible_and_unlocked ... ok
test test_all_preset_championships_launch_with_eligible_and_unlocked_cars ... ok
test test_championship_lap_calibration_across_all_modules ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.77s


running 38 tests
test test_track_categories_initial_presets ... ok
test test_portal_circuits_catalog_provenance_integrity ... ok
test test_track_deletion ... ok
test test_track_manager_clone_preset_to_drafts ... ok
test test_dev_mode_reorder_preset_tracks_up_and_down ... ok
test test_track_manager_tab_and_module_cycling ... ok
test test_custom_circuit_multi_category_assignment ... ok
test test_consistent_module_categorization_in_module_view ... ok
test test_track_manager_promotion_mask_resolution_for_promoted_track ... ok
test test_custom_circuit_promoted_to_preset_classified_as_official_preset ... ok
test test_draft_creation_and_isolation_from_main_menu ... ok
test test_metadata_editing ... ok
test test_multi_module_promotion_and_distribution ... ok
test test_marina_bay_singapore_aliases_and_osm_calibration ... ok
test test_module_subdirectories_and_file_movement ... ok
test test_promotion_and_demotion_lifecycle ... ok
test test_re_promoting_already_promoted_track_to_different_modules ... ok
test test_track_manager_repeated_cloning_unique_slugs ... ok
test test_module_filter_filtering_and_presets_in_classic ... ok
test test_module_scoped_track_deletion_preserves_other_modules ... ok
test test_category_ordering_presets_first_then_custom ... ok
test test_create_new_draft_track_with_module_templates ... ok
test test_predefined_track_demote_promote_and_delete ... ok
test test_preset_reordering_persistence ... ok
test test_preset_circuits_edit_overwrite_and_persistence_across_modules ... ok
test test_reordering_boundary_conditions ... ok
test test_track_manager_confirm_delete_modal ... ok
test test_track_manager_delete_with_backspace ... ok
test test_track_manager_drafts_category_browsing_and_shortcut_9 ... ok
test test_track_manager_confirm_delete_modal_arrow_switching ... ok
test test_session_active_module_tracks_reflects_reordered_presets ... ok
test test_standard_mode_rejects_preset_reordering ... ok
test test_empty_module_tracks_resilience ... ok
test test_race_session_with_track_manager_flow ... ok
test test_track_manager_open_in_track_editor ... ok
test test_track_manager_clone_and_open_in_track_editor ... ok
test test_workspace_rally_deletion_preserves_classic ... ok
test test_all_canonical_track_files_provenance_integrity ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.09s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/autocross_catalog_tests.rs (target/debug/deps/autocross_catalog_tests-ab17ac3e06d0d578)
     Running tests/autocross_circuits_tests.rs (target/debug/deps/autocross_circuits_tests-415d0d134b6132df)
     Running tests/autocross_rules_tests.rs (target/debug/deps/autocross_rules_tests-040bee61da09e645)
     Running tests/garage_tests.rs (target/debug/deps/garage_tests-fb475afb101c8d4e)
     Running tests/series_tests.rs (target/debug/deps/series_tests-47d5701e54b1f192)
     Running tests/track_manager_tests.rs (target/debug/deps/track_manager_tests-d12a3b3b180e7344)

```
