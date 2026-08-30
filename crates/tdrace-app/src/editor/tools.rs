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

    // Dragging interaction state
    pub is_dragging: bool,
    pub drag_start_world: Vec2,
    pub drag_current_world: Vec2,
    pub drag_initial_entity_pos: Vec2,
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
            is_dragging: false,
            drag_start_world: Vec2::ZERO,
            drag_current_world: Vec2::ZERO,
            drag_initial_entity_pos: Vec2::ZERO,
        }
    }
}

impl ToolSettings {
    /// Duplicates currently selected obstacle, surface zone, jump ramp, waypoint, checkpoint, or grid slot.
    pub fn duplicate_selected(&mut self, state: &mut EditorState) -> bool {
        match state.selection {
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
            _ => {}
        }
        false
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
}

impl ToolSettings {
    /// Handles mouse down event in world space.
    pub fn handle_mouse_down(&mut self, state: &mut EditorState, mouse_world: Vec2) {
        let snapped_mouse = state.grid_snap.snap_point(mouse_world);

        match self.active_tool {
            EditorToolType::Select => {
                // Find closest selectable entity
                if let Some(sel) = find_closest_entity(state, mouse_world) {
                    state.select(sel);
                    if let Selection::Waypoint(idx) = sel {
                        if let Some(wp) = state.track.spline.waypoints.get(idx) {
                            self.active_surface = wp.surface.unwrap_or(SurfaceType::Asphalt);
                            self.new_waypoint_width = wp.width;
                            self.new_waypoint_left_curb = wp.left_curb;
                            self.new_waypoint_right_curb = wp.right_curb;
                        }
                    }
                    self.is_dragging = true;
                    self.drag_start_world = snapped_mouse;
                    self.drag_current_world = snapped_mouse;
                    self.drag_initial_entity_pos = get_selection_position(state, sel).unwrap_or(mouse_world);
                } else {
                    state.selection = Selection::None;
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
                    }
                    self.is_dragging = true;
                    self.drag_start_world = snapped_mouse;
                    self.drag_current_world = snapped_mouse;
                    self.drag_initial_entity_pos = state.track.spline.waypoints[wp_idx].point;
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

        let snapped_mouse = state.grid_snap.snap_point(mouse_world);
        self.drag_current_world = snapped_mouse;

        match self.active_tool {
            EditorToolType::Select | EditorToolType::RoadSpline => {
                let delta = snapped_mouse - self.drag_start_world;
                let new_pos = self.drag_initial_entity_pos + delta;

                match state.selection {
                    Selection::Waypoint(idx) => {
                        if idx < state.track.spline.waypoints.len() {
                            state.track.spline.waypoints[idx].point = new_pos;
                            state.rebuild_geometry();
                        }
                    }
                    Selection::SurfaceZone(idx) => {
                        if idx < state.track.geometry.surface_zones.len() {
                            set_surface_zone_position(&mut state.track.geometry.surface_zones[idx], new_pos);
                        }
                    }
                    Selection::Obstacle(idx) => {
                        if idx < state.track.geometry.obstacles.len() {
                            set_obstacle_position(&mut state.track.geometry.obstacles[idx], new_pos);
                        }
                    }
                    Selection::JumpRamp(idx) => {
                        if idx < state.track.geometry.jump_ramps.len() {
                            set_jump_ramp_position(&mut state.track.geometry.jump_ramps[idx], new_pos);
                        }
                    }
                    Selection::Checkpoint(idx) => {
                        if idx < state.track.checkpoints.len() {
                            let gate = &mut state.track.checkpoints[idx].gate;
                            let gate_delta = new_pos - (gate.start + gate.end) * 0.5;
                            gate.start += gate_delta;
                            gate.end += gate_delta;
                        }
                    }
                    Selection::GridSlot(idx) => {
                        if idx < state.track.grid_positions.len() {
                            state.track.grid_positions[idx].position = new_pos;
                        }
                    }
                    Selection::PitBox => {
                        if let Some(SurfaceShape::Aabb { min, max }) = &mut state.track.pit_box_area {
                            let half_w = (*max - *min) * 0.5;
                            *min = new_pos - half_w;
                            *max = new_pos + half_w;
                        }
                    }
                    Selection::None => {}
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
                let ramp = JumpRamp::new(ramp_id, shape, dir, 24.0, 15.0, 1.8, format!("Jump Ramp {}", ramp_id));
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
        match state.selection {
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

fn get_selection_position(state: &EditorState, sel: Selection) -> Option<Vec2> {
    match sel {
        Selection::Waypoint(idx) => state.track.spline.waypoints.get(idx).map(|w| w.point),
        Selection::SurfaceZone(idx) => state.track.geometry.surface_zones.get(idx).map(|z| get_surface_shape_center(&z.shape)),
        Selection::Obstacle(idx) => state.track.geometry.obstacles.get(idx).map(|o| o.center()),
        Selection::JumpRamp(idx) => state.track.geometry.jump_ramps.get(idx).map(|r| get_surface_shape_center(&r.shape)),
        Selection::Checkpoint(idx) => state.track.checkpoints.get(idx).map(|c| (c.gate.start + c.gate.end) * 0.5),
        Selection::GridSlot(idx) => state.track.grid_positions.get(idx).map(|g| g.position),
        Selection::PitBox => state.track.pit_box_area.as_ref().map(get_surface_shape_center),
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

/// Renders gizmos, selection indicators, handles, and drag previews in world space.
pub fn render_editor_gizmos(state: &EditorState, tools: &ToolSettings, _camera: &EditorCamera) {
    // 1. Render Waypoint nodes & handles
    let n_wp = state.track.spline.waypoints.len();
    for (i, wp) in state.track.spline.waypoints.iter().enumerate() {
        let is_selected = state.selection == Selection::Waypoint(i);

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
        let is_selected = state.selection == Selection::Checkpoint(cp.id);
        let gate_col = if cp.is_finish_line {
            Palette::NEON_GOLD
        } else if is_selected {
            Palette::NEON_CYAN
        } else {
            Color::new(0.2, 0.85, 0.4, 0.8)
        };

        let thickness = if is_selected { 0.6 } else { 0.35 };
        draw_line(cp.gate.start.x, cp.gate.start.y, cp.gate.end.x, cp.gate.end.y, thickness, gate_col);

        // Direction arrow
        let center = (cp.gate.start + cp.gate.end) * 0.5;
        let arrow_tip = center + cp.direction * 3.0;
        draw_line(center.x, center.y, arrow_tip.x, arrow_tip.y, 0.3, gate_col);
    }

    // 3. Render Starting Grid Slot gizmos
    for slot in &state.track.grid_positions {
        let is_selected = state.selection == Selection::GridSlot(slot.grid_slot);
        let col = if is_selected { Palette::NEON_GOLD } else { Palette::NEON_MAGENTA };

        draw_circle(slot.position.x, slot.position.y, 1.4, col);
        let fwd = Vec2::new(slot.angle.cos(), slot.angle.sin()) * 2.5;
        draw_line(slot.position.x, slot.position.y, slot.position.x + fwd.x, slot.position.y + fwd.y, 0.4, col);
    }

    // 4. Render Active Drag Box preview
    if tools.is_dragging {
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
                draw_line(
                    tools.drag_start_world.x,
                    tools.drag_start_world.y,
                    tools.drag_current_world.x,
                    tools.drag_current_world.y,
                    0.6,
                    Palette::NEON_GOLD,
                );
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

    // 5. Render In-progress Polygon / Triangle Construction vertices
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
}
