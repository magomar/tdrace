//! Comprehensive Integration & Dynamic Validation Suite: SimPlayground & Turn/Obstacle Dynamics
//!
//! Validates Track A from Spec 010:
//! 1. Parametric turn type geometry, curvature, and closed-loop vehicle execution.
//! 2. Protocol G wall contact dynamics, energy dissipation, and anti-wall-riding validation ($I_{\text{wall\_ride}} < 0.85$).
//! 3. Scenery obstacles: Soft tree canopy foliage drag vs hard trunk inelastic rebound.
//! 4. Protocol H: Multi-surface composite gauntlet run with heterogeneous sector transitions and telemetry logging.

use glam::Vec2;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::sim::{
    run_path_simulation, run_playground_simulation, run_wall_contact_protocol_g, PathSimulationStatus,
    SimBarrierType, SimPlayground, SimSceneryObstacle, SimSector, SimPath,
    DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_parametric_turn_geometries_and_closed_loop_execution() {
    let dt = DEFAULT_SIMULATION_DT;
    let sports_car = CarConfig::sports_car();

    println!("\n==================================================================================");
    println!("🏎️  1. PARAMETRIC TURN GEOMETRIES & CLOSED-LOOP VEHICLE EXECUTION");
    println!("==================================================================================");

    // 1. Hairpin (R = 18m, 30m entry, 30m exit)
    let hairpin = SimPath::hairpin("Hairpin R18", 18.0, 30.0, 30.0);
    assert!(hairpin.points.len() >= 20);
    let res_hairpin = run_path_simulation(&sports_car, SurfaceType::Asphalt, &hairpin, 15.0, dt);
    println!(
        "Hairpin: dist={:.1}m/{:.1}m, completion={:.1}%, v_avg={:.1}km/h, v_peak={:.1}km/h, rms_err={:.2}m, status={:?}",
        res_hairpin.distance_traveled_m, res_hairpin.path_length_m, res_hairpin.completion_pct,
        res_hairpin.avg_speed_kmh, res_hairpin.peak_speed_kmh, res_hairpin.rms_cross_track_error_m, res_hairpin.status
    );
    assert_eq!(res_hairpin.status, PathSimulationStatus::Completed);
    assert!(res_hairpin.completion_pct >= 90.0);
    assert!(res_hairpin.rms_cross_track_error_m < 2.5);

    // 2. High-speed Sweeper (R = 80m, 90 deg)
    let sweeper = SimPath::sweeper("Sweeper R80", 80.0, 90.0);
    assert!(sweeper.points.len() >= 25);
    let res_sweeper = run_path_simulation(&sports_car, SurfaceType::Asphalt, &sweeper, 15.0, dt);
    println!(
        "Sweeper: dist={:.1}m/{:.1}m, completion={:.1}%, v_avg={:.1}km/h, v_peak={:.1}km/h, rms_err={:.2}m, status={:?}",
        res_sweeper.distance_traveled_m, res_sweeper.path_length_m, res_sweeper.completion_pct,
        res_sweeper.avg_speed_kmh, res_sweeper.peak_speed_kmh, res_sweeper.rms_cross_track_error_m, res_sweeper.status
    );
    assert_eq!(res_sweeper.status, PathSimulationStatus::Completed);
    assert!(res_sweeper.completion_pct >= 90.0);

    // 3. Tightening Clothoid Spiral (R = 70m -> 20m, 90 deg)
    let clothoid = SimPath::clothoid_spiral("Clothoid Spiral R70-20", 70.0, 20.0, 90.0);
    let res_clothoid = run_path_simulation(&sports_car, SurfaceType::Asphalt, &clothoid, 15.0, dt);
    println!(
        "Clothoid: dist={:.1}m/{:.1}m, completion={:.1}%, v_avg={:.1}km/h, v_peak={:.1}km/h, rms_err={:.2}m, status={:?}",
        res_clothoid.distance_traveled_m, res_clothoid.path_length_m, res_clothoid.completion_pct,
        res_clothoid.avg_speed_kmh, res_clothoid.peak_speed_kmh, res_clothoid.rms_cross_track_error_m, res_clothoid.status
    );
    assert_eq!(res_clothoid.status, PathSimulationStatus::Completed);
    assert!(res_clothoid.completion_pct >= 90.0);

    // 4. Alternating S-Chicane (Amplitude = 5m, half-wavelength = 20m)
    let chicane = SimPath::s_chicane("S-Chicane A5", 5.0, 20.0);
    let res_chicane = run_path_simulation(&sports_car, SurfaceType::Asphalt, &chicane, 15.0, dt);
    println!(
        "S-Chicane: dist={:.1}m/{:.1}m, completion={:.1}%, v_avg={:.1}km/h, v_peak={:.1}km/h, rms_err={:.2}m, status={:?}",
        res_chicane.distance_traveled_m, res_chicane.path_length_m, res_chicane.completion_pct,
        res_chicane.avg_speed_kmh, res_chicane.peak_speed_kmh, res_chicane.rms_cross_track_error_m, res_chicane.status
    );
    assert_eq!(res_chicane.status, PathSimulationStatus::Completed);
    assert!(res_chicane.completion_pct >= 90.0);
}

#[test]
fn test_protocol_g_wall_contact_matrix_and_anti_wall_riding() {
    let dt = DEFAULT_SIMULATION_DT;
    let config = CarConfig::sports_car();

    let barriers = [
        SimBarrierType::Concrete,
        SimBarrierType::SteelArmco,
        SimBarrierType::TireWall,
        SimBarrierType::StoneParapet,
    ];
    let speeds = [50.0, 80.0, 110.0];
    let angles = [10.0, 20.0, 45.0];

    println!("\n==================================================================================");
    println!("🛡️  2. PROTOCOL G: WALL CONTACT MATRIX & ANTI-WALL-RIDING BENCHMARK");
    println!("==================================================================================");

    for &barrier in &barriers {
        println!("\n--- Barrier: {:?} ---", barrier);
        for &spd in &speeds {
            for &deg in &angles {
                let res = run_wall_contact_protocol_g(&config, spd, deg, barrier, dt);
                println!(
                    "  v0={:5.1}km/h @ {:4.1}° -> v_exit={:5.1}km/h (ret={:.3}), impulse={:7.0}Ns, wall_index={:.3}, exploit={}",
                    res.approach_speed_kmh, res.impact_angle_deg, res.exit_speed_kmh,
                    res.speed_retention_ratio, res.peak_impulse_ns, res.wall_riding_index, res.exploit_detected
                );

                // Spec 010: Wall contact is punitive (I_wall_ride < 0.85 for regular impacts, no exploit >= 0.95)
                if deg >= 15.0 {
                    assert!(
                        res.wall_riding_index < 0.85,
                        "Wall contact must be punitive (I_wall_ride < 0.85): got {:.3} for {:?} at {}km/h {}°",
                        res.wall_riding_index, barrier, spd, deg
                    );
                }
                assert!(
                    !res.exploit_detected,
                    "Wall-riding exploit detected: I_wall_ride={:.3} >= 0.95 for {:?} at {}km/h {}°",
                    res.wall_riding_index, barrier, spd, deg
                );
                assert!(res.exit_speed_kmh < res.approach_speed_kmh);
                assert!(res.peak_impulse_ns > 0.0);
            }
        }
    }
}

#[test]
fn test_scenery_tree_canopy_drag_vs_solid_trunk_collision() {
    let dt = DEFAULT_SIMULATION_DT;
    let config = CarConfig::sports_car();

    println!("\n==================================================================================");
    println!("🌲 3. SCENERY OBSTACLE MECHANICS: SOFT CANOPY DRAG VS RIGID TRUNK SAT COLLISION");
    println!("==================================================================================");

    // Path along X-axis
    let path = SimPath::from_waypoints("Obstacle Straight", &[Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)], false);

    // Case A: Soft tree canopy situated directly on path at x = 30m, radius = 5m, drag = 3.5 m/s^2
    let canopy_playground = SimPlayground {
        name: "Canopy Test".to_string(),
        path: path.clone(),
        sectors: vec![SimSector {
            name: "Straight".to_string(),
            start_dist: 0.0,
            end_dist: 100.0,
            surface: SurfaceType::Asphalt,
            banking_rad: 0.0,
            left_wall: None,
            right_wall: None,
        }],
        obstacles: vec![SimSceneryObstacle::tree_canopy(1, Vec2::new(30.0, 0.0), 5.0, 3.5)],
    };

    let res_canopy = run_playground_simulation(&config, &canopy_playground, 6.0, dt);
    println!(
        "Canopy Test: distance={:.1}m, v_peak={:.1}km/h, v_avg={:.1}km/h, canopies_brushed={}, trunks_hit={}",
        res_canopy.distance_traveled_m, res_canopy.peak_speed_kmh, res_canopy.avg_speed_kmh,
        res_canopy.canopies_brushed, res_canopy.trunk_collisions
    );
    assert!(res_canopy.canopies_brushed > 0);
    assert_eq!(res_canopy.trunk_collisions, 0);
    assert!(res_canopy.distance_traveled_m > 80.0, "Car must smoothly pass through canopy foliage");

    // Case B: Hard tree trunk situated on path at x = 30m, radius = 0.5m
    let trunk_playground = SimPlayground {
        name: "Trunk Test".to_string(),
        path,
        sectors: vec![SimSector {
            name: "Straight".to_string(),
            start_dist: 0.0,
            end_dist: 100.0,
            surface: SurfaceType::Asphalt,
            banking_rad: 0.0,
            left_wall: None,
            right_wall: None,
        }],
        obstacles: vec![SimSceneryObstacle::tree_trunk(2, Vec2::new(30.0, 0.0), 0.5)],
    };

    let res_trunk = run_playground_simulation(&config, &trunk_playground, 6.0, dt);
    println!(
        "Trunk Test: distance={:.1}m, v_peak={:.1}km/h, v_avg={:.1}km/h, canopies_brushed={}, trunks_hit={}",
        res_trunk.distance_traveled_m, res_trunk.peak_speed_kmh, res_trunk.avg_speed_kmh,
        res_trunk.canopies_brushed, res_trunk.trunk_collisions
    );
    assert!(res_trunk.trunk_collisions > 0, "Car must register rigid collision with trunk");
}

#[test]
fn test_protocol_h_multi_surface_composite_gauntlet() {
    let dt = DEFAULT_SIMULATION_DT;
    let gauntlet = SimPlayground::composite_gauntlet();

    println!("\n==================================================================================");
    println!("🏁 4. PROTOCOL H: MULTI-SURFACE COMPOSITE GAUNTLET RUN");
    println!("==================================================================================");

    let vehicle_configs = [
        ("Sports Car", CarConfig::sports_car()),
        ("Rally Car", CarConfig::rally_car()),
        ("Drift Car", CarConfig::drift_car()),
    ];

    for (name, config) in &vehicle_configs {
        let res = run_playground_simulation(config, &gauntlet, 40.0, dt);
        println!(
            "\n--- Vehicle: {} ---", name
        );
        println!(
            "Total Length: {:.1}m | Dist Navigated: {:.1}m ({:.1}%) | Status: {:?}",
            res.total_path_length_m, res.distance_traveled_m, res.completion_pct, res.status
        );
        println!(
            "Elapsed Time: {:.2}s | Avg Speed: {:.1}km/h | Peak Speed: {:.1}km/h",
            res.elapsed_time_s, res.avg_speed_kmh, res.peak_speed_kmh
        );
        println!(
            "Cross-Track Error: max={:.2}m, rms={:.2}m | Wall Contacts: {} (tot={:.3}s)",
            res.max_cross_track_error_m, res.rms_cross_track_error_m,
            res.wall_contacts.len(), res.total_wall_contact_time_s
        );
        println!(
            "Canopies Brushed: {} | Trunk Collisions: {}",
            res.canopies_brushed, res.trunk_collisions
        );

        println!("Sector Split Times:");
        for (sec_name, sec_time) in &res.sector_times {
            println!("  - {}: {:.2}s", sec_name, sec_time);
        }

        // Basic sanity assertions
        assert!(res.distance_traveled_m > 50.0);
        assert!(res.avg_speed_kmh > 10.0);
        assert!(res.max_cross_track_error_m.is_finite());
        assert!(!res.max_cross_track_error_m.is_nan());
        assert!(res.rms_cross_track_error_m.is_finite());
        assert!(!res.rms_cross_track_error_m.is_nan());
    }
}
