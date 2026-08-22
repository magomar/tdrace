use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile};
use tdrace_core::{Car, CarConfig};
use tdrace_core::track::presets::{classic_grand_prix, oval_speedway};

#[test]
fn test_bot_profiles_creation() {
    let pro = BotProfile::pro();
    let aggressive = BotProfile::aggressive();
    let balanced = BotProfile::balanced();
    let rookie = BotProfile::rookie();

    assert!(aggressive.speed_factor > balanced.speed_factor);
    assert!(pro.steering_kp > rookie.steering_kp);
    assert!(rookie.brake_margin > pro.brake_margin);
}

#[test]
fn test_bot_ai_steering_and_throttle_on_straight() {
    let track = classic_grand_prix();
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Car is initialized facing spline direction
    let proj = track.spline.project_point(Vec2::new(50.0, 0.0));
    let spline_angle = proj.tangent.y.atan2(proj.tangent.x);
    let car = Car::new(CarConfig::sports_car()).with_pose(proj.closest_point, spline_angle);

    let ctrl = bot.compute_controls(&car, &track, &[], 0.016);

    // Car should accelerate down straight with well-aligned steering
    assert!(ctrl.throttle > 0.5);
    assert!(ctrl.steer.abs() < 0.15);
    assert_eq!(ctrl.brake, 0.0);
    assert!(!ctrl.handbrake);
}

#[test]
fn test_bot_ai_collision_avoidance() {
    let track = oval_speedway();
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Car A (our bot) is traveling at 35 m/s at x=50, y=-60
    let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, -60.0), 0.0);
    car_a.state.speed = 35.0;
    car_a.state.velocity = Vec2::new(35.0, 0.0);

    // Car B (slow opponent directly ahead) at x=54, y=-60 at 10 m/s
    let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(54.0, -60.0), 0.0);
    car_b.state.speed = 10.0;
    car_b.state.velocity = Vec2::new(10.0, 0.0);

    let ctrl = bot.compute_controls(&car_a, &track, &[&car_b], 0.016);

    // Bot should brake / lift throttle to prevent rear-ending car B
    assert!(ctrl.brake > 0.0 || ctrl.throttle < 0.5);
}

#[test]
fn test_bot_ai_cornering_slowdown() {
    let track = classic_grand_prix();
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Approaching hairpin at high speed (x=240, y=300, heading right towards hairpin)
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(240.0, 300.0), 0.0);
    car.state.speed = 50.0; // Over speeding for a tight corner
    car.state.velocity = Vec2::new(50.0, 0.0);

    let ctrl = bot.compute_controls(&car, &track, &[], 0.016);
    // Should apply brakes to prepare for hairpin turn
    assert!(ctrl.brake > 0.0 || ctrl.throttle < 0.2);
}
