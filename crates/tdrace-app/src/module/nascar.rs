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
        "NASCAR CUP SERIES & TRANS-AM TA1"
    }

    fn subtitle(&self) -> &'static str {
        "850 BHP V8 Stock Cars, Pack Drafting, 320 km/h Superspeedways & Trans-Am Road Courses"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.82, 0.08, 1.0), // Golden Yellow
            secondary_accent: Color::new(0.06, 0.42, 0.92, 1.0), // Daytona Blue
            header_badge: "NASCAR CUP SERIES",
            background_tint: Color::new(0.08, 0.07, 0.06, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "nascar_cup_v8",
                name: "NASCAR Cup Next-Gen V8",
                tag: "850 BHP PUSHROD V8",
                description: "Modern NASCAR Cup Series stock car: 850 BHP naturally aspirated 5.9L pushrod V8, 1260 kg, decklid blade ducktail spoiler, roof escape flaps, 320 km/h superspeedway package.",
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
                name: "Trans-Am TA1 Spaceframe V8",
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
                id: "dale_vance",
                name: "Dale 'The Intimidator' Vance",
                alias: "The Intimidator",
                bio: "Feared black #3 stock car legend. Master of the aggressive bumper tap, high-speed drafting lock, and relentless high-line intimidation.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_intimidator_black(),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.01, -0.02, 0.03, -0.2, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: DALE_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "chase_gordon",
                name: "Chase 'Rainbow' Gordon",
                alias: "Rainbow Flash",
                bio: "Precision road course virtuoso and aerodynamic drafting master. Lethal slingshot overtakes at Watkins Glen and Daytona.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_racing_red(),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.01, 0.00, 0.02, -0.3, 0.02),
                style: DrivingStyle::Calculating,
                favorite_cars: CHASE_GORDON_FAVORITES,
            },
            DriverCharacter {
                id: "richard_pettyfield",
                name: "Richard 'The King' Pettyfield",
                alias: "The King",
                bio: "The 200-win patriarch of American stock car racing. Runs the famous Carolina Petty Blue #43 with unmatched pack drafting defense.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_carolina_blue(),
                offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, 0.00, 0.04, -0.3, 0.01),
                style: DrivingStyle::Smooth,
                favorite_cars: RICHARD_PETTYFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "rowdy_busch",
                name: "Rowdy 'Wild Thing' Busch",
                alias: "Wild Thing",
                bio: "Raw aggression and fearless dive-bombs into turn 1. Thrives in chaotic multicar packs and high-banked superspeedway shootouts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_sunset_orange(),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.01, -0.03, 0.02, -0.4, 0.02),
                style: DrivingStyle::Aggressive,
                favorite_cars: ROWDY_BUSCH_FAVORITES,
            },
            DriverCharacter {
                id: "jimmie_johnson",
                name: "Jimmie 'Seven-Time' Johnson",
                alias: "Seven-Time",
                bio: "Seven-time Cup champion renowned for surgical consistency, tire management, and late-race charges on superspeedway restarts.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::stock_car_daytona_blue(),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, 0.01, 0.00, 0.0, 0.01),
                style: DrivingStyle::Calculating,
                favorite_cars: JIMMIE_JOHNSON_FAVORITES,
            },
            DriverCharacter {
                id: "tony_stewart",
                name: "Tony 'Smoke' Stewart",
                alias: "Smoke",
                bio: "Dirt track and short-track brawler. Fearless high-line slider who thrives under the lights at Bristol Motor Speedway.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.20, 0.20, 0.22, 1.0),
                    Color::new(0.98, 0.50, 0.05, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.03, -0.2, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: TONY_STEWART_FAVORITES,
            },
            DriverCharacter {
                id: "bobby_allison",
                name: "Bobby 'Alabama' Allison",
                alias: "Alabama Gang",
                bio: "Legendary leader of the Alabama Gang. Superspeedway high-bank specialist with ice-cold nerves in 3-wide pack racing.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.85, 0.15, 0.15, 1.0),
                    Color::new(0.95, 0.85, 0.15, 1.0),
                    Color::new(0.15, 0.15, 0.15, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.00, -0.02, 0.05, -0.4, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: BOBBY_ALLISON_FAVORITES,
            },
            DriverCharacter {
                id: "bubba_wallace",
                name: "Bubba 'The Rocket' Wallace",
                alias: "The Rocket",
                bio: "Electrifying superspeedway ace and intermediate oval charger who uses draft pushes to catapult into the lead.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.55, 0.15, 0.70, 1.0),
                    Color::new(0.10, 0.85, 0.90, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.2, 0.00, -0.02, 0.08, -0.5, 0.01),
                style: DrivingStyle::Balanced,
                favorite_cars: BUBBA_WALLACE_FAVORITES,
            },
            DriverCharacter {
                id: "joey_logano",
                name: "Joey 'Sliced Bread' Logano",
                alias: "Sliced Bread",
                bio: "Two-time Cup Series Champion famed for ruthless blocking maneuvers, restart mastery, and razor-sharp racecraft.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.95, 0.85, 0.05, 1.0),
                    Color::new(0.85, 0.10, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.3, 0.00, -0.04, 0.11, -0.6, 0.02),
                style: DrivingStyle::Balanced,
                favorite_cars: JOEY_LOGANO_FAVORITES,
            },
            DriverCharacter {
                id: "bill_elliott",
                name: "Bill 'Awesome Bill' Elliott",
                alias: "Awesome Bill",
                bio: "From Dawsonville, Georgia. Holds the all-time NASCAR qualifying speed record at Talladega (212.809 mph).",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.92, 0.92, 0.95, 1.0),
                    Color::new(0.10, 0.30, 0.85, 1.0),
                    Color::new(0.90, 0.10, 0.10, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.02, 0.0, 0.02),
                style: DrivingStyle::Smooth,
                favorite_cars: BILL_ELLIOTT_FAVORITES,
            },
            DriverCharacter {
                id: "cale_yarborough",
                name: "Cale 'The Iron Man' Yarborough",
                alias: "The Iron Man",
                bio: "Tough-as-nails three-time consecutive Cup champion. Tireless high-line charger who battles wheel-to-wheel to the checkered flag.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.88, 0.45, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                    Color::new(0.10, 0.10, 0.10, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.03, -0.1, 0.02),
                style: DrivingStyle::Bold,
                favorite_cars: CALE_YARBOROUGH_FAVORITES,
            },
            DriverCharacter {
                id: "rusty_wallace",
                name: "Rusty 'Thunder' Wallace",
                alias: "Thunder",
                bio: "Aggressive short-track and road course warrior. Legendary mastery of high-downforce braking zones and curb hops.",
                preferred_car: crate::ui::menu::CarChoice::StockCar,
                color_scheme: CarColorScheme::new(
                    Color::new(0.12, 0.15, 0.35, 1.0),
                    Color::new(0.95, 0.75, 0.10, 1.0),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, 0.01, -0.02, 0.06, -0.4, 0.01),
                style: DrivingStyle::Tenacious,
                favorite_cars: RUSTY_WALLACE_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "NASCAR Cup Series Championship".to_string(),
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
                name: "Trans-Am TA1 National Challenge".to_string(),
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


const DALE_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const CHASE_GORDON_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const RICHARD_PETTYFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const ROWDY_BUSCH_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const JIMMIE_JOHNSON_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const TONY_STEWART_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const BOBBY_ALLISON_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_crossbow_montego_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const BUBBA_WALLACE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_yamato_century_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_yamato_taiga_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const JOEY_LOGANO_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];

const BILL_ELLIOTT_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_forge_stallion_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_forge_stallion_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_crossbow_manta_t5"),
];

const CALE_YARBOROUGH_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_heartland_spec_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_crossbow_predator_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_crossbow_sierra_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_rampart_enforcer_t5"),
];

const RUSTY_WALLACE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("nascar", 1, "nascar_rampart_arrow_t1"),
    DriverFavoriteCar::new("nascar", 2, "nascar_crossbow_saber_t2"),
    DriverFavoriteCar::new("nascar", 3, "nascar_forge_reactor_t3"),
    DriverFavoriteCar::new("nascar", 4, "nascar_forge_ironclad_t4"),
    DriverFavoriteCar::new("nascar", 5, "nascar_forge_stallion_t5"),
];
