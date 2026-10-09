//! Spec 102 (`specs/102_predefined_junction_components_for_road_splits_and_joker_loops.md`) on the main splines of
//! real circuits.
//!
//! No official joker converts at the spec's limits (`docs/circuits/branch_junction_migration.md`: the old jokers leave
//! the main road too steeply, too slowly or too tightly for a 10-40 m Taper or TurnOff). These tests therefore put a
//! new joker, built from two junction components and a free road, on the main spline of nine RX circuits, and check
//! walls, validation, surfaces, routes, the nose and the bots on them.

use race_kit::{DriveControls, JokerRule, RaceEvent, RaceFormat, RaceRules, RaceWorld};
use tdrace_app::ai::bot_harness::sample_bot;
use tdrace_app::ai::{BotRaceState, BotRouteStrategy, DriverTier, DrivingStyle};
use tdrace_core::physics::{Car, CarConfig};
use tdrace_core::track::bake::{bake, BakeOptions};
use tdrace_core::track::branch_kit::{self, nose_barrier, BranchLayout};
use tdrace_core::track::geometry::BarrierType;
use tdrace_core::track::junction_kit::{JunctionComponent, JunctionShape, Side};
use tdrace_core::track::network::JunctionKind;
use tdrace_core::track::spline::TrackWaypoint;
use tdrace_core::track::{validate_track, MultiRouteProgressTracker, Track, TrackProgressTracker, ValidationSeverity};
use tdrace_core::SurfaceType;

const DT: f32 = 1.0 / 60.0;

/// (module, circuit, side, junction length, divider gap): circuits on which the free road below compiles.
const CIRCUITS: [(&str, &str, Side, f32, f32); 9] = [
    ("classic", "rx_quarry_sprint", Side::Left, 40.0, 5.0),
    ("classic", "rx_hilltop_leap", Side::Left, 30.0, 4.0),
    ("classic", "rx_canyon_flyer", Side::Left, 30.0, 4.0),
    ("rally", "lydden_hill", Side::Left, 40.0, 5.0),
    ("rally", "hell_rx", Side::Left, 40.0, 5.0),
    ("rally", "mettet_rx", Side::Right, 30.0, 4.0),
    ("rally", "killarney_rx", Side::Left, 40.0, 5.0),
    ("rally", "spa_rx", Side::Right, 40.0, 5.0),
    ("rally", "holjes_rx", Side::Right, 40.0, 5.0),
];

/// The official circuit with its joker replaced by a branch layout: split and merge on the waypoints the old joker
/// used, and a free road that runs beside the main line, 17 m out.
fn circuit_with_branch(module: &str, id: &str, side: Side, length: f32, gap: f32) -> Track {
    let mut track = tdrace_core::catalog::official_track(module, id);
    // The waypoints the old joker split and merged at: the main waypoints nearest its junction sockets. The legacy
    // network stays on the track until `install_new`, which stamps a launch chute (spec 103) again on the new one.
    let legacy = track.network.clone().expect("an RX circuit has a joker network");
    let nearest = |p: glam::Vec2| {
        (0..track.spline.waypoints.len())
            .min_by(|&a, &b| track.spline.waypoints[a].point.distance(p).total_cmp(&track.spline.waypoints[b].point.distance(p)))
            .unwrap()
    };
    let socket = |j: u32| match &legacy.get_junction(tdrace_core::track::network::JunctionId(j)).unwrap().kind {
        JunctionKind::Split { ingress_socket, .. } => ingress_socket.point,
        JunctionKind::Merge { egress_socket, .. } => egress_socket.point,
        JunctionKind::Terminal { socket } => socket.point,
    };
    let (split_idx, merge_idx) = (nearest(socket(0)), nearest(socket(1)));
    let main = track.spline.clone();
    let station = |i: usize| main.project_point(main.waypoints[i].point).progress_distance;
    let (s_split, s_merge) = (station(split_idx), station(merge_idx));

    let lateral = main.sample_at_distance(s_split).width * 0.5 + gap + 6.5;
    let mut road_waypoints = Vec::new();
    let mut s = s_split + length + 10.0;
    while s < s_merge - length - 10.0 {
        let c = main.sample_at_distance(s);
        let mut wp = TrackWaypoint::new(c.point + c.normal * side.sign() * lateral, 13.0).with_surface(SurfaceType::Dirt);
        wp.elevation = c.elevation;
        road_waypoints.push(wp);
        s += 25.0;
    }
    let layout = BranchLayout {
        layout_id: "joker".to_string(),
        name: format!("{id} test joker"),
        side,
        split: JunctionComponent { s: s_split, kind: JunctionShape::Taper, length, divider_gap: gap },
        merge: JunctionComponent { s: s_merge, kind: JunctionShape::Taper, length, divider_gap: gap },
        road_waypoints,
        road_width: 13.0,
        nose_barrier: BarrierType::TireWall,
    };
    track.branch_layout = Some(layout.clone());
    let compiled = layout.compile(&track).unwrap_or_else(|e| panic!("{id}: {e:?}"));
    branch_kit::install_new(&mut track, compiled).unwrap_or_else(|e| panic!("{id}: {e}"));
    bake(&mut track, &BakeOptions { rebuild: true, ..BakeOptions::default() }).unwrap_or_else(|e| panic!("{id}: {e}"));
    track
}

#[test]
fn test_branch_circuits_validate_without_error_or_junction_warning() {
    for (module, id, side, length, gap) in CIRCUITS {
        let official = tdrace_core::catalog::official_track(module, id);
        let track = circuit_with_branch(module, id, side, length, gap);
        // The launch chute (spec 103) is stamped again on the new network, with the same grid.
        assert!(official.launch_chute().is_some(), "{id}: an official RX circuit has a launch chute");
        assert_eq!(track.launch_chute().map(|c| c.grid_slots.len()), official.launch_chute().map(|c| c.grid_slots.len()), "{id}: chute grid");
        assert_eq!(track.grid_positions.len(), official.grid_positions.len(), "{id}: starting grid");
        let bad: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error || d.code.contains("JUNCTION"))
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        assert!(bad.is_empty(), "{id}: {bad:#?}");
    }
}

/// Scenario: the main route does not change
#[test]
fn test_branch_circuits_keep_the_main_route() {
    for (module, id, side, length, gap) in CIRCUITS {
        let official = tdrace_core::catalog::official_track(module, id);
        let track = circuit_with_branch(module, id, side, length, gap);
        assert_eq!(track.spline.waypoints, official.spline.waypoints, "{id}: main waypoints");
        // The plan geometry of every sample. Elevation, slope and vertical curvature are derived: the committed samples
        // of rx_hilltop_leap were baked by older code, and a rebuild recomputes them (3.000798 m for 3.0 m).
        for (i, (a, b)) in track.spline.samples.iter().zip(&official.spline.samples).enumerate() {
            assert!(
                a.point == b.point && a.tangent == b.tangent && a.distance == b.distance && a.width == b.width,
                "{id}: main spline sample {i} moved:\n{a:?}\n{b:?}"
            );
        }
        assert_eq!(track.spline.samples.len(), official.spline.samples.len(), "{id}: main spline sample count");
        // The main layout is the same road again, cut at the anchors.
        let main = track.network.as_ref().unwrap().build_composite_spline_for_layout("main").unwrap();
        assert_eq!(main.waypoints.len(), official.spline.waypoints.len() + 1, "{id}: the closing waypoint of the loop is repeated");
        for (a, b) in main.waypoints.iter().zip(&official.spline.waypoints) {
            assert!(a.point.distance(b.point) < 1e-4, "{id}: main layout waypoint moved");
        }
    }
}

/// Scenario: a car on the branch route gets its lap and its joker, with no wall hit and no surface mismatch.
#[test]
fn test_car_driving_the_branch_route_gets_its_lap_and_its_joker() {
    const SPEED: f32 = 15.0;
    for (module, id, side, length, gap) in CIRCUITS {
        let track = circuit_with_branch(module, id, side, length, gap);
        let network = track.network.as_ref().unwrap();
        let joker = network.build_composite_spline_for_layout("joker").unwrap();
        let main = network.build_composite_spline_for_layout("main").unwrap();
        let route = [(&joker, 1.0f32), (&main, 0.0f32)];

        let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules {
            format: RaceFormat::Laps(2),
            joker: JokerRule { mandatory: 1, penalty_s: 30.0 },
            ..RaceRules::default()
        });
        let start = joker.sample_at_distance(1.0);
        // The car starts on the loop, not on a launch chute: so does its tracker (a lap of the loop, as on laps 2+).
        let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
        tracker.multi_route = Some(MultiRouteProgressTracker::new("main", network.get_layout("main").unwrap().segment_sequence[0], 3));
        world.spawn(Car::new(CarConfig::rally_car()).with_pose(start.point, start.tangent.y.atan2(start.tangent.x)), tracker);
        let mut wall_hit = None;
        let mut wrong_way = None;
        'laps: for (spline, from) in route {
            let mut d = from;
            while d < spline.total_length() {
                let s = spline.sample_at_distance(d);
                let car = &mut world.vehicles[0];
                car.state.position = s.point;
                car.state.angle = s.tangent.y.atan2(s.tangent.x);
                car.set_velocity(s.tangent * SPEED);
                let events = world.step(&track, &[DriveControls::default()], DT);
                if events.iter().any(|e| matches!(e, RaceEvent::WallImpact { .. })) && wall_hit.is_none() {
                    wall_hit = Some(format!("at {:.0} m of the {} lap", d, if from > 0.0 { "joker" } else { "main" }));
                }
                if world.trackers[0].is_wrong_way && wrong_way.is_none() {
                    wrong_way = Some(format!("at {:.0} m", d));
                }
                if world.is_finished(0) {
                    break 'laps;
                }
                d += SPEED * DT;
            }
        }
        for k in 1..=8 {
            if world.is_finished(0) {
                break;
            }
            let s = main.sample_at_distance(k as f32 * SPEED * DT);
            world.vehicles[0].state.position = s.point;
            world.step(&track, &[DriveControls::default()], DT);
        }
        let tracker = &world.trackers[0];
        let jokers = tracker.multi_route.as_ref().map_or(0, |m| m.joker_laps_completed);
        assert!(
            world.is_finished(0) && tracker.current_lap == 3 && jokers == 1 && wall_hit.is_none() && wrong_way.is_none(),
            "{id}: finished {}, lap {}, jokers {}, wall hit {:?}, wrong way {:?}",
            world.is_finished(0),
            tracker.current_lap,
            jokers,
            wall_hit,
            wrong_way
        );
    }
}

/// Scenario: the island is closed by a collidable nose: a car driven straight at it at 20 m/s stops on it.
#[test]
fn test_car_driven_at_the_nose_stops_on_it() {
    for (module, id, side, length, gap) in CIRCUITS {
        let track = circuit_with_branch(module, id, side, length, gap);
        let layout = track.branch_layout.clone().unwrap();
        let compiled = layout.compile(&track).unwrap();
        let (nose, _) = nose_barrier(&compiled.geometry.split, BarrierType::TireWall);
        let at = nose.segment.midpoint();
        // Straight at the nose: along the bisector of the two road edges, perpendicular to the barrier.
        let mut dir = nose.segment.direction().perp();
        if dir.dot(track.spline.project_point(at).tangent) < 0.0 {
            dir = -dir;
        }

        let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::TimeAttack, ..RaceRules::default() });
        let start = at - dir * 10.0;
        world.spawn(Car::new(CarConfig::rally_car()).with_pose(start, dir.y.atan2(dir.x)), TrackProgressTracker::new(track.checkpoints.len(), 3));
        world.vehicles[0].set_velocity(dir * 20.0);
        let (mut hit, mut deepest) = (false, f32::MIN);
        for _ in 0..(3.0 / DT) as usize {
            let events = world.step(&track, &[DriveControls::default()], DT);
            hit |= events.iter().any(|e| matches!(e, RaceEvent::WallImpact { .. }));
            deepest = deepest.max((world.vehicles[0].state.position - at).dot(dir));
        }
        assert!(hit, "{id}: the car never touched the nose");
        // The car's centre never gets past the barrier: its nose stops on it, half a car length short at most.
        assert!(deepest < 0.5, "{id}: the car got {deepest:.2} m past the nose line");
        assert!(world.vehicles[0].state.speed < 6.0, "{id}: still moving at {:.1} m/s", world.vehicles[0].state.speed);
    }
}

/// Scenario: bots take the joker once and do not stall.
#[test]
fn test_bots_take_the_joker_once_and_do_not_stall() {
    const LAPS: u32 = 3;
    const MAX_NO_PROGRESS_S: f32 = 10.0;
    for (module, id, side, length, gap) in [CIRCUITS[1], CIRCUITS[3]] {
        let track = circuit_with_branch(module, id, side, length, gap);
        let rule = JokerRule { mandatory: 1, penalty_s: 30.0 };
        let mut world: RaceWorld<Car> =
            RaceWorld::new(RaceRules { format: RaceFormat::Laps(LAPS), joker: rule, ..RaceRules::default() });
        let styles = [DrivingStyle::Balanced, DrivingStyle::Aggressive, DrivingStyle::Smooth, DrivingStyle::Calculating];
        let mut bots = Vec::new();
        for (i, style) in styles.iter().enumerate() {
            let spawn = track.grid_positions[i];
            world.spawn(
                Car::new(CarConfig::rally_car()).with_pose(spawn.position, spawn.angle),
                TrackProgressTracker::new(track.checkpoints.len(), 3),
            );
            let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
            bots.push(sample_bot(*style, tier, 7 + i as u64).with_route_strategy(BotRouteStrategy::RallycrossJoker {
                planned_joker_lap: 2,
                adaptive_traffic_undercut: false,
            }));
        }
        let len = track.spline.total_length();
        let mut mark = vec![0.0f32; bots.len()];
        let mut stalled = vec![0.0f32; bots.len()];
        let mut longest = vec![0.0f32; bots.len()];
        for _ in 0..(600.0 / DT) as usize {
            if (0..bots.len()).all(|i| world.is_finished(i)) {
                break;
            }
            let mut controls = Vec::new();
            for i in 0..bots.len() {
                let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
                bots[i].sync_race_state(BotRaceState {
                    lap: world.trackers[i].current_lap,
                    jokers: world.jokers_taken(i),
                    total_laps: LAPS,
                    mandatory_jokers: rule.mandatory,
                });
                controls.push(bots[i].compute_controls(&world.vehicles[i], &track, &others, DT));
            }
            world.step(&track, &controls, DT);
            for i in 0..bots.len() {
                // From lap 2 on: lap 1 is the launch from the chute (spec 103), whose stalls are that spec's concern.
                if world.trackers[i].current_lap < 2 {
                    mark[i] = world.trackers[i].progress_distance;
                    continue;
                }
                let gained = (world.trackers[i].progress_distance - mark[i]).rem_euclid(len);
                if (5.0..0.5 * len).contains(&gained) || world.is_finished(i) {
                    mark[i] = world.trackers[i].progress_distance;
                    stalled[i] = 0.0;
                } else {
                    stalled[i] += DT;
                    longest[i] = longest[i].max(stalled[i]);
                }
            }
        }
        for i in 0..bots.len() {
            let jokers = world.jokers_taken(i);
            assert!(world.is_finished(i), "{id}: bot {i} did not finish");
            assert_eq!(jokers, 1, "{id}: bot {i} took {jokers} jokers");
            assert!(longest[i] < MAX_NO_PROGRESS_S, "{id}: bot {i} went {:.1} s without progress", longest[i]);
        }
    }
}

/// Scenario: Track Studio keeps the branch layout. It loads and saves the layout without a change.
#[test]
fn test_branch_layout_survives_a_json_round_trip_and_track_studio() {
    use tdrace_app::editor::EditorState;
    let (module, id, side, length, gap) = CIRCUITS[0];
    let track = circuit_with_branch(module, id, side, length, gap);
    let layout = track.branch_layout.clone().unwrap();

    let loaded = Track::from_json(&track.to_json().unwrap()).unwrap();
    assert_eq!(loaded.branch_layout, Some(layout.clone()));
    assert_eq!(loaded.geometry.network_walls, track.geometry.network_walls, "the walls are rebuilt, never saved");

    let state = EditorState::new(loaded);
    let saved = state.track.to_json().unwrap();
    assert!(!saved.contains("network_walls"));
    assert_eq!(Track::from_json(&saved).unwrap().branch_layout, Some(layout));
}

/// Scenario: Migration converts or reports each RX circuit. The report names exactly the circuits that carry a layout.
#[test]
fn test_migration_report_matches_the_circuits() {
    let report = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/circuits/branch_junction_migration.md"))
        .expect("docs/circuits/branch_junction_migration.md");
    let rows: Vec<(&str, bool)> = report
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| Circuit") && !l.starts_with("|--"))
        .map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            (cells[1], cells[2] == "converted")
        })
        .collect();
    assert_eq!(rows.len(), 23, "one row per RX circuit: {rows:?}");
    for (id, converted) in rows {
        let module = if id.starts_with("rx_") { "classic" } else { "rally" };
        let track = tdrace_core::catalog::official_track(module, id);
        assert_eq!(track.branch_layout.is_some(), converted, "{id}: report says converted = {converted}");
        if !converted {
            assert!(track.network.is_some(), "{id}: a circuit that keeps its legacy joker keeps its network");
        }
    }
}
