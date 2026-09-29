---
type: Changelog
title: "Shared Racing Platform Changelog"
description: "What changed in the shared crates (wheelbase, arcade-race-core, race-kit, race-ui, cabinet) per platform tag."
status: active
---

# Shared Racing Platform Changelog

The five shared crates share one version and one git tag, `platform-vX.Y.Z`. A game repo pins the
tag for all of them. The plan is in [spec 049](../../specs/049_reusable_racing_platform_layers.md).

## platform-v0.1.0 (2026-09-29)

First tag a separate game repo can use.

### arcade-race-core
- `Body2D` trait: wall and vehicle collision, LIDAR, the progress tracker and the pit-box check
  work for any rigid body. The wall broad phase uses the body's own hull, so long bodies such as a
  chariot team collide. ([spec 054](../../specs/054_body2d_trait_for_vehiclegeneric_collision_and_progress.md))
- Golden state hash test (`tests/golden_sim.rs`) and tracks-free collision and LIDAR benches.
  ([spec 049](../../specs/049_reusable_racing_platform_layers.md))

### wheelbase
- Physics bench moved here, tracks-free. ([spec 049](../../specs/049_reusable_racing_platform_layers.md))

### race-kit (new)
- `RaceWorld`: one race step, laps and time-attack formats, finish order with real times,
  projected times for vehicles still racing, wrecks and DNF, and `RaceEvent`s.
- `Vehicle` trait, implemented for `wheelbase::Car`.
- `race_kit::ai`: the bot driver, driver tiers and styles, and the human layer.
  ([spec 056](../../specs/056_racekit_headless_race_world.md))

### race-ui (new)
- Track, barrier, scenery and surface renderers, `set_asset_root`, the race camera (follows any
  `Body2D`), particles, skid marks, drift popups, the curve indicator and four HUD widgets.
  ([spec 058](../../specs/058_raceui_rendering_camera_effects_and_hud_primitives.md))
- `examples/minimal_race.rs` and a README. ([spec 059](../../specs/059_publishready_shared_crates.md))

### cabinet
- `input::set_app_id`: gamepad profile files are searched in the game's own folders.
  ([spec 059](../../specs/059_publishready_shared_crates.md))

### Known limits
- A git dependency on this repo also fetches the private `tdrace-tracks` submodule.
- `EffectsManager`, skid marks and the bot AI are still typed to `wheelbase::Car`.
