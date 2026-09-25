use serde::{Deserialize, Serialize};
use crate::config::{CarConfig, DifferentialType};

/// Metrics gathered during dynamic simulation harness runs, used to evaluate physical constraint satisfaction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicMetrics {
    /// Measured minimum turning diameter in meters at low speed.
    pub turning_diameter_m: f32,
    /// Unloading ratio of the inside rear wheel under maximum steering lock (1.0 = 100% unloaded, 0.0 = static).
    pub inside_rear_unloading_ratio: f32,
    /// Peak yaw angular acceleration in rad/s^2 observed during combined trail-braking.
    pub peak_yaw_acceleration_rad_s2: f32,
    /// Maximum body sideslip angle in degrees observed across all maneuvers.
    pub max_sideslip_deg: f32,
    /// Integral of differential slip speed across the driven axle in rad*s.
    pub axle_slip_differential_integral: f32,
    /// Ratio of front-to-total drive force delivered under acceleration.
    pub delivered_front_drive_ratio: f32,
}

/// Declares the physical constraint regime for vehicle optimization (Spec 035).
///
/// Ensures the optimizer strictly respects the mechanical nature and real-world boundaries of the drivetrain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DrivetrainConstraint {
    /// 100% mechanically locked live axle (omega_L == omega_R).
    /// Used on Karts, NASCAR Stock Cars (TA1/Cup), and Spool Buggies.
    SpoolAxle {
        /// Maximum allowable low-speed turning circle diameter in meters (e.g. 2.60m for karts).
        max_turning_diameter_m: f32,
        /// Minimum inside-rear wheel load reduction ratio under steering lock (e.g. >= 0.80 for sprint karts).
        min_caster_jacking_unloading_ratio: f32,
    },
    /// Salisbury multi-plate clutch limited slip differential on RWD powertrain.
    /// Used on GT3, GT4, Sports Cars, and Touring Cars.
    SalisburyRwd {
        /// Minimum delta by which power lock must exceed coast lock (power_lock - coast_lock >= min_delta).
        min_power_coast_delta: f32,
        /// Maximum physical clutch spring plate preload in N*m.
        max_preload_nm: f32,
        /// Maximum allowable yaw angular acceleration during trail-braking to prevent snap-oversteer spinouts.
        max_yaw_acceleration_rad_s2: f32,
    },
    /// Dual or triple limited slip differential all-wheel-drive system.
    /// Used on Rallycross Supercars, Group B Legends, and LMH/LMDh Hypercars.
    MultiLsdAwd {
        /// Permissible front drive torque bias window [min, max].
        drive_bias_range: (f32, f32),
        /// Enforce strict conservation of torque between front and rear axles.
        strict_torque_conservation: bool,
    },
    /// Conventional open differential with symmetrical 50/50 torque split.
    /// Used on starter road cars and unpowered axles.
    OpenDiff,
}

impl DrivetrainConstraint {
    /// Applies hard equality and manifold constraints directly to the configuration before simulation.
    pub fn project_hard_constraints(&self, config: &mut CarConfig) {
        match self {
            Self::SpoolAxle {
                min_caster_jacking_unloading_ratio,
                ..
            } => {
                // Solid axle: rear diff is rigidly Spool, RWD drive bias (0.0)
                config.rear_differential = DifferentialType::Spool;
                config.front_differential = DifferentialType::Open;
                config.drive_bias = 0.0;
                if *min_caster_jacking_unloading_ratio > 0.10 && config.caster_jacking_factor < 0.20 {
                    config.caster_jacking_factor = 0.80;
                }
            }
            Self::SalisburyRwd {
                min_power_coast_delta,
                max_preload_nm,
                ..
            } => {
                // RWD Salisbury: front is unpowered Open, rear is LimitedSlip
                config.front_differential = DifferentialType::Open;
                config.drive_bias = 0.0;

                match config.rear_differential {
                    DifferentialType::LimitedSlip {
                        mut power_lock,
                        mut coast_lock,
                        mut preload_nm,
                    } => {
                        power_lock = power_lock.clamp(0.10, 1.0);
                        let max_coast = (power_lock - min_power_coast_delta).max(0.05);
                        coast_lock = coast_lock.clamp(0.05, max_coast);
                        preload_nm = preload_nm.clamp(5.0, *max_preload_nm);

                        config.rear_differential = DifferentialType::LimitedSlip {
                            power_lock,
                            coast_lock,
                            preload_nm,
                        };
                    }
                    _ => {
                        config.rear_differential = DifferentialType::LimitedSlip {
                            power_lock: 0.50,
                            coast_lock: (0.50 - min_power_coast_delta).max(0.10),
                            preload_nm: 45.0_f32.min(*max_preload_nm),
                        };
                    }
                }
            }
            Self::MultiLsdAwd {
                drive_bias_range,
                strict_torque_conservation: _,
            } => {
                config.drive_bias = config
                    .drive_bias
                    .clamp(drive_bias_range.0, drive_bias_range.1);
            }
            Self::OpenDiff => {
                config.rear_differential = DifferentialType::Open;
                config.front_differential = DifferentialType::Open;
            }
        }
    }

    /// Evaluates physical dynamic inequality constraint violations and computes penalty values.
    ///
    /// Returns `(total_penalty, violations_list)`.
    pub fn evaluate_dynamic_penalties(
        &self,
        metrics: &DynamicMetrics,
        penalty_stiffness: f32,
    ) -> (f32, Vec<(String, f32)>) {
        let mut total_penalty = 0.0f32;
        let mut violations = Vec::new();

        match self {
            Self::SpoolAxle {
                max_turning_diameter_m,
                min_caster_jacking_unloading_ratio,
            } => {
                // Constraint: Turning circle must be <= max_turning_diameter_m
                if metrics.turning_diameter_m > *max_turning_diameter_m {
                    let excess = metrics.turning_diameter_m - max_turning_diameter_m;
                    let p = penalty_stiffness * excess * excess * 100.0;
                    total_penalty += p;
                    violations.push(("Turning Diameter Exceeded".to_string(), excess));
                }

                // Constraint: Inside rear wheel unloading must be >= min_caster_jacking_unloading_ratio
                if metrics.inside_rear_unloading_ratio < *min_caster_jacking_unloading_ratio {
                    let deficit = min_caster_jacking_unloading_ratio - metrics.inside_rear_unloading_ratio;
                    let p = penalty_stiffness * deficit * deficit * 200.0;
                    total_penalty += p;
                    violations.push(("Insufficient Caster Jacking Unloading".to_string(), deficit));
                }
            }
            Self::SalisburyRwd {
                max_yaw_acceleration_rad_s2,
                ..
            } => {
                // Constraint: Peak yaw acceleration in trail-braking must be <= max_yaw_acceleration_rad_s2
                if metrics.peak_yaw_acceleration_rad_s2 > *max_yaw_acceleration_rad_s2 {
                    let excess = metrics.peak_yaw_acceleration_rad_s2 - max_yaw_acceleration_rad_s2;
                    let p = penalty_stiffness * excess * excess * 50.0;
                    total_penalty += p;
                    violations.push(("Trail-Braking Yaw Acceleration Divergence".to_string(), excess));
                }

                // General stability: body sideslip angle must not exceed 15 degrees
                if metrics.max_sideslip_deg > 15.0 {
                    let excess = metrics.max_sideslip_deg - 15.0;
                    let p = penalty_stiffness * excess * excess * 10.0;
                    total_penalty += p;
                    violations.push(("Excessive Body Sideslip".to_string(), excess));
                }
            }
            Self::MultiLsdAwd { drive_bias_range, .. } => {
                if metrics.delivered_front_drive_ratio < drive_bias_range.0 {
                    let deficit = drive_bias_range.0 - metrics.delivered_front_drive_ratio;
                    total_penalty += penalty_stiffness * deficit * deficit * 100.0;
                    violations.push(("Front Drive Bias Below Range".to_string(), deficit));
                } else if metrics.delivered_front_drive_ratio > drive_bias_range.1 {
                    let excess = metrics.delivered_front_drive_ratio - drive_bias_range.1;
                    total_penalty += penalty_stiffness * excess * excess * 100.0;
                    violations.push(("Front Drive Bias Above Range".to_string(), excess));
                }
            }
            Self::OpenDiff => {
                // General slip control
                if metrics.max_sideslip_deg > 18.0 {
                    let excess = metrics.max_sideslip_deg - 18.0;
                    total_penalty += penalty_stiffness * excess * excess * 10.0;
                    violations.push(("Sideslip Spinout".to_string(), excess));
                }
            }
        }

        (total_penalty, violations)
    }
}
