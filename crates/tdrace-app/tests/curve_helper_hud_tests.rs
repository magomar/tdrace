use tdrace_app::ui::curve_indicator::{
    compute_curve_arrow_position, compute_curve_colors, compute_indicator_alpha,
    compute_smart_curve_arrow_position, CurveColorScheme,
};
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::curve::{evaluate_curve_approach, CurveDirection, TrackCurve};
use tdrace_core::track::presets::classic_grand_prix;

#[test]
fn test_color_scheme_cycling() {
    let s0 = CurveColorScheme::Traffic;
    let s1 = s0.next();
    assert_eq!(s1, CurveColorScheme::Synthwave);
    let s2 = s1.next();
    assert_eq!(s2, CurveColorScheme::Contrast);
    let s3 = s2.next();
    assert_eq!(s3, CurveColorScheme::Rally);
    let s4 = s3.next();
    assert_eq!(s4, CurveColorScheme::Traffic);
}

#[test]
fn test_compute_curve_colors_traffic_gradient() {
    // 1. Safe / Cruise (urgency = 0.0) -> Green
    let (col_safe, _) = compute_curve_colors(CurveColorScheme::Traffic, 0.0, 3, 1.0);
    assert!(col_safe.g > col_safe.r, "Safe color should have higher green ({}) than red ({})", col_safe.g, col_safe.r);
    assert!(col_safe.g > 0.8, "Safe green should be vibrant");

    // 2. Prepare (urgency = 0.5) -> Yellow/Amber
    let (col_warn, _) = compute_curve_colors(CurveColorScheme::Traffic, 0.5, 3, 1.0);
    assert!(col_warn.r > 0.8, "Warning color should have high red component");
    assert!(col_warn.g > 0.6, "Warning color should have noticeable green component for yellow/amber");

    // 3. Critical (urgency = 1.0) -> Red
    let (col_crit, _) = compute_curve_colors(CurveColorScheme::Traffic, 1.0, 3, 1.0);
    assert!(col_crit.r > 0.9, "Critical color must be bright red");
    assert!(col_crit.g < 0.3, "Critical color must have low green component");
}

#[test]
fn test_compute_curve_colors_synthwave_gradient() {
    // Synthwave starts with Cyber Cyan (high g & b, lower r)
    let (col_cyan, _) = compute_curve_colors(CurveColorScheme::Synthwave, 0.0, 3, 1.0);
    assert!(col_cyan.b > 0.8 && col_cyan.g > 0.8, "Synthwave cruise color should be cyan");

    // Synthwave critical turns to Laser Red
    let (col_red, _) = compute_curve_colors(CurveColorScheme::Synthwave, 1.0, 3, 1.0);
    assert!(col_red.r > 0.9 && col_red.g < 0.25, "Synthwave critical color should be laser red");
}

#[test]
fn test_compute_curve_colors_contrast_gradient() {
    // High contrast starts with Ice White (high r, g, b)
    let (col_white, _) = compute_curve_colors(CurveColorScheme::Contrast, 0.0, 3, 1.0);
    assert!(col_white.r > 0.8 && col_white.g > 0.8 && col_white.b > 0.8, "Contrast cruise color should be ice white");

    // High contrast critical turns to pure red
    let (col_red, _) = compute_curve_colors(CurveColorScheme::Contrast, 1.0, 3, 1.0);
    assert!(col_red.r > 0.9 && col_red.g < 0.25, "Contrast critical color should be red");
}

#[test]
fn test_compute_curve_colors_rally_pacenote_schema() {
    // Safe mode is degree-coded
    let (col_deg1, _) = compute_curve_colors(CurveColorScheme::Rally, 0.0, 1, 1.0);
    let (col_deg3, _) = compute_curve_colors(CurveColorScheme::Rally, 0.0, 3, 1.0);
    assert_ne!(col_deg1, col_deg3, "Rally pacenotes must use distinct colors for different degrees");

    // Critical brake mode flashes red regardless of degree
    let (col_deg1_crit, _) = compute_curve_colors(CurveColorScheme::Rally, 0.95, 1, 1.0);
    assert!(col_deg1_crit.r > 0.9 && col_deg1_crit.g < 0.3, "Rally critical brake must be red");
}

#[test]
fn test_classic_grand_prix_curve_evaluation_at_speed() {
    let track = classic_grand_prix();
    assert!(!track.spline.curves.is_empty(), "Track must contain detected curves");

    // Player approaching first corner on main straight (say progress_dist = 50m) at 55 m/s (198 km/h)
    let lookahead = 200.0;
    let status_fast = track.spline.upcoming_curve(50.0, 55.0, lookahead);
    assert!(status_fast.is_some(), "Should detect upcoming curve from main straight");

    let s = status_fast.unwrap();
    assert!(s.curve.degree >= 1 && s.curve.degree <= 5);
    assert!(s.required_braking_distance > 0.0, "High speed approach requires braking");

    // Player driving very slowly (5 m/s = 18 km/h)
    let status_slow = track.spline.upcoming_curve(50.0, 5.0, lookahead);
    assert!(status_slow.is_some());
    let s_slow = status_slow.unwrap();
    assert_eq!(s_slow.urgency, 0.0, "Slow approach requires zero braking urgency");
    assert_eq!(s_slow.required_braking_distance, 0.0);
}

#[test]
fn test_indicator_alpha_quick_fade_past_apex() {
    // 1. Far approach: 150m is 0.0, 140m is 0.5, 130m is 1.0
    let a_150 = compute_indicator_alpha(150.0, 180.0, false);
    assert!((a_150 - 0.0).abs() < 1e-3);

    let a_140 = compute_indicator_alpha(140.0, 170.0, false);
    assert!((a_140 - 0.5).abs() < 1e-3);

    let a_100 = compute_indicator_alpha(100.0, 130.0, false);
    assert!((a_100 - 1.0).abs() < 1e-3);

    // 2. Inside curve approaching apex: alpha is full 1.0
    let a_in_turn = compute_indicator_alpha(-10.0, 15.0, true);
    assert!((a_in_turn - 1.0).abs() < 1e-3);

    // 3. At apex: alpha is full 1.0
    let a_at_apex = compute_indicator_alpha(-25.0, 0.0, true);
    assert!((a_at_apex - 1.0).abs() < 1e-3);

    // 4. Past apex by 3m: fades quickly (1.0 - 0.3 = 0.7)
    let a_past_3m = compute_indicator_alpha(-28.0, -3.0, true);
    assert!((a_past_3m - 0.7).abs() < 1e-3);

    // 5. Past apex by 5m: half faded (0.5)
    let a_past_5m = compute_indicator_alpha(-30.0, -5.0, true);
    assert!((a_past_5m - 0.5).abs() < 1e-3);

    // 6. Past apex by 10m: fully faded (0.0)
    let a_past_10m = compute_indicator_alpha(-35.0, -10.0, true);
    assert!((a_past_10m - 0.0).abs() < 1e-3);

    // 7. Beyond 10m: clamped to 0.0
    let a_past_15m = compute_indicator_alpha(-40.0, -15.0, true);
    assert_eq!(a_past_15m, 0.0);
}

#[test]
fn test_curve_arrow_positioning_horizontal_and_clearance() {
    let track = classic_grand_prix();
    let sample = &track.spline.samples[0];
    let player_car = Car::new(CarConfig::sports_car())
        .with_pose(sample.point, 0.0);
    let zoom = 12.0;

    // 1. Left curve: arrow must be placed to the LEFT of the player car (x < car.x)
    let pos_left = compute_curve_arrow_position(&player_car, CurveDirection::Left, 3, zoom);
    assert!(
        pos_left.x < player_car.state.position.x,
        "Left curve arrow ({}) must be to the left of the player car ({})",
        pos_left.x,
        player_car.state.position.x
    );

    // 2. Right curve: arrow must be placed to the RIGHT of the player car (x > car.x)
    let pos_right = compute_curve_arrow_position(&player_car, CurveDirection::Right, 3, zoom);
    assert!(
        pos_right.x > player_car.state.position.x,
        "Right curve arrow ({}) must be to the right of the player car ({})",
        pos_right.x,
        player_car.state.position.x
    );

    // 3. Strictly horizontal: Y coordinate must always match the car's vertical elevation level
    let expected_y = player_car.state.position.y + player_car.total_elevation();
    assert_eq!(
        pos_left.y, expected_y,
        "Left arrow Y ({}) must be strictly horizontal with car ({})",
        pos_left.y, expected_y
    );
    assert_eq!(
        pos_right.y, expected_y,
        "Right arrow Y ({}) must be strictly horizontal with car ({})",
        pos_right.y, expected_y
    );

    // 4. Spacing: generous clearance from car center
    let dist_left = pos_left.distance(player_car.state.position);
    let dist_right = pos_right.distance(player_car.state.position);
    assert!(dist_left >= 4.0 && dist_left <= 12.0, "Left arrow clearance must be comfortable (got {})", dist_left);
    assert!(dist_right >= 4.0 && dist_right <= 12.0, "Right arrow clearance must be comfortable (got {})", dist_right);

    // 5. Multi-arrow expansion: even with 5 chevrons, the nearest chevron remains comfortably clear of the car
    let pos_5 = compute_curve_arrow_position(&player_car, CurveDirection::Right, 5, zoom);
    let total_w_5 = (4.0 * 16.0 + 14.0) / zoom;
    let closest_chevron_dist = (pos_5.x - total_w_5 * 0.5) - player_car.state.position.x;
    assert!(
        closest_chevron_dist >= 4.0,
        "Innermost chevron of 5-arrow alert must remain clear of car (got {:.2}m)",
        closest_chevron_dist
    );

    // 6. Backwards compatibility alias returns identical position without dynamic computation
    let pos_compat = compute_smart_curve_arrow_position(&track, &[], &player_car, CurveDirection::Right, 5, zoom);
    assert_eq!(pos_compat, pos_5);
}

#[test]
fn test_chained_curve_hud_preemption_updates_arrow_and_color() {
    let curve1 = TrackCurve {
        id: 0,
        direction: CurveDirection::Right,
        degree: 1,
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
        degree: 5,
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

    let car = Car::new(CarConfig::sports_car()).with_pose(glam::Vec2::new(50.0, 50.0), 0.0);
    let zoom = 12.0;

    // Phase 1: On far approach (dist = 0m, speed = 40 m/s):
    // Turn 1 is returned (Right, Degree 1, Cruise Green)
    let s_far = evaluate_curve_approach(&curves, 0.0, total_len, closed, 40.0, 200.0).unwrap();
    assert_eq!(s_far.curve.id, 0);
    assert_eq!(s_far.curve.direction, CurveDirection::Right);
    let pos_far = compute_curve_arrow_position(&car, s_far.curve.direction, s_far.curve.degree, zoom);
    assert!(pos_far.x > car.state.position.x, "Far approach shows Right arrow to right of car");
    let (col_far, _) = compute_curve_colors(CurveColorScheme::Traffic, s_far.urgency, s_far.curve.degree, 1.0);
    assert!(col_far.g > col_far.r, "Far approach on mild turn has green cruise color");

    // Phase 2: Approaching Turn 1 / before Turn 1 apex (dist = 80m, speed = 40 m/s):
    // Turn 2 is much harder and enters yellow/red braking zone!
    // Turn 2 immediately preempts Turn 1!
    let s_preempt = evaluate_curve_approach(&curves, 80.0, total_len, closed, 40.0, 200.0).unwrap();
    assert_eq!(s_preempt.curve.id, 1, "Turn 2 must preempt Turn 1 before Turn 1 apex");
    assert_eq!(s_preempt.curve.direction, CurveDirection::Left);
    assert_eq!(s_preempt.curve.degree, 5);
    let pos_preempt = compute_curve_arrow_position(&car, s_preempt.curve.direction, s_preempt.curve.degree, zoom);
    assert!(pos_preempt.x < car.state.position.x, "Preempted arrow updates to Left side of car");
    let alpha_preempt = compute_indicator_alpha(s_preempt.distance_to_entry, s_preempt.distance_to_apex, s_preempt.is_inside_curve);
    assert!((alpha_preempt - 1.0).abs() < 1e-3, "Turn 2 alert is fully visible");
    let (col_preempt, _) = compute_curve_colors(CurveColorScheme::Traffic, s_preempt.urgency, s_preempt.curve.degree, alpha_preempt);
    assert!(col_preempt.r > 0.8, "Turn 2 alert is urgent warning/red color");
}



