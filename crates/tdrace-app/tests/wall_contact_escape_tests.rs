//! tdrace-bw6rb: a car resting against a wall can drive away from it.
//!
//! On the kouvola_rx joker, bots that ran wide in the right-hander stopped against the outer tyre
//! wall, on the grass beside the dirt road, and stayed there. The wall bends across the car's
//! path, so the car scrapes it whichever way it drives, and the tyre wall's scraping brake held it
//! to about 0.2 m/s on grass. With the walls taken away the same car pulls away at once. The brake
//! now starts at 2 m/s: the car scrapes along the wall at ~2 m/s until it is clear (measured
//! 8-10 m in 6 s; before, under 1 m).

use glam::Vec2;
use race_kit::{RaceRules, RaceWorld};
use tdrace_app::module::rally::RallyGameModule;
use tdrace_core::catalog;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;

const DT: f32 = 1.0 / 120.0;
/// Where a bot stopped against the outer wall of the Kouvola joker, and its heading.
const STALL_POS: Vec2 = Vec2::new(48.7, 122.0);
const STALL_HEADING_DEG: f32 = -9.0;
/// Time the car has to get away (s), and how far it must get.
const DRIVE_S: f32 = 6.0;
const MIN_ESCAPE_M: f32 = 6.0;

/// How far the Rallycross car gets from the stall pose in `DRIVE_S` with `controls`.
fn distance_driven(controls: CarControls) -> f32 {
    let track = catalog::find("kouvola_rx", Some("rally")).unwrap().load().unwrap();
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
    let car = Car::new(RallyGameModule::car_wrc_rally()).with_pose(STALL_POS, STALL_HEADING_DEG.to_radians());
    world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
    for _ in 0..(DRIVE_S / DT) as usize {
        world.step(&track, &[controls.into()], DT);
    }
    world.vehicles[0].state.position.distance(STALL_POS)
}

/// Scenario: A car against the Kouvola joker's tyre wall drives away
///
/// Given a Rallycross car at rest on the grass against the outer tyre wall of the kouvola_rx joker
/// When it drives off forwards, or in reverse, at full throttle with the wheel straight
/// Then it gets at least 6 m away within 6 s
#[test]
fn test_car_against_the_kouvola_joker_tyre_wall_drives_away() {
    let fwd = distance_driven(CarControls { throttle: 1.0, steer: 0.0, brake: 0.0, handbrake: false, reverse: false });
    let rev = distance_driven(CarControls { throttle: 1.0, steer: 0.0, brake: 0.0, handbrake: false, reverse: true });
    assert!(fwd >= MIN_ESCAPE_M, "forwards escape: {fwd:.2} m < {MIN_ESCAPE_M} m");
    assert!(rev >= MIN_ESCAPE_M, "reverse escape: {rev:.2} m < {MIN_ESCAPE_M} m");
}

#[test]
fn test_bot_at_kouvola_joker_wall() {
    let track = catalog::find("kouvola_rx", Some("rally")).unwrap().load().unwrap();
    let pos = Vec2::new(81.1, 94.9);
    let angle = (-74.1f32).to_radians();
    let mut driver = tdrace_app::ai::bot_harness::sample_bot(
        tdrace_app::ai::DrivingStyle::Calculating,
        tdrace_app::ai::DriverTier::Pro,
        58,
    );
    let car = Car::new(RallyGameModule::car_wrc_rally()).with_pose(pos, angle);
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
    world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));

    for step in 0..4800 {
        let prev_turn = driver.turn;
        let ctrl = driver.compute_controls(&world.vehicles[0], &track, &[], DT);
        world.step(&track, &[ctrl.into()], DT);
        let c = &world.vehicles[0];
        if driver.turn != prev_turn || step % 240 == 0 {
            println!("t={:.2}s: pos=({:.2}, {:.2}), vel={:.2}, ctrl={:?}, turn={:?}, attempts={}",
                step as f32 * DT, c.state.position.x, c.state.position.y, c.state.speed, ctrl, driver.turn, driver.recovery_attempts);
        }
    }
}













