use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};
use tdrace_core::physics::config::DriverAssistsConfig;

#[test]
fn test_off_throttle_engine_braking_deceleration_across_all_presets() {
    let dt = 1.0 / 60.0;
    let presets = [
        ("SportsCar", CarConfig::sports_car(), 0.10, 0.22),
        ("DriftCar", CarConfig::drift_car(), 0.08, 0.20),
        ("Kart", CarConfig::kart(), 0.12, 0.30),
        ("RallyCar", CarConfig::rally_car(), 0.10, 0.24),
    ];

    for (name, config, min_g, max_g) in presets {
        let mut car = Car::new(config);
        // Accelerate to ~80 km/h (or 60 km/h for kart)
        let target_speed = if name == "Kart" { 60.0 } else { 80.0 };
        while car.speed_kmh() < target_speed {
            car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
        }

        let speed_init = car.speed_kmh();
        let coast_ctrl = CarControls::default();

        // Coast for 1.0s (60 steps)
        for _ in 0..60 {
            car.step(&coast_ctrl, SurfaceType::Asphalt, dt);
        }

        let speed_after_1s = car.speed_kmh();
        let decel_mps2 = (speed_init - speed_after_1s) / 3.6 / 1.0;
        let decel_g = decel_mps2 / 9.81;

        println!(
            "{name} Engine Braking: Init={speed_init:.1} km/h -> 1s={speed_after_1s:.1} km/h | Decel={decel_mps2:.2} m/s² ({decel_g:.3} g) [Expected: {min_g}g - {max_g}g]"
        );

        assert!(
            decel_g >= min_g && decel_g <= max_g,
            "{name} engine braking deceleration ({decel_g:.3} g) must be in realistic range [{min_g}, {max_g}]"
        );
    }
}

#[test]
fn test_lift_off_forward_pitch_weight_transfer() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Accelerate to ~80 km/h
    for _ in 0..200 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    let accel_front_load = car.state.wheels[0].normal_load + car.state.wheels[1].normal_load;

    // Lift off throttle (coasting with engine braking)
    let coast_ctrl = CarControls::default();
    for _ in 0..15 {
        car.step(&coast_ctrl, SurfaceType::Asphalt, dt);
    }
    let lift_off_front_load = car.state.wheels[0].normal_load + car.state.wheels[1].normal_load;

    println!(
        "Lift-Off Weight Transfer: Accel Front Load={accel_front_load:.1} N -> Lift-Off Front Load={lift_off_front_load:.1} N (Delta = +{:.1} N)",
        lift_off_front_load - accel_front_load
    );

    assert!(
        lift_off_front_load > accel_front_load + 200.0,
        "Throttle lift-off must transfer weight forward to front wheels (accel={accel_front_load}, lift={lift_off_front_load})"
    );
}

#[test]
fn test_abs_preserves_steering_under_threshold_braking() {
    let dt = 1.0 / 60.0;

    // 1. With ABS enabled (Arcade preset)
    let mut car_abs = Car::new(CarConfig::sports_car());
    while car_abs.speed_kmh() < 80.0 {
        car_abs.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    let init_angle_abs = car_abs.state.angle;
    let mut abs_ever_active = false;

    // Apply simultaneous steer and 100% brake from straight line
    for _ in 0..35 {
        car_abs.step(&CarControls::new(0.0, 0.6, 1.0, false), SurfaceType::Asphalt, dt);
        if car_abs.state.abs_active {
            abs_ever_active = true;
        }
    }
    let heading_delta_abs = (car_abs.state.angle - init_angle_abs).abs().to_degrees();

    // 2. With ABS disabled (Raw preset)
    let mut raw_cfg = CarConfig::sports_car();
    raw_cfg.assists = DriverAssistsConfig::raw();
    let mut car_raw = Car::new(raw_cfg);
    while car_raw.speed_kmh() < 80.0 {
        car_raw.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    let init_angle_raw = car_raw.state.angle;

    for _ in 0..35 {
        car_raw.step(&CarControls::new(0.0, 0.6, 1.0, false), SurfaceType::Asphalt, dt);
    }
    let heading_delta_raw = (car_raw.state.angle - init_angle_raw).abs().to_degrees();

    println!(
        "ABS Comparison (Turn-in under full braking): ABS Active={abs_ever_active} | ABS HeadingΔ={heading_delta_abs:.1}° | Raw HeadingΔ={heading_delta_raw:.1}°"
    );

    assert!(abs_ever_active, "ABS must activate during heavy braking while steering");
    assert!(
        heading_delta_abs >= 11.0,
        "ABS car must turn at least 11.0 degrees into corner while braking (got {heading_delta_abs:.1}°)"
    );
    assert!(
        car_abs.state.sideslip_angle.abs() < 0.20,
        "ABS must prevent vehicle instability / rear spinout during heavy braking turn (sideslip={:.3} rad)",
        car_abs.state.sideslip_angle.abs()
    );
}

#[test]
fn test_aerodynamic_downforce_scales_with_speed() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Standstill vertical load
    car.step(&CarControls::default(), SurfaceType::Asphalt, dt);
    let static_total_fz: f32 = car.state.wheels.iter().map(|w| w.normal_load).sum();

    // Accelerate to ~140 km/h (~38.9 m/s)
    while car.speed_kmh() < 140.0 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    let high_speed_total_fz: f32 = car.state.wheels.iter().map(|w| w.normal_load).sum();
    let delta_downforce = high_speed_total_fz - static_total_fz;

    println!(
        "Aerodynamic Downforce @ 140 km/h: Static Fz = {static_total_fz:.0} N -> High-Speed Fz = {high_speed_total_fz:.0} N (Downforce = +{delta_downforce:.0} N)"
    );

    assert!(
        delta_downforce > 800.0,
        "Aerodynamic downforce at 140 km/h must add significant vertical load (got +{delta_downforce:.0} N)"
    );
}

#[test]
fn test_high_speed_cornering_grip_and_esc_stability() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Accelerate to 90 km/h (~25 m/s)
    while car.speed_kmh() < 90.0 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    // Steady-state cornering at moderate steer
    let ctrl = CarControls::new(0.6, 0.35, 0.0, false);
    let mut max_ay = 0.0f32;
    let mut max_yaw = 0.0f32;

    for _ in 0..60 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);
        let ay = car.state.acceleration_local.y.abs();
        let yaw = car.state.angular_velocity.abs();
        if ay > max_ay { max_ay = ay; }
        if yaw > max_yaw { max_yaw = yaw; }
    }

    println!(
        "90 km/h Cornering: Max Ay = {max_ay:.1} m/s² ({:.2}g), Max Yaw Rate = {max_yaw:.2} rad/s",
        max_ay / 9.81
    );

    assert!(
        max_ay / 9.81 >= 0.75,
        "Sports car must generate at least 0.75g lateral grip in 90 km/h curve (got {:.2}g)",
        max_ay / 9.81
    );
}
