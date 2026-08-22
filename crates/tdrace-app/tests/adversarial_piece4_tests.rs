use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile};
use tdrace_app::camera::RaceCamera;
use tdrace_app::fx::EffectsManager;
use tdrace_app::ui::hud::format_lap_time;
use tdrace_core::collision::car_collision::{resolve_multi_car_collisions, CarCarCollisionEvent};
use tdrace_core::collision::wall::{resolve_all_wall_collisions, WallCollisionEvent};
use tdrace_core::{Car, CarConfig};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};

#[test]
fn test_long_race_fx_memory_boundedness() {
    let mut fx = EffectsManager::new(2000, 500);

    let mut cars = vec![
        Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0),
        Car::new(CarConfig::drift_car()).with_pose(Vec2::new(10.0, 0.0), 0.0),
        Car::new(CarConfig::kart()).with_pose(Vec2::new(20.0, 0.0), 0.0),
        Car::new(CarConfig::rally_car()).with_pose(Vec2::new(30.0, 0.0), 0.0),
    ];

    let surfaces = vec![
        [SurfaceType::Asphalt; 4],
        [SurfaceType::Grass; 4],
        [SurfaceType::Sand; 4],
        [SurfaceType::Curb; 4],
    ];

    let wall_events = vec![WallCollisionEvent {
        contact_point: Vec2::new(5.0, 5.0),
        normal: Vec2::new(0.0, 1.0),
        penetration: 0.1,
        impact_speed: 15.0,
        normal_impulse: 1000.0,
        friction_impulse: 100.0,
    }];

    let car_events = vec![CarCarCollisionEvent {
        car_a_idx: 0,
        car_b_idx: 1,
        contact_point: Vec2::new(2.0, 2.0),
        normal: Vec2::new(1.0, 0.0),
        penetration: 0.05,
        closing_speed: 12.0,
        impulse_magnitude: 800.0,
    }];

    // Simulate 50,000 steps (~7 minutes of extreme continuous chaos)
    for step in 0..50000 {
        for (i, car) in cars.iter_mut().enumerate() {
            car.state.position += Vec2::new(0.2, 0.1 * ((step + i) as f32).sin());
            car.state.speed = 30.0;
            car.state.velocity = Vec2::new(25.0, 10.0);
            car.state.is_drifting = (step % 20) < 10;
            car.state.drift_score = if car.state.is_drifting { 500.0 } else { 0.0 };
            for w in 0..4 {
                car.state.wheels[w].skid_intensity = 0.8;
                car.state.wheels[w].slip_ratio = 0.3;
            }
        }

        fx.update(&cars, &surfaces, &wall_events, &car_events, 0.016);

        // Verification: Particle count must NEVER exceed max capacity (500)
        assert!(
            fx.particles.count() <= 500,
            "Particle count {} exceeded max limit 500 at step {}",
            fx.particles.count(),
            step
        );

        // Verification: Skidmarks count must NEVER exceed buffer capacity (2000)
        assert!(
            fx.skidmarks.count() <= 2000,
            "Skidmark count {} exceeded max limit 2000 at step {}",
            fx.skidmarks.count(),
            step
        );

        // Verification: Drift popups must NEVER exceed capacity (32)
        assert!(
            fx.drift_popups.active_popups().len() <= 32,
            "Drift popups count {} exceeded max limit 32 at step {}",
            fx.drift_popups.active_popups().len(),
            step
        );
    }
}

#[test]
fn test_camera_extreme_edge_cases_and_teleportation() {
    let mut camera = RaceCamera::new();
    let track = classic_grand_prix();
    camera.setup_for_track_with_viewport(&track, 1920.0, 1080.0);

    let mut car = Car::new(CarConfig::sports_car());

    // 1. Extreme spin (infinite / extreme angular velocity)
    car.state.position = Vec2::new(100.0, 200.0);
    car.state.speed = 0.0;
    car.state.velocity = Vec2::ZERO;
    car.state.angular_velocity = 100_000.0;
    car.state.angle = 1e6;

    camera.update(&car, 0.016);
    assert!(camera.current_pos.is_finite());
    assert!(camera.current_zoom.is_finite());
    assert!(camera.current_zoom > 0.0);

    // 2. Sudden massive teleportation (e.g. respawn across track)
    car.state.position = Vec2::new(-1_000_000.0, 500_000.0);
    car.state.velocity = Vec2::new(200.0, -150.0);
    car.state.speed = 250.0;

    for _ in 0..120 {
        camera.update(&car, 0.016);
    }

    assert!(camera.current_pos.is_finite());
    assert!(camera.current_zoom.is_finite());
    assert!(camera.current_zoom >= camera.min_zoom_scale * 0.9);

    // 3. Trauma saturation & decay under extreme inputs
    camera.add_trauma(1000.0); // Saturation clamped to 1.0
    assert_eq!(camera.trauma, 1.0);

    let cam2d = camera.camera_2d_with_viewport(1280.0, 720.0);
    assert!(cam2d.target.x.is_finite());
    assert!(cam2d.target.y.is_finite());
    assert!(cam2d.zoom.x.is_finite());
    assert!(cam2d.zoom.y.is_finite());

    // 4. Zero / sub-normal screen dimension safety
    let (safe_w, safe_h) = RaceCamera::get_screen_dimensions_safe();
    assert!(safe_w >= 320.0);
    assert!(safe_h >= 240.0);

    // 5. Delta time = 0.0 or huge dt
    camera.update(&car, 0.0);
    assert!(camera.current_pos.is_finite());
    camera.update(&car, 1000.0);
    assert!(camera.current_pos.is_finite());
}

#[test]
fn test_bot_ai_multi_track_lap_progression() {
    let tracks = [
        ("Classic GP", classic_grand_prix()),
        ("Oval Speedway", oval_speedway()),
        ("Drift Park", drift_park()),
        ("Kart Arena", kart_arena()),
    ];

    for (name, track) in &tracks {
        let n_cars = 3;
        let mut cars = Vec::new();
        let mut ai_drivers = Vec::new();
        let mut trackers = Vec::new();

        let profiles = [
            BotProfile::pro(),
            BotProfile::aggressive(),
            BotProfile::balanced(),
        ];

        for i in 0..n_cars {
            let spawn = track.grid_positions.get(i).copied().unwrap();
            let car = Car::new(CarConfig::sports_car()).with_pose(spawn.position, spawn.angle);
            cars.push(car);
            ai_drivers.push(BotAiDriver::new(profiles[i]));
            trackers.push(tdrace_core::track::checkpoint::TrackProgressTracker::new(
                track.checkpoints.len(),
                3,
            ));
        }

        let dt = 1.0 / 60.0;
        let total_steps = 1800; // 30 seconds of simulation

        for _step in 0..total_steps {
            // Compute controls
            let mut controls = Vec::new();
            for i in 0..n_cars {
                let other_refs: Vec<&Car> = cars
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != i)
                    .map(|(_, c)| c)
                    .collect();

                let ctrl = ai_drivers[i].compute_controls(&cars[i], track, &other_refs, dt);
                controls.push(ctrl);
            }

            // Step physics
            let mut wheel_surfaces = Vec::new();
            for car in &cars {
                wheel_surfaces.push(track.sample_car_surfaces(car));
            }

            for i in 0..n_cars {
                cars[i].step_per_wheel(&controls[i], wheel_surfaces[i], dt);
            }

            // Collisions
            let _ = resolve_multi_car_collisions(&mut cars, 0.45, 0.35, 3);
            for car in &mut cars {
                let _ = resolve_all_wall_collisions(car, &track.geometry.inner_walls, &track.geometry.obstacles);
                let _ = resolve_all_wall_collisions(car, &track.geometry.outer_walls, &[]);
            }

            // Track progress
            for i in 0..n_cars {
                trackers[i].update(&cars[i], &track.spline, &track.checkpoints, dt);
            }
        }

        // Validate that all bots made substantial continuous forward progress along the track
        for (i, tr) in trackers.iter().enumerate() {
            println!(
                "Track [{}], Bot {}: Lap {}, Progress {:.2}%, Max Speed: {:.1} km/h",
                name,
                i,
                tr.current_lap,
                tr.normalized_progress * 100.0,
                cars[i].speed_kmh()
            );

            assert!(
                tr.normalized_progress > 0.15 || tr.current_lap > 1,
                "Bot {} on track {} failed to make forward progress (Progress: {:.2})",
                i,
                name,
                tr.normalized_progress
            );
            if trackers[i].is_wrong_way && trackers[i].wrong_way_timer > 2.0 {
                panic!(
                    "Bot {} on track {} drove the wrong way!",
                    i, name
                );
            }
        }
    }
}

#[test]
fn test_ui_and_hud_formatting_corner_cases() {
    assert_eq!(format_lap_time(0.0), "--:--.--");
    assert_eq!(format_lap_time(-5.0), "--:--.--");
    assert_eq!(format_lap_time(f32::NAN), "--:--.--");
    assert_eq!(format_lap_time(f32::INFINITY), "--:--.--");

    assert_eq!(format_lap_time(65.42), "01:05.42");
    assert_eq!(format_lap_time(12.05), "00:12.05");
    assert_eq!(format_lap_time(3599.99), "59:59.99");
}
