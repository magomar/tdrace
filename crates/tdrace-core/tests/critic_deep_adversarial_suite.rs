use glam::Vec2;
use std::f32::consts::PI;
use tdrace_core::collision::{
    collide_obb_obb, resolve_car_car_collision, resolve_car_wall_collision,
    resolve_multi_car_collisions, OrientedBox,
};
use tdrace_core::lidar::{LidarConfig, LidarHitType, LidarScanner};
use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::{BarrierType, WallBarrier};
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};

#[test]
fn test_adversarial_track_anti_cheat_scenarios() {
    let track = classic_grand_prix();
    let num_cps = track.checkpoints.len(); // 14 checkpoints (12 track + 2 pit)
    let mut tracker = TrackProgressTracker::new(num_cps, 3);
    let mut car = Car::new(CarConfig::sports_car());

    let dt = 0.016;
    let cp0 = &track.checkpoints[0];
    let mid0 = (cp0.gate.start + cp0.gate.end) * 0.5;

    // Scenario A: Reverse crossing of the finish line at race start
    // Car moves backwards across CP0
    car.state.position = mid0 + cp0.direction * 5.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    car.state.position = mid0 - cp0.direction * 5.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);

    assert!(tracker.is_wrong_way, "Reversing across finish line must trigger wrong-way flag");
    assert_eq!(tracker.current_lap, 1, "Lap count must NOT increase on backward crossing");
    assert!(!tracker.lap_completed, "Lap must not be completed on backward crossing");

    // Scenario B: Skipping 50% of checkpoints (cutting track infield)
    // Pass CP0, CP1, then skip directly to CP8, CP9, CP10, CP11, then CP0 (Finish)
    tracker.reset();

    // Cross CP0
    car.state.position = mid0 - cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    car.state.position = mid0 + cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    assert_eq!(tracker.last_checkpoint_idx, 0);

    // Cross CP1
    let cp1 = &track.checkpoints[1];
    let cp1_mid = (cp1.gate.start + cp1.gate.end) * 0.5;
    car.state.position = cp1_mid - cp1.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    car.state.position = cp1_mid + cp1.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    assert_eq!(tracker.last_checkpoint_idx, 1);

    // CUT TRACK: Skip CPs 2..7 and try to cross CP0
    car.state.position = mid0 - cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    car.state.position = mid0 + cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);

    // Anti-cheat verification: Lap must NOT count because checkpoints_passed < 70%
    assert_eq!(
        tracker.current_lap, 1,
        "Cheat shortcut must be rejected by anti-cheat gate rule"
    );
    assert!(!tracker.lap_completed);

    // Scenario C: Full legitimate lap sequence
    tracker.reset();
    let track_cps_count = 12; // 12 track checkpoints (excluding pit lane)
    for i in 0..track_cps_count {
        let cp = &track.checkpoints[i];
        let mid = (cp.gate.start + cp.gate.end) * 0.5;
        car.state.position = mid - cp.direction * 2.0;
        tracker.update(&car, &track.spline, &track.checkpoints, dt);
        car.state.position = mid + cp.direction * 2.0;
        tracker.update(&car, &track.spline, &track.checkpoints, dt);
        assert_eq!(tracker.last_checkpoint_idx, i);
    }
    // Cross finish line again to finish lap 1
    let cp0 = &track.checkpoints[0];
    let mid0 = (cp0.gate.start + cp0.gate.end) * 0.5;
    car.state.position = mid0 - cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    car.state.position = mid0 + cp0.direction * 2.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);

    assert!(tracker.lap_completed, "Legitimate lap must be accepted");
    assert_eq!(tracker.current_lap, 2);
    assert!(tracker.best_lap_time.is_some());
}

#[test]
fn test_high_speed_200kmh_head_on_wall_and_car_collisions() {
    // 200 km/h = 55.55 m/s
    let speed_200kmh = 200.0 / 3.6;

    // 1. Head-on car vs concrete wall at 200 km/h
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(speed_200kmh, 0.0);

    let concrete_wall = WallBarrier::new(
        Vec2::new(1.0, -15.0),
        Vec2::new(1.0, 15.0),
        BarrierType::Concrete,
    );

    let ev = resolve_car_wall_collision(&mut car, &concrete_wall);
    assert!(ev.is_some(), "Collision must be detected at 200 km/h");
    let event = ev.unwrap();
    assert!(event.impact_speed > 50.0);
    // Position must be pushed out to the left of the wall (x < 1.0)
    assert!(car.state.position.x < 1.0, "Car position must not penetrate wall");
    // Velocity must be reversed (negative X)
    assert!(car.state.velocity.x < 0.0, "Velocity must rebound");
    assert!(car.state.position.is_finite());
    assert!(car.state.velocity.is_finite());
    assert!(car.state.angular_velocity.is_finite());

    // 2. Head-on car-to-car collision at relative 400 km/h (each at 200 km/h)
    let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(-1.0, 0.0), 0.0);
    let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(1.0, 0.0), PI);

    car_a.state.velocity = Vec2::new(speed_200kmh, 0.0);
    car_b.state.velocity = Vec2::new(-speed_200kmh, 0.0);

    let c_ev = resolve_car_car_collision(&mut car_a, &mut car_b, 0.8, 0.3);
    assert!(c_ev.is_some(), "High-speed car-to-car collision must be detected");
    let event_cc = c_ev.unwrap();
    assert!(event_cc.closing_speed > 100.0, "Closing speed was {}", event_cc.closing_speed);

    // Both cars must rebound cleanly without NaNs
    assert!(car_a.state.velocity.x < 0.0, "Car A must rebound to -X");
    assert!(car_b.state.velocity.x > 0.0, "Car B must rebound to +X");
    assert!(car_a.state.position.x < car_b.state.position.x, "Cars must be separated");
    assert!(car_a.state.position.is_finite());
    assert!(car_b.state.position.is_finite());
}

#[test]
fn test_dense_16_car_starting_grid_pileup_singularity_test() {
    // 16 cars on starting grid in high-density 2-column formation
    let mut cars: Vec<Car> = (0..16)
        .map(|i| {
            let row = i / 2;
            let col = i % 2;
            let x = (row as f32) * 2.5;
            let y = if col == 0 { -1.5 } else { 1.5 };
            let heading = if row % 2 == 0 { 0.0 } else { 0.1 };
            Car::new(CarConfig::sports_car()).with_pose(Vec2::new(x, y), heading)
        })
        .collect();

    // Accelerate all cars into each other with random steer for 100 physics steps
    let dt = 1.0 / 60.0;
    let ctrl_fwd = CarControls::new(1.0, 0.2, 0.0, false);

    for step in 0..100 {
        for car in cars.iter_mut() {
            car.step(&ctrl_fwd, SurfaceType::Asphalt, dt);
        }
        let events = resolve_multi_car_collisions(&mut cars, 0.6, 0.3, 8);
        if step % 25 == 0 {
            println!("Step {step}: {} collision pairs resolved", events.len());
        }
    }

    // Verify all 16 cars have valid finite state and no extreme penetrations
    for i in 0..cars.len() {
        assert!(cars[i].state.position.is_finite(), "Car {i} position NaN");
        assert!(cars[i].state.velocity.is_finite(), "Car {i} velocity NaN");
        assert!(cars[i].state.angular_velocity.is_finite(), "Car {i} yaw NaN");
        assert!(cars[i].state.speed < 150.0, "Speed singularity exploded on car {i}: speed={}", cars[i].state.speed);

        for j in (i + 1)..cars.len() {
            let box_i = OrientedBox::from_car(&cars[i]);
            let box_j = OrientedBox::from_car(&cars[j]);
            if let Some(m) = collide_obb_obb(&box_i, &box_j) {
                assert!(
                    m.penetration < 0.35,
                    "Excessive penetration between car {i} and {j}: {}",
                    m.penetration
                );
            }
        }
    }
}

#[test]
fn test_split_mu_wheel_surface_dynamics_deep() {
    let track = classic_grand_prix();
    let sample = track.spline.sample_at_distance(60.0);
    let half_w = sample.width * 0.5;

    let heading = sample.tangent.y.atan2(sample.tangent.x);
    let car_right = Vec2::new(heading.sin(), -heading.cos());

    // Place car center right at track right edge
    let car_pos = sample.point + car_right * half_w;

    let car = Car::new(CarConfig::sports_car()).with_pose(car_pos, heading);
    let surfaces = track.sample_car_surfaces(&car);

    println!("Wheel surfaces straddling track right boundary: {:?}", surfaces);
    // Left wheels (FL surfaces[0], RL surfaces[2]) are offset inwards towards centerline (Asphalt)
    assert_eq!(surfaces[0], SurfaceType::Asphalt);
    assert_eq!(surfaces[2], SurfaceType::Asphalt);

    // Right wheels (FR surfaces[1], RR surfaces[3]) are offset outwards beyond boundary (Grass or Curb)
    assert_ne!(surfaces[1], SurfaceType::Asphalt);
    assert_ne!(surfaces[3], SurfaceType::Asphalt);

    // Test physics response: First accelerate straight on pure asphalt to 20 m/s
    let mut dynamic_car = Car::new(CarConfig::sports_car()).with_pose(car_pos, heading);
    for _ in 0..120 {
        dynamic_car.step(&CarControls::accelerate(), SurfaceType::Asphalt, 1.0 / 60.0);
    }
    let speed_before_split = dynamic_car.state.speed;
    println!("Speed before split-mu brake: {:.2} m/s", speed_before_split);
    assert!(speed_before_split > 10.0);

    // Now apply full brake with split surfaces [Asphalt, Grass/Ice, Asphalt, Grass/Ice]
    let split_surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Grass,
        SurfaceType::Asphalt,
        SurfaceType::Grass,
    ];
    let brake_ctrl = CarControls::new(0.0, 0.0, 1.0, false);
    for _ in 0..30 {
        dynamic_car.step_per_wheel(&brake_ctrl, split_surfaces, 1.0 / 60.0);
    }

    println!(
        "After split-mu braking: angular_velocity = {:.3} rad/s, speed = {:.2} m/s",
        dynamic_car.state.angular_velocity, dynamic_car.state.speed
    );

    assert!(
        dynamic_car.state.angular_velocity.abs() > 0.02,
        "Asymmetric split-mu surface braking must generate yaw rotational moment"
    );
}

#[test]
fn test_lidar_precision_all_presets_and_target_types() {
    let presets = [
        classic_grand_prix(),
        oval_speedway(),
        drift_park(),
        kart_arena(),
    ];

    let scanner = LidarScanner::new(LidarConfig::surround_32());

    for track in &presets {
        // Spawn car on the track centerline
        let sample = track.spline.sample_at_distance(20.0);
        let heading = sample.tangent.y.atan2(sample.tangent.x);
        let host = Car::new(CarConfig::sports_car()).with_pose(sample.point, heading);

        // Spawn opponent car 15m ahead along track
        let sample_ahead = track.spline.sample_at_distance(35.0);
        let opp = Car::new(CarConfig::sports_car()).with_pose(sample_ahead.point, heading);

        let hits = scanner.scan(&host, track, &[opp]);
        assert_eq!(hits.len(), 32);

        // Verify all hits are valid and bounded
        let mut hit_wall_count = 0;
        let mut hit_opp_count = 0;

        for hit in &hits {
            assert!(hit.distance >= 0.0 && hit.distance <= scanner.config.max_range);
            assert!(hit.normalized_distance >= 0.0 && hit.normalized_distance <= 1.0);
            assert!(hit.hit_point.is_finite());
            assert!(hit.hit_normal.is_finite());
            assert!(hit.relative_velocity.is_finite());

            match hit.hit_type {
                LidarHitType::TrackWall => hit_wall_count += 1,
                LidarHitType::OpponentCar => hit_opp_count += 1,
                _ => {}
            }
        }

        assert!(hit_wall_count > 0, "LIDAR must detect track walls on preset {}", track.name);
        assert!(hit_opp_count > 0, "LIDAR must detect opponent car on preset {}", track.name);
    }
}
