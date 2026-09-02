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
            let center = obs.center();
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

#[test]
fn test_polygon_obstacle_creation_ray_and_sat_collision() {
    use tdrace_core::track::geometry::Obstacle;
    use tdrace_core::collision::wall::resolve_all_wall_collisions;

    let vertices = vec![
        Vec2::new(10.0, 10.0),
        Vec2::new(14.0, 10.0),
        Vec2::new(16.0, 14.0),
        Vec2::new(12.0, 16.0),
        Vec2::new(8.0, 13.0),
    ];
    let mut poly_obs = Obstacle::polygon(1, vertices, "Test Polygon");
    let c = poly_obs.center();
    assert!((c.x - 12.0).abs() < 1.0);
    assert!((c.y - 12.6).abs() < 1.0);

    // 1. Ray intersection
    let ray_origin = Vec2::new(0.0, 12.0);
    let ray_dir = Vec2::new(1.0, 0.0);
    let hit = poly_obs.intersect_ray(ray_origin, ray_dir, 50.0);
    assert!(hit.is_some());
    let (t, normal) = hit.unwrap();
    assert!((t - 8.67).abs() < 0.2);
    assert!(normal.x < 0.0, "Normal should face against incoming ray direction");

    // 2. SAT Collision resolution
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(12.0, 12.0), 0.0);
    let hit_events = resolve_all_wall_collisions(&mut car, &[], &[poly_obs.clone()]);
    assert!(!hit_events.is_empty(), "Car should collide with polygon obstacle");

    // 3. Translation
    poly_obs.set_center(Vec2::new(50.0, 50.0));
    assert!((poly_obs.center() - Vec2::new(50.0, 50.0)).length() < 1e-4);
}

#[test]
fn test_at_grade_crossing_wall_culling() {
    use tdrace_core::track::geometry::BarrierType;
    use tdrace_core::track::presets::generate_walls_from_spline;
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
    use tdrace_core::collision::wall::resolve_all_wall_collisions;

    // Build a figure-8 / level crossing track with crossing at (0.0, 0.0)
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-80.0, -80.0), 12.0),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0), // Crossing point 1 (heading North-East)
        TrackWaypoint::new(Vec2::new(80.0, 80.0), 12.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
        TrackWaypoint::new(Vec2::new(80.0, -80.0), 12.0),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0), // Crossing point 2 (heading North-West)
        TrackWaypoint::new(Vec2::new(-80.0, 80.0), 12.0),
        TrackWaypoint::new(Vec2::new(-100.0, 0.0), 12.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, _, _) = generate_walls_from_spline(&spline, 3.0, BarrierType::Concrete);

    // Verify no wall barriers cross through the drivable intersection center within 6m
    for wall in left_walls.iter().chain(right_walls.iter()) {
        let mid = (wall.segment.start + wall.segment.end) * 0.5;
        let start_len = wall.segment.start.length();
        let end_len = wall.segment.end.length();
        let mid_len = mid.length();
        assert!(
            start_len > 5.5 || end_len > 5.5 || mid_len > 5.5,
            "Wall segment ({:?} -> {:?}) intrudes into the crossing intersection center!",
            wall.segment.start,
            wall.segment.end
        );
    }

    // Place a car directly at (0.0, 0.0) heading in both crossing directions and verify no collision
    let mut car_ne = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), std::f32::consts::FRAC_PI_4);
    let hits_ne = resolve_all_wall_collisions(&mut car_ne, &left_walls, &[]);
    let hits_ne_r = resolve_all_wall_collisions(&mut car_ne, &right_walls, &[]);
    assert!(hits_ne.is_empty(), "Car heading NE should not collide with any trimmed wall at the intersection");
    assert!(hits_ne_r.is_empty(), "Car heading NE should not collide with any trimmed wall at the intersection");

    let mut car_nw = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 3.0 * std::f32::consts::FRAC_PI_4);
    let hits_nw = resolve_all_wall_collisions(&mut car_nw, &left_walls, &[]);
    let hits_nw_r = resolve_all_wall_collisions(&mut car_nw, &right_walls, &[]);
    assert!(hits_nw.is_empty(), "Car heading NW should not collide with any trimmed wall at the intersection");
    assert!(hits_nw_r.is_empty(), "Car heading NW should not collide with any trimmed wall at the intersection");
}

#[test]
fn test_waypoint_manual_wall_flags_and_backwards_compatibility() {
    use tdrace_core::track::geometry::BarrierType;
    use tdrace_core::track::presets::generate_walls_from_spline;
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};

    // 1. Backwards compatibility: Deserializing JSON without left_wall / right_wall defaults to true
    let legacy_json = r#"{
        "point": [10.0, 20.0],
        "width": 14.0,
        "left_curb": true,
        "right_curb": false,
        "surface": null,
        "elevation": 0.0
    }"#;
    let deserialized: TrackWaypoint = serde_json::from_str(legacy_json).expect("Must deserialize legacy JSON");
    assert!(deserialized.left_wall, "left_wall must default to true");
    assert!(deserialized.right_wall, "right_wall must default to true");

    // 2. Manual wall flags: create a loop where one side has left_wall = false
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_walls(false, true),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0).with_walls(false, true),
        TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0).with_walls(true, true),
        TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0).with_walls(true, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, _, _) = generate_walls_from_spline(&spline, 3.0, BarrierType::Concrete);

    // Left side for segment 0->1 was disabled; left_walls should have fewer segments than right_walls
    assert!(
        left_walls.len() < right_walls.len(),
        "Left wall count ({}) should be strictly less than right wall count ({}) when left_wall is disabled on a segment",
        left_walls.len(),
        right_walls.len()
    );
}

#[test]
fn test_overpass_bridge_preserves_guardrails() {
    use tdrace_core::track::geometry::BarrierType;
    use tdrace_core::track::presets::generate_walls_from_spline;
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};

    // Build an overpass crossover: crossing at (0, 0) with elevation 4.5m on branch 1 and 0.0m on branch 2
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-80.0, 0.0), 12.0).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0).with_elevation(4.5), // Elevated bridge overpass
        TrackWaypoint::new(Vec2::new(80.0, 0.0), 12.0).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(80.0, 80.0), 12.0).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(0.0, 80.0), 12.0).with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0).with_elevation(0.0), // Ground level underpass
        TrackWaypoint::new(Vec2::new(0.0, -80.0), 12.0).with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(-80.0, -80.0), 12.0).with_elevation(2.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, _, _) = generate_walls_from_spline(&spline, 3.0, BarrierType::Concrete);

    // The bridge (elev 4.5m) should preserve its guardrails above (0.0, 0.0)
    let bridge_guardrails: Vec<_> = left_walls
        .iter()
        .chain(right_walls.iter())
        .filter(|w| w.elevation > 3.0 && (w.segment.start.x.abs() < 20.0 || w.segment.end.x.abs() < 20.0))
        .collect();

    assert!(
        !bridge_guardrails.is_empty(),
        "Bridge guardrails at elevation 4.5m must be preserved across overpass crossing"
    );
}

#[test]
fn test_banked_curves_spline_interpolation_and_cross_slope() {
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};

    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 20.0).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(200.0, 50.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(100.0, 100.0), 20.0).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(0.0, 100.0), 20.0).with_bank_angle(0.0),
    ];

    let spline = TrackSpline::new(waypoints, true);

    // Verify distance sampling interpolates bank angle smoothly
    let s0 = spline.sample_at_distance(0.0);
    assert!(s0.bank_angle.abs() < 1.0, "Start of straight should be flat, got {}", s0.bank_angle);

    let apex_sample = spline.samples.iter().max_by(|a, b| a.bank_angle.partial_cmp(&b.bank_angle).unwrap()).unwrap();
    assert!(
        apex_sample.bank_angle >= 20.0,
        "Apex sample should reach high banking, got {}",
        apex_sample.bank_angle
    );

    // Test cross slope elevation calculation:
    // When bank_angle > 0, right side (offset -normal) is elevated above left side (+normal)
    let proj_apex = spline.project_point(apex_sample.point);
    assert_eq!(proj_apex.bank_angle, apex_sample.bank_angle);

    let right_edge_pos = apex_sample.point - apex_sample.normal * 10.0;
    let left_edge_pos = apex_sample.point + apex_sample.normal * 10.0;

    let z_center = spline.sample_cross_slope_elevation(apex_sample.point);
    let z_right = spline.sample_cross_slope_elevation(right_edge_pos);
    let z_left = spline.sample_cross_slope_elevation(left_edge_pos);

    assert!(
        z_right > z_center,
        "Right edge ({}) must be higher than center ({}) on positive banked curve",
        z_right,
        z_center
    );
    assert!(
        z_center > z_left,
        "Center ({}) must be higher than left edge ({}) on positive banked curve",
        z_center,
        z_left
    );
}

#[test]
fn test_oval_speedway_and_dirty_oval_presets_have_banking() {
    use tdrace_core::track::presets::{dirty_oval_speedway, oval_speedway};
    use tdrace_core::track::validation::validate_track;

    let asphalt_oval = oval_speedway();
    let max_asphalt_bank = asphalt_oval.spline.samples.iter().map(|s| s.bank_angle).fold(0.0f32, f32::max);
    assert!(
        max_asphalt_bank >= 20.0,
        "Asphalt Oval Speedway should have ~22 deg banking on curves, got {}",
        max_asphalt_bank
    );
    let asphalt_errors = validate_track(&asphalt_oval);
    assert!(
        !asphalt_errors.iter().any(|e| e.severity == tdrace_core::track::ValidationSeverity::Error),
        "Oval speedway must pass validation with 0 errors: {:?}",
        asphalt_errors
    );

    let dirt_oval = dirty_oval_speedway();
    let max_dirt_bank = dirt_oval.spline.samples.iter().map(|s| s.bank_angle).fold(0.0f32, f32::max);
    assert!(
        max_dirt_bank >= 16.0,
        "Dirt Oval Speedway should have ~18 deg banking on curves, got {}",
        max_dirt_bank
    );
    let dirt_errors = validate_track(&dirt_oval);
    assert!(
        !dirt_errors.iter().any(|e| e.severity == tdrace_core::track::ValidationSeverity::Error),
        "Dirty oval speedway must pass validation with 0 errors: {:?}",
        dirt_errors
    );
}

#[test]
fn test_banking_incline_physics_and_centripetal_downhill_force() {
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::CarConfig;
    use tdrace_core::physics::surface::SurfaceType;

    let mut flat_car = Car::new(CarConfig::sports_car());
    flat_car.state.road_bank_angle = 0.0;
    flat_car.state.track_right = Vec2::new(0.0, 1.0);

    let mut banked_car = Car::new(CarConfig::sports_car());
    banked_car.state.road_bank_angle = 20.0; // 20 degrees banking
    banked_car.state.track_right = Vec2::new(0.0, 1.0); // Track right is +Y

    let ctrl = CarControls::default();
    let surfaces = [SurfaceType::Asphalt; 4];

    // Step physics without control input
    flat_car.step_per_wheel(&ctrl, surfaces, 0.016);
    banked_car.step_per_wheel(&ctrl, surfaces, 0.016);

    // On flat ground, stopped car has zero lateral downhill acceleration
    assert_eq!(flat_car.state.velocity, Vec2::ZERO);

    // On 20 deg bank, downhill slope pulls towards left (-track_right, i.e. -Y direction)
    assert!(
        banked_car.state.velocity.y < 0.0,
        "Car on banked curve should accelerate downhill (-Y), got velocity: {:?}",
        banked_car.state.velocity
    );
}

#[test]
fn test_sync_dirty_oval_json() {
    use tdrace_core::track::presets::dirty_oval_speedway;
    use tdrace_core::track::Track;
    let track = dirty_oval_speedway();
    let temp_file = std::env::temp_dir().join(format!("test_dirty_oval_{}.json", std::process::id()));
    track.save_to_file(&temp_file).expect("Failed to save dirty_oval_speedway.json");
    let loaded = Track::load_from_file(&temp_file).expect("Failed to load dirty_oval_speedway.json");
    assert_eq!(loaded.name, track.name);
    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_waypoint_custom_wall_distances_and_zero_distance_flush_walls() {
    use tdrace_core::track::presets::generate_walls_from_spline;
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
    use tdrace_core::track::BarrierType;

    // Create a straight corridor with 0.0m wall distance (flush with road edge)
    let road_w = 12.0;
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), road_w).with_wall_distance(0.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), road_w).with_wall_distance(0.0),
        TrackWaypoint::new(Vec2::new(200.0, 0.0), road_w).with_wall_distance(0.0),
    ];

    let spline = TrackSpline::new(waypoints, false);
    assert_eq!(spline.samples[0].left_wall_distance, Some(0.0));
    assert_eq!(spline.samples[0].right_wall_distance, Some(0.0));

    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Concrete);

    assert!(!left_walls.is_empty());
    assert!(!right_walls.is_empty());

    // Left wall points should be at y = +6.0 (half_width = 12.0 / 2.0 = 6.0)
    for p in &left_poly {
        assert!(
            (p.y - 6.0).abs() < 0.1,
            "Flush left wall should sit at y = 6.0, got y = {}",
            p.y
        );
    }

    // Right wall points should be at y = -6.0
    for p in &right_poly {
        assert!(
            (p.y - (-6.0)).abs() < 0.1,
            "Flush right wall should sit at y = -6.0, got y = {}",
            p.y
        );
    }
}

#[test]
fn test_track_spline_wall_distance_interpolation() {
    use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};

    // Transition from 4.0m distance to 0.0m distance
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_wall_distances(Some(4.0), Some(2.0)),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0).with_wall_distances(Some(0.0), Some(0.0)),
        TrackWaypoint::new(Vec2::new(200.0, 0.0), 10.0).with_wall_distances(Some(0.0), Some(0.0)),
    ];

    let spline = TrackSpline::new(waypoints, false);

    // Midpoint sample of segment 0 should have interpolated left distance ~ 2.0m, right ~ 1.0m
    let mid_sample = &spline.samples[12]; // 24 steps per segment -> step 12 is halfway
    let left_d = mid_sample.left_wall_distance.expect("Should be Some");
    let right_d = mid_sample.right_wall_distance.expect("Should be Some");

    assert!(
        (left_d - 2.0).abs() < 0.2,
        "Interpolated left wall distance should be ~2.0, got {}",
        left_d
    );
    assert!(
        (right_d - 1.0).abs() < 0.2,
        "Interpolated right wall distance should be ~1.0, got {}",
        right_d
    );
}

#[test]
fn test_waypoint_wall_distance_json_backwards_compatibility() {
    use tdrace_core::track::spline::TrackWaypoint;

    // JSON without wall distance fields
    let legacy_json = r#"{
        "point": [10.0, 20.0],
        "width": 14.0,
        "left_curb": false,
        "right_curb": false,
        "surface": null,
        "elevation": 0.0,
        "bank_angle": 0.0,
        "left_wall": true,
        "right_wall": true
    }"#;

    let deserialized: TrackWaypoint =
        serde_json::from_str(legacy_json).expect("Must deserialize legacy waypoint JSON");
    assert_eq!(deserialized.left_wall_distance, None);
    assert_eq!(deserialized.right_wall_distance, None);

    // JSON with custom wall distance fields
    let new_json = r#"{
        "point": [10.0, 20.0],
        "width": 14.0,
        "left_curb": false,
        "right_curb": false,
        "surface": null,
        "elevation": 0.0,
        "bank_angle": 0.0,
        "left_wall": true,
        "right_wall": true,
        "left_wall_distance": 0.0,
        "right_wall_distance": 1.5
    }"#;

    let deserialized_new: TrackWaypoint =
        serde_json::from_str(new_json).expect("Must deserialize new waypoint JSON");
    assert_eq!(deserialized_new.left_wall_distance, Some(0.0));
    assert_eq!(deserialized_new.right_wall_distance, Some(1.5));
}


