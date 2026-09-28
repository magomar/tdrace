//! Screen-to-screen navigation: Escape and the global shortcuts must return the player to the
//! screen they came from. These tests drive `RaceSession::update()` with injected key presses,
//! so the global hotkey layer that runs before each screen's own input is covered too.

use macroquad::input::KeyCode;
use tdrace_app::game::{
    inject_key_presses_for_tests, DriverCardsOrigin, GameState, GarageOrigin, GlobalHotkey, MenuOrigin, ProfileOrigin,
    RaceSession,
};
use tdrace_app::ui::menu::{GameMode, ModalityCategory};
use tdrace_app::ui::track_manager_ui::{TrackManagerModal, TrackManagerTab};

/// Runs one frame with `keys` held as "pressed this frame".
fn press(session: &mut RaceSession, keys: &[KeyCode]) {
    inject_key_presses_for_tests(keys);
    session.update();
    inject_key_presses_for_tests(&[]);
}

/// Runs frames until a running screen transition has swapped in its target state.
fn settle(session: &mut RaceSession) {
    for _ in 0..240 {
        if session.transition.is_none() && session.pending_state.is_none() {
            return;
        }
        session.update();
    }
    panic!("screen transition did not finish");
}

#[test]
fn global_hotkeys_are_limited_to_screens_that_do_not_use_the_key() {
    // D steers right in a WASD race and pages the dossier: only the pause menu opens Driver Cards.
    assert!(GameState::Paused.allows_global_hotkey(GlobalHotkey::DriverCards));
    for state in [GameState::Racing, GameState::Countdown(2.0), GameState::StartingGrid, GameState::Menu] {
        assert!(!state.allows_global_hotkey(GlobalHotkey::DriverCards), "{state:?}");
    }
    assert!(!GameState::DriverCards(DriverCardsOrigin::Paused).allows_global_hotkey(GlobalHotkey::DriverCards));

    // S is "down" in menus; [ and ] edit the grid. Sound keys only act around the race itself.
    assert!(GameState::Racing.allows_global_hotkey(GlobalHotkey::SoundKeys));
    assert!(!GameState::ModuleSelect { selected_idx: 0 }.allows_global_hotkey(GlobalHotkey::SoundKeys));
    assert!(!GameState::StartingGrid.allows_global_hotkey(GlobalHotkey::SoundKeys));

    // Controls help closes itself on K; the championship editor never opens mid-race.
    assert!(!GameState::ControlsHelp(false).allows_global_hotkey(GlobalHotkey::Controls));
    assert!(!GameState::Racing.allows_global_hotkey(GlobalHotkey::ChampionshipEditor));
    assert!(!GameState::TrackEditor.allows_global_hotkey(GlobalHotkey::ChampionshipEditor));

    // Gamepad Select means "back" in menus, so assists only cycle around the race.
    assert!(!GameState::Menu.allows_global_hotkey(GlobalHotkey::AssistCycle));
    assert!(GameState::Racing.allows_global_hotkey(GlobalHotkey::AssistCycle));
}

#[test]
fn wasd_steer_right_does_not_open_driver_cards_mid_race() {
    let mut session = RaceSession::new();
    session.init_race();
    session.input.input_map = cabinet::input::InputMap::wasd_racing();
    session.state = GameState::Racing;

    press(&mut session, &[KeyCode::D]);

    assert_eq!(session.state, GameState::Racing);
}

#[test]
fn d_in_driver_cards_pages_forward_and_keeps_the_pause_origin() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::DriverCards(DriverCardsOrigin::Paused);
    session.driver_cards_idx = 0;

    press(&mut session, &[KeyCode::D]);
    assert_eq!(session.state, GameState::DriverCards(DriverCardsOrigin::Paused));
    assert_eq!(session.driver_cards_idx, 1);

    press(&mut session, &[KeyCode::Escape]);
    assert_eq!(session.state, GameState::Paused);
}

#[test]
fn controls_help_returns_to_the_screen_that_opened_it() {
    let mut session = RaceSession::new();
    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);

    press(&mut session, &[KeyCode::K]);
    assert!(matches!(session.state, GameState::ControlsHelp(_)));

    press(&mut session, &[KeyCode::Escape]);
    assert_eq!(session.state, GameState::StartingGrid, "Esc must not drop the player on the track menu");
}

#[test]
fn k_twice_from_the_pause_menu_goes_back_to_the_pause_menu() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Racing;
    session.pause_race();

    press(&mut session, &[KeyCode::K]);
    assert_eq!(session.state, GameState::ControlsHelp(true));

    press(&mut session, &[KeyCode::K]);
    assert_eq!(session.state, GameState::Paused, "second K closes the guide instead of reopening it");
}

#[test]
fn every_pause_opens_with_resume_focused() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Racing;
    session.pause_race();
    session.pause_nav.set_focus(1, 0);
    session.pause_selected_btn = 1;
    session.resume_race();

    session.pause_race();

    assert_eq!(session.pause_nav.focused_col, 0);
    assert_eq!(session.pause_selected_btn, 0);
}

#[test]
fn pausing_during_the_countdown_resumes_the_countdown_not_the_race() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Countdown(2.5);

    press(&mut session, &[KeyCode::X]);
    assert_eq!(session.state, GameState::Paused);
    session.settings_modal = None;

    session.resume_race();
    assert!(matches!(session.state, GameState::Countdown(t) if t > 2.0), "got {:?}", session.state);
}

#[test]
fn circuit_menu_opened_from_the_grid_forgets_the_grid_once_left() {
    let mut session = RaceSession::new();
    session.init_race();
    session.open_circuit_selector_from_starting_grid();
    assert_eq!(session.state, GameState::Menu);

    press(&mut session, &[KeyCode::Escape]);
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.menu_origin, MenuOrigin::ModalitySelect);
}

#[test]
fn new_profile_from_the_hub_returns_to_the_hub() {
    let mut session = RaceSession::new();
    session.profile_origin = ProfileOrigin::StartingGrid; // stale value from an earlier visit
    session.state = GameState::ModuleSelect { selected_idx: 0 };

    press(&mut session, &[KeyCode::N]);

    assert!(matches!(session.state, GameState::ProfileCreate { .. }));
    assert_eq!(session.profile_origin, ProfileOrigin::ModuleSelect);
}

#[test]
fn quit_prompt_blocks_global_shortcuts() {
    let mut session = RaceSession::new();
    session.state = GameState::ModuleSelect { selected_idx: 0 };
    session.show_exit_confirm = true;

    press(&mut session, &[KeyCode::K]);

    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert!(session.settings_modal.is_none());
}

#[test]
fn s_moves_down_in_the_hub_without_muting_sound() {
    let mut session = RaceSession::new();
    session.state = GameState::ModuleSelect { selected_idx: 0 };
    let sfx_muted = session.audio.settings.is_sfx_muted;

    press(&mut session, &[KeyCode::S]);

    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 1 });
    assert_eq!(session.audio.settings.is_sfx_muted, sfx_muted);
}

#[test]
fn race_modality_screen_restores_the_tab_left_by_a_shortcut() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect { category: ModalityCategory::SinglePlayer, selected_idx: 1, modal: None };

    press(&mut session, &[KeyCode::G]);
    assert_eq!(session.state, GameState::Garage(GarageOrigin::ModalitySelect));

    press(&mut session, &[KeyCode::Escape]);
    assert_eq!(
        session.state,
        GameState::ModalitySelect { category: ModalityCategory::SinglePlayer, selected_idx: 1, modal: None }
    );
}

#[test]
fn career_back_chain_goes_hub_then_career_select_then_career_card() {
    let mut session = RaceSession::new();
    session.modality_cursor = (ModalityCategory::SinglePlayer, 2); // Career Mode card
    session.switch_to_gt();
    session.game_mode = GameMode::Career;
    session.state = GameState::CareerHub {
        selected_tier: 1,
        selected_slot: 0,
        calendar_tracks: tdrace_app::ui::gt_default_calendar(1),
        showing_standings: false,
    };

    press(&mut session, &[KeyCode::Escape]);
    settle(&mut session);
    let GameState::CareerSelect { selected_idx } = session.state.clone() else {
        panic!("Career Hub Esc must go to Career Select, got {:?}", session.state);
    };
    assert_eq!(session.career_select_state_for("gt"), GameState::CareerSelect { selected_idx });

    press(&mut session, &[KeyCode::Escape]);
    assert_eq!(
        session.state,
        GameState::ModalitySelect { category: ModalityCategory::SinglePlayer, selected_idx: 2, modal: None }
    );
}

#[test]
fn track_manager_exits_with_the_same_keys_as_its_dialogs() {
    let mut session = RaceSession::new();
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Main,
        module_filter: Default::default(),
        selected_idx: 0,
        modal: TrackManagerModal::None,
    };

    press(&mut session, &[KeyCode::Escape]);

    assert_eq!(session.state, GameState::Menu);
}

#[test]
fn picking_a_circuit_drops_a_quick_championship_left_open() {
    let mut session = RaceSession::new();
    session.start_gt_championship();
    // Back out to the classic circuit list, where every circuit is open to a new profile.
    session.switch_to_classic();
    assert!(session.championship_session.is_some());
    session.game_mode = GameMode::StandardRace;
    session.state = GameState::Menu;

    press(&mut session, &[KeyCode::Enter]);

    assert!(session.championship_session.is_none(), "a single race must not count as a championship round");
    assert_eq!(session.state, GameState::StartingGrid);
}

#[test]
fn leaving_a_lan_race_closes_the_network_session() {
    let mut session = RaceSession::new();
    session.init_race();
    session.is_lan_multiplayer = true;

    let target = session.race_exit_target();

    assert_eq!(target, GameState::LanHub { selected_idx: 0 });
    assert!(!session.is_lan_multiplayer);
}
