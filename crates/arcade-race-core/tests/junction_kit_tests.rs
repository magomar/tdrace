//! Spec 102 Pillar I: the apex, nose point and outer envelope of a junction component.

use arcade_race_core::track::junction_kit::{
    build_junction, JunctionComponent, JunctionRole, JunctionShape, Side, NOSE_GAP,
};
use arcade_race_core::track::spline::TrackSpline;
use glam::Vec2;

const TRACK_WIDTH: f32 = 12.0;
const ROAD_WIDTH: f32 = 8.0;

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

#[test]
fn apex_is_where_the_edge_gap_is_zero_on_the_main_edge() {
    let main = stadium();
    for kind in [JunctionShape::Taper, JunctionShape::TurnOff { angle_deg: 30.0 }] {
        let length = if matches!(kind, JunctionShape::Taper) { 30.0 } else { 50.0 };
        let c = comp(100.0, kind, length, 4.0);
        let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).expect("compiles");
        // Left of a +x straight is +y: the main edge is y = 6.
        assert!((j.apex.y - TRACK_WIDTH * 0.5).abs() < 0.2, "{kind:?}: apex {:?} is off the main edge", j.apex);
        assert!(j.apex.x > 100.0 && j.apex.x < 100.0 + length, "{kind:?}: apex {:?} is outside the junction", j.apex);
    }
}

#[test]
fn edge_apex_is_on_the_true_edge_of_the_branch() {
    let main = stadium();
    // A steep taper: the branch edge point at a sample lies ahead of the centreline point, so the edge leaves the main
    // edge later than the gap along the main normal says.
    let c = comp(100.0, JunctionShape::Taper, 20.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    assert!(j.edge_apex.x > j.apex.x, "edge apex {:?} should be past the normal-gap apex {:?}", j.edge_apex, j.apex);
    assert!((j.edge_apex.y - TRACK_WIDTH * 0.5).abs() < 0.2);
    // The branch edge facing the main road, one sample past the apex, is outside the main edge.
    let k = j.centreline.iter().position(|p| p.x > j.edge_apex.x + 0.5).unwrap();
    let t = (j.centreline[k + 1] - j.centreline[k - 1]).normalize();
    let inner = j.centreline[k] - Vec2::new(-t.y, t.x) * (ROAD_WIDTH * 0.5);
    assert!(inner.y > TRACK_WIDTH * 0.5, "inner edge {inner:?} is outside the main edge");
}

#[test]
fn nose_point_is_the_first_point_with_the_nose_gap() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 30.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    let nose = j.nose.expect("a 4 m divider gap reaches the nose gap");
    assert!(((nose.branch_edge - nose.track_edge).length() - NOSE_GAP).abs() < 0.05);
    assert!((nose.point - (nose.track_edge + nose.branch_edge) * 0.5).length() < 1e-4);
    assert!(nose.point.x > j.apex.x, "the nose lies past the apex");
    // The nose is on the branch side of the main edge, across the wedge from the main edge point.
    assert!(((nose.branch_edge - nose.track_edge).normalize() - nose.across).length() < 1e-4);
    assert!(nose.across.y > 0.5, "Left of a +x straight is +y");
}

#[test]
fn exit_nose_lies_before_the_merge_apex_in_driving_order() {
    let main = stadium();
    let c = comp(300.0, JunctionShape::Taper, 30.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Exit, Side::Left, ROAD_WIDTH).unwrap();
    let nose = j.nose.expect("nose");
    assert!(nose.point.x < j.apex.x, "driving order: nose {:?} then apex {:?}", nose.point, j.apex);
    assert!(j.apex.x < 300.0);
}

#[test]
fn a_junction_that_never_gets_wide_enough_has_no_nose() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 30.0, 1.5);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    assert!(j.nose.is_none());
}

#[test]
fn outer_envelope_follows_the_main_edge_then_the_branch_edge() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::Taper, 30.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    let env = j.outer_envelope(&main, Side::Left, ROAD_WIDTH);
    assert_eq!(env.len(), 31, "one station per metre over the 30 m span");
    assert!((env[0].s - 0.0).abs() < 1e-4 && (env[30].s - 30.0).abs() < 1e-4);

    // At the anchor the branch is centred on the main road: the main edge is the envelope.
    assert!((env[0].point().y - TRACK_WIDTH * 0.5).abs() < 0.05, "{:?}", env[0].point());
    // At the free end the branch road is parallel, with its outer edge at 6 + 4 + 8.
    let last = env[30];
    assert!((last.edge_offset() - (TRACK_WIDTH * 0.5 + 4.0 + ROAD_WIDTH)).abs() < 0.1, "{}", last.edge_offset());
    assert!((last.branch_slant - 1.0).abs() < 0.01);
    // Never narrower than the main road, never decreasing along a taper.
    for w in env.windows(2) {
        assert!(w[1].edge_offset() >= w[0].edge_offset() - 1e-3);
        assert!(w[0].edge_offset() >= TRACK_WIDTH * 0.5 - 1e-3);
    }
    // A wall 3 m outside the envelope stands 3 m outside the main edge at the anchor and the branch edge at the end.
    assert!((env[0].wall_point(3.0).y - (TRACK_WIDTH * 0.5 + 3.0)).abs() < 0.05);
    assert!((last.wall_point(3.0).y - (TRACK_WIDTH * 0.5 + 4.0 + ROAD_WIDTH + 3.0)).abs() < 0.15);
}

#[test]
fn turnoff_envelope_wall_keeps_its_gap_from_the_slanted_branch_edge() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::TurnOff { angle_deg: 40.0 }, 40.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    let env = j.outer_envelope(&main, Side::Left, ROAD_WIDTH);
    assert!(env.last().unwrap().branch_slant > 1.2, "a 40 degree branch is slanted against the main normal");
    // Distance from the wall point to the branch centreline is half the road plus the gap. Ten stations before the
    // end, so the foot of the perpendicular still lies on the sampled centreline.
    let gap = 3.0;
    let wall = env[env.len() - 11].wall_point(gap);
    let d = j.centreline.windows(2).map(|w| {
        let seg = arcade_race_core::track::geometry::LineSegment::new(w[0], w[1]);
        seg.distance_to_point(wall)
    }).fold(f32::MAX, f32::min);
    assert!((d - (ROAD_WIDTH * 0.5 + gap)).abs() < 0.3, "wall is {d} m from the branch centreline");
}

#[test]
fn envelope_normal_is_perpendicular_to_the_edge_it_stands_on() {
    let main = stadium();
    let c = comp(100.0, JunctionShape::TurnOff { angle_deg: 40.0 }, 40.0, 4.0);
    let j = build_junction(&main, &c, JunctionRole::Entry, Side::Left, ROAD_WIDTH).unwrap();
    let env = j.outer_envelope(&main, Side::Left, ROAD_WIDTH);
    // On the anchor the main road is the edge: the normal is the main normal. At the free end the branch is: the
    // normal turns back toward the main direction by the branch angle.
    assert!((env[0].normal() - env[0].outward).length() < 1e-3);
    let last = env.last().unwrap();
    let turn = last.normal().dot(last.outward).clamp(-1.0, 1.0).acos().to_degrees();
    assert!((turn - 40.0).abs() < 3.0, "normal is {turn} degrees from the main normal");
    assert!(last.normal().dot(last.tangent) < 0.0, "the normal of a diverging edge leans back");
    // A wall `gap` along the normal from an envelope point is `gap` from the branch edge.
    let e = env[env.len() - 11];
    let wall = e.point() + e.normal() * 3.0;
    let d = j
        .centreline
        .windows(2)
        .map(|w| arcade_race_core::track::geometry::LineSegment::new(w[0], w[1]).distance_to_point(wall))
        .fold(f32::MAX, f32::min);
    assert!((d - (ROAD_WIDTH * 0.5 + 3.0)).abs() < 0.3, "{d}");
}
