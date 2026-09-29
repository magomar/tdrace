//! A whole race with only the shared crates: 4 bot cars, 3 laps, on a generated oval.
//!
//! Spec 059 (`specs/059_publishready_shared_crates.md`). Run it with
//! `cargo run -p race-ui --example minimal_race`. The cars are drawn as coloured boxes; a real
//! game brings its own vehicle sprites.

use arcade_race_core::collision::{CarCarCollisionEvent, WallCollisionEvent};
use arcade_race_core::track::{create_prototypical_track, RaceDirection, TrackProgressTracker, TrackShape};
use cabinet::ui::font::Fonts;
use cabinet::ui::scaler::UiScaler;
use macroquad::prelude::*;
use race_kit::ai::{BotAiDriver, BotProfile};
use race_kit::{DriveControls, RaceEvent, RaceFormat, RaceRules, RaceWorld};
use race_ui::camera::RaceCamera;
use race_ui::fx::EffectsManager;
use race_ui::hud::widgets::{render_lap_timer, render_minimap, render_position_and_lap};
use race_ui::render::color::CarColorScheme;
use race_ui::render::{barrier, track as track_render};
use wheelbase::{Car, CarConfig};

const DT: f32 = 1.0 / 120.0;
const CARS: usize = 4;
const LAPS: u32 = 3;

#[macroquad::main("race-ui minimal race")]
async fn main() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(LAPS), ..RaceRules::default() });
    let mut drivers = Vec::new();
    let profiles = [BotProfile::pro(), BotProfile::aggressive(), BotProfile::smooth(), BotProfile::rookie()];
    for (i, profile) in profiles.into_iter().enumerate().take(CARS) {
        let slot = track.grid_positions[i];
        world.spawn(
            Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
        drivers.push(BotAiDriver::with_seed(profile, 7 + i as u64));
    }
    let colors: Vec<CarColorScheme> = (0..CARS).map(CarColorScheme::from_index).collect();

    let fonts = Fonts::load_embedded();
    let mut camera = RaceCamera::new();
    camera.setup_for_track(&track);
    camera.resume_from_pause(world.vehicles.first());
    let mut fx = EffectsManager::new(4000, 2000);
    let mut accumulator = 0.0;

    loop {
        // Fixed-step simulation: the bots drive, the world steps, the effects follow the events.
        accumulator += get_frame_time().min(0.1);
        while accumulator >= DT {
            accumulator -= DT;
            let controls: Vec<DriveControls> = (0..CARS)
                .map(|i| {
                    let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
                    drivers[i].compute_controls(&world.vehicles[i], &track, &others, DT)
                })
                .collect();
            let events = world.step(&track, &controls, DT).to_vec();
            let walls: Vec<WallCollisionEvent> =
                events.iter().filter_map(|e| if let RaceEvent::WallImpact { event, .. } = e { Some(*event) } else { None }).collect();
            let hits: Vec<CarCarCollisionEvent> =
                events.iter().filter_map(|e| if let RaceEvent::VehicleImpact(c) = e { Some(*c) } else { None }).collect();
            fx.update(&world.vehicles, &world.last_surfaces, &walls, &hits, DT);
        }
        camera.update(&world.vehicles[0], get_frame_time());

        // World: ground, ground effects, barriers, cars, then the elevated parts on top.
        clear_background(track_render::get_track_backdrop_color(track.default_surface));
        camera.apply();
        let view = Some(camera.visible_world_bounds(12.0));
        track_render::render_ground_track_culled(&track, view);
        fx.render_ground_fx_culled(view);
        barrier::render_ground_barriers_and_obstacles_culled(&track, view);
        for (car, color) in world.vehicles.iter().zip(&colors) {
            let length = car.config.cg_to_front + car.config.cg_to_rear;
            let params = DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: car.state.angle, color: color.primary };
            draw_rectangle_ex(car.state.position.x, car.state.position.y, length, car.config.track_width, params);
        }
        fx.render_airborne_fx();
        track_render::render_elevated_track_culled(&track, view);
        barrier::render_elevated_barriers_and_obstacles_culled(&track, view);

        // HUD for car 0.
        camera.reset_to_screen();
        let (sw, sh) = (screen_width(), screen_height());
        let scaler = UiScaler::new(sw, sh);
        let position = world.standings().iter().position(|&i| i == 0).unwrap_or(0) + 1;
        let lap = world.trackers[0].current_lap.min(LAPS);
        render_position_and_lap(&fonts, &scaler, scaler.s(20.0), scaler.s(20.0), position, CARS, lap, LAPS, false);
        render_lap_timer(&fonts, &scaler, sw * 0.5, scaler.s(20.0), &world.trackers[0]);
        let map = scaler.s(220.0);
        render_minimap(&fonts, &scaler, sw - map - scaler.s(20.0), sh - map - scaler.s(20.0), map, map, &track, &world.vehicles, &colors);
        if (0..CARS).all(|i| world.is_finished(i)) {
            for (row, r) in world.results(&track).iter().enumerate() {
                let text = format!("P{}  car {}  {:.2} s", r.position, r.car, r.time);
                draw_text(&text, sw * 0.5 - 120.0, sh * 0.4 + 32.0 * row as f32, 32.0, WHITE);
            }
        }
        next_frame().await;
    }
}
