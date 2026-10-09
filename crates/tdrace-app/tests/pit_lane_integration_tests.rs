//! Pit lane integration tests: Speed limiter enforcement, governor clamping, and HUD integration (Spec 062).

use glam::Vec2;
use race_kit::{DriveControls, RaceEvent, RaceRules, RaceWorld};
use tdrace_core::track::{Checkpoint, LineSegment, PitBox, PitLane, TrackSpline};
use tdrace_core::{Body2D, Car, CarConfig};

#[test]
fn test_pit_limiter_speed_clamping() {
    let mut track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");

    // Define pit road spline, stalls, and entry/exit gates
    let entry_gate = LineSegment::new(Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0));
    let exit_gate = LineSegment::new(Vec2::new(100.0, -10.0), Vec2::new(100.0, 10.0));
    let pit_spline = TrackSpline::from_points(
        &[Vec2::new(10.0, 0.0), Vec2::new(55.0, 0.0), Vec2::new(100.0, 0.0)],
        6.0,
        false,
    );
    let pit_box = PitBox::new(Vec2::new(50.0, 0.0), Vec2::new(1.0, 0.0), 3.0, 0.0);

    let speed_limit = PitLane::DEFAULT_ROAD_SPEED_LIMIT; // 16.67 m/s = 60 km/h
    track.pit_lane = Some(PitLane::new(
        pit_spline,
        6.0,
        speed_limit,
        vec![pit_box],
        entry_gate,
        exit_gate,
    ));

    // Register checkpoints with pit entry/exit flags
    track.checkpoints = vec![
        Checkpoint::new(0, entry_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(true, false),
        Checkpoint::new(1, exit_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(false, true),
    ];

    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
    let mut car = Car::new(CarConfig::sports_car());

    // Scenario: Vehicle racing at ~150 km/h (41.67 m/s) on the main straight approaching pit entry
    car.state.position = Vec2::new(5.0, 0.0);
    car.state.velocity = Vec2::new(41.67, 0.0);
    car.state.speed = 41.67;

    let tracker = tdrace_core::track::checkpoint::TrackProgressTracker::new(track.checkpoints.len(), 1);
    world.spawn(car, tracker);

    // Initial state: not pitting, limiter inactive
    assert!(!world.pit_states[0].is_limiter_active());

    // Step 1: Vehicle crosses pit entry gate at high speed with full throttle
    let full_throttle = vec![DriveControls::new(1.0, 0.0, 0.0, false)];
    let events = world.step(&track, &full_throttle, 0.2).to_vec();

    // Verify PitEntry event fired
    assert!(
        events.iter().any(|ev| matches!(ev, RaceEvent::PitEntry { car: 0 })),
        "Vehicle crossing is_pit_entry gate must trigger PitEntry event"
    );

    // Speed limiter must now be authoritatively active
    assert!(
        world.pit_states[0].is_limiter_active(),
        "Pit limiter must be engaged inside the pit lane"
    );

    // Speed must be clamped to <= 60 km/h (16.67 m/s)
    let speed_after_entry = world.vehicles[0].speed();
    assert!(
        speed_after_entry <= speed_limit + 0.05,
        "Vehicle speed ({:.2} m/s) must be clamped to pit speed limit ({:.2} m/s)",
        speed_after_entry,
        speed_limit
    );

    // Step 2: Continuous driving with 100% throttle inside pit lane
    for _ in 0..10 {
        world.step(&track, &full_throttle, 0.1);
        let current_speed = world.vehicles[0].speed();
        assert!(
            current_speed <= speed_limit + 0.05,
            "Speed limiter override prevention failed: speed reached {:.2} m/s under full throttle",
            current_speed
        );
    }

    // Step 3: Advance past pit exit gate
    world.vehicles[0].state.position = Vec2::new(98.0, 0.0);
    let exit_events = world.step(&track, &full_throttle, 0.2).to_vec();

    assert!(
        exit_events.iter().any(|ev| matches!(ev, RaceEvent::PitExit { car: 0 })),
        "Crossing pit exit gate must trigger PitExit event"
    );
    assert!(
        !world.pit_states[0].is_limiter_active(),
        "Pit limiter must disengage after crossing pit exit gate"
    );

    // Step 4: With limiter released, full throttle accelerates beyond 60 km/h
    for _ in 0..15 {
        world.step(&track, &full_throttle, 0.1);
    }
    assert!(
        world.vehicles[0].speed() > speed_limit + 2.0,
        "Vehicle must regain full acceleration authority after exiting pit lane"
    );
}

/// GT circuits whose pit lane is built from a spec 101 layout (`docs/circuits/pit_layout_migration.md`).
const LAYOUT_CIRCUITS: [&str; 5] = ["monza", "nurburgring_gp", "portimao_gp", "red_bull_ring", "suzuka"];

/// Every GT circuit: all have a pit lane, and spec 101 rebuilt the walls around each of them.
const GT_CIRCUITS: [&str; 18] = [
    "bahrain", "bathurst", "catalunya", "cota", "interlagos", "le_mans_sarthe", "madring", "marina_bay", "monaco",
    "montreal", "monza", "nurburgring_gp", "portimao_gp", "red_bull_ring", "silverstone", "spa", "suzuka", "zandvoort",
];

/// Converted circuits where the bot misses the entry gate at race speed, as it does with the old lanes there
/// (tdrace-l9kh4).
const MISSED_PIT_ENTRY: [&str; 4] = ["monza", "nurburgring_gp", "portimao_gp", "suzuka"];

/// Scenario: Pit stops work on a converted circuit (spec 101)
///
/// Given each converted GT circuit in a race with pit stops enabled, except MISSED_PIT_ENTRY
/// When a GT bot with tire wear above 0.70 reaches the pit entry
/// Then it enters the lane, the limiter engages, it stops in a stall, `pit_stops` increments,
/// and it rejoins the race and completes the next lap without stalling
#[test]
fn test_bot_pits_on_every_layout_circuit() {
    use race_kit::PitServiceState;
    use tdrace_app::ai::bot_harness::{sample_bot, HARNESS_DT};
    use tdrace_app::ai::{DriverTier, DrivingStyle};
    use tdrace_app::module::gt::GtWorldChallengeModule;
    use tdrace_core::track::checkpoint::TrackProgressTracker;

    let mut failures = Vec::new();
    for id in LAYOUT_CIRCUITS.into_iter().filter(|id| !MISSED_PIT_ENTRY.contains(id)) {
        let track = tdrace_core::catalog::official_track("gt", id);
        assert!(track.pit_lane_layout.is_some(), "{id} has no pit lane layout");
        let limit = track.pit_lane.as_ref().unwrap().speed_limit;
        let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
        let spawn = track.grid_positions[0];
        let mut car = Car::new(GtWorldChallengeModule::car_gt3_evo()).with_pose(spawn.position, spawn.angle);
        for w in &mut car.state.wheels {
            w.wear = 0.75;
        }
        for a in &mut car.state.wheel_assemblies {
            a.wear = 0.75;
        }
        world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
        let mut bot = sample_bot(DrivingStyle::Balanced, DriverTier::Pro, 101).with_pit_stall(0);

        let (mut entered, mut serviced, mut exit_lap, mut over_limit) = (false, false, None, 0.0f32);
        let (mut mark, mut stuck, mut worst_stuck) = (0.0f32, 0.0f32, 0.0f32);
        let len = track.spline.total_length();
        for _ in 0..(480.0 / HARNESS_DT) as usize {
            let c = bot.compute_controls(&world.vehicles[0], &track, &[], HARNESS_DT);
            let events = world.step(&track, &[c], HARNESS_DT).to_vec();
            entered |= events.iter().any(|e| matches!(e, RaceEvent::PitEntry { car: 0 }));
            serviced |= events.iter().any(|e| matches!(e, RaceEvent::PitServiceComplete { car: 0 }));
            if events.iter().any(|e| matches!(e, RaceEvent::PitExit { car: 0 })) {
                exit_lap = Some(world.trackers[0].current_lap);
            }
            if matches!(world.pit_states[0], PitServiceState::InTransit { .. }) {
                over_limit = over_limit.max(world.vehicles[0].speed() - limit);
            }
            let t = &world.trackers[0];
            let gained = (t.progress_distance - mark).rem_euclid(len);
            let in_stall = matches!(world.pit_states[0], PitServiceState::StationaryInBox { .. });
            if (gained > 5.0 && gained < 0.5 * len) || in_stall {
                mark = t.progress_distance;
                stuck = 0.0;
            } else {
                stuck += HARNESS_DT;
                worst_stuck = worst_stuck.max(stuck);
            }
            if exit_lap.is_some_and(|l| t.current_lap > l) {
                break;
            }
        }
        let t = &world.trackers[0];
        let rejoined = exit_lap.is_some_and(|l| t.current_lap > l);
        if !(entered && serviced && t.pit_stops == 1 && rejoined && over_limit <= 0.5 && worst_stuck < 15.0) {
            failures.push(format!(
                "{id}: entered {entered}, serviced {serviced}, pit_stops {}, rejoined and lapped {rejoined}, \
                 over limit {over_limit:.2} m/s, longest no-progress {worst_stuck:.1} s",
                t.pit_stops
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// GT circuits where bots already stalled on main before spec 101: a detached pit lane (tdrace-yb7yd) and the main
/// track (tdrace-a08s9).
const KNOWN_STALLS: [&str; 3] = ["bahrain", "silverstone", "zandvoort"];

/// A bot that makes no progress for this long is stuck. Spins, offs and pit stops recover in under 14 s here;
/// stuck bots stayed 88-489 s.
const STUCK_S: f32 = 30.0;

/// Scenario: bots do not stall on the GT circuits (spec 101 gameplay check)
///
/// Given each GT circuit except KNOWN_STALLS, with its own GT car
/// When a grid of 8 bots (4 Rookie, 4 Pro) races 2 laps
/// Then every bot finishes and none goes STUCK_S without progress at the new junctions and walls
///
/// Ignored in the default run: the race is chaotic, so debug and release builds send bots onto different lines, and a
/// debug run takes 17 min. In debug, bots found old main-track traps at monza s 3632 and madring s 3956 (walls equal
/// to main; tdrace-5hnlv). Run it with
/// `cargo test --release -p tdrace-app --test pit_lane_integration_tests -- --ignored`.
#[test]
#[ignore]
fn test_bots_do_not_stall_on_gt_circuits() {
    use tdrace_app::ai::bot_harness::{run_harness_race, sample_bot, HarnessEntry};
    use tdrace_app::ai::{DriverTier, DrivingStyle};
    use tdrace_app::module::gt::GtWorldChallengeModule;

    let styles = [DrivingStyle::Balanced, DrivingStyle::Aggressive, DrivingStyle::Smooth, DrivingStyle::Calculating];
    let mut failures = Vec::new();
    for id in GT_CIRCUITS.into_iter().filter(|id| !KNOWN_STALLS.contains(id)) {
        let track = tdrace_core::catalog::official_track("gt", id);
        let entries = (0..8)
            .map(|i| {
                let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
                HarnessEntry::bot(sample_bot(styles[i % styles.len()], tier, 101 + i as u64), GtWorldChallengeModule::car_gt3_evo())
            })
            .collect();
        for (i, r) in run_harness_race(&track, entries, 2, 600.0).iter().enumerate() {
            if !r.finished || r.longest_no_progress_s >= STUCK_S {
                failures.push(format!(
                    "{id}: bot {i} finished {} laps (longest no-progress {:.1} s)",
                    r.lap_times.len(),
                    r.longest_no_progress_s
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// GT circuits whose free-form pit lane starts or ends off the main road: no wall can enclose it without walling off
/// the way in, so they keep the legacy walls until the lane is redrawn (`docs/circuits/pit_layout_migration.md`).
const DETACHED_PIT_LANES: [&str; 6] = ["bahrain", "interlagos", "le_mans_sarthe", "madring", "marina_bay", "zandvoort"];

/// Scenario: The pit lane is fully enclosed (spec 101)
///
/// Given each GT circuit whose pit lane touches the main road at both ends
/// When rays are cast every 2 m from the pit lane to both sides, and from the main edge beside the pit lane outward
/// Then every ray hits a wall or reaches the main road within 40 m, so no car can leave the pit lane for open ground
#[test]
fn test_gt_pit_lanes_are_fully_enclosed() {
    use tdrace_core::track::pit_kit::{enclosure_holes, perimeter_span};
    let mut failures = Vec::new();
    for id in GT_CIRCUITS {
        let track = tdrace_core::catalog::official_track("gt", id);
        let detached = DETACHED_PIT_LANES.contains(&id);
        if detached {
            assert!(perimeter_span(&track).is_none(), "{id} now touches the main road; enclose it and drop it from the list");
            continue;
        }
        let holes = enclosure_holes(&track);
        if !holes.is_empty() {
            failures.push(format!("{id}: {} open rays, first {:?}", holes.len(), holes.first()));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
