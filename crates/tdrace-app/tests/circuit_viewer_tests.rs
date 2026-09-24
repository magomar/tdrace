use glam::Vec2;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::circuit_viewer::{
    compute_full_track_bounds, CircuitViewerOrigin, CircuitViewerState,
};
use tdrace_app::ui::menu::track_select_preview_rect;
use tdrace_core::track::geometry::Obstacle;
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::scenery::{Tree, TreeType};

#[test]
fn test_open_circuit_viewer_transitions_state_and_stores_state() {
    let mut session = RaceSession::new();
    let track = classic_grand_prix();
    let title = "Monza Classic Grand Prix".to_string();

    session.open_circuit_viewer(track, title.clone(), CircuitViewerOrigin::Menu);

    assert_eq!(
        session.state,
        GameState::CircuitViewer(CircuitViewerOrigin::Menu)
    );
    assert!(session.circuit_viewer_state.is_some());

    let state = session.circuit_viewer_state.as_ref().unwrap();
    assert_eq!(state.origin, CircuitViewerOrigin::Menu);
    assert_eq!(state.title, title);
    assert!(state.camera_zoom > 0.0);
    assert!(state.base_zoom > 0.0);
    assert_eq!(state.camera_zoom, state.base_zoom);
    assert_eq!(state.camera_center, state.base_center);
}

#[test]
fn test_circuit_viewer_bounds_and_fit_zoom() {
    let track = classic_grand_prix();
    let sw = 1920.0;
    let sh = 1080.0;

    let (min, max) = compute_full_track_bounds(&track);
    let expected_center = (min + max) * 0.5;

    let state = CircuitViewerState::new(
        track,
        "Test Circuit".to_string(),
        "gt".to_string(),
        CircuitViewerOrigin::Menu,
        sw,
        sh,
    );

    assert_eq!(state.camera_center, expected_center);
    assert_eq!(state.base_center, expected_center);
    assert!(state.base_zoom > 0.0);

    // Zoom should be scaled such that the full extent fits within available screen dimensions
    let extent = max - min;
    let rendered_w = extent.x * state.base_zoom;
    let rendered_h = extent.y * state.base_zoom;

    assert!(rendered_w <= sw);
    assert!(rendered_h <= sh);
}

#[test]
fn test_circuit_viewer_pan_and_reset_to_fit() {
    let track = classic_grand_prix();
    let mut state = CircuitViewerState::new(
        track,
        "Test Track".to_string(),
        "gt".to_string(),
        CircuitViewerOrigin::Menu,
        1920.0,
        1080.0,
    );

    let initial_center = state.camera_center;
    let initial_zoom = state.camera_zoom;

    // Simulate user pan & zoom
    state.camera_center += Vec2::new(250.0, -180.0);
    state.camera_zoom *= 3.0;

    assert_ne!(state.camera_center, initial_center);
    assert_ne!(state.camera_zoom, initial_zoom);

    // Reset to fit should restore initial overview center & zoom
    state.reset_to_fit();

    assert_eq!(state.camera_center, initial_center);
    assert_eq!(state.camera_zoom, initial_zoom);
    assert!(!state.is_panning);
}

#[test]
fn test_update_circuit_viewer_state_exit() {
    let mut session = RaceSession::new();
    let track = classic_grand_prix();
    session.open_circuit_viewer(track, "Classic GP".to_string(), CircuitViewerOrigin::Menu);

    assert_eq!(
        session.state,
        GameState::CircuitViewer(CircuitViewerOrigin::Menu)
    );

    // When circuit_viewer_state is cleared (simulating exit), update_circuit_viewer restores Menu
    session.circuit_viewer_state = None;
    session.update_circuit_viewer(CircuitViewerOrigin::Menu, 0.016);
    assert_eq!(session.state, GameState::Menu);

    // Test StartingGrid origin
    let track2 = classic_grand_prix();
    session.open_circuit_viewer(
        track2,
        "Classic GP".to_string(),
        CircuitViewerOrigin::StartingGrid,
    );
    assert_eq!(
        session.state,
        GameState::CircuitViewer(CircuitViewerOrigin::StartingGrid)
    );
    session.circuit_viewer_state = None;
    session.update_circuit_viewer(CircuitViewerOrigin::StartingGrid, 0.016);
    assert_eq!(session.state, GameState::StartingGrid);
}

#[test]
fn test_track_select_preview_rect_geometry() {
    let sw = 1920.0;
    let sh = 1080.0;

    let (x1, y1, w1, h1) = track_select_preview_rect(sw, sh, false, false, false);
    assert!(w1 > 200.0);
    assert!(h1 > 100.0);
    assert!(x1 > 0.0);
    assert!(y1 > 0.0);

    // With status banner, preview card starts lower
    let (x2, y2, w2, h2) = track_select_preview_rect(sw, sh, false, true, false);
    assert_eq!(x1, x2);
    assert_eq!(w1, w2);
    assert!(y2 > y1);
    assert!(h2 <= h1);
}

#[test]
fn test_compute_full_track_bounds_incorporates_scenery() {
    let mut track = classic_grand_prix();
    let (initial_min, initial_max) = compute_full_track_bounds(&track);

    // Add an obstacle far outside current bounds
    let far_point = initial_max + Vec2::new(500.0, 500.0);
    track.geometry.obstacles.push(Obstacle::oriented_box(
        999,
        far_point,
        Vec2::new(20.0, 20.0),
        0.0,
        "Far Obstacle",
    ));

    let (expanded_min, expanded_max) = compute_full_track_bounds(&track);
    assert_eq!(initial_min, expanded_min);
    assert!(expanded_max.x >= far_point.x + 20.0);
    assert!(expanded_max.y >= far_point.y + 20.0);

    // Add a tree far in the negative direction
    let neg_point = initial_min - Vec2::new(400.0, 400.0);
    track.geometry.trees.push(Tree::new(500, neg_point, TreeType::Pine));

    let (neg_min, _) = compute_full_track_bounds(&track);
    assert!(neg_min.x <= neg_point.x);
    assert!(neg_min.y <= neg_point.y);
}
