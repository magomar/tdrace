use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// NASCAR Cup Series & Trans-Am TA1 Game Module.
pub struct NascarGameModule;

impl NascarGameModule {
    pub fn new() -> Self {
        Self
    }

    /// 850 BHP NASCAR Cup Next-Gen Pushrod V8 Spec
    pub fn car_stock_car() -> CarConfig {
        CarConfig::stock_car_ta1()
    }

    /// 850 BHP Trans-Am TA1 Spaceframe V8 Road Course Spec
    pub fn car_trans_am() -> CarConfig {
        CarConfig::trans_am_ta1()
    }
}

impl Default for NascarGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for NascarGameModule {
    fn id(&self) -> &'static str {
        "nascar"
    }

    fn title(&self) -> &'static str {
        "STOCK CAR CUP & TA1 SILHOUETTE"
    }

    fn subtitle(&self) -> &'static str {
        "850 BHP V8 Stock Cars, Pack Drafting, 320 km/h Superspeedways & Silhouette Road Courses"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.82, 0.08, 1.0), // Golden Yellow
            secondary_accent: Color::new(0.06, 0.42, 0.92, 1.0), // Daytona Blue
            header_badge: "STOCK CAR CUP SERIES",
            background_tint: Color::new(0.08, 0.07, 0.06, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "nascar_cup_v8",
                name: "Stock Car Cup Next-Gen V8",
                tag: "850 BHP PUSHROD V8",
                description: "Modern Stock Car Cup Series stock car: 850 BHP naturally aspirated 5.9L pushrod V8, 1260 kg, decklid blade ducktail spoiler, roof escape flaps, 320 km/h superspeedway package.",
                config: Self::car_stock_car(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: false,
                    roof_fins: true,
                    window_net: true,
                },
                stats: (0.97, 0.90, 0.86, 0.88),
                default_schemes: vec![
                    CarColorScheme::stock_car_daytona_blue(),
                    CarColorScheme::stock_car_racing_red(),
                    CarColorScheme::stock_car_sunset_orange(),
                    CarColorScheme::stock_car_intimidator_black(),
                    CarColorScheme::stock_car_carolina_blue(),
                ],
                audio_profile: Some(EngineAudioProfile::nascar_v8_pushrod()),
            },
            VehicleModelDefinition {
                id: "trans_am_ta1",
                name: "TA1 Silhouette Spaceframe V8",
                tag: "850 BHP SPACEFRAME",
                description: "Pure American road racing silhouette monster: tube-frame chassis, high-mount carbon GT wing, lightweight bodywork, quick-ratio steering, side boom tubes.",
                config: Self::car_trans_am(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: true,
                    roof_fins: false,
                    window_net: true,
                },
                stats: (0.95, 0.92, 0.91, 0.85),
                default_schemes: vec![
                    CarColorScheme::stock_car_intimidator_black(),
                    CarColorScheme::stock_car_sunset_orange(),
                    CarColorScheme::stock_car_racing_red(),
                    CarColorScheme::stock_car_daytona_blue(),
                ],
                audio_profile: Some(EngineAudioProfile::nascar_v8_pushrod()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "nascar_cup_v8"
    }

    fn default_off_track_surface(&self) -> SurfaceType {
        SurfaceType::Grass
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("nascar")
    }

    fn default_track_id(&self) -> &'static str {
        "daytona_superspeedway"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "colt_reynolds",
                name: "Colt 'The Ironclad' Reynolds",
                alias: "The Ironclad",
                bio: "Feared black #3 stock car legend. Master of the aggressive bumper tap, high-speed drafting lock, and relentless high-line intimidation.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_intimidator_black(),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, -0.02, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: COLT_REYNOLDS_FAVORITES,
            },
            DriverCharacter {
                id: "clayton_reed",
                name: "Clayton 'Viper' Reed",
                alias: "Viper Flash",
                bio: "Precision road course virtuoso and aerodynamic drafting master. Lethal slingshot overtakes at Watkins Glen and Daytona.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_racing_red(),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.01, 0.00, 0.02, -0.3, 0.02),
                style: DrivingStyle::Calculating,
                favorite_cars: CLAYTON_REED_FAVORITES,
            },
            DriverCharacter {
                id: "rex_montgomery",
                name: "Rex 'The Crown' Montgomery",
                alias: "The Crown",
                bio: "The 200-win patriarch of American stock car racing. Runs the famous Carolina Petty Blue #43 with unmatched pack drafting defense.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_carolina_blue(),
                offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, 0.00, 0.04, -0.3, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: REX_MONTGOMERY_FAVORITES,
            },
            DriverCharacter {
                id: "brant_harlan",
                name: "Brant 'Thunder' Harlan",
                alias: "Thunder",
                bio: "Raw aggression and fearless dive-bombs into turn 1. Thrives in chaotic multicar packs and high-banked superspeedway shootouts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_sunset_orange(),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.01, -0.03, 0.02, -0.4, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: BRANT_HARLAN_FAVORITES,
            },
            DriverCharacter {
                id: "judson_vance",
                name: "Judson 'Gold Rush' Vance",
                alias: "Gold Rush",
                bio: "Multi-time Cup champion renowned for surgical consistency, tire management, and late-race charges on superspeedway restarts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_daytona_blue(),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, 0.01, 0.00, 0.0, 0.01),
                style: DrivingStyle::Calculating,
                favorite_cars: JUDSON_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "tanner_cobb",
                name: "Tanner 'Hot Lap' Cobb",
                alias: "Hot Lap",
                bio: "Dirt track and short-track brawler. Fearless high-line slider who thrives under the lights at Bristol Motor Speedway.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.20, 0.20, 0.22, 1.0),
                    Color::new(0.98, 0.50, 0.05, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: TANNER_COBB_FAVORITES,
            },
            DriverCharacter {
                id: "bo_mercer",
                name: "Bo 'Heartland' Mercer",
                alias: "Heartland Gang",
                bio: "Legendary leader of the Heartland Gang. Superspeedway high-bank specialist with ice-cold nerves in 3-wide pack racing.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.85, 0.15, 0.15, 1.0),
                    Color::new(0.95, 0.85, 0.15, 1.0),
                    Color::new(0.15, 0.15, 0.15, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.00, -0.02, 0.05, -0.4, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: BO_MERCER_FAVORITES,
            },
            DriverCharacter {
                id: "beau_chambers",
                name: "Beau 'The Comet' Chambers",
                alias: "The Comet",
                bio: "Electrifying superspeedway ace and intermediate oval charger who uses draft pushes to catapult into the lead.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.55, 0.15, 0.70, 1.0),
                    Color::new(0.10, 0.85, 0.90, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, 0.00, -0.02, 0.08, -0.5, 0.01),
                style: DrivingStyle::Balanced,
                favorite_cars: BEAU_CHAMBERS_FAVORITES,
            },
            DriverCharacter {
                id: "jasper_lowe",
                name: "Jasper 'Ace' Lowe",
                alias: "Ace",
                bio: "Two-time Cup Series Champion famed for ruthless blocking maneuvers, restart mastery, and razor-sharp racecraft.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.95, 0.85, 0.05, 1.0),
                    Color::new(0.85, 0.10, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.3, 0.00, -0.04, 0.11, -0.6, 0.02),
                style: DrivingStyle::Balanced,
                favorite_cars: JASPER_LOWE_FAVORITES,
            },
            DriverCharacter {
                id: "brayden_ellis",
                name: "Brayden 'Wildcard' Ellis",
                alias: "Wildcard",
                bio: "From Dawsonville, Georgia. Holds the all-time stock car qualifying speed record at Talladega (212.809 mph).",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.92, 0.92, 0.95, 1.0),
                    Color::new(0.10, 0.30, 0.85, 1.0),
                    Color::new(0.90, 0.10, 0.10, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.02, 0.0, 0.02),
                style: DrivingStyle::Smooth,
                favorite_cars: BRAYDEN_ELLIS_FAVORITES,
            },
            DriverCharacter {
                id: "curtis_yancey",
                name: "Curtis 'The Wall' Yancey",
                alias: "The Wall",
                bio: "Tough-as-nails three-time consecutive Cup champion. Tireless high-line charger who battles wheel-to-wheel to the checkered flag.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.88, 0.45, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                    Color::new(0.10, 0.10, 0.10, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.03, -0.1, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: CURTIS_YANCEY_FAVORITES,
            },
            DriverCharacter {
                id: "russ_warnock",
                name: "Russ 'Hammer' Warnock",
                alias: "Hammer",
                bio: "Aggressive short-track and road course warrior. Legendary mastery of high-downforce braking zones and curb hops.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.12, 0.15, 0.35, 1.0),
                    Color::new(0.95, 0.75, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.01, -0.02, 0.06, -0.4, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: RUSS_WARNOCK_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "Premier Stock Car Cup Championship".to_string(),
                point_system: PointSystem::NascarCup { stage_win_bonus: true },
                track_ids: vec![
                    "daytona_superspeedway".to_string(),
                    "talladega_superspeedway".to_string(),
                    "eldora_speedway".to_string(),
                    "iowa_speedway".to_string(),
                    "indianapolis_motor_speedway".to_string(),
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "martinsville_speedway".to_string(),
                    "road_america".to_string(),
                    "chicago_street_course".to_string(),
                    "watkins_glen_nascar".to_string(),
                ],
                laps_per_round: 4,
            },
            TournamentFormat::EliminationCup {
                elimination_interval: 3,
            },
            TournamentFormat::Championship {
                name: "Trans-National TA1 National Challenge".to_string(),
                point_system: PointSystem::NascarCup { stage_win_bonus: false },
                track_ids: vec![
                    "watkins_glen_nascar".to_string(),
                    "road_america".to_string(),
                    "chicago_street_course".to_string(),
                    "indianapolis_motor_speedway".to_string(),
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "daytona_superspeedway".to_string(),
                    "talladega_superspeedway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "martinsville_speedway".to_string(),
                ],
                laps_per_round: 4,
            },
            TournamentFormat::QuickRace {
                default_laps: 3,
                default_bots: 11,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::late_model_v8()
    }
}


const COLT_REYNOLDS_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const CLAYTON_REED_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const REX_MONTGOMERY_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const BRANT_HARLAN_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const JUDSON_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const TANNER_COBB_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const BO_MERCER_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const BEAU_CHAMBERS_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const JASPER_LOWE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const BRAYDEN_ELLIS_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const CURTIS_YANCEY_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const RUSS_WARNOCK_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];
