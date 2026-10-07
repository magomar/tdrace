//! Golden state hash for a scripted multi-car race on generated tracks.
//!
//! Spec 049 (`specs/049_reusable_racing_platform_layers.md`), Phase 0. Later phases move and
//! generalize the engine code (a `Body2D` trait, the `race-kit` world). This test pins the
//! exact floating-point result of today's step order, so a refactor that changes physics
//! results fails here. It needs no `tracks/` checkout: both tracks are generated.
//!
//! The step order is the one in `crates/tdrace-app/src/ai/bot_harness.rs`:
//! surfaces with hint, draft, road projection, per-wheel step, car-car collisions,
//! walls (inner + scenery, then outer), progress tracker.
//!
//! If a change is meant to alter physics results, re-record the constants below in a
//! commit of its own and say why in the message.

use arcade_race_core::collision::{resolve_all_wall_collisions, resolve_multi_car_collisions};
use arcade_race_core::track::{create_prototypical_track, RaceDirection, Track, TrackProgressTracker, TrackShape};
use glam::Vec2;
use wheelbase::{Car, CarConfig, CarControls};

const DT: f32 = 1.0 / 120.0;
const STEPS: usize = 3600;
const CARS: usize = 6;

/// Recorded hashes as `(oval, figure-eight)`, per platform and build mode.
///
/// `sin`, `cos` and `atan2` come from the platform math library, so results differ between
/// operating systems. They also differ between debug and optimized builds on macOS: the
/// optimizer merges a `sin` and `cos` of the same angle into one `__sincosf_stret` call,
/// which rounds differently (measured 2026-09-28). `debug_assertions` stands in for "not
/// optimized" here. On a platform with no recorded value the test checks only that two runs
/// in one process agree, and prints the hash to record.
fn recorded() -> Option<(u64, u64)> {
    let mac_arm = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    match (mac_arm, cfg!(debug_assertions)) {
        (true, true) => Some((0x2c6e253755a8b0c8, 0x60f0393864830551)),
        (true, false) => Some((0xd0dc9c0f29c664f5, 0xa4743b4a9ce86a50)),
        _ => None,
    }
}

struct RunSummary {
    hash: u64,
    wall_events: usize,
    car_events: usize,
}

fn fnv(h: u64, v: u64) -> u64 {
    (h ^ v).wrapping_mul(0x100000001B3)
}

/// Deterministic controls: full-width steering sweeps at speed, so cars reach the walls and
/// each other. Car 1 and car 2 steer toward each other at the start to force early contact.
fn scripted_controls(car: usize, step: usize) -> CarControls {
    let t = step as f32 * DT;
    let phase = car as f32 * 1.3;
    let mut steer = (t * (0.45 + 0.08 * car as f32) + phase).sin() * 0.9;
    if step < 240 {
        if car == 1 {
            steer = 0.6;
        } else if car == 2 {
            steer = -0.6;
        }
    }
    let throttle = 0.75 + 0.25 * (t * 0.3 + phase).sin();
    let brake = if (t * 0.21 + phase).sin() > 0.93 { 0.8 } else { 0.0 };
    let handbrake = (t * 0.17 + phase).cos() > 0.97;
    CarControls::new(throttle, steer, brake, handbrake)
}

fn run(track: &Track) -> RunSummary {
    assert!(track.grid_positions.len() >= CARS, "track needs {} grid slots", CARS);
    let mut cars: Vec<Car> = (0..CARS)
        .map(|i| {
            let slot = track.grid_positions[i];
            Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle)
        })
        .collect();
    let mut trackers: Vec<TrackProgressTracker> =
        (0..CARS).map(|_| TrackProgressTracker::new(track.checkpoints.len(), 3)).collect();
    let scenery = track.geometry.all_obstacles_with_scenery();

    let mut hash: u64 = 0xcbf29ce484222325;
    let mut wall_events = 0usize;
    let mut car_events = 0usize;

    for step in 0..STEPS {
        let controls: Vec<CarControls> = (0..CARS).map(|i| scripted_controls(i, step)).collect();

        let surfaces: Vec<_> =
            (0..CARS).map(|i| track.sample_car_surfaces_with_hint(&cars[i], trackers[i].progress_distance)).collect();
        let drafts: Vec<f32> = (0..CARS)
            .map(|i| {
                let others: Vec<&Car> = cars.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
                cars[i].compute_draft_intensity(&others)
            })
            .collect();
        for i in 0..CARS {
            cars[i].state.draft_intensity = drafts[i];
            let proj = track.spline.project_point_continuity(cars[i].state.position, trackers[i].progress_distance, 50.0);
            cars[i].state.road_elevation = proj.elevation;
            cars[i].state.road_bank_angle = proj.bank_angle;
            cars[i].state.road_grade_slope = proj.grade_slope;
            cars[i].state.road_vertical_curvature = proj.vertical_curvature;
            cars[i].state.track_right = Vec2::new(proj.tangent.y, -proj.tangent.x);
            cars[i].state.track_forward = proj.tangent;
            cars[i].step_per_wheel(&controls[i], surfaces[i], DT);
        }
        car_events += resolve_multi_car_collisions(&mut cars, 0.45, 0.35, 3).len();
        for car in &mut cars {
            wall_events += resolve_all_wall_collisions(car, &track.geometry.inner_walls, &scenery).len();
            wall_events += resolve_all_wall_collisions(car, &track.geometry.outer_walls, &[]).len();
        }
        for i in 0..CARS {
            trackers[i].update(&cars[i], &track.spline, &track.checkpoints, DT);
        }

        for (car, tracker) in cars.iter().zip(&trackers) {
            let s = &car.state;
            for v in [
                s.position.x,
                s.position.y,
                s.angle,
                s.velocity.x,
                s.velocity.y,
                s.angular_velocity,
                s.speed,
                tracker.progress_distance,
            ] {
                assert!(v.is_finite(), "non-finite state at step {}", step);
                hash = fnv(hash, v.to_bits() as u64);
            }
            hash = fnv(hash, tracker.current_lap as u64);
            hash = fnv(hash, tracker.next_checkpoint_idx as u64);
        }
    }
    RunSummary { hash, wall_events, car_events }
}

fn check(name: &str, track: &Track, golden: Option<u64>) {
    let first = run(track);
    assert!(first.wall_events > 0, "{}: script must hit a wall", name);
    assert!(first.car_events > 0, "{}: script must make two cars touch", name);
    match golden {
        Some(golden) => assert_eq!(
            first.hash, golden,
            "{}: state hash changed (got {:#018x}, recorded {:#018x}; wall events {}, car events {})",
            name, first.hash, golden, first.wall_events, first.car_events
        ),
        None => {
            let second = run(track);
            assert_eq!(first.hash, second.hash, "{}: two runs in one process differ", name);
            eprintln!("{}: no recorded hash for this platform; got {:#018x}", name, first.hash);
        }
    }
}

#[test]
fn golden_oval() {
    check("oval", &create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right), recorded().map(|r| r.0));
}

#[test]
fn golden_figure_eight() {
    check(
        "figure-eight",
        &create_prototypical_track("gt", TrackShape::HorizontalEight, RaceDirection::Left),
        recorded().map(|r| r.1),
    );
}
