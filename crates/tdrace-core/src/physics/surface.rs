use serde::{Deserialize, Serialize};

/// Surface types representing different racing track terrain and hazards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Standard dry asphalt track: optimal grip and tire smoke on slip.
    Asphalt,
    /// Kerb / rumble strip: high grip with slight vibration and higher rolling resistance.
    Curb,
    /// Grassy run-off area: significantly reduced grip and high rolling resistance.
    Grass,
    /// Deep sand / gravel trap: heavy rolling resistance and very low grip.
    Sand,
    /// Oil slick hazard: extremely low friction, vehicle spins easily.
    Oil,
    /// Frozen icy patch: near zero friction, almost zero stopping power.
    Ice,
}

impl Default for SurfaceType {
    fn default() -> Self {
        Self::Asphalt
    }
}

impl SurfaceType {
    /// Friction coefficient (mu) scaling available tire traction.
    #[inline]
    pub const fn friction_coefficient(self) -> f32 {
        match self {
            Self::Asphalt => 1.0,
            Self::Curb => 0.88,
            Self::Grass => 0.45,
            Self::Sand => 0.30,
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
            Self::Grass => 18.0,
            Self::Sand => 45.0,
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
            Self::Grass => 2.2,
            Self::Sand => 4.5,
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
        matches!(self, Self::Grass | Self::Sand)
    }

    /// Display name of the surface type.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Asphalt => "Asphalt",
            Self::Curb => "Curb",
            Self::Grass => "Grass",
            Self::Sand => "Sand",
            Self::Oil => "Oil",
            Self::Ice => "Ice",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_properties() {
        assert!(SurfaceType::Asphalt.friction_coefficient() > SurfaceType::Grass.friction_coefficient());
        assert!(SurfaceType::Grass.friction_coefficient() > SurfaceType::Ice.friction_coefficient());
        assert!(SurfaceType::Sand.rolling_resistance_multiplier() > SurfaceType::Asphalt.rolling_resistance_multiplier());
        assert!(SurfaceType::Asphalt.produces_tire_smoke());
        assert!(!SurfaceType::Ice.produces_tire_smoke());
        assert!(SurfaceType::Grass.produces_debris_particles());
    }
}
