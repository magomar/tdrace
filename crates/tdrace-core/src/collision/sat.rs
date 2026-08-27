use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::physics::car::Car;
use crate::track::geometry::LineSegment;

/// 2D Oriented Bounding Box (OBB) representing a car chassis or rectangular obstacle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrientedBox {
    pub center: Vec2,
    pub half_extents: Vec2, // x = half_length, y = half_width
    pub angle: f32,         // orientation in radians
}

impl OrientedBox {
    pub const fn new(center: Vec2, half_extents: Vec2, angle: f32) -> Self {
        Self {
            center,
            half_extents,
            angle,
        }
    }

    /// Constructs an OBB bounding volume enclosing a car instance.
    pub fn from_car(car: &Car) -> Self {
        let fwd = car.forward_vector();
        let total_length = car.config.cg_to_front + car.config.cg_to_rear;
        // Geometric center offset relative to CG
        let cg_offset = (car.config.cg_to_front - car.config.cg_to_rear) * 0.5;
        let geometric_center = car.state.position + fwd * cg_offset;

        let half_length = total_length * 0.5 + 0.15; // bumper margin
        let half_width = car.config.track_width * 0.5 + 0.15; // fender margin

        Self {
            center: geometric_center,
            half_extents: Vec2::new(half_length, half_width),
            angle: car.state.angle,
        }
    }

    /// Unit vectors for local X (forward) and local Y (lateral).
    #[inline]
    pub fn axes(&self) -> [Vec2; 2] {
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();
        [
            Vec2::new(cos_a, sin_a),   // Local X (forward)
            Vec2::new(-sin_a, cos_a),  // Local Y (left)
        ]
    }

    /// Forward unit vector.
    #[inline]
    pub fn forward(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    /// Left unit normal vector.
    #[inline]
    pub fn left(&self) -> Vec2 {
        Vec2::new(-self.angle.sin(), self.angle.cos())
    }

    /// Computes the 4 world-space corners [FL, RL, RR, FR].
    pub fn corners(&self) -> [Vec2; 4] {
        let fwd = self.forward() * self.half_extents.x;
        let left = self.left() * self.half_extents.y;

        [
            self.center + fwd + left, // FL
            self.center - fwd + left, // RL
            self.center - fwd - left, // RR
            self.center + fwd - left, // FR
        ]
    }

    /// Returns the 4 boundary line segments of the box.
    pub fn edges(&self) -> [LineSegment; 4] {
        let c = self.corners();
        [
            LineSegment::new(c[0], c[1]), // Left side
            LineSegment::new(c[1], c[2]), // Rear side
            LineSegment::new(c[2], c[3]), // Right side
            LineSegment::new(c[3], c[0]), // Front side
        ]
    }

    /// Projects all 4 corners onto an arbitrary 1D axis, returning (min, max).
    #[inline]
    pub fn project_onto_axis(&self, axis: Vec2) -> (f32, f32) {
        let corners = self.corners();
        let mut min = corners[0].dot(axis);
        let mut max = min;
        for &c in &corners[1..] {
            let p = c.dot(axis);
            if p < min {
                min = p;
            }
            if p > max {
                max = p;
            }
        }
        (min, max)
    }

    /// Tests if a 2D point lies inside the oriented box.
    pub fn contains_point(&self, point: Vec2) -> bool {
        let d = point - self.center;
        let fwd = self.forward();
        let left = self.left();
        let local_x = d.dot(fwd).abs();
        let local_y = d.dot(left).abs();
        local_x <= self.half_extents.x && local_y <= self.half_extents.y
    }
}

/// Contact manifold describing collision contact points, normal, and penetration depth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactManifold {
    /// True if an overlap / collision occurs.
    pub colliding: bool,
    /// Penetration overlap depth (meters).
    pub penetration: f32,
    /// Normal pointing from Body A to Body B.
    pub normal: Vec2,
    /// Exact contact points on the contact interface.
    pub contact_points: Vec<Vec2>,
}

impl ContactManifold {
    pub const fn empty() -> Self {
        Self {
            colliding: false,
            penetration: 0.0,
            normal: Vec2::ZERO,
            contact_points: Vec::new(),
        }
    }
}

/// Computes Separating Axis Theorem (SAT) collision test and contact manifold between two OBBs.
pub fn collide_obb_obb(box_a: &OrientedBox, box_b: &OrientedBox) -> Option<ContactManifold> {
    let axes_a = box_a.axes();
    let axes_b = box_b.axes();

    let candidate_axes = [axes_a[0], axes_a[1], axes_b[0], axes_b[1]];

    let mut min_overlap = f32::INFINITY;
    let mut collision_normal = Vec2::ZERO;

    for &axis in &candidate_axes {
        let (min_a, max_a) = box_a.project_onto_axis(axis);
        let (min_b, max_b) = box_b.project_onto_axis(axis);

        if min_a >= max_b || min_b >= max_a {
            return None; // Separating axis found: no collision
        }

        let overlap = (max_a.min(max_b) - min_a.max(min_b)).max(0.0);
        if overlap < min_overlap {
            min_overlap = overlap;
            collision_normal = axis;
        }
    }

    // Ensure normal points from A to B
    let d_centers = box_b.center - box_a.center;
    if d_centers.dot(collision_normal) < 0.0 {
        collision_normal = -collision_normal;
    }

    // Find contact points via polygon clipping
    let contact_points = find_obb_contact_points(box_a, box_b, collision_normal);

    Some(ContactManifold {
        colliding: true,
        penetration: min_overlap,
        normal: collision_normal,
        contact_points,
    })
}

/// Finds contact points by clipping the incident face against reference face side planes.
fn find_obb_contact_points(box_a: &OrientedBox, box_b: &OrientedBox, _normal: Vec2) -> Vec<Vec2> {
    // Determine reference box and incident box
    let corners_a = box_a.corners();
    let corners_b = box_b.corners();

    let mut contact_points = Vec::new();

    // Check which corners of B penetrate A along the normal
    for &cb in &corners_b {
        if box_a.contains_point(cb) {
            contact_points.push(cb);
        }
    }

    // Check which corners of A penetrate B
    for &ca in &corners_a {
        if box_b.contains_point(ca) {
            contact_points.push(ca);
        }
    }

    // If no corners strictly inside (e.g. edge-edge grazing), use midpoints
    if contact_points.is_empty() {
        let edges_a = box_a.edges();
        let edges_b = box_b.edges();
        for ea in &edges_a {
            for eb in &edges_b {
                if let Some(pt) = ea.intersect_segment(eb) {
                    contact_points.push(pt);
                }
            }
        }
    }

    // Fallback: contact point along center line
    if contact_points.is_empty() {
        let mid = (box_a.center + box_b.center) * 0.5;
        contact_points.push(mid);
    }

    contact_points
}

/// Tests collision between an OBB and a circle.
pub fn collide_obb_circle(
    obb: &OrientedBox,
    circle_center: Vec2,
    circle_radius: f32,
) -> Option<ContactManifold> {
    let d = circle_center - obb.center;
    let fwd = obb.forward();
    let left = obb.left();

    // Project circle into OBB local coordinates
    let local_x = d.dot(fwd);
    let local_y = d.dot(left);

    let clamped_x = local_x.clamp(-obb.half_extents.x, obb.half_extents.x);
    let clamped_y = local_y.clamp(-obb.half_extents.y, obb.half_extents.y);

    let closest_local = Vec2::new(clamped_x, clamped_y);
    let diff_local = Vec2::new(local_x - clamped_x, local_y - clamped_y);
    let dist_sq = diff_local.length_squared();

    if dist_sq <= circle_radius * circle_radius {
        let dist = dist_sq.sqrt();
        let normal_local = if dist > 1e-5 {
            diff_local / dist
        } else {
            Vec2::X
        };

        let normal_world = fwd * normal_local.x + left * normal_local.y;
        let penetration = circle_radius - dist;
        let contact_world = obb.center + fwd * closest_local.x + left * closest_local.y;

        Some(ContactManifold {
            colliding: true,
            penetration,
            normal: normal_world,
            contact_points: vec![contact_world],
        })
    } else {
        None
    }
}

fn project_points_on_axis(points: &[Vec2], axis: Vec2) -> (f32, f32) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    let mut min = points[0].dot(axis);
    let mut max = min;
    for &p in &points[1..] {
        let proj = p.dot(axis);
        if proj < min {
            min = proj;
        }
        if proj > max {
            max = proj;
        }
    }
    (min, max)
}

/// Tests collision between an OBB and a 2D polygon using Separating Axis Theorem (SAT).
pub fn collide_obb_polygon(
    obb: &OrientedBox,
    vertices: &[Vec2],
) -> Option<ContactManifold> {
    let n = vertices.len();
    if n < 3 {
        return None;
    }

    let obb_corners = obb.corners();
    let mut min_penetration = f32::MAX;
    let mut best_axis = Vec2::ZERO;

    // 1. Test OBB axes (2 axes)
    let obb_axes = [obb.forward(), obb.left()];
    for axis in obb_axes {
        let (min_a, max_a) = project_points_on_axis(&obb_corners, axis);
        let (min_b, max_b) = project_points_on_axis(vertices, axis);

        if max_a < min_b || max_b < min_a {
            return None; // Separating axis found
        }

        let overlap = (max_a.min(max_b) - min_a.max(min_b)).max(0.0);
        if overlap < min_penetration {
            min_penetration = overlap;
            best_axis = axis;
        }
    }

    // 2. Test Polygon edge normal axes
    for i in 0..n {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % n];
        let edge = v2 - v1;
        let axis = Vec2::new(-edge.y, edge.x).normalize_or_zero();
        if axis.length_squared() < 1e-4 {
            continue;
        }

        let (min_a, max_a) = project_points_on_axis(&obb_corners, axis);
        let (min_b, max_b) = project_points_on_axis(vertices, axis);

        if max_a < min_b || max_b < min_a {
            return None; // Separating axis found
        }

        let overlap = (max_a.min(max_b) - min_a.max(min_b)).max(0.0);
        if overlap < min_penetration {
            min_penetration = overlap;
            best_axis = axis;
        }
    }

    if min_penetration <= 1e-5 {
        return None;
    }

    // Centroid of polygon
    let centroid: Vec2 = vertices.iter().copied().sum::<Vec2>() / (n as f32);
    // Orient normal pointing from OBB towards polygon
    if (centroid - obb.center).dot(best_axis) < 0.0 {
        best_axis = -best_axis;
    }

    // Contact point: vertex of OBB deepest along normal
    let mut deepest_pt = obb_corners[0];
    let mut max_depth = f32::MIN;
    for &pt in &obb_corners {
        let depth = (pt - obb.center).dot(best_axis);
        if depth > max_depth {
            max_depth = depth;
            deepest_pt = pt;
        }
    }

    Some(ContactManifold {
        colliding: true,
        penetration: min_penetration,
        normal: best_axis,
        contact_points: vec![deepest_pt],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obb_obb_overlap_and_separation() {
        let box_a = OrientedBox::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), 0.0);
        let box_b = OrientedBox::new(Vec2::new(3.0, 0.0), Vec2::new(2.0, 1.0), 0.0);
        let box_c = OrientedBox::new(Vec2::new(10.0, 0.0), Vec2::new(2.0, 1.0), 0.0);

        // A and B overlap (distance 3.0 < 2.0 + 2.0 = 4.0)
        let hit_ab = collide_obb_obb(&box_a, &box_b);
        assert!(hit_ab.is_some());
        let manifold = hit_ab.unwrap();
        assert!(manifold.colliding);
        assert!((manifold.penetration - 1.0).abs() < 1e-4);
        assert_eq!(manifold.normal, Vec2::new(1.0, 0.0));

        // A and C are separated
        let hit_ac = collide_obb_obb(&box_a, &box_c);
        assert!(hit_ac.is_none());
    }

    #[test]
    fn test_rotated_obb_collision() {
        use std::f32::consts::PI;
        let box_a = OrientedBox::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), PI * 0.25);
        let box_b = OrientedBox::new(Vec2::new(1.5, 1.5), Vec2::new(2.0, 1.0), 0.0);

        let hit = collide_obb_obb(&box_a, &box_b);
        assert!(hit.is_some());
        assert!(hit.unwrap().penetration > 0.0);
    }
}
