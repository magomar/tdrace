//! tdrace-joker-wall-ghost-ebrgn: a car beside a joker or launch chute wall must hit it.
//!
//! A wall stops a car only when the two stand within 1.8 m of each other in height. The joker
//! walls of rx_hilltop_leap take the height of the joker road, which climbs to 4.5 m, while the
//! main road beside it stays near 2.4 m. `RaceWorld` gave every car the main road height, so the
//! game drew the joker wall and a car drove through it.

use glam::Vec2;
use race_kit::{DriveControls, RaceRules, RaceWorld};
use tdrace_core::catalog;
use tdrace_core::physics::car::Car;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;
use tdrace_core::CarConfig;

const DT: f32 = 1.0 / 120.0;
/// Gap between the car centre and the wall at the start (m).
const START_GAP_M: f32 = 3.0;
/// Speed of the car into the wall (m/s).
const SPEED: f32 = 12.0;

/// How far `p` is outside the drivable ribbon of the road nearest to it.
fn off_road(track: &Track, p: Vec2) -> f32 {
    let proj = track.project_point(p);
    proj.distance_to_spline - proj.track_width * 0.5
}

/// Drives one car straight into the middle of every network wall of `track` from its road side.
/// Returns the walls the car passed through.
fn walls_crossed(track: &Track) -> Vec<String> {
    let mut crossed = Vec::new();
    for wall in track.geometry.network_walls.iter().filter(|w| w.is_physical() && w.segment.length() >= 1.0) {
        let (a, b) = (wall.segment.start, wall.segment.end);
        let mid = (a + b) * 0.5;
        let dir = (b - a).normalize();
        let mut n = Vec2::new(-dir.y, dir.x);
        if off_road(track, mid - n * START_GAP_M) < off_road(track, mid + n * START_GAP_M) {
            n = -n;
        }
        let start = mid + n * START_GAP_M;
        let heading = (-n).y.atan2(-n.x);

        let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
        let mut car = Car::new(CarConfig::sports_car()).with_pose(start, heading);
        car.state.velocity = -n * SPEED;
        let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
        tracker.progress_distance = track.spline.project_point(start).progress_distance;
        world.spawn(car, tracker);
        for _ in 0..60 {
            world.step(track, &[DriveControls::default()], DT);
        }
        let side = (world.vehicles[0].state.position - a).dot(n);
        if side < 0.0 {
            crossed.push(format!("({:.1},{:.1})-({:.1},{:.1}) e={:.2}: car ended {:.2} m past it", a.x, a.y, b.x, b.y, wall.elevation, -side));
        }
    }
    crossed
}

/// Scenario: A car drives into a wall of the Hilltop Leap joker or launch chute
///
/// Given rx_hilltop_leap, whose joker road climbs 2 m above the main road beside it
/// When a car drives at 12 m/s into the middle of each joker and launch chute wall
/// Then the car stays on the road side of every wall
#[test]
fn test_hilltop_leap_network_walls_stop_a_car() {
    let track = catalog::find("rx_hilltop_leap", Some("classic")).unwrap().load().unwrap();
    assert!(!track.geometry.network_walls.is_empty());
    let crossed = walls_crossed(&track);
    assert!(crossed.is_empty(), "the car drove through {} network walls:\n{}", crossed.len(), crossed.join("\n"));
}
