use serde::{Deserialize, Serialize};

/// Target performance goals derived from real-world telemetry or BoP regulations (Spec 035).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationTarget {
    pub name: String,
    pub modality: String,
    pub tier: usize,
    /// Desired low-speed turning circle diameter in meters.
    pub target_turning_diameter_m: Option<f32>,
    /// Desired peak steady-state lateral acceleration in Gs.
    pub target_peak_lat_g: f32,
    /// Desired 0-100 km/h standing acceleration time in seconds.
    pub target_0_100_time_s: Option<f32>,
    /// Target understeer gradient in rad/g (positive = progressive understeer, 0 = neutral).
    pub target_understeer_gradient: f32,
    /// Maximum acceptable body sideslip angle in degrees during stable cornering.
    pub max_sideslip_deg: f32,
}

impl CalibrationTarget {
    /// Sprint Kart (100cc/125cc) calibration benchmark: tight 2.5m circle, high lateral G, minimal sideslip.
    pub fn sprint_kart() -> Self {
        Self {
            name: "FIA Karting Sprint Regulation".to_string(),
            modality: "kart".to_string(),
            tier: 2,
            target_turning_diameter_m: Some(2.54),
            target_peak_lat_g: 2.65,
            target_0_100_time_s: Some(3.8),
            target_understeer_gradient: 0.02,
            max_sideslip_deg: 9.0,
        }
    }

    /// GT3 World Challenge calibration benchmark: 11m turning circle, 2.8g lateral grip with downforce.
    pub fn gt3_homologation() -> Self {
        Self {
            name: "SRO GT3 Homologation Window".to_string(),
            modality: "gt".to_string(),
            tier: 2,
            target_turning_diameter_m: Some(10.95),
            target_peak_lat_g: 2.85,
            target_0_100_time_s: Some(3.2),
            target_understeer_gradient: 0.05,
            max_sideslip_deg: 7.5,
        }
    }

    /// NASCAR TA1 Stock Car: heavy V8, solid spool axle, 11m turning circle, high-speed banked oval grip.
    pub fn stock_car_ta1() -> Self {
        Self {
            name: "Trans-Am TA1 Homologation Spec".to_string(),
            modality: "nascar".to_string(),
            tier: 5,
            target_turning_diameter_m: Some(11.05),
            target_peak_lat_g: 3.50,
            target_0_100_time_s: Some(3.1),
            target_understeer_gradient: 0.06,
            max_sideslip_deg: 8.0,
        }
    }

    /// Rallycross Supercar AWD: dual mechanical LSDs, tight 6.5m hairpin rotation, explosive launch.
    pub fn rallycross_supercar() -> Self {
        Self {
            name: "World RX Supercar Technical Regulations".to_string(),
            modality: "rally".to_string(),
            tier: 2,
            target_turning_diameter_m: Some(6.55),
            target_peak_lat_g: 1.65,
            target_0_100_time_s: Some(2.1),
            target_understeer_gradient: 0.01,
            max_sideslip_deg: 14.0,
        }
    }

    /// Extreme Off-Road Sand Rail Buggy: 590kg chromoly tube chassis, 300 BHP, spool rear, paddle tires.
    pub fn sand_rail_buggy() -> Self {
        Self {
            name: "Extreme Off-Road Sand Rail Regulation".to_string(),
            modality: "extreme_offroad".to_string(),
            tier: 1,
            target_turning_diameter_m: Some(7.20),
            target_peak_lat_g: 1.55,
            target_0_100_time_s: Some(3.4),
            target_understeer_gradient: 0.01,
            max_sideslip_deg: 18.0,
        }
    }
}
