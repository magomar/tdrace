use macroquad::color::Color;
use tdrace_core::physics::config::{
    CarConfig, ChassisSkeleton, DifferentialType, DriverAssistsConfig, EnginePlacement,
    SuspensionConfig, TerrainInteractionConfig, TireConfig,
};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DrivingStyle};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// GT World Challenge Game Module
pub struct GtWorldChallengeModule;

impl GtWorldChallengeModule {
    pub fn new() -> Self {
        Self
    }




















    /// FIA GT3 Competition Vehicle Spec (paradigmatic endurance benchmark)
    pub fn car_gt3_evo() -> CarConfig {
        CarConfig {
            mass: 1260.0,
            inertia: 1550.0,
            wheelbase: 2.70,
            track_width: 1.95,
            cg_to_front: 1.30,
            cg_to_rear: 1.40,
            cg_height: 0.28,

            max_engine_force: 9500.0, // ~600 BHP FIA GT3 naturally aspirated / biturbo
            max_reverse_force: 6175.0,
            max_brake_force: 22000.0,
            handbrake_force: 7000.0,
            brake_bias: 0.65,
            drive_bias: 0.0, // RWD
            front_differential: DifferentialType::Open,
            rear_differential: DifferentialType::LimitedSlip {
                power_lock: 0.70,
                coast_lock: 0.50,
                preload_nm: 100.0,
            },
            top_speed_mps: 82.5, // ~297 km/h

            max_steer_angle: 0.50, // ~28.6 deg responsive GT rack
            steer_speed: 9.0,
            steer_return_speed: 12.0,
            counter_steer_assist: 1.15,

            air_drag_coefficient: 0.65,
            lateral_drag_coefficient: 1.40,
            rolling_resistance_coefficient: 0.014,
            angular_damping: 180.0,

            roll_balance: 0.50,
            weight_transfer_hz: 4.0,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.22,
            engine_brake_front_share: 0.35,
            downforce_coefficient: 2.10, // Strong GT3 aerodynamic package

            tire: TireConfig {
                grip: 1.20,
                peak_slip_angle_deg: 8.0,
                slide_grip: 0.82,
                skid_threshold: 0.08,
                skid_full_threshold: 0.24,
                ..TireConfig::default()
            },
            rear_axle: Default::default(),
            assists: DriverAssistsConfig::sport(),
            terrain: TerrainInteractionConfig::default(),
            player: Default::default(),
            wheels: CarConfig::default_wheel_assemblies_for(
                TireConfig {
                    grip: 1.20,
                    peak_slip_angle_deg: 8.0,
                    slide_grip: 0.82,
                    skid_threshold: 0.08,
                    skid_full_threshold: 0.24,
                    ..TireConfig::default()
                },
                0.65,
                0.0,
            ),
            chassis: ChassisSkeleton::new(0.88, 1.18, 2.04, -0.40, 0.30, 0.65, 0.70, 0.05),
            suspension: SuspensionConfig::double_wishbone(),
            engine_placement: EnginePlacement::RearEngine,
        }
        .finalized()
    }

    /// SRO GT2 Biturbo Sprint Spec (raw straight-line missile)
    pub fn car_gt2_biturbo() -> CarConfig {
        let mut cfg = Self::car_gt3_evo();
        cfg.mass = 1390.0;
        cfg.inertia = 1750.0;
        cfg.max_engine_force = 11200.0; // ~707 BHP SRO GT2 Biturbo V8
        cfg.max_reverse_force = 7280.0;
        cfg.top_speed_mps = 91.0; // ~328 km/h
        cfg.downforce_coefficient = 1.40; // Lower downforce than GT3
        cfg.air_drag_coefficient = 0.58;
        cfg
    }

    /// GT4 Clubsport Spec: 420 BHP, RWD, lightweight, low aero (Cl=0.85)
    pub fn car_gt4_clubsport() -> CarConfig {
        let mut cfg = Self::car_gt3_evo();
        cfg.engine_placement = EnginePlacement::FrontEngine;
        cfg.mass = 1320.0;
        cfg.inertia = 1620.0;
        cfg.suspension = SuspensionConfig::macpherson_strut();
        cfg.max_engine_force = 7200.0; // ~420 BHP GT4
        cfg.max_reverse_force = 4680.0;
        cfg.max_brake_force = 18000.0;
        cfg.top_speed_mps = 75.5; // ~272 km/h
        cfg.downforce_coefficient = 0.85; // Low aero downforce
        cfg.air_drag_coefficient = 0.52;
        cfg.assists = DriverAssistsConfig::arcade();
        cfg
    }

    /// 90s Le Mans GT1 Legend Spec: 650 BHP, raw RWD, twin-turbo, high aero (Cl=2.60), analog zero assists
    pub fn car_gt1_legend() -> CarConfig {
        let mut cfg = Self::car_gt3_evo();
        cfg.engine_placement = EnginePlacement::MidEngine;
        cfg.mass = 1120.0;
        cfg.inertia = 1380.0;
        cfg.max_engine_force = 10400.0; // ~650 BHP GT1 Twin-Turbo
        cfg.max_reverse_force = 6760.0;
        cfg.max_brake_force = 24000.0;
        cfg.top_speed_mps = 93.0; // ~335 km/h
        cfg.downforce_coefficient = 2.60; // High Le Mans GT1 wing aero
        cfg.air_drag_coefficient = 0.60;
        cfg.assists = DriverAssistsConfig::raw(); // Pure analog, zero electronic assists
        cfg
    }

    /// LMH & LMDh Hypercar Prototype Spec: 800 BHP, hybrid deploy, ground-effect tunnels (Cl=3.10)
    pub fn car_hypercar_prototype() -> CarConfig {
        let mut cfg = Self::car_gt3_evo();
        cfg.engine_placement = EnginePlacement::MidEngine;
        cfg.mass = 1030.0;
        cfg.inertia = 1250.0;
        cfg.wheelbase = 3.15;
        cfg.track_width = 2.00;
        cfg.suspension = SuspensionConfig::pushrod_inboard();
        cfg.max_engine_force = 12200.0; // ~800 BHP LMH Hybrid
        cfg.max_reverse_force = 7930.0;
        cfg.max_brake_force = 26000.0;
        cfg.top_speed_mps = 95.0; // ~342 km/h
        cfg.downforce_coefficient = 3.10; // Extreme ground-effect aero tunnels
        cfg.air_drag_coefficient = 0.68;
        cfg.assists = DriverAssistsConfig::sport();
        cfg
    }
}

impl Default for GtWorldChallengeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for GtWorldChallengeModule {
    fn id(&self) -> &'static str {
        "gt"
    }

    fn title(&self) -> &'static str {
        "GT WORLD CHALLENGE"
    }

    fn subtitle(&self) -> &'static str {
        "Global Multi-Class GT3 & GT2 Endurance and Sprint Championship"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(0.95, 0.25, 0.05, 1.0), // GT Championship Red-Orange
            secondary_accent: Color::new(0.95, 0.75, 0.20, 1.0), // GT Gold/Platinum
            header_badge: "GT WORLD CHALLENGE MOTORSPORT",
            background_tint: Color::new(0.04, 0.05, 0.08, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "gt4_clubsport",
                name: "420 BHP GT4 Clubsport",
                tag: "GT4 ENTRY SPEC",
                description: "Agile 420 BHP lightweight RWD racer, agile cornering, gentle aero (Cl=0.85).",
                config: Self::car_gt4_clubsport(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: false,
                    gt_wing: true,
                    diffuser: false,
                },
                stats: (0.78, 0.82, 0.85, 0.70),
                default_schemes: vec![
                    CarColorScheme::from_index(2), // Rosso Corsa GT
                    CarColorScheme::from_index(4), // Sapphire Racing Blue
                    CarColorScheme::from_index(3), // Sunburst Orange
                    CarColorScheme::from_index(0), // Gunmetal Platinum
                ],
                audio_profile: Some(EngineAudioProfile::gt4_clubsport()),
            },
            VehicleModelDefinition {
                id: "gt3_evo",
                name: "600 BHP GT3 Evo Racer",
                tag: "FIA GT3 SPEC",
                description: "Balanced 600 BHP RWD racer, Cl=2.1 aero downforce, carbon-ceramic brakes, ABS & TC.",
                config: Self::car_gt3_evo(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.92, 0.94, 0.95, 0.50),
                default_schemes: vec![
                    CarColorScheme::from_index(2), // Rosso Corsa GT
                    CarColorScheme::from_index(0), // Gunmetal Platinum
                    CarColorScheme::from_index(4), // Sapphire Racing Blue
                    CarColorScheme::from_index(3), // Sunburst Orange
                ],
                audio_profile: Some(EngineAudioProfile::gt3_high_rev()),
            },
            VehicleModelDefinition {
                id: "gt2_biturbo",
                name: "707 BHP GT2 Biturbo Sprint",
                tag: "SRO GT2 SPRINT",
                description: "High-power 707 BHP biturbo straight-line missile, 328 km/h top speed, lower downforce (Cl=1.4).",
                config: Self::car_gt2_biturbo(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.96, 0.97, 0.89, 0.65),
                default_schemes: vec![
                    CarColorScheme::from_index(6), // Stealth Carbon & Gold
                    CarColorScheme::from_index(1), // Electric Cyan GT
                    CarColorScheme::from_index(5), // Vivid Competition Yellow
                ],
                audio_profile: Some(EngineAudioProfile::gt2_biturbo()),
            },
            VehicleModelDefinition {
                id: "gt1_legend",
                name: "650 BHP GT1 Le Mans Legend",
                tag: "90s GT1 LEGEND",
                description: "Raw 650 BHP twin-turbo beast with high downforce (Cl=2.60) and pure analog handling (zero electronic assists).",
                config: Self::car_gt1_legend(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.98, 0.98, 0.93, 0.40),
                default_schemes: vec![
                    CarColorScheme::from_index(0), // Gunmetal & Silver
                    CarColorScheme::from_index(2), // Rosso Heritage
                    CarColorScheme::from_index(6), // Stealth Carbon
                ],
                audio_profile: Some(EngineAudioProfile::gt1_v12_analogue()),
            },
            VehicleModelDefinition {
                id: "hypercar_prototype",
                name: "800 BHP LMH Hypercar Prototype",
                tag: "LE MANS HYPERCAR",
                description: "Cutting-edge 800 BHP hybrid prototype with ground-effect aero tunnels (Cl=3.10) and hybrid boost.",
                config: Self::car_hypercar_prototype(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.99, 0.99, 0.98, 0.35),
                default_schemes: vec![
                    CarColorScheme::from_index(2), // Factory Racing Red
                    CarColorScheme::from_index(6), // Carbon Black / Gold
                    CarColorScheme::from_index(1), // Electric Aero Cyan
                ],
                audio_profile: Some(EngineAudioProfile::hypercar_v6_hybrid()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "gt4_clubsport"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("gt")
    }

    fn default_track_id(&self) -> &'static str {
        "monza"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "max_hunter",
                name: "Max Hunter",
                alias: "The Dominator",
                bio: "4-time World Champion renowned for relentless pace, surgical overtakes, and unwavering consistency in all conditions.",
                style: DrivingStyle::Aggressive,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(6),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, -0.01, 0.02, 0.0, 0.03),
                favorite_cars: MAX_HUNTER_FAVORITES,
            },
            DriverCharacter {
                id: "charles_laurent",
                name: "Charles Laurent",
                alias: "The Qualifying King",
                bio: "Scuderia prodigy with unbelievable single-lap hot-lap qualifying pace and unmatched precision on street circuits.",
                style: DrivingStyle::Smooth,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(2),
                offsets: DriverPersonalityOffsets::new(-0.02, 0.2, -0.01, -0.02, 0.04, -0.3, 0.04),
                favorite_cars: CHARLES_LAURENT_FAVORITES,
            },
            DriverCharacter {
                id: "lewis_vance",
                name: "Lewis Vance",
                alias: "The Master",
                bio: "7-time World Champion with legendary wet-weather mastery, flawless tire preservation, and icy composure.",
                style: DrivingStyle::Smooth,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(7),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.01, 0.02, 0.0, 0.03),
                favorite_cars: LEWIS_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "fernando_toro",
                name: "Fernando Toro",
                alias: "El Matador",
                bio: "Relentless Spanish gladiator who wrestles ill-handling cars to the podium through sheer grit and racecraft.",
                style: DrivingStyle::Tenacious,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(3),
                offsets: DriverPersonalityOffsets::new(-0.04, 0.3, -0.01, -0.03, 0.08, -0.5, 0.02),
                favorite_cars: FERNANDO_TORO_FAVORITES,
            },
            DriverCharacter {
                id: "george_speed",
                name: "George Speed",
                alias: "The Silver Bullet",
                bio: "Analytical young British ace who capitalizes on strategy and executes millimeter-perfect overtakes into hairpins.",
                style: DrivingStyle::Balanced,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(1),
                offsets: DriverPersonalityOffsets::new(0.01, 0.2, 0.00, -0.02, 0.06, -0.5, 0.02),
                favorite_cars: GEORGE_SPEED_FAVORITES,
            },
            DriverCharacter {
                id: "lando_vance",
                name: "Lando Vance",
                alias: "Papaya Prodigy",
                bio: "Twitch-reflex specialist with lightning high-speed chicane flicks and formidable wet-weather bravery.",
                style: DrivingStyle::Bold,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(4),
                offsets: DriverPersonalityOffsets::new(0.02, 0.0, 0.01, 0.02, 0.01, 0.2, 0.02),
                favorite_cars: LANDO_VANCE_FAVORITES,
            },
            DriverCharacter {
                id: "oscar_rocket",
                name: "Oscar Rocket",
                alias: "Melbourne Missile",
                bio: "Ultra-composed Australian sensation known for ice-cold nerve and textbook race craft on high-speed circuits.",
                style: DrivingStyle::Calculating,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(5),
                offsets: DriverPersonalityOffsets::new(0.01, 0.1, 0.01, 0.02, 0.00, 0.0, 0.02),
                favorite_cars: OSCAR_ROCKET_FAVORITES,
            },
            DriverCharacter {
                id: "carlos_sainzfield",
                name: "Carlos Sainzfield",
                alias: "Smooth Operator",
                bio: "Methodical Iberian racer who reads tire degradation with micro-precision, making strategic overtakes look effortless.",
                style: DrivingStyle::Calculating,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(8),
                offsets: DriverPersonalityOffsets::new(-0.01, 0.1, 0.00, 0.02, 0.02, -0.1, 0.02),
                favorite_cars: CARLOS_SAINZFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "pierre_gaslyfield",
                name: "Pierre Gaslyfield",
                alias: "The Underdog",
                bio: "Fiery underdog specialist whose daring late-braking passes through the chicane catch favorites off guard.",
                style: DrivingStyle::Bold,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.35, 0.38, 0.42, 1.0),
                    macroquad::color::Color::new(0.60, 0.95, 0.10, 1.0),
                    macroquad::color::Color::new(0.12, 0.12, 0.14, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.01, -0.1, 0.00, 0.01, 0.01, 0.2, 0.01),
                favorite_cars: PIERRE_GASLYFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "esteban_connor",
                name: "Esteban Connor",
                alias: "The Sentinel",
                bio: "Resolute defender who plants his machine on the apex line and refuses to concede an inch on narrow circuits.",
                style: DrivingStyle::Tenacious,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.08, 0.15, 0.45, 1.0),
                    macroquad::color::Color::new(0.98, 0.40, 0.35, 1.0),
                    macroquad::color::Color::new(0.95, 0.95, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, 0.01, 0.03, 0.2, 0.01),
                favorite_cars: ESTEBAN_CONNOR_FAVORITES,
            },
            DriverCharacter {
                id: "alexander_albonfield",
                name: "Alexander Albonfield",
                alias: "Apex Hunter",
                bio: "Dynamic overtaker specializing in aggressive switchback cut-backs and high-speed outside line sweeps.",
                style: DrivingStyle::Aggressive,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.10, 0.38, 0.22, 1.0),
                    macroquad::color::Color::new(0.92, 0.82, 0.50, 1.0),
                    macroquad::color::Color::new(0.90, 0.85, 0.40, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.00, 0.2, 0.00, -0.03, 0.02, 0.1, 0.02),
                favorite_cars: ALEXANDER_ALBONFIELD_FAVORITES,
            },
            DriverCharacter {
                id: "nico_hulkenstorm",
                name: "Nico Hulkenstorm",
                alias: "The Hulk",
                bio: "Iron-willed endurance veteran whose rock-solid pace and zero-mistake discipline make him a relentless podium threat.",
                style: DrivingStyle::Balanced,
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::new(
                    macroquad::color::Color::new(0.98, 0.98, 0.98, 1.0),
                    macroquad::color::Color::new(0.90, 0.15, 0.60, 1.0),
                    macroquad::color::Color::new(0.10, 0.85, 0.95, 1.0),
                ),
                offsets: DriverPersonalityOffsets::new(0.02, 0.1, 0.01, 0.00, 0.04, -0.3, 0.02),
                favorite_cars: NICO_HULKENSTORM_FAVORITES,
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "GT World Challenge Championship 2026".to_string(),
                point_system: PointSystem::FiaStandard { fastest_lap_bonus: true },
                track_ids: vec![
                    "bahrain".to_string(),
                    "suzuka".to_string(),
                    "monaco".to_string(),
                    "montreal".to_string(),
                    "catalunya".to_string(),
                    "madring".to_string(),
                    "red_bull_ring".to_string(),
                    "silverstone".to_string(),
                    "spa".to_string(),
                    "zandvoort".to_string(),
                    "monza".to_string(),
                    "marina_bay".to_string(),
                    "cota".to_string(),
                    "interlagos".to_string(),
                    "portimao_gp".to_string(),
                ],
                laps_per_round: 3,
            },
            TournamentFormat::QualifyingShootout {
                time_limit: 180.0,
            },
            TournamentFormat::QuickRace {
                default_laps: 3,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::gt4_clubsport()
    }
}


const MAX_HUNTER_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
];

const CHARLES_LAURENT_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
];

const LEWIS_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
];

const FERNANDO_TORO_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_audi_r8_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_nissan_r390_gt1"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
];

const GEORGE_SPEED_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
];

const LANDO_VANCE_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
];

const OSCAR_ROCKET_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_audi_r8_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
];

const CARLOS_SAINZFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
];

const PIERRE_GASLYFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_audi_r8_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_nissan_r390_gt1"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
];

const ESTEBAN_CONNOR_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
];

const ALEXANDER_ALBONFIELD_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
];

const NICO_HULKENSTORM_FAVORITES: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
];
