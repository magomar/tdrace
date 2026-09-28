use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile};
use tdrace_core::{Car, CarConfig};


#[test]
fn test_bot_profiles_creation() {
    let pro = BotProfile::pro();
    let aggressive = BotProfile::aggressive();
    let balanced = BotProfile::balanced();
    let rookie = BotProfile::rookie();

    assert!(aggressive.speed_factor > balanced.speed_factor);
    assert!(pro.steering_kp > rookie.steering_kp);
    assert!(rookie.brake_margin > pro.brake_margin);
}

#[test]
fn test_bot_ai_steering_and_throttle_on_straight() {
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Car is initialized facing spline direction
    let proj = track.spline.project_point(Vec2::new(50.0, 0.0));
    let spline_angle = proj.tangent.y.atan2(proj.tangent.x);
    let car = Car::new(CarConfig::sports_car()).with_pose(proj.closest_point, spline_angle);

    let ctrl = bot.compute_controls(&car, &track, &[], 0.016);

    // Car should accelerate down straight with well-aligned steering
    assert!(ctrl.throttle > 0.5);
    assert!(ctrl.steer.abs() < 0.15);
    assert_eq!(ctrl.brake, 0.0);
    assert!(!ctrl.handbrake);
}

#[test]
fn test_bot_ai_collision_avoidance() {
    let track = tdrace_core::catalog::official_track("classic", "oval_speedway");
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Car A (our bot) is traveling at 35 m/s at x=50, y=-60
    let mut car_a = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, -60.0), 0.0);
    car_a.state.speed = 35.0;
    car_a.state.velocity = Vec2::new(35.0, 0.0);

    // Car B (slow opponent directly ahead) at x=54, y=-60 at 10 m/s
    let mut car_b = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(54.0, -60.0), 0.0);
    car_b.state.speed = 10.0;
    car_b.state.velocity = Vec2::new(10.0, 0.0);

    let ctrl = bot.compute_controls(&car_a, &track, &[&car_b], 0.016);

    // Bot should brake / lift throttle to prevent rear-ending car B
    assert!(ctrl.brake > 0.0 || ctrl.throttle < 0.5);
}

#[test]
fn test_bot_ai_cornering_slowdown() {
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let mut bot = BotAiDriver::new(BotProfile::pro());

    // Approaching hairpin at high speed (x=240, y=300, heading right towards hairpin)
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(240.0, 300.0), 0.0);
    car.state.speed = 50.0; // Over speeding for a tight corner
    car.state.velocity = Vec2::new(50.0, 0.0);

    let ctrl = bot.compute_controls(&car, &track, &[], 0.016);
    // Should apply brakes to prepare for hairpin turn
    assert!(ctrl.brake > 0.0 || ctrl.throttle < 0.2);
}

#[test]
fn test_bot_ai_slipstream_drafting_and_slingshot_pack_racing() {
    let track = tdrace_core::catalog::official_track("nascar", "daytona_superspeedway");
    let mut bot = BotAiDriver::new(BotProfile::aggressive());

    // Bot car trailing 15m behind lead car on the back straight (heading left, angle PI)
    // Superstretch is at y = 180.0, heading towards negative X
    let mut trailing_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(100.0, 180.0), std::f32::consts::PI);
    trailing_car.state.speed = 65.0; // ~234 km/h
    trailing_car.state.velocity = Vec2::new(-65.0, 0.0);

    // Lead car slightly offset laterally (e.g. y = 180.6)
    let mut lead_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(85.0, 180.6), std::f32::consts::PI);
    lead_car.state.speed = 64.0;
    lead_car.state.velocity = Vec2::new(-64.0, 0.0);

    let ctrl = bot.compute_controls(&trailing_car, &track, &[&lead_car], 0.016);

    // In high-speed pack draft, bot should stay pinned on full throttle and not panic brake
    assert_eq!(ctrl.brake, 0.0, "Drafting bot should not brake on high-speed straight");
    assert!(ctrl.throttle > 0.8, "Drafting bot should maintain full throttle");

    // When closing in closely (< 8m), bot executes slingshot lateral pass
    let mut close_trailing_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(92.0, 180.0), std::f32::consts::PI);
    close_trailing_car.state.speed = 70.0;
    close_trailing_car.state.velocity = Vec2::new(-70.0, 0.0);

    let slingshot_ctrl = bot.compute_controls(&close_trailing_car, &track, &[&lead_car], 0.016);
    assert!(
        slingshot_ctrl.steer.abs() > 0.01,
        "Bot closing in under draft should steer to initiate slingshot pass"
    );
}

#[test]
fn test_bot_ai_on_kart_grid_positions() {
    let track_mgr = tdrace_app::track_manager::TrackManager::default();
    for track_slug in &["lonato", "sarno", "genk", "pfi"] {
        let track = track_mgr.load_track_by_slug(track_slug).expect("Load track");
        let model = tdrace_app::catalog::find_model_by_id("kart_crg_hero_60").expect("model");
        let car_config = model.to_car_config();
        let mut cars = Vec::new();
        for grid_pose in &track.grid_positions {
            cars.push(Car::new(car_config).with_pose(grid_pose.position, grid_pose.angle));
        }
        for (i, car) in cars.iter().enumerate().skip(1) {
            let other_refs: Vec<&Car> = cars.iter().enumerate().filter(|(idx, _)| *idx != i).map(|(_, c)| c).collect();
            let mut bot = BotAiDriver::new(BotProfile::rookie());
            let ctrl = bot.compute_controls(car, &track, &other_refs, 0.016);
            assert!(ctrl.throttle > 0.05, "Track {} slot {}: bot throttle too low: {}", track_slug, i, ctrl.throttle);
            assert!(ctrl.brake < 0.2, "Track {} slot {}: bot is braking on grid: {}", track_slug, i, ctrl.brake);
        }

        // Now test stepping 180 frames with step_per_wheel and TrackProgressTracker with hint
        let mut moving_cars = cars.clone();
        let mut trackers: Vec<tdrace_core::track::checkpoint::TrackProgressTracker> = track.grid_positions.iter().map(|g| {
            let mut tr = tdrace_core::track::checkpoint::TrackProgressTracker::new(track.checkpoints.len(), 3);
            tr.sync_to_position(&track.spline, g.position);
            tr
        }).collect();
        let mut bots: Vec<BotAiDriver> = (1..moving_cars.len()).map(|_| BotAiDriver::new(BotProfile::rookie())).collect();
        for _step in 0..180 {
            let n = moving_cars.len();
            let mut controls = Vec::new();
            // Player
            controls.push(tdrace_core::CarControls { throttle: 1.0, steer: 0.0, brake: 0.0, handbrake: false, reverse: false });
            for i in 1..n {
                let other_refs: Vec<&Car> = moving_cars.iter().enumerate().filter(|(idx, _)| *idx != i).map(|(_, c)| c).collect();
                let ctrl = bots[i - 1].compute_controls(&moving_cars[i], &track, &other_refs, 1.0 / 60.0);
                assert!(!ctrl.reverse, "Track {} bot {} engaged REVERSE on starting grid!", track_slug, i);
                controls.push(ctrl);
            }
            for i in 0..n {
                let prog = trackers[i].progress_distance;
                let surfaces = track.sample_car_surfaces_with_hint(&moving_cars[i], prog);
                moving_cars[i].step_per_wheel(&controls[i], surfaces, 1.0 / 60.0);
                trackers[i].update(&moving_cars[i], &track.spline, &track.checkpoints, 1.0 / 60.0);
            }
        }
        for (i, car) in moving_cars.iter().enumerate().skip(1) {
            let dist = car.state.position.distance(cars[i].state.position);
            assert!(dist > 2.0, "Track {} bot {} did not move! Moved dist: {}, speed: {}", track_slug, i, dist, car.state.speed);
        }
    }
}

