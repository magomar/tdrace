use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::sat::OrientedBox;
use crate::body::Body2D;
use crate::track::geometry::{
    BarrierType, JumpRamp, Obstacle, ObstacleShape, RampKind, TrackGeometry, WallBarrier,
};

/// Detailed telemetry and physics result of a vehicle-wall collision impact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WallCollisionEvent {
    /// World position of the collision contact point.
    pub contact_point: Vec2,
    /// Collision normal pointing outward from the barrier into open space.
    pub normal: Vec2,
    /// Penetration depth in meters before positional correction.
    pub penetration: f32,
    /// Relative impact speed perpendicular to the wall (m/s).
    pub impact_speed: f32,
    /// Scalar normal impulse delivered to the chassis (N*s).
    pub normal_impulse: f32,
    /// Tangential friction impulse delivered along the barrier (N*s).
    pub friction_impulse: f32,
    /// Barrier material type involved in the collision.
    #[serde(default = "default_barrier_type")]
    pub barrier_type: BarrierType,
}

const fn default_barrier_type() -> BarrierType {
    BarrierType::Concrete
}

impl WallCollisionEvent {
    /// Computes estimated raw mechanical damage energy for vehicle damage simulation.
    /// Incorporates the barrier's structural energy absorption factor (e.g. TireWall absorbs 75%, Concrete absorbs 10%).
    /// Minor bumper impacts (< 3.0 m/s or normal impulse < 1000 N*s) produce zero structural damage energy.
    /// Longitudinal scraping friction is excluded from structural deformation damage.
    pub fn estimated_damage_energy(&self) -> f32 {
        if self.impact_speed < 3.0 || self.normal_impulse < 1000.0 {
            return 0.0;
        }
        let absorbed = self.barrier_type.energy_absorption_factor();
        let eff_speed = (self.impact_speed - 2.0).max(0.0);
        let normal_energy = 0.5 * self.normal_impulse * eff_speed;
        (normal_energy * (1.0 - absorbed)).max(0.0)
    }
}

/// Sliding speed along a wall below which it applies no scraping brake (m/s). Spec 115.
pub const SCRAPE_BRAKE_START_MPS: f32 = 2.0;
/// Sliding speed from which the wall's full scraping brake applies (m/s). Spec 115.
pub const SCRAPE_BRAKE_FULL_MPS: f32 = 4.0;

/// Resolves collision between a vehicle and a static line barrier.
pub fn resolve_car_wall_collision<B: Body2D>(
    car: &mut B,
    wall: &WallBarrier,
) -> Option<WallCollisionEvent> {
    if !wall.is_physical() {
        return None;
    }

    let road_elevation = car.total_elevation() - car.dynamic_elevation();
    if (road_elevation - wall.elevation).abs() > 1.8 {
        return None;
    }

    let seg = &wall.segment;
    let seg_len_sq = seg.length_squared();
    if seg_len_sq < 1e-6 {
        return None;
    }

    // Fast broad-phase AABB rejection: body bounding radius + margin. Never below the
    // 3.6 m every car used before spec 054, so car results are unchanged.
    let car_reach = car.hull().reach().max(3.6f32);
    let min_x = seg.start.x.min(seg.end.x) - car_reach;
    let max_x = seg.start.x.max(seg.end.x) + car_reach;
    let min_y = seg.start.y.min(seg.end.y) - car_reach;
    let max_y = seg.start.y.max(seg.end.y) + car_reach;
    let pos = car.position();
    if pos.x < min_x || pos.x > max_x || pos.y < min_y || pos.y > max_y {
        return None;
    }

    let obb = OrientedBox::from_body(car);
    let corners = obb.corners();

    let seg_ab = seg.end - seg.start;
    let seg_len = seg_ab.length();
    let seg_dir = if seg_len > 1e-6 { seg_ab / seg_len } else { Vec2::X };
    let wall_norm = Vec2::new(-seg_dir.y, seg_dir.x);
    // Determine which side of the wall the car center is on
    let car_center_side = (car.position() - seg.start).dot(wall_norm);
    let (approach_normal, is_positive_side) = if car_center_side >= 0.0 {
        (wall_norm, true)
    } else {
        (-wall_norm, false)
    };

    let skin_thickness = 0.02f32;
    let mut deepest_penetration = 0.0f32;
    // At most 4 car OBB corners + 2 wall segment endpoints can penetrate (Spec 119).
    let mut penetrating_points = [(Vec2::ZERO, 0.0f32, Vec2::ZERO); 6];
    let mut penetrating_len = 0usize;

    // Test each car corner against the line segment
    for &c in &corners {
        let closest_pt = seg.closest_point(c);
        let dist_to_seg = (c - closest_pt).length();

        // Only consider corners close to the segment span
        if dist_to_seg > 1.5 {
            continue;
        }

        let ac = c - seg.start;
        let proj_t = ac.dot(seg_dir);

        if proj_t >= -0.05 && proj_t <= seg_len + 0.05 {
            // Corner falls within the segment's longitudinal span
            let perp_dist = if is_positive_side {
                ac.dot(wall_norm)
            } else {
                -ac.dot(wall_norm)
            };
            let penetration = skin_thickness - perp_dist;
            if penetration > 0.0 {
                if penetration > deepest_penetration {
                    deepest_penetration = penetration;
                }
                penetrating_points[penetrating_len] = (c, penetration, approach_normal);
                penetrating_len += 1;
            }
        } else if dist_to_seg < skin_thickness {
            // Corner is past segment endpoints: only collide if within skin_thickness of endpoint
            let penetration = skin_thickness - dist_to_seg;
            let pt_norm = if dist_to_seg > 1e-5 {
                (c - closest_pt) / dist_to_seg
            } else {
                approach_normal
            };
            if penetration > deepest_penetration {
                deepest_penetration = penetration;
            }
            penetrating_points[penetrating_len] = (c, penetration, pt_norm);
            penetrating_len += 1;
        }
    }

    // Also test segment endpoints against car OBB
    let fwd = obb.forward();
    let left = obb.left();
    for &endpoint in &[seg.start, seg.end] {
        let rel = endpoint - obb.center;
        let local_x = rel.dot(fwd);
        let local_y = rel.dot(left);
        if local_x.abs() <= obb.half_extents.x && local_y.abs() <= obb.half_extents.y {
            let overlap_x = obb.half_extents.x - local_x.abs();
            let overlap_y = obb.half_extents.y - local_y.abs();
            if overlap_x > 0.0 && overlap_y > 0.0 {
                let (pen, pt_norm) = if overlap_x < overlap_y {
                    let sign = if local_x >= 0.0 { -1.0 } else { 1.0 };
                    (overlap_x + skin_thickness, fwd * sign)
                } else {
                    let sign = if local_y >= 0.0 { -1.0 } else { 1.0 };
                    (overlap_y + skin_thickness, left * sign)
                };
                if pen > deepest_penetration {
                    deepest_penetration = pen;
                }
                penetrating_points[penetrating_len] = (endpoint, pen, pt_norm);
                penetrating_len += 1;
            }
        }
    }

    if penetrating_len == 0 || deepest_penetration <= 1e-5 {
        return None;
    }

    // Average the contact points and normals that are close to maximum penetration
    let threshold = (deepest_penetration - 0.02).max(0.0);
    let mut contact_sum = Vec2::ZERO;
    let mut normal_sum = Vec2::ZERO;
    let mut count = 0.0f32;
    for (pt, pen, norm) in &penetrating_points[..penetrating_len] {
        if *pen >= threshold {
            contact_sum += *pt;
            normal_sum += *norm;
            count += 1.0;
        }
    }
    let contact_point = if count > 0.0 {
        contact_sum / count
    } else {
        penetrating_points[0].0
    };

    let normal = if normal_sum.length_squared() > 1e-4 {
        normal_sum.normalize()
    } else {
        approach_normal
    };

    // 1. Positional pushout to resolve penetration
    let pushout = normal * (deepest_penetration + 0.002);
    car.translate(pushout);

    // 2. Rigid body impulse resolution
    let r = contact_point - car.position(); // Vector from CG to contact point
    let omega = car.angular_velocity();
    // Velocity at contact point: V_c = V + omega x r
    let v_rot = Vec2::new(-omega * r.y, omega * r.x);
    let v_contact = car.velocity() + v_rot;

    let v_n = v_contact.dot(normal);

    let mass = car.mass();
    let inertia = car.inertia();

    let mut j_n = 0.0;
    if v_n < 0.0 {
        let r_cross_n = r.x * normal.y - r.y * normal.x;
        let k_n = (1.0 / mass) + (r_cross_n * r_cross_n) / inertia;
        let restitution = wall.restitution;
        j_n = (-(1.0 + restitution) * v_n) / k_n;

        // Apply normal impulse
        let impulse_n = normal * j_n;
        car.add_velocity(impulse_n / mass);
        car.add_angular_velocity((r.x * impulse_n.y - r.y * impulse_n.x) / inertia);
    }

    // 3. Tangential sliding friction and contact braking resistance
    let v_contact_after_n = car.velocity() + Vec2::new(-car.angular_velocity() * r.y, car.angular_velocity() * r.x);
    let v_tangent_vec = v_contact_after_n - normal * v_contact_after_n.dot(normal);
    let v_t_mag = v_tangent_vec.length();

    let mut j_t_applied = 0.0;
    if v_t_mag > 1e-4 {
        let tangent = v_tangent_vec / v_t_mag;
        let r_cross_t = r.x * tangent.y - r.y * tangent.x;
        let k_t = (1.0 / mass) + (r_cross_t * r_cross_t) / inertia;

        let j_t_desired = -v_t_mag / k_t;
        let max_impact_friction = wall.friction * j_n;

        // Contact wall resistance acting as brakes when running next to or touching the wall
        let brake_decel = wall.barrier_type.scraping_deceleration();
        // resolve_all_wall_collisions runs 2 sub-iterations per 60Hz frame (dt_sub ~ 0.01667 / 2 = 0.00833s)
        // No contact brake below SCRAPE_BRAKE_START_MPS (2.0 m/s), ramping linearly to full brake at SCRAPE_BRAKE_FULL_MPS (4.0 m/s):
        // a car resting against a curved wall on grass/gravel scrapes whichever way it drives; starting at 0 m/s locked
        // the car to ~0.2 m/s crawl. (Spec 115, tdrace-7eda5).
        let speed_scale =
            ((v_t_mag - SCRAPE_BRAKE_START_MPS) / (SCRAPE_BRAKE_FULL_MPS - SCRAPE_BRAKE_START_MPS)).clamp(0.0, 1.0);
        let contact_brake_impulse = mass * brake_decel * 0.00833 * speed_scale;

        let max_total_friction = max_impact_friction + contact_brake_impulse;
        let j_t = j_t_desired.clamp(-max_total_friction, max_total_friction);
        j_t_applied = j_t.abs();

        let impact_ratio = if max_total_friction > 1e-4 {
            (max_impact_friction / max_total_friction).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let impulse_t = tangent * j_t;
        car.add_velocity(impulse_t / mass);
        let base_snag = wall.barrier_type.snag_torque_factor();
        let torque_fraction = impact_ratio + (1.0 - impact_ratio) * base_snag;
        car.add_angular_velocity(((r.x * impulse_t.y - r.y * impulse_t.x) / inertia) * torque_fraction);
    }

    Some(WallCollisionEvent {
        contact_point,
        normal,
        penetration: deepest_penetration,
        impact_speed: (-v_n).max(0.0),
        normal_impulse: j_n,
        friction_impulse: j_t_applied,
        barrier_type: wall.barrier_type,
    })
}

/// Resolves collision between a vehicle and a static obstacle.
pub fn resolve_car_obstacle_collision<B: Body2D>(
    car: &mut B,
    obstacle: &Obstacle,
) -> Option<WallCollisionEvent> {
    if (car.total_elevation() - obstacle.elevation).abs() > 1.8 || car.jump_height() > 1.0 {
        return None;
    }
    let obb = OrientedBox::from_body(car);

    let manifold = match &obstacle.shape {
        ObstacleShape::Circle { center, radius } => {
            super::sat::collide_obb_circle(&obb, *center, *radius)
        }
        ObstacleShape::Box {
            center,
            half_extents,
            angle,
        } => {
            let obs_box = OrientedBox::new(*center, *half_extents, *angle);
            super::sat::collide_obb_obb(&obb, &obs_box)
        }
        ObstacleShape::Polygon { vertices } => {
            super::sat::collide_obb_polygon(&obb, vertices)
        }
    }?;

    if !manifold.colliding || manifold.penetration <= 1e-5 {
        return None;
    }

    // Normal points from car to obstacle -> invert so normal points from obstacle into car
    let normal = -manifold.normal;
    let contact_pt = manifold
        .contact_points
        .first()
        .copied()
        .unwrap_or(car.position());

    // Positional correction
    let pushout = normal * (manifold.penetration + 0.002);
    car.translate(pushout);

    // Impulse
    let r = contact_pt - car.position();
    let omega = car.angular_velocity();
    let v_rot = Vec2::new(-omega * r.y, omega * r.x);
    let v_contact = car.velocity() + v_rot;

    let v_n = v_contact.dot(normal);

    let mass = car.mass();
    let inertia = car.inertia();

    let mut j_n = 0.0;
    if v_n < 0.0 {
        let r_cross_n = r.x * normal.y - r.y * normal.x;
        let k_n = (1.0 / mass) + (r_cross_n * r_cross_n) / inertia;
        let restitution = obstacle.restitution;
        j_n = (-(1.0 + restitution) * v_n) / k_n;

        let impulse_n = normal * j_n;
        car.add_velocity(impulse_n / mass);
        car.add_angular_velocity((r.x * impulse_n.y - r.y * impulse_n.x) / inertia);
    }

    // Tangential sliding friction and contact resistance
    let v_contact_after_n = car.velocity() + Vec2::new(-car.angular_velocity() * r.y, car.angular_velocity() * r.x);
    let v_tangent_vec = v_contact_after_n - normal * v_contact_after_n.dot(normal);
    let v_t_mag = v_tangent_vec.length();

    let mut j_t_applied = 0.0;
    if v_t_mag > 1e-4 {
        let tangent = v_tangent_vec / v_t_mag;
        let r_cross_t = r.x * tangent.y - r.y * tangent.x;
        let k_t = (1.0 / mass) + (r_cross_t * r_cross_t) / inertia;

        let j_t_desired = -v_t_mag / k_t;
        let max_impact_friction = obstacle.friction * j_n;
        let speed_scale = (v_t_mag / 1.0).clamp(0.0, 1.0);
        let contact_brake_impulse = mass * (obstacle.friction * 20.0) * 0.00833 * speed_scale;
        let max_total_friction = max_impact_friction + contact_brake_impulse;
        let j_t = j_t_desired.clamp(-max_total_friction, max_total_friction);
        j_t_applied = j_t.abs();

        let impulse_t = tangent * j_t;
        car.add_velocity(impulse_t / mass);
        car.add_angular_velocity(((r.x * impulse_t.y - r.y * impulse_t.x) / inertia) * 0.2);
    }

    Some(WallCollisionEvent {
        contact_point: contact_pt,
        normal,
        penetration: manifold.penetration,
        impact_speed: (-v_n).max(0.0),
        normal_impulse: j_n,
        friction_impulse: j_t_applied,
        barrier_type: BarrierType::Concrete,
    })
}

/// Resolves all wall and obstacle collisions for a car against track barriers,
/// appending events into `events_out` without allocating a new vector (Spec 119).
pub fn resolve_all_wall_collisions_into<B: Body2D>(
    car: &mut B,
    walls: &[WallBarrier],
    obstacles: &[Obstacle],
    events_out: &mut Vec<WallCollisionEvent>,
) {
    if walls.is_empty() && obstacles.is_empty() {
        return;
    }

    let car_reach = car.hull().reach().max(3.6f32);
    let reach_pad = car_reach + 0.5;
    let pos0 = car.position();
    let min_x = pos0.x - reach_pad;
    let max_x = pos0.x + reach_pad;
    let min_y = pos0.y - reach_pad;
    let max_y = pos0.y + reach_pad;

    let mut near_walls = [0usize; 32];
    let mut near_len = 0usize;
    let mut overflow = false;

    // Iteration 0: broad-phase filter + collect candidate wall indices for Iteration 1
    for (idx, wall) in walls.iter().enumerate() {
        let seg = &wall.segment;
        let w_min_x = seg.start.x.min(seg.end.x);
        let w_max_x = seg.start.x.max(seg.end.x);
        let w_min_y = seg.start.y.min(seg.end.y);
        let w_max_y = seg.start.y.max(seg.end.y);
        if w_max_x < min_x || w_min_x > max_x || w_max_y < min_y || w_min_y > max_y {
            continue;
        }
        if near_len < near_walls.len() {
            near_walls[near_len] = idx;
            near_len += 1;
        } else {
            overflow = true;
        }
        if let Some(ev) = resolve_car_wall_collision(car, wall) {
            events_out.push(ev);
        }
    }

    for obs in obstacles {
        if let Some(ev) = resolve_car_obstacle_collision(car, obs) {
            events_out.push(ev);
        }
    }

    // Iteration 1: only re-test the nearby candidate walls
    if overflow {
        for wall in walls {
            let _ = resolve_car_wall_collision(car, wall);
        }
    } else {
        for &idx in &near_walls[..near_len] {
            let _ = resolve_car_wall_collision(car, &walls[idx]);
        }
    }
    for obs in obstacles {
        let _ = resolve_car_obstacle_collision(car, obs);
    }
}

/// Resolves collisions between a vehicle and the vertical front face of the landing segment
/// on any [`RampKind::MegaRamp`] when the vehicle fails to clear the gap onto the landing deck.
pub fn resolve_mega_ramp_wall_collisions_into<B: Body2D>(
    car: &mut B,
    jump_ramps: &[JumpRamp],
    events_out: &mut Vec<WallCollisionEvent>,
) {
    if jump_ramps.is_empty() {
        return;
    }

    let car_reach = car.hull().reach().max(3.6f32);
    let road_elevation = car.total_elevation() - car.dynamic_elevation();

    for ramp in jump_ramps {
        if ramp.kind != RampKind::MegaRamp {
            continue;
        }
        let min_clear_h = ramp.mega_clear_height();
        if car.dynamic_elevation() >= min_clear_h {
            continue;
        }

        let (s_takeoff, s_landing) = ramp.mega_segment_bounds();
        let center = ramp.shape.center();
        let half = ramp.half_extents();
        let half_len = half.x.max(0.5);
        let half_wid = half.y.max(0.5);
        let full_len = 2.0 * half_len;
        let fwd = ramp.direction.normalize_or_zero();
        let right = Vec2::new(fwd.y, -fwd.x);

        let x_takeoff_end = -half_len + s_takeoff * full_len;
        let x_landing_start = -half_len + s_landing * full_len;

        let rel = car.position() - center;
        let local_x = rel.dot(fwd);
        let local_y = rel.dot(right);

        // Only collide when approaching the landing wall from the gap (or if center slightly tunneled past it at ground level)
        if local_x <= x_takeoff_end || local_x > x_landing_start + 2.0 {
            continue;
        }
        if local_y.abs() > half_wid + car_reach {
            continue;
        }

        // If the vehicle center already crossed x_landing_start without clearing the deck,
        // shift it back onto the gap side of the wall so the collision normal points into the gap (-fwd).
        if local_x >= x_landing_start - 0.05 && local_y.abs() <= half_wid + 0.5 {
            let pullback = (local_x - (x_landing_start - 0.10)).max(0.0);
            if pullback > 0.0 {
                car.translate(-fwd * pullback);
            }
        }

        if let Some(wall) = ramp.mega_landing_wall_at_elevation(road_elevation) {
            for iter in 0..2 {
                if let Some(ev) = resolve_car_wall_collision(car, &wall) {
                    if iter == 0 {
                        events_out.push(ev);
                    }
                }
            }
        }
    }
}

/// Resolves all wall and obstacle collisions for a car against [`TrackGeometry`] using its
/// static 2D uniform spatial grid (`GeometrySpatialGrid`), appending events into `events_out`
/// with zero heap allocations (Spec 119).
pub fn resolve_track_wall_collisions_into<B: Body2D>(
    car: &mut B,
    geometry: &TrackGeometry,
    events_out: &mut Vec<WallCollisionEvent>,
) {
    resolve_mega_ramp_wall_collisions_into(car, &geometry.jump_ramps, events_out);

    let obstacles = geometry.all_obstacles_with_scenery();
    let Some(grid) = geometry.spatial_grid() else {
        resolve_all_wall_collisions_into(car, &geometry.inner_walls, obstacles, events_out);
        resolve_all_wall_collisions_into(car, &geometry.outer_walls, &[], events_out);
        resolve_all_wall_collisions_into(car, &geometry.network_walls, &[], events_out);
        return;
    };

    let reach_pad = car.hull().reach().max(3.6f32) + 0.5;
    let pos = car.position();
    let q_min = pos - Vec2::splat(reach_pad);
    let q_max = pos + Vec2::splat(reach_pad);

    let mut wall_idxs = [0usize; 64];
    let mut w_len = 0usize;
    let mut w_overflow = false;
    grid.for_each_wall_in_aabb(q_min, q_max, |idx, _| {
        if w_len < wall_idxs.len() {
            wall_idxs[w_len] = idx;
            w_len += 1;
        } else {
            w_overflow = true;
        }
    });

    let mut obs_idxs = [0usize; 32];
    let mut o_len = 0usize;
    let mut o_overflow = false;
    grid.for_each_obstacle_in_aabb(obstacles, q_min, q_max, |idx, _| {
        if o_len < obs_idxs.len() {
            obs_idxs[o_len] = idx;
            o_len += 1;
        } else {
            o_overflow = true;
        }
    });

    if w_overflow || o_overflow {
        resolve_all_wall_collisions_into(car, &geometry.inner_walls, obstacles, events_out);
        resolve_all_wall_collisions_into(car, &geometry.outer_walls, &[], events_out);
        resolve_all_wall_collisions_into(car, &geometry.network_walls, &[], events_out);
        return;
    }

    if w_len == 0 && o_len == 0 {
        return;
    }

    wall_idxs[..w_len].sort_unstable();
    obs_idxs[..o_len].sort_unstable();

    let n_inner = grid.inner_count;
    let n_inner_outer = n_inner + grid.outer_count;
    let split_inner = wall_idxs[..w_len].partition_point(|&idx| idx < n_inner);
    let split_outer = wall_idxs[..w_len].partition_point(|&idx| idx < n_inner_outer);

    let inner_slice = &wall_idxs[..split_inner];
    let outer_slice = &wall_idxs[split_inner..split_outer];
    let net_slice = &wall_idxs[split_outer..w_len];
    let obs_slice = &obs_idxs[..o_len];

    // 1. Inner walls + scenery obstacles (2 iterations)
    if !inner_slice.is_empty() || !obs_slice.is_empty() {
        for iter in 0..2 {
            for &idx in inner_slice {
                if let Some(ev) = resolve_car_wall_collision(car, &grid.all_walls[idx]) {
                    if iter == 0 {
                        events_out.push(ev);
                    }
                }
            }
            for &idx in obs_slice {
                if let Some(ev) = resolve_car_obstacle_collision(car, &obstacles[idx]) {
                    if iter == 0 {
                        events_out.push(ev);
                    }
                }
            }
        }
    }

    // 2. Outer walls (2 iterations)
    if !outer_slice.is_empty() {
        for iter in 0..2 {
            for &idx in outer_slice {
                if let Some(ev) = resolve_car_wall_collision(car, &grid.all_walls[idx]) {
                    if iter == 0 {
                        events_out.push(ev);
                    }
                }
            }
        }
    }

    // 3. Network walls (2 iterations)
    if !net_slice.is_empty() {
        for iter in 0..2 {
            for &idx in net_slice {
                if let Some(ev) = resolve_car_wall_collision(car, &grid.all_walls[idx]) {
                    if iter == 0 {
                        events_out.push(ev);
                    }
                }
            }
        }
    }
}

/// Resolves all wall and obstacle collisions for a car against track barriers.
pub fn resolve_all_wall_collisions<B: Body2D>(
    car: &mut B,
    walls: &[WallBarrier],
    obstacles: &[Obstacle],
) -> Vec<WallCollisionEvent> {
    let mut events = Vec::new();
    resolve_all_wall_collisions_into(car, walls, obstacles, &mut events);
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use wheelbase::{Car, CarConfig};
    use crate::track::geometry::BarrierType;

    #[test]
    fn test_head_on_wall_collision_restitution() {
        let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
        car.state.velocity = Vec2::new(20.0, 0.0); // Driving in +X at 20 m/s

        // Wall placed vertically at x=1.0 with normal pointing -X
        let wall = WallBarrier::new(
            Vec2::new(1.0, -10.0),
            Vec2::new(1.0, 10.0),
            BarrierType::Concrete,
        );

        let event = resolve_car_wall_collision(&mut car, &wall);
        assert!(event.is_some(), "Collision must be detected");
        let ev = event.unwrap();
        assert!(ev.impact_speed > 10.0);
        // After impact, velocity along X must be negative (bounced backwards)
        assert!(
            car.state.velocity.x < 0.0,
            "Car must bounce back, velocity.x is {}",
            car.state.velocity.x
        );
    }

    #[test]
    fn test_corner_impact_angular_impulse() {
        let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.2); // Angled 0.2 rad towards wall
        car.state.velocity = Vec2::new(15.0, 3.0);

        // Wall placed above the car along Y = 0.6
        let wall = WallBarrier::with_physics(
            Vec2::new(-5.0, 0.6),
            Vec2::new(5.0, 0.6),
            BarrierType::Steel,
            0.5,
            0.4,
        );

        let res = resolve_car_wall_collision(&mut car, &wall);
        assert!(res.is_some(), "Should collide with wall at y=0.6");
        // Left front corner clips the wall -> should introduce yaw torque
        assert!(
            car.state.angular_velocity.abs() > 0.1,
            "Yaw rate should be induced on corner wall clip, got {}",
            car.state.angular_velocity
        );
    }

    #[test]
    fn test_virtual_barrier_collision_bypass() {
        let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
        car.state.velocity = Vec2::new(20.0, 0.0);

        let virtual_wall = WallBarrier::new(
            Vec2::new(1.0, -10.0),
            Vec2::new(1.0, 10.0),
            BarrierType::Virtual,
        );

        let event = resolve_car_wall_collision(&mut car, &virtual_wall);
        assert!(event.is_none(), "Virtual barriers must not produce collisions");
        assert_eq!(
            car.state.velocity.x, 20.0,
            "Car velocity must remain completely unaffected by virtual barrier"
        );
    }

    #[test]
    fn test_low_speed_wall_impact_damage_deadzone() {
        let low_speed_event = WallCollisionEvent {
            contact_point: Vec2::ZERO,
            normal: Vec2::new(-1.0, 0.0),
            penetration: 0.01,
            impact_speed: 2.5, // < 3.0 m/s
            normal_impulse: 500.0,
            friction_impulse: 300.0,
            barrier_type: BarrierType::Concrete,
        };
        assert_eq!(
            low_speed_event.estimated_damage_energy(),
            0.0,
            "Low-speed collision below 3 m/s must produce 0 damage energy"
        );

        let scraping_event = WallCollisionEvent {
            contact_point: Vec2::ZERO,
            normal: Vec2::new(-1.0, 0.0),
            penetration: 0.01,
            impact_speed: 1.0,
            normal_impulse: 400.0,
            friction_impulse: 2000.0, // High friction from sliding alongside barrier
            barrier_type: BarrierType::Concrete,
        };
        assert_eq!(
            scraping_event.estimated_damage_energy(),
            0.0,
            "Wall scraping with low normal impulse must produce 0 damage energy"
        );
    }

    #[test]
    fn test_spec_119_zero_alloc_wall_collision_and_spatial_grid_parity() {
        use crate::lidar::{LidarConfig, LidarHit, LidarScanner};

        let track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let grid = track
            .geometry
            .spatial_grid()
            .expect("TrackGeometry must build a GeometrySpatialGrid");
        assert!(
            !grid.all_walls.is_empty(),
            "Spatial grid must index circuit walls"
        );

        // Verify multi-cell deduplication: querying the full track AABB visits each wall once
        let mut visited_indices = Vec::new();
        grid.for_each_wall_in_aabb(Vec2::splat(-10_000.0), Vec2::splat(10_000.0), |idx, _| {
            visited_indices.push(idx);
        });
        visited_indices.sort_unstable();
        let expected_indices: Vec<usize> = (0..grid.all_walls.len()).collect();
        assert_eq!(
            visited_indices, expected_indices,
            "Spatial grid query over full bounds must visit every wall exactly once"
        );

        // Verify resolve_track_wall_collisions_into matches brute-force 3-pass solver
        // and does not reallocate a pre-allocated events_out buffer
        let inner_wall = &track.geometry.inner_walls[0];
        let wall_mid = (inner_wall.segment.start + inner_wall.segment.end) * 0.5;
        let mut car_grid = Car::new(CarConfig::sports_car()).with_pose(wall_mid, 0.15);
        car_grid.state.velocity = -inner_wall.segment.normal() * 18.0;
        let mut car_linear = car_grid.clone();

        let mut events_grid = Vec::with_capacity(16);
        let initial_ptr = events_grid.as_ptr();
        let initial_cap = events_grid.capacity();
        resolve_track_wall_collisions_into(&mut car_grid, &track.geometry, &mut events_grid);
        assert_eq!(
            events_grid.as_ptr(),
            initial_ptr,
            "Pre-allocated events buffer must not reallocate during collision resolution"
        );
        assert_eq!(events_grid.capacity(), initial_cap);
        assert!(!events_grid.is_empty(), "Must record wall collision event");

        let mut events_linear = Vec::new();
        events_linear.extend(resolve_all_wall_collisions(
            &mut car_linear,
            &track.geometry.inner_walls,
            track.geometry.all_obstacles_with_scenery(),
        ));
        events_linear.extend(resolve_all_wall_collisions(
            &mut car_linear,
            &track.geometry.outer_walls,
            &[],
        ));
        events_linear.extend(resolve_all_wall_collisions(
            &mut car_linear,
            &track.geometry.network_walls,
            &[],
        ));

        assert_eq!(events_grid.len(), events_linear.len());
        assert!(
            (car_grid.state.position - car_linear.state.position).length() < 1e-6,
            "Grid-accelerated solver position must match linear solver"
        );
        assert!(
            (car_grid.state.velocity - car_linear.state.velocity).length() < 1e-6,
            "Grid-accelerated solver velocity must match linear solver"
        );

        // Verify LidarScanner::scan_into detects walls via GeometrySpatialGrid
        let scanner = LidarScanner::new(LidarConfig::default());
        let mut out_hits = vec![LidarHit::default(); scanner.config.num_rays];
        let sample = &track.spline.samples[0];
        let lidar_car = Car::new(CarConfig::sports_car()).with_pose(sample.point, 0.0);
        scanner.scan_into(&lidar_car, &track, &[], &mut out_hits);
        assert!(
            out_hits.iter().any(|h| h.distance < scanner.config.max_range),
            "LIDAR must detect track walls via GeometrySpatialGrid"
        );
    }
}

