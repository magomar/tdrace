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
use tdrace_core::track::spline::TrackWaypoint;
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
    track.spline.waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
        TrackWaypoint::new(Vec2::new(100.0, 100.0), 14.0),
        TrackWaypoint::new(Vec2::new(0.0, 100.0), 12.0),
    ];
    track.spline.closed = true;

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

    let mut state = EditorState::new(classic_grand_prix());
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

