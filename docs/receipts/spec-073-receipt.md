---
type: validation_receipt
schema_version: "1.0"
spec: "specs/073_realistic_vehicle_sprite_harmonization_and_modular_steered_wheel_articulation.md"
epic: "tdrace-gnu2"
candidate_commit: "a38cfca229480b736b9331321ad7bd3640de35c6"
verifier: "local-user"
evaluated_at: "2026-10-02T08:50:48Z"
command: "cargo test -p tdrace-app --test render_tests && cargo test -p tdrace-app --test audio_tests"
exit_code: 0
duration_ms: 6273
status: passed
---

# 🧾 Validation Receipt: Spec 073

- **Candidate Commit**: `a38cfca229480b736b9331321ad7bd3640de35c6`
- **Spec**: `specs/073_realistic_vehicle_sprite_harmonization_and_modular_steered_wheel_articulation.md`
- **Command**: `cargo test -p tdrace-app --test render_tests && cargo test -p tdrace-app --test audio_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 45 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_car_body_roll_and_geometry ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_palette_and_car_color_schemes ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_075_chassis_skeleton_render_geometry_and_fixture_alignment ... ok
test test_surface_material_quality_and_properties ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_track_backdrop_colors ... ok
test test_track_wear_state_phase_2_hooks ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_surface_asset_files_exist_and_are_valid_png ... ok
test test_procedural_surface_image_generators_all_15_surfaces ... ok
test test_seamless_periodic_grass_and_asphalt_generators ... ok
test test_spec_026_standalone_wheel_texture_asset_integrity ... ok
test test_peugeot_208_rally4_topdown_sprite_orientation ... ok
test test_classic_kart_topdown_sprite_orientation ... ok
test test_vortex_dune_crusher_topdown_sprite_orientation ... ok
test test_tony_kart_topdown_sprite_orientation ... ok
test test_spec_026_colorway_tinting_consistency_on_decomposed_kart ... ok
test test_spline_ribbon_and_world_space_uv_mappings ... ok
test test_segment_curvature_and_apex_rubbering_lateral_distribution ... ok
test test_track_presets_geometry_for_rendering ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_spec_073_classic_12_vehicle_harmonization_and_steered_wheels ... ok
test test_backdrop_ground_pass_execution ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.00s


running 20 tests
test test_adsr_envelope_stages ... ok
test test_audio_settings_and_mixer_gain ... ok
test test_car_choice_sound_type_mapping ... ok
test test_biquad_lowpass_filter_attenuation ... ok
test test_dsp_oscillators_bounds_and_shapes ... ok
test test_engine_rpm_model_gear_shifts_and_revs ... ok
test test_classic_cars_audio_profile_and_tier_one_sound_bank_mapping ... ok
test test_midi_tuning_frequencies ... ok
test test_soft_saturation_curve ... ok
test test_wav_riff_header_and_data_integrity ... ok
test test_stereo_delay_echo ... ok
test test_all_arcade_sfx_generators ... ok
test test_multi_engine_sound_types_synthesis_and_fallback ... ok
test test_audio_manager_engine_switching_and_shift_gap ... ok
test test_all_28_rpm_bands_synthesis_and_equal_power_weights ... ok
test test_menu_theme_synthesis ... ok
test test_nightcall_race_theme_synthesis_and_stereo_width ... ok
test test_session_resolve_active_sound_type_for_classic_vehicles ... ok
test test_independent_music_and_sound_separation_in_session ... ok
test test_race_session_audio_wiring_and_countdown_state ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.14s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/render_tests.rs (target/debug/deps/render_tests-9d3623b1bb6ff468)
warning: tdrace-core@0.1.0: embedded official circuits are 10.4 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/audio_tests.rs (target/debug/deps/audio_tests-70c07be4a6609834)

```
