use glam::Vec2;
use tdrace_app::config::{CameraConfig, ZoomLevelConfig};
use tdrace_app::editor::{EditorCamera, EditorState};
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::menu::TrackChoice;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, SurfaceShape, SurfaceZone};
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway,
    ramp_raceway,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::Track;

#[test]
fn test_all_seven_presets_json_roundtrip_and_validation() {
    let presets: Vec<(&str, Track)> = vec![
        ("Classic Grand Prix", classic_grand_prix()),
        ("Oval Speedway", oval_speedway()),
        ("Drift Park", drift_park()),
        ("Kart Arena", kart_arena()),
        ("Oasis Rally", oasis_rally()),
        ("Outlaw Pass", outlaw_pass()),
        ("Ramp Raceway", ramp_raceway()),
    ];

    for (name, track) in presets {
        // 1. JSON Roundtrip
        let json_str = track.to_json().expect("Failed to serialize track preset to JSON");
        assert!(!json_str.is_empty(), "Serialized JSON for {} was empty", name);

        let roundtrip_track = Track::from_json(&json_str)
            .unwrap_or_else(|e| panic!("Failed to deserialize track JSON for {}: {}", name, e));

        assert_eq!(track.name, roundtrip_track.name);
        assert_eq!(track.spline.waypoints.len(), roundtrip_track.spline.waypoints.len());
        assert_eq!(track.checkpoints.len(), roundtrip_track.checkpoints.len());
        assert_eq!(track.grid_positions.len(), roundtrip_track.grid_positions.len());
        assert_eq!(track.geometry.surface_zones.len(), roundtrip_track.geometry.surface_zones.len());
        assert_eq!(track.geometry.jump_ramps.len(), roundtrip_track.geometry.jump_ramps.len());

        // 2. Validation Engine
        let diagnostics = validate_track(&roundtrip_track);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "Preset {} contained validation errors: {:?}",
            name,
            errors
        );
    }
}

#[test]
fn test_track_editor_custom_circuit_lifecycle_and_io() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_editor_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut manager = TrackManager::new(temp_dir.clone());

    // 1. Create a custom track from scratch
    let mut track = classic_grand_prix();
    track.name = "Test Ring Raceway".to_string();
    track.geometry.surface_zones.clear();
    track.geometry.jump_ramps.clear();
    track.geometry.obstacles.clear();
    track.spline = TrackSpline::new(
        vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 14.0),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 12.0),
        ],
        true,
    );

    // Add surface zone
    track.geometry.surface_zones.push(SurfaceZone::new(
        SurfaceShape::OrientedBox {
            center: Vec2::new(50.0, -15.0),
            half_extents: Vec2::new(20.0, 10.0),
            angle: 0.0,
        },
        SurfaceType::Sand,
        "Runoff Sand",
    ));

    // Add jump ramp
    track.geometry.jump_ramps.push(JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: Vec2::new(50.0, 0.0),
            half_extents: Vec2::new(5.0, 7.0),
            angle: 0.0,
        },
        Vec2::new(1.0, 0.0),
        25.0,
        15.0,
        1.2,
        "Mega Jump",
    ));

    // Auto-generate checkpoints and grid
    track.auto_generate_checkpoints(8, 3);
    track.auto_generate_grid(6, 8.0, 3.0);
    track.rebuild_geometry(2.5, BarrierType::Armco);

    // Validate
    let errors: Vec<_> = validate_track(&track)
        .into_iter()
        .filter(|e| e.severity == ValidationSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "Created custom track had validation errors: {:?}", errors);

    // 2. Save via TrackManager
    let saved_path = manager
        .save_custom_track(&track, Some("test_ring_raceway.json"))
        .expect("Failed to save custom track");
    assert!(std::path::Path::new(&saved_path).exists());

    // 3. Reload from Disk
    let choice = TrackChoice::Custom {
        id: "test_ring_raceway".to_string(),
        title: "Test Ring Raceway".to_string(),
        description: "Test description".to_string(),
        path: saved_path,
    };
    let loaded = manager.load_track(&choice).expect("Failed to load saved track");
    assert_eq!(loaded.name, "Test Ring Raceway");
    assert_eq!(loaded.spline.waypoints.len(), 4);
    assert_eq!(loaded.geometry.jump_ramps.len(), 1);
    assert_eq!(loaded.geometry.surface_zones.len(), 1);

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_editor_state_undo_redo_and_selection() {
    let initial_track = classic_grand_prix();
    let mut state = EditorState::new(initial_track.clone());

    assert_eq!(state.history.undo_count(), 0);
    assert_eq!(state.history.redo_count(), 0);

    // Mutate state with snapshot
    state.record_undo();
    state.track.name = "Modified GP".to_string();
    assert_eq!(state.history.undo_count(), 1);

    // Undo
    let undid = state.undo();
    assert!(undid);
    assert_eq!(state.track.name, initial_track.name);
    assert_eq!(state.history.redo_count(), 1);

    // Redo
    let redid = state.redo();
    assert!(redid);
    assert_eq!(state.track.name, "Modified GP");
    assert_eq!(state.history.redo_count(), 0);
}

#[test]
fn test_validation_engine_catches_flaws() {
    // 1. Incomplete circuit (< 4 waypoints)
    let mut broken_track = classic_grand_prix();
    broken_track.name = "Broken Short Track".to_string();
    broken_track.spline.waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
        TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0),
    ];
    let errors = validate_track(&broken_track);
    assert!(errors.iter().any(|e| e.severity == ValidationSeverity::Error && e.message.contains("waypoint")));

    // 2. Missing finish line checkpoint
    broken_track.spline.waypoints.push(TrackWaypoint::new(Vec2::new(50.0, 50.0), 10.0));
    broken_track.spline.waypoints.push(TrackWaypoint::new(Vec2::new(0.0, 50.0), 10.0));
    broken_track.spline.closed = true;
    broken_track.checkpoints.clear();
    let errors2 = validate_track(&broken_track);
    assert!(errors2.iter().any(|e| e.severity == ValidationSeverity::Error && e.message.contains("checkpoint")));
}

#[test]
fn test_race_session_instant_test_drive_flow() {
    let mut session = RaceSession::new();
    let track = oval_speedway();

    // 1. Enter Track Studio
    session.enter_track_editor(track);
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(session.editor_state.is_some());

    // 2. Launch Instant Test Drive
    session.start_editor_test_drive();
    assert_eq!(session.state, GameState::EditorTestDrive);
    assert!(session.test_drive_car.is_some());
    assert!(session.test_drive_tracker.is_some());

    // 3. Step physics for 60 ticks (1.0 second)
    for _ in 0..60 {
        session.update_editor_test_drive(1.0 / 60.0);
    }

    let car = session.test_drive_car.as_ref().unwrap();
    assert!(car.state.position.is_finite());

    // 4. Return to Track Studio
    session.state = GameState::TrackEditor;
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(session.editor_state.is_some());
}

#[test]
fn test_editor_camera_zoom_levels_cycling_and_parity() {
    let mut camera = EditorCamera::new();
    assert_eq!(camera.levels.len(), 4);
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.current_zoom_level().name, "Close");
    assert_eq!(camera.target_zoom, 22.0);

    // 1. Cycle to Medium
    let lvl1 = camera.cycle_zoom_level();
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(lvl1.name, "Medium");
    assert_eq!(camera.target_zoom, 16.5);

    // 2. Cycle to Far
    let lvl2 = camera.cycle_zoom_level();
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(lvl2.name, "Far");
    assert_eq!(camera.target_zoom, 11.5);

    // 3. Cycle to Overview (without bounds)
    let lvl3 = camera.cycle_zoom_level();
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(lvl3.name, "Overview");
    assert_eq!(camera.target_zoom, 3.5);

    // 4. Cycle wraps around to Close
    let lvl0 = camera.cycle_zoom_level();
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(lvl0.name, "Close");
    assert_eq!(camera.target_zoom, 22.0);

    // Custom CameraConfig instantiation parity
    let mut custom_config = CameraConfig::default();
    custom_config.levels = vec![
        ZoomLevelConfig {
            name: "Tight".to_string(),
            mode: "follow".to_string(),
            min_zoom: 18.0,
            max_zoom: 28.0,
        },
        ZoomLevelConfig {
            name: "Wide".to_string(),
            mode: "overview".to_string(),
            min_zoom: 4.0,
            max_zoom: 4.0,
        },
    ];
    custom_config.default_level_index = 1;

    let custom_cam = EditorCamera::from_config(&custom_config);
    assert_eq!(custom_cam.levels.len(), 2);
    assert_eq!(custom_cam.current_level_idx, 1);
    assert_eq!(custom_cam.current_zoom_level().name, "Wide");
    assert_eq!(custom_cam.target_zoom, 4.0);
}

#[test]
fn test_editor_camera_zoom_in_and_zoom_out() {
    let mut camera = EditorCamera::new();
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.current_zoom_level().name, "Close");
    assert_eq!(camera.target_zoom, 22.0);

    // Boundary at index 0
    assert!(camera.zoom_in().is_none());
    assert_eq!(camera.current_level_idx, 0);

    // Step-by-step zoom out
    let lvl1 = camera.zoom_out().expect("Should zoom out to Medium");
    assert_eq!(lvl1.name, "Medium");
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.target_zoom, 16.5);

    let lvl2 = camera.zoom_out().expect("Should zoom out to Far");
    assert_eq!(lvl2.name, "Far");
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(camera.target_zoom, 11.5);

    let lvl3 = camera.zoom_out().expect("Should zoom out to Overview");
    assert_eq!(lvl3.name, "Overview");
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(camera.target_zoom, 3.5);

    // Boundary at index 3
    assert!(camera.zoom_out().is_none());
    assert_eq!(camera.current_level_idx, 3);

    // Step-by-step zoom in
    let in2 = camera.zoom_in().expect("Should zoom in to Far");
    assert_eq!(in2.name, "Far");
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(camera.target_zoom, 11.5);

    let in1 = camera.zoom_in().expect("Should zoom in to Medium");
    assert_eq!(in1.name, "Medium");
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.target_zoom, 16.5);

    let in0 = camera.zoom_in().expect("Should zoom in to Close");
    assert_eq!(in0.name, "Close");
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.target_zoom, 22.0);

    assert!(camera.zoom_in().is_none());
}

#[test]
fn test_editor_camera_overview_with_bounds_framing() {
    let mut camera = EditorCamera::new();
    let min_pt = Vec2::new(0.0, 0.0);
    let max_pt = Vec2::new(200.0, 100.0);
    let bounds = Some((min_pt, max_pt));

    // Jump to Overview level (index 3) with track bounds
    let lvl = camera.set_zoom_level_with_bounds(3, bounds, 1280.0, 720.0);
    assert_eq!(lvl.name, "Overview");
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(camera.target_center, Vec2::new(100.0, 50.0));
    assert!(camera.target_zoom > 0.5 && camera.target_zoom < 20.0);

    // focus_bounds explicitly sets level index to Overview
    camera.set_zoom_level(0); // Switch to Close
    assert_eq!(camera.current_level_idx, 0);
    camera.focus_bounds(min_pt, max_pt, 1280.0, 720.0);
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(camera.current_zoom_level().name, "Overview");
}

#[test]
fn test_obstacle_duplication_and_undo() {
    use tdrace_app::editor::{Selection, ToolSettings};
    use tdrace_core::track::geometry::Obstacle;

    let mut state = EditorState::new(classic_grand_prix());
    state.track.geometry.obstacles.push(Obstacle::circle(1, Vec2::new(100.0, 50.0), 1.5, "Editor Test Obstacle"));
    let initial_obs_count = state.track.geometry.obstacles.len();
    assert!(initial_obs_count > 0);

    // Select obstacle 0
    state.selection = Selection::Obstacle(0);
    let original_center = state.track.geometry.obstacles[0].center();

    let mut tools = ToolSettings::default();
    let dup_success = tools.duplicate_selected(&mut state);
    assert!(dup_success, "Duplication should succeed for selected obstacle");

    // Check new obstacle count and selection
    assert_eq!(state.track.geometry.obstacles.len(), initial_obs_count + 1);
    assert_eq!(state.selection, Selection::Obstacle(initial_obs_count));
    let dup_center = state.track.geometry.obstacles[initial_obs_count].center();
    assert_eq!(dup_center, original_center + Vec2::new(4.0, 4.0));

    // Test Undo
    assert!(state.undo());
    assert_eq!(state.track.geometry.obstacles.len(), initial_obs_count);

    // Test Redo
    assert!(state.redo());
    assert_eq!(state.track.geometry.obstacles.len(), initial_obs_count + 1);
}

#[test]
fn test_polygon_obstacle_tool_vertex_placement() {
    use tdrace_app::editor::{EditorToolType, ObstacleShapeType, Selection, ToolSettings};
    use tdrace_core::track::geometry::ObstacleShape;

    let mut state = EditorState::new(classic_grand_prix());
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::Obstacle;
    tools.active_obstacle_shape = ObstacleShapeType::Polygon;

    let initial_obs_count = state.track.geometry.obstacles.len();

    // 1. Add vertex 1
    tools.handle_mouse_down(&mut state, Vec2::new(10.0, 10.0));
    assert_eq!(tools.active_polygon_vertices.len(), 1);

    // 2. Add vertex 2
    tools.handle_mouse_down(&mut state, Vec2::new(20.0, 10.0));
    assert_eq!(tools.active_polygon_vertices.len(), 2);

    // 3. Add vertex 3
    tools.handle_mouse_down(&mut state, Vec2::new(15.0, 20.0));
    assert_eq!(tools.active_polygon_vertices.len(), 3);

    // 4. Click close near vertex 1 (within 1.5m radius)
    tools.handle_mouse_down(&mut state, Vec2::new(10.5, 10.5));

    // Should close polygon and create obstacle!
    assert!(tools.active_polygon_vertices.is_empty());
    assert_eq!(state.track.geometry.obstacles.len(), initial_obs_count + 1);
    assert_eq!(state.selection, Selection::Obstacle(initial_obs_count));

    let created_obs = &state.track.geometry.obstacles[initial_obs_count];
    match &created_obs.shape {
        ObstacleShape::Polygon { vertices } => {
            assert_eq!(vertices.len(), 3);
            assert_eq!(vertices[0], Vec2::new(10.0, 10.0));
            assert_eq!(vertices[1], Vec2::new(20.0, 10.0));
            assert_eq!(vertices[2], Vec2::new(15.0, 20.0));
        }
        _ => panic!("Expected Polygon obstacle shape!"),
    }
}

#[test]
fn test_editor_camera_arrow_panning_and_speed_scaling() {
    let mut camera = EditorCamera::new();
    camera.target_center = Vec2::new(100.0, 100.0);
    camera.center = Vec2::new(100.0, 100.0);
    camera.zoom = 20.0;
    camera.target_zoom = 20.0;

    // 1. Pan Up (+Y)
    camera.pan_direction(Vec2::new(0.0, 1.0), 1.0, 0.1);
    assert!(camera.target_center.y > 100.0);
    assert_eq!(camera.target_center.x, 100.0);

    // 2. Pan Right (+X) with 2.5x Shift boost
    let prev_y = camera.target_center.y;
    camera.pan_direction(Vec2::new(1.0, 0.0), 2.5, 0.1);
    assert!(camera.target_center.x > 100.0);
    assert_eq!(camera.target_center.y, prev_y);

    // 3. Smooth update interpolates center towards target_center
    camera.update(0.1);
    assert!(camera.center.x > 100.0);
    assert!(camera.center.y > 100.0);
}

#[test]
fn test_road_spline_add_relative_to_current_or_last_point() {
    use tdrace_app::editor::{EditorToolType, Selection, ToolSettings};

    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::RoadSpline;

    let initial_count = state.track.spline.waypoints.len();

    // Select waypoint 4
    state.select(Selection::Waypoint(4));
    let wp4_pt = state.track.spline.waypoints[4].point;

    // Add a new spline point at (350.0, 85.0)
    let new_pos = Vec2::new(350.0, 85.0);
    tools.handle_mouse_down(&mut state, new_pos);
    tools.handle_mouse_up(&mut state, new_pos);

    // It should be inserted at index 5 (right after waypoint 4)
    assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
    assert_eq!(state.selection, Selection::Waypoint(5));
    assert_eq!(state.track.spline.waypoints[4].point, wp4_pt);
    assert_eq!(state.track.spline.waypoints[5].point, new_pos);
    assert_eq!(state.last_selected_waypoint, Some(5));

    // Clear selection, but remember last selected point 5
    state.deselect();
    assert_eq!(state.selection, Selection::None);

    // Add another point at (360.0, 100.0) -> should be placed at index 6 (after last selected point 5)
    let new_pos2 = Vec2::new(360.0, 100.0);
    tools.handle_mouse_down(&mut state, new_pos2);
    tools.handle_mouse_up(&mut state, new_pos2);

    assert_eq!(state.track.spline.waypoints.len(), initial_count + 2);
    assert_eq!(state.selection, Selection::Waypoint(6));
    assert_eq!(state.track.spline.waypoints[6].point, new_pos2);
    assert_eq!(state.last_selected_waypoint, Some(6));
}

#[test]
fn test_all_entity_duplications_and_undo() {
    use tdrace_app::editor::{Selection, ToolSettings};
    use tdrace_core::track::geometry::{JumpRamp, SurfaceShape, SurfaceZone};
    use tdrace_core::physics::surface::SurfaceType;

    let mut state = EditorState::new(classic_grand_prix());
    let mut tools = ToolSettings::default();

    // 1. Surface Zone Duplication
    state.track.geometry.surface_zones.push(SurfaceZone::new(
        SurfaceShape::Circle { center: Vec2::new(100.0, 100.0), radius: 15.0 },
        SurfaceType::Dirt,
        "Gravel Trap",
    ));
    let zone_idx = state.track.geometry.surface_zones.len() - 1;
    state.selection = Selection::SurfaceZone(zone_idx);
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.geometry.surface_zones.len(), zone_idx + 2);
    assert_eq!(state.selection, Selection::SurfaceZone(zone_idx + 1));
    assert_eq!(state.track.geometry.surface_zones[zone_idx + 1].name, "Gravel Trap (Copy)");
    assert!(state.undo());
    assert_eq!(state.track.geometry.surface_zones.len(), zone_idx + 1);

    // 2. Jump Ramp Duplication
    state.track.geometry.jump_ramps.push(JumpRamp {
        id: 10,
        name: "Big Air Ramp".to_string(),
        shape: SurfaceShape::OrientedBox {
            center: Vec2::new(50.0, 50.0),
            half_extents: Vec2::new(10.0, 5.0),
            angle: 0.0,
        },
        direction: Vec2::new(1.0, 0.0),
        launch_speed: 25.0,
        height: 2.5,
        ramp_angle_deg: 18.0,
    });
    let ramp_idx = state.track.geometry.jump_ramps.len() - 1;
    state.selection = Selection::JumpRamp(ramp_idx);
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.geometry.jump_ramps.len(), ramp_idx + 2);
    assert_eq!(state.selection, Selection::JumpRamp(ramp_idx + 1));
    assert_eq!(state.track.geometry.jump_ramps[ramp_idx + 1].name, "Big Air Ramp (Copy)");
    assert!(state.undo());
    assert_eq!(state.track.geometry.jump_ramps.len(), ramp_idx + 1);

    // 3. Waypoint Duplication
    let wp_count = state.track.spline.waypoints.len();
    state.selection = Selection::Waypoint(2);
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.spline.waypoints.len(), wp_count + 1);
    assert_eq!(state.selection, Selection::Waypoint(3));
    assert!(state.undo());
    assert_eq!(state.track.spline.waypoints.len(), wp_count);

    // 4. Checkpoint Duplication
    let cp_count = state.track.checkpoints.len();
    state.selection = Selection::Checkpoint(0);
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.checkpoints.len(), cp_count + 1);
    assert_eq!(state.selection, Selection::Checkpoint(cp_count));
    assert!(state.undo());
    assert_eq!(state.track.checkpoints.len(), cp_count);

    // 5. Grid Slot Duplication
    let grid_count = state.track.grid_positions.len();
    state.selection = Selection::GridSlot(0);
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.grid_positions.len(), grid_count + 1);
    assert_eq!(state.selection, Selection::GridSlot(grid_count));
    assert!(state.undo());
    assert_eq!(state.track.grid_positions.len(), grid_count);
}

#[test]
fn test_track_editor_spline_surface_inheritance_and_switching() {
    use tdrace_app::editor::{EditorToolType, Selection, ToolSettings};

    let track = oasis_rally();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::RoadSpline;

    let initial_count = state.track.spline.waypoints.len();

    // 1. Select a dirt waypoint (all waypoints in Oasis Rally are Dirt)
    state.select(Selection::Waypoint(3));
    assert_eq!(state.track.spline.waypoints[3].surface, Some(SurfaceType::Dirt));

    // 2. Add a new spline point after waypoint 3
    let new_pos = Vec2::new(170.0, 30.0);
    tools.handle_mouse_down(&mut state, new_pos);
    tools.handle_mouse_up(&mut state, new_pos);

    // It should be inserted at index 4 and inherit SurfaceType::Dirt (not tarmac/asphalt!)
    assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
    assert_eq!(state.selection, Selection::Waypoint(4));
    assert_eq!(state.track.spline.waypoints[4].surface, Some(SurfaceType::Dirt));

    // 3. User switches surface of waypoint 4 to Sand
    state.track.spline.waypoints[4].surface = Some(SurfaceType::Sand);
    tools.active_surface = SurfaceType::Sand;
    state.rebuild_geometry();

    // 4. Add another point after waypoint 4
    let new_pos2 = Vec2::new(180.0, 45.0);
    tools.handle_mouse_down(&mut state, new_pos2);
    tools.handle_mouse_up(&mut state, new_pos2);

    // It should be inserted at index 5 and inherit SurfaceType::Sand
    assert_eq!(state.track.spline.waypoints.len(), initial_count + 2);
    assert_eq!(state.selection, Selection::Waypoint(5));
    assert_eq!(state.track.spline.waypoints[5].surface, Some(SurfaceType::Sand));
}

#[test]
fn test_editor_camera_progressive_zoom_in_out() {
    let mut camera = EditorCamera::new();
    camera.target_zoom = 10.0;
    camera.zoom = 10.0;

    let center_screen = Vec2::new(640.0, 360.0);

    // Zoom In (+1.0 dir) for 0.5s at 1.0x speed multiplier
    camera.zoom_progressive(center_screen, 1.0, 1.0, 0.5, 1280.0, 720.0);
    assert!(camera.target_zoom > 10.0);
    assert!(camera.zoom > 10.0);
    let zoomed_in = camera.zoom;

    // Zoom Out (-1.0 dir) for 0.5s at 1.0x speed multiplier
    camera.zoom_progressive(center_screen, -1.0, 1.0, 0.5, 1280.0, 720.0);
    assert!(camera.zoom < zoomed_in);
    assert!((camera.zoom - 10.0).abs() < 0.1);

    // Progressive zoom clamping bounds
    camera.zoom_progressive(center_screen, -1.0, 10.0, 10.0, 1280.0, 720.0);
    assert_eq!(camera.zoom, camera.min_zoom);
    assert_eq!(camera.target_zoom, camera.min_zoom);

    camera.zoom_progressive(center_screen, 1.0, 10.0, 10.0, 1280.0, 720.0);
    assert_eq!(camera.zoom, camera.max_zoom);
    assert_eq!(camera.target_zoom, camera.max_zoom);
}

#[test]
fn test_track_editor_overwrite_vs_save_as_new_copy_flow() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_editor_overwrite_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut manager = TrackManager::new(temp_dir.clone());

    // 1. Initial track creation and save
    let mut track = classic_grand_prix();
    track.name = "Original Circuit".to_string();
    let initial_path = manager
        .save_custom_track_with_options(&track, Some("original_circuit"), true)
        .expect("Initial save must succeed");

    // 2. Simulate loading track into editor state
    let choice = TrackChoice::Custom {
        id: "original_circuit".to_string(),
        title: "Original Circuit".to_string(),
        description: "Desc".to_string(),
        path: initial_path.clone(),
    };
    let loaded_track = manager.load_track(&choice).expect("Must load track");
    let mut editor_state = EditorState::new(loaded_track);
    editor_state.current_file_path = Some(initial_path.clone());

    assert_eq!(editor_state.current_file_path, Some(initial_path.clone()));

    // 3. User modifies the track name and description
    editor_state.record_undo();
    editor_state.track.name = "Updated Circuit".to_string();
    editor_state.track.description = "Updated circuit description with high speed turns.".to_string();

    // 4. Overwrite existing track
    let slug = std::path::Path::new(editor_state.current_file_path.as_ref().unwrap())
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    assert_eq!(slug, Some("original_circuit".to_string()));

    let overwritten_path = manager
        .save_custom_track_with_options(&editor_state.track, slug.as_deref(), true)
        .expect("Overwrite save must succeed");
    assert_eq!(overwritten_path, initial_path);

    // Verify overwritten content
    let verify_track = Track::load_from_file(&overwritten_path).expect("Must load overwritten file");
    assert_eq!(verify_track.name, "Updated Circuit");
    assert_eq!(verify_track.description, "Updated circuit description with high speed turns.");

    // 6. User chooses "Save as Copy" with custom specified filename
    let custom_filename_path = manager
        .save_custom_track_with_options(&editor_state.track, Some("my_custom_filename"), false)
        .expect("Save with custom filename must succeed");
    assert!(custom_filename_path.ends_with("my_custom_filename.json"));
    assert!(std::path::Path::new(&custom_filename_path).exists());
    let custom_saved_track = Track::load_from_file(&custom_filename_path).expect("Load custom filename file");
    assert_eq!(custom_saved_track.name, "Updated Circuit");

    // 7. Verify preset overwrite and disk loading
    let preset_path = manager
        .save_custom_track_with_options(&editor_state.track, Some("classic_grand_prix"), true)
        .expect("Save preset to disk must succeed");
    assert!(preset_path.ends_with("classic_grand_prix.json"));
    let loaded_preset_choice = manager
        .load_track(&TrackChoice::ClassicGrandPrix)
        .expect("Load preset must load from disk file when present");
    assert_eq!(loaded_preset_choice.name, "Updated Circuit");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_editor_unsaved_changes_exit_flow() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_unsaved_exit_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let mut session = RaceSession::new();
    session.track_manager = TrackManager::new(temp_dir.clone());

    let track = classic_grand_prix();
    session.enter_track_editor(track);

    assert_eq!(session.state, GameState::TrackEditor);
    assert_eq!(session.editor_modal, tdrace_app::editor::EditorModal::None);
    assert!(!session.editor_state.as_ref().unwrap().is_dirty);

    // 1. If clean, exiting transitions directly to Menu
    session.handle_editor_action(tdrace_app::editor::EditorAction::ExitToMenu);
    assert_eq!(session.state, GameState::Menu);

    // 2. Re-enter and modify track to make state dirty
    let track = classic_grand_prix();
    session.enter_track_editor(track);
    session.editor_state.as_mut().unwrap().record_undo();
    session.editor_state.as_mut().unwrap().track.name = "Modified GP".to_string();
    assert!(session.editor_state.as_ref().unwrap().is_dirty);

    // 3. Simulate SaveTrack without exit_after (stays in editor)
    session.handle_editor_action(tdrace_app::editor::EditorAction::SaveTrack {
        name: "Modified GP".to_string(),
        filename: "modified_gp_stay".to_string(),
        description: "Stay in editor".to_string(),
        overwrite: false,
        exit_after: false,
    });
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(!session.editor_state.as_ref().unwrap().is_dirty);

    // 4. Modify again and simulate SaveTrack with exit_after (transitions to Menu)
    session.editor_state.as_mut().unwrap().record_undo();
    assert!(session.editor_state.as_ref().unwrap().is_dirty);

    session.handle_editor_action(tdrace_app::editor::EditorAction::SaveTrack {
        name: "Modified GP".to_string(),
        filename: "modified_gp_exit".to_string(),
        description: "Exit after save".to_string(),
        overwrite: false,
        exit_after: true,
    });
    assert_eq!(session.state, GameState::Menu);
    assert!(!session.editor_state.as_ref().unwrap().is_dirty);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_editing_snapshot_regeneration_and_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_snapshot_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    let initial_track = classic_grand_prix();
    let initial_wp_count = initial_track.spline.waypoints.len();
    let (initial_min, initial_max) = tdrace_app::ui::compute_track_bounds(&initial_track);

    // 1. Enter editor with track
    session.enter_track_editor(initial_track);
    assert_eq!(session.state, GameState::TrackEditor);

    // 2. Modify track waypoints (e.g. extend circuit with a far waypoint)
    if let Some(state) = &mut session.editor_state {
        state.record_undo();
        state.track.spline.waypoints.push(tdrace_core::track::spline::TrackWaypoint::new(
            glam::Vec2::new(5000.0, 5000.0),
            16.0,
        ));
        state.rebuild_geometry();
    }

    // 3. Save modified track
    session.handle_editor_action(tdrace_app::editor::EditorAction::SaveTrack {
        name: "Extended Circuit".to_string(),
        filename: "extended_circuit".to_string(),
        description: "Regenerated snapshot track".to_string(),
        overwrite: false,
        exit_after: true,
    });
    assert_eq!(session.state, GameState::Menu);

    // 4. Verify TrackManager custom tracks metadata and bounds updated
    let draft_choices = session.track_manager.draft_track_choices();
    assert_eq!(draft_choices.len(), 1);
    let choice = &draft_choices[0];

    let reloaded_track = session.track_manager.load_track(choice).expect("Must load updated track");
    assert_eq!(reloaded_track.spline.waypoints.len(), initial_wp_count + 1);

    // 5. Verify snapshot bounds regenerated
    let (new_min, new_max) = tdrace_app::ui::compute_track_bounds(&reloaded_track);
    assert!(new_max.x >= 5000.0, "Bounds max X must expand to include new waypoint");
    assert!(new_max.y >= 5000.0, "Bounds max Y must expand to include new waypoint");
    assert_ne!((initial_min, initial_max), (new_min, new_max));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_save_modal_track_name_integrity_and_initialization() {
    let mut track = classic_grand_prix();
    track.name = "Silverstone International".to_string();
    track.description = "Historic high-speed circuit.".to_string();

    let mut session = RaceSession::new();
    session.enter_track_editor(track.clone());

    if let Some(state) = &session.editor_state {
        let initial_filename = TrackManager::sanitize_slug(&state.track.name);
        assert_eq!(state.track.name, "Silverstone International");
        assert_eq!(initial_filename, "silverstone_international");

        // Verify SaveAs modal initializes with exact name without appended characters
        let modal = tdrace_app::editor::EditorModal::SaveAs {
            input_name: state.track.name.clone(),
            input_filename: initial_filename,
            input_description: state.track.description.clone(),
            active_field: 0,
            overwrite: false,
            custom_filename_edited: false,
            exit_on_save: false,
        };

        if let tdrace_app::editor::EditorModal::SaveAs {
            input_name,
            input_filename,
            input_description,
            active_field,
            overwrite,
            custom_filename_edited,
            exit_on_save,
        } = modal {
            assert_eq!(input_name, "Silverstone International");
            assert_eq!(input_filename, "silverstone_international");
            assert_eq!(input_description, "Historic high-speed circuit.");
            assert_eq!(active_field, 0);
            assert!(!overwrite);
            assert!(!custom_filename_edited);
            assert!(!exit_on_save);
        } else {
            panic!("Expected SaveAs modal");
        }
    } else {
        panic!("Expected editor state");
    }
}

#[test]
fn test_track_editor_surface_shapes_and_layering_e2e() {
    use tdrace_core::track::geometry::{SurfaceLayer, SurfaceShape, SurfaceZone};
    use tdrace_core::track::presets::classic_grand_prix;

    let mut track = classic_grand_prix();
    let sample_road_pt = track.spline.waypoints[0].point;

    // 1. BelowTrack zone under the road should be overridden by road surface
    track.geometry.surface_zones.push(
        SurfaceZone::new(
            SurfaceShape::Circle { center: sample_road_pt, radius: 25.0 },
            SurfaceType::Sand,
            "Under-Road Sand",
        )
        .with_layer(SurfaceLayer::BelowTrack),
    );

    let sampled_surf = track.sample_surface(sample_road_pt);
    assert_eq!(sampled_surf, SurfaceType::Asphalt, "Road asphalt must take precedence over BelowTrack sand");

    // 2. AboveTrack zone over the road should override road asphalt (e.g. oil slick or water puddle on track)
    track.geometry.surface_zones.push(
        SurfaceZone::new(
            SurfaceShape::Circle { center: sample_road_pt, radius: 10.0 },
            SurfaceType::Water,
            "On-Track Water Hazard",
        )
        .with_layer(SurfaceLayer::AboveTrack),
    );

    let sampled_surf_over = track.sample_surface(sample_road_pt);
    assert_eq!(sampled_surf_over, SurfaceType::Water, "AboveTrack water hazard must take precedence over road ribbon");

    // 3. Add Triangle and Polygon shapes
    track.geometry.surface_zones.push(
        SurfaceZone::new(
            SurfaceShape::triangle(
                Vec2::new(300.0, 300.0),
                Vec2::new(340.0, 300.0),
                Vec2::new(320.0, 340.0),
            ),
            SurfaceType::Dirt,
            "Triangle Dirt",
        )
        .with_layer(SurfaceLayer::BelowTrack),
    );

    track.geometry.surface_zones.push(
        SurfaceZone::new(
            SurfaceShape::Polygon {
                vertices: vec![
                    Vec2::new(400.0, 400.0),
                    Vec2::new(450.0, 400.0),
                    Vec2::new(460.0, 440.0),
                    Vec2::new(410.0, 450.0),
                ],
            },
            SurfaceType::Grass,
            "Custom Polygon Runoff",
        )
        .with_layer(SurfaceLayer::AboveTrack),
    );

    // 4. JSON Serialization & Deserialization Roundtrip
    let json = serde_json::to_string_pretty(&track).expect("Failed to serialize track with multi-shapes & layers");
    let restored: Track = serde_json::from_str(&json).expect("Failed to deserialize track");

    assert_eq!(restored.geometry.surface_zones.len(), track.geometry.surface_zones.len());
    assert_eq!(restored.geometry.surface_zones.last().unwrap().layer, SurfaceLayer::AboveTrack);
    assert!(matches!(restored.geometry.surface_zones.last().unwrap().shape, SurfaceShape::Polygon { .. }));
}

#[test]
fn test_track_editor_multi_segment_batch_operations_e2e() {
    use tdrace_app::editor::{EditorState, Selection, ToolSettings};
    use tdrace_core::track::presets::classic_grand_prix;

    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Multi-select waypoints 0, 1, 2
    state.selection = Selection::MultipleWaypoints(vec![0, 1, 2]);

    let w0 = state.track.spline.waypoints[0].width;
    let w1 = state.track.spline.waypoints[1].width;
    let w2 = state.track.spline.waypoints[2].width;

    // 2. Batch adjust width +2m
    assert!(tools.batch_adjust_width(&mut state, 2.0));
    assert_eq!(state.track.spline.waypoints[0].width, w0 + 2.0);
    assert_eq!(state.track.spline.waypoints[1].width, w1 + 2.0);
    assert_eq!(state.track.spline.waypoints[2].width, w2 + 2.0);

    // Batch set fixed width
    assert!(tools.batch_set_width(&mut state, 18.0));
    assert_eq!(state.track.spline.waypoints[0].width, 18.0);
    assert_eq!(state.track.spline.waypoints[1].width, 18.0);
    assert_eq!(state.track.spline.waypoints[2].width, 18.0);

    // 3. Batch set curbs to both sides
    assert!(tools.batch_set_curbs(&mut state, true, true));
    assert!(state.track.spline.waypoints[0].left_curb && state.track.spline.waypoints[0].right_curb);
    assert!(state.track.spline.waypoints[1].left_curb && state.track.spline.waypoints[1].right_curb);

    // 4. Batch set surface to Ice
    assert!(tools.batch_set_surface(&mut state, Some(SurfaceType::Ice)));
    assert_eq!(state.track.spline.waypoints[0].surface, Some(SurfaceType::Ice));
    assert_eq!(state.track.spline.waypoints[1].surface, Some(SurfaceType::Ice));

    // 5. Batch duplicate and undo
    let initial_count = state.track.spline.waypoints.len();
    assert!(tools.duplicate_selected(&mut state));
    assert_eq!(state.track.spline.waypoints.len(), initial_count + 3);

    assert!(state.undo());
    assert_eq!(state.track.spline.waypoints.len(), initial_count);
}

