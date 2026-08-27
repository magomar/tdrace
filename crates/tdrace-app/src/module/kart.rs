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
