use glam::Vec2;
use std::f32::consts::PI;
use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

#[test]
fn test_extreme_and_invalid_inputs_resilience() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // Test extreme numeric inputs
    let extreme_controls = [
        CarControls { throttle: 1e6, steer: 1e6, brake: 1e6, handbrake: true, reverse: true },
        CarControls { throttle: -1e6, steer: -1e6, brake: -1e6, handbrake: false, reverse: false },
        CarControls { throttle: f32::INFINITY, steer: f32::INFINITY, brake: f32::INFINITY, handbrake: true, reverse: false },
        CarControls { throttle: f32::NEG_INFINITY, steer: f32::NEG_INFINITY, brake: f32::NEG_INFINITY, handbrake: false, reverse: true },
    ];

    for ctrl in &extreme_controls {
        for _ in 0..100 {
            car.step(ctrl, SurfaceType::Asphalt, dt);
        }
        assert!(!car.state.position.x.is_nan() && !car.state.position.y.is_nan(), "Position must not be NaN");
        assert!(!car.state.velocity.x.is_nan() && !car.state.velocity.y.is_nan(), "Velocity must not be NaN");
        assert!(!car.state.angle.is_nan(), "Angle must not be NaN");
        assert!(!car.state.angular_velocity.is_nan(), "Angular velocity must not be NaN");
        assert!(!car.state.steer_angle.is_nan(), "Steer angle must not be NaN");
        assert!(car.state.speed < 200.0, "Speed must remain physically bounded, was {}", car.state.speed);
    }
}

#[test]
fn test_nan_input_handling() {
    let mut car = Car::new(CarConfig::sports_car());
    let ctrl_nan = CarControls {
        throttle: f32::NAN,
        steer: f32::NAN,
        brake: f32::NAN,
        handbrake: false,
        reverse: false,
    };
    car.step(&ctrl_nan, SurfaceType::Asphalt, 1.0 / 60.0);
    println!("State after NaN input: pos={:?}, vel={:?}, steer={}", car.state.position, car.state.velocity, car.state.steer_angle);
}

#[test]
fn test_timestep_extremes() {
    let mut car = Car::new(CarConfig::sports_car());
    let ctrl = CarControls::new(1.0, 0.5, 0.0, false);

    // dt = 0.0
    car.step(&ctrl, SurfaceType::Asphalt, 0.0);
    assert_eq!(car.state.speed, 0.0);
    assert_eq!(car.state.position, Vec2::ZERO);

    // dt = 1e-9 (tiny timestep)
    for _ in 0..1000 {
        car.step(&ctrl, SurfaceType::Asphalt, 1e-9);
    }
    assert!(!car.state.speed.is_nan());
    assert!(!car.state.position.x.is_nan());

    // dt = 0.1 (large timestep for 10Hz tick)
    let mut car2 = Car::new(CarConfig::sports_car());
    for _ in 0..100 {
        car2.step(&ctrl, SurfaceType::Asphalt, 0.1);
    }
    assert!(!car2.state.position.x.is_nan());
    assert!(!car2.state.velocity.x.is_nan());
    assert!(car2.state.speed < 100.0);
}

#[test]
fn test_full_reverse_to_full_forward_straight() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // 1. Accelerate backwards to top reverse speed
    let mut reverse_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    reverse_ctrl.reverse = true;
    for _ in 0..300 {
        car.step(&reverse_ctrl, SurfaceType::Asphalt, dt);
    }
    let reverse_speed = car.state.speed;
    println!("Straight Max reverse speed: {:.2} m/s ({:.1} km/h)", reverse_speed, reverse_speed * 3.6);
    assert!(car.state.local_velocity.x < -5.0);

    // 2. Straight forward throttle
    let forward_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    for step in 0..300 {
        car.step(&forward_ctrl, SurfaceType::Asphalt, dt);
        if step % 50 == 0 {
            println!("  Step {}: v_long={:.2}, speed={:.2}, x={:.2}", step, car.state.local_velocity.x, car.state.speed, car.state.position.x);
        }
    }
    // In 5 seconds (300 steps), vehicle reverses from -14.38 m/s to +12.0 m/s (~5.2 m/s^2 acceleration against drag)
    assert!(car.state.local_velocity.x > 10.0, "Car must accelerate forward in straight line, was {}", car.state.local_velocity.x);
}

#[test]
fn test_full_reverse_to_full_forward_and_steering() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // 1. Accelerate backwards
    let mut reverse_ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    reverse_ctrl.reverse = true;
    for _ in 0..300 {
        car.step(&reverse_ctrl, SurfaceType::Asphalt, dt);
    }

    // 2. Slam forward + full right steer
    let forward_steer_ctrl = CarControls::new(1.0, 1.0, 0.0, false);
    for step in 0..300 {
        car.step(&forward_steer_ctrl, SurfaceType::Asphalt, dt);
        if step % 50 == 0 {
            println!("  Steer Step {}: v_long={:.2}, v_lat={:.2}, omega={:.2}, angle={:.2}", 
                step, car.state.local_velocity.x, car.state.local_velocity.y, car.state.angular_velocity, car.state.angle);
        }
    }
}

#[test]
fn test_high_angular_velocity_spin_damping_across_all_surfaces() {
    let dt = 1.0 / 60.0;
    let initial_spin = 50.0; // 50 rad/s (~477 RPM)

    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::Sand,
        SurfaceType::Oil,
        SurfaceType::Ice,
    ];

    for &surf in &surfaces {
        let mut car = Car::new(CarConfig::sports_car());
        car.state.angular_velocity = initial_spin;

        let neutral_ctrl = CarControls::default();

        // Simulate 3 seconds of spinning with no control inputs
        for _ in 0..180 {
            car.step(&neutral_ctrl, surf, dt);
        }

        println!(
            "Surface {:?}: Initial yaw rate = {:.1} rad/s -> Final yaw rate = {:.3} rad/s",
            surf, initial_spin, car.state.angular_velocity
        );

        assert!(
            !car.state.angular_velocity.is_nan(),
            "Yaw rate became NaN on {:?}",
            surf
        );
        assert!(
            car.state.angular_velocity.abs() < initial_spin * 0.20,
            "Yaw rate failed to damp on {:?}: was {:.3} rad/s",
            surf,
            car.state.angular_velocity
        );
    }
}

#[test]
fn test_friction_circle_conservation_under_random_fuzz() {
    let mut car = Car::new(CarConfig::sports_car());
    let dt = 1.0 / 60.0;

    // Fuzz test 10,000 steps with random control inputs
    let mut state: u64 = 0xCAFEBABE_98765432;
    let mut rng = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((state >> 32) as u32 as f32) / (u32::MAX as f32)
    };

    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::Sand,
        SurfaceType::Oil,
        SurfaceType::Ice,
    ];

    for step in 0..10_000 {
        let ctrl = CarControls {
            throttle: rng(),
            steer: rng() * 2.0 - 1.0,
            brake: if rng() > 0.7 { rng() } else { 0.0 },
            handbrake: rng() > 0.85,
            reverse: rng() > 0.95,
        };
        let surf = surfaces[(rng() * 6.0) as usize % 6];
        car.step(&ctrl, surf, dt);

        for (i, wheel) in car.state.wheels.iter().enumerate() {
            let fx = wheel.longitudinal_force;
            let fy = wheel.lateral_force;
            let total_force = (fx * fx + fy * fy).sqrt();

            let mu = wheel.surface.friction_coefficient();
            let max_allowed_force = mu * wheel.normal_load;

            let tolerance = 1e-2;
            assert!(
                total_force <= max_allowed_force + tolerance,
                "Step {step} Wheel {i} violated friction circle: total_force={total_force}, max_allowed={max_allowed_force} (Fz={}, mu={})",
                wheel.normal_load, mu
            );

            assert!(
                wheel.normal_load > 0.0,
                "Normal load must remain strictly positive"
            );
        }
    }
}

#[test]
fn test_telemetry_wheel_positions_world_exact_geometry() {
    let car = Car::new(CarConfig::sports_car())
        .with_pose(Vec2::new(100.0, 50.0), PI / 3.0); // 60 deg angle

    let wheel_pos = car.wheel_positions_world();
    let angle = PI / 3.0;
    let fwd = Vec2::new(angle.cos(), angle.sin());
    let right = Vec2::new(angle.sin(), -angle.cos());

    let lf = car.config.cg_to_front;
    let lr = car.config.cg_to_rear;
    let half_w = car.config.track_width * 0.5;

    let expected_fl = car.state.position + fwd * lf - right * half_w;
    let expected_fr = car.state.position + fwd * lf + right * half_w;
    let expected_rl = car.state.position - fwd * lr - right * half_w;
    let expected_rr = car.state.position - fwd * lr + right * half_w;

    assert!((wheel_pos[0] - expected_fl).length() < 1e-5);
    assert!((wheel_pos[1] - expected_fr).length() < 1e-5);
    assert!((wheel_pos[2] - expected_rl).length() < 1e-5);
    assert!((wheel_pos[3] - expected_rr).length() < 1e-5);
}
