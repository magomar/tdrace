use glam::Vec2;
use tdrace_core::track::curve::{
    classify_curve_degree, compute_safe_apex_speed, evaluate_curve_approach, CurveDirection,
    TrackCurve,
};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};

#[test]
fn test_straight_track_has_no_curves() {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
        TrackWaypoint::new(Vec2::new(200.0, 0.0), 10.0),
        TrackWaypoint::new(Vec2::new(300.0, 0.0), 10.0),
    ];
    let spline = TrackSpline::new(waypoints, false);
    assert_eq!(
        spline.curves.len(),
        0,
        "A straight spline should produce zero curve detections"
    );
}

#[test]
fn test_oval_track_has_two_turns() {
    let mut waypoints = Vec::new();
    // Simple oval: straight, turn 180, straight, turn 180
    waypoints.push(TrackWaypoint::new(Vec2::new(-100.0, -50.0), 12.0));
    waypoints.push(TrackWaypoint::new(Vec2::new(100.0, -50.0), 12.0));
    // Turn 1 (right)
    waypoints.push(TrackWaypoint::new(Vec2::new(150.0, 0.0), 12.0));
    waypoints.push(TrackWaypoint::new(Vec2::new(100.0, 50.0), 12.0));
    // Back straight
    waypoints.push(TrackWaypoint::new(Vec2::new(-100.0, 50.0), 12.0));
    // Turn 2 (right)
    waypoints.push(TrackWaypoint::new(Vec2::new(-150.0, 0.0), 12.0));

    let spline = TrackSpline::new(waypoints, true);
    assert!(
        spline.curves.len() >= 2,
        "Oval track should detect at least 2 turns, got {}",
        spline.curves.len()
    );
}

#[test]
fn test_curve_degree_classification() {
    // Degree 1: High speed gentle bend
    assert_eq!(classify_curve_degree(120.0, 0.25), 1);
    assert_eq!(classify_curve_degree(95.0, 0.35), 1);

    // Degree 2: Mild curve (55m < R <= 85m)
    assert_eq!(classify_curve_degree(70.0, 0.45), 2);

    // Degree 3: Medium corner (32m < R <= 55m)
    assert_eq!(classify_curve_degree(45.0, 0.8), 3);

    // Degree 4: Sharp turn (18m < R <= 32m)
    assert_eq!(classify_curve_degree(25.0, 1.2), 4);

    // Degree 5: Hairpin (R <= 18m or large deflection)
    assert_eq!(classify_curve_degree(12.0, 2.5), 5);
    assert_eq!(classify_curve_degree(20.0, 2.2), 5); // Hairpin with ~126° angle
}

#[test]
fn test_safe_apex_speed_scales_with_radius() {
    let speed_hairpin = compute_safe_apex_speed(12.0, 0.0);
    let speed_medium = compute_safe_apex_speed(45.0, 0.0);
    let speed_sweeper = compute_safe_apex_speed(120.0, 0.0);

    assert!(
        speed_hairpin < speed_medium,
        "Hairpin speed ({}) must be less than medium speed ({})",
        speed_hairpin,
        speed_medium
    );
    assert!(
        speed_medium < speed_sweeper,
        "Medium speed ({}) must be less than sweeper speed ({})",
        speed_medium,
        speed_sweeper
    );
}

#[test]
fn test_banking_increases_safe_apex_speed() {
    let flat_speed = compute_safe_apex_speed(50.0, 0.0);
    let banked_speed = compute_safe_apex_speed(50.0, 18.0); // 18-degree banking

    assert!(
        banked_speed > flat_speed,
        "Banked curve ({}) must have higher safe speed than flat curve ({})",
        banked_speed,
        flat_speed
    );
}

#[test]
fn test_braking_urgency_calculation_and_relief() {
    let test_curve = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 4,
        entry_distance: 200.0,
        apex_distance: 235.0,
        exit_distance: 260.0,
        min_radius: 24.0,
        peak_curvature: 1.0 / 24.0,
        total_turn_angle: 1.5,
        safe_apex_speed_mps: 14.0, // ~50 km/h
        bank_angle: 0.0,
    };
    let curves = vec![test_curve];
    let total_len = 1000.0;
    let closed = true;

    // 1. Far away at high speed (50 m/s = 180 km/h): entry is 150m away (current_dist = 50m)
    // Required braking dist from 50 to 14 m/s at 7.2 m/s²: (2500 - 196) / 14.4 = 160m
    // Dist to entry is 150m < 160m => inside braking zone!
    let status_critical = evaluate_curve_approach(&curves, 50.0, total_len, closed, 50.0, 200.0);
    assert!(status_critical.is_some());
    let s_crit = status_critical.unwrap();
    assert_eq!(s_crit.curve.degree, 4);
    assert_eq!(s_crit.curve.direction, CurveDirection::Right);
    assert!(
        (s_crit.urgency - 1.0).abs() < 1e-3,
        "Should be critical urgency (1.0), got {}",
        s_crit.urgency
    );
    assert!(s_crit.must_brake);

    // 2. Far away at moderate speed (30 m/s = 108 km/h): entry is 150m away (current_dist = 50m)
    // Required braking dist: (900 - 196) / 14.4 = 48.8m
    // Dist to entry is 150m > 48.8 + 35 = 83.8m => safe!
    let status_safe = evaluate_curve_approach(&curves, 50.0, total_len, closed, 30.0, 200.0);
    assert!(status_safe.is_some());
    let s_safe = status_safe.unwrap();
    assert_eq!(
        s_safe.urgency, 0.0,
        "Should be zero urgency when well outside braking zone"
    );
    assert!(!s_safe.must_brake);

    // 3. Relief: When speed is AT OR BELOW safe apex speed (12 m/s < 14 m/s), urgency is always 0
    let status_slow = evaluate_curve_approach(&curves, 180.0, total_len, closed, 12.0, 200.0);
    assert!(status_slow.is_some());
    let s_slow = status_slow.unwrap();
    assert_eq!(
        s_slow.urgency, 0.0,
        "Cruising below safe speed should always yield 0.0 urgency"
    );
    assert_eq!(s_slow.required_braking_distance, 0.0);
    assert!(!s_slow.must_brake);
}

#[test]
fn test_classic_grand_prix_has_detected_curves() {
    let track = classic_grand_prix();
    assert!(
        !track.spline.curves.is_empty(),
        "Classic Grand Prix must have pre-detected curves"
    );

    // Verify all curves have valid degrees (1 to 5)
    for curve in &track.spline.curves {
        assert!(
            curve.degree >= 1 && curve.degree <= 5,
            "Degree must be between 1 and 5, got {}",
            curve.degree
        );
        assert!(curve.safe_apex_speed_mps > 0.0);
        assert!(curve.entry_distance < track.spline.total_length);
    }
}
