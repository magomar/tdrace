//! The `golden_sim` race, run through `RaceWorld<Car>`.
//!
//! Spec 056 (`specs/056_racekit_headless_race_world.md`). `crates/arcade-race-core/tests/golden_sim.rs`
//! pins the step order with a hand-written loop. This test runs the same race, with the same
//! scripted controls, through `RaceWorld::step`, and must give the same hashes. So the world
//! reproduces today's physics bit for bit.
//!
//! The recorded values below are copies of the ones in `golden_sim.rs`. Change them only
//! together.

use arcade_race_core::track::{create_prototypical_track, RaceDirection, Track, TrackProgressTracker, TrackShape};
use race_kit::{RaceEvent, RaceRules, RaceWorld};
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
        (true, true) => Some((0x0795362422c932f4, 0x013e17911dd54576)),
        (true, false) => Some((0x70135423dc5f2600, 0x8f076569eace3b96)),
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
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
    for i in 0..CARS {
        let slot = track.grid_positions[i];
        world.spawn(
            Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
    }

    let mut hash: u64 = 0xcbf29ce484222325;
    let mut wall_events = 0usize;
    let mut car_events = 0usize;

    for step in 0..STEPS {
        let controls: Vec<CarControls> = (0..CARS).map(|i| scripted_controls(i, step)).collect();
        for ev in world.step(track, &controls, DT) {
            match ev {
                RaceEvent::WallImpact { .. } => wall_events += 1,
                RaceEvent::VehicleImpact(_) => car_events += 1,
                _ => {}
            }
        }

        for (car, tracker) in world.vehicles.iter().zip(&world.trackers) {
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
