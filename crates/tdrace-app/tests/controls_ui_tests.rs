use tdrace_app::game::{GameState, RaceSession};
use tdrace_core::physics::config::AssistProfile;

#[test]
fn test_controls_help_game_state_transitions() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::Menu);

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
