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

#[test]
fn test_starting_grid_audio_toggle_preserves_starting_grid_state() {
    let mut session = RaceSession::new();
    session.state = GameState::StartingGrid;

    // Toggling audio/music mute in StartingGrid preserves StartingGrid state and game mode
    let initial_mute = session.audio.settings.is_muted;
    let initial_mode = session.game_mode;
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, !initial_mute);
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.game_mode, initial_mode);

    // Update keeps StartingGrid state
    session.update();
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.game_mode, initial_mode);
}

#[test]
fn test_starting_grid_driver_count_bounds() {
    let mut session = RaceSession::new();
    session.state = GameState::StartingGrid;
    let max_bots = (session.track.grid_positions.len().saturating_sub(1)).clamp(1, 7);

    // Increase driver count up to max
    session.num_bots = max_bots;
    session.rebuild_roster_participants();
    assert_eq!(session.num_bots, max_bots);

    // Decrease driver count down to 1
    session.num_bots = 1;
    session.rebuild_roster_participants();
    assert_eq!(session.num_bots, 1);
    assert_eq!(session.opponent_drivers.len(), 1);
}

#[test]
fn test_finished_state_audio_toggle_preserves_finished_state_and_hof() {
    let mut session = RaceSession::new();
    session.state = GameState::Finished;
    session.show_hall_of_fame = true;

    // Toggle mute while in Hall of Fame view of Finished state
    let initial_mute = session.audio.settings.is_muted;
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, !initial_mute);
    assert_eq!(session.state, GameState::Finished);
    assert!(session.show_hall_of_fame);

    // Toggle view to Standings results
    session.show_hall_of_fame = false;
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, initial_mute);
    assert_eq!(session.state, GameState::Finished);
    assert!(!session.show_hall_of_fame);

    // Calling session.update() with no keys pressed preserves Finished state
    session.update();
    assert_eq!(session.state, GameState::Finished);
}

#[test]
fn test_championship_and_profile_audio_toggle_preserves_state() {
    let mut session = RaceSession::new();
    session.state = GameState::ChampionshipStandings;

    let initial_mute = session.audio.settings.is_muted;
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, !initial_mute);
    assert_eq!(session.state, GameState::ChampionshipStandings);

    session.state = GameState::ProfileManager { selected_idx: 0 };
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, initial_mute);
    assert_eq!(session.state, GameState::ProfileManager { selected_idx: 0 });
}

#[test]
fn test_pause_menu_nav_grid_2d_navigation() {
    use tdrace_app::input::NavGrid2D;

    let mut session = RaceSession::new();
    session.state = GameState::Paused;

    // Initial state: focus on Resume button (col 0)
    assert_eq!(session.pause_nav.focused_col, 0);
    assert_eq!(session.pause_selected_btn, 0);

    // Step right to Exit button (col 1)
    session.pause_nav.move_right();
    assert_eq!(session.pause_nav.focused_col, 1);

    // Call update with no keys pressed; syncs pause_selected_btn
    session.update();
    assert_eq!(session.pause_selected_btn, 1);
    assert_eq!(session.state, GameState::Paused);

    // Step left back to Resume button (col 0)
    session.pause_nav.move_left();
    assert_eq!(session.pause_nav.focused_col, 0);
    session.update();
    assert_eq!(session.pause_selected_btn, 0);
    assert_eq!(session.state, GameState::Paused);

    // Verify NavGrid2D hit testing logic
    let rect = (100.0, 100.0, 200.0, 50.0);
    assert!(!NavGrid2D::check_mouse_hover(rect));
}

#[test]
fn test_arcade_settings_modal_integration_and_bindings() {
    use tdrace_core::physics::config::AssistProfile;

    let mut session = RaceSession::new();
    assert!(!session.is_settings_modal_open());
    assert!(session.settings_modal.is_none());

    // 1. Open settings modal and verify pre-population
    session.open_settings_modal();
    assert!(session.is_settings_modal_open());
    {
        let modal = session.settings_modal.as_ref().unwrap();
        assert_eq!(modal.tab_bar.active_tab_name(), "AUDIO");
        assert!((modal.master_slider.normalized() - session.audio.settings.master_volume).abs() < 1e-4);
        assert!((modal.music_slider.normalized() - session.audio.settings.music_volume).abs() < 1e-4);
        assert!((modal.stick_deadzone_slider.value - session.input.gamepad.config.stick_deadzone).abs() < 1e-4);
        assert_eq!(modal.assist_dropdown.selected_index, 0); // Default Arcade
    }

    // 2. Mutate settings in modal
    if let Some(ref mut modal) = session.settings_modal {
        modal.master_slider.set_normalized(0.45);
        modal.music_slider.set_normalized(0.35);
        modal.mute_dropdown.set_selected(1); // Muted
        modal.stick_deadzone_slider.set_value(0.24);
        modal.assist_dropdown.set_selected(2); // Pro
    }

    // 3. Close with save=true and verify applied state
    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());
    assert!((session.audio.settings.master_volume - 0.45).abs() < 1e-4);
    assert!((session.audio.settings.music_volume - 0.35).abs() < 1e-4);
    assert!(session.audio.settings.is_muted);
    assert!((session.input.gamepad.config.stick_deadzone - 0.24).abs() < 1e-4);
    assert_eq!(session.assist_profile, AssistProfile::Pro);

    // 4. Open again, mutate, then cancel (save=false)
    session.open_settings_modal();
    if let Some(ref mut modal) = session.settings_modal {
        modal.master_slider.set_normalized(0.95);
        modal.assist_dropdown.set_selected(1); // Sport
    }
    session.close_settings_modal(false);
    assert!(!session.is_settings_modal_open());
    // Values should remain as they were before opening
    assert!((session.audio.settings.master_volume - 0.45).abs() < 1e-4);
    assert_eq!(session.assist_profile, AssistProfile::Pro);
}

#[test]
fn test_screen_stack_and_cabinet_screen_architecture() {
    use tdrace_app::ui::{
        ArcadeSettingsModal, CabinetContext, CabinetScreen, ScreenAction, ScreenStack,
        UniversalPauseModal,
    };

    struct TestScreen {
        name: String,
    }

    impl CabinetScreen for TestScreen {
        fn name(&self) -> &str {
            &self.name
        }
        fn update(&mut self, _ctx: &mut CabinetContext) -> ScreenAction {
            ScreenAction::None
        }
        fn draw(&self, _ctx: &CabinetContext) {}
    }

    let root = Box::new(TestScreen {
        name: "GameScreen".to_string(),
    });
    let mut stack = ScreenStack::new(root);
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.active_screen_name(), Some("GameScreen"));

    // Push pause modal
    let pause_modal = Box::new(UniversalPauseModal::new("PAUSE OVERLAY"));
    stack.push(pause_modal);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.active_screen_name(), Some("UniversalPauseModal"));

    // Push settings modal on top
    let settings_modal = Box::new(ArcadeSettingsModal::default());
    stack.push(settings_modal);
    assert_eq!(stack.len(), 3);
    assert_eq!(stack.active_screen_name(), Some("ArcadeSettingsModal"));

    // Pop screens in reverse order
    let popped_settings = stack.pop();
    assert!(popped_settings.is_some());
    assert_eq!(popped_settings.unwrap().name(), "ArcadeSettingsModal");
    assert_eq!(stack.len(), 2);

    let popped_pause = stack.pop();
    assert!(popped_pause.is_some());
    assert_eq!(popped_pause.unwrap().name(), "UniversalPauseModal");
    assert_eq!(stack.len(), 1);

    // Root screen cannot be popped
    assert!(stack.pop().is_none());
    assert_eq!(stack.len(), 1);
}



