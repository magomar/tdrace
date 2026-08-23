use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

#[test]
fn test_steering_response_and_turning_radius() {
    let dt = 1.0 / 60.0;

    // 1. Low speed tight turn
    let mut low_speed_car = Car::new(CarConfig::sports_car());
    let low_speed_ctrl = CarControls::new(0.2, 1.0, 0.0, false);
    // Run for a full circle
    let start_pos = low_speed_car.state.position;
    for _ in 0..300 {
        low_speed_car.step(&low_speed_ctrl, SurfaceType::Asphalt, dt);
    }
    let max_dist_low = (low_speed_car.state.position - start_pos).length();

    // 2. High speed wider turn
    let mut high_speed_car = Car::new(CarConfig::sports_car());
    // Get up to speed
    for _ in 0..180 {
        high_speed_car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    let high_speed_pos = high_speed_car.state.position;
    let high_speed_ctrl = CarControls::new(0.8, 1.0, 0.0, false);
    for _ in 0..300 {
        high_speed_car.step(&high_speed_ctrl, SurfaceType::Asphalt, dt);
    }
    let max_dist_high = (high_speed_car.state.position - high_speed_pos).length();

    println!(
        "Turning response: Low-speed arc span = {:.2}m, High-speed arc span = {:.2}m",
        max_dist_low, max_dist_high
    );
    assert!(
        max_dist_high > max_dist_low * 1.5,
        "High speed turning radius must be significantly larger than low speed radius"
    );
}

#[test]
fn test_cornering_lateral_weight_transfer() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // Accelerate to ~50 km/h
    for _ in 0..90 {
        car.step(&CarControls::new(0.6, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }

    // Steer hard right (steer = +1.0)
    for _ in 0..20 {
        car.step(&CarControls::new(0.6, 1.0, 0.0, false), SurfaceType::Asphalt, dt);
    }

    // In a right turn, centrifugal force pushes weight to outer (left) wheels
    let left_load = car.state.wheels[0].normal_load + car.state.wheels[2].normal_load; // FL + RL
    let right_load = car.state.wheels[1].normal_load + car.state.wheels[3].normal_load; // FR + RR

    println!(
        "Right turn weight transfer: Left (outer) load = {:.1} N, Right (inner) load = {:.1} N",
        left_load, right_load
    );

    assert!(
        left_load > right_load * 1.2,
        "Outer (left) wheels should carry significantly more load during a right turn (left={left_load}, right={right_load})"
    );
}

#[test]
fn test_drift_initiation_and_counter_steer_recovery() {
    let mut car = Car::new(CarConfig::drift_car());
    let dt = 1.0 / 60.0;

    // 1. Build up entry speed (~80 km/h)
    for _ in 0..220 {
        car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    println!("Drift entry speed: {:.1} km/h", car.speed_kmh());
    assert!(car.speed_kmh() > 55.0, "Car must reach entry speed, was {:.1} km/h", car.speed_kmh());

    // 2. Initiate drift: steer hard left + rip handbrake
    let mut drifted = false;
    let mut max_sideslip: f32 = 0.0;
    let mut max_skid_intensity: f32 = 0.0;

    for _ in 0..40 {
        let ctrl = CarControls::new(0.8, -1.0, 0.0, true);
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        let sideslip = car.state.sideslip_angle.abs();
        if sideslip > max_sideslip {
            max_sideslip = sideslip;
        }
        for w in &car.state.wheels {
            if w.skid_intensity > max_skid_intensity {
                max_skid_intensity = w.skid_intensity;
            }
        }
        if car.state.is_drifting {
            drifted = true;
        }
    }

    println!(
        "Drift Initiation: drifted={}, max_sideslip={:.2} rad ({:.1} deg), max_skid_intensity={:.2}, drift_score={:.2}",
        drifted, max_sideslip, max_sideslip.to_degrees(), max_skid_intensity, car.state.drift_score
    );

    assert!(
        drifted,
        "Handbrake turn at speed must trigger drift state (is_drifting=true)"
    );
    assert!(
        max_sideslip > 0.18,
        "Sideslip angle ({:.2} rad) must exceed drift threshold",
        max_sideslip
    );
    assert!(
        max_skid_intensity > 0.6,
        "Skid intensity ({:.2}) must be high during handbrake slide",
        max_skid_intensity
    );
    assert!(
        car.state.drift_score > 0.0,
        "Drift score must accumulate during drift"
    );

    // 3. Counter-steer recovery: steer opposite to the slide (steer right + throttle)
    for _ in 0..30 {
        let recover_ctrl = CarControls::new(0.8, 0.8, 0.0, false);
        car.step(&recover_ctrl, SurfaceType::Asphalt, dt);
    }

    // Now straighten wheel and accelerate
    for _ in 0..80 {
        let straight_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
        car.step(&straight_ctrl, SurfaceType::Asphalt, dt);
    }

    let recovered_sideslip = car.state.sideslip_angle.abs();
    let recovered_angular_vel = car.state.angular_velocity.abs();

    println!(
        "Counter-steer recovery: final sideslip = {:.3} rad, yaw rate = {:.3} rad/s, speed = {:.1} km/h",
        recovered_sideslip, recovered_angular_vel, car.speed_kmh()
    );

    assert!(
        recovered_sideslip < 0.20,
        "Vehicle should recover from drift and align heading (sideslip={:.3} rad)",
        recovered_sideslip
    );
    assert!(
        recovered_angular_vel < 0.8,
        "Yaw rate should stabilize after recovery (angular_velocity={:.3} rad/s)",
        recovered_angular_vel
    );
}

#[test]
fn test_telemetry_output_consistency() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    for step in 0..100 {
        let steer = (step as f32 * 0.05).sin();
        let ctrl = CarControls::new(0.8, steer, 0.0, false);
        car.step(&ctrl, SurfaceType::Asphalt, dt);

        for (i, wheel) in car.state.wheels.iter().enumerate() {
            assert!(
                wheel.normal_load > 0.0,
                "Wheel {} normal load must remain positive (was {})",
                i,
                wheel.normal_load
            );
            assert!(
                wheel.skid_intensity >= 0.0 && wheel.skid_intensity <= 1.0,
                "Skid intensity must be clamped to [0, 1] (was {})",
                wheel.skid_intensity
            );
            assert!(
                !wheel.world_velocity.x.is_nan() && !wheel.world_velocity.y.is_nan(),
                "Wheel world velocity must not be NaN"
            );
        }
    }
}
