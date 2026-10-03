//! Regression test suite for directional collision masking, engine placement immunity,
//! and archetype suspension failure dynamics (Spec 078).

use glam::Vec2;
use wheelbase::config::{CarConfig, EnginePlacement, SuspensionConfig};
use wheelbase::{Car, CarControls, ImpactZone, SurfaceType};

/// Verifies that contact points in vehicle-local coordinates classify accurately into
/// all 8 discrete impact zones.
#[test]
fn test_directional_impact_masking_zones() {
    let car = Car::new(CarConfig::sports_car());

    // Front Nose and Rear Tail along vehicle centerline
    assert_eq!(car.classify_impact_zone_local(2.0, 0.0), ImpactZone::FrontNose);
    assert_eq!(car.classify_impact_zone_local(-2.0, 0.0), ImpactZone::RearTail);

    // Front and Rear 4-corner wheel assemblies
    assert_eq!(car.classify_impact_zone_local(1.5, -1.0), ImpactZone::CornerFL);
    assert_eq!(car.classify_impact_zone_local(1.5, 1.0), ImpactZone::CornerFR);
    assert_eq!(car.classify_impact_zone_local(-1.5, -1.0), ImpactZone::CornerRL);
    assert_eq!(car.classify_impact_zone_local(-1.5, 1.0), ImpactZone::CornerRR);

    // Left and Right flanks
    assert_eq!(car.classify_impact_zone_local(0.0, -1.0), ImpactZone::FlankLeft);
    assert_eq!(car.classify_impact_zone_local(0.0, 1.0), ImpactZone::FlankRight);

    // World coordinate classification
    let pos = car.state.position;
    let fwd = car.forward_vector();
    let right = car.right_vector();

    assert_eq!(car.classify_impact_zone(pos + fwd * 2.0), ImpactZone::FrontNose);
    assert_eq!(car.classify_impact_zone(pos - fwd * 2.0), ImpactZone::RearTail);
    assert_eq!(car.classify_impact_zone(pos + fwd * 1.5 - right * 1.0), ImpactZone::CornerFL);
    assert_eq!(car.classify_impact_zone(pos + fwd * 1.5 + right * 1.0), ImpactZone::CornerFR);
    assert_eq!(car.classify_impact_zone(pos - right * 1.0), ImpactZone::FlankLeft);
    assert_eq!(car.classify_impact_zone(pos + right * 1.0), ImpactZone::FlankRight);
}

/// Scenario: Head-on barrier crash on rear-engine car spares engine block (0% mechanical damage),
/// whereas front-engine car engine drops below 40% and loses >= 25% power.
#[test]
fn test_rear_engine_headon_collision_immunity() {
    // 1. Rear-Engine Vehicle (Porsche 911 GT3 / Sand Rail layout)
    let mut car_rear = Car::new(CarConfig::sand_rail());
    car_rear.config.engine_placement = EnginePlacement::RearEngine;
    car_rear.state.engine_health = 1.0;
    car_rear.state.chassis_health = 1.0;
    car_rear.state.suspension_health = [1.0, 1.0, 1.0, 1.0];

    let nose_contact = car_rear.state.position + car_rear.forward_vector() * 2.0;
    let zone_rear = car_rear.apply_collision_damage(nose_contact, 6000.0);

    assert_eq!(zone_rear, ImpactZone::FrontNose);
    assert_eq!(
        car_rear.state.engine_health, 1.0,
        "RearEngine car must suffer 0% mechanical engine damage from head-on nose collision"
    );
    assert!(
        car_rear.state.chassis_health < 0.65,
        "Chassis monocoque must absorb front collision damage, got health={:.3}",
        car_rear.state.chassis_health
    );
    assert!(
        car_rear.state.suspension_health[0] < 1.0 && car_rear.state.suspension_health[1] < 1.0,
        "Front suspension corners must absorb front collision damage"
    );

    // 2. Front-Engine Vehicle (Thunderbolt Stock Car / Sports Car)
    let mut car_front = Car::new(CarConfig::sports_car());
    car_front.config.engine_placement = EnginePlacement::FrontEngine;
    car_front.state.engine_health = 1.0;
    car_front.state.chassis_health = 1.0;
    car_front.state.suspension_health = [1.0, 1.0, 1.0, 1.0];

    let front_nose_contact = car_front.state.position + car_front.forward_vector() * 2.0;
    let zone_front = car_front.apply_collision_damage(front_nose_contact, 6000.0);

    assert_eq!(zone_front, ImpactZone::FrontNose);
    assert!(
        car_front.state.engine_health < 0.40,
        "FrontEngine car engine health ({:.3}) must drop below 0.40 on 6000J head-on impact",
        car_front.state.engine_health
    );
    assert!(
        car_front.available_engine_power_ratio() <= 0.75,
        "Available engine power must be reduced by >= 25% (ratio was {:.3})",
        car_front.available_engine_power_ratio()
    );
}

/// Scenario: Apex kerb bottoming on MacPherson strut bends arm and creates steering pull bias.
#[test]
fn test_macpherson_curb_bottoming_steering_pull() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sports_car();
    cfg.suspension = SuspensionConfig::macpherson_strut();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(110.0 / 3.6, 0.0);
    car.state.speed = 110.0 / 3.6;

    let baseline_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    // Pristine baseline run
    car.step(&baseline_ctrl, SurfaceType::Asphalt, dt);
    assert_eq!(car.state.suspension_health[0], 1.0);
    assert_eq!(car.steering_pull_bias(), 0.0);

    // Front-left wheel strikes a high apex curb (0.085m elevation) at 110 km/h
    car.state.wheel_elevations = [0.085, 0.0, 0.0, 0.0];
    car.step(&baseline_ctrl, SurfaceType::Asphalt, dt);

    assert!(
        car.state.suspension_health[0] < 0.65,
        "Front-left suspension health ({:.3}) must drop below 0.65 on apex curb bottoming",
        car.state.suspension_health[0]
    );

    let pull_bias = car.steering_pull_bias();
    assert!(
        pull_bias < -0.03,
        "Steering pull bias ({:.4} rad) must be < -0.03 rad",
        pull_bias
    );

    // Clear elevation: vehicle continues on flat asphalt
    car.state.wheel_elevations = [0.0, 0.0, 0.0, 0.0];

    // Driving with zero driver steer input causes vehicle to veer left toward damaged FL corner
    let mut car_hands_off = car.clone();
    for _ in 0..60 {
        car_hands_off.step(&baseline_ctrl, SurfaceType::Asphalt, dt);
    }
    assert!(
        car_hands_off.state.angle > 0.015,
        "Vehicle must veer toward damaged FL corner when driving without steering input (angle was {:.4} rad)",
        car_hands_off.state.angle
    );

    // Holding right steering keeps the vehicle tracking straight
    let mut car_corrected = car.clone();
    let right_ctrl = CarControls::new(1.0, 0.30, 0.0, false);
    for _ in 0..60 {
        car_corrected.step(&right_ctrl, SurfaceType::Asphalt, dt);
    }
    assert!(
        car_corrected.state.angle < car_hands_off.state.angle,
        "Player holding right steering must counteract left pull"
    );
}

/// Scenario: Inboard pushrod hypercar shatters rocker on violent kerb strike, collapsing onto
/// bump stop and increasing aerodynamic drag coefficient by +40%.
#[test]
fn test_pushrod_fragility_drag_penalty() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sports_car();
    cfg.suspension = SuspensionConfig::pushrod_inboard();
    let baseline_drag = Car::new(cfg.clone()).effective_air_drag_coefficient();

    let mut car = Car::new(cfg);
    car.state.velocity = Vec2::new(160.0 / 3.6, 0.0);
    car.state.speed = 160.0 / 3.6;

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    // Strike a kerb (+0.05m) at 160 km/h
    car.state.wheel_elevations = [0.05, 0.0, 0.0, 0.0];
    car.step(&ctrl, SurfaceType::Asphalt, dt);

    assert!(
        car.state.suspension_health[0] < 0.30,
        "Pushrod fragility factor (k_rob=0.45) must cause corner health to drop below 0.30, got {:.3}",
        car.state.suspension_health[0]
    );

    assert!(
        car.state.suspension[0].deflection >= car.config.suspension.front.max_bump_travel - 1e-4,
        "Corner deflection must collapse onto bump stop, got {:.4}m vs limit {:.4}m",
        car.state.suspension[0].deflection,
        car.config.suspension.front.max_bump_travel
    );
    assert!(
        car.state.suspension[0].bottomed_out,
        "Corner must report bottomed_out = true"
    );

    let damaged_drag = car.effective_air_drag_coefficient();
    let drag_increase = (damaged_drag - baseline_drag) / baseline_drag;
    assert!(
        (drag_increase - 0.40).abs() < 1e-3,
        "Vehicle air drag coefficient must increase by +40%, got {:.1}%",
        drag_increase * 100.0
    );
}

/// Scenario: Jump landing with blown off-road damper causes violent bounce and roll snap.
#[test]
fn test_jump_landing_damping_loss_roll_snap() {
    let dt = 1.0 / 120.0;
    let mut cfg = CarConfig::sand_rail();
    cfg.suspension = SuspensionConfig::long_travel_offroad();

    let mut car = Car::new(cfg);
    // Front-right blown damper seals (20% health)
    car.state.suspension_health[1] = 0.20;
    car.state.elevation = 0.03;
    car.state.vertical_velocity = -5.0; // Touchdown impact speed vz <= -5.0 m/s
    car.state.velocity = Vec2::new(25.0, 0.0);
    car.state.speed = 25.0;

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    car.step(&ctrl, SurfaceType::Dirt, dt);

    assert!(
        car.state.just_landed,
        "Vehicle must have touched down during this step"
    );

    // Asymmetric landing impulses induce roll snap rotation upon touchdown
    assert!(
        car.state.roll_angle.abs() > 0.015 || car.state.angular_velocity.abs() > 0.01,
        "Asymmetric landing with damaged FR damper must induce sharp roll rotation, got roll_angle={:.4}, omega={:.4}",
        car.state.roll_angle,
        car.state.angular_velocity
    );
}

/// Scenario: Low-energy collisions (< 500J) and minor component wear (< 10-15%)
/// are absorbed with zero damage and zero driving penalties (Beads: tdrace-dqwn).
#[test]
fn test_minor_collision_and_deadzone_immunity() {
    let mut car = Car::new(CarConfig::sports_car());

    // 1. Minor bumper collision (< 500 J) produces zero damage
    let nose_contact = car.state.position + car.forward_vector() * 2.0;
    car.apply_collision_damage(nose_contact, 450.0);
    assert_eq!(car.state.chassis_health, 1.0, "Minor impact under 500J must not damage chassis");
    assert_eq!(car.state.engine_health, 1.0, "Minor impact under 500J must not damage engine");
    assert_eq!(car.state.suspension_health, [1.0, 1.0, 1.0, 1.0], "Minor impact under 500J must not damage suspension");

    // 2. Minor asymmetric suspension wear (delta <= 10%) produces 0.0 steering pull
    car.state.suspension_health[0] = 0.92;
    car.state.suspension_health[1] = 1.0;
    assert_eq!(
        car.steering_pull_bias(),
        0.0,
        "Asymmetric front suspension wear <= 10% must remain within the steering pull deadzone"
    );

    // 3. Engine health >= 80% maintains 100% available horsepower
    car.state.engine_health = 0.85;
    assert_eq!(
        car.available_engine_power_ratio(),
        1.0,
        "Engine health >= 80% must deliver 100% engine power ratio"
    );

    // 4. Moderate structural impact (> 500J) deducts damage beyond absorption buffer
    car.state.engine_health = 1.0;
    car.state.chassis_health = 1.0;
    car.apply_collision_damage(nose_contact, 2000.0);
    assert!(car.state.engine_health < 1.0, "Impact exceeding 500J buffer must cause damage");
    assert!(car.state.chassis_health < 1.0, "Impact exceeding 500J buffer must cause chassis damage");
}

