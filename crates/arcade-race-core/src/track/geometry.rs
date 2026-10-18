use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;
use super::junction_kit::Side;
use super::network::SegmentId;
use super::scenery::{Building, Grandstand, Rock, Tree};
use super::spline::TrackSpline;

/// 2D Line Segment defined by two endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSegment {
    pub start: Vec2,
    pub end: Vec2,
}

impl LineSegment {
    pub const fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    /// Length of the segment in meters.
    #[inline]
    pub fn length(&self) -> f32 {
        (self.end - self.start).length()
    }

    /// Squared length of the segment.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        (self.end - self.start).length_squared()
    }

    /// Normalized direction unit vector from start to end.
    #[inline]
    pub fn direction(&self) -> Vec2 {
        let delta = self.end - self.start;
        let len = delta.length();
        if len > 1e-6 {
            delta / len
        } else {
            Vec2::X
        }
    }

    /// Normalized normal vector pointing to the left of the segment direction.
    #[inline]
    pub fn normal(&self) -> Vec2 {
        let dir = self.direction();
        Vec2::new(-dir.y, dir.x)
    }

    /// Midpoint of the segment.
    #[inline]
    pub fn midpoint(&self) -> Vec2 {
        (self.start + self.end) * 0.5
    }

    /// Finds the closest point on this line segment to an external query point `p`.
    #[inline]
    pub fn closest_point(&self, p: Vec2) -> Vec2 {
        let ab = self.end - self.start;
        let len_sq = ab.length_squared();
        if len_sq < 1e-6 {
            return self.start;
        }
        let ap = p - self.start;
        let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
        self.start + ab * t
    }

    /// Computes the distance from an external point `p` to the segment.
    #[inline]
    pub fn distance_to_point(&self, p: Vec2) -> f32 {
        (p - self.closest_point(p)).length()
    }

    /// Computes squared distance from an external point `p` to the segment.
    #[inline]
    pub fn distance_sq_to_point(&self, p: Vec2) -> f32 {
        (p - self.closest_point(p)).length_squared()
    }

    /// Intersects two 2D line segments, returning the intersection point if they cross.
    pub fn intersect_segment(&self, other: &LineSegment) -> Option<Vec2> {
        let p = self.start;
        let r = self.end - self.start;
        let q = other.start;
        let s = other.end - other.start;

        let r_cross_s = r.x * s.y - r.y * s.x;
        let q_minus_p = q - p;
        let q_minus_p_cross_r = q_minus_p.x * r.y - q_minus_p.y * r.x;

        if r_cross_s.abs() < 1e-6 {
            return None; // Collinear or parallel
        }

        let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / r_cross_s;
        let u = q_minus_p_cross_r / r_cross_s;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some(p + r * t)
        } else {
            None
        }
    }

    /// Tests intersection of a ray (origin + dir * t) with this line segment.
    /// Returns Some((distance, hit_normal)) if hit, where distance <= max_range.
    #[inline(always)]
    pub fn intersect_ray(&self, origin: Vec2, dir: Vec2, max_range: f32) -> Option<(f32, Vec2)> {
        // Quick AABB rejection
        let seg_min_x = self.start.x.min(self.end.x);
        let seg_max_x = self.start.x.max(self.end.x);
        let seg_min_y = self.start.y.min(self.end.y);
        let seg_max_y = self.start.y.max(self.end.y);

        let end_x = origin.x + dir.x * max_range;
        let end_y = origin.y + dir.y * max_range;
        let ray_min_x = origin.x.min(end_x);
        let ray_max_x = origin.x.max(end_x);
        let ray_min_y = origin.y.min(end_y);
        let ray_max_y = origin.y.max(end_y);

        if ray_max_x < seg_min_x || ray_min_x > seg_max_x || ray_max_y < seg_min_y || ray_min_y > seg_max_y {
            return None;
        }

        let v1 = origin - self.start;
        let v2 = self.end - self.start;
        let dot = v2.y * dir.x - v2.x * dir.y;
        if dot.abs() < 1e-6 {
            return None; // Parallel
        }

        let inv_dot = 1.0 / dot;
        let t1 = (v2.x * v1.y - v2.y * v1.x) * inv_dot;
        if t1 < 0.0 || t1 > max_range {
            return None;
        }

        let t2 = (v1.y * dir.x - v1.x * dir.y) * inv_dot;
        if !(0.0..=1.0).contains(&t2) {
            return None;
        }

        let mut normal = self.normal();
        if normal.dot(dir) > 0.0 {
            normal = -normal;
        }
        Some((t1, normal))
    }

    /// Tests intersection of a ray with this line segment, returning only the hit distance.
    /// Avoids square roots and normal vector calculations for high-throughput ray sweeps (Spec 084).
    #[inline(always)]
    pub fn intersect_ray_dist(&self, origin: Vec2, dir: Vec2, max_range: f32) -> Option<f32> {
        // Quick AABB rejection
        let seg_min_x = self.start.x.min(self.end.x);
        let seg_max_x = self.start.x.max(self.end.x);
        let seg_min_y = self.start.y.min(self.end.y);
        let seg_max_y = self.start.y.max(self.end.y);

        let end_x = origin.x + dir.x * max_range;
        let end_y = origin.y + dir.y * max_range;
        let ray_min_x = origin.x.min(end_x);
        let ray_max_x = origin.x.max(end_x);
        let ray_min_y = origin.y.min(end_y);
        let ray_max_y = origin.y.max(end_y);

        if ray_max_x < seg_min_x || ray_min_x > seg_max_x || ray_max_y < seg_min_y || ray_min_y > seg_max_y {
            return None;
        }

        let v1 = origin - self.start;
        let v2 = self.end - self.start;
        let dot = v2.y * dir.x - v2.x * dir.y;
        if dot.abs() < 1e-6 {
            return None; // Parallel
        }

        let inv_dot = 1.0 / dot;
        let t1 = (v2.x * v1.y - v2.y * v1.x) * inv_dot;
        if t1 < 0.0 || t1 > max_range {
            return None;
        }

        let t2 = (v1.y * dir.x - v1.x * dir.y) * inv_dot;
        if !(0.0..=1.0).contains(&t2) {
            return None;
        }

        Some(t1)
    }
}

/// Physical classification of track barriers and walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarrierType {
    /// Rigid concrete wall (high restitution, low-to-moderate sliding friction).
    Concrete,
    /// Steel barrier (medium restitution, deformable, moderate friction & bodywork snagging).
    Steel,
    /// Energy-absorbing tire stack barrier (low restitution, high rubber friction).
    TireWall,
    /// Low track-edge curb wall.
    CurbWall,
    /// Virtual non-physical boundary delimiter (zero collision, delimits surface transition zones).
    Virtual,
}

impl BarrierType {
    /// Whether this barrier possesses physical collidable geometry.
    #[inline]
    pub const fn is_physical(self) -> bool {
        !matches!(self, Self::Virtual)
    }

    /// Human-readable display name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Concrete => "Concrete",
            Self::Steel => "Steel",
            Self::TireWall => "Rubber Tyres",
            Self::CurbWall => "Curb Wall",
            Self::Virtual => "Virtual Boundary",
        }
    }

    /// Normal restitution coefficient ($e \in [0, 1]$) determining bounce elasticity on impact.
    pub const fn default_restitution(self) -> f32 {
        match self {
            Self::Concrete => 0.65,
            Self::Steel => 0.42,
            Self::TireWall => 0.18,
            Self::CurbWall => 0.30,
            Self::Virtual => 0.0,
        }
    }

    /// Surface Coulomb friction coefficient ($\mu$) when tires or bodywork slide against the barrier.
    pub const fn default_friction(self) -> f32 {
        match self {
            Self::Concrete => 0.32,
            Self::Steel => 0.45,
            Self::TireWall => 0.80,
            Self::CurbWall => 0.40,
            Self::Virtual => 0.0,
        }
    }

    /// Tangential deceleration rate (m/s²) applied as scraping braking resistance when rubbing against the wall.
    pub const fn scraping_deceleration(self) -> f32 {
        match self {
            Self::Concrete => 9.0,   // Smooth concrete grinding (~0.9g)
            Self::Steel => 14.0,     // Corrugated steel beam and post catching (~1.4g)
            Self::TireWall => 24.0,  // Soft high-grip rubber compression drag (~2.4g)
            Self::CurbWall => 7.0,   // Low curb wall resistance (~0.7g)
            Self::Virtual => 0.0,
        }
    }

    /// Rotational snag factor ($[0, 1]$) controlling yaw torque induced when vehicle scrapes along barrier.
    pub const fn snag_torque_factor(self) -> f32 {
        match self {
            Self::Concrete => 0.12,  // Flat plane allows smooth sliding with minimal yaw deflection
            Self::Steel => 0.25,     // Corrugations and upright posts snag corners and induce moderate yaw
            Self::TireWall => 0.45,  // Aggressive rubber bite grips corners and spins car into barrier
            Self::CurbWall => 0.10,  // Low curb redirects car with little torque
            Self::Virtual => 0.0,
        }
    }

    /// Structural impact energy absorption factor ($[0, 1]$) for vehicle damage simulation.
    /// Higher values mean the barrier yields/cushions more, transferring less kinetic energy into vehicle chassis damage.
    pub const fn energy_absorption_factor(self) -> f32 {
        match self {
            Self::Concrete => 0.10,  // Stiffest structure: 90% of impact energy transferred to vehicle
            Self::Steel => 0.40,     // Deformable steel beam yields, absorbing 40% of impact energy
            Self::TireWall => 0.75,  // Compressive tire stacks absorb 75% of impact energy
            Self::CurbWall => 0.25,  // Low barrier absorption
            Self::Virtual => 1.0,    // Fully yielding: zero collision damage
        }
    }
}

/// A track barrier or perimeter wall segment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallBarrier {
    pub segment: LineSegment,
    pub restitution: f32,
    pub friction: f32,
    pub barrier_type: BarrierType,
    #[serde(default)]
    pub elevation: f32,
    #[serde(default)]
    pub is_bridge: bool,
}

impl WallBarrier {
    /// Returns true if this wall has physical collision geometry.
    #[inline]
    pub fn is_physical(&self) -> bool {
        self.barrier_type.is_physical()
    }

    pub fn new(start: Vec2, end: Vec2, barrier_type: BarrierType) -> Self {
        Self {
            segment: LineSegment::new(start, end),
            restitution: barrier_type.default_restitution(),
            friction: barrier_type.default_friction(),
            barrier_type,
            elevation: 0.0,
            is_bridge: false,
        }
    }

    pub fn with_elevation(
        start: Vec2,
        end: Vec2,
        barrier_type: BarrierType,
        elevation: f32,
    ) -> Self {
        Self {
            segment: LineSegment::new(start, end),
            restitution: barrier_type.default_restitution(),
            friction: barrier_type.default_friction(),
            barrier_type,
            elevation,
            is_bridge: false,
        }
    }

    pub fn with_bridge(mut self, is_bridge: bool) -> Self {
        self.is_bridge = is_bridge;
        self
    }

    pub fn with_physics(
        start: Vec2,
        end: Vec2,
        barrier_type: BarrierType,
        restitution: f32,
        friction: f32,
    ) -> Self {
        Self {
            segment: LineSegment::new(start, end),
            restitution,
            friction,
            barrier_type,
            elevation: 0.0,
            is_bridge: false,
        }
    }
}

/// Record of a wall segment pruned or suppressed due to clearance conflicts with adjacent roads (Spec 112).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallSuppressionRecord {
    /// Left or right wall relative to track spline.
    pub side: Side,
    /// Spatial span of the suppressed wall segment.
    pub span: LineSegment,
    /// ID of the adjacent road segment causing the clearance conflict.
    pub conflicting_segment: SegmentId,
    /// Arc-length progress distance on the track spline where suppression starts (m).
    pub start_station: f32,
    /// Arc-length progress distance on the track spline where suppression ends (m).
    pub end_station: f32,
}

impl WallSuppressionRecord {
    pub const fn new(
        side: Side,
        span: LineSegment,
        conflicting_segment: SegmentId,
        start_station: f32,
        end_station: f32,
    ) -> Self {
        Self {
            side,
            span,
            conflicting_segment,
            start_station,
            end_station,
        }
    }

    /// Shortest distance from an arbitrary point to this suppressed wall span.
    #[inline]
    pub fn distance_to_point(&self, point: Vec2) -> f32 {
        self.span.distance_to_point(point)
    }

    /// Whether this suppression covers or is within `margin` of a spline station `s`.
    #[inline]
    pub fn contains_station(&self, station: f32, margin: f32) -> bool {
        let (lo, hi) = if self.start_station <= self.end_station {
            (self.start_station, self.end_station)
        } else {
            (self.end_station, self.start_station)
        };
        station >= lo - margin && station <= hi + margin
    }
}

/// Geometric shape for 2D surface hazard zones and runoffs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SurfaceShape {
    Circle { center: Vec2, radius: f32 },
    Aabb { min: Vec2, max: Vec2 },
    OrientedBox { center: Vec2, half_extents: Vec2, angle: f32 },
    Polygon { vertices: Vec<Vec2> },
}

impl SurfaceShape {
    /// Tests whether a 2D world point lies within this surface shape.
    pub fn contains(&self, p: Vec2) -> bool {
        match self {
            Self::Circle { center, radius } => (*center - p).length_squared() <= radius * radius,
            Self::Aabb { min, max } => {
                p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
            }
            Self::OrientedBox {
                center,
                half_extents,
                angle,
            } => {
                let d = p - *center;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let local_x = (d.x * cos_a + d.y * sin_a).abs();
                let local_y = (-d.x * sin_a + d.y * cos_a).abs();
                local_x <= half_extents.x && local_y <= half_extents.y
            }
            Self::Polygon { vertices } => point_in_polygon(p, vertices),
        }
    }

    pub fn center(&self) -> Vec2 {
        match self {
            Self::Circle { center, .. } => *center,
            Self::Aabb { min, max } => (*min + *max) * 0.5,
            Self::OrientedBox { center, .. } => *center,
            Self::Polygon { vertices } => {
                if vertices.is_empty() {
                    Vec2::ZERO
                } else {
                    let sum: Vec2 = vertices.iter().copied().sum();
                    sum / vertices.len() as f32
                }
            }
        }
    }

    pub fn triangle(v0: Vec2, v1: Vec2, v2: Vec2) -> Self {
        Self::Polygon {
            vertices: vec![v0, v1, v2],
        }
    }
}

/// Standard ray-casting 2D point-in-polygon containment test.
pub fn point_in_polygon(p: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let vi = vertices[i];
        let vj = vertices[j];
        if ((vi.y > p.y) != (vj.y > p.y))
            && (p.x < (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Standard 2D point-in-triangle containment test using 2D cross products.
#[inline]
pub fn point_in_triangle_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    // A zero-area triangle (coincident or collinear corners, e.g. a junction whose ingress and
    // egress sockets share one point) contains nothing; without this every point tests inside.
    let area2 = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area2.abs() < 1e-6 {
        return false;
    }
    let cross1 = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
    let cross2 = (c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x);
    let cross3 = (a.x - c.x) * (p.y - c.y) - (a.y - c.y) * (p.x - c.x);
    (cross1 >= -1e-3 && cross2 >= -1e-3 && cross3 >= -1e-3)
        || (cross1 <= 1e-3 && cross2 <= 1e-3 && cross3 <= 1e-3)
}

/// Tests whether point `p` lies inside the convex or non-self-intersecting quad ABCD.
#[inline]
pub fn point_in_quad_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    point_in_triangle_2d(p, a, b, c) || point_in_triangle_2d(p, a, c, d)
}

/// Layering depth of a surface zone relative to the drivable track ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceLayer {
    /// Rendered below the track ribbon and only affects car physics when off-track.
    #[default]
    BelowTrack,
    /// Rendered on top of the track ribbon and affects car physics both on and off track.
    AboveTrack,
}

/// A specific spatial surface zone (e.g. sand trap, oil slick, ice patch, runoff).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceZone {
    pub shape: SurfaceShape,
    pub surface: SurfaceType,
    pub name: String,
    #[serde(default)]
    pub layer: SurfaceLayer,
}

impl SurfaceZone {
    pub fn new(shape: SurfaceShape, surface: SurfaceType, name: impl Into<String>) -> Self {
        let layer = if surface.is_on_track_hazard() {
            SurfaceLayer::AboveTrack
        } else {
            SurfaceLayer::BelowTrack
        };
        Self {
            shape,
            surface,
            name: name.into(),
            layer,
        }
    }

    pub fn with_layer(mut self, layer: SurfaceLayer) -> Self {
        self.layer = layer;
        self
    }

    pub fn is_above_track(&self) -> bool {
        self.layer == SurfaceLayer::AboveTrack
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.shape.contains(p)
    }
}

/// Static obstacle shape on or near the track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObstacleShape {
    Circle { center: Vec2, radius: f32 },
    Box { center: Vec2, half_extents: Vec2, angle: f32 },
    Polygon { vertices: Vec<Vec2> },
}

/// A static track obstacle (bollard, tire bundle, trackside post).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    pub id: usize,
    pub shape: ObstacleShape,
    pub restitution: f32,
    pub friction: f32,
    pub name: String,
    #[serde(default)]
    pub elevation: f32,
}

impl Obstacle {
    pub fn circle(id: usize, center: Vec2, radius: f32, name: impl Into<String>) -> Self {
        Self {
            id,
            shape: ObstacleShape::Circle { center, radius },
            restitution: 0.3,
            friction: 0.5,
            name: name.into(),
            elevation: 0.0,
        }
    }

    pub fn oriented_box(
        id: usize,
        center: Vec2,
        half_extents: Vec2,
        angle: f32,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            shape: ObstacleShape::Box {
                center,
                half_extents,
                angle,
            },
            restitution: 0.4,
            friction: 0.4,
            name: name.into(),
            elevation: 0.0,
        }
    }

    pub fn polygon(id: usize, vertices: Vec<Vec2>, name: impl Into<String>) -> Self {
        Self {
            id,
            shape: ObstacleShape::Polygon { vertices },
            restitution: 0.4,
            friction: 0.4,
            name: name.into(),
            elevation: 0.0,
        }
    }

    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn center(&self) -> Vec2 {
        match &self.shape {
            ObstacleShape::Circle { center, .. } => *center,
            ObstacleShape::Box { center, .. } => *center,
            ObstacleShape::Polygon { vertices } => {
                if vertices.is_empty() {
                    Vec2::ZERO
                } else {
                    let sum: Vec2 = vertices.iter().copied().sum();
                    sum / (vertices.len() as f32)
                }
            }
        }
    }

    pub fn set_center(&mut self, new_center: Vec2) {
        match &mut self.shape {
            ObstacleShape::Circle { center, .. } => *center = new_center,
            ObstacleShape::Box { center, .. } => *center = new_center,
            ObstacleShape::Polygon { vertices } => {
                if !vertices.is_empty() {
                    let sum: Vec2 = vertices.iter().copied().sum();
                    let current = sum / (vertices.len() as f32);
                    let delta = new_center - current;
                    for v in vertices {
                        *v += delta;
                    }
                }
            }
        }
    }

    /// Computes the world-space axis-aligned bounding box `(min, max)` of this obstacle.
    #[inline]
    pub fn aabb(&self) -> (Vec2, Vec2) {
        match &self.shape {
            ObstacleShape::Circle { center, radius } => {
                let r = Vec2::splat(*radius);
                (*center - r, *center + r)
            }
            ObstacleShape::Box {
                center,
                half_extents,
                angle,
            } => {
                let (sin_a, cos_a) = angle.sin_cos();
                let rx = half_extents.x * cos_a.abs() + half_extents.y * sin_a.abs();
                let ry = half_extents.x * sin_a.abs() + half_extents.y * cos_a.abs();
                let r = Vec2::new(rx, ry);
                (*center - r, *center + r)
            }
            ObstacleShape::Polygon { vertices } => {
                if vertices.is_empty() {
                    (Vec2::ZERO, Vec2::ZERO)
                } else {
                    let mut min = Vec2::splat(f32::INFINITY);
                    let mut max = Vec2::splat(f32::NEG_INFINITY);
                    for &v in vertices {
                        min = min.min(v);
                        max = max.max(v);
                    }
                    (min, max)
                }
            }
        }
    }

    /// Intersects a ray against this obstacle.
    pub fn intersect_ray(&self, origin: Vec2, dir: Vec2, max_range: f32) -> Option<(f32, Vec2)> {
        match &self.shape {
            ObstacleShape::Circle { center, radius } => {
                let m = origin - *center;
                let b = m.dot(dir);
                let c = m.length_squared() - radius * radius;

                if c > 0.0 && b > 0.0 {
                    return None;
                }
                let discr = b * b - c;
                if discr < 0.0 {
                    return None;
                }
                let mut t = -b - discr.sqrt();
                if t < 0.0 {
                    t = 0.0;
                }
                if t <= max_range {
                    let hit_point = origin + dir * t;
                    let mut normal = (hit_point - *center).normalize_or_zero();
                    if normal.length_squared() < 1e-4 {
                        normal = -dir;
                    }
                    Some((t, normal))
                } else {
                    None
                }
            }
            ObstacleShape::Box {
                center,
                half_extents,
                angle,
            } => {
                // Transform ray into OBB local frame
                let d = origin - *center;
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                let local_origin = Vec2::new(d.x * cos_a + d.y * sin_a, -d.x * sin_a + d.y * cos_a);
                let local_dir =
                    Vec2::new(dir.x * cos_a + dir.y * sin_a, -dir.x * sin_a + dir.y * cos_a);

                let mut t_min = 0.0f32;
                let mut t_max = max_range;
                let mut hit_normal_local = Vec2::ZERO;

                // Test X slab
                if local_dir.x.abs() > 1e-6 {
                    let inv_d = 1.0 / local_dir.x;
                    let mut t1 = (-half_extents.x - local_origin.x) * inv_d;
                    let mut t2 = (half_extents.x - local_origin.x) * inv_d;
                    let mut n1 = Vec2::new(-1.0, 0.0);
                    if t1 > t2 {
                        std::mem::swap(&mut t1, &mut t2);
                        n1 = Vec2::new(1.0, 0.0);
                    }
                    if t1 > t_min {
                        t_min = t1;
                        hit_normal_local = n1;
                    }
                    t_max = t_max.min(t2);
                    if t_min > t_max {
                        return None;
                    }
                } else if local_origin.x.abs() > half_extents.x {
                    return None;
                }

                // Test Y slab
                if local_dir.y.abs() > 1e-6 {
                    let inv_d = 1.0 / local_dir.y;
                    let mut t1 = (-half_extents.y - local_origin.y) * inv_d;
                    let mut t2 = (half_extents.y - local_origin.y) * inv_d;
                    let mut n1 = Vec2::new(0.0, -1.0);
                    if t1 > t2 {
                        std::mem::swap(&mut t1, &mut t2);
                        n1 = Vec2::new(0.0, 1.0);
                    }
                    if t1 > t_min {
                        t_min = t1;
                        hit_normal_local = n1;
                    }
                    t_max = t_max.min(t2);
                    if t_min > t_max {
                        return None;
                    }
                } else if local_origin.y.abs() > half_extents.y {
                    return None;
                }

                if t_min <= max_range {
                    // Transform local normal to world normal
                    let world_normal = Vec2::new(
                        hit_normal_local.x * cos_a - hit_normal_local.y * sin_a,
                        hit_normal_local.x * sin_a + hit_normal_local.y * cos_a,
                    );
                    Some((t_min, world_normal))
                } else {
                    None
                }
            }
            ObstacleShape::Polygon { vertices } => {
                let n = vertices.len();
                if n < 3 {
                    return None;
                }
                let mut closest_hit: Option<(f32, Vec2)> = None;
                for i in 0..n {
                    let seg = LineSegment::new(vertices[i], vertices[(i + 1) % n]);
                    if let Some((t, normal)) = seg.intersect_ray(origin, dir, max_range) {
                        match closest_hit {
                            Some((best_t, _)) if t < best_t => closest_hit = Some((t, normal)),
                            None => closest_hit = Some((t, normal)),
                            _ => {}
                        }
                    }
                }
                closest_hit
            }
        }
    }
}

/// Starting grid / spawn pose for cars.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpawnPose {
    pub position: Vec2,
    pub angle: f32,
    pub grid_slot: usize,
}

impl SpawnPose {
    pub const fn new(position: Vec2, angle: f32, grid_slot: usize) -> Self {
        Self {
            position,
            angle,
            grid_slot,
        }
    }
}

/// Type of jump component in the track studio and physics engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RampKind {
    /// Standard single-segment curved launch ramp.
    #[default]
    Ramp,
    /// Cohesive multi-ridge rhythm section (multiple small mogul jumps close together).
    Whoops,
    /// Two-segment stunt mega jump: upward take-off ramp, open gap, and downslope landing ramp.
    MegaRamp,
}

impl RampKind {
    pub const ALL: [RampKind; 3] = [Self::Ramp, Self::Whoops, Self::MegaRamp];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ramp => "Ramp",
            Self::Whoops => "Whoops Moguls",
            Self::MegaRamp => "Mega Ramp",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Ramp => "Ramp",
            Self::Whoops => "Whoops",
            Self::MegaRamp => "Mega",
        }
    }
}

fn default_whoop_spacing() -> f32 {
    4.0
}

fn default_mega_gap_length() -> f32 {
    12.0
}

/// A track jump component launching vehicles into the air (Ramp, Whoops Moguls, or two-segment Mega Ramp).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JumpRamp {
    pub id: usize,
    pub shape: SurfaceShape,
    pub direction: Vec2,
    pub launch_speed: f32,
    pub ramp_angle_deg: f32,
    pub height: f32,
    pub name: String,
    #[serde(default)]
    pub surface: SurfaceType,
    #[serde(default)]
    pub kind: RampKind,
    #[serde(default = "default_whoop_spacing")]
    pub whoop_spacing: f32,
    #[serde(default = "default_mega_gap_length")]
    pub gap_length: f32,
}

impl JumpRamp {
    pub fn new(
        id: usize,
        shape: SurfaceShape,
        direction: Vec2,
        launch_speed: f32,
        ramp_angle_deg: f32,
        height: f32,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            shape,
            direction: direction.normalize_or_zero(),
            launch_speed,
            ramp_angle_deg,
            height,
            name: name.into(),
            surface: SurfaceType::Asphalt,
            kind: RampKind::Ramp,
            whoop_spacing: default_whoop_spacing(),
            gap_length: default_mega_gap_length(),
        }
    }

    pub fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = surface;
        self
    }

    pub fn with_kind(mut self, kind: RampKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_whoop_spacing(mut self, spacing: f32) -> Self {
        self.whoop_spacing = spacing.clamp(1.5, 25.0);
        self
    }

    pub fn with_gap_length(mut self, gap_length: f32) -> Self {
        self.gap_length = gap_length.clamp(2.0, 80.0);
        self
    }

    /// Number of closely spaced mogul ridges when `kind == RampKind::Whoops`.
    pub fn whoop_count(&self) -> usize {
        ((self.length() / self.whoop_spacing.max(1.5)).round() as usize).clamp(2, 32)
    }

    /// Clamped gap length in meters between take-off and landing segments for `RampKind::MegaRamp`.
    pub fn effective_gap_length(&self) -> f32 {
        let len = self.length().max(4.0);
        let raw = if self.gap_length > 0.1 {
            self.gap_length
        } else {
            len * 0.34
        };
        raw.clamp(len * 0.15, len * 0.60)
    }

    /// Returns `(s_takeoff_end, s_landing_start)` in normalized `[0.0, 1.0]` progress for `RampKind::MegaRamp`.
    pub fn mega_segment_bounds(&self) -> (f32, f32) {
        let len = self.length().max(4.0);
        let gap = self.effective_gap_length();
        let rem = (len - gap).max(2.0);
        let takeoff_len = rem * 0.46;
        let s_takeoff_end = (takeoff_len / len).clamp(0.18, 0.48);
        let s_landing_start = ((takeoff_len + gap) / len).clamp(s_takeoff_end + 0.12, 0.82);
        (s_takeoff_end, s_landing_start)
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.shape.contains(p)
    }

    /// Returns the 2D orientation/yaw angle in radians.
    pub fn angle(&self) -> f32 {
        match &self.shape {
            SurfaceShape::OrientedBox { angle, .. } => *angle,
            _ => self.direction.y.atan2(self.direction.x),
        }
    }

    /// Returns normalized orientation angle in degrees in the range [0.0, 360.0).
    pub fn angle_deg(&self) -> f32 {
        let deg = self.angle().to_degrees() % 360.0;
        if deg < 0.0 {
            deg + 360.0
        } else {
            deg
        }
    }

    /// Sets the 2D orientation angle, updating both the shape and the launch direction vector.
    pub fn set_angle(&mut self, angle_rad: f32) {
        let dir = Vec2::new(angle_rad.cos(), angle_rad.sin()).normalize_or_zero();
        self.direction = dir;
        match &mut self.shape {
            SurfaceShape::OrientedBox { angle, .. } => {
                *angle = angle_rad;
            }
            SurfaceShape::Aabb { min, max } => {
                let center = (*min + *max) * 0.5;
                let half_extents = (*max - *min) * 0.5;
                self.shape = SurfaceShape::OrientedBox {
                    center,
                    half_extents,
                    angle: angle_rad,
                };
            }
            _ => {}
        }
    }

    /// Sets orientation angle directly from degrees (e.g. 0 to 365 degrees).
    pub fn set_angle_deg(&mut self, angle_deg: f32) {
        self.set_angle(angle_deg.to_radians());
    }

    /// Rotates the ramp by a delta angle in radians.
    pub fn rotate(&mut self, delta_rad: f32) {
        let current = self.angle();
        self.set_angle(current + delta_rad);
    }

    /// Returns the half-extents (half_length, half_width) of the ramp.
    pub fn half_extents(&self) -> Vec2 {
        match &self.shape {
            SurfaceShape::OrientedBox { half_extents, .. } => *half_extents,
            SurfaceShape::Aabb { min, max } => (*max - *min) * 0.5,
            SurfaceShape::Circle { radius, .. } => Vec2::splat(*radius),
            SurfaceShape::Polygon { vertices } => {
                if vertices.is_empty() {
                    Vec2::splat(2.0)
                } else {
                    let mut min = Vec2::splat(f32::MAX);
                    let mut max = Vec2::splat(f32::MIN);
                    for v in vertices {
                        min = min.min(*v);
                        max = max.max(*v);
                    }
                    (max - min) * 0.5
                }
            }
        }
    }

    /// Returns the full length of the ramp (along its forward axis).
    pub fn length(&self) -> f32 {
        self.half_extents().x * 2.0
    }

    /// Returns the full width of the ramp (across its lateral axis).
    pub fn width(&self) -> f32 {
        self.half_extents().y * 2.0
    }

    /// Sets the length and width of the ramp.
    pub fn set_size(&mut self, length: f32, width: f32) {
        let half_len = (length * 0.5).max(1.0);
        let half_wid = (width * 0.5).max(1.0);
        match &mut self.shape {
            SurfaceShape::OrientedBox { half_extents, .. } => {
                *half_extents = Vec2::new(half_len, half_wid);
            }
            SurfaceShape::Aabb { min, max } => {
                let center = (*min + *max) * 0.5;
                *min = center - Vec2::new(half_len, half_wid);
                *max = center + Vec2::new(half_len, half_wid);
            }
            SurfaceShape::Circle { radius, .. } => {
                *radius = half_len.max(half_wid);
            }
            SurfaceShape::Polygon { .. } => {}
        }
    }

    /// Sets the length of the ramp.
    pub fn set_length(&mut self, length: f32) {
        let wid = self.width();
        self.set_size(length, wid);
    }

    /// Sets the width of the ramp.
    pub fn set_width(&mut self, width: f32) {
        let len = self.length();
        self.set_size(len, width);
    }

    /// Returns the horizontal run of the incline portion up to height H: min(L, H / tan(theta)).
    pub fn incline_length(&self) -> f32 {
        let angle_rad = self.ramp_angle_deg.to_radians();
        let tan_val = angle_rad.tan();
        if tan_val > 1e-4 {
            (self.height / tan_val).min(self.length()).max(0.0)
        } else {
            self.length()
        }
    }

    /// Returns the length of the flat tabletop portion on top of the ramp (if any).
    pub fn flat_length(&self) -> f32 {
        (self.length() - self.incline_length()).max(0.0)
    }

    /// Returns the fitted pitch angle in degrees to eliminate the flat portion for the current length and height.
    pub fn fitted_pitch_deg(&self) -> f32 {
        let len = self.length().max(0.1);
        let h = self.height.max(0.01);
        (h / len).atan().to_degrees().clamp(1.0, 60.0)
    }

    /// Returns the fitted height in meters to match the incline slope for the current length and pitch angle.
    pub fn fitted_height(&self) -> f32 {
        let len = self.length().max(0.1);
        let angle_rad = self.ramp_angle_deg.to_radians();
        (len * angle_rad.tan()).clamp(0.2, 20.0)
    }

    /// Adjusts the length and width of the ramp by deltas.
    pub fn adjust_size(&mut self, delta_length: f32, delta_width: f32) {
        let current_len = self.length();
        let current_wid = self.width();
        self.set_size(
            (current_len + delta_length).clamp(2.0, 100.0),
            (current_wid + delta_width).clamp(1.0, 100.0),
        );
    }

    /// Scales the size of the ramp by a multiplier factor.
    pub fn scale_size(&mut self, factor: f32) {
        let current_len = self.length();
        let current_wid = self.width();
        self.set_size(
            (current_len * factor).clamp(2.0, 100.0),
            (current_wid * factor).clamp(1.0, 100.0),
        );
    }

    /// Returns the normalized progress [0.0, 1.0] from entrance (0.0) to exit (1.0).
    pub fn progress_along_ramp(&self, point: Vec2) -> f32 {
        let center = self.shape.center();
        let half_len = self.half_extents().x;
        let delta = point - center;
        let u = delta.dot(self.direction);
        ((u + half_len) / (2.0 * half_len)).clamp(0.0, 1.0)
    }

    /// Returns the surface elevation at normalized progress `s` in `[0.0, 1.0]`.
    pub fn surface_elevation_at_progress(&self, s: f32) -> f32 {
        let s = s.clamp(0.0, 1.0);
        match self.kind {
            RampKind::Ramp => {
                let len = self.length().max(0.1);
                let incline_len = self.incline_length().min(len);
                let s_incline = (incline_len / len).clamp(0.01, 1.0);

                if s <= s_incline {
                    let t = s / s_incline;
                    (self.height * t * t).clamp(0.0, self.height)
                } else {
                    self.height
                }
            }
            RampKind::Whoops => {
                let n = self.whoop_count() as f32;
                let f = (s * n).fract();
                (self.height * 0.5 * (1.0 - (std::f32::consts::TAU * f).cos()))
                    .clamp(0.0, self.height)
            }
            RampKind::MegaRamp => {
                let (s_takeoff, s_landing) = self.mega_segment_bounds();
                if s <= s_takeoff {
                    let t = (s / s_takeoff.max(0.01)).clamp(0.0, 1.0);
                    (self.height * t * t).clamp(0.0, self.height)
                } else if s < s_landing {
                    0.0
                } else {
                    let u = ((s - s_landing) / (1.0 - s_landing).max(0.01)).clamp(0.0, 1.0);
                    let catch_h = self.height * 0.92;
                    if u <= 0.15 {
                        catch_h
                    } else {
                        let d = (u - 0.15) / 0.85;
                        (catch_h * 0.5 * (1.0 + (std::f32::consts::PI * d).cos()))
                            .clamp(0.0, catch_h)
                    }
                }
            }
        }
    }

    /// Returns the natural curved surface elevation at a given world point on the component.
    pub fn surface_elevation_at(&self, point: Vec2) -> f32 {
        let s = self.progress_along_ramp(point);
        self.surface_elevation_at_progress(s)
    }

    /// Returns the local instantaneous pitch angle in radians of the ramp surface at the given point.
    /// Positive values indicate an upward incline; negative values indicate a downward landing slope.
    pub fn surface_slope_angle_rad_at(&self, point: Vec2) -> f32 {
        let s = self.progress_along_ramp(point);
        let len = self.length().max(0.1);
        match self.kind {
            RampKind::Ramp => {
                let incline_len = self.incline_length().min(len);
                let s_incline = (incline_len / len).clamp(0.01, 1.0);

                if s <= s_incline {
                    let t = s / s_incline;
                    let slope_tan = (2.0 * self.height / incline_len.max(0.1)) * t;
                    let max_slope = self.ramp_angle_deg.to_radians().max(0.05);
                    slope_tan.atan().clamp(0.0, max_slope)
                } else {
                    0.0
                }
            }
            RampKind::Whoops => {
                let n = self.whoop_count() as f32;
                let ridge_len = (len / n).max(1.0);
                let f = (s * n).fract();
                let dh_dx =
                    self.height * 0.5 * (std::f32::consts::TAU / ridge_len) * (std::f32::consts::TAU * f).sin();
                let max_slope = self.ramp_angle_deg.max(12.0).to_radians();
                dh_dx.atan().clamp(-max_slope, max_slope)
            }
            RampKind::MegaRamp => {
                let (s_takeoff, s_landing) = self.mega_segment_bounds();
                if s <= s_takeoff {
                    let takeoff_len = (s_takeoff * len).max(0.5);
                    let t = (s / s_takeoff.max(0.01)).clamp(0.0, 1.0);
                    let slope_tan = (2.0 * self.height / takeoff_len) * t;
                    let max_slope = self.ramp_angle_deg.to_radians().max(0.05);
                    slope_tan.atan().clamp(0.0, max_slope)
                } else if s < s_landing {
                    0.0
                } else {
                    let landing_len = ((1.0 - s_landing) * len).max(0.5);
                    let u = ((s - s_landing) / (1.0 - s_landing).max(0.01)).clamp(0.0, 1.0);
                    if u <= 0.15 {
                        0.0
                    } else {
                        let d = (u - 0.15) / 0.85;
                        let catch_h = self.height * 0.92;
                        let dh_dx = -catch_h
                            * 0.5
                            * (std::f32::consts::PI / (landing_len * 0.85))
                            * (std::f32::consts::PI * d).sin();
                        dh_dx.atan().clamp(-0.65, 0.0)
                    }
                }
            }
        }
    }

    /// Minimum elevation required at the landing lip (`s = s_landing`) to clear the gap of a
    /// `MegaRamp` onto the catch deck rather than colliding with the vertical landing wall.
    pub fn mega_clear_height(&self) -> f32 {
        let catch_h = self.height * 0.92;
        (catch_h - 1.15).max(0.05)
    }

    /// Returns the physical concrete wall barrier along the vertical front face of the landing
    /// segment (`s = s_landing`) for a `MegaRamp` at the given base road elevation, or `None`
    /// for other ramp kinds.
    pub fn mega_landing_wall_at_elevation(&self, elevation: f32) -> Option<WallBarrier> {
        if self.kind != RampKind::MegaRamp {
            return None;
        }
        let (_, s_landing) = self.mega_segment_bounds();
        let center = self.shape.center();
        let half = self.half_extents();
        let half_len = half.x.max(0.5);
        let half_wid = half.y.max(0.5);
        let fwd = self.direction.normalize_or_zero();
        let right = Vec2::new(fwd.y, -fwd.x);
        let x_landing_start = -half_len + s_landing * (2.0 * half_len);
        let l0 = center + fwd * x_landing_start - right * half_wid;
        let l3 = center + fwd * x_landing_start + right * half_wid;
        Some(WallBarrier::with_elevation(
            l0,
            l3,
            BarrierType::Concrete,
            elevation,
        ))
    }

    /// Returns the physical concrete wall barrier along the vertical front face of the landing
    /// segment (`s = s_landing`) at base elevation `0.0` for a `MegaRamp`.
    pub fn mega_landing_wall(&self) -> Option<WallBarrier> {
        self.mega_landing_wall_at_elevation(0.0)
    }
}

/// Extension trait allowing Car to interact with and trigger jump ramps.
pub trait JumpRampCarExt {
    /// Attempts to launch the vehicle off the specified jump ramp.
    fn try_trigger_jump_ramp(&mut self, ramp: &JumpRamp) -> bool;

    /// Updates vehicle traversal along the ramp surface, applying grade deceleration,
    /// and triggers takeoff if reaching the takeoff lip at launch speed.
    /// Returns true if a jump launch was triggered.
    fn step_ramp_interaction(&mut self, ramp: &JumpRamp, dt: f32) -> bool;
}

impl JumpRampCarExt for wheelbase::Car {
    fn try_trigger_jump_ramp(&mut self, ramp: &JumpRamp) -> bool {
        let is_on_ramp = ramp.contains(self.state.position);
        let props = wheelbase::JumpRampProperties {
            direction: ramp.direction,
            launch_speed: ramp.launch_speed,
            ramp_angle_deg: ramp.ramp_angle_deg,
            height: ramp.height,
        };
        self.try_trigger_jump(is_on_ramp, &props)
    }

    fn step_ramp_interaction(&mut self, ramp: &JumpRamp, dt: f32) -> bool {
        if !ramp.contains(self.state.position) {
            return false;
        }

        // Handle descending airborne car landing onto an elevated ramp surface (e.g. MegaRamp landing deck or Whoops crest)
        if self.state.is_airborne {
            if ramp.kind == RampKind::MegaRamp {
                let (s_takeoff, s_landing) = ramp.mega_segment_bounds();
                let s = ramp.progress_along_ramp(self.state.position);
                let min_clear_h = ramp.mega_clear_height();

                if s > s_takeoff && s < s_landing {
                    let center = ramp.shape.center();
                    let fwd = ramp.direction.normalize_or_zero();
                    let half_len = ramp.half_extents().x.max(0.5);
                    let x_landing_start = -half_len + s_landing * (2.0 * half_len);
                    let local_x = (self.state.position - center).dot(fwd);
                    let obb = crate::collision::OrientedBox::from_body(self);
                    let max_local_x = obb
                        .corners()
                        .iter()
                        .map(|&c| (c - center).dot(fwd))
                        .fold(f32::NEG_INFINITY, f32::max);

                    if max_local_x >= x_landing_start - 0.02 {
                        let dx = (x_landing_start - local_x).max(0.0);
                        let speed_along = self.state.velocity.dot(fwd);
                        let t_rem = if speed_along > 0.5 { dx / speed_along } else { 1.0 };
                        let elev_at_wall = self.state.elevation
                            + self.state.vertical_velocity * t_rem
                            - 0.5 * 13.5 * t_rem * t_rem;
                        let eff_elev = self
                            .state
                            .elevation
                            .max(self.state.elevation - self.state.vertical_velocity * dt);
                        if eff_elev < min_clear_h || elev_at_wall < min_clear_h {
                            self.state.elevation = 0.0;
                            self.state.ramp_elevation = 0.0;
                            self.state.vertical_velocity = 0.0;
                            self.state.is_airborne = false;
                            self.state.air_time = 0.0;
                            return false;
                        }
                    }
                } else if s >= s_landing {
                    let surf_elev = ramp.surface_elevation_at(self.state.position);
                    let min_landing_h = (surf_elev - 1.15).max(0.05);
                    let eff_elev = self
                        .state
                        .elevation
                        .max(self.state.elevation - self.state.vertical_velocity * dt);
                    if surf_elev > 0.25 && eff_elev < min_landing_h {
                        self.state.elevation = 0.0;
                        self.state.ramp_elevation = 0.0;
                        self.state.vertical_velocity = 0.0;
                        self.state.is_airborne = false;
                        self.state.air_time = 0.0;
                        return false;
                    }
                }
            }

            if self.state.vertical_velocity < 0.0 {
                let surf_elev = ramp.surface_elevation_at(self.state.position);
                if surf_elev > 0.05 && self.state.elevation <= surf_elev {
                    let touchdown_vz = self.state.vertical_velocity.abs();
                    let slope = ramp.surface_slope_angle_rad_at(self.state.position);
                    let cushion = if slope < 0.0 { 0.45 } else { 0.85 };
                    self.state.elevation = 0.0;
                    self.state.ramp_elevation = surf_elev;
                    self.state.vertical_velocity = 0.0;
                    self.state.is_airborne = false;
                    self.state.last_air_time = self.state.air_time;
                    self.state.last_touchdown_speed = touchdown_vz * cushion;
                    self.state.air_time = 0.0;
                    self.state.just_landed = true;
                }
            }
            return false;
        }

        let s = ramp.progress_along_ramp(self.state.position);
        let speed_along = self.state.velocity.dot(ramp.direction);

        match ramp.kind {
            RampKind::Ramp => {
                // Takeoff lip trigger when vehicle reaches the lip (s >= 0.85) at launch speed (> 3.5 m/s)
                if s >= 0.85 && speed_along > 3.5 {
                    let props = wheelbase::JumpRampProperties {
                        direction: ramp.direction,
                        launch_speed: ramp.launch_speed,
                        ramp_angle_deg: ramp.ramp_angle_deg,
                        height: ramp.height,
                    };
                    if self.try_trigger_jump(true, &props) {
                        self.state.ramp_elevation = 0.0;
                        return true;
                    }
                }
            }
            RampKind::Whoops => {
                // Multi-ridge rhythm section: each mogul ridge crests at local phase f ~ 0.42..0.54
                let n = ramp.whoop_count() as f32;
                let f = (s * n).fract();
                if (0.40..=0.55).contains(&f) && speed_along > 6.5 {
                    let props = wheelbase::JumpRampProperties {
                        direction: ramp.direction,
                        launch_speed: ramp.launch_speed.min(6.0),
                        ramp_angle_deg: ramp.ramp_angle_deg.clamp(6.0, 16.0),
                        height: ramp.height,
                    };
                    if self.try_trigger_jump(true, &props) {
                        self.state.ramp_elevation = 0.0;
                        return true;
                    }
                }
            }
            RampKind::MegaRamp => {
                let (s_takeoff, s_landing) = ramp.mega_segment_bounds();
                // Launch at the end of the Take-Off segment (just before the gap)
                let lip_start = (s_takeoff - 0.025).max(0.10);
                if s >= lip_start && s <= s_takeoff + 0.02 && speed_along > 3.5 {
                    let props = wheelbase::JumpRampProperties {
                        direction: ramp.direction,
                        launch_speed: ramp.launch_speed,
                        ramp_angle_deg: ramp.ramp_angle_deg,
                        height: ramp.height,
                    };
                    if self.try_trigger_jump(true, &props) {
                        self.state.ramp_elevation = 0.0;
                        return true;
                    }
                } else if s > s_takeoff && s < s_landing && self.state.ramp_elevation > 0.20 {
                    // Rolled slowly off the take-off lip into the gap
                    self.state.elevation = self.state.ramp_elevation;
                    self.state.ramp_elevation = 0.0;
                    self.state.is_airborne = true;
                    self.state.vertical_velocity = 0.0;
                    return false;
                } else if s >= s_landing {
                    let surf_elev = ramp.surface_elevation_at(self.state.position);
                    let min_landing_h = (surf_elev - 1.15).max(0.05);
                    if surf_elev > 0.25 && self.state.ramp_elevation < min_landing_h {
                        // Grounded in the gap and failed to cross the whole gap: do not snap up onto the landing deck
                        self.state.ramp_elevation = 0.0;
                        return false;
                    }
                }
            }
        }

        // On-ramp continuous surface traversal:
        self.state.ramp_elevation = ramp.surface_elevation_at(self.state.position);
        let slope = ramp.surface_slope_angle_rad_at(self.state.position);
        // Slope gravity (uphill decel when slope > 0, downhill acceleration when slope < 0):
        let grade_decel = 9.81 * slope.sin();
        if speed_along > 0.0 {
            let dv = (grade_decel * dt).min(speed_along);
            self.state.velocity -= ramp.direction * dv;
            self.state.speed = self.state.velocity.length();
        }

        false
    }
}

/// Lazy, clone-transparent spatial grid cache attached to [`TrackGeometry`] (Spec 119).
#[derive(Debug, Default)]
pub struct SpatialGridCache(std::sync::OnceLock<GeometrySpatialGrid>);

impl Clone for SpatialGridCache {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for SpatialGridCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Static 2D uniform spatial grid indexing walls and obstacles into square world cells (Spec 119).
#[derive(Debug)]
pub struct GeometrySpatialGrid {
    counts: (usize, usize, usize, usize),
    first_last_wall: Option<(LineSegment, LineSegment)>,
    pub all_walls: Vec<WallBarrier>,
    pub wall_aabbs: Vec<(Vec2, Vec2)>,
    pub obstacle_aabbs: Vec<(Vec2, Vec2)>,
    pub inner_count: usize,
    pub outer_count: usize,
    origin: Vec2,
    inv_cell_size: Vec2,
    cols: usize,
    rows: usize,
    wall_cell_offsets: Vec<u32>,
    wall_cell_items: Vec<u32>,
    obs_cell_offsets: Vec<u32>,
    obs_cell_items: Vec<u32>,
}

impl GeometrySpatialGrid {
    pub const TARGET_CELL_SIZE: f32 = 24.0;
    pub const MAX_GRID_DIM: usize = 128;

    fn fingerprint(geom: &TrackGeometry) -> ((usize, usize, usize, usize), Option<(LineSegment, LineSegment)>) {
        let obs = geom.all_obstacles_with_scenery();
        let counts = (
            geom.inner_walls.len(),
            geom.outer_walls.len(),
            geom.network_walls.len(),
            obs.len(),
        );
        let first = geom
            .inner_walls
            .first()
            .or_else(|| geom.outer_walls.first())
            .or_else(|| geom.network_walls.first())
            .map(|w| w.segment);
        let last = geom
            .network_walls
            .last()
            .or_else(|| geom.outer_walls.last())
            .or_else(|| geom.inner_walls.last())
            .map(|w| w.segment);
        (counts, first.zip(last))
    }

    #[inline]
    pub fn matches(&self, geom: &TrackGeometry) -> bool {
        let (counts, first_last) = Self::fingerprint(geom);
        self.counts == counts && self.first_last_wall == first_last
    }

    pub fn build(geom: &TrackGeometry) -> Self {
        let (counts, first_last_wall) = Self::fingerprint(geom);
        let inner_count = geom.inner_walls.len();
        let outer_count = geom.outer_walls.len();
        let total_walls = inner_count + outer_count + geom.network_walls.len();

        let mut all_walls = Vec::with_capacity(total_walls);
        all_walls.extend_from_slice(&geom.inner_walls);
        all_walls.extend_from_slice(&geom.outer_walls);
        all_walls.extend_from_slice(&geom.network_walls);

        let mut min_pt = Vec2::splat(f32::INFINITY);
        let mut max_pt = Vec2::splat(f32::NEG_INFINITY);

        let mut wall_aabbs = Vec::with_capacity(total_walls);
        for w in &all_walls {
            let lo = w.segment.start.min(w.segment.end);
            let hi = w.segment.start.max(w.segment.end);
            min_pt = min_pt.min(lo);
            max_pt = max_pt.max(hi);
            wall_aabbs.push((lo, hi));
        }

        let obstacles = geom.all_obstacles_with_scenery();
        let mut obstacle_aabbs = Vec::with_capacity(obstacles.len());
        for obs in obstacles {
            let (lo, hi) = obs.aabb();
            min_pt = min_pt.min(lo);
            max_pt = max_pt.max(hi);
            obstacle_aabbs.push((lo, hi));
        }

        if !min_pt.x.is_finite() || !max_pt.x.is_finite() {
            min_pt = Vec2::splat(-100.0);
            max_pt = Vec2::splat(100.0);
        } else {
            min_pt -= Vec2::splat(1.0);
            max_pt += Vec2::splat(1.0);
        }

        let span = (max_pt - min_pt).max(Vec2::splat(1.0));
        let cols = ((span.x / Self::TARGET_CELL_SIZE).ceil() as usize).clamp(1, Self::MAX_GRID_DIM);
        let rows = ((span.y / Self::TARGET_CELL_SIZE).ceil() as usize).clamp(1, Self::MAX_GRID_DIM);
        let inv_cell_size = Vec2::new((cols as f32) / span.x, (rows as f32) / span.y);
        let num_cells = cols * rows;

        let map_range = |lo: Vec2, hi: Vec2| -> (usize, usize, usize, usize) {
            let c0 = (((lo.x - min_pt.x) * inv_cell_size.x).floor() as isize).clamp(0, (cols - 1) as isize) as usize;
            let c1 = (((hi.x - min_pt.x) * inv_cell_size.x).floor() as isize).clamp(0, (cols - 1) as isize) as usize;
            let r0 = (((lo.y - min_pt.y) * inv_cell_size.y).floor() as isize).clamp(0, (rows - 1) as isize) as usize;
            let r1 = (((hi.y - min_pt.y) * inv_cell_size.y).floor() as isize).clamp(0, (rows - 1) as isize) as usize;
            (c0.min(c1), c0.max(c1), r0.min(r1), r0.max(r1))
        };

        // 2-pass CSR construction for walls
        let mut wall_cell_offsets = vec![0u32; num_cells + 1];
        for &(lo, hi) in &wall_aabbs {
            let (c0, c1, r0, r1) = map_range(lo, hi);
            for r in r0..=r1 {
                let row_base = r * cols;
                for c in c0..=c1 {
                    wall_cell_offsets[row_base + c + 1] += 1;
                }
            }
        }
        for i in 1..=num_cells {
            wall_cell_offsets[i] += wall_cell_offsets[i - 1];
        }
        let mut wall_cell_items = vec![0u32; wall_cell_offsets[num_cells] as usize];
        let mut cursor = wall_cell_offsets[..num_cells].to_vec();
        for (idx, &(lo, hi)) in wall_aabbs.iter().enumerate() {
            let (c0, c1, r0, r1) = map_range(lo, hi);
            for r in r0..=r1 {
                let row_base = r * cols;
                for c in c0..=c1 {
                    let cell = row_base + c;
                    let pos = cursor[cell] as usize;
                    wall_cell_items[pos] = idx as u32;
                    cursor[cell] += 1;
                }
            }
        }

        // 2-pass CSR construction for obstacles
        let mut obs_cell_offsets = vec![0u32; num_cells + 1];
        for &(lo, hi) in &obstacle_aabbs {
            let (c0, c1, r0, r1) = map_range(lo, hi);
            for r in r0..=r1 {
                let row_base = r * cols;
                for c in c0..=c1 {
                    obs_cell_offsets[row_base + c + 1] += 1;
                }
            }
        }
        for i in 1..=num_cells {
            obs_cell_offsets[i] += obs_cell_offsets[i - 1];
        }
        let mut obs_cell_items = vec![0u32; obs_cell_offsets[num_cells] as usize];
        cursor.copy_from_slice(&obs_cell_offsets[..num_cells]);
        for (idx, &(lo, hi)) in obstacle_aabbs.iter().enumerate() {
            let (c0, c1, r0, r1) = map_range(lo, hi);
            for r in r0..=r1 {
                let row_base = r * cols;
                for c in c0..=c1 {
                    let cell = row_base + c;
                    let pos = cursor[cell] as usize;
                    obs_cell_items[pos] = idx as u32;
                    cursor[cell] += 1;
                }
            }
        }

        Self {
            counts,
            first_last_wall,
            all_walls,
            wall_aabbs,
            obstacle_aabbs,
            inner_count,
            outer_count,
            origin: min_pt,
            inv_cell_size,
            cols,
            rows,
            wall_cell_offsets,
            wall_cell_items,
            obs_cell_offsets,
            obs_cell_items,
        }
    }

    #[inline]
    fn cell_range(&self, min: Vec2, max: Vec2) -> Option<(usize, usize, usize, usize)> {
        if self.cols == 0 || self.rows == 0 {
            return None;
        }
        let max_x_cell = (self.cols as isize) - 1;
        let max_y_cell = (self.rows as isize) - 1;
        let rx0 = ((min.x - self.origin.x) * self.inv_cell_size.x).floor() as isize;
        let rx1 = ((max.x - self.origin.x) * self.inv_cell_size.x).floor() as isize;
        let ry0 = ((min.y - self.origin.y) * self.inv_cell_size.y).floor() as isize;
        let ry1 = ((max.y - self.origin.y) * self.inv_cell_size.y).floor() as isize;
        if rx1 < 0 || rx0 > max_x_cell || ry1 < 0 || ry0 > max_y_cell {
            return None;
        }
        let c0 = rx0.clamp(0, max_x_cell) as usize;
        let c1 = rx1.clamp(0, max_x_cell) as usize;
        let r0 = ry0.clamp(0, max_y_cell) as usize;
        let r1 = ry1.clamp(0, max_y_cell) as usize;
        Some((c0.min(c1), c0.max(c1), r0.min(r1), r0.max(r1)))
    }

    /// Calls `f(wall_index, wall)` once for every wall whose AABB overlaps `[min, max]`.
    /// Zero heap allocations.
    #[inline]
    pub fn for_each_wall_in_aabb(&self, min: Vec2, max: Vec2, mut f: impl FnMut(usize, &WallBarrier)) {
        if self.all_walls.is_empty() {
            return;
        }
        let Some((c0, c1, r0, r1)) = self.cell_range(min, max) else {
            return;
        };

        if c0 == c1 && r0 == r1 {
            let cell = r0 * self.cols + c0;
            let start = self.wall_cell_offsets[cell] as usize;
            let end = self.wall_cell_offsets[cell + 1] as usize;
            for &idx_u32 in &self.wall_cell_items[start..end] {
                let idx = idx_u32 as usize;
                let (w_min, w_max) = self.wall_aabbs[idx];
                if w_max.x >= min.x && w_min.x <= max.x && w_max.y >= min.y && w_min.y <= max.y {
                    f(idx, &self.all_walls[idx]);
                }
            }
            return;
        }

        let mut visited = [0u64; 128]; // Covers up to 8,192 walls on stack
        for r in r0..=r1 {
            let row_base = r * self.cols;
            for c in c0..=c1 {
                let cell = row_base + c;
                let start = self.wall_cell_offsets[cell] as usize;
                let end = self.wall_cell_offsets[cell + 1] as usize;
                for &idx_u32 in &self.wall_cell_items[start..end] {
                    let idx = idx_u32 as usize;
                    if idx < 8192 {
                        let word = idx >> 6;
                        let bit = 1u64 << (idx & 63);
                        if (visited[word] & bit) != 0 {
                            continue;
                        }
                        visited[word] |= bit;
                    }
                    let (w_min, w_max) = self.wall_aabbs[idx];
                    if w_max.x >= min.x && w_min.x <= max.x && w_max.y >= min.y && w_min.y <= max.y {
                        f(idx, &self.all_walls[idx]);
                    }
                }
            }
        }
    }

    /// Calls `f(obs_index, obstacle)` once for every obstacle whose AABB overlaps `[min, max]`.
    #[inline]
    pub fn for_each_obstacle_in_aabb<'a>(
        &self,
        obstacles: &'a [Obstacle],
        min: Vec2,
        max: Vec2,
        mut f: impl FnMut(usize, &'a Obstacle),
    ) {
        if obstacles.is_empty() || self.obstacle_aabbs.len() != obstacles.len() {
            return;
        }
        let Some((c0, c1, r0, r1)) = self.cell_range(min, max) else {
            return;
        };

        if c0 == c1 && r0 == r1 {
            let cell = r0 * self.cols + c0;
            let start = self.obs_cell_offsets[cell] as usize;
            let end = self.obs_cell_offsets[cell + 1] as usize;
            for &idx_u32 in &self.obs_cell_items[start..end] {
                let idx = idx_u32 as usize;
                let (o_min, o_max) = self.obstacle_aabbs[idx];
                if o_max.x >= min.x && o_min.x <= max.x && o_max.y >= min.y && o_min.y <= max.y {
                    f(idx, &obstacles[idx]);
                }
            }
            return;
        }

        let mut visited = [0u64; 32]; // Covers up to 2,048 obstacles on stack
        for r in r0..=r1 {
            let row_base = r * self.cols;
            for c in c0..=c1 {
                let cell = row_base + c;
                let start = self.obs_cell_offsets[cell] as usize;
                let end = self.obs_cell_offsets[cell + 1] as usize;
                for &idx_u32 in &self.obs_cell_items[start..end] {
                    let idx = idx_u32 as usize;
                    if idx < 2048 {
                        let word = idx >> 6;
                        let bit = 1u64 << (idx & 63);
                        if (visited[word] & bit) != 0 {
                            continue;
                        }
                        visited[word] |= bit;
                    }
                    let (o_min, o_max) = self.obstacle_aabbs[idx];
                    if o_max.x >= min.x && o_min.x <= max.x && o_max.y >= min.y && o_min.y <= max.y {
                        f(idx, &obstacles[idx]);
                    }
                }
            }
        }
    }
}

/// Collection of boundaries, walls, surface zones, static obstacles, jump ramps, and scenery comprising track geometry.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackGeometry {
    pub inner_walls: Vec<WallBarrier>,
    pub outer_walls: Vec<WallBarrier>,
    pub obstacles: Vec<Obstacle>,
    pub surface_zones: Vec<SurfaceZone>,
    pub jump_ramps: Vec<JumpRamp>,
    pub left_boundary_polyline: Vec<Vec2>,
    pub right_boundary_polyline: Vec<Vec2>,
    #[serde(default)]
    pub grandstands: Vec<Grandstand>,
    #[serde(default)]
    pub trees: Vec<Tree>,
    #[serde(default)]
    pub rocks: Vec<Rock>,
    #[serde(default)]
    pub buildings: Vec<Building>,
    #[serde(default, skip_serializing)]
    pub scenery_obstacles: Vec<Obstacle>,
    /// Walls along network branch segments (the Rallycross joker), rebuilt on load by
    /// `Track::generate_network_walls` and never saved.
    #[serde(default, skip_serializing)]
    pub network_walls: Vec<WallBarrier>,
    /// Suppressed wall segments pruned by network clearance conflicts (Spec 112).
    #[serde(default, skip_serializing)]
    pub wall_suppressions: Vec<WallSuppressionRecord>,
    /// Lazily initialized 2D uniform spatial grid indexing walls and obstacles (Spec 119).
    #[serde(skip)]
    pub spatial_grid: SpatialGridCache,
}

impl TrackGeometry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidates the cached 2D spatial grid so the next query rebuilds it from current geometry.
    #[inline]
    pub fn invalidate_spatial_grid(&mut self) {
        self.spatial_grid = SpatialGridCache::default();
    }

    /// Returns a valid [`GeometrySpatialGrid`] if the cached grid matches current walls and obstacles,
    /// building it on first access.
    #[inline]
    pub fn spatial_grid(&self) -> Option<&GeometrySpatialGrid> {
        let grid = self.spatial_grid.0.get_or_init(|| GeometrySpatialGrid::build(self));
        grid.matches(self).then_some(grid)
    }

    /// Visits every wall whose bounding box overlaps `[min, max]`, using the O(1) spatial grid
    /// when valid and falling back to `all_walls()` if geometry was mutated in place without invalidation.
    #[inline]
    pub fn for_each_wall_in_aabb(&self, min: Vec2, max: Vec2, mut f: impl FnMut(&WallBarrier)) {
        if let Some(grid) = self.spatial_grid() {
            grid.for_each_wall_in_aabb(min, max, |_, w| f(w));
        } else {
            for w in self.all_walls() {
                let lo = w.segment.start.min(w.segment.end);
                let hi = w.segment.start.max(w.segment.end);
                if hi.x >= min.x && lo.x <= max.x && hi.y >= min.y && lo.y <= max.y {
                    f(w);
                }
            }
        }
    }

    /// All barrier segments combined (inner, outer and network branch walls).
    pub fn all_walls(&self) -> impl Iterator<Item = &WallBarrier> {
        self.inner_walls.iter().chain(self.outer_walls.iter()).chain(self.network_walls.iter())
    }

    /// Recomputes and caches aggregated static and scenery obstacles (tree trunks, grandstands, rocks, buildings) (Spec 084).
    pub fn recompute_scenery_obstacles(&mut self) {
        let mut obs = self.obstacles.clone();
        for tree in &self.trees {
            if tree.has_trunk() {
                obs.push(tree.trunk_obstacle());
            }
        }
        for grandstand in &self.grandstands {
            obs.push(grandstand.to_obstacle());
        }
        for rock in &self.rocks {
            obs.push(rock.to_obstacle());
        }
        for building in &self.buildings {
            obs.push(building.to_obstacle());
        }
        self.scenery_obstacles = obs;
        self.invalidate_spatial_grid();
    }

    /// Returns all static obstacles plus solid tree trunk colliders, grandstand collision boxes, rocks, and buildings.
    /// Returns a borrowed slice with zero heap allocations (Spec 084).
    pub fn all_obstacles_with_scenery(&self) -> &[Obstacle] {
        if !self.scenery_obstacles.is_empty() {
            &self.scenery_obstacles
        } else {
            &self.obstacles
        }
    }
}

/// Designated pit service stall along the pit road.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PitBox {
    /// World coordinates of the service box center.
    pub position: Vec2,
    /// Forward orientation vector of the stall.
    pub direction: Vec2,
    /// Stop target radius in meters (typically 2.5 - 3.5m).
    pub stop_radius: f32,
    /// Elevation above ground datum.
    pub elevation: f32,
}

impl PitBox {
    pub fn new(position: Vec2, direction: Vec2, stop_radius: f32, elevation: f32) -> Self {
        Self {
            position,
            direction: direction.normalize_or_zero(),
            stop_radius,
            elevation,
        }
    }

    /// Checks if a world position falls within this pit box's stop radius.
    #[inline]
    pub fn contains_point(&self, p: Vec2) -> bool {
        self.position.distance(p) <= self.stop_radius
    }
}

/// Comprehensive physical pit lane definition for a circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitLane {
    /// Dedicated spline ribbon for the pit road bypass.
    pub spline: TrackSpline,
    /// Road width of the pit road (typically 4.0 - 6.0m).
    pub road_width: f32,
    /// Pit lane maximum regulated speed in m/s (default: 16.67 m/s = 60 km/h).
    pub speed_limit: f32,
    /// Service box stalls for racing participants.
    pub pit_boxes: Vec<PitBox>,
    /// Entrance line segment separating main track from pit lane.
    pub entry_gate: LineSegment,
    /// Exit merge line segment rejoining main track.
    pub exit_gate: LineSegment,
}

impl PitLane {
    pub const DEFAULT_ROAD_SPEED_LIMIT: f32 = 16.67; // 60 km/h
    pub const DEFAULT_OVAL_SPEED_LIMIT: f32 = 22.22; // 80 km/h
    /// Extra width added on the outer (garage) side along the pit stop stall row (Spec 120).
    pub const BAY_WIDTH: f32 = 3.6;
    /// Lateral offset of pit box centers outward from the fast-lane edge `road_width * 0.5` (Spec 120).
    pub const BAY_STALL_OFFSET: f32 = 1.8;
    /// Flat longitudinal padding before the first stall center and after the last stall center (m).
    pub const BAY_PAD_S: f32 = 4.0;
    /// Longitudinal smoothstep taper length at the entry and exit of the service bay (m).
    pub const BAY_TAPER_LEN: f32 = 16.0;

    pub fn new(
        spline: TrackSpline,
        road_width: f32,
        speed_limit: f32,
        pit_boxes: Vec<PitBox>,
        entry_gate: LineSegment,
        exit_gate: LineSegment,
    ) -> Self {
        Self {
            spline,
            road_width,
            speed_limit,
            pit_boxes,
            entry_gate,
            exit_gate,
        }
    }

    /// Returns `(s_bay_start, s_bay_end, sigma)` for the outer service side bay when `pit_boxes` sit off the centerline
    /// (`sigma = +1.0` for left/`+normal`, `-1.0` for right/`-normal`).
    pub fn bay_span_and_side(&self) -> Option<(f32, f32, f32)> {
        if self.pit_boxes.is_empty() || self.spline.samples.len() < 2 {
            return None;
        }
        let mut s_first = f32::INFINITY;
        let mut s_last = f32::NEG_INFINITY;
        let mut normal_dot_sum = 0.0f32;
        for b in &self.pit_boxes {
            let proj = self.spline.project_point(b.position);
            s_first = s_first.min(proj.progress_distance);
            s_last = s_last.max(proj.progress_distance);
            normal_dot_sum += (b.position - proj.closest_point).dot(proj.normal);
        }
        if !s_first.is_finite() || !s_last.is_finite() {
            return None;
        }
        if normal_dot_sum.abs() < 0.1 * (self.pit_boxes.len() as f32) {
            return None;
        }
        let sigma = if normal_dot_sum >= 0.0 { 1.0 } else { -1.0 };
        let total = self.spline.total_length;
        let s_bay_start = (s_first - Self::BAY_PAD_S).clamp(0.0, total);
        let s_bay_end = (s_last + Self::BAY_PAD_S).clamp(s_bay_start, total);
        Some((s_bay_start, s_bay_end, sigma))
    }

    /// Returns `(left_extra, right_extra)` outer bay width expansion in meters at arc length `s` along `self.spline`
    /// (`left_extra` along `+normal`, `right_extra` along `-normal`).
    pub fn bay_extension_at_s(&self, s: f32) -> (f32, f32) {
        let Some((s_bay_start, s_bay_end, sigma)) = self.bay_span_and_side() else {
            return (0.0, 0.0);
        };
        let total = self.spline.total_length;
        let taper_in_start = (s_bay_start - Self::BAY_TAPER_LEN).max(0.0);
        let taper_out_end = (s_bay_end + Self::BAY_TAPER_LEN).min(total);
        let extra = if s < taper_in_start || s > taper_out_end {
            0.0
        } else if s < s_bay_start {
            let span = (s_bay_start - taper_in_start).max(1e-3);
            let t = ((s - taper_in_start) / span).clamp(0.0, 1.0);
            Self::BAY_WIDTH * t * t * (3.0 - 2.0 * t)
        } else if s <= s_bay_end {
            Self::BAY_WIDTH
        } else {
            let span = (taper_out_end - s_bay_end).max(1e-3);
            let t = ((taper_out_end - s) / span).clamp(0.0, 1.0);
            Self::BAY_WIDTH * t * t * (3.0 - 2.0 * t)
        };
        if sigma > 0.0 {
            (extra, 0.0)
        } else {
            (0.0, extra)
        }
    }

    /// Returns `(left_half_width, right_half_width)` at arc length `s` along `self.spline`, including any side-bay
    /// expansion.
    #[inline]
    pub fn half_widths_at_s(&self, s: f32) -> (f32, f32) {
        let half_w = self.road_width * 0.5;
        let (left_extra, right_extra) = self.bay_extension_at_s(s);
        (half_w + left_extra, half_w + right_extra)
    }

    /// Generates `(left_boundary, right_boundary)` polylines along `self.spline` sampled every `sample_step` meters
    /// (or at `self.spline.samples` when `sample_step <= 0.0`), incorporating the outer service side bay.
    pub fn generate_boundaries(&self, sample_step: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        if self.spline.samples.is_empty() {
            return (Vec::new(), Vec::new());
        }
        if sample_step <= 0.0 {
            let left = self
                .spline
                .samples
                .iter()
                .map(|s| {
                    let (lh, _) = self.half_widths_at_s(s.distance);
                    s.point + s.normal * lh
                })
                .collect();
            let right = self
                .spline
                .samples
                .iter()
                .map(|s| {
                    let (_, rh) = self.half_widths_at_s(s.distance);
                    s.point - s.normal * rh
                })
                .collect();
            return (left, right);
        }
        let step = sample_step.max(0.25);
        let total = self.spline.total_length;
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut d = 0.0f32;
        while d < total {
            let s = self.spline.sample_at_distance(d);
            let (lh, rh) = self.half_widths_at_s(d);
            left.push(s.point + s.normal * lh);
            right.push(s.point - s.normal * rh);
            d += step;
        }
        let s_end = self.spline.sample_at_distance(total);
        let (lh, rh) = self.half_widths_at_s(total);
        left.push(s_end.point + s_end.normal * lh);
        right.push(s_end.point - s_end.normal * rh);
        (left, right)
    }

    /// Computes untangled `(left_road, right_road, left_curb, right_curb)` boundary slices matching
    /// `self.spline.samples.len()`, incorporating the outer service side bay (Spec 120).
    pub fn untangled_boundaries(
        &self,
        curb_extra_width: f32,
    ) -> (Vec<Vec2>, Vec<Vec2>, Vec<Vec2>, Vec<Vec2>) {
        let n = self.spline.samples.len();
        if n == 0 {
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        }
        let mut left_road: Vec<Vec2> = self
            .spline
            .samples
            .iter()
            .map(|s| {
                let (lh, _) = self.half_widths_at_s(s.distance);
                s.point + s.normal * lh
            })
            .collect();
        let mut right_road: Vec<Vec2> = self
            .spline
            .samples
            .iter()
            .map(|s| {
                let (_, rh) = self.half_widths_at_s(s.distance);
                s.point - s.normal * rh
            })
            .collect();
        let mut left_curb: Vec<Vec2> = self
            .spline
            .samples
            .iter()
            .map(|s| {
                let (lh, _) = self.half_widths_at_s(s.distance);
                let extra = if s.left_curb { curb_extra_width } else { 0.0 };
                s.point + s.normal * (lh + extra)
            })
            .collect();
        let mut right_curb: Vec<Vec2> = self
            .spline
            .samples
            .iter()
            .map(|s| {
                let (_, rh) = self.half_widths_at_s(s.distance);
                let extra = if s.right_curb { curb_extra_width } else { 0.0 };
                s.point - s.normal * (rh + extra)
            })
            .collect();

        super::spline::untangle_road_and_curb(&mut left_road, &mut left_curb, self.spline.closed);
        super::spline::untangle_road_and_curb(&mut right_road, &mut right_curb, self.spline.closed);

        (left_road, right_road, left_curb, right_curb)
    }
}

/// Precomputed chevron stripe line in the pit lane entrance gore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitLaneChevron {
    pub apex: Vec2,
    pub pt_track: Vec2,
    pub pt_pit: Vec2,
}

/// Precomputed paved exit merge quad.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitLaneExitQuad {
    pub quad: [Vec2; 4],
    pub has_dashed_line: bool,
    pub line_start: Vec2,
    pub line_end: Vec2,
}

/// Precomputed geometry for pit lane entrance/exit paved junctions and gore markings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PitLaneJunctionData {
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub entrance_quads: Vec<[Vec2; 4]>,
    pub has_gore: bool,
    pub p_apex: Vec2,
    pub track_edge_apex: Vec2,
    pub pit_inner_apex: Vec2,
    pub te_start: Vec2,
    pub pe_start: Vec2,
    pub chevrons: Vec<PitLaneChevron>,
    pub exit_quads: Vec<PitLaneExitQuad>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_segment_projection_and_distance() {
        let seg = LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0));
        assert_eq!(seg.closest_point(Vec2::new(5.0, 3.0)), Vec2::new(5.0, 0.0));
        assert_eq!(seg.closest_point(Vec2::new(-2.0, 3.0)), Vec2::new(0.0, 0.0));
        assert_eq!(seg.closest_point(Vec2::new(12.0, -1.0)), Vec2::new(10.0, 0.0));
        assert_eq!(seg.distance_to_point(Vec2::new(5.0, 4.0)), 4.0);
    }

    #[test]
    fn test_line_segment_intersection() {
        let seg1 = LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let seg2 = LineSegment::new(Vec2::new(0.0, 10.0), Vec2::new(10.0, 0.0));
        let hit = seg1.intersect_segment(&seg2).expect("Must intersect");
        assert!((hit.x - 5.0).abs() < 1e-4 && (hit.y - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_segment_intersection() {
        let seg = LineSegment::new(Vec2::new(10.0, -5.0), Vec2::new(10.0, 5.0));
        let hit = seg.intersect_ray(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 50.0);
        assert!(hit.is_some());
        let (dist, norm) = hit.unwrap();
        assert!((dist - 10.0).abs() < 1e-4);
        assert_eq!(norm, Vec2::new(-1.0, 0.0));
    }

    #[test]
    fn test_surface_shapes() {
        let circle = SurfaceShape::Circle {
            center: Vec2::new(5.0, 5.0),
            radius: 3.0,
        };
        assert!(circle.contains(Vec2::new(5.0, 6.0)));
        assert!(!circle.contains(Vec2::new(10.0, 10.0)));

        let poly = SurfaceShape::Polygon {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 10.0),
                Vec2::new(0.0, 10.0),
            ],
        };
        assert!(poly.contains(Vec2::new(5.0, 5.0)));
        assert!(!poly.contains(Vec2::new(15.0, 5.0)));
    }

    #[test]
    fn test_jump_ramp_angle_and_size_manipulation() {
        let mut ramp = JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(20.0, 30.0),
                half_extents: Vec2::new(6.0, 4.0),
                angle: 0.0,
            },
            Vec2::new(1.0, 0.0),
            24.0,
            15.0,
            1.8,
            "Test Ramp",
        );

        assert_eq!(ramp.angle(), 0.0);
        assert_eq!(ramp.length(), 12.0);
        assert_eq!(ramp.width(), 8.0);
        assert_eq!(ramp.direction, Vec2::new(1.0, 0.0));

        // Rotate by PI/2 (90 deg)
        ramp.rotate(std::f32::consts::FRAC_PI_2);
        assert!((ramp.angle() - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
        assert!((ramp.direction.x - 0.0).abs() < 1e-4);
        assert!((ramp.direction.y - 1.0).abs() < 1e-4);

        // Adjust size
        ramp.adjust_size(4.0, -2.0);
        assert_eq!(ramp.length(), 16.0);
        assert_eq!(ramp.width(), 6.0);

        // Scale size
        ramp.scale_size(1.5);
        assert_eq!(ramp.length(), 24.0);
        assert_eq!(ramp.width(), 9.0);
    }

    #[test]
    fn test_jump_ramp_surface_and_flat_portion_fit() {
        let mut ramp = JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(10.0, 10.0),
                half_extents: Vec2::new(10.0, 4.0),
                angle: 0.0,
            },
            Vec2::new(1.0, 0.0),
            20.0,
            10.0, // 10 degrees pitch
            2.0,  // 2 meters height
            "Surface Ramp",
        )
        .with_surface(SurfaceType::Dirt);

        assert_eq!(ramp.surface, SurfaceType::Dirt);
        assert_eq!(ramp.length(), 20.0);
        assert_eq!(ramp.width(), 8.0);

        // incline length: 2.0 / tan(10 deg) = 2.0 / 0.17632698 = 11.342563
        let inc_len = ramp.incline_length();
        assert!((inc_len - 11.34).abs() < 0.1);
        let flat_len = ramp.flat_length();
        assert!((flat_len - (20.0 - inc_len)).abs() < 1e-4);
        assert!(flat_len > 0.0);

        // Auto-fit pitch: atan(2.0 / 20.0) = atan(0.1) = 5.71 deg
        let fitted_pitch = ramp.fitted_pitch_deg();
        assert!((fitted_pitch - 5.71).abs() < 0.1);

        ramp.ramp_angle_deg = fitted_pitch;
        assert!(ramp.flat_length() < 0.01);

        // Auto-fit height: len * tan(10 deg) = 20.0 * 0.17632698 = 3.5265m
        ramp.ramp_angle_deg = 10.0;
        let fitted_h = ramp.fitted_height();
        assert!((fitted_h - 3.53).abs() < 0.05);
        ramp.height = fitted_h;
        assert!(ramp.flat_length() < 0.01);

        // Setters
        ramp.set_length(30.0);
        assert_eq!(ramp.length(), 30.0);
        ramp.set_width(12.0);
        assert_eq!(ramp.width(), 12.0);

        // JSON backwards compatibility test: missing "surface" field defaults to Asphalt
        let json_data = r#"{
            "id": 5,
            "shape": {
                "OrientedBox": {
                    "center": [0.0, 0.0],
                    "half_extents": [5.0, 2.0],
                    "angle": 0.0
                }
            },
            "direction": [1.0, 0.0],
            "launch_speed": 22.0,
            "ramp_angle_deg": 15.0,
            "height": 1.5,
            "name": "Legacy Ramp"
        }"#;

        let deserialized: JumpRamp = serde_json::from_str(json_data).expect("Should deserialize legacy ramp without surface field");
        assert_eq!(deserialized.surface, SurfaceType::Asphalt);
        assert_eq!(deserialized.name, "Legacy Ramp");
    }

    #[test]
    fn test_barrier_type_virtual_properties() {
        let b_virt = BarrierType::Virtual;
        assert!(!b_virt.is_physical());
        assert_eq!(b_virt.name(), "Virtual Boundary");
        assert_eq!(b_virt.default_restitution(), 0.0);
        assert_eq!(b_virt.default_friction(), 0.0);
        assert_eq!(b_virt.scraping_deceleration(), 0.0);
        assert_eq!(b_virt.snag_torque_factor(), 0.0);
        assert_eq!(b_virt.energy_absorption_factor(), 1.0);

        let wall = WallBarrier::new(Vec2::ZERO, Vec2::new(10.0, 0.0), BarrierType::Virtual);
        assert!(!wall.is_physical());

        let wall_conc = WallBarrier::new(Vec2::ZERO, Vec2::new(10.0, 0.0), BarrierType::Concrete);
        assert!(wall_conc.is_physical());
    }

    #[test]
    fn test_pit_lane_geometry() {
        let p1 = Vec2::new(0.0, 0.0);
        let p2 = Vec2::new(100.0, 0.0);
        let spline = TrackSpline::from_points(&[p1, p2], 5.0, false);
        let pit_box = PitBox::new(Vec2::new(50.0, 5.0), Vec2::new(1.0, 0.0), 3.0, 0.0);
        assert!(pit_box.contains_point(Vec2::new(51.0, 5.0)));
        assert!(!pit_box.contains_point(Vec2::new(60.0, 5.0)));

        let entry = LineSegment::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 10.0));
        let exit = LineSegment::new(Vec2::new(100.0, 0.0), Vec2::new(100.0, 10.0));
        let pit_lane = PitLane::new(spline, 5.0, PitLane::DEFAULT_ROAD_SPEED_LIMIT, vec![pit_box], entry, exit);

        assert_eq!(pit_lane.road_width, 5.0);
        assert!((pit_lane.speed_limit - 16.67).abs() < 1e-2);
        assert_eq!(pit_lane.pit_boxes.len(), 1);

        // Serialization roundtrip
        let serialized = serde_json::to_string(&pit_lane).expect("Serialize pit lane");
        let deserialized: PitLane = serde_json::from_str(&serialized).expect("Deserialize pit lane");
        assert_eq!(pit_lane, deserialized);
    }

    #[test]
    fn test_whoops_moguls_and_mega_ramp_profiles_and_vehicle_interaction() {
        use wheelbase::{Car, CarConfig};

        // 1. Whoops Moguls: 30m long, 5m spacing => 6 ridges
        let whoops = JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(15.0, 0.0),
                half_extents: Vec2::new(15.0, 6.0),
                angle: 0.0,
            },
            Vec2::X,
            4.0,
            14.0,
            0.9,
            "Whoops Section",
        )
        .with_kind(RampKind::Whoops)
        .with_whoop_spacing(5.0);

        assert_eq!(whoops.whoop_count(), 6);
        // Crest 0 is at s = 0.5 / 6.0 -> x = 2.5m
        let elev_crest = whoops.surface_elevation_at(Vec2::new(2.5, 0.0));
        let elev_trough = whoops.surface_elevation_at(Vec2::new(5.0, 0.0));
        assert!((elev_crest - 0.9).abs() < 0.05);
        assert!(elev_trough < 0.05);
        // Upslope has positive slope, downslope has negative slope
        assert!(whoops.surface_slope_angle_rad_at(Vec2::new(1.25, 0.0)) > 0.05);
        assert!(whoops.surface_slope_angle_rad_at(Vec2::new(3.75, 0.0)) < -0.05);

        // 2. Mega Ramp: 40m total length (x = 0..40), 12m gap
        let mega = JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(20.0, 0.0),
                half_extents: Vec2::new(20.0, 6.0),
                angle: 0.0,
            },
            Vec2::X,
            45.0,
            30.0,
            4.0,
            "Mega Ramp",
        )
        .with_kind(RampKind::MegaRamp)
        .with_gap_length(12.0);

        let (s_takeoff, s_landing) = mega.mega_segment_bounds();
        assert!((s_takeoff - 0.322).abs() < 0.02);
        assert!((s_landing - 0.622).abs() < 0.02);

        // Take-off lip near x = 12.7m is near peak height (4.0m)
        assert!(mega.surface_elevation_at(Vec2::new(12.7, 0.0)) > 3.7);
        // Open gap at x = 20.0m has 0.0m elevation
        assert_eq!(mega.surface_elevation_at(Vec2::new(20.0, 0.0)), 0.0);
        // Landing catch deck at x = 25.5m is elevated and slopes downward
        assert!(mega.surface_elevation_at(Vec2::new(25.5, 0.0)) > 3.5);
        assert!(mega.surface_slope_angle_rad_at(Vec2::new(33.0, 0.0)) < -0.05);

        // Vehicle interaction: car reaching take-off lip at speed launches into the air
        let mut car = Car::new(CarConfig::default());
        car.state.position = Vec2::new(12.6, 0.0);
        car.state.velocity = Vec2::new(25.0, 0.0);
        assert!(car.step_ramp_interaction(&mega, 1.0 / 60.0));
        assert!(car.state.is_airborne);
        assert!(car.state.vertical_velocity > 5.0);

        // Descending airborne car reaching the landing segment surface is caught smoothly on the downslope
        let landing_elev = mega.surface_elevation_at(Vec2::new(30.0, 0.0));
        car.state.position = Vec2::new(30.0, 0.0);
        car.state.elevation = landing_elev - 0.02;
        car.state.vertical_velocity = -8.0;
        car.state.is_airborne = true;
        car.step_ramp_interaction(&mega, 1.0 / 60.0);
        assert!(!car.state.is_airborne);
        assert!(car.state.just_landed);
        assert_eq!(car.state.vertical_velocity, 0.0);
        assert!((car.state.ramp_elevation - landing_elev).abs() < 0.05);
    }

    #[test]
    fn test_mega_ramp_failed_gap_behaves_like_wall_collision() {
        use crate::collision::resolve_track_wall_collisions_into;
        use wheelbase::{Car, CarConfig, CarControls, SurfaceType};

        // Mega Ramp from x = 0..40, center at (20, 0), 12m gap
        let mega = JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(20.0, 0.0),
                half_extents: Vec2::new(20.0, 6.0),
                angle: 0.0,
            },
            Vec2::X,
            45.0,
            30.0,
            4.0,
            "Mega Ramp",
        )
        .with_kind(RampKind::MegaRamp)
        .with_gap_length(12.0);

        let (_, s_landing) = mega.mega_segment_bounds();
        let x_landing_start = s_landing * 40.0;

        let mut geom = TrackGeometry::default();
        geom.jump_ramps.push(mega.clone());

        // Case 1: Car lands short in the gap and drives into the front face of the landing ramp
        let mut car_short = Car::new(CarConfig::sports_car())
            .with_pose(Vec2::new(x_landing_start - 1.5, 0.0), 0.0);
        car_short.state.velocity = Vec2::new(15.0, 0.0);
        car_short.state.speed = 15.0;

        car_short.step_ramp_interaction(&mega, 1.0 / 60.0);
        assert_eq!(
            car_short.state.ramp_elevation, 0.0,
            "Car in gap must not snap onto elevated landing deck"
        );

        let mut events = Vec::new();
        resolve_track_wall_collisions_into(&mut car_short, &geom, &mut events);
        assert_eq!(
            events.len(),
            1,
            "Car failing to cross MegaRamp gap must trigger a WallCollisionEvent"
        );
        assert_eq!(events[0].barrier_type, BarrierType::Concrete);
        assert!(
            events[0].normal.x < -0.95,
            "Wall normal must oppose forward entry into landing ramp, got {:?}",
            events[0].normal
        );
        assert!(
            car_short.state.velocity.x < 0.0,
            "Car must bounce back off the landing ramp wall, got vx = {}",
            car_short.state.velocity.x
        );
        assert!(
            car_short.state.position.x < x_landing_start - 2.0,
            "Car must be pushed out before the landing ramp start, got x = {}",
            car_short.state.position.x
        );

        // Case 2: Car launches off the take-off lip with insufficient speed (10 m/s) to cross the 12m gap
        let mut car_slow_jump = Car::new(CarConfig::sports_car())
            .with_pose(Vec2::new(12.6, 0.0), 0.0);
        car_slow_jump.state.velocity = Vec2::new(10.0, 0.0);
        car_slow_jump.state.speed = 10.0;
        let dt = 1.0 / 60.0;
        let ctrl = CarControls {
            throttle: 0.5,
            ..Default::default()
        };
        let surfaces = [SurfaceType::Asphalt; 4];

        let mut hit_landing_wall = false;
        for _ in 0..180 {
            car_slow_jump.step_per_wheel(&ctrl, surfaces, dt);
            car_slow_jump.step_ramp_interaction(&mega, dt);
            events.clear();
            resolve_track_wall_collisions_into(&mut car_slow_jump, &geom, &mut events);
            if !events.is_empty() {
                hit_landing_wall = true;
                assert_eq!(events[0].barrier_type, BarrierType::Concrete);
                assert!(events[0].normal.x < -0.95);
                break;
            }
            assert!(
                car_slow_jump.state.ramp_elevation < 0.1,
                "Slow-jumping car must never climb onto the landing deck"
            );
        }
        assert!(
            hit_landing_wall,
            "Car launching with insufficient speed to clear MegaRamp gap must hit the landing wall"
        );
        assert!(car_slow_jump.state.position.x < x_landing_start);
        assert_eq!(car_slow_jump.state.ramp_elevation, 0.0);

        // Case 3: FIA Rallycross MegaRamp (18m long, 1.3m high, 6.2 deg, 6m gap):
        // slow launch (7.5 m/s) fails the gap and hits the landing wall; normal speed (16 m/s) clears it.
        let rx_mega = JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(9.0, 0.0),
                half_extents: Vec2::new(9.0, 5.4),
                angle: 0.0,
            },
            Vec2::X,
            9.0,
            6.2,
            1.3,
            "FIA RX Mega Jump",
        )
        .with_kind(RampKind::MegaRamp)
        .with_gap_length(6.0);
        let (rx_s_takeoff, rx_s_landing) = rx_mega.mega_segment_bounds();
        let rx_x_takeoff = rx_s_takeoff * 18.0;
        let rx_x_landing = rx_s_landing * 18.0;
        let mut rx_geom = TrackGeometry::default();
        rx_geom.jump_ramps.push(rx_mega.clone());

        let mut rx_slow = Car::new(CarConfig::rally_car())
            .with_pose(Vec2::new(rx_x_takeoff - 0.2, 0.0), 0.0);
        rx_slow.state.velocity = Vec2::new(7.5, 0.0);
        rx_slow.state.speed = 7.5;
        let mut rx_hit_wall = false;
        for _ in 0..120 {
            rx_slow.step_per_wheel(&ctrl, [SurfaceType::Dirt; 4], dt);
            rx_slow.step_ramp_interaction(&rx_mega, dt);
            events.clear();
            resolve_track_wall_collisions_into(&mut rx_slow, &rx_geom, &mut events);
            if !events.is_empty() {
                rx_hit_wall = true;
                break;
            }
        }
        assert!(rx_hit_wall, "Slow RX jump failing the 6m gap must hit the landing wall");
        assert!(rx_slow.state.position.x < rx_x_landing);

        let mut rx_fast = Car::new(CarConfig::rally_car())
            .with_pose(Vec2::new(rx_x_takeoff - 0.2, 0.0), 0.0);
        rx_fast.state.velocity = Vec2::new(16.0, 0.0);
        rx_fast.state.speed = 16.0;
        let mut rx_cleared = false;
        for _ in 0..120 {
            rx_fast.step_per_wheel(&ctrl, [SurfaceType::Dirt; 4], dt);
            rx_fast.step_ramp_interaction(&rx_mega, dt);
            events.clear();
            resolve_track_wall_collisions_into(&mut rx_fast, &rx_geom, &mut events);
            assert!(events.is_empty(), "Fast RX jump clearing the gap must not hit the landing wall");
            if rx_fast.state.position.x > rx_x_landing + 1.0 && rx_fast.state.ramp_elevation > 0.5 {
                rx_cleared = true;
                break;
            }
        }
        assert!(rx_cleared, "Fast RX jump must clear the gap and land on the elevated landing deck");
    }
}
