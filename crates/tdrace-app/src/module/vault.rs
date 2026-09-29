//! The Vault Game Module for Archived and Deprecated Material.
//!
//! Governed by `specs/056_vault_module_for_archived_and_deprecated_content.md`.
//! Serves as an isolated cold-storage repository for decommissioned vehicles,
//! legacy test circuits, and experimental staging rulesets.

use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::DriverCharacter;
use crate::render::color::CarColorScheme;
use crate::tournament::TournamentFormat;

/// The Vault Game Module: Quarantined archive for decommissioned and temporal assets.
pub struct VaultGameModule;

impl VaultGameModule {
    pub fn new() -> Self {
        Self
    }

    /// 28 BHP Prototype Racing Lawnmower V1: narrow track, high CG, lift-off curb hopping.
    pub fn car_vault_prototype_mower() -> CarConfig {
        let mut cfg = CarConfig::kart();
        cfg.mass = 210.0;
        cfg.max_engine_force = 1800.0;
        cfg.top_speed_mps = 25.0; // ~90 km/h
        cfg.max_steer_angle = 0.58;
        cfg.steer_speed = 9.0;
        cfg.steer_return_speed = 12.0;
        cfg.tire.slide_grip = 0.88;
        cfg.tire.peak_slip_angle_deg = 8.5;
        cfg.finalized()
    }

    /// 18 BHP Slick Drift Trike 150cc: low-friction rear slide rings for continuous drifting.
    pub fn car_vault_drift_trike() -> CarConfig {
        let mut cfg = CarConfig::kart();
        cfg.mass = 120.0;
        cfg.max_engine_force = 1400.0;
        cfg.top_speed_mps = 22.0; // ~79 km/h
        cfg.max_steer_angle = 0.72;
        cfg.steer_speed = 12.0;
        cfg.steer_return_speed = 15.0;
        cfg.tire.slide_grip = 0.75;
        cfg.tire.peak_slip_angle_deg = 5.5;
        cfg.finalized()
    }

    /// 400 BHP Modular Development Test Mule: neutral 50:50 weight distribution test bench.
    pub fn car_vault_test_mule() -> CarConfig {
        let mut cfg = CarConfig::sports_car();
        cfg.mass = 1100.0;
        cfg.max_engine_force = 6500.0;
        cfg.top_speed_mps = 60.0; // ~216 km/h
        cfg.max_brake_force = 12000.0;
        cfg.downforce_coefficient = 1.0;
        cfg.steer_speed = 8.0;
        cfg.steer_return_speed = 10.0;
        cfg.drive_bias = 0.5;
        cfg.tire.slide_grip = 0.92;
        cfg.tire.peak_slip_angle_deg = 9.0;
        cfg.finalized()
    }
}

impl Default for VaultGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for VaultGameModule {
    fn id(&self) -> &'static str {
        "vault"
    }

    fn title(&self) -> &'static str {
        "THE VAULT"
    }

    fn subtitle(&self) -> &'static str {
        "Decommissioned chassis, legacy test circuits & experimental staging material"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.65, 0.0, 1.0), // Amber Caution / Cold Storage
            secondary_accent: Color::new(0.60, 0.65, 0.75, 1.0), // Slate Steel
            header_badge: "COLD STORAGE / DEV ARCHIVE",
            background_tint: Color::new(0.04, 0.04, 0.06, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "vault_test_mule",
                name: "Modular Development Mule",
                tag: "ENGINEERING TEST CHASSIS",
                description: "Neutral 50:50 test bench chassis for physics sensor calibration and benchmarking.",
                config: Self::car_vault_test_mule(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: false,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.80, 0.85, 0.85, 0.80),
                default_schemes: vec![
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(1),
                ],
                audio_profile: Some(EngineAudioProfile::default()),
            },
            VehicleModelDefinition {
                id: "vault_prototype_mower",
                name: "Prototype Racing Lawnmower V1",
                tag: "DECOMMISSIONED NOVELTY",
                description: "Experimental narrow-track lawn tractor with high center of gravity and bumpy curb hopping.",
                config: Self::car_vault_prototype_mower(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.45, 0.60, 0.70, 0.50),
                default_schemes: vec![
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(3),
                ],
                audio_profile: Some(EngineAudioProfile::racing_mower_v2()),
            },
            VehicleModelDefinition {
                id: "vault_drift_trike",
                name: "Slick Drift Trike 150cc",
                tag: "PROTOTYPE SLIDE VEHICLE",
                description: "Low-friction rear slide rings for effortless continuous pendulum drifting and 360 entries.",
                config: Self::car_vault_drift_trike(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.40, 0.70, 0.50, 0.99),
                default_schemes: vec![
                    CarColorScheme::from_index(4),
                    CarColorScheme::from_index(5),
                ],
                audio_profile: Some(EngineAudioProfile::kart_cadet_60()),
            },
            VehicleModelDefinition {
                id: "kart_honda_mean_mower",
                name: "Honda Mean Mower V2 Tuned",
                tag: "DECOMMISSIONED MOWER",
                description: "Guinness World Record 999cc CBR1000RR powered monster transferred from Karting.",
                config: Self::car_vault_prototype_mower(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.78, 0.88, 0.70, 0.92),
                default_schemes: vec![
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(0),
                ],
                audio_profile: Some(EngineAudioProfile::racing_mower_v2()),
            },
            VehicleModelDefinition {
                id: "kart_john_deere_racing_mower",
                name: "John Deere Spec Racing Mower",
                tag: "DECOMMISSIONED MOWER",
                description: "Classic green & yellow 850cc V-Twin racing mower transferred from Karting.",
                config: Self::car_vault_prototype_mower(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.76, 0.86, 0.70, 0.93),
                default_schemes: vec![
                    CarColorScheme::from_index(5),
                    CarColorScheme::from_index(1),
                ],
                audio_profile: Some(EngineAudioProfile::racing_mower_v2()),
            },
            VehicleModelDefinition {
                id: "kart_viking_t6_tractor",
                name: "Viking T6 Racing Tractor",
                tag: "DECOMMISSIONED TRACTOR",
                description: "Austrian modified 1000cc V-Twin racing tractor transferred from Karting.",
                config: Self::car_vault_prototype_mower(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.79, 0.88, 0.71, 0.92),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(2),
                ],
                audio_profile: Some(EngineAudioProfile::racing_mower_v2()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "vault_test_mule"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("vault")
    }

    fn default_track_id(&self) -> &'static str {
        ""
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::QuickRace {
                default_laps: 3,
                default_bots: 0,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_game_module() {
        let vault = VaultGameModule::new();
        assert_eq!(vault.id(), "vault");
        assert_eq!(vault.title(), "THE VAULT");
        assert_eq!(vault.default_vehicle_id(), "vault_test_mule");
        assert_eq!(vault.vehicles().len(), 6);
        assert_eq!(vault.theme().header_badge, "COLD STORAGE / DEV ARCHIVE");
        assert_eq!(vault.theme().primary_accent, Color::new(1.0, 0.65, 0.0, 1.0));
        assert!(!vault.supported_game_modes().is_empty());
    }

    #[test]
    fn test_vault_vehicle_configs() {
        let mule = VaultGameModule::car_vault_test_mule();
        assert_eq!(mule.mass, 1100.0);
        assert_eq!(mule.drive_bias, 0.5);

        let mower = VaultGameModule::car_vault_prototype_mower();
        assert_eq!(mower.mass, 210.0);

        let trike = VaultGameModule::car_vault_drift_trike();
        assert_eq!(trike.mass, 120.0);
        assert_eq!(trike.tire.slide_grip, 0.75);
    }
}
