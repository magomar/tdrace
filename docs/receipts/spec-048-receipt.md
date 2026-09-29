---
type: validation_receipt
schema_version: "1.0"
spec: "specs/048_rallycross_labels_rx_tier_45_cars_and_mint_400_desert_circuits.md"
epic: "tdrace-9pae"
candidate_commit: "d27f82cb04f85d32895b4fa70e9e5ca438ea8c86"
verifier: "local-user"
evaluated_at: "2026-09-28T14:40:38Z"
command: "cargo test -p tdrace-app --test garage_tests --test track_manager_tests --test extreme_offroad_module_tests --test series_tests --test module_track_unlock_tests --test render_tests --test profile_tests --test audio_tier_tests --test grid_positions_sync_tests"
exit_code: 0
duration_ms: 130027
status: passed
---

# 🧾 Validation Receipt: Spec 048

- **Candidate Commit**: `d27f82cb04f85d32895b4fa70e9e5ca438ea8c86`
- **Spec**: `specs/048_rallycross_labels_rx_tier_45_cars_and_mint_400_desert_circuits.md`
- **Command**: `cargo test -p tdrace-app --test garage_tests --test track_manager_tests --test extreme_offroad_module_tests --test series_tests --test module_track_unlock_tests --test render_tests --test profile_tests --test audio_tier_tests --test grid_positions_sync_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 5 tests
test test_classic_arcade_vehicles_preserve_tier_one_mapping ... ok
test test_all_25_tiers_resolve_unique_dedicated_archetypes ... ok
test test_vehicles_within_same_category_have_distinct_audio ... ok
test test_dsp_synthesis_all_25_archetypes_valid_buffers ... ok
test test_garage_showroom_dynamic_vehicle_switching_sound_resolution ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.80s


running 6 tests
test test_extreme_offroad_roster_integrity ... ok
test test_extreme_offroad_module_identity_and_vehicles ... ok
test test_extreme_offroad_tournament_formats ... ok
test test_extreme_offroad_session_switch_and_car_choice ... ok
test test_extreme_offroad_championship_flow ... ok
test test_extreme_offroad_tracks_and_geometry_validation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.15s


running 17 tests
test test_fleet_gallery_filter_conversions ... ok
test test_garage_geometry_and_car_card_carousel_layout ... ok
test test_fleet_gallery_tab_and_card_geometry ... ok
test test_all_80_real_cars_attributes_and_data_integrity ... ok
test test_fleet_gallery_module_tabs_only_and_no_all_tab ... ok
test test_garage_shows_all_module_models_across_tiers ... ok
test test_garage_view_modes ... ok
test test_starting_grid_footer_prompt_space_reserved_for_launch ... ok
test test_starting_grid_garage_button_rect_geometry ... ok
test test_roster_featured_cars_have_valid_lateral_assets ... ok
test test_starting_grid_card_0_enters_garage ... ok
test test_starting_grid_car_card_1_direct_garage_transition ... ok
test test_fleet_gallery_session_navigation_by_module ... ok
test test_menu_direct_garage_shortcut ... ok
test test_starting_grid_card_0_enter_vs_space_reservation ... ok
test test_career_tier_gating_in_garage ... ok
test test_garage_stops_music_and_plays_engine ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s


running 1 test
test test_all_96_tracks_grid_positions_count_and_validation has been running for over 60 seconds
test test_all_96_tracks_grid_positions_count_and_validation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 88.01s


running 4 tests
test test_gt_madring_unlocks_at_tier_five ... ok
test test_every_module_locks_circuits_above_tier_one ... ok
test test_every_module_circuit_unlocks_by_max_level ... ok
test test_session_locks_circuits_in_every_module_but_classic ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s


running 34 tests
test test_car_purchasing_with_spendable_xp_and_deduction ... ok
test test_country_registry_and_banner_metadata ... ok
test test_metric_distance_lap_xp_rounding_and_first_time_bonus ... ok
test test_gt_career_hub_calendar_helpers_and_customization ... ok
test test_multi_level_career_stunt_and_collision_metrics ... ok
test test_module_career_progress_persistence_and_xp_leveling ... ok
test test_championship_award_persistence_and_upgrade ... ok
test test_championship_car_original_sprite_factory_colors ... ok
test test_championships_sorted_by_tier_ascending ... ok
test test_profile_schema_and_crud ... ok
test test_race_history_logging_and_career_stats ... ok
test test_profile_editing_workflow ... ok
test test_profile_champ_tab_navigation_and_scroll ... ok
test test_profile_focus_hierarchy_and_filter_selection ... ok
test test_player_card_focus_and_roster_manager_navigation ... ok
test test_tier_1_starter_cars_single_entry_vehicle ... ok
test test_career_hub_focus_and_navigation ... ok
test test_trophy_filename_and_asset_resolution ... ok
test test_two_condition_tier_advancement_gates ... ok
test test_circuit_defaults_unlocked_and_dev_mode_unblocks_all ... ok
test test_option_a_tabbed_dashboard_navigation_and_filters ... ok
test test_real_championships_listing_and_filter ... ok
test test_resolve_championship_car_model_id_fallback_and_history ... ok
test test_clear_profile_history_and_hall_of_fame ... ok
test test_race_session_profile_integration_and_race_finish_logging ... ok
test test_race_finish_records_authentic_model_title_in_history ... ok
test test_module_career_progress_isolation_and_xp_crediting ... ok
test test_championship_completion_podium_trophy_awarded ... ok
test test_trophy_cabinet_grid_navigation_and_provenance_display ... ok
test test_championship_navigation_selection_and_launch ... ok
test test_gt_career_tier_launch_with_custom_calendar ... ok
test test_rally_championship_track_choice_sync_across_rounds ... ok
test test_gt_career_session_gating_and_cup_launch ... ok
test test_all_modules_career_tier_launch_and_calendar_counts ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.12s


running 43 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_car_body_roll_and_geometry ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_palette_and_car_color_schemes ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_procedural_surface_image_generators_all_15_surfaces ... ok
test test_surface_material_quality_and_properties ... ok
test test_surface_asset_files_exist_and_are_valid_png ... ok
test test_track_backdrop_colors ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_seamless_periodic_grass_and_asphalt_generators ... ok
test test_track_wear_state_phase_2_hooks ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_spline_ribbon_and_world_space_uv_mappings ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_segment_curvature_and_apex_rubbering_lateral_distribution ... ok
test test_spec_026_standalone_wheel_texture_asset_integrity ... ok
test test_classic_kart_topdown_sprite_orientation ... ok
test test_peugeot_208_rally4_topdown_sprite_orientation ... ok
test test_track_presets_geometry_for_rendering ... ok
test test_tony_kart_topdown_sprite_orientation ... ok
test test_vortex_dune_crusher_topdown_sprite_orientation ... ok
test test_spec_026_colorway_tinting_consistency_on_decomposed_kart ... ok
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_backdrop_ground_pass_execution ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.76s


running 28 tests
test test_modality_select_championship_editor_option ... ok
test test_all_embedded_presets_are_valid ... ok
test test_championship_manager_discovery_and_saving ... ok
test test_championship_editor_open_and_load_modal ... ok
test test_gt_tiers_1_to_5_specifications ... ok
test test_all_motorsport_modules_tiers_1_to_5_specifications ... ok
test test_primary_confirm_action_does_not_engage_handbrake ... ok
test test_nascar_and_kart_module_switch_defaults_to_tier_1_starter ... ok
test test_series_toml_syntax_and_session ... ok
test test_to_session_runtime_bridging ... ok
test test_toml_deserialization_and_validation ... ok
test test_toml_roundtrip_fidelity ... ok
test test_validation_rejects_invalid ... ok
test test_game_session_championship_editor_integration ... ok
test test_championship_cancel_latest_round_rolls_back_points_and_history ... ok
test test_player_throttle_in_kart_championship ... ok
test test_rally_tier_1_championship_starting_grid_eligibility ... ok
test test_post_race_rerun_cancels_uncommitted_results_and_restarts_round ... ok
test test_championship_standings_screen_rerun_rolls_back_and_restarts_round ... ok
test test_rally_championship_points_awarded_to_all_drivers_and_persisted_across_rounds ... ok
test test_kart_and_gt_championship_rosters_match_modules ... ok
test test_reset_championship_clears_session_and_database_history ... ok
test test_kart_career_tiers_1_to_5_launch_eligibility ... ok
test test_nascar_career_tiers_1_to_5_launch_eligibility ... ok
test test_kart_championship_first_round_bots_move ... ok
test test_all_modules_tier_1_championship_starters_are_eligible_and_unlocked ... ok
test test_all_25_preset_championships_launch_with_eligible_and_unlocked_cars ... ok
test test_championship_lap_calibration_across_all_modules ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.57s


running 38 tests
test test_portal_circuits_catalog_provenance_integrity ... ok
test test_consistent_module_categorization_in_module_view ... ok
test test_custom_circuit_multi_category_assignment ... ok
test test_dev_mode_reorder_preset_tracks_up_and_down ... ok
test test_draft_creation_and_isolation_from_main_menu ... ok
test test_metadata_editing ... ok
test test_multi_module_promotion_and_distribution ... ok
test test_custom_circuit_promoted_to_preset_classified_as_official_preset ... ok
test test_marina_bay_singapore_aliases_and_osm_calibration ... ok
test test_module_subdirectories_and_file_movement ... ok
test test_track_categories_initial_presets ... ok
test test_track_deletion ... ok
test test_promotion_and_demotion_lifecycle ... ok
test test_module_filter_filtering_and_presets_in_classic ... ok
test test_module_scoped_track_deletion_preserves_other_modules ... ok
test test_re_promoting_already_promoted_track_to_different_modules ... ok
test test_track_manager_clone_preset_to_drafts ... ok
test test_category_ordering_presets_first_then_custom ... ok
test test_create_new_draft_track_with_module_templates ... ok
test test_track_manager_promotion_mask_resolution_for_promoted_track ... ok
test test_predefined_track_demote_promote_and_delete ... ok
test test_track_manager_tab_and_module_cycling ... ok
test test_preset_circuits_edit_overwrite_and_persistence_across_modules ... ok
test test_track_manager_repeated_cloning_unique_slugs ... ok
test test_preset_reordering_persistence ... ok
test test_reordering_boundary_conditions ... ok
test test_session_active_module_tracks_reflects_reordered_presets ... ok
test test_track_manager_drafts_category_browsing_and_shortcut_9 ... ok
test test_standard_mode_rejects_preset_reordering ... ok
test test_track_manager_confirm_delete_modal ... ok
test test_track_manager_delete_with_backspace ... ok
test test_track_manager_confirm_delete_modal_arrow_switching ... ok
test test_empty_module_tracks_resilience ... ok
test test_track_manager_open_in_track_editor ... ok
test test_track_manager_clone_and_open_in_track_editor ... ok
test test_race_session_with_track_manager_flow ... ok
test test_all_canonical_track_files_provenance_integrity ... ok
test test_workspace_rally_deletion_preserves_classic ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests/audio_tier_tests.rs (target/debug/deps/audio_tier_tests-cae328ecd5e054c6)
     Running tests/extreme_offroad_module_tests.rs (target/debug/deps/extreme_offroad_module_tests-f6e3044d4e1411e6)
     Running tests/garage_tests.rs (target/debug/deps/garage_tests-c7f501bf2856b9f9)
     Running tests/grid_positions_sync_tests.rs (target/debug/deps/grid_positions_sync_tests-421da444581a70d6)
     Running tests/module_track_unlock_tests.rs (target/debug/deps/module_track_unlock_tests-ee2a4e254217a396)
     Running tests/profile_tests.rs (target/debug/deps/profile_tests-ebc403746e14f5ec)
     Running tests/render_tests.rs (target/debug/deps/render_tests-775578e7c65e1503)
     Running tests/series_tests.rs (target/debug/deps/series_tests-5946907894847472)
     Running tests/track_manager_tests.rs (target/debug/deps/track_manager_tests-e8b8bbc76e9c4ccd)

```
