use serde::{Deserialize, Serialize};

/// Tire parameters using an adapted Pacejka Magic Formula curve tuned for arcade drifting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TireConfig {
    /// Pacejka B (Stiffness factor). Determines slope at low slip angles.
    pub stiffness_b: f32,
    /// Pacejka C (Shape factor). Controls shape and peak prominence (typically ~1.4 - 1.6).
    pub shape_c: f32,
    /// Pacejka D (Peak factor multiplier, base peak is scaled by mu * Fz).
    pub peak_d: f32,
    /// Pacejka E (Curvature factor). Controls drop-off after peak.
    pub curvature_e: f32,
    /// Friction retention ratio during high slip drift (slide friction / peak friction).
    /// Ensures controllable drifts without instant spinouts.
    pub drift_slide_friction: f32,
    /// Rear tire lateral friction multiplier when handbrake is engaged (allows rear breakout).
    pub handbrake_lateral_friction_multiplier: f32,
    /// Minimum slip angle (radians) to trigger tire squeal and skid marks.
    pub skid_threshold: f32,
    /// Slip angle (radians) corresponding to 100% skid intensity and dense tire smoke.
    pub skid_full_threshold: f32,
}

impl Default for TireConfig {
    fn default() -> Self {
        Self {
            stiffness_b: 9.5,
            shape_c: 1.45,
            peak_d: 1.0,
            curvature_e: -0.15,
            drift_slide_friction: 0.88,
            handbrake_lateral_friction_multiplier: 0.38,
            skid_threshold: 0.10,
            skid_full_threshold: 0.35,
        }
    }
}

/// Configuration for electronic driver assists (TCS, ESC, Counter-Steer Drift Recovery).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DriverAssistsConfig {
    /// Traction Control System (TCS) enabled.
    /// Prevents excessive drive wheel slip under acceleration to eliminate snap power-oversteer.
    pub tcs_enabled: bool,
    /// TCS sensitivity / slip threshold: wheel slip ratio above which drive torque is modulated.
    pub tcs_slip_threshold: f32,
    /// TCS torque reduction strength [0.0 = none, 1.0 = full cut down to grip limit].
    pub tcs_strength: f32,

    /// Electronic Stability Control (ESC) enabled.
    /// Applies corrective stabilizing yaw moment when unintended sideslip/yaw rate occurs.
    pub esc_enabled: bool,
    /// ESC yaw rate error threshold (rad/s) before intervention starts.
    pub esc_yaw_threshold: f32,
    /// ESC stabilizing damping strength.
    pub esc_strength: f32,

    /// Counter-steer / Self-aligning drift recovery assist enabled.
    /// Helps digital and gamepad players catch slides and self-align when counter-steering.
    pub counter_steer_assist_enabled: bool,
    /// Strength of self-aligning steering torque assistance.
    pub counter_steer_assist_strength: f32,

    /// Handbrake bypass: whether holding the handbrake temporarily disengages TCS and relaxes ESC
    /// so intentional handbrake power-drifts are 100% responsive and uninhibited.
    pub handbrake_bypass: bool,
}

impl Default for DriverAssistsConfig {
    fn default() -> Self {
        Self::arcade()
    }
}

impl DriverAssistsConfig {
    /// Arcade preset: full assists active for forgiving, accessible keyboard and controller driving.
    pub const fn arcade() -> Self {
        Self {
            tcs_enabled: true,
            tcs_slip_threshold: 0.18,
            tcs_strength: 0.75,
            esc_enabled: true,
            esc_yaw_threshold: 0.25,
            esc_strength: 0.65,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.55,
            handbrake_bypass: true,
        }
    }

    /// Sport preset: mild assists allowing moderate slip angles and aggressive powerslides.
    pub const fn sport() -> Self {
        Self {
            tcs_enabled: true,
            tcs_slip_threshold: 0.30,
            tcs_strength: 0.40,
            esc_enabled: true,
            esc_yaw_threshold: 0.50,
            esc_strength: 0.35,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.35,
            handbrake_bypass: true,
        }
    }

    /// Pro / Raw preset: all electronic assists completely off for pure simulation physics.
    pub const fn raw() -> Self {
        Self {
            tcs_enabled: false,
            tcs_slip_threshold: 0.50,
            tcs_strength: 0.0,
            esc_enabled: false,
            esc_yaw_threshold: 1.0,
            esc_strength: 0.0,
            counter_steer_assist_enabled: false,
            counter_steer_assist_strength: 0.0,
            handbrake_bypass: true,
        }
    }
}

/// Standard driver assist difficulty profiles of varied difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssistProfile {
    /// Arcade / Beginner: Full TCS, ESC yaw stabilization, and counter-steer assist. Easiest handling.
    Arcade,
    /// Sport / Intermediate: Mild TCS, relaxed ESC, responsive power slides and agile rotation.
    Sport,
    /// Pro / Expert: All electronic aids OFF. Pure simulation physics.
    Pro,
}

impl AssistProfile {
    pub const ALL: [Self; 3] = [Self::Arcade, Self::Sport, Self::Pro];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Arcade => "Arcade (Assists: Full)",
            Self::Sport => "Sport (Assists: Mild)",
            Self::Pro => "Pro (Assists: OFF)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Arcade => "ARCADE",
            Self::Sport => "SPORT",
            Self::Pro => "PRO",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Arcade => "Full TCS + ESC + Counter-steer. Highly stable, zero snap spinouts.",
            Self::Sport => "Mild TCS + Relaxed ESC. Allows responsive power slides & tight turns.",
            Self::Pro => "Electronic aids disabled. Pure raw vehicle physics simulation.",
        }
    }

    pub fn to_config(&self) -> DriverAssistsConfig {
        match self {
            Self::Arcade => DriverAssistsConfig::arcade(),
            Self::Sport => DriverAssistsConfig::sport(),
            Self::Pro => DriverAssistsConfig::raw(),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Arcade => Self::Sport,
            Self::Sport => Self::Pro,
            Self::Pro => Self::Arcade,
        }
    }
}

/// Vehicle physical dimensions, mass properties, powertrain parameters, and steering geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CarConfig {
    /// Total vehicle mass in kilograms.
    pub mass: f32,
    /// Yaw moment of inertia around the vertical axis in kg*m^2.
    pub inertia: f32,
    /// Wheelbase (distance from front to rear axle) in meters.
    pub wheelbase: f32,
    /// Track width (distance between left and right wheels) in meters.
    pub track_width: f32,
    /// Distance from vehicle center of gravity (CG) to front axle in meters.
    pub cg_to_front: f32,
    /// Distance from vehicle center of gravity (CG) to rear axle in meters.
    pub cg_to_rear: f32,
    /// Height of center of gravity above ground in meters (governs dynamic weight transfer).
    pub cg_height: f32,

    /// Maximum forward engine tractive force at drive wheels in Newtons.
    pub max_engine_force: f32,
    /// Maximum reverse tractive force in Newtons.
    pub max_reverse_force: f32,
    /// Maximum total braking force in Newtons.
    pub max_brake_force: f32,
    /// Additional braking force applied to rear wheels during handbrake in Newtons.
    pub handbrake_force: f32,
    /// Front braking distribution bias [0.0 = full rear, 0.5 = 50/50, 1.0 = full front].
    pub brake_bias: f32,
    /// Drive power distribution bias [0.0 = RWD, 0.5 = AWD, 1.0 = FWD].
    pub drive_bias: f32,
    /// Maximum top speed reachable in m/s (engine power tapers near top speed).
    pub top_speed_mps: f32,

    /// Maximum front wheel steering angle in radians.
    pub max_steer_angle: f32,
    /// Steering actuator rate of change in rad/s.
    pub steer_speed: f32,
    /// Steering auto-centering return rate in rad/s.
    pub steer_return_speed: f32,
    /// Steering speed multiplier when player is counter-steering during a drift.
    pub counter_steer_assist: f32,
    /// Factor reducing maximum steer lock at high vehicle speeds to stabilize fast corners.
    pub speed_sensitive_steer_factor: f32,

    /// Aerodynamic drag coefficient (0.5 * Cd * A * air_density).
    pub air_drag_coefficient: f32,
    /// Lateral aerodynamic drag coefficient.
    pub lateral_drag_coefficient: f32,
    /// Rolling resistance coefficient on standard asphalt.
    pub rolling_resistance_coefficient: f32,
    /// Yaw angular velocity damping coefficient in N*m*s/rad.
    pub angular_damping: f32,

    /// Longitudinal weight transfer scaling factor (squat & dive).
    pub weight_transfer_longitudinal: f32,
    /// Lateral weight transfer scaling factor (cornering body roll).
    pub weight_transfer_lateral: f32,

    /// Tire friction and slip parameters.
    pub tire: TireConfig,
    /// Driver electronic stability and traction assistance settings.
    pub assists: DriverAssistsConfig,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self::sports_car()
    }
}

impl CarConfig {
    /// Standard balanced sports car tuned for GeneRally-style arcade drift racing.
    pub fn sports_car() -> Self {
        Self {
            mass: 1050.0,
            inertia: 1450.0,
            wheelbase: 2.40,
            track_width: 1.40,
            cg_to_front: 1.10,
            cg_to_rear: 1.30,
            cg_height: 0.35,

            max_engine_force: 6800.0,
            max_reverse_force: 3200.0,
            max_brake_force: 11500.0,
            handbrake_force: 7500.0,
            brake_bias: 0.60,
            drive_bias: 0.0, // RWD arcade feel
            top_speed_mps: 58.0, // ~208 km/h

            max_steer_angle: 0.68, // ~39 deg responsive turning lock
            steer_speed: 5.5,
            steer_return_speed: 7.0,
            counter_steer_assist: 1.3,
            speed_sensitive_steer_factor: 0.003,

            air_drag_coefficient: 0.42,
            lateral_drag_coefficient: 1.20,
            rolling_resistance_coefficient: 0.015,
            angular_damping: 160.0,

            weight_transfer_longitudinal: 1.0,
            weight_transfer_lateral: 1.0,

            tire: TireConfig::default(),
            assists: DriverAssistsConfig::arcade(),
        }
    }

    /// Dedicated drift machine: aggressive rear power, loose tail, quick counter-steer.
    pub fn drift_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.mass = 980.0;
        cfg.inertia = 1250.0;
        cfg.max_engine_force = 8200.0;
        cfg.max_steer_angle = 0.78; // ~45 deg wide drift lock
        cfg.counter_steer_assist = 1.6;
        cfg.speed_sensitive_steer_factor = 0.002;
        cfg.tire.drift_slide_friction = 0.92;
        cfg.tire.handbrake_lateral_friction_multiplier = 0.30;
        cfg.drive_bias = 0.0;
        cfg.assists = DriverAssistsConfig::sport();
        cfg
    }

    /// Go-kart preset: ultra-responsive, lightweight, high lateral grip, direct steering.
    pub fn kart() -> Self {
        Self {
            mass: 180.0,
            inertia: 120.0,
            wheelbase: 1.05,
            track_width: 0.85,
            cg_to_front: 0.50,
            cg_to_rear: 0.55,
            cg_height: 0.18,

            max_engine_force: 2200.0,
            max_reverse_force: 800.0,
            max_brake_force: 3600.0,
            handbrake_force: 2400.0,
            brake_bias: 0.50,
            drive_bias: 0.0,
            top_speed_mps: 32.0, // ~115 km/h

            max_steer_angle: 0.58, // ~33 deg agile direct lock
            steer_speed: 9.0,
            steer_return_speed: 12.0,
            counter_steer_assist: 1.2,
            speed_sensitive_steer_factor: 0.001,

            air_drag_coefficient: 0.35,
            lateral_drag_coefficient: 1.00,
            rolling_resistance_coefficient: 0.018,
            angular_damping: 60.0,

            weight_transfer_longitudinal: 0.8,
            weight_transfer_lateral: 0.8,

            tire: TireConfig {
                stiffness_b: 12.0,
                shape_c: 1.50,
                peak_d: 1.05,
                curvature_e: -0.20,
                drift_slide_friction: 0.82,
                handbrake_lateral_friction_multiplier: 0.35,
                skid_threshold: 0.08,
                skid_full_threshold: 0.28,
            },
            assists: DriverAssistsConfig::arcade(),
        }
    }

    /// Rally spec: AWD traction, softened tire curve for loose surfaces, high ride height.
    pub fn rally_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.drive_bias = 0.5; // AWD
        cfg.cg_height = 0.42;
        cfg.max_engine_force = 7500.0;
        cfg.tire.stiffness_b = 8.0;
        cfg.tire.drift_slide_friction = 0.90;
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_presets() {
        let sports = CarConfig::sports_car();
        let drift = CarConfig::drift_car();
        let kart = CarConfig::kart();
        let rally = CarConfig::rally_car();

        assert_eq!(sports.drive_bias, 0.0);
        assert_eq!(rally.drive_bias, 0.5);
        assert!(drift.max_steer_angle > sports.max_steer_angle);
        assert!(kart.mass < sports.mass);
    }
}
