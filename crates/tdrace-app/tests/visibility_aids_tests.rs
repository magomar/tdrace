use tdrace_app::render::{
    compute_adaptive_alpha, compute_adaptive_visibility, compute_traffic_crowding,
    PlayerVisibilityOptions,
};
use tdrace_app::ui::CurveColorScheme;
use tdrace_app::ui::hud::VisibilityToast;
use tdrace_app::game::RaceSession;

#[test]
fn test_player_visibility_options_defaults() {
    let opts = PlayerVisibilityOptions::default();
    assert!(opts.frontal_chevron, "Option 1 (Frontal Chevron) must be enabled by default");
    assert!((opts.frontal_chevron_scale - 1.0).abs() < 1e-4);
    assert!((opts.frontal_chevron_brightness - 1.0).abs() < 1e-4);
    assert!(opts.ground_aura, "Option 2 (Ground Aura) must be enabled by default");
    assert!((opts.ground_aura_radius_ratio - 1.0).abs() < 1e-4);
    assert!((opts.ground_aura_brightness - 1.0).abs() < 1e-4);
    assert!(opts.adaptive_visibility, "Option 3 (Adaptive Visibility) must be enabled by default");
    assert!(!opts.roof_beacon, "Roof Beacon must be disabled entirely");
    assert!(opts.sonar_ping, "Option 4 (Sonar Ping) must be enabled by default");
    assert!(opts.curve_helper, "Option 5 (Curve Helper) must be enabled by default");
    assert!((opts.curve_helper_scale - 1.0).abs() < 1e-4);
    assert!((opts.curve_helper_brightness - 1.0).abs() < 1e-4);
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Traffic);
}

#[test]
fn test_player_visibility_options_individual_toggles() {
    let mut opts = PlayerVisibilityOptions::default();

    // Toggle 1
    opts.frontal_chevron = !opts.frontal_chevron;
    assert!(!opts.frontal_chevron);
    assert!(opts.ground_aura);

    // Toggle 2
    opts.ground_aura = !opts.ground_aura;
    assert!(!opts.ground_aura);
    assert!(opts.adaptive_visibility);

    // Toggle 3
    opts.adaptive_visibility = !opts.adaptive_visibility;
    assert!(!opts.adaptive_visibility);
    assert!(!opts.roof_beacon);

    // Toggle 4 (Sonar Ping Shockwave Ripple)
    opts.sonar_ping = !opts.sonar_ping;
    assert!(!opts.sonar_ping);
    assert!(opts.curve_helper);

    // Toggle 5 (Curve Helper)
    opts.curve_helper = !opts.curve_helper;
    assert!(!opts.curve_helper);

    // Cycle 6 (Color Scheme: Traffic -> Synthwave -> Contrast -> Rally -> Traffic)
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Traffic);
    opts.curve_color_scheme = opts.curve_color_scheme.next();
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Synthwave);
    opts.curve_color_scheme = opts.curve_color_scheme.next();
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Contrast);
    opts.curve_color_scheme = opts.curve_color_scheme.next();
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Rally);
    opts.curve_color_scheme = opts.curve_color_scheme.next();
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Traffic);

    // Turn back on
    opts.frontal_chevron = true;
    opts.curve_helper = true;
    assert!(opts.frontal_chevron);
    assert!(opts.curve_helper);
}

#[test]
fn test_compute_adaptive_alpha_behavior() {
    use glam::Vec2;

    // Disabled should always return exact 1.0 multipliers regardless of zoom, speed, or traffic
    assert_eq!(compute_adaptive_alpha(false, 22.0, 40.0, 0.0), 1.0);
    assert_eq!(compute_adaptive_alpha(false, 3.5, 0.0, 0.0), 1.0);
    let disabled_factors = compute_adaptive_visibility(false, 5.0, 0.0, 0.0, 1.0);
    assert_eq!(disabled_factors.alpha_multiplier, 1.0);
    assert_eq!(disabled_factors.scale_multiplier, 1.0);

    // Enabled: Close zoom at racing speed with no traffic scales down to 0.50x
    let close_fast = compute_adaptive_visibility(true, 20.0, 40.0, 0.0, 0.0);
    assert!(
        (close_fast.alpha_multiplier - 0.50).abs() < 1e-4,
        "Close zoom with clear visibility should scale down to 0.50x: got {}",
        close_fast.alpha_multiplier
    );
    assert!(close_fast.scale_multiplier < 1.0);

    // Enabled: Far/Overview zoom boosts above 1.0 relative to configured settings
    let overview_fast = compute_adaptive_visibility(true, 3.5, 40.0, 0.0, 0.0);
    assert!(
        overview_fast.alpha_multiplier >= 1.30 && overview_fast.alpha_multiplier <= 1.50,
        "Overview zoom should boost above 1.0 within [0.5, 1.5]: got {}",
        overview_fast.alpha_multiplier
    );
    assert!(overview_fast.scale_multiplier > 1.05);

    // Enabled: Surrounding traffic crowding boosts visibility above 1.0 even at Close zoom
    let player_pos = Vec2::new(100.0, 100.0);
    let pack_positions = [
        Vec2::new(103.5, 101.0),
        Vec2::new(97.0, 99.0),
        Vec2::new(101.0, 105.0),
        Vec2::new(96.0, 104.0),
    ];
    let crowding = compute_traffic_crowding(player_pos, pack_positions);
    assert!(crowding > 0.75, "Close 4-car pack should produce high traffic crowding: got {}", crowding);
    let close_in_pack = compute_adaptive_visibility(true, 20.0, 40.0, 0.0, crowding);
    assert!(
        close_in_pack.alpha_multiplier > 1.15 && close_in_pack.alpha_multiplier <= 1.50,
        "Pack traffic at close zoom must boost visibility above 1.0 (up to 1.50x): got {}",
        close_in_pack.alpha_multiplier
    );
    assert!(close_in_pack.scale_multiplier > 1.0);

    // Enabled: Maximum boost (overview + pack + stopped) clamps cleanly at 1.50x
    let max_boost = compute_adaptive_visibility(true, 3.5, 0.0, 0.3, 1.0);
    assert!(
        (max_boost.alpha_multiplier - 1.50).abs() < 1e-4,
        "Maximum adaptive visibility multiplier must clamp at 1.50x: got {}",
        max_boost.alpha_multiplier
    );

    // Enabled: Low speed / stationary boost
    let overview_stopped = compute_adaptive_alpha(true, 3.5, 0.0, 0.0);
    assert!(
        overview_stopped > overview_fast.alpha_multiplier && overview_stopped <= 1.50,
        "Stopped car in overview should have extra recovery boost up to 1.50x: got {}",
        overview_stopped
    );

    // Low speed boost should be greater than high speed at same zoom level
    let mid_zoom_stopped = compute_adaptive_alpha(true, 12.0, 0.0, 0.0);
    let mid_zoom_fast = compute_adaptive_alpha(true, 12.0, 30.0, 0.0);
    assert!(mid_zoom_stopped > mid_zoom_fast, "Stopped car should have higher alpha than fast car");
}

#[test]
fn test_visibility_toast_struct() {
    let toast = VisibilityToast {
        text: "[1] FRONTAL CHEVRON: ON".to_string(),
        is_on: true,
        timer: 1.8,
        duration: 1.8,
    };

    assert_eq!(toast.text, "[1] FRONTAL CHEVRON: ON");
    assert!(toast.is_on);
    assert!((toast.timer - 1.8).abs() < 1e-4);
    assert!((toast.duration - 1.8).abs() < 1e-4);
}

#[test]
fn test_race_session_visibility_initialization() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("helpers_visibility_init");
    let session = RaceSession::new();
    assert!(session.visibility_options.frontal_chevron);
    assert!(session.visibility_options.ground_aura);
    assert!(session.visibility_options.adaptive_visibility);
    assert!(!session.visibility_options.roof_beacon);
    assert!(session.visibility_options.curve_helper);
    assert!(session.visibility_options.sonar_ping);
    assert_eq!(session.visibility_options.curve_color_scheme, CurveColorScheme::Traffic);
    assert!(session.visibility_toast.is_none());
}

#[test]
fn test_render_player_visual_clues_headless_execution() {
    use glam::Vec2;
    use macroquad::color::Color;
    use tdrace_app::render::color::CarColorScheme;
    use tdrace_app::render::marker::{
        render_player_frontal_chevron, render_player_ground_aura, render_player_roof_beacon,
    };

    let schemes = [
        CarColorScheme::default(),
        CarColorScheme {
            primary: Color::new(0.05, 0.05, 0.05, 1.0),
            secondary: Color::new(0.02, 0.02, 0.02, 1.0),
            helmet: Color::new(0.8, 0.8, 0.1, 1.0),
        },
        CarColorScheme {
            primary: Color::new(1.0, 0.9, 0.1, 1.0),
            secondary: Color::new(0.1, 0.9, 0.9, 1.0),
            helmet: Color::new(1.0, 1.0, 1.0, 1.0),
        },
    ];

    let zooms = [5.0f32, 10.0, 16.5, 22.0];
    let pos = Vec2::new(100.0, 150.0);
    let fwd = Vec2::new(1.0, 0.0);

    for scheme in &schemes {
        for &zoom in &zooms {
            for &alpha in &[0.0f32, 0.20, 0.50, 1.0] {
                // Must execute calculations cleanly without panic prior to GPU submission
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    render_player_frontal_chevron(pos, fwd, zoom, scheme, alpha, 1.2, 1.5, 0.8);
                    render_player_ground_aura(pos, zoom, scheme, alpha, 0.8, 1.4);
                    render_player_roof_beacon(pos, fwd, 0.0, zoom, 1.25, scheme, alpha, 1.3);
                }));
            }
        }
    }
}

#[test]
fn test_player_chevron_points_toward_car_front_and_matches_aura() {
    use glam::Vec2;
    use tdrace_app::render::{compute_player_chevron_geometry, player_aura_radius};

    let pos = Vec2::new(10.0, 20.0);
    // Test northwest heading (like the reference buggy screenshot) and other orientations
    let headings = [
        Vec2::new(-0.70710677, 0.70710677),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, -1.0),
        Vec2::new(-0.5, -0.8660254),
    ];

    for &fwd in &headings {
        for &zoom in &[3.5f32, 8.0, 16.0] {
            for &ratio in &[0.75f32, 1.0, 1.35] {
                let expected_aura_r = player_aura_radius(zoom, ratio);
                let geom = compute_player_chevron_geometry(pos, fwd, zoom, 1.0, ratio);

                assert!(
                    (geom.aura_radius - expected_aura_r).abs() < 1e-5,
                    "Chevron base radius must match ground aura radius"
                );
                let tip_dir = (geom.tip - pos).normalize();
                assert!(
                    (tip_dir - fwd.normalize()).length() < 1e-4,
                    "Chevron tip must point toward the front of the car"
                );
                // Base corners sit flush on the aura perimeter
                assert!(((geom.base_left - pos).length() - expected_aura_r).abs() < 1e-4);
                assert!(((geom.base_right - pos).length() - expected_aura_r).abs() < 1e-4);
                // When zoomed far out (zoom = 3.5), screen-space tip length stays >= 10px
                assert!(geom.length * zoom >= 10.0 - 1e-3);
            }
        }
    }
}

#[test]
fn test_player_helpers_config_roundtrip() {
    use tdrace_app::PlayerHelpersConfig;

    let mut cfg = PlayerHelpersConfig::default();
    cfg.frontal_chevron = false;
    cfg.frontal_chevron_scale = 1.35;
    cfg.frontal_chevron_brightness = 2.10;
    cfg.ground_aura = false;
    cfg.ground_aura_radius_ratio = 0.55;
    cfg.ground_aura_brightness = 1.75;
    cfg.roof_beacon = false;
    cfg.roof_beacon_brightness = 0.70;
    cfg.curve_helper = false;
    cfg.curve_helper_scale = 1.60;
    cfg.curve_helper_brightness = 2.25;
    cfg.curve_color_scheme = "synthwave".to_string();
    cfg.adaptive_visibility = false;
    cfg.bot_nameplates = false;
    cfg.radar_sonar_ping = false;

    let opts = PlayerVisibilityOptions::from(&cfg);
    assert!(!opts.frontal_chevron);
    assert!((opts.frontal_chevron_scale - 1.35).abs() < 1e-4);
    assert!((opts.frontal_chevron_brightness - 2.10).abs() < 1e-4);
    assert!(!opts.ground_aura);
    assert!((opts.ground_aura_radius_ratio - 0.55).abs() < 1e-4);
    assert!((opts.ground_aura_brightness - 1.75).abs() < 1e-4);
    assert!(!opts.roof_beacon);
    assert!((opts.roof_beacon_brightness - 0.70).abs() < 1e-4);
    assert!(!opts.curve_helper);
    assert!((opts.curve_helper_scale - 1.60).abs() < 1e-4);
    assert!((opts.curve_helper_brightness - 2.25).abs() < 1e-4);
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Synthwave);
    assert!(!opts.adaptive_visibility);
    assert!(!opts.bot_nameplates);
    assert!(!opts.sonar_ping);

    let roundtrip = PlayerHelpersConfig::from(&opts);
    assert_eq!(roundtrip.frontal_chevron, cfg.frontal_chevron);
    assert!((roundtrip.frontal_chevron_scale - cfg.frontal_chevron_scale).abs() < 1e-4);
    assert!((roundtrip.frontal_chevron_brightness - cfg.frontal_chevron_brightness).abs() < 1e-4);
    assert_eq!(roundtrip.ground_aura, cfg.ground_aura);
    assert!((roundtrip.ground_aura_radius_ratio - cfg.ground_aura_radius_ratio).abs() < 1e-4);
    assert!((roundtrip.ground_aura_brightness - cfg.ground_aura_brightness).abs() < 1e-4);
    assert_eq!(roundtrip.roof_beacon, cfg.roof_beacon);
    assert!((roundtrip.roof_beacon_brightness - cfg.roof_beacon_brightness).abs() < 1e-4);
    assert_eq!(roundtrip.curve_helper, cfg.curve_helper);
    assert!((roundtrip.curve_helper_scale - cfg.curve_helper_scale).abs() < 1e-4);
    assert!((roundtrip.curve_helper_brightness - cfg.curve_helper_brightness).abs() < 1e-4);
    assert_eq!(roundtrip.curve_color_scheme, "synthwave");
    assert_eq!(roundtrip.adaptive_visibility, cfg.adaptive_visibility);
    assert_eq!(roundtrip.bot_nameplates, cfg.bot_nameplates);
    assert_eq!(roundtrip.radar_sonar_ping, cfg.radar_sonar_ping);
}

#[test]
fn test_race_session_settings_modal_helpers_workflow() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("helpers_settings_workflow");
    let mut session = RaceSession::new();
    session.open_settings_modal();
    assert!(session.is_settings_modal_open());

    if let Some(ref mut modal) = session.settings_modal {
        assert_eq!(modal.chevron_dropdown.label, "FRONTAL CHEVRON");
        modal.aura_dropdown.set_selected(1); // Disabled
        modal.aura_ratio_slider.set_value(0.60);
        modal.aura_brightness_slider.set_value(2.10);
        modal.ribbon_dropdown.set_selected(0); // Enabled
        modal.ribbon_scale_slider.set_value(1.75);
        modal.ribbon_brightness_slider.set_value(2.40);
        modal.chevron_dropdown.set_selected(1); // Disabled
        modal.chevron_brightness_slider.set_value(0.80);
        modal.adaptive_dropdown.set_selected(1); // Disabled
        modal.radar_ping_dropdown.set_selected(1); // Disabled
        modal.bot_nameplates_dropdown.set_selected(1); // Disabled
        modal.bot_nameplates_max_slider.set_value(18.0);
        modal.bot_nameplates_opacity_slider.set_value(65.0);
        modal.bot_nameplates_detail_dropdown.set_selected(1); // Compact
    }

    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());

    assert!(!session.config.player_helpers.ground_aura);
    assert!((session.config.player_helpers.ground_aura_radius_ratio - 0.60).abs() < 1e-4);
    assert!((session.config.player_helpers.ground_aura_brightness - 2.10).abs() < 1e-4);
    assert!(session.config.player_helpers.curve_helper);
    assert!((session.config.player_helpers.curve_helper_scale - 1.75).abs() < 1e-4);
    assert!((session.config.player_helpers.curve_helper_brightness - 2.40).abs() < 1e-4);
    assert!(!session.config.player_helpers.frontal_chevron);
    assert!((session.config.player_helpers.frontal_chevron_brightness - 0.80).abs() < 1e-4);
    assert!(!session.config.player_helpers.roof_beacon);
    assert!(!session.config.player_helpers.adaptive_visibility);
    assert!(!session.config.player_helpers.radar_sonar_ping);
    assert!(!session.config.player_helpers.bot_nameplates);
    assert_eq!(session.config.player_helpers.bot_nameplates_max_visible, 18);
    assert!((session.config.player_helpers.bot_nameplates_opacity - 0.65).abs() < 1e-4);
    assert!(session.config.player_helpers.bot_nameplates_compact);

    // Also assert runtime visibility_options synced
    assert!(!session.visibility_options.ground_aura);
    assert!((session.visibility_options.ground_aura_radius_ratio - 0.60).abs() < 1e-4);
    assert!((session.visibility_options.ground_aura_brightness - 2.10).abs() < 1e-4);
    assert!(session.visibility_options.curve_helper);
    assert!((session.visibility_options.curve_helper_scale - 1.75).abs() < 1e-4);
    assert!((session.visibility_options.curve_helper_brightness - 2.40).abs() < 1e-4);
    assert!(!session.visibility_options.frontal_chevron);
    assert!((session.visibility_options.frontal_chevron_brightness - 0.80).abs() < 1e-4);
    assert!(!session.visibility_options.roof_beacon);
    assert!(!session.visibility_options.adaptive_visibility);
    assert!(!session.visibility_options.sonar_ping);
    assert!(!session.visibility_options.bot_nameplates);
    assert_eq!(session.visibility_options.bot_nameplates_max_visible, 18);
    assert!((session.visibility_options.bot_nameplates_opacity - 0.65).abs() < 1e-4);
    assert!(session.visibility_options.bot_nameplates_compact);
}

#[test]
fn test_render_sonar_ping_headless_and_session_triggers() {
    use glam::Vec2;
    use tdrace_app::render::marker::render_player_sonar_ping;

    // 1. Headless render execution checks
    let pos = Vec2::new(50.0, 100.0);
    for &zoom in &[4.0f32, 10.0, 18.0] {
        for &progress in &[0.0f32, 0.15, 0.50, 0.85, 1.0] {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                render_player_sonar_ping(pos, zoom, progress, 0.75);
            }));
        }
    }

    // 2. Session zoom cycle trigger
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("helpers_sonar_trigger");
    let mut session = RaceSession::new();
    assert!(session.visibility_options.sonar_ping);
    assert_eq!(session.sonar_ping_timer, 0.0);

    // Zoom cycle triggers sonar ping when enabled
    session.cycle_camera_zoom();
    assert_eq!(session.sonar_ping_timer, 0.75);
    if let Some(pos) = session.world.vehicles.first().map(|c| c.state.position) {
        assert_eq!(session.sonar_ping_origin, pos);
    }

    // When disabled, trigger is bypassed
    session.visibility_options.sonar_ping = false;
    session.sonar_ping_timer = 0.0;
    session.cycle_camera_zoom();
    assert_eq!(session.sonar_ping_timer, 0.0);

    // 3. Auto-trigger on spin-out in Racing state with 3.0s cooldown
    session.visibility_options.sonar_ping = true;
    session.sonar_ping_timer = 0.0;
    session.sonar_ping_cooldown = 0.0;
    session.state = tdrace_app::game::GameState::Racing;
    if let Some(pc) = session.world.vehicles.first_mut() {
        pc.state.angular_velocity = 5.2; // > 4.5 rad/s
    }
    let expected_pos = session.world.vehicles.first().map(|c| c.state.position).unwrap();
    session.update();
    assert_eq!(session.sonar_ping_timer, 0.75);
    assert_eq!(session.sonar_ping_origin, expected_pos);
    assert!((session.sonar_ping_cooldown - 3.0).abs() < 0.1);
}

#[test]
fn test_sonar_ping_key4_toggle_and_modal_governance() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("sonar_key4_governance");
    let mut session = RaceSession::new();
    assert!(session.visibility_options.sonar_ping);

    // Toggle off via Key 4 action
    session.visibility_options.sonar_ping = !session.visibility_options.sonar_ping;
    assert!(!session.visibility_options.sonar_ping);

    // Turn off via modal
    session.open_settings_modal();
    if let Some(ref mut modal) = session.settings_modal {
        modal.radar_ping_dropdown.set_selected(1); // Disabled
    }
    session.close_settings_modal(true);
    assert!(!session.visibility_options.sonar_ping);

    // Verify in Racing state, session.update() preserves disabled state
    session.state = tdrace_app::game::GameState::Racing;
    session.update();
    assert!(!session.visibility_options.sonar_ping);

    // Re-enable via Key 4 toggle
    session.visibility_options.sonar_ping = !session.visibility_options.sonar_ping;
    assert!(session.visibility_options.sonar_ping);
    session.trigger_sonar_ping();
    assert_eq!(session.sonar_ping_timer, 0.75);

    // Re-enable via modal
    session.open_settings_modal();
    if let Some(ref mut modal) = session.settings_modal {
        modal.radar_ping_dropdown.set_selected(0); // Enabled
    }
    session.close_settings_modal(true);
    assert!(session.visibility_options.sonar_ping);
}



#[test]
fn test_curve_indicator_selector_roundtrip() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("curve_indicator_selector");
    let mut session = RaceSession::new();

    // Only two options: Rally Pacenote (default, option 0) and Disabled (option 1)
    session.open_settings_modal();
    let modal = session.settings_modal.as_ref().unwrap();
    assert_eq!(modal.ribbon_dropdown.options, vec!["Rally Pacenote".to_string(), "Disabled".to_string()]);
    assert_eq!(modal.ribbon_dropdown.selected_index, 0);

    // 1. Pick "Disabled": helper turns off
    session.settings_modal.as_mut().unwrap().ribbon_dropdown.set_selected(1);
    session.close_settings_modal(true);
    assert!(!session.config.player_helpers.curve_helper);
    assert!(!session.visibility_options.curve_helper);

    session.open_settings_modal();
    assert_eq!(session.settings_modal.as_ref().unwrap().ribbon_dropdown.selected_index, 1);

    // 2. Full preset turns it back on
    let modal = session.settings_modal.as_mut().unwrap();
    modal.apply_visual_aids_preset(0);
    assert_eq!(modal.ribbon_dropdown.selected_index, 0, "Full preset enables the pacenote");
    assert_eq!(modal.visual_aids_preset_dropdown.selected_index, 0);
    modal.apply_visual_aids_preset(2);
    assert_eq!(modal.ribbon_dropdown.selected_index, 1, "Off preset disables the indicator");
    modal.apply_visual_aids_preset(0);
    session.close_settings_modal(true);
    assert!(session.visibility_options.curve_helper);
}

#[test]
fn test_render_curve_pacenote_headless_execution() {
    use tdrace_app::ui::curve_indicator::render_curve_pacenote;
    use tdrace_core::physics::car::Car;
    use tdrace_core::physics::config::CarConfig;

    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let car = Car::new(CarConfig::sports_car());
    if let Some(status) = track.spline.upcoming_curve(10.0, 20.0, 150.0) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_curve_pacenote(&car, &status, &track.spline, CurveColorScheme::Rally, 12.0, 1.5, 1.25, 1.8);
        }));
    }
}
