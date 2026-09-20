use glam::Vec2;
use tdrace_core::collision::wall::resolve_car_wall_collision;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, JumpRampCarExt, SurfaceShape, WallBarrier};
use tdrace_core::track::presets::{classic_rallycross, dune_raid, oasis_rally, ramp_raceway, sahara_dunes};

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
fn test_ramp_raceway_preset() {
    let track = ramp_raceway();
    assert_eq!(track.name, "Ramp Raceway");
    assert_eq!(track.geometry.jump_ramps.len(), 1, "Must have 1 jump ramp");
    assert!(track.checkpoints.len() >= 8);
    assert!(track.grid_positions.len() >= 6);
    assert_eq!(track.default_surface, SurfaceType::Dirt);
    assert_eq!(track.predefined_car.as_deref(), Some("classic_rally"));

    // Verify sample surface on track is dirt
    let surf = track.sample_surface(Vec2::new(82.5, 50.0));
    assert_eq!(surf, SurfaceType::Dirt);
}

#[test]
fn test_classic_rallycross_preset() {
    let track = classic_rallycross();
    assert_eq!(track.name, "Classic Rallycross");
    assert_eq!(track.predefined_car.as_deref(), Some("classic_rally"));
    assert_eq!(track.geometry.jump_ramps.len(), 1);
    let len = track.spline.total_length();
    assert!(len >= 900.0 && len <= 1200.0, "Length ~1km: got {:.1}m", len);
    let has_asphalt = track.spline.waypoints.iter().any(|wp| wp.surface == Some(SurfaceType::Asphalt));
    let has_dirt = track.spline.waypoints.iter().any(|wp| wp.surface == Some(SurfaceType::Dirt));
    assert!(has_asphalt, "Classic Rallycross must contain asphalt sections");
    assert!(has_dirt, "Classic Rallycross must contain dirt sections");
}

#[test]
fn test_oasis_rally_preset() {
    let track = oasis_rally();
    assert_eq!(track.name, "Oasis Rally");
    assert_eq!(track.default_surface, SurfaceType::Sand, "Must be desert sand off-track");
    assert_eq!(track.geometry.surface_zones.len(), 3);
    assert_eq!(track.geometry.obstacles.len(), 0);
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

    // Verify Northern Oasis Lagoon water hazard is circular
    let water_zones: Vec<_> = track
        .geometry
        .surface_zones
        .iter()
        .filter(|z| z.surface == SurfaceType::Water)
        .collect();
    assert_eq!(water_zones.len(), 1, "Must have exactly one Northern Oasis Lagoon water hazard");

    match &water_zones[0].shape {
        SurfaceShape::Circle { center, radius } => {
            assert!(*radius > 0.0);
            assert_eq!(*center, Vec2::new(25.0, 190.0));
        }
        other => panic!("Water hazard must be circular, found: {:?}", other),
    }

    // Surface sampling on track ribbon: Dirt
    let start_surf = track.sample_surface(Vec2::new(0.0, 0.0));
    assert_eq!(start_surf, SurfaceType::Dirt, "Track ribbon must be playable Dirt");

    // Surface sampling in Northern Oasis Lagoon
    let water_surf1 = track.sample_surface(Vec2::new(25.0, 190.0));
    assert_eq!(water_surf1, SurfaceType::Water, "Northern Oasis Lagoon must sample Water");

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
    assert!(sand_res >= 25.0, "Sand must act as an aggressive stopping trap");
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

    // On-track water hazards (like the Northern Oasis Lagoon at 25, 190) MUST override the track ribbon
    let on_track_water = Vec2::new(25.0, 190.0);
    assert_eq!(
        track.sample_surface(on_track_water),
        SurfaceType::Water,
        "On-track water hazard must override the underlying track ribbon"
    );
}






