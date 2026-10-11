//! Integration tests for optimal racing line rendering primitives (Spec 110).

use glam::Vec2;
use arcade_race_core::track::{
    optimal_line::{OptimalLineNode, OptimalRacingLine, RacingLineMode},
    spline::TrackSpline,
};
use race_ui::render::optimal_line::{compute_racing_line_color, render_optimal_racing_line};

#[test]
fn test_optimal_line_render_color_transitions() {
    let target = 25.0; // 90 km/h
    let opacity = 0.70;

    // Full mode: Below target speed -> Green
    let c_green = compute_racing_line_color(RacingLineMode::Full, 20.0, target, opacity);
    assert_eq!(c_green.a, opacity);
    assert!(c_green.g > 0.85);
    assert!(c_green.r < 0.25);

    // Full mode: At safe speed threshold -> Electric Yellow
    let c_yellow = compute_racing_line_color(RacingLineMode::Full, 28.5, target, opacity);
    assert_eq!(c_yellow.a, opacity);
    assert!(c_yellow.r > 0.90);
    assert!(c_yellow.g > 0.80);

    // Full mode: Fast approach -> Bright Crimson Red
    let c_red = compute_racing_line_color(RacingLineMode::Full, 50.0, target, opacity);
    assert_eq!(c_red.a, opacity);
    assert!(c_red.r > 0.90);
    assert!(c_red.g < 0.30);

    // LineOnly mode: Constant color irrespective of speed
    let c_lo_slow = compute_racing_line_color(RacingLineMode::LineOnly, 10.0, target, opacity);
    let c_lo_fast = compute_racing_line_color(RacingLineMode::LineOnly, 60.0, target, opacity);
    assert_eq!(c_lo_slow.r, c_lo_fast.r);
    assert_eq!(c_lo_slow.g, c_lo_fast.g);
    assert_eq!(c_lo_slow.b, c_lo_fast.b);
    assert_eq!(c_lo_slow.a, opacity);

    // BrakingOnly mode: Transparent when within safe speed, Crimson when braking required
    let c_bo_slow = compute_racing_line_color(RacingLineMode::BrakingOnly, 20.0, target, opacity);
    assert_eq!(c_bo_slow.a, 0.0);

    let c_bo_fast = compute_racing_line_color(RacingLineMode::BrakingOnly, 50.0, target, opacity);
    assert!(c_bo_fast.a > 0.65);
    assert!(c_bo_fast.r > 0.90);
}

#[test]
fn test_optimal_line_render_modes_and_culling() {
    let mut spline = TrackSpline::default();
    spline.closed = true;

    let mut nodes = Vec::new();
    for i in 0..60 {
        let angle = (i as f32 / 60.0) * std::f32::consts::TAU;
        let pt = Vec2::new(angle.cos() * 100.0, angle.sin() * 100.0);
        let tangent = Vec2::new(-angle.sin(), angle.cos());
        let dist = i as f32 * 10.47;
        nodes.push(OptimalLineNode {
            point: pt,
            tangent,
            lateral_offset: 0.0,
            distance: dist,
            spline_distance: dist,
            curvature: 0.01,
            target_speed: 25.0,
        });
    }
    let optimal_line = OptimalRacingLine {
        nodes,
        total_length: 628.3,
        integrated_curvature: 0.0628,
    };

    // Disabled mode: Executes cleanly with zero operations
    render_optimal_racing_line(
        &spline,
        &optimal_line,
        Vec2::new(100.0, 0.0),
        35.0,
        RacingLineMode::Disabled,
        0.70,
        None,
    );

    // Full mode: Culled viewport
    render_optimal_racing_line(
        &spline,
        &optimal_line,
        Vec2::new(100.0, 0.0),
        35.0,
        RacingLineMode::Full,
        0.70,
        Some((Vec2::new(80.0, -20.0), Vec2::new(120.0, 20.0))),
    );

    // LineOnly mode: Open viewport
    render_optimal_racing_line(
        &spline,
        &optimal_line,
        Vec2::new(100.0, 0.0),
        20.0,
        RacingLineMode::LineOnly,
        0.50,
        None,
    );

    // BrakingOnly mode
    render_optimal_racing_line(
        &spline,
        &optimal_line,
        Vec2::new(100.0, 0.0),
        45.0,
        RacingLineMode::BrakingOnly,
        0.80,
        None,
    );
}
