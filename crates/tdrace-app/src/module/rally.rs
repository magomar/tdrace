use macroquad::color::Color;
use tdrace_core::physics::config::{CarConfig, DriverAssistsConfig, TireConfig};
use tdrace_core::physics::surface::SurfaceType;
use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Rallycross World Cup Game Module
pub struct RallyGameModule;

impl RallyGameModule {
    pub fn new() -> Self {
        Self
    }

    /// WRC Rally Championship AWD Turbo Spec
    pub fn car_wrc_rally() -> CarConfig {
        let mut cfg = CarConfig::sports_car();
        cfg.mass = 1190.0;
        cfg.inertia = 1450.0;
        cfg.wheelbase = 2.45;
        cfg.track_width = 1.60;
        cfg.cg_height = 0.44; // Raised suspension for jumps and rough terrain
        cfg.drive_bias = 0.50; // AWD 50/50 torque split

        cfg.max_engine_force = 9200.0; // High torque turbo 380 BHP
        cfg.max_brake_force = 14500.0;
        cfg.handbrake_force = 9500.0;
        cfg.top_speed_mps = 64.0; // ~230 km/h

        cfg.max_steer_angle = 0.72; // Wide steering lock for Scandinavian flicks
        cfg.steer_speed = 7.5;
        cfg.steer_return_speed = 9.0;
        cfg.counter_steer_assist = 1.4;

        cfg.engine_braking_coefficient = 0.16;
        cfg.downforce_coefficient = 0.85;

        cfg.tire = TireConfig {
            grip: 1.05,
            peak_slip_angle_deg: 14.2, // Compliant tire sidewall for gravel/dirt
            slide_grip: 0.92,          // High controllable slide grip
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
            ..TireConfig::default()
        };
        cfg.assists = DriverAssistsConfig::sport();
        cfg
    }

    /// Group B Legend: 500+ BHP Lightweight Turbo Monster
    pub fn car_group_b() -> CarConfig {
        let mut cfg = Self::car_wrc_rally();
        cfg.mass = 960.0;
        cfg.inertia = 1180.0;
        cfg.max_engine_force = 12500.0;
        cfg.top_speed_mps = 75.0; // ~270 km/h
        cfg.counter_steer_assist = 1.6;
        cfg.assists = DriverAssistsConfig::raw();
        cfg
    }
}

impl Default for RallyGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for RallyGameModule {
    fn id(&self) -> &'static str {
        "rally"
    }

    fn title(&self) -> &'static str {
        "RALLYCROSS WORLD CUP"
    }

    fn subtitle(&self) -> &'static str {
        "World RX & Euro RX Mixed Surface Stages, Stadium Jumps & Drifts"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.55, 0.15, 1.0), // Dirt Rally Orange
            secondary_accent: Color::new(0.95, 0.85, 0.20, 1.0), // Desert Sand Yellow
            header_badge: "RALLYCROSS WORLD CUP",
            background_tint: Color::new(0.07, 0.06, 0.05, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "wrc_turbo_rally",
                name: "Apex WRC Turbo AWD",
                tag: "380 BHP AWD",
                description: "Modern WRC rally machine with permanent AWD, long-travel suspension, and quick anti-lag response.",
                config: Self::car_wrc_rally(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                },
                stats: (0.85, 0.92, 0.88, 0.90),
                default_schemes: vec![
                    CarColorScheme::from_index(3), // Rally Orange / White
                    CarColorScheme::from_index(1), // Subie Blue / Gold Rims
                    CarColorScheme::from_index(6), // Matte Black Stealth
                ],
                audio_profile: Some(EngineAudioProfile::rally2_turbo()),
            },
            VehicleModelDefinition {
                id: "group_b_beast",
                name: "Quattro Group B Spec",
                tag: "550 BHP TURBO",
                description: "Terrifying 1980s Group B turbo legend with explosive boost, flame-spitting anti-lag, and wild slides.",
                config: Self::car_group_b(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                },
                stats: (0.95, 0.98, 0.75, 0.98),
                default_schemes: vec![
                    CarColorScheme::from_index(2), // Historic Racing Red
                    CarColorScheme::from_index(5), // Rally Yellow / White
                ],
                audio_profile: Some(EngineAudioProfile::group_b_inline5()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "wrc_turbo_rally"
    }

    fn default_off_track_surface(&self) -> SurfaceType {
        SurfaceType::Dirt
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("rally")
    }

    fn default_track_id(&self) -> &'static str {
        "holjes_rx"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "johan_vance",
                name: "Johan Vance",
                alias: "Ice Master",
                bio: "Reigning multi-time Rallycross World Champion whose relentless joker lap strategy and surgical sliding precision dominate mixed surfaces.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(1),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, 0.01, -0.02, 0.04, -0.3, 0.03),
                style: DrivingStyle::Calculating,
                favorite_cars: JOHAN_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "mattias_storm",
                name: "Mattias Storm",
                alias: "Stormy",
                bio: "DTM and Rallycross double champion known for thunderous launches, aggressive switchbacks, and fearless dirt drifts.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(2),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, -0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: MATTIAS_STORM_FAVORITES,
            },
            DriverCharacter {
                id: "timmy_hansenfield",
                name: "Timmy Hansenfield",
                alias: "Apex Predator",
                bio: "World RX titleholder born into rallycross royalty. Renowned for surgical overtaking and momentum conservation through loose gravel.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(3),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.1, 0.00, 0.00, 0.02, -0.2, 0.02),
                style: DrivingStyle::Calculating,
                favorite_cars: TIMMY_HANSENFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "kevin_hansenfield",
                name: "Kevin Hansenfield",
                alias: "Young Gun",
                bio: "Junior European RX prodigy whose fearless high-speed Scandinavian flicks and rapid reflexes challenge the old guard.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(4),
                offsets: DriverPersonalityOffsets::new(-0.03, 0.2, 0.00, -0.02, 0.07, -0.4, 0.02),
                style: DrivingStyle::Tenacious,
                favorite_cars: KEVIN_HANSENFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "niclas_gron",
                name: "Niclas Gron",
                alias: "Flying Finn",
                bio: "Second-generation Finnish rallycross master renowned for unyielding speed on fast gravel sweeps and high-altitude jumps.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(5),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, 0.01, -0.01, 0.06, -0.4, 0.02),
                style: DrivingStyle::Balanced,
                favorite_cars: NICLAS_GRON_FAVORITES,
            },
            DriverCharacter {
                id: "anton_mark",
                name: "Anton Mark",
                alias: "The Hammer",
                bio: "Euro RX champion celebrated for ruthless defensive lines, heavy braking into hairpins, and exceptional wet tarmac pace.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(6),
                offsets: DriverPersonalityOffsets::new(-0.03, 0.1, 0.00, -0.02, 0.05, -0.5, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: ANTON_MARK_FAVORITES,
            },
            DriverCharacter {
                id: "timo_scheider",
                name: "Timo Scheider",
                alias: "The Veteran",
                bio: "Two-time touring car champion turned rallycross warrior, leveraging decades of racecraft and composure under intense pressure.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(7),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, -0.01, 0.01, 0.02, 0.0, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: TIMO_SCHEIDER_FAVORITES,
            },
            DriverCharacter {
                id: "sebastien_loebfield",
                name: "Sebastien Loebfield",
                alias: "The Maestro",
                bio: "Nine-time rally champion and rallycross titan whose unmatched car control, gravel line intuition, and poise conquer any surface.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(8),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.00, -0.02, 0.05, -0.4, 0.03),
                style: DrivingStyle::Smooth,
                favorite_cars: SEBASTIEN_LOEBFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "petter_solbergfield",
                name: "Petter Solbergfield",
                alias: "Hollywood",
                bio: "Showman and rallycross legend whose blistering sideways slides, crowd-thrilling entries, and raw pace ignite every stadium.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.08, 0.22, 0.65, 1.0),
                    Color::new(0.98, 0.85, 0.10, 1.0),
                    Color::new(0.95, 0.95, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.00, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: PETTER_SOLBERGFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "ken_blaster",
                name: "Ken Blaster",
                alias: "Gymkhana King",
                bio: "Stunt drifting maestro and rally specialist famous for smoke-filled all-wheel-drive donuts, fearless jumps, and barrier-grazing slides.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.12, 0.12, 0.14, 1.0),
                    Color::new(0.15, 0.95, 0.30, 1.0),
                    Color::new(0.10, 0.85, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.01, 0.3, 0.01, -0.01, 0.04, -0.3, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: KEN_BLASTER_FAVORITES,
            },
            DriverCharacter {
                id: "andreas_bakkerud",
                name: "Andreas Bakkerud",
                alias: "Baby Blue",
                bio: "Norwegian rallycross powerhouse with aggressive apex-hugging lines, explosive launches, and legendary Scandinavian flick entries.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.20, 0.65, 0.95, 1.0),
                    Color::new(0.98, 0.98, 0.98, 1.0),
                    Color::new(0.15, 0.15, 0.20, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, -0.01, 0.02, -0.1, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: ANDREAS_BAKKERUD_FAVORITES,
            },
            DriverCharacter {
                id: "reinis_nitissfield",
                name: "Reinis Nitissfield",
                alias: "Baltic Bullet",
                bio: "The youngest European RX champion in history, famed for cold-blooded calculated overtakes and razor-sharp joker lap execution.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.65, 0.10, 0.20, 1.0),
                    Color::new(0.95, 0.95, 0.95, 1.0),
                    Color::new(0.85, 0.70, 0.15, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.01, -0.01, 0.06, -0.4, 0.02),
                style: DrivingStyle::Balanced,
                favorite_cars: REINIS_NITISSFIELD_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::StageRally {
                name: "World RX Tour".to_string(),
                stage_track_ids: vec![
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "estering_rx".to_string(),
                    "montalegre_rx".to_string(),
                    "nyirad_rx".to_string(),
                    "kouvola_rx".to_string(),
                    "catalunya_rx".to_string(),
                    "mettet_rx".to_string(),
                    "silverstone_rx".to_string(),
                    "riga_rx".to_string(),
                    "killarney_rx".to_string(),
                    "yas_marina_rx".to_string(),
                    "essay_rx".to_string(),
                ],
            },
            TournamentFormat::Championship {
                name: "Rallycross World Cup".to_string(),
                point_system: PointSystem::FiaStandard { fastest_lap_bonus: false },
                track_ids: vec![
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "estering_rx".to_string(),
                    "montalegre_rx".to_string(),
                    "nyirad_rx".to_string(),
                    "kouvola_rx".to_string(),
                    "catalunya_rx".to_string(),
                    "mettet_rx".to_string(),
                    "silverstone_rx".to_string(),
                    "riga_rx".to_string(),
                    "killarney_rx".to_string(),
                    "yas_marina_rx".to_string(),
                    "essay_rx".to_string(),
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
        EngineAudioProfile::cross_car_motorcycle()
    }
}

const JOHAN_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_polo_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_omse_fc1x"),
];

const MATTIAS_STORM_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_polo_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_dodge_hornet_fc1x"),
];

const TIMMY_HANSENFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_peugeot_208_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_vsc_fc1x"),
];

const KEVIN_HANSENFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_peugeot_208_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_vsc_fc1x"),
];

const NICLAS_GRON_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_lancia_delta_evo_e_rx"),
    DriverFavoriteCar::new("rally", 5, "rally_omse_fc1x"),
];

const ANTON_MARK_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_polo_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_dodge_hornet_fc1x"),
];

const TIMO_SCHEIDER_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_lancia_delta_evo_e_rx"),
    DriverFavoriteCar::new("rally", 5, "rally_omse_fc1x"),
];

const SEBASTIEN_LOEBFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_lancia_delta_evo_e_rx"),
    DriverFavoriteCar::new("rally", 5, "rally_vsc_fc1x"),
];

const PETTER_SOLBERGFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_peugeot_208_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_dodge_hornet_fc1x"),
];

const KEN_BLASTER_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_polo_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_omse_fc1x"),
];

const ANDREAS_BAKKERUD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_peugeot_208_rx1e"),
    DriverFavoriteCar::new("rally", 5, "rally_vsc_fc1x"),
];

const REINIS_NITISSFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_lancia_delta_evo_e_rx"),
    DriverFavoriteCar::new("rally", 5, "rally_dodge_hornet_fc1x"),
];
