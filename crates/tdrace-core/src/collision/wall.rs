use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::sat::OrientedBox;
use crate::physics::car::Car;
use crate::track::geometry::{Obstacle, ObstacleShape, WallBarrier};

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
}

/// Resolves collision between a vehicle and a static line barrier.
pub fn resolve_car_wall_collision(
    car: &mut Car,
    wall: &WallBarrier,
) -> Option<WallCollisionEvent> {
    if (car.total_elevation() - wall.elevation).abs() > 1.8 || car.state.elevation > 1.2 {
        return None;
    }
    let obb = OrientedBox::from_car(car);
    let corners = obb.corners();

    let seg = &wall.segment;
    let seg_len_sq = seg.length_squared();
    if seg_len_sq < 1e-6 {
        return None;
    }

    let wall_norm = seg.normal();
    // Determine which side of the wall the car center is on
    let car_center_side = (car.state.position - seg.start).dot(wall_norm);
    let (approach_normal, is_positive_side) = if car_center_side >= 0.0 {
        (wall_norm, true)
    } else {
        (-wall_norm, false)
    };

    let skin_thickness = 0.02f32;
    let mut deepest_penetration = 0.0f32;
    let mut penetrating_points = Vec::new();

    // Test each car corner against the line segment
    for &c in &corners {
        let closest_pt = seg.closest_point(c);
        let dist_to_seg = (c - closest_pt).length();

        // Only consider corners close to the segment span
        if dist_to_seg > 1.5 {
            continue;
        }

        let signed_dist = (c - seg.start).dot(wall_norm);
        let penetration = if is_positive_side {
            skin_thickness - signed_dist
        } else {
            skin_thickness + signed_dist
        };

        if penetration > 0.0 {
            if penetration > deepest_penetration {
                deepest_penetration = penetration;
            }
            penetrating_points.push((c, penetration));
        }
    }

    // Also test segment endpoints against car OBB
    for &endpoint in &[seg.start, seg.end] {
        if obb.contains_point(endpoint) {
            let closest_on_car = obb.center + (endpoint - obb.center).clamp(
                -obb.half_extents,
                obb.half_extents,
            );
            let pen = (endpoint - closest_on_car).length() + skin_thickness;
            if pen > 0.0 {
                if pen > deepest_penetration {
                    deepest_penetration = pen;
                }
                penetrating_points.push((endpoint, pen));
            }
        }
    }

    if penetrating_points.is_empty() || deepest_penetration <= 1e-5 {
        return None;
    }

    // Average the contact points that are close to maximum penetration
    let threshold = (deepest_penetration - 0.02).max(0.0);
    let mut contact_sum = Vec2::ZERO;
    let mut count = 0.0f32;
    for (pt, pen) in &penetrating_points {
        if *pen >= threshold {
            contact_sum += *pt;
            count += 1.0;
        }
    }
    let contact_point = if count > 0.0 {
        contact_sum / count
    } else {
        penetrating_points[0].0
    };

    let normal = approach_normal;

    // 1. Positional pushout to resolve penetration
    let pushout = normal * (deepest_penetration + 0.002);
    car.state.position += pushout;

    // 2. Rigid body impulse resolution
    let r = contact_point - car.state.position; // Vector from CG to contact point
    let omega = car.state.angular_velocity;
    // Velocity at contact point: V_c = V + omega x r
    let v_rot = Vec2::new(-omega * r.y, omega * r.x);
    let v_contact = car.state.velocity + v_rot;

    let v_n = v_contact.dot(normal);

    let mass = car.config.mass;
    let inertia = car.config.inertia;

    let mut j_n = 0.0;
    if v_n < 0.0 {
        let r_cross_n = r.x * normal.y - r.y * normal.x;
        let k_n = (1.0 / mass) + (r_cross_n * r_cross_n) / inertia;
        let restitution = wall.restitution;
        j_n = (-(1.0 + restitution) * v_n) / k_n;

        // Apply normal impulse
        let impulse_n = normal * j_n;
        car.state.velocity += impulse_n / mass;
        car.state.angular_velocity += (r.x * impulse_n.y - r.y * impulse_n.x) / inertia;
    }

    // 3. Tangential sliding friction and contact braking resistance
    let v_contact_after_n = car.state.velocity + Vec2::new(-car.state.angular_velocity * r.y, car.state.angular_velocity * r.x);
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
        let brake_decel = match wall.barrier_type {
            crate::track::geometry::BarrierType::TireWall => 22.0, // High rubber grip and compression drag (~2.2g)
            crate::track::geometry::BarrierType::Concrete => 9.5,  // Solid concrete scraping resistance (~0.95g)
            crate::track::geometry::BarrierType::Armco => 11.5,    // Steel Armco barrier resistance (~1.15g)
            crate::track::geometry::BarrierType::CurbWall => 7.5,  // Low curb wall resistance (~0.75g)
        };
        // resolve_all_wall_collisions runs 2 sub-iterations per 60Hz frame (dt_sub ~ 0.01667 / 2 = 0.00833s)
        let contact_brake_impulse = mass * brake_decel * 0.00833;

        let max_total_friction = max_impact_friction + contact_brake_impulse;
        let j_t = j_t_desired.clamp(-max_total_friction, max_total_friction);
        j_t_applied = j_t.abs();

        let impact_ratio = if max_total_friction > 1e-4 {
            (max_impact_friction / max_total_friction).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let impulse_t = tangent * j_t;
        car.state.velocity += impulse_t / mass;
        let torque_fraction = impact_ratio + (1.0 - impact_ratio) * 0.15;
        car.state.angular_velocity += ((r.x * impulse_t.y - r.y * impulse_t.x) / inertia) * torque_fraction;
    }

    Some(WallCollisionEvent {
        contact_point,
        normal,
        penetration: deepest_penetration,
        impact_speed: (-v_n).max(0.0),
        normal_impulse: j_n,
        friction_impulse: j_t_applied,
    })
}

/// Resolves collision between a vehicle and a static obstacle.
pub fn resolve_car_obstacle_collision(
    car: &mut Car,
    obstacle: &Obstacle,
) -> Option<WallCollisionEvent> {
    if (car.total_elevation() - obstacle.elevation).abs() > 1.8 || car.state.elevation > 1.0 {
        return None;
    }
    let obb = OrientedBox::from_car(car);

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
        .unwrap_or(car.state.position);

    // Positional correction
    let pushout = normal * (manifold.penetration + 0.002);
    car.state.position += pushout;

    // Impulse
    let r = contact_pt - car.state.position;
    let omega = car.state.angular_velocity;
    let v_rot = Vec2::new(-omega * r.y, omega * r.x);
    let v_contact = car.state.velocity + v_rot;

    let v_n = v_contact.dot(normal);

    let mass = car.config.mass;
    let inertia = car.config.inertia;

    let mut j_n = 0.0;
    if v_n < 0.0 {
        let r_cross_n = r.x * normal.y - r.y * normal.x;
        let k_n = (1.0 / mass) + (r_cross_n * r_cross_n) / inertia;
        let restitution = obstacle.restitution;
        j_n = (-(1.0 + restitution) * v_n) / k_n;

        let impulse_n = normal * j_n;
        car.state.velocity += impulse_n / mass;
        car.state.angular_velocity += (r.x * impulse_n.y - r.y * impulse_n.x) / inertia;
    }

    // Tangential sliding friction and contact resistance
    let v_contact_after_n = car.state.velocity + Vec2::new(-car.state.angular_velocity * r.y, car.state.angular_velocity * r.x);
    let v_tangent_vec = v_contact_after_n - normal * v_contact_after_n.dot(normal);
    let v_t_mag = v_tangent_vec.length();

    let mut j_t_applied = 0.0;
    if v_t_mag > 1e-4 {
        let tangent = v_tangent_vec / v_t_mag;
        let r_cross_t = r.x * tangent.y - r.y * tangent.x;
        let k_t = (1.0 / mass) + (r_cross_t * r_cross_t) / inertia;

        let j_t_desired = -v_t_mag / k_t;
        let max_impact_friction = obstacle.friction * j_n;
        let contact_brake_impulse = mass * (obstacle.friction * 20.0) * 0.00833;
        let max_total_friction = max_impact_friction + contact_brake_impulse;
        let j_t = j_t_desired.clamp(-max_total_friction, max_total_friction);
        j_t_applied = j_t.abs();

        let impulse_t = tangent * j_t;
        car.state.velocity += impulse_t / mass;
        car.state.angular_velocity += ((r.x * impulse_t.y - r.y * impulse_t.x) / inertia) * 0.2;
    }

    Some(WallCollisionEvent {
        contact_point: contact_pt,
        normal,
        penetration: manifold.penetration,
        impact_speed: (-v_n).max(0.0),
        normal_impulse: j_n,
        friction_impulse: j_t_applied,
    })
}

/// Resolves all wall and obstacle collisions for a car against track barriers.
pub fn resolve_all_wall_collisions(
    car: &mut Car,
    walls: &[WallBarrier],
    obstacles: &[Obstacle],
) -> Vec<WallCollisionEvent> {
    let mut events = Vec::new();

    for _ in 0..2 {
        for wall in walls {
            if let Some(ev) = resolve_car_wall_collision(car, wall) {
                events.push(ev);
            }
        }
        for obs in obstacles {
            if let Some(ev) = resolve_car_obstacle_collision(car, obs) {
                events.push(ev);
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::config::CarConfig;
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
            BarrierType::Armco,
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
}
