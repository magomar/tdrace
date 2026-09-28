//! # LAN Multiplayer Integration Tests
//!
//! Verifies complete LAN multiplayer state transitions, hub navigation, host lobby creation,
//! join browser flow, synchronized launch session setup, and remote participant nameplates.

use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::ModalityCategory;

mod lan_support;

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
fn test_lan_host_prevents_split_screen_and_does_not_simulate_remote_cars() {
    let net = lan_support::quiet_net(11);
    let (host, clients) = lan_support::build_lobby(&net, &["RemoteClient"], "classic_grand_prix", 3);
    let mut sessions = lan_support::launch(&net, host, clients);

    // The LAN launch resets any earlier split-screen choice: one player per machine.
    assert_eq!(sessions[0].game_mode, tdrace_app::ui::menu::GameMode::StandardRace);
    assert!(sessions[0].is_lan_multiplayer);
    assert!(sessions[0].is_lan_host);
    assert!(!sessions[0].is_split_screen(), "LAN session must never run in split screen mode");
    assert_eq!(sessions[0].player_car_index(), 0);

    lan_support::run_until_racing(&net, &mut sessions);

    // Host drives its own car; the client car stays still on the host until the client moves it.
    let throttle = tdrace_core::physics::car::CarControls { throttle: 1.0, ..Default::default() };
    sessions[0].lan_race.as_mut().unwrap().input_override = Some(throttle);
    let remote_start = sessions[0].cars[1].state.position;
    let own_start = sessions[0].cars[0].state.position;
    for _ in 0..30 {
        net.advance(lan_support::FRAME_DT);
        sessions[0].update();
    }
    assert!(sessions[0].cars[0].state.speed > 0.0, "own car must accelerate");
    assert_ne!(sessions[0].cars[0].state.position, own_start);
    assert_eq!(sessions[0].cars[1].state.position, remote_start, "host must not simulate the client car");

    // The client drives; the host shows the client car where the client put it.
    sessions[1].lan_race.as_mut().unwrap().input_override = Some(throttle);
    for _ in 0..90 {
        lan_support::step(&net, &mut sessions);
    }
    let on_client = sessions[1].cars[1].state.position;
    let on_host = sessions[0].cars[1].state.position;
    assert_ne!(on_host, remote_start, "host must follow the client car");
    assert!(on_host.distance(on_client) < 5.0, "host view {on_host:?} vs owner {on_client:?}");

    for s in sessions.iter_mut() {
        s.exit_lan_session();
        assert!(!s.is_lan_multiplayer);
        assert!(s.lan_race.is_none());
    }
}

#[test]
fn test_lan_livery_synchronization_and_countdown_handshake() {
    use cabinet::net::{CabinetLanClientLobbyScreen, LanClient, LanHost};
    use tdrace_app::render::color::CarColorScheme;

    // 1. Host creates room with Corsa Red (index 0)
    let net = lan_support::quiet_net(5);
    let mut host = LanHost::with_transport(Box::new(net.endpoint()), "Livery Sync GP", "RedHost")
        .expect("failed to create test host");
    host.update_host_slot("gt_ferrari_296_gt3", "corsa_red");
    host.set_track_and_rules("classic_grand_prix", 3, cabinet::net::LanCollisionMode::FullSatSolid);
    let host_addr = host.local_addr().expect("local addr");

    // 2. Client connects with Viper Green (index 2)
    let mut client = LanClient::connect_with_transport(
        Box::new(net.endpoint()),
        host_addr,
        "GreenRacer",
        "FRA",
        "gt_porsche_911_gt3r",
        "viper_green",
    ).expect("failed to connect");

    for _ in 0..60 {
        net.advance(0.016);
        let _ = client.update(0.016);
        let _ = host.update(0.016);
    }

    assert!(client.is_connected());
    assert_eq!(client.color_scheme_id(), "viper_green");
    assert_eq!(client.car_model_id(), "gt_porsche_911_gt3r");

    // 3. Client lobby screen initialized from connected client
    let mut client_lobby = CabinetLanClientLobbyScreen::new(client);
    assert_eq!(client_lobby.selected_livery_idx, 2, "Viper Green must be selected (index 2)");
    assert_eq!(client_lobby.selected_car_idx, 1, "Porsche 911 GT3 R must be selected (index 1)");
    assert!(!client_lobby.is_in_race(), "Lobby must not be in race before launch");

    // 4. Host launches the race
    host.launch_race().expect("race launch");
    for _ in 0..30 {
        net.advance(0.016);
        let _ = host.update(0.016);
        let _ = client_lobby.client_mut().update(0.016);
        if client_lobby.is_in_race() {
            break;
        }
    }
    assert!(client_lobby.is_in_race(), "Client lobby must enter the race once RaceLaunch arrives");
    let client_inner = client_lobby.into_client();
    let slot = client_inner.assigned_slot_id().expect("slot");
    assert_eq!(slot, 1);

    // 5. Both sessions build the grid from the same frozen roster
    let mut host_session = RaceSession::new();
    host_session.launch_lan_race_session(Some(host), None, 0);
    let mut client_session = RaceSession::new();
    client_session.launch_lan_race_session(None, Some(client_inner), slot);

    for s in [&host_session, &client_session] {
        assert_eq!(s.color_schemes.len(), 2);
        assert_eq!(s.color_schemes[0], CarColorScheme::from_index(0));
        assert_eq!(s.color_schemes[1], CarColorScheme::from_index(2));
        assert_eq!(s.state, GameState::Countdown(tdrace_app::game::LAN_WAITING_COUNTDOWN));
    }
    assert_eq!(client_session.player_car_index(), 1);

    // 6. Everyone is loaded: both machines count down to the same green light
    let mut sessions = vec![host_session, client_session];
    for _ in 0..20 {
        lan_support::step(&net, &mut sessions);
    }
    let (h, c) = (sessions[0].lan_race_clock().unwrap(), sessions[1].lan_race_clock().unwrap());
    assert!(h < 0.0 && h > -3.0, "host countdown {h}");
    assert!((h - c).abs() < 0.02, "shared clock: host {h}, client {c}");
    for s in &sessions {
        assert!(matches!(s.state, GameState::Countdown(r) if r > 0.0 && r <= 3.0));
    }
    lan_support::run_until_racing(&net, &mut sessions);

    for s in sessions.iter_mut() {
        s.exit_lan_session();
    }
}

#[test]
fn test_d5_slot_gap_maps_every_player_to_its_own_car() {
    // Slots 0 (host), 1, 2, 3 join; slot 1 leaves before the launch -> roster 0, 2, 3.
    let net = lan_support::quiet_net(8);
    let (mut host, mut clients) = lan_support::build_lobby(&net, &["Leaver", "Second", "Third"], "classic_grand_prix", 3);
    let mut leaver = clients.remove(0);
    leaver.disconnect().unwrap();
    lan_support::pump_lobby(&net, &mut host, &mut clients, 60);
    assert_eq!(host.active_slots().iter().map(|s| s.slot_id).collect::<Vec<_>>(), vec![0, 2, 3]);

    let sessions = lan_support::launch(&net, host, clients);
    let names = ["Host", "Second", "Third"];
    for (s, me) in sessions.iter().zip(names) {
        let roster: Vec<&str> = s.grid_participants.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(roster, names, "same car list on every machine");
        assert_eq!(s.grid_participants[s.player_car_index()].name, me, "own car");
        assert!(s.grid_participants[s.player_car_index()].is_player);
    }
}
