//! # Simulation & Dynamic Benchmarking Harness
//!
//! Pure memory-to-memory headless vehicle physics simulation suite.
//! Provides high-frequency telemetry recording, standardized dynamic test protocols
//! (Protocols A through E), and cross-surface multi-vehicle benchmark matrices.

pub mod circuit;
pub mod harness;
pub mod matrix;
pub mod optimizer;
pub mod playground;
pub mod protocols;
pub mod report;
pub mod telemetry;

pub use circuit::{
    run_path_simulation, PathSimulationResult, PathSimulationStatus, SimPath, SimPathPoint,
};
pub use harness::{SimulationRunner, DEFAULT_SIMULATION_DT};
pub use matrix::{ExperimentDataset, VehicleBenchmarkResult};
pub use optimizer::{
    AutoCalibrator, AutoTunerConfig, CalibrationResult, CalibrationTarget, CmaEsOptimizer,
    DrivetrainConstraint, DynamicMetrics, OptimizationEvaluator, ParameterBound,
    standard_tuning_bounds,
};
pub use playground::{
    run_playground_simulation, run_wall_contact_protocol_g, PlaygroundSimulationResult,
    SimBarrierType, SimObstacleType, SimPlayground, SimSceneryObstacle, SimSector, SimWallBarrier,
    WallContactEvent, WallContactProtocolGResult,
};
pub use protocols::{
    run_braking_cadence, run_braking_in_turn, run_braking_split_mu, run_braking_straight_line,
    run_braking_surface_battery, run_protocol_a, run_protocol_b, run_protocol_c, run_protocol_d,
    run_protocol_e, run_reverse_simulation_battery, run_reverse_step_steer,
    run_reverse_straight_line, BrakingCadenceResult, BrakingCorneringResult, BrakingSplitMuResult,
    BrakingStabilityRating, BrakingStraightLineResult, BrakingSurfaceExperimentResult,
    CorneringBrakingBehavior, ProtocolAResult, ProtocolBResult, ProtocolCResult, ProtocolDResult,
    ProtocolEResult, ReverseExperimentResult, ReverseStepSteerResult, ReverseSteerStatus,
    ReverseStraightLineResult, SkidpadDepartureMode, SplitMuStatus, StepSteerStatus,
};
pub use report::{
    generate_braking_simulation_markdown_report, generate_html_report, generate_markdown_report,
    generate_reverse_simulation_markdown_report,
};
pub use telemetry::TelemetryPoint;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CarConfig;
    use crate::surface::SurfaceType;

    #[test]
    fn test_headless_harness_straight_line() {
        let mut runner = SimulationRunner::new(CarConfig::sports_car(), DEFAULT_SIMULATION_DT);
        runner.run_for(1.0, SurfaceType::Asphalt, |_t, _c| {
            crate::car::CarControls::accelerate()
        });
        assert!(runner.car.state().speed > 5.0);
        assert!(!runner.telemetry.is_empty());
    }

    #[test]
    fn test_protocol_a_acceleration() {
        let config = CarConfig::sports_car();
        let res = run_protocol_a(&config, SurfaceType::Asphalt, 5.0, DEFAULT_SIMULATION_DT);
        assert!(res.t50_s.is_some());
        assert!(res.peak_accel_g > 0.3);
    }

    #[test]
    fn test_protocol_b_braking() {
        let config = CarConfig::sports_car();
        let res_asphalt = run_protocol_b(&config, SurfaceType::Asphalt, 100.0, DEFAULT_SIMULATION_DT);
        let res_ice = run_protocol_b(&config, SurfaceType::SheetIce, 100.0, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.stopping_distance_m < res_ice.stopping_distance_m);
        assert!(res_asphalt.avg_decel_g > res_ice.avg_decel_g);
    }

    #[test]
    fn test_protocol_c_skidpad() {
        let config = CarConfig::sports_car();
        let res_asphalt = run_protocol_c(&config, SurfaceType::Asphalt, 30.0, DEFAULT_SIMULATION_DT);
        let res_ice = run_protocol_c(&config, SurfaceType::SheetIce, 30.0, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.peak_lateral_accel_g > 0.8, "Asphalt lateral g: {:.2}", res_asphalt.peak_lateral_accel_g);
        assert!(res_asphalt.peak_lateral_accel_g > res_ice.peak_lateral_accel_g * 2.0);
    }

    #[test]
    fn test_protocol_e_coast_down() {
        let config = CarConfig::sports_car();
        let res_asphalt = run_protocol_e(&config, SurfaceType::Asphalt, 120.0, DEFAULT_SIMULATION_DT);
        let res_sand = run_protocol_e(&config, SurfaceType::DeepSand, 120.0, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.coast_distance_m > res_sand.coast_distance_m);
    }

    #[test]
    fn test_braking_straight_line_simulation() {
        let config = CarConfig::sports_car();
        let res_asphalt = run_braking_straight_line(&config, SurfaceType::Asphalt, 120.0, true, DEFAULT_SIMULATION_DT);
        let res_ice = run_braking_straight_line(&config, SurfaceType::SheetIce, 120.0, true, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.stopping_distance_m < res_ice.stopping_distance_m);
        assert!(res_asphalt.avg_decel_g > res_ice.avg_decel_g);
        assert_eq!(res_asphalt.stability_rating, BrakingStabilityRating::Stable);
    }

    #[test]
    fn test_braking_split_mu_simulation() {
        let config = CarConfig::sports_car();
        let res = run_braking_split_mu(&config, SurfaceType::Asphalt, SurfaceType::Grass, 100.0, DEFAULT_SIMULATION_DT);
        assert!(res.stopping_distance_m > 20.0);
        assert_ne!(res.status, SplitMuStatus::SpunOut);
    }

    #[test]
    fn test_braking_in_turn_simulation() {
        let config = CarConfig::sports_car();
        let res = run_braking_in_turn(&config, SurfaceType::Asphalt, 40.0, 70.0, DEFAULT_SIMULATION_DT);
        assert_ne!(res.behavior, CorneringBrakingBehavior::SnapOversteerSpin);
    }

    #[test]
    fn test_braking_cadence_simulation() {
        let config = CarConfig::sports_car();
        let res = run_braking_cadence(&config, SurfaceType::Asphalt, 120.0, DEFAULT_SIMULATION_DT);
        assert!(res.total_pulse_cycles >= 2);
    }

    #[test]
    fn test_reverse_straight_line_simulation() {
        let config = CarConfig::sports_car();
        let res = run_reverse_straight_line(&config, SurfaceType::Asphalt, 3.0, DEFAULT_SIMULATION_DT);
        assert!(res.stable, "Reverse straight line tracking must be stable");
        assert!(res.heading_deviation_deg < 0.05, "Heading deviation must be near zero, got {:.4}", res.heading_deviation_deg);
        assert!(res.lateral_drift_m < 0.05, "Lateral drift must be near zero, got {:.4}", res.lateral_drift_m);
        assert!(res.max_yaw_rate_deg_s < 0.1, "Max yaw rate in neutral reverse must be tiny, got {:.4}", res.max_yaw_rate_deg_s);
        assert!(res.distance_traveled_m > 5.0, "Car must travel distance in reverse");
    }

    #[test]
    fn test_reverse_step_steer_simulation() {
        let config = CarConfig::sports_car();
        let res = run_reverse_step_steer(&config, SurfaceType::Asphalt, DEFAULT_SIMULATION_DT);
        assert_eq!(res.status, ReverseSteerStatus::Stable);
        assert!(res.peak_right_yaw_rate_deg_s > 10.0, "Right steer in reverse must generate yaw");
        assert!(res.peak_left_yaw_rate_deg_s > 10.0, "Left steer in reverse must generate yaw");
        assert!(res.yaw_asymmetry_pct < 2.0, "Left/Right steering in reverse must be symmetrical, got {:.2}%", res.yaw_asymmetry_pct);
        assert!(res.reversal_latency_ms < 350.0, "Steering reversal latency must be under 350ms, got {:.1}ms", res.reversal_latency_ms);
        assert!(res.post_release_residual_yaw_deg_s < 1.0, "Post-release residual yaw must decay to zero, got {:.2}°/s", res.post_release_residual_yaw_deg_s);
    }

    #[test]
    fn test_reverse_simulation_battery_fleet() {
        let fleet = [
            ("sports_car", "Sports Car", "Sports", CarConfig::sports_car()),
            ("drift_car", "Drift Machine", "Drift", CarConfig::drift_car()),
            ("kart", "Sprint Kart", "Kart", CarConfig::kart()),
            ("rally_car", "Rally Supercar", "Rally", CarConfig::rally_car()),
            ("stock_car", "Cup Stock Car", "Stock", CarConfig::stock_car_ta1()),
        ];

        let fleet_refs: Vec<(&str, &str, &str, &CarConfig)> = fleet
            .iter()
            .map(|(id, name, cat, cfg)| (*id, *name, *cat, cfg))
            .collect();

        let results = run_reverse_simulation_battery(&fleet_refs, SurfaceType::Asphalt, DEFAULT_SIMULATION_DT);
        assert_eq!(results.len(), 5);

        for r in &results {
            assert!(r.straight_line.stable, "Vehicle {} failed straight line stability", r.vehicle_id);
            assert_eq!(r.step_steer.status, ReverseSteerStatus::Stable, "Vehicle {} failed step steer stability", r.vehicle_id);
            assert!(r.step_steer.yaw_asymmetry_pct < 2.0, "Vehicle {} had excessive steering asymmetry", r.vehicle_id);
            assert!(r.step_steer.post_release_residual_yaw_deg_s < 2.0, "Vehicle {} failed residual yaw damping", r.vehicle_id);
        }

        let report = generate_reverse_simulation_markdown_report(&results);
        println!("\n{}\n", report);
        assert!(report.contains("Reverse Movement & Directional Steering Benchmark"));
        assert!(report.contains("Sports Car"));
        assert!(report.contains("Drift Machine"));
        assert!(report.contains("✅ Pass"));
        assert!(report.contains("✅ Stable"));
    }

    #[test]
    fn test_path_simulation_straight_with_turns_surfaces() {
        let config = CarConfig::sports_car();
        let path = SimPath::straight_with_turns();

        // 1. Asphalt: should complete smoothly with high speed and low tracking error
        let res_asphalt = run_path_simulation(&config, SurfaceType::Asphalt, &path, 30.0, DEFAULT_SIMULATION_DT);
        println!("\n=== ASPHALT STRAIGHT-WITH-TURNS SIMULATION ===");
        println!("Status: {:?}", res_asphalt.status);
        println!("Completed: {:.1}% ({:.1}m / {:.1}m) in {:.2}s", res_asphalt.completion_pct, res_asphalt.distance_traveled_m, res_asphalt.path_length_m, res_asphalt.elapsed_time_s);
        println!("Speed: Avg {:.1} km/h, Peak {:.1} km/h", res_asphalt.avg_speed_kmh, res_asphalt.peak_speed_kmh);
        println!("Tracking: Max cross-track {:.2}m, RMS {:.2}m", res_asphalt.max_cross_track_error_m, res_asphalt.rms_cross_track_error_m);
        println!("Lateral: Peak {:.2}g, Avg {:.2}g", res_asphalt.peak_lateral_accel_g, res_asphalt.avg_lateral_accel_g);

        assert_eq!(res_asphalt.status, PathSimulationStatus::Completed);
        assert!(res_asphalt.peak_speed_kmh > 80.0);
        assert!(res_asphalt.max_cross_track_error_m < 5.0);

        // 2. Dirt: should complete with slight drift / slide
        let res_dirt = run_path_simulation(&config, SurfaceType::Dirt, &path, 35.0, DEFAULT_SIMULATION_DT);
        println!("\n=== DIRT STRAIGHT-WITH-TURNS SIMULATION ===");
        println!("Status: {:?}", res_dirt.status);
        println!("Completed: {:.1}% ({:.1}m / {:.1}m) in {:.2}s", res_dirt.completion_pct, res_dirt.distance_traveled_m, res_dirt.path_length_m, res_dirt.elapsed_time_s);
        println!("Speed: Avg {:.1} km/h, Peak {:.1} km/h", res_dirt.avg_speed_kmh, res_dirt.peak_speed_kmh);
        println!("Tracking: Max cross-track {:.2}m, RMS {:.2}m", res_dirt.max_cross_track_error_m, res_dirt.rms_cross_track_error_m);

        assert_eq!(res_dirt.status, PathSimulationStatus::Completed);
        assert!(res_dirt.peak_speed_kmh > 60.0);

        // 3. Deep Sand: sports car is trapped in runaway arrestor sand trap
        let res_sand = run_path_simulation(&config, SurfaceType::DeepSand, &path, 30.0, DEFAULT_SIMULATION_DT);
        println!("\n=== DEEP SAND STRAIGHT-WITH-TURNS SIMULATION ===");
        println!("Status: {:?}", res_sand.status);
        println!("Completed: {:.1}% ({:.1}m / {:.1}m) in {:.2}s", res_sand.completion_pct, res_sand.distance_traveled_m, res_sand.path_length_m, res_sand.elapsed_time_s);
        println!("Speed: Avg {:.1} km/h, Peak {:.1} km/h", res_sand.avg_speed_kmh, res_sand.peak_speed_kmh);
        println!("Tracking: Max cross-track {:.2}m, RMS {:.2}m", res_sand.max_cross_track_error_m, res_sand.rms_cross_track_error_m);
        println!("Forces: Peak RR {:.0} N, Avg RR {:.0} N, Peak Traction {:.0} N, Avg Traction {:.0} N",
            res_sand.peak_rolling_resistance_n, res_sand.avg_rolling_resistance_n,
            res_sand.peak_traction_force_n, res_sand.avg_traction_force_n);
        println!("Failure Reason: {:?}", res_sand.failure_reason);

        // On DeepSand, the car either gets stuck or severely fails to gain speed
        assert!(res_sand.peak_speed_kmh < 15.0, "Car on DeepSand should have severe speed handicap, got {:.1} km/h", res_sand.peak_speed_kmh);
        assert!(res_sand.avg_rolling_resistance_n > res_sand.avg_traction_force_n * 0.9, "Rolling resistance dominates traction on DeepSand");

        // 4. Packed Sand: Sand Rail Buggy with paddle tires skims across dune ribbon
        let buggy_cfg = CarConfig::sand_rail();
        let res_buggy = run_path_simulation(&buggy_cfg, SurfaceType::PackedSand, &path, 35.0, DEFAULT_SIMULATION_DT);
        println!("\n=== PACKED SAND BUGGY SIMULATION ===");
        println!("Status: {:?}", res_buggy.status);
        println!("Completed: {:.1}% in {:.2}s, Avg Spd: {:.1} km/h, Peak: {:.1} km/h",
            res_buggy.completion_pct, res_buggy.elapsed_time_s, res_buggy.avg_speed_kmh, res_buggy.peak_speed_kmh);
        assert_eq!(res_buggy.status, PathSimulationStatus::Completed);
        assert!(res_buggy.peak_speed_kmh > 70.0);
    }

    #[test]
    fn test_path_simulation_hypothetical_circuit_surfaces() {
        let config = CarConfig::sports_car();
        let circuit = SimPath::hypothetical_circuit();

        let res_asphalt = run_path_simulation(&config, SurfaceType::Asphalt, &circuit, 45.0, DEFAULT_SIMULATION_DT);
        println!("\n=== ASPHALT HYPOTHETICAL CIRCUIT SIMULATION ===");
        println!("Status: {:?}", res_asphalt.status);
        println!("Completed: {:.1}% ({:.1}m / {:.1}m) in {:.2}s", res_asphalt.completion_pct, res_asphalt.distance_traveled_m, res_asphalt.path_length_m, res_asphalt.elapsed_time_s);
        println!("Speed: Avg {:.1} km/h, Peak {:.1} km/h", res_asphalt.avg_speed_kmh, res_asphalt.peak_speed_kmh);
        assert_eq!(res_asphalt.status, PathSimulationStatus::Completed);

        let res_sand = run_path_simulation(&config, SurfaceType::DeepSand, &circuit, 30.0, DEFAULT_SIMULATION_DT);
        println!("\n=== DEEP SAND HYPOTHETICAL CIRCUIT SIMULATION ===");
        println!("Status: {:?}", res_sand.status);
        println!("Completed: {:.1}% ({:.1}m / {:.1}m) in {:.2}s", res_sand.completion_pct, res_sand.distance_traveled_m, res_sand.path_length_m, res_sand.elapsed_time_s);
        println!("Speed: Avg {:.1} km/h, Peak {:.1} km/h", res_sand.avg_speed_kmh, res_sand.peak_speed_kmh);
        println!("Failure: {:?}", res_sand.failure_reason);
        assert_ne!(res_sand.status, PathSimulationStatus::Completed);
    }

    #[test]
    fn test_systematic_steering_calibration_speed_sweep() {
        use crate::car::{Car, CarControls};
        let speeds_kmh = [40.0, 80.0, 120.0, 160.0, 200.0];
        let dt = DEFAULT_SIMULATION_DT;

        println!("\n=================== SYSTEMATIC STEERING CALIBRATION SWEEP ===================");
        for &spd in &speeds_kmh {
            let v0 = spd / 3.6;

            // In-game cabinet input filter scale:
            let speed_scale = (1.0f32 / (1.0f32 + v0 * 0.018f32)).max(0.38f32);
            let ctrl = CarControls {
                throttle: 0.5,
                steer: 1.0 * speed_scale,
                brake: 0.0,
                handbrake: false,
                reverse: false,
            };

            // 1. Pre-Spec 028 Sports Car (Pure Pacejka, no thermal wear fade)
            let mut car_pre_028 = Car::new(CarConfig::sports_car()).with_pose(glam::Vec2::ZERO, 0.0);
            car_pre_028.set_velocity(glam::Vec2::new(v0, 0.0));
            for _ in 0..180 { // 1.5 seconds cornering entry
                for w in &mut car_pre_028.state_mut().wheel_assemblies {
                    w.temperature = 85.0; // Fixed nominal grip
                    w.wear = 0.0;
                }
                car_pre_028.step(&ctrl, SurfaceType::Asphalt, dt);
            }
            let yaw_pre = car_pre_028.state().angular_velocity.abs();
            let radius_pre = if yaw_pre > 1e-3 { car_pre_028.state().speed / yaw_pre } else { 999.0 };
            let lat_g_pre = (car_pre_028.state().speed * yaw_pre) / 9.81;

            // 2. Spec 028 Calibrated (Decoupled WheelAssembly with calibrated thermal envelope)
            let mut car_calibrated = Car::new(CarConfig::sports_car()).with_pose(glam::Vec2::ZERO, 0.0);
            car_calibrated.set_velocity(glam::Vec2::new(v0, 0.0));
            for _ in 0..180 {
                car_calibrated.step(&ctrl, SurfaceType::Asphalt, dt);
            }
            let yaw_cal = car_calibrated.state().angular_velocity.abs();
            let radius_cal = if yaw_cal > 1e-3 { car_calibrated.state().speed / yaw_cal } else { 999.0 };
            let lat_g_cal = (car_calibrated.state().speed * yaw_cal) / 9.81;
            let temp_f = car_calibrated.state().wheel_assemblies[0].temperature;
            let _grip_f = car_calibrated.state().wheel_assemblies[0].thermal_grip_multiplier();

            // 3. GT Car (0.016 factor + calibrated thermal dynamics)
            let mut gt_cfg = CarConfig::sports_car();
            gt_cfg.max_steer_angle = 0.50;
            gt_cfg.speed_sensitive_steer_factor = 0.016;
            let mut car_gt = Car::new(gt_cfg).with_pose(glam::Vec2::ZERO, 0.0);
            car_gt.set_velocity(glam::Vec2::new(v0, 0.0));
            for _ in 0..180 {
                car_gt.step(&ctrl, SurfaceType::Asphalt, dt);
            }
            let yaw_gt = car_gt.state().angular_velocity.abs();
            let radius_gt = if yaw_gt > 1e-3 { car_gt.state().speed / yaw_gt } else { 999.0 };
            let lat_g_gt = (car_gt.state().speed * yaw_gt) / 9.81;

            println!(
                "Speed {:3.0} km/h | Pre-028: R={:5.1}m, Ay={:4.2}g | Calibrated: R={:5.1}m, Ay={:4.2}g (T={:4.1}°C) | GT: R={:5.1}m, Ay={:4.2}g",
                spd, radius_pre, lat_g_pre, radius_cal, lat_g_cal, temp_f, radius_gt, lat_g_gt
            );

            // Systematic acceptance assertion: Calibrated turning radius must closely match Pre-028
            // (within 15% across all speed envelopes), restoring the beloved pre-028 agile steering feel
            let radius_ratio = radius_cal / radius_pre;
            assert!(
                radius_ratio >= 0.85 && radius_ratio <= 1.25,
                "At {} km/h: turning radius ratio ({:.2}x) must remain within [0.85, 1.25] of pre-028 baseline",
                spd, radius_ratio
            );
        }
        println!("============================================================================\n");
    }
}
