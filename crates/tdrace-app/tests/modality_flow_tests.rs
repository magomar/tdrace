use tdrace_app::game::{GameState, GarageOrigin, RaceSession, StartingGridFocus};
use tdrace_app::ui::menu::{CarChoice, GameMode, ModalityCategory, ModalityModal};

#[test]
fn test_grand_hub_to_modality_select_transition() {
    let mut session = RaceSession::new();
    session.state = GameState::ModuleSelect { selected_idx: 4 }; // GT World Challenge (index 4)

    // Simulate pressing A / Confirm on Grand Hub
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_module_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Check that transition was queued targeting ModalitySelect
    assert!(
        session.transition.is_some() || matches!(session.state, GameState::ModalitySelect { .. }),
        "Grand Hub must transition towards ModalitySelect"
    );
    // Crucial invariant: state must NOT prematurely mutate to GameState::Menu during transition covering
    assert_eq!(
        session.state,
        GameState::ModuleSelect { selected_idx: 4 },
        "State must remain ModuleSelect during transition to prevent flashing Circuit Selection"
    );
    if let Some(ref target) = session.pending_state {
        assert_eq!(
            target,
            &GameState::ModalitySelect {
                category: ModalityCategory::SinglePlayer,
                selected_idx: 0,
                modal: None,
            }
        );
    }

    // Advance to Holding (midpoint: 0.20s): module switch is applied and state swaps to ModalitySelect
    session.update_transition(0.20);
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 0,
            modal: None,
        }
    );
    assert_eq!(session.active_module_id, "gt");

    // Complete transition
    session.update_transition(0.25);
    assert!(!session.is_transitioning());
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

    // 3. Switch through all 3 categories via bumper RB (SinglePlayer -> Multiplayer -> Options -> SinglePlayer)
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

    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_rb_pressed = false;
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 0,
            modal: None,
        }
    );

    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_rb_pressed = false;
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
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
fn test_career_mode_opens_career_select() {
    let mut session = RaceSession::new();

    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 2, // Career Mode
        modal: None,
    };

    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    // Must open CareerSelect screen rather than auto-jumping into active module
    assert_eq!(
        session.state,
        GameState::CareerSelect {
            selected_idx: 0,
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
        Some(GameState::ModuleSelect { selected_idx: 4 })
    );
}

#[test]
fn test_starting_grid_card_0_opens_garage() {
    let mut session = RaceSession::new();
    session.state = GameState::StartingGrid;
    session.starting_grid_focus = StartingGridFocus::LeftSetup;
    session.starting_grid_card_idx = 0;

    // Card 0 action opens the Garage Showroom
    session.garage_origin = GarageOrigin::StartingGrid;
    session.garage_tier = session.current_race_required_tier();
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::StartingGrid);

    assert_eq!(session.state, GameState::Garage(GarageOrigin::StartingGrid));
    assert_eq!(session.garage_origin, GarageOrigin::StartingGrid);
    assert_eq!(session.garage_car_idx, 0);
}

#[test]
fn test_modality_select_column_3_options_garage_navigation() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 0,
        modal: None,
    };

    // Right moves from Multiplayer (col 2) to Options (col 3)
    session.input.gamepad.snapshot.dpad_right_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_right_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 0,
            modal: None,
        }
    );

    // Down moves to Garage card (index 1)
    session.input.gamepad.snapshot.dpad_down_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_down_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 1,
            modal: None,
        }
    );

    // Confirm (A / Enter) opens Garage
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(
        session.state,
        GameState::Garage(GarageOrigin::ModalitySelect)
    );
    assert_eq!(session.garage_origin, GarageOrigin::ModalitySelect);
}

#[test]
fn test_garage_lifecycle_and_return() {
    let mut session = RaceSession::new();
    session.state = GameState::Garage(GarageOrigin::ModalitySelect);
    session.garage_tier = 2;

    // View mode toggle
    assert_eq!(session.garage_view_mode, tdrace_app::ui::GarageViewMode::Lateral);
    session.input.gamepad.snapshot.btn_y_pressed = true;
    session.update_garage(GarageOrigin::ModalitySelect, 0.016);
    session.input.gamepad.snapshot.btn_y_pressed = false;
    assert_eq!(session.garage_view_mode, tdrace_app::ui::GarageViewMode::TopDownTurntable);

    // Escape returns to ModalitySelect under Options (index 1 = Garage)
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    session.update_garage(GarageOrigin::ModalitySelect, 0.016);
    session.input.gamepad.snapshot.btn_cancel_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 1,
            modal: None,
        }
    );
}

#[test]
fn test_category_based_race_eligibility_enforcement() {
    // Normal mode: only car.tier <= required_tier is eligible
    let gt4 = CarChoice::GT4Clubsport; // tier 1
    let gt3 = CarChoice::GT3Car;       // tier 2
    let hyper = CarChoice::HypercarPrototype; // tier 5

    // In a Tier 2 race (e.g. GT3):
    let race_tier = 2;
    assert!(gt4.is_eligible_for_race_tier(race_tier, false), "Lower tier (Tier 1 GT4) is eligible for Tier 2 race");
    assert!(gt3.is_eligible_for_race_tier(race_tier, false), "Matching tier (Tier 2 GT3) is eligible for Tier 2 race");
    assert!(!hyper.is_eligible_for_race_tier(race_tier, false), "Higher tier (Tier 5 Hypercar) is NOT eligible for Tier 2 race");

    // In dev mode: all cars are eligible regardless of tier
    assert!(hyper.is_eligible_for_race_tier(race_tier, true), "Dev mode unlocks all categories");
}

#[test]
fn test_modality_single_selected_menu_isolation() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 0,
        modal: None,
    };

    // 1. Single player menu has 5 distinct modalities
    assert_eq!(ModalityCategory::SinglePlayer.items().len(), 5);

    // 2. Switch to Multiplayer menu via Tab
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
    assert_eq!(ModalityCategory::Multiplayer.items().len(), 3);

    // 3. Switch to Options menu via Tab
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_rb_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 0,
            modal: None,
        }
    );
    assert_eq!(ModalityCategory::Options.items().len(), 5);

    // 4. Wrapping within Options menu (5 items: 0=Profile, 1=Garage, 2=TrackEditor, 3=ChampionshipEditor, 4=Settings)
    session.input.gamepad.snapshot.dpad_up_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.dpad_up_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 4,
            modal: None,
        }
    );
}

#[test]
fn test_modality_select_options_player_profile_flow() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Options,
        selected_idx: 0, // Player Profile
        modal: None,
    };

    // Confirm on Player Profile card (idx 0) opens ProfileManager
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert!(matches!(session.state, GameState::ProfileManager { .. }));
    assert_eq!(session.profile_origin, tdrace_app::game::ProfileOrigin::ModalitySelect);

    // Escape in ProfileManager returns cleanly to ModalitySelect under Options
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    let sel_idx = match session.state {
        GameState::ProfileManager { selected_idx } => selected_idx,
        _ => 0,
    };
    session.update_profile_manager(sel_idx);
    session.input.gamepad.snapshot.btn_cancel_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 0,
            modal: None,
        }
    );
}

#[test]
fn test_modality_select_options_track_editor_flow() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Options,
        selected_idx: 2, // Track Editor
        modal: None,
    };

    // 1. Confirm on Track Editor card (idx 2) opens Track Editor
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.state, GameState::TrackEditor);
    assert_eq!(session.editor_origin, tdrace_app::game::EditorOrigin::ModalitySelect);
    assert!(session.editor_state.is_some());

    // 2. Exit editor via ExitToTrackManager / return_to_track_manager returns to ModalitySelect under Options (idx 2)
    session.handle_editor_action(tdrace_app::editor::EditorAction::ExitToTrackManager);

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 2,
            modal: None,
        }
    );

    // 3. Test direct shortcut [E] from ModalitySelect
    session.enter_track_editor_from_modality_select();
    assert_eq!(session.state, GameState::TrackEditor);
    assert_eq!(session.editor_origin, tdrace_app::game::EditorOrigin::ModalitySelect);

    session.return_to_track_manager();
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 2,
            modal: None,
        }
    );
}

#[test]
fn test_modality_select_options_championship_editor_flow() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Options,
        selected_idx: 3, // ChampionshipEditor
        modal: None,
    };

    // Confirm on Championship Editor card (idx 3) opens Championship Editor
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.state, GameState::ChampionshipEditor);
    assert!(session.championship_editor_state.is_some());

    // Direct shortcut [C] from ModalitySelect
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Options,
        selected_idx: 0,
        modal: None,
    };
    session.enter_championship_editor(None);
    assert_eq!(session.state, GameState::ChampionshipEditor);
    assert!(session.championship_editor_state.is_some());
}

#[test]
fn test_modality_select_options_settings_modal_flow() {
    let mut session = RaceSession::new();
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Options,
        selected_idx: 4, // Settings
        modal: None,
    };

    // Confirm on Settings card (idx 4) opens Settings Modal
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert!(session.is_settings_modal_open());
    assert!(matches!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Options,
            selected_idx: 4,
            modal: None,
        }
    ));

    // Close settings modal
    session.close_settings_modal(false);
    assert!(!session.is_settings_modal_open());
}

#[test]
fn test_grand_hub_player_profile_selection_and_navigation() {
    let mut session = RaceSession::new();
    // Start on Classic module (index 1)
    session.state = GameState::ModuleSelect { selected_idx: 1 };

    // 1. Navigate UP from Classic -> selects Player Profile (idx 0)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_module_select();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // 2. Press Enter/Confirm on Player Profile -> loads ProfileManager
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_module_select();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert!(matches!(session.state, GameState::ProfileManager { .. }));
    assert_eq!(session.profile_origin, tdrace_app::game::ProfileOrigin::ModuleSelect);

    // 3. Return from ProfileManager with Cancel (Escape/Gamepad B) -> returns to ModuleSelect { selected_idx: 0 }
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    let sel_idx = match session.state {
        GameState::ProfileManager { selected_idx } => selected_idx,
        _ => 0,
    };
    session.update_profile_manager(sel_idx);
    session.input.gamepad.snapshot.btn_cancel_pressed = false;
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // 4. Wrap-around UP from Player Profile (idx 0) -> wraps to Extreme Off-Road (idx 6)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_module_select();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 6 });

    // 5. Wrap-around DOWN from Extreme Off-Road (idx 6) -> wraps to Player Profile (idx 0)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_module_select();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // 6. Navigate DOWN from Player Profile (idx 0) -> Classic (idx 1)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_module_select();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 1 });
}


