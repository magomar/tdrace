use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::config::{CarConfig, DriverAssistsConfig, TireConfig};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, TrackGeometry};
use tdrace_core::track::presets::{generate_checkpoints, generate_grid_positions, generate_walls_from_spline};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{Track, TrackCategory, TrackKind};

use super::{EngineAudioProfile, GameModule, ModuleTheme, TrackDefinition, VehicleModelDefinition, VehicleVisualType};
use crate::ai::{BotProfile, DriverCharacter, DriverStats};
use crate::render::color::CarColorScheme;
use crate::tournament::{PointSystem, TournamentFormat};

/// GT World Challenge Game Module
pub struct GtWorldChallengeModule;

impl GtWorldChallengeModule {
    pub fn new() -> Self {
        Self
    }

    /// Monza Autodromo Nazionale: High-speed Italian Grand Prix temple of speed.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2896.5m (Real FIA: 5793m).
    pub fn track_monza() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(103.4, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(206.9, 1.8), 15.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(310.3, 3.7), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(413.7, 5.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(508.4, -15.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(467.5, -100.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(367.3, -123.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(264.2, -131.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(160.8, -133.3), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(57.4, -135.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-46.0, -138.7), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-149.4, -142.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-232.7, -137.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-334.7, -150.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-431.8, -121.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-491.7, -39.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-515.7, 61.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-534.7, 163.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-582.1, 246.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-624.5, 340.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-548.1, 391.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-454.1, 388.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-393.1, 304.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-327.5, 225.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-250.4, 156.2), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-173.4, 87.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-94.2, 22.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Monza Autodromo Nazionale".to_string(),
            description: "High-speed Italian Grand Prix temple of speed.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt4_clubsport".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit de Spa-Francorchamps: Belgian Ardennes rollercoaster featuring Eau Rouge and Pouhon.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 3502.0m (Real FIA: 7004m).
    pub fn track_spa() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(109.2, 0.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(213.7, -31.4), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(316.4, -68.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(404.2, -123.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.5),
            TrackWaypoint::new(Vec2::new(499.7, -176.8), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(584.3, -244.7), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(660.8, -322.9), 14.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(737.3, -401.2), 14.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(814.0, -479.2), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(842.0, -572.8), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(822.8, -670.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(762.7, -755.5), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(772.2, -666.6), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(714.7, -583.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(645.0, -499.1), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(556.8, -458.4), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(497.5, -539.9), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(509.6, -648.7), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(478.5, -737.4), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(420.6, -806.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(379.0, -887.5), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(289.6, -841.5), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(292.8, -733.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(337.1, -634.1), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(392.4, -540.2), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(394.0, -431.6), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(338.4, -343.4), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(260.5, -267.1), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(216.2, -177.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(157.1, -116.3), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(64.2, -58.5), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de Spa-Francorchamps".to_string(),
            description: "Belgian Ardennes rollercoaster featuring Eau Rouge and Pouhon.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 3,
            predefined_car: Some("gt2_biturbo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Silverstone Grand Prix Circuit: High-speed sweeping esses through Maggotts, Becketts and Chapel.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2945.5m (Real FIA: 5891m).
    pub fn track_silverstone() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(97.7, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(106.8, -73.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(167.4, -3.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(150.0, 92.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(132.2, 189.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(113.7, 285.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(50.5, 315.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-1.8, 337.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(85.4, 380.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(179.4, 365.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(262.9, 313.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(345.2, 260.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(341.9, 169.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(294.9, 83.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(243.0, 0.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(189.4, -73.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(153.9, -159.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(58.9, -169.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-30.5, -210.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-119.5, -251.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-209.1, -291.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-303.0, -318.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-335.0, -241.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-335.8, -143.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-377.8, -79.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-358.7, 8.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-264.1, 34.4), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-169.4, 60.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-80.4, 49.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Silverstone Grand Prix Circuit".to_string(),
            description: "High-speed sweeping esses through Maggotts, Becketts and Chapel.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit de Monaco: Legendary Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 1668.5m (Real FIA: 3337m).
    pub fn track_monaco() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(64.1, -0.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(125.7, -16.5), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(142.7, -70.4), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.5),
            TrackWaypoint::new(Vec2::new(152.7, -133.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(164.5, -196.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(172.7, -259.3), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(222.7, -284.6), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(1.0),
            TrackWaypoint::new(Vec2::new(276.3, -278.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(323.4, -321.5), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(309.1, -354.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(315.2, -359.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(320.5, -402.2), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(257.8, -389.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(203.4, -355.8), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(165.3, -304.6), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(144.0, -244.3), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(132.7, -181.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(130.3, -123.1), 10.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(128.3, -59.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(80.4, -24.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(21.5, -38.3), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-36.0, -29.7), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-94.0, -54.2), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.4, -35.5), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-63.4, -10.0), 10.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de Monaco".to_string(),
            description: "Legendary Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 6,
            predefined_car: Some("hypercar_prototype".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Suzuka International Racing Course: Iconic Japanese figure-8 layout featuring Esses, Degner, overpass crossover bridge, and 130R.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2903.5m (Real FIA: 5807m).
    pub fn track_suzuka() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(68.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(18.6, -80.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-65.1, -80.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-137.8, -100.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-211.5, -115.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-283.0, -113.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-346.3, -70.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-412.5, -121.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-423.6, -204.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-417.3, -286.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-474.8, -331.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-550.0, -290.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-612.8, -237.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-636.4, -246.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-619.9, -329.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-667.7, -398.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_wall_distances(Some(2.2), Some(2.2)),
            TrackWaypoint::new(Vec2::new(-743.2, -436.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_wall_distances(Some(2.2), Some(2.2)),
            TrackWaypoint::new(Vec2::new(-827.0, -427.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-905.5, -429.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-899.8, -498.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-816.0, -488.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-735.9, -458.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_wall_distances(Some(2.2), Some(2.2)),
            TrackWaypoint::new(Vec2::new(-661.1, -417.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.5).with_wall_distances(Some(2.2), Some(2.2)),
            TrackWaypoint::new(Vec2::new(-588.0, -373.5), 13.0).with_surface(SurfaceType::Asphalt).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(-515.0, -329.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(-461.8, -267.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(-457.5, -182.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(-455.6, -98.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-417.2, -34.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-341.6, -1.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-256.2, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-170.8, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-85.4, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.8, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Suzuka International Racing Course".to_string(),
            description: "Iconic Japanese figure-8 layout featuring Esses, Degner, overpass crossover bridge, and 130R.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt1_legend".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Autodromo Jose Carlos Pace (Interlagos): Thrilling anti-clockwise Brazilian Grand Prix circuit with Senna 'S', Ferradura, and Juncao.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2154.5m (Real FIA: 4309m).
    pub fn track_interlagos() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(76.9, -0.0), 14.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(153.9, -0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(230.5, 3.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(235.7, 65.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(267.5, 131.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(226.4, 193.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(160.3, 232.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(92.8, 269.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(25.8, 307.3), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-41.1, 345.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-87.1, 293.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-69.7, 223.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-20.5, 163.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(27.9, 104.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(6.3, 37.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-69.5, 40.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-57.6, 101.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-122.8, 92.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-189.2, 101.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-127.7, 144.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-133.6, 214.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-186.8, 270.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-238.8, 228.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-241.6, 152.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-210.9, 83.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-144.3, 45.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-75.7, 10.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Autodromo Jose Carlos Pace (Interlagos)".to_string(),
            description: "Thrilling anti-clockwise Brazilian Grand Prix circuit with Senna 'S', Ferradura, and Juncao.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 5,
            predefined_car: Some("gt1_legend".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit Gilles Villeneuve (Montreal): High-speed Canadian island circuit featuring Virage Senna, L'Epingle hairpin, and Wall of Champions.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2180.5m (Real FIA: 4361m).
    pub fn track_montreal() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(70.8, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(146.3, 19.0), 14.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(221.9, 38.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(298.3, 51.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(375.3, 49.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(413.8, 50.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(386.6, -22.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(336.9, -81.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(270.6, -108.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(215.8, -162.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(138.7, -169.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(103.1, -217.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(31.3, -242.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-46.3, -248.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-124.0, -244.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-200.1, -228.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-254.0, -184.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-330.1, -167.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-407.5, -159.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-484.3, -170.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-518.6, -159.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-445.3, -136.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-374.3, -104.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-302.2, -75.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-226.5, -57.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-151.0, -38.0), 14.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-75.5, -18.9), 14.5).with_surface(SurfaceType::Asphalt),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::Concrete);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit Gilles Villeneuve (Montreal)".to_string(),
            description: "High-speed Canadian island circuit featuring Virage Senna, L'Epingle hairpin, and Wall of Champions.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 5,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Red Bull Ring (Spielberg): High-speed Austrian alpine circuit with steep uphill climbs and heavy downhill braking into Remus.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2159.0m (Real FIA: 4318m).
    pub fn track_red_bull_ring() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(83.0, -0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(166.0, -1.7), 14.5).with_surface(SurfaceType::Asphalt).with_elevation(3.5),
            TrackWaypoint::new(Vec2::new(249.1, -3.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(323.2, -18.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(349.1, -97.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(373.2, -176.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(1.0),
            TrackWaypoint::new(Vec2::new(387.1, -258.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(406.2, -338.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(444.7, -412.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(422.2, -448.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(342.9, -423.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(268.7, -386.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(194.1, -350.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(115.8, -322.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(116.5, -272.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(197.0, -265.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(272.2, -300.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(333.3, -268.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(315.7, -187.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(249.6, -169.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(171.3, -187.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(91.3, -164.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(11.0, -143.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-63.0, -111.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-67.3, -29.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Red Bull Ring (Spielberg)".to_string(),
            description: "High-speed Austrian alpine circuit with steep uphill climbs and heavy downhill braking into Remus.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 5,
            predefined_car: Some("gt4_clubsport".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit de Barcelona-Catalunya: Famous Spanish GP circuit in Montmelo featuring Curva Renault, Campsa crest, and restored high-speed final sector.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2328.5m (Real FIA: 4657m).
    pub fn track_catalunya() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(69.7, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(137.1, -39.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(123.4, -120.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(102.8, -201.4), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(82.4, -282.1), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(61.9, -362.7), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(41.8, -443.3), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(21.2, -523.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-20.0, -581.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-82.3, -622.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-156.4, -635.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-189.4, -562.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-172.9, -481.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-143.9, -406.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-93.2, -454.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(-112.3, -535.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(3.5),
            TrackWaypoint::new(Vec2::new(-71.4, -553.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(-15.0, -493.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(1.6, -413.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-64.2, -367.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-113.1, -301.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-72.3, -233.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-13.5, -174.5), 13.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(45.2, -115.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(51.5, -59.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-12.5, -104.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-50.1, -61.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de Barcelona-Catalunya".to_string(),
            description: "Famous Spanish GP circuit in Montmelo featuring Curva Renault, Campsa crest, and restored high-speed final sector.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 5,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit Zandvoort: Dune rollercoaster in the Netherlands featuring 18-degree banked corners at Hugenholtz and Arie Luyendyk.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2129.5m (Real FIA: 4259m).
    pub fn track_zandvoort() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(43.6, -0.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(73.0, -69.7), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(84.7, -134.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(115.2, -153.6), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(159.3, -92.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.5),
            TrackWaypoint::new(Vec2::new(223.4, -51.6), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(257.6, 14.7), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(305.8, 71.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(373.1, 99.1), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(412.7, 40.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(427.7, -33.4), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(388.2, -87.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(328.4, -73.4), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(367.4, -9.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(327.8, 21.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(270.0, -27.8), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(223.7, -88.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(189.5, -155.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(143.0, -194.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(199.7, -244.7), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(253.5, -296.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(208.5, -353.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(136.6, -352.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(100.0, -287.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(75.0, -215.5), 12.5).with_surface(SurfaceType::Asphalt).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(50.0, -143.6), 14.0).with_surface(SurfaceType::Asphalt).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(25.0, -71.8), 14.0).with_surface(SurfaceType::Asphalt),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit Zandvoort".to_string(),
            description: "Dune rollercoaster in the Netherlands featuring 18-degree banked corners at Hugenholtz and Arie Luyendyk.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 5,
            predefined_car: Some("gt2_biturbo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Bahrain International Circuit (Sakhir): High-power desert battleground under the floodlights with heavy braking zones and abrasive tarmac.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2706.0m (Real FIA: 5412m).
    pub fn track_bahrain() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(89.3, -0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(179.3, -6.4), 15.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(269.3, -12.8), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(359.3, -19.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(376.4, -80.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(323.9, -154.2), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(271.4, -227.5), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(218.8, -300.9), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(166.3, -374.2), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(113.7, -447.5), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(61.0, -520.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(8.4, -467.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-62.8, -414.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-127.6, -351.7), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-192.4, -288.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-184.2, -237.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-95.6, -221.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-18.5, -224.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(66.2, -215.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(104.3, -236.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(43.4, -302.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(11.3, -379.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(67.0, -343.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(120.0, -270.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(172.6, -197.3), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(225.2, -124.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(172.7, -77.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(93.3, -117.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(18.8, -80.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.8, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Bahrain International Circuit (Sakhir)".to_string(),
            description: "High-power desert battleground under the floodlights with heavy braking zones and abrasive tarmac.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Marina Bay Street Circuit (Singapore): High-intensity Singapore night race through the dazzling city streets and harbor waterfront.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2470.0m (Real FIA: 4940m).
    pub fn track_marina_bay() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(82.3, 0.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(151.5, 17.4), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(168.8, 77.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(88.2, 75.0), 11.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(6.7, 72.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(2.5, 148.6), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(18.1, 229.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(40.2, 308.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(90.8, 373.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(142.1, 437.4), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(94.9, 490.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(98.5, 538.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(121.8, 598.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(56.6, 648.7), 11.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-8.6, 698.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-70.6, 686.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-140.6, 655.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-181.3, 609.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-103.6, 582.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-27.7, 550.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(44.9, 512.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(1.6, 460.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-48.7, 397.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-65.2, 317.2), 11.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-81.7, 236.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-120.9, 180.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-150.5, 107.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-155.6, 29.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-82.3, 2.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Marina Bay Street Circuit (Singapore)".to_string(),
            description: "High-intensity Singapore night race through the dazzling city streets and harbor waterfront.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("hypercar_prototype".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit of the Americas (COTA): Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2756.5m (Real FIA: 5513m).
    pub fn track_cota() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(91.9, -0.0), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(183.8, -0.3), 15.0).with_surface(SurfaceType::Asphalt).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(275.6, -0.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.5),
            TrackWaypoint::new(Vec2::new(367.5, 1.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(330.9, 51.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(298.3, 127.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(328.7, 213.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(336.7, 300.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(325.2, 387.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(404.7, 424.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(431.8, 508.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(505.4, 544.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(538.9, 625.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(541.5, 717.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(518.0, 768.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(465.6, 692.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(410.3, 619.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(351.7, 548.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(290.1, 480.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(225.2, 415.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(159.5, 351.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(209.6, 312.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(214.3, 264.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(129.9, 270.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(205.3, 229.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(231.4, 153.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(158.5, 109.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(69.9, 133.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(13.8, 71.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit of the Americas (COTA)".to_string(),
            description: "Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// MadRing Circuito de Madrid: Spanish Grand Prix hybrid street circuit navigating the IFEMA complex and Valdebebas avenues.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2737.0m (Real FIA: 5474m).
    pub fn track_madring() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(90.7, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(171.8, -39.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(246.0, -71.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(334.6, -57.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(408.4, -87.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(496.7, -96.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(552.5, -159.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(642.6, -162.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(662.5, -240.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(582.5, -260.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(520.7, -195.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(466.3, -121.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(390.4, -156.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(302.6, -145.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(230.6, -188.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(217.2, -275.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(129.4, -297.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(84.6, -368.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(11.6, -402.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-78.7, -389.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-108.5, -448.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-184.7, -463.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-268.7, -442.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-257.9, -351.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-246.0, -261.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-233.2, -171.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-247.6, -95.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-176.6, -45.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-88.4, -22.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Concrete);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "MadRing Circuito de Madrid".to_string(),
            description: "Spanish Grand Prix hybrid street circuit navigating the IFEMA complex and Valdebebas avenues.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("hypercar_prototype".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Nürburgring Grand Prix-Strecke: Challenging Eifel circuit featuring Castrol-S chicane, Mercedes Arena, and Schumacher S.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2574.0m (Real FIA: 5148m).
    pub fn track_nurburgring_gp() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(85.8, -0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(170.9, -9.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(149.2, -46.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(-1.0),
            TrackWaypoint::new(Vec2::new(136.9, -108.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(-2.0),
            TrackWaypoint::new(Vec2::new(210.4, -127.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(-1.5),
            TrackWaypoint::new(Vec2::new(211.6, -59.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(281.7, -10.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(358.0, 28.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(384.0, 96.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(425.5, 133.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(508.9, 114.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(594.4, 116.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(671.8, 114.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(604.0, 88.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(518.2, 89.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(456.8, 37.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(381.9, -4.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(305.5, -43.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(270.5, -103.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(272.0, -180.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(187.4, -190.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(101.9, -198.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(22.2, -174.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-51.5, -130.4), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-125.2, -86.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-203.9, -72.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-252.7, -13.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-171.6, 0.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-85.8, 0.3), 13.5).with_surface(SurfaceType::Asphalt),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Nurburgring Grand Prix-Strecke".to_string(),
            description: "Challenging Eifel circuit featuring Castrol-S chicane, Mercedes Arena, and Schumacher S.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt4_clubsport".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Mount Panorama (Bathurst): The iconic Australian mountain rollercoaster: Hell Corner, Skyline, The Dipper, and Conrod Straight.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 3106.5m (Real FIA: 6213m).
    pub fn track_bathurst() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(97.1, -0.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(118.9, 82.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(119.7, 179.4), 12.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(120.6, 276.4), 12.5).with_surface(SurfaceType::Asphalt).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(121.4, 373.5), 12.5).with_surface(SurfaceType::Asphalt).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(122.3, 470.6), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(4.5),
            TrackWaypoint::new(Vec2::new(129.5, 566.2), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(5.5),
            TrackWaypoint::new(Vec2::new(219.3, 563.6), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(6.0),
            TrackWaypoint::new(Vec2::new(311.6, 534.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(5.5),
            TrackWaypoint::new(Vec2::new(332.6, 591.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.5),
            TrackWaypoint::new(Vec2::new(307.5, 679.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.5),
            TrackWaypoint::new(Vec2::new(365.4, 752.7), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.5),
            TrackWaypoint::new(Vec2::new(401.0, 841.1), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(353.8, 922.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(0.5),
            TrackWaypoint::new(Vec2::new(275.6, 972.2), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(179.5, 958.4), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(85.3, 959.1), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(3.0, 998.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-76.9, 1047.7), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-132.4, 1011.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-148.8, 916.6), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-148.4, 819.5), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-147.0, 722.4), 12.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-145.6, 625.4), 12.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-144.2, 528.3), 12.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-142.7, 431.2), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-163.3, 336.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-155.2, 252.9), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-138.8, 160.2), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-137.8, 63.2), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-97.1, 0.3), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 3.5, BarrierType::Concrete);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Mount Panorama (Bathurst)".to_string(),
            description: "The iconic Australian mountain rollercoaster: Hell Corner, Skyline, The Dipper, and Conrod Straight.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt3_evo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Autodromo Internacional do Algarve: Spectacular undulating Portuguese rollercoaster featuring Torre VIP and sweeping downhill Galp curve.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 2326.5m (Real FIA: 4653m).
    pub fn track_portimao_gp() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 15.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(83.1, 0.0), 15.0).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(166.2, 0.2), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(249.3, 0.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(332.3, 0.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(364.5, -74.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(323.0, -143.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(283.3, -76.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.0),
            TrackWaypoint::new(Vec2::new(200.4, -74.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(117.3, -74.5), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(34.2, -74.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(45.0, -106.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(123.8, -129.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(205.3, -113.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(262.3, -165.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(192.3, -176.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(110.0, -165.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(40.0, -200.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-24.7, -247.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-51.0, -183.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-85.7, -115.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-157.6, -104.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-114.9, -174.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-171.1, -195.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-219.1, -128.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(-228.9, -49.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(1.0),
            TrackWaypoint::new(Vec2::new(-166.2, -0.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-83.1, 0.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Autodromo Internacional do Algarve".to_string(),
            description: "Spectacular undulating Portuguese rollercoaster featuring Torre VIP and sweeping downhill Galp curve.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt2_biturbo".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
    }

    /// Circuit de la Sarthe (Le Mans): The crown jewel of endurance motorsport: Dunlop Bridge, Mulsanne Straight, Indianapolis, and Porsche Curves.
    /// Surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length: 6813.0m (Real FIA: 13626m).
    pub fn track_le_mans_sarthe() -> Track {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(189.2, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(1.5),
            TrackWaypoint::new(Vec2::new(351.1, -79.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(479.8, -196.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true).with_elevation(2.0),
            TrackWaypoint::new(Vec2::new(569.6, -348.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(688.2, -476.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(584.6, -625.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(425.6, -727.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(248.4, -793.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(71.1, -859.9), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-106.1, -926.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-280.6, -969.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-446.2, -1053.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-623.3, -1119.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-800.7, -1185.6), 13.5).with_surface(SurfaceType::Asphalt),
            TrackWaypoint::new(Vec2::new(-978.0, -1251.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-1144.7, -1329.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1316.5, -1378.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1503.7, -1403.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1691.9, -1423.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1880.3, -1442.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1944.2, -1324.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1902.9, -1139.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1840.3, -961.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1759.3, -791.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1653.6, -634.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1511.0, -520.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-1539.0, -361.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-1356.9, -309.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-1185.2, -232.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-1025.2, -132.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-861.9, -149.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-698.0, -101.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-530.6, -136.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-344.5, -113.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-164.6, -54.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        ];

        let spline = TrackSpline::new(waypoints, true);
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.5, BarrierType::Steel);

        let checkpoints = generate_checkpoints(&spline, 20, 3);
        let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

        Track {
            name: "Circuit de la Sarthe (Le Mans)".to_string(),
            description: "The crown jewel of endurance motorsport: Dunlop Bridge, Mulsanne Straight, Indianapolis, and Porsche Curves.".to_string(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline,
            geometry: TrackGeometry {
                inner_walls: left_walls,
                outer_walls: right_walls,
                obstacles: Vec::new(),
                surface_zones: Vec::new(),
                jump_ramps: Vec::new(),
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            checkpoints,
            grid_positions: starting_grid,
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            default_laps: 4,
            predefined_car: Some("gt1_legend".to_string()),
            module_id: Some("gt".to_string()),
            modules: vec!["gt".to_string()],
        }
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
            top_speed_mps: 82.5, // ~297 km/h

            max_steer_angle: 0.50, // ~28.6 deg responsive GT rack
            steer_speed: 9.0,
            steer_return_speed: 12.0,
            counter_steer_assist: 1.15,
            speed_sensitive_steer_factor: 0.016,

            air_drag_coefficient: 0.65,
            lateral_drag_coefficient: 1.40,
            rolling_resistance_coefficient: 0.014,
            angular_damping: 180.0,

            weight_transfer_longitudinal: 0.55,
            weight_transfer_lateral: 0.45,

            engine_braking_coefficient: 0.22,
            downforce_coefficient: 2.10, // Strong GT3 aerodynamic package

            tire: TireConfig {
                stiffness_b: 13.0,
                shape_c: 1.45,
                peak_d: 1.20,
                curvature_e: -0.08,
                drift_slide_friction: 0.82,
                handbrake_lateral_friction_multiplier: 0.50,
                skid_threshold: 0.08,
                skid_full_threshold: 0.24,
            },
            assists: DriverAssistsConfig::sport(),
        }
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
        cfg.mass = 1320.0;
        cfg.inertia = 1620.0;
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
        cfg.mass = 1030.0;
        cfg.inertia = 1250.0;
        cfg.wheelbase = 3.15;
        cfg.track_width = 2.00;
        cfg.max_engine_force = 12200.0; // ~800 BHP LMH Hybrid
        cfg.max_reverse_force = 7930.0;
        cfg.max_brake_force = 26000.0;
        cfg.top_speed_mps = 95.0; // ~342 km/h
        cfg.downforce_coefficient = 3.10; // Extreme ground-effect aero tunnels
        cfg.air_drag_coefficient = 0.68;
        cfg.assists = DriverAssistsConfig::sport();
        cfg
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
            max_reverse_force: 8775.0,
            max_brake_force: 28000.0, // Carbon-carbon brake discs (up to 5.5G deceleration)
            handbrake_force: 8000.0,
            brake_bias: 0.64,
            drive_bias: 0.0, // RWD
            top_speed_mps: 96.0, // ~346 km/h

            max_steer_angle: 0.42, // ~24 deg precise open-wheel rack
            steer_speed: 10.0,
            steer_return_speed: 14.0,
            counter_steer_assist: 1.1,
            speed_sensitive_steer_factor: 0.015,

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
            },
            VehicleModelDefinition {
                id: "f1_hybrid_26",
                name: "1050 BHP Hybrid F1 Turbo (Experimental)",
                tag: "EXPERIMENTAL OPEN-WHEEL",
                description: "Experimental 1050 BHP hybrid open-wheel test bench, 346 km/h, extreme downforce (Cl=3.4). Available from Level 1.",
                config: Self::car_f1_hybrid(),
                visual_type: VehicleVisualType::OpenWheel {
                    front_wing_span: 1.80,
                    rear_wing_height: 0.85,
                    halo: true,
                },
                stats: (0.99, 0.99, 0.99, 0.30),
                default_schemes: vec![
                    CarColorScheme::from_index(6), // Red Bull Stealth Carbon / Navy
                    CarColorScheme::from_index(1), // Electric Cyan
                    CarColorScheme::from_index(2), // Ferrari Rosso Corsa
                    CarColorScheme::from_index(3), // McLaren Papaya
                ],
            },
        ]
    }

    fn default_vehicle_id(&self) -> &'static str {
        "gt4_clubsport"
    }

    fn tracks(&self) -> Vec<TrackDefinition> {
        vec![
            // Level 1: Fluid Speed & Classic Flow
            TrackDefinition {
                id: "monza",
                title: "Monza Autodromo Nazionale",
                tag: "TEMPLE OF SPEED",
                description: "Iconic high-speed Italian Grand Prix circuit with Variante Rettifilo and Parabolica.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_monza,
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
                id: "nurburgring_gp",
                title: "Nurburgring Grand Prix-Strecke",
                tag: "EIFEL MOTORSPORT MECCA",
                description: "Modern German GP circuit featuring the Castrol-S, Dunlop hairpin, and Schumacher S.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_nurburgring_gp,
            },
            // Level 2: GT Temples & Medium Downforce
            TrackDefinition {
                id: "silverstone",
                title: "Silverstone Grand Prix Circuit",
                tag: "HOME OF BRITISH MOTORSPORT",
                description: "Ultra-fast flowing esses through Maggotts, Becketts, Chapel, and Stowe.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_silverstone,
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
                id: "bathurst",
                title: "Mount Panorama Circuit (Bathurst)",
                tag: "MOUNTAIN ROLLERCOASTER",
                description: "Legendary Australian mountain course through the Cutting, Skyline, the Dipper, and Conrod Straight.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_bathurst,
            },
            // Level 3: High Speed & Elevation Rollercoasters
            TrackDefinition {
                id: "spa",
                title: "Circuit de Spa-Francorchamps",
                tag: "ARDENNES ROLLERCOASTER",
                description: "Legendary 7km Belgian circuit featuring Eau Rouge, Kemmel Straight, and Pouhon.",
                category: "Official GP Circuit",
                default_laps: 3,
                generator: Self::track_spa,
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
                id: "portimao_gp",
                title: "Autodromo Internacional do Algarve",
                tag: "PORTUGUESE ROLLERCOASTER",
                description: "Spectacular undulating Portuguese rollercoaster featuring Torre VIP and sweeping downhill Galp curve.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_portimao_gp,
            },
            // Level 4: Legendary Technical Benchmarks
            TrackDefinition {
                id: "suzuka",
                title: "Suzuka International Racing Course",
                tag: "JAPANESE FIGURE-8",
                description: "Technical figure-8 circuit with Esses, Degner, crossover bridge, and 130R.",
                category: "Official GP Circuit",
                default_laps: 4,
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
                id: "le_mans_sarthe",
                title: "Circuit de la Sarthe (Le Mans)",
                tag: "24 HOURS OF LE MANS",
                description: "The crown jewel of endurance motorsport: Dunlop Bridge, Mulsanne Straight, Indianapolis, and Porsche Curves.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_le_mans_sarthe,
            },
            // Level 5: Street Circuits & Maximum Aero Prototypes
            TrackDefinition {
                id: "monaco",
                title: "Circuit de Monaco",
                tag: "JEWEL IN THE CROWN",
                description: "Prestigious Monte Carlo street circuit with Loews Hairpin, the Tunnel, and Swimming Pool.",
                category: "Official GP Circuit",
                default_laps: 6,
                generator: Self::track_monaco,
            },
            TrackDefinition {
                id: "madring",
                title: "MadRing Circuito de Madrid",
                tag: "SPANISH STREET GP",
                description: "Spanish Grand Prix hybrid street circuit navigating the IFEMA complex and Valdebebas avenues.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_madring,
            },
            TrackDefinition {
                id: "marina_bay",
                title: "Marina Bay Street Circuit",
                tag: "SINGAPORE NIGHT RACE",
                description: "Spectacular floodlit street race navigating tight harbor chicanes and city avenues.",
                category: "Official GP Circuit",
                default_laps: 4,
                generator: Self::track_marina_bay,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
                color_scheme: CarColorScheme::from_index(6),
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                preferred_car: crate::ui::menu::CarChoice::GT3Car,
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
                name: "GT World Challenge Championship 2026".to_string(),
                point_system: PointSystem::F1Standard { fastest_lap_bonus: true },
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
        EngineAudioProfile::gt_v8()
    }
}
