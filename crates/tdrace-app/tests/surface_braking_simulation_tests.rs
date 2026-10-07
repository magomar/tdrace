//! Integration tests for multi-surface car braking dynamics and stability simulation.

use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::RallyGameModule;
use tdrace_core::physics::sim::{
    run_braking_cadence, run_braking_in_turn, run_braking_split_mu, run_braking_straight_line,
    BrakingStabilityRating, CorneringBrakingBehavior, SplitMuStatus, DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::{CarConfig, SurfaceType};

#[test]
fn test_multi_surface_panic_stopping_distances() {
    let test_vehicles = [
        ("gt3_evo", GtWorldChallengeModule::car_gt3_evo(), 150.0),
        ("wrc_turbo_rally", RallyGameModule::car_wrc_rally(), 140.0),
        ("classic_sports_car", CarConfig::sports_car(), 130.0),
    ];

    for (id, config, v0) in test_vehicles {
        let res_asphalt = run_braking_straight_line(&config, SurfaceType::Asphalt, v0, false, DEFAULT_SIMULATION_DT);
        let res_concrete = run_braking_straight_line(&config, SurfaceType::Concrete, v0, false, DEFAULT_SIMULATION_DT);
        let res_gravel = run_braking_straight_line(&config, SurfaceType::Gravel, v0, false, DEFAULT_SIMULATION_DT);
        let res_grass = run_braking_straight_line(&config, SurfaceType::Grass, v0, false, DEFAULT_SIMULATION_DT);
        let res_ice = run_braking_straight_line(&config, SurfaceType::SheetIce, v0, false, DEFAULT_SIMULATION_DT);

        // 1. Friction hierarchy: Asphalt < Concrete <= Gravel < Grass < SheetIce
        assert!(
            res_asphalt.stopping_distance_m <= res_concrete.stopping_distance_m + 1.0,
            "{}: Asphalt ({:.1}m) should stop equal or shorter than Concrete ({:.1}m)",
            id,
            res_asphalt.stopping_distance_m,
            res_concrete.stopping_distance_m
        );
        assert!(
            res_concrete.stopping_distance_m < res_gravel.stopping_distance_m,
            "{}: Concrete ({:.1}m) should stop shorter than Gravel ({:.1}m)",
            id,
            res_concrete.stopping_distance_m,
            res_gravel.stopping_distance_m
        );
        assert!(
            res_gravel.stopping_distance_m < res_grass.stopping_distance_m,
            "{}: Gravel ({:.1}m) should stop shorter than Grass ({:.1}m)",
            id,
            res_gravel.stopping_distance_m,
            res_grass.stopping_distance_m
        );
        assert!(
            res_ice.stopping_distance_m > res_asphalt.stopping_distance_m * 3.0,
            "{}: SheetIce ({:.1}m) should exceed 3x Asphalt ({:.1}m)",
            id,
            res_ice.stopping_distance_m,
            res_asphalt.stopping_distance_m
        );
    }
}

#[test]
fn test_panic_braking_with_yaw_disturbance_stability() {
    let config = GtWorldChallengeModule::car_gt3_evo();

    for &surface in &[SurfaceType::Asphalt, SurfaceType::Concrete, SurfaceType::Curb, SurfaceType::Dirt] {
        let res = run_braking_straight_line(&config, surface, 150.0, true, DEFAULT_SIMULATION_DT);

        assert_eq!(
            res.stability_rating,
            BrakingStabilityRating::Stable,
            "GT3 Evo with yaw disturbance on {} should remain Stable (sideslip = {:.1}°, heading dev = {:.1}°)",
            surface.name(),
            res.max_sideslip_deg,
            res.heading_deviation_deg
        );
        assert!(
            res.max_sideslip_deg < 10.0,
            "Sideslip on {} was {:.1}°, expected < 10°",
            surface.name(),
            res.max_sideslip_deg
        );
    }
}

#[test]
fn test_dynamic_ebd_rear_lockup_prevention() {
    let config = GtWorldChallengeModule::car_gt3_evo();

    for &surface in &SurfaceType::ALL {
        let res = run_braking_straight_line(&config, surface, 150.0, false, DEFAULT_SIMULATION_DT);

        // Rear lockup duration should be strictly less than or equal to front lockup + 0.1s tolerance
        assert!(
            res.rear_lockup_duration_s <= res.front_lockup_duration_s + 0.10,
            "Dynamic EBD violated on {}: rear lockup ({:.2}s) exceeded front lockup ({:.2}s)",
            surface.name(),
            res.rear_lockup_duration_s,
            res.front_lockup_duration_s
        );
    }
}

#[test]
fn test_split_mu_asymmetric_braking_stability() {
    let config = GtWorldChallengeModule::car_gt3_evo();

    // Spec 074 gives slicks 0.40 grip on grass and 0.32 on gravel: braking hard with two wheels there
    // spins a GT3, as it would a real one (Mario, 2026-10-06: accept). The splits below stay on surfaces
    // slicks are made for. Dirt is 0.85 for Medium Slick (open question tdrace-md1u).
    // 1. Moderate split-mu: Asphalt vs Dirt
    let res_grass = run_braking_split_mu(&config, SurfaceType::Asphalt, SurfaceType::Dirt, 100.0, DEFAULT_SIMULATION_DT);
    assert_ne!(
        res_grass.status,
        SplitMuStatus::SpunOut,
        "Vehicle spun out on Asphalt vs Dirt split-mu (heading dev = {:.1}°)",
        res_grass.heading_deviation_deg
    );

    // 2. Minor split-mu: Asphalt vs Curb
    let res_gravel = run_braking_split_mu(&config, SurfaceType::Asphalt, SurfaceType::Curb, 100.0, DEFAULT_SIMULATION_DT);
    assert_ne!(
        res_gravel.status,
        SplitMuStatus::SpunOut,
        "Vehicle spun out on Asphalt vs Curb split-mu"
    );

    // 3. Stopping distance on split-mu should be between uniform asphalt and uniform dirt
    let res_asp_pure = run_braking_straight_line(&config, SurfaceType::Asphalt, 100.0, false, DEFAULT_SIMULATION_DT);
    let res_dirt_pure = run_braking_straight_line(&config, SurfaceType::Dirt, 100.0, false, DEFAULT_SIMULATION_DT);
    assert!(
        res_grass.stopping_distance_m >= res_asp_pure.stopping_distance_m * 0.95,
        "Split-mu stop ({:.1}m) should be >= pure asphalt ({:.1}m)",
        res_grass.stopping_distance_m,
        res_asp_pure.stopping_distance_m
    );
    assert!(
        res_grass.stopping_distance_m <= res_dirt_pure.stopping_distance_m * 1.05,
        "Split-mu stop ({:.1}m) should be <= pure dirt ({:.1}m)",
        res_grass.stopping_distance_m,
        res_dirt_pure.stopping_distance_m
    );
}

#[test]
fn test_cadence_braking_wheel_spinup_recovery() {
    let config = GtWorldChallengeModule::car_gt3_evo();

    // Slick surfaces only: on gravel (0.32 grip, spec 074) a locked slick takes ~660 ms to spin up again.
    for &surface in &[SurfaceType::Asphalt, SurfaceType::Concrete, SurfaceType::Curb, SurfaceType::Dirt] {
        let res = run_braking_cadence(&config, surface, 140.0, DEFAULT_SIMULATION_DT);

        assert!(
            res.total_pulse_cycles >= 2,
            "Should have completed multiple pulse cycles on {}",
            surface.name()
        );
        // Wheel recovery latency must be under 50ms upon brake release (EDR working)
        assert!(
            res.wheel_recovery_latency_ms < 50.0,
            "Wheel recovery latency on {} was {:.1}ms (expected < 50ms)",
            surface.name(),
            res.wheel_recovery_latency_ms
        );
    }
}

#[test]
fn test_cornering_trail_braking_cbc_stability() {
    let config = GtWorldChallengeModule::car_gt3_evo();

    for &surface in &[SurfaceType::Asphalt, SurfaceType::Concrete, SurfaceType::Curb] {
        let res = run_braking_in_turn(&config, surface, 40.0, 70.0, DEFAULT_SIMULATION_DT);

        assert_ne!(
            res.behavior,
            CorneringBrakingBehavior::SnapOversteerSpin,
            "GT3 Evo should not snap oversteer while trail-braking on {}",
            surface.name()
        );
        assert!(
            res.max_sideslip_deg < 25.0,
            "Sideslip during cornering braking on {} was {:.1}° (expected < 25°)",
            surface.name(),
            res.max_sideslip_deg
        );
    }
}
