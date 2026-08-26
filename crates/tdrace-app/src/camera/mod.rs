use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::prelude::{screen_height, screen_width};
use glam::Vec2;
use tdrace_core::physics::car::Car;
use tdrace_core::track::Track;
use crate::config::{CameraConfig, ZoomLevelConfig};

/// Camera mode setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// Smooth follow camera with speed zoom, velocity look-ahead, and screen shake.
    SmoothFollow,
    /// Classic GeneRally static full-track overview camera fitting the entire circuit.
    StaticOverview,
}

/// Advanced 2D arcade race camera system supporting multi-level zoom perspectives.
#[derive(Debug, Clone)]
pub struct RaceCamera {
    pub mode: CameraMode,
    pub target_pos: Vec2,
    pub current_pos: Vec2,
    pub current_zoom: f32,
    pub target_zoom: f32,

    // Smooth follow parameters
    pub position_smoothing: f32,
    pub zoom_smoothing: f32,
    pub velocity_lookahead_time: f32,
    pub min_zoom_scale: f32, // At high speed (zoomed out)
    pub max_zoom_scale: f32, // At low speed / stationary (zoomed in)

    // Static overview parameters
    pub overview_center: Vec2,
    pub overview_zoom: f32,

    // Multi-level zoom management
    pub levels: Vec<ZoomLevelConfig>,
    pub current_level_idx: usize,

    // Trauma-based screen shake
    pub trauma: f32,
    pub trauma_decay: f32,
    pub max_shake_offset: f32,
    shake_seed: f32,
}

impl Default for RaceCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl RaceCamera {
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
        let mode = if active_level.is_overview() {
            CameraMode::StaticOverview
        } else {
            CameraMode::SmoothFollow
        };

        Self {
            mode,
            target_pos: Vec2::ZERO,
            current_pos: Vec2::ZERO,
            current_zoom: active_level.max_zoom,
            target_zoom: active_level.max_zoom,

            position_smoothing: config.position_smoothing,
            zoom_smoothing: config.zoom_smoothing,
            velocity_lookahead_time: config.velocity_lookahead_time,
            min_zoom_scale: active_level.min_zoom,
            max_zoom_scale: active_level.max_zoom,

            overview_center: Vec2::ZERO,
            overview_zoom: 3.5,

            levels,
            current_level_idx: initial_idx,

            trauma: 0.0,
            trauma_decay: config.trauma_decay,
            max_shake_offset: config.max_shake_offset,
            shake_seed: 0.0,
        }
    }

    /// Returns the currently active zoom level configuration.
    pub fn current_zoom_level(&self) -> &ZoomLevelConfig {
        &self.levels[self.current_level_idx]
    }

    /// Explicitly activates a zoom level by index and returns its configuration.
    pub fn set_zoom_level(&mut self, idx: usize) -> ZoomLevelConfig {
        if self.levels.is_empty() {
            self.levels = CameraConfig::default().levels;
        }
        self.current_level_idx = idx % self.levels.len();
        let lvl = self.levels[self.current_level_idx].clone();

        if lvl.is_overview() {
            self.mode = CameraMode::StaticOverview;
            self.min_zoom_scale = lvl.min_zoom;
            self.max_zoom_scale = lvl.max_zoom;
        } else {
            self.mode = CameraMode::SmoothFollow;
            self.min_zoom_scale = lvl.min_zoom;
            self.max_zoom_scale = lvl.max_zoom;
        }

        lvl
    }

    /// Cycles to the next zoom level in sequence and returns its configuration.
    pub fn cycle_zoom_level(&mut self) -> ZoomLevelConfig {
        if self.levels.is_empty() {
            self.levels = CameraConfig::default().levels;
        }
        let next_idx = (self.current_level_idx + 1) % self.levels.len();
        self.set_zoom_level(next_idx)
    }

    /// Safely gets screen dimensions without panicking if running outside macroquad loop.
    pub fn get_screen_dimensions_safe() -> (f32, f32) {
        let sw = std::panic::catch_unwind(screen_width).unwrap_or(1280.0);
        let sh = std::panic::catch_unwind(screen_height).unwrap_or(720.0);
        (sw.max(320.0), sh.max(240.0))
    }

    /// Initializes camera parameters for a given track circuit with explicit viewport dimensions.
    pub fn setup_for_track_with_viewport(&mut self, track: &Track, sw: f32, sh: f32) {
        if track.spline.samples.is_empty() {
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for s in &track.spline.samples {
            let hw = s.width * 0.5 + 8.0; // padding for walls and runoff
            let p_left = s.point + s.normal * hw;
            let p_right = s.point - s.normal * hw;

            min_x = min_x.min(p_left.x).min(p_right.x);
            min_y = min_y.min(p_left.y).min(p_right.y);
            max_x = max_x.max(p_left.x).max(p_right.x);
            max_y = max_y.max(p_left.y).max(p_right.y);
        }

        let track_w = (max_x - min_x).max(20.0);
        let track_h = (max_y - min_y).max(20.0);
        self.overview_center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);

        let zoom_x = (sw * 0.88) / track_w;
        let zoom_y = (sh * 0.88) / track_h;
        self.overview_zoom = zoom_x.min(zoom_y).max(1.0);

        if let Some(first_grid) = track.grid_positions.first() {
            self.current_pos = first_grid.position;
            self.target_pos = first_grid.position;
        }
    }

    /// Initializes camera parameters for a given track circuit.
    pub fn setup_for_track(&mut self, track: &Track) {
        let (sw, sh) = Self::get_screen_dimensions_safe();
        self.setup_for_track_with_viewport(track, sw, sh);
    }

    /// Toggles between smooth follow and full-track overview modes.
    pub fn toggle_mode(&mut self) {
        match self.mode {
            CameraMode::SmoothFollow => {
                // Find first overview level, or switch directly
                if let Some(idx) = self.levels.iter().position(|l| l.is_overview()) {
                    self.set_zoom_level(idx);
                } else {
                    self.mode = CameraMode::StaticOverview;
                }
            }
            CameraMode::StaticOverview => {
                // Find first follow level, or switch directly
                if let Some(idx) = self.levels.iter().position(|l| !l.is_overview()) {
                    self.set_zoom_level(idx);
                } else {
                    self.mode = CameraMode::SmoothFollow;
                }
            }
        }
    }

    /// Adds screen shake trauma from collision impact (clamped to [0.0, 1.0]).
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    /// Updates camera positioning, speed-dependent zoom, and shake.
    pub fn update(&mut self, target_car: &Car, dt: f32) {
        self.shake_seed += dt * 35.0;

        // Decay screen shake trauma
        if self.trauma > 0.0 {
            self.trauma = (self.trauma - self.trauma_decay * dt).max(0.0);
        }

        match self.mode {
            CameraMode::SmoothFollow => {
                // Velocity lookahead: look forward in travel direction proportional to speed
                let speed = target_car.state.speed;
                let lookahead = target_car.state.velocity * self.velocity_lookahead_time;
                self.target_pos = target_car.state.position + lookahead;

                // Speed-dependent zoom: zoom out as car accelerates
                let speed_ratio = (speed / 50.0).clamp(0.0, 1.0);
                self.target_zoom = self.max_zoom_scale
                    - speed_ratio * (self.max_zoom_scale - self.min_zoom_scale);

                // Smooth exponential position & zoom interpolation
                let pos_blend = 1.0 - (-self.position_smoothing * dt).exp();
                self.current_pos += (self.target_pos - self.current_pos) * pos_blend;

                let zoom_blend = 1.0 - (-self.zoom_smoothing * dt).exp();
                self.current_zoom += (self.target_zoom - self.current_zoom) * zoom_blend;
            }
            CameraMode::StaticOverview => {
                let pos_blend = 1.0 - (-self.position_smoothing * dt).exp();
                self.current_pos += (self.overview_center - self.current_pos) * pos_blend;

                let zoom_blend = 1.0 - (-self.zoom_smoothing * dt).exp();
                self.current_zoom += (self.overview_zoom - self.current_zoom) * zoom_blend;
            }
        }
    }

    /// Computes current camera view including screen shake offset with given viewport.
    pub fn camera_2d_with_viewport(&self, sw: f32, sh: f32) -> Camera2D {
        let shake_amount = self.trauma * self.trauma;
        let shake_offset = if shake_amount > 1e-4 {
            let sx = (self.shake_seed * 1.3).sin() * self.max_shake_offset * shake_amount;
            let sy = (self.shake_seed * 1.7).cos() * self.max_shake_offset * shake_amount;
            Vec2::new(sx, sy)
        } else {
            Vec2::ZERO
        };

        let view_center = self.current_pos + shake_offset;
        let zoom_x = (2.0 * self.current_zoom) / sw;
        let zoom_y = (-2.0 * self.current_zoom) / sh; // Invert Y so +Y is up / standard 2D Cartesian

        Camera2D {
            target: macroquad::prelude::Vec2::new(view_center.x, view_center.y),
            zoom: macroquad::prelude::Vec2::new(zoom_x, zoom_y),
            offset: macroquad::prelude::Vec2::ZERO,
            rotation: 0.0,
            render_target: None,
            viewport: None,
        }
    }

    /// Computes current camera view including screen shake offset.
    pub fn camera_2d(&self) -> Camera2D {
        let (sw, sh) = Self::get_screen_dimensions_safe();
        self.camera_2d_with_viewport(sw, sh)
    }

    /// Activates this camera in Macroquad.
    pub fn apply(&self) {
        set_camera(&self.camera_2d());
    }

    /// Resets back to screen-space default camera (for HUD/UI rendering).
    pub fn reset_to_screen(&self) {
        set_default_camera();
    }

    /// Converts world coordinates to screen pixel coordinates with given viewport.
    pub fn world_to_screen_with_viewport(&self, world_pos: Vec2, sw: f32, sh: f32) -> Vec2 {
        let rel = world_pos - self.current_pos;
        let screen_x = sw * 0.5 + rel.x * self.current_zoom;
        let screen_y = sh * 0.5 - rel.y * self.current_zoom;
        Vec2::new(screen_x, screen_y)
    }

    /// Converts screen pixel coordinates to world coordinates with given viewport.
    pub fn screen_to_world_with_viewport(&self, screen_pos: Vec2, sw: f32, sh: f32) -> Vec2 {
        let rel_x = (screen_pos.x - sw * 0.5) / self.current_zoom;
        let rel_y = -(screen_pos.y - sh * 0.5) / self.current_zoom;
        Vec2::new(self.current_pos.x + rel_x, self.current_pos.y + rel_y)
    }

    /// Converts world coordinates to screen pixel coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let (sw, sh) = Self::get_screen_dimensions_safe();
        self.world_to_screen_with_viewport(world_pos, sw, sh)
    }

    /// Converts screen pixel coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let (sw, sh) = Self::get_screen_dimensions_safe();
        self.screen_to_world_with_viewport(screen_pos, sw, sh)
    }
}
