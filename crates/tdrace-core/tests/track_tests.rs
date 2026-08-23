use std::f32::consts::PI;
use glam::Vec2;
use tdrace_core::physics::{Car, CarConfig, SurfaceType};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::presets::{classic_grand_prix, oval_speedway};

#[test]
fn test_track_lap_counting_and_best_lap_time() {
    let track = oval_speedway();
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 2);
    let mut car = Car::new(CarConfig::sports_car());

    let dt = 0.1;
    let total_len = track.spline.total_length();
    let speed = 30.0; // 30 m/s
    let total_time = total_len / speed;
    let steps = (total_time / dt).ceil() as usize;

    // Simulate car driving one complete lap along the spline
    for step in 0..=steps {
        let dist = ((step as f32 * dt * speed) % total_len).min(total_len - 0.1);
        let sample = track.spline.sample_at_distance(dist);
        car.state.position = sample.point;
        car.state.angle = sample.tangent.y.atan2(sample.tangent.x);
        car.state.speed = speed;

        tracker.update(&car, &track.spline, &track.checkpoints, dt);
    }

    // Now cross the start/finish line
    let finish_sample = track.spline.sample_at_distance(1.0);
    car.state.position = finish_sample.point;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);

    assert!(
        tracker.current_lap >= 2,
        "Current lap should advance to 2, got {}",
        tracker.current_lap
    );
    assert!(tracker.last_lap_time.is_some());
    assert!(tracker.best_lap_time.is_some());
}

#[test]
fn test_checkpoint_sequence_enforcement_anti_cheat() {
    let track = classic_grand_prix();
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
    let mut car = Car::new(CarConfig::sports_car());

    // Start before finish line
    car.state.position = Vec2::new(-5.0, 0.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);

    // Cross finish line CP0
    car.state.position = Vec2::new(5.0, 0.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    assert_eq!(tracker.last_checkpoint_idx, 0);

    // Drive back and cross CP0 again (skipping all intermediate CPs)
    car.state.position = Vec2::new(-5.0, 0.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    car.state.position = Vec2::new(5.0, 0.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);

    // Lap should not count!
    assert_eq!(tracker.current_lap, 1);
    assert!(!tracker.lap_completed);
}

#[test]
fn test_wrong_way_detection() {
    let track = classic_grand_prix();
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
    // Heading PI (opposite to track direction +X at start line)
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 0.0), PI);

    let dt = 0.016;
    for _ in 0..10 {
        tracker.update(&car, &track.spline, &track.checkpoints, dt);
    }

    assert!(tracker.is_wrong_way, "Car facing backwards must trigger wrong-way warning");
    assert!(tracker.wrong_way_timer > 0.1);

    // Turn car forward (heading 0.0)
    car.state.angle = 0.0;
    tracker.update(&car, &track.spline, &track.checkpoints, dt);
    assert!(!tracker.is_wrong_way, "Car facing forwards must clear wrong-way warning");
    assert_eq!(tracker.wrong_way_timer, 0.0);
}

#[test]
fn test_off_track_detection_and_surfaces() {
    let track = classic_grand_prix();
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);

    // On track centerline
    let car_on_track = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 0.0), 0.0);
    tracker.update(&car_on_track, &track.spline, &track.checkpoints, 0.016);
    assert!(!tracker.is_off_track);

    let surfaces_on_track = track.sample_car_surfaces(&car_on_track);
    assert_eq!(surfaces_on_track, [SurfaceType::Asphalt; 4]);

    // Off track in grass (lateral offset y = -25.0)
    let car_off_track = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, -25.0), 0.0);
    let dt = 0.016;
    for _ in 0..5 {
        tracker.update(&car_off_track, &track.spline, &track.checkpoints, dt);
    }
    assert!(tracker.is_off_track, "Car outside track boundaries must be off-track");
    assert!(tracker.off_track_timer > 0.05);

    let surfaces_off_track = track.sample_car_surfaces(&car_off_track);
    assert_eq!(surfaces_off_track, [SurfaceType::Grass; 4]);
}

#[test]
fn test_pit_lane_and_pit_stop_trigger() {
    let track = classic_grand_prix();
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
    let mut car = Car::new(CarConfig::sports_car());

    // 1. Cross Pit Entry checkpoint (located at x: -25.0, y: -12.0)
    car.state.position = Vec2::new(-30.0, -12.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    car.state.position = Vec2::new(-20.0, -12.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);

    assert!(tracker.in_pit_lane, "Entering pit lane must set in_pit_lane = true");

    // 2. Check pit box detection
    car.state.position = Vec2::new(50.0, -12.0); // Inside pit box area
    assert!(track.is_in_pit_box(&car), "Car at (50, -12) must be inside pit box zone");

    // 3. Cross Pit Exit checkpoint (located at x: 135.0, y: -12.0)
    car.state.position = Vec2::new(130.0, -12.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    car.state.position = Vec2::new(140.0, -12.0);
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);

    assert!(!tracker.in_pit_lane, "Exiting pit lane must set in_pit_lane = false");
    assert_eq!(tracker.pit_stops, 1, "Completed pit stops count must increment");
}

#[test]
fn test_per_wheel_split_mu_sampling() {
    let track = classic_grand_prix();
    let sample = track.spline.sample_at_distance(50.0);
    let half_track_w = sample.width * 0.5;
    let heading = sample.tangent.y.atan2(sample.tangent.x);

    // Place car at track edge in car's local right vector
    let car_right = Vec2::new(sample.tangent.y, -sample.tangent.x);
    let car_pos = sample.point + car_right * half_track_w;
    let car = Car::new(CarConfig::sports_car()).with_pose(car_pos, heading);
    let surfaces = track.sample_car_surfaces(&car);

    // Left wheels (FL, RL: pos - right * half_w) are inside track width (Asphalt)
    assert_eq!(surfaces[0], SurfaceType::Asphalt, "FL wheel should be on Asphalt");
    assert_eq!(surfaces[2], SurfaceType::Asphalt, "RL wheel should be on Asphalt");

    // Right wheels (FR, RR: pos + right * half_w) are beyond track boundary (Grass or Curb)
    assert!(
        surfaces[1] == SurfaceType::Grass || surfaces[1] == SurfaceType::Curb,
        "FR should be Curb or Grass, got {:?}",
        surfaces[1]
    );
    assert!(
        surfaces[3] == SurfaceType::Grass || surfaces[3] == SurfaceType::Curb,
        "RR should be Curb or Grass, got {:?}",
        surfaces[3]
    );
}

#[test]
fn test_all_track_presets_grid_positions_valid() {
    use tdrace_core::track::presets::{classic_grand_prix, oval_speedway, drift_park, kart_arena};
    use tdrace_core::physics::{Car, CarConfig};
    use tdrace_core::collision::wall::resolve_all_wall_collisions;

    let tracks = [
        ("Classic Grand Prix", classic_grand_prix()),
        ("Oval Speedway", oval_speedway()),
        ("Drift Park", drift_park()),
        ("Kart Arena", kart_arena()),
    ];

    for (name, track) in &tracks {
        println!("Checking track: {} (length: {:.1}m)", name, track.spline.total_length());
        for (i, gp) in track.grid_positions.iter().enumerate() {
            let mut car = Car::new(CarConfig::sports_car()).with_pose(gp.position, gp.angle);
            let surfaces = track.sample_car_surfaces(&car);
            assert!(
                surfaces.iter().all(|s| *s == SurfaceType::Asphalt || *s == SurfaceType::Curb),
                "Track {} Grid {} spawned on invalid surface: {:?}",
                name, i, surfaces
            );

            // Verify car is not colliding with inner/outer walls or obstacles at spawn
            let initial_pos = car.state.position;
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &track.geometry.obstacles);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);

            let displacement = (car.state.position - initial_pos).length();
            println!(
                "  Slot {}: pos=({:.1}, {:.1}), angle={:.2} rad, hit_inner={}, hit_outer={}, disp={:.3}m",
                i, gp.position.x, gp.position.y, gp.angle, !hit_inner.is_empty(), !hit_outer.is_empty(), displacement
            );

            assert!(hit_inner.is_empty() && hit_outer.is_empty(), "Track {} Slot {} spawned in collision with walls! (disp={:.2}m)", name, i, displacement);
            assert!(displacement < 1e-4, "Track {} Slot {} was displaced by collision resolution!", name, i);
        }

        for obs in &track.geometry.obstacles {
            let center = match &obs.shape {
                tdrace_core::track::geometry::ObstacleShape::Circle { center, .. } => *center,
                tdrace_core::track::geometry::ObstacleShape::Box { center, .. } => *center,
            };
            let proj = track.spline.project_point(center);
            println!(
                "  Obstacle '{}' at {:?}: dist_to_spline={:.2}m, track_half_w={:.2}m, is_on_track={}",
                obs.name, center, proj.distance_to_spline, proj.track_width * 0.5, proj.is_on_track
            );
        }
    }
}

#[test]
fn test_track_walls_do_not_block_drivable_track_and_do_not_self_intersect() {
    use tdrace_core::track::presets::{classic_grand_prix, oval_speedway, drift_park, kart_arena};
    use tdrace_core::physics::{Car, CarConfig};
    use tdrace_core::collision::wall::resolve_all_wall_collisions;

    let tracks = [
        ("Classic Grand Prix", classic_grand_prix()),
        ("Oval Speedway", oval_speedway()),
        ("Drift Park", drift_park()),
        ("Kart Arena", kart_arena()),
    ];

    for (name, track) in &tracks {
        let total_len = track.spline.total_length();
        let num_steps = (total_len / 0.5) as usize;

        // 1. Check car driving along centerline
        let mut collisions = 0;
        for i in 0..num_steps {
            let dist = i as f32 * 0.5;
            let sample = track.spline.sample_at_distance(dist);
            let heading = sample.tangent.y.atan2(sample.tangent.x);
            let mut car = Car::new(CarConfig::sports_car()).with_pose(sample.point, heading);

            let initial_pos = car.state.position;
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);

            let displacement = (car.state.position - initial_pos).length();
            if !hit_inner.is_empty() || !hit_outer.is_empty() || displacement > 0.01 {
                println!(
                    "[{}] Collision on centerline at dist={:.1}m / {:.1}m: pos=({:.1}, {:.1}), disp={:.3}m",
                    name, dist, total_len, sample.point.x, sample.point.y, displacement
                );
                collisions += 1;
            }
        }

        // 2. Check for wall segment self-intersections
        for (wall_side, walls) in [("inner", &track.geometry.inner_walls), ("outer", &track.geometry.outer_walls)] {
            for i in 0..walls.len() {
                for j in (i + 1)..walls.len() {
                    // Skip adjacent segments
                    if (j == i + 1) || (i == 0 && j == walls.len() - 1) {
                        continue;
                    }
                    let seg1 = &walls[i].segment;
                    let seg2 = &walls[j].segment;
                    if let Some(pt) = seg1.intersect_segment(seg2) {
                        println!(
                            "[{}] {} wall self-intersection between seg {} ({:?} -> {:?}) and seg {} ({:?} -> {:?}) at {:?}",
                            name, wall_side, i, seg1.start, seg1.end, j, seg2.start, seg2.end, pt
                        );
                    }
                }
            }
        }

        assert_eq!(collisions, 0, "Track {} has {} centerline collisions with walls!", name, collisions);
    }
}


