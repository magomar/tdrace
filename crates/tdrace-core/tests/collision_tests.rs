use std::f32::consts::PI;
use glam::Vec2;
use tdrace_core::collision::{
    collide_obb_obb, resolve_car_car_collision, resolve_car_obstacle_collision,
    resolve_car_wall_collision, resolve_multi_car_collisions, OrientedBox,
};
use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};
use tdrace_core::track::geometry::{BarrierType, Obstacle, WallBarrier};

#[test]
fn test_wall_restitution_comparison_concrete_vs_tire_wall() {
    let mut car_concrete = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car_concrete.state.velocity = Vec2::new(20.0, 0.0);

    let wall_concrete = WallBarrier::new(
        Vec2::new(1.0, -10.0),
        Vec2::new(1.0, 10.0),
        BarrierType::Concrete, // e = 0.65
    );

    let ev_conc = resolve_car_wall_collision(&mut car_concrete, &wall_concrete);
    assert!(ev_conc.is_some());
    let v_out_concrete = car_concrete.state.velocity.x.abs();

    let mut car_tire = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car_tire.state.velocity = Vec2::new(20.0, 0.0);

    let wall_tire = WallBarrier::new(
        Vec2::new(1.0, -10.0),
        Vec2::new(1.0, 10.0),
        BarrierType::TireWall, // e = 0.20
    );

    let ev_tire = resolve_car_wall_collision(&mut car_tire, &wall_tire);
    assert!(ev_tire.is_some());
    let v_out_tire = car_tire.state.velocity.x.abs();

    assert!(
        v_out_concrete > v_out_tire,
        "Concrete wall ({}) must bounce harder than tire wall ({})",
        v_out_concrete,
        v_out_tire
    );
}

#[test]
fn test_oblique_wall_bounce_angle_and_friction() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    // 45 degree approach into a vertical wall at x = 1.0
    car.state.velocity = Vec2::new(15.0, 15.0);

    let wall = WallBarrier::with_physics(
        Vec2::new(1.0, -10.0),
        Vec2::new(1.0, 10.0),
        BarrierType::Armco,
        0.60,
        0.40,
    );

    let ev = resolve_car_wall_collision(&mut car, &wall);
    assert!(ev.is_some());

    // Normal velocity (X) reversed
    assert!(car.state.velocity.x < 0.0, "Velocity X must reverse");
    // Tangential velocity (Y) reduced by friction
    assert!(
        car.state.velocity.y < 15.0,
        "Tangential velocity Y must be reduced by friction, got {}",
        car.state.velocity.y
    );
    assert!(
        car.state.velocity.y > 0.0,
        "Tangential velocity should remain positive along wall"
    );
}

#[test]
fn test_corner_clip_yaw_deflection() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.15);
    car.state.velocity = Vec2::new(20.0, 2.0);

    let wall = WallBarrier::new(
        Vec2::new(-5.0, 0.65),
        Vec2::new(5.0, 0.65),
        BarrierType::Armco,
    );

    let res = resolve_car_wall_collision(&mut car, &wall);
    assert!(res.is_some());
    assert!(
        car.state.angular_velocity.abs() > 0.2,
        "Angular velocity must be induced on corner wall clip, got {}",
        car.state.angular_velocity
    );
}

#[test]
fn test_car_to_car_head_on_momentum_conservation() {
    let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(-0.8, 0.0), 0.0);
    let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.8, 0.0), PI);

    let m_a = car_a.config.mass;
    let m_b = car_b.config.mass;

    car_a.state.velocity = Vec2::new(12.0, 0.0);
    car_b.state.velocity = Vec2::new(-8.0, 0.0);

    let initial_momentum = car_a.state.velocity * m_a + car_b.state.velocity * m_b;

    let res = resolve_car_car_collision(&mut car_a, &mut car_b, 0.9, 0.0);
    assert!(res.is_some());

    let final_momentum = car_a.state.velocity * m_a + car_b.state.velocity * m_b;
    let diff = (final_momentum - initial_momentum).length();
    assert!(
        diff < 1e-3,
        "Linear momentum must be conserved, diff = {}",
        diff
    );

    assert!(car_a.state.velocity.x < 0.0);
    assert!(car_b.state.velocity.x > 0.0);
}

#[test]
fn test_car_to_car_t_bone_collision_yaw_spin() {
    // Car A travelling East (+X), Car B travelling North (+Y) colliding at origin
    let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.5, -0.6), PI * 0.5);

    car_a.state.velocity = Vec2::new(15.0, 0.0);
    car_b.state.velocity = Vec2::new(0.0, 15.0);

    let res = resolve_car_car_collision(&mut car_a, &mut car_b, 0.5, 0.3);
    assert!(res.is_some());

    assert!(
        car_a.state.angular_velocity.abs() > 0.1 || car_b.state.angular_velocity.abs() > 0.1,
        "T-bone collision must induce rotation"
    );
}

#[test]
fn test_multi_car_pileup_high_speed_non_tunneling() {
    let num_cars = 6;
    let mut cars = Vec::with_capacity(num_cars);

    // Spawn 6 cars tightly packed heading toward each other
    for i in 0..num_cars {
        let x = (i as f32) * 0.6;
        let heading = if i % 2 == 0 { 0.0 } else { PI };
        let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(x, 0.0), heading);
        car.state.velocity = Vec2::new(if i % 2 == 0 { 25.0 } else { -25.0 }, 0.0);
        cars.push(car);
    }

    // Step physics & collision resolution over 20 sub-frames
    let dt = 1.0 / 120.0;
    let ctrl = CarControls::accelerate();

    for _ in 0..20 {
        for car in cars.iter_mut() {
            car.step(&ctrl, SurfaceType::Asphalt, dt);
        }
        resolve_multi_car_collisions(&mut cars, 0.4, 0.3, 8);
    }

    // Verify no catastrophic overlaps or NaN values
    for i in 0..cars.len() {
        assert!(cars[i].state.position.is_finite());
        assert!(cars[i].state.velocity.is_finite());
        for j in (i + 1)..cars.len() {
            let box_i = OrientedBox::from_car(&cars[i]);
            let box_j = OrientedBox::from_car(&cars[j]);
            if let Some(m) = collide_obb_obb(&box_i, &box_j) {
                assert!(
                    m.penetration < 0.20,
                    "Residual penetration between car {} and {} must be minimal, got {}",
                    i,
                    j,
                    m.penetration
                );
            }
        }
    }
}

#[test]
fn test_static_obstacle_collision() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(-1.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(15.0, 0.0);

    let obstacle = Obstacle::circle(1, Vec2::new(0.5, 0.0), 0.8, "Tire Stack");

    let hit = resolve_car_obstacle_collision(&mut car, &obstacle);
    assert!(hit.is_some());
    assert!(car.state.velocity.x < 0.0, "Car must bounce off obstacle");
}

#[test]
fn test_wall_contact_braking_resistance_when_running_alongside() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(25.0, 0.0);

    // Wall running parallel to X axis at y = 0.82 (car half width is ~0.85, so touching)
    let wall = WallBarrier::new(
        Vec2::new(-20.0, 0.82),
        Vec2::new(20.0, 0.82),
        BarrierType::Concrete,
    );

    let hit = resolve_car_wall_collision(&mut car, &wall);
    assert!(hit.is_some(), "Car should be in contact with the wall");
    let ev = hit.unwrap();
    assert!(ev.friction_impulse > 0.0, "Friction/braking impulse must be applied when touching wall");
    assert!(car.state.velocity.x < 25.0, "Car forward speed must be reduced by wall braking resistance");
}

#[test]
fn test_rubber_tyres_wall_braking_resistance_stronger_than_concrete() {
    let mut car_concrete = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car_concrete.state.velocity = Vec2::new(25.0, 0.0);

    let wall_concrete = WallBarrier::new(
        Vec2::new(-20.0, 0.82),
        Vec2::new(20.0, 0.82),
        BarrierType::Concrete,
    );

    let _ = resolve_car_wall_collision(&mut car_concrete, &wall_concrete);
    let speed_loss_concrete = 25.0 - car_concrete.state.velocity.x;

    let mut car_tire = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car_tire.state.velocity = Vec2::new(25.0, 0.0);

    let wall_tire = WallBarrier::new(
        Vec2::new(-20.0, 0.82),
        Vec2::new(20.0, 0.82),
        BarrierType::TireWall,
    );

    let _ = resolve_car_wall_collision(&mut car_tire, &wall_tire);
    let speed_loss_tire = 25.0 - car_tire.state.velocity.x;

    assert!(
        speed_loss_tire > speed_loss_concrete,
        "Rubber tyre wall braking resistance ({}) must exceed concrete wall resistance ({})",
        speed_loss_tire,
        speed_loss_concrete
    );
}
