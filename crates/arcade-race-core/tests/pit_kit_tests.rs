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
