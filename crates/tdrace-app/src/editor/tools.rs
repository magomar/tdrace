use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle_lines};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::Checkpoint;
use tdrace_core::track::geometry::{JumpRamp, LineSegment, Obstacle, SpawnPose, SurfaceLayer, SurfaceShape, SurfaceZone};
use tdrace_core::track::spline::TrackWaypoint;

use super::camera::EditorCamera;
use super::state::{EditorState, Selection};
use crate::render::color::Palette;

/// Available editor tools in the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorToolType {
    Select,
    RoadSpline,
    SurfaceZone,
    JumpRamp,
    Obstacle,
    Checkpoint,
    StartingGrid,
    PitLane,
}

impl EditorToolType {
    pub const ALL: [Self; 8] = [
        Self::Select,
        Self::RoadSpline,
        Self::SurfaceZone,
        Self::JumpRamp,
        Self::Obstacle,
        Self::Checkpoint,
        Self::StartingGrid,
        Self::PitLane,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Select => "Select & Move [1]",
            Self::RoadSpline => "Road Spline [2]",
            Self::SurfaceZone => "Surfaces & Hazards [3]",
            Self::JumpRamp => "Jump Ramp [4]",
            Self::Obstacle => "Obstacle Prop [5]",
            Self::Checkpoint => "Checkpoint Gate [6]",
            Self::StartingGrid => "Starting Grid [7]",
            Self::PitLane => "Pit Lane [8]",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Select => "1",
            Self::RoadSpline => "2",
            Self::SurfaceZone => "3",
            Self::JumpRamp => "4",
            Self::Obstacle => "5",
            Self::Checkpoint => "6",
            Self::StartingGrid => "7",
            Self::PitLane => "8",
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
    pub new_waypoint_width: f32,
    pub new_waypoint_left_curb: bool,
    pub new_waypoint_right_curb: bool,
    pub new_waypoint_left_wall: bool,
    pub new_waypoint_right_wall: bool,

    // Dragging / selection interaction state
    pub is_dragging: bool,
    pub is_box_selecting: bool,
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
            new_waypoint_width: 14.0,
            new_waypoint_left_curb: false,
            new_waypoint_right_curb: false,
            new_waypoint_left_wall: true,
            new_waypoint_right_wall: true,
            is_dragging: false,
            is_box_selecting: false,
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
        }
    }
}

impl ToolSettings {
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
            EditorToolType::RoadSpline => {
                let waypoints = (0..state.track.spline.waypoints.len()).collect();
                Selection::from_multi(waypoints, vec![], vec![], vec![], vec![], vec![], false)
            }
            EditorToolType::SurfaceZone => {
                let surface_zones = (0..state.track.geometry.surface_zones.len()).collect();
                Selection::from_multi(vec![], surface_zones, vec![], vec![], vec![], vec![], false)
            }
            EditorToolType::JumpRamp => {
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
            EditorToolType::StartingGrid => {
                let grid_slots = (0..state.track.grid_positions.len()).collect();
                Selection::from_multi(vec![], vec![], vec![], vec![], vec![], grid_slots, false)
            }
            EditorToolType::PitLane => {
                let pit_box = state.track.pit_box_area.is_some();
                Selection::from_multi(vec![], vec![], vec![], vec![], vec![], vec![], pit_box)
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

    /// Sets the exact 2D orientation angle in degrees for selected jump ramp(s) (e.g. 0 to 365 degrees).
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
                ramp.ramp_angle_deg = (ramp.ramp_angle_deg + delta_deg).clamp(1.0, 60.0);
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
                ramp.height = (ramp.height + delta_h).clamp(0.2, 20.0);
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
        let clamped_len = length.clamp(2.0, 100.0);
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
        let clamped_wid = width.clamp(1.0, 100.0);
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
        let clamped_h = height.clamp(0.2, 20.0);
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
        let clamped_pitch = pitch_deg.clamp(1.0, 60.0);
        for &idx in &indices {
            if let Some(ramp) = state.track.geometry.jump_ramps.get_mut(idx) {
                ramp.ramp_angle_deg = clamped_pitch;
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

    /// Sets the track's predefined vehicle model.
    pub fn set_track_predefined_car(&mut self, state: &mut EditorState, car: Option<String>) -> bool {
        if state.track.predefined_car != car {
            state.record_undo();
            state.track.predefined_car = car;
            state.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Cycles the track's predefined vehicle model through available archetype options.
    pub fn cycle_track_predefined_car(&mut self, state: &mut EditorState) -> Option<String> {
        const CAR_PRESETS: [&str; 5] = ["sports_car", "drift_car", "kart", "rally_car", "f1_car"];
        let current = state.track.predefined_car.as_deref().unwrap_or("sports_car");
        let idx = CAR_PRESETS.iter().position(|&c| c == current).unwrap_or(0);
        let next = CAR_PRESETS[(idx + 1) % CAR_PRESETS.len()].to_string();
        self.set_track_predefined_car(state, Some(next.clone()));
        Some(next)
    }

    /// Batch modifies track width for all selected waypoints.
    pub fn batch_set_width(&mut self, state: &mut EditorState, width: f32) -> bool {
        let indices = state.selection.selected_waypoint_indices();
        if !indices.is_empty() {
            state.record_undo();
            for idx in indices {
                if idx < state.track.spline.waypoints.len() {
                    state.track.spline.waypoints[idx].width = width.clamp(6.0, 40.0);
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
                    state.track.spline.waypoints[idx].width = (w + delta).clamp(6.0, 40.0);
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
}

impl ToolSettings {
    /// Handles mouse down event in world space.
    pub fn handle_mouse_down(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        self.handle_mouse_down_with_mods(state, mouse_world, false);
    }

    /// Handles mouse down event with explicit modifier key flag (e.g. Shift / Ctrl for multi-select).
    pub fn handle_mouse_down_with_mods(&mut self, state: &mut EditorState, mouse_world: Vec2, is_multi_key: bool) {
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        match self.active_tool {
            EditorToolType::Select => {
                // Check if clicking near the orientation tip handle of any selected jump ramp (for continuous angular rotation)
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
            EditorToolType::RoadSpline => {
                // If clicked near an existing waypoint, select it
                if let Some(wp_idx) = find_closest_waypoint(state, mouse_world, 8.0) {
                    state.select(Selection::Waypoint(wp_idx));
                    if let Some(wp) = state.track.spline.waypoints.get(wp_idx) {
                        self.active_surface = wp.surface.unwrap_or(SurfaceType::Asphalt);
                        self.new_waypoint_width = wp.width;
                        self.new_waypoint_left_curb = wp.left_curb;
                        self.new_waypoint_right_curb = wp.right_curb;
                        self.new_waypoint_left_wall = wp.left_wall;
                        self.new_waypoint_right_wall = wp.right_wall;
                    }
                    self.is_box_selecting = false;
                    self.is_dragging = true;
                    self.drag_start_world = snapped_mouse;
                    self.drag_current_world = snapped_mouse;
                    self.drag_initial_entity_pos = state.track.spline.waypoints[wp_idx].point;
                    prepare_drag_initial_positions(state, self);
                } else {
                    // Insert new waypoint in relation to current/last selected waypoint
                    state.record_undo();

                    let inherited_surface = match state
                        .current_or_last_waypoint_idx()
                        .and_then(|idx| state.track.spline.waypoints.get(idx))
                    {
                        Some(prev_wp) => prev_wp.surface.unwrap_or(self.active_surface),
                        None => self.active_surface,
                    };

                    let mut wp = TrackWaypoint::new(snapped_mouse, self.new_waypoint_width);
                    wp.surface = Some(inherited_surface);
                    wp.left_curb = self.new_waypoint_left_curb;
                    wp.right_curb = self.new_waypoint_right_curb;
                    wp.left_wall = self.new_waypoint_left_wall;
                    wp.right_wall = self.new_waypoint_right_wall;

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
            }
            EditorToolType::SurfaceZone => {
                match self.active_surface_shape {
                    SurfaceShapeType::Square | SurfaceShapeType::Circle => {
                        self.is_dragging = true;
                        self.drag_start_world = snapped_mouse;
                        self.drag_current_world = snapped_mouse;
                    }
                    SurfaceShapeType::Triangle => {
                        self.is_dragging = true;
                        self.drag_start_world = snapped_mouse;
                        self.drag_current_world = snapped_mouse;
                    }
                    SurfaceShapeType::Polygon => {
                        // If clicked near first vertex and we have at least 3 vertices, close the polygon!
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
                self.is_dragging = true;
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
                        self.is_dragging = true;
                        self.drag_start_world = snapped_mouse;
                        self.drag_current_world = snapped_mouse;
                    }
                    ObstacleShapeType::Polygon => {
                        // If clicked near first vertex and we have at least 3 vertices, close the polygon!
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
                self.is_dragging = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
            EditorToolType::StartingGrid => {
                state.record_undo();
                let slot_idx = state.track.grid_positions.len();
                let angle = 0.0;
                state.track.grid_positions.push(SpawnPose::new(snapped_mouse, angle, slot_idx));
                state.selection = Selection::GridSlot(slot_idx);
                state.revalidate();
            }
            EditorToolType::PitLane => {
                self.is_dragging = true;
                self.drag_start_world = snapped_mouse;
                self.drag_current_world = snapped_mouse;
            }
        }
    }

    /// Handles mouse drag event.
    pub fn handle_mouse_drag(&mut self, state: &mut EditorState, mouse_world: Vec2) {
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

        match self.active_tool {
            EditorToolType::Select | EditorToolType::RoadSpline => {
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
            _ => {}
        }
    }

    /// Handles mouse up event.
    pub fn handle_mouse_up(&mut self, state: &mut EditorState, mouse_world: Vec2) {
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
                let ramp = JumpRamp::new(ramp_id, shape, dir, 24.0, 15.0, 1.8, format!("Jump Ramp {}", ramp_id))
                    .with_surface(self.active_surface);
                state.track.geometry.jump_ramps.push(ramp);
                state.selection = Selection::JumpRamp(state.track.geometry.jump_ramps.len() - 1);
                state.revalidate();
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
            _ => {}
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

    // 3. Grid Slots
    for (idx, slot) in state.track.grid_positions.iter().enumerate() {
        if (slot.position - point).length() < pick_dist {
            return Some(Selection::GridSlot(idx));
        }
    }

    // 4. Obstacles
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

    // 7. Pit Box
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
        Selection::PitBox => state.track.pit_box_area.as_ref().map(get_surface_shape_center),
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
    let mut grid_slots = Vec::new();
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

    for (i, slot) in state.track.grid_positions.iter().enumerate() {
        if point_in_aabb(slot.position, min, max) {
            grid_slots.push(i);
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
        grid_slots,
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

        // Draw line connecting to next waypoint node
        if i + 1 < n_wp || state.track.spline.closed {
            let next_i = (i + 1) % n_wp;
            let next_p = state.track.spline.waypoints[next_i].point;
            draw_line(wp.point.x, wp.point.y, next_p.x, next_p.y, 0.35, Color::new(0.3, 0.9, 1.0, 0.3));
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
        let is_selected = state.selection.is_grid_slot_selected(slot.grid_slot);
        let col = if is_selected { Palette::NEON_GOLD } else { Palette::NEON_MAGENTA };

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

    // 8. Render Active Drag Box / Marquee Selection preview
    if tools.is_dragging {
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
        } else {
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
                _ => {}
            }
        }
    }

    // 9. Render In-progress Polygon / Triangle Construction vertices
    if !tools.active_polygon_vertices.is_empty() {
        let col = if tools.active_tool == EditorToolType::SurfaceZone {
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
            draw_circle_lines(first.x, first.y, 1.5, 0.35, Palette::NEON_GOLD);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdrace_core::track::presets::classic_grand_prix;

    #[test]
    fn test_road_spline_tool_add_and_move_waypoint() {
        let track = classic_grand_prix();
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
        let track = classic_grand_prix();
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
        let track = classic_grand_prix();
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
        let track = classic_grand_prix();
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
        use tdrace_core::track::presets::oasis_rally;

        let track = oasis_rally();
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::RoadSpline;

        // 1. Select waypoint 2 (which is Dirt in Oasis Rally)
        state.select(Selection::Waypoint(2));
        assert_eq!(state.track.spline.waypoints[2].surface, Some(SurfaceType::Dirt));

        // Click to add a new waypoint -> should inherit Dirt surface
        let new_pos = Vec2::new(170.0, 30.0);
        tools.handle_mouse_down(&mut state, new_pos);
        tools.handle_mouse_up(&mut state, new_pos);

        assert_eq!(state.selection, Selection::Waypoint(3));
        assert_eq!(state.track.spline.waypoints[3].surface, Some(SurfaceType::Dirt));
        assert_eq!(tools.active_surface, SurfaceType::Dirt);

        // 2. Change waypoint 3 surface to Sand
        state.track.spline.waypoints[3].surface = Some(SurfaceType::Sand);
        tools.active_surface = SurfaceType::Sand;
        state.rebuild_geometry();

        // 3. Click to add another waypoint -> should inherit Sand surface from waypoint 3
        let new_pos2 = Vec2::new(180.0, 40.0);
        tools.handle_mouse_down(&mut state, new_pos2);
        tools.handle_mouse_up(&mut state, new_pos2);

        assert_eq!(state.selection, Selection::Waypoint(4));
        assert_eq!(state.track.spline.waypoints[4].surface, Some(SurfaceType::Sand));
        assert_eq!(tools.active_surface, SurfaceType::Sand);
    }

    #[test]
    fn test_surface_zone_multi_shapes_and_layer_controls() {
        let track = classic_grand_prix();
        let mut state = EditorState::new(track);
        let mut tools = ToolSettings::default();
        tools.active_tool = EditorToolType::SurfaceZone;

        // 1. Square Surface Zone Creation via Drag
        tools.active_surface_shape = SurfaceShapeType::Square;
        tools.active_surface = SurfaceType::Sand;
        tools.active_surface_layer = SurfaceLayer::BelowTrack;
        tools.handle_mouse_down(&mut state, Vec2::new(10.0, 10.0));
        tools.handle_mouse_drag(&mut state, Vec2::new(30.0, 30.0));
        tools.handle_mouse_up(&mut state, Vec2::new(30.0, 30.0));

        let zone_idx = state.track.geometry.surface_zones.len() - 1;
        assert_eq!(state.selection, Selection::SurfaceZone(zone_idx));
        assert_eq!(state.track.geometry.surface_zones[zone_idx].surface, SurfaceType::Sand);
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
        let track = classic_grand_prix();
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
        let track = classic_grand_prix();
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

        // 6. StartingGrid Tool selects grid slots
        tools.active_tool = EditorToolType::StartingGrid;
        assert!(tools.select_all_for_active_tool(&mut state));
        assert_eq!(state.selection.selected_grid_slot_indices().len(), state.track.grid_positions.len());
    }

    #[test]
    fn test_box_selection_and_multi_entity_drag_and_batch_ops() {
        let track = classic_grand_prix();
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
        let track = classic_grand_prix();
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

        // 4. Adjust Pitch & Height
        assert!(tools.adjust_selected_jump_ramp_pitch(&mut state, 5.0));
        assert_eq!(state.track.geometry.jump_ramps[ramp_idx].ramp_angle_deg, 20.0);
        assert!(tools.adjust_selected_jump_ramp_height(&mut state, 0.5));
        assert!((state.track.geometry.jump_ramps[ramp_idx].height - 2.3).abs() < 1e-4);

        // 5. Undo restores previous states
        assert!(state.undo());
        assert!((state.track.geometry.jump_ramps[ramp_idx].height - 1.8).abs() < 1e-4);
    }

    #[test]
    fn test_batch_set_walls_and_waypoint_wall_toggles() {
        let track = classic_grand_prix();
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
}

