use glam::Vec2;
use tdrace_core::collision::wall::resolve_car_wall_collision;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, SurfaceShape, WallBarrier};
use tdrace_core::track::presets::{dune_raid, oasis_rally, outlaw_pass, ramp_raceway, sahara_dunes};

#[test]
fn test_car_jump_launch_and_gravity_arc() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::ZERO, 0.0);
    car.state.velocity = Vec2::new(25.0, 0.0);

    let ramp = JumpRamp::new(
        1,
        SurfaceShape::Aabb {
            min: Vec2::new(-5.0, -5.0),
            max: Vec2::new(5.0, 5.0),
        },
        Vec2::new(1.0, 0.0),
        4.0,
        20.0,
        2.5,
        "Test Ramp",
    );

    let triggered = car.try_trigger_jump_ramp(&ramp);
    assert!(triggered, "Car at speed should trigger jump ramp");
    assert!(car.state.is_airborne, "Car should be airborne");
    assert!(car.state.elevation > 0.0, "Car elevation must be positive");
    assert!(car.state.vertical_velocity > 5.0, "Car vertical velocity must be positive");
    assert_eq!(car.state.jump_count, 1);

    // Step physics forward while airborne
    let ctrl = CarControls::accelerate();
    let mut apex_elevation = 0.0f32;
    let mut steps_to_landing = 0;

    for _ in 0..120 {
        car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        if car.state.elevation > apex_elevation {
            apex_elevation = car.state.elevation;
        }
        if car.state.just_landed {
            break;
        }
        steps_to_landing += 1;
    }

    assert!(apex_elevation > 1.5, "Apex elevation should exceed 1.5m, got {}", apex_elevation);
    assert!(car.state.just_landed, "Car should have landed");
    assert_eq!(car.state.elevation, 0.0);
    assert_eq!(car.state.vertical_velocity, 0.0);
    assert!(!car.state.is_airborne);
    assert!(steps_to_landing > 30, "Jump should last multiple frames, took {} steps", steps_to_landing);
}

#[test]
fn test_airborne_grip_attenuation() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::ZERO, 0.0);
    car.state.velocity = Vec2::new(20.0, 0.0);
    car.state.elevation = 2.0; // High in the air
    car.state.is_airborne = true;

    // Command full steering and braking in mid-air
    let ctrl = CarControls::new(0.0, 1.0, 1.0, true);
    let initial_speed = car.state.velocity.length();
    car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);

    // Speed should barely drop (only minimal air drag, no ground braking)
    assert!(
        (car.state.speed - initial_speed).abs() < 0.2,
        "Ground brakes should not stop an airborne car in mid-air"
    );
    assert_eq!(car.state.wheels[0].skid_intensity, 0.0, "No tire skidding in mid-air");
}

#[test]
fn test_jump_over_low_wall_no_collision() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(20.0, 0.0);
    car.state.elevation = 2.0; // Airborne above wall

    let wall = WallBarrier::new(Vec2::new(10.0, -5.0), Vec2::new(10.0, 5.0), BarrierType::Armco);
    let hit = resolve_car_wall_collision(&mut car, &wall);
    assert!(hit.is_none(), "Airborne car above 1.2m should clear ground barriers");
}

#[test]
fn test_ramp_raceway_preset() {
    let track = ramp_raceway();
    assert_eq!(track.name, "Ramp Raceway");
    assert_eq!(track.geometry.jump_ramps.len(), 1, "Must have 1 jump ramp");
    assert!(track.checkpoints.len() >= 8);
    assert!(track.grid_positions.len() >= 6);

    // Verify sample surface on track
    let surf = track.sample_surface(Vec2::new(82.5, 50.0));
    assert_eq!(surf, SurfaceType::Asphalt);
}

#[test]
fn test_oasis_rally_preset() {
    let track = oasis_rally();
    assert_eq!(track.name, "Oasis Rally");
    assert_eq!(track.default_surface, SurfaceType::Sand, "Must be desert sand off-track");
    assert!(track.geometry.surface_zones.len() >= 4);
    assert!(track.geometry.obstacles.len() >= 3);
    assert!(track.spline.total_length() > 900.0, "Track must be extended and longer");

    // Verify pure dirt circuit: NO red-white curbs anywhere on the track
    let has_any_curbs = track
        .spline
        .samples
        .iter()
        .any(|s| s.left_curb || s.right_curb);
    assert!(
        !has_any_curbs,
        "Oasis Rally must be a pure dirt circuit without any asphalt rumble curbs"
    );

    // Verify both water hazards are circular and separated across different sectors
    let water_zones: Vec<_> = track
        .geometry
        .surface_zones
        .iter()
        .filter(|z| z.surface == SurfaceType::Water)
        .collect();
    assert_eq!(water_zones.len(), 2, "Must have exactly two water hazard zones");

    let mut centers = Vec::new();
    for zone in &water_zones {
        match &zone.shape {
            SurfaceShape::Circle { center, radius } => {
                assert!(*radius > 0.0);
                centers.push(*center);
            }
            other => panic!("All water hazards must be circular, found: {:?}", other),
        }
    }
    let separation_dist = (centers[0] - centers[1]).length();
    assert!(
        separation_dist > 80.0,
        "The two water patches must be separated into distinct sectors (dist={:.1}m)",
        separation_dist
    );

    // Surface sampling on track ribbon: Dirt
    let start_surf = track.sample_surface(Vec2::new(0.0, 0.0));
    assert_eq!(start_surf, SurfaceType::Dirt, "Track ribbon must be playable Dirt");

    // Surface sampling in Northern Oasis Lagoon
    let water_surf1 = track.sample_surface(Vec2::new(45.0, 195.0));
    assert_eq!(water_surf1, SurfaceType::Water, "Northern Oasis Lagoon must sample Water");

    // Surface sampling in Southern Desert Spring (in the middle of the track)
    let water_surf2 = track.sample_surface(Vec2::new(-65.0, -2.5));
    assert_eq!(water_surf2, SurfaceType::Water, "Southern Desert Spring must sample Water in the middle of track");

    // Surface sampling off-track: deep sand terrain
    let off_track_surf = track.sample_surface(Vec2::new(500.0, 500.0));
    assert_eq!(off_track_surf, SurfaceType::Sand, "Off-track must be Sand");

    // Verify aliases work identically
    let alias_track1 = dune_raid();
    assert_eq!(alias_track1.name, "Oasis Rally");
    let alias_track2 = sahara_dunes();
    assert_eq!(alias_track2.name, "Oasis Rally");
}

#[test]
fn test_dirt_and_water_dynamics() {
    // Dirt provides good controllable slide traction
    let dirt_mu = SurfaceType::Dirt.friction_coefficient();
    assert!((0.70..0.85).contains(&dirt_mu));

    // Water gives extreme low friction (aquaplaning) and high drag
    let water_mu = SurfaceType::Water.friction_coefficient();
    let water_drag = SurfaceType::Water.surface_drag_multiplier();
    assert!(water_mu < 0.30, "Water must induce aquaplaning with mu < 0.30");
    assert!(water_drag >= 2.0, "Water must create significant displacement drag");

    // Sand provides strong deceleration power
    let sand_res = SurfaceType::Sand.rolling_resistance_multiplier();
    assert!(sand_res >= 40.0, "Sand must act as an aggressive stopping trap");
}

#[test]
fn test_sand_under_track_does_not_override_dirt_ribbon() {
    let track = oasis_rally();
    
    // In Oasis Rally, "Canyon Sand Trap 1" AABB is (230..290, 130..210).
    // The track spline has a waypoint at (255, 175) which passes right through this region.
    // When the car is on the track at (255, 175), it MUST sample Dirt (the visible surface), NOT Sand.
    let on_track_point = Vec2::new(255.0, 175.0);
    assert_eq!(
        track.sample_surface(on_track_point),
        SurfaceType::Dirt,
        "Car on track ribbon must sample Dirt even if an underlying sand trap overlaps"
    );

    // When the car moves off-track into the sand trap (e.g. at 285, 195, which is outside the ribbon width),
    // it MUST sample Sand (the visible off-track hazard).
    let off_track_in_trap = Vec2::new(285.0, 195.0);
    assert_eq!(
        track.sample_surface(off_track_in_trap),
        SurfaceType::Sand,
        "Car off track in sand trap must sample Sand"
    );

    // On-track water hazards (like the Northern Oasis Lagoon at 45, 195) MUST override the track ribbon
    let on_track_water = Vec2::new(45.0, 195.0);
    assert_eq!(
        track.sample_surface(on_track_water),
        SurfaceType::Water,
        "On-track water hazard must override the underlying track ribbon"
    );
}

#[test]
fn test_outlaw_pass_narrow_mountain_pass_preset() {
    use tdrace_core::collision::wall::resolve_all_wall_collisions;

    let track = outlaw_pass();
    assert_eq!(track.name, "Outlaw Pass");
    assert!(track.geometry.jump_ramps.is_empty(), "Outlaw Pass has no jump ramps");
    assert!(
        !track.geometry.surface_zones.iter().any(|z| z.surface == SurfaceType::Water),
        "Outlaw Pass has no water hazards"
    );
    assert_eq!(track.geometry.obstacles.len(), 4, "Must have 4 mountain cliff obstacles");

    // 1. Verify narrow mountain pass section (width <= 7.5m)
    let min_width = track
        .spline
        .samples
        .iter()
        .map(|s| s.width)
        .fold(f32::INFINITY, f32::min);
    assert!(
        min_width <= 7.05,
        "The Pass section must narrow down to ~7.0m, found min width {:.2}m",
        min_width
    );

    let max_width = track
        .spline
        .samples
        .iter()
        .map(|s| s.width)
        .fold(0.0f32, f32::max);
    assert_eq!(max_width, 13.0, "High-speed straights must be 13.0m wide");

    // 2. Drive the entire circuit centerline (all sectors + narrow pass)
    let total_len = track.spline.total_length();
    let num_steps = (total_len / 0.5) as usize;
    for i in 0..num_steps {
        let dist = i as f32 * 0.5;
        let sample = track.spline.sample_at_distance(dist);
        let heading = sample.tangent.y.atan2(sample.tangent.x);
        let mut car = Car::new(CarConfig::sports_car()).with_pose(sample.point, heading);

        let initial_pos = car.state.position;
        let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &track.geometry.obstacles);
        let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);

        let displacement = (car.state.position - initial_pos).length();
        assert!(
            hit_inner.is_empty() && hit_outer.is_empty() && displacement < 0.01,
            "Centerline collision on Outlaw Pass at dist={:.1}m / {:.1}m: disp={:.3}m",
            dist, total_len, displacement
        );

        // Surface must be Asphalt or Curb
        let surface = track.sample_surface(sample.point);
        assert!(
            surface == SurfaceType::Asphalt || surface == SurfaceType::Curb,
            "Surface on track ribbon at dist={:.1}m must be Asphalt/Curb, got {:?}",
            dist, surface
        );
    }

    // 3. Lap progression and timing
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
    let mut car = Car::new(CarConfig::sports_car());

    for i in 0..track.checkpoints.len() {
        let cp = &track.checkpoints[i];
        let mid = (cp.gate.start + cp.gate.end) * 0.5;
        car.state.position = mid - cp.direction * 1.5;
        tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
        car.state.position = mid + cp.direction * 1.5;
        tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    }

    let cp0 = &track.checkpoints[0];
    car.state.position = cp0.gate.start - cp0.direction * 1.5;
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);
    car.state.position = cp0.gate.start + cp0.direction * 1.5;
    tracker.update(&car, &track.spline, &track.checkpoints, 0.016);

    assert_eq!(tracker.current_lap, 2, "Full lap must be counted as lap 2");
    assert!(tracker.best_lap_time.is_some(), "Lap time must be recorded");
}






