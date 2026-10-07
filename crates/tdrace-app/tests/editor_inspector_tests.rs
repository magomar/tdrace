//! Track Studio inspector tests for specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use glam::Vec2;
use tdrace_app::editor::inspector::{begin_inspector_frame, HoldRepeat, HOLD_REPEAT_DELAY, HOLD_REPEAT_INTERVAL};
use tdrace_app::editor::state::dev_selection;
use tdrace_app::editor::{EditorState, Selection, ToolSettings};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, LineSegment, Obstacle, PitBox, PitLane, SurfaceShape, SurfaceZone};
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

const FIXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor_inspector_fixture.json");

/// Builds the inspector fixture: a rounded 240 m x 140 m loop with every editor entity kind.
fn build_fixture() -> Track {
    let points = [
        Vec2::new(-100.0, -70.0),
        Vec2::new(0.0, -70.0),
        Vec2::new(100.0, -70.0),
        Vec2::new(120.0, -50.0),
        Vec2::new(120.0, 50.0),
        Vec2::new(100.0, 70.0),
        Vec2::new(0.0, 70.0),
        Vec2::new(-100.0, 70.0),
        Vec2::new(-120.0, 50.0),
        Vec2::new(-120.0, -50.0),
    ];
    let mut track = Track {
        name: "Editor Inspector Fixture".to_string(),
        spline: TrackSpline::from_points(&points, 12.0, true),
        ..Track::default()
    };
    track.rebuild_geometry(8.0, BarrierType::Steel);

    let g = &mut track.geometry;
    for (i, (x, surface)) in [(-40.0, SurfaceType::Gravel), (40.0, SurfaceType::Water)].into_iter().enumerate() {
        let shape = SurfaceShape::OrientedBox { center: Vec2::new(x, 20.0), half_extents: Vec2::new(10.0, 6.0), angle: 0.0 };
        g.surface_zones.push(SurfaceZone::new(shape, surface, format!("Fixture Zone {}", i + 1)));
    }
    for i in 0..3 {
        let center = Vec2::new(-20.0 + 20.0 * i as f32, -20.0);
        g.obstacles.push(Obstacle::oriented_box(i + 1, center, Vec2::new(2.0, 2.0), 0.0, format!("Fixture Box {}", i + 1)));
    }
    for (i, x) in [-50.0, 50.0].into_iter().enumerate() {
        let shape = SurfaceShape::OrientedBox { center: Vec2::new(x, 70.0), half_extents: Vec2::new(6.0, 4.0), angle: 0.0 };
        g.jump_ramps.push(JumpRamp::new(i + 1, shape, Vec2::NEG_X, 4.0, 15.0, 1.8, format!("Fixture Ramp {}", i + 1)));
    }

    let pit_points = [Vec2::new(-80.0, -55.0), Vec2::new(0.0, -55.0), Vec2::new(80.0, -55.0)];
    track.pit_lane = Some(PitLane::new(
        TrackSpline::from_points(&pit_points, 6.0, false),
        6.0,
        PitLane::DEFAULT_ROAD_SPEED_LIMIT,
        vec![PitBox::new(Vec2::new(0.0, -55.0), Vec2::X, 3.0, 0.0)],
        LineSegment::new(Vec2::new(-80.0, -58.0), Vec2::new(-80.0, -52.0)),
        LineSegment::new(Vec2::new(80.0, -58.0), Vec2::new(80.0, -52.0)),
    ));
    track.auto_generate_checkpoints(8, 3);
    track.auto_generate_grid_default();
    track
}

/// Regenerates the fixture file: `cargo test -p tdrace-app --test editor_inspector_tests -- --ignored`.
#[test]
#[ignore]
fn regenerate_editor_inspector_fixture() {
    let json = build_fixture().to_json().expect("fixture serializes");
    std::fs::write(FIXTURE_PATH, json).expect("fixture written");
}

#[test]
fn fixture_has_every_entity_kind() {
    let track = Track::load_from_file(FIXTURE_PATH).expect("fixture loads");
    assert!(track.spline.waypoints.len() >= 3);
    assert_eq!(track.geometry.surface_zones.len(), 2);
    assert_eq!(track.geometry.obstacles.len(), 3);
    assert_eq!(track.geometry.jump_ramps.len(), 2);
    assert!(!track.checkpoints.is_empty());
    assert!(!track.grid_positions.is_empty());
    assert!(track.pit_lane.is_some());
}

#[test]
fn dev_selection_picks_each_kind_from_the_fixture() {
    let track = Track::load_from_file(FIXTURE_PATH).expect("fixture loads");
    assert_eq!(dev_selection("waypoint", &track), Selection::Waypoint(0));
    assert_eq!(dev_selection("waypoints", &track), Selection::MultipleWaypoints(vec![0, 1, 2]));
    assert_eq!(dev_selection("zone", &track), Selection::SurfaceZone(0));
    assert_eq!(dev_selection("obstacle", &track), Selection::Obstacle(0));
    assert_eq!(dev_selection("ramp", &track), Selection::JumpRamp(0));
    assert_eq!(dev_selection("checkpoint", &track), Selection::Checkpoint(0));
    assert_eq!(dev_selection("grid", &track), Selection::GridSlot(0));
    assert_eq!(dev_selection("pit", &track), Selection::PitBox);
    assert_eq!(dev_selection("none", &track), Selection::None);
    assert_eq!(dev_selection("zones", &track).selected_surface_zone_indices(), vec![0, 1]);
    assert_eq!(dev_selection("obstacles", &track).selected_obstacle_indices(), vec![0, 1, 2]);
    assert_eq!(dev_selection("ramps", &track).selected_jump_ramp_indices(), vec![0, 1]);
    let mixed = dev_selection("mixed", &track);
    assert_eq!(mixed.selected_waypoint_indices(), vec![0, 1]);
    assert_eq!(mixed.selected_surface_zone_indices(), vec![0]);
    assert_eq!(mixed.selected_jump_ramp_indices(), vec![0]);
}

#[test]
fn dev_selection_drops_missing_indices() {
    let track = Track::default();
    assert_eq!(dev_selection("ramps", &track), Selection::None);
    assert_eq!(dev_selection("pit", &track), Selection::None);
    assert_eq!(dev_selection("unknown-kind", &track), Selection::None);
}

fn fixture_state() -> EditorState {
    EditorState::new(Track::load_from_file(FIXTURE_PATH).expect("fixture loads"))
}

#[test]
fn one_slider_drag_is_one_undo_step() {
    let mut state = fixture_state();
    let mut tools = ToolSettings::default();
    // An earlier, separate edit.
    state.record_undo();
    state.track.spline.waypoints[1].width = 20.0;
    let before_drag = state.track.spline.waypoints[0].width;

    // Press on the inspector, then 120 frames of drag, each recording undo like a slider call site.
    begin_inspector_frame(&mut state, &mut tools, true, true, true);
    for frame in 0..120 {
        state.record_undo();
        state.track.spline.waypoints[0].width = 6.0 + frame as f32 * 0.1;
        begin_inspector_frame(&mut state, &mut tools, false, false, true);
    }
    begin_inspector_frame(&mut state, &mut tools, true, false, false);

    assert_eq!(state.history.undo_count(), 2, "earlier edit + one step for the whole drag");
    assert!(state.undo());
    assert_eq!(state.track.spline.waypoints[0].width, before_drag);
    assert_eq!(state.track.spline.waypoints[1].width, 20.0, "earlier edit is still applied");
    assert!(state.undo());
    assert_ne!(state.track.spline.waypoints[1].width, 20.0, "earlier edit is still in the history");
}

#[test]
fn canvas_press_does_not_group_undo_steps() {
    let mut state = fixture_state();
    let mut tools = ToolSettings::default();
    begin_inspector_frame(&mut state, &mut tools, false, true, true);
    state.record_undo();
    state.record_undo();
    assert_eq!(state.history.undo_count(), 2);
}

#[test]
fn hold_repeat_steps_on_press_then_after_delay_at_ten_hz() {
    let mut hold = HoldRepeat::default();
    assert!(hold.poll("w:inc", true, true, true, 0.0), "first step on press");
    assert!(!hold.poll("w:inc", false, true, true, HOLD_REPEAT_DELAY - 0.05));
    assert!(hold.poll("w:inc", false, true, true, HOLD_REPEAT_DELAY));
    assert!(!hold.poll("w:inc", false, true, true, HOLD_REPEAT_DELAY + HOLD_REPEAT_INTERVAL * 0.5));
    assert!(hold.poll("w:inc", false, true, true, HOLD_REPEAT_DELAY + HOLD_REPEAT_INTERVAL));
    assert!(hold.poll("w:inc", false, true, true, 3.0), "a stalled frame steps once");
    assert!(!hold.poll("w:inc", false, true, true, 3.0 + 1.0 / 60.0), "and does not burst afterwards");
    assert!(!hold.poll("w:dec", false, true, true, 5.0), "another button never repeats");
    assert!(!hold.poll("w:inc", false, true, false, 5.0), "no repeat while the pointer is off the button");
    assert!(!hold.poll("w:inc", false, false, true, 5.1), "release stops the repeat");
    assert!(!hold.poll("w:inc", false, true, true, 6.0), "moving back without a new press does not restart");
}

#[test]
fn holding_a_step_button_is_one_undo_step() {
    let mut state = fixture_state();
    let mut tools = ToolSettings::default();
    let original = state.track.spline.waypoints[0].width;
    let mut now = 0.0;
    begin_inspector_frame(&mut state, &mut tools, true, true, true);
    let mut pressed = true;
    while now <= 1.05 {
        if tools.inspector.hold.poll("width:inc", pressed, true, true, now) {
            state.record_undo();
            state.track.spline.waypoints[0].width += 0.5;
        }
        pressed = false;
        now += 1.0 / 60.0;
        begin_inspector_frame(&mut state, &mut tools, true, false, true);
    }
    begin_inspector_frame(&mut state, &mut tools, true, false, false);
    // 1 step on press + repeats from 0.4 s to 1.0 s at 10 Hz = 1 + 7.
    assert_eq!(state.track.spline.waypoints[0].width, original + 0.5 * 8.0);
    assert!(state.undo());
    assert_eq!(state.track.spline.waypoints[0].width, original);
    assert_eq!(state.history.undo_count(), 0);
}

#[test]
fn a_click_elsewhere_or_a_new_selection_clears_slider_focus() {
    let mut state = fixture_state();
    let mut tools = ToolSettings::default();
    begin_inspector_frame(&mut state, &mut tools, true, false, false);

    tools.select_bar("wp_width");
    begin_inspector_frame(&mut state, &mut tools, false, true, true);
    assert!(!tools.is_bar_selected("wp_width"), "a click on the canvas clears focus");

    tools.select_bar("wp_width");
    tools.start_editing_bar("wp_width", "12");
    state.select(Selection::Waypoint(1));
    begin_inspector_frame(&mut state, &mut tools, true, false, false);
    assert!(!tools.is_bar_selected("wp_width"), "a new selection clears focus");
    assert!(!tools.is_editing_text(), "a new selection cancels the inline edit");

    tools.select_bar("wp_width");
    begin_inspector_frame(&mut state, &mut tools, true, false, false);
    assert!(tools.is_bar_selected("wp_width"), "hovering without a click keeps focus");
}
