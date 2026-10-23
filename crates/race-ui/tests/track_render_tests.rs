//! Integration tests for track rendering, runoff consistency, and wall suppression tapering (Spec 112).

use glam::Vec2;
use arcade_race_core::track::geometry::LineSegment;
use arcade_race_core::track::junction_kit::Side;
use arcade_race_core::track::spline::SplineSample;
use race_ui::render::track::{
    clamp_runoff_outer_to_walls, compute_runoff_outer_point,
    compute_runoff_outer_point_with_fallback, compute_runoff_quad_vertices,
    compute_runoff_quad_vertices_with_fallback,
};

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

#[test]
fn test_runoff_quad_follows_widened_pit_side_bay_boundary() {
    let mut s0 = SplineSample::default();
    s0.point = Vec2::new(0.0, 0.0);
    s0.tangent = Vec2::new(1.0, 0.0);
    s0.normal = Vec2::new(0.0, 1.0);
    s0.width = 7.0;
    s0.right_wall_distance = Some(1.5);

    let mut s1 = s0.clone();
    s1.point = Vec2::new(10.0, 0.0);

    // Side bay bulges outward on the right (-normal) by 3.6 m beyond hw = 3.5 m -> y = -7.1 m
    let p0_inner = Vec2::new(0.0, -7.1);
    let p1_inner = Vec2::new(10.0, -7.1);

    let quad = compute_runoff_quad_vertices(&s0, &s1, Side::Right, p0_inner, p1_inner, false, false)
        .expect("pit side-bay runoff quad should exist");
    assert!((quad.p0_outer.y - (-8.6)).abs() < 1e-4, "expected outer edge at -8.6m, got {:?}", quad.p0_outer);
    assert!((quad.p1_outer.y - (-8.6)).abs() < 1e-4, "expected outer edge at -8.6m, got {:?}", quad.p1_outer);
}

#[test]
fn test_runoff_clamped_against_physical_walls_and_respects_fallback_distance() {
    let mut s0 = SplineSample::default();
    s0.point = Vec2::new(0.0, 0.0);
    s0.tangent = Vec2::new(1.0, 0.0);
    s0.normal = Vec2::new(0.0, 1.0);
    s0.width = 10.0;
    // None for wall distances, simulating branch segments or standard tracks
    s0.left_wall_distance = None;
    s0.right_wall_distance = None;

    let mut s1 = s0.clone();
    s1.point = Vec2::new(10.0, 0.0);

    // 1. With 3.5m fallback barrier offset, outer point is at 5.0 + 3.5 = 8.5m instead of 5.0 + 8.0 = 13.0m
    let p_fallback = compute_runoff_outer_point_with_fallback(&s0, Side::Left, false, 3.5);
    assert!((p_fallback.y - 8.5).abs() < 1e-4);

    let p0_inner = Vec2::new(0.0, 5.0);
    let p1_inner = Vec2::new(10.0, 5.0);
    let quad = compute_runoff_quad_vertices_with_fallback(
        &s0, &s1, Side::Left, p0_inner, p1_inner, false, false, 3.5,
    ).expect("quad should exist");
    assert!((quad.p0_outer.y - 8.5).abs() < 1e-4);
    assert!((quad.p1_outer.y - 8.5).abs() < 1e-4);

    // 2. Physical wall positioned at y = 7.0 (between inner at 5.0 and unconstrained outer at 8.5)
    // Runoff ray from (0.0, 5.0) to (0.0, 8.5) must be clamped precisely to the wall at y = 7.0
    let walls = [LineSegment::new(Vec2::new(-5.0, 7.0), Vec2::new(15.0, 7.0))];
    let clamped_p0 = clamp_runoff_outer_to_walls(quad.p0_inner, quad.p0_outer, &walls);
    let clamped_p1 = clamp_runoff_outer_to_walls(quad.p1_inner, quad.p1_outer, &walls);
    assert!((clamped_p0.y - 7.0).abs() < 1e-4, "runoff must terminate at the wall, got {:?}", clamped_p0);
    assert!((clamped_p1.y - 7.0).abs() < 1e-4, "runoff must terminate at the wall, got {:?}", clamped_p1);

    // 3. Wall behind outer point at y = 10.0 does not clamp outer point
    let distant_walls = [LineSegment::new(Vec2::new(-5.0, 10.0), Vec2::new(15.0, 10.0))];
    let unclamped_p0 = clamp_runoff_outer_to_walls(quad.p0_inner, quad.p0_outer, &distant_walls);
    assert!((unclamped_p0.y - 8.5).abs() < 1e-4);
}


