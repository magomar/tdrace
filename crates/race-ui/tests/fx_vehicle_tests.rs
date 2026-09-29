//! Spec 065 (`specs/065_vehiclegeneric_bot_ai_and_effects.md`): the effects follow any vehicle
//! that implements `FxVehicle`, not only `wheelbase::Car`.

use arcade_race_core::{Body2D, BodyHull};
use glam::Vec2;
use race_ui::fx::{EffectsManager, FxVehicle};
use wheelbase::{SurfaceType, WheelTelemetry};

/// A sliding body with two wheels and two hoof groups as its contact points.
struct Sled {
    position: Vec2,
    velocity: Vec2,
    telemetry: [WheelTelemetry; 4],
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
        900.0
    }
    fn inertia(&self) -> f32 {
        6000.0
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
        BodyHull { front: 4.0, rear: 1.0, half_width: 1.2 }
    }
    fn translate(&mut self, d: Vec2) {
        self.position += d;
    }
    fn add_velocity(&mut self, d: Vec2) {
        self.velocity += d;
    }
    fn add_angular_velocity(&mut self, _d: f32) {}
}

impl FxVehicle for Sled {
    fn contact_points(&self) -> [Vec2; 4] {
        // Two wheels at the back, two hoof groups ahead.
        [Vec2::new(-0.8, 0.9), Vec2::new(-0.8, -0.9), Vec2::new(3.0, 0.6), Vec2::new(3.0, -0.6)].map(|p| self.position + p)
    }
    fn contact_telemetry(&self) -> &[WheelTelemetry; 4] {
        &self.telemetry
    }
    fn right_vector(&self) -> Vec2 {
        Vec2::NEG_Y
    }
    fn is_airborne(&self) -> bool {
        false
    }
}

/// Scenario: The effects follow a vehicle that is not a car
///
/// Given a test vehicle that implements FxVehicle, sliding on sand with high slip on its four
/// contact points
/// When EffectsManager::update runs for 1 s
/// Then skid or rut segments are laid at its contact points, and dust particles are emitted
#[test]
fn effects_follow_a_non_car_vehicle() {
    let slip = WheelTelemetry { slip_angle: 0.3, slip_ratio: 0.2, skid_intensity: 0.6, is_skidding: true, ..WheelTelemetry::default() };
    let mut sled = Sled { position: Vec2::ZERO, velocity: Vec2::new(12.0, 0.0), telemetry: [slip; 4] };
    let mut fx = EffectsManager::new(2000, 2000);
    let sand = [[SurfaceType::PackedSand; 4]];
    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        sled.position += sled.velocity * dt;
        fx.update(std::slice::from_ref(&sled), &sand, &[], &[], dt);
    }

    assert!(fx.skidmarks.count() > 0, "sliding on sand must lay ruts");
    assert!(fx.particles.count() > 0, "sliding on sand must throw dust");
    // Every segment lies on one of the four contact tracks (y = ±0.9 for wheels, ±0.6 for hooves).
    for seg in fx.skidmarks.segments() {
        let y = ((seg.p0 + seg.p1 + seg.p2 + seg.p3) * 0.25).y;
        assert!((y.abs() - 0.9).abs() < 0.5 || (y.abs() - 0.6).abs() < 0.5, "segment off the contact tracks at y = {}", y);
    }
}
