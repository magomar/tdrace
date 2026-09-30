use glam::Vec2;
use tdrace_core::collision::wall::resolve_car_wall_collision;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, JumpRampCarExt, SurfaceShape, WallBarrier};


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
fn test_continuous_curved_ramp_traversal_and_lip_takeoff() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(1.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(20.0, 0.0);

    let mut ramp = JumpRamp::new(
        1,
        SurfaceShape::Aabb {
            min: Vec2::new(0.0, -3.0),
            max: Vec2::new(20.0, 3.0),
        },
        Vec2::new(1.0, 0.0),
        4.0,
        15.0,
        2.0,
        "Curved Test Ramp",
    );
    // Fit pitch so the continuous transition runs smoothly all the way to the takeoff lip
    ramp.ramp_angle_deg = ramp.fitted_pitch_deg();

    let dt = 1.0 / 60.0;

    // 1. Entrance at s = 0.05: car must NOT launch into the air
    let launched = car.step_ramp_interaction(&ramp, dt);
    assert!(!launched, "Car at ramp entrance must not launch into the air");
    assert!(!car.state.is_airborne, "Car must remain grounded on ramp entrance");
    assert!(
        car.state.ramp_elevation > 0.0 && car.state.ramp_elevation < 0.02,
        "Entrance elevation must follow smooth parabolic transition (tangent to flat ground), got {}",
        car.state.ramp_elevation
    );
    assert_eq!(car.state.elevation, 0.0);

    // 2. Mid-ramp progression at s = 0.50 (x = 10.0)
    car.state.position = Vec2::new(10.0, 0.0);
    let pre_v = car.state.velocity.x;
    let launched_mid = car.step_ramp_interaction(&ramp, dt);
    assert!(!launched_mid, "Car at mid-ramp must not launch");
    assert!(!car.state.is_airborne);
    assert!(
        (car.state.ramp_elevation - 0.50).abs() < 0.05,
        "Mid-ramp elevation on parabolic curve H*s^2 should be ~0.50m, got {}",
        car.state.ramp_elevation
    );
    assert!(
        car.state.velocity.x < pre_v,
        "Downhill grade resistance must decelerate vehicle climbing the slope"
    );
    assert!(
        (car.total_elevation() - car.state.ramp_elevation).abs() < 1e-4,
        "total_elevation() must reflect on-ramp surface elevation"
    );

    // 3. Takeoff lip at s = 0.90 (x = 18.0): reaching the lip at speed triggers launch
    car.state.position = Vec2::new(18.0, 0.0);
    let launched_lip = car.step_ramp_interaction(&ramp, dt);
    assert!(launched_lip, "Car reaching lip at speed must trigger launch");
    assert!(car.state.is_airborne, "Car must become airborne upon lip takeoff");
    assert!(
        car.state.elevation >= 2.0,
        "Takeoff elevation must match ramp height (2.0m), got {}",
        car.state.elevation
    );
    assert_eq!(
        car.state.ramp_elevation, 0.0,
        "ramp_elevation must be cleared upon takeoff to transfer elevation smoothly"
    );
    assert!(
        car.state.vertical_velocity > 1.0,
        "Lip takeoff must impart vertical launch velocity"
    );
}

#[test]
fn test_low_speed_crawling_does_not_launch_at_lip() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(18.0, 0.0), 0.0);
    car.state.velocity = Vec2::new(2.0, 0.0); // Crawling under 3.5 m/s launch threshold

    let ramp = JumpRamp::new(
        1,
        SurfaceShape::Aabb {
            min: Vec2::new(0.0, -3.0),
            max: Vec2::new(20.0, 3.0),
        },
        Vec2::new(1.0, 0.0),
        4.0,
        15.0,
        2.0,
        "Curved Test Ramp",
    );

    let launched = car.step_ramp_interaction(&ramp, 1.0 / 60.0);
    assert!(!launched, "Crawling car must not launch off lip");
    assert!(!car.state.is_airborne, "Crawling car remains grounded");
    assert!(car.state.ramp_elevation > 1.5, "Ramp elevation reflects surface height at lip");
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

    let wall = WallBarrier::new(Vec2::new(10.0, -5.0), Vec2::new(10.0, 5.0), BarrierType::Steel);
    let hit = resolve_car_wall_collision(&mut car, &wall);
    assert!(hit.is_none(), "Airborne car above 1.2m should clear ground barriers");
}

#[test]
fn test_rx_hilltop_leap_preset() {
    let track = tdrace_core::catalog::official_track("classic", "rx_hilltop_leap");
    assert_eq!(track.name, "Hilltop Leap");
    assert!(!track.geometry.jump_ramps.is_empty(), "Must have jump ramps");
    assert!(track.checkpoints.len() >= 8);
    assert!(track.grid_positions.len() >= 6);
    assert_eq!(track.car_category, arcade_race_core::CarCategory::Rally);
}

#[test]
fn test_rx_quarry_sprint_preset() {
    let track = tdrace_core::catalog::official_track("classic", "rx_quarry_sprint");
    assert_eq!(track.name, "Quarry Sprint");
    assert_eq!(track.car_category, arcade_race_core::CarCategory::Rally);
    assert!(!track.geometry.jump_ramps.is_empty());
    let len = track.spline.total_length();
    assert!(len >= 750.0 && len <= 1400.0, "Length ~1km: got {:.1}m", len);
    let has_asphalt = track.spline.waypoints.iter().any(|wp| wp.surface == Some(SurfaceType::Asphalt));
    let has_dirt = track.spline.waypoints.iter().any(|wp| wp.surface == Some(SurfaceType::Dirt));
    assert!(has_asphalt, "Quarry Sprint must contain asphalt sections");
    assert!(has_dirt, "Quarry Sprint must contain dirt sections");
}

#[test]
fn test_at_dune_sea_preset() {
    let track = tdrace_core::catalog::official_track("classic", "at_dune_sea");
    assert_eq!(track.name, "Dune Sea");
    assert_eq!(track.default_surface, SurfaceType::DeepSand, "Must be desert sand off-track");
    assert!(!track.geometry.jump_ramps.is_empty(), "Must have jump ramps");
    assert!(track.spline.total_length() > 900.0, "Track must be extended and longer");

    // Surface sampling on track ribbon: PackedSand
    let p0 = track.spline.samples[0].point;
    assert_eq!(track.sample_surface(p0), SurfaceType::PackedSand);

    // Surface sampling far off-track: deep sand terrain
    let off_track_surf = track.sample_surface(p0 + glam::Vec2::new(500.0, 500.0));
    assert_eq!(off_track_surf, SurfaceType::DeepSand, "Off-track must be DeepSand");
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

    // DeepSand provides strong deceleration power
    let sand_res = SurfaceType::DeepSand.rolling_resistance_multiplier();
    assert!(sand_res >= 8.0, "DeepSand must act as an aggressive stopping trap");
}

#[test]
fn test_sand_under_track_does_not_override_dirt_ribbon() {
    let mut track = tdrace_core::catalog::official_track("classic", "at_dune_sea");
    let p0 = track.spline.samples[0].point;
    let n0 = track.spline.samples[0].normal;

    // Add a BelowTrack sand zone overlapping the start
    track.geometry.surface_zones.push(
        arcade_race_core::track::geometry::SurfaceZone::new(
            arcade_race_core::track::geometry::SurfaceShape::Aabb {
                min: p0 - glam::Vec2::splat(20.0),
                max: p0 + glam::Vec2::splat(20.0),
            },
            SurfaceType::DeepSand,
            "Sand Trap",
        ).with_layer(arcade_race_core::track::geometry::SurfaceLayer::BelowTrack),
    );

    // Ribbon is PackedSand, so point on track must sample PackedSand
    assert_eq!(track.sample_surface(p0), SurfaceType::PackedSand);

    // Point off track inside sand trap beyond runoff corridor must sample DeepSand
    assert_eq!(track.sample_surface(p0 + n0 * 15.0), SurfaceType::DeepSand);

    // Add an AboveTrack water puddle overlapping start
    track.geometry.surface_zones.push(
        arcade_race_core::track::geometry::SurfaceZone::new(
            arcade_race_core::track::geometry::SurfaceShape::Circle {
                center: p0,
                radius: 5.0,
            },
            SurfaceType::Water,
            "Water Puddle",
        ).with_layer(arcade_race_core::track::geometry::SurfaceLayer::AboveTrack),
    );
    assert_eq!(track.sample_surface(p0), SurfaceType::Water);
}






