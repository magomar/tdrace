//! Spec 065 (`specs/065_vehiclegeneric_bot_ai_and_effects.md`): the bot driver drives any
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

/// Scenario: Bot AI evaluates pit heuristics and executes pit lane braking/acceleration
#[test]
fn test_bot_ai_pit_tactics_and_stall_stopping() {
    use arcade_race_core::track::{Checkpoint, LineSegment, PitBox, PitLane, TrackSpline};
    use wheelbase::{Car, CarConfig};

    let mut track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let entry_gate = LineSegment::new(Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0));
    let exit_gate = LineSegment::new(Vec2::new(100.0, -10.0), Vec2::new(100.0, 10.0));
    let pit_spline = TrackSpline::from_points(&[Vec2::new(10.0, 0.0), Vec2::new(55.0, 0.0), Vec2::new(100.0, 0.0)], 6.0, false);
    let pit_box = PitBox::new(Vec2::new(50.0, 0.0), Vec2::new(1.0, 0.0), 3.0, 0.0);

    track.pit_lane = Some(PitLane::new(
        pit_spline,
        6.0,
        PitLane::DEFAULT_ROAD_SPEED_LIMIT,
        vec![pit_box],
        entry_gate,
        exit_gate,
    ));

    track.checkpoints = vec![
        Checkpoint::new(0, entry_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(true, false),
        Checkpoint::new(1, exit_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(false, true),
    ];

    let mut bot = BotAiDriver::with_seed(BotProfile::pro(), 42);

    // 1. Pit decision heuristic tests
    let mut fresh_car = Car::new(CarConfig::sports_car());
    fresh_car.state.health = 1.0;
    for w in &mut fresh_car.state.wheels {
        w.wear = 0.20;
    }
    assert!(!bot.should_pit(&fresh_car), "Fresh car with 20% wear should not pit");

    fresh_car.state.wheels[0].wear = 0.75;
    assert!(bot.should_pit(&fresh_car), "Car with 75% wear on any wheel should pit");

    fresh_car.state.wheels[0].wear = 0.20;
    fresh_car.state.health = 0.55;
    assert!(bot.should_pit(&fresh_car), "Car with 55% health should pit");

    // 2. Bot navigating pit lane approaching stall brakes to a stop
    let mut pitting_car = Car::new(CarConfig::sports_car());
    pitting_car.state.health = 0.50;
    pitting_car.state.position = Vec2::new(50.0, 0.0); // Inside pit stall
    pitting_car.state.velocity = Vec2::new(2.0, 0.0);
    pitting_car.state.speed = 2.0;

    bot.is_pitting = true;
    let ctrl = bot.compute_controls(&pitting_car, &track, &[], 0.016);
    assert_eq!(ctrl.brake, 1.0, "Bot inside pit stall must apply full brake");
    assert_eq!(ctrl.throttle, 0.0, "Bot inside pit stall must not apply throttle");

    // 3. Once serviced, bot accelerates
    pitting_car.state.health = 1.0;
    for w in &mut pitting_car.state.wheels {
        w.wear = 0.0;
    }
    bot.pit_serviced = true;
    let ctrl_exit = bot.compute_controls(&pitting_car, &track, &[], 0.016);
    assert!(ctrl_exit.throttle > 0.0, "Bot after service must apply throttle to exit");
    assert_eq!(ctrl_exit.brake, 0.0, "Bot after service must release brake");
}

/// Scenario: Bot AI respects Lap 1 gating, assigned stall targeting, and 30s cooldown
#[test]
fn test_bot_ai_lap1_pit_gating_stall_assignment_and_cooldown() {
    use arcade_race_core::track::{LineSegment, PitBox, PitLane, TrackSpline};
    use wheelbase::{Car, CarConfig};

    let mut track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let entry_gate = LineSegment::new(Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0));
    let exit_gate = LineSegment::new(Vec2::new(100.0, -10.0), Vec2::new(100.0, 10.0));
    let pit_spline = TrackSpline::from_points(&[Vec2::new(10.0, 0.0), Vec2::new(55.0, 0.0), Vec2::new(100.0, 0.0)], 6.0, false);
    let pit_box_0 = PitBox::new(Vec2::new(35.0, 0.0), Vec2::new(1.0, 0.0), 3.0, 0.0);
    let pit_box_1 = PitBox::new(Vec2::new(65.0, 0.0), Vec2::new(1.0, 0.0), 3.0, 0.0);

    track.pit_lane = Some(PitLane::new(
        pit_spline,
        6.0,
        PitLane::DEFAULT_ROAD_SPEED_LIMIT,
        vec![pit_box_0, pit_box_1],
        entry_gate,
        exit_gate,
    ));

    // 1. Bot on Lap 1 with damaged car does not pit
    let mut bot_lap1 = BotAiDriver::with_seed(BotProfile::pro(), 1).with_current_lap(1);
    let mut damaged_car = Car::new(CarConfig::sports_car());
    damaged_car.state.health = 0.40;
    damaged_car.state.position = Vec2::new(8.0, 0.0);
    damaged_car.state.speed = 15.0;

    let _ = bot_lap1.compute_controls(&damaged_car, &track, &[], 0.016);
    assert!(!bot_lap1.is_pitting, "Bot on Lap 1 must NOT enter pit lane even when damaged");

    // 2. Bot on Lap 2 with damaged car DOES pit
    let mut bot_lap2 = BotAiDriver::with_seed(BotProfile::pro(), 2).with_current_lap(2).with_pit_stall(1);
    let _ = bot_lap2.compute_controls(&damaged_car, &track, &[], 0.016);
    assert!(bot_lap2.is_pitting, "Bot on Lap 2 with severe damage MUST enter pit lane");

    // 3. Assigned stall targeting: bot with stall_idx = 1 targets stall 1 at x=65.0, not stall 0 at x=35.0
    let mut pit_car = Car::new(CarConfig::sports_car());
    pit_car.state.health = 0.40;
    pit_car.state.position = Vec2::new(35.0, 0.0); // Inside stall 0
    pit_car.state.speed = 10.0;
    let ctrl_stall1 = bot_lap2.compute_controls(&pit_car, &track, &[], 0.016);
    assert!(ctrl_stall1.throttle > 0.0, "Bot assigned to stall 1 must NOT stop at stall 0");

    // Move to stall 1: must brake to full stop
    pit_car.state.position = Vec2::new(65.0, 0.0);
    pit_car.state.speed = 1.0;
    let ctrl_in_stall1 = bot_lap2.compute_controls(&pit_car, &track, &[], 0.016);
    assert_eq!(ctrl_in_stall1.brake, 1.0, "Bot in its assigned stall 1 must brake");

    // 4. Stall timer release after 2.0s even with low health
    pit_car.state.speed = 0.0;
    for _ in 0..150 {
        bot_lap2.compute_controls(&pit_car, &track, &[], 0.016);
    }
    assert!(bot_lap2.pit_serviced, "Bot must be released after 2s stationary in stall");

    // 5. Leaving pit lane at exit gate sets 30s cooldown and resets is_pitting
    pit_car.state.position = Vec2::new(98.0, 0.0);
    pit_car.state.speed = 15.0;
    bot_lap2.compute_controls(&pit_car, &track, &[], 0.016);
    assert!(!bot_lap2.is_pitting, "Bot at exit gate must reset is_pitting");
    assert!(bot_lap2.pit_cooldown >= 29.0, "Bot exiting pit must receive ~30s cooldown");
}
