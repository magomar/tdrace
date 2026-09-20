//! Integration test verifying surface-car dynamics simulation against Spec 010.

use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::{
    ExtremeOffRoadModule, KartGameModule, NascarGameModule, RallyGameModule,
};
use tdrace_core::physics::sim::{
    run_protocol_a, run_protocol_b, run_protocol_c, run_protocol_e, DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::SurfaceType;

#[test]
fn test_all_categories_surface_dynamics_degradation() {
    let test_vehicles = [
        ("gt4_clubsport", GtWorldChallengeModule::car_gt4_clubsport()),
        ("gt3_evo", GtWorldChallengeModule::car_gt3_evo()),
        ("nascar_cup_v8", NascarGameModule::car_stock_car()),
        ("wrc_turbo_rally", RallyGameModule::car_wrc_rally()),
        ("shifter_kart_125", KartGameModule::car_shifter_kart()),
        ("sand_rail_buggy", ExtremeOffRoadModule::car_sand_rail()),
    ];

    for (id, config) in test_vehicles {
        // 1. Acceleration: Asphalt must be quicker or reach higher speed than Sand
        let a_asphalt = run_protocol_a(&config, SurfaceType::Asphalt, 6.0, DEFAULT_SIMULATION_DT);
        let a_sand = run_protocol_a(&config, SurfaceType::Sand, 6.0, DEFAULT_SIMULATION_DT);
        assert!(
            a_asphalt.v_terminal_kmh > a_sand.v_terminal_kmh,
            "{}: Asphalt terminal speed ({:.1} km/h) should exceed Sand ({:.1} km/h)",
            id,
            a_asphalt.v_terminal_kmh,
            a_sand.v_terminal_kmh
        );

        // 2. Braking: Stopping distance on Ice must be > 2x Asphalt, and Concrete must be close to Asphalt
        let b_asphalt = run_protocol_b(&config, SurfaceType::Asphalt, 100.0, DEFAULT_SIMULATION_DT);
        let b_concrete = run_protocol_b(&config, SurfaceType::Concrete, 100.0, DEFAULT_SIMULATION_DT);
        let b_ice = run_protocol_b(&config, SurfaceType::Ice, 100.0, DEFAULT_SIMULATION_DT);
        assert!(
            b_ice.stopping_distance_m > b_asphalt.stopping_distance_m * 2.0,
            "{}: Stopping distance on Ice ({:.1}m) should exceed 2x Asphalt ({:.1}m)",
            id,
            b_ice.stopping_distance_m,
            b_asphalt.stopping_distance_m
        );
        assert!(
            b_concrete.stopping_distance_m >= b_asphalt.stopping_distance_m * 0.95
                && b_concrete.stopping_distance_m < b_ice.stopping_distance_m * 0.5,
            "{}: Stopping distance on Concrete ({:.1}m) should be close to Asphalt ({:.1}m)",
            id,
            b_concrete.stopping_distance_m,
            b_asphalt.stopping_distance_m
        );

        // 3. Coast-down: Asphalt must roll further than Sand due to rolling resistance
        let e_asphalt = run_protocol_e(&config, SurfaceType::Asphalt, 120.0, DEFAULT_SIMULATION_DT);
        let e_sand = run_protocol_e(&config, SurfaceType::Sand, 120.0, DEFAULT_SIMULATION_DT);
        assert!(
            e_asphalt.coast_distance_m > e_sand.coast_distance_m,
            "{}: Coast distance on Asphalt ({:.1}m) should exceed Sand ({:.1}m)",
            id,
            e_asphalt.coast_distance_m,
            e_sand.coast_distance_m
        );

        // 4. Skidpad: Asphalt cornering limit must realistically exceed Ice (> 5x) and be >= 0.85g
        let c_asphalt = run_protocol_c(&config, SurfaceType::Asphalt, 30.0, DEFAULT_SIMULATION_DT);
        let c_ice = run_protocol_c(&config, SurfaceType::Ice, 30.0, DEFAULT_SIMULATION_DT);
        assert!(
            c_asphalt.peak_lateral_accel_g > c_ice.peak_lateral_accel_g * 5.0,
            "{}: Asphalt lateral g ({:.2}g) should exceed 5x Ice ({:.2}g)",
            id,
            c_asphalt.peak_lateral_accel_g,
            c_ice.peak_lateral_accel_g
        );
        assert!(
            c_asphalt.peak_lateral_accel_g >= 0.85,
            "{}: Asphalt lateral g ({:.2}g) should be >= 0.85g",
            id,
            c_asphalt.peak_lateral_accel_g
        );
    }
}
