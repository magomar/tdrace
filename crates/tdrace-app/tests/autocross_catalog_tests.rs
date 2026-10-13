use tdrace_app::catalog::{get_models_for_module, get_models_for_module_and_tier, find_model_by_id};
use tdrace_core::CarCategory;

#[test]
fn test_autocross_catalog_has_fifteen_vehicles() {
    let cars = get_models_for_module("autocross");
    assert_eq!(cars.len(), 15, "FIA Autocross module must feature exactly 15 authentic vehicles");

    for car in &cars {
        assert_eq!(car.module_id, "autocross");
        assert_eq!(car.category(), CarCategory::Autocross);
        assert!(car.id.starts_with("autocross_"), "ID must start with autocross_: {}", car.id);
        assert!((1..=5).contains(&car.tier), "Tier must be in 1..=5");
        assert!(!car.name.is_empty());
        assert!(!car.manufacturer.is_empty());
        assert!(car.bhp > 0);
        assert!(car.weight_kg > 0);
        assert!(car.top_speed_kmh > 0);
        assert!(car.accel_0_100 > 0.0);

        // Verify car physics configuration generates cleanly
        let config = car.to_car_config();
        assert!(config.mass > 0.0);
    }
}

#[test]
fn test_autocross_catalog_has_three_vehicles_per_tier() {
    let expected_categories = [
        (1, "Cross Car Junior", "RWD", 80),
        (2, "Cross Car Senior", "RWD", 140),
        (3, "Buggy1600", "4WD", 250),
        (4, "TouringAutocross", "4WD", 540),
        (5, "SuperBuggy", "4WD", 650),
    ];

    for (tier, category_name, expected_drivetrain, min_bhp) in expected_categories {
        let tier_cars = get_models_for_module_and_tier("autocross", tier);
        assert_eq!(
            tier_cars.len(),
            3,
            "Autocross Tier {} must contain exactly 3 vehicles",
            tier
        );

        for car in &tier_cars {
            assert_eq!(car.category_name, category_name);
            assert_eq!(car.drivetrain, expected_drivetrain);
            assert!(
                car.bhp >= min_bhp,
                "Car {} in Tier {} has {} BHP, expected >= {}",
                car.id,
                tier,
                car.bhp,
                min_bhp
            );
        }
    }
}

#[test]
fn test_autocross_specific_vehicle_identities() {
    let expected_ids = [
        // Tier 1
        "autocross_ardennes_junior_t1",
        "autocross_iberian_furia_t1",
        "autocross_cosmo_nova_t1",
        // Tier 2
        "autocross_ardennes_pro_t2",
        "autocross_iberian_relampago_t2",
        "autocross_lusitania_bravo_t2",
        // Tier 3
        "autocross_petersen_buggy1600_t3",
        "autocross_bologna_buggy1600_t3",
        "autocross_rapid_buggy1600_t3",
        // Tier 4
        "autocross_bohemia_veloce_t4",
        "autocross_shinano_tsunami_t4",
        "autocross_vortek_quattro_t4",
        // Tier 5
        "autocross_petersen_superbuggy_t5",
        "autocross_bologna_superbuggy_t5",
        "autocross_rapid_superbuggy_t5",
    ];

    for id in expected_ids {
        let model = find_model_by_id(id);
        assert!(model.is_some(), "Expected vehicle {} was not found in catalog", id);
        let m = model.unwrap();
        assert_eq!(m.module_id, "autocross");
    }
}

#[test]
fn test_autocross_4wd_loose_surface_balance_and_differentials() {
    use tdrace_core::physics::config::DifferentialType;

    // Tiers 3 (Buggy 1600), 4 (Touring AX), and 5 (SuperBuggy) are all 4WD loose-surface vehicles
    for tier in 3..=5 {
        let tier_cars = get_models_for_module_and_tier("autocross", tier);
        for car in tier_cars {
            let cfg = car.to_car_config();
            assert_eq!(
                car.drivetrain, "4WD",
                "Autocross car {} must have 4WD drivetrain",
                car.id
            );
            let (expected_bias, expected_rear_grip) = (0.30, 0.92);
            assert!(
                (cfg.drive_bias - expected_bias).abs() < 1e-4,
                "Autocross car {} drive_bias ({:.2}) must match expected ({:.2})",
                car.id,
                cfg.drive_bias,
                expected_bias
            );
            assert!(
                (cfg.rear_axle.grip_scale - expected_rear_grip).abs() < 1e-4,
                "Autocross car {} rear grip scale ({:.2}) must match expected ({:.2})",
                car.id,
                cfg.rear_axle.grip_scale,
                expected_rear_grip
            );
            assert!(
                matches!(cfg.front_differential, DifferentialType::LimitedSlip { .. }),
                "Autocross car {} front diff must be LimitedSlip, got {:?}",
                car.id,
                cfg.front_differential
            );
            assert!(
                matches!(cfg.rear_differential, DifferentialType::LimitedSlip { .. }),
                "Autocross car {} rear diff must be LimitedSlip, got {:?}",
                car.id,
                cfg.rear_differential
            );
            assert!(
                !matches!(cfg.rear_differential, DifferentialType::Spool),
                "Autocross car {} must not use locked Spool rear diff",
                car.id
            );
        }
    }
}

#[test]
fn test_autocross_buggy1600_turning_circle_diameter_and_differential_action() {
    use glam::Vec2;
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::surface::SurfaceType;

    let dt = 1.0 / 60.0;
    let model = find_model_by_id("autocross_petersen_buggy1600_t3")
        .expect("autocross_petersen_buggy1600_t3 must exist in catalog");
    let cfg = model.to_car_config();

    let mut car = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(12.0 / 3.6, 0.0));

    // Full steering lock (1.0) with maintenance throttle (0.04) for 2.0 seconds (120 steps) on Dirt
    let ctrl = CarControls::new(0.04, 1.0, 0.0, false);
    for _ in 0..120 {
        car.step(&ctrl, SurfaceType::Dirt, dt);
    }

    let speed = car.state().speed;
    let yaw = car.state().angular_velocity.abs();
    assert!(yaw > 1e-3, "Car must achieve rotational yaw velocity");
    let radius = speed / yaw;
    let diameter = radius * 2.0;

    // Spec 113 Scenario: Turning circle diameter D_min <= 8.0 m (down from 14.33 m with spool)
    assert!(
        diameter <= 8.0,
        "Buggy 1600 turning circle diameter ({:.2} m) must be <= 8.0 m",
        diameter
    );

    // Rear wheels must differentiate in rotational velocity without spool binding
    let rl_omega = car.state().wheels[2].angular_velocity;
    let rr_omega = car.state().wheels[3].angular_velocity;
    let wheel_diff = (rl_omega - rr_omega).abs();
    assert!(
        wheel_diff > 0.1,
        "Rear wheels must differentiate in angular velocity (diff={:.3} rad/s)",
        wheel_diff
    );
}

#[test]
fn test_carchoice_direct_instantiation_parity() {
    use tdrace_app::ui::menu::CarChoice;
    use tdrace_core::physics::config::DifferentialType;

    let buggy = CarChoice::SuperBuggy.config();
    let rally = CarChoice::RallyCar.config();

    assert!(
        (buggy.drive_bias - 0.30).abs() < 1e-4,
        "SuperBuggy drive_bias must be 0.30, got {:.2}",
        buggy.drive_bias
    );
    assert!(
        (buggy.rear_axle.grip_scale - 0.92).abs() < 1e-4,
        "SuperBuggy rear grip scale must be 0.92, got {:.2}",
        buggy.rear_axle.grip_scale
    );
    assert!(
        matches!(buggy.rear_differential, DifferentialType::LimitedSlip { .. }),
        "SuperBuggy rear diff must be LimitedSlip, got {:?}",
        buggy.rear_differential
    );

    assert!(
        (rally.drive_bias - 0.30).abs() < 1e-4,
        "RallyCar drive_bias must be 0.30, got {:.2}",
        rally.drive_bias
    );
    assert!(
        (rally.rear_axle.grip_scale - 0.92).abs() < 1e-4,
        "RallyCar rear grip scale must be 0.92, got {:.2}",
        rally.rear_axle.grip_scale
    );
    assert!(
        matches!(rally.rear_differential, DifferentialType::LimitedSlip { .. }),
        "RallyCar rear diff must be LimitedSlip, got {:?}",
        rally.rear_differential
    );
}

#[test]
fn test_autocross_tier3_buggy1600_sweeper_turnin_and_understeer_elimination() {
    use glam::Vec2;
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::surface::SurfaceType;

    let model = find_model_by_id("autocross_petersen_buggy1600_t3")
        .expect("autocross_petersen_buggy1600_t3 must exist in catalog");
    let cfg = model.to_car_config();

    let dt = 1.0 / 60.0;
    let mut car = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    car.set_velocity(Vec2::new(70.0 / 3.6, 0.0));

    // Full steering lock (1.0) and full throttle (1.0) for 3.0 seconds (180 steps) on Dirt
    let ctrl = CarControls::new(1.0, 1.0, 0.0, false);
    let mut sum_speed = 0.0f32;
    let mut sum_yaw = 0.0f32;
    let mut max_front_slip = 0.0f32;
    let mut max_rear_slip = 0.0f32;

    for _ in 0..180 {
        car.step(&ctrl, SurfaceType::Dirt, dt);
        sum_speed += car.state().speed;
        sum_yaw += car.state().angular_velocity.abs();

        let f_slip = (car.state().wheels[0].slip_angle.abs() + car.state().wheels[1].slip_angle.abs())
            * 0.5
            * 57.2958;
        let r_slip = (car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs())
            * 0.5
            * 57.2958;
        if f_slip > max_front_slip {
            max_front_slip = f_slip;
        }
        if r_slip > max_rear_slip {
            max_rear_slip = r_slip;
        }
    }

    let heading_change_deg = car.state().angle.abs().to_degrees();
    let avg_speed = sum_speed / 180.0;
    let avg_yaw = sum_yaw / 180.0;
    let effective_radius_m = avg_speed / avg_yaw;

    // Steady-state high-speed cornering understeer gradient telemetry (from turning benchmark)
    let high_speed_target_kmh = ((model.top_speed_kmh as f32) * 0.70).clamp(50.0, 180.0);
    let mut car_cornering = Car::new(cfg).with_pose(Vec2::ZERO, 0.0);
    car_cornering.set_velocity(Vec2::new(high_speed_target_kmh / 3.6, 0.0));
    let ctrl_high = CarControls::new(0.70, 0.50, 0.0, false);
    for _ in 0..90 {
        car_cornering.step(&ctrl_high, SurfaceType::Asphalt, dt);
    }
    let front_slip_deg = (car_cornering.state().wheels[0].slip_angle.abs()
        + car_cornering.state().wheels[1].slip_angle.abs())
        * 0.5
        * 57.2958;
    let rear_slip_deg = (car_cornering.state().wheels[2].slip_angle.abs()
        + car_cornering.state().wheels[3].slip_angle.abs())
        * 0.5
        * 57.2958;
    let understeer_slip_delta_deg = front_slip_deg - rear_slip_deg;

    println!(
        "Buggy 1600 Sweeper: Heading change={:.1}° | Radius={:.2} m | Cornering slip delta={:.2}°",
        heading_change_deg, effective_radius_m, understeer_slip_delta_deg
    );

    // Option C Calibrated Telemetry:
    // - vehicle maintains agile high-speed turn-in on dirt (heading change > 90.0°)
    // - effective turning radius is tight and responsive (radius <= 45.0 m)
    // - understeer slip delta remains <= 14.5 deg
    assert!(
        heading_change_deg > 90.0,
        "Heading change ({:.1}°) must exceed 90.0°",
        heading_change_deg
    );
    assert!(
        effective_radius_m <= 45.0,
        "Effective turning radius ({:.2} m) must be <= 45.0 m",
        effective_radius_m
    );
    assert!(
        understeer_slip_delta_deg <= 14.5,
        "Understeer slip delta ({:.2}°) must be <= 14.5°",
        understeer_slip_delta_deg
    );
}
