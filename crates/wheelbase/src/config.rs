use serde::{Deserialize, Serialize};

/// Configuration for an individual wheel corner or axle assembly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelAssemblyConfig {
    /// Rolling radius of the tire under nominal load (meters).
    pub tire_radius: f32,
    /// Width of the tire contact patch (meters).
    pub tire_width: f32,
    /// Rotational polar moment of inertia (kg·m²).
    pub rotational_inertia: f32,
    /// Pacejka Magic Formula compound configuration for this wheel.
    pub tire_model: TireConfig,
    /// Brake torque distribution factor for this wheel [0.0 = none, 1.0 = full].
    pub brake_bias_factor: f32,
    /// Drive torque distribution factor from differential [0.0 = unpowered, 1.0 = spool/locked].
    pub drive_torque_factor: f32,
}

impl Default for WheelAssemblyConfig {
    fn default() -> Self {
        Self {
            tire_radius: 0.32,
            tire_width: 0.24,
            rotational_inertia: 1.25,
            tire_model: TireConfig::default(),
            brake_bias_factor: 0.25, // 25% per wheel = 50% front / 50% rear baseline
            drive_torque_factor: 0.50, // RWD: 50% per rear wheel
        }
    }
}

impl WheelAssemblyConfig {
    pub const fn new(
        tire_radius: f32,
        tire_width: f32,
        rotational_inertia: f32,
        tire_model: TireConfig,
        brake_bias_factor: f32,
        drive_torque_factor: f32,
    ) -> Self {
        Self {
            tire_radius,
            tire_width,
            rotational_inertia,
            tire_model,
            brake_bias_factor,
            drive_torque_factor,
        }
    }

    /// Computes rotational polar moment of inertia I = 0.5 * m * r^2 from wheel mass.
    pub fn from_mass(
        tire_radius: f32,
        tire_width: f32,
        tire_mass: f32,
        tire_model: TireConfig,
        brake_bias_factor: f32,
        drive_torque_factor: f32,
    ) -> Self {
        let rotational_inertia = 0.5 * tire_mass * tire_radius * tire_radius;
        Self {
            tire_radius,
            tire_width,
            rotational_inertia,
            tire_model,
            brake_bias_factor,
            drive_torque_factor,
        }
    }
}

/// Helper returning standard default wheel assemblies for all 4 corners.
pub fn default_wheel_assemblies() -> [WheelAssemblyConfig; 4] {
    [WheelAssemblyConfig::default(); 4]
}

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

    /// Anti-lock Braking System (ABS) enabled.
    /// Prevents excessive brake lockup to preserve lateral steering grip during braking.
    pub abs_enabled: bool,
    /// ABS target lateral grip retention factor [0.0 = none, 1.0 = full lateral priority].
    pub abs_slip_threshold: f32,
    /// ABS modulation strength [0.0 = disabled, 1.0 = full pressure modulation].
    pub abs_strength: f32,

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
            esc_yaw_threshold: 0.10,
            esc_strength: 0.85,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.70,
            abs_enabled: true,
            abs_slip_threshold: 0.15,
            abs_strength: 0.95,
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
            esc_yaw_threshold: 0.22,
            esc_strength: 0.50,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.60,
            abs_enabled: true,
            abs_slip_threshold: 0.20,
            abs_strength: 0.75,
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
            abs_enabled: false,
            abs_slip_threshold: 0.50,
            abs_strength: 0.0,
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

/// Vehicle-specific terrain interaction parameters (tire flotation, paddle thrust, and ice stud penetration).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainInteractionConfig {
    /// Sand flotation factor gamma_sand in [0.10, 1.00].
    /// Scales rolling resistance increase on sand: 1.0 = standard road tires, 0.30 = paddle tires / lightweight dune buggy.
    pub sand_flotation: f32,
    /// Mud flotation factor gamma_mud in [0.10, 1.00].
    /// Scales rolling resistance increase on mud: 1.0 = standard tires, 0.25 = chevron tractor paddles / mud bogger.
    pub mud_flotation: f32,
    /// Ice grip multiplier alpha_ice in [0.50, 10.00].
    /// Scales available friction on ice: 1.0 = unstudded road tire (mu=0.08), 8.125 = tungsten-studded ice racer (mu=0.65).
    pub ice_grip_multiplier: f32,
}

impl Default for TerrainInteractionConfig {
    fn default() -> Self {
        Self {
            sand_flotation: 1.0,
            mud_flotation: 1.0,
            ice_grip_multiplier: 1.0,
        }
    }
}

/// Vehicle physical dimensions, mass properties, powertrain parameters, and steering geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(from = "CarConfigRaw")]
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
    /// Caster jacking diagonal load transfer factor [0.0 = cars with differential, ~1.0-1.5 = karts with solid axle].
    #[serde(default)]
    pub caster_jacking_factor: f32,

    /// Engine braking retarding coefficient on throttle release [0.0 = none, 0.15 = strong].
    pub engine_braking_coefficient: f32,
    /// Aerodynamic downforce coefficient (0.5 * Cl * A * air_density) scaling vertical load with V^2.
    pub downforce_coefficient: f32,

    /// Tire friction and slip parameters.
    pub tire: TireConfig,
    /// Driver electronic stability and traction assistance settings.
    pub assists: DriverAssistsConfig,
    /// Terrain interaction modifiers (sand flotation, mud paddles, ice studs).
    pub terrain: TerrainInteractionConfig,
    /// Decoupled wheel assembly configurations for all 4 corners [FL, FR, RL, RR].
    #[serde(default = "default_wheel_assemblies")]
    pub wheels: [WheelAssemblyConfig; 4],
}

#[derive(Deserialize)]
struct CarConfigRaw {
    pub mass: f32,
    pub inertia: f32,
    pub wheelbase: f32,
    pub track_width: f32,
    pub cg_to_front: f32,
    pub cg_to_rear: f32,
    pub cg_height: f32,
    pub max_engine_force: f32,
    pub max_reverse_force: f32,
    pub max_brake_force: f32,
    pub handbrake_force: f32,
    pub brake_bias: f32,
    pub drive_bias: f32,
    pub top_speed_mps: f32,
    pub max_steer_angle: f32,
    pub steer_speed: f32,
    pub steer_return_speed: f32,
    pub counter_steer_assist: f32,
    pub speed_sensitive_steer_factor: f32,
    pub air_drag_coefficient: f32,
    pub lateral_drag_coefficient: f32,
    pub rolling_resistance_coefficient: f32,
    pub angular_damping: f32,
    pub weight_transfer_longitudinal: f32,
    pub weight_transfer_lateral: f32,
    #[serde(default)]
    pub caster_jacking_factor: f32,
    pub engine_braking_coefficient: f32,
    pub downforce_coefficient: f32,
    pub tire: TireConfig,
    pub assists: DriverAssistsConfig,
    pub terrain: TerrainInteractionConfig,
    #[serde(default)]
    pub wheels: Option<[WheelAssemblyConfig; 4]>,
}

impl From<CarConfigRaw> for CarConfig {
    fn from(raw: CarConfigRaw) -> Self {
        let wheels = raw.wheels.unwrap_or_else(|| {
            CarConfig::default_wheel_assemblies_for(raw.tire, raw.brake_bias, raw.drive_bias)
        });

        Self {
            mass: raw.mass,
            inertia: raw.inertia,
            wheelbase: raw.wheelbase,
            track_width: raw.track_width,
            cg_to_front: raw.cg_to_front,
            cg_to_rear: raw.cg_to_rear,
            cg_height: raw.cg_height,
            max_engine_force: raw.max_engine_force,
            max_reverse_force: raw.max_reverse_force,
            max_brake_force: raw.max_brake_force,
            handbrake_force: raw.handbrake_force,
            brake_bias: raw.brake_bias,
            drive_bias: raw.drive_bias,
            top_speed_mps: raw.top_speed_mps,
            max_steer_angle: raw.max_steer_angle,
            steer_speed: raw.steer_speed,
            steer_return_speed: raw.steer_return_speed,
            counter_steer_assist: raw.counter_steer_assist,
            speed_sensitive_steer_factor: raw.speed_sensitive_steer_factor,
            air_drag_coefficient: raw.air_drag_coefficient,
            lateral_drag_coefficient: raw.lateral_drag_coefficient,
            rolling_resistance_coefficient: raw.rolling_resistance_coefficient,
            angular_damping: raw.angular_damping,
            weight_transfer_longitudinal: raw.weight_transfer_longitudinal,
            weight_transfer_lateral: raw.weight_transfer_lateral,
            caster_jacking_factor: raw.caster_jacking_factor,
            engine_braking_coefficient: raw.engine_braking_coefficient,
            downforce_coefficient: raw.downforce_coefficient,
            tire: raw.tire,
            assists: raw.assists,
            terrain: raw.terrain,
            wheels,
        }
    }
}

impl Default for CarConfig {
    fn default() -> Self {
        Self::sports_car()
    }
}

impl CarConfig {
    /// Generates standard 4-wheel assemblies matching current tire model, brake bias, and drive bias.
    pub fn default_wheel_assemblies_for(
        tire: TireConfig,
        brake_bias: f32,
        drive_bias: f32,
    ) -> [WheelAssemblyConfig; 4] {
        let front_brake = brake_bias * 0.5;
        let rear_brake = (1.0 - brake_bias) * 0.5;
        let front_drive = drive_bias * 0.5;
        let rear_drive = (1.0 - drive_bias) * 0.5;
        [
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                tire_model: tire,
                brake_bias_factor: front_brake,
                drive_torque_factor: front_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                tire_model: tire,
                brake_bias_factor: front_brake,
                drive_torque_factor: front_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                tire_model: tire,
                brake_bias_factor: rear_brake,
                drive_torque_factor: rear_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                tire_model: tire,
                brake_bias_factor: rear_brake,
                drive_torque_factor: rear_drive,
            },
        ]
    }

    /// Standard balanced sports car tuned for GeneRally-style arcade drift racing.
    pub fn sports_car() -> Self {
        let tire = TireConfig::default();
        Self {
            mass: 1050.0,
            inertia: 1450.0,
            wheelbase: 2.40,
            track_width: 1.40,
            cg_to_front: 1.10,
            cg_to_rear: 1.30,
            cg_height: 0.35,

            max_engine_force: 6800.0,
            max_reverse_force: 4420.0,
            max_brake_force: 11500.0,
            handbrake_force: 7500.0,
            brake_bias: 0.60,
            drive_bias: 0.0, // RWD arcade feel
            top_speed_mps: 58.0, // ~208 km/h

            max_steer_angle: 0.68, // ~39 deg responsive turning lock
            steer_speed: 5.5,
            steer_return_speed: 7.0,
            counter_steer_assist: 1.3,
            speed_sensitive_steer_factor: 0.002,

            air_drag_coefficient: 0.42,
            lateral_drag_coefficient: 1.20,
            rolling_resistance_coefficient: 0.015,
            angular_damping: 160.0,

            weight_transfer_longitudinal: 1.0,
            weight_transfer_lateral: 1.0,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.12,
            downforce_coefficient: 0.65,

            tire,
            assists: DriverAssistsConfig::arcade(),
            terrain: TerrainInteractionConfig::default(),
            wheels: Self::default_wheel_assemblies_for(tire, 0.60, 0.0),
        }
    }

    /// Dedicated drift machine: aggressive rear power, loose tail, quick counter-steer.
    pub fn drift_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.mass = 980.0;
        cfg.inertia = 1250.0;
        cfg.max_engine_force = 8200.0;
        cfg.max_reverse_force = 5330.0;
        cfg.max_steer_angle = 0.78; // ~45 deg wide drift lock
        cfg.counter_steer_assist = 1.6;
        cfg.speed_sensitive_steer_factor = 0.0015;
        cfg.engine_braking_coefficient = 0.10;
        cfg.downforce_coefficient = 0.45;
        cfg.tire.drift_slide_friction = 0.92;
        cfg.tire.handbrake_lateral_friction_multiplier = 0.30;
        cfg.drive_bias = 0.0;
        cfg.assists = DriverAssistsConfig::sport();
        for w in &mut cfg.wheels {
            w.tire_model = cfg.tire;
        }
        cfg
    }

    /// Go-kart preset: ultra-responsive, lightweight, high lateral grip, direct steering.
    /// Features staggered open-wheel dimensions: narrow front tires (r=0.18m, w=0.12m)
    /// and wide rear tires (r=0.20m, w=0.21m) delivering >= 35% higher peak lateral force.
    pub fn kart() -> Self {
        let front_tire = TireConfig {
            stiffness_b: 13.5,
            shape_c: 1.50,
            peak_d: 1.35,
            curvature_e: -0.20,
            drift_slide_friction: 0.88,
            handbrake_lateral_friction_multiplier: 0.35,
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
        };
        let rear_tire = TireConfig {
            stiffness_b: 13.5,
            shape_c: 1.50,
            peak_d: 1.85, // >= 35% higher peak lateral force than front axle under equal load (1.85 >= 1.35 * 1.35)
            curvature_e: -0.20,
            drift_slide_friction: 0.88,
            handbrake_lateral_friction_multiplier: 0.35,
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
        };
        // I_front = 0.15 kg*m^2, I_rear = 0.24 kg*m^2 (I_rear > I_front)
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.18,
                tire_width: 0.12,
                rotational_inertia: 0.15,
                tire_model: front_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.18,
                tire_width: 0.12,
                rotational_inertia: 0.15,
                tire_model: front_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.20,
                tire_width: 0.21,
                rotational_inertia: 0.24,
                tire_model: rear_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.20,
                tire_width: 0.21,
                rotational_inertia: 0.24,
                tire_model: rear_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 180.0,
            inertia: 120.0,
            wheelbase: 1.05,
            track_width: 0.85,
            cg_to_front: 0.60,
            cg_to_rear: 0.45,
            cg_height: 0.18,

            max_engine_force: 2200.0,
            max_reverse_force: 1430.0,
            max_brake_force: 2400.0,
            handbrake_force: 1800.0,
            brake_bias: 0.50,
            drive_bias: 0.0,
            top_speed_mps: 32.0, // ~115 km/h

            max_steer_angle: 0.73, // ~41.8 deg direct 1:1 racing kart lock
            steer_speed: 10.5,
            steer_return_speed: 14.0,
            counter_steer_assist: 1.25,
            speed_sensitive_steer_factor: 0.0008,

            air_drag_coefficient: 0.35,
            lateral_drag_coefficient: 1.00,
            rolling_resistance_coefficient: 0.018,
            angular_damping: 35.0,

            weight_transfer_longitudinal: 0.8,
            weight_transfer_lateral: 0.8,
            caster_jacking_factor: 1.25,

            engine_braking_coefficient: 0.18,
            downforce_coefficient: 0.10,

            tire: front_tire,
            assists: DriverAssistsConfig {
                tcs_enabled: true,
                tcs_slip_threshold: 0.16,
                tcs_strength: 0.70,
                esc_enabled: false, // Pure analog chassis yaw rotation for karts
                esc_yaw_threshold: 0.40,
                esc_strength: 0.0,
                counter_steer_assist_enabled: true,
                counter_steer_assist_strength: 0.60,
                abs_enabled: true,
                abs_slip_threshold: 0.20,
                abs_strength: 0.75,
                handbrake_bypass: true,
            },
            terrain: TerrainInteractionConfig {
                sand_flotation: 1.0,
                mud_flotation: 1.0,
                ice_grip_multiplier: 0.80,
            },
            wheels,
        }
    }

    /// Alias for `kart()` representing the classic 200cc sprint kart.
    pub fn classic_kart() -> Self {
        Self::kart()
    }

    /// Rally spec: AWD traction, softened tire curve for loose surfaces, high ride height.
    pub fn rally_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.drive_bias = 0.5; // AWD
        cfg.cg_height = 0.42;
        cfg.max_engine_force = 7500.0;
        cfg.max_reverse_force = 4875.0;
        cfg.engine_braking_coefficient = 0.14;
        cfg.downforce_coefficient = 0.70;
        cfg.tire.stiffness_b = 8.0;
        cfg.tire.drift_slide_friction = 0.90;
        cfg.terrain = TerrainInteractionConfig {
            sand_flotation: 0.70,
            mud_flotation: 0.70,
            ice_grip_multiplier: 3.50,
        };
        for w in &mut cfg.wheels {
            w.tire_radius = 0.33;
            w.tire_width = 0.22;
            w.rotational_inertia = 1.30;
            w.tire_model = cfg.tire;
            w.drive_torque_factor = 0.25; // AWD 4-wheel drive distribution
            w.brake_bias_factor = 0.25;
        }
        cfg
    }

    /// 850 BHP Trans-Am TA1 / NASCAR Cup tubular spaceframe V8 stock car spec.
    ///
    /// Characterized by massive pushrod V8 power (11,800 N tractive force, ~850 BHP),
    /// heavy inertia, low-to-moderate aerodynamic downforce, and progressive rear slip
    /// that demands precise throttle modulation to avoid power-oversteer while remaining
    /// planted and controllable at high superspeedway speeds.
    pub fn stock_car_ta1() -> Self {
        let tire = TireConfig {
            stiffness_b: 11.5,
            shape_c: 1.48,
            peak_d: 1.15,
            curvature_e: -0.12,
            drift_slide_friction: 0.86,
            handbrake_lateral_friction_multiplier: 0.42,
            skid_threshold: 0.09,
            skid_full_threshold: 0.28,
        };
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                tire_model: tire,
                brake_bias_factor: 0.31,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                tire_model: tire,
                brake_bias_factor: 0.31,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                tire_model: tire,
                brake_bias_factor: 0.19,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                tire_model: tire,
                brake_bias_factor: 0.19,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 1260.0,
            inertia: 1680.0,
            wheelbase: 2.75,
            track_width: 1.85,
            cg_to_front: 1.35,
            cg_to_rear: 1.40,
            cg_height: 0.32,

            max_engine_force: 11800.0, // ~850 BHP pushrod V8
            max_reverse_force: 7670.0,
            max_brake_force: 21000.0,
            handbrake_force: 7000.0,
            brake_bias: 0.62,
            drive_bias: 0.0, // RWD
            top_speed_mps: 89.0, // ~320 km/h (~200 mph)

            max_steer_angle: 0.48, // ~27.5 deg quick-ratio stock car steering box
            steer_speed: 7.5,
            steer_return_speed: 10.0,
            counter_steer_assist: 1.35,
            speed_sensitive_steer_factor: 0.0012,

            air_drag_coefficient: 0.52,
            lateral_drag_coefficient: 1.35,
            rolling_resistance_coefficient: 0.013,
            angular_damping: 175.0,

            weight_transfer_longitudinal: 0.85,
            weight_transfer_lateral: 0.75,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.18,
            downforce_coefficient: 1.25, // Moderate downforce package

            tire,
            assists: DriverAssistsConfig::sport(),
            terrain: TerrainInteractionConfig::default(),
            wheels,
        }
    }

    /// Alias for `stock_car_ta1()` representing the 850 BHP Trans-Am TA1 spaceframe racer.
    pub fn trans_am_ta1() -> Self {
        Self::stock_car_ta1()
    }

    /// 300 BHP Sand Rail Buggy: ultralight chromoly spaceframe, pure RWD, wide stance,
    /// rear-biased weight, high-travel suspension compliance, and paddle tire grip.
    pub fn sand_rail() -> Self {
        let tire = TireConfig {
            stiffness_b: 8.2,
            shape_c: 1.35,
            peak_d: 1.12,
            curvature_e: -0.15,
            drift_slide_friction: 0.94,
            handbrake_lateral_friction_multiplier: 0.35,
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
        };
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.38,
                tire_width: 0.18,
                rotational_inertia: 1.10,
                tire_model: tire,
                brake_bias_factor: 0.275,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.38,
                tire_width: 0.18,
                rotational_inertia: 1.10,
                tire_model: tire,
                brake_bias_factor: 0.275,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.42,
                tire_width: 0.38,
                rotational_inertia: 1.85,
                tire_model: tire,
                brake_bias_factor: 0.225,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.42,
                tire_width: 0.38,
                rotational_inertia: 1.85,
                tire_model: tire,
                brake_bias_factor: 0.225,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 590.0,
            inertia: 720.0,
            wheelbase: 2.40,
            track_width: 1.75,
            cg_to_front: 1.44,
            cg_to_rear: 0.96,
            cg_height: 0.46,

            max_engine_force: 8800.0, // 300 BHP explosive power-to-weight
            max_reverse_force: 5000.0,
            max_brake_force: 11500.0,
            handbrake_force: 8200.0,
            brake_bias: 0.55,
            drive_bias: 0.0, // Pure RWD
            top_speed_mps: 55.5, // ~200 km/h

            max_steer_angle: 0.75, // ~43 deg responsive off-road lock
            steer_speed: 8.5,
            steer_return_speed: 9.5,
            counter_steer_assist: 1.55,
            speed_sensitive_steer_factor: 0.001,

            air_drag_coefficient: 0.48,
            lateral_drag_coefficient: 1.40,
            rolling_resistance_coefficient: 0.018,
            angular_damping: 140.0,

            weight_transfer_longitudinal: 1.5,
            weight_transfer_lateral: 1.3,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.14,
            downforce_coefficient: 0.35,

            tire,
            assists: DriverAssistsConfig::sport(),
            terrain: TerrainInteractionConfig {
                sand_flotation: 0.30,
                mud_flotation: 0.65,
                ice_grip_multiplier: 1.50,
            },
            wheels,
        }
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
        let stock = CarConfig::stock_car_ta1();
        let sand = CarConfig::sand_rail();

        assert_eq!(sports.drive_bias, 0.0);
        assert_eq!(rally.drive_bias, 0.5);
        assert!(drift.max_steer_angle > sports.max_steer_angle);
        assert!(kart.mass < sports.mass);

        assert_eq!(stock.drive_bias, 0.0);
        assert!(stock.max_engine_force > 10000.0);
        assert!(stock.top_speed_mps * 3.6 > 310.0);
        assert!(stock.mass > sports.mass);
        assert_eq!(CarConfig::trans_am_ta1(), stock);

        // Sand rail verification
        assert_eq!(sand.drive_bias, 0.0);
        assert_eq!(sand.mass, 590.0);
        assert_eq!(sand.max_engine_force, 8800.0);
        assert!(sand.top_speed_mps * 3.6 > 195.0);
        assert!(sand.track_width > sports.track_width);

        // All presets must have 4 populated wheels
        assert_eq!(sports.wheels.len(), 4);
        assert_eq!(drift.wheels.len(), 4);
        assert_eq!(kart.wheels.len(), 4);
        assert_eq!(rally.wheels.len(), 4);
        assert_eq!(stock.wheels.len(), 4);
        assert_eq!(sand.wheels.len(), 4);
    }

    #[test]
    fn test_staggered_kart_wheel_assemblies() {
        let kart = CarConfig::classic_kart();
        // Front narrow tires: r = 0.18, w = 0.12
        assert_eq!(kart.wheels[0].tire_radius, 0.18);
        assert_eq!(kart.wheels[0].tire_width, 0.12);
        assert_eq!(kart.wheels[1].tire_radius, 0.18);
        assert_eq!(kart.wheels[1].tire_width, 0.12);

        // Rear wide tires: r = 0.20, w = 0.21
        assert_eq!(kart.wheels[2].tire_radius, 0.20);
        assert_eq!(kart.wheels[2].tire_width, 0.21);
        assert_eq!(kart.wheels[3].tire_radius, 0.20);
        assert_eq!(kart.wheels[3].tire_width, 0.21);

        // Rear rotational inertia is greater than front: I_rear > I_front
        assert!(kart.wheels[2].rotational_inertia > kart.wheels[0].rotational_inertia);

        // Rear delivers >= 35% higher peak lateral force than front under equal load
        let f_front = kart.wheels[0].tire_model.peak_d;
        let f_rear = kart.wheels[2].tire_model.peak_d;
        assert!(f_rear >= f_front * 1.35, "f_rear ({}) should be >= 1.35 * f_front ({})", f_rear, f_front);
    }

    #[test]
    fn test_legacy_config_deserialization_backward_compatibility() {
        // Legacy JSON without `wheels` array
        let legacy_json = r#"{
            "mass": 1050.0,
            "inertia": 1450.0,
            "wheelbase": 2.40,
            "track_width": 1.40,
            "cg_to_front": 1.10,
            "cg_to_rear": 1.30,
            "cg_height": 0.35,
            "max_engine_force": 6800.0,
            "max_reverse_force": 4420.0,
            "max_brake_force": 11500.0,
            "handbrake_force": 7500.0,
            "brake_bias": 0.60,
            "drive_bias": 0.0,
            "top_speed_mps": 58.0,
            "max_steer_angle": 0.68,
            "steer_speed": 5.5,
            "steer_return_speed": 7.0,
            "counter_steer_assist": 1.3,
            "speed_sensitive_steer_factor": 0.002,
            "air_drag_coefficient": 0.42,
            "lateral_drag_coefficient": 1.20,
            "rolling_resistance_coefficient": 0.015,
            "angular_damping": 160.0,
            "weight_transfer_longitudinal": 1.0,
            "weight_transfer_lateral": 1.0,
            "engine_braking_coefficient": 0.12,
            "downforce_coefficient": 0.65,
            "tire": {
                "stiffness_b": 15.0,
                "shape_c": 1.45,
                "peak_d": 1.10,
                "curvature_e": -0.15,
                "drift_slide_friction": 0.88,
                "handbrake_lateral_friction_multiplier": 0.38,
                "skid_threshold": 0.10,
                "skid_full_threshold": 0.35
            },
            "assists": {
                "tcs_enabled": true,
                "tcs_slip_threshold": 0.18,
                "tcs_strength": 0.75,
                "esc_enabled": true,
                "esc_yaw_threshold": 0.10,
                "esc_strength": 0.85,
                "counter_steer_assist_enabled": true,
                "counter_steer_assist_strength": 0.70,
                "abs_enabled": true,
                "abs_slip_threshold": 0.15,
                "abs_strength": 0.95,
                "handbrake_bypass": true
            },
            "terrain": {
                "sand_flotation": 1.0,
                "mud_flotation": 1.0,
                "ice_grip_multiplier": 1.0
            }
        }"#;

        let deserialized: Result<CarConfig, _> = serde_json::from_str(legacy_json);
        assert!(deserialized.is_ok(), "Failed to deserialize legacy config: {:?}", deserialized.err());
        let config = deserialized.unwrap();

        // Check that all 4 wheels inherited the custom tire model (stiffness_b = 15.0, peak_d = 1.10)
        for i in 0..4 {
            assert_eq!(config.wheels[i].tire_model.stiffness_b, 15.0);
            assert_eq!(config.wheels[i].tire_model.peak_d, 1.10);
            assert_eq!(config.wheels[i].tire_radius, 0.32);
            assert_eq!(config.wheels[i].rotational_inertia, 1.25);
        }

        // Check brake bias distribution across axles (0.60 brake_bias -> 0.30 front, 0.20 rear)
        assert!((config.wheels[0].brake_bias_factor - 0.30).abs() < 1e-4);
        assert!((config.wheels[1].brake_bias_factor - 0.30).abs() < 1e-4);
        assert!((config.wheels[2].brake_bias_factor - 0.20).abs() < 1e-4);
        assert!((config.wheels[3].brake_bias_factor - 0.20).abs() < 1e-4);

        // Check drive bias (0.0 RWD -> 0.0 front, 0.50 rear)
        assert_eq!(config.wheels[0].drive_torque_factor, 0.0);
        assert_eq!(config.wheels[1].drive_torque_factor, 0.0);
        assert_eq!(config.wheels[2].drive_torque_factor, 0.50);
        assert_eq!(config.wheels[3].drive_torque_factor, 0.50);
    }
}
