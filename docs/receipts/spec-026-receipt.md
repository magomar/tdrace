---
type: validation_receipt
schema_version: "1.0"
spec: "specs/026_topdown_wheel_steering_animations.md"
epic: "tdrace-ngxk"
candidate_commit: "3c0e253df1a8e1bf33c683d08502830d76e041d8"
verifier: "local-user"
evaluated_at: "2026-09-26T16:12:37Z"
command: "cargo test -p tdrace-app --test render_tests"
exit_code: 0
duration_ms: 3771
status: passed
---

# 🧾 Validation Receipt: Spec 026

- **Candidate Commit**: `3c0e253df1a8e1bf33c683d08502830d76e041d8`
- **Spec**: `specs/026_topdown_wheel_steering_animations.md`
- **Command**: `cargo test -p tdrace-app --test render_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 43 tests
test test_all_80_motorsport_cars_catalog_integrity ... ok
test test_car_body_roll_and_geometry ... ok
test test_cabinet_color_utilities_and_theme_reexport ... ok
test test_classic_arcade_fantasy_sprites_presence ... ok
test test_all_modality_emblem_assets_and_integrity ... ok
test test_palette_and_car_color_schemes ... ok
test test_porsche_gt3r_lateral_sprite_asset_presence ... ok
test test_porsche_gt3r_topdown_sprite_asset_presence ... ok
test test_sand_rail_visual_archetype_and_liveries ... ok
test test_scenery_culling_and_grandstand_render_geometry ... ok
test test_macro_modulation_value_range_and_spatial_continuity ... ok
test test_spec_026_steered_wheel_config_lookup_and_legacy_fallback ... ok
test test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling ... ok
test test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars ... ok
test test_spec_031_all_catalog_cars_lighting_by_modality ... ok
test test_spec_031_modality_realistic_lighting_profiles ... ok
test test_spec_031_procedural_archetype_lighting_fallbacks ... ok
test test_stock_car_visual_archetype_and_liveries ... ok
test test_surface_material_quality_and_properties ... ok
test test_track_backdrop_colors ... ok
test test_tree_cenital_canopy_and_alpha_modulation ... ok
test test_vehicle_asset_registry_color_helpers ... ok
test test_vehicle_lighting_toggle_switch_on_off ... ok
test test_track_wear_state_phase_2_hooks ... ok
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
test test_classic_mask_tinting_transforms_bodywork_pixels ... ok
test test_gt_models_mask_tinting_transforms_bodywork_pixels ... ok
test test_classic_cars_dual_sprites_showroom_and_chassis ... ok
test test_backdrop_ground_pass_execution ... ok
test test_classic_mode_bot_color_schemes_distinct_from_player_sprite ... ok
test test_track_render_execution_under_all_quality_tiers_headless_safety ... ok
test test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory ... ok

test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/render_tests.rs (target/debug/deps/render_tests-a27b5a0904c78d77)

```
