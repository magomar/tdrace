//! # Simulation & Dynamic Benchmarking Harness
//!
//! Pure memory-to-memory headless vehicle physics simulation suite.
//! Provides high-frequency telemetry recording, standardized dynamic test protocols
//! (Protocols A through E), and cross-surface multi-vehicle benchmark matrices.

pub mod harness;
pub mod matrix;
pub mod protocols;
pub mod report;
pub mod telemetry;

pub use harness::{SimulationRunner, DEFAULT_SIMULATION_DT};
pub use matrix::{ExperimentDataset, VehicleBenchmarkResult};
pub use protocols::{
    run_protocol_a, run_protocol_b, run_protocol_c, run_protocol_d, run_protocol_e,
    ProtocolAResult, ProtocolBResult, ProtocolCResult, ProtocolDResult, ProtocolEResult,
    SkidpadDepartureMode, StepSteerStatus,
};
pub use report::{generate_html_report, generate_markdown_report};
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
        let res_ice = run_protocol_b(&config, SurfaceType::Ice, 100.0, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.stopping_distance_m < res_ice.stopping_distance_m);
        assert!(res_asphalt.avg_decel_g > res_ice.avg_decel_g);
    }

    #[test]
    fn test_protocol_e_coast_down() {
        let config = CarConfig::sports_car();
        let res_asphalt = run_protocol_e(&config, SurfaceType::Asphalt, 120.0, DEFAULT_SIMULATION_DT);
        let res_sand = run_protocol_e(&config, SurfaceType::Sand, 120.0, DEFAULT_SIMULATION_DT);
        assert!(res_asphalt.coast_distance_m > res_sand.coast_distance_m);
    }
}
