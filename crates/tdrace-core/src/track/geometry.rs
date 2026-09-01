use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::physics::surface::SurfaceType;

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
}

/// Physical classification of track barriers and walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarrierType {
    /// Rigid concrete wall (high restitution, moderate friction).
    Concrete,
    /// Armco steel barrier (medium restitution, steel friction).
    Armco,
    /// Energy-absorbing tire stack barrier (low restitution, high friction).
    TireWall,
    /// Low track-edge curb wall.
    CurbWall,
}

impl BarrierType {
    pub const fn default_restitution(self) -> f32 {
        match self {
            Self::Concrete => 0.65,
            Self::Armco => 0.45,
            Self::TireWall => 0.20,
            Self::CurbWall => 0.35,
        }
    }

    pub const fn default_friction(self) -> f32 {
        match self {
            Self::Concrete => 0.35,
            Self::Armco => 0.40,
            Self::TireWall => 0.70,
            Self::CurbWall => 0.50,
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
}

impl WallBarrier {
    pub fn new(start: Vec2, end: Vec2, barrier_type: BarrierType) -> Self {
        Self {
            segment: LineSegment::new(start, end),
            restitution: barrier_type.default_restitution(),
            friction: barrier_type.default_friction(),
            barrier_type,
            elevation: 0.0,
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
        }
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
        }
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
            Self::Polygon { vertices } => {
                if vertices.len() < 3 {
                    return false;
                }
                // Standard ray casting point-in-polygon test
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

/// A track jump ramp launching vehicles into the air.
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
        }
    }

    pub fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = surface;
        self
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
}

/// Collection of boundaries, walls, surface zones, static obstacles, and jump ramps comprising track geometry.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackGeometry {
    pub inner_walls: Vec<WallBarrier>,
    pub outer_walls: Vec<WallBarrier>,
    pub obstacles: Vec<Obstacle>,
    pub surface_zones: Vec<SurfaceZone>,
    pub jump_ramps: Vec<JumpRamp>,
    pub left_boundary_polyline: Vec<Vec2>,
    pub right_boundary_polyline: Vec<Vec2>,
}

impl TrackGeometry {
    pub fn new() -> Self {
        Self::default()
    }

    /// All barrier segments combined (inner and outer).
    pub fn all_walls(&self) -> impl Iterator<Item = &WallBarrier> {
        self.inner_walls.iter().chain(self.outer_walls.iter())
    }
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
}

