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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    None,
    Waypoint(usize),
    SurfaceZone(usize),
    Obstacle(usize),
    JumpRamp(usize),
    Checkpoint(usize),
    GridSlot(usize),
    PitBox,
}

impl Selection {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_waypoint(&self) -> Option<usize> {
        if let Self::Waypoint(idx) = self {
            Some(*idx)
        } else {
            None
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
        if let Selection::Waypoint(idx) = selection {
            self.last_selected_waypoint = Some(idx);
        }
        self.selection = selection;
    }

    /// Deselects any active entity without losing last selected waypoint index.
    pub fn deselect(&mut self) {
        self.selection = Selection::None;
    }

    /// Returns the index of the currently or last selected waypoint, if valid.
    pub fn current_or_last_waypoint_idx(&self) -> Option<usize> {
        if let Selection::Waypoint(idx) = self.selection {
            if idx < self.track.spline.waypoints.len() {
                return Some(idx);
            }
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
}
