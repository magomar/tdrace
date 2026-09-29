//! Spec 057 (`specs/057_raceui_rendering_camera_effects_and_hud_primitives.md`): the race camera
//! follows any `Body2D`, not only a `wheelbase::Car`.

use arcade_race_core::{Body2D, BodyHull};
use glam::Vec2;
use race_ui::camera::{CameraMode, RaceCamera};
use wheelbase::{Car, CarConfig};

/// A plain body: position, velocity and nothing else.
struct Sled {
    position: Vec2,
    velocity: Vec2,
}

impl Body2D for Sled {
    fn position(&self) -> Vec2 {
        self.position
    }
    fn angle(&self) -> f32 {
        0.0
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
        500.0
    }
    fn inertia(&self) -> f32 {
        3000.0
    }
    fn jump_height(&self) -> f32 {
        0.0
    }
    fn total_elevation(&self) -> f32 {
        0.0
    }
    fn forward_vector(&self) -> Vec2 {
        Vec2::X
    }
    fn hull(&self) -> BodyHull {
        BodyHull { front: 4.5, rear: 4.5, half_width: 1.0 }
    }
    fn translate(&mut self, d: Vec2) {
        self.position += d;
    }
    fn add_velocity(&mut self, d: Vec2) {
        self.velocity += d;
    }
    fn add_angular_velocity(&mut self, _d: f32) {}
}

/// Scenario: The camera follows a body that is not a car
///
/// Given a RaceCamera and a test body that implements Body2D
/// When update runs for 2 s while the body moves in a straight line
/// Then the camera centre ends near the body's position plus its look-ahead, the same result as
/// for a Car with the same position and velocity
#[test]
fn camera_follows_a_non_car_body_like_a_car() {
    let dt = 1.0 / 60.0;
    let velocity = Vec2::new(20.0, 0.0);
    let mut sled = Sled { position: Vec2::new(100.0, 50.0), velocity };
    let mut car = Car::new(CarConfig::sports_car()).with_pose(sled.position, 0.0);
    car.state.velocity = velocity;
    car.state.speed = velocity.length();

    let mut cam_sled = RaceCamera::new();
    let mut cam_car = RaceCamera::new();
    assert_eq!(cam_sled.mode, CameraMode::SmoothFollow);
    cam_sled.resume_from_pause(Some(&sled));
    cam_car.resume_from_pause(Some(&car));

    for _ in 0..120 {
        sled.position += velocity * dt;
        car.state.position = sled.position;
        cam_sled.update(&sled, dt);
        cam_car.update(&car, dt);
    }

    assert_eq!(cam_sled.current_pos, cam_car.current_pos, "same pose and motion must give the same camera");
    assert_eq!(cam_sled.current_zoom, cam_car.current_zoom);
    let target = sled.position + velocity * cam_sled.velocity_lookahead_time;
    assert!(
        cam_sled.current_pos.distance(target) < 5.0,
        "camera {:?} must trail the look-ahead point {:?} closely",
        cam_sled.current_pos,
        target
    );
}
