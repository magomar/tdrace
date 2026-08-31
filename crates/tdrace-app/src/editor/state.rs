use glam::Vec2;
use tdrace_core::track::validation::{validate_track, TrackValidationError};
use tdrace_core::track::{BarrierType, Track};

/// Metric grid snap step settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSnapSetting {
    Off,
    Snap1m,
    Snap2_5m,
    Snap5m,
    Snap10m,
}

impl GridSnapSetting {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Snap1m,
        Self::Snap2_5m,
        Self::Snap5m,
        Self::Snap10m,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => "Snap: OFF",
            Self::Snap1m => "Snap: 1.0m",
            Self::Snap2_5m => "Snap: 2.5m",
            Self::Snap5m => "Snap: 5.0m",
            Self::Snap10m => "Snap: 10.0m",
        }
    }

    pub fn step(&self) -> Option<f32> {
        match self {
            Self::Off => None,
            Self::Snap1m => Some(1.0),
            Self::Snap2_5m => Some(2.5),
            Self::Snap5m => Some(5.0),
            Self::Snap10m => Some(10.0),
        }
    }

    pub fn snap_point(&self, p: Vec2) -> Vec2 {
        if let Some(s) = self.step() {
            Vec2::new((p.x / s).round() * s, (p.y / s).round() * s)
        } else {
            p
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::Snap1m,
            Self::Snap1m => Self::Snap2_5m,
            Self::Snap2_5m => Self::Snap5m,
            Self::Snap5m => Self::Snap10m,
            Self::Snap10m => Self::Off,
        }
    }
}

/// Identifies an entity selected in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    None,
    Waypoint(usize),
    MultipleWaypoints(Vec<usize>),
    SurfaceZone(usize),
    Obstacle(usize),
    JumpRamp(usize),
    Checkpoint(usize),
    GridSlot(usize),
    PitBox,
    Multi {
        waypoints: Vec<usize>,
        surface_zones: Vec<usize>,
        obstacles: Vec<usize>,
        jump_ramps: Vec<usize>,
        checkpoints: Vec<usize>,
        grid_slots: Vec<usize>,
        pit_box: bool,
    },
}

impl Selection {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            Self::Multi {
                waypoints,
                surface_zones,
                obstacles,
                jump_ramps,
                checkpoints,
                grid_slots,
                pit_box,
            } => {
                waypoints.is_empty()
                    && surface_zones.is_empty()
                    && obstacles.is_empty()
                    && jump_ramps.is_empty()
                    && checkpoints.is_empty()
                    && grid_slots.is_empty()
                    && !*pit_box
            }
            Self::MultipleWaypoints(wps) => wps.is_empty(),
            _ => false,
        }
    }

    pub fn is_waypoint(&self) -> Option<usize> {
        if let Self::Waypoint(idx) = self {
            Some(*idx)
        } else {
            None
        }
    }

    pub fn total_count(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Waypoint(_)
            | Self::SurfaceZone(_)
            | Self::Obstacle(_)
            | Self::JumpRamp(_)
            | Self::Checkpoint(_)
            | Self::GridSlot(_)
            | Self::PitBox => 1,
            Self::MultipleWaypoints(indices) => indices.len(),
            Self::Multi {
                waypoints,
                surface_zones,
                obstacles,
                jump_ramps,
                checkpoints,
                grid_slots,
                pit_box,
            } => {
                waypoints.len()
                    + surface_zones.len()
                    + obstacles.len()
                    + jump_ramps.len()
                    + checkpoints.len()
                    + grid_slots.len()
                    + if *pit_box { 1 } else { 0 }
            }
        }
    }

    pub fn selected_waypoint_indices(&self) -> Vec<usize> {
        match self {
            Self::Waypoint(idx) => vec![*idx],
            Self::MultipleWaypoints(indices) => indices.clone(),
            Self::Multi { waypoints, .. } => waypoints.clone(),
            _ => Vec::new(),
        }
    }

    pub fn selected_surface_zone_indices(&self) -> Vec<usize> {
        match self {
            Self::SurfaceZone(idx) => vec![*idx],
            Self::Multi { surface_zones, .. } => surface_zones.clone(),
            _ => Vec::new(),
        }
    }

    pub fn selected_obstacle_indices(&self) -> Vec<usize> {
        match self {
            Self::Obstacle(idx) => vec![*idx],
            Self::Multi { obstacles, .. } => obstacles.clone(),
            _ => Vec::new(),
        }
    }

    pub fn selected_jump_ramp_indices(&self) -> Vec<usize> {
        match self {
            Self::JumpRamp(idx) => vec![*idx],
            Self::Multi { jump_ramps, .. } => jump_ramps.clone(),
            _ => Vec::new(),
        }
    }

    pub fn selected_checkpoint_indices(&self) -> Vec<usize> {
        match self {
            Self::Checkpoint(idx) => vec![*idx],
            Self::Multi { checkpoints, .. } => checkpoints.clone(),
            _ => Vec::new(),
        }
    }

    pub fn selected_grid_slot_indices(&self) -> Vec<usize> {
        match self {
            Self::GridSlot(idx) => vec![*idx],
            Self::Multi { grid_slots, .. } => grid_slots.clone(),
            _ => Vec::new(),
        }
    }

    pub fn is_waypoint_selected(&self, idx: usize) -> bool {
        match self {
            Self::Waypoint(i) => *i == idx,
            Self::MultipleWaypoints(indices) => indices.contains(&idx),
            Self::Multi { waypoints, .. } => waypoints.contains(&idx),
            _ => false,
        }
    }

    pub fn is_surface_zone_selected(&self, idx: usize) -> bool {
        match self {
            Self::SurfaceZone(i) => *i == idx,
            Self::Multi { surface_zones, .. } => surface_zones.contains(&idx),
            _ => false,
        }
    }

    pub fn is_obstacle_selected(&self, idx: usize) -> bool {
        match self {
            Self::Obstacle(i) => *i == idx,
            Self::Multi { obstacles, .. } => obstacles.contains(&idx),
            _ => false,
        }
    }

    pub fn is_jump_ramp_selected(&self, idx: usize) -> bool {
        match self {
            Self::JumpRamp(i) => *i == idx,
            Self::Multi { jump_ramps, .. } => jump_ramps.contains(&idx),
            _ => false,
        }
    }

    pub fn is_checkpoint_selected(&self, idx: usize) -> bool {
        match self {
            Self::Checkpoint(i) => *i == idx,
            Self::Multi { checkpoints, .. } => checkpoints.contains(&idx),
            _ => false,
        }
    }

    pub fn is_grid_slot_selected(&self, idx: usize) -> bool {
        match self {
            Self::GridSlot(i) => *i == idx,
            Self::Multi { grid_slots, .. } => grid_slots.contains(&idx),
            _ => false,
        }
    }

    pub fn is_pit_box_selected(&self) -> bool {
        match self {
            Self::PitBox => true,
            Self::Multi { pit_box, .. } => *pit_box,
            _ => false,
        }
    }

    pub fn contains_entity(&self, other: &Selection) -> bool {
        match other {
            Self::Waypoint(i) => self.is_waypoint_selected(*i),
            Self::SurfaceZone(i) => self.is_surface_zone_selected(*i),
            Self::Obstacle(i) => self.is_obstacle_selected(*i),
            Self::JumpRamp(i) => self.is_jump_ramp_selected(*i),
            Self::Checkpoint(i) => self.is_checkpoint_selected(*i),
            Self::GridSlot(i) => self.is_grid_slot_selected(*i),
            Self::PitBox => self.is_pit_box_selected(),
            _ => false,
        }
    }

    pub fn from_multi(
        mut waypoints: Vec<usize>,
        mut surface_zones: Vec<usize>,
        mut obstacles: Vec<usize>,
        mut jump_ramps: Vec<usize>,
        mut checkpoints: Vec<usize>,
        mut grid_slots: Vec<usize>,
        pit_box: bool,
    ) -> Self {
        waypoints.sort_unstable();
        waypoints.dedup();
        surface_zones.sort_unstable();
        surface_zones.dedup();
        obstacles.sort_unstable();
        obstacles.dedup();
        jump_ramps.sort_unstable();
        jump_ramps.dedup();
        checkpoints.sort_unstable();
        checkpoints.dedup();
        grid_slots.sort_unstable();
        grid_slots.dedup();

        Self::Multi {
            waypoints,
            surface_zones,
            obstacles,
            jump_ramps,
            checkpoints,
            grid_slots,
            pit_box,
        }
        .normalize()
    }

    pub fn normalize(self) -> Self {
        match self {
            Self::Multi {
                waypoints,
                surface_zones,
                obstacles,
                jump_ramps,
                checkpoints,
                grid_slots,
                pit_box,
            } => {
                let total = waypoints.len()
                    + surface_zones.len()
                    + obstacles.len()
                    + jump_ramps.len()
                    + checkpoints.len()
                    + grid_slots.len()
                    + if pit_box { 1 } else { 0 };

                if total == 0 {
                    Self::None
                } else if total == 1 {
                    if let Some(&w) = waypoints.first() {
                        Self::Waypoint(w)
                    } else if let Some(&sz) = surface_zones.first() {
                        Self::SurfaceZone(sz)
                    } else if let Some(&obs) = obstacles.first() {
                        Self::Obstacle(obs)
                    } else if let Some(&ramp) = jump_ramps.first() {
                        Self::JumpRamp(ramp)
                    } else if let Some(&cp) = checkpoints.first() {
                        Self::Checkpoint(cp)
                    } else if let Some(&slot) = grid_slots.first() {
                        Self::GridSlot(slot)
                    } else if pit_box {
                        Self::PitBox
                    } else {
                        Self::None
                    }
                } else if waypoints.len() == total {
                    Self::MultipleWaypoints(waypoints)
                } else {
                    Self::Multi {
                        waypoints,
                        surface_zones,
                        obstacles,
                        jump_ramps,
                        checkpoints,
                        grid_slots,
                        pit_box,
                    }
                }
            }
            _ => self,
        }
    }

    pub fn union(&self, other: &Selection) -> Self {
        let mut waypoints = self.selected_waypoint_indices();
        let mut surface_zones = self.selected_surface_zone_indices();
        let mut obstacles = self.selected_obstacle_indices();
        let mut jump_ramps = self.selected_jump_ramp_indices();
        let mut checkpoints = self.selected_checkpoint_indices();
        let mut grid_slots = self.selected_grid_slot_indices();
        let pit_box = self.is_pit_box_selected() || other.is_pit_box_selected();

        waypoints.extend(other.selected_waypoint_indices());
        surface_zones.extend(other.selected_surface_zone_indices());
        obstacles.extend(other.selected_obstacle_indices());
        jump_ramps.extend(other.selected_jump_ramp_indices());
        checkpoints.extend(other.selected_checkpoint_indices());
        grid_slots.extend(other.selected_grid_slot_indices());

        Self::from_multi(
            waypoints,
            surface_zones,
            obstacles,
            jump_ramps,
            checkpoints,
            grid_slots,
            pit_box,
        )
    }

    pub fn toggle_waypoint(&mut self, idx: usize) {
        match self {
            Self::Waypoint(existing) => {
                if *existing == idx {
                    *self = Self::None;
                } else {
                    let mut list = vec![*existing, idx];
                    list.sort_unstable();
                    list.dedup();
                    *self = Self::MultipleWaypoints(list);
                }
            }
            Self::MultipleWaypoints(indices) => {
                if let Some(pos) = indices.iter().position(|&x| x == idx) {
                    indices.remove(pos);
                    if indices.is_empty() {
                        *self = Self::None;
                    } else if indices.len() == 1 {
                        *self = Self::Waypoint(indices[0]);
                    }
                } else {
                    indices.push(idx);
                    indices.sort_unstable();
                    indices.dedup();
                }
            }
            Self::Multi { waypoints, .. } => {
                if let Some(pos) = waypoints.iter().position(|&x| x == idx) {
                    waypoints.remove(pos);
                } else {
                    waypoints.push(idx);
                    waypoints.sort_unstable();
                    waypoints.dedup();
                }
                *self = self.clone().normalize();
            }
            _ => {
                *self = Self::Waypoint(idx);
            }
        }
    }
}

/// Robust snapshot-based Undo / Redo history ring buffer.
#[derive(Debug, Clone)]
pub struct HistoryStack {
    undo_stack: Vec<Track>,
    redo_stack: Vec<Track>,
    max_depth: usize,
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self::new(50)
    }
}

impl HistoryStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn push_snapshot(&mut self, track: &Track) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(track.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_track: &mut Track) -> bool {
        if let Some(prev_state) = self.undo_stack.pop() {
            self.redo_stack.push(current_track.clone());
            *current_track = prev_state;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, current_track: &mut Track) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(current_track.clone());
            *current_track = next_state;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

/// Central state container for the Track Editor.
pub struct EditorState {
    pub track: Track,
    pub history: HistoryStack,
    pub selection: Selection,
    pub last_selected_waypoint: Option<usize>,
    pub grid_snap: GridSnapSetting,
    pub show_grid: bool,
    pub show_diagnostics_overlay: bool,
    pub is_dirty: bool,
    pub current_file_path: Option<String>,
    pub diagnostics: Vec<TrackValidationError>,

    // Generator settings
    pub barrier_offset: f32,
    pub barrier_type: BarrierType,
}

impl EditorState {
    pub fn new(track: Track) -> Self {
        let diagnostics = validate_track(&track);
        Self {
            track,
            history: HistoryStack::default(),
            selection: Selection::None,
            last_selected_waypoint: None,
            grid_snap: GridSnapSetting::Snap2_5m,
            show_grid: true,
            show_diagnostics_overlay: true,
            is_dirty: false,
            current_file_path: None,
            diagnostics,
            barrier_offset: 4.0,
            barrier_type: BarrierType::Armco,
        }
    }

    /// Records current state in undo history before making modifications.
    pub fn record_undo(&mut self) {
        self.history.push_snapshot(&self.track);
        self.is_dirty = true;
    }

    /// Performs undo operation.
    pub fn undo(&mut self) -> bool {
        if self.history.undo(&mut self.track) {
            self.revalidate();
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Performs redo operation.
    pub fn redo(&mut self) -> bool {
        if self.history.redo(&mut self.track) {
            self.revalidate();
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Re-evaluates all circuit diagnostics and validation rules.
    pub fn revalidate(&mut self) {
        self.diagnostics = validate_track(&self.track);
    }

    /// Rebuilds spline, boundary walls, and polylines from current waypoints.
    pub fn rebuild_geometry(&mut self) {
        self.track.rebuild_geometry(self.barrier_offset, self.barrier_type);
        self.revalidate();
    }

    /// Updates current selection and records last selected waypoint if applicable.
    pub fn select(&mut self, selection: Selection) {
        match &selection {
            Selection::Waypoint(idx) => {
                self.last_selected_waypoint = Some(*idx);
            }
            Selection::MultipleWaypoints(indices) => {
                if let Some(&first) = indices.first() {
                    self.last_selected_waypoint = Some(first);
                }
            }
            Selection::Multi { waypoints, .. } => {
                if let Some(&first) = waypoints.first() {
                    self.last_selected_waypoint = Some(first);
                }
            }
            _ => {}
        }
        self.selection = selection;
    }

    /// Deselects any active entity without losing last selected waypoint index.
    pub fn deselect(&mut self) {
        self.selection = Selection::None;
    }

    /// Returns the index of the currently or last selected waypoint, if valid.
    pub fn current_or_last_waypoint_idx(&self) -> Option<usize> {
        match &self.selection {
            Selection::Waypoint(idx) => {
                if *idx < self.track.spline.waypoints.len() {
                    return Some(*idx);
                }
            }
            Selection::MultipleWaypoints(indices) => {
                if let Some(&last) = indices.last() {
                    if last < self.track.spline.waypoints.len() {
                        return Some(last);
                    }
                }
            }
            Selection::Multi { waypoints, .. } => {
                if let Some(&last) = waypoints.last() {
                    if last < self.track.spline.waypoints.len() {
                        return Some(last);
                    }
                }
            }
            _ => {}
        }
        if let Some(idx) = self.last_selected_waypoint {
            if idx < self.track.spline.waypoints.len() {
                return Some(idx);
            }
        }
        None
    }

    /// Computes track bounding box (min, max) for camera framing.
    pub fn compute_bounds(&self) -> (Vec2, Vec2) {
        if self.track.spline.waypoints.is_empty() {
            return (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0));
        }

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for wp in &self.track.spline.waypoints {
            let p = wp.point;
            let hw = wp.width * 0.5 + self.barrier_offset + 10.0;
            min = min.min(p - Vec2::splat(hw));
            max = max.max(p + Vec2::splat(hw));
        }

        for zone in &self.track.geometry.surface_zones {
            match &zone.shape {
                tdrace_core::track::SurfaceShape::Circle { center, radius } => {
                    min = min.min(*center - Vec2::splat(*radius));
                    max = max.max(*center + Vec2::splat(*radius));
                }
                tdrace_core::track::SurfaceShape::Aabb {
                    min: z_min,
                    max: z_max,
                } => {
                    min = min.min(*z_min);
                    max = max.max(*z_max);
                }
                tdrace_core::track::SurfaceShape::OrientedBox {
                    center,
                    half_extents,
                    ..
                } => {
                    let rad = half_extents.length();
                    min = min.min(*center - Vec2::splat(rad));
                    max = max.max(*center + Vec2::splat(rad));
                }
                tdrace_core::track::SurfaceShape::Polygon { vertices } => {
                    for v in vertices {
                        min = min.min(*v);
                        max = max.max(*v);
                    }
                }
            }
        }

        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdrace_core::track::presets::classic_grand_prix;

    #[test]
    fn test_grid_snap_steps() {
        let snap = GridSnapSetting::Snap5m;
        let p = Vec2::new(12.3, 18.7);
        let snapped = snap.snap_point(p);
        assert_eq!(snapped, Vec2::new(10.0, 20.0));

        let snap_off = GridSnapSetting::Off;
        assert_eq!(snap_off.snap_point(p), p);
    }

    #[test]
    fn test_history_undo_redo_stack() {
        let track = classic_grand_prix();
        let mut editor = EditorState::new(track.clone());

        // Perform edit 1
        editor.record_undo();
        editor.track.name = "Modified Track".to_string();

        assert_eq!(editor.track.name, "Modified Track");
        assert!(editor.history.can_undo());
        assert!(!editor.history.can_redo());

        // Undo edit 1
        assert!(editor.undo());
        assert_eq!(editor.track.name, "Classic Grand Prix");
        assert!(editor.history.can_redo());

        // Redo edit 1
        assert!(editor.redo());
        assert_eq!(editor.track.name, "Modified Track");
    }

    #[test]
    fn test_waypoint_selection_and_last_selected_tracking() {
        let track = classic_grand_prix();
        let mut editor = EditorState::new(track);

        assert_eq!(editor.selection, Selection::None);
        assert_eq!(editor.last_selected_waypoint, None);
        assert_eq!(editor.current_or_last_waypoint_idx(), None);

        // Select waypoint 2
        editor.select(Selection::Waypoint(2));
        assert_eq!(editor.selection, Selection::Waypoint(2));
        assert_eq!(editor.last_selected_waypoint, Some(2));
        assert_eq!(editor.current_or_last_waypoint_idx(), Some(2));

        // Deselect -> selection is None, but last_selected_waypoint is still Some(2)
        editor.deselect();
        assert_eq!(editor.selection, Selection::None);
        assert_eq!(editor.last_selected_waypoint, Some(2));
        assert_eq!(editor.current_or_last_waypoint_idx(), Some(2));

        // Select an obstacle -> selection is Obstacle(0), last_selected_waypoint is still Some(2)
        editor.select(Selection::Obstacle(0));
        assert_eq!(editor.selection, Selection::Obstacle(0));
        assert_eq!(editor.last_selected_waypoint, Some(2));
        assert_eq!(editor.current_or_last_waypoint_idx(), Some(2));

        // Select waypoint 5
        editor.select(Selection::Waypoint(5));
        assert_eq!(editor.current_or_last_waypoint_idx(), Some(5));
    }

    #[test]
    fn test_multi_selection_normalization_and_union() {
        // Empty -> None
        let empty = Selection::from_multi(vec![], vec![], vec![], vec![], vec![], vec![], false);
        assert_eq!(empty, Selection::None);
        assert!(empty.is_none());

        // Single waypoint -> Waypoint(1)
        let single_wp = Selection::from_multi(vec![1], vec![], vec![], vec![], vec![], vec![], false);
        assert_eq!(single_wp, Selection::Waypoint(1));
        assert_eq!(single_wp.total_count(), 1);

        // Multiple waypoints -> MultipleWaypoints([1, 3])
        let multi_wp = Selection::from_multi(vec![3, 1], vec![], vec![], vec![], vec![], vec![], false);
        assert_eq!(multi_wp, Selection::MultipleWaypoints(vec![1, 3]));
        assert_eq!(multi_wp.total_count(), 2);

        // Mixed entities -> Multi
        let mixed = Selection::from_multi(vec![1], vec![0], vec![2], vec![], vec![], vec![], false);
        assert!(matches!(mixed, Selection::Multi { .. }));
        assert_eq!(mixed.total_count(), 3);
        assert!(mixed.is_waypoint_selected(1));
        assert!(mixed.is_surface_zone_selected(0));
        assert!(mixed.is_obstacle_selected(2));
        assert!(!mixed.is_jump_ramp_selected(0));

        // Union of single obstacle and waypoint
        let obs_sel = Selection::Obstacle(5);
        let combined = single_wp.union(&obs_sel);
        assert_eq!(combined.total_count(), 2);
        assert!(combined.is_waypoint_selected(1));
        assert!(combined.is_obstacle_selected(5));
    }
}
