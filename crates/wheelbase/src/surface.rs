use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Surface types representing different racing track terrain and hazards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Standard dry asphalt track: optimal grip and tire smoke on slip.
    #[default]
    Asphalt,
    /// Solid cast or poured concrete: high grip, smooth pavement, and grandstand/stadium aprons.
    Concrete,
    /// Kerb / rumble strip: high grip with slight vibration and higher rolling resistance.
    Curb,
    /// Playable compacted dirt / gravel rally track: good controllable slide grip.
    Dirt,
    /// Loose stone gravel track / rally runoff: moderate grip, high stone debris roost.
    Gravel,
    /// Grassy run-off area: significantly reduced grip and high rolling resistance.
    Grass,
    /// Compacted desert sand/dune ribbon: drivable racing line for desert circuits.
    PackedSand,
    /// Deep loose sand arrestor bed / runaway gravel trap: heavy rolling resistance and low grip.
    DeepSand,
    /// Compacted dirt/mud rallycross track ribbon: good controllable slide grip on wet courses.
    MudTrack,
    /// Deep viscous mud bog / swamp hazard: high rolling resistance, low lateral slide friction, heavy spray plumes.
    DeepMud,
    /// Compacted snow rally ribbon: moderate rolling resistance, low traction, powder roost trails.
    PackedSnow,
    /// Deep snowbank barrier / off-track snow: high rolling resistance, low friction, gentle deceleration.
    DeepSnow,
    /// Glacial mirror sheet ice: near-zero friction, requires studded tires for directional control.
    SheetIce,
    /// Water puddle / wet patch hazard: very low friction, high drag, aquaplaning/hydroplaning.
    Water,
    /// Oil slick hazard: extremely low friction, vehicle spins easily.
    Oil,
}

impl SurfaceType {
    /// Friction coefficient (mu) scaling available tire traction.
    #[inline]
    pub const fn friction_coefficient(self) -> f32 {
        match self {
            Self::Asphalt => 1.0,
            Self::Concrete => 0.95,
            Self::Curb => 0.88,
            Self::Dirt => 0.78,
            Self::Gravel => 0.70,
            Self::PackedSand => 0.62,
            Self::MudTrack => 0.58,
            Self::PackedSnow => 0.48,
            Self::Grass => 0.45,
            Self::DeepMud => 0.40,
            Self::DeepSand => 0.30,
            Self::DeepSnow => 0.28,
            Self::Water => 0.22,
            Self::Oil => 0.12,
            Self::SheetIce => 0.08,
        }
    }

    /// Rolling resistance multiplier relative to standard asphalt.
    #[inline]
    pub const fn rolling_resistance_multiplier(self) -> f32 {
        match self {
            Self::SheetIce => 0.4,
            Self::Oil => 0.8,
            Self::Asphalt => 1.0,
            Self::Concrete => 1.05,
            Self::Dirt => 1.2,
            Self::Curb => 1.3,
            Self::PackedSnow => 2.2,
            Self::Gravel => 2.5,
            Self::Grass => 2.5,
            Self::Water => 3.5,
            Self::MudTrack => 5.0,
            Self::PackedSand => 5.2,
            Self::DeepSnow => 12.0,
            Self::DeepMud => 14.0,
            Self::DeepSand => 30.0,
        }
    }

    /// Additional surface deceleration drag (aerodynamic/viscous drag multiplier).
    #[inline]
    pub const fn surface_drag_multiplier(self) -> f32 {
        match self {
            Self::SheetIce => 0.90,
            Self::Oil => 0.95,
            Self::Asphalt => 1.0,
            Self::Concrete => 1.00,
            Self::Curb => 1.05,
            Self::Dirt => 1.10,
            Self::Gravel => 1.25,
            Self::PackedSnow => 1.40,
            Self::Water => 2.0,
            Self::PackedSand => 2.10,
            Self::Grass => 2.2,
            Self::MudTrack => 2.50,
            Self::DeepSnow => 3.80,
            Self::DeepSand => 4.5,
            Self::DeepMud => 5.00,
        }
    }

    /// Whether this surface produces standard rubber skid marks and tire smoke.
    #[inline]
    pub const fn produces_tire_smoke(self) -> bool {
        matches!(self, Self::Asphalt | Self::Concrete | Self::Curb)
    }

    /// Whether this surface kicks up dust/grass/gravel particles.
    #[inline]
    pub const fn produces_debris_particles(self) -> bool {
        matches!(
            self,
            Self::Grass
                | Self::PackedSand
                | Self::DeepSand
                | Self::Dirt
                | Self::MudTrack
                | Self::DeepMud
                | Self::PackedSnow
                | Self::DeepSnow
                | Self::Gravel
        )
    }

    /// Whether this surface is a loose or deformable terrain where tires physically
    /// displace and indent material rather than depositing vulcanized rubber.
    #[inline]
    pub const fn is_loose_deformable(self) -> bool {
        matches!(
            self,
            Self::Gravel
                | Self::PackedSand
                | Self::DeepSand
                | Self::Dirt
                | Self::MudTrack
                | Self::DeepMud
                | Self::PackedSnow
                | Self::DeepSnow
        )
    }

    /// Whether this surface is a rigid, non-deformable pavement where marks
    /// result purely from friction rubber transfer.
    #[inline]
    pub const fn is_rigid_pavement(self) -> bool {
        matches!(self, Self::Asphalt | Self::Concrete | Self::Curb)
    }

    /// Whether rolling tires leave visible depression ruts without requiring wheel slip.
    #[inline]
    pub const fn leaves_rolling_rut(self) -> bool {
        matches!(
            self,
            Self::Gravel
                | Self::PackedSand
                | Self::DeepSand
                | Self::Dirt
                | Self::MudTrack
                | Self::DeepMud
                | Self::PackedSnow
                | Self::DeepSnow
                | Self::Grass
        )
    }

    /// Whether this surface produces water splash and spray plumes.
    #[inline]
    pub const fn produces_water_splash(self) -> bool {
        matches!(self, Self::Water)
    }

    /// Whether this surface acts as an on-track hazard overlay (e.g. water puddle, oil slick, ice patch, mud bog)
    /// that sits on top of the road ribbon and overrides the underlying surface.
    #[inline]
    pub const fn is_on_track_hazard(self) -> bool {
        matches!(self, Self::Water | Self::Oil | Self::SheetIce)
    }

    /// Returns true if this surface is any form of sand (packed ribbon or deep trap).
    #[inline]
    pub const fn is_sand(self) -> bool {
        matches!(self, Self::PackedSand | Self::DeepSand)
    }

    /// Returns true if this surface is any form of mud (track ribbon or deep bog).
    #[inline]
    pub const fn is_mud(self) -> bool {
        matches!(self, Self::MudTrack | Self::DeepMud)
    }

    /// Returns true if this surface is any form of snow (packed ribbon or deep snowbank).
    #[inline]
    pub const fn is_snow(self) -> bool {
        matches!(self, Self::PackedSnow | Self::DeepSnow)
    }

    /// Returns true if this surface is sheet ice.
    #[inline]
    pub const fn is_ice(self) -> bool {
        matches!(self, Self::SheetIce)
    }

    /// All 15 supported surface types.
    pub const ALL: [SurfaceType; 15] = [
        SurfaceType::Asphalt,
        SurfaceType::Concrete,
        SurfaceType::Curb,
        SurfaceType::Dirt,
        SurfaceType::Gravel,
        SurfaceType::Grass,
        SurfaceType::PackedSand,
        SurfaceType::DeepSand,
        SurfaceType::MudTrack,
        SurfaceType::DeepMud,
        SurfaceType::PackedSnow,
        SurfaceType::DeepSnow,
        SurfaceType::SheetIce,
        SurfaceType::Water,
        SurfaceType::Oil,
    ];

    /// All valid global off-track terrain types that can be selected as a track's default surface.
    pub const OFF_TRACK_TYPES: [SurfaceType; 12] = [
        SurfaceType::Grass,
        SurfaceType::DeepSand,
        SurfaceType::PackedSand,
        SurfaceType::Dirt,
        SurfaceType::Asphalt,
        SurfaceType::Concrete,
        SurfaceType::DeepMud,
        SurfaceType::MudTrack,
        SurfaceType::DeepSnow,
        SurfaceType::PackedSnow,
        SurfaceType::Gravel,
        SurfaceType::SheetIce,
    ];

    /// Whether this surface can serve as a global off-track default terrain.
    #[inline]
    pub const fn is_valid_off_track(self) -> bool {
        matches!(
            self,
            Self::Grass
                | Self::DeepSand
                | Self::PackedSand
                | Self::Dirt
                | Self::Asphalt
                | Self::Concrete
                | Self::DeepMud
                | Self::MudTrack
                | Self::DeepSnow
                | Self::PackedSnow
                | Self::Gravel
                | Self::SheetIce
        )
    }

    /// Display name of the surface type.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Asphalt => "Asphalt",
            Self::Concrete => "Concrete",
            Self::Dirt => "Dirt",
            Self::Curb => "Curb",
            Self::Grass => "Grass",
            Self::PackedSand => "Packed Sand",
            Self::DeepSand => "Deep Sand",
            Self::MudTrack => "Mud Track",
            Self::DeepMud => "Deep Mud",
            Self::PackedSnow => "Packed Snow",
            Self::DeepSnow => "Deep Snow",
            Self::SheetIce => "Sheet Ice",
            Self::Water => "Water",
            Self::Oil => "Oil",
            Self::Gravel => "Gravel",
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
    /// Longitudinal road grade slope in radians (+ = uphill, - = downhill).
    pub grade_slope: f32,
    /// Vertical road curvature d(slope)/ds in rad/m (+ = dip/compression, - = crest/unloading).
    pub vertical_curvature: f32,
    /// Track longitudinal forward vector in world space for resolving grade incline gravity.
    pub track_forward: Vec2,
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
            grade_slope: 0.0,
            vertical_curvature: 0.0,
            track_forward: Vec2::ZERO,
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

/// Standardized motorsport tire compound identifiers (Spec 074).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CompoundId {
    /// Ultra-high grip asphalt slick with rapid thermal degradation.
    SoftSlick,
    /// Balanced dry asphalt competition slick.
    #[default]
    MediumSlick,
    /// Durable endurance asphalt slick with high heat resistance.
    HardSlick,
    /// Grooved transitional tire for damp tracks and standing drizzle.
    IntermediateWet,
    /// Deep-tread directional rain tire with maximum hydroplaning evacuation.
    MonsoonWet,
    /// Dual-purpose multi-surface tire for gravel, dirt, and light tarmac.
    AllTerrain,
    /// Heavy open-lug mud and sand tire with high loose-soil bite.
    ExtremeMud,
    /// Steel-studded winter competition tire for hard-packed snow and sheet ice.
    StuddedIce,
}

impl CompoundId {
    pub const ALL: [Self; 8] = [
        Self::SoftSlick,
        Self::MediumSlick,
        Self::HardSlick,
        Self::IntermediateWet,
        Self::MonsoonWet,
        Self::AllTerrain,
        Self::ExtremeMud,
        Self::StuddedIce,
    ];

    /// Human-readable display label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::SoftSlick => "Soft Slick",
            Self::MediumSlick => "Medium Slick",
            Self::HardSlick => "Hard Slick",
            Self::IntermediateWet => "Intermediate Wet",
            Self::MonsoonWet => "Monsoon Wet",
            Self::AllTerrain => "All-Terrain",
            Self::ExtremeMud => "Extreme Mud",
            Self::StuddedIce => "Studded Ice",
        }
    }

    /// Single or two-letter compact acronym for HUD badges.
    pub const fn badge_code(self) -> &'static str {
        match self {
            Self::SoftSlick => "S",
            Self::MediumSlick => "M",
            Self::HardSlick => "H",
            Self::IntermediateWet => "INT",
            Self::MonsoonWet => "W",
            Self::AllTerrain => "AT",
            Self::ExtremeMud => "MUD",
            Self::StuddedIce => "ICE",
        }
    }

    /// Normalized RGBA color [r, g, b, a] for compound presentation accents.
    pub const fn accent_rgba(self) -> [f32; 4] {
        match self {
            Self::SoftSlick => [0.95, 0.15, 0.15, 1.0],       // Red [S]
            Self::MediumSlick => [0.95, 0.85, 0.10, 1.0],     // Yellow [M]
            Self::HardSlick => [0.90, 0.90, 0.90, 1.0],       // White [H]
            Self::IntermediateWet => [0.15, 0.80, 0.20, 1.0], // Green [INT]
            Self::MonsoonWet => [0.10, 0.50, 0.95, 1.0],      // Blue [WET]
            Self::AllTerrain => [0.95, 0.55, 0.10, 1.0],      // Orange [AT]
            Self::ExtremeMud => [0.55, 0.35, 0.15, 1.0],      // Brown [MUD]
            Self::StuddedIce => [0.60, 0.90, 1.00, 1.0],      // Cyan [ICE]
        }
    }
}

/// Compact 15-element array mapping each SurfaceType to its compound friction multiplier (Spec 074).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceAffinityMap {
    affinities: [f32; 15],
}

impl Default for SurfaceAffinityMap {
    fn default() -> Self {
        Self::for_compound(CompoundId::MediumSlick)
    }
}

impl SurfaceAffinityMap {
    #[inline]
    pub const fn new(affinities: [f32; 15]) -> Self {
        Self { affinities }
    }

    #[inline]
    pub fn get(&self, surface: SurfaceType) -> f32 {
        self.affinities[surface as usize]
    }

    #[inline]
    pub fn set(&mut self, surface: SurfaceType, val: f32) {
        self.affinities[surface as usize] = val;
    }

    /// Returns calibrated affinity multipliers for a standard compound across all 15 SurfaceTypes.
    pub const fn for_compound(id: CompoundId) -> Self {
        match id {
            CompoundId::SoftSlick => Self::new([
                1.20, // Asphalt
                1.18, // Concrete
                1.10, // Curb
                0.45, // Dirt
                0.35, // Gravel
                0.40, // Grass
                0.30, // PackedSand
                0.15, // DeepSand
                0.25, // MudTrack
                0.10, // DeepMud
                0.15, // PackedSnow
                0.10, // DeepSnow
                0.05, // SheetIce
                0.20, // Water
                0.10, // Oil
            ]),
            CompoundId::MediumSlick => Self::new([
                1.00, // Asphalt
                0.98, // Concrete
                0.95, // Curb
                0.85, // Dirt
                0.32, // Gravel
                0.40, // Grass
                0.28, // PackedSand
                0.15, // DeepSand
                0.22, // MudTrack
                0.10, // DeepMud
                0.13, // PackedSnow
                0.09, // DeepSnow
                0.05, // SheetIce
                0.22, // Water
                0.10, // Oil
            ]),
            CompoundId::HardSlick => Self::new([
                1.00, // Asphalt
                0.98, // Concrete
                0.95, // Curb
                0.40, // Dirt
                0.30, // Gravel
                0.40, // Grass
                0.25, // PackedSand
                0.15, // DeepSand
                0.20, // MudTrack
                0.10, // DeepMud
                0.12, // PackedSnow
                0.08, // DeepSnow
                0.05, // SheetIce
                0.25, // Water
                0.10, // Oil
            ]),
            CompoundId::IntermediateWet => Self::new([
                0.88, // Asphalt
                0.86, // Concrete
                0.85, // Curb
                0.65, // Dirt
                0.55, // Gravel
                0.50, // Grass
                0.40, // PackedSand
                0.25, // DeepSand
                0.50, // MudTrack
                0.25, // DeepMud
                0.30, // PackedSnow
                0.20, // DeepSnow
                0.15, // SheetIce
                1.10, // Water
                0.15, // Oil
            ]),
            CompoundId::MonsoonWet => Self::new([
                0.72, // Asphalt
                0.70, // Concrete
                0.75, // Curb
                0.70, // Dirt
                0.60, // Gravel
                0.60, // Grass
                0.45, // PackedSand
                0.30, // DeepSand
                0.65, // MudTrack
                0.40, // DeepMud
                0.40, // PackedSnow
                0.30, // DeepSnow
                0.20, // SheetIce
                1.35, // Water
                0.20, // Oil
            ]),
            CompoundId::AllTerrain => Self::new([
                0.85, // Asphalt
                0.83, // Concrete
                0.88, // Curb
                1.15, // Dirt
                1.20, // Gravel
                0.95, // Grass
                1.10, // PackedSand
                1.20, // DeepSand
                1.05, // MudTrack
                0.75, // DeepMud
                0.80, // PackedSnow
                1.30, // DeepSnow
                0.90, // SheetIce
                0.90, // Water
                0.25, // Oil
            ]),
            CompoundId::ExtremeMud => Self::new([
                0.65, // Asphalt
                0.62, // Concrete
                0.70, // Curb
                1.10, // Dirt
                1.05, // Gravel
                1.00, // Grass
                1.25, // PackedSand
                1.35, // DeepSand
                1.30, // MudTrack
                1.40, // DeepMud
                0.90, // PackedSnow
                1.10, // DeepSnow
                0.50, // SheetIce
                0.80, // Water
                0.25, // Oil
            ]),
            CompoundId::StuddedIce => Self::new([
                0.50, // Asphalt
                0.48, // Concrete
                0.55, // Curb
                0.80, // Dirt
                0.75, // Gravel
                0.60, // Grass
                0.50, // PackedSand
                0.40, // DeepSand
                0.60, // MudTrack
                0.40, // DeepMud
                1.35, // PackedSnow
                1.30, // DeepSnow
                1.45, // SheetIce
                0.60, // Water
                0.20, // Oil
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_properties() {
        assert!(SurfaceType::Asphalt.friction_coefficient() > SurfaceType::Dirt.friction_coefficient());
        assert!(SurfaceType::Dirt.friction_coefficient() > SurfaceType::Gravel.friction_coefficient());
        assert!(SurfaceType::Gravel.friction_coefficient() > SurfaceType::Grass.friction_coefficient());
        assert!(SurfaceType::Gravel.rolling_resistance_multiplier() > SurfaceType::Dirt.rolling_resistance_multiplier());
        assert!(SurfaceType::Gravel.produces_debris_particles());
        assert!(SurfaceType::Grass.friction_coefficient() > SurfaceType::Water.friction_coefficient());
        assert!(SurfaceType::Water.friction_coefficient() > SurfaceType::SheetIce.friction_coefficient());
        assert!(SurfaceType::PackedSand.friction_coefficient() > SurfaceType::DeepSand.friction_coefficient());
        assert!(SurfaceType::MudTrack.friction_coefficient() > SurfaceType::DeepMud.friction_coefficient());
        assert!(SurfaceType::PackedSnow.friction_coefficient() > SurfaceType::DeepSnow.friction_coefficient());
        assert!(SurfaceType::DeepSand.rolling_resistance_multiplier() > SurfaceType::PackedSand.rolling_resistance_multiplier());
        assert!(SurfaceType::DeepMud.rolling_resistance_multiplier() > SurfaceType::MudTrack.rolling_resistance_multiplier());
        assert!(SurfaceType::DeepSnow.rolling_resistance_multiplier() > SurfaceType::PackedSnow.rolling_resistance_multiplier());
        assert!(SurfaceType::PackedSand.rolling_resistance_multiplier() > SurfaceType::Dirt.rolling_resistance_multiplier());
        assert!(SurfaceType::Asphalt.produces_tire_smoke());
        assert!(SurfaceType::Concrete.produces_tire_smoke());
        assert!(SurfaceType::Asphalt.friction_coefficient() >= SurfaceType::Concrete.friction_coefficient());
        assert!(SurfaceType::Concrete.friction_coefficient() > SurfaceType::Curb.friction_coefficient());
        assert!(!SurfaceType::SheetIce.produces_tire_smoke());
        assert!(SurfaceType::Grass.produces_debris_particles());
        assert!(SurfaceType::Dirt.produces_debris_particles());
        assert!(SurfaceType::Water.produces_water_splash());
        assert!(!SurfaceType::Asphalt.produces_water_splash());
        assert!(!SurfaceType::Concrete.produces_water_splash());
    }

    #[test]
    fn test_uniform_surface_sampler() {
        let sampler = UniformSurface(SurfaceType::Dirt);
        let props = sampler.sample_surface(Vec2::new(100.0, -50.0));
        assert_eq!(props.surface_type, SurfaceType::Dirt);
        assert_eq!(props.friction, SurfaceType::Dirt.friction_coefficient());
    }

    #[test]
    fn test_surface_taxonomy_and_properties() {
        assert!(SurfaceType::Gravel.is_loose_deformable());
        assert!(SurfaceType::PackedSand.is_loose_deformable());
        assert!(SurfaceType::DeepSand.is_loose_deformable());
        assert!(SurfaceType::Dirt.is_loose_deformable());
        assert!(SurfaceType::MudTrack.is_loose_deformable());
        assert!(SurfaceType::DeepMud.is_loose_deformable());
        assert!(SurfaceType::PackedSnow.is_loose_deformable());
        assert!(SurfaceType::DeepSnow.is_loose_deformable());
        assert!(!SurfaceType::Asphalt.is_loose_deformable());
        assert!(!SurfaceType::Concrete.is_loose_deformable());

        assert!(SurfaceType::Asphalt.is_rigid_pavement());
        assert!(SurfaceType::Concrete.is_rigid_pavement());
        assert!(SurfaceType::Curb.is_rigid_pavement());
        assert!(!SurfaceType::Gravel.is_rigid_pavement());
        assert!(!SurfaceType::Grass.is_rigid_pavement());

        assert!(SurfaceType::Gravel.leaves_rolling_rut());
        assert!(SurfaceType::PackedSand.leaves_rolling_rut());
        assert!(SurfaceType::DeepSand.leaves_rolling_rut());
        assert!(SurfaceType::Dirt.leaves_rolling_rut());
        assert!(SurfaceType::MudTrack.leaves_rolling_rut());
        assert!(SurfaceType::DeepMud.leaves_rolling_rut());
        assert!(SurfaceType::PackedSnow.leaves_rolling_rut());
        assert!(SurfaceType::DeepSnow.leaves_rolling_rut());
        assert!(SurfaceType::Grass.leaves_rolling_rut());
        assert!(!SurfaceType::Asphalt.leaves_rolling_rut());
        assert!(!SurfaceType::Concrete.leaves_rolling_rut());

        assert!(SurfaceType::PackedSand.is_sand());
        assert!(SurfaceType::DeepSand.is_sand());
        assert!(!SurfaceType::Dirt.is_sand());

        assert!(SurfaceType::MudTrack.is_mud());
        assert!(SurfaceType::DeepMud.is_mud());
        assert!(!SurfaceType::Water.is_mud());

        assert!(SurfaceType::PackedSnow.is_snow());
        assert!(SurfaceType::DeepSnow.is_snow());
        assert!(!SurfaceType::SheetIce.is_snow());

        assert!(SurfaceType::SheetIce.is_ice());
    }

    #[test]
    fn test_compound_surface_affinities() {
        let soft = SurfaceAffinityMap::for_compound(CompoundId::SoftSlick);
        let hard = SurfaceAffinityMap::for_compound(CompoundId::HardSlick);
        let inter = SurfaceAffinityMap::for_compound(CompoundId::IntermediateWet);
        let monsoon = SurfaceAffinityMap::for_compound(CompoundId::MonsoonWet);
        let at = SurfaceAffinityMap::for_compound(CompoundId::AllTerrain);
        let mud = SurfaceAffinityMap::for_compound(CompoundId::ExtremeMud);
        let ice = SurfaceAffinityMap::for_compound(CompoundId::StuddedIce);

        // Asphalt: SoftSlick > HardSlick > AllTerrain > MonsoonWet > ExtremeMud > StuddedIce
        assert!(soft.get(SurfaceType::Asphalt) > hard.get(SurfaceType::Asphalt));
        assert!(hard.get(SurfaceType::Asphalt) > at.get(SurfaceType::Asphalt));
        assert!(at.get(SurfaceType::Asphalt) > monsoon.get(SurfaceType::Asphalt));

        // Water: MonsoonWet (1.35) > IntermediateWet (1.10) > HardSlick (0.25)
        assert_eq!(monsoon.get(SurfaceType::Water), 1.35);
        assert_eq!(inter.get(SurfaceType::Water), 1.10);
        assert_eq!(hard.get(SurfaceType::Water), 0.25);

        // DeepMud: ExtremeMud (1.40) > AllTerrain (0.75) > SoftSlick (0.10)
        assert_eq!(mud.get(SurfaceType::DeepMud), 1.40);
        assert_eq!(at.get(SurfaceType::DeepMud), 0.75);
        assert_eq!(soft.get(SurfaceType::DeepMud), 0.10);

        // SheetIce: StuddedIce (1.45) > AllTerrain (0.90) > SoftSlick (0.05)
        assert_eq!(ice.get(SurfaceType::SheetIce), 1.45);
        assert!(ice.get(SurfaceType::SheetIce) > at.get(SurfaceType::SheetIce));
        assert!(at.get(SurfaceType::SheetIce) > soft.get(SurfaceType::SheetIce));
    }
}
