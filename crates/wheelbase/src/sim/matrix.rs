use serde::{Deserialize, Serialize};

use crate::config::CarConfig;
use crate::surface::SurfaceType;
use super::protocols::{
    run_protocol_a, run_protocol_b, run_protocol_c, run_protocol_d, run_protocol_e,
    ProtocolAResult, ProtocolBResult, ProtocolCResult, ProtocolDResult, ProtocolEResult,
};

/// Complete benchmark test results for a single vehicle across all tested surfaces and protocols.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleBenchmarkResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub category: String,
    pub module: String,
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub drivetrain: String,
    #[serde(default)]
    pub mass_kg: f32,
    #[serde(default)]
    pub power_bhp: u16,
    #[serde(default)]
    pub top_speed_kmh: u16,
    pub protocol_a: Vec<ProtocolAResult>,
    pub protocol_b: Vec<ProtocolBResult>,
    pub protocol_c: Vec<ProtocolCResult>,
    pub protocol_d: Vec<ProtocolDResult>,
    pub protocol_e: Vec<ProtocolEResult>,
}

impl VehicleBenchmarkResult {
    /// Sets engineering metadata for reporting.
    pub fn with_specs(
        mut self,
        tier: u8,
        drivetrain: impl Into<String>,
        mass_kg: f32,
        power_bhp: u16,
        top_speed_kmh: u16,
    ) -> Self {
        self.tier = tier;
        self.drivetrain = drivetrain.into();
        self.mass_kg = mass_kg;
        self.power_bhp = power_bhp;
        self.top_speed_kmh = top_speed_kmh;
        self
    }
    /// Executes the full benchmark suite (Protocols A through E) across all 12 surfaces for a vehicle.
    pub fn run(
        vehicle_id: impl Into<String>,
        vehicle_name: impl Into<String>,
        category: impl Into<String>,
        module: impl Into<String>,
        config: &CarConfig,
        dt: f32,
    ) -> Self {
        let surfaces = SurfaceType::ALL;

        let mut a_results = Vec::with_capacity(surfaces.len());
        let mut b_results = Vec::with_capacity(surfaces.len());
        let mut c_results = Vec::with_capacity(surfaces.len());
        let mut d_results = Vec::with_capacity(surfaces.len());
        let mut e_results = Vec::with_capacity(surfaces.len());

        for &surf in &surfaces {
            a_results.push(run_protocol_a(config, surf, 25.0, dt));
            b_results.push(run_protocol_b(config, surf, 100.0, dt));
            c_results.push(run_protocol_c(config, surf, 30.0, dt));
            d_results.push(run_protocol_d(config, surf, 80.0, dt));
            e_results.push(run_protocol_e(config, surf, 120.0, dt));
        }

        Self {
            vehicle_id: vehicle_id.into(),
            vehicle_name: vehicle_name.into(),
            category: category.into(),
            module: module.into(),
            tier: 0,
            drivetrain: String::new(),
            mass_kg: 0.0,
            power_bhp: 0,
            top_speed_kmh: 0,
            protocol_a: a_results,
            protocol_b: b_results,
            protocol_c: c_results,
            protocol_d: d_results,
            protocol_e: e_results,
        }
    }
}

/// Aggregated multi-vehicle simulation experiment dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentDataset {
    pub experiment_name: String,
    pub timestamp_utc: String,
    pub vehicles: Vec<VehicleBenchmarkResult>,
}

impl ExperimentDataset {
    pub fn new(name: impl Into<String>, timestamp_utc: impl Into<String>) -> Self {
        Self {
            experiment_name: name.into(),
            timestamp_utc: timestamp_utc.into(),
            vehicles: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
