//! Spec 056 (`specs/056_racekit_headless_race_world.md`): finish order, real times and DNF.

use arcade_race_core::track::{
    create_prototypical_track, BarrierType, RaceDirection, SplineProjection, Track, TrackProgressTracker, TrackShape,
    WallBarrier,
};
use arcade_race_core::{Body2D, BodyHull};
use glam::Vec2;
use race_kit::ai::{BotAiDriver, BotProfile};
use race_kit::{DnfCause, DriveControls, FinishState, RaceEvent, RaceFormat, RaceRules, RaceWorld, Vehicle};
use wheelbase::{Car, CarConfig, SurfaceType};

const DT: f32 = 1.0 / 120.0;

/// Scenario: A laps race finishes with real times
///
/// Given 3 cars driven by race_kit::ai::BotAiDriver in Laps(2) on the prototypical oval
/// When the world steps until every car finishes
/// Then every car is Finished, positions are 1, 2 and 3 in finish order, times go up with
/// position, and each time equals world.time at the step the car crossed the line
#[test]
fn laps_race_finishes_with_real_times() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(2), ..RaceRules::default() });
    let mut drivers = Vec::new();
    for i in 0..3 {
        let slot = track.grid_positions[i];
        world.spawn(
            Car::new(CarConfig::sports_car()).with_pose(slot.position, slot.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
        drivers.push(BotAiDriver::with_seed(BotProfile::pro(), 17 + i as u64));
    }

    let mut crossings = Vec::new();
    for _ in 0..(600.0 / DT) as usize {
        let controls: Vec<DriveControls> = (0..3)
            .map(|i| {
                let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
                drivers[i].compute_controls(&world.vehicles[i], &track, &others, DT)
            })
            .collect();
        let events: Vec<RaceEvent> = world.step(&track, &controls, DT).to_vec();
        for ev in events {
            if let RaceEvent::Finished { car, position, time } = ev {
                crossings.push((car, position, time, world.time));
            }
        }
        if (0..3).all(|i| world.is_finished(i)) {
            break;
        }
    }

    assert_eq!(crossings.len(), 3, "every car must finish within 600 s, got {:?}", world.finish);
    for (k, &(car, position, time, clock)) in crossings.iter().enumerate() {
        assert_eq!(position, k + 1, "positions follow finish order");
        assert_eq!(time, clock, "finish time is the world clock at the crossing step");
        assert_eq!(world.finish[car], FinishState::Finished { time, position });
        assert!(world.trackers[car].current_lap > 2, "a finished car has completed 2 laps");
    }
    assert!(crossings.windows(2).all(|w| w[0].2 <= w[1].2), "times go up with position: {:?}", crossings);

    let results = world.results(&track);
    let order: Vec<usize> = crossings.iter().map(|c| c.0).collect();
    assert_eq!(results.iter().map(|r| r.car).collect::<Vec<_>>(), order, "results follow finish order");
    assert!(results.iter().all(|r| !r.projected), "finished cars have real times");
}

/// A plain rigid body that moves in a straight line and wrecks above a set impact speed.
struct TestCart {
    position: Vec2,
    angle: f32,
    velocity: Vec2,
    wreck_speed: f32,
}

impl TestCart {
    fn new(position: Vec2, angle: f32, speed: f32, wreck_speed: f32) -> Self {
        Self { position, angle, velocity: Vec2::new(angle.cos(), angle.sin()) * speed, wreck_speed }
    }
}

impl Body2D for TestCart {
    fn position(&self) -> Vec2 {
        self.position
    }
    fn angle(&self) -> f32 {
        self.angle
    }
    fn velocity(&self) -> Vec2 {
        self.velocity
    }
    fn angular_velocity(&self) -> f32 {
        0.0
    }
    fn speed(&self) -> f32 {
        self.velocity.length()
    }
    fn mass(&self) -> f32 {
        600.0
    }
    fn inertia(&self) -> f32 {
        4000.0
    }
    fn jump_height(&self) -> f32 {
        0.0
    }
    fn total_elevation(&self) -> f32 {
        0.0
    }
    fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
    fn hull(&self) -> BodyHull {
        BodyHull { front: 2.0, rear: 2.0, half_width: 1.0 }
    }
    fn translate(&mut self, d: Vec2) {
        self.position += d;
    }
    fn add_velocity(&mut self, d: Vec2) {
        self.velocity += d;
    }
    fn add_angular_velocity(&mut self, _d: f32) {}
}

impl Vehicle for TestCart {
    fn sample_surfaces(&self, _track: &Track, _hint: f32) -> [SurfaceType; 4] {
        [SurfaceType::Asphalt; 4]
    }
    fn draft_intensity(&self, _others: &[&Self]) -> f32 {
        0.0
    }
    fn set_draft(&mut self, _intensity: f32) {}
    fn set_road(&mut self, _proj: &SplineProjection) {}
    fn step(&mut self, controls: &DriveControls, _surfaces: [SurfaceType; 4], dt: f32) {
        if controls.throttle > 0.0 {
            self.velocity += self.forward_vector() * (8.0 * controls.throttle * dt);
        } else {
            self.velocity *= 0.99;
        }
        self.position += self.velocity * dt;
    }
    fn on_impact(&mut self, impact_speed: f32) -> Option<DnfCause> {
        (impact_speed > self.wreck_speed).then_some(DnfCause::Impact)
    }
}

/// Scenario: A wreck gives a DNF
///
/// Given a test vehicle whose on_impact returns a cause above 5 m/s, driven into a wall at 10 m/s
/// When the world steps
/// Then the vehicle is Dnf, one Wrecked event is emitted, its controls are ignored from then on,
/// and a second vehicle that drives into it still collides with it
#[test]
fn wreck_gives_dnf_and_stays_collidable() {
    let mut track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    track.geometry.inner_walls.clear();
    track.geometry.outer_walls = vec![WallBarrier::new(Vec2::new(10.0, -30.0), Vec2::new(10.0, 30.0), BarrierType::Concrete)];
    track.geometry.obstacles.clear();
    let tracker = || TrackProgressTracker::new(track.checkpoints.len(), 3);

    let mut world: RaceWorld<TestCart> = RaceWorld::new(RaceRules::default());
    world.spawn(TestCart::new(Vec2::ZERO, 0.0, 10.0, 5.0), tracker());
    let full_throttle = |n: usize| vec![DriveControls::new(1.0, 0.0, 0.0, false); n];

    let mut wrecked = 0;
    let mut wreck_speed = None;
    for _ in 0..240 {
        for ev in world.step(&track, &full_throttle(1), DT) {
            if let RaceEvent::Wrecked { car, cause } = ev {
                assert_eq!((*car, *cause), (0, DnfCause::Impact));
                wrecked += 1;
            }
        }
        if wrecked > 0 && wreck_speed.is_none() {
            wreck_speed = Some(world.vehicles[0].speed());
        }
    }
    assert_eq!(wrecked, 1, "one Wrecked event");
    assert!(matches!(world.finish[0], FinishState::Dnf { cause: DnfCause::Impact, .. }), "state {:?}", world.finish[0]);
    let after = world.vehicles[0].speed();
    assert!(after < wreck_speed.unwrap(), "full throttle must be ignored after the wreck: {} then {}", wreck_speed.unwrap(), after);

    // A second, sturdy vehicle drives into the wreck from above.
    let wreck_at = world.vehicles[0].position;
    world.spawn(TestCart::new(wreck_at + Vec2::new(0.0, 12.0), -std::f32::consts::FRAC_PI_2, 8.0, f32::INFINITY), tracker());
    let mut contact = false;
    for _ in 0..360 {
        contact |= world
            .step(&track, &full_throttle(2), DT)
            .iter()
            .any(|ev| matches!(ev, RaceEvent::VehicleImpact(c) if (c.car_a_idx, c.car_b_idx) == (0, 1)));
    }
    assert!(contact, "the wreck must stay collidable");
    assert_eq!(world.finish[1], FinishState::Racing, "the sturdy vehicle is not wrecked");
    assert_eq!(world.standings().last(), Some(&0), "a DNF ranks last");
}
