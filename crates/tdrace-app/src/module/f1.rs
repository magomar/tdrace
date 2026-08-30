use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::config::{CarConfig, DriverAssistsConfig, TireConfig};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, Obstacle, TrackGeometry};
use tdrace_core::track::presets::{classic_grand_prix, generate_checkpoints, generate_grid_positions, generate_walls_from_spline};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{Track, TrackCategory};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// Formula 1 World Championship Game Module
pub struct F1GameModule;

impl F1GameModule {
    pub fn new() -> Self {
        Self
    }

    /// Monza Autodromo: The High-Speed Temple of Speed
    pub fn track_monza() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 15.0),
            TrackWaypoint::new(Vec2::new(260.0, 0.0), 15.0),
            // Variante del Rettifilo (Tight Turn 1-2 Chicane)
            TrackWaypoint::new(Vec2::new(340.0, 15.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(380.0, -10.0), 13.0).with_curbs(false, true),
            // Curva Grande (High speed sweeping right)
            TrackWaypoint::new(Vec2::new(490.0, 60.0), 14.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(540.0, 170.0), 14.0).with_curbs(true, false),
            // Variante della Roggia (Turn 4-5 Chicane)
            TrackWaypoint::new(Vec2::new(510.0, 260.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(470.0, 290.0), 13.0).with_curbs(true, false),
            // Curva di Lesmo 1 & 2
            TrackWaypoint::new(Vec2::new(380.0, 320.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(300.0, 330.0), 13.0).with_curbs(true, false),
            // Serraglio & Approach to Ascari
            TrackWaypoint::new(Vec2::new(180.0, 280.0), 14.0),
            TrackWaypoint::new(Vec2::new(60.0, 230.0), 14.0),
            // Variante Ascari (Fast Left-Right-Left Complex)
            TrackWaypoint::new(Vec2::new(-20.0, 190.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-60.0, 220.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-110.0, 200.0), 13.0).with_curbs(true, false),
            // Back Straight into Parabolica (Curva Alboreto)
            TrackWaypoint::new(Vec2::new(-160.0, 120.0), 14.0),
            TrackWaypoint::new(Vec2::new(-150.0, 30.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-80.0, -20.0), 15.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 15.0),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(345.0, 10.0), 1.2, "Rettifilo Foam Barrier"),
            Obstacle::circle(2, Vec2::new(515.0, 255.0), 1.2, "Roggia Chicane Bollard"),
        ];

        Track {
            name: "Monza Autodromo Nazionale".to_string(),
            description: "High-speed Italian Grand Prix temple of speed.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Spa-Francorchamps: Circuit de Spa-Francorchamps (Eau Rouge, Kemmel, Pouhon, Blanchimont)
    pub fn track_spa() -> Track {
        let waypoints = vec![
            // La Source Hairpin to Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(120.0, -10.0), 14.0),
            // Eau Rouge & Raidillon (Steep Left-Right uphill crest)
            TrackWaypoint::new(Vec2::new(200.0, 30.0), 15.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(260.0, 70.0), 15.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(310.0, 100.0), 15.0).with_curbs(true, false),
            // Kemmel Straight (Top speed drag strip)
            TrackWaypoint::new(Vec2::new(440.0, 160.0), 15.0),
            TrackWaypoint::new(Vec2::new(560.0, 220.0), 15.0),
            // Les Combes & Malmedy Chicane
            TrackWaypoint::new(Vec2::new(620.0, 260.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(640.0, 310.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(600.0, 370.0), 13.0).with_curbs(false, true),
            // Bruxelles & Speaker's Corner
            TrackWaypoint::new(Vec2::new(510.0, 420.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(430.0, 440.0), 13.0).with_curbs(true, false),
            // Pouhon (High-speed double-apex left)
            TrackWaypoint::new(Vec2::new(340.0, 380.0), 14.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(260.0, 350.0), 14.0).with_curbs(true, false),
            // Campus & Stavelot
            TrackWaypoint::new(Vec2::new(170.0, 370.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(100.0, 340.0), 14.0).with_curbs(true, false),
            // Blanchimont (Flat-out left sweeping curve)
            TrackWaypoint::new(Vec2::new(10.0, 260.0), 15.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-60.0, 170.0), 15.0).with_curbs(true, false),
            // Bus Stop Chicane onto Main Straight
            TrackWaypoint::new(Vec2::new(-80.0, 90.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-50.0, 30.0), 13.0).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de Spa-Francorchamps".to_string(),
            description: "Belgian Ardennes rollercoaster featuring Eau Rouge and Pouhon.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Silverstone GP: The Home of British Motor Racing (Maggotts, Becketts, Chapel, Stowe, Club)
    pub fn track_silverstone() -> Track {
        let waypoints = vec![
            // Hamilton Straight (Start/Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(140.0, 0.0), 14.0),
            // Abbey & Farm Curve
            TrackWaypoint::new(Vec2::new(220.0, 30.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(280.0, 80.0), 14.0).with_curbs(true, false),
            // Village & The Loop (Tight infield chicane & hairpin)
            TrackWaypoint::new(Vec2::new(260.0, 160.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(190.0, 180.0), 12.0).with_curbs(true, false),
            // Aintree onto Wellington Straight
            TrackWaypoint::new(Vec2::new(140.0, 130.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(60.0, 120.0), 14.0),
            TrackWaypoint::new(Vec2::new(-60.0, 110.0), 14.0),
            // Brooklands & Luffield
            TrackWaypoint::new(Vec2::new(-140.0, 150.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-170.0, 220.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-120.0, 270.0), 13.0).with_curbs(true, false),
            // Copse Corner (High-speed 7th gear right)
            TrackWaypoint::new(Vec2::new(0.0, 300.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(100.0, 350.0), 14.0).with_curbs(false, true),
            // Maggotts, Becketts & Chapel Complex
            TrackWaypoint::new(Vec2::new(180.0, 420.0), 14.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(240.0, 460.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(290.0, 430.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(320.0, 360.0), 14.0).with_curbs(false, true),
            // Hangar Straight Overpass Bridge (Passes elevated over Abbey & Farm Curve)
            TrackWaypoint::new(Vec2::new(280.0, 240.0), 15.0).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(230.0, 100.0), 15.0).with_elevation(5.0),
            // Stowe Corner Descent Ramp
            TrackWaypoint::new(Vec2::new(170.0, -40.0), 13.0)
                .with_curbs(false, true)
                .with_elevation(2.0),
            // Vale & Club Corner onto Main Straight
            TrackWaypoint::new(Vec2::new(80.0, -80.0), 12.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-30.0, -60.0), 13.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Silverstone Grand Prix Circuit".to_string(),
            description: "High-speed sweeping esses through Maggotts, Becketts and Chapel.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Circuit de Monaco: Monte Carlo Street Circuit (Sainte Devote, Casino Square, Loews Hairpin, Tunnel, Swimming Pool, Rascasse)
    pub fn track_monaco() -> Track {
        let waypoints = vec![
            // Boulevard Albert 1er (Start / Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
            // Sainte-Devote (Turn 1 right)
            TrackWaypoint::new(Vec2::new(160.0, 15.0), 11.0).with_curbs(true, false),
            // Beau Rivage uphill climb
            TrackWaypoint::new(Vec2::new(190.0, 60.0), 12.0),
            TrackWaypoint::new(Vec2::new(200.0, 130.0), 12.0),
            // Massenet (sweeping left around Hotel de Paris)
            TrackWaypoint::new(Vec2::new(180.0, 190.0), 12.0).with_curbs(true, false),
            // Casino Square (sharp right)
            TrackWaypoint::new(Vec2::new(140.0, 220.0), 11.5).with_curbs(false, true),
            // Mirabeau Haute (downhill right)
            TrackWaypoint::new(Vec2::new(90.0, 230.0), 11.5).with_curbs(false, true),
            // Fairmont / Loews Hairpin (tightest hairpin in F1)
            TrackWaypoint::new(Vec2::new(50.0, 210.0), 11.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(40.0, 175.0), 11.0).with_curbs(true, true),
            // Mirabeau Bas & Portier (heading to tunnel)
            TrackWaypoint::new(Vec2::new(70.0, 145.0), 11.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(110.0, 125.0), 12.0).with_curbs(false, true),
            // The Tunnel (high speed right sweeper)
            TrackWaypoint::new(Vec2::new(140.0, 95.0), 13.0),
            TrackWaypoint::new(Vec2::new(145.0, 45.0), 13.0),
            // Nouvelle Chicane (heavy braking off tunnel descent)
            TrackWaypoint::new(Vec2::new(120.0, -10.0), 11.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(80.0, -30.0), 11.0).with_curbs(false, true),
            // Tabac Corner (fast left along harbor)
            TrackWaypoint::new(Vec2::new(20.0, -45.0), 12.0).with_curbs(true, false),
            // Louis Chiron Swimming Pool complex (Piscine chicane 1 & 2)
            TrackWaypoint::new(Vec2::new(-30.0, -50.0), 12.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-80.0, -60.0), 11.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.0, -50.0), 11.5).with_curbs(true, false),
            // La Rascasse (tight hairpin right)
            TrackWaypoint::new(Vec2::new(-130.0, -15.0), 11.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-90.0, 15.0), 11.0).with_curbs(false, true),
            // Anthony Noghes (final right onto start straight)
            TrackWaypoint::new(Vec2::new(-40.0, 10.0), 11.5).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(122.0, -8.0), 1.0, "Nouvelle Chicane Apex Curb"),
            Obstacle::circle(2, Vec2::new(-80.0, -58.0), 1.0, "Piscine Chicane Kerb"),
        ];

        Track {
            name: "Circuit de Monaco".to_string(),
            description: "Legendary Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Suzuka International Racing Course: Figure-8 Japanese Masterpiece (S-Curves, Degner, 130R, Casio Triangle)
    pub fn track_suzuka() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish Line (Gantry located 70m down the straight)
            TrackWaypoint::new(Vec2::new(70.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(150.0, 0.0), 14.0),
            // First Corner (Turns 1-2 double right)
            TrackWaypoint::new(Vec2::new(215.0, 15.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(250.0, 55.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(250.0, 100.0), 13.0).with_curbs(false, true),
            // S-Curves (Esses: Turns 3, 4, 5, 6)
            TrackWaypoint::new(Vec2::new(225.0, 145.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(190.0, 185.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(210.0, 230.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(180.0, 275.0), 13.0).with_curbs(false, true),
            // Dunlop Curve (Turn 7 uphill sweeping left)
            TrackWaypoint::new(Vec2::new(130.0, 310.0), 13.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(65.0, 315.0), 13.5).with_curbs(true, false),
            // Degner 1 & 2 (Turns 8-9)
            TrackWaypoint::new(Vec2::new(25.0, 290.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(5.0, 245.0), 12.5).with_curbs(false, true),
            // Underpass passage (crossing under the elevated back straight at elevation 0.0)
            TrackWaypoint::new(Vec2::new(0.0, 175.0), 13.0),
            TrackWaypoint::new(Vec2::new(-15.0, 110.0), 13.0),
            // Hairpin (Turn 11 slow hairpin right)
            TrackWaypoint::new(Vec2::new(-40.0, 65.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-80.0, 55.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.0, 65.0), 12.0).with_curbs(false, true),
            // 200R Sweeping Curve (Turn 12)
            TrackWaypoint::new(Vec2::new(-165.0, 90.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-215.0, 130.0), 14.0).with_curbs(false, true),
            // Spoon Curve (Turns 13-14 double-apex left)
            TrackWaypoint::new(Vec2::new(-260.0, 185.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-280.0, 240.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-245.0, 275.0), 13.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-185.0, 265.0), 14.0).with_curbs(true, false),
            // Back Straight Crossover Bridge (climbs to 5.0m elevation crossing over the underpass at y: 175)
            TrackWaypoint::new(Vec2::new(-120.0, 235.0), 14.0).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(-60.0, 205.0), 14.5).with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(0.0, 175.0), 15.0).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(60.0, 150.0), 14.5).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(100.0, 120.0), 14.0).with_elevation(1.0),
            // Back Straight approach to 130R
            TrackWaypoint::new(Vec2::new(110.0, 75.0), 14.5),
            // 130R (Turn 15 legendary high-speed flat-out left)
            TrackWaypoint::new(Vec2::new(80.0, 30.0), 14.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(25.0, 15.0), 14.5).with_curbs(true, false),
            // Casio Triangle Chicane (Turns 16-17)
            TrackWaypoint::new(Vec2::new(-30.0, 15.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-75.0, 20.0), 12.0).with_curbs(true, false),
            // Final Corner (Turn 18 sweeping right onto main straight)
            TrackWaypoint::new(Vec2::new(-125.0, 18.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-155.0, 0.0), 14.0).with_curbs(false, true),
            // Main Straight approach to start line
            TrackWaypoint::new(Vec2::new(-80.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(-30.0, 9.0), 1.2, "Casio Triangle Foam Bollard"),
        ];

        Track {
            name: "Suzuka International Racing Course".to_string(),
            description: "Iconic Japanese figure-8 layout featuring Esses, Degner, overpass crossover bridge, and 130R.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Autodromo Jose Carlos Pace (Interlagos): Brazilian Anti-Clockwise Classic (Senna S, Curva do Sol, Juncao)
    pub fn track_interlagos() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(-120.0, 0.0), 14.0),
            // Senna 'S' (Turns 1-2 downhill left-right chicane)
            TrackWaypoint::new(Vec2::new(-180.0, 15.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-220.0, 45.0), 13.0).with_curbs(false, true),
            // Curva do Sol (Turn 3 long sweeping left)
            TrackWaypoint::new(Vec2::new(-230.0, 110.0), 14.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-200.0, 170.0), 14.0).with_curbs(true, false),
            // Reta Oposta (Back straight)
            TrackWaypoint::new(Vec2::new(-130.0, 220.0), 14.5),
            TrackWaypoint::new(Vec2::new(-30.0, 270.0), 14.5),
            // Descida do Lago (Turns 4-5 fast double-left)
            TrackWaypoint::new(Vec2::new(40.0, 300.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(90.0, 280.0), 13.0).with_curbs(true, false),
            // Ferradura (Turns 6-7 long sweeping right carousel)
            TrackWaypoint::new(Vec2::new(110.0, 220.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(140.0, 160.0), 13.5).with_curbs(false, true),
            // Pinheirinho (Turn 8 tight technical left)
            TrackWaypoint::new(Vec2::new(120.0, 110.0), 12.5).with_curbs(true, false),
            // Bico de Pato (Turn 10 slow hairpin right)
            TrackWaypoint::new(Vec2::new(80.0, 90.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(50.0, 70.0), 12.0).with_curbs(false, true),
            // Mergulho (Turn 11 fast downhill left)
            TrackWaypoint::new(Vec2::new(70.0, 30.0), 13.5).with_curbs(true, false),
            // Juncao (Turn 12 crucial left onto uphill climb)
            TrackWaypoint::new(Vec2::new(110.0, -10.0), 13.0).with_curbs(true, false),
            // Subida dos Boxes & Arquibancadas (uphill sweeping left onto start straight)
            TrackWaypoint::new(Vec2::new(120.0, -70.0), 14.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(70.0, -60.0), 14.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(20.0, -25.0), 14.5),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Autodromo Jose Carlos Pace (Interlagos)".to_string(),
            description: "Thrilling anti-clockwise Brazilian Grand Prix circuit with Senna 'S', Ferradura, and Juncao.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Circuit Gilles Villeneuve (Montreal): Ile Notre-Dame Island Circuit (Virage Senna, L'Epingle, Wall of Champions)
    pub fn track_montreal() -> Track {
        let waypoints = vec![
            // Pit Straight (Start / Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(140.0, 0.0), 14.0),
            // Virage Senna (Turns 1-2 tight left-right chicane)
            TrackWaypoint::new(Vec2::new(200.0, 25.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(220.0, 70.0), 12.5).with_curbs(false, true),
            // Turns 3-4 chicane (right-left)
            TrackWaypoint::new(Vec2::new(190.0, 130.0), 12.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(160.0, 180.0), 12.5).with_curbs(true, false),
            // Pont de la Concorde & Turns 6-7 chicane (left-right)
            TrackWaypoint::new(Vec2::new(120.0, 240.0), 13.5),
            TrackWaypoint::new(Vec2::new(70.0, 300.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(40.0, 350.0), 13.0).with_curbs(false, true),
            // Droit du Casino Kink (Turns 8-9 chicane)
            TrackWaypoint::new(Vec2::new(0.0, 420.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-30.0, 470.0), 13.0).with_curbs(true, false),
            // L'Epingle Hairpin (Turn 10 180-degree heavy braking hairpin)
            TrackWaypoint::new(Vec2::new(-80.0, 520.0), 12.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.0, 500.0), 12.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-110.0, 440.0), 13.0).with_curbs(false, true),
            // Droit du Casino Straight (High-speed 335+ km/h drag strip)
            TrackWaypoint::new(Vec2::new(-90.0, 320.0), 14.5),
            TrackWaypoint::new(Vec2::new(-70.0, 180.0), 14.5),
            TrackWaypoint::new(Vec2::new(-50.0, 70.0), 14.5),
            // Wall of Champions Chicane (Turns 13-14 tight right-left chicane)
            TrackWaypoint::new(Vec2::new(-35.0, 20.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-15.0, -10.0), 12.0).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Concrete);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(-10.0, -16.0), 1.2, "Wall of Champions Barrier"),
        ];

        Track {
            name: "Circuit Gilles Villeneuve (Montreal)".to_string(),
            description: "High-speed Canadian island circuit featuring Virage Senna, L'Epingle hairpin, and Wall of Champions.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Red Bull Ring: Austrian Alpine High-Speed Rollercoaster (Niki Lauda Kurve, Remus Hairpin, Schlossgold)
    pub fn track_red_bull_ring() -> Track {
        let waypoints = vec![
            // Start/Finish Straight
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(130.0, 0.0), 14.0),
            // Niki Lauda Kurve (Turn 1 uphill 90-degree right)
            TrackWaypoint::new(Vec2::new(190.0, 25.0), 13.0).with_curbs(false, true),
            // Long uphill run to Remus
            TrackWaypoint::new(Vec2::new(260.0, 100.0), 14.0),
            TrackWaypoint::new(Vec2::new(320.0, 190.0), 14.0),
            // Remus Hairpin (Turn 3 tight uphill right hairpin)
            TrackWaypoint::new(Vec2::new(340.0, 260.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(310.0, 280.0), 12.0).with_curbs(false, true),
            // Downhill braking run into Schlossgold
            TrackWaypoint::new(Vec2::new(230.0, 240.0), 14.0),
            TrackWaypoint::new(Vec2::new(150.0, 200.0), 14.0),
            // Schlossgold (Turn 4 downhill heavy braking right)
            TrackWaypoint::new(Vec2::new(90.0, 180.0), 13.0).with_curbs(false, true),
            // Rauch & Wurth (Turns 6-7 fast downhill sweeping lefts)
            TrackWaypoint::new(Vec2::new(30.0, 150.0), 13.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-20.0, 110.0), 13.5).with_curbs(true, false),
            // Turn 8 (fast right sweeper)
            TrackWaypoint::new(Vec2::new(-40.0, 60.0), 14.0).with_curbs(false, true),
            // Jochen Rindt (Turn 9 fast downhill right)
            TrackWaypoint::new(Vec2::new(-50.0, 20.0), 13.5).with_curbs(false, true),
            // Red Bull Mobile (Turn 10 final fast right onto main straight)
            TrackWaypoint::new(Vec2::new(-30.0, -20.0), 14.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 16, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Red Bull Ring (Spielberg)".to_string(),
            description: "High-speed Austrian alpine circuit with steep uphill climbs and heavy downhill braking into Remus.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Circuit de Barcelona-Catalunya: Aerodynamic Testing Benchmark (Elf, Curva Renault, Repsol, Campsa)
    pub fn track_catalunya() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(150.0, 0.0), 14.0),
            // Elf Chicane (Turns 1-2 right-left)
            TrackWaypoint::new(Vec2::new(210.0, 20.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(240.0, 55.0), 13.0).with_curbs(true, false),
            // Curva Renault (Turn 3 long high-speed sweeping right)
            TrackWaypoint::new(Vec2::new(270.0, 110.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(270.0, 170.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(230.0, 210.0), 14.0).with_curbs(false, true),
            // Repsol (Turn 4 90-degree right)
            TrackWaypoint::new(Vec2::new(170.0, 220.0), 13.0).with_curbs(false, true),
            // Seat (Turn 5 slow downhill left hairpin)
            TrackWaypoint::new(Vec2::new(120.0, 210.0), 12.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(90.0, 180.0), 12.0).with_curbs(true, false),
            // Moreneta (Turns 7-8 uphill chicane left-right)
            TrackWaypoint::new(Vec2::new(80.0, 120.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(60.0, 70.0), 13.0).with_curbs(false, true),
            // Campsa (Turn 9 fast blind uphill right)
            TrackWaypoint::new(Vec2::new(20.0, 90.0), 13.5).with_curbs(false, true),
            // Back Straight
            TrackWaypoint::new(Vec2::new(-40.0, 130.0), 14.5),
            TrackWaypoint::new(Vec2::new(-110.0, 180.0), 14.5),
            // Caixa (Turn 10 hairpin left)
            TrackWaypoint::new(Vec2::new(-160.0, 200.0), 12.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-180.0, 160.0), 12.5).with_curbs(true, false),
            // Banc de Sabadell (Turn 12 right sweeper)
            TrackWaypoint::new(Vec2::new(-160.0, 100.0), 13.5).with_curbs(false, true),
            // Fast sweeping final corners (Turns 13-14 right-handers)
            TrackWaypoint::new(Vec2::new(-120.0, 50.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-60.0, 15.0), 14.5).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de Barcelona-Catalunya".to_string(),
            description: "Premier Spanish Grand Prix aerodynamic benchmark featuring Curva Renault and Campsa.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Circuit Zandvoort: Dutch Seaside Rollercoaster with Banked Corners (Tarzan, Hugenholtz, Scheivlak, Arie Luyendyk)
    pub fn track_zandvoort() -> Track {
        let waypoints = vec![
            // Pit Straight (Start / Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(120.0, 0.0), 14.0),
            // Tarzanbocht (Turn 1 iconic 180-deg hairpin right)
            TrackWaypoint::new(Vec2::new(175.0, 20.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(190.0, 60.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(165.0, 95.0), 13.0).with_curbs(false, true),
            // Gerlachbocht (Turn 2 fast left)
            TrackWaypoint::new(Vec2::new(120.0, 110.0), 13.5).with_curbs(true, false),
            // Hugenholtzbocht (Turn 3 steep bowl banked hairpin left)
            TrackWaypoint::new(Vec2::new(70.0, 115.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(30.0, 135.0), 13.0).with_curbs(true, false),
            // Hunserug (Turn 4 crest onto straight)
            TrackWaypoint::new(Vec2::new(10.0, 180.0), 14.0),
            TrackWaypoint::new(Vec2::new(0.0, 240.0), 14.0),
            // Slotemakerbocht & Scheivlak (Turns 6-7 high-speed blind crest right)
            TrackWaypoint::new(Vec2::new(-10.0, 300.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-40.0, 350.0), 14.0).with_curbs(false, true),
            // Mastersbocht (Turn 8 downhill fast right)
            TrackWaypoint::new(Vec2::new(-90.0, 370.0), 13.5).with_curbs(false, true),
            // Bocht 9 & 10 (technical slow sweepers)
            TrackWaypoint::new(Vec2::new(-140.0, 340.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-160.0, 280.0), 13.0).with_curbs(false, true),
            // Hans Ernst Bocht (Turns 11-12 chicane right-left)
            TrackWaypoint::new(Vec2::new(-150.0, 210.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.0, 160.0), 12.0).with_curbs(true, false),
            // Kumhobocht (Turn 13)
            TrackWaypoint::new(Vec2::new(-100.0, 100.0), 13.5).with_curbs(false, true),
            // Arie Luyendykbocht (Turn 14 steep banked high-speed right onto main straight)
            TrackWaypoint::new(Vec2::new(-70.0, 40.0), 14.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-30.0, 10.0), 15.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.5, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit Zandvoort".to_string(),
            description: "Undulating Dutch coastal circuit with high-banked Hugenholtz and Arie Luyendyk corners.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Bahrain International Circuit (Sakhir): Desert Grand Prix (Turn 1 Schumacher Hairpin, Turn 9-10 Complex)
    pub fn track_bahrain() -> Track {
        let waypoints = vec![
            // Main Straight & Start/Finish
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 15.0),
            TrackWaypoint::new(Vec2::new(150.0, 0.0), 15.0),
            // Michael Schumacher Turn 1 Hairpin (heavy braking right)
            TrackWaypoint::new(Vec2::new(210.0, 15.0), 13.0).with_curbs(false, true),
            // Turns 2-3 acceleration flick left-right
            TrackWaypoint::new(Vec2::new(240.0, 45.0), 13.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(255.0, 90.0), 14.0).with_curbs(false, true),
            // Turn 4 (wide sweeping uphill right)
            TrackWaypoint::new(Vec2::new(240.0, 150.0), 13.5).with_curbs(false, true),
            // Downhill Esses (Turns 5-6-7 left-right-left)
            TrackWaypoint::new(Vec2::new(190.0, 190.0), 13.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(140.0, 220.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(90.0, 230.0), 13.0).with_curbs(true, false),
            // Turn 8 (tight right hairpin)
            TrackWaypoint::new(Vec2::new(50.0, 210.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(40.0, 170.0), 12.0).with_curbs(false, true),
            // Turns 9-10 (downhill off-camber locking braking left)
            TrackWaypoint::new(Vec2::new(20.0, 120.0), 12.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-15.0, 80.0), 12.5).with_curbs(true, false),
            // Back Straight
            TrackWaypoint::new(Vec2::new(-40.0, 30.0), 14.5),
            TrackWaypoint::new(Vec2::new(-80.0, -30.0), 14.5),
            // Turn 11 (uphill sweeping left)
            TrackWaypoint::new(Vec2::new(-120.0, -75.0), 13.5).with_curbs(true, false),
            // Turn 12 (flat-out right kink)
            TrackWaypoint::new(Vec2::new(-110.0, -130.0), 14.0).with_curbs(false, true),
            // Turn 13 (medium right)
            TrackWaypoint::new(Vec2::new(-70.0, -160.0), 13.5).with_curbs(false, true),
            // Turns 14-15 (final 90-degree right onto main straight)
            TrackWaypoint::new(Vec2::new(-20.0, -140.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(0.0, -70.0), 14.5).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Bahrain International Circuit (Sakhir)".to_string(),
            description: "Modern desert Grand Prix circuit with heavy Turn 1 braking and technical downhill off-camber Turn 9-10 complex.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Marina Bay Street Circuit (Singapore): Spectacular Floodlit Night Race (Sheares, Padang, Anderson Bridge)
    pub fn track_marina_bay() -> Track {
        let waypoints = vec![
            // Pit Straight on Republic Boulevard (Start / Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0),
            TrackWaypoint::new(Vec2::new(110.0, 0.0), 13.0),
            // Sheares Complex (Turns 1-2-3 left-right-left)
            TrackWaypoint::new(Vec2::new(160.0, 20.0), 12.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(190.0, 55.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(175.0, 95.0), 12.0).with_curbs(true, false),
            // Republic Boulevard Straight & Turn 5 (fast right)
            TrackWaypoint::new(Vec2::new(150.0, 150.0), 13.0),
            TrackWaypoint::new(Vec2::new(140.0, 205.0), 12.5).with_curbs(false, true),
            // Raffles Boulevard to Turn 7 (heavy braking left into Stamford Road)
            TrackWaypoint::new(Vec2::new(110.0, 255.0), 13.0),
            TrackWaypoint::new(Vec2::new(60.0, 280.0), 12.0).with_curbs(true, false),
            // Turn 8 (right into St Andrew's Road) & Turn 9 (left onto Padang)
            TrackWaypoint::new(Vec2::new(10.0, 260.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-25.0, 220.0), 12.0).with_curbs(true, false),
            // Anderson Bridge straight
            TrackWaypoint::new(Vec2::new(-50.0, 160.0), 12.5),
            TrackWaypoint::new(Vec2::new(-70.0, 100.0), 12.0),
            // Turn 13 hairpin (tight left around Fullerton Hotel)
            TrackWaypoint::new(Vec2::new(-95.0, 50.0), 11.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-80.0, 10.0), 11.5).with_curbs(true, false),
            // Esplanade waterfront drive
            TrackWaypoint::new(Vec2::new(-50.0, -30.0), 12.5),
            // Turns 16-17 chicane
            TrackWaypoint::new(Vec2::new(-20.0, -70.0), 12.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(10.0, -90.0), 12.0).with_curbs(true, false),
            // Turns 18-19 high-speed sweepers back onto main straight
            TrackWaypoint::new(Vec2::new(35.0, -60.0), 12.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(20.0, -25.0), 13.0).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        let obstacles = vec![
            Obstacle::circle(1, Vec2::new(162.0, 18.0), 1.0, "Sheares Apex Curb"),
            Obstacle::circle(2, Vec2::new(-18.0, -68.0), 1.0, "Bayfront Chicane Curb"),
        ];

        Track {
            name: "Marina Bay Street Circuit (Singapore)".to_string(),
            description: "High-intensity Singapore night race through the dazzling city streets and harbor waterfront.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Circuit of the Americas (COTA - Austin): Texas Thriller (Turn 1 Blind Crest, Esses, Stadium, Multi-Apex Carousel)
    pub fn track_cota() -> Track {
        let waypoints = vec![
            // Main Straight (Start / Finish)
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.5),
            TrackWaypoint::new(Vec2::new(130.0, 0.0), 14.5),
            // Turn 1 (Steep uphill blind apex hairpin left)
            TrackWaypoint::new(Vec2::new(180.0, 30.0), 13.5)
                .with_curbs(true, false)
                .with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(185.0, 75.0), 13.0)
                .with_curbs(true, false)
                .with_elevation(2.5),
            // Turns 2-6 (Fast flowing esses downhill)
            TrackWaypoint::new(Vec2::new(160.0, 130.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(115.0, 175.0), 13.5).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(135.0, 225.0), 13.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(100.0, 270.0), 13.5).with_curbs(true, false),
            // Turns 7-9 (Blind technical sweepers)
            TrackWaypoint::new(Vec2::new(50.0, 295.0), 13.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(0.0, 280.0), 13.0).with_curbs(true, false),
            // Turn 11 (Tight hairpin left onto back straight)
            TrackWaypoint::new(Vec2::new(-40.0, 230.0), 12.0).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-70.0, 190.0), 12.0).with_curbs(true, false),
            // Long Back Straight (1.2km drag strip)
            TrackWaypoint::new(Vec2::new(-95.0, 120.0), 15.0),
            TrackWaypoint::new(Vec2::new(-120.0, 30.0), 15.0),
            TrackWaypoint::new(Vec2::new(-140.0, -60.0), 15.0),
            // Turn 12 (Heavy braking left 90-degree)
            TrackWaypoint::new(Vec2::new(-135.0, -120.0), 12.5).with_curbs(true, false),
            // Stadium Section (Turns 13-15 right-left complex)
            TrackWaypoint::new(Vec2::new(-90.0, -140.0), 12.5).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-45.0, -130.0), 12.5).with_curbs(true, false),
            // Multi-Apex Carousel (Turns 16-17-18 colossal right-hand sweeper)
            TrackWaypoint::new(Vec2::new(-10.0, -110.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(15.0, -80.0), 14.0).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(20.0, -40.0), 14.0).with_curbs(false, true),
            // Turns 19-20 (Final technical lefts onto pit straight)
            TrackWaypoint::new(Vec2::new(5.0, -20.0), 13.5).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

        let checkpoints = generate_checkpoints(&spline, 18, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit of the Americas (COTA)".to_string(),
            description: "Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.".to_string(),
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
            default_laps: 5,
            predefined_car: Some("f1_car".to_string()),
            module_id: Some("f1".to_string()),
            modules: vec!["f1".to_string()],
        }
    }

    /// Modern Formula 1 Turbo Hybrid Vehicle Spec
    pub fn car_f1_hybrid() -> CarConfig {
        CarConfig {
            mass: 798.0, // FIA 2026 minimum regulation weight
            inertia: 980.0,
            wheelbase: 3.60,
            track_width: 1.80,
            cg_to_front: 1.65,
            cg_to_rear: 1.95,
            cg_height: 0.20,

            max_engine_force: 13500.0, // ~1000+ BHP Hybrid Power Unit
            max_reverse_force: 4000.0,
            max_brake_force: 28000.0, // Carbon-carbon brake discs (up to 5.5G deceleration)
            handbrake_force: 8000.0,
            brake_bias: 0.58,
            drive_bias: 0.0, // RWD
            top_speed_mps: 96.0, // ~346 km/h

            max_steer_angle: 0.42, // ~24 deg precise open-wheel rack
            steer_speed: 10.0,
            steer_return_speed: 14.0,
            counter_steer_assist: 1.1,
            speed_sensitive_steer_factor: 0.0008,

            air_drag_coefficient: 0.72,
            lateral_drag_coefficient: 1.60,
            rolling_resistance_coefficient: 0.012,
            angular_damping: 220.0,

            weight_transfer_longitudinal: 0.6,
            weight_transfer_lateral: 0.5,

            engine_braking_coefficient: 0.20,
            downforce_coefficient: 3.40, // Massive aerodynamic downforce scaling with V^2

            tire: TireConfig {
                stiffness_b: 14.5,
                shape_c: 1.55,
                peak_d: 1.28,
                curvature_e: -0.10,
                drift_slide_friction: 0.75,
                handbrake_lateral_friction_multiplier: 0.45,
                skid_threshold: 0.06,
                skid_full_threshold: 0.22,
            },
            assists: DriverAssistsConfig::arcade(),
        }
    }

    /// Classic 3.0L Screaming V10 Formula 1 Vehicle Spec
    pub fn car_f1_v10() -> CarConfig {
        let mut cfg = Self::car_f1_hybrid();
        cfg.mass = 605.0; // Ultra lightweight screaming V10 era
        cfg.inertia = 750.0;
        cfg.max_engine_force = 12200.0;
        cfg.downforce_coefficient = 3.10;
        cfg.top_speed_mps = 98.5; // ~355 km/h
        cfg
    }
}

impl Default for F1GameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModule for F1GameModule {
    fn id(&self) -> &'static str {
        "f1"
    }

    fn title(&self) -> &'static str {
        "TDRACE FORMULA 1 GRAND PRIX"
    }

    fn subtitle(&self) -> &'static str {
        "FIA World Championship Simulation & High-Downforce Open-Wheel Motorsport"
    }

    fn theme(&self) -> ModuleTheme {
        ModuleTheme {
            primary_accent: Color::new(0.95, 0.15, 0.15, 1.0), // F1 Championship Red
            secondary_accent: Color::new(0.20, 0.85, 1.0, 1.0), // Neon Electric Cyan
            header_badge: "FIA FORMULA 1 WORLD CHAMPIONSHIP",
            background_tint: Color::new(0.04, 0.05, 0.08, 0.98),
        }
    }

    fn vehicles(&self) -> Vec<VehicleModelDefinition> {
        vec![
            VehicleModelDefinition {
                id: "f1_hybrid_26",
                name: "Apex-26 Turbo Hybrid",
                tag: "1050 BHP HYBRID",
                description: "State-of-the-art 1.6L V6 Turbo Hybrid delivering 1050 BHP with active ground effect aerodynamics.",
                config: Self::car_f1_hybrid(),
                visual_type: VehicleVisualType::OpenWheel {
                    front_wing_span: 1.80,
                    rear_wing_height: 0.85,
                    halo: true,
                },
                stats: (0.98, 0.99, 0.99, 0.30),
                default_schemes: vec![
                    CarColorScheme::from_index(0), // Red Bull Dark Navy / Red
                    CarColorScheme::from_index(1), // Electric Cyan
                    CarColorScheme::from_index(2), // Ferrari Rosso Corsa
                    CarColorScheme::from_index(3), // McLaren Papaya
                ],
            },
            VehicleModelDefinition {
                id: "f1_v10_classic",
                name: "Scuderia 3.0L V10 Classic",
                tag: "19,000 RPM V10",
                description: "Legendary screaming 950 BHP V10 engine, 605 kg featherweight monocoque with raw analog response.",
                config: Self::car_f1_v10(),
                visual_type: VehicleVisualType::OpenWheel {
                    front_wing_span: 1.70,
                    rear_wing_height: 0.95,
                    halo: false,
                },
                stats: (1.00, 0.97, 0.96, 0.35),
                default_schemes: vec![
                    CarColorScheme::from_index(2), // Ferrari Racing Red
                    CarColorScheme::from_index(4), // Williams Deep Blue
                    CarColorScheme::from_index(5), // Jordan Vivid Yellow
                ],
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "f1_hybrid_26"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            TrackDefinition {
                id: "monza",
                title: "Monza Autodromo Nazionale",
                tag: "TEMPLE OF SPEED",
                description: "Iconic high-speed Italian Grand Prix circuit with Variante Rettifilo and Parabolica.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_monza,
            },
            TrackDefinition {
                id: "spa",
                title: "Circuit de Spa-Francorchamps",
                tag: "ARDENNES ROLLERCOASTER",
                description: "Legendary 7km Belgian circuit featuring Eau Rouge, Kemmel Straight, and Pouhon.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_spa,
            },
            TrackDefinition {
                id: "silverstone",
                title: "Silverstone Grand Prix Circuit",
                tag: "HOME OF BRITISH MOTORSPORT",
                description: "Ultra-fast flowing esses through Maggotts, Becketts, Chapel, and Stowe.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_silverstone,
            },
            TrackDefinition {
                id: "monaco",
                title: "Circuit de Monaco",
                tag: "JEWEL IN THE CROWN",
                description: "Prestigious Monte Carlo street circuit with Loews Hairpin, the Tunnel, and Swimming Pool.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_monaco,
            },
            TrackDefinition {
                id: "suzuka",
                title: "Suzuka International Racing Course",
                tag: "JAPANESE FIGURE-8",
                description: "Technical figure-8 circuit with Esses, Degner, crossover bridge, and 130R.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_suzuka,
            },
            TrackDefinition {
                id: "interlagos",
                title: "Autodromo Jose Carlos Pace",
                tag: "BRAZILIAN ROLLERCOASTER",
                description: "Anti-clockwise Brazilian thriller featuring Senna 'S', Curva do Sol, and Juncao.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_interlagos,
            },
            TrackDefinition {
                id: "montreal",
                title: "Circuit Gilles Villeneuve",
                tag: "ILE NOTRE-DAME",
                description: "Canadian island circuit with Virage Senna, L'Epingle hairpin, and Wall of Champions.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_montreal,
            },
            TrackDefinition {
                id: "red_bull_ring",
                title: "Red Bull Ring",
                tag: "AUSTRIAN ALPS",
                description: "Undulating Austrian alpine sprint circuit with steep climbs and heavy braking into Remus.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_red_bull_ring,
            },
            TrackDefinition {
                id: "catalunya",
                title: "Circuit de Barcelona-Catalunya",
                tag: "SPANISH GP BENCHMARK",
                description: "Premier aerodynamic benchmark testing high-speed downforce and technical precision.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_catalunya,
            },
            TrackDefinition {
                id: "zandvoort",
                title: "Circuit Zandvoort",
                tag: "DUTCH DUNES",
                description: "Seaside rollercoaster featuring high-banked Hugenholtz and Arie Luyendyk curves.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_zandvoort,
            },
            TrackDefinition {
                id: "bahrain",
                title: "Bahrain International Circuit",
                tag: "DESERT GRAND PRIX",
                description: "Sakhir desert circuit with heavy Turn 1 braking and technical off-camber Turns 9-10.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_bahrain,
            },
            TrackDefinition {
                id: "marina_bay",
                title: "Marina Bay Street Circuit",
                tag: "SINGAPORE NIGHT RACE",
                description: "Spectacular floodlit street race navigating tight harbor chicanes and city avenues.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_marina_bay,
            },
            TrackDefinition {
                id: "cota",
                title: "Circuit of the Americas",
                tag: "AUSTIN SPECTACLE",
                description: "Grand Prix venue featuring steep uphill Turn 1 blind crest and high-speed Esses.",
                category: "Official GP Circuit",
                default_laps: 5,
                generator: Self::track_cota,
            },
            TrackDefinition {
                id: "classic_grand_prix",
                title: "Classic Grand Prix Circuit",
                tag: "FIA TEST TRACK",
                description: "Technical GP testing circuit with high-speed chicanes and strategic pit lane.",
                category: "FIA Test Circuit",
                default_laps: 5,
                generator: classic_grand_prix,
            },
        ]
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
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(0),
                profile: BotProfile {
                    name: "Max Hunter",
                    lookahead_time: 0.42,
                    speed_factor: 1.05,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 1.01,
                    aggression: 0.88,
                    avoidance_distance: 6.0,
                },
                stats: DriverStats {
                    speed: 0.99,
                    aggression: 0.92,
                    precision: 0.98,
                    defense: 0.95,
                },
            },
            DriverCharacter {
                id: "charles_laurent",
                name: "Charles Laurent",
                alias: "The Qualifying King",
                bio: "Scuderia prodigy with unbelievable single-lap hot-lap qualifying pace and unmatched precision on street circuits.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(2),
                profile: BotProfile {
                    name: "Charles Laurent",
                    lookahead_time: 0.38,
                    speed_factor: 1.04,
                    steering_kp: 2.7,
                    steering_kd: 0.07,
                    brake_margin: 1.00,
                    aggression: 0.78,
                    avoidance_distance: 6.2,
                },
                stats: DriverStats {
                    speed: 0.98,
                    aggression: 0.80,
                    precision: 1.00,
                    defense: 0.86,
                },
            },
            DriverCharacter {
                id: "lewis_vance",
                name: "Lewis Vance",
                alias: "The Master",
                bio: "7-time World Champion whose supreme tire management and legendary racecraft allow him to hunt down leaders from any grid slot.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(1),
                profile: BotProfile {
                    name: "Lewis Vance",
                    lookahead_time: 0.44,
                    speed_factor: 1.03,
                    steering_kp: 2.5,
                    steering_kd: 0.09,
                    brake_margin: 1.03,
                    aggression: 0.75,
                    avoidance_distance: 6.8,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.76,
                    precision: 0.99,
                    defense: 0.98,
                },
            },
            DriverCharacter {
                id: "fernando_toro",
                name: "Fernando Toro",
                alias: "El Matador",
                bio: "Veteran motorsport warrior who exploits every millimeter of asphalt and turns defensive driving into high art.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(3),
                profile: BotProfile {
                    name: "Fernando Toro",
                    lookahead_time: 0.40,
                    speed_factor: 1.02,
                    steering_kp: 2.8,
                    steering_kd: 0.08,
                    brake_margin: 1.02,
                    aggression: 0.95,
                    avoidance_distance: 5.5,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.98,
                    precision: 0.95,
                    defense: 1.00,
                },
            },
            DriverCharacter {
                id: "george_speed",
                name: "George Speed",
                alias: "The Silver Bullet",
                bio: "Methodical British racer with blistering speed and unyielding qualifying pace for the Silver Arrows.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(7),
                profile: BotProfile {
                    name: "George Speed",
                    lookahead_time: 0.41,
                    speed_factor: 1.02,
                    steering_kp: 2.6,
                    steering_kd: 0.08,
                    brake_margin: 1.01,
                    aggression: 0.82,
                    avoidance_distance: 6.0,
                },
                stats: DriverStats {
                    speed: 0.96,
                    aggression: 0.84,
                    precision: 0.96,
                    defense: 0.90,
                },
            },
            DriverCharacter {
                id: "lando_vance",
                name: "Lando Vance",
                alias: "Papaya Prodigy",
                bio: "High-octane fan favorite who excels in dynamic mixed conditions with aggressive late-braking passes.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(4),
                profile: BotProfile {
                    name: "Lando Vance",
                    lookahead_time: 0.39,
                    speed_factor: 1.03,
                    steering_kp: 2.7,
                    steering_kd: 0.07,
                    brake_margin: 0.99,
                    aggression: 0.86,
                    avoidance_distance: 5.8,
                },
                stats: DriverStats {
                    speed: 0.97,
                    aggression: 0.87,
                    precision: 0.95,
                    defense: 0.88,
                },
            },
            DriverCharacter {
                id: "oscar_rocket",
                name: "Oscar Rocket",
                alias: "Melbourne Missile",
                bio: "Ultra-composed Australian rookie sensation known for ice-cold nerve and textbook race craft on high-speed circuits.",
                preferred_car: crate::ui::menu::CarChoice::F1Car,
                color_scheme: CarColorScheme::from_index(5),
                profile: BotProfile {
                    name: "Oscar Rocket",
                    lookahead_time: 0.40,
                    speed_factor: 1.02,
                    steering_kp: 2.5,
                    steering_kd: 0.08,
                    brake_margin: 1.02,
                    aggression: 0.80,
                    avoidance_distance: 6.2,
                },
                stats: DriverStats {
                    speed: 0.95,
                    aggression: 0.82,
                    precision: 0.97,
                    defense: 0.92,
                },
            },
        ]
    }

    fn supported_game_modes(&self) -> Vec<TournamentFormat> {
        vec![
            TournamentFormat::Championship {
                name: "FIA Formula 1 World Championship 2026".to_string(),
                point_system: PointSystem::F1Standard { fastest_lap_bonus: true },
                track_ids: vec![
                    "bahrain".to_string(),
                    "suzuka".to_string(),
                    "monaco".to_string(),
                    "montreal".to_string(),
                    "catalunya".to_string(),
                    "red_bull_ring".to_string(),
                    "silverstone".to_string(),
                    "spa".to_string(),
                    "zandvoort".to_string(),
                    "monza".to_string(),
                    "marina_bay".to_string(),
                    "cota".to_string(),
                    "interlagos".to_string(),
                    "classic_grand_prix".to_string(),
                ],
                laps_per_round: 5,
            },
            TournamentFormat::QualifyingShootout {
                time_limit: 180.0,
            },
            TournamentFormat::QuickRace {
                default_laps: 5,
                default_bots: 7,
            },
            TournamentFormat::TimeAttack,
        ]
    }

    fn audio_profile(&self) -> EngineAudioProfile {
        EngineAudioProfile::f1_v6_turbo_hybrid()
    }
}
