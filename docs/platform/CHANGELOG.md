---
type: Changelog
title: "Shared Racing Platform Changelog"
description: "What changed in the shared crates (wheelbase, arcade-race-core, race-kit, race-ui, cabinet) per platform tag."
status: active
---

# Shared Racing Platform Changelog

The five shared crates share one version and one git tag, `platform-vX.Y.Z`. A game repo pins the
tag for all of them. The plan is in [spec 049](../../specs/049_reusable_racing_platform_layers.md).

## platform-v0.3.0 (2026-10-02)

The reusable `cabinet::ui` component catalog and unified nav-intent/key-repeat input layer ship,
TdRace migrates all its legacy screens onto them, and track splines gain centripetal Catmull-Rom
and variable-density sampling. ([spec 068](../../specs/068_unified_ui_and_ux_consistency_across_tdrace_and_shared_platform_crate.md), [spec 069](../../specs/069_legacy_ui_migration_and_reusable_platform_component_adoption.md), [spec 071](../../specs/071_centripetal_catmullrom_and_variable_density_track_splines.md))

### cabinet
- `ui` component catalog (spec 068): layout containers (`HStack`, `VStack`, `CardGrid<T>`, `GridLayout`, `FlowLayout`, `SplitPane`, `ScrollIndicator`, `Accordion<T>`), selectors (`FilterBar`, `TabBar`, `DropdownWidget`, `OptionCycler<T>`, `RadioGroup<T>`, `Toggle`, `SwatchPicker`), counters (`ValueStepper<T>`, `Counter`, `SliderWidget`), and data & feedback surfaces (`DataTable<T>`, `MetricBar`, `KpiTile`, `ProgressBar`, `ToastOverlay`, `TextInputWidget`, `ChecklistModal<T>`, `ModalContainer`, `CountDown`, `Tooltip`, `HelpChip`, `PageDots`, `HeroActionButton`, `ScreenFooter`), all with gamepad/keyboard/mouse parity.
- `input` unified nav layer (spec 068): `NavIntent`/`NavAction` and `KeyRepeat` hold-to-repeat.
- `net::ui::VirtualKeypad` (spec 068, completed in 069): on-screen IP and alphanumeric keypad with `draw` and gamepad/mouse `handle_input`; the LAN host/join/client lobby screens migrated to `SplitPane`, `VStack`, `SwatchPicker`, `Toggle`, and `ValueStepper` (spec 069).

### arcade-race-core
- Centripetal Catmull-Rom and variable-density track spline sampling (spec 071).
- Classic circuit revamp: collinear wall merging and barrier optimization (spec 055).
- `LicenseGrade` hierarchy with per-category entry requirements on the profile model (spec 053/060).

### race-ui
- Scenery and track ribbon renderers (spec 055/071), plus reworked skidmark and tire-smoke effects.

## platform-v0.2.0 (2026-09-29)

Any vehicle model can use the bot driver and the effects, so the chariot game can bring its own
chariot physics. ([spec 065](../../specs/065_vehiclegeneric_bot_ai_and_effects.md))

### race-kit
- `ai::BotVehicle` (right vector, top speed, grip, and `planning_grip`, the grip the bot plans
  corner speeds with; 0.78 by default, as before). `BotAiDriver::compute_controls` and the human
  layer are generic over it. `wheelbase::Car` implements it; car bots drive exactly as before.

### race-ui
- Depends on `cabinet` without default features, so a game's web build does not pull `gilrs`
  and `wasm-bindgen` (they broke macroquad's plain JS loader).
- `fx::FxVehicle` (four contact points and their slip data, right vector, airborne, drift).
  `EffectsManager::update` and the skid marks are generic over it. `wheelbase::Car` implements it.

### cabinet
- Builds without its `gamepad` feature (`GamepadManager::is_button_down` is behind it now).

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
- `EffectsManager`, skid marks and the bot AI are still typed to `wheelbase::Car` (fixed in
  platform-v0.2.0).
