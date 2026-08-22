use glam::Vec2;
use tdrace_app::fx::{DriftPopupManager, EffectsManager, ParticleSystem, SkidmarkBuffer};
use tdrace_core::collision::car_collision::CarCarCollisionEvent;
use tdrace_core::collision::wall::WallCollisionEvent;
use tdrace_core::{Car, CarConfig};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_skidmarks_buffer_lifecycle() {
    let mut buffer = SkidmarkBuffer::new(50);
    assert_eq!(buffer.count(), 0);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.5;
    car.state.wheels[1].skid_intensity = 0.5;

    let cars = vec![car.clone()];
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    // Step 1: Initial position recorded
    buffer.update_for_cars(&cars, &surfaces);
    assert_eq!(buffer.count(), 0);

    // Step 2: Car moves while skidding
    car.state.position = Vec2::new(0.5, 0.0);
    let cars2 = vec![car.clone()];
    buffer.update_for_cars(&cars2, &surfaces);
    assert!(buffer.count() > 0);

    // Test clear
    buffer.clear();
    assert_eq!(buffer.count(), 0);
}

#[test]
fn test_particle_system_emission_and_updates() {
    let mut ps = ParticleSystem::new(200);
    assert_eq!(ps.count(), 0);

    // Smoke
    ps.emit_tire_smoke(Vec2::ZERO, Vec2::new(5.0, 0.0), 0.8);
    assert!(ps.count() > 0);

    // Dirt roost
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::Grass, Vec2::new(10.0, 0.0));
    assert!(ps.count() > 0);

    // Collision sparks
    ps.emit_sparks(Vec2::new(5.0, 5.0), Vec2::new(-1.0, 0.0), 8.0);
    let spark_count = ps.count();
    assert!(spark_count > 0);

    // Step forward 0.1s
    ps.update(0.1);
    assert!(ps.count() > 0);

    // Step forward 2.0s -> all particles should expire
    ps.update(2.0);
    assert_eq!(ps.count(), 0);
}

#[test]
fn test_drift_popup_manager() {
    let mut popups = DriftPopupManager::new(10);
    assert_eq!(popups.active_popups().len(), 0);

    popups.spawn_drift_score(Vec2::new(10.0, 10.0), 350.0, 1.5);
    assert_eq!(popups.active_popups().len(), 1);
    assert!(popups.active_popups()[0].text.contains("+350 DRIFT!"));

    popups.update(0.5);
    assert_eq!(popups.active_popups().len(), 1);
    assert!(popups.active_popups()[0].world_pos.y < 10.0); // Floats upward (-Y in Cartesian)

    popups.update(1.5);
    assert_eq!(popups.active_popups().len(), 0); // Expired
}

#[test]
fn test_effects_manager_integration() {
    let mut fx = EffectsManager::new(500, 500);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.speed = 25.0;
    car.state.velocity = Vec2::new(25.0, 0.0);
    car.state.wheels[0].skid_intensity = 0.6;
    car.state.is_drifting = true;
    car.state.drift_score = 450.0;

    let cars = vec![car.clone()];
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    let wall_events = vec![WallCollisionEvent {
        contact_point: Vec2::new(10.0, 5.0),
        normal: Vec2::new(0.0, -1.0),
        penetration: 0.05,
        impact_speed: 6.5,
        normal_impulse: 500.0,
        friction_impulse: 50.0,
    }];

    let car_events = vec![CarCarCollisionEvent {
        car_a_idx: 0,
        car_b_idx: 1,
        contact_point: Vec2::new(0.0, 0.0),
        normal: Vec2::new(1.0, 0.0),
        penetration: 0.02,
        closing_speed: 4.5,
        impulse_magnitude: 300.0,
    }];

    fx.update(&cars, &surfaces, &wall_events, &car_events, 0.016);
    assert!(fx.particles.count() > 0);

    // End drift -> triggers drift popup
    let mut car_stopped_drift = car.clone();
    car_stopped_drift.state.is_drifting = false;
    let cars2 = vec![car_stopped_drift];
    fx.update(&cars2, &surfaces, &[], &[], 0.016);
    assert_eq!(fx.drift_popups.active_popups().len(), 1);
}
