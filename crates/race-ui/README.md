# race-ui

Drawing primitives for top-down racing games, on `macroquad`: track, barrier, scenery and surface
renderers, the race camera, particles and skid marks, the curve indicator and basic HUD widgets.
Spec: [`specs/058_raceui_rendering_camera_effects_and_hud_primitives.md`](../../specs/058_raceui_rendering_camera_effects_and_hud_primitives.md).

## Use it from another repo

```toml
[dependencies]
race-ui = { git = "https://github.com/magomar/tdrace", tag = "platform-v0.1.0" }
race-kit = { git = "https://github.com/magomar/tdrace", tag = "platform-v0.1.0" }
```

Cargo also fetches the private `tdrace-tracks` submodule; see the note in the
[`race-kit` README](../race-kit/README.md).

## A whole race in one file

[`examples/minimal_race.rs`](examples/minimal_race.rs) runs 4 bot cars for 3 laps on a generated
oval, with the follow camera, effects, a lap timer and a minimap:

```sh
cargo run -p race-ui --example minimal_race
```

## Draw order for one frame

1. `clear_background(render::track::get_track_backdrop_color(track.default_surface))`
2. `camera.apply()`, then `let view = Some(camera.visible_world_bounds(12.0))`
3. `render::track::render_ground_track_culled(&track, view)`
4. `fx.render_ground_fx_culled(view)`
5. `render::barrier::render_ground_barriers_and_obstacles_culled(&track, view)`
6. your vehicles
7. `fx.render_airborne_fx()`, then `render_elevated_track_culled` and
   `render_elevated_barriers_and_obstacles_culled` for bridges and overpasses
8. `camera.reset_to_screen()`, then the HUD: `hud::widgets::{render_position_and_lap,
   render_lap_timer, render_minimap}`

Feed the effects once per simulation step with
`fx.update(&world.vehicles, &world.last_surfaces, &wall_events, &vehicle_events, dt)`, taking the
two event lists from the `race-kit` step.

## Camera

`camera::RaceCamera` follows any `Body2D`: call `setup_for_track`, then `resume_from_pause(Some(&vehicle))`
to start in follow mode, then `update(&vehicle, frame_dt)` each frame.

## Your own textures

Surface textures load from `assets/textures/surfaces/` near the working directory. Call
`render::set_asset_root("path/to/assets")` to look in your game's folder first.

`EffectsManager` and the skid marks still need a `wheelbase::Car`; vehicle sprites are not part of
this crate yet.
