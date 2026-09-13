use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Surface types representing different racing track terrain and hazards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Standard dry asphalt track: optimal grip and tire smoke on slip.
    #[default]
    Asphalt,
    /// Playable compacted dirt / gravel rally track: good controllable slide grip.
    Dirt,
    /// Kerb / rumble strip: high grip with slight vibration and higher rolling resistance.
    Curb,
    /// Grassy run-off area: significantly reduced grip and high rolling resistance.
    Grass,
    /// Deep sand / gravel trap: heavy rolling resistance and very low grip (obstacle/trap only).
    Sand,
    /// Water puddle / wet patch hazard: very low friction, high drag, aquaplaning/hydroplaning.
    Water,
    /// Oil slick hazard: extremely low friction, vehicle spins easily.
    Oil,
    /// Frozen icy patch: near zero friction, almost zero stopping power.
    Ice,
}


impl SurfaceType {
    /// Friction coefficient (mu) scaling available tire traction.
    #[inline]
    pub const fn friction_coefficient(self) -> f32 {
        match self {
            Self::Asphalt => 1.0,
            Self::Curb => 0.88,
            Self::Dirt => 0.78,
            Self::Grass => 0.45,
            Self::Sand => 0.30,
            Self::Water => 0.22,
            Self::Oil => 0.12,
            Self::Ice => 0.08,
        }
    }

    /// Rolling resistance multiplier relative to standard asphalt.
    #[inline]
    pub const fn rolling_resistance_multiplier(self) -> f32 {
        match self {
            Self::Asphalt => 1.0,
            Self::Curb => 1.3,
            Self::Dirt => 1.2,
            Self::Grass => 18.0,
            Self::Sand => 45.0,
            Self::Water => 3.5,
            Self::Oil => 0.8,
            Self::Ice => 0.4,
        }
    }

    /// Additional surface deceleration drag (aerodynamic/viscous drag multiplier).
    #[inline]
    pub const fn surface_drag_multiplier(self) -> f32 {
        match self {
            Self::Asphalt => 1.0,
            Self::Curb => 1.05,
            Self::Dirt => 1.10,
            Self::Grass => 2.2,
            Self::Sand => 4.5,
            Self::Water => 2.0,
            Self::Oil => 0.95,
            Self::Ice => 0.90,
        }
    }

    /// Whether this surface produces standard rubber skid marks and tire smoke.
    #[inline]
    pub const fn produces_tire_smoke(self) -> bool {
        matches!(self, Self::Asphalt | Self::Curb)
    }

    /// Whether this surface kicks up dust/grass/gravel particles.
    #[inline]
    pub const fn produces_debris_particles(self) -> bool {
        matches!(self, Self::Grass | Self::Sand | Self::Dirt)
    }

    /// Whether this surface produces water splash and spray plumes.
    #[inline]
    pub const fn produces_water_splash(self) -> bool {
        matches!(self, Self::Water)
    }

    /// Whether this surface acts as an on-track hazard overlay (e.g. water puddle, oil slick, ice patch)
    /// that sits on top of the road ribbon and overrides the underlying surface.
    #[inline]
    pub const fn is_on_track_hazard(self) -> bool {
        matches!(self, Self::Water | Self::Oil | Self::Ice)
    }

    /// All valid global off-track terrain types that can be selected as a track's default surface.
    pub const OFF_TRACK_TYPES: [SurfaceType; 4] = [
        SurfaceType::Grass,
        SurfaceType::Sand,
        SurfaceType::Dirt,
        SurfaceType::Asphalt,
    ];

    /// Whether this surface can serve as a global off-track default terrain.
    #[inline]
    pub const fn is_valid_off_track(self) -> bool {
        matches!(self, Self::Grass | Self::Sand | Self::Dirt | Self::Asphalt)
    }

    /// Display name of the surface type.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Asphalt => "Asphalt",
            Self::Dirt => "Dirt",
            Self::Curb => "Curb",
            Self::Grass => "Grass",
            Self::Sand => "Sand",
            Self::Water => "Water",
            Self::Oil => "Oil",
            Self::Ice => "Ice",
        }
    }
}

/// Comprehensive physical properties of a ground contact patch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceProperties {
    /// Categorical classification for particle and audio FX.
    pub surface_type: SurfaceType,
    /// Friction coefficient multiplier (1.0 = baseline dry asphalt).
    pub friction: f32,
    /// Rolling resistance / drag multiplier.
    pub rolling_resistance: f32,
    /// Additional viscous/aerodynamic surface deceleration drag.
    pub drag_multiplier: f32,
    /// Ground elevation in meters (z >= 0.0) beneath the contact patch.
    pub elevation: f32,
    /// Cross-slope road banking angle in degrees (+ = right side elevated).
    pub bank_angle: f32,
    /// Track transverse right vector in world space for resolving banking incline gravity.
    pub track_right: Vec2,
}

impl Default for SurfaceProperties {
    fn default() -> Self {
        Self::from_type(SurfaceType::Asphalt)
    }
}

impl SurfaceProperties {
    /// Creates default flat surface properties for a given categorical surface type.
    pub const fn from_type(surface_type: SurfaceType) -> Self {
        Self {
            surface_type,
            friction: surface_type.friction_coefficient(),
            rolling_resistance: surface_type.rolling_resistance_multiplier(),
            drag_multiplier: surface_type.surface_drag_multiplier(),
            elevation: 0.0,
            bank_angle: 0.0,
            track_right: Vec2::ZERO,
        }
    }
}

impl From<SurfaceType> for SurfaceProperties {
    #[inline]
    fn from(surface_type: SurfaceType) -> Self {
        Self::from_type(surface_type)
    }
}

/// Abstract interface for sampling terrain properties beneath vehicle contact patches.
pub trait SurfaceSampler {
    /// Queries the surface properties at a specific 2D world coordinate.
    fn sample_surface(&self, world_pos: Vec2) -> SurfaceProperties;
}

/// A uniform surface sampler that returns identical properties everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UniformSurface(pub SurfaceType);

impl SurfaceSampler for UniformSurface {
    #[inline]
    fn sample_surface(&self, _world_pos: Vec2) -> SurfaceProperties {
        SurfaceProperties::from_type(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_properties() {
        assert!(SurfaceType::Asphalt.friction_coefficient() > SurfaceType::Dirt.friction_coefficient());
        assert!(SurfaceType::Dirt.friction_coefficient() > SurfaceType::Grass.friction_coefficient());
        assert!(SurfaceType::Grass.friction_coefficient() > SurfaceType::Water.friction_coefficient());
        assert!(SurfaceType::Water.friction_coefficient() > SurfaceType::Ice.friction_coefficient());
        assert!(SurfaceType::Sand.rolling_resistance_multiplier() > SurfaceType::Dirt.rolling_resistance_multiplier());
        assert!(SurfaceType::Asphalt.produces_tire_smoke());
        assert!(!SurfaceType::Ice.produces_tire_smoke());
        assert!(SurfaceType::Grass.produces_debris_particles());
        assert!(SurfaceType::Dirt.produces_debris_particles());
        assert!(SurfaceType::Water.produces_water_splash());
        assert!(!SurfaceType::Asphalt.produces_water_splash());
    }

    #[test]
    fn test_uniform_surface_sampler() {
        let sampler = UniformSurface(SurfaceType::Dirt);
        let props = sampler.sample_surface(Vec2::new(100.0, -50.0));
        assert_eq!(props.surface_type, SurfaceType::Dirt);
        assert_eq!(props.friction, SurfaceType::Dirt.friction_coefficient());
    }
}
