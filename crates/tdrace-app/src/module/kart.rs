use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Karting World Cup Game Module
pub struct KartGameModule;

impl KartGameModule {
    pub fn new() -> Self {
        Self
    }

    pub fn car_shifter_kart() -> CarConfig {
        CarConfig::kart()
    }








}

impl Default for KartGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for KartGameModule {
    fn id(&self) -> &'static str {
        "kart"
    }

    fn title(&self) -> &'static str {
        "KARTING WORLD CUP"
    }

    fn subtitle(&self) -> &'static str {
        "Direct-Drive 125cc Shifter Karts, 3G Apex Cornering & Wheel-to-Wheel Heats"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(0.30, 0.95, 0.40, 1.0), // Electric Kart Green
            secondary_accent: Color::new(1.0, 0.85, 0.20, 1.0), // Racing Yellow
            header_badge: "KARTING WORLD CUP",
            background_tint: Color::new(0.04, 0.07, 0.05, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "shifter_kart_125",
                name: "125cc Shifter Kart Super Sprint",
                tag: "125cc 2-STROKE",
                description: "Lightweight 180kg tubular chassis, direct 1:1 steering ratio, and extreme 3.5g lateral grip.",
                config: CarConfig::kart(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: true,
                },
                stats: (0.70, 0.98, 0.98, 0.40),
                default_schemes: vec![
                    CarColorScheme::from_index(5), // Neon Kart Yellow
                    CarColorScheme::from_index(1), // Cyan / Black
                    CarColorScheme::from_index(2), // Red / White
                ],
                audio_profile: Some(EngineAudioProfile::kart_shifter_kz()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "shifter_kart_125"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("kart")
    }

    fn default_track_id(&self) -> &'static str {
        "lonato"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "marco_armani",
                name: "Marco Armani",
                alias: "Apex Predator",
                bio: "Tony Kart factory prodigy whose aggressive turn-in and late-braking moves dominate grassroots cadet and junior karting.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(1),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.3, 0.01, -0.01, 0.03, -0.3, 0.03),
                style: DrivingStyle::Aggressive,
                favorite_cars: MARCO_ARMANI_FAVORITES,
            },
            DriverCharacter {
                id: "lucas_vance",
                name: "Lucas Vance",
                alias: "The Professor",
                bio: "CRG factory master who calculates slipstream drafts and tire scrub angles to maintain momentum through every chicane.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(2),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.00, 0.00, 0.02, -0.2, 0.02),
                style: DrivingStyle::Calculating,
                favorite_cars: LUCAS_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "alex_rossi",
                name: "Alex Rossi",
                alias: "Rocket Rossi",
                bio: "High-octane Birel ART pilot celebrated for explosive launches off the grid and relentless pressure in wheel-to-wheel duels.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(3),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, 0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: ALEX_ROSSI_FAVORITES,
            },
            DriverCharacter {
                id: "sofia_lind",
                name: "Sofia Lind",
                alias: "The Metronome",
                bio: "Kosmic Racing prodigy with unmatched qualifying consistency and millimeter-perfect apex clipping.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(4),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, -0.01, 0.01, 0.02, -0.1, 0.02),
                style: DrivingStyle::Smooth,
                favorite_cars: SOFIA_LIND_FAVORITES,
            },
            DriverCharacter {
                id: "finn_korhonen",
                name: "Finn Korhonen",
                alias: "Flying Finn",
                bio: "Sodi Kart talent who carries unbelievable corner entry speed and thrives in damp and low-grip track conditions.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(5),
                offsets: DriverPersonalityOffsets::new(-0.03, 0.3, 0.00, -0.02, 0.06, -0.4, 0.02),
                style: DrivingStyle::Tenacious,
                favorite_cars: FINN_KORHONEN_FAVORITES,
            },
            DriverCharacter {
                id: "leo_dupont",
                name: "Leo Dupont",
                alias: "Le Chasseur",
                bio: "Energy Corse tactical racer who stalks rivals through hairpins and pounces on the exit with instant acceleration.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(6),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.1, 0.00, 0.00, 0.02, -0.2, 0.01),
                style: DrivingStyle::Calculating,
                favorite_cars: LEO_DUPONT_FAVORITES,
            },
            DriverCharacter {
                id: "mateo_silva",
                name: "Mateo Silva",
                alias: "El Pistolero",
                bio: "Parolin Motorsport racer known for lightning reflexes, daring outside passes, and fearless defensive positioning.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(7),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, 0.01, 0.02, -0.2, 0.01),
                style: DrivingStyle::Bold,
                favorite_cars: MATEO_SILVA_FAVORITES,
            },
            DriverCharacter {
                id: "dante_moretti",
                name: "Dante Moretti",
                alias: "Il Prodigio",
                bio: "Italian junior kart prodigy with aggressive curb-hopping lines and phenomenal tire management in endurance heats.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(8),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.3, 0.00, -0.02, 0.03, -0.3, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: DANTE_MORETTI_FAVORITES,
            },
            DriverCharacter {
                id: "marta_santos",
                name: "Marta Santos",
                alias: "Valkyrie",
                bio: "Spanish kart champion known for razor-sharp late braking, defensive placement, and fearless damp-weather pace.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.95, 0.40, 0.10, 1.0),
                    macroquad::color::Color::new(0.15, 0.15, 0.20, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.00, -0.01, 0.05, -0.3, 0.02),
                style: DrivingStyle::Tenacious,
                favorite_cars: MARTA_SANTOS_FAVORITES,
            },
            DriverCharacter {
                id: "kenzo_yamamoto",
                name: "Kenzo Yamamoto",
                alias: "Tarmac Whisperer",
                bio: "Suzuka kart circuit specialist with silky smooth steering inputs and microscopic steering scrubbing through fast sweeps.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                    macroquad::color::Color::new(0.85, 0.15, 0.15, 1.0),
                    macroquad::color::Color::new(0.10, 0.10, 0.15, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, -0.01, 0.01, 0.02, -0.1, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: KENZO_YAMAMOTO_FAVORITES,
            },
            DriverCharacter {
                id: "liam_callaghan",
                name: "Liam Callaghan",
                alias: "The Shamrock",
                bio: "Irish indoor and outdoor kart veteran who thrives in tight chicanes and high-contact hairpins with fearless positioning.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.10, 0.65, 0.25, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                    macroquad::color::Color::new(0.95, 0.60, 0.10, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.3, 0.01, -0.02, 0.08, -0.5, 0.01),
                style: DrivingStyle::Balanced,
                favorite_cars: LIAM_CALLAGHAN_FAVORITES,
            },
            DriverCharacter {
                id: "charlie_webb",
                name: "Charlie Webb",
                alias: "Pocket Rocket",
                bio: "British Superkart contender known for explosive acceleration out of slow corners and fearless draft slingshots.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.55, 0.15, 0.75, 1.0),
                    macroquad::color::Color::new(0.95, 0.85, 0.10, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, 0.00, -0.01, 0.05, -0.3, 0.01),
                style: DrivingStyle::Balanced,
                favorite_cars: CHARLIE_WEBB_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::EliminationCup {
                elimination_interval: 2,
            },
            TournamentFormat::Championship {
                name: "Karting World Cup".to_string(),
                point_system: PointSystem::ClassicArcade,
                track_ids: vec![
                    "lonato".to_string(),
                    "genk".to_string(),
                    "pfi".to_string(),
                    "sarno".to_string(),
                    "zuera".to_string(),
                    "le_mans_kart".to_string(),
                    "portimao_kart".to_string(),
                    "franciacorta".to_string(),
                    "wackersdorf".to_string(),
                    "kristianstad".to_string(),
                    "seven_laghi".to_string(),
                    "ampfing".to_string(),
                    "silverstone_national_kart".to_string(),
                    "valencia_kart".to_string(),
                    "campillos".to_string(),
                ],
                laps_per_round: 5,
            },
            TournamentFormat::QuickRace {
                default_laps: 5,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::kart_cadet_60()
    }
}

const MARCO_ARMANI_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_verde_sprout_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_verde_sprint_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_verde_apex_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_verde_pro_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_highland_hawk_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_highland_eagle_t6"),
];

const LUCAS_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_blackline_cadet_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_blackline_obsidian_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_blackline_phantom_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_blackline_renegade_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_moravia_arrow_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_moravia_falcon_t6"),
];

const ALEX_ROSSI_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_rosso_junior_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_rosso_veloce_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_rosso_modena_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_rosso_corsa_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_vanguard_spyder_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_venom_cobra_t6"),
];

const SOFIA_LIND_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_verde_sprout_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_verde_sprint_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_verde_apex_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_verde_pro_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_highland_hawk_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_highland_eagle_t6"),
];

const FINN_KORHONEN_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_blackline_cadet_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_blackline_obsidian_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_blackline_phantom_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_blackline_renegade_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_moravia_arrow_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_moravia_falcon_t6"),
];

const LEO_DUPONT_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_rosso_junior_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_rosso_veloce_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_rosso_modena_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_rosso_corsa_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_vanguard_spyder_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_venom_cobra_t6"),
];

const MATEO_SILVA_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_verde_sprout_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_verde_sprint_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_blackline_phantom_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_verde_pro_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_highland_hawk_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_highland_eagle_t6"),
];

const DANTE_MORETTI_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_rosso_junior_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_rosso_veloce_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_verde_apex_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_rosso_corsa_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_vanguard_spyder_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_venom_cobra_t6"),
];

const MARTA_SANTOS_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_blackline_cadet_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_blackline_obsidian_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_rosso_modena_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_blackline_renegade_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_moravia_arrow_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_moravia_falcon_t6"),
];

const KENZO_YAMAMOTO_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_verde_sprout_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_verde_sprint_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_verde_apex_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_verde_pro_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_highland_hawk_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_highland_eagle_t6"),
];

const LIAM_CALLAGHAN_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_rosso_junior_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_rosso_veloce_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_blackline_phantom_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_rosso_corsa_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_vanguard_spyder_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_venom_cobra_t6"),
];

const CHARLIE_WEBB_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("kart", 1, "kart_blackline_cadet_t1"),
    DriverFavoriteCar::new("kart", 2, "kart_blackline_obsidian_t2"),
    DriverFavoriteCar::new("kart", 3, "kart_rosso_modena_t3"),
    DriverFavoriteCar::new("kart", 4, "kart_blackline_renegade_t4"),
    DriverFavoriteCar::new("kart", 5, "kart_moravia_arrow_t5"),
    DriverFavoriteCar::new("kart", 6, "kart_moravia_falcon_t6"),
];
