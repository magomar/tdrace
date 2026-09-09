use tdrace_app::ui::curve_indicator::{compute_curve_colors, CurveColorScheme};
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
