use glam::Vec2;
use tdrace_app::ai::{BotAiDriver, BotProfile, BotRouteStrategy};
use tdrace_core::track::{
    RoadSegment, SegmentId, Track, TrackCategory, TrackLayout, TrackNetwork,
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
    // Superstretch runs at y ~= 596 near x = 100, heading towards negative X
    let mut trailing_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(100.0, 595.8), std::f32::consts::PI);
    trailing_car.state.speed = 65.0; // ~234 km/h
    trailing_car.state.velocity = Vec2::new(-65.0, 0.0);

    // Lead car slightly offset laterally (0.6 m)
    let mut lead_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(85.0, 597.2), std::f32::consts::PI);
    lead_car.state.speed = 64.0;
    lead_car.state.velocity = Vec2::new(-64.0, 0.0);

    let ctrl = bot.compute_controls(&trailing_car, &track, &[&lead_car], 0.016);

    // In high-speed pack draft, bot should stay pinned on full throttle and not panic brake
    assert_eq!(ctrl.brake, 0.0, "Drafting bot should not brake on high-speed straight");
    assert!(ctrl.throttle > 0.8, "Drafting bot should maintain full throttle");

    // When closing in closely (< 8m), bot executes slingshot lateral pass
    let mut close_trailing_car = Car::new(CarConfig::stock_car_ta1()).with_pose(Vec2::new(92.0, 596.2), std::f32::consts::PI);
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
        let model = tdrace_app::catalog::find_model_by_id("kart_blackline_cadet_t1").expect("model");
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
        ..Default::default()
    };

    let spline = network.build_composite_spline_for_layout("main").unwrap();

    Track {
        name: "Test Branching Circuit".to_string(),
        description: "Test circuit with split and merge junctions".to_string(),
        category: TrackCategory::Main,
        spline,
        network: Some(network),
        default_surface: SurfaceType::Asphalt,
        default_laps: 3,
        ..Track::default()
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
    let other: [&Car; 0] = [];
    let layout1 = bot.decide_active_layout(&track, &other);
    assert_eq!(layout1, Some("main".to_string()));

    // Lap 2: planned joker lap reached -> joker layout chosen
    bot.current_lap = 2;
    let layout2 = bot.decide_active_layout(&track, &other);
    assert_eq!(layout2, Some("joker".to_string()));

    // Completed joker lap; now on Lap 3: return to main layout
    bot.joker_laps_taken = 1;
    bot.current_lap = 3;
    let layout3 = bot.decide_active_layout(&track, &other);
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


/// Scenario: Bots take exactly one joker and never switch inside a branch (spec 082), unless their car has left
/// its route onto the other branch, which they then follow and which can be an extra joker (spec 088)
#[test]
fn test_bot_ai_strategic_joker_rx_race_compliance() {
    use race_kit::{DriveControls, JokerRule, RaceFormat, RaceRules, RaceWorld};
    use tdrace_app::ai::BotRaceState;
    use tdrace_core::track::TrackProgressTracker;

    const DT: f32 = 1.0 / 60.0;
    const LAPS: u32 = 5;
    // Bots get stuck on the Classic RX tracks even on the main route (tdrace-lxkv), so this
    // uses two World RX tracks where a bot drives both routes cleanly.
    for (module, id) in [("rally", "holjes_rx"), ("rally", "hell_rx")] {
        let track = tdrace_core::catalog::official_track(module, id);
        let network = track.network.as_ref().expect("RX network");
        let main = network.get_layout("main").unwrap().segment_sequence.clone();
        let joker = network.get_layout("joker").unwrap().segment_sequence.clone();
        let route_splines: Vec<(String, _)> = ["main", "joker"]
            .iter()
            .map(|l| (l.to_string(), network.build_composite_spline_for_layout(l).unwrap()))
            .collect();
        let rules = RaceRules {
            format: RaceFormat::Laps(LAPS),
            joker: JokerRule { mandatory: 1, penalty_s: 30.0 },
            ..RaceRules::default()
        };
        let mut world: RaceWorld<Car> = RaceWorld::new(rules);
        let mut bots = Vec::new();
        for i in 0..8 {
            let slot = track.grid_positions[i];
            world.spawn(
                Car::new(CarConfig::rally_car()).with_pose(slot.position, slot.angle),
                TrackProgressTracker::new(track.checkpoints.len(), 3),
            );
            let profile = [BotProfile::pro(), BotProfile::aggressive(), BotProfile::balanced(), BotProfile::rookie()][i % 4];
            let strategy = match i % 3 {
                0 => BotRouteStrategy::RallycrossJoker { planned_joker_lap: 2, adaptive_traffic_undercut: true },
                1 => BotRouteStrategy::RallycrossJoker { planned_joker_lap: 3, adaptive_traffic_undercut: false },
                _ => BotRouteStrategy::DynamicTrafficAvoidance,
            };
            bots.push(BotAiDriver::with_seed(profile, 40 + i as u64).with_route_strategy(strategy));
        }

        let mut branch_switches = Vec::new();
        let mut onto_joker = [false; 8];
        for _ in 0..(600.0 / DT) as usize {
            let mut controls = Vec::with_capacity(8);
            for i in 0..8 {
                bots[i].sync_race_state(BotRaceState {
                    lap: world.trackers[i].current_lap,
                    jokers: world.jokers_taken(i),
                    total_laps: LAPS,
                    mandatory_jokers: 1,
                });
                let before = bots[i].active_layout_id.clone();
                let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
                controls.push(bots[i].compute_controls(&world.vehicles[i], &track, &others, DT));
                let seg = world.trackers[i].multi_route.as_ref().map(|m| m.current_segment_id);
                let in_branch = seg.is_some_and(|s| !(main.contains(&s) && joker.contains(&s)));
                if before.is_some() && before != bots[i].active_layout_id && in_branch {
                    // Allowed only where the car is off the road of the route it leaves.
                    let pos = world.vehicles[i].state.position;
                    let old_route = &route_splines.iter().find(|(l, _)| Some(l) == before.as_ref()).unwrap().1;
                    let proj = old_route.project_point(pos);
                    if proj.distance_to_spline > proj.track_width * 0.5 {
                        onto_joker[i] |= bots[i].active_layout_id.as_deref() == Some("joker");
                    } else {
                        branch_switches.push((i, world.trackers[i].current_lap, before, bots[i].active_layout_id.clone()));
                    }
                }
            }
            let controls: Vec<DriveControls> = controls;
            world.step(&track, &controls, DT);
            if (0..8).all(|i| world.finish[i] != race_kit::FinishState::Racing) {
                break;
            }
        }

        let finished: Vec<usize> = (0..8).filter(|&i| world.is_finished(i)).collect();
        assert!(finished.len() >= 6, "{}: only {} of 8 bots finished: {:?}", id, finished.len(), world.finish);
        for &i in &finished {
            if onto_joker[i] {
                assert!(world.jokers_taken(i) >= 1, "{}: bot {} jokers", id, i);
            } else {
                assert_eq!(world.jokers_taken(i), 1, "{}: bot {} jokers", id, i);
            }
            assert_eq!(world.penalty[i], 0.0, "{}: bot {} penalty", id, i);
        }
        assert!(branch_switches.is_empty(), "{}: bots switched route inside a branch: {:?}", id, branch_switches);
    }
}
