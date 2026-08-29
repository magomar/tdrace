use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::sat::{collide_obb_obb, OrientedBox};
use crate::physics::car::Car;

/// Telemetry record of an elastic/inelastic collision between two racing vehicles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CarCarCollisionEvent {
    pub car_a_idx: usize,
    pub car_b_idx: usize,
    pub contact_point: Vec2,
    pub normal: Vec2, // Points from A to B
    pub penetration: f32,
    pub closing_speed: f32,
    pub impulse_magnitude: f32,
}

/// Resolves pairwise rigid body collision between two cars.
pub fn resolve_car_car_collision(
    car_a: &mut Car,
    car_b: &mut Car,
    restitution: f32,
    friction: f32,
) -> Option<CarCarCollisionEvent> {
    if (car_a.total_elevation() - car_b.total_elevation()).abs() > 1.2 {
        return None;
    }
    let obb_a = OrientedBox::from_car(car_a);
    let obb_b = OrientedBox::from_car(car_b);

    let manifold = collide_obb_obb(&obb_a, &obb_b)?;
    if !manifold.colliding || manifold.penetration <= 1e-5 {
        return None;
    }

    let normal = manifold.normal; // Normal points from A to B
    let contact_pt = if !manifold.contact_points.is_empty() {
        let sum: Vec2 = manifold.contact_points.iter().copied().sum();
        sum / (manifold.contact_points.len() as f32)
    } else {
        (car_a.state.position + car_b.state.position) * 0.5
    };

    let mass_a = car_a.config.mass;
    let mass_b = car_b.config.mass;
    let inv_m_a = 1.0 / mass_a;
    let inv_m_b = 1.0 / mass_b;
    let total_inv_m = inv_m_a + inv_m_b;

    // 1. Positional pushout to prevent overlap and tunneling
    let weight_a = inv_m_a / total_inv_m;
    let weight_b = inv_m_b / total_inv_m;
    let separation = normal * (manifold.penetration + 0.002);

    car_a.state.position -= separation * weight_a;
    car_b.state.position += separation * weight_b;

    // 2. Rigid Body Impulse
    let r_a = contact_pt - car_a.state.position;
    let r_b = contact_pt - car_b.state.position;

    let v_rot_a = Vec2::new(-car_a.state.angular_velocity * r_a.y, car_a.state.angular_velocity * r_a.x);
    let v_rot_b = Vec2::new(-car_b.state.angular_velocity * r_b.y, car_b.state.angular_velocity * r_b.x);

    let v_ca = car_a.state.velocity + v_rot_a;
    let v_cb = car_b.state.velocity + v_rot_b;
    let v_rel = v_cb - v_ca;

    let v_n = v_rel.dot(normal);

    // If cars are separating, no impulse needed
    if v_n >= 0.0 {
        return Some(CarCarCollisionEvent {
            car_a_idx: 0,
            car_b_idx: 1,
            contact_point: contact_pt,
            normal,
            penetration: manifold.penetration,
            closing_speed: 0.0,
            impulse_magnitude: 0.0,
        });
    }

    let inertia_a = car_a.config.inertia;
    let inertia_b = car_b.config.inertia;

    let r_a_cross_n = r_a.x * normal.y - r_a.y * normal.x;
    let r_b_cross_n = r_b.x * normal.y - r_b.y * normal.x;

    let k_n = inv_m_a + inv_m_b + (r_a_cross_n * r_a_cross_n) / inertia_a + (r_b_cross_n * r_b_cross_n) / inertia_b;
    let j_n = (-(1.0 + restitution) * v_n) / k_n;

    // Apply normal impulse
    let impulse_n = normal * j_n;
    car_a.state.velocity -= impulse_n * inv_m_a;
    car_a.state.angular_velocity -= (r_a.x * impulse_n.y - r_a.y * impulse_n.x) / inertia_a;

    car_b.state.velocity += impulse_n * inv_m_b;
    car_b.state.angular_velocity += (r_b.x * impulse_n.y - r_b.y * impulse_n.x) / inertia_b;

    // 3. Tangential friction impulse
    let v_ca_after = car_a.state.velocity + Vec2::new(-car_a.state.angular_velocity * r_a.y, car_a.state.angular_velocity * r_a.x);
    let v_cb_after = car_b.state.velocity + Vec2::new(-car_b.state.angular_velocity * r_b.y, car_b.state.angular_velocity * r_b.x);
    let v_rel_after = v_cb_after - v_ca_after;

    let v_t_vec = v_rel_after - normal * v_rel_after.dot(normal);
    let v_t_mag = v_t_vec.length();

    if v_t_mag > 1e-4 {
        let tangent = v_t_vec / v_t_mag;
        let r_a_cross_t = r_a.x * tangent.y - r_a.y * tangent.x;
        let r_b_cross_t = r_b.x * tangent.y - r_b.y * tangent.x;

        let k_t = inv_m_a + inv_m_b + (r_a_cross_t * r_a_cross_t) / inertia_a + (r_b_cross_t * r_b_cross_t) / inertia_b;
        let j_t_desired = -v_t_mag / k_t;
        let max_j_t = friction * j_n;
        let j_t = j_t_desired.clamp(-max_j_t, max_j_t);

        let impulse_t = tangent * j_t;
        car_a.state.velocity -= impulse_t * inv_m_a;
        car_a.state.angular_velocity -= (r_a.x * impulse_t.y - r_a.y * impulse_t.x) / inertia_a;

        car_b.state.velocity += impulse_t * inv_m_b;
        car_b.state.angular_velocity += (r_b.x * impulse_t.y - r_b.y * impulse_t.x) / inertia_b;
    }

    Some(CarCarCollisionEvent {
        car_a_idx: 0,
        car_b_idx: 1,
        contact_point: contact_pt,
        normal,
        penetration: manifold.penetration,
        closing_speed: -v_n,
        impulse_magnitude: j_n,
    })
}

/// Iteratively resolves all pairwise collisions across a group of racing cars.
/// Handles multi-car pileups and tight pack racing without tunneling.
pub fn resolve_multi_car_collisions(
    cars: &mut [Car],
    restitution: f32,
    friction: f32,
    solver_iterations: usize,
) -> Vec<CarCarCollisionEvent> {
    let n = cars.len();
    if n < 2 {
        return Vec::new();
    }

    let mut events = Vec::new();
    let iters = solver_iterations.max(1);

    for iter in 0..iters {
        for i in 0..n {
            for j in (i + 1)..n {
                // Split slice into two disjoint mutable references
                let (left, right) = cars.split_at_mut(j);
                let car_i = &mut left[i];
                let car_j = &mut right[0];

                if let Some(mut ev) = resolve_car_car_collision(car_i, car_j, restitution, friction) {
                    if iter == 0 {
                        ev.car_a_idx = i;
                        ev.car_b_idx = j;
                        events.push(ev);
                    }
                }
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::config::CarConfig;

    #[test]
    fn test_head_on_car_car_elastic_collision() {
        let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(-1.0, 0.0), 0.0);
        let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(1.0, 0.0), std::f32::consts::PI);

        car_a.state.velocity = Vec2::new(10.0, 0.0);
        car_b.state.velocity = Vec2::new(-10.0, 0.0);

        let res = resolve_car_car_collision(&mut car_a, &mut car_b, 0.8, 0.3);
        assert!(res.is_some());

        // Both cars should reverse direction
        assert!(car_a.state.velocity.x < 0.0, "Car A must bounce back (-X)");
        assert!(car_b.state.velocity.x > 0.0, "Car B must bounce back (+X)");
    }

    #[test]
    fn test_multi_car_pileup_non_tunneling() {
        let mut cars = vec![
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0),
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(1.0, 0.0), 0.0),
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(2.0, 0.0), 0.0),
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(3.0, 0.0), 0.0),
        ];

        let _ = resolve_multi_car_collisions(&mut cars, 0.5, 0.3, 6);

        // Check that no cars remain overlapping
        for i in 0..cars.len() {
            for j in (i + 1)..cars.len() {
                let box_i = OrientedBox::from_car(&cars[i]);
                let box_j = OrientedBox::from_car(&cars[j]);
                let hit = collide_obb_obb(&box_i, &box_j);
                if let Some(m) = hit {
                    assert!(
                        m.penetration < 0.05,
                        "Overlap between car {i} and {j} must be resolved, got penetration {}",
                        m.penetration
                    );
                }
            }
        }
    }
}
