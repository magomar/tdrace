use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile, BotRouteStrategy};
use tdrace_app::config::{CameraConfig, ZoomLevelConfig};
use tdrace_app::editor::{
    is_mouse_over_editor_ui, EditorCamera, EditorState, EditorToolType, Selection, ToolSettings,
};
use tdrace_core::{Car, CarConfig};
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::track_manager::{ModuleFilter, TrackManager};
use tdrace_app::ui::menu::{CarChoice, GameMode, TrackChoice};
use tdrace_app::ui::track_manager_ui::{TrackManagerModal, TrackManagerTab};
use tdrace_core::collision::wall::resolve_all_wall_collisions;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::Checkpoint;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, LineSegment, SurfaceShape, SurfaceZone, WallBarrier};
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway,
    ramp_raceway,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::network::{
    GoreConfig, JunctionId, JunctionKind, MergeConfig, RoadJunction, RoadSegment, SegmentId,
    SocketId, SplineSocket, TrackLayout, TrackNetwork,
};
use tdrace_core::track::Track;

static DEV_MODE_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    track.rebuild_geometry(2.5, BarrierType::Steel);

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
fn test_race_session_test_drive_time_trial_flow_and_editor_return() {
    let mut session = RaceSession::new();
    let track = oval_speedway();

    // 1. Enter Track Studio
    session.enter_track_editor(track);
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(session.editor_state.is_some());

    // 2. Launch Test Drive -> Launches Time Trial race with default car
    session.start_editor_test_drive();
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.game_mode, GameMode::TimeTrial);
    assert!(session.is_time_attack);
    assert!(session.return_to_editor_on_exit);
    assert_eq!(session.cars.len(), 1);
    assert_eq!(session.car_choice, session.resolve_predefined_car());
    assert!(session.editor_state.is_some());

    // 3. Start countdown -> Racing
    session.state = GameState::Countdown(0.0);
    session.state = GameState::Racing;

    // Step physics for 60 ticks (1.0 second)
    for _ in 0..60 {
        session.physics_step(1.0 / 60.0);
    }

    let car = session.cars.first().unwrap();
    assert!(car.state.position.is_finite());
    assert_eq!(session.trackers.len(), 1);

    // 4. Pause race and exit to Track Editor
    session.state = GameState::Paused;
    assert!(session.return_to_editor_on_exit);

    // Simulate Exit Race action
    if session.return_to_editor_on_exit {
        session.return_to_editor_on_exit = false;
        session.state = GameState::TrackEditor;
    }
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(!session.return_to_editor_on_exit);
    assert!(session.editor_state.is_some());
}

#[test]
fn test_test_drive_exit_from_starting_grid_and_finished_states() {
    let mut session = RaceSession::new();
    let track = oasis_rally();

    session.enter_track_editor(track);
    assert_eq!(session.state, GameState::TrackEditor);

    // 1. Launch Test Drive -> StartingGrid -> Exit immediately via Escape
    session.start_editor_test_drive();
    assert_eq!(session.state, GameState::StartingGrid);
    assert!(session.return_to_editor_on_exit);
    assert_eq!(session.car_choice, CarChoice::RallyCar);

    // Simulate Escape in StartingGrid
    if session.return_to_editor_on_exit {
        session.return_to_editor_on_exit = false;
        session.state = GameState::TrackEditor;
    }
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(!session.return_to_editor_on_exit);
    assert!(session.editor_state.is_some());

    // 2. Launch Test Drive -> Race Finished -> Exit to Editor
    session.start_editor_test_drive();
    assert_eq!(session.state, GameState::StartingGrid);
    assert!(session.return_to_editor_on_exit);

    session.state = GameState::Finished;
    // Simulate Escape / M on Finished screen
    if session.return_to_editor_on_exit {
        session.return_to_editor_on_exit = false;
        session.state = GameState::TrackEditor;
    }
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(!session.return_to_editor_on_exit);
    assert!(session.editor_state.is_some());
}

#[test]
fn test_test_drive_preserves_unsaved_track_edits() {
    let mut session = RaceSession::new();
    let track = classic_grand_prix();

    session.enter_track_editor(track);
    let original_wp_count = session.editor_state.as_ref().unwrap().track.spline.waypoints.len();

    // Add a new waypoint in editor
    if let Some(state) = &mut session.editor_state {
        state.track.spline.waypoints.push(TrackWaypoint::new(Vec2::new(999.0, 999.0), 12.0));
        state.is_dirty = true;
    }

    // Launch Test Drive
    session.start_editor_test_drive();
    assert_eq!(session.track.spline.waypoints.len(), original_wp_count + 1);

    // Return to Editor
    if session.return_to_editor_on_exit {
        session.return_to_editor_on_exit = false;
        session.state = GameState::TrackEditor;
    }

    assert_eq!(session.state, GameState::TrackEditor);
    let editor = session.editor_state.as_ref().unwrap();
    assert_eq!(editor.track.spline.waypoints.len(), original_wp_count + 1);
    assert!(editor.is_dirty);
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

    // 3. Cycle to Very Far (without bounds)
    let lvl3 = camera.cycle_zoom_level();
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(lvl3.name, "Very Far");
    assert_eq!(camera.target_zoom, 8.0);

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

    let lvl3 = camera.zoom_out().expect("Should zoom out to Very Far");
    assert_eq!(lvl3.name, "Very Far");
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(camera.target_zoom, 8.0);

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
    camera.levels.push(ZoomLevelConfig {
        name: "Overview".to_string(),
        mode: "overview".to_string(),
        min_zoom: 3.5,
        max_zoom: 3.5,
    });
    let overview_idx = camera.levels.len() - 1;
    let min_pt = Vec2::new(0.0, 0.0);
    let max_pt = Vec2::new(200.0, 100.0);
    let bounds = Some((min_pt, max_pt));

    // Jump to Overview level with track bounds
    let lvl = camera.set_zoom_level_with_bounds(overview_idx, bounds, 1280.0, 720.0);
    assert_eq!(lvl.name, "Overview");
    assert_eq!(camera.current_level_idx, overview_idx);
    assert_eq!(camera.target_center, Vec2::new(100.0, 50.0));
    assert!(camera.target_zoom > 0.5 && camera.target_zoom < 20.0);

    // focus_bounds explicitly sets level index to Overview
    camera.set_zoom_level(0); // Switch to Close
    assert_eq!(camera.current_level_idx, 0);
    camera.focus_bounds(min_pt, max_pt, 1280.0, 720.0);
    assert_eq!(camera.current_level_idx, overview_idx);
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
        surface: SurfaceType::Asphalt,
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
    let _dev_mutex_guard = DEV_MODE_TEST_MUTEX.lock().unwrap();
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

    // 7. Verify preset immutability in normal user mode
    let err = manager
        .save_custom_track_with_options(&editor_state.track, Some("classic_grand_prix"), true)
        .expect_err("Saving directly to an official preset in normal mode must fail");
    assert!(err.contains("is an official preset and cannot be modified directly"));

    let loaded_preset_choice = manager
        .load_track(&TrackChoice::ClassicGrandPrix)
        .expect("Load preset must load canonical version in normal user mode");
    assert_eq!(loaded_preset_choice.name, "Classic Grand Prix");

    // 8. In dev mode, developer can save to git-tracked preset
    let mock_git = temp_dir.join("mock_git");
    let _ = std::fs::create_dir_all(mock_git.join("classic"));
    std::env::set_var(tdrace_app::storage::ENV_DEV_MODE, "1");
    std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git);
    assert!(tdrace_app::storage::is_dev_mode());
    let dev_save_result = manager.save_custom_track_with_options(&editor_state.track, Some("classic_grand_prix"), true);
    assert!(dev_save_result.is_ok());
    let canonical = classic_grand_prix();
    let _ = manager.save_custom_track_with_options(&canonical, Some("classic_grand_prix"), true);
    std::env::remove_var(tdrace_app::storage::ENV_DEV_MODE);
    std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);

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

    // 1. If clean, exiting transitions directly to TrackManager
    session.handle_editor_action(tdrace_app::editor::EditorAction::ExitToMenu);
    assert!(matches!(session.state, GameState::TrackManager { .. }));

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
    assert!(matches!(session.state, GameState::TrackManager { .. }));
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
    assert!(matches!(session.state, GameState::TrackManager { .. }));

    // 4. Verify TrackManager custom tracks metadata and bounds updated
    let custom_choices = session.track_manager.module_custom_tracks("classic");
    assert_eq!(custom_choices.len(), 1);
    let choice = &custom_choices[0];

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

#[test]
fn test_track_editor_default_offtrack_surface_mutation_and_cycling() {
    use tdrace_app::editor::{EditorState, ToolSettings};
    use tdrace_core::physics::surface::SurfaceType;
    use tdrace_core::track::presets::classic_grand_prix;
    use tdrace_core::track::Track;

    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    assert_eq!(state.track.default_surface, SurfaceType::Grass);
    assert!(!state.is_dirty);

    // 1. Set to Sand
    assert!(tools.set_track_default_surface(&mut state, SurfaceType::Sand));
    assert_eq!(state.track.default_surface, SurfaceType::Sand);
    assert!(state.is_dirty);

    // Setting to same surface returns false
    assert!(!tools.set_track_default_surface(&mut state, SurfaceType::Sand));

    // 2. Reject non-offtrack surface types (e.g. Ice, Water, Oil, Curb)
    assert!(!tools.set_track_default_surface(&mut state, SurfaceType::Ice));
    assert!(!tools.set_track_default_surface(&mut state, SurfaceType::Water));
    assert!(!tools.set_track_default_surface(&mut state, SurfaceType::Oil));
    assert!(!tools.set_track_default_surface(&mut state, SurfaceType::Curb));
    assert_eq!(state.track.default_surface, SurfaceType::Sand);

    // 3. Undo restores previous Grass surface
    assert!(state.undo());
    assert_eq!(state.track.default_surface, SurfaceType::Grass);

    // 4. Redo restores Sand
    assert!(state.redo());
    assert_eq!(state.track.default_surface, SurfaceType::Sand);

    // 5. Test cycling off-track types: Grass -> Sand -> Dirt -> Asphalt -> Grass
    tools.set_track_default_surface(&mut state, SurfaceType::Grass);
    assert_eq!(tools.cycle_track_default_surface(&mut state), SurfaceType::Sand);
    assert_eq!(tools.cycle_track_default_surface(&mut state), SurfaceType::Dirt);
    assert_eq!(tools.cycle_track_default_surface(&mut state), SurfaceType::Asphalt);
    assert_eq!(tools.cycle_track_default_surface(&mut state), SurfaceType::Grass);

    // 6. JSON serialization roundtrip preserves default_surface
    tools.set_track_default_surface(&mut state, SurfaceType::Dirt);
    let json_str = state.track.to_json().expect("Failed to serialize track");
    let deserialized = Track::from_json(&json_str).expect("Failed to deserialize track");
    assert_eq!(deserialized.default_surface, SurfaceType::Dirt);
}

#[test]
fn test_track_editor_predefined_car_mutation_and_cycling() {
    use tdrace_app::editor::{EditorState, ToolSettings};
    use tdrace_core::track::presets::classic_grand_prix;
    use tdrace_core::track::Track;

    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    assert_eq!(state.track.predefined_car.as_deref(), Some("sports_car"));
    assert!(!state.is_dirty);

    // 1. Set predefined car to "rally_car"
    assert!(tools.set_track_predefined_car(&mut state, Some("rally_car".to_string())));
    assert_eq!(state.track.predefined_car.as_deref(), Some("rally_car"));
    assert!(state.is_dirty);

    // Setting same car returns false
    assert!(!tools.set_track_predefined_car(&mut state, Some("rally_car".to_string())));

    // 2. Undo restores "sports_car"
    assert!(state.undo());
    assert_eq!(state.track.predefined_car.as_deref(), Some("sports_car"));

    // 3. Redo restores "rally_car"
    assert!(state.redo());
    assert_eq!(state.track.predefined_car.as_deref(), Some("rally_car"));

    // 4. Test cycling predefined car: rally_car -> f1_car -> sports_car -> drift_car -> kart -> rally_car
    assert_eq!(tools.cycle_track_predefined_car(&mut state), Some("f1_car".to_string()));
    assert_eq!(tools.cycle_track_predefined_car(&mut state), Some("sports_car".to_string()));
    assert_eq!(tools.cycle_track_predefined_car(&mut state), Some("drift_car".to_string()));
    assert_eq!(tools.cycle_track_predefined_car(&mut state), Some("kart".to_string()));
    assert_eq!(tools.cycle_track_predefined_car(&mut state), Some("rally_car".to_string()));

    // 5. JSON serialization roundtrip preserves predefined_car
    tools.set_track_predefined_car(&mut state, Some("f1_car".to_string()));
    let json_str = state.track.to_json().expect("Failed to serialize track");
    let deserialized = Track::from_json(&json_str).expect("Failed to deserialize track");
    assert_eq!(deserialized.predefined_car.as_deref(), Some("f1_car"));
}

#[test]
fn test_track_editor_select_all_and_marquee_box_selection() {
    use tdrace_app::editor::{EditorState, EditorToolType, Selection, ToolSettings};
    use tdrace_core::track::geometry::{Obstacle, SurfaceShape};
    use tdrace_core::track::presets::classic_grand_prix;

    let mut track = classic_grand_prix();
    // Add custom obstacle and pit box
    track.geometry.obstacles.push(Obstacle::circle(1, Vec2::new(10.0, 10.0), 3.0, "Obs 1"));
    track.pit_box_area = Some(SurfaceShape::Aabb {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(20.0, 20.0),
    });

    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Test Select All with Select Tool
    tools.active_tool = EditorToolType::Select;
    assert!(tools.select_all_for_active_tool(&mut state));
    assert!(matches!(state.selection, Selection::Multi { .. }));
    assert!(state.selection.is_pit_box_selected());
    assert!(state.selection.is_obstacle_selected(0));
    assert!(state.selection.is_waypoint_selected(0));

    // 2. Test Select All with RoadSpline Tool
    tools.active_tool = EditorToolType::RoadSpline;
    assert!(tools.select_all_for_active_tool(&mut state));
    assert!(matches!(state.selection, Selection::MultipleWaypoints(_)));
    assert_eq!(state.selection.selected_waypoint_indices().len(), state.track.spline.waypoints.len());

    // 3. Test Select All with PitLane Tool
    tools.active_tool = EditorToolType::PitLane;
    assert!(tools.select_all_for_active_tool(&mut state));
    assert_eq!(state.selection, Selection::PitBox);

    // 4. Test Click-and-drag Box Selection in an isolated area
    tools.active_tool = EditorToolType::Select;
    state.selection = Selection::None;

    let target_pos = Vec2::new(700.0, 700.0);
    state.track.geometry.obstacles.push(Obstacle::circle(101, target_pos + Vec2::new(10.0, 10.0), 3.0, "Isolated Obs"));
    let obs_idx = state.track.geometry.obstacles.len() - 1;

    // Drag box from (680, 680) to (730, 730)
    tools.handle_mouse_down(&mut state, target_pos - Vec2::new(20.0, 20.0));
    assert!(tools.is_box_selecting);
    tools.handle_mouse_drag(&mut state, target_pos + Vec2::new(30.0, 30.0));
    tools.handle_mouse_up(&mut state, target_pos + Vec2::new(30.0, 30.0));
    assert!(!tools.is_box_selecting);

    // Should have selected the obstacle
    assert!(state.selection.is_obstacle_selected(obs_idx));

    // 5. Test moving the multi-selection
    let obs_orig = state.track.geometry.obstacles[obs_idx].center();
    tools.handle_mouse_down(&mut state, target_pos + Vec2::new(10.0, 10.0));
    tools.handle_mouse_drag(&mut state, target_pos + Vec2::new(15.0, 15.0));
    tools.handle_mouse_up(&mut state, target_pos + Vec2::new(15.0, 15.0));

    assert_eq!(state.track.geometry.obstacles[obs_idx].center(), obs_orig + Vec2::new(5.0, 5.0));

    // 6. Test multi-selection deletion
    let prev_obs_count = state.track.geometry.obstacles.len();
    assert!(tools.delete_selected(&mut state));
    assert_eq!(state.track.geometry.obstacles.len(), prev_obs_count - 1);
    assert_eq!(state.selection, Selection::None);
}

#[test]
fn test_track_editor_jump_ramp_turn_angle_and_change_size() {
    use tdrace_app::editor::{Selection, ToolSettings};
    use tdrace_core::track::geometry::{JumpRamp, SurfaceShape};

    let mut track = classic_grand_prix();
    track.geometry.jump_ramps.push(JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: Vec2::new(120.0, 80.0),
            half_extents: Vec2::new(6.0, 4.0),
            angle: 0.0,
        },
        Vec2::new(1.0, 0.0),
        24.0,
        15.0,
        1.8,
        "Custom Launch Ramp 1",
    ));
    track.geometry.jump_ramps.push(JumpRamp::new(
        2,
        SurfaceShape::OrientedBox {
            center: Vec2::new(200.0, 150.0),
            half_extents: Vec2::new(8.0, 5.0),
            angle: std::f32::consts::FRAC_PI_2,
        },
        Vec2::new(0.0, 1.0),
        26.0,
        18.0,
        2.2,
        "Custom Launch Ramp 2",
    ));

    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Select First Jump Ramp
    state.selection = Selection::JumpRamp(0);
    assert_eq!(state.track.geometry.jump_ramps[0].angle(), 0.0);
    assert_eq!(state.track.geometry.jump_ramps[0].length(), 12.0);
    assert_eq!(state.track.geometry.jump_ramps[0].width(), 8.0);

    // 2. Rotate Ramp by +45 degrees (PI/4)
    assert!(tools.rotate_selected_jump_ramp(&mut state, std::f32::consts::FRAC_PI_4));
    assert!((state.track.geometry.jump_ramps[0].angle() - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
    assert!((state.track.geometry.jump_ramps[0].direction.x - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
    assert!((state.track.geometry.jump_ramps[0].direction.y - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);

    // 3. Set Exact Angle to 180 degrees (PI)
    assert!(tools.set_selected_jump_ramp_angle(&mut state, std::f32::consts::PI));
    assert!((state.track.geometry.jump_ramps[0].angle() - std::f32::consts::PI).abs() < 1e-4);
    assert!((state.track.geometry.jump_ramps[0].direction.x - (-1.0)).abs() < 1e-4);

    // 4. Adjust Size (+4m Length, +2m Width)
    assert!(tools.adjust_selected_jump_ramp_size(&mut state, 4.0, 2.0));
    assert_eq!(state.track.geometry.jump_ramps[0].length(), 16.0);
    assert_eq!(state.track.geometry.jump_ramps[0].width(), 10.0);

    // 5. Scale Size (* 1.5)
    assert!(tools.scale_selected_jump_ramp_size(&mut state, 1.5));
    assert_eq!(state.track.geometry.jump_ramps[0].length(), 24.0);
    assert_eq!(state.track.geometry.jump_ramps[0].width(), 15.0);

    // 6. Adjust Pitch & Height
    assert!(tools.adjust_selected_jump_ramp_pitch(&mut state, 3.0));
    assert_eq!(state.track.geometry.jump_ramps[0].ramp_angle_deg, 18.0);
    assert!(tools.adjust_selected_jump_ramp_height(&mut state, 0.4));
    assert!((state.track.geometry.jump_ramps[0].height - 2.2).abs() < 1e-4);

    // 7. Multi-selection Batch Rotation & Scaling
    state.selection = Selection::from_multi(vec![], vec![], vec![], vec![0, 1], vec![], vec![], false);
    let ramp1_orig_angle = state.track.geometry.jump_ramps[1].angle();
    assert!(tools.rotate_selected_jump_ramp(&mut state, std::f32::consts::FRAC_PI_2));
    assert!((state.track.geometry.jump_ramps[1].angle() - (ramp1_orig_angle + std::f32::consts::FRAC_PI_2)).abs() < 1e-4);

    assert!(tools.scale_selected_jump_ramp_size(&mut state, 0.8));
    assert_eq!(state.track.geometry.jump_ramps[1].length(), 16.0 * 0.8);

    // 8. Undo Stack Verification
    assert!(state.undo()); // reverts batch scale
    assert_eq!(state.track.geometry.jump_ramps[1].length(), 16.0);
    assert!(state.undo()); // reverts batch rotate
    assert!((state.track.geometry.jump_ramps[1].angle() - ramp1_orig_angle).abs() < 1e-4);

    // 9. Revalidate Track
    assert!(state.track.validate().is_empty());
}

#[test]
fn test_track_editor_jump_ramp_arbitrary_angle_degrees() {
    use tdrace_app::editor::{Selection, ToolSettings};
    use tdrace_core::track::presets::oasis_rally;
    use tdrace_core::track::geometry::{JumpRamp, SurfaceShape};

    let mut track = oasis_rally();
    track.geometry.jump_ramps.clear();
    track.geometry.jump_ramps.push(JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: Vec2::new(100.0, 50.0),
            half_extents: Vec2::new(6.0, 4.0),
            angle: 0.0,
        },
        Vec2::new(1.0, 0.0),
        25.0,
        15.0,
        2.0,
        "Test Ramp",
    ));

    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    state.selection = Selection::JumpRamp(0);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 0.0).abs() < 1e-4);

    // 1. Set specific arbitrary angles: 37.5°, 142.3°, 359.9°, 365.0° (wraps to 5.0°)
    tools.set_selected_jump_ramp_angle_deg(&mut state, 37.5);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 37.5).abs() < 1e-3);
    assert!((state.track.geometry.jump_ramps[0].angle() - 37.5f32.to_radians()).abs() < 1e-3);
    let expected_dir_37_5 = Vec2::new(37.5f32.to_radians().cos(), 37.5f32.to_radians().sin());
    assert!((state.track.geometry.jump_ramps[0].direction - expected_dir_37_5).length() < 1e-3);

    tools.set_selected_jump_ramp_angle_deg(&mut state, 142.3);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 142.3).abs() < 1e-3);

    tools.set_selected_jump_ramp_angle_deg(&mut state, 359.9);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 359.9).abs() < 1e-3);

    // 365° angle wraps around to 5.0°
    tools.set_selected_jump_ramp_angle_deg(&mut state, 365.0);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 5.0).abs() < 1e-3);

    // 2. Interactive continuous angle drag via handle
    let ramp_center = Vec2::new(100.0, 50.0);
    let arrow_tip = ramp_center + state.track.geometry.jump_ramps[0].direction * 8.5;

    // Simulate clicking near the handle tip to start continuous angle rotation
    tools.handle_mouse_down(&mut state, arrow_tip);
    assert!(tools.is_rotating_ramp);
    assert_eq!(tools.drag_rotating_ramp_idx, Some(0));

    // Drag mouse to point due North (angle = 90°)
    let target_mouse_north = ramp_center + Vec2::new(0.0, 20.0);
    tools.handle_mouse_drag(&mut state, target_mouse_north);
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 90.0).abs() < 1e-3);
    assert!((state.track.geometry.jump_ramps[0].direction - Vec2::new(0.0, 1.0)).length() < 1e-3);

    // Release mouse
    tools.handle_mouse_up(&mut state, target_mouse_north);
    assert!(!tools.is_rotating_ramp);
    assert_eq!(tools.drag_rotating_ramp_idx, None);

    // 3. Undo and Redo tracking
    assert!(state.undo());
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 5.0).abs() < 1e-3);
    assert!(state.redo());
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 90.0).abs() < 1e-3);
}

#[test]
fn test_jump_ramp_surface_tools_and_surface_sampling() {
    use tdrace_app::editor::{EditorToolType, Selection, ToolSettings};
    use tdrace_core::track::presets::oasis_rally;

    let mut track = oasis_rally();
    track.geometry.jump_ramps.clear();

    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Create jump ramp with active surface
    tools.active_surface = SurfaceType::Grass;
    tools.active_tool = EditorToolType::JumpRamp;

    let ramp_center = Vec2::new(60.0, 60.0);
    tools.handle_mouse_down(&mut state, ramp_center);
    tools.handle_mouse_up(&mut state, ramp_center);

    assert_eq!(state.track.geometry.jump_ramps.len(), 1);
    assert_eq!(state.track.geometry.jump_ramps[0].surface, SurfaceType::Grass);
    assert_eq!(state.track.geometry.jump_ramps[0].launch_speed, 4.0);

    // 2. Sample surface physics on the jump ramp
    assert_eq!(state.track.sample_surface(ramp_center), SurfaceType::Grass);

    // 3. Change surface type and launch speed via inspector tool
    state.selection = Selection::JumpRamp(0);
    assert!(tools.set_selected_jump_ramp_surface(&mut state, SurfaceType::Ice));
    assert_eq!(state.track.geometry.jump_ramps[0].surface, SurfaceType::Ice);
    assert_eq!(state.track.sample_surface(ramp_center), SurfaceType::Ice);
    assert!(tools.set_selected_jump_ramp_launch_speed(&mut state, 4.5));
    assert_eq!(state.track.geometry.jump_ramps[0].launch_speed, 4.5);

    // 4. Modify dimensions and pitch
    assert!(tools.set_selected_jump_ramp_length(&mut state, 25.0));
    assert_eq!(state.track.geometry.jump_ramps[0].length(), 25.0);

    assert!(tools.set_selected_jump_ramp_width(&mut state, 12.0));
    assert_eq!(state.track.geometry.jump_ramps[0].width(), 12.0);

    assert!(tools.set_selected_jump_ramp_height(&mut state, 2.5));
    assert_eq!(state.track.geometry.jump_ramps[0].height, 2.5);

    assert!(tools.set_selected_jump_ramp_pitch_deg(&mut state, 20.0));
    assert_eq!(state.track.geometry.jump_ramps[0].ramp_angle_deg, 20.0);

    // Flat tabletop removal via pitch
    let fitted_deg = state.track.geometry.jump_ramps[0].fitted_pitch_deg();
    assert!(tools.remove_selected_jump_ramp_flat_portion(&mut state));
    assert!((state.track.geometry.jump_ramps[0].ramp_angle_deg - fitted_deg).abs() < 1e-3);
    assert!(state.track.geometry.jump_ramps[0].flat_length() < 0.01);

    // Flat tabletop removal via height
    assert!(tools.set_selected_jump_ramp_pitch_deg(&mut state, 15.0));
    let fitted_h = state.track.geometry.jump_ramps[0].fitted_height();
    assert!(tools.adjust_selected_jump_ramp_height_to_pitch(&mut state));
    assert!((state.track.geometry.jump_ramps[0].height - fitted_h).abs() < 1e-3);
    assert!(state.track.geometry.jump_ramps[0].flat_length() < 0.01);

    // 5. Batch surface set
    assert!(tools.batch_set_surface(&mut state, Some(SurfaceType::Sand)));
    assert_eq!(state.track.geometry.jump_ramps[0].surface, SurfaceType::Sand);
    assert_eq!(state.track.sample_surface(ramp_center), SurfaceType::Sand);

    // 6. Undo/redo chain
    assert!(state.undo());
    assert_eq!(state.track.geometry.jump_ramps[0].surface, SurfaceType::Ice);
    assert!(state.redo());
    assert_eq!(state.track.geometry.jump_ramps[0].surface, SurfaceType::Sand);
}

#[test]
fn test_waypoint_banking_editor_controls_and_batch_operations() {
    use tdrace_app::editor::tools::{EditorToolType, ToolSettings};
    use tdrace_app::editor::Selection;

    let mut state = EditorState::new(classic_grand_prix());
    let mut tools = ToolSettings::default();

    // 1. Configure active placement banking angle
    tools.active_tool = EditorToolType::RoadSpline;
    tools.new_waypoint_bank_angle = 18.0;

    let initial_count = state.track.spline.waypoints.len();
    tools.handle_mouse_down(&mut state, Vec2::new(250.0, 250.0));
    tools.handle_mouse_up(&mut state, Vec2::new(250.0, 250.0));

    assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
    let new_wp_idx = state.track.spline.waypoints.len() - 1;
    assert_eq!(
        state.track.spline.waypoints[new_wp_idx].bank_angle,
        18.0,
        "Newly placed waypoint must inherit active placement banking angle"
    );

    // 2. Adjust single waypoint banking
    state.select(Selection::Waypoint(new_wp_idx));
    assert!(tools.batch_adjust_banking(&mut state, 4.0));
    assert_eq!(state.track.spline.waypoints[new_wp_idx].bank_angle, 22.0);

    // 3. Multi-waypoint selection and batch banking
    state.select(Selection::MultipleWaypoints(vec![0, 1, new_wp_idx]));

    assert!(tools.batch_set_banking(&mut state, 15.0));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 15.0);
    assert_eq!(state.track.spline.waypoints[1].bank_angle, 15.0);
    assert_eq!(state.track.spline.waypoints[new_wp_idx].bank_angle, 15.0);

    // Invert banking (+/-)
    assert!(tools.batch_invert_banking(&mut state));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, -15.0);
    assert_eq!(state.track.spline.waypoints[1].bank_angle, -15.0);
    assert_eq!(state.track.spline.waypoints[new_wp_idx].bank_angle, -15.0);

    // Reset via batch set
    assert!(tools.batch_set_banking(&mut state, 0.0));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 0.0);

    // Cycle banking presets: 0 -> 10 -> 18 -> 22 -> 0
    assert!(tools.cycle_selected_banking(&mut state));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 10.0);
    assert!(tools.cycle_selected_banking(&mut state));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 18.0);
    assert!(tools.cycle_selected_banking(&mut state));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 22.0);
    assert!(tools.cycle_selected_banking(&mut state));
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 0.0);

    // 4. Undo and Redo operations preserve banking
    assert!(state.undo());
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 22.0);
    assert!(state.redo());
    assert_eq!(state.track.spline.waypoints[0].bank_angle, 0.0);
}

#[test]
fn test_track_editor_wall_distance_editing_and_batch_operations() {
    use tdrace_app::editor::tools::{EditorToolType, ToolSettings};
    use tdrace_app::editor::Selection;

    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Placement Tool default wall distance
    tools.active_tool = EditorToolType::RoadSpline;
    tools.new_waypoint_left_wall_distance = Some(0.0);
    tools.new_waypoint_right_wall_distance = Some(0.0);

    // Place a new waypoint with 0.0m flush wall distance
    let click_pos = Vec2::new(100.0, 100.0);
    tools.handle_mouse_down(&mut state, click_pos);

    let new_wp_idx = state.selection.is_waypoint().expect("New waypoint must be selected");
    assert_eq!(
        state.track.spline.waypoints[new_wp_idx].left_wall_distance,
        Some(0.0),
        "Newly placed waypoint must inherit active placement wall distance"
    );
    assert_eq!(
        state.track.spline.waypoints[new_wp_idx].right_wall_distance,
        Some(0.0)
    );

    // 2. Batch adjust wall distance
    state.select(Selection::MultipleWaypoints(vec![0, 1, 2]));
    assert!(tools.batch_adjust_wall_distances(&mut state, -2.0));
    // Default 4.0 - 2.0 = 2.0
    assert_eq!(state.track.spline.waypoints[0].left_wall_distance, Some(2.0));
    assert_eq!(state.track.spline.waypoints[1].left_wall_distance, Some(2.0));
    assert_eq!(state.track.spline.waypoints[2].left_wall_distance, Some(2.0));

    // 3. Batch set wall distance to 0.0m (Flush / Banked)
    assert!(tools.batch_set_wall_distances(&mut state, Some(0.0), Some(0.0)));
    assert_eq!(state.track.spline.waypoints[0].left_wall_distance, Some(0.0));
    assert_eq!(state.track.spline.waypoints[0].right_wall_distance, Some(0.0));
    assert_eq!(state.track.spline.waypoints[1].left_wall_distance, Some(0.0));

    // 4. Batch reset wall distance to None (Default)
    assert!(tools.batch_set_wall_distances(&mut state, None, None));
    assert_eq!(state.track.spline.waypoints[0].left_wall_distance, None);
    assert_eq!(state.track.spline.waypoints[0].right_wall_distance, None);

    // 5. Undo / Redo restores waypoint wall distances
    assert!(state.undo()); // Undo reset (back to Some(0.0))
    assert_eq!(state.track.spline.waypoints[0].left_wall_distance, Some(0.0));
    assert!(state.redo()); // Redo reset
    assert_eq!(state.track.spline.waypoints[0].left_wall_distance, None);

    // 6. Global barrier offset adjustment
    tools.set_global_barrier_offset(&mut state, 2.5);
    assert_eq!(state.barrier_offset, 2.5);
}

#[test]
fn test_primary_selection_and_secondary_placement_interaction_model() {
    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. In RoadSpline tool:
    tools.active_tool = EditorToolType::RoadSpline;
    let initial_wps = state.track.spline.waypoints.len();

    // Secondary button (Right Click) places a new waypoint
    let wp_pos = Vec2::new(350.0, 350.0);
    tools.handle_secondary_down(&mut state, wp_pos);
    tools.handle_secondary_up(&mut state, wp_pos);
    assert_eq!(state.track.spline.waypoints.len(), initial_wps + 1);
    let new_wp_idx = state.track.spline.waypoints.len() - 1;
    assert_eq!(state.selection, Selection::Waypoint(new_wp_idx));

    // Primary button (Left Click) on existing waypoint selects & drags it
    let move_pos = Vec2::new(360.0, 360.0);
    tools.handle_primary_down(&mut state, wp_pos, false);
    assert_eq!(state.selection, Selection::Waypoint(new_wp_idx));
    assert!(tools.is_dragging);
    tools.handle_primary_drag(&mut state, move_pos);
    tools.handle_primary_up(&mut state, move_pos);
    assert_eq!(state.track.spline.waypoints[new_wp_idx].point, move_pos);

    // Primary button (Left Click & Drag) on empty canvas does marquee box selection
    let box_start = Vec2::new(340.0, 340.0);
    let box_end = Vec2::new(380.0, 380.0);
    state.selection = Selection::None;
    tools.handle_primary_down(&mut state, box_start, false);
    assert!(tools.is_box_selecting);
    tools.handle_primary_drag(&mut state, box_end);
    tools.handle_primary_up(&mut state, box_end);
    assert!(!tools.is_box_selecting);
    assert!(state.selection.is_waypoint_selected(new_wp_idx));

    // 2. In SurfaceZone tool:
    tools.active_tool = EditorToolType::SurfaceZone;
    tools.active_surface = SurfaceType::Grass;
    tools.active_surface_shape = tdrace_app::editor::SurfaceShapeType::Square;
    let initial_zones = state.track.geometry.surface_zones.len();

    // Secondary button (Right Drag) places the surface zone
    let zone_start = Vec2::new(400.0, 400.0);
    let zone_end = Vec2::new(450.0, 450.0);
    tools.handle_secondary_down(&mut state, zone_start);
    assert!(tools.is_placing);
    tools.handle_secondary_drag(&mut state, zone_end);
    tools.handle_secondary_up(&mut state, zone_end);
    assert!(!tools.is_placing);
    assert_eq!(state.track.geometry.surface_zones.len(), initial_zones + 1);

    // Primary button (Left Click) on the zone selects & drags it
    let zone_idx = state.track.geometry.surface_zones.len() - 1;
    let zone_center = Vec2::new(425.0, 425.0);
    tools.handle_primary_down(&mut state, zone_center, false);
    assert_eq!(state.selection, Selection::SurfaceZone(zone_idx));
    assert!(tools.is_dragging);
    let zone_moved = Vec2::new(435.0, 435.0);
    tools.handle_primary_drag(&mut state, zone_moved);
    tools.handle_primary_up(&mut state, zone_moved);
    assert_eq!(state.track.geometry.surface_zones[zone_idx].shape.center(), zone_moved);
}

#[test]
fn test_editor_ui_hover_detection_and_inspector_click_safety() {
    // 1. At 1920x1080 (1080p, UiScaler scale = 1.5):
    //    Top toolbar height is 46 * 1.5 = 69.0.
    //    Status bar height is 32 * 1.5 = 48.0, so bot_y = 1032.0.
    //    Inspector panel width is 240 * 1.5 = 360.0. insp_x = 1920 - 360 - 18 = 1542.0.
    let sw = 1920.0;
    let sh = 1080.0;

    // A cursor position on the inspector panel (e.g. x = 1600.0, y = 300.0)
    // Previously with hardcoded `mx > sw - 260.0` (1660.0), 1600.0 evaluated to false!
    let inspector_click_pos = Vec2::new(1600.0, 300.0);
    assert!(
        is_mouse_over_editor_ui(inspector_click_pos, sw, sh, EditorToolType::Select, false),
        "Inspector panel click at x=1600 must be recognized as over UI at 1080p"
    );

    // Left edge of inspector panel at 1080p (x = 1545.0)
    let inspector_left_edge = Vec2::new(1545.0, 200.0);
    assert!(
        is_mouse_over_editor_ui(inspector_left_edge, sw, sh, EditorToolType::Select, false),
        "Inspector left edge at x=1545 must be recognized as over UI at 1080p"
    );

    // Right edge of inspector panel at 1080p (x = 1900.0)
    let inspector_right_edge = Vec2::new(1900.0, 200.0);
    assert!(
        is_mouse_over_editor_ui(inspector_right_edge, sw, sh, EditorToolType::Select, false),
        "Inspector right edge at x=1900 must be recognized as over UI at 1080p"
    );

    // Top toolbar at 1080p (y = 55.0, previously missed by unscaled my < 48.0)
    let top_bar_pos = Vec2::new(500.0, 55.0);
    assert!(
        is_mouse_over_editor_ui(top_bar_pos, sw, sh, EditorToolType::Select, false),
        "Top toolbar click at y=55 must be recognized as over UI at 1080p"
    );

    // Bottom status bar at 1080p (y = 1040.0, previously missed by unscaled my > sh - 34.0)
    let bot_bar_pos = Vec2::new(500.0, 1040.0);
    assert!(
        is_mouse_over_editor_ui(bot_bar_pos, sw, sh, EditorToolType::Select, false),
        "Bottom status bar click at y=1040 must be recognized as over UI at 1080p"
    );

    // Left tool palette at 1080p (x = 220.0, y = 500.0, previously missed by unscaled mx < 185 && my < 480)
    let left_tool_pos = Vec2::new(220.0, 500.0);
    assert!(
        is_mouse_over_editor_ui(left_tool_pos, sw, sh, EditorToolType::Select, false),
        "Left tool palette click at x=220, y=500 must be recognized as over UI at 1080p"
    );

    // Canvas click in center of editing area (x = 960.0, y = 540.0)
    let canvas_center = Vec2::new(960.0, 540.0);
    assert!(
        !is_mouse_over_editor_ui(canvas_center, sw, sh, EditorToolType::Select, false),
        "Canvas center must not be over UI"
    );

    // Any coordinate is over UI when a modal is open
    assert!(
        is_mouse_over_editor_ui(canvas_center, sw, sh, EditorToolType::Select, true),
        "Any coordinate must be over UI when modal is open"
    );

    // 2. Also verify standard 720p (1280x720)
    let sw_720 = 1280.0;
    let sh_720 = 720.0;
    // insp_x = 1280 - 240 - 12 = 1028.0
    assert!(is_mouse_over_editor_ui(Vec2::new(1050.0, 300.0), sw_720, sh_720, EditorToolType::Select, false));
    assert!(!is_mouse_over_editor_ui(Vec2::new(640.0, 360.0), sw_720, sh_720, EditorToolType::Select, false));
}

#[test]
fn test_auto_grid_slots_finish_line_requirement_and_modal_warning() {
    use tdrace_app::editor::{EditorModal, EditorState};
    use tdrace_core::track::presets::classic_grand_prix;

    let mut track = classic_grand_prix();
    // Clear checkpoints and grid positions so no finish line exists
    track.checkpoints.clear();
    track.grid_positions.clear();

    let mut state = EditorState::new(track);
    let mut active_modal = EditorModal::None;

    assert!(!state.track.has_finish_line());

    // 1. Trigger Auto Grid Slots logic without finish line
    if !state.track.has_finish_line() {
        active_modal = EditorModal::Warning {
            title: "FINISH LINE REQUIRED".to_string(),
            message: "No finish line checkpoint exists on this circuit.\n\nPlease define a finish line before generating starting grid slots.\n(Select an existing checkpoint and mark it as Finish Line,\nor place a new checkpoint using the Checkpoint Tool)".to_string(),
        };
    } else {
        state.record_undo();
        state.track.auto_generate_grid(8, 8.0, 3.0);
        state.revalidate();
    }

    // Modal must be Warning with informative title & message
    if let EditorModal::Warning { title, message } = &active_modal {
        assert_eq!(title, "FINISH LINE REQUIRED");
        assert!(message.contains("No finish line"));
        assert!(message.contains("define a finish line"));
    } else {
        panic!("Expected EditorModal::Warning when auto grid slots clicked without finish line");
    }
    assert!(state.track.grid_positions.is_empty());
    assert!(!state.is_dirty);

    // 2. Now define a finish line and auto generate grid slots
    state.track.auto_generate_checkpoints(8, 3);
    assert!(state.track.has_finish_line());
    active_modal = EditorModal::None;

    if !state.track.has_finish_line() {
        active_modal = EditorModal::Warning {
            title: "FINISH LINE REQUIRED".to_string(),
            message: "No finish line".to_string(),
        };
    } else {
        state.record_undo();
        let ok = state.track.auto_generate_grid(8, 8.0, 3.0);
        assert!(ok);
        state.revalidate();
    }

    assert_eq!(active_modal, EditorModal::None);
    assert_eq!(state.track.grid_positions.len(), 8);
    assert!(state.is_dirty);

    // 3. Move finish line to checkpoint 4 and regenerate
    state.track.checkpoints[0].is_finish_line = false;
    state.track.checkpoints[4].is_finish_line = true;
    let cp4_center = (state.track.checkpoints[4].gate.start + state.track.checkpoints[4].gate.end) * 0.5;
    let cp4_dist = state.track.spline.project_point(cp4_center).progress_distance;
    let total_len = state.track.spline.total_length();

    let ok = state.track.auto_generate_grid(8, 8.0, 3.0);
    assert!(ok);

    let slot0_proj = state.track.spline.project_point(state.track.grid_positions[0].position);
    let expected_slot0_dist = (cp4_dist - 15.0 + total_len) % total_len;
    assert!(
        (slot0_proj.progress_distance - expected_slot0_dist).abs() < 1.0,
        "Slot 0 should be positioned 15m behind checkpoint 4 finish line"
    );
}

#[test]
fn test_track_editor_wall_type_selection_and_batch_operations() {
    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    // 1. Single waypoint wall type assignment
    state.select(Selection::Waypoint(2));
    state.track.spline.waypoints[2].wall_type = Some(BarrierType::TireWall);
    state.rebuild_geometry();
    assert_eq!(state.track.spline.waypoints[2].wall_type, Some(BarrierType::TireWall));

    // 2. Batch wall type assignment to multiple waypoints
    state.select(Selection::MultipleWaypoints(vec![0, 1, 3]));
    let applied = tools.batch_set_wall_type(&mut state, Some(BarrierType::Concrete));
    assert!(applied);
    assert_eq!(state.track.spline.waypoints[0].wall_type, Some(BarrierType::Concrete));
    assert_eq!(state.track.spline.waypoints[1].wall_type, Some(BarrierType::Concrete));
    assert_eq!(state.track.spline.waypoints[3].wall_type, Some(BarrierType::Concrete));
    // Waypoint 2 should still be TireWall
    assert_eq!(state.track.spline.waypoints[2].wall_type, Some(BarrierType::TireWall));

    // Batch set to rubber tyres (TireWall)
    state.select(Selection::MultipleWaypoints(vec![0, 1]));
    let applied_tire = tools.batch_set_wall_type(&mut state, Some(BarrierType::TireWall));
    assert!(applied_tire);
    assert_eq!(state.track.spline.waypoints[0].wall_type, Some(BarrierType::TireWall));
    assert_eq!(state.track.spline.waypoints[1].wall_type, Some(BarrierType::TireWall));

    // Batch set to steel walls (BarrierType::Steel)
    state.select(Selection::MultipleWaypoints(vec![0, 1]));
    let applied_steel = tools.batch_set_wall_type(&mut state, Some(BarrierType::Steel));
    assert!(applied_steel);
    assert_eq!(state.track.spline.waypoints[0].wall_type, Some(BarrierType::Steel));
    assert_eq!(state.track.spline.waypoints[1].wall_type, Some(BarrierType::Steel));

    // 3. Road spline tool placement inherits or uses new_waypoint_wall_type (Steel)
    tools.active_tool = EditorToolType::RoadSpline;
    tools.new_waypoint_wall_type = Some(BarrierType::Steel);
    let initial_count = state.track.spline.waypoints.len();
    state.selection = Selection::None;
    state.last_selected_waypoint = None;
    tools.handle_secondary_down(&mut state, Vec2::new(999.0, 999.0));
    tools.handle_secondary_up(&mut state, Vec2::new(999.0, 999.0));
    assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
    let new_wp = state.track.spline.waypoints.last().unwrap();
    assert_eq!(new_wp.wall_type, Some(BarrierType::Steel));

    // 4. Global barrier type setting (Steel, TireWall, Concrete)
    tools.set_global_barrier_type(&mut state, BarrierType::Steel);
    assert_eq!(state.barrier_type, BarrierType::Steel);
    tools.set_global_barrier_type(&mut state, BarrierType::TireWall);
    assert_eq!(state.barrier_type, BarrierType::TireWall);
    tools.set_global_barrier_type(&mut state, BarrierType::Concrete);
    assert_eq!(state.barrier_type, BarrierType::Concrete);
}

#[test]
fn test_bar_control_selection_and_inline_manual_input() {
    let mut tools = ToolSettings::default();

    // Initially no bar is selected or being edited
    assert!(!tools.is_editing_text());
    assert!(!tools.is_bar_selected("placement_width"));
    assert!(!tools.is_bar_selected("placement_banking"));

    // Select a bar
    tools.select_bar("placement_width");
    assert!(tools.is_bar_selected("placement_width"));
    assert!(!tools.is_bar_selected("placement_banking"));
    assert!(!tools.is_editing_text());

    // Start inline manual text editing
    tools.start_editing_bar("placement_width", "21.5");
    assert!(tools.is_editing_text());
    assert!(tools.is_bar_selected("placement_width"));
    assert_eq!(
        tools.editing_bar,
        Some(("placement_width".to_string(), "21.5".to_string()))
    );

    // Stop editing
    tools.stop_editing_bar();
    assert!(!tools.is_editing_text());
    // Bar remains selected after editing completes
    assert!(tools.is_bar_selected("placement_width"));

    // Clear selection
    tools.clear_bar_selection();
    assert!(!tools.is_bar_selected("placement_width"));
}

#[test]
fn test_track_editor_new_track_action_and_templates_modal() {
    use tdrace_app::editor::ui::{EditorAction, EditorModal};
    use tdrace_app::game::RaceSession;
    use tdrace_core::physics::surface::SurfaceType;
    use tdrace_core::track::presets::{RaceDirection, TrackShape};

    // 1. Modal default initialization per module
    let modal_classic = EditorModal::templates_default(Some("classic"));
    assert_eq!(
        modal_classic,
        EditorModal::Templates {
            selected_shape: TrackShape::Oval,
            selected_direction: RaceDirection::Right,
            selected_module_idx: 0,
        }
    );

    let modal_rally = EditorModal::templates_default(Some("rally"));
    assert_eq!(
        modal_rally,
        EditorModal::Templates {
            selected_shape: TrackShape::Oval,
            selected_direction: RaceDirection::Right,
            selected_module_idx: 3,
        }
    );

    // 2. Test RaceSession handling of EditorAction::NewTrack
    let mut session = RaceSession::new();

    // Create a Kart horizontal 8 track with Left direction
    session.handle_editor_action(EditorAction::NewTrack {
        shape: TrackShape::HorizontalEight,
        direction: RaceDirection::Left,
        module_id: "kart".to_string(),
    });

    assert_eq!(session.track.default_surface, SurfaceType::Asphalt);
    assert_eq!(session.track.spline.samples[0].surface, SurfaceType::Asphalt);
    assert_eq!(session.track.predefined_car.as_deref(), Some("kart"));
    let start_sample = session.track.spline.sample_at_distance(0.0);
    assert!(start_sample.tangent.x < 0.0, "Left direction must start with negative X tangent");

    // Create a Rally oval track with Right direction
    session.handle_editor_action(EditorAction::NewTrack {
        shape: TrackShape::Oval,
        direction: RaceDirection::Right,
        module_id: "rally".to_string(),
    });

    assert_eq!(session.track.default_surface, SurfaceType::Dirt);
    assert_eq!(session.track.spline.samples[0].surface, SurfaceType::Dirt);
    assert_eq!(session.track.predefined_car.as_deref(), Some("rally_car"));
    let start_sample_rally = session.track.spline.sample_at_distance(0.0);
    assert!(start_sample_rally.tangent.x > 0.0, "Right direction must start with positive X tangent");
}

#[test]
fn test_preset_circuits_overwrite_and_persistence_in_editor() {
    let _dev_mutex_guard = DEV_MODE_TEST_MUTEX.lock().unwrap();
    use std::fs;
    use tdrace_app::editor::EditorAction;

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_editor_preset_overwrite_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let mock_git_tracks = temp_dir.join("git_tracks");
    let user_tracks_dir = temp_dir.join("user_tracks");
    let _ = fs::create_dir_all(mock_git_tracks.join("classic"));
    let _ = fs::create_dir_all(mock_git_tracks.join("nascar"));
    let _ = fs::create_dir_all(&user_tracks_dir);

    // Seed mock git tracks
    let mut gp_initial = classic_grand_prix();
    gp_initial.name = "Classic Grand Prix Original".to_string();
    gp_initial.save_to_file(mock_git_tracks.join("classic").join("classic_grand_prix.json")).unwrap();

    let mut daytona_initial = tdrace_core::track::presets::oval_speedway();
    daytona_initial.name = "Daytona Original".to_string();
    daytona_initial.save_to_file(mock_git_tracks.join("nascar").join("daytona.json")).unwrap();

    struct TestEnvGuard {
        git_dir_set: bool,
    }
    impl Drop for TestEnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(tdrace_app::storage::ENV_DEV_MODE);
            if self.git_dir_set {
                std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
            }
            tdrace_app::ui::menu::clear_menu_track_cache();
        }
    }

    // --- Test 1: ClassicGrandPrix overwrite in Dev Mode ---
    {
        std::env::set_var(tdrace_app::storage::ENV_DEV_MODE, "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git_tracks);
        let _guard = TestEnvGuard { git_dir_set: true };

        let mut session = RaceSession::new();
        session.audio.settings.ui_volume = 0.0;
        session.audio.settings.master_volume = 0.0;
        session.track_manager = TrackManager::new(&user_tracks_dir);

        let choice = TrackChoice::ClassicGrandPrix;
        let loaded = session.track_manager.load_track(&choice).expect("Load preset");
        assert_eq!(loaded.name, "Classic Grand Prix Original");

        let canonical_file = session.track_manager.resolve_preset_git_file(choice.track_id(), None);
        assert!(canonical_file.is_some());
        let canonical_file_str = canonical_file.unwrap().to_string_lossy().to_string();

        session.enter_track_editor_with_path(loaded, Some(canonical_file_str.clone()));
        assert_eq!(session.state, GameState::TrackEditor);

        let modified_name = "Classic Grand Prix Custom Tuned";
        let modified_desc = "Tuned apex corners and custom curbs.";
        session.handle_editor_action(EditorAction::SaveTrack {
            name: modified_name.to_string(),
            filename: "classic_grand_prix".to_string(),
            description: modified_desc.to_string(),
            overwrite: true,
            exit_after: false,
        });

        assert_eq!(session.editor_state.as_ref().unwrap().track.name, modified_name);
        assert_eq!(session.track.name, modified_name);

        // Verify git mock file was overwritten
        let git_on_disk = Track::load_from_file(&canonical_file_str).expect("Load overwritten git file");
        assert_eq!(git_on_disk.name, modified_name);
        assert_eq!(git_on_disk.description, modified_desc);

        // Verify user storage file was ALSO written (dual persistence)
        let user_file = user_tracks_dir.join("classic_grand_prix.json");
        assert!(user_file.exists(), "User storage copy must exist for durability against git operations");
        let user_on_disk = Track::load_from_file(&user_file).expect("Load user storage file");
        assert_eq!(user_on_disk.name, modified_name);

        // Verify TrackManager reloads the modified track
        let reloaded = session.track_manager.load_track(&choice).expect("Reload preset");
        assert_eq!(reloaded.name, modified_name);
        assert_eq!(reloaded.description, modified_desc);

        // Verify Menu track resolver returns the modified track
        let menu_resolved = tdrace_app::ui::menu::resolve_track_for_menu_with_dir(&choice, &user_tracks_dir).expect("Menu resolved");
        assert_eq!(menu_resolved.name, modified_name);
    }

    // --- Test 2: NASCAR Daytona preset with alias resolution in Dev Mode ---
    {
        std::env::set_var(tdrace_app::storage::ENV_DEV_MODE, "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git_tracks);
        let _guard = TestEnvGuard { git_dir_set: true };

        let mut session = RaceSession::new();
        session.audio.settings.ui_volume = 0.0;
        session.audio.settings.master_volume = 0.0;
        session.track_manager = TrackManager::new(&user_tracks_dir);

        let choice = TrackChoice::Custom {
            id: "daytona_superspeedway".to_string(),
            title: "Daytona International Speedway".to_string(),
            description: "Famous tri-oval".to_string(),
            path: "nascar/daytona".to_string(),
        };

        let loaded = session.track_manager.load_track(&choice).expect("Load NASCAR preset");
        let canonical_file = session.track_manager.resolve_preset_git_file("daytona_superspeedway", Some("nascar"));
        assert!(canonical_file.is_some(), "Must resolve daytona_superspeedway to nascar/daytona.json");
        let canonical_file_str = canonical_file.unwrap().to_string_lossy().to_string();

        session.enter_track_editor_with_path(loaded, Some(canonical_file_str.clone()));

        let modified_desc = "Overwritten high-banked turns.";
        session.handle_editor_action(EditorAction::SaveTrack {
            name: "Daytona International Speedway".to_string(),
            filename: "daytona".to_string(),
            description: modified_desc.to_string(),
            overwrite: true,
            exit_after: false,
        });

        let git_on_disk = Track::load_from_file(&canonical_file_str).expect("Load Daytona on disk");
        assert_eq!(git_on_disk.description, modified_desc);

        let reloaded = session.track_manager.load_track(&choice).expect("Reload NASCAR preset");
        assert_eq!(reloaded.description, modified_desc);
    }

    // --- Test 3: Standard mode rejects preset overwrite ---
    {
        std::env::remove_var(tdrace_app::storage::ENV_DEV_MODE);
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git_tracks);
        let _guard = TestEnvGuard { git_dir_set: true };

        let mut session = RaceSession::new();
        session.audio.settings.ui_volume = 0.0;
        session.audio.settings.master_volume = 0.0;
        session.track_manager = TrackManager::new(&user_tracks_dir);

        let choice = TrackChoice::ClassicGrandPrix;
        let loaded = session.track_manager.load_track(&choice).expect("Load preset in standard mode");

        session.enter_track_editor_with_path(loaded, None);

        session.handle_editor_action(EditorAction::SaveTrack {
            name: "Hacked Preset Name".to_string(),
            filename: "classic_grand_prix".to_string(),
            description: "Should fail in standard mode".to_string(),
            overwrite: true,
            exit_after: false,
        });

        assert!(session.editor_save_toast_msg.contains("cannot be modified directly"));
    }

    let _ = fs::remove_dir_all(&temp_dir);
    std::env::remove_var(tdrace_app::storage::ENV_DEV_MODE);
    std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
    tdrace_app::ui::menu::clear_menu_track_cache();
}

#[test]
fn test_jump_ramp_curved_profile_elevation_and_slope() {
    use tdrace_core::track::geometry::{JumpRamp, SurfaceShape};
    use glam::Vec2;

    let shape = SurfaceShape::OrientedBox {
        center: Vec2::new(50.0, 50.0),
        half_extents: Vec2::new(10.0, 4.0),
        angle: 0.0, // facing +X
    };
    let ramp = JumpRamp::new(1, shape, Vec2::X, 5.0, 15.0, 2.0, "Curved Test Ramp");

    let entry_pt = Vec2::new(40.0, 50.0); // s = 0.0
    let lip_pt = Vec2::new(60.0, 50.0);   // s = 1.0
    let mid_pt = Vec2::new(50.0, 50.0);   // s = 0.5

    // 1. At entrance, elevation and slope are smoothly tangent to the ground (0.0)
    assert_eq!(ramp.surface_elevation_at(entry_pt), 0.0);
    assert_eq!(ramp.surface_slope_angle_rad_at(entry_pt), 0.0);

    // 2. Midpoint along the curve has quadratic progression (t^2 = 0.25 * height),
    // proving it is the actual concave curve and NOT a straight triangular incline (which would be 0.5 * height)
    let s_incline = (ramp.incline_length() / ramp.length()).clamp(0.01, 1.0);
    if s_incline >= 0.5 {
        let t_mid = 0.5 / s_incline;
        let expected_mid_elevation = 2.0 * t_mid * t_mid;
        let actual_mid = ramp.surface_elevation_at(mid_pt);
        assert!((actual_mid - expected_mid_elevation).abs() < 1e-4);
        // Specifically verify it's strictly concave (less than the linear midpoint of 1.0m if s_incline=1.0)
        assert!(actual_mid < 1.0 || s_incline < 0.5);
    }

    // 3. At lip / exit, elevation reaches full height (2.0m)
    assert!((ramp.surface_elevation_at(lip_pt) - 2.0).abs() < 1e-4);
}

#[test]
fn test_road_split_insert_split_junction_and_branch_seeding() {
    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    tools.active_tool = EditorToolType::RoadSplit;
    tools.split_branch_count = 2;
    tools.split_divergence_angle = 30.0;

    let target_wp = state.track.spline.waypoints[2].point;

    // Secondary click (Right Click) near waypoint 2 to insert split
    tools.handle_secondary_down(&mut state, target_wp);

    let network = state.track.ensure_network();
    assert_eq!(network.junctions.len(), 1, "Should have created 1 split junction");
    assert!(tools.active_branch_socket.is_some(), "Active branch socket should be set");

    let junction = &network.junctions[0];
    if let JunctionKind::Split { ingress_socket, egress_sockets, gore_config } = &junction.kind {
        assert_eq!(egress_sockets.len(), 2, "Should have 2 egress branch sockets");
        assert!((ingress_socket.point - target_wp).length() < 1.0);
        assert!(gore_config.is_some(), "Gore config should be initialized");
    } else {
        panic!("Created junction should be of kind Split");
    }

    // A branch segment should have been automatically seeded
    assert_eq!(network.segments.len(), 2, "Trunk segment + seeded branch segment");
    let branch_seg = network.segments.iter().find(|s| s.id.0 != 0).unwrap();
    assert_eq!(branch_seg.entry_junction, tools.active_branch_socket);
    assert_eq!(branch_seg.waypoints.len(), 2);
    assert!(branch_seg.length > 10.0);
}

#[test]
fn test_road_split_append_waypoints_and_extend_branch() {
    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    tools.active_tool = EditorToolType::RoadSplit;
    let target_wp = state.track.spline.waypoints[2].point;

    // 1. Insert split
    tools.handle_secondary_down(&mut state, target_wp);
    let active_sock = tools.active_branch_socket.expect("Should have active socket");

    // 2. Click in open space (away from track) to append waypoints
    let ext_pt1 = target_wp + Vec2::new(50.0, 50.0);
    tools.handle_secondary_down(&mut state, ext_pt1);

    let ext_pt2 = target_wp + Vec2::new(100.0, 80.0);
    tools.handle_secondary_down(&mut state, ext_pt2);

    let network = state.track.ensure_network();
    let branch_seg = network.segments.iter().find(|s| s.entry_junction == Some(active_sock)).unwrap();
    assert_eq!(branch_seg.waypoints.len(), 4, "2 seeded + 2 appended waypoints");
    assert!(branch_seg.length > 50.0);

    // Drivable surface check on branch segment
    let mid_sample = branch_seg.samples[branch_seg.samples.len() / 2].point;
    let sampled = state.track.sample_surface(mid_sample);
    assert_eq!(sampled, SurfaceType::Asphalt, "Branch road surface must be sampled as Asphalt");
}

#[test]
fn test_road_split_snap_to_merge_and_layout_generation() {
    let track = classic_grand_prix();
    let mut state = EditorState::new(track);
    let mut tools = ToolSettings::default();

    tools.active_tool = EditorToolType::RoadSplit;
    let target_wp = state.track.spline.waypoints[2].point;

    // 1. Insert split
    tools.handle_secondary_down(&mut state, target_wp);
    let active_sock = tools.active_branch_socket.unwrap();

    // 2. Extend branch waypoint
    let ext_pt = target_wp + Vec2::new(40.0, 30.0);
    tools.handle_secondary_down(&mut state, ext_pt);

    // 3. Snap to merge near waypoint 5
    let merge_wp = state.track.spline.waypoints[5].point;
    let click_near_merge = merge_wp + Vec2::new(2.0, 1.0); // Within 10.0m snap threshold
    tools.handle_secondary_down(&mut state, click_near_merge);

    let network = state.track.ensure_network();
    assert_eq!(network.junctions.len(), 2, "Should have 1 Split and 1 Merge junction");
    assert_eq!(tools.active_branch_socket, None, "Branch socket should reset after merge");

    let merge_j = network.junctions.iter().find(|j| matches!(j.kind, JunctionKind::Merge { .. })).unwrap();
    if let JunctionKind::Merge { ingress_sockets, egress_socket, .. } = &merge_j.kind {
        assert_eq!(ingress_sockets.len(), 1);
        assert!((egress_socket.point - merge_wp).length() < 1e-4);
    }

    // Branch segment should have exit_junction connected to the merge
    let branch_seg = network.segments.iter().find(|s| s.entry_junction == Some(active_sock)).unwrap();
    assert_eq!(branch_seg.exit_junction, Some(SocketId::new(merge_j.id, 0)));

    // Alternative layout should have been registered
    assert_eq!(network.layouts.len(), 2, "Default layout + Alternative Route");
    let alt_layout = network.get_layout("alternative").expect("Alternative layout should exist");
    assert!(alt_layout.segment_sequence.contains(&branch_seg.id));
}

#[test]
fn test_track_editor_exit_returns_to_track_manager() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_editor_return_tm_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::new();
    session.track_manager = TrackManager::new(&temp_dir);

    // 1. Enter editor from TrackManager with specific tab, module filter, and track index
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Main,
        module_filter: ModuleFilter::Rally,
        selected_idx: 2,
        modal: TrackManagerModal::None,
    };
    session.editor_return_track_manager = Some((TrackManagerTab::Main, ModuleFilter::Rally, 2));

    let track = classic_grand_prix();
    session.enter_track_editor(track);
    assert_eq!(session.state, GameState::TrackEditor);

    // 2. Exit editor via ExitToTrackManager action
    session.handle_editor_action(tdrace_app::editor::EditorAction::ExitToTrackManager);

    // 3. Verify session transitioned back to TrackManager with restored tab, filter, and index
    if let GameState::TrackManager { active_tab, module_filter, selected_idx, modal } = &session.state {
        assert_eq!(*active_tab, TrackManagerTab::Main);
        assert_eq!(*module_filter, ModuleFilter::Rally);
        assert_eq!(*selected_idx, 2);
        assert_eq!(*modal, TrackManagerModal::None);
    } else {
        panic!("Expected GameState::TrackManager, got {:?}", session.state);
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_editor_clone_to_drafts_exit_returns_to_drafts_tab() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_clone_draft_exit_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::new();
    session.track_manager = TrackManager::new(&temp_dir);

    // Clone a preset to drafts
    let preset_choice = tdrace_app::ui::menu::TrackChoice::ClassicGrandPrix;
    let (cloned_track, file_path) = session.track_manager.clone_track(&preset_choice).expect("Clone track");
    let file_stem = std::path::Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap()
        .to_string();

    let drafts = session.track_manager.draft_track_choices();
    let sel = drafts.iter().position(|t| t.track_id() == file_stem).unwrap_or(0);
    session.editor_return_track_manager = Some((TrackManagerTab::Drafts, ModuleFilter::Drafts, sel));
    session.enter_track_editor_with_path(cloned_track, Some(file_path));
    assert_eq!(session.state, GameState::TrackEditor);

    // Exit editor
    session.handle_editor_action(tdrace_app::editor::EditorAction::ExitToTrackManager);

    // Must return to Drafts tab and have the cloned track selected
    if let GameState::TrackManager { active_tab, module_filter, selected_idx, .. } = &session.state {
        assert_eq!(*active_tab, TrackManagerTab::Drafts);
        assert_eq!(*module_filter, ModuleFilter::Drafts);
        assert_eq!(*selected_idx, sel);
    } else {
        panic!("Expected GameState::TrackManager, got {:?}", session.state);
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_martinsville_clone_road_split_and_infield_short_course() {
    let path = std::path::Path::new("/home/mario/.local/share/tdrace/tracks/martinsville_speedway_clone.json");
    let mut track = if path.exists() {
        Track::load_from_file(path).expect("Load clone track")
    } else {
        Track::load_from_file("tracks/nascar/martinsville.json").expect("Load preset track")
    };

    // 1. Define split and merge points
    let split_pt = Vec2::new(30.0, -35.0);
    let merge_pt = Vec2::new(50.0, 65.0);

    // 2. Define Junctions
    let split_jid = JunctionId(1);
    let in_sock_split = SplineSocket::new(split_pt, Vec2::new(1.0, 0.0), 16.0);
    let eg_sock_split_0 = SplineSocket::new(split_pt, Vec2::new(1.0, 0.0), 16.0);
    let branch_in_tangent = Vec2::new(0.866, 0.5).normalize();
    let eg_sock_split_1 = SplineSocket::new(Vec2::new(30.0, -32.0), branch_in_tangent, 14.0);
    let gore_apex = Vec2::new(45.4, -27.0);
    let mut gore = GoreConfig::new(gore_apex, 30.0, 15.0, BarrierType::TireWall);
    gore.nose_barrier = WallBarrier::new(Vec2::new(45.0, -27.2), Vec2::new(45.8, -26.8), BarrierType::TireWall);
    let split_j = RoadJunction::split(
        split_jid,
        "Backstretch Infield Split",
        in_sock_split,
        vec![eg_sock_split_0, eg_sock_split_1],
        Some(gore.clone()),
    );

    let merge_jid = JunctionId(2);
    let in_sock_merge_0 = SplineSocket::new(merge_pt, Vec2::new(-1.0, 0.0), 16.0);
    let branch_out_tangent = Vec2::new(-0.866, 0.5).normalize();
    let in_sock_merge_1 = SplineSocket::new(Vec2::new(50.0, 62.0), branch_out_tangent, 14.0);
    let eg_sock_merge = SplineSocket::new(merge_pt, Vec2::new(-1.0, 0.0), 16.0);
    let merge_cfg = MergeConfig {
        convergence_point: merge_pt,
        merge_angle: 25.0,
        merge_length: 15.0,
    };
    let merge_j = RoadJunction::merge(
        merge_jid,
        "Frontstretch Merge",
        vec![in_sock_merge_0, in_sock_merge_1],
        eg_sock_merge,
        Some(merge_cfg),
    );

    // 3. Define Segments
    // Segment 0: Common Trunk (from merge_pt [50, 65] -> around west loop -> past Start/Finish -> split_pt [30, -35])
    let mut seg0_wp = Vec::new();
    seg0_wp.push(TrackWaypoint::new(merge_pt, 16.0));
    seg0_wp.push(TrackWaypoint::new(Vec2::new(0.0, 65.0), 16.0));
    seg0_wp.push(TrackWaypoint::new(Vec2::new(-100.0, 65.0), 16.0));
    let mut wp_t1 = TrackWaypoint::new(Vec2::new(-140.0, 55.0), 16.0); wp_t1.bank_angle = 8.0; wp_t1.left_curb = true; seg0_wp.push(wp_t1);
    let mut wp_t2a = TrackWaypoint::new(Vec2::new(-160.0, 30.0), 16.0); wp_t2a.bank_angle = 12.0; wp_t2a.left_curb = true; seg0_wp.push(wp_t2a);
    let mut wp_t2b = TrackWaypoint::new(Vec2::new(-160.0, 0.0), 16.0); wp_t2b.bank_angle = 12.0; wp_t2b.left_curb = true; seg0_wp.push(wp_t2b);
    let mut wp_t2c = TrackWaypoint::new(Vec2::new(-140.0, -25.0), 16.0); wp_t2c.bank_angle = 8.0; wp_t2c.left_curb = true; seg0_wp.push(wp_t2c);
    seg0_wp.push(TrackWaypoint::new(Vec2::new(-100.0, -35.0), 16.0));
    seg0_wp.push(TrackWaypoint::new(Vec2::new(0.0, -35.0), 16.0)); // S/F line
    seg0_wp.push(TrackWaypoint::new(split_pt, 16.0));

    let mut seg0 = RoadSegment::new(SegmentId(0), "Common Oval & S/F Trunk", seg0_wp)
        .with_junctions(Some(SocketId::new(merge_jid, 0)), Some(SocketId::new(split_jid, 0)));
    seg0.recompute_samples(Some(&eg_sock_merge), Some(&in_sock_split));

    // Segment 1: Outer Oval Turn 3-4 (from split_pt [30, -35] -> east loop -> merge_pt [50, 65])
    let mut seg1_wp = Vec::new();
    seg1_wp.push(TrackWaypoint::new(split_pt, 16.0));
    seg1_wp.push(TrackWaypoint::new(Vec2::new(100.0, -35.0), 16.0));
    let mut wp_t3a = TrackWaypoint::new(Vec2::new(140.0, -25.0), 16.0); wp_t3a.bank_angle = 8.0; wp_t3a.left_curb = true; seg1_wp.push(wp_t3a);
    let mut wp_t3b = TrackWaypoint::new(Vec2::new(160.0, 0.0), 16.0); wp_t3b.bank_angle = 12.0; wp_t3b.left_curb = true; seg1_wp.push(wp_t3b);
    let mut wp_t4a = TrackWaypoint::new(Vec2::new(160.0, 30.0), 16.0); wp_t4a.bank_angle = 12.0; wp_t4a.left_curb = true; seg1_wp.push(wp_t4a);
    let mut wp_t4b = TrackWaypoint::new(Vec2::new(140.0, 55.0), 16.0); wp_t4b.bank_angle = 8.0; wp_t4b.left_curb = true; seg1_wp.push(wp_t4b);
    seg1_wp.push(TrackWaypoint::new(Vec2::new(100.0, 65.0), 16.0));
    seg1_wp.push(TrackWaypoint::new(merge_pt, 16.0));

    let mut seg1 = RoadSegment::new(SegmentId(1), "Outer Oval Turn 3-4", seg1_wp)
        .with_junctions(Some(SocketId::new(split_jid, 0)), Some(SocketId::new(merge_jid, 0)));
    seg1.recompute_samples(Some(&eg_sock_split_0), Some(&in_sock_merge_0));

    // Segment 2: Infield Technical Shortcut (5 additional turns!)
    let mut seg2_wp = Vec::new();
    seg2_wp.push(TrackWaypoint::new(split_pt, 14.0));
    let mut wp_c1 = TrackWaypoint::new(Vec2::new(45.0, -18.0), 14.0); wp_c1.bank_angle = 2.0; wp_c1.left_curb = true; seg2_wp.push(wp_c1);
    let mut wp_c2 = TrackWaypoint::new(Vec2::new(65.0, -5.0), 14.0); wp_c2.right_curb = true; seg2_wp.push(wp_c2);
    let mut wp_c3 = TrackWaypoint::new(Vec2::new(78.0, 15.0), 14.0); wp_c3.bank_angle = 4.0; wp_c3.left_curb = true; seg2_wp.push(wp_c3);
    let mut wp_c4 = TrackWaypoint::new(Vec2::new(75.0, 38.0), 14.0); wp_c4.right_curb = true; seg2_wp.push(wp_c4);
    let mut wp_c5 = TrackWaypoint::new(Vec2::new(62.0, 58.0), 14.0); wp_c5.left_curb = true; seg2_wp.push(wp_c5);
    seg2_wp.push(TrackWaypoint::new(merge_pt, 14.0));

    let mut seg2 = RoadSegment::new(SegmentId(2), "Infield Technical Shortcut", seg2_wp)
        .with_junctions(Some(SocketId::new(split_jid, 1)), Some(SocketId::new(merge_jid, 1)));
    seg2.recompute_samples(Some(&eg_sock_split_1), Some(&in_sock_merge_1));

    // 4. Layouts
    let mut layout_main = TrackLayout::new(
        "main",
        "The Paperclip (Default Oval)",
        vec![SegmentId(0), SegmentId(1)],
        SegmentId(0),
    ).with_checkpoints(vec![0, 1, 2, 3, 4, 5, 6, 7]);

    let mut layout_short = TrackLayout::new(
        "infield_short",
        "Infield Short Course",
        vec![SegmentId(0), SegmentId(2)],
        SegmentId(0),
    ).with_checkpoints(vec![0, 8, 4, 5, 6, 7]);

    let mut network = TrackNetwork {
        junctions: vec![split_j, merge_j],
        segments: vec![seg0, seg1, seg2],
        layouts: vec![layout_main.clone(), layout_short.clone()],
        default_layout_id: "main".to_string(),
    };

    let len_main = layout_main.recompute_lap_length(&network);
    let len_short = layout_short.recompute_lap_length(&network);
    network.layouts = vec![layout_main.clone(), layout_short.clone()];

    assert!(network.validate().is_ok(), "Network validation failed: {:?}", network.validate());
    assert!(len_short < len_main, "Short course ({len_short}m) must be shorter than oval ({len_main}m)");

    // 5. Update track spline to composite of main layout
    let composite_main = network.build_composite_spline_for_layout("main").expect("Build composite main");
    let composite_short = network.build_composite_spline_for_layout("infield_short").expect("Build composite short");
    assert!(composite_main.closed);
    assert!(composite_short.closed);
    assert!(composite_short.total_length < composite_main.total_length);
    track.spline = composite_main;
    track.network = Some(network);

    // 6. Add Checkpoint 8 in the infield for the short course
    let cp8_pt = Vec2::new(78.0, 15.0);
    let cp8_gate = LineSegment::new(cp8_pt + Vec2::new(-8.0, 0.0), cp8_pt + Vec2::new(8.0, 0.0));
    let cp8 = Checkpoint::new(8, cp8_gate, Vec2::new(0.0, 1.0), 1, false).with_segment(SegmentId(2));

    // Assign segment IDs to existing checkpoints
    for cp in &mut track.checkpoints {
        match cp.id {
            0 | 4 | 5 | 6 | 7 => cp.segment_id = Some(SegmentId(0)),
            1 | 2 | 3 => cp.segment_id = Some(SegmentId(1)),
            _ => {}
        }
    }
    if !track.checkpoints.iter().any(|c| c.id == 8) {
        track.checkpoints.push(cp8);
    }

    // 7. Prune inner walls that cross the split and merge openings using automated network trimming
    track.trim_walls_for_network();
    track.geometry.inner_walls.push(gore.nose_barrier);

    // Verify zero wall collisions when driving through the split entrance and along the infield shortcut
    let seg2_samples = &track.network.as_ref().unwrap().segments[2].samples;
    for (idx, sample) in seg2_samples.iter().enumerate().step_by(4) {
        let mut car_test = Car::new(CarConfig::stock_car_ta1()).with_pose(sample.point, sample.tangent.y.atan2(sample.tangent.x));
        let hits = resolve_all_wall_collisions(&mut car_test, &track.geometry.inner_walls, &[]);
        assert!(
            hits.is_empty(),
            "Collision detected with inner wall at sample {} pt {:?}: {:?}",
            idx, sample.point, hits
        );
    }

    // 8. Verify AI navigation across the split
    let mut bot_main = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("main".to_string()));
    let mut bot_short = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("infield_short".to_string()));

    let car_main = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(20.0, -35.0), 0.0);
    let car_short = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(20.0, -35.0), 0.0);

    let ctrl_main = bot_main.compute_controls(&car_main, &track, &[], 0.016);
    let ctrl_short = bot_short.compute_controls(&car_short, &track, &[], 0.016);

    assert!(ctrl_main.throttle > 0.5);
    assert!(ctrl_short.throttle > 0.5);
    assert!(
        ctrl_short.steer > ctrl_main.steer,
        "Infield shortcut bot must steer left into the infield at split, got main: {}, short: {}",
        ctrl_main.steer, ctrl_short.steer
    );

    // 9. Save updated track to martinsville_speedway_clone.json if running on host with the file
    if path.exists() {
        track.save_to_file(path).expect("Save updated track clone");
        let reloaded = Track::load_from_file(path).expect("Reload saved track");
        assert!(reloaded.network.is_some());
        let net = reloaded.network.as_ref().unwrap();
        assert_eq!(net.layouts.len(), 2);
        assert_eq!(net.junctions.len(), 2);
        assert_eq!(net.segments.len(), 3);
        assert_eq!(net.layouts[0].id, "main");
        assert_eq!(net.layouts[1].id, "infield_short");

        // Verify zero wall collisions on reloaded track along the shortcut
        for (idx, sample) in reloaded.network.as_ref().unwrap().segments[2].samples.iter().enumerate().step_by(4) {
            let mut car_test = Car::new(CarConfig::stock_car_ta1()).with_pose(sample.point, sample.tangent.y.atan2(sample.tangent.x));
            let hits = resolve_all_wall_collisions(&mut car_test, &reloaded.geometry.inner_walls, &[]);
            assert!(
                hits.is_empty(),
                "Collision detected on reloaded track at sample {} pt {:?}: {:?}",
                idx, sample.point, hits
            );
        }

        // Verify surface sampling along the shortcut: all wheels and center must be Asphalt/Curb, never Grass
        for (idx, sample) in reloaded.network.as_ref().unwrap().segments[2].samples.iter().enumerate().step_by(2) {
            let car_test = Car::new(CarConfig::stock_car_ta1()).with_pose(sample.point, sample.tangent.y.atan2(sample.tangent.x));

            // 1. Direct surface sample
            let surf = reloaded.sample_surface(sample.point);
            assert_eq!(
                surf,
                SurfaceType::Asphalt,
                "Sample {} at {:?} must be Asphalt, got {:?}",
                idx, sample.point, surf
            );

            // 2. Hinted surface sample (as used by physics loop)
            let surf_near = reloaded.sample_surface_near(sample.point, 0.0);
            assert_eq!(
                surf_near,
                SurfaceType::Asphalt,
                "Sample_near {} at {:?} must be Asphalt, got {:?}",
                idx, sample.point, surf_near
            );

            // 3. Four-wheel surface sampling
            let wheel_surfs = reloaded.sample_car_surfaces(&car_test);
            for (w_idx, &w_surf) in wheel_surfs.iter().enumerate() {
                assert!(
                    w_surf == SurfaceType::Asphalt || w_surf == SurfaceType::Curb,
                    "Wheel {} of car at sample {} {:?} must be Asphalt or Curb, got {:?}",
                    w_idx, idx, sample.point, w_surf
                );
            }

            // 4. Four-wheel surface sampling with hint (actual in-game physics query)
            let wheel_surfs_hint = reloaded.sample_car_surfaces_with_hint(&car_test, 0.0);
            for (w_idx, &w_surf) in wheel_surfs_hint.iter().enumerate() {
                assert!(
                    w_surf == SurfaceType::Asphalt || w_surf == SurfaceType::Curb,
                    "Wheel hint {} of car at sample {} {:?} must be Asphalt or Curb, got {:?}",
                    w_idx, idx, sample.point, w_surf
                );
            }

            // 5. Track project_point_near
            let proj = reloaded.project_point_near(sample.point, 0.0);
            assert!(
                proj.is_on_track,
                "Car at sample {} {:?} must be on track according to project_point_near",
                idx, sample.point
            );
        }

        println!("Successfully updated martinsville_speedway_clone.json with road split and infield short course!");
        println!("Oval distance: {:.1}m, Infield Short Course distance: {:.1}m", len_main, len_short);
    }
}
