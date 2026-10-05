use super::{
    EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition,
    VehicleVisualType,
};
use crate::ai::DriverCharacter;
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};
use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;

/// Classic Arcade All-in-One Game Module
pub struct ClassicGameModule;

impl ClassicGameModule {
    pub fn new() -> Self {
        Self
    }

    /// 480 BHP Arcade GT Coupe: balanced RWD dynamics, high grip, forgiving slip.
    pub fn car_classic_gt() -> CarConfig {
        let mut cfg = CarConfig::sports_car();
        cfg.mass = 1150.0;
        cfg.max_engine_force = 7800.0;
        cfg.top_speed_mps = 58.0; // ~208 km/h
        cfg.max_brake_force = 13500.0;
        cfg.downforce_coefficient = 0.95;
        cfg.roll_balance = 0.55;
        cfg.rear_axle.peak_slip_scale = 0.92;
        cfg.rear_differential = tdrace_core::physics::config::DifferentialType::LimitedSlip {
            power_lock: 0.62,
            coast_lock: 0.25,
            preload_nm: 65.0,
        };
        cfg.steer_speed = 7.0;
        cfg.steer_return_speed = 9.0;
        cfg.tire.slide_grip = 0.94;
        cfg.tire.peak_slip_angle_deg = 9.7;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 750 BHP Arcade Stock Car: roaring speedway V8, planted rear, spin-proof stability.
    pub fn car_classic_nascar() -> CarConfig {
        let mut cfg = CarConfig::stock_car_ta1();
        cfg.mass = 1280.0;
        cfg.max_engine_force = 10500.0;
        cfg.top_speed_mps = 68.0; // ~245 km/h
        cfg.max_brake_force = 18000.0;
        cfg.max_steer_angle = 0.56;
        cfg.steer_speed = 8.0;
        cfg.steer_return_speed = 10.5;
        cfg.downforce_coefficient = 1.45;
        cfg.tire.slide_grip = 0.93;
        cfg.tire.peak_slip_angle_deg = 9.3;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 350 BHP Extreme Off-Road Buggy: high suspension travel, all-terrain forgiving grip.
    pub fn car_classic_offroad() -> CarConfig {
        let mut cfg = CarConfig::sand_rail();
        cfg.mass = 680.0;
        cfg.max_engine_force = 8200.0;
        cfg.top_speed_mps = 54.2; // ~195 km/h
        cfg.max_steer_angle = 0.76;
        cfg.steer_speed = 9.0;
        cfg.steer_return_speed = 11.0;
        cfg.downforce_coefficient = 0.85;
        cfg.tire.slide_grip = 0.95;
        cfg.tire.peak_slip_angle_deg = 11.3;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 45 BHP 200cc Arcade Sprint Kart: 1:1 direct steering, ultra-light, razor apex grip.
    pub fn car_classic_kart() -> CarConfig {
        let mut cfg = CarConfig::kart();
        cfg.mass = 180.0;
        cfg.max_engine_force = 2600.0;
        cfg.top_speed_mps = 32.0; // ~115 km/h
        cfg.max_steer_angle = 0.65; // ~37.2 deg calibrated FIA benchmark (Spec 038)
        cfg.steer_speed = 10.5;
        cfg.steer_return_speed = 14.0;
        cfg.tire.slide_grip = 0.90;
        cfg.tire.peak_slip_angle_deg = 7.0;
        cfg.tire.grip = 1.38;
        cfg.rear_axle.grip_scale = 1.45 / 1.38;
        cfg.caster_jacking_factor = 1.40;
        cfg.finalized()
    }

    /// 450 BHP Fantasy Group B Rally Beast: explosive 4WD acceleration, multi-surface suspension compliance, agile slide damping.
    pub fn car_classic_rally() -> CarConfig {
        let mut cfg = CarConfig::rally_car();
        cfg.mass = 1050.0;
        cfg.max_engine_force = 9200.0;
        cfg.top_speed_mps = 59.7; // ~215 km/h
        cfg.max_brake_force = 15000.0;
        cfg.max_steer_angle = 0.62;
        cfg.steer_speed = 8.5;
        cfg.steer_return_speed = 12.0;
        cfg.downforce_coefficient = 1.10;
        cfg.drive_bias = 0.45; // 4WD 45:55 front/rear torque split
        cfg.tire.slide_grip = 0.94;
        cfg.tire.peak_slip_angle_deg = 10.2;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 150 BHP Mudlark Cross Car: single-seat motorcycle-engine cross car, RWD, light and agile on turn-in.
    pub fn car_classic_ax_mudlark() -> CarConfig {
        let mut cfg = CarConfig::sand_rail();
        cfg.mass = 420.0;
        cfg.max_engine_force = 4800.0;
        cfg.top_speed_mps = 44.4; // ~160 km/h
        cfg.max_brake_force = 9000.0;
        cfg.max_steer_angle = 0.68;
        cfg.steer_speed = 9.5;
        cfg.steer_return_speed = 12.0;
        cfg.downforce_coefficient = 0.40;
        cfg.drive_bias = 0.0; // RWD
        cfg.tire.slide_grip = 0.95;
        cfg.tire.peak_slip_angle_deg = 8.5;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 250 BHP Brawler Touring AX: retro 1980s silhouette touring car, AWD, heavy and stable, slides wide through dirt berms.
    pub fn car_classic_ax_brawler() -> CarConfig {
        let mut cfg = CarConfig::rally_car();
        cfg.mass = 1150.0;
        cfg.max_engine_force = 6800.0;
        cfg.top_speed_mps = 51.4; // ~185 km/h
        cfg.max_brake_force = 13000.0;
        cfg.max_steer_angle = 0.60;
        cfg.steer_speed = 8.0;
        cfg.steer_return_speed = 11.0;
        cfg.downforce_coefficient = 0.75;
        cfg.drive_bias = 0.5; // AWD
        cfg.tire.slide_grip = 0.93;
        cfg.tire.peak_slip_angle_deg = 9.8;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 210 BHP Corsica '73 RS: classic 1973 air-cooled GT coupe, light rear-engine RWD balance, communicative slip.
    pub fn car_classic_gt_vintage() -> CarConfig {
        let mut cfg = CarConfig::sports_car();
        cfg.mass = 980.0;
        cfg.max_engine_force = 5400.0;
        cfg.top_speed_mps = 66.7; // ~240 km/h
        cfg.max_brake_force = 11000.0;
        cfg.downforce_coefficient = 0.55;
        cfg.roll_balance = 0.52;
        cfg.rear_axle.peak_slip_scale = 0.90;
        cfg.rear_differential = tdrace_core::physics::config::DifferentialType::LimitedSlip {
            power_lock: 0.50,
            coast_lock: 0.20,
            preload_nm: 45.0,
        };
        cfg.steer_speed = 7.2;
        cfg.steer_return_speed = 9.5;
        cfg.tire.slide_grip = 0.92;
        cfg.tire.peak_slip_angle_deg = 9.5;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 420 BHP Cyclone '69 Fastback: 1969 grand national muscle stocker, heavy roaring V8, momentum handling.
    pub fn car_classic_stock_vintage() -> CarConfig {
        let mut cfg = CarConfig::stock_car_ta1();
        cfg.mass = 1520.0;
        cfg.max_engine_force = 8500.0;
        cfg.top_speed_mps = 65.3; // ~235 km/h
        cfg.max_brake_force = 14500.0;
        cfg.max_steer_angle = 0.52;
        cfg.steer_speed = 7.5;
        cfg.steer_return_speed = 9.8;
        cfg.downforce_coefficient = 0.65;
        cfg.tire.slide_grip = 0.91;
        cfg.tire.peak_slip_angle_deg = 9.8;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 28 BHP Comet 100 Classic: 1970s air-cooled 100cc direct-drive vintage sprint kart.
    pub fn car_classic_kart_vintage() -> CarConfig {
        let mut cfg = CarConfig::kart();
        cfg.mass = 140.0;
        cfg.max_engine_force = 1800.0;
        cfg.top_speed_mps = 27.2; // ~98 km/h
        cfg.max_steer_angle = 0.62;
        cfg.steer_speed = 10.0;
        cfg.steer_return_speed = 13.0;
        cfg.tire.slide_grip = 0.88;
        cfg.tire.peak_slip_angle_deg = 7.2;
        cfg.tire.grip = 1.30;
        cfg.rear_axle.grip_scale = 1.40 / 1.30;
        cfg.caster_jacking_factor = 1.30;
        cfg.finalized()
    }

    /// 160 BHP Firebolt RS 2000: 1970s RWD twin-cam lightweight rally coupe, nimble and slide-happy.
    pub fn car_classic_rx_vintage() -> CarConfig {
        let mut cfg = CarConfig::rally_car();
        cfg.mass = 920.0;
        cfg.max_engine_force = 4500.0;
        cfg.top_speed_mps = 54.2; // ~195 km/h
        cfg.max_brake_force = 11500.0;
        cfg.max_steer_angle = 0.65;
        cfg.steer_speed = 8.8;
        cfg.steer_return_speed = 12.0;
        cfg.downforce_coefficient = 0.45;
        cfg.drive_bias = 0.0; // Pure RWD classic rally
        cfg.tire.slide_grip = 0.92;
        cfg.tire.peak_slip_angle_deg = 10.5;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 240 BHP Ironclad 4x4 Safari: 1980s Paris-Dakar 4WD trail rig, heavy and unstoppable through mud and ice.
    pub fn car_classic_at_safari() -> CarConfig {
        let mut cfg = CarConfig::sand_rail();
        cfg.mass = 1420.0;
        cfg.max_engine_force = 6800.0;
        cfg.top_speed_mps = 46.7; // ~168 km/h
        cfg.max_brake_force = 13500.0;
        cfg.max_steer_angle = 0.64;
        cfg.steer_speed = 7.8;
        cfg.steer_return_speed = 10.5;
        cfg.downforce_coefficient = 0.50;
        cfg.drive_bias = 0.5; // 4WD
        cfg.tire.slide_grip = 0.95;
        cfg.tire.peak_slip_angle_deg = 11.0;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }

    /// 560 BHP Talon Super Buggy (Legacy / Deprecated): open-wheel super buggy, AWD, raw acceleration.
    pub fn car_classic_ax_talon() -> CarConfig {
        let mut cfg = CarConfig::sand_rail();
        cfg.mass = 800.0;
        cfg.max_engine_force = 11000.0;
        cfg.top_speed_mps = 55.6; // ~200 km/h
        cfg.max_brake_force = 15500.0;
        cfg.max_steer_angle = 0.70;
        cfg.steer_speed = 9.0;
        cfg.steer_return_speed = 12.0;
        cfg.downforce_coefficient = 0.80;
        cfg.drive_bias = 0.5; // AWD
        cfg.tire.slide_grip = 0.94;
        cfg.tire.peak_slip_angle_deg = 10.5;
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg.finalized()
    }
}

impl Default for ClassicGameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for ClassicGameModule {
    fn id(&self) -> &'static str {
        "classic"
    }

    fn title(&self) -> &'static str {
        "TDRACE ARCADE MOTORSPORT"
    }

    fn subtitle(&self) -> &'static str {
        "Modern Cross-Platform 2D/2.5D Arcade Racing & CAD Circuit Studio"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.82, 0.20, 1.0), // Neon Gold
            secondary_accent: Color::new(0.20, 0.85, 1.0, 1.0), // Electric Cyan
            header_badge: "GENERIC MOTORSPORT SIMULATION & STUDIO",
            background_tint: Color::new(0.05, 0.06, 0.09, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "classic_gt",
                name: "Apex Phantom GT",
                tag: "ARCADE GT COUPE",
                description: "Balanced fantasy GT racer with razor-sharp arcade handling, high grip & 208 km/h top speed.",
                config: Self::car_classic_gt(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.85, 0.88, 0.90, 0.70),
                default_schemes: vec![
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(3),
                ],
                audio_profile: Some(EngineAudioProfile::gt4_clubsport()),
            },
            VehicleModelDefinition {
                id: "classic_gt_vintage",
                name: "Corsica '73 RS",
                tag: "CLASSIC GT SPORTS",
                description: "Air-cooled 1973 classic GT sports coupe with light rear-engine agility, ducktail aero, and authentic vintage slide dynamics.",
                config: Self::car_classic_gt_vintage(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: false,
                    gt_wing: false,
                    diffuser: false,
                },
                stats: (0.78, 0.82, 0.86, 0.80),
                default_schemes: vec![
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(4),
                ],
                audio_profile: Some(EngineAudioProfile::gt4_clubsport()),
            },
            VehicleModelDefinition {
                id: "classic_nascar",
                name: "Thunderbolt Stock V8",
                tag: "ARCADE SPEEDWAY STOCK",
                description: "Roaring 750 BHP stock car with planted high-speed stability and forgiving drift control.",
                config: Self::car_classic_nascar(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: true,
                    roof_fins: true,
                    window_net: true,
                },
                stats: (0.95, 0.90, 0.85, 0.80),
                default_schemes: vec![
                    CarColorScheme::from_index(4),
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(5),
                ],
                audio_profile: Some(EngineAudioProfile::late_model_v8()),
            },
            VehicleModelDefinition {
                id: "classic_stock_vintage",
                name: "Cyclone '69 Fastback",
                tag: "VINTAGE GRAND NATIONAL",
                description: "1969 Grand National muscle fastback packing 420 BHP of naturally-aspirated big-block V8 thunder and heavyweight momentum.",
                config: Self::car_classic_stock_vintage(),
                visual_type: VehicleVisualType::StockCar {
                    tall_wing: false,
                    roof_fins: false,
                    window_net: false,
                },
                stats: (0.90, 0.85, 0.80, 0.85),
                default_schemes: vec![
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(1),
                ],
                audio_profile: Some(EngineAudioProfile::late_model_v8()),
            },
            VehicleModelDefinition {
                id: "classic_offroad",
                name: "Vortex Dune Crusher",
                tag: "EXTREME OFF-ROAD BUGGY",
                description: "Long-travel dune & stunt buggy with all-terrain arcade traction and high jump compliance.",
                config: Self::car_classic_offroad(),
                visual_type: VehicleVisualType::SandRail {
                    lightbar: true,
                    whip_antenna: true,
                    paddle_tires: true,
                },
                stats: (0.82, 0.92, 0.88, 0.92),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(2),
                ],
                audio_profile: Some(EngineAudioProfile::sand_rail_boxer()),
            },
            VehicleModelDefinition {
                id: "classic_at_safari",
                name: "Ironclad 4x4 Safari",
                tag: "VINTAGE ALL-TERRAIN RIG",
                description: "1980s Desert Raid vintage 4WD trail rig with unstoppable mud flotation, high ground clearance, and snow/ice stability.",
                config: Self::car_classic_at_safari(),
                visual_type: VehicleVisualType::SandRail {
                    lightbar: false,
                    whip_antenna: false,
                    paddle_tires: false,
                },
                stats: (0.72, 0.82, 0.94, 0.90),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(0),
                ],
                audio_profile: Some(EngineAudioProfile::sand_rail_boxer()),
            },
            VehicleModelDefinition {
                id: "classic_kart",
                name: "Turbo Dart 200cc",
                tag: "ARCADE SPRINT KART",
                description: "Ultra-agile fantasy micro-kart with 1:1 direct steering and impossible-to-spin apex grip.",
                config: Self::car_classic_kart(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: true,
                },
                stats: (0.70, 0.96, 0.98, 0.45),
                default_schemes: vec![
                    CarColorScheme::from_index(5),
                    CarColorScheme::from_index(2),
                ],
                audio_profile: Some(EngineAudioProfile::kart_cadet_60()),
            },
            VehicleModelDefinition {
                id: "classic_kart_vintage",
                name: "Comet 100 Classic",
                tag: "VINTAGE SPRINT KART",
                description: "1970s air-cooled 100cc direct-drive vintage kart with tactile road feel, bare chassis frame, and pure mechanical feedback.",
                config: Self::car_classic_kart_vintage(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: false,
                },
                stats: (0.64, 0.90, 0.94, 0.50),
                default_schemes: vec![
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(0),
                ],
                audio_profile: Some(EngineAudioProfile::kart_cadet_60()),
            },
            VehicleModelDefinition {
                id: "classic_rally",
                name: "Trailfire Turbo 4WD",
                tag: "ARCADE GROUP B RALLY",
                description: "Explosive 4WD fantasy rally beast with long-travel suspension, jump composure, and fearless multi-surface slides.",
                config: Self::car_classic_rally(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                },
                stats: (0.88, 0.94, 0.92, 0.88),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(1),
                ],
                audio_profile: Some(EngineAudioProfile::rally2_turbo()),
            },
            VehicleModelDefinition {
                id: "classic_rx_vintage",
                name: "Firebolt RS 2000",
                tag: "CLASSIC RWD RALLY",
                description: "1970s lightweight RWD twin-cam rally coupe built for pendulum slides, dirt flick entries, and tarmac precision.",
                config: Self::car_classic_rx_vintage(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: false,
                    mudflaps: true,
                    large_wing: false,
                },
                stats: (0.76, 0.88, 0.88, 0.92),
                default_schemes: vec![
                    CarColorScheme::from_index(4),
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(2),
                ],
                audio_profile: Some(EngineAudioProfile::rally2_turbo()),
            },
            VehicleModelDefinition {
                id: "classic_ax_mudlark",
                name: "Mudlark Cross Car",
                tag: "ARCADE CROSS CAR",
                description: "Agile single-seat cross car powered by an 850cc motorcycle engine, engineered for razor-sharp turn-in and high-revving dirt sprints.",
                config: Self::car_classic_ax_mudlark(),
                visual_type: VehicleVisualType::SandRail {
                    lightbar: false,
                    whip_antenna: false,
                    paddle_tires: false,
                },
                stats: (0.76, 0.95, 0.92, 0.85),
                default_schemes: vec![
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(0),
                ],
                audio_profile: Some(EngineAudioProfile::cross_car_motorcycle()),
            },
            VehicleModelDefinition {
                id: "classic_ax_brawler",
                name: "Brawler Touring AX",
                tag: "ARCADE TOURING AX",
                description: "Robust touring silhouette autocross machine built by Stonecairn Works. Heavy and stable, slides wide through dirt berms while maintaining fierce four-wheel traction.",
                config: Self::car_classic_ax_brawler(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                },
                stats: (0.82, 0.92, 0.88, 0.90),
                default_schemes: vec![
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(4),
                ],
                audio_profile: Some(EngineAudioProfile::rally2_turbo()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "classic_gt"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        crate::module::catalog_tracks("classic")
    }

    fn default_track_id(&self) -> &'static str {
        "gt_coastal_grand_prix"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        DriverCharacter::ROSTER.to_vec()
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::QuickRace {
                default_laps: 5,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
            TournamentFormat::Championship {
                name: "TDRace Grand Championship".to_string(),
                point_system: PointSystem::ClassicArcade,
                track_ids: vec![
                    "kart_pine_grove".to_string(),
                    "rx_quarry_sprint".to_string(),
                    "ax_meadow_sprint".to_string(),
                    "gt_velocity_park".to_string(),
                    "stock_thunder_bowl".to_string(),
                    "at_dune_sea".to_string(),
                ],
                laps_per_round: 5,
            },
            TournamentFormat::EliminationCup {
                elimination_interval: 1,
            },
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::gt4_clubsport()
    }
}
