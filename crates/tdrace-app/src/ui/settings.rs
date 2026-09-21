//! Surface texture quality and environmental material settings controls.

use serde::{Deserialize, Serialize};
use crate::render::surface_material::SurfaceTextureQuality;
use crate::render::track::{get_surface_texture_quality, set_surface_texture_quality};

/// Cycles the surface texture quality to the next tier:
/// `High -> Standard -> Off -> High`.
pub fn cycle_surface_texture_quality(current: SurfaceTextureQuality) -> SurfaceTextureQuality {
    match current {
        SurfaceTextureQuality::High => SurfaceTextureQuality::Standard,
        SurfaceTextureQuality::Standard => SurfaceTextureQuality::Off,
        SurfaceTextureQuality::Off => SurfaceTextureQuality::High,
    }
}

/// Settings UI presentation and options for surface textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SurfaceTextureSettings {
    pub quality: SurfaceTextureQuality,
}

impl SurfaceTextureSettings {
    pub fn new(quality: SurfaceTextureQuality) -> Self {
        Self { quality }
    }

    /// Returns the active quality from the global registry.
    pub fn current() -> Self {
        Self {
            quality: get_surface_texture_quality(),
        }
    }

    /// Sets the quality tier in the global registry.
    pub fn apply(&self) {
        set_surface_texture_quality(self.quality);
    }

    /// Cycles to the next quality tier and immediately applies it.
    pub fn cycle(&mut self) -> SurfaceTextureQuality {
        self.quality = cycle_surface_texture_quality(self.quality);
        self.apply();
        self.quality
    }

    /// Returns human-readable label and resolution description.
    pub fn description(&self) -> &'static str {
        match self.quality {
            SurfaceTextureQuality::Off => "Flat vector shading without bitmap textures. Minimal GPU overhead.",
            SurfaceTextureQuality::Standard => "256x256 micro-textures with linear filtering. No macro-modulation.",
            SurfaceTextureQuality::High => "512x512 micro-textures with macro-modulation and organic edge fringe.",
        }
    }
}
