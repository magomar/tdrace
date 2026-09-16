use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile, BotRouteStrategy};
use tdrace_core::track::presets::{classic_grand_prix, oval_speedway};
use tdrace_core::track::{
    RoadSegment, SegmentId, Track, TrackCategory, TrackGeometry, TrackLayout, TrackNetwork,
    TrackWaypoint,
};
use tdrace_core::{Car, CarConfig, SurfaceType};

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
    let track = classic_grand_prix();
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
    let track = oval_speedway();
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
    let track = classic_grand_prix();
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
    let track = tdrace_core::track::presets::daytona_superspeedway();
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

fn create_test_branching_track() -> Track {
    let seg0 = RoadSegment::new(
        SegmentId(0),
        "Trunk Entry",
        vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
        ],
    );

    let seg1 = RoadSegment::new(
        SegmentId(1),
        "Main Branch",
        vec![
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(110.0, -25.0), 12.0),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
        ],
    );

    let seg2 = RoadSegment::new(
        SegmentId(2),
        "Joker Branch",
        vec![
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(110.0, 35.0), 12.0),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
        ],
    );

    let seg3 = RoadSegment::new(
        SegmentId(3),
        "Common Return",
        vec![
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(220.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(220.0, -60.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, -60.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
        ],
    );

    let layout_main = TrackLayout::new(
        "main",
        "Grand Prix Circuit",
        vec![SegmentId(0), SegmentId(1), SegmentId(3)],
        SegmentId(0),
    );

    let layout_joker = TrackLayout::new(
        "joker",
        "Rallycross Joker Route",
        vec![SegmentId(0), SegmentId(2), SegmentId(3)],
        SegmentId(0),
    );

    let network = TrackNetwork {
        junctions: Vec::new(),
        segments: vec![seg0, seg1, seg2, seg3],
        layouts: vec![layout_main, layout_joker],
        default_layout_id: "main".to_string(),
    };

    let spline = network.build_composite_spline_for_layout("main").unwrap();

    Track {
        name: "Test Branching Circuit".to_string(),
        description: "Test circuit with split and merge junctions".to_string(),
        category: TrackCategory::Main,
        spline,
        network: Some(network),
        geometry: TrackGeometry::default(),
        checkpoints: Vec::new(),
        grid_positions: Vec::new(),
        default_surface: SurfaceType::Asphalt,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: None,
        module_id: None,
        modules: Vec::new(),
    }
}

#[test]
fn test_bot_ai_multi_route_split_junction_navigation() {
    let track = create_test_branching_track();
    let mut bot_main = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("main".to_string()));
    let mut bot_joker = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("joker".to_string()));

    // Cars approach split junction at x=45.0 heading along trunk towards x=60.0
    let car_main = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(45.0, 0.0), 0.0);
    let car_joker = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(45.0, 0.0), 0.0);

    let ctrl_main = bot_main.compute_controls(&car_main, &track, &[], 0.016);
    let ctrl_joker = bot_joker.compute_controls(&car_joker, &track, &[], 0.016);

    // Both bots should accelerate positively along the trunk
    assert!(ctrl_main.throttle > 0.5);
    assert!(ctrl_joker.throttle > 0.5);

    // Steering commands across the split bifurcation must steer into their respective branch
    assert!(
        ctrl_main.steer != ctrl_joker.steer,
        "Main and Joker routes must produce distinct steering angles at split junction"
    );
    assert!(
        ctrl_main.steer * ctrl_joker.steer < 0.0,
        "Main (diverging right) and Joker (diverging left) steering commands must have opposite signs"
    );
}

#[test]
fn test_bot_ai_strategic_joker_lap_decision() {
    let track = create_test_branching_track();
    let mut bot = BotAiDriver::new(BotProfile::pro()).with_route_strategy(
        BotRouteStrategy::RallycrossJoker {
            planned_joker_lap: 2,
            adaptive_traffic_undercut: true,
        },
    );

    // Lap 1: main layout chosen
    bot.current_lap = 1;
    let layout1 = bot.decide_active_layout(&track, &[]);
    assert_eq!(layout1, Some("main".to_string()));

    // Lap 2: planned joker lap reached -> joker layout chosen
    bot.current_lap = 2;
    let layout2 = bot.decide_active_layout(&track, &[]);
    assert_eq!(layout2, Some("joker".to_string()));

    // Completed joker lap; now on Lap 3: return to main layout
    bot.joker_laps_taken = 1;
    bot.current_lap = 3;
    let layout3 = bot.decide_active_layout(&track, &[]);
    assert_eq!(layout3, Some("main".to_string()));
}

#[test]
fn test_bot_ai_dynamic_traffic_avoidance() {
    let track = create_test_branching_track();
    let mut bot = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::DynamicTrafficAvoidance);

    // Place 3 opponent cars along the main branch (around x=110, y=-25)
    let opp1 = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(100.0, -25.0), 0.0);
    let opp2 = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(110.0, -25.0), 0.0);
    let opp3 = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(120.0, -25.0), 0.0);

    let chosen = bot.decide_active_layout(&track, &[&opp1, &opp2, &opp3]);
    // The joker branch has 0 opponents, so traffic avoidance chooses "joker"
    assert_eq!(chosen, Some("joker".to_string()));
}

#[test]
fn test_multi_bot_branching_race_simulation() {
    let track = create_test_branching_track();
    let mut bot_main = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("main".to_string()));
    let mut bot_joker = BotAiDriver::new(BotProfile::pro())
        .with_route_strategy(BotRouteStrategy::FixedLayout("joker".to_string()));

    let mut car_main = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(15.0, -2.0), 0.0);
    let mut car_joker = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 2.0), 0.0);

    let dt = 0.016;
    for _ in 0..200 {
        let ctrl_m = bot_main.compute_controls(&car_main, &track, &[&car_joker], dt);
        let ctrl_j = bot_joker.compute_controls(&car_joker, &track, &[&car_main], dt);

        car_main.step(&ctrl_m, SurfaceType::Asphalt, dt);
        car_joker.step(&ctrl_j, SurfaceType::Asphalt, dt);

        assert!(bot_main.stuck_timer < 0.5, "Main bot must not get stuck");
        assert!(bot_joker.stuck_timer < 0.5, "Joker bot must not get stuck");
    }

    // Both cars have progressed down the track
    assert!(car_main.state.position.x > 30.0);
    assert!(car_joker.state.position.x > 30.0);
    assert!(car_main.state.speed > 8.0);
    assert!(car_joker.state.speed > 8.0);
}

