use tdrace_core::physics::{Car, CarConfig, CarControls, SurfaceType};

#[test]
fn test_surface_grip_and_acceleration_scaling() {
    let dt = 1.0 / 60.0;
    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);

    // Compare 2 seconds of acceleration across different surfaces
    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Concrete,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::DeepSand,
        SurfaceType::SheetIce,
    ];

    let mut speeds = Vec::new();

    for &surf in &surfaces {
        let mut car = Car::new(CarConfig::sports_car());
        for _ in 0..120 {
            car.step(&ctrl, surf, dt);
        }
        speeds.push((surf, car.state.speed));
    }

    println!("2-second acceleration test across surfaces:");
    for (surf, speed) in &speeds {
        println!("  {:?}: {:.2} m/s ({:.2} km/h)", surf, speed, speed * 3.6);
    }

    // Asphalt should achieve highest speed, followed closely by Concrete, Sand and Ice much lower
    let speed_asphalt = speeds[0].1;
    let speed_concrete = speeds[1].1;
    let speed_grass = speeds[3].1;
    let speed_sand = speeds[4].1;
    let speed_ice = speeds[5].1;

    assert!(
        speed_asphalt >= speed_concrete,
        "Asphalt ({speed_asphalt}) must yield at least concrete acceleration ({speed_concrete})"
    );
    assert!(
        speed_concrete > speed_grass,
        "Concrete ({speed_concrete}) must yield higher acceleration than grass ({speed_grass})"
    );
    assert!(
        speed_asphalt > speed_grass,
        "Asphalt ({speed_asphalt}) must yield higher acceleration than grass ({speed_grass})"
    );
    assert!(
        speed_grass > speed_sand,
        "Grass ({speed_grass}) must yield higher acceleration than sand ({speed_sand})"
    );
    assert!(
        speed_asphalt > speed_ice,
        "Asphalt ({speed_asphalt}) must yield higher acceleration than ice ({speed_ice})"
    );
}

#[test]
fn test_surface_transition_asphalt_to_grass_deceleration() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // 1. Accelerate to speed on asphalt (4 seconds)
    for _ in 0..240 {
        car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    let speed_on_asphalt = car.state.speed;
    assert!(speed_on_asphalt > 18.0, "Speed on asphalt was {:.2} m/s", speed_on_asphalt);

    // 2. Coasting on asphalt vs coasting on grass
    let mut car_asphalt_coast = car.clone();
    let mut car_grass_coast = car.clone();

    for _ in 0..120 {
        car_asphalt_coast.step(&CarControls::default(), SurfaceType::Asphalt, dt);
        car_grass_coast.step(&CarControls::default(), SurfaceType::Grass, dt);
    }

    let speed_after_asphalt = car_asphalt_coast.state.speed;
    let speed_after_grass = car_grass_coast.state.speed;

    println!(
        "Coasting from {:.1} m/s for 2s: Asphalt remaining = {:.2} m/s, Grass remaining = {:.2} m/s",
        speed_on_asphalt, speed_after_asphalt, speed_after_grass
    );

    assert!(
        speed_after_grass < speed_after_asphalt * 0.85,
        "Grass rolling resistance and surface drag must decelerate the car significantly faster than asphalt"
    );
}

#[test]
fn test_sand_trap_stopping_power() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Accelerate to ~70 km/h (3.5 seconds)
    for _ in 0..210 {
        car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }
    let entry_speed = car.speed_kmh();
    let entry_pos = car.state.position.x;

    // Run into sand trap with zero throttle
    let mut steps_in_sand = 0;
    while car.state.speed > 0.1 && steps_in_sand < 300 {
        car.step(&CarControls::default(), SurfaceType::DeepSand, dt);
        steps_in_sand += 1;
    }

    let distance_in_sand = car.state.position.x - entry_pos;
    let time_in_sand = steps_in_sand as f32 * dt;
    println!(
        "Sand trap deceleration: Entry speed = {:.1} km/h, distance = {:.2} m, time = {:.2} s",
        entry_speed, distance_in_sand, time_in_sand
    );

    assert!(
        distance_in_sand < 50.0,
        "Sand trap must quickly stop vehicle (stopped in {:.2} m)",
        distance_in_sand
    );
}

#[test]
fn test_per_wheel_surface_split_mu() {
    let dt = 1.0 / 60.0;
    let mut car = Car::new(CarConfig::sports_car());

    // Accelerate straight on asphalt
    for _ in 0..120 {
        car.step(&CarControls::new(1.0, 0.0, 0.0, false), SurfaceType::Asphalt, dt);
    }

    // Split-mu under braking: Left wheels on Asphalt, Right wheels on Ice
    // Left: FL (0) Asphalt, RL (2) Asphalt
    // Right: FR (1) Ice, RR (3) Ice
    let split_surfaces = [
        SurfaceType::Asphalt,  // FL
        SurfaceType::SheetIce, // FR
        SurfaceType::Asphalt,  // RL
        SurfaceType::SheetIce, // RR
    ];

    let brake_ctrl = CarControls::new(0.0, 0.0, 1.0, false);
    for _ in 0..30 {
        car.step_per_wheel(&brake_ctrl, split_surfaces, dt);
    }

    // Because left wheels have high braking force and right wheels have near zero braking force,
    // a strong counter-clockwise yaw moment is generated (yaw angle/angular velocity changes)
    println!(
        "Split-mu braking: angular velocity = {:.3} rad/s, angle = {:.3} rad",
        car.state.angular_velocity, car.state.angle
    );

    assert!(
        car.state.angular_velocity.abs() > 0.05,
        "Split-mu braking must induce yaw moment due to asymmetrical grip"
    );
}

#[test]
fn test_off_track_surface_types_and_track_sampling() {
    use glam::Vec2;
    use tdrace_core::track::presets::classic_grand_prix;

    // 1. Verify valid and invalid off-track types
    for surf in SurfaceType::OFF_TRACK_TYPES {
        assert!(surf.is_valid_off_track(), "{:?} must be a valid off-track type", surf);
    }
    assert!(!SurfaceType::Curb.is_valid_off_track());
    assert!(!SurfaceType::Water.is_valid_off_track());
    assert!(!SurfaceType::Oil.is_valid_off_track());

    // 2. Verify Track::sample_surface returns the configured default_surface when far off-track
    let mut track = classic_grand_prix();
    let far_off_track_point = Vec2::new(1000.0, 1000.0);

    for &surf in &SurfaceType::OFF_TRACK_TYPES {
        track.default_surface = surf;
        assert_eq!(
            track.sample_surface(far_off_track_point),
            surf,
            "Track must sample configured default_surface {:?} when off-track",
            surf
        );
    }
}

#[test]
fn test_segment_runoff_corridor_and_virtual_boundary() {
    use glam::Vec2;
    use tdrace_core::track::geometry::BarrierType;
    use tdrace_core::track::spline::TrackWaypoint;
    use tdrace_core::track::{Track, TrackSpline};
    use tdrace_core::collision::wall::resolve_car_wall_collision;
    use tdrace_core::track::geometry::WallBarrier;

    // Track heading along +X with asphalt track, left gravel runoff corridor up to 8m, and virtual barrier
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0)
            .with_curbs(false, false)
            .with_runoff_surfaces(Some(SurfaceType::Gravel), None)
            .with_wall_distances(Some(8.0), None),
        TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0)
            .with_curbs(false, false)
            .with_runoff_surfaces(Some(SurfaceType::Gravel), None)
            .with_wall_distances(Some(8.0), None),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0)
            .with_curbs(false, false)
            .with_runoff_surfaces(Some(SurfaceType::Gravel), None)
            .with_wall_distances(Some(8.0), None),
    ];

    let track = Track {
        spline: TrackSpline::new(waypoints, false),
        default_surface: SurfaceType::Grass,
        ..Default::default()
    };

    // Track half width is 5.0m. Left side is +Y.
    // 1. Center of track (y = 0.0) -> Asphalt
    assert_eq!(track.sample_surface(Vec2::new(50.0, 0.0)), SurfaceType::Asphalt);

    // 2. Off track left in runoff corridor (y = 9.0 -> 4.0m off track, within 8.0m corridor) -> Gravel
    assert_eq!(track.sample_surface(Vec2::new(50.0, 9.0)), SurfaceType::Gravel);

    // 3. Beyond corridor (y = 15.0 -> 10.0m off track, beyond 8.0m limit) -> Grass
    assert_eq!(track.sample_surface(Vec2::new(50.0, 15.0)), SurfaceType::Grass);

    // 4. Car passing through virtual boundary at y = 13.0 suffers zero collision impulse
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 12.0), std::f32::consts::FRAC_PI_2);
    car.state.velocity = Vec2::new(0.0, 15.0);

    let virtual_boundary = WallBarrier::new(
        Vec2::new(0.0, 13.0),
        Vec2::new(100.0, 13.0),
        BarrierType::Virtual,
    );

    let col_event = resolve_car_wall_collision(&mut car, &virtual_boundary);
    assert!(col_event.is_none(), "Virtual boundary must produce no collision event");
    assert_eq!(car.state.velocity.y, 15.0, "Car velocity must not be altered by virtual boundary");
}
