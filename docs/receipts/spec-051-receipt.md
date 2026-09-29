---
type: validation_receipt
schema_version: "1.0"
spec: "specs/051_rallycross_6tier_career_progression_and_expanded_circuit_roster.md"
epic: "tdrace-ek8k"
candidate_commit: "595f03f5cf1e69090f83c2b99bc628daf539f307"
verifier: "local-user"
evaluated_at: "2026-09-29T14:13:17Z"
command: "cargo test -p tdrace-app --test rally_tracks_tests --test series_tests --test profile_tests --test garage_tests --test module_track_unlock_tests --test driver_favorite_cars_tests"
exit_code: 0
duration_ms: 19116
status: passed
---

# 🧾 Validation Receipt: Spec 051

- **Candidate Commit**: `595f03f5cf1e69090f83c2b99bc628daf539f307`
- **Spec**: `specs/051_rallycross_6tier_career_progression_and_expanded_circuit_roster.md`
- **Command**: `cargo test -p tdrace-app --test rally_tracks_tests --test series_tests --test profile_tests --test garage_tests --test module_track_unlock_tests --test driver_favorite_cars_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 12 tests
test test_all_six_modules_have_exactly_twelve_drivers ... ok
test test_classic_module_normalizes_all_tiers_to_tier_one ... ok
test test_all_seventy_two_driver_ids_are_unique ... ok
test test_rally_tier_4_ai_driver_signature_cars ... ok
test test_no_driver_uses_player_default_colors ... ok
test test_module_drivers_per_tier_favorite_cars_resolve_in_catalog ... ok
test test_classic_free_car_selection_assigns_signature_cars ... ok
test test_extreme_offroad_starting_grid_assigns_signature_cars ... ok
test test_kart_starting_grid_assigns_signature_cars ... ok
test test_rally_starting_grid_assigns_signature_cars ... ok
test test_nascar_starting_grid_assigns_signature_cars ... ok
test test_gt_tier_starting_grid_assigns_signature_cars ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s


running 22 tests
test test_fleet_gallery_filter_conversions ... ok
test test_fleet_gallery_module_tabs_only_and_no_all_tab ... ok
test test_all_80_real_cars_attributes_and_data_integrity ... ok
test test_garage_geometry_and_car_card_carousel_layout ... ok
test test_fleet_gallery_tab_and_card_geometry ... ok
test test_garage_shows_all_module_models_across_tiers ... ok
test test_garage_view_modes ... ok
test test_roster_featured_cars_have_valid_lateral_assets ... ok
test test_starting_grid_garage_button_rect_geometry ... ok
test test_starting_grid_footer_prompt_space_reserved_for_launch ... ok
test test_fleet_gallery_session_navigation_by_module ... ok
test test_starting_grid_car_card_1_direct_garage_transition ... ok
test test_menu_direct_garage_shortcut ... ok
test test_starting_grid_card_0_enters_garage ... ok
test test_starting_grid_card_0_enter_vs_space_reservation ... ok
test test_garage_stops_music_and_plays_engine ... ok
test leaving_the_garage_by_any_path_restores_the_discipline ... ok
test test_garage_engine_sound_config_cached_and_not_reset_per_frame ... ok
test browsing_another_discipline_then_esc_restores_the_session_discipline ... ok
test picking_another_disciplines_car_from_the_circuit_menu_switches_fully ... ok
test a_set_up_race_refuses_a_car_from_another_discipline ... ok
test test_career_tier_gating_in_garage ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s


running 5 tests
test test_gt_madring_unlocks_at_tier_five ... ok
test test_rally_six_tier_circuit_unlock_matrix ... ok
test test_every_module_locks_circuits_above_tier_one ... ok
test test_every_module_circuit_unlocks_by_max_level ... ok
test test_session_locks_circuits_in_every_module_but_classic ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s


running 34 tests
test test_car_purchasing_with_spendable_xp_and_deduction ... ok
test test_country_registry_and_banner_metadata ... ok
test test_metric_distance_lap_xp_rounding_and_first_time_bonus ... ok
test test_gt_career_hub_calendar_helpers_and_customization ... ok
test test_tier_1_starter_cars_single_entry_vehicle ... ok
test test_two_condition_tier_advancement_gates ... ok
test test_trophy_filename_and_asset_resolution ... ok
test test_race_history_logging_and_career_stats ... ok
test test_multi_level_career_stunt_and_collision_metrics ... ok
test test_championship_award_persistence_and_upgrade ... ok
test test_profile_schema_and_crud ... ok
test test_module_career_progress_persistence_and_xp_leveling ... ok
test test_real_championships_listing_and_filter ... ok
test test_championships_sorted_by_tier_ascending ... ok
test test_championship_car_original_sprite_factory_colors ... ok
test test_resolve_championship_car_model_id_fallback_and_history ... ok
test test_career_hub_focus_and_navigation ... ok
test test_trophy_cabinet_grid_navigation_and_provenance_display ... ok
test test_profile_champ_tab_navigation_and_scroll ... ok
test test_option_a_tabbed_dashboard_navigation_and_filters ... ok
test test_player_card_focus_and_roster_manager_navigation ... ok
test test_clear_profile_history_and_hall_of_fame ... ok
test test_profile_focus_hierarchy_and_filter_selection ... ok
test test_profile_editing_workflow ... ok
test test_race_session_profile_integration_and_race_finish_logging ... ok
test test_circuit_defaults_unlocked_and_dev_mode_unblocks_all ... ok
test test_module_career_progress_isolation_and_xp_crediting ... ok
test test_championship_completion_podium_trophy_awarded ... ok
test test_race_finish_records_authentic_model_title_in_history ... ok
test test_championship_navigation_selection_and_launch ... ok
test test_gt_career_tier_launch_with_custom_calendar ... ok
test test_rally_championship_track_choice_sync_across_rounds ... ok
test test_gt_career_session_gating_and_cup_launch ... ok
test test_all_modules_career_tier_launch_and_calendar_counts ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s


running 11 tests
test test_dirt_figure_eight_horizontal_flat_dirt_arena ... ok
test test_dirt_figure_eight_jump_ramps_proportional_trajectory ... ok
test test_blyton_ids_resolve_to_croft ... ok
test test_silverstone_and_yas_marina_ids_resolve_to_their_replacements ... ok
test test_world_rx_jump_ramps_dirt_surface_and_containment_landing ... ok
test test_world_rx_tracks_jump_ramps_and_mixed_surfaces ... ok
test test_export_and_save_rally_tracks_to_disk ... ok
test test_famous_rally_tracks_in_track_manager_and_menu_resolution ... ok
test test_rally_tracks_centerline_driving_and_no_wall_obstructions ... ok
test test_rally_race_session_simulation_on_new_tracks ... ok
test test_rally_module_tracks_integrity_and_validation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.55s


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
test test_championship_editor_open_and_load_modal ... ok
test test_all_motorsport_modules_tiers_1_to_5_specifications ... ok
test test_primary_confirm_action_does_not_engage_handbrake ... ok
test test_nascar_and_kart_module_switch_defaults_to_tier_1_starter ... ok
test test_game_session_championship_editor_integration ... ok
test test_championship_cancel_latest_round_rolls_back_points_and_history ... ok
test test_player_throttle_in_kart_championship ... ok
test test_post_race_rerun_cancels_uncommitted_results_and_restarts_round ... ok
test test_rally_tier_1_championship_starting_grid_eligibility ... ok
test test_championship_standings_screen_rerun_rolls_back_and_restarts_round ... ok
test test_reset_championship_clears_session_and_database_history ... ok
test test_rally_championship_points_awarded_to_all_drivers_and_persisted_across_rounds ... ok
test test_kart_and_gt_championship_rosters_match_modules ... ok
test test_kart_career_tiers_1_to_5_launch_eligibility ... ok
test test_nascar_career_tiers_1_to_5_launch_eligibility ... ok
test test_kart_championship_first_round_bots_move ... ok
test test_all_modules_tier_1_championship_starters_are_eligible_and_unlocked ... ok
test test_all_27_preset_championships_launch_with_eligible_and_unlocked_cars ... ok
test test_championship_lap_calibration_across_all_modules ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.24s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/driver_favorite_cars_tests.rs (target/debug/deps/driver_favorite_cars_tests-28a17fecffb5fcaf)
     Running tests/garage_tests.rs (target/debug/deps/garage_tests-fb475afb101c8d4e)
     Running tests/module_track_unlock_tests.rs (target/debug/deps/module_track_unlock_tests-dcf7ccd14f17a13f)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-26a7d55282133aed)
     Running tests/rally_tracks_tests.rs (target/debug/deps/rally_tracks_tests-cdc82b05c22ed1d1)
Checking track: holjes_rx
Checking track: lydden_hill
Checking track: hell_rx
Checking track: loheac_rx
Checking track: estering_rx
Checking track: montalegre_rx
Checking track: nyirad_rx
Checking track: kouvola_rx
Checking track: catalunya_rx
Checking track: mettet_rx
Checking track: lavare_rx
Checking track: riga_rx
Checking track: killarney_rx
Checking track: lessay_rx
Checking track: essay_rx
Checking track: dreux_rx
Checking track: croft_rx
Checking track: spa_rx
Checking track: silverstone_rx
Checking track: erx_motor_park
     Running tests/series_tests.rs (target/debug/deps/series_tests-47d5701e54b1f192)

```
