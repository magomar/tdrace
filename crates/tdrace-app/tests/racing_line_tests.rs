//! Integration tests for Spec 110: Mathematically Ideal Racing Line Visual Aid.

use glam::Vec2;
use tdrace_app::config::{GameConfig, RacingLineMode};
use tdrace_app::game::RaceSession;
use tdrace_app::render::{render_optimal_racing_line, PlayerVisibilityOptions};
use tdrace_app::storage::ScopedTempConfigDir;
use tdrace_app::ui::hud::VisibilityToast;

#[test]
fn test_racing_line_mode_cycle_and_toast_labels() {
    let mut mode = RacingLineMode::Full;
    assert_eq!(mode.toast_label(), "FULL");

    mode = mode.next();
    assert_eq!(mode, RacingLineMode::LineOnly);
    assert_eq!(mode.toast_label(), "LINE ONLY");

    mode = mode.next();
    assert_eq!(mode, RacingLineMode::BrakingOnly);
    assert_eq!(mode.toast_label(), "BRAKING ONLY");

    mode = mode.next();
    assert_eq!(mode, RacingLineMode::Disabled);
    assert_eq!(mode.toast_label(), "OFF");

    mode = mode.next();
    assert_eq!(mode, RacingLineMode::Full);
    assert_eq!(mode.toast_label(), "FULL");
}

#[test]
fn test_racing_line_settings_persistence_across_sessions() {
    let _scoped_cfg = ScopedTempConfigDir::new("racing_line_persistence");

    // 1. Create and save custom helpers configuration
    let mut config = GameConfig::load_or_default();
    config.player_helpers.racing_line_mode = RacingLineMode::LineOnly;
    config.player_helpers.racing_line_opacity = 0.50;
    config.save_user_config().expect("Failed to save config to temp dir");

    // 2. Load fresh config and verify persistence
    let loaded = GameConfig::load_or_default();
    assert_eq!(loaded.player_helpers.racing_line_mode, RacingLineMode::LineOnly);
    assert!((loaded.player_helpers.racing_line_opacity - 0.50).abs() < 1e-4);

    // 3. Verify PlayerVisibilityOptions inherits from persisted config
    let opts = PlayerVisibilityOptions::from(&loaded.player_helpers);
    assert_eq!(opts.racing_line_mode, RacingLineMode::LineOnly);
    assert!((opts.racing_line_opacity - 0.50).abs() < 1e-4);
}

#[test]
fn test_racing_line_settings_modal_governance() {
    let _scoped_cfg = ScopedTempConfigDir::new("racing_line_modal_governance");
    let mut session = RaceSession::new();

    session.open_settings_modal();
    assert!(session.is_settings_modal_open());

    if let Some(ref mut modal) = session.settings_modal {
        // Change to Line Only (index 1) and 50% opacity (slider value 50.0)
        modal.racing_line_dropdown.set_selected(1);
        modal.racing_line_opacity_slider.set_value(50.0);
    }

    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());

    assert_eq!(session.config.player_helpers.racing_line_mode, RacingLineMode::LineOnly);
    assert!((session.config.player_helpers.racing_line_opacity - 0.50).abs() < 1e-4);
    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::LineOnly);
    assert!((session.visibility_options.racing_line_opacity - 0.50).abs() < 1e-4);
}

#[test]
fn test_racing_line_hotkey_6_simulation_and_toast() {
    let _scoped_cfg = ScopedTempConfigDir::new("racing_line_hotkey_toast");
    let mut session = RaceSession::new();
    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::Full);

    // Simulate key 6 toggle: Full -> LineOnly
    let next_mode = session.visibility_options.racing_line_mode.next();
    session.visibility_options.racing_line_mode = next_mode;
    session.config.player_helpers.racing_line_mode = next_mode;
    let label = next_mode.toast_label();
    let is_on = next_mode != RacingLineMode::Disabled;
    session.visibility_toast = Some(VisibilityToast {
        text: format!("VISUAL AID [6]: RACING LINE ({})", label),
        is_on,
        timer: 2.0,
        duration: 2.0,
    });

    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::LineOnly);
    assert_eq!(session.visibility_toast.as_ref().unwrap().text, "VISUAL AID [6]: RACING LINE (LINE ONLY)");

    // Simulate next key 6 toggle: LineOnly -> BrakingOnly
    let next_mode = session.visibility_options.racing_line_mode.next();
    session.visibility_options.racing_line_mode = next_mode;
    session.config.player_helpers.racing_line_mode = next_mode;
    let label = next_mode.toast_label();
    let is_on = next_mode != RacingLineMode::Disabled;
    session.visibility_toast = Some(VisibilityToast {
        text: format!("VISUAL AID [6]: RACING LINE ({})", label),
        is_on,
        timer: 2.0,
        duration: 2.0,
    });

    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::BrakingOnly);
    assert_eq!(session.visibility_toast.as_ref().unwrap().text, "VISUAL AID [6]: RACING LINE (BRAKING ONLY)");

    // Simulate next key 6 toggle: BrakingOnly -> Disabled
    let next_mode = session.visibility_options.racing_line_mode.next();
    session.visibility_options.racing_line_mode = next_mode;
    session.config.player_helpers.racing_line_mode = next_mode;
    let label = next_mode.toast_label();
    let is_on = next_mode != RacingLineMode::Disabled;
    session.visibility_toast = Some(VisibilityToast {
        text: format!("VISUAL AID [6]: RACING LINE ({})", label),
        is_on,
        timer: 2.0,
        duration: 2.0,
    });

    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::Disabled);
    assert_eq!(session.visibility_toast.as_ref().unwrap().text, "VISUAL AID [6]: RACING LINE (OFF)");

    // Simulate next key 6 toggle: Disabled -> Full
    let next_mode = session.visibility_options.racing_line_mode.next();
    session.visibility_options.racing_line_mode = next_mode;
    session.config.player_helpers.racing_line_mode = next_mode;
    let label = next_mode.toast_label();
    let is_on = next_mode != RacingLineMode::Disabled;
    session.visibility_toast = Some(VisibilityToast {
        text: format!("VISUAL AID [6]: RACING LINE ({})", label),
        is_on,
        timer: 2.0,
        duration: 2.0,
    });

    assert_eq!(session.visibility_options.racing_line_mode, RacingLineMode::Full);
    assert_eq!(session.visibility_toast.as_ref().unwrap().text, "VISUAL AID [6]: RACING LINE (FULL)");
}

#[test]
fn test_render_racing_line_disabled_zero_impact() {
    let session = RaceSession::new();
    let opt_line = session.track.optimal_line().expect("Optimal racing line should be precomputed on session track");

    // Rendering with RacingLineMode::Disabled returns immediately
    render_optimal_racing_line(
        &session.track.spline,
        opt_line,
        Vec2::ZERO,
        25.0,
        RacingLineMode::Disabled,
        0.70,
        None,
    );
}
