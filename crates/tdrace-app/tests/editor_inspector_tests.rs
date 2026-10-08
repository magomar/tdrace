//! Track Studio inspector tests for specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use glam::Vec2;
use tdrace_app::editor::inspector::{
    apply_edit, begin_inspector_frame, body_height, build_inspector, common_value, content_height, Action, Common, Edit, HoldRepeat,
    InspectorModel, Options, Prop, Row, HOLD_REPEAT_DELAY, HOLD_REPEAT_INTERVAL, RAMP_LENGTH_RANGE, SURFACES, WALL_DISTANCE_RANGE,
    WIDTH_RANGE,
};
use tdrace_app::editor::state::dev_selection;
use tdrace_app::editor::{EditorState, EditorToolType, Selection, ToolSettings};
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
    // Waypoints 0-2 differ in banking, width, curb and surface so a 3-waypoint selection shows mixed values.
    track.spline.waypoints[1].bank_angle = 5.0;
    track.spline.waypoints[2].bank_angle = 10.0;
    track.spline.waypoints[2].width = 14.0;
    track.spline.waypoints[2].left_curb = true;
    track.spline.waypoints[2].surface = Some(SurfaceType::Concrete);
    track.rebuild_geometry(8.0, BarrierType::Steel);

    let g = &mut track.geometry;
    for (i, (x, surface)) in [(-40.0, SurfaceType::PackedGravel), (40.0, SurfaceType::Water)].into_iter().enumerate() {
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

/// Inspector card height at the 1280x720 reference resolution (`sh - top_h - 54`).
const CARD_H_720P: f32 = 720.0 - 46.0 - 54.0;

fn selected(indices: &[usize]) -> EditorState {
    let mut state = fixture_state();
    let sel = if indices.len() == 1 { Selection::Waypoint(indices[0]) } else { Selection::MultipleWaypoints(indices.to_vec()) };
    state.select(sel);
    state
}

fn find_row<'a>(model: &'a InspectorModel, label: &str) -> &'a Row {
    model
        .sections
        .iter()
        .flat_map(|s| &s.rows)
        .find(|r| match r {
            Row::Stepper { label: l, .. } | Row::Dropdown { label: l, .. } | Row::Segmented { label: l, .. } | Row::Sides { label: l, .. } | Row::Chips { label: l, .. } => *l == label,
            _ => false,
        })
        .unwrap_or_else(|| panic!("row {label} not found"))
}

/// Section ids and row labels, to compare the shape of two inspector views.
fn shape(model: &InspectorModel) -> Vec<(String, Vec<String>)> {
    model
        .sections
        .iter()
        .map(|s| {
            let rows = s
                .rows
                .iter()
                .map(|r| match r {
                    Row::Stepper { label, .. } | Row::Dropdown { label, .. } | Row::Segmented { label, .. } | Row::Sides { label, .. } | Row::Chips { label, .. } => label.to_string(),
                    _ => "other".to_string(),
                })
                .collect();
            (s.id.to_string(), rows)
        })
        .collect()
}

fn banking(state: &EditorState, i: usize) -> f32 {
    state.track.spline.waypoints[i].bank_angle
}

#[test]
fn common_value_reports_same_mixed_and_empty() {
    assert_eq!(common_value([2.0, 2.0, 2.0]), Common::Same(2.0));
    assert_eq!(common_value([2.0, 3.0]), Common::Mixed);
    assert_eq!(common_value(Vec::<f32>::new()), Common::Empty);
}

#[test]
fn waypoint_view_fits_the_720p_card_fully_expanded() {
    for indices in [&[0][..], &[0, 1, 2][..]] {
        let model = build_inspector(&selected(indices), &ToolSettings::default()).expect("waypoint selections use the new inspector");
        let used = content_height(&model, |_| false);
        let room = body_height(&model, CARD_H_720P);
        assert!(used <= room, "{indices:?}: content {used} px > body {room} px");
        assert!(model.footer.is_some(), "duplicate and delete stay in the fixed footer");
    }
}

#[test]
fn several_waypoints_show_the_same_controls_as_one() {
    let one = build_inspector(&selected(&[0]), &ToolSettings::default()).unwrap();
    let many = build_inspector(&selected(&[0, 1, 2]), &ToolSettings::default()).unwrap();
    assert_eq!(shape(&one), shape(&many));
    assert_eq!(one.count, 1);
    assert_eq!(many.count, 3);
}

#[test]
fn equal_values_show_the_value_and_different_values_show_mixed() {
    let model = build_inspector(&selected(&[0, 1]), &ToolSettings::default()).unwrap();
    assert!(matches!(find_row(&model, "Width"), Row::Stepper { value: Common::Same(w), .. } if *w == 12.0));
    assert!(matches!(find_row(&model, "Angle"), Row::Stepper { value: Common::Mixed, .. }));

    let model = build_inspector(&selected(&[0, 1, 2]), &ToolSettings::default()).unwrap();
    assert!(matches!(find_row(&model, "Surface"), Row::Dropdown { value: Common::Mixed, .. }));
    assert!(matches!(find_row(&model, "Curb"), Row::Sides { left: (_, Common::Mixed), right: (_, Common::Same(false)), .. }));
    match find_row(&model, "Presets") {
        Row::Chips { chips, .. } => assert!(chips.iter().all(|c| !c.active), "no preset is lit for mixed banking"),
        _ => unreachable!(),
    }
}

#[test]
fn absolute_edits_set_all_and_relative_edits_keep_differences() {
    let mut state = selected(&[0, 1, 2]);
    let mut tools = ToolSettings::default();
    assert_eq!([banking(&state, 0), banking(&state, 1), banking(&state, 2)], [0.0, 5.0, 10.0]);

    apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpBanking, 18.0));
    assert_eq!([banking(&state, 0), banking(&state, 1), banking(&state, 2)], [18.0, 18.0, 18.0]);
    assert!(state.undo());
    assert_eq!([banking(&state, 0), banking(&state, 1), banking(&state, 2)], [0.0, 5.0, 10.0]);

    apply_edit(&mut state, &mut tools, Edit::Step(Prop::WpBanking, 1.0));
    assert_eq!([banking(&state, 0), banking(&state, 1), banking(&state, 2)], [1.0, 6.0, 11.0]);

    apply_edit(&mut state, &mut tools, Edit::Do(Action::InvertBanking));
    assert_eq!([banking(&state, 0), banking(&state, 1), banking(&state, 2)], [-1.0, -6.0, -11.0]);
}

#[test]
fn single_and_multi_selection_share_clamp_ranges() {
    for indices in [&[0][..], &[0, 1, 2][..]] {
        let mut state = selected(indices);
        let mut tools = ToolSettings::default();
        apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWidth, 999.0));
        apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWallDistL, 999.0));
        for &i in indices {
            assert_eq!(state.track.spline.waypoints[i].width, WIDTH_RANGE.1);
            assert_eq!(state.track.spline.waypoints[i].left_wall_distance, Some(WALL_DISTANCE_RANGE.1));
        }
        apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWidth, -5.0));
        apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWallDistL, 0.0));
        apply_edit(&mut state, &mut tools, Edit::Step(Prop::WpWallDistR, -999.0));
        for &i in indices {
            let wp = &state.track.spline.waypoints[i];
            assert_eq!(wp.width, WIDTH_RANGE.0);
            assert_eq!(wp.left_wall_distance, Some(WALL_DISTANCE_RANGE.0), "smallest distance is still valid (> 0)");
            assert_eq!(wp.right_wall_distance, Some(WALL_DISTANCE_RANGE.0));
        }
        assert!(
            !state.diagnostics.iter().any(|d| d.code == "ERR_INVALID_WALL_DISTANCE" || d.code == "ERR_TRACK_TOO_NARROW"),
            "range ends pass validation"
        );
    }
}

#[test]
fn wall_distance_is_set_per_side() {
    let mut state = selected(&[0]);
    let mut tools = ToolSettings::default();
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWallDistL, 3.0));
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::WpWallDistR, 6.0));
    let wp = &state.track.spline.waypoints[0];
    assert_eq!((wp.left_wall_distance, wp.right_wall_distance), (Some(3.0), Some(6.0)));
    assert_eq!(tools.new_waypoint_left_wall_distance, Some(3.0), "edits become placement defaults");
    assert_eq!(tools.new_waypoint_right_wall_distance, Some(6.0));
}

#[test]
fn surface_wall_type_and_side_flags_apply_to_every_selected_waypoint() {
    let mut state = selected(&[0, 1, 2]);
    let mut tools = ToolSettings::default();
    let gravel = SURFACES.iter().position(|&s| s == SurfaceType::PackedGravel).unwrap();
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::WpSurface, gravel));
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::WpWallType, 2));
    apply_edit(&mut state, &mut tools, Edit::Flag(Prop::WpLeftCurb, true));
    for i in 0..3 {
        let wp = &state.track.spline.waypoints[i];
        assert_eq!(wp.surface, Some(SurfaceType::PackedGravel));
        assert_eq!(wp.wall_type, Some(BarrierType::TireWall));
        assert!(wp.left_curb);
    }
    assert_eq!(tools.active_surface, SurfaceType::PackedGravel);
    let model = build_inspector(&state, &tools).unwrap();
    assert!(matches!(find_row(&model, "Surface"), Row::Dropdown { value: Common::Same(i), .. } if *i == gravel));
}

#[test]
fn footer_actions_act_on_the_whole_selection() {
    let mut state = selected(&[0, 1]);
    let mut tools = ToolSettings::default();
    let before = state.track.spline.waypoints.len();
    apply_edit(&mut state, &mut tools, Edit::Do(Action::Delete));
    assert_eq!(state.track.spline.waypoints.len(), before - 2);
}

fn with_selection(selection: Selection) -> EditorState {
    let mut state = fixture_state();
    state.select(selection);
    state
}

fn fits_720p(model: &InspectorModel) -> (f32, f32) {
    (content_height(model, |_| false), body_height(model, CARD_H_720P))
}

#[test]
fn every_single_entity_view_fits_the_720p_card_fully_expanded() {
    let tools = ToolSettings::default();
    for selection in [
        Selection::Waypoint(0),
        Selection::SurfaceZone(0),
        Selection::Obstacle(0),
        Selection::JumpRamp(0),
        Selection::Checkpoint(0),
        Selection::GridSlot(0),
        Selection::PitBox,
    ] {
        let model = build_inspector(&with_selection(selection.clone()), &tools).unwrap_or_else(|| panic!("{selection:?} uses the new inspector"));
        let (used, room) = fits_720p(&model);
        assert!(used <= room, "{selection:?}: content {used} px > body {room} px");
        assert!(model.footer.is_some(), "{selection:?} has the footer");
    }
}

#[test]
fn track_level_views_fit_for_every_tool_and_list_every_off_track_surface() {
    let state = with_selection(Selection::None);
    for tool in [EditorToolType::Select, EditorToolType::RoadSpline, EditorToolType::RoadSplit, EditorToolType::JumpRamp, EditorToolType::Obstacle] {
        let tools = ToolSettings { active_tool: tool, ..ToolSettings::default() };
        let model = build_inspector(&state, &tools).unwrap();
        let (used, room) = fits_720p(&model);
        assert!(used <= room, "{tool:?}: content {used} px > body {room} px");
        assert!(model.sections.iter().any(|s| s.id == "circuit"));
    }
    let labels: Vec<&str> = (0..Options::OffTrack.len()).map(|i| Options::OffTrack.label(i)).collect();
    for s in SurfaceType::OFF_TRACK_TYPES {
        assert!(labels.contains(&s.name()), "off-track list misses {}", s.name());
    }
    assert!(labels.contains(&"Sheet Ice"));
}

#[test]
fn grid_slot_view_is_not_empty() {
    let model = build_inspector(&with_selection(Selection::GridSlot(0)), &ToolSettings::default()).unwrap();
    assert_eq!(model.title, "Grid Slot #0");
    assert!(model.sections.iter().flat_map(|s| &s.rows).any(|r| matches!(r, Row::Info(t) if t.contains("position"))));
}

#[test]
fn zone_material_and_layer_apply_to_every_selected_zone() {
    let mut state = with_selection(Selection::SurfaceZone(1));
    let mut tools = ToolSettings::default();
    let mud = SURFACES.iter().position(|&s| s == SurfaceType::DeepMud).unwrap();
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::ZoneSurface, mud));
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::ZoneLayer, 1));
    let zone = &state.track.geometry.surface_zones[1];
    assert_eq!(zone.surface, SurfaceType::DeepMud);
    assert!(zone.is_above_track());
    assert_eq!(state.history.undo_count(), 2);
}

#[test]
fn ramp_edits_clamp_wrap_and_fit() {
    let mut state = with_selection(Selection::JumpRamp(0));
    let mut tools = ToolSettings::default();
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::RampLength, 500.0));
    assert!((state.track.geometry.jump_ramps[0].length() - RAMP_LENGTH_RANGE.1).abs() < 1e-3);
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::RampAngle, 350.0));
    apply_edit(&mut state, &mut tools, Edit::Step(Prop::RampAngle, 15.0));
    assert!((state.track.geometry.jump_ramps[0].angle_deg() - 5.0).abs() < 0.01, "angle wraps past 360");
    apply_edit(&mut state, &mut tools, Edit::Do(Action::FitPitch));
    let ramp = &state.track.geometry.jump_ramps[0];
    assert!((ramp.ramp_angle_deg - ramp.fitted_pitch_deg()).abs() < 1e-3);
}

#[test]
fn checkpoint_finish_line_toggle() {
    let mut state = with_selection(Selection::Checkpoint(3));
    let mut tools = ToolSettings::default();
    apply_edit(&mut state, &mut tools, Edit::Flag(Prop::CpFinish, true));
    let cp = state.track.checkpoints.iter().find(|c| c.id == 3).unwrap();
    assert!(cp.is_finish_line);
}

#[test]
fn track_level_edits_change_the_circuit() {
    let mut state = with_selection(Selection::None);
    let mut tools = ToolSettings::default();
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::GridSlots, 12.0));
    assert_eq!(state.grid_count(), 12);
    let ice = (0..Options::OffTrack.len()).find(|&i| Options::OffTrack.label(i) == "Sheet Ice").unwrap();
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::OffTrack, ice));
    assert_eq!(state.track.default_surface, SurfaceType::SheetIce);
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::Category, 3));
    assert_eq!(state.track.car_category, tdrace_core::CarCategory::Kart);
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::ToolWidth, 99.0));
    assert_eq!(tools.new_waypoint_width, WIDTH_RANGE.1);
}

fn multi(zones: Vec<usize>, ramps: Vec<usize>, obstacles: Vec<usize>) -> Selection {
    Selection::from_multi(vec![], zones, obstacles, ramps, vec![], vec![], false)
}

#[test]
fn several_zones_or_ramps_keep_their_controls_and_edit_both() {
    let tools = ToolSettings::default();
    for (single, many) in [(Selection::SurfaceZone(0), multi(vec![0, 1], vec![], vec![])), (Selection::JumpRamp(0), multi(vec![], vec![0, 1], vec![])), (Selection::Obstacle(0), multi(vec![], vec![], vec![0, 1, 2]))] {
        let one = build_inspector(&with_selection(single.clone()), &tools).unwrap();
        let both = build_inspector(&with_selection(many.clone()), &tools).unwrap_or_else(|| panic!("{many:?} uses the new inspector"));
        let sections = |m: &InspectorModel| m.sections.iter().map(|s| s.id).collect::<Vec<_>>();
        assert_eq!(sections(&one), sections(&both), "{single:?} vs {many:?}");
        assert!(both.count > 1);
    }

    let mut state = with_selection(multi(vec![0, 1], vec![], vec![]));
    let mut tools = ToolSettings::default();
    let model = build_inspector(&state, &tools).unwrap();
    assert!(matches!(find_row(&model, "Material"), Row::Dropdown { value: Common::Mixed, .. }), "fixture zones are gravel and water");
    let snow = SURFACES.iter().position(|&s| s == SurfaceType::PackedSnow).unwrap();
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::ZoneSurface, snow));
    assert!(state.track.geometry.surface_zones.iter().all(|z| z.surface == SurfaceType::PackedSnow));

    let mut state = with_selection(multi(vec![], vec![0, 1], vec![]));
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::RampHeight, 3.0));
    apply_edit(&mut state, &mut tools, Edit::Step(Prop::RampWidth, 1.0));
    for r in &state.track.geometry.jump_ramps {
        assert_eq!(r.height, 3.0);
        assert!((r.width() - 9.0).abs() < 1e-3);
    }
}

fn mixed_selection() -> Selection {
    Selection::from_multi(vec![0, 1], vec![0], vec![], vec![0], vec![], vec![], false)
}

#[test]
fn mixed_selection_reaches_every_kind_and_edits_only_that_kind() {
    use tdrace_app::editor::inspector::MixedKind;
    let state = with_selection(mixed_selection());
    let mut tools = ToolSettings::default();
    let b = build_inspector(&state, &tools).unwrap();
    assert_eq!(b.count, 4);
    match &b.sections[0].rows[0] {
        Row::Chips { chips, .. } => assert_eq!(chips.iter().map(|c| c.label.as_str()).collect::<Vec<_>>(), ["Waypoints 2", "Zones 1", "Ramps 1"]),
        _ => unreachable!(),
    }
    assert_eq!(b.sections[0].id, "mixed.kinds");
    assert!(b.sections.iter().any(|s| s.id == "wp.road"), "first kind shown by default");
    let mut state_b = with_selection(mixed_selection());
    apply_edit(&mut state_b, &mut tools, Edit::Do(Action::ShowKind(MixedKind::Ramps)));
    let b = build_inspector(&state_b, &tools).unwrap();
    assert!(b.sections.iter().any(|s| s.id == "ramp.shape") && !b.sections.iter().any(|s| s.id == "wp.road"));

    // An edit from one kind's control leaves the other kinds alone.
    let mut state = state;
    let zone_before = state.track.geometry.surface_zones[0].surface;
    let ramp_before = state.track.geometry.jump_ramps[0].surface;
    let gravel = SURFACES.iter().position(|&s| s == SurfaceType::PackedGravel).unwrap();
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::WpSurface, gravel));
    assert_eq!(state.track.spline.waypoints[0].surface, Some(SurfaceType::PackedGravel));
    assert_eq!(state.track.spline.waypoints[1].surface, Some(SurfaceType::PackedGravel));
    assert_eq!(state.track.geometry.surface_zones[0].surface, zone_before);
    assert_eq!(state.track.geometry.jump_ramps[0].surface, ramp_before);
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::RampHeight, 2.5));
    assert_eq!(state.track.geometry.jump_ramps[0].height, 2.5);
    assert_eq!(state.track.spline.waypoints[2].surface, Some(SurfaceType::Concrete), "unselected waypoint untouched");
}

#[test]
fn batch_tools_share_the_inspector_ranges() {
    let mut state = selected(&[0, 1, 2]);
    let mut tools = ToolSettings::default();
    tools.batch_set_width(&mut state, 999.0);
    assert!(state.track.spline.waypoints[..3].iter().all(|w| w.width == WIDTH_RANGE.1));
    tools.batch_adjust_width(&mut state, -999.0);
    assert!(state.track.spline.waypoints[..3].iter().all(|w| w.width == WIDTH_RANGE.0));
    tools.batch_adjust_wall_distances(&mut state, -999.0);
    assert!(state.track.spline.waypoints[..3].iter().all(|w| w.left_wall_distance == Some(WALL_DISTANCE_RANGE.0)));
}

#[test]
fn tooltips_name_the_control_and_its_shortcut() {
    use tdrace_app::editor::inspector::row_tooltip;
    let wp = build_inspector(&selected(&[0]), &ToolSettings::default()).unwrap();
    assert!(row_tooltip(find_row(&wp, "Angle")).unwrap().contains("[ / ]"));
    assert!(row_tooltip(find_row(&wp, "Presets")).unwrap().contains("Shift+B"));
    assert!(row_tooltip(find_row(&wp, "Width")).unwrap().contains("click to type"));
    let ramp = build_inspector(&with_selection(Selection::JumpRamp(0)), &ToolSettings::default()).unwrap();
    assert!(row_tooltip(find_row(&ramp, "Angle")).unwrap().contains("R / Shift+R"));
    let zone = build_inspector(&with_selection(Selection::SurfaceZone(0)), &ToolSettings::default()).unwrap();
    assert!(row_tooltip(find_row(&zone, "Layer")).unwrap().contains("Ctrl+F"));
}

#[test]
fn slider_drag_keeps_working_off_the_bar_and_only_starts_from_a_press_on_it() {
    use tdrace_app::editor::inspector::{update_stepper_drag, DragOutcome, StepperDrag};
    // Bar from x=100 to x=250 (150 px), range 4-50 m, step 0.5.
    let range = (4.0, 50.0, 0.5);
    let mut drag = StepperDrag::press(Prop::WpWidth, true, 120.0, 12.0);
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, true, 121.0, 150.0, range, 3.0, false), DragOutcome::None, "inside the slop");
    // Far right of the bar, outside it: the value follows to the maximum.
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, true, 600.0, 150.0, range, 3.0, false), DragOutcome::Set(50.0));
    // Back to the left of the bar, still held: the value follows down.
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, true, 40.0, 150.0, range, 3.0, false), DragOutcome::Set(4.0));
    // Release off the bar ends the drag without opening text entry.
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, false, 40.0, 150.0, range, 3.0, false), DragOutcome::None);
    assert!(drag.is_none());

    // A press without a drag opens text entry on release.
    let mut drag = StepperDrag::press(Prop::WpWidth, true, 120.0, 12.0);
    update_stepper_drag(&mut drag, Prop::WpWidth, true, 120.0, 150.0, range, 3.0, false);
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, false, 120.0, 150.0, range, 3.0, false), DragOutcome::OpenTextEntry);

    // A press outside the bar, then moving onto it while held, never starts a drag.
    let mut drag = StepperDrag::press(Prop::WpWidth, false, 20.0, 12.0);
    assert!(drag.is_none());
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpWidth, true, 150.0, 150.0, range, 3.0, false), DragOutcome::None);

    // A drag captured by one stepper is ignored by the others.
    let mut drag = StepperDrag::press(Prop::WpWidth, true, 120.0, 12.0);
    assert_eq!(update_stepper_drag(&mut drag, Prop::WpBanking, true, 600.0, 150.0, (-45.0, 45.0, 1.0), 3.0, false), DragOutcome::None);
}
