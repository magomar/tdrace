use serde::{Deserialize, Serialize};
use crate::config::CarConfig;
use super::constraints::DynamicMetrics;

/// Complete result receipt of an automated constrained optimization run (Spec 035).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub vehicle_name: String,
    pub constraint_name: String,
    pub target_name: String,
    pub initial_cost: f32,
    pub final_cost: f32,
    pub cost_improvement_pct: f32,
    pub generations_evaluated: usize,
    pub total_simulations: usize,
    pub wall_clock_seconds: f32,
    pub initial_metrics: DynamicMetrics,
    pub optimized_metrics: DynamicMetrics,
    pub constraint_violations: Vec<(String, f32)>,
    pub optimized_config: CarConfig,
}

impl CalibrationResult {
    /// Formats the calibration receipt as human-readable Markdown.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# 🏁 Auto-Calibration Receipt: {}\n\n", self.vehicle_name));
        out.push_str(&format!("* **Target Benchmark**: {}\n", self.target_name));
        out.push_str(&format!("* **Constraint Regime**: {}\n", self.constraint_name));
        out.push_str(&format!("* **Generations Evaluated**: {}\n", self.generations_evaluated));
        out.push_str(&format!("* **Total Simulations**: {}\n", self.total_simulations));
        out.push_str(&format!("* **Execution Time**: {:.3} s\n", self.wall_clock_seconds));
        out.push_str(&format!("* **Cost Improvement**: {:.1}% (Initial: {:.2} -> Final: {:.2})\n\n",
            self.cost_improvement_pct, self.initial_cost, self.final_cost));

        out.push_str("## 📊 Dynamic Metrics Telemetry Comparison\n\n");
        out.push_str("| Metric | Baseline | Optimized Calibrated | Unit |\n");
        out.push_str("| :--- | :---: | :---: | :---: |\n");
        out.push_str(&format!("| Turning Circle Diameter | {:.2} | **{:.2}** | meters |\n",
            self.initial_metrics.turning_diameter_m, self.optimized_metrics.turning_diameter_m));
        out.push_str(&format!("| Inside Rear Unloading | {:.1}% | **{:.1}%** | load ratio |\n",
            self.initial_metrics.inside_rear_unloading_ratio * 100.0, self.optimized_metrics.inside_rear_unloading_ratio * 100.0));
        out.push_str(&format!("| Trail-Braking Peak Yaw Accel | {:.2} | **{:.2}** | rad/s² |\n",
            self.initial_metrics.peak_yaw_acceleration_rad_s2, self.optimized_metrics.peak_yaw_acceleration_rad_s2));
        out.push_str(&format!("| Max Body Sideslip | {:.2}° | **{:.2}°** | degrees |\n",
            self.initial_metrics.max_sideslip_deg, self.optimized_metrics.max_sideslip_deg));
        out.push_str(&format!("| Differential Slip Integral | {:.2} | **{:.2}** | rad·s |\n\n",
            self.initial_metrics.axle_slip_differential_integral, self.optimized_metrics.axle_slip_differential_integral));

        if self.constraint_violations.is_empty() {
            out.push_str("✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!\n");
        } else {
            out.push_str("⚠️ **Active Constraint Boundaries**:\n");
            for (name, val) in &self.constraint_violations {
                out.push_str(&format!("- *{}*: deviation = {:.3}\n", name, val));
            }
        }

        out
    }
}
