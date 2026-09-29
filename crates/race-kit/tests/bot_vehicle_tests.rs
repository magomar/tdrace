//! Spec 061 (`specs/061_vehiclegeneric_bot_ai_and_effects.md`): the bot driver drives any
//! vehicle that implements `BotVehicle`, not only `wheelbase::Car`.

use arcade_race_core::track::{create_prototypical_track, RaceDirection, SplineProjection, Track, TrackProgressTracker, TrackShape};
use arcade_race_core::{Body2D, BodyHull};
use glam::Vec2;
use race_kit::ai::{BotAiDriver, BotProfile, BotVehicle};
use race_kit::{DriveControls, FinishState, RaceFormat, RaceRules, RaceWorld, Vehicle};
use wheelbase::SurfaceType;

const DT: f32 = 1.0 / 120.0;

/// A kinematic cart: it goes where it points, turns like a bicycle and has no tyre model.
struct Cart {
    position: Vec2,
    angle: f32,
    speed: f32,
}

impl Cart {
    const TOP_SPEED: f32 = 30.0;
    const WHEELBASE: f32 = 3.0;
    const MAX_STEER: f32 = 0.5;

    fn forward(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
}

impl Body2D for Cart {
    fn position(&self) -> Vec2 {
        self.position
    }
    fn angle(&self) -> f32 {
        self.angle
    }
    fn velocity(&self) -> Vec2 {
        self.forward() * self.speed
    }
    fn angular_velocity(&self) -> f32 {
        0.0
    }
    fn speed(&self) -> f32 {
        self.speed
    }
    fn mass(&self) -> f32 {
        700.0
    }
    fn inertia(&self) -> f32 {
        1500.0
    }
    fn jump_height(&self) -> f32 {
        0.0
    }
    fn total_elevation(&self) -> f32 {
        0.0
    }
    fn forward_vector(&self) -> Vec2 {
        self.forward()
    }
    fn hull(&self) -> BodyHull {
        BodyHull { front: 1.8, rear: 1.8, half_width: 0.9 }
    }
    fn translate(&mut self, d: Vec2) {
        self.position += d;
    }
    fn add_velocity(&mut self, d: Vec2) {
        // Keep only the part along the heading: a cart does not slide sideways.
        self.speed = (self.speed + d.dot(self.forward())).max(0.0);
    }
    fn add_angular_velocity(&mut self, _d: f32) {}
}

impl Vehicle for Cart {
    fn sample_surfaces(&self, _track: &Track, _hint: f32) -> [SurfaceType; 4] {
        [SurfaceType::Asphalt; 4]
    }
    fn draft_intensity(&self, _others: &[&Self]) -> f32 {
        0.0
    }
    fn set_draft(&mut self, _intensity: f32) {}
    fn set_road(&mut self, _proj: &SplineProjection) {}
    fn step(&mut self, c: &DriveControls, _surfaces: [SurfaceType; 4], dt: f32) {
        let accel = 6.0 * c.throttle - 12.0 * c.brake;
        self.speed = (self.speed + accel * dt).clamp(0.0, Self::TOP_SPEED);
        // Positive steer turns right (clockwise), as for `wheelbase::Car`.
        let yaw_rate = -c.steer.clamp(-1.0, 1.0) * Self::MAX_STEER * self.speed / Self::WHEELBASE;
        self.angle += yaw_rate * dt;
        self.position += self.forward() * self.speed * dt;
    }
}

impl BotVehicle for Cart {
    fn right_vector(&self) -> Vec2 {
        Vec2::new(self.angle.sin(), -self.angle.cos())
    }
    fn top_speed_mps(&self) -> f32 {
        Self::TOP_SPEED
    }
    fn grip(&self) -> f32 {
        1.0
    }
}

/// Scenario: A bot drives a vehicle that is not a car
///
/// Given a test vehicle that implements Vehicle and BotVehicle with simple point-mass physics,
/// on the prototypical oval
/// When a BotAiDriver drives it in a RaceWorld for 2 laps
/// Then it finishes both laps, and it never stays below 2 m/s for more than 3 s after the start
#[test]
fn bot_drives_a_non_car_vehicle() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let slot = track.grid_positions[0];
    let mut world: RaceWorld<Cart> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(2), ..RaceRules::default() });
    world.spawn(Cart { position: slot.position, angle: slot.angle, speed: 0.0 }, TrackProgressTracker::new(track.checkpoints.len(), 3));
    let mut bot = BotAiDriver::with_seed(BotProfile::pro(), 3);

    let mut slow_for = 0.0f32;
    let mut longest_slow = 0.0f32;
    for _ in 0..(400.0 / DT) as usize {
        let controls = [bot.compute_controls(&world.vehicles[0], &track, &[], DT)];
        world.step(&track, &controls, DT);
        if world.time > 3.0 && world.vehicles[0].speed < 2.0 {
            slow_for += DT;
            longest_slow = longest_slow.max(slow_for);
        } else {
            slow_for = 0.0;
        }
        if world.is_finished(0) {
            break;
        }
    }
    assert!(matches!(world.finish[0], FinishState::Finished { .. }), "the cart must finish 2 laps, state {:?}, lap {}", world.finish[0], world.trackers[0].current_lap);
    assert!(longest_slow <= 3.0, "the cart stalled for {:.1} s", longest_slow);
}
