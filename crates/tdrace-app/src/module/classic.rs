use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
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
                id: "sports_car",
                name: "GT Sports Coupe",
                tag: "BALANCED RWD",
                description: "Balanced RWD arcade dynamics, responsive rack, 208 km/h top speed.",
                config: CarConfig::sports_car(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.85, 0.80, 0.75, 0.65),
                default_schemes: vec![
                    CarColorScheme::from_index(0),
                    CarColorScheme::from_index(1),
                    CarColorScheme::from_index(2),
                    CarColorScheme::from_index(3),
                ],
            },
            VehicleModelDefinition {
                id: "drift_car",
                name: "Tuned Drift Spec",
                tag: "PRO SLIDE",
                description: "High-power slide machine with loose rear, wide lock & snappy counter-steer.",
                config: CarConfig::drift_car(),
                visual_type: VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                },
                stats: (0.80, 0.85, 0.50, 0.98),
                default_schemes: vec![
                    CarColorScheme::from_index(4),
                    CarColorScheme::from_index(5),
                ],
            },
            VehicleModelDefinition {
                id: "kart",
                name: "125cc Shifter Kart",
                tag: "APEX GRIP",
                description: "Ultra-lightweight direct steering with extreme apex cornering grip.",
                config: CarConfig::kart(),
                visual_type: VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: true,
                },
                stats: (0.65, 0.95, 0.95, 0.40),
                default_schemes: vec![
                    CarColorScheme::from_index(5),
                    CarColorScheme::from_index(1),
                ],
            },
            VehicleModelDefinition {
                id: "rally_car",
                name: "AWD Turbo Rally",
                tag: "AWD ALL-TERRAIN",
                description: "All-wheel-drive traction with compliant suspension for mixed surfaces.",
                config: CarConfig::rally_car(),
                visual_type: VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                },
                stats: (0.78, 0.90, 0.85, 0.75),
                default_schemes: vec![
                    CarColorScheme::from_index(3),
                    CarColorScheme::from_index(2),
                ],
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "sports_car"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "classic_grand_prix",
                title: "Classic Grand Prix",
                tag: "FIA GP CIRCUIT",
                description: "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.",
                category: "Asphalt Circuit",
                default_laps: 3,
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
                id: "drift_park",
                title: "Drift Park",
                tag: "TECHNICAL DRIFT",
                description: "Technical hairpin slides, wide transitions & dynamic apex clipping zones.",
                category: "Drift Arena",
                default_laps: 3,
                generator: drift_park,
            },
            TrackDefinition {
                id: "kart_arena",
                title: "Kart Arena",
                tag: "AGILE SPRINT",
                description: "Tight 90-degree corners, rapid switchbacks & aggressive rumble curbs.",
                category: "Sprint Arena",
                default_laps: 4,
                generator: kart_arena,
            },
            TrackDefinition {
                id: "ramp_raceway",
                title: "Ramp Raceway",
                tag: "STUNT RAMPS & JUMPS",
                description: "High-speed stadium circuit with launch ramps, water hazards & gap jumps.",
                category: "Stunt Track",
                default_laps: 3,
                generator: ramp_raceway,
            },
            TrackDefinition {
                id: "oasis_rally",
                title: "Oasis Rally",
                tag: "DESERT DIRT RALLY",
                description: "Pure dirt desert rally circuit with oasis water hazards & sand traps.",
                category: "Desert Rally",
                default_laps: 3,
                generator: oasis_rally,
            },
            TrackDefinition {
                id: "outlaw_pass",
                title: "Outlaw Pass",
                tag: "NARROW MOUNTAIN PASS",
                description: "Perilous mountain circuit carving through a dramatic narrow canyon pass.",
                category: "Mountain Pass",
                default_laps: 3,
                generator: outlaw_pass,
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
                default_laps: 3,
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
                    "outlaw_pass".to_string(),
                ],
                laps_per_round: 3,
            },
            TournamentFormat::EliminationCup {
                elimination_interval: 1,
            },
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::gt_v8()
    }
}
