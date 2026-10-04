//! Spec 082 (`specs/082_mandatory_rallycross_joker_lap_tracking_and_penalty_enforcement.md`): the joker
//! rule, its penalty, and the order of standings and results.

use arcade_race_core::track::checkpoint::{Checkpoint, TrackProgressTracker};
use arcade_race_core::track::geometry::LineSegment;
use arcade_race_core::track::network::{RoadSegment, SegmentId, TrackLayout, TrackNetwork};
use arcade_race_core::track::spline::{TrackSpline, TrackWaypoint};
use arcade_race_core::track::Track;
use glam::Vec2;
use race_kit::{DriveControls, FinishState, JokerRule, RaceFormat, RaceRules, RaceWorld};
use wheelbase::{Car, CarConfig};

const DT: f32 = 1.0 / 60.0;

/// A loop with a trunk, a main branch, a longer joker branch and a return straight.
fn joker_track() -> Track {
    let wp = |x: f32, y: f32| TrackWaypoint::new(Vec2::new(x, y), 12.0);
    let segments = vec![
        RoadSegment::new(SegmentId(0), "Trunk", vec![wp(0.0, 0.0), wp(60.0, 0.0)]),
        RoadSegment::new(SegmentId(1), "Main", vec![wp(60.0, 0.0), wp(110.0, -25.0), wp(160.0, 0.0)]),
        RoadSegment::new(SegmentId(2), "Joker", vec![wp(60.0, 0.0), wp(110.0, 45.0), wp(160.0, 0.0)]),
        RoadSegment::new(
            SegmentId(3),
            "Return",
            vec![wp(160.0, 0.0), wp(220.0, 0.0), wp(220.0, -80.0), wp(0.0, -80.0), wp(0.0, 0.0)],
        ),
    ];
    let layouts = vec![
        TrackLayout::new("main", "Main", vec![SegmentId(0), SegmentId(1), SegmentId(3)], SegmentId(0))
            .with_checkpoints(vec![0, 1, 2, 4]),
        TrackLayout::new("joker", "Joker", vec![SegmentId(0), SegmentId(2), SegmentId(3)], SegmentId(0))
            .with_checkpoints(vec![0, 1, 3, 4]),
    ];
    let gate = |id: usize, x: f32, y: f32, finish: bool, seg: u32| {
        Checkpoint::new(id, LineSegment::new(Vec2::new(x, y - 10.0), Vec2::new(x, y + 10.0)), Vec2::X, 0, finish)
            .with_segment(SegmentId(seg))
    };
    let checkpoints = vec![
        gate(0, 0.0, 0.0, true, 0),
        gate(1, 40.0, 0.0, false, 0),
        gate(2, 110.0, -25.0, false, 1),
        gate(3, 110.0, 45.0, false, 2).with_joker(true),
        gate(4, 190.0, 0.0, false, 3),
    ];
    let network = TrackNetwork { segments, layouts, default_layout_id: "main".to_string(), ..Default::default() };
    let spline = network.build_composite_spline_for_layout("main").expect("main spline");
    Track { spline, network: Some(network), checkpoints, ..Track::default() }
}

/// Moves each car along its own route (one spline per lap) at its own speed and steps the world until
/// every car has finished.
fn race(track: &Track, rules: RaceRules, routes: &[(Vec<&str>, f32)]) -> RaceWorld<Car> {
    let network = track.network.as_ref().unwrap();
    let splines: Vec<Vec<TrackSpline>> = routes
        .iter()
        .map(|(laps, _)| laps.iter().map(|l| network.build_composite_spline_for_layout(l).unwrap()).collect())
        .collect();
    let mut world: RaceWorld<Car> = RaceWorld::new(rules);
    for laps in &splines {
        let s = laps[0].sample_at_distance(1.0);
        world.spawn(
            Car::new(CarConfig::sports_car()).with_pose(s.point, s.tangent.y.atan2(s.tangent.x)),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
    }
    let mut dist = vec![1.0f32; routes.len()];
    let controls = vec![DriveControls::default(); routes.len()];
    for _ in 0..(300.0 / DT) as usize {
        for (i, laps) in splines.iter().enumerate() {
            if world.is_finished(i) {
                continue;
            }
            let (mut lap, mut d) = (0, dist[i]);
            while lap + 1 < laps.len() && d >= laps[lap].total_length() {
                d -= laps[lap].total_length();
                lap += 1;
            }
            let s = laps[lap].sample_at_distance(d % laps[lap].total_length());
            let car = &mut world.vehicles[i];
            car.state.position = s.point;
            car.state.angle = s.tangent.y.atan2(s.tangent.x);
            car.set_velocity(s.tangent * routes[i].1);
            dist[i] += routes[i].1 * DT;
        }
        world.step(track, &controls, DT);
        if (0..routes.len()).all(|i| world.is_finished(i)) {
            break;
        }
    }
    assert!((0..routes.len()).all(|i| world.is_finished(i)), "every car must finish: {:?}", world.finish);
    world
}

fn rx_rules() -> RaceRules {
    RaceRules { format: RaceFormat::Laps(5), joker: JokerRule { mandatory: 1, penalty_s: 30.0 }, ..RaceRules::default() }
}

/// Scenario: A missed joker adds 30 s and drops the driver down the order
#[test]
fn missed_joker_adds_penalty_and_drops_the_driver_behind_a_slower_finisher() {
    let track = joker_track();
    // 5 laps. Car 0 skips the joker and is a little faster; car 1 takes the joker on lap 3.
    let world = race(
        &track,
        rx_rules(),
        &[(vec!["main"; 5], 21.0), (vec!["main", "main", "joker", "main", "main"], 20.0)],
    );

    let (t0, t1) = match (world.finish[0], world.finish[1]) {
        (FinishState::Finished { time: a, position: 1 }, FinishState::Finished { time: b, position: 2 }) => (a, b),
        other => panic!("car 0 must cross the line first: {:?}", other),
    };
    assert!(t1 - t0 > 0.0 && t1 - t0 < 30.0, "car 1 crosses {:.1} s later, less than the penalty", t1 - t0);
    assert_eq!(world.jokers_taken(0), 0);
    assert_eq!(world.jokers_taken(1), 1);
    assert_eq!(world.penalty, vec![30.0, 0.0]);
    assert_eq!(world.standings(), vec![1, 0]);

    let rows = world.results(&track);
    assert_eq!((rows[0].car, rows[0].position, rows[0].penalty, rows[0].jokers), (1, 1, 0.0, 1));
    assert_eq!((rows[1].car, rows[1].position, rows[1].penalty, rows[1].jokers), (0, 2, 30.0, 0));
    assert_eq!(rows[1].time, t0, "time stays the raw finish time; the penalty is separate");
}

/// Scenario: A driver who took the joker gets no penalty
#[test]
fn driver_who_took_the_joker_gets_no_penalty() {
    let track = joker_track();
    let world = race(&track, rx_rules(), &[(vec!["main", "main", "joker", "main", "main"], 20.0)]);
    let row = world.results(&track)[0];
    assert_eq!((row.penalty, row.jokers), (0.0, 1));
}

/// Scenario: Other categories are not affected (the rule is off by default)
#[test]
fn default_rules_never_penalise() {
    let track = joker_track();
    let rules = RaceRules { format: RaceFormat::Laps(2), ..RaceRules::default() };
    assert_eq!(rules.joker, JokerRule { mandatory: 0, penalty_s: 0.0 });
    let world = race(&track, rules, &[(vec!["main", "main"], 22.0), (vec!["joker", "main"], 20.0)]);
    assert_eq!(world.penalty, vec![0.0, 0.0]);
    assert_eq!(world.standings(), vec![0, 1]);
    let rows = world.results(&track);
    assert_eq!((rows[0].car, rows[1].car), (0, 1));
}
