use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;
use super::geometry::{BarrierType, Obstacle, SurfaceLayer, SurfaceShape, SurfaceZone, WallBarrier};

fn default_tree_scale() -> f32 {
    1.0
}

fn default_grandstand_tiers() -> u32 {
    6
}

/// Botanical classification of decorative trees optimized for cenital (top-down) recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeType {
    /// Coniferous evergreen: tiered radial star needle clusters with central needle apex.
    #[default]
    Pine,
    /// Tropical palm: radial starburst of arching feather fronds radiating from a central core with trunk shadow.
    Palm,
    /// Deciduous oak / broadleaf: billowing multi-lobed organic cloud canopy with rich dappled greens.
    Oak,
    /// Italian cypress: compact, dense, narrow flame/oval crown with dark-green gradient.
    Cypress,
    /// Flowering cherry blossom (Sakura): soft vibrant floral cloud canopy of pastel pink and magenta petals.
    Sakura,
    /// Seasonal autumn maple: warm fiery crown of golden-amber, orange, and crimson foliage lobes.
    AutumnMaple,
    /// Low-profile dense shrub / bush: canopy only with no solid trunk obstacle.
    Bush,
    /// Desert saguaro / succulent: small solid trunk core with branching arms.
    Cactus,
    /// Winter high-altitude pine: coniferous pine dusted with snow on outer needle tips.
    SnowPine,
}

impl TreeType {
    pub const ALL: [Self; 9] = [
        Self::Pine,
        Self::Palm,
        Self::Oak,
        Self::Cypress,
        Self::Sakura,
        Self::AutumnMaple,
        Self::Bush,
        Self::Cactus,
        Self::SnowPine,
    ];

    /// Display name of the tree species.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pine => "Pine",
            Self::Palm => "Palm",
            Self::Oak => "Oak",
            Self::Cypress => "Cypress",
            Self::Sakura => "Sakura",
            Self::AutumnMaple => "Autumn Maple",
            Self::Bush => "Bush",
            Self::Cactus => "Cactus",
            Self::SnowPine => "Snow Pine",
        }
    }

    /// Default solid trunk rigid-body collider radius in meters.
    pub const fn default_trunk_radius(self) -> f32 {
        match self {
            Self::Pine => 0.35,
            Self::Palm => 0.28,
            Self::Oak => 0.48,
            Self::Cypress => 0.24,
            Self::Sakura => 0.35,
            Self::AutumnMaple => 0.40,
            Self::Bush => 0.0,
            Self::Cactus => 0.22,
            Self::SnowPine => 0.35,
        }
    }

    /// Default outer foliage canopy radius in meters.
    pub const fn default_canopy_radius(self) -> f32 {
        match self {
            Self::Pine => 2.8,
            Self::Palm => 3.8,
            Self::Oak => 4.6,
            Self::Cypress => 1.6,
            Self::Sakura => 3.4,
            Self::AutumnMaple => 3.8,
            Self::Bush => 1.8,
            Self::Cactus => 1.2,
            Self::SnowPine => 2.8,
        }
    }

    /// Viscous foliage brush drag deceleration rate (m/s²) when vehicle brushes through canopy.
    pub const fn canopy_drag_deceleration(self) -> f32 {
        match self {
            Self::Palm => 1.6,        // Light airy fronds offer low brush drag
            Self::Cactus => 1.4,      // Desert succulent / light brush
            Self::Sakura => 2.0,      // Soft blossoms and delicate twigs
            Self::AutumnMaple => 2.4, // Moderate deciduous leaf drag
            Self::Bush => 2.5,        // Dense shrub canopy brush resistance
            Self::Oak => 2.8,         // Dense broadleaf canopy resistance
            Self::Pine => 3.2,        // Stiff conifer needle branches
            Self::SnowPine => 3.4,    // Heavy snow-dusted conifer branches
            Self::Cypress => 3.6,     // Dense, tight columnar foliage
        }
    }
}

/// A standalone decorative tree prop featuring a solid trunk collider and soft canopy foliage zone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub id: usize,
    pub position: Vec2,
    #[serde(default)]
    pub tree_type: TreeType,
    #[serde(default = "default_tree_scale")]
    pub scale: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub elevation: f32,
}

impl Tree {
    pub fn new(id: usize, position: Vec2, tree_type: TreeType) -> Self {
        Self {
            id,
            position,
            tree_type,
            scale: 1.0,
            rotation: 0.0,
            elevation: 0.0,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.clamp(0.2, 5.0);
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Computes the effective solid trunk radius scaled by `self.scale`.
    #[inline]
    pub fn trunk_radius(&self) -> f32 {
        self.tree_type.default_trunk_radius() * self.scale.max(0.2)
    }

    /// Computes the effective outer foliage canopy radius scaled by `self.scale`.
    #[inline]
    pub fn canopy_radius(&self) -> f32 {
        self.tree_type.default_canopy_radius() * self.scale.max(0.2)
    }

    /// Tests whether a point lies within the outer foliage canopy.
    #[inline]
    pub fn contains_canopy(&self, point: Vec2) -> bool {
        let r = self.canopy_radius();
        (self.position - point).length_squared() <= r * r
    }

    /// Whether this tree possesses a solid collidable trunk.
    #[inline]
    pub fn has_trunk(&self) -> bool {
        self.trunk_radius() > 0.05
    }

    /// Tests whether a point lies within the solid wood trunk collider.
    #[inline]
    pub fn contains_trunk(&self, point: Vec2) -> bool {
        if !self.has_trunk() {
            return false;
        }
        let r = self.trunk_radius();
        (self.position - point).length_squared() <= r * r
    }

    /// Generates a rigid circular obstacle representing the solid wood trunk.
    pub fn trunk_obstacle(&self) -> Obstacle {
        let mut obs = Obstacle::circle(
            self.id,
            self.position,
            self.trunk_radius(),
            format!("{} Trunk #{}", self.tree_type.name(), self.id),
        );
        // Wood physical parameters: lower restitution than concrete, higher bark friction
        obs.restitution = 0.25;
        obs.friction = 0.55;
        obs.elevation = self.elevation;
        obs
    }
}

fn default_rock_scale() -> f32 {
    1.0
}

/// Geological classification of decorative and obstacle rock formations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RockType {
    /// Dense hard grey igneous granite boulder with rugged angular facets.
    #[default]
    Granite,
    /// Warm ochre / desert tan sedimentary sandstone with rounded weathering.
    Sandstone,
    /// Dark blue-grey / graphite metamorphic slate with flat layered strata.
    Slate,
    /// High-altitude alpine stone formation capped with bright white snow and ice.
    SnowCapped,
}

impl RockType {
    pub const ALL: [Self; 4] = [
        Self::Granite,
        Self::Sandstone,
        Self::Slate,
        Self::SnowCapped,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Granite => "Granite",
            Self::Sandstone => "Sandstone",
            Self::Slate => "Slate",
            Self::SnowCapped => "Snow-Capped",
        }
    }

    pub const fn default_radius(self) -> f32 {
        match self {
            Self::Granite => 1.2,
            Self::Sandstone => 1.4,
            Self::Slate => 1.1,
            Self::SnowCapped => 1.3,
        }
    }
}

/// A solid decorative rock obstacle with stone collision physics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rock {
    pub id: usize,
    pub position: Vec2,
    #[serde(default)]
    pub rock_type: RockType,
    #[serde(default = "default_rock_scale")]
    pub scale: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub elevation: f32,
}

impl Rock {
    pub fn new(id: usize, position: Vec2, rock_type: RockType) -> Self {
        Self {
            id,
            position,
            rock_type,
            scale: 1.0,
            rotation: 0.0,
            elevation: 0.0,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.clamp(0.2, 5.0);
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.rock_type.default_radius() * self.scale.max(0.2)
    }

    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        let r = self.radius();
        (self.position - point).length_squared() <= r * r
    }

    pub fn to_obstacle(&self) -> Obstacle {
        let mut obs = Obstacle::circle(
            self.id,
            self.position,
            self.radius(),
            format!("{} Rock #{}", self.rock_type.name(), self.id),
        );
        obs.restitution = 0.25;
        obs.friction = 0.60;
        obs.elevation = self.elevation;
        obs
    }
}

fn default_building_roof_color() -> [f32; 3] {
    [0.28, 0.30, 0.34]
}

/// Architectural style and layout of racetrack buildings and trackside facilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingStyle {
    /// Formula / GT pit garages with shutter bays and team equipment.
    #[default]
    PitGarage,
    /// Race control observation tower with panoramic glass windows and antennas.
    ControlTower,
    /// Offroad, rallycross or autocross paddock service awning tent.
    PaddockTent,
    /// Indoor hall perimeter wall with reinforced industrial panels.
    HallWall,
    /// Solid concrete structural support pillar.
    Pillar,
}

impl BuildingStyle {
    pub const ALL: [Self; 5] = [
        Self::PitGarage,
        Self::ControlTower,
        Self::PaddockTent,
        Self::HallWall,
        Self::Pillar,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::PitGarage => "Pit Garage",
            Self::ControlTower => "Control Tower",
            Self::PaddockTent => "Paddock Tent",
            Self::HallWall => "Hall Wall",
            Self::Pillar => "Pillar",
        }
    }
}

/// A solid building or facility structure with rigid boundary collision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: usize,
    pub center: Vec2,
    pub size: Vec2,
    pub angle: f32,
    #[serde(default)]
    pub style: BuildingStyle,
    #[serde(default = "default_building_roof_color")]
    pub roof_color: [f32; 3],
    #[serde(default)]
    pub elevation: f32,
}

impl Building {
    pub fn new(id: usize, center: Vec2, size: Vec2, angle: f32, style: BuildingStyle) -> Self {
        Self {
            id,
            center,
            size: Vec2::new(size.x.clamp(1.0, 500.0), size.y.clamp(1.0, 500.0)),
            angle,
            style,
            roof_color: default_building_roof_color(),
            elevation: 0.0,
        }
    }

    pub fn with_roof_color(mut self, rgb: [f32; 3]) -> Self {
        self.roof_color = rgb;
        self
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Unit forward vector along the building width.
    #[inline]
    pub fn direction(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    /// Unit perpendicular normal vector across the building depth.
    #[inline]
    pub fn normal(&self) -> Vec2 {
        let dir = self.direction();
        Vec2::new(-dir.y, dir.x)
    }

    /// Computes the 4 corners of the building's rectangular footprint.
    pub fn corners(&self) -> [Vec2; 4] {
        let fwd = self.direction() * (self.size.x * 0.5);
        let nrm = self.normal() * (self.size.y * 0.5);

        [
            self.center - fwd - nrm,
            self.center + fwd - nrm,
            self.center + fwd + nrm,
            self.center - fwd + nrm,
        ]
    }

    /// Tests whether a point lies within the building footprint.
    pub fn contains(&self, p: Vec2) -> bool {
        self.surface_shape().contains(p)
    }

    /// Returns the oriented 2D surface shape of the building footprint.
    pub fn surface_shape(&self) -> SurfaceShape {
        SurfaceShape::OrientedBox {
            center: self.center,
            half_extents: self.size * 0.5,
            angle: self.angle,
        }
    }

    /// Generates a solid oriented box obstacle with structural collision properties.
    pub fn to_obstacle(&self) -> Obstacle {
        let mut obs = Obstacle::oriented_box(
            self.id,
            self.center,
            self.size * 0.5,
            self.angle,
            format!("{} #{}", self.style.name(), self.id),
        );
        obs.restitution = 0.20;
        obs.friction = 0.50;
        obs.elevation = self.elevation;
        obs
    }
}

/// Architectural style and layout of a racetrack grandstand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrandstandStyle {
    /// Open stepped concrete bleachers with colored seating and crowd.
    #[default]
    OpenBleachers,
    /// Covered stadium grandstand with rear cantilever roof canopy casting shade.
    CoveredStadium,
    /// Low-profile hillside or earthwork tiered concrete seating embankment.
    HillsideBleachers,
}

impl GrandstandStyle {
    pub const ALL: [Self; 3] = [
        Self::OpenBleachers,
        Self::CoveredStadium,
        Self::HillsideBleachers,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::OpenBleachers => "Open Bleachers",
            Self::CoveredStadium => "Covered Stadium",
            Self::HillsideBleachers => "Hillside Bleachers",
        }
    }
}

/// A structured spectator grandstand ("gradas") with concrete collision boundaries and surface footprint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grandstand {
    pub id: usize,
    pub center: Vec2,
    pub length: f32,
    pub depth: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(default = "default_grandstand_tiers")]
    pub tiers: u32,
    #[serde(default)]
    pub style: GrandstandStyle,
    #[serde(default)]
    pub seat_color: Option<[f32; 3]>,
    #[serde(default)]
    pub elevation: f32,
}

impl Grandstand {
    pub fn new(id: usize, center: Vec2, length: f32, depth: f32, angle: f32) -> Self {
        Self {
            id,
            center,
            length: length.clamp(2.0, 500.0),
            depth: depth.clamp(2.0, 100.0),
            angle,
            tiers: 6,
            style: GrandstandStyle::OpenBleachers,
            seat_color: None,
            elevation: 0.0,
        }
    }

    pub fn with_style(mut self, style: GrandstandStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_tiers(mut self, tiers: u32) -> Self {
        self.tiers = tiers.clamp(2, 40);
        self
    }

    pub fn with_seat_color(mut self, rgb: [f32; 3]) -> Self {
        self.seat_color = Some(rgb);
        self
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Unit forward vector along the grandstand's length (parallel to track).
    #[inline]
    pub fn length_direction(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    /// Unit normal vector pointing from the back toward the front trackside edge.
    #[inline]
    pub fn facing_normal(&self) -> Vec2 {
        let dir = self.length_direction();
        Vec2::new(-dir.y, dir.x)
    }

    /// Computes the 4 corners of the grandstand's rectangular footprint.
    /// [Front-Left, Front-Right, Rear-Right, Rear-Left] relative to facing normal.
    pub fn corners(&self) -> [Vec2; 4] {
        let fwd = self.length_direction() * (self.length * 0.5);
        let facing = self.facing_normal() * (self.depth * 0.5);

        // Front edge is at -facing, rear edge is at +facing
        let front_left = self.center - fwd - facing;
        let front_right = self.center + fwd - facing;
        let rear_right = self.center + fwd + facing;
        let rear_left = self.center - fwd + facing;

        [front_left, front_right, rear_right, rear_left]
    }

    /// Tests whether a point lies within the grandstand's rectangular footprint.
    pub fn contains(&self, p: Vec2) -> bool {
        self.surface_shape().contains(p)
    }

    /// Returns the oriented 2D surface shape of the grandstand footprint.
    pub fn surface_shape(&self) -> SurfaceShape {
        SurfaceShape::OrientedBox {
            center: self.center,
            half_extents: Vec2::new(self.length * 0.5, self.depth * 0.5),
            angle: self.angle,
        }
    }

    /// Generates a concrete surface zone representing the grandstand's pavement apron and footprint.
    pub fn surface_zone(&self) -> SurfaceZone {
        SurfaceZone::new(
            self.surface_shape(),
            SurfaceType::Concrete,
            format!("Grandstand #{} Concrete Apron", self.id),
        )
        .with_layer(SurfaceLayer::BelowTrack)
    }

    /// Generates a solid oriented box obstacle with concrete collision properties.
    pub fn to_obstacle(&self) -> Obstacle {
        let mut obs = Obstacle::oriented_box(
            self.id,
            self.center,
            Vec2::new(self.length * 0.5, self.depth * 0.5),
            self.angle,
            format!("Grandstand #{}", self.id),
        );
        obs.restitution = BarrierType::Concrete.default_restitution();
        obs.friction = BarrierType::Concrete.default_friction();
        obs.elevation = self.elevation;
        obs
    }

    /// Returns the front trackside wall barrier segment protecting the grandstand.
    pub fn front_barrier(&self) -> WallBarrier {
        let corners = self.corners();
        WallBarrier::with_elevation(
            corners[0],
            corners[1],
            BarrierType::Concrete,
            self.elevation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_type_properties() {
        for &t in &TreeType::ALL {
            if t == TreeType::Bush {
                assert_eq!(t.default_trunk_radius(), 0.0);
            } else {
                assert!(t.default_trunk_radius() > 0.1);
            }
            assert!(t.default_canopy_radius() > t.default_trunk_radius());
            assert!(t.canopy_drag_deceleration() > 0.5);
            assert!(!t.name().is_empty());
        }

        assert!(TreeType::Pine.canopy_drag_deceleration() > TreeType::Palm.canopy_drag_deceleration());
    }

    #[test]
    fn test_tree_trunk_and_canopy_containment() {
        let tree = Tree::new(1, Vec2::new(10.0, 20.0), TreeType::Oak).with_scale(1.2);
        assert!(tree.contains_trunk(Vec2::new(10.1, 20.1)));
        assert!(tree.contains_canopy(Vec2::new(10.1, 20.1)));

        // In canopy but outside trunk
        let edge_point = Vec2::new(10.0 + tree.canopy_radius() * 0.7, 20.0);
        assert!(tree.contains_canopy(edge_point));
        assert!(!tree.contains_trunk(edge_point));

        // Outside tree completely
        let far_point = Vec2::new(50.0, 50.0);
        assert!(!tree.contains_canopy(far_point));
        assert!(!tree.contains_trunk(far_point));

        // Trunk obstacle properties
        let obs = tree.trunk_obstacle();
        assert_eq!(obs.restitution, 0.25);
        assert_eq!(obs.friction, 0.55);

        // Bush has no trunk
        let bush = Tree::new(2, Vec2::new(10.0, 20.0), TreeType::Bush);
        assert!(!bush.has_trunk());
        assert!(!bush.contains_trunk(Vec2::new(10.0, 20.0)));
    }

    #[test]
    fn test_rock_properties_and_obstacle() {
        for &r in &RockType::ALL {
            assert!(r.default_radius() >= 1.0);
            assert!(!r.name().is_empty());
        }

        let rock = Rock::new(1, Vec2::new(15.0, 25.0), RockType::Granite).with_scale(1.5);
        assert!(rock.contains(Vec2::new(15.2, 25.1)));
        assert!(!rock.contains(Vec2::new(25.0, 25.0)));

        let obs = rock.to_obstacle();
        assert_eq!(obs.restitution, 0.25);
        assert_eq!(obs.friction, 0.60);
    }

    #[test]
    fn test_building_properties_and_obstacle() {
        for &b in &BuildingStyle::ALL {
            assert!(!b.name().is_empty());
        }

        let building = Building::new(1, Vec2::new(50.0, 50.0), Vec2::new(20.0, 10.0), 0.0, BuildingStyle::PitGarage);
        assert!(building.contains(Vec2::new(50.0, 50.0)));
        assert!(building.contains(Vec2::new(55.0, 52.0)));
        assert!(!building.contains(Vec2::new(70.0, 50.0)));

        let obs = building.to_obstacle();
        assert_eq!(obs.restitution, 0.20);
        assert_eq!(obs.friction, 0.50);
    }

    #[test]
    fn test_grandstand_geometry_and_surface() {
        let stand = Grandstand::new(1, Vec2::new(0.0, 10.0), 40.0, 12.0, 0.0)
            .with_style(GrandstandStyle::CoveredStadium)
            .with_tiers(10);

        assert_eq!(stand.length, 40.0);
        assert_eq!(stand.depth, 12.0);
        assert_eq!(stand.tiers, 10);
        assert_eq!(stand.style, GrandstandStyle::CoveredStadium);

        // Center should be inside
        assert!(stand.contains(Vec2::new(0.0, 10.0)));
        // Far point outside
        assert!(!stand.contains(Vec2::new(0.0, 30.0)));

        // Surface zone should be concrete
        let zone = stand.surface_zone();
        assert_eq!(zone.surface, SurfaceType::Concrete);

        // Obstacle should have concrete restitution
        let obs = stand.to_obstacle();
        assert_eq!(obs.restitution, BarrierType::Concrete.default_restitution());
        assert_eq!(obs.friction, BarrierType::Concrete.default_friction());

        // Front barrier should be concrete
        let wall = stand.front_barrier();
        assert_eq!(wall.barrier_type, BarrierType::Concrete);
    }
}
