use tdrace_app::module::f1::GtWorldChallengeModule;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::DriverAssistsConfig;
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_gt3_high_speed_straight_line_panic_stop_arcade() {
    let mut config = GtWorldChallengeModule::car_gt3_evo();
    config.assists = DriverAssistsConfig::arcade();
    let mut car = Car::new(config);

    // Accelerate to ~250 km/h (70 m/s)
    let dt = 1.0 / 60.0;
    car.state.velocity = glam::Vec2::new(70.0, 0.0);
    car.state.speed = 70.0;

    // Apply a sudden yaw/steer disturbance while slamming the brakes
    // (simulating a slight steering twitch or road crown bump at 250 km/h)
    car.state.angular_velocity = 0.08; // 0.08 rad/s perturbation

    let mut max_sideslip = 0.0f32;
    let mut max_yaw_rate = 0.0f32;
    let mut stopped = false;

    for frame in 0..600 {
        // First 6 frames (0.1s): slight steering twitch 0.05, then centered
        let steer = if frame < 6 { 0.05 } else { 0.0 };
        let ctrl = CarControls::new(0.0, steer, 1.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        let sideslip = car.state.sideslip_angle.abs();
        let yaw_rate = car.state.angular_velocity.abs();

        if car.state.speed > 2.0 && sideslip > max_sideslip {
            max_sideslip = sideslip;
        }
        if yaw_rate > max_yaw_rate {
            max_yaw_rate = yaw_rate;
        }

        if car.state.speed < 0.1 {
            stopped = true;
            break;
        }
    }

    assert!(stopped, "Vehicle must come to a complete stop");
    assert!(
        max_sideslip < 0.10,
        "Vehicle sideslip must remain tightly controlled under Arcade panic stop (was {:.3} rad = {:.1} deg)",
        max_sideslip,
        max_sideslip.to_degrees()
    );
    assert!(
        max_yaw_rate < 0.35,
        "Vehicle yaw rate must be actively damped by assists (was {:.3} rad/s)",
        max_yaw_rate
    );
    assert!(
        car.state.angle.abs() < 0.15,
        "Vehicle heading must remain straight without spinning (heading change was {:.2} deg)",
        car.state.angle.to_degrees()
    );
}

#[test]
fn test_gt3_high_speed_panic_stop_sport() {
    let mut config = GtWorldChallengeModule::car_gt3_evo();
    config.assists = DriverAssistsConfig::sport();
    let mut car = Car::new(config);

    // High speed 65 m/s (~234 km/h)
    let dt = 1.0 / 60.0;
    car.state.velocity = glam::Vec2::new(65.0, 0.0);
    car.state.speed = 65.0;
    car.state.angular_velocity = 0.05;

    let mut max_sideslip = 0.0f32;
    let mut stopped = false;

    for frame in 0..600 {
        let steer = if frame < 5 { 0.04 } else { 0.0 };
        let ctrl = CarControls::new(0.0, steer, 1.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        let sideslip = car.state.sideslip_angle.abs();
        if car.state.speed > 2.0 && sideslip > max_sideslip {
            max_sideslip = sideslip;
        }

        if car.state.speed < 0.1 {
            stopped = true;
            break;
        }
    }

    assert!(stopped, "Vehicle must stop completely in Sport mode");
    assert!(
        max_sideslip < 0.15,
        "Sport mode sideslip must not spin out under emergency braking (was {:.3} rad = {:.1} deg)",
        max_sideslip,
        max_sideslip.to_degrees()
    );
    assert!(
        car.state.angle.abs() < 0.25,
        "Sport mode must maintain directional stability without spinning out"
    );
}

#[test]
fn test_gt3_brake_pumping_recovery() {
    // Replicates user report: "I try to do quick press and release of the brakes, but still lose control"
    let mut config = GtWorldChallengeModule::car_gt3_evo();
    config.assists = DriverAssistsConfig::arcade();
    let mut car = Car::new(config);

    let dt = 1.0 / 60.0;
    car.state.velocity = glam::Vec2::new(60.0, 0.0);
    car.state.speed = 60.0;

    let mut max_sideslip = 0.0f32;

    // Simulate pumping: 9 frames brake, 6 frames release, repeated 4 times
    for cycle in 0..4 {
        // Press brake hard (9 frames = 0.15s)
        for _ in 0..9 {
            let ctrl = CarControls::new(0.0, 0.0, 1.0, false);
            car.step(&ctrl, SurfaceType::Asphalt, dt);
            max_sideslip = max_sideslip.max(car.state.sideslip_angle.abs());
        }

        // Release brake completely (6 frames = 0.10s) with 0 throttle
        for _ in 0..6 {
            let ctrl = CarControls::new(0.0, 0.0, 0.0, false);
            car.step(&ctrl, SurfaceType::Asphalt, dt);
            max_sideslip = max_sideslip.max(car.state.sideslip_angle.abs());

            // Check that rear wheels are NOT in locked slide upon release
            let rear_locked = car.state.wheels[2].is_skidding && car.state.wheels[2].slip_ratio.abs() > 0.90;
            assert!(
                !rear_locked,
                "Cycle {cycle}: Rear wheels must not remain in locked slide when brake is released"
            );
        }
    }

    assert!(
        max_sideslip < 0.06,
        "Brake pumping must remain completely stable without tail-wag or snap oversteer (was {:.3} rad)",
        max_sideslip
    );
}

#[test]
fn test_gt3_trail_braking_into_corner() {
    let mut config = GtWorldChallengeModule::car_gt3_evo();
    config.assists = DriverAssistsConfig::arcade();
    let mut car = Car::new(config);

    let dt = 1.0 / 60.0;
    // Enter corner at 50 m/s (180 km/h) while applying 50% trail-braking and moderate steering
    car.state.velocity = glam::Vec2::new(50.0, 0.0);
    car.state.speed = 50.0;

    let mut max_sideslip = 0.0f32;
    for _ in 0..60 {
        // Steering right (steer = 0.25) while trail-braking (brake = 0.50)
        let ctrl = CarControls::new(0.0, 0.25, 0.50, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        let sideslip = car.state.sideslip_angle.abs();
        max_sideslip = max_sideslip.max(sideslip);
    }

    // Vehicle should turn smoothly without snapping into a 180 spin
    assert!(
        car.state.angular_velocity.abs() > 0.15,
        "Vehicle should respond to steering during trail-braking"
    );
    assert!(
        max_sideslip < 0.35,
        "Trail-braking sideslip must not enter uncontrolled snap spin (was {:.3} rad)",
        max_sideslip
    );
}
