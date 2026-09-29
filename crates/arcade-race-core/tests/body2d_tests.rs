//! Spec 050 (`specs/050_body2d_trait_for_vehiclegeneric_collision_and_progress.md`):
//! collision and progress tracking work for a rigid body that is not a `wheelbase::Car`.
//!
//! `TestBody` stands in for a future chariot: a plain rigid body with a long hull.

use arcade_race_core::collision::{resolve_all_wall_collisions, resolve_multi_car_collisions};
use arcade_race_core::track::{
    create_prototypical_track, BarrierType, RaceDirection, TrackProgressTracker, TrackShape, WallBarrier,
};
use arcade_race_core::{Body2D, BodyHull};
use glam::Vec2;

#[derive(Clone)]
struct TestBody {
    position: Vec2,
    angle: f32,
    velocity: Vec2,
    angular_velocity: f32,
    mass: f32,
    inertia: f32,
    hull: BodyHull,
}

impl TestBody {
    /// A 9 m long, 2 m wide hull, about the size of a chariot with a four-horse team.
    fn chariot_sized(position: Vec2, angle: f32, velocity: Vec2, mass: f32) -> Self {
        Self {
            position,
            angle,
            velocity,
            angular_velocity: 0.0,
            mass,
            inertia: mass * 7.0,
            hull: BodyHull { front: 4.5, rear: 4.5, half_width: 1.0 },
        }
    }
}

impl Body2D for TestBody {
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
        self.angular_velocity
    }
    fn speed(&self) -> f32 {
        self.velocity.length()
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn inertia(&self) -> f32 {
        self.inertia
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
        self.hull
    }
    fn translate(&mut self, d: Vec2) {
        self.position += d;
    }
    fn add_velocity(&mut self, d: Vec2) {
        self.velocity += d;
    }
    fn add_angular_velocity(&mut self, d: f32) {
        self.angular_velocity += d;
    }
}

/// Scenario: A long body hits a wall that the old broad phase skipped
///
/// Given a 9 m hull whose front corner crosses a wall while its centre is 4.55 m away
/// When resolve_all_wall_collisions runs
/// Then a collision is reported and the body is pushed back out of the wall
#[test]
fn long_body_hits_wall_beyond_old_reach() {
    let wall_x = 4.55;
    let wall = WallBarrier::new(Vec2::new(wall_x, -10.0), Vec2::new(wall_x, 10.0), BarrierType::Concrete);
    let mut body = TestBody::chariot_sized(Vec2::ZERO, 0.0, Vec2::new(5.0, 0.0), 600.0);
    assert!(body.hull().reach() > 3.6, "test needs a hull longer than the old 3.6 m reach");
    assert!(wall_x > 3.6, "centre must be outside the old broad phase");

    let events = resolve_all_wall_collisions(&mut body, &[wall], &[]);

    assert!(!events.is_empty(), "the front corner overlaps the wall, so a collision must be reported");
    assert!(body.position.x < 0.0, "body must be pushed away from the wall, x = {}", body.position.x);
    assert!(body.velocity.x < 5.0, "impact must slow the body, vx = {}", body.velocity.x);
}

/// Scenario: Two non-car bodies collide and keep momentum
///
/// Given two long bodies of different mass moving toward each other and overlapping
/// When resolve_multi_car_collisions runs on a Vec of them
/// Then one contact event is returned and total linear momentum is unchanged within 1e-3
#[test]
fn two_bodies_collide_and_conserve_momentum() {
    let mut bodies = vec![
        TestBody::chariot_sized(Vec2::ZERO, 0.0, Vec2::new(3.0, 0.2), 500.0),
        TestBody::chariot_sized(Vec2::new(8.5, 0.3), 0.0, Vec2::new(-1.0, 0.0), 800.0),
    ];
    let momentum = |b: &[TestBody]| b.iter().map(|x| x.velocity * x.mass).fold(Vec2::ZERO, |a, v| a + v);
    let before = momentum(&bodies);

    let events = resolve_multi_car_collisions(&mut bodies, 0.45, 0.35, 3);

    let after = momentum(&bodies);
    assert_eq!(events.len(), 1, "one overlapping pair gives one event");
    assert!((after - before).length() < 1e-3, "momentum changed: before {:?}, after {:?}", before, after);
    assert!(bodies[0].velocity.x < 3.0 && bodies[1].velocity.x > -1.0, "the bodies must push each other apart");
}

/// Scenario: The progress tracker follows a non-car body
///
/// Given a test body moved one lap along the generated oval, starting before the line
/// When TrackProgressTracker::update runs each step
/// Then its progress distance goes up (wrapping at the line) and it reaches every checkpoint in order
#[test]
fn tracker_follows_a_non_car_body() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let n = track.checkpoints.len();
    assert!(n > 1);
    let mut tracker = TrackProgressTracker::new(n, 3);
    let dt = 1.0 / 120.0;
    let speed = 30.0;
    let lap = track.spline.total_length();

    let mut body = TestBody::chariot_sized(Vec2::ZERO, 0.0, Vec2::ZERO, 600.0);
    // Start 30 m before the start/finish gate (checkpoint 0) and drive one full lap.
    let start = lap - 30.0;
    let mut travelled = 0.0;
    let mut last_progress = f32::NEG_INFINITY;
    let mut increases = 0usize;
    let mut last_next = tracker.next_checkpoint_idx;
    let mut changes = 0usize;
    while travelled < lap {
        let sample = track.spline.sample_at_distance((start + travelled) % lap);
        body.position = sample.point;
        body.angle = sample.tangent.y.atan2(sample.tangent.x);
        body.velocity = sample.tangent * speed;
        tracker.update(&body, &track.spline, &track.checkpoints, dt);

        // Progress wraps to 0 at the start/finish gate on a closed track.
        if tracker.progress_distance > last_progress || last_progress - tracker.progress_distance > 0.5 * lap {
            increases += 1;
        }
        last_progress = tracker.progress_distance;
        if tracker.next_checkpoint_idx != last_next {
            assert_eq!(tracker.next_checkpoint_idx, (last_next + 1) % n, "checkpoints must be reached in order");
            last_next = tracker.next_checkpoint_idx;
            changes += 1;
        }
        travelled += speed * dt;
    }
    assert!(changes >= n, "every checkpoint must be reached in one lap, got {} of {}", changes, n);
    assert!(increases > 1000, "progress must keep going up");
    assert!(!tracker.is_wrong_way, "a body driving forward is not going the wrong way");
}
