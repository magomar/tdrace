---
type: Architecture Spec
title: "Racing Platform Reuse Analysis"
description: "As-built analysis of how reusable the TdRace crates are for new top-down racing games (chariot races, streamed rally-raid), with the gaps that block reuse."
status: active
category: engineering
tags: [architecture, reuse, crates, chariot, rally-raid, analysis]
---

# Racing Platform Reuse Analysis

This report describes how reusable the workspace is **today** for new top-down 2D
racing games. It was written for two planned games, each in its own repo:

- **Roman chariot races** (priority 1): a horse-drawn chariot on a closed oval. Crashes
  and wrecks matter.
- **Rally-raid** (priority 2): endless top-down scrolling. Stages are generated on the
  fly from GPX data plus procedural detail. There are no laps.

The plan that follows from this analysis is
[spec 049](../../specs/049_reusable_racing_platform_layers.md).

Line numbers are for commit `57c1635` (2026-09-28). `main` at `239f16f` differs only in
these crate files:

- `game/mod.rs` (+51 lines)
- `fx/`
- `render/marker.rs`
- `ui/curve_indicator.rs`
- `cabinet/src/state/settings.rs`

Line numbers in those files may be a few lines off.

---

## 0. Summary

1. **The engine crates are a good start but are typed on one vehicle.** Every
   collision, LIDAR, progress and surface API takes a concrete 4-wheel `wheelbase::Car`.
   There is no vehicle trait.
2. **The spline can already be open, but everything above it assumes a loop.** That
   covers checkpoint and grid generation, bake, validation and the lap tracker. No tool
   can produce an open course.
3. **The whole track must be in memory.** There is no chunking, no spatial index and no
   floating origin. Many queries scan every sample or every wall. This blocks an endless,
   streamed course.
4. **There is no headless race world.** `RaceSession` in `game/mod.rs` mixes simulation,
   rules, screens, audio, FX, network and saving. The step order exists in three copies:
   the game, the bot harness and the Python engine.
5. **Race rules only know laps and the player.** Bots never finish, result times are
   invented, and there is no DNF.
6. **`cabinet` is mostly generic.** Its settings modal and LAN protocol are
   tdrace-specific.
7. **Rendering, camera and AI assume a 4-wheel car on a closed loop.**

---

## 1. Crates

| Crate | Lines of code | Role | Depends on |
|---|---|---|---|
| `wheelbase` | 11.8k | Car and bike Pacejka physics, surfaces, sim harness, CMA-ES tuner | glam, serde |
| `arcade-race-core` | 9k | Track spline, geometry, checkpoints, SAT collision, LIDAR | wheelbase |
| `tdrace-core` | 92 | Facade, plus `build.rs` that embeds the `tracks/` submodule | both above |
| `cabinet` | 16.3k | Arcade shell: screens, UI, FX, audio, profiles, input, LAN | macroquad |
| `tdrace-app` | 78k | The game. `game/mod.rs` alone is 14.3k lines. | tdrace-core, cabinet |
| `tdrace-py` | 1.7k | Gymnasium bindings | tdrace-core |

[Spec 005](../../specs/005_modular_racing_architecture.md) extracted `wheelbase` and
`arcade-race-core` for a family of games. What it left open is below.

---

## 2. Engine gaps (`wheelbase`, `arcade-race-core`)

| # | Gap | Where |
|---|---|---|
| E1 | No vehicle trait. `Car` is a concrete 4-wheel, engine-driven struct: `[_;4]` wheels, Ackermann front axle, engine force, differentials. `Motorbike` duplicates the pose helpers, and it is the only model with crash states. `VehicleControls` from spec 005 does not exist. | `wheelbase/src/car.rs:165,168,272,406-431`; `bike.rs:136-139` |
| E2 | Every engine API takes `&wheelbase::Car`. The functions only read pose, velocity, angular velocity, cached speed, mass, inertia, elevation and hull size. They only write through three raw adders. | `collision/wall.rs:43,266,367`; `collision/car_collision.rs:20,141`; `collision/sat.rs:25`; `lidar/mod.rs:157`; `track/checkpoint.rs:214`; `track/geometry.rs:942`; `track/mod.rs:400,521` |
| E3 | The broad phase hardcodes `car_reach = 3.6`. A chariot with a four-horse team is about 8-10 m long, so its collisions would be skipped. | `collision/wall.rs:62` |
| E4 | The spline supports `closed: false`, but the layers above assume a loop. The finish is placed at distance 0. The grid walks backwards and collapses on open splines. Bake and validation require a finish line. The tracker wraps its checkpoint index, starts its clock at construction, and has no start gate. | `spline.rs:231,709-717`; `presets.rs:465,493-521`; `bake.rs:102-110`; `validation.rs:399-407`; `checkpoint.rs:96-145,225,280,305,330-362` |
| E5 | The whole track is held in memory. `project_point` is brute force. `SurfaceSampler for Track` projects twice per call, and a car step calls it 5 times. Every off-track query loops over the walls. Bridge detection, trimming and validation are O(N²). Collision and LIDAR scan every wall. Coordinates are `f32`. | `geometry.rs:996-1010`; `spline.rs:826-847`; `track/mod.rs:506-507,626-641` |
| E6 | Spec 006 (branching `TrackNetwork`) says `implemented`, but its code sits only on the unmerged branch `origin/feat/road-split-branching-tracks`. That branch is 456 commits behind `main`. | Beads `tdrace-m2jk`; [circuit analysis D10](circuit_building_analysis.md) |
| E7 | tdrace product categories live inside the engine: `CarCategory` with hardcoded car ids, `default_laps`, `module_id`, runoff guessed from track names, module-keyed barrier offsets and grid counts, and module-keyed prototype tracks. | `car_category.rs`; `track/mod.rs:535-603,618-662,826-852`; `presets.rs:776-861` |
| E8 | `SurfaceType` is a closed enum. `Car` reads friction from the enum and ignores the numeric `SurfaceProperties`. | `surface.rs:6-38`; `car.rs:573-582,643` |
| E9 | `tdrace-core/build.rs` hardcodes `../../tracks` and embeds every tdrace circuit in anything that depends on it. | `tdrace-core/build.rs:29` |
| E10 | Weak determinism net. The `.tdr` replay check re-simulates only the player car, with no walls, opponents or trackers. The benches assert 500k steps/s, but the roadmap target is 4.0M. | `replay/mod.rs:313-371`; `tdrace-core/benches/physics_bench.rs:57` |
| E11 | Physics is not bit-identical across build modes or platforms. On macOS the release optimizer merges `sin` and `cos` of one angle into `__sincosf_stret`, which rounds differently. A debug replay therefore diverges in release. Spec 005's claim of cross-platform bit-identity does not hold for sine and cosine. Measured 2026-09-28. | `arcade-race-core/tests/golden_sim.rs`; Beads `tdrace-d3m7` |

---

## 3. Shell gaps (`cabinet`)

**Generic today:**
- `ScreenStack`, `CabinetScreen`, `ScreenAction` and `CabinetContext` (`state/stack.rs:98-112`)
- the pause, confirm, leaderboard and profile modals
- all of `ui`, all of `fx`, `profile`, `records` and `audio`
- `NavGrid2D`, `InputMap::default_arcade` and `GamepadManager`

Proof: `crates/cabinet/examples/space_arena.rs`.

| # | Gap | Where |
|---|---|---|
| S1 | `ArcadeSettingsModal` is tdrace's settings screen: fixed tabs, racing options, no way to extend it. | `state/settings.rs` (2340 lines) |
| S2 | `cabinet::net` is a racing protocol. The `Packet` enum is closed, input is steer/throttle/brake, snapshots carry laps, and `laps.max(1)` is forced. It defaults to `gt_ferrari_296_gt3` and `monza`. The magic `TDLN` and ports 7776/7777 are shared by every game, and there is no game id. | `net/protocol.rs:383-399`; `net/host.rs:168,311` |
| S3 | Gamepad profile search paths are hardcoded to tdrace folders. | `input/gamepad.rs:283-298` |
| S4 | tdrace does not use `ScreenStack`. It runs on a 30-variant `GameState` enum, and there is no `impl CabinetScreen` in the app. No reusable race flow (garage, grid, countdown, HUD, results) exists. | `tdrace-app/src/game/mod.rs:299` |

---

## 4. Data and build gaps

| # | Gap | Where |
|---|---|---|
| D1 | No tool can produce an open course. 93 of 96 tracks are closed. The 3 open ones are arenas with no waypoints. Both importers force a closed loop. | `scripts/osm_importer.py:2425`; `scripts/gpx_importer.py:98-120` |
| D2 | Series, config and storage live in the app. They compile TOML in with `include_str!` and hardcode the `tdrace` user folder. `StageRallySession` exists but nothing uses it. | `series/format.rs`; `config.rs`; `storage.rs`; `series/mod.rs:560-635` |
| D3 | `tdrace-py` loads catalog circuits only, wraps progress for loops, and ends an episode on the lap count. | `tdrace-py/src/engine.rs:98-110,328-386` |
| D4 | Asset paths are hardcoded as `format!("assets/...")`. | `render/vehicle_assets.rs:33`; `render/surface_material.rs:248`; `render/trophy_textures.rs:43` |

---

## 5. App gaps (`tdrace-app`)

| # | Gap | Where |
|---|---|---|
| A1 | `RaceSession` has 170+ fields and one 13.6k-line impl. `physics_step` is about 35% simulation and 65% presentation. The bot harness has a second copy of the step order (no canopy drag, no ramps, but it finishes every car). The Python engine has a third. | `game/mod.rs:498-680,11059-11792`; `ai/bot_harness.rs:118-164`; `tdrace-py/src/engine.rs:295-323` |
| A2 | Only the player can finish. Bots never do. Result times are `session_time + rank*0.65`. There is no DNF and no penalties. | `game/mod.rs:11795-11805,11887,12149` |
| A3 | Disciplines are strings with no registry. There are about 48 `match module_id` sites in 15 files and 218 arms. About 15 hand-kept lists have already drifted, and unknown ids fall back to `"classic"`. The closed enums `CarChoice`, `VehicleVisualType`, `EngineSoundType` and `ModuleFilter` make it worse. | `module/mod.rs:528-561`; `tracks/catalog.rs:31`; `tracks/dev_store.rs:91` |
| A4 | Dependency cycles: module ↔ ui::menu, catalog ↔ ui::menu, ai ↔ module. The domain types `TrackChoice`, `CarChoice` and `GameMode` live in a UI file, and 19 files import them. | `ui/menu.rs:22,329,668` |
| A5 | The AI assumes a closed loop: look-ahead with `% total_length`, a `rem_euclid` watchdog, and corner search that wraps. It also reads `Car` fields directly. | `ai/mod.rs:519,595,617,622,627,644,648`; `ai/humanize.rs:207-274` |
| A6 | Rendering assumes a 4-wheel car. There are no multi-part sprites. Meshes are rebuilt every frame. The minimap always closes the loop. | `render/car.rs:304,453`; `render/track.rs:101,200-211`; `ui/hud.rs:339` |
| A7 | The camera has only follow and overview modes, and rotation is pinned to 0. | `camera/mod.rs:14-19,480,503` |
| A8 | The gamepad code in the app is a fork of cabinet's. `game/mod.rs` has 555 raw `KeyCode::` uses. | `input/gamepad.rs` vs `cabinet/src/input/gamepad.rs` |
| A9 | Audio assumes an engine: an RPM gearbox model inside `game/mod.rs`, and a closed `SfxType` with no hooves, whip or crowd. | `game/mod.rs:686-750`; `audio/manager.rs:44,189-219` |
| A10 | Content lives in code: the car roster, the career calendars (six copies) and the unlock lists. | `catalog/mod.rs:553-2793`; `game/mod.rs:2407-3346`; `profile/mod.rs:591-786` |

---

## 6. Already reusable

- **Engine:**
  - the open-capable spline
  - `TrackCurve.degree`, usable for rally pace notes
  - the oval template (`presets.rs:637`), close to a Circus Maximus shape
  - the SAT math and the tire free functions (`tire.rs:144-272`)
  - `WheelAssembly` and `SurfaceSampler`
  - the fixed 1/120 s step and the CMA-ES tuner
- **Shell:** all of `cabinet` except S1-S3.
- **App:**
  - the fixed-step accumulator
  - `RaceCamera` follow mode and split-screen viewports
  - layered TOML config (`config.rs:536-676`)
  - the series TOML format and point systems
  - `TrackManager`
  - procedural surface textures, scenery, barrier and marker renderers, and the ghost
  - skidmarks and particles
  - the AI personality layer (`DriverTier`, `DrivingStyle`, `humanize.rs`)
  - the replay recorder and SQLite storage with its WASM fallback

---

## 7. What each new game needs

| Need | Chariot | Rally-raid |
|---|---|---|
| Non-car vehicle behind a trait (E1, E2, E3) | yes | no (uses `Car`) |
| Headless world with per-car finish, DNF and real times (A1, A2) | yes | yes |
| Wreck or DNF on impact | yes | optional |
| Multi-part sprites (A6) | yes | no |
| Chase camera (A7) | optional | yes |
| Open courses (E4, D1, A5) | no | yes |
| Streaming, floating origin, spatial queries (E5) | no | yes |
| Shared crates buildable without `tracks/` (E9) | yes | yes |
