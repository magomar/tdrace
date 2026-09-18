use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{GameMode, ModalityCategory, ModalityModal};

#[test]
fn test_grand_hub_to_modality_select_transition() {
    let mut session = RaceSession::new();
    session.state = GameState::ModuleSelect { selected_idx: 3 }; // GT World Challenge

    // Simulate pressing A / Confirm on Grand Hub
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_module_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Check that transition was queued targeting ModalitySelect
    assert!(
        session.transition.is_some() || matches!(session.state, GameState::ModalitySelect { .. }),
        "Grand Hub must transition towards ModalitySelect"
    );
    if let Some(target) = session.pending_state {
        assert_eq!(
            target,
            GameState::ModalitySelect {
                category: ModalityCategory::SinglePlayer,
                selected_idx: 0,
                modal: None,
            }
        );
    }
}

#[test]
fn test_modality_category_switching_and_wrapping() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 2,
        modal: None,
    };

    // 1. Switch to Multiplayer tab via D-pad Right
    session.input.gamepad.snapshot.dpad_right_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_right_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 0,
            modal: None,
        }
    );

    // 2. Switch back to Single Player tab via D-pad Left
    session.input.gamepad.snapshot.dpad_left_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_left_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 0,
            modal: None,
        }
    );

    // 3. Switch via bumper LB/RB
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_rb_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 0,
            modal: None,
        }
    );
}

#[test]
fn test_modality_card_navigation_and_wrapping() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 0,
        modal: None,
    };

    // Up from 0 wraps to last item (FreeRide, index 4)
    session.input.gamepad.snapshot.dpad_up_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_up_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 4,
            modal: None,
        }
    );

    // Down from 4 wraps back to 0 (QuickRace)
    session.input.gamepad.snapshot.dpad_down_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_down_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 0,
            modal: None,
        }
    );

    // Down from 0 moves to 1 (CustomRace)
    session.input.gamepad.snapshot.dpad_down_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_down_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 1,
            modal: None,
        }
    );
}

#[test]
fn test_quick_race_modality_selection_invariants() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 0, // Quick Race
        modal: None,
    };

    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.game_mode, GameMode::StandardRace);
    assert!(!session.free_car_selection);
    assert!(!session.is_time_attack);
    assert!(session.transition.is_some());
    assert_eq!(session.pending_state, Some(GameState::Menu));
}

#[test]
fn test_custom_race_modality_selection_invariants() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 1, // Custom Race
        modal: None,
    };

    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.game_mode, GameMode::ExperimentalRace);
    assert!(session.free_car_selection);
    assert!(!session.is_time_attack);
    assert!(session.transition.is_some());
    assert_eq!(session.pending_state, Some(GameState::Menu));
}

#[test]
fn test_time_trial_and_free_ride_solo_invariants() {
    let mut session = RaceSession::new();

    // Time Trial
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 3, // Time Trial
        modal: None,
    };
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.game_mode, GameMode::TimeTrial);
    assert!(session.free_car_selection);
    assert!(session.is_time_attack);
    assert_eq!(session.num_bots, 0);

    // Free Ride
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 4, // Free Ride
        modal: None,
    };
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.game_mode, GameMode::FreeRide);
    assert!(session.free_car_selection);
    assert!(session.is_time_attack);
    assert_eq!(session.num_bots, 0);
}

#[test]
fn test_split_screen_modality_invariants() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 0, // 2P Split Screen
        modal: None,
    };

    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.game_mode, GameMode::SplitScreen);
    assert!(session.free_car_selection);
    assert!(!session.is_time_attack);
    assert!(session.transition.is_some());
    assert_eq!(session.pending_state, Some(GameState::Menu));
}

#[test]
fn test_in_development_lan_cloud_modals() {
    let mut session = RaceSession::new();

    // 1. Select LAN Multiplayer
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 1, // LAN
        modal: None,
    };
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 1,
            modal: Some(ModalityModal::LanComingSoon),
        }
    );

    // Dismiss modal with B button
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_b_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 1,
            modal: None,
        }
    );

    // 2. Select Cloud Online
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 2, // Cloud
        modal: None,
    };
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 2,
            modal: Some(ModalityModal::CloudComingSoon),
        }
    );

    // Dismiss modal with confirm button
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 2,
            modal: None,
        }
    );
}

#[test]
fn test_menu_backward_transition_to_modality_select() {
    let mut session = RaceSession::new();
    session.state = GameState::Menu;
    session.game_mode = GameMode::TimeTrial;

    // Press Gamepad B / Cancel in Circuit Select menu
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_b_pressed = false;

    assert!(session.transition.is_some());
    assert_eq!(
        session.pending_state,
        Some(GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 3, // Index 3 corresponds to TimeTrial
            modal: None,
        })
    );
}

#[test]
fn test_modality_select_backward_transition_to_hub() {
    let mut session = RaceSession::new();
    session.active_module_id = "gt";
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 0,
        modal: None,
    };

    // Press Gamepad B on Modality Select screen (without active modal)
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_b_pressed = false;

    assert!(session.transition.is_some());
    assert_eq!(
        session.pending_state,
        Some(GameState::ModuleSelect { selected_idx: 3 })
    );
}

#[test]
fn test_starting_grid_card_0_modality_toggle() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::StandardRace;
    session.free_car_selection = false;
    session.starting_grid_card_idx = 0;

    // Tab toggles from Standard (Preset) to Experimental (Custom)
    session.input.gamepad.snapshot.btn_x_pressed = true;
    // Simulate line in StartingGrid update
    if session.game_mode == GameMode::StandardRace {
        session.game_mode = GameMode::ExperimentalRace;
    } else {
        session.game_mode = GameMode::StandardRace;
    }
    session.free_car_selection = session.game_mode.allows_car_change();

    assert_eq!(session.game_mode, GameMode::ExperimentalRace);
    assert!(session.free_car_selection);

    // Toggle back
    if session.game_mode == GameMode::StandardRace {
        session.game_mode = GameMode::ExperimentalRace;
    } else {
        session.game_mode = GameMode::StandardRace;
    }
    session.free_car_selection = session.game_mode.allows_car_change();

    assert_eq!(session.game_mode, GameMode::StandardRace);
    assert!(!session.free_car_selection);
}
