use glam::Vec2;
use tdrace_app::fx::{DriftPopupManager, EffectsManager, ParticleSystem, SkidmarkBuffer};
use tdrace_core::collision::car_collision::CarCarCollisionEvent;
use tdrace_core::collision::wall::WallCollisionEvent;
use tdrace_core::{Car, CarConfig};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_skidmarks_buffer_lifecycle() {
    let mut buffer = SkidmarkBuffer::new(50);
    assert_eq!(buffer.count(), 0);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.5;
    car.state.wheels[1].skid_intensity = 0.5;

    let cars = vec![car.clone()];
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    // Step 1: Initial position recorded
    buffer.update_for_cars(&cars, &surfaces);
    assert_eq!(buffer.count(), 0);

    // Step 2: Car moves while skidding
    car.state.position = Vec2::new(0.5, 0.0);
    let cars2 = vec![car.clone()];
    buffer.update_for_cars(&cars2, &surfaces);
    assert!(buffer.count() > 0);

    // Test clear
    buffer.clear();
    assert_eq!(buffer.count(), 0);
}

#[test]
fn test_particle_system_emission_and_updates() {
    let mut ps = ParticleSystem::new(200);
    assert_eq!(ps.count(), 0);

    // Smoke
    ps.emit_tire_smoke(Vec2::ZERO, Vec2::new(5.0, 0.0), 0.8);
    assert!(ps.count() > 0);

    // Dirt roost
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::Grass, Vec2::new(10.0, 0.0));
    assert!(ps.count() > 0);

    // Collision sparks
    ps.emit_sparks(Vec2::new(5.0, 5.0), Vec2::new(-1.0, 0.0), 8.0);
    let spark_count = ps.count();
    assert!(spark_count > 0);

    // Step forward 0.1s
    ps.update(0.1);
    assert!(ps.count() > 0);

    // Step forward 2.0s -> all particles should expire
    ps.update(2.0);
    assert_eq!(ps.count(), 0);
}

#[test]
fn test_drift_popup_manager() {
    let mut popups = DriftPopupManager::new(10);
    assert_eq!(popups.active_popups().len(), 0);

    popups.spawn_drift_score(Vec2::new(10.0, 10.0), 350.0, 1.5);
    assert_eq!(popups.active_popups().len(), 1);
    assert!(popups.active_popups()[0].text.contains("+350 DRIFT!"));

    popups.update(0.5);
    assert_eq!(popups.active_popups().len(), 1);
    assert!(popups.active_popups()[0].world_pos.y < 10.0); // Floats upward (-Y in Cartesian)

    popups.update(1.5);
    assert_eq!(popups.active_popups().len(), 0); // Expired
}

#[test]
fn test_effects_manager_integration() {
    let mut fx = EffectsManager::new(500, 500);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.speed = 25.0;
    car.state.velocity = Vec2::new(25.0, 0.0);
    car.state.wheels[0].skid_intensity = 0.6;
    car.state.is_drifting = true;
    car.state.drift_score = 450.0;

    let cars = vec![car.clone()];
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    let wall_events = vec![WallCollisionEvent {
        contact_point: Vec2::new(10.0, 5.0),
        normal: Vec2::new(0.0, -1.0),
        penetration: 0.05,
        impact_speed: 6.5,
        normal_impulse: 500.0,
        friction_impulse: 50.0,
        barrier_type: tdrace_core::track::geometry::BarrierType::Concrete,
    }];

    let car_events = vec![CarCarCollisionEvent {
        car_a_idx: 0,
        car_b_idx: 1,
        contact_point: Vec2::new(0.0, 0.0),
        normal: Vec2::new(1.0, 0.0),
        penetration: 0.02,
        closing_speed: 4.5,
        impulse_magnitude: 300.0,
    }];

    fx.update(&cars, &surfaces, &wall_events, &car_events, 0.016);
    assert!(fx.particles.count() > 0);

    // End drift -> triggers drift popup
    let mut car_stopped_drift = car.clone();
    car_stopped_drift.state.is_drifting = false;
    let cars2 = vec![car_stopped_drift];
    fx.update(&cars2, &surfaces, &[], &[], 0.016);
    assert_eq!(fx.drift_popups.active_popups().len(), 1);
}

#[test]
fn test_no_particles_while_car_is_on_the_air() {
    let mut fx = EffectsManager::new(500, 500);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.speed = 30.0;
    car.state.velocity = Vec2::new(30.0, 0.0);
    car.state.is_airborne = true;
    car.state.elevation = 1.5;

    // Simulate car skidding over grass / water / dirt while jumping
    for w in 0..4 {
        car.state.wheels[w].skid_intensity = 0.9;
        car.state.wheels[w].slip_ratio = 0.5;
    }

    let cars = vec![car.clone()];
    let surfaces_grass = vec![[SurfaceType::Grass; 4]];
    let surfaces_water = vec![[SurfaceType::Water; 4]];
    let surfaces_dirt = vec![[SurfaceType::Dirt; 4]];
    let surfaces_asphalt = vec![[SurfaceType::Asphalt; 4]];

    // 1. In air over grass
    fx.update(&cars, &surfaces_grass, &[], &[], 0.016);
    assert_eq!(fx.particles.count(), 0, "No particles should emit over grass while airborne");
    assert_eq!(fx.skidmarks.count(), 0, "No skidmarks should be laid down while airborne");

    // 2. In air over water
    fx.update(&cars, &surfaces_water, &[], &[], 0.016);
    assert_eq!(fx.particles.count(), 0, "No particles should emit over water while airborne");
    assert_eq!(fx.skidmarks.count(), 0, "No skidmarks should be laid down while airborne");

    // 3. In air over dirt
    fx.update(&cars, &surfaces_dirt, &[], &[], 0.016);
    assert_eq!(fx.particles.count(), 0, "No particles should emit over dirt while airborne");

    // 4. In air over asphalt
    fx.update(&cars, &surfaces_asphalt, &[], &[], 0.016);
    assert_eq!(fx.particles.count(), 0, "No particles should emit over asphalt while airborne");

    // 5. When car lands on the ground (is_airborne = false, elevation = 0.0), skid particles emit normally
    car.state.is_airborne = false;
    car.state.elevation = 0.0;
    let grounded_cars = vec![car];
    fx.update(&grounded_cars, &surfaces_asphalt, &[], &[], 0.016);
    assert!(fx.particles.count() > 0, "Grounded car with skid intensity should emit tire smoke");
}

#[test]
fn test_skidmarks_dual_tread_and_irregularity() {
    let mut buffer = SkidmarkBuffer::new(50);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.6;
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0);

    // Car moves forward 0.5m while skidding
    car.state.position = Vec2::new(0.5, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);

    // One skidding wheel produces 2 sub-ribbon tread tracks for realistic texturing
    assert_eq!(buffer.count(), 2, "Single skidding wheel should produce dual-tread ribbons for texturing");
}

#[test]
fn test_skidmarks_uv_mapping_and_persistent_capacity() {
    let mut buffer = SkidmarkBuffer::new(64000);
    assert_eq!(buffer.count(), 0);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.8;
    car.state.wheels[1].skid_intensity = 0.8;
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    buffer.update_for_cars(&[car.clone()], &surfaces);

    // Car slides forward across multiple steps
    for step in 1..=10 {
        car.state.position = Vec2::new(step as f32 * 0.4, 0.0);
        buffer.update_for_cars(&[car.clone()], &surfaces);
    }

    // 2 wheels skidding * 2 ribbons * 10 steps = 40 segments
    assert_eq!(buffer.count(), 40);

    // Verify safe batch rendering call
    buffer.render();
}

#[test]
fn test_persistent_skidmarks_across_multiple_laps() {
    let mut buffer = SkidmarkBuffer::new_persistent();
    assert!(buffer.is_persistent());
    assert_eq!(buffer.count(), 0);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.75;
    car.state.wheels[1].skid_intensity = 0.75;
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    // Establish baseline anchor
    buffer.update_for_cars(&[car.clone()], &surfaces);

    // Lap 1: simulate laying down rubber marks
    for step in 1..=50 {
        car.state.position = Vec2::new(step as f32 * 0.35, 0.0);
        buffer.update_for_cars(&[car.clone()], &surfaces);
    }

    let lap1_count = buffer.count();
    assert!(lap1_count > 0, "Lap 1 must lay down rubber traces");
    let lap1_snapshot = buffer.segments()[..lap1_count].to_vec();

    // Laps 2 through 10: simulate repeated laps generating hundreds of segments
    for lap in 2..=10 {
        // Car circles around another corner of the track
        for step in 1..=50 {
            car.state.position = Vec2::new(
                step as f32 * 0.35,
                lap as f32 * 10.0,
            );
            buffer.update_for_cars(&[car.clone()], &surfaces);
        }
    }

    // Crucial requirement: Buffer must never wrap or drop segments from Lap 1
    assert_eq!(
        &buffer.segments()[..lap1_count],
        &lap1_snapshot[..],
        "Lap 1 tire rubber traces must remain completely intact and preserved through the final lap without overwriting"
    );
    assert!(
        buffer.count() > lap1_count * 9,
        "Persistent buffer must accumulate segments monotonically across all laps"
    );

    // Verify clear only empties on explicit race restart
    buffer.clear();
    assert_eq!(buffer.count(), 0);
}

#[test]
fn test_skidmarks_viewport_culling() {
    let mut buffer = SkidmarkBuffer::new_persistent();

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.8;
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    buffer.update_for_cars(&[car.clone()], &surfaces);

    // Emit segments in the origin region
    for step in 1..=5 {
        car.state.position = Vec2::new(step as f32 * 0.4, 0.0);
        buffer.update_for_cars(&[car.clone()], &surfaces);
    }

    // Car teleports far away (> 3m breaks contiguous ribbon) and skids far off-screen
    car.state.position = Vec2::new(1000.0, 1000.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);
    for step in 1..=5 {
        car.state.position = Vec2::new(1000.0 + step as f32 * 0.4, 1000.0);
        buffer.update_for_cars(&[car.clone()], &surfaces);
    }

    // Verify culled rendering completes safely for both focused view and off-screen view
    let in_view_bounds = Some((Vec2::new(-10.0, -10.0), Vec2::new(20.0, 20.0)));
    buffer.render_culled(in_view_bounds);

    let far_view_bounds = Some((Vec2::new(990.0, 990.0), Vec2::new(1020.0, 1020.0)));
    buffer.render_culled(far_view_bounds);

    let nowhere_view_bounds = Some((Vec2::new(500.0, 500.0), Vec2::new(510.0, 510.0)));
    buffer.render_culled(nowhere_view_bounds);
}

#[test]
fn test_slow_speed_hairpin_distance_accumulation() {
    let mut buffer = SkidmarkBuffer::new_persistent();

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.wheels[0].skid_intensity = 0.9;
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    // Step 0: Record initial wheel anchor
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0);

    // Car moves in micro-steps of 0.05m (slow hairpin cornering)
    // Step 1: 0.05m -> dist < 0.20 -> no segment yet, anchor accumulates
    car.state.position = Vec2::new(0.05, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0, "Micro-step 0.05m should accumulate without emitting");

    // Step 2: 0.10m cumulative -> dist < 0.20 -> still accumulating
    car.state.position = Vec2::new(0.10, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0, "Cumulative 0.10m should accumulate without emitting");

    // Step 3: 0.15m cumulative -> dist < 0.20 -> still accumulating
    car.state.position = Vec2::new(0.15, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0, "Cumulative 0.15m should accumulate without emitting");

    // Step 4: 0.20m cumulative -> dist >= 0.20 -> emits dual-ribbon segments!
    car.state.position = Vec2::new(0.20, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 2, "Cumulative distance reaching 0.20m must cleanly emit dual-tread segments");
}

#[test]
fn test_gravel_rolling_rut_without_slip() {
    let mut buffer = SkidmarkBuffer::new(100);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.speed = 10.0;
    car.state.velocity = Vec2::new(10.0, 0.0);
    // Zero slip on all wheels
    for w in 0..4 {
        car.state.wheels[w].skid_intensity = 0.0;
        car.state.wheels[w].is_skidding = false;
        car.state.wheels[w].slip_ratio = 0.0;
        car.state.wheels[w].slip_angle = 0.0;
    }
    let surfaces = vec![[SurfaceType::Gravel; 4]];

    // Step 0: Anchor
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0);

    // Step 1: Move forward 0.5m on gravel
    car.state.position = Vec2::new(0.5, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);

    // Deformable terrain leaves rolling ruts even without slip!
    assert!(
        buffer.count() > 0,
        "Rolling on gravel without slip must still create visible depression ruts"
    );

    let seg = &buffer.segments()[0];
    // Gravel color should be dark slate (r ~ 0.28, g ~ 0.26, b ~ 0.24), NOT asphalt black or sand yellow
    assert!((seg.color.r - 0.28).abs() < 0.05);
    assert!((seg.color.g - 0.26).abs() < 0.05);
    assert!((seg.color.b - 0.24).abs() < 0.05);
}

#[test]
fn test_dirt_contamination_deposit_on_pavement() {
    let mut buffer = SkidmarkBuffer::new(100);

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    car.state.speed = 12.0;
    car.state.velocity = Vec2::new(12.0, 0.0);
    for w in 0..4 {
        car.state.wheels[w].skid_intensity = 0.0;
        car.state.wheels[w].is_skidding = false;
        car.state.wheels[w].slip_ratio = 0.0;
        car.state.wheels[w].slip_angle = 0.0;
        // Contaminated wheel from previous off-track excursion
        car.state.wheels[w].dirt_contamination = 0.75;
        car.state.wheels[w].dirt_surface = SurfaceType::Dirt;
    }
    let surfaces = vec![[SurfaceType::Asphalt; 4]];

    // Step 0: Anchor
    buffer.update_for_cars(&[car.clone()], &surfaces);
    assert_eq!(buffer.count(), 0);

    // Step 1: Car moves onto clean asphalt with dirty tires
    car.state.position = Vec2::new(0.5, 0.0);
    buffer.update_for_cars(&[car.clone()], &surfaces);

    assert!(
        buffer.count() > 0,
        "Contaminated tires rolling on clean pavement must deposit dirt trails without needing slip"
    );

    let seg = &buffer.segments()[0];
    // Color should reflect dirt (r ~ 0.35, g ~ 0.22, b ~ 0.12)
    assert!((seg.color.r - 0.35).abs() < 0.05);
    assert!((seg.color.g - 0.22).abs() < 0.05);
    assert!((seg.color.b - 0.12).abs() < 0.05);
}

#[test]
fn test_multi_surface_skidmark_distinct_palettes() {
    let test_cases = [
        (SurfaceType::PackedSand, 0.65, 0.52, 0.28),
        (SurfaceType::DeepMud, 0.18, 0.12, 0.06),
        (SurfaceType::PackedSnow, 0.65, 0.72, 0.82),
        (SurfaceType::SheetIce, 0.92, 0.96, 1.0),
    ];

    for (surf, exp_r, exp_g, exp_b) in test_cases {
        let mut buffer = SkidmarkBuffer::new(50);
        let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
        car.state.speed = 15.0;
        car.state.wheels[0].skid_intensity = 0.8;
        let surfaces = vec![[surf; 4]];

        buffer.update_for_cars(&[car.clone()], &surfaces);
        car.state.position = Vec2::new(0.5, 0.0);
        buffer.update_for_cars(&[car.clone()], &surfaces);

        assert!(buffer.count() > 0, "Surface {:?} should emit skidmarks", surf);
        let seg = &buffer.segments()[0];
        assert!(
            (seg.color.r - exp_r).abs() < 0.06 && (seg.color.g - exp_g).abs() < 0.06 && (seg.color.b - exp_b).abs() < 0.06,
            "Surface {:?} color ({}, {}, {}) did not match expected ({}, {}, {})",
            surf, seg.color.r, seg.color.g, seg.color.b, exp_r, exp_g, exp_b
        );
    }
}

#[test]
fn test_debris_roost_particle_emission_by_surface() {
    let mut ps = ParticleSystem::new(200);

    // Gravel roost
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::Gravel, Vec2::new(10.0, 0.0));
    let count_after_gravel = ps.count();
    assert!(count_after_gravel > 0, "Gravel should produce roost particles");

    // Mud roost
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::MudTrack, Vec2::new(10.0, 0.0));
    let count_after_mud = ps.count();
    assert!(count_after_mud > count_after_gravel, "Mud should produce roost particles");

    // Snow roost
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::PackedSnow, Vec2::new(10.0, 0.0));
    let count_after_snow = ps.count();
    assert!(count_after_snow > count_after_mud, "Snow should produce roost particles");

    // Asphalt - should NOT produce roost particles
    ps.emit_dirt_roost(Vec2::ZERO, SurfaceType::Asphalt, Vec2::new(10.0, 0.0));
    assert_eq!(ps.count(), count_after_snow, "Asphalt must NOT produce dirt roost particles");
}

