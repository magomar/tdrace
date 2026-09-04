use tdrace_app::render::{compute_adaptive_alpha, PlayerVisibilityOptions};
use tdrace_app::ui::hud::VisibilityToast;
use tdrace_app::game::RaceSession;

#[test]
fn test_player_visibility_options_defaults() {
    let opts = PlayerVisibilityOptions::default();
    assert!(opts.overhead_chevron, "Option 1 (Overhead Chevron) must be enabled by default");
    assert!(opts.ground_aura, "Option 2 (Ground Aura) must be enabled by default");
    assert!(opts.adaptive_visibility, "Option 3 (Adaptive Visibility) must be enabled by default");
    assert!(opts.roof_beacon, "Option 4 (Roof Beacon) must be enabled by default");
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

    // Turn back on
    opts.overhead_chevron = true;
    assert!(opts.overhead_chevron);
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
    let session = RaceSession::new();
    assert!(session.visibility_options.overhead_chevron);
    assert!(session.visibility_options.ground_aura);
    assert!(session.visibility_options.adaptive_visibility);
    assert!(session.visibility_options.roof_beacon);
    assert!(session.visibility_toast.is_none());
}
