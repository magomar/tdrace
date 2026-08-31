pub mod classic;
pub mod f1;
pub mod kart;
pub mod rally;

use macroquad::color::Color;
use serde::{Deserialize, Serialize};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::Track;

use crate::ai::DriverCharacter;
use crate::render::color::CarColorScheme;
use crate::tournament::TournamentFormat;

/// Visual rendering archetype for a vehicle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VehicleVisualType {
    /// Formula 1 / Open-Wheel: exposed front/rear wings, sidepods, halo, suspension wishbones, open tires.
    OpenWheel {
        front_wing_span: f32,
        rear_wing_height: f32,
        halo: bool,
    },
    /// GT / Touring / Sports Coupe: enclosed widebody chassis, aerodynamic front splitter, GT wing, canopy.
    TouringGT {
        widebody: bool,
        gt_wing: bool,
        diffuser: bool,
    },
    /// WRC Rally: compact hatchback/sedan, roof air scoop, wide mudflaps, large rally spoiler.
    RallyHatch {
        roof_scoop: bool,
        mudflaps: bool,
        large_wing: bool,
    },
    /// Sprint Go-Kart: ultra-low tubular chassis, side pod impact bars, exposed driver body & steering column.
    GoKart {
        exposed_driver: bool,
        side_bumpers: bool,
    },
}

impl Default for VehicleVisualType {
    fn default() -> Self {
        Self::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        }
    }
}

/// Complete vehicle definition including physics, visual model, ratings, and liveries.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleModelDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub config: CarConfig,
    pub visual_type: VehicleVisualType,
    /// Normalized performance ratings: (Speed, Acceleration, Grip, Drift/Aero) [0.0..1.0]
    pub stats: (f32, f32, f32, f32),
    pub default_schemes: Vec<CarColorScheme>,
}

/// Track catalog entry for built-in or module-specific circuits.
#[derive(Debug, Clone)]
pub struct TrackDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub default_laps: u32,
    pub generator: fn() -> Track,
}

/// Theme, branding, and color palette for a game module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModuleTheme {
    pub primary_accent: Color,
    pub secondary_accent: Color,
    pub header_badge: &'static str,
    pub background_tint: Color,
}

impl Default for ModuleTheme {
    fn default() -> Self {
        Self {
            primary_accent: Color::new(1.0, 0.82, 0.20, 1.0), // Neon Gold
            secondary_accent: Color::new(0.20, 0.85, 1.0, 1.0), // Cyan
            header_badge: "MOTORSPORT SIMULATION",
            background_tint: Color::new(0.05, 0.06, 0.09, 0.98),
        }
    }
}

use crate::audio::EngineSoundType;

/// Modular audio synthesis profile for vehicle engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineAudioProfile {
    pub sound_type: EngineSoundType,
    pub idle_rpm: f32,
    pub max_rpm: f32,
    pub base_pitch: f32,
    pub pitch_scale: f32,
    pub harmonic_ratio: f32,
    pub turbo_flutter: bool,
    pub anti_lag_pops: bool,
}

impl Default for EngineAudioProfile {
    fn default() -> Self {
        Self {
            sound_type: EngineSoundType::Generic,
            idle_rpm: 1100.0,
            max_rpm: 7500.0,
            base_pitch: 65.0,
            pitch_scale: 0.038,
            harmonic_ratio: 2.0,
            turbo_flutter: false,
            anti_lag_pops: false,
        }
    }
}

impl EngineAudioProfile {
    pub fn f1_v6_turbo_hybrid() -> Self {
        Self {
            sound_type: EngineSoundType::F1V6Turbo,
            idle_rpm: 4200.0,
            max_rpm: 15000.0,
            base_pitch: 140.0,
            pitch_scale: 0.065,
            harmonic_ratio: 3.0,
            turbo_flutter: true,
            anti_lag_pops: false,
        }
    }

    pub fn kart_2stroke() -> Self {
        Self {
            sound_type: EngineSoundType::Kart125cc,
            idle_rpm: 2500.0,
            max_rpm: 14000.0,
            base_pitch: 160.0,
            pitch_scale: 0.058,
            harmonic_ratio: 1.0,
            turbo_flutter: false,
            anti_lag_pops: false,
        }
    }

    pub fn rally_turbo_antilag() -> Self {
        Self {
            sound_type: EngineSoundType::RallyTurbo,
            idle_rpm: 1200.0,
            max_rpm: 8500.0,
            base_pitch: 75.0,
            pitch_scale: 0.042,
            harmonic_ratio: 2.5,
            turbo_flutter: true,
            anti_lag_pops: true,
        }
    }

    pub fn gt_v8() -> Self {
        Self {
            sound_type: EngineSoundType::SportGT,
            idle_rpm: 950.0,
            max_rpm: 8200.0,
            base_pitch: 55.0,
            pitch_scale: 0.035,
            harmonic_ratio: 4.0,
            turbo_flutter: false,
            anti_lag_pops: false,
        }
    }
}

/// The core `GameModule` trait. Any standalone game subproject implements this trait.
pub trait GameModule: Send + Sync + 'static {
    /// Unique identifier (e.g. "f1", "rally", "kart", "classic").
    fn id(&self) -> &'static str;
    /// Display title for UI screens and headers.
    fn title(&self) -> &'static str;
    /// Subtitle / tagline.
    fn subtitle(&self) -> &'static str;
    /// UI theme styling.
    fn theme(&self) -> ModuleTheme;

    /// Available vehicle models in this module.
    fn vehicles(&self) -> Vec<VehicleModelDefinition>;
    /// Default vehicle ID selected initially.
    fn default_vehicle_id(&self) -> &'static str;

    /// Available track presets in this module.
    fn tracks(&self) -> Vec<TrackDefinition>;
    /// Default track ID selected initially.
    fn default_track_id(&self) -> &'static str;

    /// Driver character roster with AI personalities and teams.
    fn drivers(&self) -> Vec<DriverCharacter>;

    /// Tournament modes supported by this game module.
    fn supported_game_modes(&self) -> Vec<TournamentFormat>;

    /// Engine audio synthesis profile.
    fn audio_profile(&self) -> EngineAudioProfile;

    /// Default off-track terrain surface type for this module.
    fn default_off_track_surface(&self) -> tdrace_core::physics::surface::SurfaceType {
        tdrace_core::physics::surface::SurfaceType::Grass
    }
}

pub use classic::ClassicGameModule;
pub use f1::F1GameModule;
pub use kart::KartGameModule;
pub use rally::RallyGameModule;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f1_game_module() {
        let f1 = F1GameModule::new();
        assert_eq!(f1.id(), "f1");
        assert!(!f1.title().is_empty());
        assert!(!f1.vehicles().is_empty());
        assert!(f1.vehicles().len() >= 2);
        assert!(!f1.tracks().is_empty());
        assert_eq!(f1.tracks().len(), 14); // 13 F1 circuits + 1 FIA test track
        assert_eq!(f1.drivers().len(), 7);
        assert!(!f1.supported_game_modes().is_empty());

        assert_eq!(f1.default_vehicle_id(), "f1_hybrid_26");
        assert_eq!(f1.default_off_track_surface(), tdrace_core::physics::surface::SurfaceType::Grass);

        let monza = F1GameModule::track_monza();
        assert_eq!(monza.name, "Monza Autodromo Nazionale");
        assert!(!monza.checkpoints.is_empty());

        let car = F1GameModule::car_f1_hybrid();
        assert!(car.downforce_coefficient > 3.0);
        assert!(car.top_speed_mps * 3.6 > 340.0);

        // Verify that every single F1 track definition generates a valid track with 0 validation errors
        for track_def in f1.tracks() {
            let track = (track_def.generator)();
            assert!(!track.name.is_empty(), "Track name cannot be empty for {}", track_def.id);
            assert!(track.spline.total_length() > 300.0, "Track length too short for {}", track_def.id);
            assert!(track.checkpoints.len() >= 10, "Checkpoints too few for {}", track_def.id);
            assert_eq!(track.grid_positions.len(), 20.min(track.grid_positions.len()), "Grid slots check for {}", track_def.id);

            let diagnostics = tdrace_core::track::validation::validate_track(&track);
            let errors: Vec<_> = diagnostics
                .into_iter()
                .filter(|d| d.severity == tdrace_core::track::validation::ValidationSeverity::Error)
                .collect();
            assert!(
                errors.is_empty(),
                "Track '{}' ({}) had validation errors: {:?}",
                track.name,
                track_def.id,
                errors
            );
        }
    }

    #[test]
    fn test_rally_game_module() {
        let rally = RallyGameModule::new();
        assert_eq!(rally.id(), "rally");
        assert!(!rally.title().is_empty());
        assert!(!rally.vehicles().is_empty());
        assert!(rally.vehicles().len() >= 2);
        assert_eq!(rally.tracks().len(), 8);
        assert_eq!(rally.drivers().len(), 7);
        assert_eq!(rally.default_vehicle_id(), "wrc_turbo_rally");
        assert_eq!(rally.default_off_track_surface(), tdrace_core::physics::surface::SurfaceType::Dirt);

        let rally_car = RallyGameModule::car_wrc_rally();
        assert_eq!(rally_car.drive_bias, 0.5);
    }

    #[test]
    fn test_kart_game_module() {
        let kart = KartGameModule::new();
        assert_eq!(kart.id(), "kart");
        assert!(!kart.title().is_empty());
        assert!(!kart.vehicles().is_empty());
        assert!(kart.vehicles().len() >= 1);
        assert!(!kart.tracks().is_empty());
        assert_eq!(kart.tracks().len(), 10); // 8 world-famous + 2 sprint presets
        assert_eq!(kart.drivers().len(), 7);
        assert_eq!(kart.default_vehicle_id(), "shifter_kart_125");
        assert_eq!(kart.default_off_track_surface(), tdrace_core::physics::surface::SurfaceType::Grass);

        let kart_car = KartGameModule::car_shifter_kart();
        assert!(kart_car.mass < 250.0);

        // Verify that every single Kart track definition generates a valid track with 0 validation errors
        for track_def in kart.tracks() {
            let track = (track_def.generator)();
            assert!(!track.name.is_empty(), "Track name cannot be empty for {}", track_def.id);
            assert!(track.spline.total_length() > 200.0, "Track length too short for {}", track_def.id);
            assert!(track.checkpoints.len() >= 8, "Checkpoints too few for {}", track_def.id);
            assert!(track.grid_positions.len() >= 8, "Grid slots too few for {}", track_def.id);

            let diagnostics = tdrace_core::track::validation::validate_track(&track);
            let errors: Vec<_> = diagnostics
                .into_iter()
                .filter(|d| d.severity == tdrace_core::track::validation::ValidationSeverity::Error)
                .collect();
            assert!(
                errors.is_empty(),
                "Kart track '{}' ({}) had validation errors: {:?}",
                track.name,
                track_def.id,
                errors
            );
        }
    }

    #[test]
    fn test_classic_game_module() {
        let classic = ClassicGameModule::new();
        assert_eq!(classic.id(), "classic");
        assert!(!classic.title().is_empty());
        assert_eq!(classic.vehicles().len(), 4);
        assert_eq!(classic.tracks().len(), 7);
        assert!(!classic.drivers().is_empty());
        assert_eq!(classic.default_vehicle_id(), "sports_car");
        assert_eq!(classic.default_off_track_surface(), tdrace_core::physics::surface::SurfaceType::Grass);
    }
}

