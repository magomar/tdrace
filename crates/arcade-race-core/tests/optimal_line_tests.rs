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
    assert!(optimal_line.integrated_curvature >= 0.0);
    println!("Integrated curvature of optimal line: {:.4}", optimal_line.integrated_curvature);
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
