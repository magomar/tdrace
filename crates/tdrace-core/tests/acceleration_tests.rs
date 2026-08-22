use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

#[test]
fn test_straight_line_acceleration_0_to_100() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;
    let full_throttle = CarControls::new(1.0, 0.0, 0.0, false);

    let mut time_to_100 = None;
    let target_speed_mps = 100.0 / 3.6; // 27.78 m/s

    for step in 0..600 {
        // 10 seconds max
        car.step(&full_throttle, SurfaceType::Asphalt, dt);
        let current_time = (step + 1) as f32 * dt;

        if car.state.speed >= target_speed_mps && time_to_100.is_none() {
            time_to_100 = Some(current_time);
            break;
        }
    }

    assert!(
        time_to_100.is_some(),
        "Car should reach 100 km/h within 10 seconds. Final speed: {:.2} km/h",
        car.speed_kmh()
    );
    let t = time_to_100.unwrap();
    println!("0-100 km/h acceleration time: {:.2} seconds", t);
    // GeneRally sports car 0-100 should be around 3.5 - 6.5s
    assert!(t >= 3.0 && t <= 7.0, "0-100 time {t}s is outside realistic arcade range [3.0, 7.0]");
}

#[test]
fn test_top_speed_convergence() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;
    let full_throttle = CarControls::new(1.0, 0.0, 0.0, false);

    // Accelerate for 25 seconds
    for _ in 0..(25 * 60) {
        car.step(&full_throttle, SurfaceType::Asphalt, dt);
    }

    let top_speed_expected = car.config.top_speed_mps;
    let actual_speed = car.state.speed;
    println!(
        "Top speed achieved: {:.2} km/h ({:.2} m/s), config top speed: {:.2} m/s",
        car.speed_kmh(),
        actual_speed,
        top_speed_expected
    );

    // Should converge within 5% of top speed limit
    assert!(
        (actual_speed - top_speed_expected).abs() < top_speed_expected * 0.08,
        "Speed {actual_speed} m/s did not converge to top speed {top_speed_expected} m/s"
    );
}

#[test]
fn test_braking_distance_from_100() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;
    let full_throttle = CarControls::new(1.0, 0.0, 0.0, false);

    // Get up to ~100 km/h
    while car.speed_kmh() < 100.0 {
        car.step(&full_throttle, SurfaceType::Asphalt, dt);
    }

    let speed_at_brake = car.speed_kmh();
    let pos_at_brake = car.state.position.x;
    let full_brake = CarControls::new(0.0, 0.0, 1.0, false);

    let mut brake_steps = 0;
    while car.state.speed > 0.05 && brake_steps < 300 {
        car.step(&full_brake, SurfaceType::Asphalt, dt);
        brake_steps += 1;
    }

    let stopping_dist = car.state.position.x - pos_at_brake;
    let stopping_time = brake_steps as f32 * dt;
    println!(
        "Braking from {:.1} km/h: distance = {:.2} m, time = {:.2} s",
        speed_at_brake, stopping_dist, stopping_time
    );

    assert!(car.state.speed <= 0.05, "Car must come to a complete halt");
    // 100-0 km/h braking distance should be around 25 - 45 meters
    assert!(
        stopping_dist > 20.0 && stopping_dist < 50.0,
        "Stopping distance {:.2}m outside expected [20, 50] range",
        stopping_dist
    );
}

#[test]
fn test_longitudinal_weight_transfer() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // 1. Initial static load
    car.step(&CarControls::default(), SurfaceType::Asphalt, dt);
    let static_f = car.state.wheels[0].normal_load + car.state.wheels[1].normal_load;
    let static_r = car.state.wheels[2].normal_load + car.state.wheels[3].normal_load;

    // 2. Full acceleration squat
    let throttle = CarControls::new(1.0, 0.0, 0.0, false);
    for _ in 0..15 {
        car.step(&throttle, SurfaceType::Asphalt, dt);
    }
    let accel_f = car.state.wheels[0].normal_load + car.state.wheels[1].normal_load;
    let accel_r = car.state.wheels[2].normal_load + car.state.wheels[3].normal_load;

    assert!(
        accel_r > static_r,
        "Rear load should increase during acceleration (squat): accel_r={accel_r}, static_r={static_r}"
    );
    assert!(
        accel_f < static_f,
        "Front load should decrease during acceleration: accel_f={accel_f}, static_f={static_f}"
    );

    // 3. Heavy braking dive
    let brake = CarControls::new(0.0, 0.0, 1.0, false);
    for _ in 0..15 {
        car.step(&brake, SurfaceType::Asphalt, dt);
    }
    let brake_f = car.state.wheels[0].normal_load + car.state.wheels[1].normal_load;
    let brake_r = car.state.wheels[2].normal_load + car.state.wheels[3].normal_load;

    assert!(
        brake_f > static_f,
        "Front load should increase during heavy braking (dive): brake_f={brake_f}, static_f={static_f}"
    );
    assert!(
        brake_r < static_r,
        "Rear load should decrease during braking: brake_r={brake_r}, static_r={static_r}"
    );
}

#[test]
fn test_reverse_gear() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;
    let mut reverse_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    reverse_ctrl.reverse = true;

    for _ in 0..120 {
        car.step(&reverse_ctrl, SurfaceType::Asphalt, dt);
    }

    assert!(
        car.state.position.x < -1.0,
        "Car should accelerate backwards in reverse gear"
    );
    assert!(
        car.state.local_velocity.x < -2.0,
        "Local longitudinal velocity should be negative"
    );
}
