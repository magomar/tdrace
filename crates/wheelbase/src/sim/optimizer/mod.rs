//! # Automated Simulation Harness Parameter Optimization Subsystem (Spec 035)
//!
//! Provides constrained evolutionary optimization (CMA-ES) to calibrate vehicle dynamics
//! under strict physical, kinematic, and homologation constraints.

pub mod cma_es;
pub mod constraints;
pub mod evaluator;
pub mod report;
pub mod targets;

pub use cma_es::{CmaEsOptimizer, ParameterBound};
pub use constraints::{DrivetrainConstraint, DynamicMetrics};
pub use evaluator::OptimizationEvaluator;
pub use report::CalibrationResult;
pub use targets::CalibrationTarget;

use serde::{Deserialize, Serialize};
use crate::config::CarConfig;
use std::time::Instant;

/// Execution configuration for an automated calibration run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTunerConfig {
    pub max_generations: usize,
    pub population_size: Option<usize>,
    pub convergence_epsilon: f32,
    pub penalty_stiffness: f32,
    pub dt: f32,
    pub seed: u64,
}

impl Default for AutoTunerConfig {
    fn default() -> Self {
        Self {
            max_generations: 40,
            population_size: Some(16),
            convergence_epsilon: 1e-4,
            penalty_stiffness: 1.0,
            dt: 1.0 / 60.0,
            seed: 42,
        }
    }
}

/// Helper that returns standard tunable parameter bounds adapted to the drivetrain constraint regime.
pub fn standard_tuning_bounds(constraint: &DrivetrainConstraint, base_config: &CarConfig) -> Vec<ParameterBound> {
    let mut bounds = Vec::new();

    bounds.push(ParameterBound::new(
        "speed_sensitive_steer_factor",
        (base_config.speed_sensitive_steer_factor * 0.4).max(0.0002),
        (base_config.speed_sensitive_steer_factor * 2.5).min(0.030),
        base_config.speed_sensitive_steer_factor,
    ));

    bounds.push(ParameterBound::new(
        "angular_damping",
        (base_config.angular_damping * 0.7).max(10.0),
        (base_config.angular_damping * 1.4).max(30.0),
        base_config.angular_damping,
    ));

    bounds.push(ParameterBound::new(
        "weight_transfer_lateral",
        (base_config.weight_transfer_lateral * 0.7).max(0.2),
        (base_config.weight_transfer_lateral * 1.3).min(2.5),
        base_config.weight_transfer_lateral,
    ));

    bounds.push(ParameterBound::new(
        "weight_transfer_longitudinal",
        (base_config.weight_transfer_longitudinal * 0.7).max(0.2),
        (base_config.weight_transfer_longitudinal * 1.3).min(2.5),
        base_config.weight_transfer_longitudinal,
    ));

    bounds.push(ParameterBound::new(
        "brake_bias",
        0.50,
        0.70,
        base_config.brake_bias,
    ));

    match constraint {
        DrivetrainConstraint::SpoolAxle {
            min_caster_jacking_unloading_ratio,
            ..
        } => {
            if *min_caster_jacking_unloading_ratio > 0.10 {
                bounds.push(ParameterBound::new(
                    "caster_jacking_factor",
                    0.20,
                    1.80,
                    base_config.caster_jacking_factor,
                ));
            }
            bounds.push(ParameterBound::new(
                "max_steer_angle",
                0.40,
                0.80,
                base_config.max_steer_angle,
            ));
        }
        DrivetrainConstraint::SalisburyRwd { max_preload_nm, .. } => {
            bounds.push(ParameterBound::new(
                "power_lock",
                0.20,
                0.90,
                0.50,
            ));
            bounds.push(ParameterBound::new(
                "coast_lock",
                0.05,
                0.60,
                0.25,
            ));
            bounds.push(ParameterBound::new(
                "preload_nm",
                10.0,
                *max_preload_nm,
                45.0,
            ));
        }
        DrivetrainConstraint::MultiLsdAwd { drive_bias_range, .. } => {
            bounds.push(ParameterBound::new(
                "drive_bias",
                drive_bias_range.0,
                drive_bias_range.1,
                base_config.drive_bias,
            ));
        }
        DrivetrainConstraint::OpenDiff => {}
    }

    bounds
}

/// Orchestrator for executing automated vehicle calibrations.
pub struct AutoCalibrator {
    pub config: AutoTunerConfig,
    pub constraint: DrivetrainConstraint,
    pub target: CalibrationTarget,
    pub bounds: Vec<ParameterBound>,
}

impl AutoCalibrator {
    pub fn new(
        config: AutoTunerConfig,
        constraint: DrivetrainConstraint,
        target: CalibrationTarget,
        bounds: Vec<ParameterBound>,
    ) -> Self {
        Self {
            config,
            constraint,
            target,
            bounds,
        }
    }

    /// Executes the constrained evolutionary optimization loop and returns a verified `CalibrationResult`.
    pub fn calibrate(&self, vehicle_name: &str, base_config: &CarConfig) -> CalibrationResult {
        let start_time = Instant::now();

        let evaluator = OptimizationEvaluator::new(
            *base_config,
            self.bounds.clone(),
            self.constraint.clone(),
            self.target.clone(),
            self.config.dt,
            self.config.penalty_stiffness,
        );

        let mut optimizer = CmaEsOptimizer::new(
            &self.bounds,
            self.config.population_size,
            self.config.seed,
        );

        // Evaluate baseline configuration
        let initial_normalized: Vec<f32> = self.bounds.iter().map(|b| b.to_normalized(b.default)).collect();
        let (initial_cost, initial_metrics, _) = evaluator.evaluate(&initial_normalized);

        let mut best_cost = f32::INFINITY;
        let mut best_normalized = initial_normalized.clone();
        let mut total_simulations = 0;
        let mut consecutive_stalls = 0;

        for _gen in 0..self.config.max_generations {
            let candidates = optimizer.ask();
            let mut evaluated = Vec::with_capacity(candidates.len());

            for cand in candidates {
                let (cost, _metrics, _violations) = evaluator.evaluate(&cand);
                evaluated.push((cand, cost));
                total_simulations += 1;
            }

            optimizer.tell(evaluated);

            let gen_best_cost = optimizer.best_cost;
            let cost_delta = (best_cost - gen_best_cost).abs();
            if gen_best_cost < best_cost {
                best_cost = gen_best_cost;
                best_normalized = optimizer.best_solution.clone();
                consecutive_stalls = 0;
            } else if cost_delta < self.config.convergence_epsilon {
                consecutive_stalls += 1;
                if consecutive_stalls >= 15 {
                    break;
                }
            }
        }

        let elapsed = start_time.elapsed().as_secs_f32();
        let (final_cost, optimized_metrics, violations) = evaluator.evaluate(&best_normalized);
        let optimized_config = evaluator.decode_config(&best_normalized);

        let cost_improvement_pct = if initial_cost > 1e-4 {
            ((initial_cost - final_cost) / initial_cost) * 100.0
        } else {
            0.0
        };

        CalibrationResult {
            vehicle_name: vehicle_name.to_string(),
            constraint_name: format!("{:?}", self.constraint),
            target_name: self.target.name.clone(),
            initial_cost,
            final_cost,
            cost_improvement_pct,
            generations_evaluated: optimizer.generation,
            total_simulations,
            wall_clock_seconds: elapsed,
            initial_metrics,
            optimized_metrics,
            constraint_violations: violations,
            optimized_config,
        }
    }
}
