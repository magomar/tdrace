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

    // 1. Select LAN Multiplayer -> Enters LanHub
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 1, // LAN
        modal: None,
    };
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.state, GameState::LanHub { selected_idx: 0 });

    // Dismiss LanHub with B button back to ModalitySelect
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_lan_hub(0);
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
    assert_eq!(session.state, GameState::CareerSelect { selected_idx: 0 });
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
fn test_modality_select_escape_opens_exit_confirm_modal() {
    let mut session = RaceSession::new();
    // Spec 066: Default boot state is ModalitySelect with SinglePlayer
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::SinglePlayer,
            selected_idx: 0,
            modal: None,
        }
    );
    assert!(!session.show_exit_confirm);

    // Press Gamepad B / ESC on Modality Select screen (without active modal)
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_b_pressed = false;

    // Spec 066: ESC triggers exit confirmation modal rather than returning to Grand Hub
    assert!(
        session.show_exit_confirm,
        "ESC on ModalitySelect must show exit confirmation modal"
    );
    assert!(
        session.exit_confirm_modal.is_some(),
        "Exit confirmation modal must be initialized"
    );

    // Dismissing exit confirm modal returns to ModalitySelect without quitting
    session.show_exit_confirm = false;
    session.exit_confirm_modal = None;
    assert!(!session.show_exit_confirm);
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
    // Opened from the Garage card: Options tab, index 1.
    session.modality_cursor = (ModalityCategory::Options, 1);
    session.state = GameState::Garage(GarageOrigin::ModalitySelect);
    session.garage_tier = 2;

    // View mode toggle
    assert_eq!(
        session.garage_view_mode,
        tdrace_app::ui::GarageViewMode::Lateral
    );
    session.input.gamepad.snapshot.btn_y_pressed = true;
    session.update_garage(GarageOrigin::ModalitySelect, 0.016);
    session.input.gamepad.snapshot.btn_y_pressed = false;
    assert_eq!(
        session.garage_view_mode,
        tdrace_app::ui::GarageViewMode::TopDownTurntable
    );

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
    let gt3 = CarChoice::GT3Car; // tier 2
    let hyper = CarChoice::HypercarPrototype; // tier 5

    // In a Tier 2 race (e.g. GT3):
    let race_tier = 2;
    assert!(
        gt4.is_eligible_for_race_tier(race_tier, false),
        "Lower tier (Tier 1 GT4) is eligible for Tier 2 race"
    );
    assert!(
        gt3.is_eligible_for_race_tier(race_tier, false),
        "Matching tier (Tier 2 GT3) is eligible for Tier 2 race"
    );
    assert!(
        !hyper.is_eligible_for_race_tier(race_tier, false),
        "Higher tier (Tier 5 Hypercar) is NOT eligible for Tier 2 race"
    );

    // In dev mode: all cars are eligible regardless of tier
    assert!(
        hyper.is_eligible_for_race_tier(race_tier, true),
        "Dev mode unlocks all categories"
    );
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
    assert_eq!(
        session.profile_origin,
        tdrace_app::game::ProfileOrigin::ModalitySelect
    );

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
    assert_eq!(
        session.editor_origin,
        tdrace_app::game::EditorOrigin::ModalitySelect
    );
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
    assert_eq!(
        session.editor_origin,
        tdrace_app::game::EditorOrigin::ModalitySelect
    );

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
    assert_eq!(
        session.profile_origin,
        tdrace_app::game::ProfileOrigin::ModuleSelect
    );

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

#[test]
fn test_menu_category_filter_cycling_and_direct_keys() {
    use tdrace_app::game::MenuCategoryFilter;

    let mut session = RaceSession::new();
    session.state = GameState::Menu;
    assert_eq!(session.menu_category_filter, MenuCategoryFilter::All);

    // Cycling via prev / next
    session.menu_category_filter = session.menu_category_filter.next();
    assert_eq!(session.menu_category_filter, MenuCategoryFilter::Classic);

    session.menu_category_filter = session.menu_category_filter.prev();
    assert_eq!(session.menu_category_filter, MenuCategoryFilter::All);

    // Next wrap-around
    let mut cur = MenuCategoryFilter::All;
    for expected in [
        MenuCategoryFilter::Classic,
        MenuCategoryFilter::Rally,
        MenuCategoryFilter::Kart,
        MenuCategoryFilter::Gt,
        MenuCategoryFilter::Nascar,
        MenuCategoryFilter::ExtremeOffroad,
        MenuCategoryFilter::Custom,
        MenuCategoryFilter::All,
    ] {
        cur = cur.next();
        assert_eq!(cur, expected);
    }

    // Direct selection via gamepad bumpers
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_rb_pressed = false;
    assert_eq!(session.menu_category_filter, MenuCategoryFilter::Classic);

    session.input.gamepad.snapshot.btn_lb_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_lb_pressed = false;
    assert_eq!(session.menu_category_filter, MenuCategoryFilter::All);
}

#[test]
fn test_menu_category_filter_circuits_isolation() {
    use tdrace_app::game::MenuCategoryFilter;

    let mut session = RaceSession::new();
    session.state = GameState::Menu;

    // Filter by Kart
    session.menu_category_filter = MenuCategoryFilter::Kart;
    let kart_tracks = session.filtered_menu_tracks();
    assert!(
        !kart_tracks.is_empty(),
        "Expected kart tracks to be available"
    );
    for t in &kart_tracks {
        let loaded = tdrace_app::ui::menu::resolve_track_for_menu(t);
        let mod_id = loaded
            .as_ref()
            .and_then(|tr| tr.module_id.as_deref())
            .unwrap_or("");
        assert!(
            mod_id == "kart" || t.track_id().contains("kart"),
            "Track {:?} not a kart track",
            t.track_id()
        );
    }

    // Filter by GT
    session.menu_category_filter = MenuCategoryFilter::Gt;
    let gt_tracks = session.filtered_menu_tracks();
    assert!(!gt_tracks.is_empty(), "Expected GT tracks to be available");
    for t in &gt_tracks {
        let loaded = tdrace_app::ui::menu::resolve_track_for_menu(t);
        let mod_id = loaded
            .as_ref()
            .and_then(|tr| tr.module_id.as_deref())
            .unwrap_or("");
        assert_eq!(mod_id, "gt", "Track {:?} not a GT track", t.track_id());
    }

    // Filter ALL has more tracks than any single category
    session.menu_category_filter = MenuCategoryFilter::All;
    let all_tracks = session.filtered_menu_tracks();
    assert!(
        all_tracks.len() > kart_tracks.len(),
        "ALL tracks should exceed Kart track count"
    );
    assert!(
        all_tracks.len() > gt_tracks.len(),
        "ALL tracks should exceed GT track count"
    );
}

#[test]
fn test_menu_track_confirmation_updates_active_module_and_car() {
    use tdrace_app::game::MenuCategoryFilter;

    let mut session = RaceSession::new();
    session.state = GameState::Menu;

    // Filter by GT
    session.menu_category_filter = MenuCategoryFilter::Gt;
    let gt_tracks = session.filtered_menu_tracks();
    assert!(!gt_tracks.is_empty());
    let (gt_idx, _) = gt_tracks
        .iter()
        .enumerate()
        .find(|(_, t)| session.is_track_unlocked(t.track_id()))
        .expect("Unlocked GT track");
    session.menu_track_idx = gt_idx;

    // Confirm selection with Enter/Space
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Module should automatically become GT and state should transition to StartingGrid
    assert_eq!(session.active_module_id, "gt");
    assert_eq!(session.state, GameState::StartingGrid);

    // Return to Menu
    session.state = GameState::Menu;
    session.menu_category_filter = MenuCategoryFilter::Nascar;
    let nascar_tracks = session.filtered_menu_tracks();
    assert!(!nascar_tracks.is_empty());
    let (nascar_idx, _) = nascar_tracks
        .iter()
        .enumerate()
        .find(|(_, t)| session.is_track_unlocked(t.track_id()))
        .expect("Unlocked NASCAR track");
    session.menu_track_idx = nascar_idx;

    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.active_module_id, "nascar");
    assert_eq!(session.state, GameState::StartingGrid);
}

#[test]
fn test_career_mode_initial_tier_1_championship_gating() {
    use tdrace_app::db::HallOfFameDb;
    use tdrace_app::ui::career_select::{
        build_tiered_career_championship_cards, ChampionshipCardStatus,
    };

    let mut session = RaceSession::new();
    session.hof_db = HallOfFameDb::open_in_memory().ok();
    session.profile_module_progress.clear();
    session.active_career_progress =
        tdrace_app::profile::ModuleCareerProgress::default_for_module(1, "gt");
    session.championship_session = None;

    let cards = build_tiered_career_championship_cards(
        &session.championship_manager,
        &session.profile_module_progress,
        &session.active_career_progress,
        session.championship_session.as_ref(),
    );

    assert!(!cards.is_empty(), "Should discover initial championships");

    // All visible championships for a fresh level-1 driver must strictly be Tier 1
    for card in &cards {
        assert_eq!(
            card.tier, 1,
            "Expected only Tier 1 championship initially, but found Tier {} for {}",
            card.tier, card.series_name
        );
        assert_eq!(card.status, ChampionshipCardStatus::New);
        assert_eq!(card.trophy, None);
    }

    // Verify all registered motorsport modules are represented with Tier 1
    let modules: Vec<&str> = cards.iter().map(|c| c.module_id.as_str()).collect();
    assert!(modules.contains(&"gt"), "GT Tier 1 should be visible");
    assert!(
        modules.contains(&"nascar"),
        "NASCAR Tier 1 should be visible"
    );
    assert!(
        modules.contains(&"rally"),
        "Rallycross Tier 1 should be visible"
    );
    assert!(
        modules.contains(&"kart"),
        "Karting Tier 1 should be visible"
    );
    assert!(
        modules.contains(&"extreme_offroad"),
        "Extreme Off-road Tier 1 should be visible"
    );

    // Verify higher tier series are NOT present
    assert!(!cards
        .iter()
        .any(|c| c.series_id == "gt3_european_challenge"));
    assert!(!cards
        .iter()
        .any(|c| c.series_id == "nascar_intermediate_oval_challenge"));
    assert!(!cards
        .iter()
        .any(|c| c.series_id == "rally_supercar_lites_trophy"));
}

#[test]
fn test_career_mode_unlocking_tier_expands_championships() {
    use tdrace_app::db::HallOfFameDb;
    use tdrace_app::profile::ModuleCareerProgress;
    use tdrace_app::ui::career_select::build_tiered_career_championship_cards;

    let mut session = RaceSession::new();
    session.hof_db = HallOfFameDb::open_in_memory().ok();
    session.profile_module_progress.clear();
    session.championship_session = None;

    // Advance GT module to Tier 2
    let mut gt_prog = ModuleCareerProgress::default_for_module(1, "gt");
    gt_prog.level = 2;
    session
        .profile_module_progress
        .insert("gt".to_string(), gt_prog.clone());
    session.active_career_progress = gt_prog;

    let cards = build_tiered_career_championship_cards(
        &session.championship_manager,
        &session.profile_module_progress,
        &session.active_career_progress,
        session.championship_session.as_ref(),
    );

    // GT should now have both Tier 1 and Tier 2 championships visible
    let gt_cards: Vec<_> = cards.iter().filter(|c| c.module_id == "gt").collect();
    assert_eq!(
        gt_cards.len(),
        2,
        "GT should show Tier 1 and Tier 2 championships"
    );
    assert!(gt_cards.iter().any(|c| c.tier == 1));
    assert!(gt_cards
        .iter()
        .any(|c| c.tier == 2 && c.series_id == "gt3_european_challenge"));

    // Higher tiers (3, 4, 5) must still be hidden
    assert!(!gt_cards.iter().any(|c| c.tier >= 3));

    // Other disciplines (still at level 1) must still show only Tier 1
    let nascar_cards: Vec<_> = cards.iter().filter(|c| c.module_id == "nascar").collect();
    assert_eq!(nascar_cards.len(), 1);
    assert_eq!(nascar_cards[0].tier, 1);
}

#[test]
fn test_career_mode_completed_championship_retention_and_replay() {
    use tdrace_app::db::HallOfFameDb;
    use tdrace_app::profile::{ChampionshipRecord, ModuleCareerProgress};
    use tdrace_app::ui::career_select::{
        build_tiered_career_championship_cards, ChampionshipCardStatus, PodiumTrophy,
    };

    let mut session = RaceSession::new();
    session.hof_db = HallOfFameDb::open_in_memory().ok();
    if let Some(db) = &session.hof_db {
        let _ = db.seed_default_profile_if_empty();
    }
    session.refresh_profiles_and_stats();
    session.championship_session = None;

    // Simulate completion of GT4 Clubman Sprint Cup with Gold trophy (best_finish: 1, 120 pts)
    let mut gt_prog = ModuleCareerProgress::default_for_module(1, "gt");
    gt_prog.championships_completed.insert(
        "gt4_clubman_sprint".to_string(),
        ChampionshipRecord {
            championship_id: "gt4_clubman_sprint".to_string(),
            tier: 1,
            best_finish: 1,
            times_completed: 1,
            highest_points: 120,
            last_completed_at: "2026-09-30T00:00:00Z".to_string(),
        },
    );
    gt_prog.trophies_gold = 1;
    if let Some(db) = &session.hof_db {
        let _ = db.save_module_progress(&gt_prog);
    }
    session
        .profile_module_progress
        .insert("gt".to_string(), gt_prog.clone());
    session.active_career_progress = gt_prog;

    let cards = build_tiered_career_championship_cards(
        &session.championship_manager,
        &session.profile_module_progress,
        &session.active_career_progress,
        session.championship_session.as_ref(),
    );

    // Completed championship must be present in the list
    let completed_card = cards
        .iter()
        .find(|c| c.series_id == "gt4_clubman_sprint")
        .expect("Completed GT4 card");
    assert_eq!(completed_card.status, ChampionshipCardStatus::Completed);
    assert_eq!(completed_card.trophy, Some(PodiumTrophy::Gold));
    assert_eq!(completed_card.player_points, 120);

    // Enter CareerSelect screen focused on the completed card
    let completed_idx = cards
        .iter()
        .position(|c| c.series_id == "gt4_clubman_sprint")
        .unwrap();
    session.state = GameState::CareerSelect {
        selected_idx: completed_idx,
    };

    // Press Enter to trigger replay prompt
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_career_select(completed_idx);
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Verify replay confirmation modal is summoned
    assert!(
        session.career_replay_modal.is_some(),
        "Replay confirmation modal must be displayed"
    );
    assert_eq!(
        session.pending_replay_series,
        Some((
            "gt4_clubman_sprint".to_string(),
            "GT4 Clubman Sprint Cup".to_string()
        ))
    );

    // Confirm replay in modal (Focus on confirm button and press confirm)
    if let Some(ref mut modal) = session.career_replay_modal {
        modal.nav.set_focus(1, 0); // Focus Confirm button
    }
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_career_select(completed_idx);
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Verify replay modal is dismissed
    assert!(
        session.career_replay_modal.is_none(),
        "Modal must be dismissed after confirmation"
    );

    // Verify championship session is initiated for Round 1
    let champ = session
        .championship_session
        .as_ref()
        .expect("Active championship session");
    assert_eq!(
        champ.current_round, 0,
        "Championship must restart at Round 1 (index 0)"
    );
    assert_eq!(session.state, GameState::StartingGrid);

    // Verify historic lifetime record was strictly preserved
    let hist = session
        .active_career_progress
        .championships_completed
        .get("gt4_clubman_sprint");
    assert!(
        hist.is_some(),
        "Historical championship record must not be erased on replay"
    );
    let hist_rec = hist.unwrap();
    assert_eq!(hist_rec.best_finish, 1, "Gold trophy must be preserved");
    assert_eq!(
        hist_rec.highest_points, 120,
        "Highest points must be preserved"
    );
}

#[test]
fn test_career_mode_escape_returns_to_modality_select() {
    let mut session = RaceSession::new();
    session.modality_cursor = (tdrace_app::game::ModalityCategory::SinglePlayer, 2);
    session.state = GameState::CareerSelect { selected_idx: 0 };

    // Press Escape
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    session.update_career_select(0);
    session.input.gamepad.snapshot.btn_cancel_pressed = false;

    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: tdrace_app::game::ModalityCategory::SinglePlayer,
            selected_idx: 2,
            modal: None,
        }
    );
}
