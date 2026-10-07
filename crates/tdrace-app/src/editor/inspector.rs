//! Track Studio inspector interaction state.
//! Governed by specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use std::collections::HashSet;

use cabinet::ui::FieldDropdown;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, ObstacleShape, SurfaceLayer, SurfaceShape};
use tdrace_core::CarCategory;

use super::state::{EditorState, Selection};
use super::tools::{EditorToolType, ToolSettings};

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
/// Circuit-wide default wall offset in metres (0 = walls on the road edge).
pub const GLOBAL_WALL_OFFSET_RANGE: (f32, f32) = (0.0, 25.0);
pub const RAMP_ANGLE_RANGE: (f32, f32) = (0.0, 360.0);
pub const RAMP_LENGTH_RANGE: (f32, f32) = (2.0, 50.0);
pub const RAMP_WIDTH_RANGE: (f32, f32) = (1.0, 30.0);
pub const RAMP_HEIGHT_RANGE: (f32, f32) = (0.2, 10.0);
pub const RAMP_PITCH_RANGE: (f32, f32) = (1.0, 60.0);
pub const GRID_SLOTS_RANGE: (f32, f32) = (1.0, 24.0);

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
pub const LAYER_LABELS: [&str; 2] = ["Back", "Front"];
pub const BRANCH_LABELS: [&str; 2] = ["2 branches", "3 branches"];
pub const CATEGORY_LABELS: [&str; 6] = ["GT", "Stock Car", "Rallycross", "Kart", "Off-Road", "Autocross"];

/// Banking preset chips, in degrees.
pub const BANKING_PRESETS: [f32; 4] = [0.0, 10.0, 18.0, 22.0];
pub const WALL_OFFSET_PRESETS: [f32; 4] = [0.0, 1.5, 4.0, 8.0];
pub const GRID_PRESETS: [usize; 6] = [8, 10, 12, 14, 16, 18];

/// Off-track surfaces in catalogue order: every entry of `SurfaceType::OFF_TRACK_TYPES`.
pub fn off_track_surfaces() -> Vec<SurfaceType> {
    SURFACES.iter().copied().filter(|s| SurfaceType::OFF_TRACK_TYPES.contains(s)).collect()
}

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
    ZoneSurface,
    ZoneLayer,
    RampAngle,
    RampLength,
    RampWidth,
    RampHeight,
    RampPitch,
    RampSurface,
    CpFinish,
    ToolSurface,
    ToolWidth,
    ToolBanking,
    ToolWallDist,
    ToolWallType,
    ToolBranches,
    GlobalWallOffset,
    GlobalWallType,
    OffTrack,
    Category,
    GridSlots,
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
    FitPitch,
    FitHeight,
    AutoCheckpoints,
    RebuildGeometry,
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

/// Option list of a dropdown row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Options {
    /// [`SURFACES`].
    Surfaces,
    /// [`off_track_surfaces`].
    OffTrack,
    /// `CarCategory::ALL`.
    Category,
}

impl Options {
    pub fn len(self) -> usize {
        match self {
            Options::Surfaces => SURFACES.len(),
            Options::OffTrack => off_track_surfaces().len(),
            Options::Category => CarCategory::ALL.len(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn label(self, i: usize) -> &'static str {
        match self {
            Options::Category => CATEGORY_LABELS.get(i).copied().unwrap_or(""),
            _ => self.surface(i).map(|s| s.name()).unwrap_or(""),
        }
    }

    /// Surface of option `i`, for colour swatches.
    pub fn surface(self, i: usize) -> Option<SurfaceType> {
        match self {
            Options::Surfaces => SURFACES.get(i).copied(),
            Options::OffTrack => off_track_surfaces().get(i).copied(),
            Options::Category => None,
        }
    }
}

/// A preset chip in a chip row.
#[derive(Debug, Clone, PartialEq)]
pub struct Chip {
    pub label: String,
    pub edit: Edit,
    pub active: bool,
}

/// A full-width action button in a button row.
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub label: String,
    pub edit: Edit,
    /// Draws the button with an accent border (e.g. a fit action that would change something).
    pub highlight: bool,
}

/// One control row of the inspector.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    /// `[-] bar [+]` with drag, click-to-type, focused wheel and hold-to-repeat.
    /// `anchor` is the value a drag starts from when the selection is mixed.
    Stepper { prop: Prop, label: &'static str, value: Common<f32>, anchor: f32, min: f32, max: f32, step: f32, unit: &'static str, signed: bool },
    Dropdown { prop: Prop, label: &'static str, options: Options, value: Common<usize> },
    Segmented { prop: Prop, label: &'static str, labels: &'static [&'static str], value: Common<usize> },
    /// One on/off toggle at the right of the row.
    Toggle { prop: Prop, label: &'static str, value: Common<bool> },
    /// Left and right on/off toggles on one row.
    Sides { label: &'static str, left: (Prop, Common<bool>), right: (Prop, Common<bool>) },
    Chips { label: &'static str, chips: Vec<Chip> },
    Buttons(Vec<Button>),
    /// Side view of a jump ramp profile.
    RampProfile(Box<JumpRamp>),
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

impl Section {
    fn new(id: &'static str, title: &'static str, rows: Vec<Row>) -> Self {
        Self { id, title, side_columns: false, collapsed_by_default: false, rows }
    }

    fn collapsed(mut self) -> Self {
        self.collapsed_by_default = true;
        self
    }

    fn with_side_columns(mut self) -> Self {
        self.side_columns = true;
        self
    }
}

/// The fixed action bar at the bottom of the inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footer {
    pub duplicate: bool,
    pub delete_label: &'static str,
}

const FOOTER: Option<Footer> = Some(Footer { duplicate: true, delete_label: "Delete" });

/// Everything the inspector draws for the current selection.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorModel {
    pub title: String,
    /// Number of selected entities; the header shows "N selected" when above 1.
    pub count: usize,
    /// Right-hand header text when at most one entity is selected (e.g. the active tool).
    pub subtitle: Option<&'static str>,
    pub sections: Vec<Section>,
    pub footer: Option<Footer>,
}

// Layout metrics in 720p reference pixels (scale with `UiScaler::s`).
pub const TITLE_H: f32 = 30.0;
pub const HEADER_H: f32 = 26.0;
pub const SECTION_HEADER_H: f32 = 20.0;
pub const SECTION_GAP: f32 = 6.0;
pub const ROW_H: f32 = 22.0;
pub const ROW_GAP: f32 = 4.0;
pub const INFO_H: f32 = 18.0;
pub const RAMP_PROFILE_H: f32 = 74.0;
pub const FOOTER_H: f32 = 38.0;
pub const BODY_PAD: f32 = 8.0;

/// Reference-pixel height of one row, including its gap.
pub fn row_height(row: &Row) -> f32 {
    match row {
        Row::Info(_) => INFO_H,
        Row::RampProfile(_) => RAMP_PROFILE_H + ROW_GAP,
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
    card_h - TITLE_H - HEADER_H - if model.footer.is_some() { FOOTER_H } else { 0.0 }
}

fn surface_index(s: SurfaceType) -> usize {
    SURFACES.iter().position(|&x| x == s).unwrap_or(0)
}

fn wall_type_index(t: BarrierType) -> usize {
    WALL_TYPES.iter().position(|&x| x == t).unwrap_or(0)
}

fn stepper(prop: Prop, label: &'static str, value: Common<f32>, anchor: f32, (min, max): (f32, f32), step: f32, unit: &'static str) -> Row {
    Row::Stepper { prop, label, value, anchor, min, max, step, unit, signed: false }
}

fn banking_chips(prop: Prop, value: Common<f32>, invert: Edit) -> Row {
    let mut chips: Vec<Chip> = BANKING_PRESETS
        .iter()
        .map(|&deg| Chip { label: format!("{deg:.0}°"), edit: Edit::Set(prop, deg), active: value.same().is_some_and(|b| (b - deg).abs() < 0.05) })
        .collect();
    chips.push(Chip { label: "±".to_string(), edit: invert, active: false });
    Row::Chips { label: "Presets", chips }
}

fn title_for(kind: &str, plural: &str, indices: &[usize]) -> String {
    match indices {
        [i] => format!("{kind} #{i}"),
        _ => plural.to_string(),
    }
}

/// Builds the inspector model for the current selection, or `None` for selections still drawn
/// by the legacy inspector.
pub fn build_inspector(state: &EditorState, tools: &ToolSettings) -> Option<InspectorModel> {
    match &state.selection {
        Selection::None => Some(build_track(state, tools)),
        Selection::Waypoint(_) | Selection::MultipleWaypoints(_) => build_waypoints(state),
        Selection::SurfaceZone(_) => build_zones(state),
        Selection::Obstacle(_) => build_obstacles(state),
        Selection::JumpRamp(_) => build_ramps(state),
        Selection::Checkpoint(_) => build_checkpoints(state),
        Selection::GridSlot(_) => build_grid_slots(state),
        Selection::PitBox => Some(build_pit(state)),
        Selection::Multi { waypoints, surface_zones, obstacles, jump_ramps, checkpoints, grid_slots, pit_box } => {
            // Several entities of one kind get that kind's view; mixed kinds wait for HC-3.
            let kinds = [!waypoints.is_empty(), !surface_zones.is_empty(), !obstacles.is_empty(), !jump_ramps.is_empty(), !checkpoints.is_empty(), !grid_slots.is_empty(), *pit_box];
            if kinds.iter().filter(|&&k| k).count() != 1 {
                return None;
            }
            if !waypoints.is_empty() {
                build_waypoints(state)
            } else if !surface_zones.is_empty() {
                build_zones(state)
            } else if !obstacles.is_empty() {
                build_obstacles(state)
            } else if !jump_ramps.is_empty() {
                build_ramps(state)
            } else if !checkpoints.is_empty() {
                build_checkpoints(state)
            } else if !grid_slots.is_empty() {
                build_grid_slots(state)
            } else {
                Some(build_pit(state))
            }
        }
    }
}

fn build_waypoints(state: &EditorState) -> Option<InspectorModel> {
    let wps = &state.track.spline.waypoints;
    let indices: Vec<usize> = state.selection.selected_waypoint_indices().into_iter().filter(|&i| i < wps.len()).collect();
    let first = *indices.first()?;
    let sel = || indices.iter().map(|&i| &wps[i]);
    let offset = state.barrier_offset;

    let banking = common_value(sel().map(|w| w.bank_angle));
    let info = if let [i] = indices[..] {
        let p = wps[i].point;
        Row::Info(format!("Position ({:.1}, {:.1})", p.x, p.y))
    } else {
        let list: Vec<String> = indices.iter().map(|i| format!("#{i}")).collect();
        Row::Info(format!("Waypoints {}", list.join(", ")))
    };

    Some(InspectorModel {
        title: title_for("Waypoint", "Waypoints", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![
            Section::new("wp.road", "ROAD", vec![
                stepper(Prop::WpWidth, "Width", common_value(sel().map(|w| w.width)), wps[first].width, WIDTH_RANGE, 0.5, "m"),
                Row::Dropdown { prop: Prop::WpSurface, label: "Surface", options: Options::Surfaces, value: common_value(sel().map(|w| surface_index(w.surface.unwrap_or(SurfaceType::Asphalt)))) },
            ]),
            Section::new("wp.banking", "BANKING", vec![
                Row::Stepper { prop: Prop::WpBanking, label: "Angle", value: banking, anchor: wps[first].bank_angle, min: BANKING_RANGE.0, max: BANKING_RANGE.1, step: 1.0, unit: "°", signed: true },
                banking_chips(Prop::WpBanking, banking, Edit::Do(Action::InvertBanking)),
            ]),
            Section::new("wp.edges", "EDGES", vec![
                Row::Sides { label: "Curb", left: (Prop::WpLeftCurb, common_value(sel().map(|w| w.left_curb))), right: (Prop::WpRightCurb, common_value(sel().map(|w| w.right_curb))) },
                Row::Sides { label: "Wall", left: (Prop::WpLeftWall, common_value(sel().map(|w| w.left_wall))), right: (Prop::WpRightWall, common_value(sel().map(|w| w.right_wall))) },
                stepper(Prop::WpWallDistL, "Dist L", common_value(sel().map(|w| w.left_wall_distance.unwrap_or(offset))), wps[first].left_wall_distance.unwrap_or(offset), WALL_DISTANCE_RANGE, 0.5, "m"),
                stepper(Prop::WpWallDistR, "Dist R", common_value(sel().map(|w| w.right_wall_distance.unwrap_or(offset))), wps[first].right_wall_distance.unwrap_or(offset), WALL_DISTANCE_RANGE, 0.5, "m"),
                Row::Segmented { prop: Prop::WpWallType, label: "Wall type", labels: &WALL_TYPE_LABELS, value: common_value(sel().map(|w| wall_type_index(w.wall_type.unwrap_or(state.barrier_type)))) },
            ])
            .with_side_columns(),
            Section::new("wp.info", "INFO", vec![info]).collapsed(),
        ],
    })
}

fn shape_name(shape: &SurfaceShape) -> &'static str {
    match shape {
        SurfaceShape::Circle { .. } => "Circle",
        SurfaceShape::Aabb { .. } => "Box",
        SurfaceShape::OrientedBox { .. } => "Oriented box",
        SurfaceShape::Polygon { vertices } if vertices.len() == 3 => "Triangle",
        SurfaceShape::Polygon { .. } => "Polygon",
    }
}

fn build_zones(state: &EditorState) -> Option<InspectorModel> {
    let zones = &state.track.geometry.surface_zones;
    let indices: Vec<usize> = state.selection.selected_surface_zone_indices().into_iter().filter(|&i| i < zones.len()).collect();
    indices.first()?;
    let sel = || indices.iter().map(|&i| &zones[i]);
    let shapes = match common_value(sel().map(|z| shape_name(&z.shape))) {
        Common::Same(name) => format!("Shape: {name}"),
        _ => "Shape: mixed".to_string(),
    };
    Some(InspectorModel {
        title: title_for("Surface Zone", "Surface Zones", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![
            Section::new("zone.zone", "ZONE", vec![
                Row::Dropdown { prop: Prop::ZoneSurface, label: "Material", options: Options::Surfaces, value: common_value(sel().map(|z| surface_index(z.surface))) },
                Row::Segmented { prop: Prop::ZoneLayer, label: "Layer", labels: &LAYER_LABELS, value: common_value(sel().map(|z| usize::from(z.is_above_track()))) },
            ]),
            Section::new("zone.info", "INFO", vec![Row::Info(shapes)]).collapsed(),
        ],
    })
}

fn build_obstacles(state: &EditorState) -> Option<InspectorModel> {
    let obstacles = &state.track.geometry.obstacles;
    let indices: Vec<usize> = state.selection.selected_obstacle_indices().into_iter().filter(|&i| i < obstacles.len()).collect();
    indices.first()?;
    let rows = if let [i] = indices[..] {
        let obs = &obstacles[i];
        let shape = match obs.shape {
            ObstacleShape::Circle { .. } => "Circle",
            ObstacleShape::Box { .. } => "Box",
            ObstacleShape::Polygon { .. } => "Polygon",
        };
        let c = obs.center();
        vec![Row::Info(obs.name.clone()), Row::Info(format!("Shape: {shape}")), Row::Info(format!("Position ({:.1}, {:.1})", c.x, c.y))]
    } else {
        indices.iter().map(|&i| Row::Info(obstacles[i].name.clone())).collect()
    };
    Some(InspectorModel {
        title: title_for("Obstacle", "Obstacles", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![Section::new("obstacle.info", "INFO", rows)],
    })
}

fn build_ramps(state: &EditorState) -> Option<InspectorModel> {
    let ramps = &state.track.geometry.jump_ramps;
    let indices: Vec<usize> = state.selection.selected_jump_ramp_indices().into_iter().filter(|&i| i < ramps.len()).collect();
    let first = &ramps[*indices.first()?];
    let sel = || indices.iter().map(|&i| &ramps[i]);
    let (fit_pitch, fit_height) = if indices.len() == 1 {
        (format!("Fit pitch {:.1}°", first.fitted_pitch_deg()), format!("Fit height {:.1}m", first.fitted_height()))
    } else {
        ("Fit pitch".to_string(), "Fit height".to_string())
    };
    let has_flat = sel().any(|r| r.flat_length() > 0.05);
    Some(InspectorModel {
        title: title_for("Jump Ramp", "Jump Ramps", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![
            Section::new("ramp.shape", "SHAPE", vec![
                stepper(Prop::RampAngle, "Angle", common_value(sel().map(|r| r.angle_deg())), first.angle_deg(), RAMP_ANGLE_RANGE, 1.0, "°"),
                stepper(Prop::RampLength, "Length", common_value(sel().map(|r| r.length())), first.length(), RAMP_LENGTH_RANGE, 0.5, "m"),
                stepper(Prop::RampWidth, "Width", common_value(sel().map(|r| r.width())), first.width(), RAMP_WIDTH_RANGE, 0.5, "m"),
            ]),
            Section::new("ramp.profile", "PROFILE", vec![
                stepper(Prop::RampHeight, "Height", common_value(sel().map(|r| r.height)), first.height, RAMP_HEIGHT_RANGE, 0.1, "m"),
                stepper(Prop::RampPitch, "Pitch", common_value(sel().map(|r| r.ramp_angle_deg)), first.ramp_angle_deg, RAMP_PITCH_RANGE, 1.0, "°"),
                Row::Buttons(vec![
                    Button { label: fit_pitch, edit: Edit::Do(Action::FitPitch), highlight: has_flat },
                    Button { label: fit_height, edit: Edit::Do(Action::FitHeight), highlight: has_flat },
                ]),
                Row::RampProfile(Box::new(first.clone())),
            ]),
            Section::new("ramp.surface", "SURFACE", vec![Row::Dropdown { prop: Prop::RampSurface, label: "Material", options: Options::Surfaces, value: common_value(sel().map(|r| surface_index(r.surface))) }]),
        ],
    })
}

/// Positions in `track.checkpoints` of the selected checkpoint ids.
fn selected_checkpoint_positions(state: &EditorState) -> Vec<usize> {
    let ids = state.selection.selected_checkpoint_indices();
    state.track.checkpoints.iter().enumerate().filter(|(_, c)| ids.contains(&c.id)).map(|(i, _)| i).collect()
}

fn build_checkpoints(state: &EditorState) -> Option<InspectorModel> {
    let positions = selected_checkpoint_positions(state);
    positions.first()?;
    let ids: Vec<usize> = positions.iter().map(|&i| state.track.checkpoints[i].id).collect();
    Some(InspectorModel {
        title: title_for("Checkpoint Gate", "Checkpoint Gates", &ids),
        count: ids.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![Section::new("cp.gate", "GATE", vec![Row::Toggle { prop: Prop::CpFinish, label: "Finish line", value: common_value(positions.iter().map(|&i| state.track.checkpoints[i].is_finish_line)) }])],
    })
}

fn build_grid_slots(state: &EditorState) -> Option<InspectorModel> {
    let slots = &state.track.grid_positions;
    let indices: Vec<usize> = state.selection.selected_grid_slot_indices().into_iter().filter(|&i| i < slots.len()).collect();
    indices.first()?;
    let rows = indices
        .iter()
        .map(|&i| {
            let p = slots[i].position;
            Row::Info(format!("Slot {}: position ({:.1}, {:.1})", slots[i].grid_slot + 1, p.x, p.y))
        })
        .collect();
    Some(InspectorModel {
        title: title_for("Grid Slot", "Grid Slots", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![Section::new("grid.info", "INFO", rows)],
    })
}

fn build_pit(state: &EditorState) -> InspectorModel {
    let rows = match &state.track.pit_lane {
        Some(lane) => vec![
            Row::Info(format!("Length: {} m", lane.spline.total_length as u32)),
            Row::Info(format!("Stalls: {}", lane.pit_boxes.len())),
            Row::Info(format!("Speed limit: {:.0} km/h", lane.speed_limit * 3.6)),
        ],
        None => vec![Row::Info(format!("Pit box area: {}", if state.track.pit_box_area.is_some() { "set" } else { "none" }))],
    };
    InspectorModel {
        title: "Pit Lane".to_string(),
        count: 1,
        subtitle: None,
        footer: Some(Footer { duplicate: false, delete_label: "Clear pit lane" }),
        sections: vec![Section::new("pit.info", "PIT LANE", rows)],
    }
}

fn build_track(state: &EditorState, tools: &ToolSettings) -> InspectorModel {
    let mut sections = Vec::new();
    match tools.active_tool {
        EditorToolType::RoadSpline => {
            let bank = tools.new_waypoint_bank_angle;
            let dist = tools.new_waypoint_left_wall_distance.unwrap_or(state.barrier_offset);
            let wall = tools.new_waypoint_wall_type.unwrap_or(state.barrier_type);
            sections.push(Section::new("tool.spline", "NEW WAYPOINTS", vec![
                Row::Dropdown { prop: Prop::ToolSurface, label: "Surface", options: Options::Surfaces, value: Common::Same(surface_index(tools.active_surface)) },
                stepper(Prop::ToolWidth, "Width", Common::Same(tools.new_waypoint_width), tools.new_waypoint_width, WIDTH_RANGE, 0.5, "m"),
                Row::Stepper { prop: Prop::ToolBanking, label: "Banking", value: Common::Same(bank), anchor: bank, min: BANKING_RANGE.0, max: BANKING_RANGE.1, step: 1.0, unit: "°", signed: true },
                banking_chips(Prop::ToolBanking, Common::Same(bank), Edit::Set(Prop::ToolBanking, -bank)),
                stepper(Prop::ToolWallDist, "Wall dist", Common::Same(dist), dist, WALL_DISTANCE_RANGE, 0.5, "m"),
                Row::Segmented { prop: Prop::ToolWallType, label: "Wall type", labels: &WALL_TYPE_LABELS, value: Common::Same(wall_type_index(wall)) },
            ]));
        }
        EditorToolType::RoadSplit => {
            sections.push(Section::new("tool.split", "ROAD SPLIT", vec![
                Row::Info("Right-click the track to insert a split.".to_string()),
                Row::Info("Left-click a socket to select a branch.".to_string()),
                Row::Info("Right-click empty space to extend it.".to_string()),
                Row::Info("Right-click near the track to merge.".to_string()),
                Row::Segmented { prop: Prop::ToolBranches, label: "Branches", labels: &BRANCH_LABELS, value: Common::Same(tools.split_branch_count.clamp(2, 3) - 2) },
            ]));
        }
        EditorToolType::JumpRamp => {
            let preview = JumpRamp::new(
                0,
                SurfaceShape::OrientedBox { center: glam::Vec2::ZERO, half_extents: glam::Vec2::new(6.0, 4.0), angle: 0.0 },
                glam::Vec2::X,
                4.0,
                15.0,
                1.8,
                "Preview Ramp",
            )
            .with_surface(tools.active_surface);
            sections.push(Section::new("tool.ramp", "NEW RAMPS", vec![
                Row::Info("Click and drag on the track to place a ramp.".to_string()),
                Row::Dropdown { prop: Prop::ToolSurface, label: "Surface", options: Options::Surfaces, value: Common::Same(surface_index(tools.active_surface)) },
                Row::RampProfile(Box::new(preview)),
            ]));
        }
        _ => {}
    }

    let offset = state.barrier_offset;
    let wall_chips = WALL_OFFSET_PRESETS
        .iter()
        .map(|&m| Chip { label: if m.fract() == 0.0 { format!("{m:.0} m") } else { format!("{m:.1}") }, edit: Edit::Set(Prop::GlobalWallOffset, m), active: (offset - m).abs() < 0.05 })
        .collect();
    sections.push(Section::new("circuit.walls", "CIRCUIT WALLS", vec![
        stepper(Prop::GlobalWallOffset, "Offset", Common::Same(offset), offset, GLOBAL_WALL_OFFSET_RANGE, 0.5, "m"),
        Row::Chips { label: "Presets", chips: wall_chips },
        Row::Segmented { prop: Prop::GlobalWallType, label: "Type", labels: &WALL_TYPE_LABELS, value: Common::Same(wall_type_index(state.barrier_type)) },
    ]));

    let grid = state.grid_count();
    let off_track = off_track_surfaces().iter().position(|&s| s == state.track.default_surface);
    let grid_chips = GRID_PRESETS.iter().map(|&n| Chip { label: n.to_string(), edit: Edit::Set(Prop::GridSlots, n as f32), active: grid == n }).collect();
    sections.push(Section::new("circuit", "CIRCUIT", vec![
        Row::Dropdown { prop: Prop::OffTrack, label: "Off-track", options: Options::OffTrack, value: off_track.map_or(Common::Mixed, Common::Same) },
        Row::Dropdown { prop: Prop::Category, label: "Category", options: Options::Category, value: CarCategory::ALL.iter().position(|&c| c == state.track.car_category).map_or(Common::Mixed, Common::Same) },
        stepper(Prop::GridSlots, "Grid", Common::Same(grid as f32), grid as f32, GRID_SLOTS_RANGE, 1.0, " slots"),
        Row::Chips { label: "Presets", chips: grid_chips },
        Row::Buttons(vec![
            Button { label: "Auto checkpoints".to_string(), edit: Edit::Do(Action::AutoCheckpoints), highlight: false },
            Button { label: "Rebuild".to_string(), edit: Edit::Do(Action::RebuildGeometry), highlight: false },
        ]),
    ]));

    InspectorModel { title: "Circuit".to_string(), count: 0, subtitle: Some(tools.active_tool.title()), sections, footer: None }
}

fn clamp_to(v: f32, (lo, hi): (f32, f32)) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        lo
    }
}

/// Applies one inspector edit to every selected entity, as one undo step.
pub fn apply_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    use Prop::*;
    let prop = match edit {
        Edit::Set(p, _) | Edit::Step(p, _) | Edit::Flag(p, _) | Edit::Pick(p, _) => Some(p),
        Edit::Do(_) => None,
    };
    match (prop, edit) {
        (_, Edit::Do(Action::Duplicate)) => {
            tools.duplicate_selected(state);
        }
        (_, Edit::Do(Action::Delete)) => {
            tools.delete_selected(state);
        }
        (Some(WpWidth | WpBanking | WpLeftCurb | WpRightCurb | WpLeftWall | WpRightWall | WpWallDistL | WpWallDistR | WpWallType | WpSurface), _)
        | (_, Edit::Do(Action::InvertBanking)) => apply_waypoint_edit(state, tools, edit),
        (Some(ZoneSurface | ZoneLayer), _) => apply_zone_edit(state, edit),
        (Some(RampAngle | RampLength | RampWidth | RampHeight | RampPitch | RampSurface), _) | (_, Edit::Do(Action::FitPitch | Action::FitHeight)) => {
            apply_ramp_edit(state, tools, edit)
        }
        (Some(CpFinish), Edit::Flag(_, on)) => {
            let positions = selected_checkpoint_positions(state);
            if !positions.is_empty() {
                state.record_undo();
                for i in positions {
                    state.track.checkpoints[i].is_finish_line = on;
                }
                state.auto_generate_grid();
                state.revalidate();
            }
        }
        _ => apply_track_edit(state, tools, edit),
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
    for &i in &indices {
        let wp = &mut state.track.spline.waypoints[i];
        match edit {
            Edit::Set(Prop::WpWidth, v) => wp.width = clamp_to(v, WIDTH_RANGE),
            Edit::Step(Prop::WpWidth, d) => wp.width = clamp_to(wp.width + d, WIDTH_RANGE),
            Edit::Set(Prop::WpBanking, v) => wp.bank_angle = clamp_to(v, BANKING_RANGE),
            Edit::Step(Prop::WpBanking, d) => wp.bank_angle = clamp_to(wp.bank_angle + d, BANKING_RANGE),
            Edit::Do(Action::InvertBanking) => wp.bank_angle = -wp.bank_angle,
            Edit::Set(Prop::WpWallDistL, v) => wp.left_wall_distance = Some(clamp_to(v, WALL_DISTANCE_RANGE)),
            Edit::Step(Prop::WpWallDistL, d) => wp.left_wall_distance = Some(clamp_to(wp.left_wall_distance.unwrap_or(offset) + d, WALL_DISTANCE_RANGE)),
            Edit::Set(Prop::WpWallDistR, v) => wp.right_wall_distance = Some(clamp_to(v, WALL_DISTANCE_RANGE)),
            Edit::Step(Prop::WpWallDistR, d) => wp.right_wall_distance = Some(clamp_to(wp.right_wall_distance.unwrap_or(offset) + d, WALL_DISTANCE_RANGE)),
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

fn apply_zone_edit(state: &mut EditorState, edit: Edit) {
    let len = state.track.geometry.surface_zones.len();
    let indices: Vec<usize> = state.selection.selected_surface_zone_indices().into_iter().filter(|&i| i < len).collect();
    if indices.is_empty() {
        return;
    }
    state.record_undo();
    for i in indices {
        let zone = &mut state.track.geometry.surface_zones[i];
        match edit {
            Edit::Pick(Prop::ZoneSurface, k) => {
                if let Some(&s) = SURFACES.get(k) {
                    zone.surface = s;
                }
            }
            Edit::Pick(Prop::ZoneLayer, k) => zone.layer = if k == 1 { SurfaceLayer::AboveTrack } else { SurfaceLayer::BelowTrack },
            _ => {}
        }
    }
    state.revalidate();
}

fn apply_ramp_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    let len = state.track.geometry.jump_ramps.len();
    let indices: Vec<usize> = state.selection.selected_jump_ramp_indices().into_iter().filter(|&i| i < len).collect();
    if indices.is_empty() {
        return;
    }
    state.record_undo();
    for i in indices {
        let ramp = &mut state.track.geometry.jump_ramps[i];
        match edit {
            Edit::Set(Prop::RampAngle, v) => ramp.set_angle_deg(clamp_to(v, RAMP_ANGLE_RANGE)),
            Edit::Step(Prop::RampAngle, d) => ramp.set_angle_deg((ramp.angle_deg() + d).rem_euclid(360.0)),
            Edit::Set(Prop::RampLength, v) => ramp.set_length(clamp_to(v, RAMP_LENGTH_RANGE)),
            Edit::Step(Prop::RampLength, d) => ramp.set_length(clamp_to(ramp.length() + d, RAMP_LENGTH_RANGE)),
            Edit::Set(Prop::RampWidth, v) => ramp.set_width(clamp_to(v, RAMP_WIDTH_RANGE)),
            Edit::Step(Prop::RampWidth, d) => ramp.set_width(clamp_to(ramp.width() + d, RAMP_WIDTH_RANGE)),
            Edit::Set(Prop::RampHeight, v) => ramp.height = clamp_to(v, RAMP_HEIGHT_RANGE),
            Edit::Step(Prop::RampHeight, d) => ramp.height = clamp_to(ramp.height + d, RAMP_HEIGHT_RANGE),
            Edit::Set(Prop::RampPitch, v) => ramp.ramp_angle_deg = clamp_to(v, RAMP_PITCH_RANGE),
            Edit::Step(Prop::RampPitch, d) => ramp.ramp_angle_deg = clamp_to(ramp.ramp_angle_deg + d, RAMP_PITCH_RANGE),
            Edit::Pick(Prop::RampSurface, k) => {
                if let Some(&s) = SURFACES.get(k) {
                    ramp.surface = s;
                    tools.active_surface = s;
                }
            }
            Edit::Do(Action::FitPitch) => ramp.ramp_angle_deg = ramp.fitted_pitch_deg(),
            Edit::Do(Action::FitHeight) => ramp.height = ramp.fitted_height(),
            _ => {}
        }
    }
    state.revalidate();
}

fn apply_track_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    match edit {
        Edit::Pick(Prop::ToolSurface, k) => {
            if let Some(&s) = SURFACES.get(k) {
                tools.active_surface = s;
            }
        }
        Edit::Set(Prop::ToolWidth, v) => tools.new_waypoint_width = clamp_to(v, WIDTH_RANGE),
        Edit::Step(Prop::ToolWidth, d) => tools.new_waypoint_width = clamp_to(tools.new_waypoint_width + d, WIDTH_RANGE),
        Edit::Set(Prop::ToolBanking, v) => tools.new_waypoint_bank_angle = clamp_to(v, BANKING_RANGE),
        Edit::Step(Prop::ToolBanking, d) => tools.new_waypoint_bank_angle = clamp_to(tools.new_waypoint_bank_angle + d, BANKING_RANGE),
        Edit::Set(Prop::ToolWallDist, _) | Edit::Step(Prop::ToolWallDist, _) => {
            let current = tools.new_waypoint_left_wall_distance.unwrap_or(state.barrier_offset);
            let v = match edit {
                Edit::Step(_, d) => current + d,
                Edit::Set(_, v) => v,
                _ => current,
            };
            let d = Some(clamp_to(v, WALL_DISTANCE_RANGE));
            tools.new_waypoint_left_wall_distance = d;
            tools.new_waypoint_right_wall_distance = d;
        }
        Edit::Pick(Prop::ToolWallType, k) => tools.new_waypoint_wall_type = WALL_TYPES.get(k).copied(),
        Edit::Pick(Prop::ToolBranches, k) => tools.split_branch_count = 2 + k.min(1),
        Edit::Set(Prop::GlobalWallOffset, v) => tools.set_global_barrier_offset(state, clamp_to(v, GLOBAL_WALL_OFFSET_RANGE)),
        Edit::Step(Prop::GlobalWallOffset, d) => {
            let v = clamp_to(state.barrier_offset + d, GLOBAL_WALL_OFFSET_RANGE);
            tools.set_global_barrier_offset(state, v);
        }
        Edit::Pick(Prop::GlobalWallType, k) => {
            if let Some(&t) = WALL_TYPES.get(k) {
                tools.set_global_barrier_type(state, t);
            }
        }
        Edit::Pick(Prop::OffTrack, k) => {
            if let Some(&s) = off_track_surfaces().get(k) {
                tools.set_track_default_surface(state, s);
            }
        }
        Edit::Pick(Prop::Category, k) => {
            if let Some(&c) = CarCategory::ALL.get(k) {
                tools.set_track_car_category(state, c);
            }
        }
        Edit::Set(Prop::GridSlots, _) | Edit::Step(Prop::GridSlots, _) => {
            let current = state.grid_count() as f32;
            let v = match edit {
                Edit::Step(_, d) => current + d,
                Edit::Set(_, v) => v,
                _ => current,
            };
            let n = clamp_to(v.round(), GRID_SLOTS_RANGE) as usize;
            if n != state.grid_count() {
                state.set_grid_count(n);
            }
        }
        Edit::Do(Action::AutoCheckpoints) => {
            state.record_undo();
            state.track.auto_generate_checkpoints(8, 3);
            state.auto_generate_grid();
            state.revalidate();
        }
        Edit::Do(Action::RebuildGeometry) => {
            state.record_undo();
            state.rebuild_geometry();
        }
        _ => {}
    }
}
