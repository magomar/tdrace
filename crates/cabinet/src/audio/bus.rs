use serde::{Deserialize, Serialize};

/// 4-channel volume bus settings with persistence.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ui_volume: f32,
    pub is_muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.85,
            music_volume: 0.70,
            sfx_volume: 0.90,
            ui_volume: 0.90,
            is_muted: false,
        }
    }
}

impl AudioSettings {
    /// Toggles global audio mute.
    #[inline]
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    /// Computes effective output volume for background music.
    #[inline]
    pub fn effective_music_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            (self.master_volume * self.music_volume).clamp(0.0, 1.0)
        }
    }

    /// Computes effective output volume for in-game sound effects without extra scaling.
    #[inline]
    pub fn effective_sfx_volume(&self) -> f32 {
        self.effective_sfx_volume_scaled(1.0)
    }

    /// Computes effective output volume for in-game sound effects scaled by additional gain.
    #[inline]
    pub fn effective_sfx_volume_scaled(&self, sfx_gain: f32) -> f32 {
        if self.is_muted {
            0.0
        } else {
            (self.master_volume * self.sfx_volume * sfx_gain).clamp(0.0, 1.0)
        }
    }

    /// Computes effective output volume for UI sounds.
    #[inline]
    pub fn effective_ui_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            (self.master_volume * self.ui_volume).clamp(0.0, 1.0)
        }
    }
}
