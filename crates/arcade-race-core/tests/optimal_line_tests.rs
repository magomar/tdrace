use arcade_race_core::track::presets::{create_prototypical_track, RaceDirection, TrackShape};
use arcade_race_core::track::{OptimalLineConfig, OptimalRacingLine};

#[test]
fn test_optimal_line_solver_convergence_and_boundaries() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let config = OptimalLineConfig::default();

    let optimal_line = OptimalRacingLine::compute_for_spline_with_config(&track.spline, &config);

    assert!(!optimal_line.nodes.is_empty(), "Optimal line should produce nodes");
    assert_eq!(optimal_line.nodes.len(), track.spline.samples.len());

    // Verify all nodes remain strictly within the drivable track bounds
    for (i, node) in optimal_line.nodes.iter().enumerate() {
        let sample = &track.spline.samples[i];
        let max_half_width = sample.width * 0.5 + config.curb_extension_m;

        assert!(
            node.lateral_offset.abs() <= max_half_width,
            "Node {} lateral offset {:.2} exceeds half width {:.2}",
            i,
            node.lateral_offset,
            max_half_width
        );

        // Verify world point is exactly centerline + normal * lateral_offset
        let expected_point = sample.point + sample.normal * node.lateral_offset;
        assert!(
            (node.point - expected_point).length() < 1e-3,
            "Node {} world position mismatch",
            i
        );
    }
}

#[test]
fn test_optimal_line_curvature_reduction() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let optimal_line = OptimalRacingLine::compute_for_spline(&track.spline);

    assert!(optimal_line.total_length > 50.0);

    // Integrated squared curvature of the optimal line should be finite and smooth
    let max_offset = optimal_line.nodes.iter().map(|n| n.lateral_offset).fold(f32::NEG_INFINITY, f32::max);
    let min_offset = optimal_line.nodes.iter().map(|n| n.lateral_offset).fold(f32::INFINITY, f32::min);

    // Verify significant lateral excursion: outside entry/exit (< -4.5m) and inside apex (> +4.5m)
    assert!(max_offset > 4.5, "Optimal line must hug inside apex curb (max offset: {:.2}m)", max_offset);
    assert!(min_offset < -4.5, "Optimal line must take outside entry/exit (min offset: {:.2}m)", min_offset);

    // Verify specific apex clipping for Curve 1 (apex at d=209.6m) and Curve 2 (apex at d=574.4m)
    let apex1_node = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 209.6).abs().partial_cmp(&(b.spline_distance - 209.6).abs()).unwrap())
        .unwrap();
    assert!(apex1_node.lateral_offset > 5.0, "Curve 1 apex should clip inside (offset={:.2}m)", apex1_node.lateral_offset);

    let apex2_node = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 574.4).abs().partial_cmp(&(b.spline_distance - 574.4).abs()).unwrap())
        .unwrap();
    assert!(apex2_node.lateral_offset > 5.0, "Curve 2 apex should clip inside (offset={:.2}m)", apex2_node.lateral_offset);

    // Verify entry and exit are on the outside of the turn (< -4.5m)
    let entry1_node = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 138.8).abs().partial_cmp(&(b.spline_distance - 138.8).abs()).unwrap())
        .unwrap();
    assert!(entry1_node.lateral_offset < -4.5, "Curve 1 entry should be wide on outside (offset={:.2}m)", entry1_node.lateral_offset);

    let exit1_node = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 283.0).abs().partial_cmp(&(b.spline_distance - 283.0).abs()).unwrap())
        .unwrap();
    assert!(exit1_node.lateral_offset < -4.5, "Curve 1 exit should be wide on outside (offset={:.2}m)", exit1_node.lateral_offset);
}

#[test]
fn test_optimal_line_velocity_profiling_and_braking_evaluation() {
    let track = create_prototypical_track("gt", TrackShape::Oval, RaceDirection::Right);
    let optimal_line = OptimalRacingLine::compute_for_spline(&track.spline);

    // Target speeds should be within physical limits [7.0, 75.0]
    for (i, node) in optimal_line.nodes.iter().enumerate() {
        assert!(
            node.target_speed >= 7.0 && node.target_speed <= 75.0,
            "Node {} speed {:.1} outside bounds [7, 75]",
            i,
            node.target_speed
        );
    }

    // Find a corner node with lowest target speed
    let (_min_idx, min_node) = optimal_line
        .nodes
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.target_speed.partial_cmp(&b.1.target_speed).unwrap())
        .unwrap();

    let apex_speed = min_node.target_speed;
    let apex_dist = min_node.spline_distance;

    // Test 1: Approaching the corner at high speed (e.g. 50 m/s when corner target is 15 m/s) from 60m back
    let approach_dist = (apex_dist + track.spline.total_length() - 50.0) % track.spline.total_length();
    let high_speed = 50.0;
    let eval_high = optimal_line.evaluate_braking(approach_dist, high_speed);

    assert!(eval_high.urgency > 0.0, "High speed approach should have positive braking urgency");
    assert!(eval_high.required_braking_dist > 0.0);

    // Test 2: Approaching the same corner already at or below safe speed (e.g. 10 m/s)
    let slow_speed = apex_speed.min(10.0);
    let eval_slow = optimal_line.evaluate_braking(approach_dist, slow_speed);

    assert_eq!(eval_slow.urgency, 0.0, "Slow speed approach should require 0 braking urgency");
    assert!(!eval_slow.must_brake);
}

#[test]
fn test_optimal_line_figure_eight_apex_clipping() {
    let track = create_prototypical_track("gt", TrackShape::HorizontalEight, RaceDirection::Right);
    let optimal_line = OptimalRacingLine::compute_for_spline(&track.spline);

    assert_eq!(optimal_line.nodes.len(), track.spline.samples.len());

    // In HorizontalEight, there are both Left and Right turns.
    // Verify that apex offsets clip to the inside for both turn directions:
    // Curve 2 (Right hairpin, apex at d=292.8m) should have large negative offset (< -5.0m).
    let right_hairpin = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 292.8).abs().partial_cmp(&(b.spline_distance - 292.8).abs()).unwrap())
        .unwrap();
    assert!(right_hairpin.lateral_offset < -5.0, "Right hairpin apex should clip inside right (offset={:.2}m)", right_hairpin.lateral_offset);

    // Curve 5 (Left hairpin, apex at d=695.9m) should have large positive offset (> 5.0m).
    let left_hairpin = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 695.9).abs().partial_cmp(&(b.spline_distance - 695.9).abs()).unwrap())
        .unwrap();
    assert!(left_hairpin.lateral_offset > 5.0, "Left hairpin apex should clip inside left (offset={:.2}m)", left_hairpin.lateral_offset);

    // Curve 0 (Left chicane turn, apex at d=39.6m) should have positive offset (> 0.5m).
    let left_turn = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 39.6).abs().partial_cmp(&(b.spline_distance - 39.6).abs()).unwrap())
        .unwrap();
    assert!(left_turn.lateral_offset > 0.5, "Left chicane apex should clip inside left (offset={:.2}m)", left_turn.lateral_offset);

    // Curve 1 (Right chicane turn, apex at d=171.6m) should have negative offset (< -0.5m).
    let right_turn = optimal_line.nodes.iter()
        .min_by(|a, b| (a.spline_distance - 171.6).abs().partial_cmp(&(b.spline_distance - 171.6).abs()).unwrap())
        .unwrap();
    assert!(right_turn.lateral_offset < -0.5, "Right chicane apex should clip inside right (offset={:.2}m)", right_turn.lateral_offset);
}
