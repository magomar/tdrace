//! Camera zoom levels and follow settings. Moved from `tdrace-app/src/config.rs` (spec 058).

use serde::{Deserialize, Serialize};

/// Baseline reference display resolution for camera zoom calibration (1280x720 HD).
pub const REFERENCE_SCREEN_WIDTH: f32 = 1280.0;
/// Baseline reference display height for camera zoom calibration.
pub const REFERENCE_SCREEN_HEIGHT: f32 = 720.0;

/// Configuration for a specific camera zoom level or mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoomLevelConfig {
    /// Friendly label displayed in HUD popups (e.g. "Close", "Medium", "Far", "Overview").
    pub name: String,
    /// Mode type: "follow" (dynamic speed follow) or "overview" (static full-circuit view).
    pub mode: String,
    /// Pixels per meter at high speed (for follow mode) or base scale (for overview).
    pub min_zoom: f32,
    /// Pixels per meter at zero speed / stationary (for follow mode).
    pub max_zoom: f32,
}

impl Default for ZoomLevelConfig {
    fn default() -> Self {
        Self {
            name: "Medium".to_string(),
            mode: "follow".to_string(),
            min_zoom: 10.0,
            max_zoom: 16.5,
        }
    }
}

impl ZoomLevelConfig {
    pub fn is_overview(&self) -> bool {
        self.mode.eq_ignore_ascii_case("overview")
    }

    /// Computes the resolution scale factor relative to the reference screen height (720p).
    ///
    /// Predefined zoom levels were calibrated for 1280x720. If screen resolution has
    /// a height twice that (e.g. 1440p), the zoom scale factor increases by 2.0.
    #[inline]
    pub fn resolution_scale(screen_h: f32) -> f32 {
        (screen_h / REFERENCE_SCREEN_HEIGHT).max(0.1)
    }

    /// Returns a new `ZoomLevelConfig` with min_zoom and max_zoom multiplied by `factor`.
    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            name: self.name.clone(),
            mode: self.mode.clone(),
            min_zoom: self.min_zoom * factor,
            max_zoom: self.max_zoom * factor,
        }
    }

    /// Returns a new `ZoomLevelConfig` scaled relative to the reference screen height (720p).
    pub fn scaled_for_screen(&self, screen_h: f32) -> Self {
        self.scaled(Self::resolution_scale(screen_h))
    }
}

/// Global camera configuration and list of selectable zoom levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Smooth follow position interpolation speed.
    pub position_smoothing: f32,
    /// Smooth follow zoom interpolation speed.
    pub zoom_smoothing: f32,
    /// Velocity lookahead projection time in seconds.
    pub velocity_lookahead_time: f32,
    /// Screen shake trauma decay rate per second.
    pub trauma_decay: f32,
    /// Maximum screen shake pixel displacement.
    pub max_shake_offset: f32,
    /// Initial active zoom level index in the `levels` list.
    pub default_level_index: usize,
    /// Ordered list of zoom levels cycled through during gameplay.
    pub levels: Vec<ZoomLevelConfig>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            position_smoothing: 8.5,
            zoom_smoothing: 4.0,
            velocity_lookahead_time: 0.40,
            trauma_decay: 2.2,
            max_shake_offset: 1.5,
            default_level_index: 0,
            levels: vec![
                ZoomLevelConfig {
                    name: "Close".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 13.5,
                    max_zoom: 22.0,
                },
                ZoomLevelConfig {
                    name: "Medium".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 10.0,
                    max_zoom: 16.5,
                },
                ZoomLevelConfig {
                    name: "Far".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 7.0,
                    max_zoom: 11.5,
                },
                ZoomLevelConfig {
                    name: "Very Far".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 5.0,
                    max_zoom: 8.0,
                },
            ],
        }
    }
}
