---
type: validation_receipt
schema_version: "1.0"
spec: "specs/052_karting_6tier_career_progression_and_standalone_garden_gp.md"
epic: "tdrace-5f0d"
candidate_commit: "5791c01b67bad1c9e4bff91123ef77874ee63e37"
verifier: "local-user"
evaluated_at: "2026-09-29T10:21:01Z"
command: "cargo test -p tdrace-app --test series_tests --test profile_tests --test garage_tests --test render_tests"
exit_code: 0
duration_ms: 18408
status: passed
---

# 🧾 Validation Receipt: Spec 052

- **Candidate Commit**: `5791c01b67bad1c9e4bff91123ef77874ee63e37`
- **Spec**: `specs/052_karting_6tier_career_progression_and_standalone_garden_gp.md`
- **Command**: `cargo test -p tdrace-app --test series_tests --test profile_tests --test garage_tests --test render_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 22 tests
test test_fleet_gallery_filter_conversions ... ok
test test_fleet_gallery_tab_and_card_geometry ... ok
test test_all_80_real_cars_attributes_and_data_integrity ... ok
test test_fleet_gallery_module_tabs_only_and_no_all_tab ... ok
test test_garage_view_modes ... ok
test test_garage_shows_all_module_models_across_tiers ... ok
test test_garage_geometry_and_car_card_carousel_layout ... ok
test test_roster_featured_cars_have_valid_lateral_assets ... ok
test test_starting_grid_footer_prompt_space_reserved_for_launch ... ok
test test_starting_grid_garage_button_rect_geometry ... ok
test test_starting_grid_car_card_1_direct_garage_transition ... ok
test test_fleet_gallery_session_navigation_by_module ... ok
test test_starting_grid_card_0_enters_garage ... ok
test test_starting_grid_card_0_enter_vs_space_reservation ... ok
test test_menu_direct_garage_shortcut ... ok
test test_garage_stops_music_and_plays_engine ... ok
test leaving_the_garage_by_any_path_restores_the_discipline ... ok
test test_garage_engine_sound_config_cached_and_not_reset_per_frame ... ok
test browsing_another_discipline_then_esc_restores_the_session_discipline ... ok
test picking_another_disciplines_car_from_the_circuit_menu_switches_fully ... ok
test test_career_tier_gating_in_garage ... ok
test a_set_up_race_refuses_a_car_from_another_discipline ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s


running 38 tests
test test_car_purchasing_with_credits_and_zero_xp_deduction ... ok
test test_branching_championship_records_and_tier_advancement ... ok
test test_country_registry_and_banner_metadata ... ok
test test_gt_career_hub_calendar_helpers_and_customization ... ok
test test_metric_distance_lap_xp_rounding_and_first_time_bonus ... ok
test test_kart_career_6_tier_progression_and_20_track_unlocks ... ok
test test_spec_053_round_purse_and_clean_race_bonuses ... ok
test test_tier_1_starter_cars_single_entry_vehicle ... ok
test test_two_condition_tier_advancement_gates ... ok
test test_trophy_filename_and_asset_resolution ... ok
test test_championship_award_persistence_and_upgrade ... ok
test test_profile_schema_and_crud ... ok
test test_race_history_logging_and_career_stats ... ok
test test_multi_level_career_stunt_and_collision_metrics ... ok
test test_module_career_progress_persistence_and_xp_leveling ... ok
test test_series_manager_multi_series_tier_query ... ok
test test_championships_sorted_by_tier_ascending ... ok
test test_championship_car_original_sprite_factory_colors ... ok
test test_real_championships_listing_and_filter ... ok
test test_resolve_championship_car_model_id_fallback_and_history ... ok
test test_profile_editing_workflow ... ok
test test_option_a_tabbed_dashboard_navigation_and_filters ... ok
test test_player_card_focus_and_roster_manager_navigation ... ok
test test_profile_focus_hierarchy_and_filter_selection ... ok
test test_career_hub_focus_and_navigation ... ok
test test_profile_champ_tab_navigation_and_scroll ... ok
test test_race_session_profile_integration_and_race_finish_logging ... ok
test test_clear_profile_history_and_hall_of_fame ... ok
test test_trophy_cabinet_grid_navigation_and_provenance_display ... ok
test test_circuit_defaults_unlocked_and_dev_mode_unblocks_all ... ok
test test_race_finish_records_authentic_model_title_in_history ... ok
test test_championship_completion_podium_trophy_awarded ... ok
test test_module_career_progress_isolation_and_xp_crediting ... ok
test test_championship_navigation_selection_and_launch ... ok
test test_gt_career_tier_launch_with_custom_calendar ... ok
test test_rally_championship_track_choice_sync_across_rounds ... ok
test test_gt_career_session_gating_and_cup_launch ... ok
test test_all_modules_career_tier_launch_and_calendar_counts ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.16s


running 43 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_car_body_roll_and_geometry ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_palette_and_car_color_schemes ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_track_backdrop_colors ... ok
test test_track_wear_state_phase_2_hooks ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_surface_material_quality_and_properties ... ok
test test_surface_asset_files_exist_and_are_valid_png ... ok
test test_spline_ribbon_and_world_space_uv_mappings ... ok
test test_procedural_surface_image_generators_all_15_surfaces ... ok
test test_seamless_periodic_grass_and_asphalt_generators ... ok
test test_segment_curvature_and_apex_rubbering_lateral_distribution ... ok
test test_spec_026_standalone_wheel_texture_asset_integrity ... ok
test test_track_presets_geometry_for_rendering ... ok
test test_peugeot_208_rally4_topdown_sprite_orientation ... ok
test test_classic_kart_topdown_sprite_orientation ... ok
test test_vortex_dune_crusher_topdown_sprite_orientation ... ok
test test_tony_kart_topdown_sprite_orientation ... ok
test test_spec_026_colorway_tinting_consistency_on_decomposed_kart ... ok
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_backdrop_ground_pass_execution ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.84s


running 30 tests
test test_modality_select_championship_editor_option ... ok
test test_series_toml_syntax_and_session ... ok
test test_to_session_runtime_bridging ... ok
test test_toml_roundtrip_fidelity ... ok
test test_toml_deserialization_and_validation ... ok
test test_validation_rejects_invalid ... ok
test test_all_embedded_presets_are_valid ... ok
test test_championship_manager_discovery_and_saving ... ok
test test_championship_editor_open_and_load_modal ... ok
test test_all_motorsport_modules_tiers_1_to_5_specifications ... ok
test test_gt_tiers_1_to_5_specifications ... ok
test test_primary_confirm_action_does_not_engage_handbrake ... ok
test test_championship_studio_exits_with_gamepad_b ... ok
test test_nascar_and_kart_module_switch_defaults_to_tier_1_starter ... ok
test test_player_throttle_in_kart_championship ... ok
test test_championship_cancel_latest_round_rolls_back_points_and_history ... ok
test test_game_session_championship_editor_integration ... ok
test test_rally_tier_1_championship_starting_grid_eligibility ... ok
test test_post_race_rerun_cancels_uncommitted_results_and_restarts_round ... ok
test test_post_race_esc_keeps_the_championship_round ... ok
test test_reset_championship_clears_session_and_database_history ... ok
test test_kart_and_gt_championship_rosters_match_modules ... ok
test test_championship_standings_screen_rerun_rolls_back_and_restarts_round ... ok
test test_rally_championship_points_awarded_to_all_drivers_and_persisted_across_rounds ... ok
test test_kart_career_tiers_1_to_6_launch_eligibility ... ok
test test_nascar_career_tiers_1_to_5_launch_eligibility ... ok
test test_all_modules_tier_1_championship_starters_are_eligible_and_unlocked ... ok
test test_all_26_preset_championships_launch_with_eligible_and_unlocked_cars ... ok
test test_kart_championship_first_round_bots_move ... ok
test test_championship_lap_calibration_across_all_modules ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.20s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/garage_tests.rs (target/debug/deps/garage_tests-c313d1ebfd98dbe0)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-a53fbd98c9fcae29)
     Running tests/render_tests.rs (target/debug/deps/render_tests-2a3175fa2fe858fb)
     Running tests/series_tests.rs (target/debug/deps/series_tests-26c891b199a516f9)

```
