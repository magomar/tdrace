//! Integration tests for track rendering, runoff consistency, and wall suppression tapering (Spec 112).

use glam::Vec2;
use arcade_race_core::track::junction_kit::Side;
use arcade_race_core::track::spline::SplineSample;
use race_ui::render::track::{compute_runoff_outer_point, compute_runoff_quad_vertices};

#[test]
fn test_runoff_shoulder_tapers_where_wall_suppressed() {
    let mut s0 = SplineSample::default();
    s0.point = Vec2::new(0.0, 0.0);
    s0.tangent = Vec2::new(1.0, 0.0);
    s0.normal = Vec2::new(0.0, 1.0);
    s0.width = 10.0;
    s0.left_wall_distance = Some(4.0);
    s0.right_wall_distance = Some(4.0);
    s0.left_curb = false;
    s0.right_curb = false;

    let mut s1 = SplineSample::default();
    s1.point = Vec2::new(10.0, 0.0);
    s1.tangent = Vec2::new(1.0, 0.0);
    s1.normal = Vec2::new(0.0, 1.0);
    s1.width = 10.0;
    s1.left_wall_distance = Some(4.0);
    s1.right_wall_distance = Some(4.0);
    s1.left_curb = false;
    s1.right_curb = false;

    let p0_inner = s0.point + s0.normal * (s0.width * 0.5); // (0.0, 5.0)
    let p1_inner = s1.point + s1.normal * (s1.width * 0.5); // (10.0, 5.0)

    // 1. Normal unsuppressed span: maintains full 4 m width at both stations.
    let quad_normal = compute_runoff_quad_vertices(&s0, &s1, Side::Left, p0_inner, p1_inner, false, false)
        .expect("unsuppressed runoff quad should exist");
    assert!((quad_normal.p0_outer.distance(p0_inner) - 4.0).abs() < 1e-4);
    assert!((quad_normal.p1_outer.distance(p1_inner) - 4.0).abs() < 1e-4);

    // 2. Boundary where wall is suppressed at station 1: tapers from 4 m down to 0 m width at s1.
    let quad_tapered_end = compute_runoff_quad_vertices(&s0, &s1, Side::Left, p0_inner, p1_inner, false, true)
        .expect("tapered runoff quad should exist");
    assert!((quad_tapered_end.p0_outer.distance(p0_inner) - 4.0).abs() < 1e-4);
    assert_eq!(quad_tapered_end.p1_outer, p1_inner);
    assert_eq!(quad_tapered_end.p1_outer.distance(p1_inner), 0.0);

    // 3. Boundary where wall is suppressed at station 0: tapers from 0 m up to 4 m width at s1.
    let quad_tapered_start = compute_runoff_quad_vertices(&s0, &s1, Side::Left, p0_inner, p1_inner, true, false)
        .expect("tapered runoff quad should exist");
    assert_eq!(quad_tapered_start.p0_outer, p0_inner);
    assert_eq!(quad_tapered_start.p0_outer.distance(p0_inner), 0.0);
    assert!((quad_tapered_start.p1_outer.distance(p1_inner) - 4.0).abs() < 1e-4);

    // 4. Suppressed zone where walls at both stations are suppressed: zero geometry (no phantom corridor).
    let quad_suppressed = compute_runoff_quad_vertices(&s0, &s1, Side::Left, p0_inner, p1_inner, true, true);
    assert!(quad_suppressed.is_none(), "fully suppressed runoff corridor must return None");

    // 5. Test compute_runoff_outer_point directly with and without curb
    let p_unsuppressed = compute_runoff_outer_point(&s0, Side::Left, false);
    assert_eq!(p_unsuppressed, Vec2::new(0.0, 9.0)); // 5.0 + 4.0 = 9.0

    let p_suppressed = compute_runoff_outer_point(&s0, Side::Left, true);
    assert_eq!(p_suppressed, Vec2::new(0.0, 5.0)); // 5.0 + 0.0 = 5.0 (0 m runoff)

    s0.left_curb = true;
    let p_suppressed_with_curb = compute_runoff_outer_point(&s0, Side::Left, true);
    assert_eq!(p_suppressed_with_curb, Vec2::new(0.0, 5.0 + 1.35)); // 0 m extra runoff past curb
}
