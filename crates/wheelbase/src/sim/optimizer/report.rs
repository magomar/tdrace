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
            out.push_str("✅ **Constraint Verification**: All physical, kinematic, and homologation constraints strictly satisfied!\n\n");
        } else {
            out.push_str("⚠️ **Active Constraint Boundaries**:\n");
            for (name, val) in &self.constraint_violations {
                out.push_str(&format!("- *{}*: deviation = {:.3}\n", name, val));
            }
            out.push_str("\n");
        }

        out.push_str("### ⚙️ Calibrated Vehicle Parameters (Rust Source Representation)\n\n");
        out.push_str("```rust\n");
        out.push_str(&self.to_rust_snippet());
        out.push_str("```\n");

        out
    }

    /// Formats the tuned parameters as Rust struct initialization lines.
    pub fn to_rust_snippet(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("speed_sensitive_steer_factor: {:.5},\n", self.optimized_config.speed_sensitive_steer_factor));
        s.push_str(&format!("angular_damping: {:.1},\n", self.optimized_config.angular_damping));
        s.push_str(&format!("weight_transfer_lateral: {:.2},\n", self.optimized_config.weight_transfer_lateral));
        s.push_str(&format!("weight_transfer_longitudinal: {:.2},\n", self.optimized_config.weight_transfer_longitudinal));
        s.push_str(&format!("brake_bias: {:.2},\n", self.optimized_config.brake_bias));
        s.push_str(&format!("max_steer_angle: {:.2},\n", self.optimized_config.max_steer_angle));
        s.push_str(&format!("caster_jacking_factor: {:.2},\n", self.optimized_config.caster_jacking_factor));
        s.push_str(&format!("drive_bias: {:.2},\n", self.optimized_config.drive_bias));
        s.push_str(&format!("rear_differential: {:?},\n", self.optimized_config.rear_differential));
        s
    }

    /// Exports the full calibrated CarConfig as JSON.
    pub fn export_json(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json_str = serde_json::to_string_pretty(&self.optimized_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json_str)
    }
}
