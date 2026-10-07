---
type: validation_receipt
schema_version: "1.0"
spec: "specs/091_global_prebaked_vehicle_steered_wheel_articulation.md"
epic: "tdrace-a2ht"
candidate_commit: "14bb15e615130916b45ff02a412b13de29e7f614"
verifier: "local-user"
evaluated_at: "2026-10-06T13:32:27Z"
command: "cargo test -p tdrace-app --test render_tests"
exit_code: 0
duration_ms: 5868
status: passed
---

# 🧾 Validation Receipt: Spec 091

- **Candidate Commit**: `14bb15e615130916b45ff02a412b13de29e7f614`
- **Spec**: `specs/091_global_prebaked_vehicle_steered_wheel_articulation.md`
- **Command**: `cargo test -p tdrace-app --test render_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 53 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_car_body_roll_and_geometry ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_palette_and_car_color_schemes ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_074_steered_wheel_accent_scale_and_dimensions ... ok
test test_spec_075_chassis_skeleton_render_geometry_and_fixture_alignment ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_surface_material_quality_and_properties ... ok
test test_track_wear_state_phase_2_hooks ... ok
test test_spec_094_universal_steered_wheel_derivation_across_all_platforms ... ok
test test_spec_091_global_steered_wheel_articulation_and_runtime_integration ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_track_backdrop_colors ... ok
test test_surface_asset_files_exist_and_are_valid_png ... ok
test test_procedural_surface_image_generators_all_15_surfaces ... ok
test test_seamless_periodic_grass_and_asphalt_generators ... ok
test test_wheel_texture_cache_memory_bounds ... ok
test test_spec_026_standalone_wheel_texture_asset_integrity ... ok
test test_spline_ribbon_and_world_space_uv_mappings ... ok
test test_vortex_dune_crusher_topdown_sprite_orientation ... ok
test test_classic_kart_topdown_sprite_orientation ... ok
test test_classic_at_safari_topdown_cenital_symmetry ... ok
test test_spec_026_colorway_tinting_consistency_on_decomposed_kart ... ok
test test_peugeot_208_rally4_topdown_sprite_orientation ... ok
test test_tony_kart_topdown_sprite_orientation ... ok
test test_segment_curvature_and_apex_rubbering_lateral_distribution ... ok
test test_crg_black_mirror_okj_lateral_sprite_orientation ... ok
test test_classic_ax_mudlark_lateral_sprite_orientation ... ok
test test_track_presets_geometry_for_rendering ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_backdrop_ground_pass_execution ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_spec_073_classic_12_vehicle_harmonization_and_steered_wheels ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_spec_091_global_chassis_sprites_integrity_and_dimensions ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.80s


stderr:
warning: tdrace-core@0.1.0: embedded official circuits are 12.1 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/render_tests.rs (target/debug/deps/render_tests-1eaa53f16a20180a)

```
