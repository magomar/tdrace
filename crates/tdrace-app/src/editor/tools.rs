use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle_lines};
use macroquad::window::{screen_height, screen_width};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::Checkpoint;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, LineSegment, Obstacle, PitBox, PitLane, SurfaceLayer, SurfaceShape, SurfaceZone, WallBarrier};
use tdrace_core::track::network::{GoreConfig, JunctionId, JunctionKind, MergeConfig, RoadJunction, RoadSegment, SegmentId, SocketId, SplineSocket, TrackLayout};
use tdrace_core::track::junction_kit::{JunctionComponent, JunctionShape, Side};
use tdrace_core::track::pit_kit::{self, PitBoxRow, PitLaneLayout};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{CarCategory, ChuteSide, LaunchChuteSpec, PackedGridPattern, Track, TrackKind};

use super::camera::EditorCamera;
use super::inspector::{
    InspectorView, BANKING_RANGE, GLOBAL_WALL_OFFSET_RANGE, RAMP_HEIGHT_RANGE, RAMP_LENGTH_RANGE, RAMP_PITCH_RANGE, RAMP_WIDTH_RANGE,
    WALL_DISTANCE_RANGE, WIDTH_RANGE,
};
use super::state::{EditorState, Selection};
use crate::render::color::Palette;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// How near a click must be to a waypoint to pick it for a launch chute (m).
const LAUNCH_CHUTE_PICK_M: f32 = 8.0;

/// The waypoint of the circuit nearest `point`, within `radius`.
fn nearest_waypoint(state: &EditorState, point: Vec2, radius: f32) -> Option<usize> {
    state
        .track
        .spline
        .waypoints
        .iter()
        .enumerate()
        .map(|(i, w)| (i, w.point.distance(point)))
        .filter(|(_, d)| *d <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Available editor tools in the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorToolType {
    Select,
    RoadSpline,
    RoadSplit,
    SurfaceZone,
    JumpRamp,
    Obstacle,
    Checkpoint,
    PitLane,
    ArenaFloor,
    WhoopSection,
    StuntRamp,
    LaunchChute,
}

impl EditorToolType {
    pub const ALL: [Self; 12] = [
        Self::Select,
        Self::RoadSpline,
        Self::RoadSplit,
        Self::SurfaceZone,
        Self::JumpRamp,
        Self::Obstacle,
        Self::Checkpoint,
        Self::PitLane,
        Self::ArenaFloor,
        Self::WhoopSection,
        Self::StuntRamp,
        Self::LaunchChute,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Select => "Select & Move [1]",
            Self::RoadSpline => "Road Spline [2]",
            Self::RoadSplit => "Road Split [3]",
            Self::SurfaceZone => "Surfaces & Hazards [4]",
            Self::JumpRamp => "Jump Ramp [5]",
            Self::Obstacle => "Obstacle Prop [6]",
            Self::Checkpoint => "Checkpoint Gate [7]",
            Self::PitLane => "Pit Lane [8]",
            Self::ArenaFloor => "Arena Floor [9]",
            Self::WhoopSection => "Whoop Section [0]",
            Self::StuntRamp => "Stunt Mega Ramp [-]",
            Self::LaunchChute => "Launch Chute [L]",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Select => "1",
            Self::RoadSpline => "2",
            Self::RoadSplit => "3",
            Self::SurfaceZone => "4",
            Self::JumpRamp => "5",
            Self::Obstacle => "6",
            Self::Checkpoint => "7",
            Self::PitLane => "8",
            Self::ArenaFloor => "9",
            Self::WhoopSection => "0",
            Self::StuntRamp => "-",
            Self::LaunchChute => "L",
        }
    }
}

/// Active settings for tool placement.
#[derive(Debug, Clone)]
pub struct ToolSettings {
    pub active_tool: EditorToolType,
    pub active_surface: SurfaceType,
    pub active_surface_shape: SurfaceShapeType,
    pub active_surface_layer: SurfaceLayer,
    pub active_obstacle_shape: ObstacleShapeType,
    pub active_polygon_vertices: Vec<Vec2>,
    pub active_pit_waypoints: Vec<Vec2>,
    pub active_pit_boxes: Vec<PitBox>,
    /// Pit Lane tool mode: layout (spec 101, default) or free-form.
    pub pit_layout_mode: bool,
    /// Layout mode: main-spline arc length and side of the first click (the entry), until the second click.
    pub pit_layout_entry: Option<(f32, Side)>,
    /// Layout mode: guard error of the current layout. The preview is hidden while it is set.
    pub pit_layout_error: Option<String>,
    /// Layout mode: index of the road waypoint being dragged.
    pub drag_pit_road_point: Option<usize>,
    pub new_waypoint_width: f32,
    pub new_waypoint_left_curb: bool,
    pub new_waypoint_right_curb: bool,
    pub new_waypoint_left_wall: bool,
    pub new_waypoint_right_wall: bool,
    pub new_waypoint_left_wall_distance: Option<f32>,
    pub new_waypoint_right_wall_distance: Option<f32>,
    pub new_waypoint_wall_type: Option<BarrierType>,
    pub new_waypoint_bank_angle: f32,

    // Arena & Stunt settings
    pub active_arena_barrier: Option<BarrierType>,
    pub whoop_spacing: f32,
    pub whoop_height: f32,
    pub whoop_width: f32,
    pub stunt_ramp_height: f32,
    pub stunt_ramp_multiplier: f32,

    // Dragging / selection interaction state
    pub is_dragging: bool,
    pub is_box_selecting: bool,
    pub is_placing: bool,
    pub is_rotating_ramp: bool,
    pub drag_rotating_ramp_idx: Option<usize>,
    pub drag_start_world: Vec2,
    pub drag_current_world: Vec2,
    pub drag_initial_entity_pos: Vec2,
    pub drag_initial_waypoints: Vec<(usize, Vec2)>,
    pub drag_initial_surface_zones: Vec<(usize, Vec2)>,
    pub drag_initial_obstacles: Vec<(usize, Vec2)>,
    pub drag_initial_jump_ramps: Vec<(usize, Vec2)>,
    pub drag_initial_checkpoints: Vec<(usize, Vec2, Vec2)>,
    pub drag_initial_grid_slots: Vec<(usize, Vec2)>,
    pub drag_initial_pit_box: Option<(Vec2, Vec2)>,

    // Bar control selection and inline manual text editing
    pub selected_bar: Option<String>,
    pub editing_bar: Option<(String, String)>,
    pub inspector: InspectorView,

    // Road split and branching track settings
    pub active_branch_socket: Option<SocketId>,
    pub split_divergence_angle: f32,
    pub split_branch_count: usize,

    // Launch chute stamp settings (spec 103)
    pub chute_pad_width: f32,
    pub chute_pad_length: f32,
    pub chute_pattern: PackedGridPattern,
    pub chute_surface: SurfaceType,
    /// What the last chute insertion did (the reason when it was refused), shown in the inspector.
    pub chute_status: Option<String>,

    /// Set when this frame's Escape already cancelled an unfinished polygon, so the
    /// editor UI does not also treat it as "exit the editor".
    pub escape_consumed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceShapeType {
    Square,
    Circle,
    Triangle,
    Polygon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleShapeType {
    Circle,
    Box,
    Polygon,
}

impl Default for ToolSettings {
    fn default() -> Self {
        Self {
            active_tool: EditorToolType::Select,
            active_surface: SurfaceType::Asphalt,
            active_surface_shape: SurfaceShapeType::Square,
            active_surface_layer: SurfaceLayer::BelowTrack,
            active_obstacle_shape: ObstacleShapeType::Circle,
            active_polygon_vertices: Vec::new(),
            active_pit_waypoints: Vec::new(),
            active_pit_boxes: Vec::new(),
            pit_layout_mode: true,
            pit_layout_entry: None,
            pit_layout_error: None,
            drag_pit_road_point: None,
            new_waypoint_width: 14.0,
            new_waypoint_left_curb: false,
            new_waypoint_right_curb: false,
            new_waypoint_left_wall: true,
            new_waypoint_right_wall: true,
            new_waypoint_left_wall_distance: None,
            new_waypoint_right_wall_distance: None,
            new_waypoint_wall_type: None,
            new_waypoint_bank_angle: 0.0,
            active_arena_barrier: Some(BarrierType::Concrete),
            whoop_spacing: 5.0,
            whoop_height: 0.70,
            whoop_width: 14.0,
            stunt_ramp_height: 3.5,
            stunt_ramp_multiplier: 1.5,
            is_dragging: false,
            is_box_selecting: false,
            is_placing: false,
            is_rotating_ramp: false,
            drag_rotating_ramp_idx: None,
            drag_start_world: Vec2::ZERO,
            drag_current_world: Vec2::ZERO,
            drag_initial_entity_pos: Vec2::ZERO,
            drag_initial_waypoints: Vec::new(),
            drag_initial_surface_zones: Vec::new(),
            drag_initial_obstacles: Vec::new(),
            drag_initial_jump_ramps: Vec::new(),
            drag_initial_checkpoints: Vec::new(),
            drag_initial_grid_slots: Vec::new(),
            drag_initial_pit_box: None,
            selected_bar: None,
            editing_bar: None,
            inspector: InspectorView::default(),
            active_branch_socket: None,
            split_divergence_angle: 30.0,
            split_branch_count: 2,
            chute_pad_width: 16.0,
            chute_pad_length: 40.0,
            chute_pattern: PackedGridPattern::AutocrossFiveThree,
            chute_surface: SurfaceType::Concrete,
            chute_status: None,
            escape_consumed: false,
        }
    }
}

impl ToolSettings {
    pub fn is_editing_text(&self) -> bool {
        self.editing_bar.is_some()
    }

    pub fn is_bar_selected(&self, id: &str) -> bool {
        self.selected_bar.as_deref() == Some(id)
    }

    pub fn select_bar(&mut self, id: &str) {
        self.selected_bar = Some(id.to_string());
    }

    pub fn clear_bar_selection(&mut self) {
        self.selected_bar = None;
    }

    pub fn start_editing_bar(&mut self, id: &str, initial_text: &str) {
        self.selected_bar = Some(id.to_string());
        self.editing_bar = Some((id.to_string(), initial_text.to_string()));
    }

    pub fn stop_editing_bar(&mut self) {
        self.editing_bar = None;
    }

    /// Finalizes the current polygon vertices as an Arena perimeter boundary hull and floor.
    pub fn finalize_arena_hull(&mut self, state: &mut EditorState) -> bool {
        if self.active_polygon_vertices.len() < 3 {
            return false;
        }
        state.record_undo();
        let boundary_hull = std::mem::take(&mut self.active_polygon_vertices);
        let perimeter_barrier = self.active_arena_barrier;
        state.track.kind = TrackKind::Arena {
            boundary_hull: boundary_hull.clone(),
            floor_surface: self.active_surface,
            perimeter_barrier,
        };
        if let Some(barrier_type) = perimeter_barrier {
            for i in 0..boundary_hull.len() {
                let start = boundary_hull[i];
                let end = boundary_hull[(i + 1) % boundary_hull.len()];
                state.track.geometry.outer_walls.push(WallBarrier::new(start, end, barrier_type));
            }
        }
        let zone_name = format!("{:?} Arena Floor", self.active_surface);
        let floor_zone = SurfaceZone::new(
            SurfaceShape::Polygon { vertices: boundary_hull },
            self.active_surface,
            zone_name,
        )
        .with_layer(SurfaceLayer::BelowTrack);
        state.track.geometry.surface_zones.push(floor_zone);
        state.selection = Selection::None;
        state.revalidate();
        true
    }

    /// Layout mode (spec 101): the first click on the main track places the entry junction, the second the exit
    /// junction. The editor then builds a default layout on the clicked side and compiles it.
    pub fn add_pit_layout_click(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        let proj = state.track.spline.project_point(mouse_world);
        if state.track.spline.samples.len() < 2 || proj.distance_to_spline > proj.track_width * 0.5 + 2.0 {
            return;
        }
        let s = proj.progress_distance;
        let Some((entry_s, side)) = self.pit_layout_entry.take() else {
            // `lateral_offset` is positive to the right of the driving direction.
            let side = if proj.lateral_offset > 0.0 { Side::Right } else { Side::Left };
            self.pit_layout_entry = Some((s, side));
            return;
        };
        let layout = default_pit_layout(&state.track, entry_s, s, side);
        state.record_undo();
        self.install_pit_layout(state, layout);
    }

    /// Compiles `layout` into the track. On a guard error the layout is kept, the preview (pit lane) is removed and
    /// the error is shown.
    pub fn install_pit_layout(&mut self, state: &mut EditorState, layout: PitLaneLayout) {
        match layout.compile(&state.track) {
            Ok(compiled) => {
                pit_kit::install(&mut state.track, compiled);
                self.pit_layout_error = None;
            }
            Err(e) => {
                state.track.pit_lane = None;
                self.pit_layout_error = Some(format!("{e:?}"));
            }
        }
        state.track.pit_lane_layout = Some(layout);
        state.selection = Selection::PitBox;
        state.rebuild_geometry();
    }

    /// Edits the current layout and recompiles it, as one undo step.
    pub fn update_pit_layout(&mut self, state: &mut EditorState, edit: impl FnOnce(&mut PitLaneLayout)) {
        let Some(mut layout) = state.track.pit_lane_layout.clone() else { return };
        edit(&mut layout);
        state.record_undo();
        self.install_pit_layout(state, layout);
    }

    /// Handles left-click waypoint node placement for pit lane spline authoring (Spec 062).
    pub fn add_pit_lane_node(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if self.pit_layout_mode {
            self.add_pit_layout_click(state, mouse_world);
            return;
        }
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        // If at least 2 waypoints already exist, check if clicking near main track ribbon to merge
        if self.active_pit_waypoints.len() >= 2 {
            let proj = state.track.spline.project_point(snapped_mouse);
            if proj.distance_to_spline < proj.track_width * 0.75 {
                self.active_pit_waypoints.push(snapped_mouse);
                self.finalize_pit_lane(state);
                return;
            }
        }

        self.active_pit_waypoints.push(snapped_mouse);
    }

    /// Places a PitBox stall at cursor position (Shift + Left Click).
    pub fn place_pit_box_stall(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);
        let stall = PitBox::new(snapped_mouse, Vec2::X, 3.0, 0.0);

        if state.track.pit_lane.is_some() {
            state.record_undo();
            if let Some(lane) = &mut state.track.pit_lane {
                lane.pit_boxes.push(stall);
            }
            state.selection = Selection::PitBox;
            state.revalidate();
        } else {
            self.active_pit_boxes.push(stall);
        }
    }

    /// Finalizes in-progress pit lane waypoints into a dedicated PitLane on state.track.
    pub fn finalize_pit_lane(&mut self, state: &mut EditorState) -> bool {
        if self.active_pit_waypoints.len() < 2 {
            return false;
        }

        let waypoints = std::mem::take(&mut self.active_pit_waypoints);
        let mut pit_boxes = std::mem::take(&mut self.active_pit_boxes);

        state.record_undo();

        let mut spline_points = waypoints.clone();
        if spline_points.len() == 2 {
            spline_points.insert(1, (spline_points[0] + spline_points[1]) * 0.5);
        }

        let road_width = 6.0;
        let pit_spline = TrackSpline::from_points(&spline_points, road_width, false);

        let p0 = waypoints[0];
        let p1 = waypoints[1];
        let dir0 = (p1 - p0).normalize_or_zero();
        let norm0 = Vec2::new(-dir0.y, dir0.x);
        let entry_gate = LineSegment::new(p0 - norm0 * 3.0, p0 + norm0 * 3.0);

        let pn = *waypoints.last().unwrap();
        let pn_prev = waypoints[waypoints.len() - 2];
        let dirn = (pn - pn_prev).normalize_or_zero();
        let normn = Vec2::new(-dirn.y, dirn.x);
        let exit_gate = LineSegment::new(pn - normn * 3.0, pn + normn * 3.0);

        if pit_boxes.is_empty() {
            let mid = spline_points[spline_points.len() / 2];
            pit_boxes.push(PitBox::new(mid, dir0, 3.0, 0.0));
        }

        state.track.pit_lane = Some(PitLane::new(
            pit_spline,
            road_width,
            PitLane::DEFAULT_ROAD_SPEED_LIMIT,
            pit_boxes,
            entry_gate,
            exit_gate,
        ));
        state.selection = Selection::PitBox;
        state.revalidate();
        true
    }

    /// Selects all elements matching the active tool (or all track elements if Select tool is active).
    pub fn select_all_for_active_tool(&mut self, state: &mut EditorState) -> bool {
        let selection = match self.active_tool {
            EditorToolType::Select => {
                let waypoints = (0..state.track.spline.waypoints.len()).collect();
                let surface_zones = (0..state.track.geometry.surface_zones.len()).collect();
                let obstacles = (0..state.track.geometry.obstacles.len()).collect();
                let jump_ramps = (0..state.track.geometry.jump_ramps.len()).collect();
                let checkpoints = (0..state.track.checkpoints.len()).collect();
                let grid_slots = (0..state.track.grid_positions.len()).collect();
                let pit_box = state.track.pit_box_area.is_some();
                Selection::from_multi(
                    waypoints,
                    surface_zones,
                    obstacles,
                    jump_ramps,
                    checkpoints,
                    grid_slots,
                    pit_box,
                )
            }
            EditorToolType::RoadSpline | EditorToolType::RoadSplit | EditorToolType::LaunchChute => {
                let waypoints = (0..state.track.spline.waypoints.len()).collect();
                Selection::from_multi(waypoints, vec![], vec![], vec![], vec![], vec![], false)
            }
            EditorToolType::SurfaceZone => {
                let surface_zones = (0..state.track.geometry.surface_zones.len()).collect();
                Selection::from_multi(vec![], surface_zones, vec![], vec![], vec![], vec![], false)
            }
            EditorToolType::JumpRamp | EditorToolType::WhoopSection | EditorToolType::StuntRamp => {
                let jump_ramps = (0..state.track.geometry.jump_ramps.len()).collect();
                Selection::from_multi(vec![], vec![], vec![], jump_ramps, vec![], vec![], false)
            }
            EditorToolType::Obstacle => {
                let obstacles = (0..state.track.geometry.obstacles.len()).collect();
                Selection::from_multi(vec![], vec![], obstacles, vec![], vec![], vec![], false)
            }
            EditorToolType::Checkpoint => {
                let checkpoints = (0..state.track.checkpoints.len()).collect();
                Selection::from_multi(vec![], vec![], vec![], vec![], checkpoints, vec![], false)
            }
            EditorToolType::PitLane => {
                let pit_box = state.track.pit_box_area.is_some() || state.track.pit_lane.is_some();
                Selection::from_multi(vec![], vec![], vec![], vec![], vec![], vec![], pit_box)
            }
            EditorToolType::ArenaFloor => {
                Selection::None
            }
        };

        if selection.is_none() {
            false
        } else {
            state.select(selection);
            true
        }
    }

    /// Duplicates currently selected obstacle, surface zone, jump ramp, waypoint, checkpoint, or grid slot.
    pub fn duplicate_selected(&mut self, state: &mut EditorState) -> bool {
        match state.selection.clone() {
            Selection::Obstacle(idx) => {
                if let Some(obs) = state.track.geometry.obstacles.get(idx).cloned() {
                    state.record_undo();
                    let new_id = state.track.geometry.obstacles.len() + 1;
                    let mut copy = obs.clone();
                    copy.id = new_id;
                    copy.name = format!("{} (Copy)", obs.name);
                    let old_center = copy.center();
                    copy.set_center(old_center + Vec2::new(4.0, 4.0));
                    state.track.geometry.obstacles.push(copy);
                    state.selection = Selection::Obstacle(state.track.geometry.obstacles.len() - 1);
                    state.revalidate();
                    return true;
                }
            }
            Selection::SurfaceZone(idx) => {
                if let Some(zone) = state.track.geometry.surface_zones.get(idx).cloned() {
                    state.record_undo();
                    let mut copy = zone.clone();
                    copy.name = format!("{} (Copy)", zone.name);
                    let old_center = get_surface_shape_center(&copy.shape);
                    set_surface_zone_position(&mut copy, old_center + Vec2::new(4.0, 4.0));
                    state.track.geometry.surface_zones.push(copy);
                    state.selection = Selection::SurfaceZone(state.track.geometry.surface_zones.len() - 1);
                    state.revalidate();
                    return true;
                }
            }
            Selection::JumpRamp(idx) => {
                if let Some(ramp) = state.track.geometry.jump_ramps.get(idx).cloned() {
                    state.record_undo();
                    let new_id = state.track.geometry.jump_ramps.len() + 1;
                    let mut copy = ramp.clone();
                    copy.id = new_id;
                    copy.name = format!("{} (Copy)", ramp.name);
                    let old_center = get_surface_shape_center(&copy.shape);
                    set_jump_ramp_position(&mut copy, old_center + Vec2::new(4.0, 4.0));
                    state.track.geometry.jump_ramps.push(copy);
                    state.selection = Selection::JumpRamp(state.track.geometry.jump_ramps.len() - 1);
                    state.revalidate();
                    return true;
                }
            }
            Selection::Waypoint(idx) => {
                if idx < state.track.spline.waypoints.len() {
                    state.record_undo();
                    let mut copy = state.track.spline.waypoints[idx].clone();
                    copy.point += Vec2::new(4.0, 4.0);
                    let insert_idx = idx + 1;
                    if insert_idx < state.track.spline.waypoints.len() {
                        state.track.spline.waypoints.insert(insert_idx, copy);
                    } else {
                        state.track.spline.waypoints.push(copy);
                    }
                    state.rebuild_geometry();
                    state.select(Selection::Waypoint(insert_idx));
                    if let Some(wp) = state.track.spline.waypoints.get(insert_idx) {
                        self.active_surface = wp.surface.unwrap_or(SurfaceType::Asphalt);
                        self.new_waypoint_width = wp.width;
                        self.new_waypoint_left_curb = wp.left_curb;
                        self.new_waypoint_right_curb = wp.right_curb;
                        self.new_waypoint_left_wall = wp.left_wall;
                        self.new_waypoint_right_wall = wp.right_wall;
                        self.new_waypoint_left_wall_distance = wp.left_wall_distance;
                        self.new_waypoint_right_wall_distance = wp.right_wall_distance;
                        self.new_waypoint_bank_angle = wp.bank_angle;
                    }
                    return true;
                }
            }
            Selection::Checkpoint(idx) => {
                if idx < state.track.checkpoints.len() {
                    state.record_undo();
                    let mut copy = state.track.checkpoints[idx].clone();
                    let offset = Vec2::new(4.0, 4.0);
                    copy.gate.start += offset;
                    copy.gate.end += offset;
                    copy.is_finish_line = false;
                    state.track.checkpoints.push(copy);
                    for (new_id, cp) in state.track.checkpoints.iter_mut().enumerate() {
                        cp.id = new_id;
                    }
                    let new_idx = state.track.checkpoints.len() - 1;
                    state.selection = Selection::Checkpoint(new_idx);
                    state.revalidate();
                    return true;
                }
            }
            Selection::GridSlot(idx) => {
                if idx < state.track.grid_positions.len() {
                    state.record_undo();
                    let mut copy = state.track.grid_positions[idx].clone();
                    copy.position += Vec2::new(4.0, 4.0);
                    state.track.grid_positions.push(copy);
                    for (new_id, slot) in state.track.grid_positions.iter_mut().enumerate() {
                        slot.grid_slot = new_id;
                    }
                    let new_idx = state.track.grid_positions.len() - 1;
                    state.selection = Selection::GridSlot(new_idx);
                    state.revalidate();
                    return true;
                }
            }
            Selection::MultipleWaypoints(indices) => {
                if !indices.is_empty() {
                    state.record_undo();
                    let mut sorted = indices.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    let mut new_indices = Vec::new();
                    for &idx in &sorted {
                        if idx < state.track.spline.waypoints.len() {
                            let mut copy = state.track.spline.waypoints[idx].clone();
                            copy.point += Vec2::new(4.0, 4.0);
                            state.track.spline.waypoints.push(copy);
                            new_indices.push(state.track.spline.waypoints.len() - 1);
                        }
                    }
                    state.rebuild_geometry();
                    state.select(Selection::MultipleWaypoints(new_indices));
                    return true;
                }
            }
            Selection::Multi {
                waypoints,
                surface_zones,
                obstacles,
                jump_ramps,
                checkpoints,
                grid_slots,
                pit_box: _,
            } => {
                let mut any_duplicated = false;
                let mut new_waypoints = Vec::new();
                let mut new_surface_zones = Vec::new();
                let mut new_obstacles = Vec::new();
                let mut new_jump_ramps = Vec::new();
                let mut new_checkpoints = Vec::new();
                let mut new_grid_slots = Vec::new();

                if !waypoints.is_empty()
                    || !surface_zones.is_empty()
                    || !obstacles.is_empty()
                    || !jump_ramps.is_empty()
                    || !checkpoints.is_empty()
                    || !grid_slots.is_empty()
                {
                    state.record_undo();
                }

                // 1. Waypoints
                if !waypoints.is_empty() {
                    let mut sorted = waypoints.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    for &idx in &sorted {
                        if idx < state.track.spline.waypoints.len() {
                            let mut copy = state.track.spline.waypoints[idx].clone();
                            copy.point += Vec2::new(4.0, 4.0);
                            state.track.spline.waypoints.push(copy);
                            new_waypoints.push(state.track.spline.waypoints.len() - 1);
                            any_duplicated = true;
                        }
                    }
                    state.rebuild_geometry();
                }

                // 2. Surface Zones
                for &idx in &surface_zones {
                    if let Some(zone) = state.track.geometry.surface_zones.get(idx).cloned() {
                        let mut copy = zone.clone();
                        copy.name = format!("{} (Copy)", zone.name);
                        let old_center = get_surface_shape_center(&copy.shape);
                        set_surface_zone_position(&mut copy, old_center + Vec2::new(4.0, 4.0));
                        state.track.geometry.surface_zones.push(copy);
                        new_surface_zones.push(state.track.geometry.surface_zones.len() - 1);
                        any_duplicated = true;
                    }
                }

                // 3. Obstacles
                for &idx in &obstacles {
                    if let Some(obs) = state.track.geometry.obstacles.get(idx).cloned() {
                        let new_id = state.track.geometry.obstacles.len() + 1;
                        let mut copy = obs.clone();
                        copy.id = new_id;
                        copy.name = format!("{} (Copy)", obs.name);
                        let old_center = copy.center();
                        copy.set_center(old_center + Vec2::new(4.0, 4.0));
                        state.track.geometry.obstacles.push(copy);
                        new_obstacles.push(state.track.geometry.obstacles.len() - 1);
                        any_duplicated = true;
                    }
                }

                // 4. Jump Ramps
                for &idx in &jump_ramps {
                    if let Some(ramp) = state.track.geometry.jump_ramps.get(idx).cloned() {
                        let new_id = state.track.geometry.jump_ramps.len() + 1;
                        let mut copy = ramp.clone();
                        copy.id = new_id;
                        copy.name = format!("{} (Copy)", ramp.name);
                        let old_center = get_surface_shape_center(&copy.shape);
                        set_jump_ramp_position(&mut copy, old_center + Vec2::new(4.0, 4.0));
                        state.track.geometry.jump_ramps.push(copy);
                        new_jump_ramps.push(state.track.geometry.jump_ramps.len() - 1);
                        any_duplicated = true;
                    }
                }

                // 5. Checkpoints
                for &idx in &checkpoints {
                    if idx < state.track.checkpoints.len() {
                        let mut copy = state.track.checkpoints[idx].clone();
                        let offset = Vec2::new(4.0, 4.0);
                        copy.gate.start += offset;
                        copy.gate.end += offset;
                        copy.is_finish_line = false;
                        state.track.checkpoints.push(copy);
                        new_checkpoints.push(state.track.checkpoints.len() - 1);
                        any_duplicated = true;
                    }
                }
                for (new_id, cp) in state.track.checkpoints.iter_mut().enumerate() {
                    cp.id = new_id;
                }

                // 6. Grid Slots
                for &idx in &grid_slots {
                    if idx < state.track.grid_positions.len() {
                        let mut copy = state.track.grid_positions[idx].clone();
                        copy.position += Vec2::new(4.0, 4.0);
                        state.track.grid_positions.push(copy);
                        new_grid_slots.push(state.track.grid_positions.len() - 1);
                        any_duplicated = true;
                    }
                }
                for (new_id, slot) in state.track.grid_positions.iter_mut().enumerate() {
                    slot.grid_slot = new_id;
                }

                if any_duplicated {
                    state.select(Selection::from_multi(
                        new_waypoints,
                        new_surface_zones,
                        new_obstacles,
                        new_jump_ramps,
                        new_checkpoints,
                        new_grid_slots,
                        false,
                    ));
                    state.revalidate();
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Rotates selected jump ramp(s) by delta radians.
    pub fn rotate_selected_jump_ramp(&mut self, state: &mut EditorState, delta_rad: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.rotate(delta_rad);
            }
        }
        state.revalidate();
        true
    }

    /// Sets the exact 2D orientation angle in radians for selected jump ramp(s).
    pub fn set_selected_jump_ramp_angle(&mut self, state: &mut EditorState, angle_rad: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.set_angle(angle_rad);
            }
        }
        state.revalidate();
        true
    }

    /// Sets the exact 2D orientation angle in degrees for selected jump ramp(s) (0 to 360 degrees).
    pub fn set_selected_jump_ramp_angle_deg(&mut self, state: &mut EditorState, angle_deg: f32) -> bool {
        self.set_selected_jump_ramp_angle(state, angle_deg.to_radians())
    }

    /// Adjusts the length and width of selected jump ramp(s).
    pub fn adjust_selected_jump_ramp_size(&mut self, state: &mut EditorState, delta_len: f32, delta_wid: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.adjust_size(delta_len, delta_wid);
            }
        }
        state.revalidate();
        true
    }

    /// Scales the size of selected jump ramp(s) by a multiplier factor.
    pub fn scale_selected_jump_ramp_size(&mut self, state: &mut EditorState, factor: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.scale_size(factor);
            }
        }
        state.revalidate();
        true
    }

    /// Adjusts the launch pitch angle of selected jump ramp(s) in degrees.
    pub fn adjust_selected_jump_ramp_pitch(&mut self, state: &mut EditorState, delta_deg: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.ramp_angle_deg = (ramp.ramp_angle_deg + delta_deg).clamp(RAMP_PITCH_RANGE.0, RAMP_PITCH_RANGE.1);
            }
        }
        state.revalidate();
        true
    }

    /// Adjusts the height of selected jump ramp(s) in meters.
    pub fn adjust_selected_jump_ramp_height(&mut self, state: &mut EditorState, delta_h: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.height = (ramp.height + delta_h).clamp(RAMP_HEIGHT_RANGE.0, RAMP_HEIGHT_RANGE.1);
            }
        }
        state.revalidate();
        true
    }

    /// Sets the surface type of selected jump ramp(s).
    pub fn set_selected_jump_ramp_surface(&mut self, state: &mut EditorState, surface: SurfaceType) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.surface = surface;
            }
        }
        self.active_surface = surface;
        state.revalidate();
        true
    }

    /// Sets the exact length of selected jump ramp(s) in meters.
    pub fn set_selected_jump_ramp_length(&mut self, state: &mut EditorState, length: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        let clamped_len = length.clamp(RAMP_LENGTH_RANGE.0, RAMP_LENGTH_RANGE.1);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.set_length(clamped_len);
            }
        }
        state.revalidate();
        true
    }

    /// Sets the exact width of selected jump ramp(s) in meters.
    pub fn set_selected_jump_ramp_width(&mut self, state: &mut EditorState, width: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        let clamped_wid = width.clamp(RAMP_WIDTH_RANGE.0, RAMP_WIDTH_RANGE.1);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.set_width(clamped_wid);
            }
        }
        state.revalidate();
        true
    }

    /// Sets the exact height of selected jump ramp(s) in meters.
    pub fn set_selected_jump_ramp_height(&mut self, state: &mut EditorState, height: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        let clamped_h = height.clamp(RAMP_HEIGHT_RANGE.0, RAMP_HEIGHT_RANGE.1);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.height = clamped_h;
            }
        }
        state.revalidate();
        true
    }

    /// Sets the exact pitch angle of selected jump ramp(s) in degrees.
    pub fn set_selected_jump_ramp_pitch_deg(&mut self, state: &mut EditorState, pitch_deg: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        let clamped_pitch = pitch_deg.clamp(RAMP_PITCH_RANGE.0, RAMP_PITCH_RANGE.1);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.ramp_angle_deg = clamped_pitch;
            }
        }
        state.revalidate();
        true
    }

    /// Sets the launch boost / speed of selected jump ramp(s) in m/s.
    pub fn set_selected_jump_ramp_launch_speed(&mut self, state: &mut EditorState, speed: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        let clamped_speed = speed.clamp(1.0, 30.0);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.launch_speed = clamped_speed;
            }
        }
        state.revalidate();
        true
    }

    /// Adjusts the launch boost / speed of selected jump ramp(s) by a delta in m/s.
    pub fn adjust_selected_jump_ramp_launch_speed(&mut self, state: &mut EditorState, delta: f32) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.launch_speed = (ramp.launch_speed + delta).clamp(1.0, 30.0);
            }
        }
        state.revalidate();
        true
    }

    /// Automatically adjusts pitch angle on selected jump ramp(s) to eliminate the flat tabletop portion.
    pub fn remove_selected_jump_ramp_flat_portion(&mut self, state: &mut EditorState) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.ramp_angle_deg = ramp.fitted_pitch_deg();
            }
        }
        state.revalidate();
        true
    }

    /// Automatically adjusts height on selected jump ramp(s) to match the incline pitch angle and eliminate the flat tabletop portion.
    pub fn adjust_selected_jump_ramp_height_to_pitch(&mut self, state: &mut EditorState) -> bool {
        let indices = state.selection.selected_jump_ramp_indices();
        if indices.is_empty() {
            return false;
        }

        state.record_undo();
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.height = ramp.fitted_height();
            }
        }
        state.revalidate();
        true
    }

    /// Sets the layer of the selected surface zone (or the active placement layer if none selected).
    pub fn set_selected_surface_layer(&mut self, state: &mut EditorState, layer: SurfaceLayer) -> bool {
        self.active_surface_layer = layer;
        if let Selection::SurfaceZone(idx) = state.selection {
            if idx < state.track.geometry.surface_zones.len() {
                if state.track.geometry.surface_zones[idx].layer != layer {
                    state.record_undo();
                    state.track.geometry.surface_zones[idx].layer = layer;
                    state.revalidate();
                    return true;
                }
            }
        }
        false
    }

    /// Moves the selected surface zone to Front (AboveTrack).
    pub fn bring_selected_surface_front(&mut self, state: &mut EditorState) -> bool {
        self.set_selected_surface_layer(state, SurfaceLayer::AboveTrack)
    }

    /// Moves the selected surface zone to Back (BelowTrack).
    pub fn send_selected_surface_back(&mut self, state: &mut EditorState) -> bool {
        self.set_selected_surface_layer(state, SurfaceLayer::BelowTrack)
    }

    /// Toggles the selected surface zone's layer between AboveTrack and BelowTrack.
    pub fn toggle_selected_surface_layer(&mut self, state: &mut EditorState) -> bool {
        if let Selection::SurfaceZone(idx) = state.selection {
            if idx < state.track.geometry.surface_zones.len() {
                let new_layer = if state.track.geometry.surface_zones[idx].is_above_track() {
                    SurfaceLayer::BelowTrack
                } else {
                    SurfaceLayer::AboveTrack
                };
                return self.set_selected_surface_layer(state, new_layer);
            }
        }
        self.active_surface_layer = if self.active_surface_layer == SurfaceLayer::AboveTrack {
            SurfaceLayer::BelowTrack
        } else {
            SurfaceLayer::AboveTrack
        };
        false
    }

    /// Sets the track's default off-track surface type.
    pub fn set_track_default_surface(&mut self, state: &mut EditorState, surface: SurfaceType) -> bool {
        if !surface.is_valid_off_track() {
            return false;
        }
        if state.track.default_surface != surface {
            state.record_undo();
            state.track.default_surface = surface;
            state.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Cycles the track's default off-track surface type between Grass, Sand, Dirt, and Asphalt.
    pub fn cycle_track_default_surface(&mut self, state: &mut EditorState) -> SurfaceType {
        let current = state.track.default_surface;
        let idx = SurfaceType::OFF_TRACK_TYPES
            .iter()
            .position(|&s| s == current)
            .unwrap_or(0);
        let next = SurfaceType::OFF_TRACK_TYPES[(idx + 1) % SurfaceType::OFF_TRACK_TYPES.len()];
        self.set_track_default_surface(state, next);
        next
    }

    /// Sets the track's car category.
    pub fn set_track_car_category(&mut self, state: &mut EditorState, category: CarCategory) -> bool {
        if state.track.car_category != category {
            state.record_undo();
            state.track.car_category = category;
            state.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Cycles the track's car category through available options.
    pub fn cycle_track_car_category(&mut self, state: &mut EditorState) -> CarCategory {
        let current = state.track.car_category;
        let idx = CarCategory::ALL.iter().position(|&c| c == current).unwrap_or(0);
        let next = CarCategory::ALL[(idx + 1) % CarCategory::ALL.len()];
        self.set_track_car_category(state, next);
        next
    }

    /// Batch modifies track width for all selected waypoints.
    pub fn batch_set_width(&mut self, state: &mut EditorState, width: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].width = width.clamp(WIDTH_RANGE.0, WIDTH_RANGE.1);
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch adjusts track width by delta (+/-) for all selected waypoints.
    pub fn batch_adjust_width(&mut self, state: &mut EditorState, delta: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    let w = state.track.spline.waypoints[idx].width;
                    state.track.spline.waypoints[idx].width = (w + delta).clamp(WIDTH_RANGE.0, WIDTH_RANGE.1);
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch applies surface material for all selected waypoints, surface zones, and jump ramps.
    pub fn batch_set_surface(&mut self, state: &mut EditorState, surface: Option<SurfaceType>) -> bool {
        let wp_indices = state.selection.selected_waypoint_indices();
        let ramp_indices = state.selection.selected_jump_ramp_indices();
        let zone_indices = state.selection.selected_surface_zone_indices();

        if !wp_indices.is_empty() || (!ramp_indices.is_empty() && surface.is_some()) || (!zone_indices.is_empty() && surface.is_some()) {
            state.record_undo();
            for idx in wp_indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].surface = surface;
                }
            }
            if let Some(st) = surface {
                self.active_surface = st;
                for idx in ramp_indices {
                    if idx < state.track.geometry.jump_ramps.len() {
                        state.track.geometry.jump_ramps[idx].surface = st;
                    }
                }
                for idx in zone_indices {
                    if idx < state.track.geometry.surface_zones.len() {
                        state.track.geometry.surface_zones[idx].surface = st;
                    }
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch applies left/right curbs for all selected waypoints.
    pub fn batch_set_curbs(&mut self, state: &mut EditorState, left: bool, right: bool) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].left_curb = left;
                    state.track.spline.waypoints[idx].right_curb = right;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch applies left/right walls for all selected waypoints.
    pub fn batch_set_walls(&mut self, state: &mut EditorState, left: bool, right: bool) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].left_wall = left;
                    state.track.spline.waypoints[idx].right_wall = right;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch applies left and right wall distances for all selected waypoints (None resets to default offset).
    pub fn batch_set_wall_distances(
        &mut self,
        state: &mut EditorState,
        left: Option<f32>,
        right: Option<f32>,
    ) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].left_wall_distance = left;
                    state.track.spline.waypoints[idx].right_wall_distance = right;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch adjusts wall distances by delta for all selected waypoints.
    pub fn batch_adjust_wall_distances(&mut self, state: &mut EditorState, delta: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    let current_l = state.track.spline.waypoints[idx]
                        .left_wall_distance
                        .unwrap_or(state.barrier_offset);
                    let current_r = state.track.spline.waypoints[idx]
                        .right_wall_distance
                        .unwrap_or(state.barrier_offset);
                    state.track.spline.waypoints[idx].left_wall_distance =
                        Some((current_l + delta).clamp(WALL_DISTANCE_RANGE.0, WALL_DISTANCE_RANGE.1));
                    state.track.spline.waypoints[idx].right_wall_distance =
                        Some((current_r + delta).clamp(WALL_DISTANCE_RANGE.0, WALL_DISTANCE_RANGE.1));
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Sets global default barrier offset for the circuit and rebuilds geometry.
    pub fn set_global_barrier_offset(&mut self, state: &mut EditorState, offset: f32) {
        state.record_undo();
        state.barrier_offset = offset.clamp(GLOBAL_WALL_OFFSET_RANGE.0, GLOBAL_WALL_OFFSET_RANGE.1);
        state.rebuild_geometry();
    }

    /// Sets global default barrier type for the circuit and rebuilds geometry.
    pub fn set_global_barrier_type(&mut self, state: &mut EditorState, barrier_type: BarrierType) {
        state.record_undo();
        state.barrier_type = barrier_type;
        state.rebuild_geometry();
    }

    /// Batch applies wall type override for all selected waypoints (None inherits global barrier type).
    pub fn batch_set_wall_type(&mut self, state: &mut EditorState, wall_type: Option<BarrierType>) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].wall_type = wall_type;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch adjusts elevation for all selected waypoints.
    pub fn batch_adjust_elevation(&mut self, state: &mut EditorState, delta: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    let elev = state.track.spline.waypoints[idx].elevation;
                    state.track.spline.waypoints[idx].elevation = (elev + delta).max(0.0);
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch adjusts banking angle for all selected waypoints.
    pub fn batch_adjust_banking(&mut self, state: &mut EditorState, delta: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    let bank = state.track.spline.waypoints[idx].bank_angle;
                    state.track.spline.waypoints[idx].bank_angle = (bank + delta).clamp(BANKING_RANGE.0, BANKING_RANGE.1);
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch sets banking angle for all selected waypoints.
    pub fn batch_set_banking(&mut self, state: &mut EditorState, bank_angle: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].bank_angle = bank_angle.clamp(BANKING_RANGE.0, BANKING_RANGE.1);
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Batch inverts banking angle (+/-) for all selected waypoints.
    pub fn batch_invert_banking(&mut self, state: &mut EditorState) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].bank_angle = -state.track.spline.waypoints[idx].bank_angle;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }

    /// Cycles banking preset (0° -> 10° -> 18° -> 22° -> 0°) for all selected waypoints.
    pub fn cycle_selected_banking(&mut self, state: &mut EditorState) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    let curr = state.track.spline.waypoints[idx].bank_angle;
                    let next = if curr.abs() < 1.0 {
                        10.0
                    } else if (curr - 10.0).abs() < 2.0 {
                        18.0
                    } else if (curr - 18.0).abs() < 2.0 {
                        22.0
                    } else {
                        0.0
                    };
                    state.track.spline.waypoints[idx].bank_angle = next;
                }
            }
            state.rebuild_geometry();
            return true;
        }
        false
    }
}

impl ToolSettings {
    /// Handles primary button (Left Click) down event in world space.
    pub fn handle_primary_down(&mut self, state: &mut EditorState, mouse_world: Vec2, is_multi_key: bool) {
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        // Check if clicking near orientation tip handle of any selected jump ramp (for continuous angular rotation)
        let mut clicked_ramp_handle = None;
        for &idx in &state.selection.selected_jump_ramp_indices() {
            if let Some(ramp) = state.track.geometry.jump_ramps.get(idx) {
                let center = ramp.shape.center();
                let angle = ramp.angle();
                let half_extents = ramp.half_extents();
                let fwd = Vec2::new(angle.cos(), angle.sin());
                let handle_pos = center + fwd * (half_extents.x + 2.5);
                if (mouse_world - handle_pos).length() <= 3.5 {
                    clicked_ramp_handle = Some(idx);
                    break;
                }
            }
        }

        if let Some(ramp_idx) = clicked_ramp_handle {
            state.record_undo();
            self.is_rotating_ramp = true;
            self.drag_rotating_ramp_idx = Some(ramp_idx);
            self.is_dragging = true;
            self.is_box_selecting = false;
            self.drag_start_world = mouse_world;
            self.drag_current_world = mouse_world;
            return;
        }

        // In LaunchChute mode a click on a waypoint stamps the chute there
        if self.active_tool == EditorToolType::LaunchChute && nearest_waypoint(state, mouse_world, LAUNCH_CHUTE_PICK_M).is_some() {
            self.insert_launch_chute_at(state, mouse_world);
            self.is_box_selecting = false;
            self.is_dragging = false;
            return;
        }

        // In RoadSplit mode, check if clicking near any branch socket to activate it for extension
        if self.active_tool == EditorToolType::RoadSplit {
            let network = state.track.ensure_network();
            for j in &network.junctions {
                if let JunctionKind::Split { egress_sockets, .. } = &j.kind {
                    for (idx, socket) in egress_sockets.iter().enumerate() {
                        if (socket.point - mouse_world).length() <= 3.5 {
                            self.active_branch_socket = Some(SocketId::new(j.id, idx));
                            self.is_box_selecting = false;
                            self.is_dragging = false;
                            return;
                        }
                    }
                }
            }
        }

        if let Some(sel) = find_closest_entity(state, mouse_world) {
            self.is_box_selecting = false;
            if is_multi_key {
                state.select(state.selection.union(&sel));
                self.is_dragging = false;
                return;
            }

            if !state.selection.contains_entity(&sel) {
                state.select(sel.clone());
            }

            if let Selection::Waypoint(idx) = sel {
                if let Some(wp) = state.track.spline.waypoints.get(idx) {
                    self.active_surface = wp.surface.unwrap_or(SurfaceType::Asphalt);
                    self.new_waypoint_width = wp.width;
                    self.new_waypoint_left_curb = wp.left_curb;
                    self.new_waypoint_right_curb = wp.right_curb;
                    self.new_waypoint_left_wall = wp.left_wall;
                    self.new_waypoint_right_wall = wp.right_wall;
                    self.new_waypoint_left_wall_distance = wp.left_wall_distance;
                    self.new_waypoint_right_wall_distance = wp.right_wall_distance;
                    self.new_waypoint_bank_angle = wp.bank_angle;
                }
            } else if let Selection::SurfaceZone(idx) = sel {
                if let Some(zone) = state.track.geometry.surface_zones.get(idx) {
                    self.active_surface = zone.surface;
                }
            } else if let Selection::JumpRamp(idx) = sel {
                if let Some(ramp) = state.track.geometry.jump_ramps.get(idx) {
                    self.active_surface = ramp.surface;
                }
            }

            state.record_undo();
            self.is_dragging = true;
            self.drag_start_world = snapped_mouse;
            self.drag_current_world = snapped_mouse;
            self.drag_initial_entity_pos = get_selection_position(state, &state.selection).unwrap_or(mouse_world);
            prepare_drag_initial_positions(state, self);
        } else {
            if !is_multi_key {
                state.selection = Selection::None;
            }
            self.is_box_selecting = true;
            self.is_dragging = true;
            self.drag_start_world = mouse_world;
            self.drag_current_world = mouse_world;
            self.drag_initial_waypoints.clear();
            self.drag_initial_surface_zones.clear();
            self.drag_initial_obstacles.clear();
            self.drag_initial_jump_ramps.clear();
            self.drag_initial_checkpoints.clear();
            self.drag_initial_grid_slots.clear();
            self.drag_initial_pit_box = None;
        }
    }

    /// Handles primary button (Left Click) drag event.
    pub fn handle_primary_drag(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if !self.is_dragging {
            return;
        }

        if self.is_rotating_ramp {
            if let Some(ramp_idx) = self.drag_rotating_ramp_idx {
                if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(ramp_idx) {
                    let center = ramp.shape.center();
                    let to_mouse = mouse_world - center;
                    if to_mouse.length_squared() > 1e-4 {
                        let new_angle = to_mouse.y.atan2(to_mouse.x);
                        ramp.set_angle(new_angle);
                    }
                }
            }
            return;
        }

        if self.is_box_selecting {
            self.drag_current_world = mouse_world;
            return;
        }

        let snapped_mouse = state.grid_snap.snap_point(mouse_world);
        self.drag_current_world = snapped_mouse;

        let delta = snapped_mouse - self.drag_start_world;
        let mut geometry_dirty = false;

        for (idx, initial_pt) in &self.drag_initial_waypoints {
            if *idx < state.track.spline.waypoints.len() {
                state.track.spline.waypoints[*idx].point = *initial_pt + delta;
                geometry_dirty = true;
            }
        }

        for (idx, initial_center) in &self.drag_initial_surface_zones {
            if *idx < state.track.geometry.surface_zones.len() {
                set_surface_zone_position(&mut state.track.geometry.surface_zones[*idx], *initial_center + delta);
            }
        }

        for (idx, initial_center) in &self.drag_initial_obstacles {
            if *idx < state.track.geometry.obstacles.len() {
                set_obstacle_position(&mut state.track.geometry.obstacles[*idx], *initial_center + delta);
            }
        }

        for (idx, initial_center) in &self.drag_initial_jump_ramps {
            if *idx < state.track.geometry.jump_ramps.len() {
                set_jump_ramp_position(&mut state.track.geometry.jump_ramps[*idx], *initial_center + delta);
            }
        }

        for (idx, initial_start, initial_end) in &self.drag_initial_checkpoints {
            if *idx < state.track.checkpoints.len() {
                state.track.checkpoints[*idx].gate.start = *initial_start + delta;
                state.track.checkpoints[*idx].gate.end = *initial_end + delta;
            }
        }

        for (idx, initial_pos) in &self.drag_initial_grid_slots {
            if *idx < state.track.grid_positions.len() {
                state.track.grid_positions[*idx].position = *initial_pos + delta;
            }
        }

        if let Some((init_min, init_max)) = self.drag_initial_pit_box {
            if let Some(SurfaceShape::Aabb { min, max }) = &mut state.track.pit_box_area {
                *min = init_min + delta;
                *max = init_max + delta;
            }
        }

        if geometry_dirty {
            state.rebuild_geometry();
        }
    }

    /// Handles primary button (Left Click) release event.
    pub fn handle_primary_up(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if !self.is_dragging {
            return;
        }
        self.is_dragging = false;

        if self.is_rotating_ramp {
            self.is_rotating_ramp = false;
            self.drag_rotating_ramp_idx = None;
            state.revalidate();
            return;
        }

        if self.is_box_selecting {
            self.is_box_selecting = false;
            let min = Vec2::new(
                self.drag_start_world.x.min(mouse_world.x),
                self.drag_start_world.y.min(mouse_world.y),
            );
            let max = Vec2::new(
                self.drag_start_world.x.max(mouse_world.x),
                self.drag_start_world.y.max(mouse_world.y),
            );

            if (max.x - min.x) > 0.5 || (max.y - min.y) > 0.5 {
                let boxed = find_entities_in_box(state, min, max);
                if !state.selection.is_none() {
                    state.select(state.selection.union(&boxed));
                } else {
                    state.select(boxed);
                }
            }
            return;
        }

        if !self.drag_initial_checkpoints.is_empty() {
            state.auto_generate_grid();
            self.drag_initial_checkpoints.clear();
        }
        self.drag_initial_grid_slots.clear();

        state.revalidate();
    }

    /// Handles secondary button (Right Click) down event to place/create elements based on active tool.
    pub fn handle_secondary_down(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        match self.active_tool {
            EditorToolType::Select => {
                // No element placement in Select tool mode
            }
            EditorToolType::RoadSpline => {
                state.record_undo();

                let inherited_surface = match state
                    .current_or_last_waypoint_idx()
                    .and_then(|idx| state.track.spline.waypoints.get(idx))
                {
                    Some(prev_wp) => prev_wp.surface.unwrap_or(self.active_surface),
                    None => self.active_surface,
                };

                let inherited_wall_type = match state
                    .current_or_last_waypoint_idx()
                    .and_then(|idx| state.track.spline.waypoints.get(idx))
                {
                    Some(prev_wp) => prev_wp.wall_type.or(self.new_waypoint_wall_type),
                    None => self.new_waypoint_wall_type,
                };

                let mut wp = TrackWaypoint::new(snapped_mouse, self.new_waypoint_width);
                wp.surface = Some(inherited_surface);
                wp.left_curb = self.new_waypoint_left_curb;
                wp.right_curb = self.new_waypoint_right_curb;
                wp.left_wall = self.new_waypoint_left_wall;
                wp.right_wall = self.new_waypoint_right_wall;
                wp.left_wall_distance = self.new_waypoint_left_wall_distance;
                wp.right_wall_distance = self.new_waypoint_right_wall_distance;
                wp.wall_type = inherited_wall_type;
                wp.bank_angle = self.new_waypoint_bank_angle;

                let insert_idx = match state.current_or_last_waypoint_idx() {
                    Some(idx) => (idx + 1).min(state.track.spline.waypoints.len()),
                    None => state.track.spline.waypoints.len(),
                };

                if insert_idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints.insert(insert_idx, wp);
                } else {
                    state.track.spline.waypoints.push(wp);
                }

                state.rebuild_geometry();
                state.select(Selection::Waypoint(insert_idx));
                self.active_surface = inherited_surface;
            }
            EditorToolType::RoadSplit => {
                self.handle_road_split_placement(state, snapped_mouse);
            }
            EditorToolType::LaunchChute => {
                self.insert_launch_chute_at(state, mouse_world);
            }
            EditorToolType::SurfaceZone => {
                match self.active_surface_shape {
                    SurfaceShapeType::Square | SurfaceShapeType::Circle | SurfaceShapeType::Triangle => {
                        self.is_placing = true;
                        self.drag_start_world = snapped_mouse;
                        self.drag_current_world = snapped_mouse;
                    }
                    SurfaceShapeType::Polygon => {
                        if self.active_polygon_vertices.len() >= 3
                            && (self.active_polygon_vertices[0] - snapped_mouse).length() < 1.5
                        {
                            state.record_undo();
                            let vertices = std::mem::take(&mut self.active_polygon_vertices);
                            let zone_name = format!("{:?} Zone", self.active_surface);
                            let zone = SurfaceZone::new(
                                SurfaceShape::Polygon { vertices },
                                self.active_surface,
                                zone_name,
                            )
                            .with_layer(self.active_surface_layer);
                            state.track.geometry.surface_zones.push(zone);
                            state.selection = Selection::SurfaceZone(state.track.geometry.surface_zones.len() - 1);
                            state.revalidate();
                        } else {
                            self.active_polygon_vertices.push(snapped_mouse);
                        }
                    }
                }
            }
            EditorToolType::JumpRamp => {
                self.is_placing = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
            EditorToolType::Obstacle => {
                match self.active_obstacle_shape {
                    ObstacleShapeType::Circle => {
                        state.record_undo();
                        let obs_id = state.track.geometry.obstacles.len() + 1;
                        let new_obs = Obstacle::circle(obs_id, snapped_mouse, 1.2, format!("Tire Stack {}", obs_id));
                        state.track.geometry.obstacles.push(new_obs);
                        state.selection = Selection::Obstacle(state.track.geometry.obstacles.len() - 1);
                        state.revalidate();
                    }
                    ObstacleShapeType::Box => {
                        self.is_placing = true;
                        self.drag_start_world = snapped_mouse;
                        self.drag_current_world = snapped_mouse;
                    }
                    ObstacleShapeType::Polygon => {
                        if self.active_polygon_vertices.len() >= 3
                            && (self.active_polygon_vertices[0] - snapped_mouse).length() < 1.5
                        {
                            state.record_undo();
                            let obs_id = state.track.geometry.obstacles.len() + 1;
                            let vertices = std::mem::take(&mut self.active_polygon_vertices);
                            let new_obs = Obstacle::polygon(obs_id, vertices, format!("Polygon Obstacle {}", obs_id));
                            state.track.geometry.obstacles.push(new_obs);
                            state.selection = Selection::Obstacle(state.track.geometry.obstacles.len() - 1);
                            state.revalidate();
                        } else {
                            self.active_polygon_vertices.push(snapped_mouse);
                        }
                    }
                }
            }
            EditorToolType::Checkpoint => {
                self.is_placing = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
            EditorToolType::PitLane => {
                self.is_placing = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
            EditorToolType::ArenaFloor => {
                if self.active_polygon_vertices.len() >= 3
                    && (self.active_polygon_vertices[0] - snapped_mouse).length() < 3.0
                {
                    self.finalize_arena_hull(state);
                } else {
                    self.active_polygon_vertices.push(snapped_mouse);
                }
            }
            EditorToolType::WhoopSection => {
                self.is_placing = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
            EditorToolType::StuntRamp => {
                self.is_placing = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
        }
    }

    /// The chute the stamp tool builds: its settings, merging at `merge_waypoint` on `side`.
    fn chute_spec(&self, merge_waypoint: usize, side: ChuteSide) -> LaunchChuteSpec {
        LaunchChuteSpec {
            pad_width: self.chute_pad_width,
            pad_length: self.chute_pad_length,
            pattern: self.chute_pattern,
            surface: self.chute_surface,
            ..LaunchChuteSpec::new(merge_waypoint, side)
        }
    }

    /// Applies a chute edit to a copy of the circuit and, when it stood, makes it the circuit (one undo step).
    /// The status line says what happened. An existing chute is replaced.
    fn edit_launch_chute(&mut self, state: &mut EditorState, edit: impl FnOnce(&mut tdrace_core::track::Track) -> Result<String, String>) -> bool {
        let mut candidate = state.track.clone();
        if candidate.launch_chute().is_some() {
            candidate.remove_launch_chute();
        }
        match edit(&mut candidate) {
            Ok(done) => {
                state.record_undo();
                state.track = candidate;
                state.deselect();
                state.revalidate();
                self.chute_status = Some(done);
                true
            }
            Err(why) => {
                self.chute_status = Some(why);
                false
            }
        }
    }

    /// Stamps a launch chute that merges at the circuit waypoint nearest `mouse_world` (within
    /// `LAUNCH_CHUTE_PICK_M`), on the side where it fits. Returns whether it was placed.
    pub fn insert_launch_chute_at(&mut self, state: &mut EditorState, mouse_world: Vec2) -> bool {
        let Some(waypoint) = nearest_waypoint(state, mouse_world, LAUNCH_CHUTE_PICK_M) else {
            self.chute_status = Some("Click a waypoint of the circuit.".to_string());
            return false;
        };
        let specs = [ChuteSide::Right, ChuteSide::Left].map(|side| self.chute_spec(waypoint, side));
        self.edit_launch_chute(state, |track| {
            let mut last = None;
            for spec in specs {
                match track.stamp_launch_chute(&spec) {
                    Ok(()) => return Ok(format!("Launch chute placed at waypoint {}.", waypoint + 1)),
                    Err(e) => last = Some(e.0),
                }
            }
            Err(last.unwrap_or_default())
        })
    }

    /// Stamps a launch chute at the waypoint that suits it best (the `[ + INSERT LAUNCH CHUTE ]` button).
    pub fn auto_insert_launch_chute(&mut self, state: &mut EditorState) -> bool {
        let template = self.chute_spec(0, ChuteSide::Right);
        self.edit_launch_chute(state, |track| {
            track.place_launch_chute(&template).map(|spec| format!("Launch chute placed at waypoint {}.", spec.merge_waypoint + 1)).map_err(|e| e.0)
        })
    }

    /// Builds the circuit's chute again from the tool settings, where it merges now.
    pub fn restamp_launch_chute(&mut self, state: &mut EditorState) -> bool {
        let Some(placed) = state.track.launch_chute_spec() else { return false };
        let spec = LaunchChuteSpec { pad_length: self.chute_pad_length, ..self.chute_spec(placed.merge_waypoint, placed.side) };
        self.edit_launch_chute(state, |track| {
            track.stamp_launch_chute(&spec).map(|()| "Launch chute updated.".to_string()).map_err(|e| e.0)
        })
    }

    /// Removes the circuit's launch chute and puts the standard grid back.
    pub fn remove_launch_chute(&mut self, state: &mut EditorState) -> bool {
        if state.track.launch_chute().is_none() {
            return false;
        }
        state.record_undo();
        state.track.remove_launch_chute();
        state.deselect();
        state.revalidate();
        self.chute_status = Some("Launch chute removed.".to_string());
        true
    }

    /// Handles RoadSplit tool placement action:
    /// - If no active branch socket: inserts a RoadJunction::Split on the closest track point with customizable egress branches,
    ///   seeds a branch segment on the divergent branch, and sets it as the active branch socket.
    /// - If an active branch socket is selected:
    ///   - If clicking near an existing segment/waypoint: snaps and merges into a RoadJunction::Merge.
    ///   - If clicking in open space: appends a new waypoint to the active branch segment with smooth C¹ continuity.
    pub fn handle_road_split_placement(&mut self, state: &mut EditorState, snapped_mouse: Vec2) {
        state.record_undo();

        if let Some(active_sock) = self.active_branch_socket {
            let merge_snap_dist = 10.0;
            let mut merge_target: Option<(Vec2, Vec2, f32)> = None;

            // 1. Check if clicking near any waypoint of the main spline
            for wp in &state.track.spline.waypoints {
                if (wp.point - snapped_mouse).length() <= merge_snap_dist {
                    let sample = state.track.spline.samples.iter().min_by(|a, b| {
                        (a.point - wp.point)
                            .length_squared()
                            .total_cmp(&(b.point - wp.point).length_squared())
                    });
                    let tangent = sample.map(|s| s.tangent).unwrap_or(Vec2::X);
                    merge_target = Some((wp.point, tangent, wp.width));
                    break;
                }
            }

            // 2. Check other network segments (not current branch)
            if merge_target.is_none() {
                if let Some(ref net) = state.track.network {
                    for seg in &net.segments {
                        if seg.entry_junction == Some(active_sock) {
                            continue;
                        }
                        for wp in &seg.waypoints {
                            if (wp.point - snapped_mouse).length() <= merge_snap_dist {
                                let sample = seg.samples.iter().min_by(|a, b| {
                                    (a.point - wp.point)
                                        .length_squared()
                                        .total_cmp(&(b.point - wp.point).length_squared())
                                });
                                let tangent = sample.map(|s| s.tangent).unwrap_or(Vec2::X);
                                merge_target = Some((wp.point, tangent, wp.width));
                                break;
                            }
                        }
                        if merge_target.is_some() {
                            break;
                        }
                    }
                }
            }

            let network = state.track.ensure_network();

            if let Some((merge_pt, merge_tangent, merge_width)) = merge_target {
                // SNAP TO MERGE: Create RoadJunction::Merge
                let next_jid = JunctionId(network.junctions.iter().map(|j| j.id.0).max().unwrap_or(0) + 1);
                let ing_socket = SplineSocket::new(merge_pt, merge_tangent, merge_width);
                let eg_socket = SplineSocket::new(merge_pt, merge_tangent, merge_width);
                let merge_cfg = MergeConfig {
                    convergence_point: merge_pt,
                    merge_angle: self.split_divergence_angle * 0.7,
                    merge_length: 15.0,
                };
                let merge_j = RoadJunction::merge(
                    next_jid,
                    format!("Merge {}", next_jid.0),
                    vec![ing_socket],
                    eg_socket,
                    Some(merge_cfg),
                );
                network.junctions.push(merge_j);

                // Retrieve entry socket for C1 continuity
                let entry_sock_obj = network.junctions.iter()
                    .find(|j| j.id == active_sock.junction_id)
                    .and_then(|j| j.egress_socket(active_sock.socket_index))
                    .copied();

                if let Some(branch_seg) = network.segments.iter_mut().find(|s| s.entry_junction == Some(active_sock)) {
                    branch_seg.exit_junction = Some(SocketId::new(next_jid, 0));
                    branch_seg.waypoints.push(TrackWaypoint::new(merge_pt, merge_width));
                    branch_seg.recompute_samples(entry_sock_obj.as_ref(), Some(&eg_socket));
                }

                // If no alternative layout exists yet, create one linking the branch
                if network.layouts.len() <= 1 {
                    if let Some(branch_seg) = network.segments.iter().find(|s| s.entry_junction == Some(active_sock)) {
                        let alt_seq = vec![SegmentId(0), branch_seg.id];
                        let alt_layout = TrackLayout::new(
                            "alternative",
                            "Alternative Route",
                            alt_seq,
                            SegmentId(0),
                        );
                        network.layouts.push(alt_layout);
                    }
                }

                self.active_branch_socket = None;
            } else {
                // Append waypoint to the active branch
                let entry_sock_obj = network.junctions.iter()
                    .find(|j| j.id == active_sock.junction_id)
                    .and_then(|j| j.egress_socket(active_sock.socket_index))
                    .copied();

                let width = entry_sock_obj.as_ref().map(|s| s.width).unwrap_or(10.0);
                let mut new_wp = TrackWaypoint::new(snapped_mouse, width);
                new_wp.surface = Some(self.active_surface);

                if let Some(branch_seg) = network.segments.iter_mut().find(|s| s.entry_junction == Some(active_sock)) {
                    branch_seg.waypoints.push(new_wp);
                    branch_seg.recompute_samples(entry_sock_obj.as_ref(), None);
                } else {
                    let next_sid = SegmentId(network.segments.iter().map(|s| s.id.0).max().unwrap_or(0) + 1);
                    let wp0 = if let Some(ref sock) = entry_sock_obj {
                        TrackWaypoint::new(sock.point, sock.width)
                    } else {
                        TrackWaypoint::new(snapped_mouse, width)
                    };
                    let mut seg = RoadSegment::new(next_sid, format!("Branch {}", next_sid.0), vec![wp0, new_wp]);
                    seg.entry_junction = Some(active_sock);
                    seg.recompute_samples(entry_sock_obj.as_ref(), None);
                    network.segments.push(seg);
                }
            }
        } else {
            // No active branch socket: Insert a new RoadJunction::Split
            let mut split_pt = snapped_mouse;
            let mut split_tangent = Vec2::X;
            let mut split_width = 10.0;

            if let Some(idx) = find_closest_waypoint(state, snapped_mouse, 8.0) {
                let wp = &state.track.spline.waypoints[idx];
                split_pt = wp.point;
                split_width = wp.width;
                if let Some(s) = state.track.spline.samples.iter().min_by(|a, b| {
                    (a.point - wp.point)
                        .length_squared()
                        .total_cmp(&(b.point - wp.point).length_squared())
                }) {
                    split_tangent = s.tangent;
                }
            } else {
                let proj = state.track.spline.project_point(snapped_mouse);
                if (proj.closest_point - snapped_mouse).length() <= 12.0 {
                    split_pt = proj.closest_point;
                    split_tangent = proj.tangent;
                    split_width = proj.track_width;
                }
            }

            let network = state.track.ensure_network();

            let tangent = split_tangent.normalize_or_zero();
            let normal = Vec2::new(-tangent.y, tangent.x);
            let half_angle = (self.split_divergence_angle * 0.5).to_radians();

            let count = self.split_branch_count.max(2);
            let mut egress_sockets = Vec::with_capacity(count);
            let branch_w = (split_width * 0.75).max(6.0);

            for i in 0..count {
                let t = (i as f32 / (count - 1) as f32) * 2.0 - 1.0;
                let angle = t * half_angle;
                let dir = Vec2::new(
                    tangent.x * angle.cos() - tangent.y * angle.sin(),
                    tangent.x * angle.sin() + tangent.y * angle.cos(),
                )
                .normalize_or_zero();
                let lateral_offset = -t * (split_width * 0.35).max(3.0);
                let pt = split_pt + normal * lateral_offset;
                egress_sockets.push(SplineSocket::new(pt, dir, branch_w));
            }

            let ingress = SplineSocket::new(split_pt, tangent, split_width);
            let gore = GoreConfig::new(
                split_pt,
                self.split_divergence_angle,
                15.0,
                BarrierType::TireWall,
            );

            let next_jid = JunctionId(network.junctions.iter().map(|j| j.id.0).max().unwrap_or(0) + 1);
            let split_j = RoadJunction::split(
                next_jid,
                format!("Split {}", next_jid.0),
                ingress,
                egress_sockets.clone(),
                Some(gore),
            );
            network.junctions.push(split_j);

            // Automatically seed a branch segment on the divergent branch (last egress socket)
            let target_sock_idx = count - 1;
            let sock = &egress_sockets[target_sock_idx];
            let next_sid = SegmentId(network.segments.iter().map(|s| s.id.0).max().unwrap_or(0) + 1);
            let wp0 = TrackWaypoint::new(sock.point, sock.width);
            let wp1 = TrackWaypoint::new(sock.point + sock.tangent * 15.0, sock.width);
            let mut branch_b = RoadSegment::new(next_sid, format!("Branch {}", next_sid.0), vec![wp0, wp1]);
            branch_b.entry_junction = Some(SocketId::new(next_jid, target_sock_idx));
            branch_b.recompute_samples(Some(sock), None);
            network.segments.push(branch_b);

            self.active_branch_socket = Some(SocketId::new(next_jid, target_sock_idx));
        }

        state.revalidate();
        state.is_dirty = true;
    }

    /// Handles secondary button (Right Click) drag event for element placement.
    pub fn handle_secondary_drag(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if !self.is_placing {
            return;
        }
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);
        self.drag_current_world = snapped_mouse;
    }

    /// Handles secondary button (Right Click) release event to finalize element placement.
    pub fn handle_secondary_up(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if !self.is_placing {
            return;
        }
        self.is_placing = false;
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        match self.active_tool {
            EditorToolType::SurfaceZone => {
                let min = Vec2::new(
                    self.drag_start_world.x.min(snapped_mouse.x),
                    self.drag_start_world.y.min(snapped_mouse.y),
                );
                let max = Vec2::new(
                    self.drag_start_world.x.max(snapped_mouse.x),
                    self.drag_start_world.y.max(snapped_mouse.y),
                );

                if (max.x - min.x) > 2.0 && (max.y - min.y) > 2.0 {
                    state.record_undo();
                    let shape = match self.active_surface_shape {
                        SurfaceShapeType::Square => SurfaceShape::Aabb { min, max },
                        SurfaceShapeType::Circle => {
                            let center = (min + max) * 0.5;
                            let radius = ((max.x - min.x) * 0.5).max(2.0);
                            SurfaceShape::Circle { center, radius }
                        }
                        SurfaceShapeType::Triangle => {
                            let center_top = Vec2::new((min.x + max.x) * 0.5, max.y);
                            let bottom_left = Vec2::new(min.x, min.y);
                            let bottom_right = Vec2::new(max.x, min.y);
                            SurfaceShape::triangle(bottom_left, bottom_right, center_top)
                        }
                        SurfaceShapeType::Polygon => {
                            return;
                        }
                    };

                    let zone_name = format!("{:?} Zone", self.active_surface);
                    state.track.geometry.surface_zones.push(
                        SurfaceZone::new(shape, self.active_surface, zone_name)
                            .with_layer(self.active_surface_layer),
                    );
                    state.selection = Selection::SurfaceZone(state.track.geometry.surface_zones.len() - 1);
                    state.revalidate();
                } else if self.active_surface_shape == SurfaceShapeType::Triangle {
                    // Single-click 3-point triangle workflow
                    if self.active_polygon_vertices.len() < 2 {
                        self.active_polygon_vertices.push(snapped_mouse);
                    } else {
                        state.record_undo();
                        let mut vertices = std::mem::take(&mut self.active_polygon_vertices);
                        vertices.push(snapped_mouse);
                        let zone_name = format!("{:?} Triangle Zone", self.active_surface);
                        let zone = SurfaceZone::new(
                            SurfaceShape::Polygon { vertices },
                            self.active_surface,
                            zone_name,
                        )
                        .with_layer(self.active_surface_layer);
                        state.track.geometry.surface_zones.push(zone);
                        state.selection = Selection::SurfaceZone(state.track.geometry.surface_zones.len() - 1);
                        state.revalidate();
                    }
                }
            }
            EditorToolType::JumpRamp => {
                let dir = (snapped_mouse - self.drag_start_world).normalize_or_zero();
                let length = (snapped_mouse - self.drag_start_world).length().max(6.0);
                let center = (self.drag_start_world + snapped_mouse) * 0.5;
                let angle = dir.y.atan2(dir.x);

                state.record_undo();
                let ramp_id = state.track.geometry.jump_ramps.len() + 1;
                let shape = SurfaceShape::OrientedBox {
                    center,
                    half_extents: Vec2::new(length * 0.5, 4.0),
                    angle,
                };
                let ramp = JumpRamp::new(ramp_id, shape, dir, 4.0, 15.0, 1.8, format!("Jump Ramp {}", ramp_id))
                    .with_surface(self.active_surface);
                state.track.geometry.jump_ramps.push(ramp);
                state.selection = Selection::JumpRamp(state.track.geometry.jump_ramps.len() - 1);
                state.revalidate();
            }
            EditorToolType::Obstacle => {
                if self.active_obstacle_shape == ObstacleShapeType::Box {
                    let min = Vec2::new(
                        self.drag_start_world.x.min(snapped_mouse.x),
                        self.drag_start_world.y.min(snapped_mouse.y),
                    );
                    let max = Vec2::new(
                        self.drag_start_world.x.max(snapped_mouse.x),
                        self.drag_start_world.y.max(snapped_mouse.y),
                    );
                    if (max.x - min.x) > 1.0 && (max.y - min.y) > 1.0 {
                        state.record_undo();
                        let obs_id = state.track.geometry.obstacles.len() + 1;
                        let center = (min + max) * 0.5;
                        let half_extents = (max - min) * 0.5;
                        let new_obs = Obstacle::oriented_box(obs_id, center, half_extents, 0.0, format!("Box Obstacle {}", obs_id));
                        state.track.geometry.obstacles.push(new_obs);
                        state.selection = Selection::Obstacle(state.track.geometry.obstacles.len() - 1);
                        state.revalidate();
                    }
                }
            }
            EditorToolType::Checkpoint => {
                if (snapped_mouse - self.drag_start_world).length() > 3.0 {
                    state.record_undo();
                    let cp_id = state.track.checkpoints.len();
                    let gate = LineSegment::new(self.drag_start_world, snapped_mouse);
                    let normal = gate.normal();
                    let is_finish = cp_id == 0;
                    let cp = Checkpoint::new(cp_id, gate, normal, 0, is_finish);
                    state.track.checkpoints.push(cp);
                    state.selection = Selection::Checkpoint(cp_id);
                    state.revalidate();
                }
            }
            EditorToolType::PitLane => {
                let min = Vec2::new(
                    self.drag_start_world.x.min(snapped_mouse.x),
                    self.drag_start_world.y.min(snapped_mouse.y),
                );
                let max = Vec2::new(
                    self.drag_start_world.x.max(snapped_mouse.x),
                    self.drag_start_world.y.max(snapped_mouse.y),
                );

                if (max.x - min.x) > 4.0 && (max.y - min.y) > 4.0 {
                    state.record_undo();
                    state.track.pit_box_area = Some(SurfaceShape::Aabb { min, max });
                    state.selection = Selection::PitBox;
                    state.revalidate();
                }
            }
            EditorToolType::WhoopSection => {
                let v = snapped_mouse - self.drag_start_world;
                let total_len = v.length();
                if total_len >= self.whoop_spacing {
                    state.record_undo();
                    let dir = v.normalize();
                    let angle = dir.y.atan2(dir.x);
                    let count = (total_len / self.whoop_spacing).floor() as usize;
                    let ramp_start_id = state.track.geometry.jump_ramps.len();
                    for i in 0..count {
                        let ramp_id = ramp_start_id + i + 1;
                        let center = self.drag_start_world + dir * (self.whoop_spacing * (i as f32 + 0.5));
                        let half_len = (self.whoop_spacing * 0.42).max(1.0);
                        let half_wid = (self.whoop_width * 0.5).max(2.0);
                        let shape = SurfaceShape::OrientedBox {
                            center,
                            half_extents: Vec2::new(half_len, half_wid),
                            angle,
                        };
                        let whoop = JumpRamp::new(
                            ramp_id,
                            shape,
                            dir,
                            20.0,
                            16.0,
                            self.whoop_height,
                            format!("Whoop Mogul {}", ramp_id),
                        )
                        .with_surface(self.active_surface);
                        state.track.geometry.jump_ramps.push(whoop);
                    }
                    if count > 0 {
                        state.selection = Selection::JumpRamp(state.track.geometry.jump_ramps.len() - 1);
                    }
                    state.revalidate();
                }
            }
            EditorToolType::StuntRamp => {
                let v = snapped_mouse - self.drag_start_world;
                let length = v.length().max(10.0);
                let dir = if v.length() > 0.1 { v.normalize() } else { Vec2::X };
                let center = (self.drag_start_world + snapped_mouse) * 0.5;
                let angle = dir.y.atan2(dir.x);
                state.record_undo();
                let ramp_id = state.track.geometry.jump_ramps.len() + 1;
                let shape = SurfaceShape::OrientedBox {
                    center,
                    half_extents: Vec2::new(length * 0.5, 6.0),
                    angle,
                };
                let ramp = JumpRamp::new(
                    ramp_id,
                    shape,
                    dir,
                    28.0 * self.stunt_ramp_multiplier,
                    35.0,
                    self.stunt_ramp_height,
                    format!("Stunt Mega Ramp {}", ramp_id),
                )
                .with_surface(self.active_surface);
                state.track.geometry.jump_ramps.push(ramp);
                state.selection = Selection::JumpRamp(state.track.geometry.jump_ramps.len() - 1);
                state.revalidate();
            }
            _ => {}
        }
    }

    /// Handles mouse down event in world space (routes to primary or secondary action based on tool & entity proximity).
    pub fn handle_mouse_down(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        self.handle_mouse_down_with_mods(state, mouse_world, false);
    }

    /// Handles mouse down event with explicit modifier key flag (e.g. Shift / Ctrl for multi-select).
    pub fn handle_mouse_down_with_mods(&mut self, state: &mut EditorState, mouse_world: Vec2, is_multi_key: bool) {
        match self.active_tool {
            EditorToolType::Select => {
                self.handle_primary_down(state, mouse_world, is_multi_key);
            }
            EditorToolType::RoadSpline => {
                if find_closest_waypoint(state, mouse_world, 8.0).is_some() {
                    self.handle_primary_down(state, mouse_world, is_multi_key);
                } else {
                    self.handle_secondary_down(state, mouse_world);
                }
            }
            EditorToolType::RoadSplit => {
                let is_near_socket = {
                    let network = state.track.ensure_network();
                    network.junctions.iter().any(|j| {
                        if let JunctionKind::Split { egress_sockets, .. } = &j.kind {
                            egress_sockets.iter().any(|s| (s.point - mouse_world).length() <= 4.0)
                        } else {
                            false
                        }
                    })
                };
                if is_near_socket {
                    self.handle_primary_down(state, mouse_world, is_multi_key);
                } else {
                    self.handle_secondary_down(state, mouse_world);
                }
            }
            EditorToolType::PitLane => {
                let road_point = state.track.pit_lane_layout.as_ref().filter(|_| self.pit_layout_mode).and_then(|l| {
                    l.road_waypoints.iter().position(|p| p.distance(mouse_world) <= PIT_ROAD_GRAB_RADIUS)
                });
                if let Some(i) = road_point {
                    state.record_undo();
                    self.drag_pit_road_point = Some(i);
                } else if is_multi_key {
                    self.place_pit_box_stall(state, mouse_world);
                } else {
                    self.add_pit_lane_node(state, mouse_world);
                }
            }
            _ => {
                self.handle_secondary_down(state, mouse_world);
            }
        }
    }

    /// Handles generic mouse drag event.
    pub fn handle_mouse_drag(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        if let Some(i) = self.drag_pit_road_point {
            // Only road waypoints move; the guide points at the junction joints follow the junctions (locked).
            if let Some(mut layout) = state.track.pit_lane_layout.clone() {
                if let Some(p) = layout.road_waypoints.get_mut(i) {
                    *p = state.grid_snap.snap_point(mouse_world);
                }
                self.install_pit_layout(state, layout);
            }
            return;
        }
        if self.is_dragging {
            self.handle_primary_drag(state, mouse_world);
        } else if self.is_placing {
            self.handle_secondary_drag(state, mouse_world);
        }
    }

    /// Handles generic mouse up event.
    pub fn handle_mouse_up(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        self.drag_pit_road_point = None;
        if self.is_dragging {
            self.handle_primary_up(state, mouse_world);
        }
        if self.is_placing {
            self.handle_secondary_up(state, mouse_world);
        }
    }

    /// Deletes currently selected entity.
    pub fn delete_selected(&mut self, state: &mut EditorState) -> bool {
        match state.selection.clone() {
            Selection::Waypoint(idx) => {
                if state.track.spline.waypoints.len() > 3 && idx < state.track.spline.waypoints.len() {
                    state.record_undo();
                    state.track.spline.waypoints.remove(idx);
                    state.rebuild_geometry();
                    state.selection = Selection::None;
                    if let Some(last) = state.last_selected_waypoint {
                        if last == idx {
                            state.last_selected_waypoint = if idx > 0 {
                                Some(idx - 1)
                            } else if !state.track.spline.waypoints.is_empty() {
                                Some(0)
                            } else {
                                None
                            };
                        } else if last > idx {
                            state.last_selected_waypoint = Some(last - 1);
                        }
                    }
                    return true;
                }
            }
            Selection::MultipleWaypoints(indices) => {
                if !indices.is_empty() && state.track.spline.waypoints.len() > indices.len() {
                    state.record_undo();
                    let mut sorted = indices.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    for &idx in sorted.iter().rev() {
                        if idx < state.track.spline.waypoints.len() {
                            state.track.spline.waypoints.remove(idx);
                        }
                    }
                    state.rebuild_geometry();
                    state.selection = Selection::None;
                    state.last_selected_waypoint = None;
                    return true;
                }
            }
            Selection::SurfaceZone(idx) => {
                if idx < state.track.geometry.surface_zones.len() {
                    state.record_undo();
                    state.track.geometry.surface_zones.remove(idx);
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::Obstacle(idx) => {
                if idx < state.track.geometry.obstacles.len() {
                    state.record_undo();
                    state.track.geometry.obstacles.remove(idx);
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::JumpRamp(idx) => {
                if idx < state.track.geometry.jump_ramps.len() {
                    state.record_undo();
                    state.track.geometry.jump_ramps.remove(idx);
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::Checkpoint(idx) => {
                if idx < state.track.checkpoints.len() {
                    state.record_undo();
                    state.track.checkpoints.remove(idx);
                    for (new_id, cp) in state.track.checkpoints.iter_mut().enumerate() {
                        cp.id = new_id;
                    }
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::GridSlot(idx) => {
                if idx < state.track.grid_positions.len() {
                    state.record_undo();
                    state.track.grid_positions.remove(idx);
                    for (new_id, slot) in state.track.grid_positions.iter_mut().enumerate() {
                        slot.grid_slot = new_id;
                    }
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::PitBox => {
                state.record_undo();
                state.track.pit_box_area = None;
                state.track.pit_lane = None;
                state.selection = Selection::None;
                state.revalidate();
                return true;
            }
            Selection::Multi {
                waypoints,
                surface_zones,
                obstacles,
                jump_ramps,
                checkpoints,
                grid_slots,
                pit_box,
            } => {
                let mut any_deleted = false;
                if !waypoints.is_empty()
                    || !surface_zones.is_empty()
                    || !obstacles.is_empty()
                    || !jump_ramps.is_empty()
                    || !checkpoints.is_empty()
                    || !grid_slots.is_empty()
                    || pit_box
                {
                    state.record_undo();
                }

                // 1. Waypoints
                if !waypoints.is_empty() && state.track.spline.waypoints.len() > waypoints.len() {
                    let mut sorted = waypoints.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    for &idx in sorted.iter().rev() {
                        if idx < state.track.spline.waypoints.len() {
                            state.track.spline.waypoints.remove(idx);
                            any_deleted = true;
                        }
                    }
                    state.rebuild_geometry();
                    state.last_selected_waypoint = None;
                }

                // 2. Obstacles (in reverse order)
                let mut sorted_obs = obstacles.clone();
                sorted_obs.sort_unstable();
                sorted_obs.dedup();
                for &idx in sorted_obs.iter().rev() {
                    if idx < state.track.geometry.obstacles.len() {
                        state.track.geometry.obstacles.remove(idx);
                        any_deleted = true;
                    }
                }

                // 3. Surface Zones (in reverse order)
                let mut sorted_sz = surface_zones.clone();
                sorted_sz.sort_unstable();
                sorted_sz.dedup();
                for &idx in sorted_sz.iter().rev() {
                    if idx < state.track.geometry.surface_zones.len() {
                        state.track.geometry.surface_zones.remove(idx);
                        any_deleted = true;
                    }
                }

                // 4. Jump Ramps (in reverse order)
                let mut sorted_ramps = jump_ramps.clone();
                sorted_ramps.sort_unstable();
                sorted_ramps.dedup();
                for &idx in sorted_ramps.iter().rev() {
                    if idx < state.track.geometry.jump_ramps.len() {
                        state.track.geometry.jump_ramps.remove(idx);
                        any_deleted = true;
                    }
                }

                // 5. Checkpoints (in reverse order)
                let mut sorted_cp = checkpoints.clone();
                sorted_cp.sort_unstable();
                sorted_cp.dedup();
                for &idx in sorted_cp.iter().rev() {
                    if idx < state.track.checkpoints.len() {
                        state.track.checkpoints.remove(idx);
                        any_deleted = true;
                    }
                }
                for (new_id, cp) in state.track.checkpoints.iter_mut().enumerate() {
                    cp.id = new_id;
                }

                // 6. Grid Slots (in reverse order)
                let mut sorted_slots = grid_slots.clone();
                sorted_slots.sort_unstable();
                sorted_slots.dedup();
                for &idx in sorted_slots.iter().rev() {
                    if idx < state.track.grid_positions.len() {
                        state.track.grid_positions.remove(idx);
                        any_deleted = true;
                    }
                }
                for (new_id, slot) in state.track.grid_positions.iter_mut().enumerate() {
                    slot.grid_slot = new_id;
                }

                // 7. Pit box
                if pit_box {
                    state.track.pit_box_area = None;
                    any_deleted = true;
                }

                if any_deleted {
                    state.selection = Selection::None;
                    state.revalidate();
                    return true;
                }
            }
            Selection::None => {}
        }
        false
    }
}

/// Helper to find closest entity to query point.
fn find_closest_entity(state: &EditorState, point: Vec2) -> Option<Selection> {
    let pick_dist = 6.0;

    // 1. Waypoints
    if let Some(idx) = find_closest_waypoint(state, point, pick_dist) {
        return Some(Selection::Waypoint(idx));
    }

    // 2. Checkpoints
    for (idx, cp) in state.track.checkpoints.iter().enumerate() {
        if cp.gate.distance_to_point(point) < pick_dist {
            return Some(Selection::Checkpoint(idx));
        }
    }

    // 3. Obstacles
    for (idx, obs) in state.track.geometry.obstacles.iter().enumerate() {
        match &obs.shape {
            tdrace_core::track::ObstacleShape::Circle { center, radius } => {
                if (*center - point).length() < radius + pick_dist {
                    return Some(Selection::Obstacle(idx));
                }
            }
            tdrace_core::track::ObstacleShape::Box { center, half_extents, .. } => {
                if (*center - point).length() < half_extents.length() + pick_dist {
                    return Some(Selection::Obstacle(idx));
                }
            }
            tdrace_core::track::ObstacleShape::Polygon { vertices } => {
                let c = obs.center();
                let max_r = vertices.iter().map(|v| (*v - c).length()).fold(0.0f32, f32::max);
                if (c - point).length() < max_r + pick_dist {
                    return Some(Selection::Obstacle(idx));
                }
            }
        }
    }

    // 5. Jump Ramps
    for (idx, ramp) in state.track.geometry.jump_ramps.iter().enumerate() {
        if ramp.contains(point) || (get_surface_shape_center(&ramp.shape) - point).length() < pick_dist {
            return Some(Selection::JumpRamp(idx));
        }
    }

    // 6. Surface Zones
    for (idx, zone) in state.track.geometry.surface_zones.iter().enumerate() {
        if zone.contains(point) || (get_surface_shape_center(&zone.shape) - point).length() < pick_dist {
            return Some(Selection::SurfaceZone(idx));
        }
    }

    // 7. Pit Box & Pit Lane
    if let Some(lane) = &state.track.pit_lane {
        for b in &lane.pit_boxes {
            if b.contains_point(point) || (b.position - point).length() < pick_dist {
                return Some(Selection::PitBox);
            }
        }
        if lane.spline.total_length > 1.0 {
            let proj = lane.spline.project_point(point);
            if proj.distance_to_spline < lane.road_width * 0.5 {
                return Some(Selection::PitBox);
            }
        }
    }
    if let Some(pit) = &state.track.pit_box_area {
        if pit.contains(point) {
            return Some(Selection::PitBox);
        }
    }

    None
}

fn find_closest_waypoint(state: &EditorState, point: Vec2, max_dist: f32) -> Option<usize> {
    let mut closest = None;
    let mut min_d = max_dist;

    for (i, wp) in state.track.spline.waypoints.iter().enumerate() {
        let d = (wp.point - point).length();
        if d < min_d {
            min_d = d;
            closest = Some(i);
        }
    }
    closest
}

fn get_selection_position(state: &EditorState, sel: &Selection) -> Option<Vec2> {
    match sel {
        Selection::Waypoint(idx) => state.track.spline.waypoints.get(*idx).map(|w| w.point),
        Selection::MultipleWaypoints(indices) => {
            let pts: Vec<Vec2> = indices
                .iter()
                .filter_map(|&i| state.track.spline.waypoints.get(i).map(|w| w.point))
                .collect();
            if pts.is_empty() {
                None
            } else {
                let sum: Vec2 = pts.iter().copied().sum();
                Some(sum / pts.len() as f32)
            }
        }
        Selection::SurfaceZone(idx) => state.track.geometry.surface_zones.get(*idx).map(|z| get_surface_shape_center(&z.shape)),
        Selection::Obstacle(idx) => state.track.geometry.obstacles.get(*idx).map(|o| o.center()),
        Selection::JumpRamp(idx) => state.track.geometry.jump_ramps.get(*idx).map(|r| get_surface_shape_center(&r.shape)),
        Selection::Checkpoint(idx) => state.track.checkpoints.get(*idx).map(|c| (c.gate.start + c.gate.end) * 0.5),
        Selection::GridSlot(idx) => state.track.grid_positions.get(*idx).map(|g| g.position),
        Selection::PitBox => {
            if let Some(lane) = &state.track.pit_lane {
                if let Some(b) = lane.pit_boxes.first() {
                    return Some(b.position);
                }
                if let Some(s) = lane.spline.samples.first() {
                    return Some(s.point);
                }
            }
            state.track.pit_box_area.as_ref().map(get_surface_shape_center)
        }
        Selection::Multi {
            waypoints,
            surface_zones,
            obstacles,
            jump_ramps,
            checkpoints,
            grid_slots,
            pit_box,
        } => {
            let mut pts = Vec::new();
            for &i in waypoints {
                if let Some(w) = state.track.spline.waypoints.get(i) {
                    pts.push(w.point);
                }
            }
            for &i in surface_zones {
                if let Some(z) = state.track.geometry.surface_zones.get(i) {
                    pts.push(get_surface_shape_center(&z.shape));
                }
            }
            for &i in obstacles {
                if let Some(o) = state.track.geometry.obstacles.get(i) {
                    pts.push(o.center());
                }
            }
            for &i in jump_ramps {
                if let Some(r) = state.track.geometry.jump_ramps.get(i) {
                    pts.push(get_surface_shape_center(&r.shape));
                }
            }
            for &i in checkpoints {
                if let Some(c) = state.track.checkpoints.get(i) {
                    pts.push((c.gate.start + c.gate.end) * 0.5);
                }
            }
            for &i in grid_slots {
                if let Some(g) = state.track.grid_positions.get(i) {
                    pts.push(g.position);
                }
            }
            if *pit_box {
                if let Some(ref pb) = state.track.pit_box_area {
                    pts.push(get_surface_shape_center(pb));
                }
            }
            if pts.is_empty() {
                None
            } else {
                let sum: Vec2 = pts.iter().copied().sum();
                Some(sum / pts.len() as f32)
            }
        }
        Selection::None => None,
    }
}

fn get_surface_shape_center(shape: &SurfaceShape) -> Vec2 {
    match shape {
        SurfaceShape::Circle { center, .. } => *center,
        SurfaceShape::Aabb { min, max } => (*min + *max) * 0.5,
        SurfaceShape::OrientedBox { center, .. } => *center,
        SurfaceShape::Polygon { vertices } => {
            if vertices.is_empty() {
                Vec2::ZERO
            } else {
                let sum: Vec2 = vertices.iter().copied().sum();
                sum / vertices.len() as f32
            }
        }
    }
}

fn set_surface_zone_position(zone: &mut SurfaceZone, new_center: Vec2) {
    let old_center = get_surface_shape_center(&zone.shape);
    let delta = new_center - old_center;

    match &mut zone.shape {
        SurfaceShape::Circle { center, .. } => *center = new_center,
        SurfaceShape::Aabb { min, max } => {
            *min += delta;
            *max += delta;
        }
        SurfaceShape::OrientedBox { center, .. } => *center = new_center,
        SurfaceShape::Polygon { vertices } => {
            for v in vertices {
                *v += delta;
            }
        }
    }
}

fn set_obstacle_position(obs: &mut Obstacle, new_center: Vec2) {
    obs.set_center(new_center);
}

fn set_jump_ramp_position(ramp: &mut JumpRamp, new_center: Vec2) {
    set_surface_shape_center(&mut ramp.shape, new_center);
}

fn set_surface_shape_center(shape: &mut SurfaceShape, new_center: Vec2) {
    let old_center = get_surface_shape_center(shape);
    let delta = new_center - old_center;

    match shape {
        SurfaceShape::Circle { center, .. } => *center = new_center,
        SurfaceShape::Aabb { min, max } => {
            *min += delta;
            *max += delta;
        }
        SurfaceShape::OrientedBox { center, .. } => *center = new_center,
        SurfaceShape::Polygon { vertices } => {
            for v in vertices {
                *v += delta;
            }
        }
    }
}

fn prepare_drag_initial_positions(state: &EditorState, tools: &mut ToolSettings) {
    tools.drag_initial_waypoints = state
        .selection
        .selected_waypoint_indices()
        .iter()
        .filter_map(|&i| state.track.spline.waypoints.get(i).map(|wp| (i, wp.point)))
        .collect();

    tools.drag_initial_surface_zones = state
        .selection
        .selected_surface_zone_indices()
        .iter()
        .filter_map(|&i| state.track.geometry.surface_zones.get(i).map(|z| (i, get_surface_shape_center(&z.shape))))
        .collect();

    tools.drag_initial_obstacles = state
        .selection
        .selected_obstacle_indices()
        .iter()
        .filter_map(|&i| state.track.geometry.obstacles.get(i).map(|o| (i, o.center())))
        .collect();

    tools.drag_initial_jump_ramps = state
        .selection
        .selected_jump_ramp_indices()
        .iter()
        .filter_map(|&i| state.track.geometry.jump_ramps.get(i).map(|r| (i, get_surface_shape_center(&r.shape))))
        .collect();

    tools.drag_initial_checkpoints = state
        .selection
        .selected_checkpoint_indices()
        .iter()
        .filter_map(|&i| state.track.checkpoints.get(i).map(|cp| (i, cp.gate.start, cp.gate.end)))
        .collect();

    tools.drag_initial_grid_slots = state
        .selection
        .selected_grid_slot_indices()
        .iter()
        .filter_map(|&i| state.track.grid_positions.get(i).map(|s| (i, s.position)))
        .collect();

    tools.drag_initial_pit_box = if state.selection.is_pit_box_selected() {
        if let Some(SurfaceShape::Aabb { min, max }) = &state.track.pit_box_area {
            Some((*min, *max))
        } else {
            None
        }
    } else {
        None
    };
}

fn point_in_aabb(p: Vec2, min: Vec2, max: Vec2) -> bool {
    p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
}

fn line_segment_intersects_aabb(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> bool {
    if point_in_aabb(start, min, max) || point_in_aabb(end, min, max) {
        return true;
    }
    let p_mid = (start + end) * 0.5;
    if point_in_aabb(p_mid, min, max) {
        return true;
    }

    let seg = LineSegment::new(start, end);
    let top_left = Vec2::new(min.x, max.y);
    let bottom_right = Vec2::new(max.x, min.y);

    let edges = [
        LineSegment::new(min, bottom_right),
        LineSegment::new(bottom_right, max),
        LineSegment::new(max, top_left),
        LineSegment::new(top_left, min),
    ];

    for edge in &edges {
        if seg.intersect_segment(edge).is_some() {
            return true;
        }
    }
    false
}

fn surface_shape_intersects_aabb(shape: &SurfaceShape, min: Vec2, max: Vec2) -> bool {
    match shape {
        SurfaceShape::Aabb { min: s_min, max: s_max } => {
            s_min.x <= max.x && s_max.x >= min.x && s_min.y <= max.y && s_max.y >= min.y
        }
        SurfaceShape::Circle { center, radius } => {
            let clamped = Vec2::new(center.x.clamp(min.x, max.x), center.y.clamp(min.y, max.y));
            (*center - clamped).length_squared() <= radius * radius
        }
        SurfaceShape::OrientedBox { center, half_extents, angle } => {
            if point_in_aabb(*center, min, max) {
                return true;
            }
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            let ux = Vec2::new(cos_a, sin_a) * half_extents.x;
            let uy = Vec2::new(-sin_a, cos_a) * half_extents.y;
            let corners = [
                *center + ux + uy,
                *center - ux + uy,
                *center - ux - uy,
                *center + ux - uy,
            ];
            for c in &corners {
                if point_in_aabb(*c, min, max) {
                    return true;
                }
            }
            for i in 0..4 {
                let next = (i + 1) % 4;
                if line_segment_intersects_aabb(corners[i], corners[next], min, max) {
                    return true;
                }
            }
            false
        }
        SurfaceShape::Polygon { vertices } => {
            if vertices.is_empty() {
                return false;
            }
            for v in vertices {
                if point_in_aabb(*v, min, max) {
                    return true;
                }
            }
            let c = shape.center();
            if point_in_aabb(c, min, max) {
                return true;
            }
            let n = vertices.len();
            for i in 0..n {
                let next = (i + 1) % n;
                if line_segment_intersects_aabb(vertices[i], vertices[next], min, max) {
                    return true;
                }
            }
            false
        }
    }
}

fn obstacle_intersects_aabb(obs: &Obstacle, min: Vec2, max: Vec2) -> bool {
    match &obs.shape {
        tdrace_core::track::ObstacleShape::Circle { center, radius } => {
            let clamped = Vec2::new(center.x.clamp(min.x, max.x), center.y.clamp(min.y, max.y));
            (*center - clamped).length_squared() <= radius * radius
        }
        tdrace_core::track::ObstacleShape::Box { center, half_extents, angle } => {
            let shape = SurfaceShape::OrientedBox {
                center: *center,
                half_extents: *half_extents,
                angle: *angle,
            };
            surface_shape_intersects_aabb(&shape, min, max)
        }
        tdrace_core::track::ObstacleShape::Polygon { vertices } => {
            let shape = SurfaceShape::Polygon {
                vertices: vertices.clone(),
            };
            surface_shape_intersects_aabb(&shape, min, max)
        }
    }
}

pub fn find_entities_in_box(state: &EditorState, min: Vec2, max: Vec2) -> Selection {
    let mut waypoints = Vec::new();
    let mut surface_zones = Vec::new();
    let mut obstacles = Vec::new();
    let mut jump_ramps = Vec::new();
    let mut checkpoints = Vec::new();
    let mut pit_box = false;

    for (i, wp) in state.track.spline.waypoints.iter().enumerate() {
        if point_in_aabb(wp.point, min, max) {
            waypoints.push(i);
        }
    }

    for (i, cp) in state.track.checkpoints.iter().enumerate() {
        if line_segment_intersects_aabb(cp.gate.start, cp.gate.end, min, max) {
            checkpoints.push(i);
        }
    }

    for (i, obs) in state.track.geometry.obstacles.iter().enumerate() {
        if obstacle_intersects_aabb(obs, min, max) {
            obstacles.push(i);
        }
    }

    for (i, ramp) in state.track.geometry.jump_ramps.iter().enumerate() {
        if surface_shape_intersects_aabb(&ramp.shape, min, max) {
            jump_ramps.push(i);
        }
    }

    for (i, zone) in state.track.geometry.surface_zones.iter().enumerate() {
        if surface_shape_intersects_aabb(&zone.shape, min, max) {
            surface_zones.push(i);
        }
    }

    if let Some(ref pb) = state.track.pit_box_area {
        if surface_shape_intersects_aabb(pb, min, max) {
            pit_box = true;
        }
    }

    Selection::from_multi(
        waypoints,
        surface_zones,
        obstacles,
        jump_ramps,
        checkpoints,
        vec![],
        pit_box,
    )
}

fn draw_oriented_box_lines(center: Vec2, half_extents: Vec2, angle: f32, thickness: f32, col: Color) {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let ux = Vec2::new(cos_a, sin_a) * half_extents.x;
    let uy = Vec2::new(-sin_a, cos_a) * half_extents.y;
    let p0 = center + ux + uy;
    let p1 = center - ux + uy;
    let p2 = center - ux - uy;
    let p3 = center + ux - uy;
    draw_line(p0.x, p0.y, p1.x, p1.y, thickness, col);
    draw_line(p1.x, p1.y, p2.x, p2.y, thickness, col);
    draw_line(p2.x, p2.y, p3.x, p3.y, thickness, col);
    draw_line(p3.x, p3.y, p0.x, p0.y, thickness, col);
}

fn draw_polygon_lines(vertices: &[Vec2], thickness: f32, col: Color) {
    if vertices.len() < 2 {
        return;
    }
    let n = vertices.len();
    for i in 0..n {
        let next = (i + 1) % n;
        draw_line(vertices[i].x, vertices[i].y, vertices[next].x, vertices[next].y, thickness, col);
    }
}

/// Renders gizmos, selection indicators, handles, and drag previews in world space.
/// Radius (m) of the apex pivot dot at a split junction in the editor.
const EDITOR_APEX_DOT_RADIUS_M: f32 = 0.4;

/// Editor tag of the branch that leaves split `junction_id` through socket `socket_index`: `[Joker]` for a road
/// that only the joker layout drives, `[Main]` for every other branch.
pub fn branch_socket_label(track: &Track, junction_id: JunctionId, socket_index: usize) -> &'static str {
    let Some(network) = track.network.as_ref() else { return "[Main]" };
    let Some(segment) = network.segments.iter().find(|s| s.entry_junction == Some(SocketId::new(junction_id, socket_index))) else {
        return "[Main]";
    };
    let in_layout = |id: &str| network.get_layout(id).is_some_and(|l| l.segment_sequence.contains(&segment.id));
    if in_layout("joker") && !network.get_layout(&network.default_layout_id).is_some_and(|l| l.segment_sequence.contains(&segment.id)) {
        "[Joker]"
    } else {
        "[Main]"
    }
}

/// Draws the `[Main]` / `[Joker]` tags of split sockets and the `[Pit]` tag of the pit lane entry. Screen-space pass:
/// call it after the world gizmos and before the editor UI.
pub fn render_editor_junction_labels(fonts: &Fonts, state: &EditorState, camera: &EditorCamera) {
    let (sw, sh) = (screen_width(), screen_height());
    let scaler = UiScaler::new(sw, sh);
    let draw_tag = |tag: &str, world: Vec2, col: Color| {
        let at = camera.world_to_screen(world, sw, sh);
        fonts.draw_ui_bold(tag, at.x, at.y, scaler.font_s(11.0), col);
    };

    let network = state.track.active_network();
    for junction in &network.junctions {
        if let JunctionKind::Split { egress_sockets, .. } = &junction.kind {
            // Branches that leave in parallel share one socket spot, so each tag goes to the side its road bends to.
            let bend: Vec<f32> = egress_sockets
                .iter()
                .enumerate()
                .map(|(i, socket)| {
                    let seg = network.segments.iter().find(|s| s.entry_junction == Some(SocketId::new(junction.id, i)));
                    seg.filter(|s| s.samples.len() >= 2)
                        .map_or(-(i as f32), |s| socket.tangent.perp_dot(s.sample_at_distance(20.0_f32.min(s.length)).point - socket.point))
                })
                .collect();
            let mean = bend.iter().sum::<f32>() / bend.len().max(1) as f32;
            for (i, socket) in egress_sockets.iter().enumerate() {
                let tag = branch_socket_label(&state.track, junction.id, i);
                let col = if tag == "[Joker]" { Palette::NEON_GOLD } else { Palette::NEON_CYAN };
                let side = if bend[i] >= mean { 1.0 } else { -1.0 };
                draw_tag(tag, socket.point + socket.normal * (3.4 * side), col);
            }
        }
    }
    if let (Some(lane), Some(junctions)) = (&state.track.pit_lane, &state.track.pit_lane_junctions) {
        let gate_mid = (lane.entry_gate.start + lane.entry_gate.end) * 0.5;
        draw_tag("[Pit]", gate_mid + (gate_mid - junctions.p_apex).normalize_or_zero() * 2.5, Palette::NEON_CYAN);
    }
}

pub fn render_editor_gizmos(state: &EditorState, tools: &ToolSettings, _camera: &EditorCamera) {
    // 1. Render Waypoint nodes & handles
    let n_wp = state.track.spline.waypoints.len();
    for (i, wp) in state.track.spline.waypoints.iter().enumerate() {
        let is_selected = state.selection.is_waypoint_selected(i);

        let node_col = if is_selected {
            Palette::NEON_GOLD // Bright gold when selected
        } else {
            Palette::NEON_CYAN // Bright cyan node
        };

        // Draw node center circle
        draw_circle(wp.point.x, wp.point.y, 1.2, node_col);
        draw_circle_lines(wp.point.x, wp.point.y, 1.2, 0.25, Color::new(0.05, 0.05, 0.08, 0.9));

        // Draw left/right curb indicator markers
        if wp.left_curb || wp.right_curb {
            let sample = state.track.spline.samples.iter().find(|s| (s.point - wp.point).length() < 2.0);
            if let Some(s) = sample {
                let hw = wp.width * 0.5;
                if wp.left_curb {
                    let curbl = wp.point + s.normal * hw;
                    draw_circle(curbl.x, curbl.y, 0.8, Palette::CURB_RED);
                }
                if wp.right_curb {
                    let curbr = wp.point - s.normal * hw;
                    draw_circle(curbr.x, curbr.y, 0.8, Palette::CURB_RED);
                }
            }
        }

        // Draw banking cross-slope indicator arrow
        if wp.bank_angle.abs() > 0.5 {
            let sample = state.track.spline.samples.iter().find(|s| (s.point - wp.point).length() < 2.0);
            if let Some(s) = sample {
                let downhill = if wp.bank_angle > 0.0 { s.normal } else { -s.normal };
                let arrow_start = wp.point;
                let arrow_end = wp.point + downhill * 3.5;
                draw_line(arrow_start.x, arrow_start.y, arrow_end.x, arrow_end.y, 0.45, Palette::NEON_GOLD);

                // Arrowhead barb lines
                let perp = Vec2::new(-downhill.y, downhill.x);
                let left_barb = arrow_end - downhill * 1.0 + perp * 0.7;
                let right_barb = arrow_end - downhill * 1.0 - perp * 0.7;
                draw_line(arrow_end.x, arrow_end.y, left_barb.x, left_barb.y, 0.4, Palette::NEON_GOLD);
                draw_line(arrow_end.x, arrow_end.y, right_barb.x, right_barb.y, 0.4, Palette::NEON_GOLD);
            }
        }

        // Draw line connecting to next waypoint node
        if i + 1 < n_wp || state.track.spline.closed {
            let next_i = (i + 1) % n_wp;
            let next_p = state.track.spline.waypoints[next_i].point;
            draw_line(wp.point.x, wp.point.y, next_p.x, next_p.y, 0.35, Color::new(0.3, 0.9, 1.0, 0.3));
        }
    }

    // 1b. Render Road Junctions (Split & Merge) & Branch Segments
    let network = state.track.active_network();
    for junction in &network.junctions {
        match &junction.kind {
            JunctionKind::Split { ingress_socket, egress_sockets, gore_config } => {
                // Ingress directional indicator
                let in_fwd = ingress_socket.tangent * 3.5;
                draw_line(
                    ingress_socket.point.x - in_fwd.x,
                    ingress_socket.point.y - in_fwd.y,
                    ingress_socket.point.x,
                    ingress_socket.point.y,
                    0.4,
                    Palette::NEON_CYAN,
                );

                // Render egress branch sockets
                for (sock_idx, socket) in egress_sockets.iter().enumerate() {
                    let is_active = tools.active_branch_socket == Some(SocketId::new(junction.id, sock_idx));
                    let sock_col = if is_active { Palette::NEON_GOLD } else { Palette::NEON_CYAN };

                    // Socket ring
                    draw_circle_lines(socket.point.x, socket.point.y, 2.0, 0.45, sock_col);
                    if is_active {
                        draw_circle_lines(socket.point.x, socket.point.y, 2.8, 0.35, Palette::NEON_GOLD);
                        draw_circle(socket.point.x, socket.point.y, 1.0, Palette::NEON_GOLD);
                    }

                    // Direction arrow
                    let arrow_end = socket.point + socket.tangent * 4.0;
                    draw_line(socket.point.x, socket.point.y, arrow_end.x, arrow_end.y, 0.3, sock_col);

                    // Barb lines
                    let barb_l = arrow_end - socket.tangent * 1.2 + socket.normal * 0.8;
                    let barb_r = arrow_end - socket.tangent * 1.2 - socket.normal * 0.8;
                    draw_line(arrow_end.x, arrow_end.y, barb_l.x, barb_l.y, 0.3, sock_col);
                    draw_line(arrow_end.x, arrow_end.y, barb_r.x, barb_r.y, 0.3, sock_col);
                }

                // Apex pivot, divergence rays and nose barrier as thin wireframe (no painted wedge or chevrons)
                if let Some(gore) = gore_config {
                    let any_active = (0..egress_sockets.len()).any(|i| tools.active_branch_socket == Some(SocketId::new(junction.id, i)));
                    let apex_col = if any_active { Palette::NEON_GOLD } else { Palette::NEON_CYAN };
                    let ray_len = gore.gore_length.max(6.0);
                    for socket in egress_sockets {
                        let tip = gore.apex_point + socket.tangent * ray_len;
                        draw_line(gore.apex_point.x, gore.apex_point.y, tip.x, tip.y, 0.2, Color::new(apex_col.r, apex_col.g, apex_col.b, 0.7));
                    }
                    draw_line(
                        gore.nose_barrier.segment.start.x,
                        gore.nose_barrier.segment.start.y,
                        gore.nose_barrier.segment.end.x,
                        gore.nose_barrier.segment.end.y,
                        0.25,
                        apex_col,
                    );
                    draw_circle(gore.apex_point.x, gore.apex_point.y, EDITOR_APEX_DOT_RADIUS_M, apex_col);
                }
            }
            JunctionKind::Merge { ingress_sockets, egress_socket, merge_config } => {
                let conv_pt = merge_config.as_ref().map(|c| c.convergence_point).unwrap_or(egress_socket.point);
                draw_circle_lines(conv_pt.x, conv_pt.y, 2.5, 0.5, Palette::NEON_GOLD);
                draw_circle(conv_pt.x, conv_pt.y, 1.0, Palette::NEON_GOLD);

                let arrow_end = egress_socket.point + egress_socket.tangent * 4.0;
                draw_line(egress_socket.point.x, egress_socket.point.y, arrow_end.x, arrow_end.y, 0.5, Palette::NEON_GOLD);

                for in_sock in ingress_sockets {
                    draw_line(in_sock.point.x, in_sock.point.y, conv_pt.x, conv_pt.y, 0.35, Color::new(1.0, 0.85, 0.2, 0.6));
                }
            }
            JunctionKind::Terminal { socket } => {
                draw_circle_lines(socket.point.x, socket.point.y, 2.0, 0.4, Palette::RED);
            }
        }
    }

    // Render branch segments (id != 0)
    for seg in &network.segments {
        if seg.id.0 != 0 && !seg.waypoints.is_empty() {
            let n_bwp = seg.waypoints.len();
            for (w_i, wp) in seg.waypoints.iter().enumerate() {
                draw_circle(wp.point.x, wp.point.y, 1.2, Palette::NEON_MAGENTA);
                draw_circle_lines(wp.point.x, wp.point.y, 1.2, 0.25, Color::new(0.05, 0.05, 0.08, 0.9));

                if w_i + 1 < n_bwp {
                    let next_p = seg.waypoints[w_i + 1].point;
                    draw_line(wp.point.x, wp.point.y, next_p.x, next_p.y, 0.4, Color::new(0.9, 0.2, 0.9, 0.5));
                }
            }
        }
    }

    // 2. Render Checkpoint Gates & Sector Tags
    for cp in &state.track.checkpoints {
        let is_selected = state.selection.is_checkpoint_selected(cp.id);
        let gate_col = if cp.is_finish_line {
            Palette::NEON_GOLD
        } else if is_selected {
            Palette::NEON_GOLD
        } else {
            Color::new(0.2, 0.85, 0.4, 0.8)
        };

        let thickness = if is_selected { 0.8 } else { 0.35 };
        draw_line(cp.gate.start.x, cp.gate.start.y, cp.gate.end.x, cp.gate.end.y, thickness, gate_col);

        // Direction arrow
        let center = (cp.gate.start + cp.gate.end) * 0.5;
        let arrow_tip = center + cp.direction * 3.0;
        draw_line(center.x, center.y, arrow_tip.x, arrow_tip.y, 0.3, gate_col);
    }

    // 3. Render Starting Grid Slot gizmos
    for slot in &state.track.grid_positions {
        let col = Palette::NEON_MAGENTA;

        draw_circle(slot.position.x, slot.position.y, 1.4, col);
        let fwd = Vec2::new(slot.angle.cos(), slot.angle.sin()) * 2.5;
        draw_line(slot.position.x, slot.position.y, slot.position.x + fwd.x, slot.position.y + fwd.y, 0.4, col);
    }

    // 4. Render Selected Obstacle Highlights
    for (i, obs) in state.track.geometry.obstacles.iter().enumerate() {
        if state.selection.is_obstacle_selected(i) {
            let col = Palette::NEON_GOLD;
            match &obs.shape {
                tdrace_core::track::ObstacleShape::Circle { center, radius } => {
                    draw_circle_lines(center.x, center.y, radius + 0.5, 0.4, col);
                }
                tdrace_core::track::ObstacleShape::Box { center, half_extents, angle } => {
                    draw_oriented_box_lines(*center, *half_extents + Vec2::splat(0.5), *angle, 0.4, col);
                }
                tdrace_core::track::ObstacleShape::Polygon { vertices } => {
                    draw_polygon_lines(vertices, 0.4, col);
                }
            }
        }
    }

    // 5. Render Selected Surface Zone Highlights
    for (i, zone) in state.track.geometry.surface_zones.iter().enumerate() {
        if state.selection.is_surface_zone_selected(i) {
            let col = Palette::NEON_GOLD;
            match &zone.shape {
                SurfaceShape::Circle { center, radius } => {
                    draw_circle_lines(center.x, center.y, radius + 0.6, 0.4, col);
                }
                SurfaceShape::Aabb { min, max } => {
                    let w = max.x - min.x + 1.2;
                    let h = max.y - min.y + 1.2;
                    draw_rectangle_lines(min.x - 0.6, min.y - 0.6, w, h, 0.4, col);
                }
                SurfaceShape::OrientedBox { center, half_extents, angle } => {
                    draw_oriented_box_lines(*center, *half_extents + Vec2::splat(0.6), *angle, 0.4, col);
                }
                SurfaceShape::Polygon { vertices } => {
                    draw_polygon_lines(vertices, 0.4, col);
                }
            }
        }
    }

    // 6. Render Selected Jump Ramp Highlights
    for (i, ramp) in state.track.geometry.jump_ramps.iter().enumerate() {
        if state.selection.is_jump_ramp_selected(i) {
            let col = Palette::NEON_GOLD;
            match &ramp.shape {
                SurfaceShape::OrientedBox { center, half_extents, angle } => {
                    draw_oriented_box_lines(*center, *half_extents + Vec2::splat(0.6), *angle, 0.5, col);

                    // Direction arrow handle from center forward through launch lip
                    let fwd = Vec2::new(angle.cos(), angle.sin());
                    let right = Vec2::new(-angle.sin(), angle.cos());
                    let arrow_start = *center;
                    let arrow_end = *center + fwd * (half_extents.x + 2.5);
                    draw_line(arrow_start.x, arrow_start.y, arrow_end.x, arrow_end.y, 0.45, col);

                    let arrow_head_l = arrow_end - fwd * 1.5 - right * 1.0;
                    let arrow_head_r = arrow_end - fwd * 1.5 + right * 1.0;
                    draw_line(arrow_end.x, arrow_end.y, arrow_head_l.x, arrow_head_l.y, 0.45, col);
                    draw_line(arrow_end.x, arrow_end.y, arrow_head_r.x, arrow_head_r.y, 0.45, col);

                    // Circular rotation handle at the tip for continuous angle rotation
                    macroquad::shapes::draw_circle(arrow_end.x, arrow_end.y, 0.8, Color::new(0.0, 0.9, 1.0, 0.4));
                    macroquad::shapes::draw_circle_lines(arrow_end.x, arrow_end.y, 0.8, 0.25, Palette::NEON_CYAN);
                }
                _ => {
                    let center = ramp.shape.center();
                    let dir = ramp.direction;
                    let right = Vec2::new(-dir.y, dir.x);
                    let arrow_end = center + dir * 5.0;
                    draw_line(center.x, center.y, arrow_end.x, arrow_end.y, 0.45, col);
                    let arrow_head_l = arrow_end - dir * 1.5 - right * 1.0;
                    let arrow_head_r = arrow_end - dir * 1.5 + right * 1.0;
                    draw_line(arrow_end.x, arrow_end.y, arrow_head_l.x, arrow_head_l.y, 0.45, col);
                    draw_line(arrow_end.x, arrow_end.y, arrow_head_r.x, arrow_head_r.y, 0.45, col);

                    macroquad::shapes::draw_circle(arrow_end.x, arrow_end.y, 0.8, Color::new(0.0, 0.9, 1.0, 0.4));
                    macroquad::shapes::draw_circle_lines(arrow_end.x, arrow_end.y, 0.8, 0.25, Palette::NEON_CYAN);
                }
            }
        }
    }

    // 7. Render Selected Pit Box Highlight
    if state.selection.is_pit_box_selected() {
        if let Some(SurfaceShape::Aabb { min, max }) = &state.track.pit_box_area {
            let w = max.x - min.x + 1.2;
            let h = max.y - min.y + 1.2;
            draw_rectangle_lines(min.x - 0.6, min.y - 0.6, w, h, 0.5, Palette::NEON_GOLD);
        }
    }

    // 8. Render Active Drag Box / Marquee Selection preview or Element Placement preview
    if tools.is_box_selecting {
        let min = Vec2::new(
            tools.drag_start_world.x.min(tools.drag_current_world.x),
            tools.drag_start_world.y.min(tools.drag_current_world.y),
        );
        let max = Vec2::new(
            tools.drag_start_world.x.max(tools.drag_current_world.x),
            tools.drag_start_world.y.max(tools.drag_current_world.y),
        );
        let w = max.x - min.x;
        let h = max.y - min.y;
        macroquad::shapes::draw_rectangle(min.x, min.y, w, h, Color::new(0.2, 0.8, 1.0, 0.15));
        draw_rectangle_lines(min.x, min.y, w, h, 0.4, Palette::NEON_CYAN);
    } else if tools.is_placing {
        match tools.active_tool {
            EditorToolType::SurfaceZone => {
                let min = Vec2::new(
                    tools.drag_start_world.x.min(tools.drag_current_world.x),
                    tools.drag_start_world.y.min(tools.drag_current_world.y),
                );
                let max = Vec2::new(
                    tools.drag_start_world.x.max(tools.drag_current_world.x),
                    tools.drag_start_world.y.max(tools.drag_current_world.y),
                );
                let w = max.x - min.x;
                let h = max.y - min.y;
                match tools.active_surface_shape {
                    SurfaceShapeType::Square => {
                        draw_rectangle_lines(min.x, min.y, w, h, 0.4, Palette::NEON_CYAN);
                    }
                    SurfaceShapeType::Circle => {
                        let center = (min + max) * 0.5;
                        let radius = (w * 0.5).max(2.0);
                        draw_circle_lines(center.x, center.y, radius, 0.4, Palette::NEON_CYAN);
                    }
                    SurfaceShapeType::Triangle => {
                        let top = Vec2::new((min.x + max.x) * 0.5, max.y);
                        let bl = Vec2::new(min.x, min.y);
                        let br = Vec2::new(max.x, min.y);
                        draw_line(bl.x, bl.y, br.x, br.y, 0.4, Palette::NEON_CYAN);
                        draw_line(br.x, br.y, top.x, top.y, 0.4, Palette::NEON_CYAN);
                        draw_line(top.x, top.y, bl.x, bl.y, 0.4, Palette::NEON_CYAN);
                    }
                    SurfaceShapeType::Polygon => {}
                }
            }
            EditorToolType::PitLane => {
                let min = Vec2::new(
                    tools.drag_start_world.x.min(tools.drag_current_world.x),
                    tools.drag_start_world.y.min(tools.drag_current_world.y),
                );
                let max = Vec2::new(
                    tools.drag_start_world.x.max(tools.drag_current_world.x),
                    tools.drag_start_world.y.max(tools.drag_current_world.y),
                );
                let w = max.x - min.x;
                let h = max.y - min.y;
                draw_rectangle_lines(min.x, min.y, w, h, 0.4, Palette::NEON_CYAN);
            }
            EditorToolType::Checkpoint => {
                draw_line(
                    tools.drag_start_world.x,
                    tools.drag_start_world.y,
                    tools.drag_current_world.x,
                    tools.drag_current_world.y,
                    0.5,
                    Palette::NEON_CYAN,
                );
            }
            EditorToolType::JumpRamp => {
                let start = tools.drag_start_world;
                let current = tools.drag_current_world;
                let dir = (current - start).normalize_or_zero();
                let length = (current - start).length().max(6.0);
                let center = (start + current) * 0.5;
                let angle = dir.y.atan2(dir.x);
                let half_extents = Vec2::new(length * 0.5, 4.0);

                draw_oriented_box_lines(center, half_extents, angle, 0.4, Palette::NEON_CYAN);
                draw_line(start.x, start.y, current.x, current.y, 0.5, Palette::NEON_GOLD);

                let right = Vec2::new(-dir.y, dir.x);
                let arrow_tip = current;
                let l_wing = arrow_tip - dir * 1.8 - right * 1.2;
                let r_wing = arrow_tip - dir * 1.8 + right * 1.2;
                draw_line(arrow_tip.x, arrow_tip.y, l_wing.x, l_wing.y, 0.5, Palette::NEON_GOLD);
                draw_line(arrow_tip.x, arrow_tip.y, r_wing.x, r_wing.y, 0.5, Palette::NEON_GOLD);

                // Progressive curved contour arcs across ramp width showing the curved surface
                let half_len = half_extents.x;
                let half_wid = half_extents.y;
                let arc_w = (half_wid * 0.82).max(0.6);
                let arc_bulge = (half_len * 0.22).min(arc_w * 0.40);
                let contour_col = Color::new(Palette::NEON_CYAN.r, Palette::NEON_CYAN.g, Palette::NEON_CYAN.b, 0.6);

                for s in 0..3 {
                    let t = (s as f32 + 1.0) / 4.0;
                    let x_center = -half_len * 0.70 + t * (half_len * 1.35);
                    let num_segments = 10;
                    let mut prev_pt: Option<Vec2> = None;
                    for seg in 0..=num_segments {
                        let frac = (seg as f32 / num_segments as f32) * 2.0 - 1.0;
                        let y_offset = frac * arc_w;
                        let curve_offset = (1.0 - frac * frac) * arc_bulge;
                        let pt = center + dir * (x_center + curve_offset) + right * y_offset;
                        if let Some(prev) = prev_pt {
                            draw_line(prev.x, prev.y, pt.x, pt.y, 0.35, contour_col);
                        }
                        prev_pt = Some(pt);
                    }
                }
            }
            EditorToolType::Obstacle => {
                let min = Vec2::new(
                    tools.drag_start_world.x.min(tools.drag_current_world.x),
                    tools.drag_start_world.y.min(tools.drag_current_world.y),
                );
                let max = Vec2::new(
                    tools.drag_start_world.x.max(tools.drag_current_world.x),
                    tools.drag_start_world.y.max(tools.drag_current_world.y),
                );
                let w = max.x - min.x;
                let h = max.y - min.y;
                draw_rectangle_lines(min.x, min.y, w, h, 0.4, Palette::NEON_MAGENTA);
            }
            EditorToolType::WhoopSection => {
                let start = tools.drag_start_world;
                let current = tools.drag_current_world;
                let v = current - start;
                let total_len = v.length();
                draw_line(start.x, start.y, current.x, current.y, 0.5, Palette::NEON_CYAN);
                if total_len >= tools.whoop_spacing {
                    let dir = v.normalize();
                    let angle = dir.y.atan2(dir.x);
                    let count = (total_len / tools.whoop_spacing).floor() as usize;
                    for i in 0..count {
                        let center = start + dir * (tools.whoop_spacing * (i as f32 + 0.5));
                        let half_len = (tools.whoop_spacing * 0.42).max(1.0);
                        let half_wid = (tools.whoop_width * 0.5).max(2.0);
                        draw_oriented_box_lines(center, Vec2::new(half_len, half_wid), angle, 0.4, Palette::NEON_GOLD);
                        draw_circle(center.x, center.y, 0.6, Palette::NEON_CYAN);
                    }
                }
            }
            EditorToolType::StuntRamp => {
                let start = tools.drag_start_world;
                let current = tools.drag_current_world;
                let v = current - start;
                let length = v.length().max(10.0);
                let dir = if v.length() > 0.1 { v.normalize() } else { Vec2::X };
                let center = (start + current) * 0.5;
                let angle = dir.y.atan2(dir.x);
                let half_extents = Vec2::new(length * 0.5, 6.0);
                draw_oriented_box_lines(center, half_extents, angle, 0.5, Palette::NEON_MAGENTA);
                draw_line(start.x, start.y, current.x, current.y, 0.6, Palette::NEON_MAGENTA);

                // Projected ballistic trajectory envelope
                let launch_tip = center + dir * half_extents.x;
                let v0 = 26.0 * tools.stunt_ramp_multiplier;
                let num_steps = 18;
                let dt = 0.08;
                let mut prev = launch_tip;
                for step in 1..=num_steps {
                    let t = step as f32 * dt;
                    let fwd_dist = v0 * t;
                    let height_offset = tools.stunt_ramp_height + 8.5 * t - 0.5 * 9.81 * t * t;
                    if height_offset < 0.0 {
                        break;
                    }
                    let next = launch_tip + dir * fwd_dist;
                    draw_line(prev.x, prev.y, next.x, next.y, 0.45, Palette::NEON_GOLD);
                    prev = next;
                }
                draw_circle(prev.x, prev.y, 1.2, Palette::NEON_GREEN);
            }
            _ => {}
        }
    }

    // 9. Render In-progress Polygon / Triangle Construction vertices
    if !tools.active_polygon_vertices.is_empty() {
        let col = if tools.active_tool == EditorToolType::ArenaFloor {
            Palette::NEON_GOLD
        } else if tools.active_tool == EditorToolType::SurfaceZone {
            Palette::NEON_CYAN
        } else {
            Palette::NEON_MAGENTA
        };
        for (i, pt) in tools.active_polygon_vertices.iter().enumerate() {
            draw_circle(pt.x, pt.y, 0.8, col);
            if i > 0 {
                let prev = tools.active_polygon_vertices[i - 1];
                draw_line(prev.x, prev.y, pt.x, pt.y, 0.35, col);
            }
        }
        // Snap closing ring around first vertex if >= 3 vertices
        if tools.active_polygon_vertices.len() >= 3 {
            let first = tools.active_polygon_vertices[0];
            let last = *tools.active_polygon_vertices.last().unwrap();
            draw_circle_lines(first.x, first.y, 3.0, 0.4, Palette::NEON_GOLD);
            draw_line(last.x, last.y, first.x, first.y, 0.25, Color::new(col.r, col.g, col.b, 0.4));
        }
    }

    // 9b. Render In-progress Pit Lane waypoints and stalls
    if !tools.active_pit_waypoints.is_empty() {
        for (i, pt) in tools.active_pit_waypoints.iter().enumerate() {
            draw_circle(pt.x, pt.y, 1.2, Palette::NEON_CYAN);
            if i > 0 {
                let prev = tools.active_pit_waypoints[i - 1];
                draw_line(prev.x, prev.y, pt.x, pt.y, 0.4, Palette::NEON_CYAN);
            }
        }
        for b in &tools.active_pit_boxes {
            draw_circle_lines(b.position.x, b.position.y, b.stop_radius, 0.4, Palette::NEON_GOLD);
        }
    }

    // 9c. Render Built Pit Lane
    if let Some(lane) = &state.track.pit_lane {
        for (i, s) in lane.spline.samples.iter().enumerate() {
            if i > 0 {
                let prev = lane.spline.samples[i - 1].point;
                draw_line(prev.x, prev.y, s.point.x, s.point.y, 0.35, Palette::NEON_CYAN);
            }
        }
        draw_line(lane.entry_gate.start.x, lane.entry_gate.start.y, lane.entry_gate.end.x, lane.entry_gate.end.y, 0.5, Palette::NEON_GREEN);
        draw_line(lane.exit_gate.start.x, lane.exit_gate.start.y, lane.exit_gate.end.x, lane.exit_gate.end.y, 0.5, Palette::NEON_GOLD);
        for b in &lane.pit_boxes {
            draw_circle_lines(b.position.x, b.position.y, b.stop_radius, 0.4, Palette::NEON_CYAN);
        }
    }
    // Spec 101 layout: draggable road waypoints.
    if let Some(layout) = &state.track.pit_lane_layout {
        for p in &layout.road_waypoints {
            draw_circle(p.x, p.y, 1.0, Palette::NEON_GOLD);
        }
    }

    // 10. Render Arena Perimeter Hull if track is an Arena
    if let Some(hull) = state.track.arena_hull() {
        if hull.len() >= 3 {
            for i in 0..hull.len() {
                let p1 = hull[i];
                let p2 = hull[(i + 1) % hull.len()];
                draw_line(p1.x, p1.y, p2.x, p2.y, 0.5, Palette::NEON_CYAN);
                draw_circle(p1.x, p1.y, 0.8, Palette::NEON_GOLD);
            }
        }
    }

    // 12. Render the waypoints a launch chute can merge at
    if tools.active_tool == EditorToolType::LaunchChute {
        for (i, wp) in state.track.spline.waypoints.iter().enumerate() {
            if state.track.can_merge_launch_chute_at(i) {
                draw_circle_lines(wp.point.x, wp.point.y, 3.0, 0.5, Palette::NEON_GOLD);
            }
        }
    }

    // 11. Render RoadSplit Snap-to-Merge Target
    if tools.active_tool == EditorToolType::RoadSplit {
        if let Some(active_sock) = tools.active_branch_socket {
            let last_wp_point = network
                .segments
                .iter()
                .find(|s| s.entry_junction == Some(active_sock))
                .and_then(|s| s.waypoints.last().map(|w| w.point));

            let check_pt = tools.drag_current_world;
            let mut snap_pt: Option<Vec2> = None;
            for wp in &state.track.spline.waypoints {
                if (wp.point - check_pt).length() <= 10.0 {
                    snap_pt = Some(wp.point);
                    break;
                }
            }
            if snap_pt.is_none() {
                for seg in &network.segments {
                    if seg.entry_junction == Some(active_sock) {
                        continue;
                    }
                    for wp in &seg.waypoints {
                        if (wp.point - check_pt).length() <= 10.0 {
                            snap_pt = Some(wp.point);
                            break;
                        }
                    }
                    if snap_pt.is_some() {
                        break;
                    }
                }
            }

            if let Some(target) = snap_pt {
                draw_circle_lines(target.x, target.y, 3.0, 0.6, Palette::NEON_GOLD);
                draw_circle_lines(target.x, target.y, 4.5, 0.35, Color::new(1.0, 0.85, 0.2, 0.4));
                if let Some(lwp) = last_wp_point {
                    draw_line(lwp.x, lwp.y, target.x, target.y, 0.4, Palette::NEON_GOLD);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_road_spline_tool_add_and_move_waypoint() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::RoadSpline;

        let initial_count = state.track.spline.waypoints.len();

        // 1. Mouse down at new coordinate adds waypoint
        let new_point = Vec2::new(500.0, 500.0);
        tools.handle_mouse_down(&mut state, new_point);
        tools.handle_mouse_up(&mut state, new_point);

        assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
        let added_idx = state.track.spline.waypoints.len() - 1;
        assert_eq!(state.selection, Selection::Waypoint(added_idx));
        assert_eq!(state.track.spline.waypoints[added_idx].point, new_point);

        // 2. Drag waypoint to new position
        tools.active_tool = EditorToolType::Select;
        tools.handle_mouse_down(&mut state, new_point);
        let dragged_point = Vec2::new(520.0, 520.0);
        tools.handle_mouse_drag(&mut state, dragged_point);
        tools.handle_mouse_up(&mut state, dragged_point);

        assert_eq!(state.track.spline.waypoints[added_idx].point, dragged_point);
    }

    #[test]
    fn test_delete_selected_entity() {
        let track = tdrace_core::catalog::official_track("classic", "gt_ridge_ring");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        let initial_zones = state.track.geometry.surface_zones.len();
        assert!(initial_zones > 0);

        state.selection = Selection::SurfaceZone(0);
        assert!(tools.delete_selected(&mut state));
        assert_eq!(state.track.geometry.surface_zones.len(), initial_zones - 1);
        assert_eq!(state.selection, Selection::None);
    }

    #[test]
    fn test_road_spline_tool_insert_relative_to_selection() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::RoadSpline;

        let initial_count = state.track.spline.waypoints.len();
        let orig_wp2 = state.track.spline.waypoints[2].clone();
        let orig_wp3 = state.track.spline.waypoints[3].clone();

        // 1. Select waypoint 2
        state.select(Selection::Waypoint(2));
        assert_eq!(state.current_or_last_waypoint_idx(), Some(2));

        // 2. Click to add a new waypoint -> should be placed at index 3 (right after waypoint 2)
        let new_pt1 = Vec2::new(210.0, 15.0);
        tools.handle_mouse_down(&mut state, new_pt1);
        tools.handle_mouse_up(&mut state, new_pt1);

        assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
        assert_eq!(state.selection, Selection::Waypoint(3));
        assert_eq!(state.last_selected_waypoint, Some(3));
        assert_eq!(state.track.spline.waypoints[2].point, orig_wp2.point);
        assert_eq!(state.track.spline.waypoints[3].point, new_pt1);
        assert_eq!(state.track.spline.waypoints[4].point, orig_wp3.point);

        // 3. Click again to add another waypoint -> should be placed at index 4 (right after waypoint 3)
        let new_pt2 = Vec2::new(220.0, 20.0);
        tools.handle_mouse_down(&mut state, new_pt2);
        tools.handle_mouse_up(&mut state, new_pt2);

        assert_eq!(state.track.spline.waypoints.len(), initial_count + 2);
        assert_eq!(state.selection, Selection::Waypoint(4));
        assert_eq!(state.track.spline.waypoints[4].point, new_pt2);

        // 4. Deselect active entity, but remember last selected waypoint (which was 4)
        state.deselect();
        assert_eq!(state.selection, Selection::None);
        assert_eq!(state.current_or_last_waypoint_idx(), Some(4));

        // 5. Click to add another waypoint -> should still insert at index 5 (after last selected waypoint 4)
        let new_pt3 = Vec2::new(230.0, 25.0);
        tools.handle_mouse_down(&mut state, new_pt3);
        tools.handle_mouse_up(&mut state, new_pt3);

        assert_eq!(state.track.spline.waypoints.len(), initial_count + 3);
        assert_eq!(state.selection, Selection::Waypoint(5));
        assert_eq!(state.track.spline.waypoints[5].point, new_pt3);

        // 6. Select waypoint 0 -> click to insert -> should be placed at index 1
        state.select(Selection::Waypoint(0));
        let new_pt0 = Vec2::new(35.0, -10.0);
        tools.handle_mouse_down(&mut state, new_pt0);
        tools.handle_mouse_up(&mut state, new_pt0);

        assert_eq!(state.selection, Selection::Waypoint(1));
        assert_eq!(state.track.spline.waypoints[1].point, new_pt0);
    }

    #[test]
    fn test_waypoint_duplication_and_delete() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        let initial_count = state.track.spline.waypoints.len();
        state.select(Selection::Waypoint(1));
        let orig_pos = state.track.spline.waypoints[1].point;

        // Duplicate waypoint 1
        assert!(tools.duplicate_selected(&mut state));
        assert_eq!(state.track.spline.waypoints.len(), initial_count + 1);
        assert_eq!(state.selection, Selection::Waypoint(2));
        assert_eq!(state.track.spline.waypoints[2].point, orig_pos + Vec2::new(4.0, 4.0));

        // Delete duplicated waypoint 2
        assert!(tools.delete_selected(&mut state));
        assert_eq!(state.track.spline.waypoints.len(), initial_count);
        assert_eq!(state.selection, Selection::None);
        assert_eq!(state.last_selected_waypoint, Some(1));
    }

    #[test]
    fn test_road_spline_surface_inheritance_and_switching() {
        
        let track = tdrace_core::catalog::official_track("classic", "ax_clay_bowl");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::RoadSpline;

        // 1. Select waypoint 2 (which is Dirt in Clay Bowl)
        state.select(Selection::Waypoint(2));
        assert_eq!(state.track.spline.waypoints[2].surface, Some(SurfaceType::Dirt));

        // Click to add a new waypoint -> should inherit Dirt surface
        let new_pos = Vec2::new(170.0, 30.0);
        tools.handle_mouse_down(&mut state, new_pos);
        tools.handle_mouse_up(&mut state, new_pos);

        assert_eq!(state.selection, Selection::Waypoint(3));
        assert_eq!(state.track.spline.waypoints[3].surface, Some(SurfaceType::Dirt));
        assert_eq!(tools.active_surface, SurfaceType::Dirt);

        // 2. Change waypoint 3 surface to PackedSand
        state.track.spline.waypoints[3].surface = Some(SurfaceType::PackedSand);
        tools.active_surface = SurfaceType::PackedSand;
        state.rebuild_geometry();

        // 3. Click to add another waypoint -> should inherit PackedSand surface from waypoint 3
        let new_pos2 = Vec2::new(180.0, 40.0);
        tools.handle_mouse_down(&mut state, new_pos2);
        tools.handle_mouse_up(&mut state, new_pos2);

        assert_eq!(state.selection, Selection::Waypoint(4));
        assert_eq!(state.track.spline.waypoints[4].surface, Some(SurfaceType::PackedSand));
        assert_eq!(tools.active_surface, SurfaceType::PackedSand);
    }

    #[test]
    fn test_surface_zone_multi_shapes_and_layer_controls() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::SurfaceZone;

        // 1. Square Surface Zone Creation via Drag
        tools.active_surface_shape = SurfaceShapeType::Square;
        tools.active_surface = SurfaceType::DeepSand;
        tools.active_surface_layer = SurfaceLayer::BelowTrack;
        tools.handle_mouse_down(&mut state, Vec2::new(10.0, 10.0));
        tools.handle_mouse_drag(&mut state, Vec2::new(30.0, 30.0));
        tools.handle_mouse_up(&mut state, Vec2::new(30.0, 30.0));

        let zone_idx = state.track.geometry.surface_zones.len() - 1;
        assert_eq!(state.selection, Selection::SurfaceZone(zone_idx));
        assert_eq!(state.track.geometry.surface_zones[zone_idx].surface, SurfaceType::DeepSand);
        assert_eq!(state.track.geometry.surface_zones[zone_idx].layer, SurfaceLayer::BelowTrack);
        assert!(matches!(state.track.geometry.surface_zones[zone_idx].shape, SurfaceShape::Aabb { .. }));

        // 2. Layer Toggle Controls
        assert!(tools.bring_selected_surface_front(&mut state));
        assert_eq!(state.track.geometry.surface_zones[zone_idx].layer, SurfaceLayer::AboveTrack);
        assert!(tools.send_selected_surface_back(&mut state));
        assert_eq!(state.track.geometry.surface_zones[zone_idx].layer, SurfaceLayer::BelowTrack);
        assert!(tools.toggle_selected_surface_layer(&mut state));
        assert_eq!(state.track.geometry.surface_zones[zone_idx].layer, SurfaceLayer::AboveTrack);

        // 3. Circle Surface Zone Creation via Drag
        tools.active_surface_shape = SurfaceShapeType::Circle;
        tools.active_surface = SurfaceType::Water;
        tools.active_surface_layer = SurfaceLayer::AboveTrack;
        tools.handle_mouse_down(&mut state, Vec2::new(50.0, 50.0));
        tools.handle_mouse_drag(&mut state, Vec2::new(70.0, 70.0));
        tools.handle_mouse_up(&mut state, Vec2::new(70.0, 70.0));

        let circle_idx = state.track.geometry.surface_zones.len() - 1;
        assert_eq!(state.selection, Selection::SurfaceZone(circle_idx));
        assert!(matches!(state.track.geometry.surface_zones[circle_idx].shape, SurfaceShape::Circle { .. }));
        assert_eq!(state.track.geometry.surface_zones[circle_idx].layer, SurfaceLayer::AboveTrack);

        // 4. Triangle Surface Zone Creation via 3-point click
        tools.active_surface_shape = SurfaceShapeType::Triangle;
        tools.active_surface = SurfaceType::Dirt;
        tools.handle_mouse_down(&mut state, Vec2::new(100.0, 100.0));
        tools.handle_mouse_up(&mut state, Vec2::new(100.0, 100.0));
        assert_eq!(tools.active_polygon_vertices.len(), 1);

        tools.handle_mouse_down(&mut state, Vec2::new(120.0, 100.0));
        tools.handle_mouse_up(&mut state, Vec2::new(120.0, 100.0));
        assert_eq!(tools.active_polygon_vertices.len(), 2);

        tools.handle_mouse_down(&mut state, Vec2::new(110.0, 120.0));
        tools.handle_mouse_up(&mut state, Vec2::new(110.0, 120.0));
        assert!(tools.active_polygon_vertices.is_empty());

        let tri_idx = state.track.geometry.surface_zones.len() - 1;
        assert_eq!(state.selection, Selection::SurfaceZone(tri_idx));
        if let SurfaceShape::Polygon { vertices } = &state.track.geometry.surface_zones[tri_idx].shape {
            assert_eq!(vertices.len(), 3);
        } else {
            panic!("Expected Polygon shape with 3 vertices for triangle");
        }

        // 5. Polygon Surface Zone Creation with arbitrary vertices
        tools.active_surface_shape = SurfaceShapeType::Polygon;
        tools.active_surface = SurfaceType::Grass;
        tools.handle_mouse_down(&mut state, Vec2::new(200.0, 200.0));
        tools.handle_mouse_up(&mut state, Vec2::new(200.0, 200.0));
        tools.handle_mouse_down(&mut state, Vec2::new(220.0, 200.0));
        tools.handle_mouse_up(&mut state, Vec2::new(220.0, 200.0));
        tools.handle_mouse_down(&mut state, Vec2::new(220.0, 220.0));
        tools.handle_mouse_up(&mut state, Vec2::new(220.0, 220.0));
        tools.handle_mouse_down(&mut state, Vec2::new(200.0, 220.0));
        tools.handle_mouse_up(&mut state, Vec2::new(200.0, 220.0));
        // Click near first vertex (< 1.5m) to close
        tools.handle_mouse_down(&mut state, Vec2::new(200.5, 200.5));
        assert!(tools.active_polygon_vertices.is_empty());

        let poly_idx = state.track.geometry.surface_zones.len() - 1;
        assert_eq!(state.selection, Selection::SurfaceZone(poly_idx));
        if let SurfaceShape::Polygon { vertices } = &state.track.geometry.surface_zones[poly_idx].shape {
            assert_eq!(vertices.len(), 4);
        } else {
            panic!("Expected Polygon shape with 4 vertices");
        }
    }

    #[test]
    fn test_multi_segment_selection_and_batch_editing() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        // 1. Multi-selection via toggle
        state.selection = Selection::Waypoint(1);
        state.selection.toggle_waypoint(2);
        state.selection.toggle_waypoint(3);
        assert_eq!(state.selection, Selection::MultipleWaypoints(vec![1, 2, 3]));
        assert!(state.selection.is_waypoint_selected(1));
        assert!(state.selection.is_waypoint_selected(2));
        assert!(state.selection.is_waypoint_selected(3));
        assert!(!state.selection.is_waypoint_selected(4));

        // 2. Batch Width Modification
        assert!(tools.batch_set_width(&mut state, 18.5));
        assert_eq!(state.track.spline.waypoints[1].width, 18.5);
        assert_eq!(state.track.spline.waypoints[2].width, 18.5);
        assert_eq!(state.track.spline.waypoints[3].width, 18.5);

        // 3. Batch Curb Application
        assert!(tools.batch_set_curbs(&mut state, true, false));
        assert!(state.track.spline.waypoints[1].left_curb);
        assert!(!state.track.spline.waypoints[1].right_curb);
        assert!(state.track.spline.waypoints[2].left_curb);
        assert!(!state.track.spline.waypoints[2].right_curb);

        // 4. Batch Surface Application
        assert!(tools.batch_set_surface(&mut state, Some(SurfaceType::Dirt)));
        assert_eq!(state.track.spline.waypoints[1].surface, Some(SurfaceType::Dirt));
        assert_eq!(state.track.spline.waypoints[2].surface, Some(SurfaceType::Dirt));
        assert_eq!(state.track.spline.waypoints[3].surface, Some(SurfaceType::Dirt));

        // 5. Batch Drag / Translation
        let p1_orig = state.track.spline.waypoints[1].point;
        let p2_orig = state.track.spline.waypoints[2].point;
        let p3_orig = state.track.spline.waypoints[3].point;

        tools.active_tool = EditorToolType::Select;
        tools.drag_start_world = Vec2::new(100.0, 100.0);
        tools.is_dragging = true;
        tools.drag_initial_waypoints = vec![(1, p1_orig), (2, p2_orig), (3, p3_orig)];
        tools.handle_mouse_drag(&mut state, Vec2::new(110.0, 115.0));

        assert_eq!(state.track.spline.waypoints[1].point, p1_orig + Vec2::new(10.0, 15.0));
        assert_eq!(state.track.spline.waypoints[2].point, p2_orig + Vec2::new(10.0, 15.0));
        assert_eq!(state.track.spline.waypoints[3].point, p3_orig + Vec2::new(10.0, 15.0));

        // 6. Batch Duplication
        let prev_len = state.track.spline.waypoints.len();
        assert!(tools.duplicate_selected(&mut state));
        assert_eq!(state.track.spline.waypoints.len(), prev_len + 3);

        // 7. Undo restores track
        assert!(state.undo());
        assert_eq!(state.track.spline.waypoints.len(), prev_len);
    }

    #[test]
    fn test_select_all_for_active_tool_variants() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        // 1. Select Tool selects all elements into Selection::Multi
        tools.active_tool = EditorToolType::Select;
        assert!(tools.select_all_for_active_tool(&mut state));
        assert!(matches!(state.selection, Selection::Multi { .. }));
        assert_eq!(state.selection.total_count(), 
            state.track.spline.waypoints.len()
            + state.track.geometry.surface_zones.len()
            + state.track.geometry.obstacles.len()
            + state.track.geometry.jump_ramps.len()
            + state.track.checkpoints.len()
            + state.track.grid_positions.len()
            + if state.track.pit_box_area.is_some() { 1 } else { 0 }
        );

        // 2. RoadSpline Tool selects only waypoints
        tools.active_tool = EditorToolType::RoadSpline;
        assert!(tools.select_all_for_active_tool(&mut state));
        assert!(matches!(state.selection, Selection::MultipleWaypoints(_)));
        assert_eq!(state.selection.selected_waypoint_indices().len(), state.track.spline.waypoints.len());

        // 2b. RoadSplit Tool selects only waypoints
        tools.active_tool = EditorToolType::RoadSplit;
        assert!(tools.select_all_for_active_tool(&mut state));
        assert!(matches!(state.selection, Selection::MultipleWaypoints(_)));
        assert_eq!(state.selection.selected_waypoint_indices().len(), state.track.spline.waypoints.len());

        // 3. SurfaceZone Tool selects surface zones
        tools.active_tool = EditorToolType::SurfaceZone;
        if !state.track.geometry.surface_zones.is_empty() {
            assert!(tools.select_all_for_active_tool(&mut state));
            assert_eq!(state.selection.selected_surface_zone_indices().len(), state.track.geometry.surface_zones.len());
        }

        // 4. Obstacle Tool selects obstacles
        tools.active_tool = EditorToolType::Obstacle;
        if !state.track.geometry.obstacles.is_empty() {
            assert!(tools.select_all_for_active_tool(&mut state));
            assert_eq!(state.selection.selected_obstacle_indices().len(), state.track.geometry.obstacles.len());
        }

        // 5. Checkpoint Tool selects checkpoints
        tools.active_tool = EditorToolType::Checkpoint;
        assert!(tools.select_all_for_active_tool(&mut state));
        assert_eq!(state.selection.selected_checkpoint_indices().len(), state.track.checkpoints.len());
    }

    #[test]
    fn test_box_selection_and_multi_entity_drag_and_batch_ops() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::Select;

        // Add an obstacle and waypoint in an isolated coordinate area
        let base_pos = Vec2::new(600.0, 600.0);
        state.track.geometry.obstacles.push(Obstacle::circle(100, base_pos + Vec2::new(10.0, 10.0), 2.0, "Test Obs"));
        let obs_idx = state.track.geometry.obstacles.len() - 1;
        state.track.spline.waypoints.push(TrackWaypoint::new(base_pos + Vec2::new(20.0, 20.0), 12.0));
        let wp_idx = state.track.spline.waypoints.len() - 1;

        // Perform box drag selection covering the new obstacle and waypoint
        let box_min = base_pos - Vec2::new(20.0, 20.0);
        let box_max = base_pos + Vec2::new(40.0, 40.0);

        tools.handle_mouse_down(&mut state, box_min);
        assert!(tools.is_box_selecting);
        assert!(tools.is_dragging);

        tools.handle_mouse_drag(&mut state, box_max);
        tools.handle_mouse_up(&mut state, box_max);
        assert!(!tools.is_box_selecting);

        // Selection should contain both entities
        assert!(state.selection.is_waypoint_selected(wp_idx));
        assert!(state.selection.is_obstacle_selected(obs_idx));

        // Drag multi-selection by (5.0, 5.0)
        let orig_wp_pos = state.track.spline.waypoints[wp_idx].point;
        let orig_obs_pos = state.track.geometry.obstacles[obs_idx].center();

        tools.handle_mouse_down(&mut state, orig_wp_pos);
        assert!(!tools.is_box_selecting);
        assert!(tools.is_dragging);

        tools.handle_mouse_drag(&mut state, orig_wp_pos + Vec2::new(5.0, 5.0));
        tools.handle_mouse_up(&mut state, orig_wp_pos + Vec2::new(5.0, 5.0));

        assert_eq!(state.track.spline.waypoints[wp_idx].point, orig_wp_pos + Vec2::new(5.0, 5.0));
        assert_eq!(state.track.geometry.obstacles[obs_idx].center(), orig_obs_pos + Vec2::new(5.0, 5.0));

        // Duplicate the multi-selection
        let prev_wp_count = state.track.spline.waypoints.len();
        let prev_obs_count = state.track.geometry.obstacles.len();
        assert!(tools.duplicate_selected(&mut state));
        assert_eq!(state.track.spline.waypoints.len(), prev_wp_count + 1);
        assert_eq!(state.track.geometry.obstacles.len(), prev_obs_count + 1);

        // Delete the duplicated selection
        assert!(tools.delete_selected(&mut state));
        assert_eq!(state.track.spline.waypoints.len(), prev_wp_count);
        assert_eq!(state.track.geometry.obstacles.len(), prev_obs_count);
        assert_eq!(state.selection, Selection::None);
    }

    #[test]
    fn test_jump_ramp_tools_rotation_and_resizing() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        let ramp = JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(50.0, 50.0),
                half_extents: Vec2::new(5.0, 4.0),
                angle: 0.0,
            },
            Vec2::new(1.0, 0.0),
            24.0,
            15.0,
            1.8,
            "Editor Test Ramp",
        );
        state.track.geometry.jump_ramps.push(ramp);
        let ramp_idx = state.track.geometry.jump_ramps.len() - 1;
        state.selection = Selection::JumpRamp(ramp_idx);

        // 1. Rotate Selected Ramp
        assert!(tools.rotate_selected_jump_ramp(&mut state, std::f32::consts::FRAC_PI_4));
        assert!((state.track.geometry.jump_ramps[ramp_idx].angle() - std::f32::consts::FRAC_PI_4).abs() < 1e-4);

        // 2. Adjust Size
        assert!(tools.adjust_selected_jump_ramp_size(&mut state, 4.0, 2.0));
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].length(), 14.0);
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].width(), 10.0);

        // 3. Scale Size
        assert!(tools.scale_selected_jump_ramp_size(&mut state, 1.5));
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].length(), 21.0);
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].width(), 15.0);

        // 4. Adjust Pitch, Height & Launch Speed
        assert!(tools.adjust_selected_jump_ramp_pitch(&mut state, 5.0));
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].ramp_angle_deg, 20.0);
        assert!(tools.adjust_selected_jump_ramp_height(&mut state, 0.5));
        assert!((state.track.geometry.jump_ramps[ramp_idx].height - 2.3).abs() < 1e-4);
        assert!(tools.set_selected_jump_ramp_launch_speed(&mut state, 4.2));
        assert!((state.track.geometry.jump_ramps[ramp_idx].launch_speed - 4.2).abs() < 1e-4);
        assert!(tools.adjust_selected_jump_ramp_launch_speed(&mut state, 0.5));
        assert!((state.track.geometry.jump_ramps[ramp_idx].launch_speed - 4.7).abs() < 1e-4);

        // 5. Undo restores previous states
        assert!(state.undo());
        assert!((state.track.geometry.jump_ramps[ramp_idx].launch_speed - 4.2).abs() < 1e-4);
        assert!(state.undo());
        assert!((state.track.geometry.jump_ramps[ramp_idx].launch_speed - 24.0).abs() < 1e-4);
        assert!(state.undo());
        assert!((state.track.geometry.jump_ramps[ramp_idx].height - 1.8).abs() < 1e-4);
    }

    #[test]
    fn test_batch_set_walls_and_waypoint_wall_toggles() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        // Select first 3 waypoints
        state.selection = Selection::MultipleWaypoints(vec![0, 1, 2]);

        // Batch remove both walls
        assert!(tools.batch_set_walls(&mut state, false, false));
        assert!(!state.track.spline.waypoints[0].left_wall);
        assert!(!state.track.spline.waypoints[0].right_wall);
        assert!(!state.track.spline.waypoints[1].left_wall);
        assert!(!state.track.spline.waypoints[1].right_wall);

        // Undo restores walls
        assert!(state.undo());
        assert!(state.track.spline.waypoints[0].left_wall);
        assert!(state.track.spline.waypoints[0].right_wall);

        // Batch set left wall only
        assert!(tools.batch_set_walls(&mut state, true, false));
        assert!(state.track.spline.waypoints[0].left_wall);
        assert!(!state.track.spline.waypoints[0].right_wall);
    }

    #[test]
    fn test_primary_button_select_and_box_select_and_drag() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::RoadSpline;

        // 1. Primary Left Click on empty canvas performs marquee box selection
        let empty_area = Vec2::new(700.0, 700.0);
        tools.handle_primary_down(&mut state, empty_area, false);
        assert!(tools.is_box_selecting);
        assert!(tools.is_dragging);
        tools.handle_primary_drag(&mut state, empty_area + Vec2::new(50.0, 50.0));
        tools.handle_primary_up(&mut state, empty_area + Vec2::new(50.0, 50.0));
        assert!(!tools.is_box_selecting);
        assert!(!tools.is_dragging);

        // 2. Primary Left Click on existing waypoint selects and prepares dragging
        let wp0_pos = state.track.spline.waypoints[0].point;
        tools.handle_primary_down(&mut state, wp0_pos, false);
        assert_eq!(state.selection, Selection::Waypoint(0));
        assert!(tools.is_dragging);
        assert!(!tools.is_box_selecting);

        // Drag waypoint
        let new_wp0_pos = wp0_pos + Vec2::new(10.0, 10.0);
        tools.handle_primary_drag(&mut state, new_wp0_pos);
        tools.handle_primary_up(&mut state, new_wp0_pos);
        assert_eq!(state.track.spline.waypoints[0].point, new_wp0_pos);
    }

    #[test]
    fn test_secondary_button_places_elements_across_tools() {
        let track = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();

        // 1. Secondary Right Click in RoadSpline tool adds a new waypoint
        tools.active_tool = EditorToolType::RoadSpline;
        let initial_wp_count = state.track.spline.waypoints.len();
        let place_pos = Vec2::new(800.0, 800.0);
        tools.handle_secondary_down(&mut state, place_pos);
        tools.handle_secondary_up(&mut state, place_pos);
        assert_eq!(state.track.spline.waypoints.len(), initial_wp_count + 1);
        let new_idx = state.track.spline.waypoints.len() - 1;
        assert_eq!(state.selection, Selection::Waypoint(new_idx));
        assert_eq!(state.track.spline.waypoints[new_idx].point, place_pos);

        // 2. Secondary Right Drag in SurfaceZone tool adds a SurfaceZone
        tools.active_tool = EditorToolType::SurfaceZone;
        tools.active_surface = SurfaceType::DeepSand;
        tools.active_surface_shape = SurfaceShapeType::Square;
        let initial_zones = state.track.geometry.surface_zones.len();
        tools.handle_secondary_down(&mut state, Vec2::new(820.0, 820.0));
        assert!(tools.is_placing);
        tools.handle_secondary_drag(&mut state, Vec2::new(850.0, 850.0));
        tools.handle_secondary_up(&mut state, Vec2::new(850.0, 850.0));
        assert!(!tools.is_placing);
        assert_eq!(state.track.geometry.surface_zones.len(), initial_zones + 1);

        // 3. Secondary Right Drag in JumpRamp tool adds a JumpRamp
        tools.active_tool = EditorToolType::JumpRamp;
        let initial_ramps = state.track.geometry.jump_ramps.len();
        tools.handle_secondary_down(&mut state, Vec2::new(900.0, 900.0));
        assert!(tools.is_placing);
        tools.handle_secondary_drag(&mut state, Vec2::new(920.0, 900.0));
        tools.handle_secondary_up(&mut state, Vec2::new(920.0, 900.0));
        assert!(!tools.is_placing);
        assert_eq!(state.track.geometry.jump_ramps.len(), initial_ramps + 1);
    }
}


/// Distance within which a click grabs a pit road waypoint (m).
const PIT_ROAD_GRAB_RADIUS: f32 = 3.0;

/// Default layout between two clicked main-spline positions (spec 101 Pillar VII): Taper junctions, a road parallel
/// to the main track at the divider gap, and six stalls with garages.
pub fn default_pit_layout(track: &Track, entry_s: f32, exit_s: f32, side: Side) -> PitLaneLayout {
    const ROAD_WIDTH: f32 = 6.0;
    const GAP: f32 = 2.0;
    const JUNCTION_LENGTH: f32 = 40.0;
    const ROAD_POINT_SPACING: f32 = 25.0;
    let main = &track.spline;
    let total = main.total_length;
    let span = if main.closed { (exit_s - entry_s).rem_euclid(total) } else { exit_s - entry_s };
    let mut road_waypoints = Vec::new();
    let mut d = JUNCTION_LENGTH + ROAD_POINT_SPACING;
    while d < span - JUNCTION_LENGTH - ROAD_POINT_SPACING * 0.5 {
        let sample = main.sample_at_distance(entry_s + d);
        let offset = sample.width * 0.5 + GAP + ROAD_WIDTH * 0.5;
        road_waypoints.push(sample.point + sample.normal * (side.sign() * offset));
        d += ROAD_POINT_SPACING;
    }
    let junction = |s: f32| JunctionComponent { s, kind: JunctionShape::Taper, length: JUNCTION_LENGTH, divider_gap: GAP };
    PitLaneLayout {
        side,
        entry: junction(entry_s),
        exit: junction(exit_s),
        road_waypoints,
        road_width: ROAD_WIDTH,
        speed_limit: PitLane::DEFAULT_ROAD_SPEED_LIMIT,
        box_row: PitBoxRow { start_s: JUNCTION_LENGTH + 25.0, count: 6, spacing: 12.0, garages: true },
    }
}
