//! Spec 103 (`specs/103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md`): the Track Studio
//! launch chute stamp tool and its inspector controls.

use std::fs;

use tdrace_app::editor::inspector::{apply_edit, build_inspector, Action, Common, Edit, Prop, Row};
use tdrace_app::editor::{EditorState, EditorToolType, ToolSettings};
use tdrace_app::tracks::UserTrackStore;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::{ChuteSide, LaunchChuteSpec, PackedGridPattern, Track, TrackCategory};

/// The Track Studio on an official circuit as it was before its launch chute: the catalog circuits already have one.
fn editor(module: &str, slug: &str) -> EditorState {
    let mut track = tdrace_core::catalog::official_track(module, slug);
    track.remove_launch_chute();
    EditorState::new(track)
}

fn error_codes(track: &Track) -> Vec<&'static str> {
    validate_track(track).into_iter().filter(|d| d.severity == ValidationSeverity::Error).map(|d| d.code).collect()
}

/// A waypoint a chute fits at (found the way the insert button finds it), and where it is.
fn merge_candidate(state: &EditorState) -> (usize, glam::Vec2) {
    let spec = state
        .track
        .clone()
        .place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Right))
        .expect("a circuit has a waypoint before its finish line that a chute fits at");
    (spec.merge_waypoint, state.track.spline.waypoints[spec.merge_waypoint].point)
}

/// Scenario: Track Studio Launch Chute Insertion
///
/// Given the Track Studio editor active on an official circuit
/// When the user selects a target waypoint with the Launch Chute tool
/// Then a templated launch spur segment, walled end cap and 8 packed grid slots are appended
#[test]
fn test_clicking_a_waypoint_stamps_a_launch_chute() {
    let mut state = editor("autocross", "matschenberg_ax");
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::LaunchChute;
    assert!(state.track.launch_chute().is_none());
    let segments_before = state.track.active_network().segments.len();

    let (k, at) = merge_candidate(&state);
    tools.handle_secondary_down(&mut state, at);

    let net = state.track.network.as_ref().expect("the circuit has a network now");
    let chute = net.launch_chute.as_ref().expect("launch chute config");
    assert!(net.validate().is_ok(), "{:?}", net.validate());
    assert!(net.segments.len() > segments_before, "a spur segment was appended");
    assert_eq!(net.layouts[0].entry_segment, Some(chute.segment_id));
    assert!(chute.terminal_barrier.segment.length() >= chute.pad_width, "walled end cap");
    assert!(!chute.side_barriers.is_empty());
    assert_eq!(chute.grid_slots.len(), 8, "8 packed grid slots");
    assert_eq!(state.track.grid_positions, chute.grid_slots);
    assert_eq!(state.track.launch_chute_spec().unwrap().merge_waypoint, k);
    assert_eq!(error_codes(&state.track), Vec::<&str>::new());
    assert!(state.is_dirty);
    assert!(tools.chute_status.as_deref().is_some_and(|s| s.contains("placed")));
}

/// A left click on a waypoint stamps too, while the tool is active.
#[test]
fn test_left_click_with_the_chute_tool_stamps_and_elsewhere_selects() {
    let mut state = editor("autocross", "nova_paka_ax");
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::LaunchChute;

    let (_, at) = merge_candidate(&state);
    tools.handle_primary_down(&mut state, at, false);
    tools.handle_primary_up(&mut state, at);
    assert!(state.track.launch_chute().is_some());
}

/// The `[ + INSERT LAUNCH CHUTE ]` button picks the waypoint itself.
#[test]
fn test_insert_button_places_a_chute_and_undo_takes_it_back() {
    let mut state = editor("rally", "holjes_rx");
    let mut tools = ToolSettings::default();
    let before = state.track.clone();

    assert!(tools.auto_insert_launch_chute(&mut state));
    assert!(state.track.launch_chute().is_some());
    assert_eq!(error_codes(&state.track), Vec::<&str>::new());
    assert!(state.diagnostics.iter().all(|d| d.severity != ValidationSeverity::Error));

    assert!(state.undo());
    assert!(state.track.launch_chute().is_none());
    assert_eq!(state.track, before, "undo restores the circuit exactly");
    assert!(state.redo());
    assert!(state.track.launch_chute().is_some());
}

/// Clicks that cannot place a chute say why and change nothing.
#[test]
fn test_refused_clicks_leave_the_circuit_and_explain() {
    let mut state = editor("autocross", "matschenberg_ax");
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::LaunchChute;
    let before = state.track.clone();

    // Far from any waypoint.
    tools.handle_secondary_down(&mut state, glam::Vec2::new(5000.0, 5000.0));
    assert_eq!(tools.chute_status.as_deref(), Some("Click a waypoint of the circuit."));

    // A waypoint that is not shortly before the finish line.
    let far = (0..state.track.spline.waypoints.len()).find(|&k| !state.track.can_merge_launch_chute_at(k)).unwrap();
    let far_point = state.track.spline.waypoints[far].point;
    tools.handle_secondary_down(&mut state, far_point);
    assert!(tools.chute_status.as_deref().is_some_and(|s| s.contains("before the start/finish line")), "{:?}", tools.chute_status);

    assert_eq!(state.track, before);
    assert!(!state.history.can_undo(), "a refused click makes no undo step");
}

/// Saving the circuit keeps the chute and a valid network.
#[test]
fn test_saved_circuit_keeps_its_launch_chute() {
    let mut state = editor("autocross", "matschenberg_ax");
    let mut tools = ToolSettings::default();
    assert!(tools.auto_insert_launch_chute(&mut state));

    let dir = std::env::temp_dir().join(format!(
        "tdrace_test_launch_chute_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let store = UserTrackStore::new(&dir);
    let mut track = state.track.clone();
    track.name = "Chute Test".to_string();
    track.category = TrackCategory::Draft;
    store.save_track(&track, "chute_test", false).expect("save");
    let loaded = store.load_track("chute_test").expect("load");
    let _ = fs::remove_dir_all(&dir);

    assert_eq!(loaded.launch_chute(), state.track.launch_chute(), "LaunchChuteConfig survives the round trip");
    let net = loaded.network.as_ref().unwrap();
    assert!(net.validate().is_ok());
    assert_eq!(net.layouts[0].entry_segment, state.track.network.as_ref().unwrap().layouts[0].entry_segment);
    assert_eq!(loaded.grid_positions, state.track.grid_positions);
    assert_eq!(error_codes(&loaded), Vec::<&str>::new());
}

fn row<'a>(model: &'a tdrace_app::editor::inspector::InspectorModel, section: &str, label: &str) -> &'a Row {
    let section = model.sections.iter().find(|s| s.id == section).unwrap_or_else(|| panic!("no section {section}"));
    section
        .rows
        .iter()
        .find(|r| match r {
            Row::Stepper { label: l, .. } | Row::Segmented { label: l, .. } => *l == label,
            _ => false,
        })
        .unwrap_or_else(|| panic!("no row {label}"))
}

/// The inspector offers the pad width, grid pattern and surface of the stamp tool, and of a chute on the circuit.
#[test]
fn test_inspector_controls_edit_the_chute() {
    let mut state = editor("rally", "holjes_rx");
    let mut tools = ToolSettings::default();
    tools.active_tool = EditorToolType::LaunchChute;

    // Before a chute exists the tool section shows the settings for the next one.
    let model = build_inspector(&state, &tools).unwrap();
    assert!(matches!(row(&model, "tool.chute", "Pad width"), Row::Stepper { min, max, .. } if (*min, *max) == (14.0, 18.0)));

    apply_edit(&mut state, &mut tools, Edit::Do(Action::InsertLaunchChute));
    assert!(state.track.launch_chute().is_some());
    assert_eq!(state.track.launch_chute().unwrap().grid_slots.len(), 8);

    // Pad width: the chute is built again, 18 m wide; an out-of-range value is clamped to the range.
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::ChutePadWidth, 18.0));
    assert_eq!(state.track.launch_chute().unwrap().pad_width, 18.0);
    apply_edit(&mut state, &mut tools, Edit::Set(Prop::ChutePadWidth, 40.0));
    assert_eq!(state.track.launch_chute().unwrap().pad_width, 18.0);
    apply_edit(&mut state, &mut tools, Edit::Step(Prop::ChutePadWidth, -1.0));
    assert_eq!(state.track.launch_chute().unwrap().pad_width, 17.0);

    // Grid pattern and surface.
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::ChutePattern, 1));
    apply_edit(&mut state, &mut tools, Edit::Pick(Prop::ChuteSurface, 1));
    let spec = state.track.launch_chute_spec().expect("chute read back");
    assert_eq!(spec.pattern, PackedGridPattern::RallycrossThreeTwoThree);
    assert_eq!(spec.surface, SurfaceType::Asphalt);
    assert_eq!(spec.pad_width, 17.0, "editing one control keeps the others");
    assert_eq!(state.track.grid_positions.len(), 8);
    assert_eq!(error_codes(&state.track), Vec::<&str>::new());

    // With another tool the chute controls are in the circuit view and show the chute's own values.
    tools.active_tool = EditorToolType::Select;
    let model = build_inspector(&state, &tools).unwrap();
    assert!(matches!(row(&model, "circuit.chute", "Pad width"), Row::Stepper { value: Common::Same(w), .. } if (*w - 17.0).abs() < 1e-4));
    assert!(matches!(row(&model, "circuit.chute", "Surface"), Row::Segmented { value: Common::Same(1), .. }));
    assert!(matches!(row(&model, "circuit.chute", "Grid"), Row::Segmented { value: Common::Same(1), .. }));

    // Remove: the circuit is a plain one again, with its standard grid.
    apply_edit(&mut state, &mut tools, Edit::Do(Action::RemoveLaunchChute));
    assert!(state.track.launch_chute().is_none());
    assert!(state.track.network.as_ref().is_none_or(|n| n.launch_chute.is_none()));
    assert!(state.track.grid_positions.len() >= 10);
    assert_eq!(error_codes(&state.track), Vec::<&str>::new());
}

/// The circuit view does not offer a grid size on a circuit whose chute holds the grid.
#[test]
fn test_circuit_view_shows_the_chute_grid_instead_of_a_grid_stepper() {
    let mut state = editor("autocross", "matschenberg_ax");
    let mut tools = ToolSettings::default();
    let model = build_inspector(&state, &tools).unwrap();
    assert!(model.sections.iter().any(|s| s.rows.iter().any(|r| matches!(r, Row::Stepper { prop: Prop::GridSlots, .. }))));

    assert!(tools.auto_insert_launch_chute(&mut state));
    let model = build_inspector(&state, &tools).unwrap();
    assert!(!model.sections.iter().any(|s| s.rows.iter().any(|r| matches!(r, Row::Stepper { prop: Prop::GridSlots, .. }))));
    assert!(model.sections.iter().any(|s| s.id == "circuit.chute"));
}

/// The tool is in the palette with its shortcut.
#[test]
fn test_launch_chute_tool_is_in_the_palette() {
    assert!(EditorToolType::ALL.contains(&EditorToolType::LaunchChute));
    assert_eq!(EditorToolType::LaunchChute.shortcut(), "L");
    assert!(EditorToolType::LaunchChute.title().contains("Launch Chute"));
}
