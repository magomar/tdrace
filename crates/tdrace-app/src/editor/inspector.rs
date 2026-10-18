//! Track Studio inspector interaction state.
//! Governed by specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use std::collections::HashSet;

use cabinet::ui::FieldDropdown;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, ObstacleShape, RampKind, SurfaceLayer, SurfaceShape, MIN_MEGA_RAMP_GAP_METERS};
use tdrace_core::track::junction_kit::{JunctionComponent, JunctionShape, Side};
use tdrace_core::track::launch_chute::PAD_WIDTH_RANGE;
use tdrace_core::track::network::SegmentId;
use tdrace_core::track::spline::TrackWaypoint;
use tdrace_core::track::PackedGridPattern;
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

/// What a captured stepper drag did this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragOutcome {
    None,
    /// New absolute value for the selection.
    Set(f32),
    /// The press was released without a drag: open text entry.
    OpenTextEntry,
}

impl StepperDrag {
    /// A drag starts only from a press on the bar; a press elsewhere never captures it.
    pub fn press(prop: Prop, pressed_on_bar: bool, mouse_x: f32, start_value: f32) -> Option<Self> {
        pressed_on_bar.then_some(Self { prop, start_x: mouse_x, start_value, moved: false, last_value: start_value })
    }
}

/// Advances the captured drag of stepper `prop` by one frame. `mouse_x` is the raw pointer x, so
/// the value keeps following the pointer off the bar until release. `slop` is the distance the
/// pointer must move before the press counts as a drag; `mixed` re-sends the value every frame.
#[allow(clippy::too_many_arguments)]
pub fn update_stepper_drag(drag: &mut Option<StepperDrag>, prop: Prop, down: bool, mouse_x: f32, bar_w: f32, (min, max, step): (f32, f32, f32), slop: f32, mixed: bool) -> DragOutcome {
    let Some(mut d) = drag.filter(|d| d.prop == prop) else {
        return DragOutcome::None;
    };
    if !down {
        *drag = None;
        return if d.moved { DragOutcome::None } else { DragOutcome::OpenTextEntry };
    }
    let dx = mouse_x - d.start_x;
    if dx.abs() > slop {
        d.moved = true;
    }
    let mut outcome = DragOutcome::None;
    if d.moved {
        let raw = d.start_value + dx / bar_w.max(1.0) * (max - min);
        let v = ((raw / step).round() * step).clamp(min, max);
        if (v - d.last_value).abs() > f32::EPSILON || mixed {
            outcome = DragOutcome::Set(v);
            d.last_value = v;
        }
    }
    *drag = Some(d);
    outcome
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
    /// Tooltip key under the pointer and the time it was first hovered.
    pub tooltip_hover: Option<(String, f64)>,
    /// Kind shown for a mixed-kind selection; `None` = the first kind.
    pub mixed_kind: Option<MixedKind>,
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
pub const RAMP_WHOOP_SPACING_RANGE: (f32, f32) = (2.0, 15.0);
pub const RAMP_MEGA_GAP_RANGE: (f32, f32) = (MIN_MEGA_RAMP_GAP_METERS, 40.0);
pub const RAMP_MEGA_BOOST_RANGE: (f32, f32) = (0.5, 3.0);
pub const RAMP_KIND_LABELS: [&str; 3] = ["Ramp", "Whoops", "Mega"];
pub const GRID_SLOTS_RANGE: (f32, f32) = (1.0, 24.0);

/// The one surface catalogue every inspector surface picker uses (order and names).
pub const SURFACES: [SurfaceType; 15] = [
    SurfaceType::Asphalt,
    SurfaceType::Concrete,
    SurfaceType::Dirt,
    SurfaceType::PackedGravel,
    SurfaceType::DeepGravel,
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
pub const PIT_MODE_LABELS: [&str; 2] = ["Layout", "Free-form"];
pub const PIT_SHAPE_LABELS: [&str; 2] = ["Taper", "TurnOff"];
pub const PIT_JUNCTION_LENGTH_RANGE: (f32, f32) = (5.0, 200.0);
/// The TurnOff guard accepts 10 <= angle < 60 degrees.
pub const PIT_ANGLE_RANGE: (f32, f32) = (10.0, 59.0);
pub const DEFAULT_TURNOFF_ANGLE: f32 = 30.0;
pub const PIT_GAP_RANGE: (f32, f32) = (0.0, 15.0);
pub const PIT_ROAD_WIDTH_RANGE: (f32, f32) = (4.0, 12.0);
pub const PIT_BOX_START_RANGE: (f32, f32) = (0.0, 2000.0);
pub const PIT_BOX_COUNT_RANGE: (f32, f32) = (1.0, 40.0);
pub const PIT_BOX_SPACING_RANGE: (f32, f32) = (4.0, 40.0);
pub const LAYER_LABELS: [&str; 2] = ["Back", "Front"];
pub const BRANCH_LABELS: [&str; 2] = ["2 branches", "3 branches"];
/// Launch chute pad grids and surfaces, in the order of the inspector's segmented controls (spec 103).
pub const CHUTE_PATTERNS: [PackedGridPattern; 3] =
    [PackedGridPattern::AutocrossFiveThree, PackedGridPattern::RallycrossThreeTwoThreeTwo, PackedGridPattern::UniformFourAcross];
pub const CHUTE_PATTERN_LABELS: [&str; 3] = ["AX 5-3", "RX 3-2-3-2", "4 x 3"];
pub const CHUTE_SURFACES: [SurfaceType; 2] = [SurfaceType::Concrete, SurfaceType::Asphalt];
pub const CHUTE_SURFACE_LABELS: [&str; 2] = ["Concrete", "Asphalt"];
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
    WpRunoffL,
    WpRunoffR,
    ZoneSurface,
    ZoneLayer,
    RampKind,
    RampAngle,
    RampLength,
    RampWidth,
    RampHeight,
    RampPitch,
    RampWhoopSpacing,
    RampMegaGap,
    RampSurface,
    CpFinish,
    ToolSurface,
    ToolRunoff,
    ToolWidth,
    ToolBanking,
    ToolWallDist,
    ToolWallType,
    ToolBranches,
    ToolRampKind,
    ToolWhoopSpacing,
    ToolWhoopHeight,
    ToolWhoopWidth,
    ToolMegaHeight,
    ToolMegaBoost,
    ToolMegaGap,
    GlobalWallOffset,
    GlobalWallType,
    GlobalRunoff,
    OffTrack,
    Category,
    GridSlots,
    PitMode,
    PitEntryShape,
    PitEntryLength,
    PitEntryAngle,
    PitExitShape,
    PitExitLength,
    PitExitAngle,
    PitGap,
    PitRoadWidth,
    PitBoxStart,
    PitBoxCount,
    PitBoxSpacing,
    PitGarages,
    ChutePadWidth,
    ChutePattern,
    ChuteSurface,
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
    InsertLaunchChute,
    RemoveLaunchChute,
    Duplicate,
    Delete,
    /// Show this kind's controls for a mixed-kind selection (HC-3 option B).
    ShowKind(MixedKind),
}

/// Entity kinds of a mixed selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedKind {
    Waypoints,
    Zones,
    Obstacles,
    Ramps,
    Gates,
    GridSlots,
    PitLane,
}

impl MixedKind {
    fn name(self) -> &'static str {
        match self {
            MixedKind::Waypoints => "Waypoints",
            MixedKind::Zones => "Zones",
            MixedKind::Obstacles => "Obstacles",
            MixedKind::Ramps => "Ramps",
            MixedKind::Gates => "Gates",
            MixedKind::GridSlots => "Grid",
            MixedKind::PitLane => "Pit",
        }
    }
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
    /// A red message, e.g. a failed pit layout guard.
    Error(String),
    /// A diagnostic warning badge with optional tooltip (Spec 112).
    Warning { text: String, tooltip: Option<String> },
}

/// A collapsible group of rows.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub id: &'static str,
    pub title: String,
    /// Draws "L" and "R" column labels in the section header, above [`Row::Sides`] toggles.
    pub side_columns: bool,
    pub collapsed_by_default: bool,
    pub rows: Vec<Row>,
}

impl Section {
    fn new(id: &'static str, title: &'static str, rows: Vec<Row>) -> Self {
        Self { id, title: title.to_string(), side_columns: false, collapsed_by_default: false, rows }
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
        Row::Info(_) | Row::Error(_) | Row::Warning { .. } => INFO_H,
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
        Selection::Waypoint(_)
        | Selection::MultipleWaypoints(_)
        | Selection::BranchWaypoint(_, _)
        | Selection::MultipleBranchWaypoints(_) => build_waypoints(state),
        Selection::SurfaceZone(_) => build_zones(state),
        Selection::Obstacle(_) => build_obstacles(state),
        Selection::JumpRamp(_) => build_ramps(state),
        Selection::Checkpoint(_) => build_checkpoints(state),
        Selection::GridSlot(_) => build_grid_slots(state),
        Selection::PitBox => Some(build_pit(state, tools)),
        Selection::Multi { waypoints, branch_waypoints, surface_zones, obstacles, jump_ramps, checkpoints, grid_slots, pit_box } => {
            // Several entities of one kind get that kind's view; mixed kinds wait for HC-3.
            let has_waypoints = !waypoints.is_empty() || !branch_waypoints.is_empty();
            let kinds = [has_waypoints, !surface_zones.is_empty(), !obstacles.is_empty(), !jump_ramps.is_empty(), !checkpoints.is_empty(), !grid_slots.is_empty(), *pit_box];
            if kinds.iter().filter(|&&k| k).count() != 1 {
                return Some(build_mixed(state, tools));
            }
            if has_waypoints {
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
                Some(build_pit(state, tools))
            }
        }
    }
}

/// Mixed-kind selection: a chip row per kind and the chosen kind's own view (HC-3 option B).
fn build_mixed(state: &EditorState, tools: &ToolSettings) -> InspectorModel {
    let sel = &state.selection;
    let wp_count = sel.selected_waypoint_indices().len() + sel.selected_branch_waypoints().len();
    let parts: Vec<(MixedKind, usize, Option<InspectorModel>)> = [
        (MixedKind::Waypoints, wp_count, build_waypoints as fn(&EditorState) -> Option<InspectorModel>),
        (MixedKind::Zones, sel.selected_surface_zone_indices().len(), build_zones),
        (MixedKind::Obstacles, sel.selected_obstacle_indices().len(), build_obstacles),
        (MixedKind::Ramps, sel.selected_jump_ramp_indices().len(), build_ramps),
        (MixedKind::Gates, sel.selected_checkpoint_indices().len(), build_checkpoints),
        (MixedKind::GridSlots, sel.selected_grid_slot_indices().len(), build_grid_slots),
        (MixedKind::PitLane, usize::from(sel.is_pit_box_selected()), |s: &EditorState| Some(build_pit(s, &ToolSettings::default()))),
    ]
    .into_iter()
    .filter(|(_, n, _)| *n > 0)
    .map(|(kind, n, build)| (kind, n, build(state)))
    .collect();

    // A chip row picks the kind; the panel then shows that kind's own sections.
    let shown = tools.inspector.mixed_kind.filter(|k| parts.iter().any(|(p, ..)| p == k)).or(parts.first().map(|(k, ..)| *k));
    let chips = parts.iter().map(|(kind, n, _)| Chip { label: format!("{} {n}", kind.name()), edit: Edit::Do(Action::ShowKind(*kind)), active: Some(*kind) == shown }).collect();
    let mut sections = vec![Section::new("mixed.kinds", "SHOW", vec![Row::Chips { label: "", chips }])];
    if let Some((_, _, Some(model))) = parts.iter().find(|(k, ..)| Some(*k) == shown) {
        sections.extend(model.sections.clone());
    }
    InspectorModel { title: "Mixed selection".to_string(), count: sel.total_count(), subtitle: None, sections, footer: FOOTER }
}

fn build_waypoints(state: &EditorState) -> Option<InspectorModel> {
    let wps = &state.track.spline.waypoints;
    let indices: Vec<usize> = state.selection.selected_waypoint_indices().into_iter().filter(|&i| i < wps.len()).collect();
    let branch_indices: Vec<(SegmentId, usize)> = state
        .selection
        .selected_branch_waypoints()
        .into_iter()
        .filter(|&(sid, i)| {
            state
                .track
                .network
                .as_ref()
                .and_then(|n| n.get_segment(sid))
                .is_some_and(|s| i < s.waypoints.len())
        })
        .collect();
    let selected_wps: Vec<&TrackWaypoint> = indices
        .iter()
        .map(|&i| &wps[i])
        .chain(branch_indices.iter().filter_map(|&(sid, i)| {
            state.track.network.as_ref()?.get_segment(sid)?.waypoints.get(i)
        }))
        .collect();
    let first_wp = *selected_wps.first()?;
    let sel = || selected_wps.iter().copied();
    let offset = state.barrier_offset;
    let default_ro = state.track.default_runoff_surface().unwrap_or(state.track.default_surface);

    let banking = common_value(sel().map(|w| w.bank_angle));
    let (title, info) = if branch_indices.is_empty() {
        let info = if let [i] = indices[..] {
            let p = wps[i].point;
            Row::Info(format!("Position ({:.1}, {:.1})", p.x, p.y))
        } else {
            let list: Vec<String> = indices.iter().map(|i| format!("#{i}")).collect();
            Row::Info(format!("Waypoints {}", list.join(", ")))
        };
        (title_for("Waypoint", "Waypoints", &indices), info)
    } else if indices.is_empty() {
        if let [(sid, i)] = branch_indices[..] {
            let p = first_wp.point;
            let seg_name = state
                .track
                .network
                .as_ref()
                .and_then(|n| n.get_segment(sid))
                .map(|s| s.name.as_str())
                .unwrap_or("Branch");
            (
                format!("{seg_name} #{i}"),
                Row::Info(format!("Segment {} · ({:.1}, {:.1})", sid.0, p.x, p.y)),
            )
        } else {
            let list: Vec<String> = branch_indices.iter().map(|(s, i)| format!("S{}#{i}", s.0)).collect();
            (
                "Branch Waypoints".to_string(),
                Row::Info(format!("Waypoints {}", list.join(", "))),
            )
        }
    } else {
        (
            "Waypoints".to_string(),
            Row::Info(format!("{} main + {} branch waypoints", indices.len(), branch_indices.len())),
        )
    };

    Some(InspectorModel {
        title,
        count: selected_wps.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![
            Section::new("wp.road", "ROAD", vec![
                stepper(Prop::WpWidth, "Width", common_value(sel().map(|w| w.width)), first_wp.width, WIDTH_RANGE, 0.5, "m"),
                Row::Dropdown { prop: Prop::WpSurface, label: "Surface", options: Options::Surfaces, value: common_value(sel().map(|w| surface_index(w.surface.unwrap_or(SurfaceType::Asphalt)))) },
            ]),
            Section::new("wp.banking", "BANKING", vec![
                Row::Stepper { prop: Prop::WpBanking, label: "Angle", value: banking, anchor: first_wp.bank_angle, min: BANKING_RANGE.0, max: BANKING_RANGE.1, step: 1.0, unit: "°", signed: true },
                banking_chips(Prop::WpBanking, banking, Edit::Do(Action::InvertBanking)),
            ]),
            {
                let mut edge_rows = vec![
                    Row::Sides { label: "Curb", left: (Prop::WpLeftCurb, common_value(sel().map(|w| w.left_curb))), right: (Prop::WpRightCurb, common_value(sel().map(|w| w.right_curb))) },
                    Row::Sides { label: "Wall", left: (Prop::WpLeftWall, common_value(sel().map(|w| w.left_wall))), right: (Prop::WpRightWall, common_value(sel().map(|w| w.right_wall))) },
                    stepper(Prop::WpWallDistL, "Dist L", common_value(sel().map(|w| w.left_wall_distance.unwrap_or(offset))), first_wp.left_wall_distance.unwrap_or(offset), WALL_DISTANCE_RANGE, 0.5, "m"),
                    stepper(Prop::WpWallDistR, "Dist R", common_value(sel().map(|w| w.right_wall_distance.unwrap_or(offset))), first_wp.right_wall_distance.unwrap_or(offset), WALL_DISTANCE_RANGE, 0.5, "m"),
                    Row::Segmented { prop: Prop::WpWallType, label: "Wall type", labels: &WALL_TYPE_LABELS, value: common_value(sel().map(|w| wall_type_index(w.wall_type.unwrap_or(state.barrier_type)))) },
                    Row::Dropdown { prop: Prop::WpRunoffL, label: "Runoff L", options: Options::Surfaces, value: common_value(sel().map(|w| surface_index(w.left_runoff_surface.unwrap_or(default_ro)))) },
                    Row::Dropdown { prop: Prop::WpRunoffR, label: "Runoff R", options: Options::Surfaces, value: common_value(sel().map(|w| surface_index(w.right_runoff_surface.unwrap_or(default_ro)))) },
                ];

                let mut suppressed_warnings = Vec::new();
                for &i in &indices {
                    let wp = &wps[i];
                    let s_dist = state.track.spline.project_point(wp.point).progress_distance;
                    if wp.left_wall {
                        if let Some(rec) = state.track.geometry.wall_suppressions.iter().find(|r| {
                            r.side == Side::Left
                                && (r.contains_station(s_dist, 2.0)
                                    || r.distance_to_point(wp.point) <= wp.width * 0.5 + wp.left_wall_distance.unwrap_or(offset) + 2.0)
                        }) {
                            let text = format!("⚠️ Left wall suppressed (conflict with segment {})", rec.conflicting_segment.0);
                            let tooltip = format!("Wall suppressed due to clearance conflict with segment {}", rec.conflicting_segment.0);
                            if !suppressed_warnings.iter().any(|(t, _)| t == &text) {
                                suppressed_warnings.push((text, tooltip));
                            }
                        }
                    }
                    if wp.right_wall {
                        if let Some(rec) = state.track.geometry.wall_suppressions.iter().find(|r| {
                            r.side == Side::Right
                                && (r.contains_station(s_dist, 2.0)
                                    || r.distance_to_point(wp.point) <= wp.width * 0.5 + wp.right_wall_distance.unwrap_or(offset) + 2.0)
                        }) {
                            let text = format!("⚠️ Right wall suppressed (conflict with segment {})", rec.conflicting_segment.0);
                            let tooltip = format!("Wall suppressed due to clearance conflict with segment {}", rec.conflicting_segment.0);
                            if !suppressed_warnings.iter().any(|(t, _)| t == &text) {
                                suppressed_warnings.push((text, tooltip));
                            }
                        }
                    }
                }
                for (text, tooltip) in suppressed_warnings {
                    edge_rows.push(Row::Warning { text, tooltip: Some(tooltip) });
                }

                Section::new("wp.edges", "EDGES", edge_rows).with_side_columns()
            },
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

fn ramp_kind_index(kind: RampKind) -> usize {
    match kind {
        RampKind::Ramp => 0,
        RampKind::Whoops => 1,
        RampKind::MegaRamp => 2,
    }
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
    let kind_val = common_value(sel().map(|r| ramp_kind_index(r.kind)));
    let mut profile_rows = vec![
        stepper(Prop::RampHeight, "Height", common_value(sel().map(|r| r.height)), first.height, RAMP_HEIGHT_RANGE, 0.1, "m"),
        stepper(Prop::RampPitch, "Pitch", common_value(sel().map(|r| r.ramp_angle_deg)), first.ramp_angle_deg, RAMP_PITCH_RANGE, 1.0, "°"),
    ];
    match first.kind {
        RampKind::Ramp => {
            profile_rows.push(Row::Buttons(vec![
                Button { label: fit_pitch, edit: Edit::Do(Action::FitPitch), highlight: has_flat },
                Button { label: fit_height, edit: Edit::Do(Action::FitHeight), highlight: has_flat },
            ]));
        }
        RampKind::Whoops => {
            profile_rows.push(stepper(
                Prop::RampWhoopSpacing,
                "Spacing",
                common_value(sel().map(|r| r.whoop_spacing)),
                first.whoop_spacing,
                RAMP_WHOOP_SPACING_RANGE,
                0.5,
                "m",
            ));
            profile_rows.push(Row::Info(format!("Mogul ridges: {} jumps", first.whoop_count())));
        }
        RampKind::MegaRamp => {
            profile_rows.push(stepper(
                Prop::RampMegaGap,
                "Gap",
                common_value(sel().map(|r| r.effective_gap_length())),
                first.effective_gap_length(),
                RAMP_MEGA_GAP_RANGE,
                1.0,
                "m",
            ));
            let (s_t, s_l) = first.mega_segment_bounds();
            let len = first.length();
            profile_rows.push(Row::Info(format!(
                "Take-off {:.1}m • Gap {:.1}m • Landing {:.1}m",
                s_t * len,
                (s_l - s_t) * len,
                (1.0 - s_l) * len
            )));
        }
    }
    profile_rows.push(Row::RampProfile(Box::new(first.clone())));

    let singular_title = first.kind.label();
    Some(InspectorModel {
        title: title_for(singular_title, "Jump Components", &indices),
        count: indices.len(),
        subtitle: None,
        footer: FOOTER,
        sections: vec![
            Section::new("ramp.shape", "SHAPE", vec![
                Row::Segmented { prop: Prop::RampKind, label: "Kind", labels: &RAMP_KIND_LABELS, value: kind_val },
                stepper(Prop::RampAngle, "Angle", common_value(sel().map(|r| r.angle_deg())), first.angle_deg(), RAMP_ANGLE_RANGE, 1.0, "°"),
                stepper(Prop::RampLength, "Length", common_value(sel().map(|r| r.length())), first.length(), RAMP_LENGTH_RANGE, 0.5, "m"),
                stepper(Prop::RampWidth, "Width", common_value(sel().map(|r| r.width())), first.width(), RAMP_WIDTH_RANGE, 0.5, "m"),
            ]),
            Section::new("ramp.profile", "PROFILE", profile_rows),
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

/// Pit lane layout controls (spec 101 Pillar VII).
fn pit_layout_sections(state: &EditorState, tools: &ToolSettings) -> Vec<Section> {
    let mut rows = vec![Row::Segmented { prop: Prop::PitMode, label: "Mode", labels: &PIT_MODE_LABELS, value: Common::Same(usize::from(!tools.pit_layout_mode)) }];
    let Some(layout) = state.track.pit_lane_layout.as_ref().filter(|_| tools.pit_layout_mode) else {
        if tools.pit_layout_mode {
            let hint = if tools.pit_layout_entry.is_some() { "Click the main track at the pit exit." } else { "Click the main track at the pit entry." };
            rows.push(Row::Info(hint.to_string()));
        }
        return vec![Section::new("pit.mode", "PIT LANE TOOL", rows)];
    };
    if let Some(err) = &tools.pit_layout_error {
        rows.push(Row::Error(format!("Guard failed: {err}")));
    }
    let shape = |s: JunctionShape| usize::from(matches!(s, JunctionShape::TurnOff { .. }));
    let angle = |s: JunctionShape| match s {
        JunctionShape::TurnOff { angle_deg } => angle_deg,
        JunctionShape::Taper => DEFAULT_TURNOFF_ANGLE,
    };
    let junction_rows = |shape_prop, length_prop, angle_prop, j: &JunctionComponent| {
        vec![
            Row::Segmented { prop: shape_prop, label: "Shape", labels: &PIT_SHAPE_LABELS, value: Common::Same(shape(j.kind)) },
            stepper(length_prop, "Length", Common::Same(j.length), j.length, PIT_JUNCTION_LENGTH_RANGE, 5.0, "m"),
            stepper(angle_prop, "Angle", Common::Same(angle(j.kind)), angle(j.kind), PIT_ANGLE_RANGE, 1.0, "°"),
        ]
    };
    let mut sections = vec![Section::new("pit.mode", "PIT LANE TOOL", rows)];
    sections.push(Section::new("pit.entry", "ENTRY JUNCTION", junction_rows(Prop::PitEntryShape, Prop::PitEntryLength, Prop::PitEntryAngle, &layout.entry)));
    sections.push(Section::new("pit.exit", "EXIT JUNCTION", junction_rows(Prop::PitExitShape, Prop::PitExitLength, Prop::PitExitAngle, &layout.exit)));
    let row = &layout.box_row;
    sections.push(Section::new("pit.road", "PIT ROAD", vec![
        stepper(Prop::PitGap, "Gap", Common::Same(layout.entry.divider_gap), layout.entry.divider_gap, PIT_GAP_RANGE, 0.5, "m"),
        stepper(Prop::PitRoadWidth, "Width", Common::Same(layout.road_width), layout.road_width, PIT_ROAD_WIDTH_RANGE, 0.5, "m"),
        stepper(Prop::PitBoxStart, "Box start", Common::Same(row.start_s), row.start_s, PIT_BOX_START_RANGE, 1.0, "m"),
        stepper(Prop::PitBoxCount, "Boxes", Common::Same(row.count as f32), row.count as f32, PIT_BOX_COUNT_RANGE, 1.0, ""),
        stepper(Prop::PitBoxSpacing, "Spacing", Common::Same(row.spacing), row.spacing, PIT_BOX_SPACING_RANGE, 1.0, "m"),
        Row::Toggle { prop: Prop::PitGarages, label: "Garages", value: Common::Same(row.garages) },
    ]));
    sections
}

fn build_pit(state: &EditorState, tools: &ToolSettings) -> InspectorModel {
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
        sections: std::iter::once(Section::new("pit.info", "PIT LANE", rows)).chain(pit_layout_sections(state, tools)).collect(),
    }
}

fn build_track(state: &EditorState, tools: &ToolSettings) -> InspectorModel {
    let mut sections = Vec::new();
    let default_ro = state.track.default_runoff_surface().unwrap_or(state.track.default_surface);
    match tools.active_tool {
        EditorToolType::RoadSpline => {
            let bank = tools.new_waypoint_bank_angle;
            let dist = tools.new_waypoint_left_wall_distance.unwrap_or(state.barrier_offset);
            let wall = tools.new_waypoint_wall_type.unwrap_or(state.barrier_type);
            let runoff = tools.new_waypoint_left_runoff_surface.unwrap_or(default_ro);
            sections.push(Section::new("tool.spline", "NEW WAYPOINTS", vec![
                Row::Dropdown { prop: Prop::ToolSurface, label: "Surface", options: Options::Surfaces, value: Common::Same(surface_index(tools.active_surface)) },
                Row::Dropdown { prop: Prop::ToolRunoff, label: "Runoff", options: Options::Surfaces, value: Common::Same(surface_index(runoff)) },
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
        EditorToolType::LaunchChute => sections.push(chute_section(state, tools, "tool.chute", "LAUNCH CHUTE")),
        EditorToolType::JumpRamp | EditorToolType::WhoopSection | EditorToolType::StuntRamp => {
            let kind = tools.effective_ramp_kind();
            let preview = match kind {
                RampKind::Ramp => JumpRamp::new(
                    0,
                    SurfaceShape::OrientedBox { center: glam::Vec2::ZERO, half_extents: glam::Vec2::new(6.0, 4.0), angle: 0.0 },
                    glam::Vec2::X,
                    4.0,
                    15.0,
                    1.8,
                    "Preview Ramp",
                )
                .with_kind(RampKind::Ramp)
                .with_surface(tools.active_surface),
                RampKind::Whoops => JumpRamp::new(
                    0,
                    SurfaceShape::OrientedBox { center: glam::Vec2::ZERO, half_extents: glam::Vec2::new(15.0, tools.whoop_width * 0.5), angle: 0.0 },
                    glam::Vec2::X,
                    4.0,
                    14.0,
                    tools.whoop_height,
                    "Preview Whoops",
                )
                .with_kind(RampKind::Whoops)
                .with_whoop_spacing(tools.whoop_spacing)
                .with_surface(tools.active_surface),
                RampKind::MegaRamp => JumpRamp::new(
                    0,
                    SurfaceShape::OrientedBox { center: glam::Vec2::ZERO, half_extents: glam::Vec2::new(18.0, 6.0), angle: 0.0 },
                    glam::Vec2::X,
                    28.0 * tools.stunt_ramp_multiplier,
                    35.0,
                    tools.stunt_ramp_height,
                    "Preview Mega Ramp",
                )
                .with_kind(RampKind::MegaRamp)
                .with_gap_length(tools.mega_gap_length)
                .with_surface(tools.active_surface),
            };
            let mut rows = vec![
                Row::Segmented { prop: Prop::ToolRampKind, label: "Kind", labels: &RAMP_KIND_LABELS, value: Common::Same(ramp_kind_index(kind)) },
                Row::Dropdown { prop: Prop::ToolSurface, label: "Surface", options: Options::Surfaces, value: Common::Same(surface_index(tools.active_surface)) },
            ];
            match kind {
                RampKind::Ramp => {
                    rows.push(Row::Info("Right-click and drag to place a launch ramp.".to_string()));
                }
                RampKind::Whoops => {
                    rows.push(stepper(Prop::ToolWhoopSpacing, "Spacing", Common::Same(tools.whoop_spacing), tools.whoop_spacing, RAMP_WHOOP_SPACING_RANGE, 0.5, "m"));
                    rows.push(stepper(Prop::ToolWhoopHeight, "Height", Common::Same(tools.whoop_height), tools.whoop_height, (0.3, 2.5), 0.1, "m"));
                    rows.push(stepper(Prop::ToolWhoopWidth, "Width", Common::Same(tools.whoop_width), tools.whoop_width, RAMP_WIDTH_RANGE, 1.0, "m"));
                }
                RampKind::MegaRamp => {
                    rows.push(stepper(Prop::ToolMegaHeight, "Height", Common::Same(tools.stunt_ramp_height), tools.stunt_ramp_height, RAMP_HEIGHT_RANGE, 0.5, "m"));
                    rows.push(stepper(Prop::ToolMegaGap, "Gap", Common::Same(tools.mega_gap_length), tools.mega_gap_length, RAMP_MEGA_GAP_RANGE, 1.0, "m"));
                    rows.push(stepper(Prop::ToolMegaBoost, "Boost", Common::Same(tools.stunt_ramp_multiplier), tools.stunt_ramp_multiplier, RAMP_MEGA_BOOST_RANGE, 0.1, "x"));
                }
            }
            rows.push(Row::RampProfile(Box::new(preview)));
            sections.push(Section::new("tool.ramp", "NEW RAMPS", rows));
        }
        _ => {}
    }

    let offset = state.barrier_offset;
    let wall_chips = WALL_OFFSET_PRESETS
        .iter()
        .map(|&m| Chip { label: if m.fract() == 0.0 { format!("{m:.0} m") } else { format!("{m:.1}") }, edit: Edit::Set(Prop::GlobalWallOffset, m), active: (offset - m).abs() < 0.05 })
        .collect();
    let global_runoff = if state.track.spline.waypoints.is_empty() {
        Common::Same(surface_index(default_ro))
    } else {
        common_value(
            state
                .track
                .spline
                .waypoints
                .iter()
                .flat_map(|w| [w.left_runoff_surface.unwrap_or(default_ro), w.right_runoff_surface.unwrap_or(default_ro)])
                .map(surface_index),
        )
    };
    sections.push(Section::new("circuit.walls", "CIRCUIT WALLS", vec![
        stepper(Prop::GlobalWallOffset, "Offset", Common::Same(offset), offset, GLOBAL_WALL_OFFSET_RANGE, 0.5, "m"),
        Row::Chips { label: "Presets", chips: wall_chips },
        Row::Segmented { prop: Prop::GlobalWallType, label: "Type", labels: &WALL_TYPE_LABELS, value: Common::Same(wall_type_index(state.barrier_type)) },
        Row::Dropdown { prop: Prop::GlobalRunoff, label: "Runoff", options: Options::Surfaces, value: global_runoff },
    ]));

    let grid = state.grid_count();
    let off_track = off_track_surfaces().iter().position(|&s| s == state.track.default_surface);
    let grid_chips = GRID_PRESETS.iter().map(|&n| Chip { label: n.to_string(), edit: Edit::Set(Prop::GridSlots, n as f32), active: grid == n }).collect();
    // A launch chute holds the grid: its slots are the chute's, not a count to set.
    let grid_rows = if state.track.launch_chute().is_some() {
        vec![Row::Info(format!("Grid: {grid} slots on the launch chute"))]
    } else {
        vec![stepper(Prop::GridSlots, "Grid", Common::Same(grid as f32), grid as f32, GRID_SLOTS_RANGE, 1.0, " slots"), Row::Chips { label: "Presets", chips: grid_chips }]
    };
    if state.track.launch_chute().is_some() && tools.active_tool != EditorToolType::LaunchChute {
        sections.push(chute_section(state, tools, "circuit.chute", "LAUNCH CHUTE"));
    }
    let mut circuit_rows = vec![
        Row::Dropdown { prop: Prop::OffTrack, label: "Off-track", options: Options::OffTrack, value: off_track.map_or(Common::Mixed, Common::Same) },
        Row::Dropdown { prop: Prop::Category, label: "Category", options: Options::Category, value: CarCategory::ALL.iter().position(|&c| c == state.track.car_category).map_or(Common::Mixed, Common::Same) },
    ];
    circuit_rows.extend(grid_rows);
    circuit_rows.push(Row::Buttons(vec![
            Button { label: "Auto checkpoints".to_string(), edit: Edit::Do(Action::AutoCheckpoints), highlight: false },
            Button { label: "Rebuild".to_string(), edit: Edit::Do(Action::RebuildGeometry), highlight: false },
        ]));
    sections.push(Section::new("circuit", "CIRCUIT", circuit_rows));

    InspectorModel { title: "Circuit".to_string(), count: 0, subtitle: Some(tools.active_tool.title()), sections, footer: None }
}

/// The launch chute controls (spec 103): pad width, grid pattern and surface of the circuit's chute, or of the next
/// one the stamp tool places. A chute that is on the circuit shows its own values; any edit builds it again.
fn chute_section(state: &EditorState, tools: &ToolSettings, id: &'static str, title: &'static str) -> Section {
    let placed = state.track.launch_chute_spec();
    let (width, pattern, surface) = placed.map_or((tools.chute_pad_width, tools.chute_pattern, tools.chute_surface), |s| (s.pad_width, s.pattern, s.surface));
    let mut rows = Vec::new();
    match (&placed, &tools.chute_status) {
        (_, Some(status)) => rows.push(Row::Info(status.clone())),
        (None, None) => rows.push(Row::Info("Click a waypoint before the finish line.".to_string())),
        (Some(_), None) => {}
    }
    rows.push(stepper(Prop::ChutePadWidth, "Pad width", Common::Same(width), width, PAD_WIDTH_RANGE, 0.5, "m"));
    rows.push(Row::Segmented {
        prop: Prop::ChutePattern,
        label: "Grid",
        labels: &CHUTE_PATTERN_LABELS,
        value: CHUTE_PATTERNS.iter().position(|&p| p == pattern).map_or(Common::Mixed, Common::Same),
    });
    rows.push(Row::Segmented {
        prop: Prop::ChuteSurface,
        label: "Surface",
        labels: &CHUTE_SURFACE_LABELS,
        value: CHUTE_SURFACES.iter().position(|&s| s == surface).map_or(Common::Mixed, Common::Same),
    });
    rows.push(Row::Buttons(match placed {
        Some(_) => vec![Button { label: "Remove chute".to_string(), edit: Edit::Do(Action::RemoveLaunchChute), highlight: false }],
        None => vec![Button { label: "+ Insert launch chute".to_string(), edit: Edit::Do(Action::InsertLaunchChute), highlight: true }],
    }));
    Section::new(id, title, rows)
}

/// Delay before a hovered inspector control shows its tooltip, in seconds.
pub const TOOLTIP_DELAY: f64 = 0.5;

/// Tooltip text for a row: its name plus keyboard shortcut, or a usage hint for steppers.
pub fn row_tooltip(row: &Row) -> Option<String> {
    match row {
        Row::Stepper { prop: Prop::WpBanking, label, .. } => Some(format!("{label}: [ / ] ±1°, Shift+[ / ] ±5°")),
        Row::Stepper { prop: Prop::RampAngle, label, .. } => Some(format!("{label}: R / Shift+R rotate 15°, comma / period rotate 5°")),
        Row::Stepper { label, .. } => Some(format!("{label}: drag, or click to type. Shift+-/+ = 5 steps")),
        Row::Chips { label, chips } if chips.iter().any(|c| matches!(c.edit, Edit::Set(Prop::WpBanking | Prop::ToolBanking, _))) => {
            Some(format!("{label}: B cycles 0/10/18/22°, Shift+B inverts"))
        }
        Row::Segmented { prop: Prop::ZoneLayer, label, .. } => Some(format!("{label}: Ctrl+F front, Ctrl+B back")),
        Row::Warning { tooltip: Some(tip), .. } => Some(tip.clone()),
        _ => None,
    }
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
        (_, Edit::Do(Action::ShowKind(kind))) => tools.inspector.mixed_kind = Some(kind),
        (Some(WpWidth | WpBanking | WpLeftCurb | WpRightCurb | WpLeftWall | WpRightWall | WpWallDistL | WpWallDistR | WpWallType | WpSurface | WpRunoffL | WpRunoffR), _)
        | (_, Edit::Do(Action::InvertBanking)) => apply_waypoint_edit(state, tools, edit),
        (Some(ZoneSurface | ZoneLayer), _) => apply_zone_edit(state, edit),
        (Some(RampKind | RampAngle | RampLength | RampWidth | RampHeight | RampPitch | RampWhoopSpacing | RampMegaGap | RampSurface), _)
        | (_, Edit::Do(Action::FitPitch | Action::FitHeight)) => {
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
        (Some(PitMode | PitEntryShape | PitEntryLength | PitEntryAngle | PitExitShape | PitExitLength | PitExitAngle | PitGap | PitRoadWidth | PitBoxStart | PitBoxCount | PitBoxSpacing | PitGarages), _) => {
            apply_pit_layout_edit(state, tools, edit)
        }
        _ => apply_track_edit(state, tools, edit),
    }
}

/// Applies a pit layout edit and recompiles the layout (spec 101).
fn apply_pit_layout_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    if let Edit::Pick(Prop::PitMode, k) = edit {
        tools.pit_layout_mode = k == 0;
        tools.pit_layout_entry = None;
        return;
    }
    let step = |v: f32, e: Edit, range: (f32, f32)| match e {
        Edit::Set(_, x) => clamp_to(x, range),
        Edit::Step(_, d) => clamp_to(v + d, range),
        _ => v,
    };
    let shape_of = |k: usize, current: JunctionShape| match (k, current) {
        (0, _) => JunctionShape::Taper,
        (_, JunctionShape::TurnOff { angle_deg }) => JunctionShape::TurnOff { angle_deg },
        _ => JunctionShape::TurnOff { angle_deg: DEFAULT_TURNOFF_ANGLE },
    };
    let set_angle = |j: &mut JunctionComponent, e: Edit| {
        if let JunctionShape::TurnOff { angle_deg } = j.kind {
            j.kind = JunctionShape::TurnOff { angle_deg: step(angle_deg, e, PIT_ANGLE_RANGE) };
        }
    };
    tools.update_pit_layout(state, |l| match edit {
        Edit::Pick(Prop::PitEntryShape, k) => l.entry.kind = shape_of(k, l.entry.kind),
        Edit::Pick(Prop::PitExitShape, k) => l.exit.kind = shape_of(k, l.exit.kind),
        Edit::Set(Prop::PitEntryLength, _) | Edit::Step(Prop::PitEntryLength, _) => l.entry.length = step(l.entry.length, edit, PIT_JUNCTION_LENGTH_RANGE),
        Edit::Set(Prop::PitExitLength, _) | Edit::Step(Prop::PitExitLength, _) => l.exit.length = step(l.exit.length, edit, PIT_JUNCTION_LENGTH_RANGE),
        Edit::Set(Prop::PitEntryAngle, _) | Edit::Step(Prop::PitEntryAngle, _) => set_angle(&mut l.entry, edit),
        Edit::Set(Prop::PitExitAngle, _) | Edit::Step(Prop::PitExitAngle, _) => set_angle(&mut l.exit, edit),
        Edit::Set(Prop::PitGap, _) | Edit::Step(Prop::PitGap, _) => {
            let gap = step(l.entry.divider_gap, edit, PIT_GAP_RANGE);
            l.entry.divider_gap = gap;
            l.exit.divider_gap = gap;
        }
        Edit::Set(Prop::PitRoadWidth, _) | Edit::Step(Prop::PitRoadWidth, _) => l.road_width = step(l.road_width, edit, PIT_ROAD_WIDTH_RANGE),
        Edit::Set(Prop::PitBoxStart, _) | Edit::Step(Prop::PitBoxStart, _) => l.box_row.start_s = step(l.box_row.start_s, edit, PIT_BOX_START_RANGE),
        Edit::Set(Prop::PitBoxCount, _) | Edit::Step(Prop::PitBoxCount, _) => l.box_row.count = step(l.box_row.count as f32, edit, PIT_BOX_COUNT_RANGE).round() as u32,
        Edit::Set(Prop::PitBoxSpacing, _) | Edit::Step(Prop::PitBoxSpacing, _) => l.box_row.spacing = step(l.box_row.spacing, edit, PIT_BOX_SPACING_RANGE),
        Edit::Flag(Prop::PitGarages, on) => l.box_row.garages = on,
        _ => {}
    });
}

fn apply_waypoint_edit(state: &mut EditorState, tools: &mut ToolSettings, edit: Edit) {
    let len = state.track.spline.waypoints.len();
    let indices: Vec<usize> = state.selection.selected_waypoint_indices().into_iter().filter(|&i| i < len).collect();
    let branch_indices: Vec<(SegmentId, usize)> = state
        .selection
        .selected_branch_waypoints()
        .into_iter()
        .filter(|&(sid, i)| {
            state
                .track
                .network
                .as_ref()
                .and_then(|n| n.get_segment(sid))
                .is_some_and(|s| i < s.waypoints.len())
        })
        .collect();
    if indices.is_empty() && branch_indices.is_empty() {
        return;
    }
    state.record_undo();
    let offset = state.barrier_offset;
    let mutate_wp = |wp: &mut TrackWaypoint| match edit {
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
        Edit::Pick(Prop::WpRunoffL, k) => wp.left_runoff_surface = SURFACES.get(k).copied(),
        Edit::Pick(Prop::WpRunoffR, k) => wp.right_runoff_surface = SURFACES.get(k).copied(),
        _ => {}
    };

    for &i in &indices {
        mutate_wp(&mut state.track.spline.waypoints[i]);
    }
    let mut edited_segments = Vec::new();
    if let Some(ref mut net) = state.track.network {
        for &(sid, i) in &branch_indices {
            if let Some(seg) = net.get_segment_mut(sid) {
                if let Some(wp) = seg.waypoints.get_mut(i) {
                    mutate_wp(wp);
                    edited_segments.push(sid);
                }
            }
        }
    }

    // Edits also become the placement defaults for new waypoints, as before.
    let first_wp = indices
        .first()
        .map(|&i| state.track.spline.waypoints[i].clone())
        .or_else(|| {
            let &(sid, i) = branch_indices.first()?;
            state.track.network.as_ref()?.get_segment(sid)?.waypoints.get(i).cloned()
        });
    if let Some(wp) = first_wp {
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
            Edit::Pick(Prop::WpRunoffL, _) => tools.new_waypoint_left_runoff_surface = wp.left_runoff_surface,
            Edit::Pick(Prop::WpRunoffR, _) => tools.new_waypoint_right_runoff_surface = wp.right_runoff_surface,
            _ => {}
        }
    }
    if !edited_segments.is_empty() {
        state.rebuild_branch_segments(&edited_segments);
    } else {
        state.rebuild_geometry();
    }
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
            Edit::Pick(Prop::RampKind, k) => {
                if let Some(&kind) = RampKind::ALL.get(k) {
                    ramp.kind = kind;
                    tools.active_ramp_kind = kind;
                }
            }
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
            Edit::Set(Prop::RampWhoopSpacing, v) => ramp.whoop_spacing = clamp_to(v, RAMP_WHOOP_SPACING_RANGE),
            Edit::Step(Prop::RampWhoopSpacing, d) => ramp.whoop_spacing = clamp_to(ramp.whoop_spacing + d, RAMP_WHOOP_SPACING_RANGE),
            Edit::Set(Prop::RampMegaGap, v) => ramp.gap_length = clamp_to(v, RAMP_MEGA_GAP_RANGE),
            Edit::Step(Prop::RampMegaGap, d) => ramp.gap_length = clamp_to(ramp.effective_gap_length() + d, RAMP_MEGA_GAP_RANGE),
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
        Edit::Pick(Prop::ToolRampKind, k) => {
            if let Some(&kind) = RampKind::ALL.get(k) {
                tools.active_ramp_kind = kind;
                tools.active_tool = EditorToolType::JumpRamp;
            }
        }
        Edit::Set(Prop::ToolWhoopSpacing, v) => tools.whoop_spacing = clamp_to(v, RAMP_WHOOP_SPACING_RANGE),
        Edit::Step(Prop::ToolWhoopSpacing, d) => tools.whoop_spacing = clamp_to(tools.whoop_spacing + d, RAMP_WHOOP_SPACING_RANGE),
        Edit::Set(Prop::ToolWhoopHeight, v) => tools.whoop_height = clamp_to(v, (0.3, 2.5)),
        Edit::Step(Prop::ToolWhoopHeight, d) => tools.whoop_height = clamp_to(tools.whoop_height + d, (0.3, 2.5)),
        Edit::Set(Prop::ToolWhoopWidth, v) => tools.whoop_width = clamp_to(v, RAMP_WIDTH_RANGE),
        Edit::Step(Prop::ToolWhoopWidth, d) => tools.whoop_width = clamp_to(tools.whoop_width + d, RAMP_WIDTH_RANGE),
        Edit::Set(Prop::ToolMegaHeight, v) => tools.stunt_ramp_height = clamp_to(v, RAMP_HEIGHT_RANGE),
        Edit::Step(Prop::ToolMegaHeight, d) => tools.stunt_ramp_height = clamp_to(tools.stunt_ramp_height + d, RAMP_HEIGHT_RANGE),
        Edit::Set(Prop::ToolMegaGap, v) => tools.mega_gap_length = clamp_to(v, RAMP_MEGA_GAP_RANGE),
        Edit::Step(Prop::ToolMegaGap, d) => tools.mega_gap_length = clamp_to(tools.mega_gap_length + d, RAMP_MEGA_GAP_RANGE),
        Edit::Set(Prop::ToolMegaBoost, v) => tools.stunt_ramp_multiplier = clamp_to(v, RAMP_MEGA_BOOST_RANGE),
        Edit::Step(Prop::ToolMegaBoost, d) => tools.stunt_ramp_multiplier = clamp_to(tools.stunt_ramp_multiplier + d, RAMP_MEGA_BOOST_RANGE),
        Edit::Pick(Prop::ToolSurface, k) => {
            if let Some(&s) = SURFACES.get(k) {
                tools.active_surface = s;
            }
        }
        Edit::Pick(Prop::ToolRunoff, k) => {
            if let Some(&s) = SURFACES.get(k) {
                tools.new_waypoint_left_runoff_surface = Some(s);
                tools.new_waypoint_right_runoff_surface = Some(s);
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
        Edit::Pick(Prop::GlobalRunoff, k) => {
            if let Some(&s) = SURFACES.get(k) {
                state.record_undo();
                for wp in &mut state.track.spline.waypoints {
                    wp.left_runoff_surface = Some(s);
                    wp.right_runoff_surface = Some(s);
                }
                tools.new_waypoint_left_runoff_surface = Some(s);
                tools.new_waypoint_right_runoff_surface = Some(s);
                state.rebuild_geometry();
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
        Edit::Set(Prop::ChutePadWidth, _) | Edit::Step(Prop::ChutePadWidth, _) | Edit::Pick(Prop::ChutePattern | Prop::ChuteSurface, _) => {
            // Start from the chute that is on the circuit, so editing one control keeps the others.
            if let Some(placed) = state.track.launch_chute_spec() {
                tools.chute_pad_width = placed.pad_width;
                tools.chute_pattern = placed.pattern;
                tools.chute_surface = placed.surface;
            }
            match edit {
                Edit::Set(_, v) => tools.chute_pad_width = clamp_to(v, PAD_WIDTH_RANGE),
                Edit::Step(_, d) => tools.chute_pad_width = clamp_to(tools.chute_pad_width + d, PAD_WIDTH_RANGE),
                Edit::Pick(Prop::ChutePattern, k) => tools.chute_pattern = CHUTE_PATTERNS.get(k).copied().unwrap_or(tools.chute_pattern),
                Edit::Pick(_, k) => tools.chute_surface = CHUTE_SURFACES.get(k).copied().unwrap_or(tools.chute_surface),
                _ => {}
            }
            tools.restamp_launch_chute(state);
        }
        Edit::Do(Action::InsertLaunchChute) => {
            tools.auto_insert_launch_chute(state);
        }
        Edit::Do(Action::RemoveLaunchChute) => {
            tools.remove_launch_chute(state);
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
