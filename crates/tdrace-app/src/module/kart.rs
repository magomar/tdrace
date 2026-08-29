use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::presets::{drift_park, kart_arena};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Sprint Karting Cup Game Module
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
        "TDRACE SPRINT KARTING CUP"
    }

    fn subtitle(&self) -> &'static str {
        "Direct-Drive 125cc Shifter Karts, 3G Apex Cornering & Wheel-to-Wheel Heats"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(0.30, 0.95, 0.40, 1.0), // Electric Kart Green
            secondary_accent: Color::new(1.0, 0.85, 0.20, 1.0), // Racing Yellow
            header_badge: "INTERNATIONAL KARTING CHAMPIONSHIP",
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
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "shifter_kart_125"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "kart_arena",
                title: "Kart Arena International",
                tag: "AGILE SPRINT",
                description: "Tight 90-degree corners, rapid switchbacks, and aggressive rumble curbs.",
                category: "Sprint Arena",
                default_laps: 6,
                generator: kart_arena,
            },
            TrackDefinition {
                id: "drift_park",
                title: "Drift Park Sprint",
                tag: "TECHNICAL HAIRPINS",
                description: "Technical hairpin slides, wide transitions, and tight apex curbs.",
                category: "Technical Sprint",
                default_laps: 6,
                generator: drift_park,
            },
        ]
    }

    fn default_track_id(&self) -> &'static str {
        "kart_arena"
    }

    fn drivers(&self) -> Vec<DriverCharacter> {
        vec![
            DriverCharacter {
                id: "leo_sprint",
                name: "Leo 'Rocket' Rossi",
                alias: "Apex Rocket",
                bio: "Rising karting prodigy with lightning-quick reflexes and an aggressive overtaking line down the inside of every hairpin.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(5),
                profile: BotProfile {
                    name: "Leo Rossi",
                    lookahead_time: 0.30,
                    speed_factor: 1.05,
                    steering_kp: 3.2,
                    steering_kd: 0.06,
                    brake_margin: 1.00,
                    aggression: 0.85,
                    avoidance_distance: 4.5,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.88,
                    precision: 0.96,
                    defense: 0.84,
                },
            },
            DriverCharacter {
                id: "mia_apex",
                name: "Mia 'Smooth' Zhang",
                alias: "The Metronome",
                bio: "Karting national champion known for ultra-smooth steering inputs and consistently hitting apex curbs to the millimeter.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(1),
                profile: BotProfile {
                    name: "Mia Zhang",
                    lookahead_time: 0.32,
                    speed_factor: 1.04,
                    steering_kp: 3.0,
                    steering_kd: 0.07,
                    brake_margin: 1.01,
                    aggression: 0.70,
                    avoidance_distance: 5.0,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.72,
                    precision: 0.99,
                    defense: 0.90,
                },
            },
            DriverCharacter {
                id: "taro_kazama",
                name: "Taro Kazama",
                alias: "Drift Apex",
                bio: "Junior kart sensation who throws the rear end out on entry, pivoting around hairpins at impossible angles.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(0),
                profile: BotProfile {
                    name: "Taro Kazama",
                    lookahead_time: 0.31,
                    speed_factor: 1.03,
                    steering_kp: 3.1,
                    steering_kd: 0.06,
                    brake_margin: 0.99,
                    aggression: 0.90,
                    avoidance_distance: 4.8,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.90,
                    precision: 0.94,
                    defense: 0.86,
                },
            },
            DriverCharacter {
                id: "sofia_vega",
                name: "Sofia Vega",
                alias: "Braking Bandit",
                bio: "Master of trail braking in shifter karts, outbraking rivals by fractions of an inch on every hairpin.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(2),
                profile: BotProfile {
                    name: "Sofia Vega",
                    lookahead_time: 0.33,
                    speed_factor: 1.02,
                    steering_kp: 2.9,
                    steering_kd: 0.07,
                    brake_margin: 0.98,
                    aggression: 0.92,
                    avoidance_distance: 4.6,
                },
                stats: DriverStats {
                    speed: 0.94,
                    aggression: 0.92,
                    precision: 0.92,
                    defense: 0.88,
                },
            },
            DriverCharacter {
                id: "lucas_meyer",
                name: "Lucas Meyer",
                alias: "Kart Maestro",
                bio: "European kart trophy holder whose surgical line discipline extracts maximum momentum through chicanes.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(3),
                profile: BotProfile {
                    name: "Lucas Meyer",
                    lookahead_time: 0.34,
                    speed_factor: 1.04,
                    steering_kp: 3.0,
                    steering_kd: 0.06,
                    brake_margin: 1.02,
                    aggression: 0.82,
                    avoidance_distance: 5.2,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.82,
                    precision: 0.97,
                    defense: 0.92,
                },
            },
            DriverCharacter {
                id: "chloe_dubois",
                name: "Chloe Dubois",
                alias: "Slipstream Ace",
                bio: "Aggressive overtaker who tracks the slipstream cone to sling past opponents on the exit of every turn.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(4),
                profile: BotProfile {
                    name: "Chloe Dubois",
                    lookahead_time: 0.32,
                    speed_factor: 1.03,
                    steering_kp: 3.1,
                    steering_kd: 0.07,
                    brake_margin: 1.00,
                    aggression: 0.84,
                    avoidance_distance: 4.9,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.84,
                    precision: 0.95,
                    defense: 0.91,
                },
            },
            DriverCharacter {
                id: "kai_sato",
                name: "Kai Sato",
                alias: "Apex Hunter",
                bio: "Fearless kart racer with lightning reaction times who never concedes an inside line in wheel-to-wheel battles.",
                preferred_car: crate::ui::menu::CarChoice::Kart,
                color_scheme: CarColorScheme::from_index(7),
                profile: BotProfile {
                    name: "Kai Sato",
                    lookahead_time: 0.29,
                    speed_factor: 1.05,
                    steering_kp: 3.3,
                    steering_kd: 0.05,
                    brake_margin: 0.97,
                    aggression: 0.94,
                    avoidance_distance: 4.4,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.94,
                    precision: 0.93,
                    defense: 0.82,
                },
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::EliminationCup {
                elimination_interval: 2,
            },
            TournamentFormat::Championship {
                name: "National Karting Trophy".to_string(),
                point_system: PointSystem::ClassicArcade,
                track_ids: vec!["kart_arena".to_string(), "drift_park".to_string()],
                laps_per_round: 6,
            },
            TournamentFormat::QuickRace {
                default_laps: 6,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::kart_2stroke()
    }
}
