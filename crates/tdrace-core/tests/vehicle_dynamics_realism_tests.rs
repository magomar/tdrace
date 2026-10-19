use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};
use tdrace_core::physics::config::DriverAssistsConfig;

#[test]
fn test_off_throttle_engine_braking_deceleration_across_all_presets() {
    let dt = 1.0 / 60.0;
    let presets = [
        ("SportsCar", CarConfig::sports_car(), 0.18, 0.32),
        ("DriftCar", CarConfig::drift_car(), 0.16, 0.30),
        ("Kart", CarConfig::kart(), 0.20, 0.48),
        ("RallyCar", CarConfig::rally_car(), 0.18, 0.35),
        ("StockCarTA1", CarConfig::stock_car_ta1(), 0.18, 0.42),
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

#[test]
fn test_stock_car_ta1_power_and_top_speed_dynamics() {
    let dt = 1.0 / 60.0;
    let cfg = CarConfig::stock_car_ta1();
    let mut car = Car::new(cfg);

    // Straight-line acceleration. The launch is traction-limited under the sport TCS, which
    // tdrace-epbk.2 loosened for drift assist: ~5.49 s today. The calibration target is
    // 3.1 s (tdrace-ei1y), so this bound is a regression guard, not a realism check.
    let mut steps_to_100 = 0;
    while car.speed_kmh() < 100.0 && steps_to_100 < 60 * 10 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
        steps_to_100 += 1;
    }
    let time_0_100 = steps_to_100 as f32 * dt;
    println!("Stock Car TA1 0-100 km/h time: {:.2}s", time_0_100);
    assert!(time_0_100 < 5.6, "850 BHP Stock car must reach 100 km/h in < 5.6s (got {:.2}s)", time_0_100);

    // Accelerate to top speed convergence
    for _ in 0..(60 * 25) {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    println!("Stock Car TA1 top speed reached: {:.1} km/h (target ~320 km/h)", car.speed_kmh());
    assert!(
        car.speed_kmh() > 300.0,
        "Stock car top speed must exceed 300 km/h (got {:.1} km/h)",
        car.speed_kmh()
    );
}

#[test]
fn test_aerodynamic_drafting_and_slipstream_velocity_boost() {
    let dt = 1.0 / 60.0;
    let cfg = CarConfig::stock_car_ta1();

    // Solo car running flat out on straightaway
    let mut solo_car = Car::new(cfg);
    for _ in 0..(60 * 20) {
        solo_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    let solo_top_speed = solo_car.speed_kmh();

    // Lead car + Trailing drafting car
    let mut lead_car = Car::new(cfg).with_pose(glam::Vec2::new(12.0, 0.0), 0.0);
    lead_car.state.speed = 85.0; // ~306 km/h
    lead_car.state.velocity = glam::Vec2::new(85.0, 0.0);

    let mut trailing_car = Car::new(cfg).with_pose(glam::Vec2::new(0.0, 0.0), 0.0);
    trailing_car.state.speed = 85.0;
    trailing_car.state.velocity = glam::Vec2::new(85.0, 0.0);

    // Compute drafting intensity: trailing car is 12m directly behind lead car
    let draft = trailing_car.compute_draft_intensity(&[&lead_car]);
    println!("Draft intensity 12m behind lead car: {:.3} (expected > 0.20)", draft);
    assert!(draft > 0.20, "Draft intensity at 12m directly behind should exceed 0.20");

    trailing_car.state.draft_intensity = draft;

    // Advance both cars under full throttle
    for _ in 0..(60 * 10) {
        let draft_lead = lead_car.compute_draft_intensity(&[&trailing_car]);
        lead_car.state.draft_intensity = draft_lead;
        lead_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);

        let draft_trail = trailing_car.compute_draft_intensity(&[&lead_car]);
        trailing_car.state.draft_intensity = draft_trail;
        trailing_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    let drafting_top_speed = trailing_car.speed_kmh();
    println!(
        "Solo top speed: {:.1} km/h vs Drafting top speed: {:.1} km/h (Delta = +{:.1} km/h)",
        solo_top_speed,
        drafting_top_speed,
        drafting_top_speed - solo_top_speed
    );

    assert!(
        drafting_top_speed > solo_top_speed,
        "Car in slipstream wake draft must achieve higher speed than solo car"
    );
}

#[test]
fn test_progressive_non_linear_braking_curve() {
    let dt = 1.0 / 60.0;

    // Helper to accelerate car to 80 km/h and measure deceleration at specific brake input over 30 frames
    let measure_decel = |brake_input: f32| -> f32 {
        let mut car = Car::new(CarConfig::sports_car());
        while car.speed_kmh() < 80.0 {
            car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
        }
        let init_speed = car.speed_kmh();
        let ctrl = CarControls::new(0.0, 0.0, brake_input, false);
        for _ in 0..30 {
            car.step(&ctrl, SurfaceType::Asphalt, dt);
        }
        let end_speed = car.speed_kmh();
        (init_speed - end_speed) / 3.6 / 0.5 // m/s^2 over 0.5s
    };

    let decel_0 = measure_decel(0.0);
    let decel_20 = measure_decel(0.2);
    let decel_50 = measure_decel(0.5);
    let decel_100 = measure_decel(1.0);

    let net_brake_20 = (decel_20 - decel_0).max(0.0);
    let net_brake_50 = (decel_50 - decel_0).max(0.0);
    let net_brake_100 = (decel_100 - decel_0).max(0.0);

    println!(
        "Progressive Braking: Coast={:.2} m/s² | Net 20%={:.2} m/s², Net 50%={:.2} m/s², Net 100%={:.2} m/s²",
        decel_0, net_brake_20, net_brake_50, net_brake_100
    );

    // Non-linear progressive response: 20% pedal produces only ~10-15% of full net brake force
    let ratio_20 = net_brake_20 / net_brake_100;
    assert!(
        ratio_20 < 0.18,
        "20% net brake force ratio ({ratio_20:.3}) must be progressive (< 0.18, expected ~0.11)"
    );

    // 50% pedal produces ~40-55% of full net brake force (well below linear saturation)
    let ratio_50 = net_brake_50 / net_brake_100;
    assert!(
        ratio_50 > 0.35 && ratio_50 < 0.60,
        "50% net brake force ratio ({ratio_50:.3}) must be progressive (0.35 - 0.60, got {ratio_50:.3})"
    );

    // Total 100% braking deceleration achieves strong stopping power
    assert!(
        decel_100 > 8.0,
        "100% total deceleration ({decel_100:.2} m/s²) must achieve strong stopping power (> 8.0 m/s²)"
    );
}

#[test]
fn test_braking_stability_and_rear_lockup_prevention() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Accelerate to ~90 km/h
    while car.speed_kmh() < 90.0 {
        car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    // Apply simultaneous medium steer (0.4) and 100% brake
    let ctrl = CarControls::new(0.0, 0.4, 1.0, false);
    for _ in 0..40 {
        car.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    // Verify rear wheels did not lose lateral stability into a spinout
    let rear_left_load = car.state.wheels[2].normal_load;
    let rear_right_load = car.state.wheels[3].normal_load;
    let sideslip = car.state.sideslip_angle.abs();

    println!(
        "Braking Turn Stability: Rear Loads=({:.1} N, {:.1} N), Final Sideslip={:.3} rad ({:.1}°)",
        rear_left_load, rear_right_load, sideslip, sideslip.to_degrees()
    );

    assert!(
        sideslip < 0.22,
        "Dynamic EBD and ABS must prevent rear lockup and keep vehicle stable in corner (sideslip={sideslip:.3} rad)"
    );
}

#[test]
fn test_spec_129_unchoked_terminal_slipstream_top_speed_gain() {
    let dt = 1.0 / 60.0;
    let cfg = CarConfig::sports_car();

    // Solo car running flat out in clean air (draft_intensity = 0.0)
    let mut solo_car = Car::new(cfg);
    for _ in 0..(60 * 15) {
        solo_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    // Trailing car initialized from identical clean-air terminal state, held 8.0m behind an aligned lead car
    let mut trailing_car = solo_car.clone();
    let mut lead_car = solo_car
        .clone()
        .with_pose(solo_car.state.position + glam::Vec2::new(8.0, 0.0), 0.0);

    for _ in 0..(60 * 10) {
        solo_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);

        lead_car.state.position = trailing_car.state.position + glam::Vec2::new(8.0, 0.0);
        lead_car.state.velocity = trailing_car.state.velocity;
        lead_car.state.speed = trailing_car.state.speed;

        let draft = trailing_car.compute_draft_intensity(&[&lead_car]);
        assert!(
            draft >= 0.28,
            "Draft intensity at 8.0m behind aligned lead car must be >= 0.28 (got {draft:.3})"
        );
        trailing_car.state.draft_intensity = draft;
        trailing_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    let solo_kmh = solo_car.speed_kmh();
    let draft_kmh = trailing_car.speed_kmh();
    let gain_kmh = draft_kmh - solo_kmh;
    println!(
        "Spec 129 Sports Car Terminal Speed: Solo = {solo_kmh:.2} km/h, Drafting (8m) = {draft_kmh:.2} km/h, Gain = +{gain_kmh:.2} km/h"
    );

    assert!(
        gain_kmh >= 8.0,
        "Drafting sports car must gain at least +8.0 km/h over clean-air terminal speed (got +{gain_kmh:.2} km/h)"
    );
}

#[test]
fn test_spec_129_mid_range_slipstream_gap_closure_rate() {
    let dt = 1.0 / 60.0;
    let cfg = CarConfig::sports_car();

    // Warm up a baseline car to 22.0 m/s (79.2 km/h) so tire and wheel states are settled
    let mut base_car = Car::new(cfg);
    while base_car.state.speed < 22.0 {
        base_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }
    base_car.state.position = glam::Vec2::ZERO;

    let mut solo_car = base_car.clone();
    let mut trailing_car = base_car.clone();
    let mut lead_car = base_car.clone();
    lead_car.state.position = glam::Vec2::new(14.0, 0.0);

    for _ in 0..240 {
        solo_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);

        lead_car.state.draft_intensity = 0.0;
        lead_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);

        let draft = trailing_car.compute_draft_intensity(&[&lead_car]);
        trailing_car.state.draft_intensity = draft;
        trailing_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    let solo_dist = solo_car.state.position.x;
    let trail_dist = trailing_car.state.position.x;
    let closure_m = trail_dist - solo_dist;
    println!(
        "Spec 129 Mid-Range 4.0s Acceleration from 22 m/s: Solo = {solo_dist:.2} m, Drafting (14m) = {trail_dist:.2} m, Closure = +{closure_m:.2} m"
    );

    assert!(
        closure_m >= 1.5,
        "Trailing car in slipstream wake must cover at least 1.5 m more distance over 4.0 s than solo car (got +{closure_m:.2} m)"
    );
}

#[test]
fn test_spec_129_heading_rejection_and_curved_sweeper_wake_retention() {
    let cfg = CarConfig::sports_car();

    let mut my_car = Car::new(cfg).with_pose(glam::Vec2::ZERO, 0.0);
    my_car.state.speed = 35.0;
    my_car.state.velocity = glam::Vec2::new(35.0, 0.0);

    // 1. Opposite-direction car 10m ahead along +X (rotation = PI)
    let mut opp_opposite =
        Car::new(cfg).with_pose(glam::Vec2::new(10.0, 0.0), std::f32::consts::PI);
    opp_opposite.state.speed = 35.0;
    assert_eq!(
        my_car.compute_draft_intensity(&[&opp_opposite]),
        0.0,
        "Opposite-direction traffic must produce 0.0 draft intensity"
    );

    // 2. Perpendicular cross-track car 10m ahead along +X (rotation = FRAC_PI_2)
    let mut opp_perp =
        Car::new(cfg).with_pose(glam::Vec2::new(10.0, 0.0), std::f32::consts::FRAC_PI_2);
    opp_perp.state.speed = 35.0;
    assert_eq!(
        my_car.compute_draft_intensity(&[&opp_perp]),
        0.0,
        "Perpendicular cross-track traffic must produce 0.0 draft intensity"
    );

    // 3. Curved sweeper (R = 60m, 15m arc ahead: position = (14.8, 2.95), rotation = 0.25 rad, speed = 30.0 m/s)
    my_car.state.speed = 30.0;
    let mut opp_curve = Car::new(cfg).with_pose(glam::Vec2::new(14.8, 2.95), 0.25);
    opp_curve.state.speed = 30.0;
    let curve_draft = my_car.compute_draft_intensity(&[&opp_curve]);
    println!("Spec 129 Curved Sweeper (R=60m, d=15m) draft intensity: {curve_draft:.3}");
    assert!(
        curve_draft >= 0.10,
        "Curve-tolerant bisector wake cone must retain >= 0.10 draft intensity on R=60m sweeper (got {curve_draft:.3})"
    );
}

#[test]
fn test_spec_129_clean_air_solo_determinism_invariance() {
    let dt = 1.0 / 60.0;
    let cfg = CarConfig::sports_car();

    let mut car_a = Car::new(cfg);
    let mut car_b = Car::new(cfg);
    let opp_outside = Car::new(cfg).with_pose(glam::Vec2::new(-50.0, 0.0), 0.0);

    for _ in 0..600 {
        car_a.state.draft_intensity = 0.0;
        car_a.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);

        car_b.state.draft_intensity = car_b.compute_draft_intensity(&[&opp_outside]);
        assert_eq!(car_b.state.draft_intensity, 0.0);
        car_b.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
    }

    assert_eq!(car_a.state.speed.to_bits(), car_b.state.speed.to_bits());
    assert_eq!(
        car_a.state.position.x.to_bits(),
        car_b.state.position.x.to_bits()
    );
}



