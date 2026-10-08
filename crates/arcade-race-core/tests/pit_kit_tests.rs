//! Spec 101: parametric pit lane kit (junction components, guards).

use arcade_race_core::track::pit_kit::{
    build_junction, JunctionComponent, JunctionError, JunctionRole, JunctionShape, Side,
};
use arcade_race_core::track::spline::TrackSpline;
use glam::Vec2;

const TRACK_WIDTH: f32 = 12.0;
const ROAD_WIDTH: f32 = 5.0;

/// Counter-clockwise stadium: 400 m straight along +x from the origin, radius-100 m left-hand ends.
fn stadium() -> TrackSpline {
    let mut pts = Vec::new();
    for i in 0..40 {
        pts.push(Vec2::new(i as f32 * 10.0, 0.0));
    }
    for i in 0..32 {
        let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 32.0;
        pts.push(Vec2::new(400.0 + 100.0 * a.cos(), 100.0 + 100.0 * a.sin()));
    }
    for i in 0..40 {
        pts.push(Vec2::new(400.0 - i as f32 * 10.0, 200.0));
    }
    for i in 0..32 {
        let a = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 32.0;
        pts.push(Vec2::new(100.0 * a.cos(), 100.0 + 100.0 * a.sin()));
    }
    TrackSpline::from_points(&pts, TRACK_WIDTH, true)
}

fn comp(s: f32, kind: JunctionShape, length: f32, divider_gap: f32) -> JunctionComponent {
    JunctionComponent { s, kind, length, divider_gap }
}

fn heading_deg(v: Vec2) -> f32 {
    v.y.atan2(v.x).to_degrees()
}

#[test]
fn taper_divergence_matches_analytic_bound() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 60.0, 2.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Right, ROAD_WIDTH).expect("taper compiles");
    let d = TRACK_WIDTH * 0.5 + 2.0 + ROAD_WIDTH * 0.5;
    let bound = (1.5 * d / 60.0).atan().to_degrees();

    // Resample the centreline every 0.5 m along x (the main straight runs along +x).
    let max_angle = j
        .centreline
        .windows(2)
        .map(|w| heading_deg(w[1] - w[0]).abs())
        .fold(0.0f32, f32::max);
    assert!((max_angle - bound).abs() <= 0.5, "max divergence {max_angle}, bound {bound}");
}

#[test]
fn taper_free_end_is_parallel_at_divider_gap() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 60.0, 2.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Right, ROAD_WIDTH).unwrap();
    assert!(heading_deg(j.free_heading).abs() < 0.5, "free end heading {}", heading_deg(j.free_heading));
    let gap = *j.edge_gaps.last().unwrap();
    assert!((gap - 2.0).abs() < 0.05, "free-end gap {gap}");
    // Right of a +x straight is -y.
    assert!(j.free_end.y < 0.0);
}

#[test]
fn turnoff_leaves_at_its_set_angle_toward_side() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::TurnOff { angle_deg: 30.0 }, 35.0, 2.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Right, ROAD_WIDTH).expect("turnoff compiles");
    let main_heading = heading_deg(main.sample_at_distance(100.0).tangent);
    let diff = heading_deg(j.free_heading) - main_heading;
    assert!((diff + 30.0).abs() <= 0.5, "heading change {diff}, want -30 (toward the right)");
    let gap = *j.edge_gaps.last().unwrap();
    assert!((gap - 2.0).abs() < 0.05, "free-end gap {gap}");
}

#[test]
fn turnoff_exit_rejoins_parallel_at_s() {
    let main = stadium();
    let c = comp(300.0, JunctionShape::TurnOff { angle_deg: 30.0 }, 35.0, 2.0);
    let j = build_junction(&main, &c, JunctionRole::Exit, Side::Right, ROAD_WIDTH).expect("exit compiles");
    let last = j.centreline.len() - 1;
    let end_heading = heading_deg(j.centreline[last] - j.centreline[last - 1]);
    assert!(end_heading.abs() < 1.0, "merge end heading {end_heading}");
    assert!((heading_deg(j.free_heading) - 30.0).abs() <= 0.5, "free end heading {}", heading_deg(j.free_heading));
    assert!((j.edge_gaps[0] - 2.0).abs() < 0.05, "free-end gap {}", j.edge_gaps[0]);
}

#[test]
fn entry_apex_is_where_the_pit_edge_leaves_the_track_edge() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 60.0, 2.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Right, ROAD_WIDTH).unwrap();
    // Analytic: the pit inner edge reaches the track edge where D * smoothstep(t) = half width + road width / 2.
    let d = TRACK_WIDTH * 0.5 + 2.0 + ROAD_WIDTH * 0.5;
    let target = (TRACK_WIDTH * 0.5 + ROAD_WIDTH * 0.5) / d;
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if mid * mid * (3.0 - 2.0 * mid) < target { lo = mid } else { hi = mid }
    }
    let expected = Vec2::new(100.0 + lo * 60.0, -TRACK_WIDTH * 0.5);
    assert!(j.apex.distance(expected) <= 0.5, "apex {:?}, expected {:?}", j.apex, expected);
    assert!(!j.quads.is_empty());
}

#[test]
fn guards_reject_bad_junctions() {
    let main = stadium();
    let build = |c: JunctionComponent, side| build_junction(&main, &c, JunctionRole::Entry, side, ROAD_WIDTH);

    assert!(matches!(
        build(comp(100.0, JunctionShape::Taper, 5.0, 2.0), Side::Right),
        Err(JunctionError::JunctionTooSteep { junction: JunctionRole::Entry, .. })
    ));
    // Inside of the radius-100 m end: the outer pit edge would pass the curve centre.
    assert!(matches!(
        build(comp(350.0, JunctionShape::Taper, 200.0, 90.0), Side::Left),
        Err(JunctionError::OffsetExceedsCurvature { .. })
    ));
    assert!(matches!(
        build(comp(100.0, JunctionShape::TurnOff { angle_deg: 50.0 }, 3.0, 2.0), Side::Right),
        Err(JunctionError::ArcTooTight { .. })
    ));
    // Arc too short: to reach the gap, the pit road would have to start outside the main road.
    assert_eq!(
        build(comp(100.0, JunctionShape::TurnOff { angle_deg: 30.0 }, 10.0, 2.0), Side::Right),
        Err(JunctionError::InvalidParameter { name: "length" })
    );
    // Arc too long: it overshoots the gap even from the main centreline.
    assert_eq!(
        build(comp(100.0, JunctionShape::TurnOff { angle_deg: 30.0 }, 60.0, 2.0), Side::Right),
        Err(JunctionError::InvalidParameter { name: "length" })
    );
    assert_eq!(
        build(comp(100.0, JunctionShape::TurnOff { angle_deg: 5.0 }, 35.0, 2.0), Side::Right),
        Err(JunctionError::InvalidParameter { name: "angle_deg" })
    );
    assert_eq!(
        build(comp(100.0, JunctionShape::Taper, f32::NAN, 2.0), Side::Right),
        Err(JunctionError::InvalidParameter { name: "length" })
    );
    assert_eq!(
        build(comp(100.0, JunctionShape::Taper, 60.0, -1.0), Side::Right),
        Err(JunctionError::InvalidParameter { name: "divider_gap" })
    );
}

#[test]
fn layout_json_uses_plain_variant_names() {
    use arcade_race_core::track::pit_kit::{PitBoxRow, PitLaneLayout};
    let layout = PitLaneLayout {
        side: Side::Right,
        entry: comp(100.0, JunctionShape::Taper, 60.0, 2.0),
        exit: comp(300.0, JunctionShape::TurnOff { angle_deg: 30.0 }, 35.0, 2.0),
        road_waypoints: vec![Vec2::new(200.0, -12.0)],
        road_width: ROAD_WIDTH,
        speed_limit: 16.67,
        box_row: PitBoxRow { start_s: 80.0, count: 6, spacing: 12.0, garages: true },
    };
    let json = serde_json::to_string(&layout).unwrap();
    assert!(json.contains("\"Taper\"") && json.contains("\"TurnOff\"") && json.contains("\"Right\""), "{json}");
    let back: PitLaneLayout = serde_json::from_str(&json).unwrap();
    assert_eq!(back, layout);
}

// ---------------------------------------------------------------------------------------------------------------
// Layout compile (Pillars III-V)
// ---------------------------------------------------------------------------------------------------------------

use arcade_race_core::track::pit_kit::{CompiledPitLane, PitBoxRow, PitKitError, PitLaneLayout};
use arcade_race_core::track::{validate_track, Track};

fn track_on(spline: TrackSpline) -> Track {
    Track { spline, ..Track::default() }
}

fn straight_layout() -> PitLaneLayout {
    PitLaneLayout {
        side: Side::Right,
        entry: comp(40.0, JunctionShape::Taper, 60.0, 2.0),
        exit: comp(360.0, JunctionShape::Taper, 60.0, 2.0),
        road_waypoints: Vec::new(),
        road_width: ROAD_WIDTH,
        speed_limit: 16.67,
        box_row: PitBoxRow { start_s: 120.0, count: 6, spacing: 12.0, garages: true },
    }
}

fn lane_distance(c: &CompiledPitLane, p: Vec2) -> f32 {
    c.lane.spline.project_point(p).distance_to_spline
}

#[test]
fn valid_layout_compiles_and_passes_track_validation() {
    let mut track = track_on(stadium());
    let c = straight_layout().compile(&track).expect("layout compiles");
    assert_eq!(c.lane.pit_boxes.len(), 6);
    assert_eq!(c.lane.road_width, ROAD_WIDTH);
    assert!(c.lane.entry_gate.start.distance(c.lane.entry_gate.end) > 1.0);
    assert!(c.lane.exit_gate.start.distance(c.lane.exit_gate.end) > 1.0);
    track.pit_lane = Some(c.lane);
    let pit_issues: Vec<_> = validate_track(&track).into_iter().filter(|e| e.code.contains("PIT")).collect();
    assert!(pit_issues.is_empty(), "{pit_issues:?}");
}

#[test]
fn junction_markings_come_from_the_components() {
    let track = track_on(stadium());
    let layout = straight_layout();
    let c = layout.compile(&track).unwrap();
    let entry = build_junction(&track.spline, &layout.entry, JunctionRole::Entry, layout.side, layout.road_width).unwrap();
    let exit = build_junction(&track.spline, &layout.exit, JunctionRole::Exit, layout.side, layout.road_width).unwrap();
    assert_eq!(c.junctions.entrance_quads, entry.quads);
    assert_eq!(c.junctions.p_apex, entry.apex);
    assert_eq!(c.junctions.chevrons, entry.chevrons);
    assert_eq!(c.junctions.exit_quads.iter().map(|q| q.quad).collect::<Vec<_>>(), exit.quads);
    assert_eq!(c.divider_start, entry.divider_end);
    assert_eq!(c.divider_end, exit.divider_end);
}

#[test]
fn pit_road_can_bend_away_from_the_track() {
    let track = track_on(stadium());
    let mut layout = straight_layout();
    layout.entry.s = 10.0;
    layout.exit.s = 390.0;
    layout.road_waypoints = vec![Vec2::new(170.0, -50.0), Vec2::new(230.0, -50.0)];
    layout.box_row = PitBoxRow { start_s: 185.0, count: 2, spacing: 12.0, garages: false };
    let c = layout.compile(&track).expect("bent road compiles");
    for &p in &layout.road_waypoints {
        assert!(lane_distance(&c, p) < 0.05, "lane misses road waypoint {p:?} by {}", lane_distance(&c, p));
    }
    // Heading jump across each junction/road joint on the compiled lane.
    for joint in [c.road_start_s, c.road_end_s] {
        let a = c.lane.spline.sample_at_distance(joint - 0.1).tangent;
        let b = c.lane.spline.sample_at_distance(joint + 0.1).tangent;
        let jump = a.perp_dot(b).atan2(a.dot(b)).abs().to_degrees();
        assert!(jump <= 2.0, "heading jump {jump} at joint s {joint}");
    }
}

#[test]
fn layout_guards_reject_bad_layouts_in_order() {
    let track = track_on(stadium());
    let base = straight_layout();
    let mut cases: Vec<PitLaneLayout> = Vec::new();

    let mut l = base.clone();
    l.entry = comp(40.0, JunctionShape::Taper, 5.0, 2.0);
    cases.push(l);
    let mut l = base.clone();
    l.side = Side::Left;
    l.entry = comp(300.0, JunctionShape::Taper, 200.0, 90.0);
    l.exit = comp(700.0, JunctionShape::Taper, 60.0, 2.0);
    cases.push(l);
    let mut l = base.clone();
    l.entry = comp(40.0, JunctionShape::TurnOff { angle_deg: 50.0 }, 3.0, 2.0);
    cases.push(l);
    let mut l = base.clone();
    l.road_waypoints = vec![
        Vec2::new(150.0, -30.0),
        Vec2::new(200.0, -30.0),
        Vec2::new(203.0, -45.0),
        Vec2::new(206.0, -30.0),
        Vec2::new(250.0, -30.0),
    ];
    l.box_row.start_s = 260.0;
    l.box_row.count = 1;
    cases.push(l);
    let mut l = base.clone();
    l.road_waypoints = vec![Vec2::new(200.0, 0.0)];
    cases.push(l);
    let mut l = base.clone();
    l.box_row.start_s = 10.0;
    cases.push(l);

    let results: Vec<_> = cases.iter().map(|l| l.compile(&track).err()).collect();
    let ok = |r: &Option<PitKitError>, f: fn(&PitKitError) -> bool| r.as_ref().is_some_and(f);
    assert!(ok(&results[0], |e| matches!(e, PitKitError::Junction(JunctionError::JunctionTooSteep { .. }))), "{:?}", results[0]);
    assert!(ok(&results[1], |e| matches!(e, PitKitError::Junction(JunctionError::OffsetExceedsCurvature { .. }))), "{:?}", results[1]);
    assert!(ok(&results[2], |e| matches!(e, PitKitError::Junction(JunctionError::ArcTooTight { .. }))), "{:?}", results[2]);
    assert!(ok(&results[3], |e| matches!(e, PitKitError::RoadTooTight { .. })), "{:?}", results[3]);
    assert!(ok(&results[4], |e| matches!(e, PitKitError::RoadOverlapsTrack { .. })), "{:?}", results[4]);
    assert!(ok(&results[5], |e| matches!(e, PitKitError::BoxRowOffRoad { box_index: 0 })), "{:?}", results[5]);
}

#[test]
fn layout_parameter_and_order_guards() {
    let track = track_on(stadium());
    let mut l = straight_layout();
    l.road_width = 3.0;
    assert_eq!(l.compile(&track).err(), Some(PitKitError::InvalidParameter { name: "road_width" }));
    let mut l = straight_layout();
    l.box_row.count = 0;
    assert_eq!(l.compile(&track).err(), Some(PitKitError::InvalidParameter { name: "count" }));
    let mut l = straight_layout();
    l.box_row.count = u32::MAX;
    assert_eq!(l.compile(&track).err(), Some(PitKitError::InvalidParameter { name: "count" }));
    let mut l = straight_layout();
    l.road_waypoints = vec![Vec2::new(f32::NAN, 0.0)];
    assert_eq!(l.compile(&track).err(), Some(PitKitError::InvalidParameter { name: "road_waypoints" }));
    // Exit inside the entry span.
    let mut l = straight_layout();
    l.exit.s = 80.0;
    assert_eq!(l.compile(&track).err(), Some(PitKitError::JunctionOrder));
}

#[test]
fn pit_lane_can_wrap_across_start_finish() {
    let track = track_on(stadium());
    let total = track.spline.total_length;
    let mut l = straight_layout();
    l.entry = comp(total - 60.0, JunctionShape::Taper, 40.0, 2.0);
    l.exit = comp(260.0, JunctionShape::Taper, 60.0, 2.0);
    // Follow the outside of the left-hand end (radius 100 m about (0, 100)), then the straight.
    let a = 265f32.to_radians();
    l.road_waypoints = vec![Vec2::new(111.0 * a.cos(), 100.0 + 111.0 * a.sin()), Vec2::new(30.0, -11.0)];
    l.box_row = PitBoxRow { start_s: 90.0, count: 6, spacing: 12.0, garages: true };
    let c = l.compile(&track).expect("wrapping layout compiles");
    assert!(l.exit.s < l.entry.s);
    let samples = &c.lane.spline.samples;
    let max_step = samples.windows(2).map(|w| w[0].point.distance(w[1].point)).fold(0.0f32, f32::max);
    assert!(max_step < 3.0, "lane has a gap of {max_step} m");
    // The lane crosses the S/F line (x = 0 on the bottom straight) beside the track.
    let crosses = samples.windows(2).any(|w| w[0].point.x < 0.0 && w[1].point.x >= 0.0 && w[1].point.y < -6.0);
    assert!(crosses, "lane does not cross the start/finish line");
}

#[test]
fn junctions_follow_a_rescaled_main_spline() {
    let base = stadium();
    let k = 1.5;
    let scaled_pts: Vec<Vec2> = base.waypoints.iter().map(|w| w.point * k).collect();
    let scaled = TrackSpline::from_points(&scaled_pts, TRACK_WIDTH, true);
    let mut l = straight_layout();
    l.road_waypoints = vec![Vec2::new(200.0, -10.5)];
    l.entry.s *= k;
    l.exit.s *= k;
    l.box_row.start_s *= k;
    for p in &mut l.road_waypoints {
        *p *= k;
    }
    for kind in [JunctionShape::Taper, JunctionShape::TurnOff { angle_deg: 30.0 }] {
        let mut l = l.clone();
        l.entry.kind = kind;
        l.exit.kind = kind;
        if let JunctionShape::TurnOff { .. } = kind {
            l.entry.length = 35.0;
            l.exit.length = 35.0;
        }
        let track = track_on(scaled.clone());
        l.compile(&track).expect("scaled layout compiles");
        for (c, role) in [(&l.entry, JunctionRole::Entry), (&l.exit, JunctionRole::Exit)] {
            let j = build_junction(&track.spline, c, role, l.side, l.road_width).unwrap();
            let gap = if role == JunctionRole::Entry { *j.edge_gaps.last().unwrap() } else { j.edge_gaps[0] };
            assert!((gap - c.divider_gap).abs() <= 0.1, "{kind:?} {role:?} free-end gap {gap}");
        }
    }
}

#[test]
fn box_row_places_stalls_and_garages() {
    let track = track_on(stadium());
    let c = straight_layout().compile(&track).unwrap();
    let lane = &c.lane;
    assert_eq!(lane.pit_boxes.len(), 6);
    let s_of: Vec<f32> = lane.pit_boxes.iter().map(|b| lane.spline.project_point(b.position).progress_distance).collect();
    for w in s_of.windows(2) {
        assert!((w[1] - w[0] - 12.0).abs() < 0.1, "stall spacing {}", w[1] - w[0]);
    }
    assert_eq!(c.garages.len(), 6);
    for (g, b) in c.garages.iter().zip(&lane.pit_boxes) {
        let main_d = |p: Vec2| track.spline.project_point(p).distance_to_spline;
        assert!(main_d(g.center) > main_d(b.position), "garage is not on the far side of its stall");
        for corner in g.corners() {
            let d = lane.spline.project_point(corner).distance_to_spline;
            assert!(d > ROAD_WIDTH * 0.5 + 1.5, "garage corner {corner:?} is {d} m from the lane centre");
        }
    }
}
