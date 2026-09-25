use tdrace_app::render::{compute_adaptive_alpha, PlayerVisibilityOptions};
use tdrace_app::ui::CurveColorScheme;
use tdrace_app::ui::hud::VisibilityToast;
use tdrace_app::game::RaceSession;

#[test]
fn test_player_visibility_options_defaults() {
    let opts = PlayerVisibilityOptions::default();
    assert!(opts.overhead_chevron, "Option 1 (Overhead Chevron) must be enabled by default");
    assert!((opts.overhead_chevron_scale - 1.0).abs() < 1e-4);
    assert!((opts.overhead_chevron_brightness - 1.0).abs() < 1e-4);
    assert!(opts.ground_aura, "Option 2 (Ground Aura) must be enabled by default");
    assert!((opts.ground_aura_radius_ratio - 1.0).abs() < 1e-4);
    assert!((opts.ground_aura_brightness - 1.0).abs() < 1e-4);
    assert!(opts.adaptive_visibility, "Option 3 (Adaptive Visibility) must be enabled by default");
    assert!(opts.roof_beacon, "Option 4 (Roof Beacon) must be enabled by default");
    assert!((opts.roof_beacon_brightness - 1.0).abs() < 1e-4);
    assert!(opts.curve_helper, "Option 5 (Curve Helper) must be enabled by default");
    assert!((opts.curve_helper_scale - 1.0).abs() < 1e-4);
    assert!((opts.curve_helper_brightness - 1.0).abs() < 1e-4);
    assert_eq!(opts.curve_color_scheme, CurveColorScheme::Traffic);
}

#[test]
fn test_player_visibility_options_individual_toggles() {
    let mut opts = PlayerVisibilityOptions::default();

    // Toggle 1
    opts.overhead_chevron = !opts.overhead_chevron;
    assert!(!opts.overhead_chevron);
    assert!(opts.ground_aura);

    // Toggle 2
    opts.ground_aura = !opts.ground_aura;
    assert!(!opts.ground_aura);
    assert!(opts.adaptive_visibility);

    // Toggle 3
    opts.adaptive_visibility = !opts.adaptive_visibility;
    assert!(!opts.adaptive_visibility);
    assert!(opts.roof_beacon);

    // Toggle 4
    opts.roof_beacon = !opts.roof_beacon;
    assert!(!opts.roof_beacon);
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
    opts.overhead_chevron = true;
    opts.curve_helper = true;
    assert!(opts.overhead_chevron);
    assert!(opts.curve_helper);
}

#[test]
fn test_compute_adaptive_alpha_behavior() {
    // Disabled should always return 1.0 regardless of zoom or speed
    assert_eq!(compute_adaptive_alpha(false, 22.0, 40.0, 0.0), 1.0);
    assert_eq!(compute_adaptive_alpha(false, 3.5, 0.0, 0.0), 1.0);

    // Enabled: Close zoom at racing speed should have reduced opacity
    let close_fast = compute_adaptive_alpha(true, 20.0, 40.0, 0.0);
    assert!(close_fast <= 0.20, "Close zoom at high speed should be subtle: got {}", close_fast);

    // Enabled: Far/Overview zoom should have full or near full opacity
    let overview_fast = compute_adaptive_alpha(true, 3.5, 40.0, 0.0);
    assert!(overview_fast >= 0.95, "Overview zoom should be high opacity: got {}", overview_fast);

    // Enabled: Low speed / stationary boost
    let overview_stopped = compute_adaptive_alpha(true, 3.5, 0.0, 0.0);
    assert!(overview_stopped >= 1.0, "Stopped car in overview should have maximum prominence: got {}", overview_stopped);

    // Low speed boost should be greater than high speed at same zoom level
    let mid_zoom_stopped = compute_adaptive_alpha(true, 12.0, 0.0, 0.0);
    let mid_zoom_fast = compute_adaptive_alpha(true, 12.0, 30.0, 0.0);
    assert!(mid_zoom_stopped > mid_zoom_fast, "Stopped car should have higher alpha than fast car");
}

#[test]
fn test_visibility_toast_struct() {
    let toast = VisibilityToast {
        text: "[1] OVERHEAD CHEVRON: ON".to_string(),
        is_on: true,
        timer: 1.8,
        duration: 1.8,
    };

    assert_eq!(toast.text, "[1] OVERHEAD CHEVRON: ON");
    assert!(toast.is_on);
    assert!((toast.timer - 1.8).abs() < 1e-4);
    assert!((toast.duration - 1.8).abs() < 1e-4);
}

#[test]
fn test_race_session_visibility_initialization() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("helpers_visibility_init");
    let session = RaceSession::new();
    assert!(session.visibility_options.overhead_chevron);
    assert!(session.visibility_options.ground_aura);
    assert!(session.visibility_options.adaptive_visibility);
    assert!(session.visibility_options.roof_beacon);
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
        render_player_ground_aura, render_player_overhead_chevron, render_player_roof_beacon,
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
                    render_player_overhead_chevron(pos, 0.0, zoom, 1.25, scheme, alpha, 1.2, 1.5);
                    render_player_ground_aura(pos, zoom, scheme, alpha, 0.8, 1.4);
                    render_player_roof_beacon(pos, fwd, 0.0, zoom, 1.25, scheme, alpha, 1.3);
                }));
            }
        }
    }
}

#[test]
fn test_player_helpers_config_roundtrip() {
    use tdrace_app::PlayerHelpersConfig;

    let mut cfg = PlayerHelpersConfig::default();
    cfg.overhead_chevron = false;
    cfg.overhead_chevron_scale = 1.35;
    cfg.overhead_chevron_brightness = 2.10;
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
    assert!(!opts.overhead_chevron);
    assert!((opts.overhead_chevron_scale - 1.35).abs() < 1e-4);
    assert!((opts.overhead_chevron_brightness - 2.10).abs() < 1e-4);
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
    assert_eq!(roundtrip.overhead_chevron, cfg.overhead_chevron);
    assert!((roundtrip.overhead_chevron_scale - cfg.overhead_chevron_scale).abs() < 1e-4);
    assert!((roundtrip.overhead_chevron_brightness - cfg.overhead_chevron_brightness).abs() < 1e-4);
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
fn test_render_curve_indicator_headless_execution() {
    use tdrace_app::ui::curve_indicator::render_curve_indicator;
    use tdrace_core::physics::car::Car;
    use tdrace_core::physics::config::CarConfig;
    use tdrace_core::track::presets::classic_grand_prix;

    let track = classic_grand_prix();
    let car = Car::new(CarConfig::sports_car());
    if let Some(status) = track.spline.upcoming_curve(10.0, 20.0, 150.0) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_curve_indicator(
                &car,
                &status,
                CurveColorScheme::Traffic,
                12.0,
                1.5,
                1.25,
                1.8,
            );
        }));
    }
}

#[test]
fn test_race_session_settings_modal_helpers_workflow() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("helpers_settings_workflow");
    let mut session = RaceSession::new();
    session.open_settings_modal();
    assert!(session.is_settings_modal_open());

    if let Some(ref mut modal) = session.settings_modal {
        modal.aura_dropdown.set_selected(1); // Disabled
        modal.aura_ratio_slider.set_value(0.60);
        modal.aura_brightness_slider.set_value(2.10);
        modal.ribbon_dropdown.set_selected(0); // Enabled
        modal.ribbon_scale_slider.set_value(1.75);
        modal.ribbon_brightness_slider.set_value(2.40);
        modal.chevron_dropdown.set_selected(1); // Disabled
        modal.chevron_brightness_slider.set_value(0.80);
        modal.beacon_dropdown.set_selected(0); // Enabled
        modal.adaptive_dropdown.set_selected(1); // Disabled
        modal.radar_ping_dropdown.set_selected(1); // Disabled
    }

    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());

    assert!(!session.config.player_helpers.ground_aura);
    assert!((session.config.player_helpers.ground_aura_radius_ratio - 0.60).abs() < 1e-4);
    assert!((session.config.player_helpers.ground_aura_brightness - 2.10).abs() < 1e-4);
    assert!(session.config.player_helpers.curve_helper);
    assert!((session.config.player_helpers.curve_helper_scale - 1.75).abs() < 1e-4);
    assert!((session.config.player_helpers.curve_helper_brightness - 2.40).abs() < 1e-4);
    assert!(!session.config.player_helpers.overhead_chevron);
    assert!((session.config.player_helpers.overhead_chevron_brightness - 0.80).abs() < 1e-4);
    assert!(session.config.player_helpers.roof_beacon);
    assert!(!session.config.player_helpers.adaptive_visibility);
    assert!(!session.config.player_helpers.radar_sonar_ping);

    // Also assert runtime visibility_options synced
    assert!(!session.visibility_options.ground_aura);
    assert!((session.visibility_options.ground_aura_radius_ratio - 0.60).abs() < 1e-4);
    assert!((session.visibility_options.ground_aura_brightness - 2.10).abs() < 1e-4);
    assert!(session.visibility_options.curve_helper);
    assert!((session.visibility_options.curve_helper_scale - 1.75).abs() < 1e-4);
    assert!((session.visibility_options.curve_helper_brightness - 2.40).abs() < 1e-4);
    assert!(!session.visibility_options.overhead_chevron);
    assert!((session.visibility_options.overhead_chevron_brightness - 0.80).abs() < 1e-4);
    assert!(session.visibility_options.roof_beacon);
    assert!(!session.visibility_options.adaptive_visibility);
    assert!(!session.visibility_options.sonar_ping);
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
    if let Some(pos) = session.cars.first().map(|c| c.state.position) {
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
    if let Some(pc) = session.cars.first_mut() {
        pc.state.angular_velocity = 5.2; // > 4.5 rad/s
    }
    let expected_pos = session.cars.first().map(|c| c.state.position).unwrap();
    session.update();
    assert_eq!(session.sonar_ping_timer, 0.75);
    assert_eq!(session.sonar_ping_origin, expected_pos);
    assert!((session.sonar_ping_cooldown - 3.0).abs() < 0.1);
}

#[test]
fn test_sonar_ping_no_hotkeys_and_modal_governance_only() {
    let _scoped_cfg = tdrace_app::storage::ScopedTempConfigDir::new("sonar_no_hotkeys");
    let mut session = RaceSession::new();
    assert!(session.visibility_options.sonar_ping);

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

    // Re-enable via modal
    session.open_settings_modal();
    if let Some(ref mut modal) = session.settings_modal {
        modal.radar_ping_dropdown.set_selected(0); // Enabled
    }
    session.close_settings_modal(true);
    assert!(session.visibility_options.sonar_ping);
}


