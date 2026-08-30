use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, Obstacle, TrackGeometry};
use tdrace_core::track::presets::{
    drift_park, generate_checkpoints, generate_grid_positions, generate_walls_from_spline, kart_arena,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{Track, TrackCategory};

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

    /// South Garda Karting (Lonato, Italy): The Mecca of International Karting
    pub fn track_lonato() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 9.0),
            // Curva del Paddock (Fast sweeping right)
            TrackWaypoint::new(Vec2::new(90.0, 15.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(115.0, 45.0), 8.5).with_curbs(false, true),
            // Variante Esse (Left-Right chicane complex)
            TrackWaypoint::new(Vec2::new(110.0, 80.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(85.0, 105.0), 8.0).with_curbs(false, true),
            // Approach Straight to Pettine Hairpin
            TrackWaypoint::new(Vec2::new(85.0, 140.0), 8.5),
            // Curva del Pettine (Hairpin 1 - tight right)
            TrackWaypoint::new(Vec2::new(65.0, 175.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(35.0, 175.0), 8.0).with_curbs(false, true),
            // Infield Downhill Sprint
            TrackWaypoint::new(Vec2::new(20.0, 140.0), 8.5).with_curbs(false, true),
            // Variante Nuova (Fast switchback)
            TrackWaypoint::new(Vec2::new(5.0, 105.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-20.0, 95.0), 8.0).with_curbs(false, true),
            // Curva dei Meccanici (Paddock Hairpin - technical left)
            TrackWaypoint::new(Vec2::new(-50.0, 110.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-75.0, 85.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-70.0, 50.0), 8.5).with_curbs(true, false),
            // Final Acceleration Sweeper onto Main Straight
            TrackWaypoint::new(Vec2::new(-50.0, 20.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-25.0, 0.0), 9.0),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(92.0, 12.0), 1.0, "Paddock Apex Curb"),
            Obstacle::circle(2, Vec2::new(-76.0, 86.0), 1.0, "Meccanici Hairpin Apex"),
        ];

        Track {
            name: "South Garda Karting (Lonato)".to_string(),
            description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// Circuito Internazionale Napoli (Sarno, Italy): The High-Speed Temple of Speed
    pub fn track_sarno() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish (140m+ high speed straight)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.5),
            TrackWaypoint::new(Vec2::new(70.0, 0.0), 9.5),
            TrackWaypoint::new(Vec2::new(140.0, 0.0), 9.5),
            // Turn 1 & 2 (High speed sweeping right)
            TrackWaypoint::new(Vec2::new(195.0, 20.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(220.0, 60.0), 9.0).with_curbs(false, true),
            // Turn 3 Sweeper Left
            TrackWaypoint::new(Vec2::new(200.0, 105.0), 8.5).with_curbs(true, false),
            // Back Straight Sprint
            TrackWaypoint::new(Vec2::new(150.0, 130.0), 9.0),
            TrackWaypoint::new(Vec2::new(90.0, 140.0), 9.0),
            // Curva Vesuvio (Heavy braking hairpin)
            TrackWaypoint::new(Vec2::new(35.0, 160.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(0.0, 140.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-10.0, 105.0), 8.5).with_curbs(false, true),
            // Sarno Technical Esses (Tight left-right flick)
            TrackWaypoint::new(Vec2::new(-35.0, 80.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-65.0, 95.0), 8.0).with_curbs(false, true),
            // Double Apex Carousel onto Home Stretch
            TrackWaypoint::new(Vec2::new(-100.0, 80.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-105.0, 45.0), 8.5).with_curbs(true, false),
            // Final Chicane & Acceleration
            TrackWaypoint::new(Vec2::new(-80.0, 15.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-40.0, 0.0), 9.0),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(0.0, 138.0), 1.0, "Vesuvio Hairpin Apex"),
            Obstacle::circle(2, Vec2::new(-35.0, 82.0), 1.0, "Sarno Esses Bollard"),
        ];

        Track {
            name: "Circuito Internazionale Napoli (Sarno)".to_string(),
            description: "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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
            // Pit Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.0),
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 9.0),
            // Turn 1 Fast Right Kink
            TrackWaypoint::new(Vec2::new(105.0, 15.0), 9.0).with_curbs(false, true),
            // The Legendary G-Curve (Multi-apex high-G sweeping right)
            TrackWaypoint::new(Vec2::new(140.0, 45.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(155.0, 85.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(140.0, 125.0), 9.0).with_curbs(false, true),
            // North Loop Transition
            TrackWaypoint::new(Vec2::new(100.0, 150.0), 8.5).with_curbs(true, false),
            // Europabocht Hairpin (Heavy braking technical hairpin)
            TrackWaypoint::new(Vec2::new(55.0, 170.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(20.0, 155.0), 8.0).with_curbs(false, true),
            // Downhill Infield Straight
            TrackWaypoint::new(Vec2::new(10.0, 120.0), 8.5),
            // Chicane des Champions (Left-Right rhythm section)
            TrackWaypoint::new(Vec2::new(-5.0, 85.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-30.0, 75.0), 8.0).with_curbs(false, true),
            // Infield Technical Hairpin Complex
            TrackWaypoint::new(Vec2::new(-60.0, 95.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-85.0, 70.0), 8.5).with_curbs(true, false),
            // Fast Esses onto Final Sweeper
            TrackWaypoint::new(Vec2::new(-75.0, 30.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-40.0, 10.0), 9.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(157.0, 85.0), 1.0, "G-Curve Apex Pylon"),
            Obstacle::circle(2, Vec2::new(21.0, 153.0), 1.0, "Europabocht Hairpin Curb"),
        ];

        Track {
            name: "Karting Genk (Home of Champions)".to_string(),
            description: "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// PF International Kart Circuit (PFI, UK): The Iconic Flyover Crossover Bridge Circuit
    pub fn track_pfi() -> Track {
        let waypoints = vec![
            // Main Grandstand Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(-30.0, 0.0), 8.5),
            TrackWaypoint::new(Vec2::new(40.0, 0.0), 8.5),
            // Turn 1 Fast Left
            TrackWaypoint::new(Vec2::new(85.0, 15.0), 8.5).with_curbs(true, false),
            // Bruno's Hairpin (Sharp right)
            TrackWaypoint::new(Vec2::new(110.0, 50.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(95.0, 85.0), 8.0).with_curbs(false, true),
            // Bridge Approach Straight
            TrackWaypoint::new(Vec2::new(60.0, 90.0), 8.5),
            // PFI Flyover Crossover Bridge (Climbs to 4.0m elevation crossing over the lower underpass)
            TrackWaypoint::new(Vec2::new(30.0, 90.0), 8.5).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(0.0, 90.0), 8.5).with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(-30.0, 90.0), 8.5).with_elevation(2.0),
            // Descent into North Infield
            TrackWaypoint::new(Vec2::new(-65.0, 105.0), 8.0).with_curbs(false, true),
            // Far North Loop Hairpin
            TrackWaypoint::new(Vec2::new(-85.0, 140.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-60.0, 165.0), 8.0).with_curbs(true, false),
            // North Straight heading to Underpass
            TrackWaypoint::new(Vec2::new(-25.0, 150.0), 8.5),
            // Lower Underpass (Crosses beneath the flyover bridge at elevation 0.0)
            TrackWaypoint::new(Vec2::new(0.0, 90.0), 8.5),
            // Underpass Exit Sweeper
            TrackWaypoint::new(Vec2::new(20.0, 45.0), 8.5).with_curbs(false, true),
            // Final Chicane onto Start Straight
            TrackWaypoint::new(Vec2::new(5.0, 15.0), 8.5).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(112.0, 52.0), 1.0, "Bruno's Hairpin Apex"),
            Obstacle::circle(2, Vec2::new(-87.0, 142.0), 1.0, "North Loop Hairpin Apex"),
        ];

        Track {
            name: "PF International Kart Circuit (PFI)".to_string(),
            description: "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// Circuito Internacional de Zuera (Zaragoza, Spain): High-Speed Spanish Supertrack
    pub fn track_zuera() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish (Ultra-wide 160m drafting straight)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(80.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 10.0),
            // Curva del Cierzo (Wide high-speed right sweeper)
            TrackWaypoint::new(Vec2::new(215.0, 25.0), 9.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(240.0, 75.0), 9.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(220.0, 130.0), 9.5).with_curbs(false, true),
            // North Hairpin (Left hairpin onto infield straight)
            TrackWaypoint::new(Vec2::new(170.0, 165.0), 9.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(120.0, 175.0), 9.0).with_curbs(true, false),
            // Infield High-Speed Straight
            TrackWaypoint::new(Vec2::new(80.0, 140.0), 9.0),
            TrackWaypoint::new(Vec2::new(40.0, 110.0), 9.0).with_curbs(false, true),
            // West Technical Hairpin Complex
            TrackWaypoint::new(Vec2::new(-10.0, 130.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-50.0, 140.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-85.0, 115.0), 9.0).with_curbs(true, false),
            // Long Western Straight
            TrackWaypoint::new(Vec2::new(-95.0, 70.0), 9.5),
            // Final Sweeping Double Right onto Main Straight
            TrackWaypoint::new(Vec2::new(-80.0, 25.0), 9.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-45.0, 5.0), 10.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(242.0, 75.0), 1.0, "Cierzo Sweeper Pylon"),
            Obstacle::circle(2, Vec2::new(-52.0, 142.0), 1.0, "West Hairpin Apex"),
        ];

        Track {
            name: "Circuito Internacional de Zuera".to_string(),
            description: "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// Le Mans Karting International (Le Mans, France): Alain Prost 24H Karting Circuit
    pub fn track_le_mans() -> Track {
        let waypoints = vec![
            // Pit Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.0),
            TrackWaypoint::new(Vec2::new(65.0, 0.0), 9.0),
            // Dunlop Chicane Tribute (Quick left-right chicane)
            TrackWaypoint::new(Vec2::new(105.0, 15.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(130.0, 40.0), 8.5).with_curbs(false, true),
            // La Chapelle Sweeper
            TrackWaypoint::new(Vec2::new(145.0, 80.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(130.0, 120.0), 9.0).with_curbs(true, false),
            // Bugatti Esses (Flowing switchbacks)
            TrackWaypoint::new(Vec2::new(95.0, 145.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(55.0, 140.0), 8.5).with_curbs(false, true),
            // Virage du Raccordement Hairpin
            TrackWaypoint::new(Vec2::new(20.0, 160.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-15.0, 150.0), 8.0).with_curbs(true, false),
            // Downhill Return Straight
            TrackWaypoint::new(Vec2::new(-30.0, 115.0), 8.5),
            // Courbe des 24 Heures (Double-apex fast left)
            TrackWaypoint::new(Vec2::new(-50.0, 80.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-80.0, 55.0), 8.5).with_curbs(true, false),
            // Maison Blanche Chicane onto Pit Straight
            TrackWaypoint::new(Vec2::new(-80.0, 20.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-45.0, 5.0), 9.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(107.0, 13.0), 1.0, "Dunlop Chicane Kerb"),
            Obstacle::circle(2, Vec2::new(-17.0, 152.0), 1.0, "Raccordement Hairpin Apex"),
        ];

        Track {
            name: "Le Mans Karting International".to_string(),
            description: "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// Kartódromo Internacional do Algarve (Portimão, Portugal): Algarve Rollercoaster
    pub fn track_portimao() -> Track {
        let waypoints = vec![
            // Pit Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.0),
            TrackWaypoint::new(Vec2::new(65.0, 0.0), 9.0),
            // Downhill Turn 1 & 2 Sweeping Right
            TrackWaypoint::new(Vec2::new(115.0, 15.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(150.0, 45.0), 9.0).with_curbs(false, true),
            // Uphill Climb into Crest
            TrackWaypoint::new(Vec2::new(160.0, 90.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(140.0, 135.0), 8.5).with_curbs(true, false),
            // Algarve Hairpin (Sharp right)
            TrackWaypoint::new(Vec2::new(100.0, 160.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(60.0, 165.0), 8.0).with_curbs(false, true),
            // Infield Downhill Sprint
            TrackWaypoint::new(Vec2::new(30.0, 135.0), 8.5),
            // Curva do Sol Carousel
            TrackWaypoint::new(Vec2::new(0.0, 110.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-25.0, 125.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-60.0, 115.0), 8.5).with_curbs(true, false),
            // Switchback Chicane onto Home Stretch
            TrackWaypoint::new(Vec2::new(-85.0, 80.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-80.0, 40.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-50.0, 15.0), 9.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(60.0, 163.0), 1.0, "Algarve Hairpin Apex"),
            Obstacle::circle(2, Vec2::new(-85.0, 78.0), 1.0, "Portimao Chicane Kerb"),
        ];

        Track {
            name: "Kartodromo Internacional do Algarve".to_string(),
            description: "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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

    /// Franciacorta Karting Track (Castrezzato, Italy): Modern Championship Benchmark
    pub fn track_franciacorta() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 9.0),
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 9.0),
            // Turn 1 & 2 Rapid Chicane
            TrackWaypoint::new(Vec2::new(105.0, 15.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(130.0, 45.0), 8.5).with_curbs(true, false),
            // Outer Sweeping Arc
            TrackWaypoint::new(Vec2::new(140.0, 85.0), 9.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(125.0, 125.0), 9.0).with_curbs(false, true),
            // Curva Franciacorta Hairpin (Trail braking left)
            TrackWaypoint::new(Vec2::new(90.0, 150.0), 8.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(50.0, 150.0), 8.0).with_curbs(true, false),
            // Central Infield Straight
            TrackWaypoint::new(Vec2::new(30.0, 115.0), 8.5),
            // Technical Switchback Chicane
            TrackWaypoint::new(Vec2::new(10.0, 80.0), 8.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-15.0, 65.0), 8.0).with_curbs(true, false),
            // West Hairpin Left
            TrackWaypoint::new(Vec2::new(-45.0, 85.0), 8.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-75.0, 75.0), 8.5).with_curbs(true, false),
            // Fast Right Sweeper onto Finish Straight
            TrackWaypoint::new(Vec2::new(-80.0, 35.0), 8.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-45.0, 10.0), 9.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 16, 5.5, 1.8);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(52.0, 148.0), 1.0, "Franciacorta Hairpin Apex"),
            Obstacle::circle(2, Vec2::new(10.0, 82.0), 1.0, "Infield Chicane Kerb"),
        ];

        Track {
            name: "Franciacorta Karting Track".to_string(),
            description: "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.".to_string(),
            category: TrackCategory::Main,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles,
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
                name: "FIA World Karting Championship".to_string(),
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
