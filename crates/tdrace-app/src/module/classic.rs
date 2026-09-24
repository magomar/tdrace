use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::presets::{
    classic_grand_prix, classic_rallycross, dirt_figure_eight, dirty_oval_speedway, drift_park, figure_eight,
    kart_arena, oasis_rally, oval_speedway, ramp_raceway,
};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::DriverCharacter;
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

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
        cfg.steer_speed = 7.0;
        cfg.steer_return_speed = 9.0;
        cfg.tire.drift_slide_friction = 0.94;
        cfg.tire.stiffness_b = 10.5;
        for w in &mut cfg.wheels {
            w.tire_model = cfg.tire;
        }
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg
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
        cfg.tire.drift_slide_friction = 0.93;
        cfg.tire.stiffness_b = 11.0;
        for w in &mut cfg.wheels {
            w.tire_model = cfg.tire;
        }
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg
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
        cfg.tire.drift_slide_friction = 0.95;
        cfg.tire.stiffness_b = 9.0;
        cfg.weight_transfer_longitudinal = 0.65;
        cfg.weight_transfer_lateral = 0.65;
        for w in &mut cfg.wheels {
            w.tire_model = cfg.tire;
        }
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg
    }

    /// 45 BHP 200cc Arcade Sprint Kart: 1:1 direct steering, ultra-light, razor apex grip.
    pub fn car_classic_kart() -> CarConfig {
        let mut cfg = CarConfig::kart();
        cfg.mass = 180.0;
        cfg.max_engine_force = 2600.0;
        cfg.top_speed_mps = 32.0; // ~115 km/h
        cfg.max_steer_angle = 0.65;
        cfg.steer_speed = 10.0;
        cfg.steer_return_speed = 14.0;
        cfg.tire.drift_slide_friction = 0.90;
        cfg.tire.stiffness_b = 13.0;
        for w in &mut cfg.wheels {
            w.tire_model.drift_slide_friction = cfg.tire.drift_slide_friction;
            w.tire_model.stiffness_b = cfg.tire.stiffness_b;
        }
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg
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
        cfg.drive_bias = 0.5; // 4WD 50:50 torque split
        cfg.tire.drift_slide_friction = 0.94;
        cfg.tire.stiffness_b = 10.0;
        cfg.weight_transfer_longitudinal = 0.50;
        cfg.weight_transfer_lateral = 0.50;
        for w in &mut cfg.wheels {
            w.tire_model = cfg.tire;
            w.drive_torque_factor = 0.25;
            w.brake_bias_factor = 0.25;
        }
        cfg.assists = tdrace_core::physics::config::DriverAssistsConfig::arcade();
        cfg
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
                audio_profile: Some(EngineAudioProfile::cross_car_motorcycle()),
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "classic_gt"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "classic_grand_prix",
                title: "Classic Grand Prix",
                tag: "FIA GP CIRCUIT",
                description: "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.",
                category: "Asphalt Circuit",
                default_laps: 5,
                generator: classic_grand_prix,
            },
            TrackDefinition {
                id: "oval_speedway",
                title: "Oval Speedway",
                tag: "SUPERSPEEDWAY",
                description: "Full-throttle banked superspeedway surrounded by concrete barriers.",
                category: "Oval Superspeedway",
                default_laps: 5,
                generator: oval_speedway,
            },
            TrackDefinition {
                id: "dirty_oval_speedway",
                title: "Dirty Oval Speedway",
                tag: "DIRT SPEEDWAY",
                description: "Banked dirt oval speedway with loose gravel cushion and high-sliding turns.",
                category: "Dirt Oval",
                default_laps: 5,
                generator: dirty_oval_speedway,
            },
            TrackDefinition {
                id: "figure_eight",
                title: "Figure 8",
                tag: "ASPHALT CROSSOVER",
                description: "High-speed asphalt figure-8 arena with at-grade flat crossover and concrete safety walls.",
                category: "Figure-8 Arena",
                default_laps: 5,
                generator: figure_eight,
            },
            TrackDefinition {
                id: "dirt_figure_eight",
                title: "Dirt Figure-8 Arena",
                tag: "DIRT CROSSOVER",
                description: "Stadium figure-8 dirt arena featuring an at-grade flat crossover, sweeping dirt carousels & tabletop jumps.",
                category: "Figure-8 Arena",
                default_laps: 5,
                generator: dirt_figure_eight,
            },
            TrackDefinition {
                id: "drift_park",
                title: "Drift Park",
                tag: "TECHNICAL DRIFT",
                description: "Technical hairpin slides, wide transitions & dynamic apex clipping zones.",
                category: "Drift Arena",
                default_laps: 5,
                generator: drift_park,
            },
            TrackDefinition {
                id: "kart_arena",
                title: "Kart Arena",
                tag: "AGILE SPRINT",
                description: "Tight 90-degree corners, rapid switchbacks & aggressive rumble curbs.",
                category: "Sprint Arena",
                default_laps: 5,
                generator: kart_arena,
            },
            TrackDefinition {
                id: "ramp_raceway",
                title: "Ramp Raceway",
                tag: "DIRT STUNT RAMPS",
                description: "High-speed dirt stadium circuit with launch ramps, water hazards & gap jumps.",
                category: "Dirt Stunt Track",
                default_laps: 5,
                generator: ramp_raceway,
            },
            TrackDefinition {
                id: "oasis_rally",
                title: "Oasis Rally",
                tag: "DESERT DIRT RALLY",
                description: "Pure dirt desert rally circuit with oasis water hazards & sand traps.",
                category: "Desert Rally",
                default_laps: 5,
                generator: oasis_rally,
            },
            TrackDefinition {
                id: "classic_rallycross",
                title: "Classic Rallycross",
                tag: "HYBRID RALLYCROSS",
                description: "Dynamic 1.0 km mixed-surface rallycross circuit with asphalt straights, dirt hairpins & tabletop jumps.",
                category: "Mixed Surface RX",
                default_laps: 5,
                generator: classic_rallycross,
            },
        ]
    }

    fn default_track_id(&self) -> &'static str {
        "classic_grand_prix"
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
                    "classic_grand_prix".to_string(),
                    "drift_park".to_string(),
                    "ramp_raceway".to_string(),
                    "oasis_rally".to_string(),
                    "classic_rallycross".to_string(),
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
