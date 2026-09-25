//! # LAN Multiplayer Integration Tests
//!
//! Verifies complete LAN multiplayer state transitions, hub navigation, host lobby creation,
//! join browser flow, synchronized launch session setup, and remote participant nameplates.

use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::ModalityCategory;

#[test]
fn test_lan_multiplayer_hub_lifecycle() {
    let mut session = RaceSession::new();

    // 1. Enter ModalitySelect on Multiplayer category, select LAN Play
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::Multiplayer,
        selected_idx: 1, // LAN Play card
        modal: None,
    };

    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_modality_select();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.state, GameState::LanHub { selected_idx: 0 });

    // 2. Navigate within LanHub (Card 0 -> Card 1)
    session.input.gamepad.snapshot.dpad_right_pressed = true;
    session.update_lan_hub(0);
    session.input.gamepad.snapshot.dpad_right_pressed = false;
    assert_eq!(session.state, GameState::LanHub { selected_idx: 1 });

    // Navigate back (Card 1 -> Card 0)
    session.input.gamepad.snapshot.dpad_left_pressed = true;
    session.update_lan_hub(1);
    session.input.gamepad.snapshot.dpad_left_pressed = false;
    assert_eq!(session.state, GameState::LanHub { selected_idx: 0 });

    // 3. Select Card 0 (Host Game)
    session.select_lan_hub_option(0);
    assert_eq!(session.state, GameState::LanHostLobby);
    assert!(session.lan_host_screen.is_some());

    // 4. Disband Host Lobby
    if let Some(ref mut host_screen) = session.lan_host_screen {
        host_screen.exit_requested = true;
    }
    session.update_lan_host_lobby(0.016);
    assert_eq!(session.state, GameState::LanHub { selected_idx: 0 });
    assert!(session.lan_host_screen.is_none());

    // 5. Select Card 1 (Join Game)
    session.select_lan_hub_option(1);
    assert_eq!(session.state, GameState::LanJoinBrowser);
    assert!(session.lan_join_screen.is_some());

    // 6. Return from Join Browser
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_lan_join_browser(0.016);
    session.input.gamepad.snapshot.btn_b_pressed = false;
    assert_eq!(session.state, GameState::LanHub { selected_idx: 1 });
    assert!(session.lan_join_screen.is_none());

    // 7. Exit LanHub back to ModalitySelect
    session.input.gamepad.snapshot.btn_b_pressed = true;
    session.update_lan_hub(1);
    session.input.gamepad.snapshot.btn_b_pressed = false;
    assert_eq!(
        session.state,
        GameState::ModalitySelect {
            category: ModalityCategory::Multiplayer,
            selected_idx: 1,
            modal: None,
        }
    );
}

#[test]
fn test_lan_launch_session_and_nameplates() {
    let mut session = RaceSession::new();

    // Bind a test host on an ephemeral port
    let host = cabinet::net::LanHost::bind(
        "Grand Prix Test Room",
        "HostDriver",
        "ESP",
        "scuderia_gt",
        "corsa_red",
        0, // ephemeral port
        4,
        "classic_grand_prix",
        "classic",
        3,
    ).expect("failed to bind host");

    // Launch LAN race session as host (slot 0)
    session.launch_lan_race_session(Some(host), None, 0);

    assert!(session.is_lan_multiplayer);
    assert!(session.is_lan_host);
    assert_eq!(session.lan_player_slot, 0);
    assert_eq!(session.cars.len(), 1);
    assert_eq!(session.trackers.len(), 1);
    assert_eq!(session.grid_participants.len(), 1);
    assert!(matches!(session.state, GameState::Countdown(_)));

    // Verify nameplates contain LAN indicator for remote peers
    session.grid_participants.push(tdrace_app::game::GridParticipant {
        is_player: false,
        bot_index: None,
        name: "RemoteRacer".to_string(),
        alias: "RemoteRacer".to_string(),
        country: Some("FRA".to_string()),
        car_title: "GT3 Car".to_string(),
        car_choice: tdrace_app::ui::menu::CarChoice::SportsCar,
        color_scheme: tdrace_app::render::color::CarColorScheme::from_index(1),
        model_id: Some("scuderia_gt"),
        best_lap: None,
        best_circuit_time: None,
        random_seed: 99,
        driver_tier: None,
    });
    session.cars.push(tdrace_core::car::Car::new(session.config.get_car_config(tdrace_app::ui::menu::CarChoice::SportsCar)));
    session.trackers.push(tdrace_core::track::checkpoint::TrackProgressTracker::new(session.track.checkpoints.len(), 3));

    let nameplates = session.collect_bot_nameplates(0);
    assert_eq!(nameplates.len(), 1);
    assert_eq!(nameplates[0].name, "RemoteRacer");
    assert_eq!(nameplates[0].tier_label, Some("LAN"));

    // Exit LAN session cleans up
    session.exit_lan_session();
    assert!(!session.is_lan_multiplayer);
    assert!(!session.is_lan_host);
    assert!(session.lan_host.is_none());
    assert!(session.lan_client.is_none());
}
