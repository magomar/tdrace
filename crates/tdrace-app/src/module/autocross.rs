use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use super::{
    EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition,
    VehicleVisualType,
};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Continental Autocross Game Module.
///
/// Dedicated European dirt racing discipline featuring lightweight spaceframe buggies,
/// Cross Car sprint weapons, 550+ BHP Touring saloons, and unlimited 700+ BHP SuperBuggies
/// racing on 100% natural unpaved dirt, clay, and sand circuits.
pub struct AutocrossGameModule;

impl AutocrossGameModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AutocrossGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for AutocrossGameModule {
    fn id(&self) -> &'static str {
        "autocross"
    }

    fn title(&self) -> &'static str {
        "CONTINENTAL AUTOCROSS"
    }

    fn subtitle(&self) -> &'static str {
        "Natural Unpaved Dirt & Buggy Racing"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.45, 0.05, 1.0), // Vibrant Clay / Dirt Orange
            secondary_accent: Color::new(0.95, 0.80, 0.15, 1.0), // Dust Yellow
            header_badge: "CONTINENTAL AUTOCROSS CHAMPIONSHIP",
            background_tint: Color::new(0.08, 0.05, 0.03, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "autocross_ardennes_junior_t1",
                name: "Ardennes Cross Junior T1",
                tag: "80 BHP RWD RESTRICTED",
                description: "Cross Car Junior Academy Trophy official spec machine. Compact, agile, and momentum-focused on loose dirt.",
                config: CarConfig::sand_rail(),
                visual_type: VehicleVisualType::SandRail {
                    lightbar: false,
                    whip_antenna: false,
                    paddle_tires: false,
                },
                stats: (0.55, 0.88, 0.90, 0.50),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(2),
                ],
                audio_profile: Some(EngineAudioProfile::cross_car_motorcycle()),
            },
            VehicleModelDefinition {
                id: "autocross_petersen_superbuggy_t5",
                name: "Petersen SuperBuggy V8 T5",
                tag: "680 BHP 4WD UNLIMITED",
                description: "Premier unlimited dirt racing machine with 1:1 power-to-weight ratio and massive downforce.",
                config: CarConfig::sand_rail(),
                visual_type: VehicleVisualType::SandRail {
                    lightbar: false,
                    whip_antenna: false,
                    paddle_tires: false,
                },
                stats: (0.98, 1.00, 0.95, 0.96),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(6),
                    CarColorScheme::from_index(5),
                ],
                audio_profile: Some(EngineAudioProfile::sand_rail_boxer()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "autocross_ardennes_junior_t1"
    }

    fn default_off_track_surface(&self) -> SurfaceType {
        SurfaceType::Dirt
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("autocross")
    }

    fn default_track_id(&self) -> &'static str {
        "nova_paka_ax"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "pavel_urban",
                name: "Pavel Urban",
                alias: "The Czech Rocket",
                bio: "Multi-time European SuperBuggy champion who attacks Nova Paka's steep descents with relentless commitment.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(3),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.00, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: PAVEL_URBAN_FAVORITES,
            },
            DriverCharacter {
                id: "bodo_richter",
                name: "Bodo Richter",
                alias: "Der Meister",
                bio: "Ten-time European SuperBuggy champion whose calculated, surgical racing lines define modern dirt circuit craft.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(1),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.00, -0.01, 0.05, -0.3, 0.03),
                style: DrivingStyle::Calculating,
                favorite_cars: BODO_RICHTER_FAVORITES,
            },
            DriverCharacter {
                id: "klaus_petersen",
                name: "Klaus Petersen",
                alias: "B1600 Maestro",
                bio: "Buggy1600 European champion with phenomenal launch traction and pinpoint rotation through tight dirt hairpins.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(2),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.00, 0.01, 0.02, 0.0, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: KLAUS_PETERSEN_FAVORITES,
            },
            DriverCharacter {
                id: "diego_morales",
                name: "Diego Morales",
                alias: "El Rápido",
                bio: "Spanish Cross Car maestro whose aggressive Scandinavian flicks on Galician clay berms excite stadium crowds.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(5),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.01, -0.02, 0.06, -0.4, 0.01),
                style: DrivingStyle::Bold,
                favorite_cars: DIEGO_MORALES_FAVORITES,
            },
            DriverCharacter {
                id: "mateo_garrido",
                name: "Mateo Garrido",
                alias: "Junior Ace",
                bio: "Cross Car Junior Academy Trophy phenom with rapid reflexes and momentum-focused lines in spec junior buggies.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(4),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.00, 0.00, 0.03, -0.1, 0.02),
                style: DrivingStyle::Balanced,
                favorite_cars: MATEO_GARRIDO_FAVORITES,
            },
            DriverCharacter {
                id: "viktor_fiala",
                name: "Viktor Fiala",
                alias: "TAX Legend",
                bio: "TouringAutocross legend who wrangles 550+ BHP turbo silhouettes through deep ruts and off-camber sweepers.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(6),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.01, -0.01, 0.04, -0.2, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: VIKTOR_FIALA_FAVORITES,
            },
            DriverCharacter {
                id: "valentin_moreau",
                name: "Valentin Moreau",
                alias: "Flying Frenchman",
                bio: "French autocross virtuoso who attacks Saint-Georges and Faleyras amphitheatre bowls with full throttle commitment.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(7),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, 0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: VALENTIN_MOREAU_FAVORITES,
            },
            DriverCharacter {
                id: "jiri_nemec",
                name: "Jiří Němec",
                alias: "Apex Hunter",
                bio: "Young Czech charger specializing in high-revving 4WD buggies with devastating switchback exit speed.",
                preferred_car: crate::ui::menu::CarChoice::SandRail,
                color_scheme: CarColorScheme::from_index(8),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.00, -0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: JIRI_NEMEC_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "World SuperBuggy Series".to_string(),
                point_system: PointSystem::FiaStandard { fastest_lap_bonus: true },
                track_ids: vec![
                    "nova_paka_ax".to_string(),
                    "prerov_ax".to_string(),
                    "matschenberg_ax".to_string(),
                    "seelow_ax".to_string(),
                    "st_georges_ax".to_string(),
                    "maggiora_ax".to_string(),
                ],
                laps_per_round: 5,
            },
            TournamentFormat::QuickRace {
                default_laps: 4,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::cross_car_motorcycle()
    }
}

const PAVEL_URBAN_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 5, "autocross_bologna_superbuggy_t5"),
    DriverFavoriteCar::new("autocross", 3, "autocross_bologna_buggy1600_t3"),
];

const BODO_RICHTER_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 5, "autocross_bologna_superbuggy_t5"),
    DriverFavoriteCar::new("autocross", 3, "autocross_bologna_buggy1600_t3"),
];

const KLAUS_PETERSEN_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 3, "autocross_petersen_buggy1600_t3"),
    DriverFavoriteCar::new("autocross", 5, "autocross_petersen_superbuggy_t5"),
];

const DIEGO_MORALES_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 2, "autocross_iberian_relampago_t2"),
    DriverFavoriteCar::new("autocross", 1, "autocross_iberian_furia_t1"),
];

const MATEO_GARRIDO_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 1, "autocross_ardennes_junior_t1"),
    DriverFavoriteCar::new("autocross", 2, "autocross_ardennes_pro_t2"),
];

const VIKTOR_FIALA_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 4, "autocross_bohemia_veloce_t4"),
];

const VALENTIN_MOREAU_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 5, "autocross_petersen_superbuggy_t5"),
    DriverFavoriteCar::new("autocross", 3, "autocross_petersen_buggy1600_t3"),
];

const JIRI_NEMEC_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("autocross", 3, "autocross_bologna_buggy1600_t3"),
    DriverFavoriteCar::new("autocross", 5, "autocross_bologna_superbuggy_t5"),
];
