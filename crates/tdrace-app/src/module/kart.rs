use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, TrackGeometry};
use tdrace_core::track::presets::{
    drift_park, generate_checkpoints, generate_grid_positions, generate_walls_from_spline, kart_arena,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{Track, TrackCategory};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Karting World Cup Game Module
pub struct KartGameModule;

impl KartGameModule {
    pub fn new() -> Self {
        Self
    }

    pub fn car_shifter_kart() -> CarConfig {
        CarConfig::kart()
    }

    /// South Garda Karting (Lonato, Italy): CIK-FIA World Championship Circuit
    pub fn track_lonato() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(39.6, -1.0), 9.2),
            TrackWaypoint::new(Vec2::new(79.4, -5.2), 9.2),
            TrackWaypoint::new(Vec2::new(119.3, -4.1), 9.2),
            TrackWaypoint::new(Vec2::new(159.3, -3.4), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(193.1, 12.8), 8.5),
            TrackWaypoint::new(Vec2::new(231.1, 22.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(262.4, -1.8), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(271.7, -39.5), 8.5),
            TrackWaypoint::new(Vec2::new(271.0, -79.2), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(236.2, -85.4), 8.5),
            TrackWaypoint::new(Vec2::new(196.7, -85.9), 8.5),
            TrackWaypoint::new(Vec2::new(156.8, -82.2), 8.5),
            TrackWaypoint::new(Vec2::new(117.0, -78.4), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(125.2, -54.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(165.1, -57.3), 8.5),
            TrackWaypoint::new(Vec2::new(205.0, -60.5), 8.5),
            TrackWaypoint::new(Vec2::new(244.2, -59.9), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(247.2, -23.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(211.1, -19.4), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(174.3, -29.8), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(134.8, -24.5), 8.5),
            TrackWaypoint::new(Vec2::new(94.8, -26.6), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(83.3, -57.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(50.1, -68.5), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(10.5, -62.9), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-9.4, -41.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(30.0, -42.3), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(54.5, -27.4), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(16.2, -20.9), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "South Garda Karting (Lonato)".to_string(),
            description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Circuito Internazionale Napoli (Sarno, Italy): High-Speed Temple of Speed
    pub fn track_sarno() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(46.8, 5.3), 9.5),
            TrackWaypoint::new(Vec2::new(95.0, 8.0), 9.5),
            TrackWaypoint::new(Vec2::new(142.9, 14.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(190.3, 4.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(196.2, -25.4), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(149.2, -33.0), 8.5),
            TrackWaypoint::new(Vec2::new(108.8, -52.3), 8.5),
            TrackWaypoint::new(Vec2::new(71.0, -57.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(37.9, -33.5), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(32.2, -80.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(64.0, -111.5), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(110.6, -103.4), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(151.7, -78.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(199.6, -78.1), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(230.7, -101.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(185.5, -107.6), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(148.5, -129.8), 8.5),
            TrackWaypoint::new(Vec2::new(104.6, -147.1), 8.5),
            TrackWaypoint::new(Vec2::new(56.8, -154.5), 8.5),
            TrackWaypoint::new(Vec2::new(9.1, -163.0), 8.5),
            TrackWaypoint::new(Vec2::new(-38.7, -170.6), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-70.8, -137.7), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-84.2, -91.2), 8.5),
            TrackWaypoint::new(Vec2::new(-97.7, -44.7), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-87.4, -0.2), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-52.0, -7.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-53.6, -53.3), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-24.0, -87.9), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-11.2, -125.6), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(6.4, -85.4), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(0.1, -39.9), 9.5),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 24, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Circuito Internazionale Napoli (Sarno)".to_string(),
            description: "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Karting Genk (Genk, Belgium): Home of Champions
    pub fn track_genk() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(44.3, 4.9), 9.2),
            TrackWaypoint::new(Vec2::new(89.6, 5.8), 9.2),
            TrackWaypoint::new(Vec2::new(134.9, 6.7), 9.2),
            TrackWaypoint::new(Vec2::new(180.3, 7.7), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(217.1, -13.9), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(197.5, -25.6), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(153.5, -21.1), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(152.0, -63.6), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(194.9, -71.3), 8.5),
            TrackWaypoint::new(Vec2::new(239.5, -67.3), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(257.5, -28.3), 8.5),
            TrackWaypoint::new(Vec2::new(273.5, 9.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(285.8, -26.3), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(280.7, -71.3), 8.5),
            TrackWaypoint::new(Vec2::new(275.6, -116.3), 8.5),
            TrackWaypoint::new(Vec2::new(270.2, -161.2), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(230.5, -163.7), 8.5),
            TrackWaypoint::new(Vec2::new(192.9, -167.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(184.0, -125.5), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(227.5, -120.1), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(247.5, -95.6), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(203.3, -92.2), 8.5),
            TrackWaypoint::new(Vec2::new(157.9, -92.9), 8.5),
            TrackWaypoint::new(Vec2::new(113.3, -90.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(119.5, -52.4), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(97.5, -22.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(52.3, -25.4), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(11.6, -45.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-24.2, -37.5), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Karting Genk (Home of Champions)".to_string(),
            description: "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// PF International Kart Circuit (PFI, UK): Elevated Flyover Crossover Bridge
    pub fn track_pfi() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.2),
            TrackWaypoint::new(Vec2::new(43.2, 0.5), 9.2),
            TrackWaypoint::new(Vec2::new(86.4, 0.2), 8.5),
            TrackWaypoint::new(Vec2::new(125.6, -12.1), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(157.9, -36.9), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(200.8, -37.9), 8.5),
            TrackWaypoint::new(Vec2::new(240.7, -51.4), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(248.5, -16.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(211.4, -0.4), 8.0).with_elevation(2.2).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(180.1, -19.3), 8.0).with_elevation(4.2).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(181.2, -56.4), 8.0).with_elevation(4.2).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(138.1, -58.8), 8.5).with_elevation(2.2),
            TrackWaypoint::new(Vec2::new(94.9, -59.5), 8.5),
            TrackWaypoint::new(Vec2::new(51.7, -59.9), 8.5),
            TrackWaypoint::new(Vec2::new(8.5, -60.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-20.8, -44.2), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(20.8, -39.7), 8.5),
            TrackWaypoint::new(Vec2::new(64.0, -39.8), 8.5),
            TrackWaypoint::new(Vec2::new(104.3, -32.8), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(69.9, -20.1), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(26.7, -19.9), 8.5),
            TrackWaypoint::new(Vec2::new(-16.5, -19.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-40.5, -46.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-75.7, -60.1), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-118.9, -63.6), 8.5),
            TrackWaypoint::new(Vec2::new(-161.8, -62.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-167.4, -17.4), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-136.0, -4.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-143.5, -39.5), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-100.9, -41.1), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-77.5, -12.9), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-43.2, 0.4), 8.5).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 24, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "PF International Kart Circuit (PFI)".to_string(),
            description: "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Circuito Internacional de Zuera (Zaragoza, Spain): Ultra-Fast Supertrack
    pub fn track_zuera() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(51.3, 9.7), 9.0),
            TrackWaypoint::new(Vec2::new(104.4, 10.8), 10.0),
            TrackWaypoint::new(Vec2::new(157.5, 11.9), 10.0),
            TrackWaypoint::new(Vec2::new(210.6, 13.4), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(257.6, -6.6), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(270.6, -56.5), 9.0),
            TrackWaypoint::new(Vec2::new(271.5, -109.6), 9.0),
            TrackWaypoint::new(Vec2::new(272.3, -162.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(228.6, -170.3), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(179.5, -161.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(136.1, -192.4), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(105.1, -187.9), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(144.8, -152.7), 9.0),
            TrackWaypoint::new(Vec2::new(185.0, -117.9), 9.0),
            TrackWaypoint::new(Vec2::new(225.1, -83.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(230.2, -36.1), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(183.7, -27.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(177.5, -75.6), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(129.1, -61.8), 9.0),
            TrackWaypoint::new(Vec2::new(81.3, -38.6), 9.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(40.5, -39.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(77.9, -75.4), 9.0),
            TrackWaypoint::new(Vec2::new(121.7, -103.9), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(101.2, -146.4), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(61.0, -181.1), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(14.3, -201.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-18.4, -168.6), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(18.2, -132.1), 9.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(36.5, -89.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-13.3, -86.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-27.3, -42.7), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 24, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Circuito Internacional de Zuera".to_string(),
            description: "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Le Mans Karting International (Le Mans, France): Alain Prost CIK Circuit
    pub fn track_le_mans() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.5),
            TrackWaypoint::new(Vec2::new(45.6, 5.7), 9.2),
            TrackWaypoint::new(Vec2::new(91.6, 8.5), 9.2),
            TrackWaypoint::new(Vec2::new(137.6, 11.4), 9.2),
            TrackWaypoint::new(Vec2::new(183.6, 12.4), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(217.0, -11.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(209.4, -55.3), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(179.5, -79.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(200.0, -121.1), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(167.7, -137.4), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(121.6, -138.6), 8.5),
            TrackWaypoint::new(Vec2::new(75.5, -139.9), 8.5),
            TrackWaypoint::new(Vec2::new(29.4, -140.3), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-8.5, -118.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-40.6, -149.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-71.5, -139.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-39.6, -105.9), 8.5),
            TrackWaypoint::new(Vec2::new(-1.5, -83.3), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(44.5, -80.4), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(71.4, -49.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(112.4, -49.9), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(96.9, -88.9), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(135.7, -106.6), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(151.7, -70.7), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(147.4, -24.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(107.0, -16.7), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(61.2, -22.5), 8.5),
            TrackWaypoint::new(Vec2::new(15.5, -28.7), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-17.5, -59.3), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-36.7, -27.2), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Le Mans Karting International".to_string(),
            description: "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Kartodromo Internacional do Algarve (Portimao, Portugal): Rollercoaster Track
    pub fn track_portimao() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.5),
            TrackWaypoint::new(Vec2::new(46.5, -5.1), 9.5),
            TrackWaypoint::new(Vec2::new(94.3, -3.2), 9.5),
            TrackWaypoint::new(Vec2::new(142.1, -0.4), 9.5),
            TrackWaypoint::new(Vec2::new(189.8, 2.6), 8.5),
            TrackWaypoint::new(Vec2::new(237.6, 6.1), 8.5),
            TrackWaypoint::new(Vec2::new(285.3, 9.6), 8.5),
            TrackWaypoint::new(Vec2::new(332.2, 6.5), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(371.1, -21.2), 8.5),
            TrackWaypoint::new(Vec2::new(402.7, -57.0), 8.5),
            TrackWaypoint::new(Vec2::new(432.4, -94.5), 8.5),
            TrackWaypoint::new(Vec2::new(462.4, -131.8), 8.5),
            TrackWaypoint::new(Vec2::new(498.3, -162.4), 8.5),
            TrackWaypoint::new(Vec2::new(527.8, -199.5), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(537.5, -245.6), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(525.8, -291.4), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(494.9, -327.1), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(451.8, -346.7), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(406.6, -336.8), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(368.3, -308.1), 8.5),
            TrackWaypoint::new(Vec2::new(330.1, -279.3), 8.5),
            TrackWaypoint::new(Vec2::new(291.8, -250.6), 8.5),
            TrackWaypoint::new(Vec2::new(253.6, -221.8), 8.5),
            TrackWaypoint::new(Vec2::new(215.3, -193.1), 8.5),
            TrackWaypoint::new(Vec2::new(177.1, -164.4), 8.5),
            TrackWaypoint::new(Vec2::new(138.8, -135.6), 8.5),
            TrackWaypoint::new(Vec2::new(100.6, -106.9), 8.5),
            TrackWaypoint::new(Vec2::new(62.3, -78.1), 8.5),
            TrackWaypoint::new(Vec2::new(24.1, -49.4), 8.5),
            TrackWaypoint::new(Vec2::new(-14.2, -20.6), 8.5),
            TrackWaypoint::new(Vec2::new(-51.7, 8.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-38.3, 28.7), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 24, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Kartodromo Internacional do Algarve".to_string(),
            description: "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
    }
    /// Franciacorta Karting Track (Castrezzato, Italy): World Championship Benchmark
    pub fn track_franciacorta() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(42.7, -5.0), 9.2),
            TrackWaypoint::new(Vec2::new(86.0, -6.4), 9.2),
            TrackWaypoint::new(Vec2::new(128.5, -2.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(136.7, 37.8), 8.5),
            TrackWaypoint::new(Vec2::new(137.7, 81.1), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(101.9, 100.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(60.3, 96.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(70.2, 64.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(113.3, 61.7), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(111.4, 22.5), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(68.9, 18.6), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(29.5, 29.1), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(28.2, 71.1), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(12.0, 102.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-31.3, 103.3), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-63.8, 85.3), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-71.3, 42.8), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-98.6, 21.3), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-110.4, 46.2), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-94.3, 83.7), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-122.9, 108.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-147.3, 82.8), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-149.2, 39.5), 8.5),
            TrackWaypoint::new(Vec2::new(-147.9, -3.2), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-105.7, -10.1), 8.5),
            TrackWaypoint::new(Vec2::new(-62.4, -11.5), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-48.6, 26.6), 8.5),
            TrackWaypoint::new(Vec2::new(-32.3, 63.9), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-5.3, 42.7), 8.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        Track {
            name: "Franciacorta Karting Track".to_string(),
            description: "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("shifter_kart_125".to_string()),
            module_id: Some("kart".to_string()),
            modules: vec!["kart".to_string()],
        }
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
        "KARTING WORLD CUP"
    }

    fn subtitle(&self) -> &'static str {
        "Direct-Drive 125cc Shifter Karts, 3G Apex Cornering & Wheel-to-Wheel Heats"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(0.30, 0.95, 0.40, 1.0), // Electric Kart Green
            secondary_accent: Color::new(1.0, 0.85, 0.20, 1.0), // Racing Yellow
            header_badge: "KARTING WORLD CUP",
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
                id: "lonato",
                title: "South Garda Karting (Lonato)",
                tag: "MECCA OF KARTING",
                description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_lonato,
            },
            TrackDefinition {
                id: "sarno",
                title: "Circuito Internazionale Napoli (Sarno)",
                tag: "TEMPLE OF SPEED",
                description: "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_sarno,
            },
            TrackDefinition {
                id: "genk",
                title: "Karting Genk (Home of Champions)",
                tag: "HOME OF CHAMPIONS",
                description: "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_genk,
            },
            TrackDefinition {
                id: "pfi",
                title: "PF International Kart Circuit",
                tag: "FLYOVER CROSSOVER",
                description: "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_pfi,
            },
            TrackDefinition {
                id: "zuera",
                title: "Circuito Internacional de Zuera",
                tag: "SPANISH SUPERTRACK",
                description: "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_zuera,
            },
            TrackDefinition {
                id: "le_mans_kart",
                title: "Le Mans Karting International",
                tag: "24H LE MANS ARENA",
                description: "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_le_mans,
            },
            TrackDefinition {
                id: "portimao_kart",
                title: "Kartodromo Internacional do Algarve",
                tag: "ALGARVE ROLLERCOASTER",
                description: "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_portimao,
            },
            TrackDefinition {
                id: "franciacorta",
                title: "Franciacorta Karting Track",
                tag: "CHAMPIONSHIP BENCHMARK",
                description: "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.",
                category: "World Championship",
                default_laps: 6,
                generator: Self::track_franciacorta,
            },
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
        "lonato"
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
                color_scheme: CarColorScheme::from_index(6),
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
                name: "Karting World Cup".to_string(),
                point_system: PointSystem::ClassicArcade,
                track_ids: vec![
                    "lonato".to_string(),
                    "genk".to_string(),
                    "pfi".to_string(),
                    "sarno".to_string(),
                    "zuera".to_string(),
                    "le_mans_kart".to_string(),
                    "portimao_kart".to_string(),
                    "franciacorta".to_string(),
                    "kart_arena".to_string(),
                    "drift_park".to_string(),
                ],
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
