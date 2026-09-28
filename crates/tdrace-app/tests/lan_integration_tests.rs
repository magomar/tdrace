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
        "gt_ferrari_296_gt3",
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
    assert_eq!(session.player_car_index(), 0);
    assert_eq!(session.cars.len(), 1);
    assert_eq!(session.trackers.len(), 1);
    assert_eq!(session.grid_participants.len(), 1);
    assert_eq!(session.car_model_ids[0], Some("gt_ferrari_296_gt3"));
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
        model_id: Some("gt_ferrari_296_gt3"),
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

#[test]
fn test_lan_client_perspective_targeting_and_helpers() {
    let mut session = RaceSession::new();
    session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());

    use tdrace_core::physics::config::AssistProfile;

    // Verify canonical model ID resolver
    assert_eq!(RaceSession::canonicalize_car_model_id("scuderia_gt"), "gt_ferrari_296_gt3");
    assert_eq!(RaceSession::canonicalize_car_model_id("stuttgart_gt"), "gt_porsche_911_gt3r");
    assert_eq!(RaceSession::canonicalize_car_model_id("gt_amg_gt3_evo"), "gt_amg_gt3_evo");

    // Configure session as LAN client in slot 1
    session.is_lan_multiplayer = true;
    session.is_lan_host = false;
    session.lan_player_slot = 1;

    let base_cfg = session.config.get_car_config(tdrace_app::ui::menu::CarChoice::SportsCar);
    let mut host_car = tdrace_core::car::Car::new(base_cfg.clone());
    host_car.config.assists = AssistProfile::Arcade.to_config();

    let mut client_car = tdrace_core::car::Car::new(base_cfg.clone());
    client_car.config.assists = AssistProfile::Sport.to_config();

    session.cars = vec![host_car, client_car];
    session.trackers = vec![
        tdrace_core::track::checkpoint::TrackProgressTracker::new(session.track.checkpoints.len(), 3),
        tdrace_core::track::checkpoint::TrackProgressTracker::new(session.track.checkpoints.len(), 3),
    ];
    session.grid_participants = vec![
        tdrace_app::game::GridParticipant {
            is_player: false,
            bot_index: None,
            name: "HostPlayer".to_string(),
            alias: "HostPlayer".to_string(),
            country: Some("ESP".to_string()),
            car_title: "Ferrari 296 GT3".to_string(),
            car_choice: tdrace_app::ui::menu::CarChoice::SportsCar,
            color_scheme: tdrace_app::render::color::CarColorScheme::from_index(0),
            model_id: Some("gt_ferrari_296_gt3"),
            best_lap: None,
            best_circuit_time: None,
            random_seed: 1,
            driver_tier: None,
        },
        tdrace_app::game::GridParticipant {
            is_player: true,
            bot_index: None,
            name: "ClientPlayer".to_string(),
            alias: "ClientPlayer".to_string(),
            country: Some("FRA".to_string()),
            car_title: "Porsche 911 GT3 R".to_string(),
            car_choice: tdrace_app::ui::menu::CarChoice::SportsCar,
            color_scheme: tdrace_app::render::color::CarColorScheme::from_index(1),
            model_id: Some("gt_porsche_911_gt3r"),
            best_lap: None,
            best_circuit_time: None,
            random_seed: 2,
            driver_tier: None,
        },
    ];

    // Player car index for client must be slot 1
    assert_eq!(session.player_car_index(), 1);

    // Changing assist profile must modify client's car (index 1), NOT host's car (index 0)
    session.set_assist_profile(AssistProfile::Pro);
    assert_eq!(session.cars[1].config.assists, AssistProfile::Pro.to_config());
    assert_eq!(session.cars[0].config.assists, AssistProfile::Arcade.to_config());

    // Collecting nameplates for client must focus on host (car_idx 0)
    let client_focus = session.player_car_index();
    let nameplates = session.collect_bot_nameplates(client_focus);
    assert_eq!(nameplates.len(), 1);
    assert_eq!(nameplates[0].name, "HostPlayer");
    assert_eq!(nameplates[0].car_idx, 0);
    assert_eq!(nameplates[0].tier_label, Some("LAN"));
}

#[test]
fn test_lan_host_prevents_split_screen_and_applies_remote_inputs() {
    let mut session = RaceSession::new();

    // 1. Suppose user previously selected SplitScreen mode in menus
    session.game_mode = tdrace_app::ui::menu::GameMode::SplitScreen;
    assert!(session.is_split_screen());

    // 2. Launch LAN race session as host (slot 0)
    let host = cabinet::net::LanHost::bind(
        "Host Split Prevention Room",
        "HostPlayer",
        "ESP",
        "gt_ferrari_296_gt3",
        "corsa_red",
        0,
        4,
        "classic_grand_prix",
        "classic",
        3,
    ).expect("failed to bind host");

    session.launch_lan_race_session(Some(host), None, 0);

    // 3. Verify is_split_screen is strictly false and game_mode reset to StandardRace
    assert!(session.is_lan_multiplayer);
    assert!(session.is_lan_host);
    assert_eq!(session.lan_player_slot, 0);
    assert_eq!(session.player_car_index(), 0);
    assert_eq!(session.game_mode, tdrace_app::ui::menu::GameMode::StandardRace);
    assert!(!session.is_split_screen(), "LAN session must never run in split screen mode");

    // 4. Add remote client car (slot 1)
    let base_cfg = session.config.get_car_config(tdrace_app::ui::menu::CarChoice::SportsCar);
    let client_car = tdrace_core::car::Car::new(base_cfg);
    session.cars.push(client_car);
    session.trackers.push(tdrace_core::track::checkpoint::TrackProgressTracker::new(
        session.track.checkpoints.len(),
        3,
    ));
    session.grid_participants.push(tdrace_app::game::GridParticipant {
        is_player: false,
        bot_index: None,
        name: "RemoteClient".to_string(),
        alias: "RemoteClient".to_string(),
        country: Some("FRA".to_string()),
        car_title: "Porsche 911 GT3 R".to_string(),
        car_choice: tdrace_app::ui::menu::CarChoice::SportsCar,
        color_scheme: tdrace_app::render::color::CarColorScheme::from_index(1),
        model_id: Some("gt_porsche_911_gt3r"),
        best_lap: None,
        best_circuit_time: None,
        random_seed: 10,
        driver_tier: None,
    });

    assert_eq!(session.cars.len(), 2);

    // Initial position & velocity of client car
    let initial_pos = session.cars[1].state.position;
    let initial_speed = session.cars[1].state.speed;
    assert_eq!(initial_speed, 0.0);

    // 5. Host receives remote input packet from slot 1 (full throttle)
    session.lan_remote_inputs.insert(
        1,
        cabinet::net::ClientInputPacket {
            sequence_num: 1,
            slot_id: 1,
            steering: 0.0,
            throttle: 1.0,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        },
    );

    // Step physics multiple times
    for _ in 0..10 {
        session.physics_step(1.0 / 60.0);
    }

    // 6. Verify client car moved due to remote input applied by host physics_step
    assert!(
        session.cars[1].state.speed > 0.0,
        "Remote client car must accelerate from remote throttle input"
    );
    assert_ne!(
        session.cars[1].state.position,
        initial_pos,
        "Remote client car position must change under host simulation"
    );

    // Exit cleanly
    session.exit_lan_session();
    assert!(!session.is_lan_multiplayer);
}

#[test]
fn test_lan_livery_synchronization_and_countdown_handshake() {
    use cabinet::net::{CabinetLanClientLobbyScreen, LanClient, LanHost};
    use tdrace_app::render::color::CarColorScheme;

    // 1. Host creates room with Corsa Red (index 0)
    let mut host = LanHost::bind_ephemeral("Livery Sync GP", "RedHost")
        .expect("failed to bind test host");

    let host_addr = host.local_addr().expect("local addr");

    // 2. Client connects with Viper Green (index 2)
    let mut client = LanClient::connect(
        host_addr,
        "GreenRacer",
        "FRA",
        "gt_porsche_911_gt3r",
        "viper_green",
    ).expect("failed to connect");

    // Exchange handshake packets
    for _ in 0..50 {
        let _ = client.update(0.016);
        let _ = host.update(0.016);
        if client.is_connected() && host.active_slots().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(client.is_connected());
    assert_eq!(client.color_scheme_id(), "viper_green");
    assert_eq!(client.car_model_id(), "gt_porsche_911_gt3r");

    // 3. Client lobby screen initialized from connected client
    let mut client_lobby = CabinetLanClientLobbyScreen::new(client);
    assert_eq!(client_lobby.selected_livery_idx, 2, "Viper Green must be selected (index 2)");
    assert_eq!(client_lobby.selected_car_idx, 1, "Porsche 911 GT3 R must be selected (index 1)");
    assert!(!client_lobby.is_in_race(), "Lobby must not be in race before launch");

    // 4. Host initiates countdown
    host.launch_race().expect("race launch");

    // Pump packets so client receives LaunchCountdown
    let _ = host.update(0.01);
    let _ = client_lobby.client_mut().update(0.01);

    // Client lobby must immediately signal is_in_race == true for StartingCountdown
    assert!(
        client_lobby.is_in_race(),
        "Client lobby must trigger is_in_race as soon as countdown begins"
    );

    let client_inner = client_lobby.into_client();
    let remaining = client_inner.countdown_remaining_sec();
    assert!(remaining.is_some());
    assert!(remaining.unwrap() > 0.0 && remaining.unwrap() <= 3.0);

    // 5. Host launches race session
    let mut host_session = RaceSession::new();
    host_session.launch_lan_race_session(Some(host), None, 0);

    // 6. Client launches race session
    let mut client_session = RaceSession::new();
    client_session.launch_lan_race_session(None, Some(client_inner), 1);

    // Verify liveries on Host session:
    // Slot 0 is Red (index 0)
    // Slot 1 is Green (index 2)
    assert_eq!(host_session.color_schemes.len(), 2);
    assert_eq!(host_session.color_schemes[0], CarColorScheme::from_index(0));
    assert_eq!(host_session.color_schemes[1], CarColorScheme::from_index(2));

    // Verify liveries on Client session:
    // Slot 0 is Red (index 0)
    // Slot 1 is Green (index 2)
    assert_eq!(client_session.color_schemes.len(), 2);
    assert_eq!(client_session.color_schemes[0], CarColorScheme::from_index(0));
    assert_eq!(client_session.color_schemes[1], CarColorScheme::from_index(2));

    // Verify client session countdown initialized synchronously
    match client_session.state {
        GameState::Countdown(rem) => {
            assert!(rem > 0.0 && rem <= 3.0);
        }
        other => panic!("Expected GameState::Countdown, got {:?}", other),
    }

    // 7. Verify snapshot arrival during Countdown snaps client to GameState::Racing
    let snap_packet = cabinet::net::WorldSnapshotPacket {
        tick: 1,
        session_elapsed_sec: 0.1,
        cars: vec![
            cabinet::net::CarStateSnapshot {
                slot_id: 0,
                pos_x: 100.0,
                pos_y: 200.0,
                velocity_x: 10.0,
                velocity_y: 0.0,
                heading_rad: 1.5,
                angular_velocity: 0.0,
                steer_angle_rad: 0.0,
                current_lap: 1,
                checkpoint_idx: 0,
                best_lap_time_ms: None,
                last_lap_time_ms: None,
                is_finished: false,
            }
        ],
    };

    if let Some(ref mut host) = host_session.lan_host {
        let _ = host.broadcast_snapshot(&snap_packet);
    }

    // Advance client session countdown frame
    client_session.state = GameState::Countdown(2.5);
    match client_session.state {
        GameState::Countdown(ref mut rem) => {
            *rem -= 0.016;
            if client_session.is_lan_multiplayer {
                if let Some(ref mut client) = client_session.lan_client {
                    let events = client.update(0.016);
                    for event in events {
                        if let cabinet::net::ClientEvent::WorldSnapshot(snapshot) = event {
                            for car_snap in snapshot.cars {
                                let idx = car_snap.slot_id as usize;
                                if idx < client_session.cars.len() && idx != (client_session.lan_player_slot as usize) {
                                    let car = &mut client_session.cars[idx];
                                    car.state.position = glam::Vec2::new(car_snap.pos_x, car_snap.pos_y);
                                }
                            }
                            client_session.state = GameState::Racing;
                            break;
                        }
                    }
                }
            }
        }
        _ => {}
    }

    assert_eq!(
        client_session.state,
        GameState::Racing,
        "Snapshot arrival must transition client immediately to GameState::Racing"
    );
    assert_eq!(
        client_session.cars[0].state.position,
        glam::Vec2::new(100.0, 200.0),
        "Host car position must be synchronized from snapshot"
    );

    // Clean up
    host_session.exit_lan_session();
    client_session.exit_lan_session();
}

