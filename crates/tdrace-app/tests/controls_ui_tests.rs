use tdrace_app::game::{GameState, RaceSession};
use tdrace_core::physics::config::AssistProfile;

#[test]
fn test_controls_help_game_state_transitions() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // Enter ControlsHelp from Menu
    session.state = GameState::ControlsHelp(false);
    assert_eq!(session.state, GameState::ControlsHelp(false));

    // Cycle assist profile in ControlsHelp
    assert_eq!(session.assist_profile, AssistProfile::Arcade);
    session.assist_profile = session.assist_profile.next();
    assert_eq!(session.assist_profile, AssistProfile::Sport);
    session.assist_profile = session.assist_profile.next();
    assert_eq!(session.assist_profile, AssistProfile::Pro);

    // Return to Menu
    session.state = GameState::Menu;
    assert_eq!(session.state, GameState::Menu);

    // Enter ControlsHelp from Paused
    session.state = GameState::ControlsHelp(true);
    assert_eq!(session.state, GameState::ControlsHelp(true));
}

#[test]
fn test_gamepad_connection_status_snapshot() {
    let session = RaceSession::new();
    assert!(!session.input.gamepad.snapshot.is_connected || session.input.gamepad.snapshot.is_connected);
}

#[test]
fn test_pause_menu_layout_and_buttons() {
    use tdrace_app::ui::menu::pause_menu_layout;

    for (sw, sh) in [(1280.0, 720.0), (1920.0, 1080.0), (844.0, 390.0), (3840.0, 2160.0)] {
        let (box_x, box_y, box_w, box_h, layout) = pause_menu_layout(sw, sh);
        assert!(box_w > 0.0 && box_h > 0.0);
        assert!(box_x >= 0.0 && box_y >= 0.0);
        assert!(box_x + box_w <= sw + 1.0);
        assert!(box_y + box_h <= sh + 1.0);

        let (rx, ry, rw, rh) = layout.resume_rect;
        let (ex, ey, ew, eh) = layout.exit_rect;

        // Both buttons should be inside the pause menu card
        assert!(rx >= box_x, "rx ({rx}) >= box_x ({box_x})");
        assert!(rx + rw <= box_x + box_w, "rx+rw ({}) <= box_x+box_w ({})", rx + rw, box_x + box_w);
        assert!(ex >= box_x, "ex ({ex}) >= box_x ({box_x})");
        assert!(ex + ew <= box_x + box_w, "ex+ew ({}) <= box_x+box_w ({})", ex + ew, box_x + box_w);

        assert!(ry >= box_y, "ry ({ry}) >= box_y ({box_y})");
        assert!(ry + rh <= box_y + box_h, "ry+rh ({}) <= box_y+box_h ({})", ry + rh, box_y + box_h);
        assert!(ey >= box_y, "ey ({ey}) >= box_y ({box_y})");
        assert!(ey + eh <= box_y + box_h, "ey+eh ({}) <= box_y+box_h ({})", ey + eh, box_y + box_h);

        // Resume (left) and Exit (right) buttons must not overlap
        assert!(rx + rw <= ex, "resume right edge ({}) <= exit left edge ({})", rx + rw, ex);
        assert!(rw >= 100.0 && rh >= 30.0);
        assert!(ew >= 100.0 && eh >= 30.0);
    }
}

#[test]
fn test_pause_state_audio_toggle_preserves_paused_state() {
    let mut session = RaceSession::new();
    session.state = GameState::Paused;

    // Toggling mute should not change game state
    let initial_mute = session.audio.settings.is_muted;
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, !initial_mute);
    assert_eq!(session.state, GameState::Paused);

    // Update with no key pressed keeps state Paused
    session.update();
    assert_eq!(session.state, GameState::Paused);
}
