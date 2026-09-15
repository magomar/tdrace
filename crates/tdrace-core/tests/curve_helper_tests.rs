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

#[test]
fn test_apex_inflexion_traversal_and_clearance() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 3,
        entry_distance: 200.0,
        apex_distance: 230.0,
        exit_distance: 260.0,
        min_radius: 40.0,
        peak_curvature: 1.0 / 40.0,
        total_turn_angle: 1.0,
        safe_apex_speed_mps: 18.0,
        bank_angle: 0.0,
    };
    let curve2 = TrackCurve {
        id: 1,
        direction: CurveDirection::Left,
        degree: 4,
        entry_distance: 320.0,
        apex_distance: 350.0,
        exit_distance: 380.0,
        min_radius: 25.0,
        peak_curvature: 1.0 / 25.0,
        total_turn_angle: 1.2,
        safe_apex_speed_mps: 14.0,
        bank_angle: 0.0,
    };
    let curves = vec![curve1, curve2];
    let total_len = 1000.0;
    let closed = true;

    // 1. Inside curve1 before apex (current_dist = 220.0, apex = 230.0): distance_to_apex is +10m
    let status_approaching_apex = evaluate_curve_approach(&curves, 220.0, total_len, closed, 25.0, 150.0);
    assert!(status_approaching_apex.is_some());
    let s_appr = status_approaching_apex.unwrap();
    assert_eq!(s_appr.curve.id, 0);
    assert!(s_appr.is_inside_curve);
    assert!((s_appr.distance_to_apex - 10.0).abs() < 1e-3);
    assert!(s_appr.must_brake, "Overspeeding before apex must alert");

    // 2. Traversed past apex by 5m (current_dist = 235.0, apex = 230.0): distance_to_apex is -5m
    let status_past_apex = evaluate_curve_approach(&curves, 235.0, total_len, closed, 25.0, 150.0);
    assert!(status_past_apex.is_some());
    let s_past = status_past_apex.unwrap();
    assert_eq!(s_past.curve.id, 0);
    assert!(s_past.is_inside_curve);
    assert!((s_past.distance_to_apex - (-5.0)).abs() < 1e-3);
    assert_eq!(s_past.urgency, 0.0, "Urgency resets to 0 past apex");
    assert!(!s_past.must_brake, "Must not brake once accelerating past apex");

    // 3. Traversed past apex by > 10m (current_dist = 245.0, apex = 230.0):
    // Curve 0 is skipped and target advances to Curve 1 (entry at 320m, 75m ahead)
    let status_cleared = evaluate_curve_approach(&curves, 245.0, total_len, closed, 25.0, 150.0);
    assert!(status_cleared.is_some());
    let s_next = status_cleared.unwrap();
    assert_eq!(s_next.curve.id, 1, "Should target next curve after traversing past apex");
    assert!(!s_next.is_inside_curve);
    assert!((s_next.distance_to_entry - 75.0).abs() < 1e-3);
}

#[test]
fn test_chained_turn_red_preemption_when_harder() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 2,
        entry_distance: 100.0,
        apex_distance: 130.0,
        exit_distance: 160.0,
        min_radius: 65.0,
        peak_curvature: 1.0 / 65.0,
        total_turn_angle: 0.6,
        safe_apex_speed_mps: 40.0,
        bank_angle: 0.0,
    };
    let curve2 = TrackCurve {
        id: 1,
        direction: CurveDirection::Left,
        degree: 4,
        entry_distance: 180.0,
        apex_distance: 205.0,
        exit_distance: 230.0,
        min_radius: 24.0,
        peak_curvature: 1.0 / 24.0,
        total_turn_angle: 1.5,
        safe_apex_speed_mps: 15.0,
        bank_angle: 0.0,
    };
    let curves = vec![curve1, curve2];
    let total_len = 1000.0;
    let closed = true;

    let car_speed = 35.0; // 126 km/h
    // Required braking for curve2: (35^2 - 15^2) / 14.4 = (1225 - 225) / 14.4 = 69.44m
    // Warning buffer: 35m. Red zone begins when dist_to_entry <= 69.44 + 0.3 * 35 = ~80m.

    // 1. Far away (current_dist = 0.0m):
    // Curve 1 entry is 100m away, Curve 2 entry is 180m away.
    // Curve 2 is well outside braking zone (180m > 69.44 + 35 = 104.44m) -> urgency 0.0.
    // Player first sees arrows for the first turn!
    let s_far = evaluate_curve_approach(&curves, 0.0, total_len, closed, car_speed, 200.0).unwrap();
    assert_eq!(s_far.curve.id, 0, "Far away player must see Turn 1 first");
    assert_eq!(s_far.curve.direction, CurveDirection::Right);
    assert_eq!(s_far.curve.degree, 2);

    // 2. Approaching/entering Turn 1 before apex (current_dist = 110.0m, apex = 130.0m):
    // Car is inside Turn 1, 20m before Turn 1 apex!
    // Curve 2 entry is 180.0 - 110.0 = 70.0m away.
    // Curve 2 is inside the red braking zone (70m <= 69.44 + 10.5m = 79.94m)!
    // Turn 2 is harder (degree 4 > degree 2) and deserves red state!
    // Turn 2 immediately preempts Turn 1 before Turn 1's apex!
    let s_preempt = evaluate_curve_approach(&curves, 110.0, total_len, closed, car_speed, 200.0).unwrap();
    assert_eq!(
        s_preempt.curve.id, 1,
        "Turn 2 must preempt Turn 1 before Turn 1 apex when Turn 2 deserves red and is harder"
    );
    assert_eq!(s_preempt.curve.direction, CurveDirection::Left);
    assert_eq!(s_preempt.curve.degree, 4);
    assert!(s_preempt.urgency >= 0.70, "Preempting Turn 2 must be in red state");
}

#[test]
fn test_chained_turn_yellow_preemption_when_much_harder() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 1, // Gentle kink
        entry_distance: 100.0,
        apex_distance: 130.0,
        exit_distance: 160.0,
        min_radius: 120.0,
        peak_curvature: 1.0 / 120.0,
        total_turn_angle: 0.25,
        safe_apex_speed_mps: 65.0,
        bank_angle: 0.0,
    };
    let curve2 = TrackCurve {
        id: 1,
        direction: CurveDirection::Left,
        degree: 5, // Hairpin (+4 degrees harder!)
        entry_distance: 180.0,
        apex_distance: 205.0,
        exit_distance: 230.0,
        min_radius: 14.0,
        peak_curvature: 1.0 / 14.0,
        total_turn_angle: 2.5,
        safe_apex_speed_mps: 12.0,
        bank_angle: 0.0,
    };
    let curves = vec![curve1, curve2];
    let total_len = 1000.0;
    let closed = true;

    let car_speed = 40.0; // 144 km/h
    // Required braking for curve2: (1600 - 144) / 14.4 = 101.1m
    // Yellow zone begins when dist_to_entry <= 101.1 + 35 = 136.1m (urgency >= 0.35 when dist <= 124m).

    // 1. Far away (current_dist = 0.0m):
    // Dist to curve2 entry is 180m > 136.1m -> urgency 0.0.
    // Shows Turn 1 first.
    let s_far = evaluate_curve_approach(&curves, 0.0, total_len, closed, car_speed, 200.0).unwrap();
    assert_eq!(s_far.curve.id, 0);
    assert_eq!(s_far.curve.degree, 1);

    // 2. Approaching Turn 1 (current_dist = 70.0m, 30m before Turn 1 entry):
    // Dist to curve2 entry is 180.0 - 70.0 = 110.0m.
    // Curve 2 urgency: (101.1 + 35 - 110) / 35 = 26.1 / 35 = 0.74 (entering red)!
    // Curve 2 is much harder (+4 degrees) and yellow/red -> immediately shows Turn 2!
    let s_mid = evaluate_curve_approach(&curves, 70.0, total_len, closed, car_speed, 200.0).unwrap();
    assert_eq!(s_mid.curve.id, 1, "Much harder Turn 2 preempts Turn 1 when yellow/red");
    assert_eq!(s_mid.curve.degree, 5);
}

#[test]
fn test_chained_turn_not_harder_does_not_preempt_turn1() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 4, // Sharp turn
        entry_distance: 100.0,
        apex_distance: 130.0,
        exit_distance: 160.0,
        min_radius: 25.0,
        peak_curvature: 1.0 / 25.0,
        total_turn_angle: 1.2,
        safe_apex_speed_mps: 15.0,
        bank_angle: 0.0,
    };
    let curve2 = TrackCurve {
        id: 1,
        direction: CurveDirection::Left,
        degree: 2, // Mild curve (easier!)
        entry_distance: 180.0,
        apex_distance: 205.0,
        exit_distance: 230.0,
        min_radius: 70.0,
        peak_curvature: 1.0 / 70.0,
        total_turn_angle: 0.5,
        safe_apex_speed_mps: 45.0,
        bank_angle: 0.0,
    };
    let curves = vec![curve1, curve2];
    let total_len = 1000.0;
    let closed = true;

    // Inside curve1 approaching apex (current_dist = 115.0m, apex = 130.0m)
    let s = evaluate_curve_approach(&curves, 115.0, total_len, closed, 25.0, 200.0).unwrap();
    assert_eq!(
        s.curve.id, 0,
        "Turn 1 must NOT be preempted when upcoming Turn 2 is easier than Turn 1"
    );
}

#[test]
fn test_equal_severity_turns_do_not_preempt_before_apex() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 4,
        entry_distance: 100.0,
        apex_distance: 130.0,
        exit_distance: 160.0,
        min_radius: 25.0,
        peak_curvature: 1.0 / 25.0,
        total_turn_angle: 1.2,
        safe_apex_speed_mps: 15.0,
        bank_angle: 0.0,
    };
    let curve2 = TrackCurve {
        id: 1,
        direction: CurveDirection::Left,
        degree: 4, // Equal severity
        entry_distance: 180.0,
        apex_distance: 205.0,
        exit_distance: 230.0,
        min_radius: 25.0,
        peak_curvature: 1.0 / 25.0,
        total_turn_angle: 1.2,
        safe_apex_speed_mps: 15.0,
        bank_angle: 0.0,
    };
    let curves = vec![curve1, curve2];
    let total_len = 1000.0;
    let closed = true;

    // Inside curve1 approaching apex (current_dist = 115.0m, apex = 130.0m)
    let s = evaluate_curve_approach(&curves, 115.0, total_len, closed, 25.0, 200.0).unwrap();
    assert_eq!(
        s.curve.id, 0,
        "Equal severity turn must retain priority until navigated"
    );
}


