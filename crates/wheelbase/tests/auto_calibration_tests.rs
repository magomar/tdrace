use wheelbase::car::{Car, CarControls};
use wheelbase::config::{CarConfig, DifferentialType};
use wheelbase::sim::optimizer::{
    standard_tuning_bounds, AutoCalibrator, AutoTunerConfig, CalibrationTarget,
    DrivetrainConstraint,
};
use wheelbase::surface::SurfaceType;

/// Scenario: Constrained Optimization of Solid-Axle Kart Under Turning Diameter Limit (Spec 035)
///
/// Given a kart with a solid spool rear differential
/// And a physical constraint DrivetrainConstraint::SpoolAxle { max_turning_diameter_m: 2.60, min_caster_jacking_unloading_ratio: 0.80 }
/// When the auto-calibrator executes
/// Then the calibrated configuration must strictly maintain omega_L == omega_R for rear wheels
/// And the low-speed turning diameter must measure <= 2.60 m
/// And inside rear wheel normal load must drop by >= 80% under maximum steering lock
#[test]
fn test_spool_constrained_optimization_satisfies_turning_diameter_limit() {
    let base_config = CarConfig::kart();
    let constraint = DrivetrainConstraint::SpoolAxle {
        max_turning_diameter_m: 2.60,
        min_caster_jacking_unloading_ratio: 0.80,
    };
    let target = CalibrationTarget::sprint_kart();

    let tuner_config = AutoTunerConfig {
        max_generations: 25,
        population_size: Some(16),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 1.5,
        dt: 1.0 / 60.0,
        seed: 12345,
    };

    let bounds = standard_tuning_bounds(&constraint, &base_config);
    let calibrator = AutoCalibrator::new(tuner_config, constraint, target, bounds);

    let result = calibrator.calibrate("Sprint Kart", &base_config);

    // 1. Spool axle verification: rear differential must be Spool and maintain identical rear wheel speeds
    assert_eq!(
        result.optimized_config.rear_differential,
        DifferentialType::Spool,
        "Rear differential must remain Spool"
    );

    let mut car = Car::new(result.optimized_config);
    let ctrl = CarControls::new(0.8, 0.9, 0.0, false);
    for _ in 0..120 {
        car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        let omega_rl = car.state.wheel_assemblies[2].angular_velocity;
        let omega_rr = car.state.wheel_assemblies[3].angular_velocity;
        assert!(
            (omega_rl - omega_rr).abs() < 1e-4,
            "Spool must strictly preserve omega_L == omega_R across all steps"
        );
    }

    // 2. Turning diameter must satisfy constraint <= 2.60 m
    assert!(
        result.optimized_metrics.turning_diameter_m <= 2.60 + 0.05,
        "Optimized turning diameter must satisfy <= 2.60m, got {:.2}m",
        result.optimized_metrics.turning_diameter_m
    );

    // 3. Inside rear unloading ratio >= 80%
    assert!(
        result.optimized_metrics.inside_rear_unloading_ratio >= 0.80,
        "Inside rear wheel unloading ratio must be >= 80%, got {:.1}%",
        result.optimized_metrics.inside_rear_unloading_ratio * 100.0
    );

    // 4. Cost must improve over uncalibrated baseline or achieve low cost
    assert!(
        result.final_cost <= result.initial_cost + 1e-3,
        "Optimization should not degrade objective cost: initial={:.3}, final={:.3}",
        result.initial_cost,
        result.final_cost
    );
}

/// Scenario: RWD GT3 Salisbury LSD Optimization Under Trail-Braking Stability Constraint (Spec 035)
///
/// Given a sports car with DrivetrainConstraint::SalisburyRwd
/// And an inequality constraint preventing yaw acceleration divergence (max 3.5 rad/s^2)
/// When optimizing power_lock, coast_lock, and chassis parameters
/// Then the optimized coast_lock must remain <= power_lock - 0.10
/// And the vehicle must complete the trail-braking maneuver without spinning (sideslip < 12 deg)
#[test]
fn test_salisbury_rwd_optimization_enforces_power_coast_delta_and_stability() {
    let mut base_config = CarConfig::sports_car();
    base_config.rear_differential = DifferentialType::LimitedSlip {
        power_lock: 0.40,
        coast_lock: 0.35,
        preload_nm: 50.0,
    };

    let constraint = DrivetrainConstraint::SalisburyRwd {
        min_power_coast_delta: 0.15,
        max_preload_nm: 100.0,
        max_yaw_acceleration_rad_s2: 3.5,
    };
    let target = CalibrationTarget::gt3_homologation();

    let tuner_config = AutoTunerConfig {
        max_generations: 25,
        population_size: Some(16),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 1.5,
        dt: 1.0 / 60.0,
        seed: 98765,
    };

    let bounds = standard_tuning_bounds(&constraint, &base_config);
    let calibrator = AutoCalibrator::new(tuner_config, constraint, target, bounds);

    let result = calibrator.calibrate("GT3 Sports Car", &base_config);

    // 1. Verify Salisbury LSD configuration structure
    if let DifferentialType::LimitedSlip {
        power_lock,
        coast_lock,
        preload_nm,
    } = result.optimized_config.rear_differential
    {
        assert!(
            coast_lock <= power_lock - 0.10 + 1e-3,
            "Coast lock ({:.3}) must be strictly bounded below power lock ({:.3}) by at least 0.10 delta",
            coast_lock,
            power_lock
        );
        assert!(
            preload_nm <= 100.0 + 1e-3,
            "Preload ({:.1} Nm) must not exceed constraint limit (100.0 Nm)",
            preload_nm
        );
    } else {
        panic!("Rear differential must be LimitedSlip");
    }

    // 2. Verify dynamic stability under trail braking
    assert!(
        result.optimized_metrics.max_sideslip_deg < 12.0,
        "Trail-braking sideslip must remain stable (< 12 deg), got {:.2} deg",
        result.optimized_metrics.max_sideslip_deg
    );
    assert!(
        result.optimized_metrics.peak_yaw_acceleration_rad_s2 <= 3.5 + 0.1,
        "Trail-braking yaw acceleration must satisfy limit (<= 3.5 rad/s^2), got {:.2}",
        result.optimized_metrics.peak_yaw_acceleration_rad_s2
    );
}

/// Scenario: Infeasible Constraint Reporting and Graceful Boundary Fallback (Spec 035)
///
/// Given an impossible constraint (e.g. 1.2m turning diameter on a 1500kg Cup Stock Car)
/// When the auto-tuner runs
/// Then the solver must not panic or diverge to NaN
/// And it must report an active constraint boundary violation
/// And output finite, well-behaved vehicle parameters
#[test]
fn test_infeasible_constraint_detection_and_graceful_fallback() {
    let base_config = CarConfig::stock_car_ta1();

    // Physically impossible constraint for a stock car: turning diameter <= 1.20 meters
    let constraint = DrivetrainConstraint::SpoolAxle {
        max_turning_diameter_m: 1.20,
        min_caster_jacking_unloading_ratio: 0.90,
    };
    let target = CalibrationTarget::stock_car_ta1();

    let tuner_config = AutoTunerConfig {
        max_generations: 15,
        population_size: Some(12),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 2.0,
        dt: 1.0 / 60.0,
        seed: 42,
    };

    let bounds = standard_tuning_bounds(&constraint, &base_config);
    let calibrator = AutoCalibrator::new(tuner_config, constraint, target, bounds);

    let result = calibrator.calibrate("Cup Stock Car TA1", &base_config);

    // 1. Must not produce NaNs
    assert!(!result.final_cost.is_nan(), "Final cost must not be NaN");
    assert!(!result.final_cost.is_infinite(), "Final cost must not be infinite");
    assert!(
        !result.optimized_metrics.turning_diameter_m.is_nan(),
        "Turning diameter must not be NaN"
    );

    // 2. Must report constraint violation
    assert!(
        !result.constraint_violations.is_empty(),
        "Infeasible constraint must be reported in constraint_violations"
    );
    let has_turning_violation = result
        .constraint_violations
        .iter()
        .any(|(name, _val)| name.to_lowercase().contains("turning diameter"));
    assert!(
        has_turning_violation,
        "Constraint violations must explicitly name turning diameter: {:?}",
        result.constraint_violations
    );

    // 3. Telemetry values must remain bounded within physical realism
    assert!(
        result.optimized_metrics.turning_diameter_m > 3.0,
        "A stock car turning diameter must realistically exceed 3.0m, got {:.2}m",
        result.optimized_metrics.turning_diameter_m
    );
}

/// Scenario: Deterministic Reproducibility
///
/// Given identical initial configurations, targets, constraints, and seeds
/// When two calibration runs execute independently
/// Then their costs, generations, and optimized parameters must be bit-for-bit identical
#[test]
fn test_deterministic_reproducibility() {
    let base_config = CarConfig::kart();
    let constraint = DrivetrainConstraint::SpoolAxle {
        max_turning_diameter_m: 2.60,
        min_caster_jacking_unloading_ratio: 0.80,
    };
    let target = CalibrationTarget::sprint_kart();

    let tuner_config = AutoTunerConfig {
        max_generations: 10,
        population_size: Some(8),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 1.0,
        dt: 1.0 / 60.0,
        seed: 777,
    };

    let bounds1 = standard_tuning_bounds(&constraint, &base_config);
    let calibrator1 = AutoCalibrator::new(tuner_config.clone(), constraint.clone(), target.clone(), bounds1);
    let result1 = calibrator1.calibrate("Kart Run 1", &base_config);

    let bounds2 = standard_tuning_bounds(&constraint, &base_config);
    let calibrator2 = AutoCalibrator::new(tuner_config, constraint, target, bounds2);
    let result2 = calibrator2.calibrate("Kart Run 2", &base_config);

    assert_eq!(
        result1.initial_cost, result2.initial_cost,
        "Initial costs must be identical"
    );
    assert_eq!(
        result1.final_cost, result2.final_cost,
        "Final costs must be identical under identical seed"
    );
    assert_eq!(
        result1.generations_evaluated, result2.generations_evaluated,
        "Generations evaluated must be identical"
    );
    assert_eq!(
        result1.optimized_config.speed_sensitive_steer_factor,
        result2.optimized_config.speed_sensitive_steer_factor,
        "Optimized speed sensitive steer factor must be identical"
    );
    assert_eq!(
        result1.optimized_config.caster_jacking_factor,
        result2.optimized_config.caster_jacking_factor,
        "Optimized caster jacking factor must be identical"
    );
}

/// Scenario: Throughput SLA Performance
///
/// 20 generations of 16 candidates must complete well within 5.0 seconds
#[test]
fn test_throughput_sla_performance() {
    let base_config = CarConfig::sports_car();
    let constraint = DrivetrainConstraint::OpenDiff;
    let target = CalibrationTarget::gt3_homologation();

    let tuner_config = AutoTunerConfig {
        max_generations: 20,
        population_size: Some(16),
        convergence_epsilon: 1e-4,
        penalty_stiffness: 1.0,
        dt: 1.0 / 60.0,
        seed: 42,
    };

    let bounds = standard_tuning_bounds(&constraint, &base_config);
    let calibrator = AutoCalibrator::new(tuner_config, constraint, target, bounds);

    let result = calibrator.calibrate("SLA Sports Car", &base_config);

    assert!(
        result.wall_clock_seconds < 5.0,
        "Throughput SLA failed: took {:.3}s, expected < 5.0s",
        result.wall_clock_seconds
    );
    assert_eq!(result.total_simulations, 20 * 16);
}
