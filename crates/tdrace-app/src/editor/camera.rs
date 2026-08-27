use glam::Vec2;
use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::prelude::{screen_height, screen_width};
use crate::config::{CameraConfig, ZoomLevelConfig};

/// Dedicated CAD-style pan, zoom, and framing camera for the track editor.
#[derive(Debug, Clone)]
pub struct EditorCamera {
    pub center: Vec2,
    pub target_center: Vec2,
    pub zoom: f32, // Pixels per meter
    pub target_zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub pan_smoothing: f32,
    pub zoom_smoothing: f32,

    pub is_panning: bool,
    pub pan_start_mouse: Vec2,
    pub pan_start_center: Vec2,

    // Multi-level zoom management
    pub levels: Vec<ZoomLevelConfig>,
    pub current_level_idx: usize,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCamera {
    pub fn new() -> Self {
        Self::from_config(&CameraConfig::default())
    }

    /// Constructs camera instance from a `CameraConfig`.
    pub fn from_config(config: &CameraConfig) -> Self {
        let levels = if config.levels.is_empty() {
            CameraConfig::default().levels
        } else {
            config.levels.clone()
        };

        let initial_idx = config.default_level_index.min(levels.len().saturating_sub(1));
        let active_level = &levels[initial_idx];
        let initial_zoom = if active_level.is_overview() {
            active_level.min_zoom
        } else {
            active_level.max_zoom
        };

        Self {
            center: Vec2::new(100.0, 100.0),
            target_center: Vec2::new(100.0, 100.0),
            zoom: initial_zoom,
            target_zoom: initial_zoom,
            min_zoom: 0.5,
            max_zoom: 60.0,
            pan_smoothing: 18.0,
            zoom_smoothing: 16.0,
            is_panning: false,
            pan_start_mouse: Vec2::ZERO,
            pan_start_center: Vec2::ZERO,
            levels,
            current_level_idx: initial_idx,
        }
    }

    /// Returns the currently active zoom level configuration.
    pub fn current_zoom_level(&self) -> &ZoomLevelConfig {
        &self.levels[self.current_level_idx]
    }

    /// Explicitly activates a zoom level by index without explicit bounds.
    pub fn set_zoom_level(&mut self, idx: usize) -> ZoomLevelConfig {
        let (sw, sh) = Self::get_screen_dimensions();
        self.set_zoom_level_with_bounds(idx, None, sw, sh)
    }

    /// Explicitly activates a zoom level by index with optional track bounding box for overview framing.
    pub fn set_zoom_level_with_bounds(
        &mut self,
        idx: usize,
        bounds: Option<(Vec2, Vec2)>,
        sw: f32,
        sh: f32,
    ) -> ZoomLevelConfig {
        if self.levels.is_empty() {
            self.levels = CameraConfig::default().levels;
        }
        self.current_level_idx = idx % self.levels.len();
        let lvl = self.levels[self.current_level_idx].clone();

        if lvl.is_overview() {
            if let Some((min, max)) = bounds {
                if min.x <= max.x {
                    self.focus_bounds(min, max, sw, sh);
                } else {
                    self.target_zoom = lvl.min_zoom;
                }
            } else {
                self.target_zoom = lvl.min_zoom;
            }
        } else {
            self.target_zoom = lvl.max_zoom;
        }

        lvl
    }

    /// Cycles to the next zoom level in sequence without explicit bounds.
    pub fn cycle_zoom_level(&mut self) -> ZoomLevelConfig {
        let (sw, sh) = Self::get_screen_dimensions();
        self.cycle_zoom_level_with_bounds(None, sw, sh)
    }

    /// Cycles to the next zoom level in sequence with optional track bounds.
    pub fn cycle_zoom_level_with_bounds(
        &mut self,
        bounds: Option<(Vec2, Vec2)>,
        sw: f32,
        sh: f32,
    ) -> ZoomLevelConfig {
        if self.levels.is_empty() {
            self.levels = CameraConfig::default().levels;
        }
        let next_idx = (self.current_level_idx + 1) % self.levels.len();
        self.set_zoom_level_with_bounds(next_idx, bounds, sw, sh)
    }

    /// Safely gets screen dimensions without panicking.
    pub fn get_screen_dimensions() -> (f32, f32) {
        let sw = std::panic::catch_unwind(screen_width).unwrap_or(1280.0);
        let sh = std::panic::catch_unwind(screen_height).unwrap_or(720.0);
        (sw.max(320.0), sh.max(240.0))
    }

    /// Converts world coordinates to screen pixel coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2, sw: f32, sh: f32) -> Vec2 {
        let rel = world_pos - self.center;
        let screen_x = sw * 0.5 + rel.x * self.zoom;
        let screen_y = sh * 0.5 - rel.y * self.zoom;
        Vec2::new(screen_x, screen_y)
    }

    /// Converts screen pixel coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2, sw: f32, sh: f32) -> Vec2 {
        let rel_x = (screen_pos.x - sw * 0.5) / self.zoom;
        let rel_y = -(screen_pos.y - sh * 0.5) / self.zoom;
        Vec2::new(self.center.x + rel_x, self.center.y + rel_y)
    }

    /// Smoothly animates camera toward target position and zoom.
    pub fn update(&mut self, dt: f32) {
        let pan_blend = 1.0 - (-self.pan_smoothing * dt).exp();
        self.center += (self.target_center - self.center) * pan_blend;

        let zoom_blend = 1.0 - (-self.zoom_smoothing * dt).exp();
        self.zoom += (self.target_zoom - self.zoom) * zoom_blend;
    }

    /// Applies Macroquad 2D camera transform for the editor world.
    pub fn apply(&self, sw: f32, sh: f32) {
        let zoom_x = (2.0 * self.zoom) / sw;
        let zoom_y = (-2.0 * self.zoom) / sh; // +Y is up in world space

        let camera = Camera2D {
            target: macroquad::prelude::Vec2::new(self.center.x, self.center.y),
            zoom: macroquad::prelude::Vec2::new(zoom_x, zoom_y),
            offset: macroquad::prelude::Vec2::ZERO,
            rotation: 0.0,
            render_target: None,
            viewport: None,
        };
        set_camera(&camera);
    }

    /// Resets back to screen-space default camera.
    pub fn reset_to_screen(&self) {
        set_default_camera();
    }

    /// Starts panning operation from a screen coordinate.
    pub fn start_pan(&mut self, mouse_screen_pos: Vec2) {
        self.is_panning = true;
        self.pan_start_mouse = mouse_screen_pos;
        self.pan_start_center = self.target_center;
    }

    /// Updates pan displacement based on current mouse screen coordinate.
    pub fn update_pan(&mut self, current_mouse_pos: Vec2) {
        if self.is_panning {
            let screen_delta = current_mouse_pos - self.pan_start_mouse;
            let world_delta = Vec2::new(
                -screen_delta.x / self.zoom,
                screen_delta.y / self.zoom,
            );
            self.target_center = self.pan_start_center + world_delta;
            self.center = self.target_center;
        }
    }

    /// Ends active panning.
    pub fn end_pan(&mut self) {
        self.is_panning = false;
    }

    /// Zooms in or out centered at a specific screen coordinate.
    pub fn zoom_at(&mut self, screen_pos: Vec2, factor: f32, sw: f32, sh: f32) {
        let world_before = self.screen_to_world(screen_pos, sw, sh);
        let new_zoom = (self.target_zoom * factor).clamp(self.min_zoom, self.max_zoom);
        self.target_zoom = new_zoom;
        self.zoom = new_zoom; // Instant response for mouse wheel

        let world_after = self.screen_to_world(screen_pos, sw, sh);
        let shift = world_before - world_after;
        self.target_center += shift;
        self.center += shift;
    }

    /// Automatically frames the given bounding box into the viewport with padding.
    pub fn focus_bounds(&mut self, min: Vec2, max: Vec2, sw: f32, sh: f32) {
        let size = max - min;
        let w = size.x.max(30.0);
        let h = size.y.max(30.0);
        let center = (min + max) * 0.5;

        let zoom_x = (sw * 0.70) / w;
        let zoom_y = (sh * 0.70) / h;
        let best_zoom = zoom_x.min(zoom_y).clamp(self.min_zoom, self.max_zoom);

        self.target_center = center;
        self.target_zoom = best_zoom;

        // Synchronize active level index to overview if available
        if let Some(idx) = self.levels.iter().position(|l| l.is_overview()) {
            self.current_level_idx = idx;
        }
    }
}
