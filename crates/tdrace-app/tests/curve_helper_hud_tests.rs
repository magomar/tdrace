use tdrace_app::ui::curve_indicator::{
    bifurcation_disc_center, compute_bifurcation_alpha, compute_curve_colors, compute_fork_points,
    compute_indicator_alpha, compute_pacenote_polyline, curve_indicator_lookahead, upcoming_bifurcation,
    BifurcationApproachStatus, CurveColorScheme,
};
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::checkpoint::{MultiRouteProgressTracker, TrackProgressTracker};
use tdrace_core::track::network::{JunctionKind, SegmentId};
use tdrace_core::track::curve::{evaluate_curve_approach, CurveDirection, TrackCurve};

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
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    assert!(!track.spline.curves.is_empty(), "Track must contain detected curves");

    // Player approaching first corner on main straight at 55 m/s (198 km/h)
    let lookahead_fast = curve_indicator_lookahead(55.0);
    let status_fast = track.spline.upcoming_curve(50.0, 55.0, lookahead_fast);
    assert!(status_fast.is_some(), "Should detect upcoming curve from main straight");

    let s = status_fast.unwrap();
    assert!(s.curve.degree >= 1 && s.curve.degree <= 5);
    assert!(s.required_braking_distance > 0.0, "High speed approach requires braking");

    // Player driving very slowly (5 m/s = 18 km/h) approaching corner (e.g. at 230m, 50m before entry at 280m)
    let lookahead_slow = curve_indicator_lookahead(5.0);
    let status_slow = track.spline.upcoming_curve(230.0, 5.0, lookahead_slow);
    assert!(status_slow.is_some());
    let s_slow = status_slow.unwrap();
    assert_eq!(s_slow.urgency, 0.0, "Slow approach requires zero braking urgency");
    assert_eq!(s_slow.required_braking_distance, 0.0);
}

#[test]
fn test_indicator_alpha_quick_fade_past_apex() {
    // 1. Far approach at 40 m/s: 4.5 s (180m) is 0.0, 4.25 s (170m) is 0.5, 4.0 s (160m) is 1.0
    let a_180 = compute_indicator_alpha(180.0, 210.0, false, 40.0);
    assert!((a_180 - 0.0).abs() < 1e-3);

    let a_170 = compute_indicator_alpha(170.0, 200.0, false, 40.0);
    assert!((a_170 - 0.5).abs() < 1e-3);

    let a_160 = compute_indicator_alpha(160.0, 190.0, false, 40.0);
    assert!((a_160 - 1.0).abs() < 1e-3);

    let a_100 = compute_indicator_alpha(100.0, 130.0, false, 40.0);
    assert!((a_100 - 1.0).abs() < 1e-3);

    // 2. Inside curve approaching apex: alpha is full 1.0
    let a_in_turn = compute_indicator_alpha(-10.0, 15.0, true, 40.0);
    assert!((a_in_turn - 1.0).abs() < 1e-3);

    // 3. At apex: alpha is full 1.0
    let a_at_apex = compute_indicator_alpha(-25.0, 0.0, true, 40.0);
    assert!((a_at_apex - 1.0).abs() < 1e-3);

    // 4. Past apex by 3m: fades quickly (1.0 - 0.3 = 0.7)
    let a_past_3m = compute_indicator_alpha(-28.0, -3.0, true, 40.0);
    assert!((a_past_3m - 0.7).abs() < 1e-3);

    // 5. Past apex by 5m: half faded (0.5)
    let a_past_5m = compute_indicator_alpha(-30.0, -5.0, true, 40.0);
    assert!((a_past_5m - 0.5).abs() < 1e-3);

    // 6. Past apex by 10m: fully faded (0.0)
    let a_past_10m = compute_indicator_alpha(-35.0, -10.0, true, 40.0);
    assert!((a_past_10m - 0.0).abs() < 1e-3);

    // 7. Beyond 10m: clamped to 0.0
    let a_past_15m = compute_indicator_alpha(-40.0, -15.0, true, 40.0);
    assert_eq!(a_past_15m, 0.0);
}

#[test]
fn test_indicator_alpha_uses_time_to_entry_not_distance() {
    // Same 300m gap: a fast car (80 m/s, 3.75 s away) already sees the turn, a slow one (20 m/s, 15 s) does not
    let fast = compute_indicator_alpha(300.0, 330.0, false, 80.0);
    let slow = compute_indicator_alpha(300.0, 330.0, false, 20.0);
    assert!((fast - 1.0).abs() < 1e-3, "Fast car must see the turn 300m out (got {})", fast);
    assert_eq!(slow, 0.0, "Slow car must not see the turn 300m out (got {})", slow);

    // Speed floor: a stopped car still sees a turn 60m ahead (60m / 15 m/s = 4.0 s)
    let stopped = compute_indicator_alpha(60.0, 90.0, false, 0.0);
    assert!((stopped - 1.0).abs() < 1e-3, "Stopped car must see a turn 60m ahead (got {})", stopped);

    // The curve search reaches the start of the fade window at every speed
    for speed in [0.0f32, 10.0, 40.0, 80.0, 110.0] {
        let lookahead = curve_indicator_lookahead(speed);
        let at_edge = compute_indicator_alpha(lookahead, lookahead + 30.0, false, speed);
        assert!(at_edge < 1e-3, "Fade must start at the lookahead edge (speed {}, alpha {})", speed, at_edge);
        let just_inside = compute_indicator_alpha(lookahead * 0.85, lookahead + 30.0, false, speed);
        assert!(just_inside > 0.99, "Turn inside the lookahead must be fully visible (speed {})", speed);
    }
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


    // Phase 1: On far approach (dist = 0m, speed = 40 m/s):
    // Turn 1 is returned (Right, Degree 1, Cruise Green)
    let s_far = evaluate_curve_approach(&curves, 0.0, total_len, closed, 40.0, 200.0).unwrap();
    assert_eq!(s_far.curve.id, 0);
    assert_eq!(s_far.curve.direction, CurveDirection::Right);
    let (col_far, _) = compute_curve_colors(CurveColorScheme::Traffic, s_far.urgency, s_far.curve.degree, 1.0);
    assert!(col_far.g > col_far.r, "Far approach on mild turn has green cruise color");

    // Phase 2: Approaching Turn 1 / before Turn 1 apex (dist = 80m, speed = 40 m/s):
    // Turn 2 is much harder and enters yellow/red braking zone!
    // Turn 2 immediately preempts Turn 1!
    let s_preempt = evaluate_curve_approach(&curves, 80.0, total_len, closed, 40.0, 200.0).unwrap();
    assert_eq!(s_preempt.curve.id, 1, "Turn 2 must preempt Turn 1 before Turn 1 apex");
    assert_eq!(s_preempt.curve.direction, CurveDirection::Left);
    assert_eq!(s_preempt.curve.degree, 5);
    let alpha_preempt = compute_indicator_alpha(s_preempt.distance_to_entry, s_preempt.distance_to_apex, s_preempt.is_inside_curve, 40.0);
    assert!((alpha_preempt - 1.0).abs() < 1e-3, "Turn 2 alert is fully visible");
    let (col_preempt, _) = compute_curve_colors(CurveColorScheme::Traffic, s_preempt.urgency, s_preempt.curve.degree, alpha_preempt);
    assert!(col_preempt.r > 0.8, "Turn 2 alert is urgent warning/red color");
}




#[test]
fn test_pacenote_polyline_draws_curve_shape_in_icon_box() {
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    assert!(!track.spline.curves.is_empty());
    let center = glam::Vec2::new(100.0, -40.0);
    let size = 3.0;

    for curve in &track.spline.curves {
        let pts = compute_pacenote_polyline(&track.spline, curve, center, size);
        assert!(pts.len() >= 2);

        // 1. Fits the icon box: longer side is `size`, centered on `center`
        let (lo, hi) = pts.iter().fold(
            (glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MIN)),
            |(lo, hi), p| (lo.min(*p), hi.max(*p)),
        );
        assert!(((hi - lo).max_element() - size).abs() < 1e-3, "Curve {} icon must be {} wide", curve.id, size);
        assert!(((lo + hi) * 0.5 - center).length() < 1e-3, "Curve {} icon must be centered", curve.id);

        // 2. Turns the same way as the curve: summed signed heading change > 0 for Left (CCW)
        let mut turn = 0.0f32;
        for w in pts.windows(3) {
            let a = w[1] - w[0];
            let b = w[2] - w[1];
            turn += a.perp_dot(b).atan2(a.dot(b));
        }
        match curve.direction {
            CurveDirection::Left => assert!(turn > 0.0, "Left curve {} icon must turn left (got {:.2})", curve.id, turn),
            CurveDirection::Right => assert!(turn < 0.0, "Right curve {} icon must turn right (got {:.2})", curve.id, turn),
        }
    }
}

#[test]
fn test_old_config_with_curve_indicator_style_still_loads() {
    // config.toml files saved while the chevrons look existed still hold this key
    let old: tdrace_app::config::PlayerHelpersConfig =
        toml::from_str("curve_helper = true\ncurve_indicator_style = \"chevrons\"").unwrap();
    assert!(old.curve_helper);
}

// ---------------------------------------------------------------------------------------------------------------
// Spec 085: bifurcation pacenote
// ---------------------------------------------------------------------------------------------------------------

fn status_at(distance: f32) -> BifurcationApproachStatus {
    BifurcationApproachStatus {
        distance_to_split: distance,
        divergence_angle: 30.0,
        branch_left_is_tactical: true,
        branch_right_is_tactical: false,
        recommended_branch_left: false,
        is_pit_entry: false,
        is_joker_split: true,
    }
}

#[test]
fn test_bifurcation_badge_fades_in_between_4_5_and_4_0_seconds() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let speed = 40.0; // lookahead 4.5 s * 40 m/s = 180 m
    assert!(compute_bifurcation_alpha(&status_at(190.0), speed) < 1e-3, "ETA 4.75 s: hidden");
    let mid = compute_bifurcation_alpha(&status_at(170.0), speed);
    assert!((mid - 0.5).abs() < 0.01, "ETA 4.25 s: half visible, got {mid}");
    assert!((compute_bifurcation_alpha(&status_at(160.0), speed) - 1.0).abs() < 1e-3, "ETA 4.0 s: full");
    assert!((compute_bifurcation_alpha(&status_at(0.0), speed) - 1.0).abs() < 1e-3, "at the split: full");
    // A badge that fades with the car crossing the split, like the curve pacenote past its apex.
    assert!((compute_bifurcation_alpha(&status_at(-5.0), speed) - 0.5).abs() < 1e-3);
    assert!(compute_bifurcation_alpha(&status_at(-10.0), speed) < 1e-3);
}

/// Tracker on `segment` of `track`, `progress` meters along it.
fn tracker_on(segment: SegmentId, progress: f32) -> TrackProgressTracker {
    let mut tracker = TrackProgressTracker::new(1, 1);
    let mut multi = MultiRouteProgressTracker::new("main", segment, 1);
    multi.segment_progress_distance = progress;
    tracker.multi_route = Some(multi);
    tracker
}

#[test]
fn test_joker_split_is_found_ahead_on_the_ingress_segment() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let track = tdrace_core::catalog::official_track("classic", "rx_canyon_flyer");
    let network = track.network.as_ref().unwrap();
    let ingress_seg = network.get_segment(SegmentId(0)).expect("start straight");
    let speed = 30.0; // lookahead 135 m

    // 100 m before the split: found, ahead by 100 m.
    let near = upcoming_bifurcation(&track, &tracker_on(SegmentId(0), ingress_seg.length - 100.0), speed, true, false)
        .expect("joker split within the lookahead");
    assert!(near.is_joker_split && !near.is_pit_entry);
    assert!((near.distance_to_split - 100.0).abs() < 1.0, "distance {}", near.distance_to_split);
    assert!(near.divergence_angle > 1.0, "the branches must diverge, got {}", near.divergence_angle);
    assert_ne!(near.branch_left_is_tactical, near.branch_right_is_tactical, "one tactical branch");

    // 200 m before: beyond the 4.5 s window.
    assert!(upcoming_bifurcation(&track, &tracker_on(SegmentId(0), ingress_seg.length - 200.0), speed, true, false).is_none());
}

#[test]
fn test_joker_split_marks_the_joker_branch_as_tactical_and_recommends_it_only_when_owed() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let track = tdrace_core::catalog::official_track("classic", "rx_canyon_flyer");
    let network = track.network.as_ref().unwrap();
    let tracker = tracker_on(SegmentId(0), network.get_segment(SegmentId(0)).unwrap().length - 80.0);

    let owed = upcoming_bifurcation(&track, &tracker, 30.0, true, false).unwrap();
    let paid = upcoming_bifurcation(&track, &tracker, 30.0, false, false).unwrap();
    assert!(owed.recommends_tactical(), "a driver who owes the joker is sent to the joker road");
    assert!(!paid.recommends_tactical(), "a driver who has taken the joker is kept on the main line");
    assert_eq!(owed.branch_left_is_tactical, paid.branch_left_is_tactical, "geometry does not depend on the plan");

    // The tactical side must be the side the joker road (segment 2) leaves to.
    let JunctionKind::Split { ingress_socket, .. } = &network.junctions[0].kind else { panic!("junction 0 is the split") };
    let joker_probe = network.get_segment(SegmentId(2)).unwrap().sample_at_distance(20.0).point;
    let main_probe = network.get_segment(SegmentId(1)).unwrap().sample_at_distance(20.0).point;
    let side = |p: glam::Vec2| ingress_socket.tangent.perp_dot(p - ingress_socket.point);
    assert_eq!(owed.branch_left_is_tactical, side(joker_probe) > side(main_probe), "tactical branch side");
}

#[test]
fn test_joker_badge_fades_out_just_past_the_split_and_ignores_the_loop_closure_link() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let track = tdrace_core::catalog::official_track("classic", "rx_canyon_flyer");
    // 4 m into the joker road: the split is 4 m behind.
    let past = upcoming_bifurcation(&track, &tracker_on(SegmentId(2), 4.0), 30.0, true, false).expect("just past the split");
    assert!((past.distance_to_split + 4.0).abs() < 1e-3);
    assert!(compute_bifurcation_alpha(&past, 30.0) > 0.0);
    // 40 m in: gone.
    assert!(upcoming_bifurcation(&track, &tracker_on(SegmentId(2), 40.0), 30.0, true, false).is_none());
    // Segment 4 is linked to the split junction as a loop closure, but its end is the start line, not the split.
    let network = track.network.as_ref().unwrap();
    let end_of_loop = network.get_segment(SegmentId(4)).unwrap().length - 50.0;
    assert!(upcoming_bifurcation(&track, &tracker_on(SegmentId(4), end_of_loop), 30.0, true, false).is_none());
}

#[test]
fn test_every_rx_circuit_gets_a_bifurcation_badge_before_its_joker_split() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let circuits = [("classic", "rx_quarry_sprint"), ("classic", "rx_hilltop_leap"), ("classic", "rx_canyon_flyer"), ("rally", "holjes_rx"), ("rally", "lydden_hill"), ("rally", "spa_rx"), ("rally", "silverstone_rx"), ("rally", "dreux_rx")];
    for (category, id) in circuits {
        let track = tdrace_core::catalog::official_track(category, id);
        let network = track.network.as_ref().unwrap();
        let JunctionKind::Split { ingress_socket, .. } = &network.junctions[0].kind else { panic!("{id}: junction 0 is the split") };
        let seg = network
            .segments
            .iter()
            .find(|s| s.samples.last().is_some_and(|p| p.point.distance(ingress_socket.point) < 3.0) && s.exit_junction.is_some())
            .unwrap_or_else(|| panic!("{id}: no segment ends at the split"));
        let progress = (seg.length - 60.0).max(0.0);
        let status = upcoming_bifurcation(&track, &tracker_on(seg.id, progress), 30.0, true, false)
            .unwrap_or_else(|| panic!("{id}: no badge 60 m before the split"));
        assert!(status.is_joker_split, "{id}");
        assert_ne!(status.branch_left_is_tactical, status.branch_right_is_tactical, "{id}: one tactical branch");
    }
}

fn pit_circuit() -> tdrace_core::track::Track {
    let track = tdrace_core::catalog::official_track("gt", "catalunya");
    assert!(track.pit_lane.is_some(), "catalunya has a pit lane");
    track
}

/// Tracker on a plain circuit at `progress` meters.
fn plain_tracker(progress: f32) -> TrackProgressTracker {
    let mut tracker = TrackProgressTracker::new(1, 1);
    tracker.progress_distance = progress;
    tracker
}

#[test]
fn test_pit_entry_badge_shows_before_the_pit_lane_on_the_pit_side() {
    // Scenario: Bifurcation Pacenote HUD indicator triggers on approach
    let track = pit_circuit();
    let lane = track.pit_lane.as_ref().unwrap();
    let junctions = track.pit_lane_junctions.clone().or_else(|| track.compute_pit_lane_junctions()).unwrap();
    let apex = track.spline.project_point(junctions.p_apex);
    let total = track.spline.total_length();
    let before = |meters: f32| plain_tracker((apex.progress_distance - meters).rem_euclid(total));

    let status = upcoming_bifurcation(&track, &before(80.0), 30.0, false, false).expect("pit entry within the lookahead");
    assert!(status.is_pit_entry && !status.is_joker_split);
    assert!((status.distance_to_split - 80.0).abs() < 1.0, "distance {}", status.distance_to_split);
    assert!(upcoming_bifurcation(&track, &before(400.0), 30.0, false, false).is_none(), "beyond 4.5 s");

    // The tactical branch is on the side the pit road lies.
    let start = track.spline.project_point(junctions.te_start);
    let free_end = (lane.entry_gate.start + lane.entry_gate.end) * 0.5;
    let pit_is_left = start.tangent.perp_dot(free_end - start.closest_point) > 0.0;
    assert_eq!(status.branch_left_is_tactical, pit_is_left);

    // A pit stop advised: the pit road is recommended. Otherwise the main line is.
    assert!(upcoming_bifurcation(&track, &before(80.0), 30.0, false, true).unwrap().recommends_tactical());
    assert!(!status.recommends_tactical());

    // Already in the pit lane: no badge.
    let mut in_lane = before(80.0);
    in_lane.in_pit_lane = true;
    assert!(upcoming_bifurcation(&track, &in_lane, 30.0, false, true).is_none());
}

#[test]
fn test_bifurcation_disc_sits_on_the_tactical_side_beside_the_car() {
    let car = Car::new(CarConfig::sports_car()).with_pose(glam::Vec2::new(10.0, 5.0), 0.7);
    for tactical_left in [true, false] {
        let mut status = status_at(100.0);
        status.branch_left_is_tactical = tactical_left;
        status.branch_right_is_tactical = !tactical_left;
        let center = bifurcation_disc_center(&car, &status, 1.0, 1.0);
        let lateral = (center - car.state.position).dot(car.right_vector());
        assert_eq!(lateral < 0.0, tactical_left, "disc on the tactical side");
        // Beyond the car's half track width, so the disc never hides the car.
        assert!(lateral.abs() > car.config.track_width * 0.5 + 0.5, "disc clear of the car body, lateral {lateral}");
        // And level with the car, not ahead of it.
        assert!((center - car.state.position).dot(car.forward_vector()).abs() < 1e-3);
    }
}

#[test]
fn test_fork_icon_branches_leave_to_the_matching_sides_with_a_readable_spread() {
    let fwd = glam::Vec2::new(0.0, 1.0);
    let left = glam::Vec2::new(-1.0, 0.0);
    for divergence in [2.0f32, 30.0, 70.0, 140.0] {
        let fork = compute_fork_points(fwd, left, glam::Vec2::ZERO, 10.0, divergence);
        assert!((fork.left_tip - fork.junction).dot(left) > 0.0, "left tip on the left");
        assert!((fork.right_tip - fork.junction).dot(left) < 0.0, "right tip on the right");
        assert!((fork.junction - fork.stem_start).dot(fwd) > 0.0, "the stem comes from behind");
        let a = (fork.left_tip - fork.junction).normalize();
        let b = (fork.right_tip - fork.junction).normalize();
        let spread = a.dot(b).clamp(-1.0, 1.0).acos().to_degrees();
        assert!((27.9..=70.1).contains(&spread), "spread {spread} for divergence {divergence}");
    }
}
