//! Track Studio inspector interaction state.
//! Governed by specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use std::collections::HashSet;

use cabinet::ui::FieldDropdown;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::BarrierType;

use super::state::{EditorState, Selection};
use super::tools::ToolSettings;

/// Delay before a held step button starts repeating, in seconds.
pub const HOLD_REPEAT_DELAY: f64 = 0.4;
/// Interval between repeated steps while a step button is held, in seconds (10 Hz).
pub const HOLD_REPEAT_INTERVAL: f64 = 0.1;

/// Hold-to-repeat timer for step buttons: one step on press, then repeats after a delay.
#[derive(Debug, Clone, Default)]
pub struct HoldRepeat {
    held: Option<String>,
    next_step_at: f64,
}

impl HoldRepeat {
    /// Returns true when the button `id` must step this frame.
    /// `pressed` is the press edge, `down` the held state, `over` whether the pointer is on the button.
    pub fn poll(&mut self, id: &str, pressed: bool, down: bool, over: bool, now: f64) -> bool {
        if pressed && over {
            self.held = Some(id.to_string());
            self.next_step_at = now + HOLD_REPEAT_DELAY;
            return true;
        }
        if self.held.as_deref() != Some(id) {
            return false;
        }
        if !down {
            self.held = None;
            return false;
        }
        if over && now >= self.next_step_at {
            // Fixed schedule, so frame timing does not slow the rate; after a stall, never burst.
            self.next_step_at += HOLD_REPEAT_INTERVAL;
            if self.next_step_at <= now {
                self.next_step_at = now + HOLD_REPEAT_INTERVAL;
            }
            return true;
        }
        false
    }
}

/// A stepper drag in progress: relative to where the press started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepperDrag {
    pub prop: Prop,
    pub start_x: f32,
    pub start_value: f32,
    /// The pointer moved far enough to count as a drag; a press without a drag opens text entry.
    pub moved: bool,
    pub last_value: f32,
}

/// Per-session interaction state of the inspector panel.
#[derive(Debug, Clone, Default)]
pub struct InspectorView {
    /// The pointer is over the inspector card this frame (and no modal is open).
    pub hovered: bool,
    /// Slider that owns the current mouse drag, from press to release.
    pub drag_capture: Option<String>,
    pub hold: HoldRepeat,
    /// Selection seen on the previous frame; a change clears control focus.
    pub last_selection: Option<Selection>,
    /// Body scroll offset in reference pixels; reset when the selection kind changes.
    pub scroll: f32,
    /// Sections the user opened or closed, against their default state.
    pub toggled_sections: HashSet<&'static str>,
    pub dropdown: FieldDropdown,
    /// Property of the open dropdown and its field rectangle in screen pixels.
    pub dropdown_prop: Option<Prop>,
    pub dropdown_field: (f32, f32, f32, f32),
    pub stepper_drag: Option<StepperDrag>,
}

impl InspectorView {
    pub fn is_collapsed(&self, section: &Section) -> bool {
        section.collapsed_by_default != self.toggled_sections.contains(section.id)
    }

    pub fn toggle_section(&mut self, id: &'static str) {
        if !self.toggled_sections.remove(id) {
            self.toggled_sections.insert(id);
        }
    }

    pub fn dropdown_open(&self) -> bool {
        self.dropdown.is_open
    }

    pub fn close_dropdown(&mut self) {
        self.dropdown.close();
        self.dropdown_prop = None;
    }
}

/// Per-frame inspector bookkeeping, run before the inspector is drawn.
/// - One press-to-release that starts on the inspector is one undo step.
/// - A click clears control focus; the control under the click claims it again while drawing.
/// - A new selection clears focus and any inline value edit.
pub fn begin_inspector_frame(state: &mut EditorState, tools: &mut ToolSettings, hovered: bool, pressed: bool, down: bool) {
    tools.inspector.hovered = hovered;
    if !down {
        state.end_undo_gesture();
        tools.inspector.drag_capture = None;
    }
    if pressed {
        tools.clear_bar_selection();
        if hovered {
            state.begin_undo_gesture();
        }
    }
    if tools.inspector.last_selection.as_ref() != Some(&state.selection) {
        let same_kind = tools.inspector.last_selection.as_ref().map(std::mem::discriminant) == Some(std::mem::discriminant(&state.selection));
        if !same_kind {
            tools.inspector.scroll = 0.0;
        }
        tools.clear_bar_selection();
        tools.stop_editing_bar();
        tools.inspector.close_dropdown();
        tools.inspector.stepper_drag = None;
        tools.inspector.last_selection = Some(state.selection.clone());
    }
}

// ---------------------------------------------------------------------------------------------
// Inspector model: what the inspector shows for the current selection, built without a window.
// ---------------------------------------------------------------------------------------------

/// Road width range in metres; `validate_track` rejects < 4 m and warns above 50 m.
pub const WIDTH_RANGE: (f32, f32) = (4.0, 50.0);
/// Banking range in degrees.
pub const BANKING_RANGE: (f32, f32) = (-45.0, 45.0);
/// Wall distance range in metres; `validate_track` rejects a distance that is not > 0.
pub const WALL_DISTANCE_RANGE: (f32, f32) = (0.5, 25.0);

/// The one surface catalogue every inspector surface picker uses (order and names).
pub const SURFACES: [SurfaceType; 14] = [
    SurfaceType::Asphalt,
    SurfaceType::Concrete,
    SurfaceType::Dirt,
    SurfaceType::Gravel,
    SurfaceType::PackedSand,
    SurfaceType::DeepSand,
    SurfaceType::MudTrack,
    SurfaceType::DeepMud,
    SurfaceType::PackedSnow,
    SurfaceType::DeepSnow,
    SurfaceType::SheetIce,
    SurfaceType::Grass,
    SurfaceType::Water,
    SurfaceType::Oil,
];

pub const WALL_TYPES: [BarrierType; 3] = [BarrierType::Concrete, BarrierType::Steel, BarrierType::TireWall];
pub const WALL_TYPE_LABELS: [&str; 3] = ["Concrete", "Steel", "Tyres"];

/// Banking preset chips, in degrees.
pub const BANKING_PRESETS: [f32; 4] = [0.0, 10.0, 18.0, 22.0];

/// One property read across every selected entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Common<T> {
    Same(T),
    Mixed,
    Empty,
}

impl<T: Copy> Common<T> {
    pub fn same(self) -> Option<T> {
        match self {
            Common::Same(v) => Some(v),
            _ => None,
        }
    }
}

/// Reads one property across the selection: the shared value, `Mixed`, or `Empty`.
pub fn common_value<T: PartialEq + Copy>(values: impl IntoIterator<Item = T>) -> Common<T> {
    let mut iter = values.into_iter();
    let Some(first) = iter.next() else {
        return Common::Empty;
    };
    if iter.all(|v| v == first) {
        Common::Same(first)
    } else {
        Common::Mixed
    }
}

/// An inspector-editable property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prop {
    WpWidth,
    WpBanking,
    WpLeftCurb,
    WpRightCurb,
    WpLeftWall,
    WpRightWall,
    WpWallDistL,
    WpWallDistR,
    WpWallType,
    WpSurface,
}

impl Prop {
    /// Stable control id, used for focus, drag capture and inline text editing.
    pub fn id(self) -> String {
        format!("insp:{self:?}")
    }
}

/// A one-shot inspector action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    InvertBanking,
    Duplicate,
    Delete,
}

/// One user edit produced by an inspector control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Edit {
    /// Absolute value for every selected entity.
    Set(Prop, f32),
    /// Relative change applied to each selected entity, keeping their differences.
    Step(Prop, f32),
    Flag(Prop, bool),
    /// Option index for dropdowns and segmented controls.
    Pick(Prop, usize),
    Do(Action),
}

/// A preset chip in a chip row.
#[derive(Debug, Clone, PartialEq)]
pub struct Chip {
    pub label: String,
    pub edit: Edit,
    pub active: bool,
}

/// One control row of the inspector.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    /// `[-] bar [+]` with drag, click-to-type, focused wheel and hold-to-repeat.
    /// `anchor` is the value a drag starts from when the selection is mixed.
    Stepper { prop: Prop, label: &'static str, value: Common<f32>, anchor: f32, min: f32, max: f32, step: f32, unit: &'static str, signed: bool },
    /// Surface dropdown over [`SURFACES`].
    Surface { prop: Prop, label: &'static str, value: Common<usize> },
    Segmented { prop: Prop, label: &'static str, labels: &'static [&'static str], value: Common<usize> },
    /// Left and right on/off toggles on one row.
    Sides { label: &'static str, left: (Prop, Common<bool>), right: (Prop, Common<bool>) },
    Chips { label: &'static str, chips: Vec<Chip> },
    Info(String),
}

/// A collapsible group of rows.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub id: &'static str,
    pub title: &'static str,
    /// Draws "L" and "R" column labels in the section header, above [`Row::Sides`] toggles.
    pub side_columns: bool,
    pub collapsed_by_default: bool,
    pub rows: Vec<Row>,
}

/// Everything the inspector draws for the current selection.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorModel {
    pub title: String,
    /// Number of selected entities; the header shows "N selected" when above 1.
    pub count: usize,
    pub sections: Vec<Section>,
    /// Shows the fixed Duplicate / Delete footer.
    pub footer: bool,
}

// Layout metrics in 720p reference pixels (scale with `UiScaler::s`).
pub const TITLE_H: f32 = 30.0;
pub const HEADER_H: f32 = 26.0;
pub const SECTION_HEADER_H: f32 = 20.0;
pub const SECTION_GAP: f32 = 6.0;
pub const ROW_H: f32 = 22.0;
pub const ROW_GAP: f32 = 4.0;
pub const INFO_H: f32 = 18.0;
pub const FOOTER_H: f32 = 38.0;
pub const BODY_PAD: f32 = 8.0;

/// Reference-pixel height of one row, including its gap.
pub fn row_height(row: &Row) -> f32 {
    match row {
        Row::Info(_) => INFO_H,
        _ => ROW_H + ROW_GAP,
    }
}

/// Reference-pixel height of the scrollable body content for `model` with the given collapsed sections.
pub fn content_height(model: &InspectorModel, is_collapsed: impl Fn(&Section) -> bool) -> f32 {
    let mut h = BODY_PAD;
    for section in &model.sections {
        h += SECTION_GAP + SECTION_HEADER_H;
        if !is_collapsed(section) {
            h += section.rows.iter().map(row_height).sum::<f32>();
        }
    }
    h + BODY_PAD
}

/// Reference-pixel height of the scrollable body inside an inspector card of `card_h`.
pub fn body_height(model: &InspectorModel, card_h: f32) -> f32 {
    card_h - TITLE_H - HEADER_H - if model.footer { FOOTER_H } else { 0.0 }
}

fn surface_index(s: SurfaceType) -> usize {
    SURFACES.iter().position(|&x| x == s).unwrap_or(0)
}

fn wall_type_index(t: BarrierType) -> usize {
    WALL_TYPES.iter().position(|&x| x == t).unwrap_or(0)
}

/// Builds the inspector model for the current selection, or `None` for selections still drawn
/// by the legacy inspector.
pub fn build_inspector(state: &EditorState) -> Option<InspectorModel> {
    match &state.selection {
        Selection::Waypoint(_) | Selection::MultipleWaypoints(_) => build_waypoints(state),
        _ => None,
    }
}

fn build_waypoints(state: &EditorState) -> Option<InspectorModel> {
    let wps = &state.track.spline.waypoints;
    let indices: Vec<usize> = state.selection.selected_waypoint_indices().into_iter().filter(|&i| i < wps.len()).collect();
    let first = *indices.first()?;
    let sel = || indices.iter().map(|&i| &wps[i]);
    let offset = state.barrier_offset;

    let width = common_value(sel().map(|w| w.width));
    let banking = common_value(sel().map(|w| w.bank_angle));
    let dist_l = common_value(sel().map(|w| w.left_wall_distance.unwrap_or(offset)));
    let dist_r = common_value(sel().map(|w| w.right_wall_distance.unwrap_or(offset)));
    let wall_type = common_value(sel().map(|w| wall_type_index(w.wall_type.unwrap_or(state.barrier_type))));
    let surface = common_value(sel().map(|w| surface_index(w.surface.unwrap_or(SurfaceType::Asphalt))));

    let mut chips: Vec<Chip> = BANKING_PRESETS
        .iter()
        .map(|&deg| Chip {
            label: format!("{deg:.0}°"),
            edit: Edit::Set(Prop::WpBanking, deg),
            active: banking.same().is_some_and(|b| (b - deg).abs() < 0.05),
        })
        .collect();
    chips.push(Chip { label: "±".to_string(), edit: Edit::Do(Action::InvertBanking), active: false });

    let single = indices.len() == 1;
    let info = if single {
        let p = wps[first].point;
        vec![Row::Info(format!("Position ({:.1}, {:.1})", p.x, p.y))]
    } else {
        let list: Vec<String> = indices.iter().map(|i| format!("#{i}")).collect();
        vec![Row::Info(format!("Waypoints {}", list.join(", ")))]
    };

    Some(InspectorModel {
        title: if single { format!("Waypoint #{first}") } else { "Waypoints".to_string() },
        count: indices.len(),
        footer: true,
        sections: vec![
            Section {
                id: "wp.road",
                title: "ROAD",
                side_columns: false,
                collapsed_by_default: false,
                rows: vec![
                    Row::Stepper { prop: Prop::WpWidth, label: "Width", value: width, anchor: wps[first].width, min: WIDTH_RANGE.0, max: WIDTH_RANGE.1, step: 0.5, unit: "m", signed: false },
                    Row::Surface { prop: Prop::WpSurface, label: "Surface", value: surface },
                ],
            },
            Section {
                id: "wp.banking",
                title: "BANKING",
                side_columns: false,
                collapsed_by_default: false,
                rows: vec![
                    Row::Stepper { prop: Prop::WpBanking, label: "Angle", value: banking, anchor: wps[first].bank_angle, min: BANKING_RANGE.0, max: BANKING_RANGE.1, step: 1.0, unit: "°", signed: true },
                    Row::Chips { label: "Presets", chips },
                ],
            },
            Section {
                id: "wp.edges",
                title: "EDGES",
                side_columns: true,
                collapsed_by_default: false,
                rows: vec![
                    Row::Sides {
                        label: "Curb",
                        left: (Prop::WpLeftCurb, common_value(sel().map(|w| w.left_curb))),
                        right: (Prop::WpRightCurb, common_value(sel().map(|w| w.right_curb))),
                    },
                    Row::Sides {
                        label: "Wall",
                        left: (Prop::WpLeftWall, common_value(sel().map(|w| w.left_wall))),
                        right: (Prop::WpRightWall, common_value(sel().map(|w| w.right_wall))),
                    },
                    Row::Stepper { prop: Prop::WpWallDistL, label: "Dist L", value: dist_l, anchor: wps[first].left_wall_distance.unwrap_or(offset), min: WALL_DISTANCE_RANGE.0, max: WALL_DISTANCE_RANGE.1, step: 0.5, unit: "m", signed: false },
                    Row::Stepper { prop: Prop::WpWallDistR, label: "Dist R", value: dist_r, anchor: wps[first].right_wall_distance.unwrap_or(offset), min: WALL_DISTANCE_RANGE.0, max: WALL_DISTANCE_RANGE.1, step: 0.5, unit: "m", signed: false },
                    Row::Segmented { prop: Prop::WpWallType, label: "Wall type", labels: &WALL_TYPE_LABELS, value: wall_type },
                ],
            },
            Section { id: "wp.info", title: "INFO", side_columns: false, collapsed_by_default: true, rows: info },
        ],
    })
}

/// Applies one inspector edit to every selected entity, as one undo step.
pub fn apply_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    match edit {
        Edit::Do(Action::Duplicate) => {
            tools.duplicate_selected(state);
        }
        Edit::Do(Action::Delete) => {
            tools.delete_selected(state);
        }
        _ => apply_waypoint_edit(state, tools, edit),
    }
}

fn apply_waypoint_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    let len = state.track.spline.waypoints.len();
    let indices: Vec<usize> = state.selection.selected_waypoint_indices().into_iter().filter(|&i| i < len).collect();
    let Some(&first) = indices.first() else {
        return;
    };
    state.record_undo();
    let offset = state.barrier_offset;
    let clamp = |v: f32, (lo, hi): (f32, f32)| if v.is_finite() { v.clamp(lo, hi) } else { lo };
    for &i in &indices {
        let wp = &mut state.track.spline.waypoints[i];
        match edit {
            Edit::Set(Prop::WpWidth, v) => wp.width = clamp(v, WIDTH_RANGE),
            Edit::Step(Prop::WpWidth, d) => wp.width = clamp(wp.width + d, WIDTH_RANGE),
            Edit::Set(Prop::WpBanking, v) => wp.bank_angle = clamp(v, BANKING_RANGE),
            Edit::Step(Prop::WpBanking, d) => wp.bank_angle = clamp(wp.bank_angle + d, BANKING_RANGE),
            Edit::Do(Action::InvertBanking) => wp.bank_angle = -wp.bank_angle,
            Edit::Set(Prop::WpWallDistL, v) => wp.left_wall_distance = Some(clamp(v, WALL_DISTANCE_RANGE)),
            Edit::Step(Prop::WpWallDistL, d) => wp.left_wall_distance = Some(clamp(wp.left_wall_distance.unwrap_or(offset) + d, WALL_DISTANCE_RANGE)),
            Edit::Set(Prop::WpWallDistR, v) => wp.right_wall_distance = Some(clamp(v, WALL_DISTANCE_RANGE)),
            Edit::Step(Prop::WpWallDistR, d) => wp.right_wall_distance = Some(clamp(wp.right_wall_distance.unwrap_or(offset) + d, WALL_DISTANCE_RANGE)),
            Edit::Flag(Prop::WpLeftCurb, on) => wp.left_curb = on,
            Edit::Flag(Prop::WpRightCurb, on) => wp.right_curb = on,
            Edit::Flag(Prop::WpLeftWall, on) => wp.left_wall = on,
            Edit::Flag(Prop::WpRightWall, on) => wp.right_wall = on,
            Edit::Pick(Prop::WpWallType, k) => wp.wall_type = WALL_TYPES.get(k).copied(),
            Edit::Pick(Prop::WpSurface, k) => wp.surface = SURFACES.get(k).copied(),
            _ => {}
        }
    }

    // Edits also become the placement defaults for new waypoints, as before.
    let wp = state.track.spline.waypoints[first].clone();
    match edit {
        Edit::Set(Prop::WpWidth, _) | Edit::Step(Prop::WpWidth, _) => tools.new_waypoint_width = wp.width,
        Edit::Set(Prop::WpBanking, _) | Edit::Step(Prop::WpBanking, _) | Edit::Do(Action::InvertBanking) => tools.new_waypoint_bank_angle = wp.bank_angle,
        Edit::Set(Prop::WpWallDistL, _) | Edit::Step(Prop::WpWallDistL, _) => tools.new_waypoint_left_wall_distance = wp.left_wall_distance,
        Edit::Set(Prop::WpWallDistR, _) | Edit::Step(Prop::WpWallDistR, _) => tools.new_waypoint_right_wall_distance = wp.right_wall_distance,
        Edit::Flag(Prop::WpLeftCurb, on) => tools.new_waypoint_left_curb = on,
        Edit::Flag(Prop::WpRightCurb, on) => tools.new_waypoint_right_curb = on,
        Edit::Flag(Prop::WpLeftWall, on) => tools.new_waypoint_left_wall = on,
        Edit::Flag(Prop::WpRightWall, on) => tools.new_waypoint_right_wall = on,
        Edit::Pick(Prop::WpWallType, _) => tools.new_waypoint_wall_type = wp.wall_type,
        Edit::Pick(Prop::WpSurface, _) => {
            if let Some(s) = wp.surface {
                tools.active_surface = s;
            }
        }
        _ => {}
    }
    state.rebuild_geometry();
}
