use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::config::TireConfig;
use super::surface::SurfaceType;

/// Identifier for each of the 4 wheels on the chassis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WheelId {
    FrontLeft = 0,
    FrontRight = 1,
    RearLeft = 2,
    RearRight = 3,
}

impl WheelId {
    pub const ALL: [Self; 4] = [
        Self::FrontLeft,
        Self::FrontRight,
        Self::RearLeft,
        Self::RearRight,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn is_front(self) -> bool {
        matches!(self, Self::FrontLeft | Self::FrontRight)
    }

    #[inline]
    pub const fn is_rear(self) -> bool {
        matches!(self, Self::RearLeft | Self::RearRight)
    }

    #[inline]
    pub const fn is_left(self) -> bool {
        matches!(self, Self::FrontLeft | Self::RearLeft)
    }

    #[inline]
    pub const fn is_right(self) -> bool {
        matches!(self, Self::FrontRight | Self::RearRight)
    }
}

/// Comprehensive telemetry data for an individual wheel contact patch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelTelemetry {
    /// Identifier of the wheel.
    pub id: WheelId,
    /// Slip angle (angle between wheel heading and actual contact patch velocity) in radians.
    pub slip_angle: f32,
    /// Longitudinal slip ratio (relative difference between wheel rotational speed and road speed).
    pub slip_ratio: f32,
    /// Dynamic normal vertical load on this tire in Newtons (Fz).
    pub normal_load: f32,
    /// Generated lateral cornering force in wheel frame in Newtons (Fy).
    pub lateral_force: f32,
    /// Generated longitudinal drive/braking force in wheel frame in Newtons (Fx).
    pub longitudinal_force: f32,
    /// Steering angle of the wheel relative to vehicle centerline in radians.
    pub steer_angle: f32,
    /// Contact patch linear velocity vector in world coordinates (m/s).
    pub world_velocity: Vec2,
    /// Contact patch position in world coordinates (meters).
    pub wheel_pos_world: Vec2,
    /// Skid intensity normalized to [0.0, 1.0] for tire smoke, skid marks, and sound effects.
    pub skid_intensity: f32,
    /// Whether this tire is currently actively slipping/skidding.
    pub is_skidding: bool,
    /// Surface type currently under this wheel.
    pub surface: SurfaceType,
}

impl Default for WheelTelemetry {
    fn default() -> Self {
        Self {
            id: WheelId::FrontLeft,
            slip_angle: 0.0,
            slip_ratio: 0.0,
            normal_load: 0.0,
            lateral_force: 0.0,
            longitudinal_force: 0.0,
            steer_angle: 0.0,
            world_velocity: Vec2::ZERO,
            wheel_pos_world: Vec2::ZERO,
            skid_intensity: 0.0,
            is_skidding: false,
            surface: SurfaceType::Asphalt,
        }
    }
}

/// Computes pure lateral force using the Pacejka Magic Formula curve adapted for arcade drifting.
///
/// Returns lateral force Fy in Newtons.
#[inline]
pub fn pacejka_lateral_force(
    slip_angle: f32,
    normal_load: f32,
    friction_coeff: f32,
    config: &TireConfig,
    is_handbraking: bool,
) -> f32 {
    if normal_load <= 1e-4 {
        return 0.0;
    }

    let mu = if is_handbraking {
        friction_coeff * config.handbrake_lateral_friction_multiplier
    } else {
        friction_coeff
    };

    let b = config.stiffness_b;
    let c = config.shape_c;
    let d = mu * normal_load * config.peak_d;
    let e = config.curvature_e;

    // Pacejka formulation: Fy = D * sin(C * arctan(B*alpha - E*(B*alpha - arctan(B*alpha))))
    let b_alpha = b * slip_angle;
    let inner = b_alpha - e * (b_alpha - b_alpha.atan());
    let base_force = d * (c * inner.atan()).sin();

    // Post-peak slide retention for smooth arcade drift control
    let abs_slip = slip_angle.abs();
    let peak_slip = 0.22; // ~12.6 degrees peak grip
    if abs_slip > peak_slip {
        let slide_d = mu * normal_load * config.drift_slide_friction;
        let slide_force = slide_d * slip_angle.signum();
        // Blend from peak to sliding plateau smoothly
        let blend = ((abs_slip - peak_slip) / 0.35).min(1.0);
        base_force * (1.0 - blend) + slide_force * blend
    } else {
        base_force
    }
}

/// Applies the friction circle / ellipse limit to combine longitudinal and lateral forces.
///
/// Ensures that sqrt(Fx^2 + Fy^2) <= mu * Fz.
#[inline]
pub fn solve_combined_slip_forces(
    fx_demand: f32,
    fy_demand: f32,
    max_friction_force: f32,
) -> (f32, f32) {
    if max_friction_force <= 1e-4 {
        return (0.0, 0.0);
    }

    let total_demand_sq = fx_demand * fx_demand + fy_demand * fy_demand;
    let max_force_sq = max_friction_force * max_friction_force;

    if total_demand_sq > max_force_sq {
        let scale = max_friction_force / total_demand_sq.sqrt();
        (fx_demand * scale, fy_demand * scale)
    } else {
        (fx_demand, fy_demand)
    }
}

/// Calculates normalized skid intensity [0.0, 1.0] and skidding status.
#[inline]
pub fn compute_skid_telemetry(
    slip_angle: f32,
    slip_ratio: f32,
    speed: f32,
    is_handbraking: bool,
    config: &TireConfig,
    surface: SurfaceType,
) -> (f32, bool) {
    if speed < 0.5 {
        return (0.0, false);
    }

    let abs_slip = slip_angle.abs();
    let lat_intensity = if abs_slip > config.skid_threshold {
        let range = (config.skid_full_threshold - config.skid_threshold).max(1e-3);
        ((abs_slip - config.skid_threshold) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let abs_ratio = slip_ratio.abs();
    let long_intensity = if abs_ratio > 0.15 {
        ((abs_ratio - 0.15) / 0.50).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut intensity = (lat_intensity + long_intensity * 0.7).clamp(0.0, 1.0);

    if is_handbraking && speed > 1.5 {
        intensity = intensity.max(0.75);
    }

    let is_skidding = intensity > 0.08 && (surface.produces_tire_smoke() || surface.produces_debris_particles());

    (intensity, is_skidding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacejka_lateral_force() {
        let cfg = TireConfig::default();
        let normal_load = 2500.0;
        let friction_coeff = 1.0;

        // Zero slip produces zero lateral force
        let f0 = pacejka_lateral_force(0.0, normal_load, friction_coeff, &cfg, false);
        assert!(f0.abs() < 1e-4);

        // Small slip angle generates restoring lateral force
        let f_small = pacejka_lateral_force(0.1, normal_load, friction_coeff, &cfg, false);
        assert!(f_small > 0.0);

        // Negative slip angle produces negative lateral force
        let f_neg = pacejka_lateral_force(-0.1, normal_load, friction_coeff, &cfg, false);
        assert!(f_neg < 0.0);
        assert!((f_small + f_neg).abs() < 1e-3);

        // Handbrake reduces lateral grip
        let f_hb = pacejka_lateral_force(0.1, normal_load, friction_coeff, &cfg, true);
        assert!(f_hb < f_small);
    }

    #[test]
    fn test_friction_circle_clamping() {
        let max_f = 2000.0;
        let (fx, fy) = solve_combined_slip_forces(3000.0, 4000.0, max_f);
        let total = (fx * fx + fy * fy).sqrt();
        assert!((total - max_f).abs() < 1e-2);
        assert!(fx > 0.0 && fy > 0.0);

        // Within circle returns unchanged
        let (fx_in, fy_in) = solve_combined_slip_forces(500.0, 500.0, max_f);
        assert_eq!(fx_in, 500.0);
        assert_eq!(fy_in, 500.0);
    }
}
