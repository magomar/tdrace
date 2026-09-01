use macroquad::color::Color;
use tdrace_core::physics::config::{CarConfig, DriverAssistsConfig, TireConfig};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    dirt_figure_eight, hell_rx, holjes_rx, loheac_rx, lydden_hill, oasis_rally, outlaw_pass,
    sahara_dunes,
};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// World Rally Stage Championship Game Module
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
            stiffness_b: 7.8, // Compliant tire sidewall for gravel/dirt
            shape_c: 1.40,
            peak_d: 1.05,
            curvature_e: -0.18,
            drift_slide_friction: 0.92, // High controllable slide grip
            handbrake_lateral_friction_multiplier: 0.32,
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
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
        "TDRACE WORLD RALLY CHAMPIONSHIP"
    }

    fn subtitle(&self) -> &'static str {
        "Loose Surface AWD Stage Time Trials & Extreme Mountain Passes"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(1.0, 0.55, 0.15, 1.0), // Dirt Rally Orange
            secondary_accent: Color::new(0.95, 0.85, 0.20, 1.0), // Desert Sand Yellow
            header_badge: "WORLD RALLY CHAMPIONSHIP (WRC)",
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
        vec![
            TrackDefinition {
                id: "dirt_figure_eight",
                title: "Dirt Figure-8 Arena",
                tag: "FIGURE-8 CROSSOVER",
                description: "Stadium figure-8 circuit featuring an elevated crossover bridge overpass, dirt carousels & tabletop jumps.",
                category: "Figure-8 Arena",
                default_laps: 4,
                generator: dirt_figure_eight,
            },
            TrackDefinition {
                id: "holjes_rx",
                title: "Höljes Motorstadion",
                tag: "WORLD RX SWEDEN",
                description: "The holy grail of Rallycross featuring the iconic Höljes jump crest, banked Velodrome & mixed gravel sliding.",
                category: "World RX",
                default_laps: 4,
                generator: holjes_rx,
            },
            TrackDefinition {
                id: "lydden_hill",
                title: "Lydden Hill Race Circuit",
                tag: "WORLD RX GREAT BRITAIN",
                description: "The historic birthplace of Rallycross featuring Chessons Drift gravel slide, North Bend & Devil's Elbow.",
                category: "World RX",
                default_laps: 4,
                generator: lydden_hill,
            },
            TrackDefinition {
                id: "hell_rx",
                title: "Lånkebanen / Hell RX",
                tag: "WORLD RX NORWAY",
                description: "Welcome to Hell! Fast downhill asphalt sweep, loose gravel carousel, technical esses & high-flying crests.",
                category: "World RX",
                default_laps: 4,
                generator: hell_rx,
            },
            TrackDefinition {
                id: "loheac_rx",
                title: "Circuit de Lohéac",
                tag: "WORLD RX FRANCE",
                description: "The French Rallycross classic with long asphalt drag straight, gravel tabletop jump & tight switchbacks.",
                category: "World RX",
                default_laps: 4,
                generator: loheac_rx,
            },
            TrackDefinition {
                id: "oasis_rally",
                title: "Oasis Desert Rally",
                tag: "DESERT STAGE",
                description: "High-sliding sand and gravel rally stage surrounding a desert oasis water hazard.",
                category: "Desert Stage",
                default_laps: 3,
                generator: oasis_rally,
            },
            TrackDefinition {
                id: "outlaw_pass",
                title: "Outlaw Canyon Pass",
                tag: "MOUNTAIN PASS",
                description: "Dangerous mountain pass carving through narrow rock canyons and tight switchbacks.",
                category: "Mountain Pass",
                default_laps: 3,
                generator: outlaw_pass,
            },
            TrackDefinition {
                id: "sahara_dunes",
                title: "Sahara Dunes Stage",
                tag: "DUNES & DIRT",
                description: "Fast undulating sand dunes with heavy drift zones and sweeping crests.",
                category: "Dune Stage",
                default_laps: 3,
                generator: sahara_dunes,
            },
        ]
    }

    fn default_track_id(&self) -> &'static str {
        "holjes_rx"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "marco_rossi",
                name: "Marco Rossi",
                alias: "The Desert Fox",
                bio: "Fearless Finnish rally master who throws the car sideways at 180 km/h into blind gravel crests without lifting.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(3),
                profile: BotProfile {
                    name: "Marco Rossi",
                    lookahead_time: 0.38,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.08,
                    brake_margin: 1.00,
                    aggression: 0.92,
                    avoidance_distance: 5.5,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.95,
                    precision: 0.92,
                    defense: 0.90,
                },
            },
            DriverCharacter {
                id: "colin_mcrae_tribute",
                name: "Calum McTavish",
                alias: "Flat Out",
                bio: "If in doubt, flat out. Legendary sliding geometry and maximum commitment over every jump and dirt crest.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(1),
                profile: BotProfile {
                    name: "Calum McTavish",
                    lookahead_time: 0.36,
                    speed_factor: 1.04,
                    steering_kp: 2.9,
                    steering_kd: 0.09,
                    brake_margin: 0.98,
                    aggression: 0.98,
                    avoidance_distance: 5.0,
                },
                stats: DriverStats {
                    speed: 0.98,
                    aggression: 1.00,
                    precision: 0.90,
                    defense: 0.85,
                },
            },
            DriverCharacter {
                id: "seb_laurent",
                name: "Sebastien Laurent",
                alias: "The Maestro",
                bio: "9-time Rally Champion whose razor-sharp racing line and surgical balance dominate tarmac and gravel alike.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(7),
                profile: BotProfile {
                    name: "Sebastien Laurent",
                    lookahead_time: 0.40,
                    speed_factor: 1.05,
                    steering_kp: 2.8,
                    steering_kd: 0.07,
                    brake_margin: 1.00,
                    aggression: 0.88,
                    avoidance_distance: 5.8,
                },
                stats: DriverStats {
                    speed: 0.98,
                    aggression: 0.88,
                    precision: 0.99,
                    defense: 0.94,
                },
            },
            DriverCharacter {
                id: "carlos_matador",
                name: "Carlos Matador",
                alias: "El Toro",
                bio: "Relentless Spanish rally legend with unmatched mechanical sympathy, endurance, and fearless high-speed commitment.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(2),
                profile: BotProfile {
                    name: "Carlos Matador",
                    lookahead_time: 0.39,
                    speed_factor: 1.02,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 1.01,
                    aggression: 0.92,
                    avoidance_distance: 5.4,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.92,
                    precision: 0.96,
                    defense: 0.96,
                },
            },
            DriverCharacter {
                id: "juha_flying",
                name: "Juha Korhonen",
                alias: "The Flying Finn",
                bio: "Ice and snow veteran who glides across loose gravel with Scandinavian flicks and effortless countersteering.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(4),
                profile: BotProfile {
                    name: "Juha Korhonen",
                    lookahead_time: 0.37,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.08,
                    brake_margin: 0.99,
                    aggression: 0.94,
                    avoidance_distance: 5.2,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.94,
                    precision: 0.93,
                    defense: 0.89,
                },
            },
            DriverCharacter {
                id: "walter_meister",
                name: "Walter Baum",
                alias: "Der Meister",
                bio: "Group B powerhouse master whose lightning clutch kicks and left-foot braking conquer treacherous mountain ridges.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(5),
                profile: BotProfile {
                    name: "Walter Baum",
                    lookahead_time: 0.38,
                    speed_factor: 1.04,
                    steering_kp: 2.8,
                    steering_kd: 0.07,
                    brake_margin: 0.99,
                    aggression: 0.90,
                    avoidance_distance: 5.5,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.90,
                    precision: 0.98,
                    defense: 0.92,
                },
            },
            DriverCharacter {
                id: "miki_brawler",
                name: "Miki Bassi",
                alias: "Torino Bullet",
                bio: "Aggressive Italian rally driver famed for full-throttle slides and bold overtaking in dust clouds and sand storms.",
                preferred_car: crate::ui::menu::CarChoice::RallyCar,
                color_scheme: CarColorScheme::from_index(6),
                profile: BotProfile {
                    name: "Miki Bassi",
                    lookahead_time: 0.35,
                    speed_factor: 1.02,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 0.97,
                    aggression: 0.96,
                    avoidance_distance: 5.0,
                },
                stats: DriverStats {
                    speed: 0.94,
                    aggression: 0.96,
                    precision: 0.91,
                    defense: 0.90,
                },
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::StageRally {
                name: "World RX Tour".to_string(),
                stage_track_ids: vec![
                    "dirt_figure_eight".to_string(),
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "oasis_rally".to_string(),
                    "outlaw_pass".to_string(),
                    "sahara_dunes".to_string(),
                ],
            },
            TournamentFormat::Championship {
                name: "FIA World Rallycross Championship".to_string(),
                point_system: PointSystem::F1Standard { fastest_lap_bonus: false },
                track_ids: vec![
                    "dirt_figure_eight".to_string(),
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "oasis_rally".to_string(),
                    "outlaw_pass".to_string(),
                    "sahara_dunes".to_string(),
                ],
                laps_per_round: 4,
            },
            TournamentFormat::QuickRace {
                default_laps: 4,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::rally_turbo_antilag()
    }
}
