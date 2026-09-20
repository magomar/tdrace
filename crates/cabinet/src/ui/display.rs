use serde::{Deserialize, Serialize};

/// Screen resolution representation with width and height in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayResolution {
    pub width: u32,
    pub height: u32,
}

impl DisplayResolution {
    pub const HD_720P: Self = Self { width: 1280, height: 720 };
    pub const HD_PLUS_900P: Self = Self { width: 1600, height: 900 };
    pub const FHD_1080P: Self = Self { width: 1920, height: 1080 };
    pub const QHD_1440P: Self = Self { width: 2560, height: 1440 };
    pub const UHD_4K: Self = Self { width: 3840, height: 2160 };
    pub const UW_FHD: Self = Self { width: 2560, height: 1080 };
    pub const UW_QHD: Self = Self { width: 3440, height: 1440 };
    pub const DECK_800P: Self = Self { width: 1280, height: 800 };

    /// Returns the array of standard supported resolutions.
    pub const fn standard_presets() -> &'static [Self] {
        &[
            Self::HD_720P,
            Self::HD_PLUS_900P,
            Self::FHD_1080P,
            Self::QHD_1440P,
            Self::UHD_4K,
            Self::UW_FHD,
            Self::UW_QHD,
            Self::DECK_800P,
        ]
    }

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// User-friendly label for display in settings menus.
    pub fn label(&self) -> String {
        match (self.width, self.height) {
            (1280, 720) => "1280 x 720 (16:9 HD)".to_string(),
            (1600, 900) => "1600 x 900 (16:9 HD+)".to_string(),
            (1920, 1080) => "1920 x 1080 (16:9 FHD)".to_string(),
            (2560, 1440) => "2560 x 1440 (16:9 2K QHD)".to_string(),
            (3840, 2160) => "3840 x 2160 (16:9 4K UHD)".to_string(),
            (2560, 1080) => "2560 x 1080 (21:9 UW-FHD)".to_string(),
            (3440, 1440) => "3440 x 1440 (21:9 UW-QHD)".to_string(),
            (1280, 800) => "1280 x 800 (16:10 Steam Deck)".to_string(),
            (w, h) => format!("{} x {} (Custom)", w, h),
        }
    }

    /// Aspect ratio as a string slice (e.g. "16:9", "21:9", "16:10", "4:3").
    pub fn aspect_ratio_str(&self) -> &'static str {
        if self.height == 0 {
            return "Unknown";
        }
        let ratio = self.width as f32 / self.height as f32;
        if (ratio - 16.0 / 9.0).abs() < 0.05 {
            "16:9"
        } else if (ratio - 21.0 / 9.0).abs() < 0.08 || (ratio - 43.0 / 18.0).abs() < 0.08 {
            "21:9"
        } else if (ratio - 16.0 / 10.0).abs() < 0.05 {
            "16:10"
        } else if (ratio - 4.0 / 3.0).abs() < 0.05 {
            "4:3"
        } else {
            "Custom"
        }
    }

    /// List of formatted preset option labels for dropdown widgets.
    pub fn preset_labels() -> Vec<String> {
        Self::standard_presets().iter().map(|res| res.label()).collect()
    }

    /// Finds the index of the closest matching standard preset for a given screen width and height.
    pub fn find_closest_preset_index(width: u32, height: u32) -> usize {
        let presets = Self::standard_presets();
        let mut best_idx = 0;
        let mut min_diff = u64::MAX;

        for (idx, preset) in presets.iter().enumerate() {
            let dw = (preset.width as i64 - width as i64).abs() as u64;
            let dh = (preset.height as i64 - height as i64).abs() as u64;
            let diff = dw * dw + dh * dh;
            if diff < min_diff {
                min_diff = diff;
                best_idx = idx;
            }
        }
        best_idx
    }

    pub const DEFAULT_PRESET_INDEX: usize = 2;
}

impl Default for DisplayResolution {
    fn default() -> Self {
        Self::FHD_1080P
    }
}

/// Window display mode: Windowed or Fullscreen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
}

impl WindowMode {
    pub fn options() -> Vec<String> {
        vec!["Windowed".to_string(), "Fullscreen".to_string()]
    }

    pub fn from_index(index: usize) -> Self {
        if index == 1 {
            Self::Fullscreen
        } else {
            Self::Windowed
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::Windowed => 0,
            Self::Fullscreen => 1,
        }
    }

    pub fn is_fullscreen(self) -> bool {
        matches!(self, Self::Fullscreen)
    }
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::Windowed
    }
}

/// Safely requests a new screen / window size in Macroquad without panicking in headless or test environments.
pub fn safe_request_screen_size(width: f32, height: f32) {
    let _ = std::panic::catch_unwind(|| {
        macroquad::window::request_new_screen_size(width, height);
    });
}

/// Safely toggles fullscreen in Macroquad without panicking in headless or test environments.
pub fn safe_set_fullscreen(fullscreen: bool) {
    let _ = std::panic::catch_unwind(|| {
        macroquad::window::set_fullscreen(fullscreen);
    });
}
