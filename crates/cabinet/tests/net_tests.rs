//! # Cabinet Net Loopback Integration Tests
//!
//! Verifies end-to-end socket handshakes, slot assignments, state sync,
//! input streaming, snapshot broadcasting, and disconnect handling.

use cabinet::net::{ClientEvent, HostEvent, LanClient, LanHost};

#[test]
fn test_host_and_client_loopback_handshake() {
    let mut host = LanHost::bind_ephemeral("Championship Room", "HostMario")
        .expect("Bind host on loopback");
    let host_addr = host.local_addr().expect("Host local addr");

    let mut client = LanClient::connect(
        host_addr,
        "ClientLuigi",
        "ITA",
        "gt3_roma",
        "yellow",
    )
    .expect("Connect client");

    // Pump ticks until connected
    let mut client_connected = false;
    let mut host_saw_join = false;

    for _ in 0..50 {
        let host_events = host.update(0.016);
        for event in host_events {
            if let HostEvent::PlayerJoined { slot_id, player_name, .. } = event {
                assert_eq!(slot_id, 1);
                assert_eq!(player_name, "ClientLuigi");
                host_saw_join = true;
            }
        }

        let client_events = client.update(0.016);
        for event in client_events {
            if let ClientEvent::Connected { slot_id, room_name, .. } = event {
                assert_eq!(slot_id, 1);
                assert_eq!(room_name, "Championship Room");
                client_connected = true;
            }
        }

        if client_connected && host_saw_join {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(client_connected, "Client should successfully connect");
    assert!(host_saw_join, "Host should register player join");
    assert_eq!(host.active_slots().len(), 2);
    assert_eq!(client.assigned_slot_id(), Some(1));
}

#[test]
fn test_client_slot_customization_and_ready_check() {
    let mut host = LanHost::bind_ephemeral("Custom Room", "HostDriver")
        .expect("Bind host");
    let host_addr = host.local_addr().expect("Host addr");

    let mut client = LanClient::connect(
        host_addr,
        "GuestRacer",
        "FRA",
        "starter_kart",
        "blue",
    )
    .expect("Connect");

    // Connect
    for _ in 0..50 {
        host.update(0.016);
        client.update(0.016);
        if client.is_connected() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(!host.is_all_ready(), "Client is initially not ready");

    // Client toggles ready and changes vehicle
    client
        .send_slot_update("super_kart", "cyan", true)
        .expect("Send slot update");

    let mut host_updated = false;
    for _ in 0..50 {
        let events = host.update(0.016);
        for event in events {
            if let HostEvent::PlayerSlotUpdated { slot_id, car_model_id, is_ready, .. } = event {
                assert_eq!(slot_id, 1);
                assert_eq!(car_model_id, "super_kart");
                assert!(is_ready);
                host_updated = true;
            }
        }
        client.update(0.016);
        if host_updated {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(host_updated, "Host should register slot customization");
    assert!(host.is_all_ready(), "Both players are now marked ready");
}

#[test]
fn test_launch_start_and_car_state_relay() {
    let mut host = LanHost::bind_ephemeral("Race Room", "Pilot1")
        .expect("Bind host");
    let host_addr = host.local_addr().expect("Host addr");

    let mut client = LanClient::connect(
        host_addr,
        "Pilot2",
        "GBR",
        "classic_kart",
        "red",
    )
    .expect("Connect");

    // Connect and let a few clock pings complete.
    for _ in 0..60 {
        host.update(0.016);
        client.update(0.016);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(client.is_connected());
    assert!(client.clock_offset().is_some(), "client must have a clock sample");

    // Host launches the race with a frozen roster.
    let config = host.launch_race().expect("Launch race");
    assert_eq!(config.roster.len(), 2);
    assert_eq!(config.car_index_of(1), Some(1));

    let mut launched = None;
    for _ in 0..50 {
        host.update(0.016);
        for event in client.update(0.016) {
            if let ClientEvent::RaceLaunched(cfg) = event {
                launched = Some(cfg);
            }
        }
        if launched.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(launched.as_ref(), Some(&config), "client must receive the same roster");

    // Both sides load; the host schedules the green light.
    client.send_loaded().expect("Send loaded");
    host.set_local_loaded();
    let mut scheduled = false;
    for _ in 0..50 {
        for event in host.update(0.016) {
            if let HostEvent::RaceStartScheduled { .. } = event {
                scheduled = true;
            }
        }
        client.update(0.016);
        if scheduled && client.race_clock().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(scheduled, "host must schedule the start once everyone is loaded");
    let host_clock = host.race_clock().expect("host race clock");
    let client_clock = client.race_clock().expect("client race clock");
    assert!(host_clock < 0.0 && host_clock > -3.1, "countdown on host: {host_clock}");
    assert!((host_clock - client_clock).abs() < 0.05, "shared clock: host {host_clock}, client {client_clock}");

    // Client streams its own car state; the host receives it.
    client
        .send_car_state(NetCarState { time_ms: 16, pos_x: 10.0, pos_y: 25.0, vel_x: 6.0, ..Default::default() })
        .expect("Send car state");
    let mut host_got = None;
    for _ in 0..50 {
        for event in host.update(0.016) {
            if let HostEvent::CarState(state) = event {
                host_got = Some(state);
            }
        }
        client.update(0.016);
        if host_got.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let host_got = host_got.expect("host must receive the client car state");
    assert_eq!(host_got.slot, 1);
    assert_eq!(host_got.pos_y, 25.0);

    // Host relays the world; the client gets only the other car.
    host.send_car_state(NetCarState { time_ms: 20, pos_x: 10.0, pos_y: 20.0, ..Default::default() })
        .expect("Relay world");
    let mut client_got = None;
    for _ in 0..50 {
        host.update(0.016);
        for event in client.update(0.016) {
            if let ClientEvent::CarStates(states) = event {
                client_got = Some(states);
            }
        }
        if client_got.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let client_got = client_got.expect("client must receive the world state");
    assert_eq!(client_got.len(), 1);
    assert_eq!(client_got[0].slot, 0);
    assert_eq!(client_got[0].pos_y, 20.0);
}

#[test]
fn test_client_graceful_disconnect() {
    let mut host = LanHost::bind_ephemeral("Disconnect Room", "Host")
        .expect("Bind host");
    let host_addr = host.local_addr().expect("Host addr");

    let mut client = LanClient::connect(
        host_addr,
        "Leaver",
        "USA",
        "muscle_v8",
        "black",
    )
    .expect("Connect");

    for _ in 0..50 {
        host.update(0.016);
        client.update(0.016);
        if client.is_connected() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(host.active_slots().len(), 2);

    client.disconnect().expect("Disconnect");

    let mut host_saw_leave = false;
    for _ in 0..50 {
        let events = host.update(0.016);
        for event in events {
            if let HostEvent::PlayerLeft { slot_id, .. } = event {
                assert_eq!(slot_id, 1);
                host_saw_leave = true;
            }
        }
        if host_saw_leave {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(host_saw_leave, "Host should register client leaving");
    assert_eq!(host.active_slots().len(), 1);
}

#[test]
fn test_lobby_pump_network_keeps_client_connected_and_syncs_car() {
    use cabinet::net::{CabinetLanClientLobbyScreen, CabinetLanHostScreen};

    let host = LanHost::bind_ephemeral("Pump Room", "HostRacer").expect("Bind host");
    let host_addr = host.local_addr().expect("local addr");
    let mut host_screen = CabinetLanHostScreen::new(host);

    let client = LanClient::connect(host_addr, "Guest", "FRA", "gt_valente_corsa_t2", "viper_green").expect("connect");
    let mut lobby = CabinetLanClientLobbyScreen::new(client);
    for _ in 0..100 {
        host_screen.pump_network(0.016);
        lobby.pump_network(0.016);
        if lobby.client().is_connected() && host_screen.host.active_slots().len() == 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(host_screen.host.active_slots().len(), 2);

    // Client picks a car and becomes not ready, as when the game opens its Garage.
    lobby.set_ready(true);
    lobby.set_local_car("gt_bmr_bavaria_t1");
    lobby.set_ready(false);

    // Pump both sides for 6 s of game time: longer than both timeouts (3.5 s / 4.5 s).
    let mut t = 0.0;
    while t < 6.0 {
        host_screen.pump_network(0.05);
        assert!(lobby.pump_network(0.05), "client must stay connected while pumping");
        t += 0.05;
    }

    let slot = host_screen.host.slots()[1].clone().expect("client slot");
    assert_eq!(slot.car_model_id, "gt_bmr_bavaria_t1");
    assert_eq!(slot.color_scheme_id, "viper_green");
    assert!(!slot.is_ready);
}

#[test]
fn test_cabinet_lan_host_and_join_screens_lifecycle() {
    use cabinet::net::{CabinetLanClientLobbyScreen, CabinetLanHostScreen, CabinetLanJoinScreen, LanCollisionMode};

    // 1. Host Screen setup & controls cycling
    let host = LanHost::bind_ephemeral("Host UI Room", "HostRacer").expect("Bind host");
    let mut host_screen = CabinetLanHostScreen::new(host);

    assert_eq!(host_screen.host.laps(), 5);
    host_screen.cycle_laps();
    assert_eq!(host_screen.host.laps(), 10);

    host_screen.cycle_collision();
    assert_eq!(host_screen.host.collision_mode(), LanCollisionMode::GhostPassing);

    host_screen.set_track("monza", "Autodromo Nazionale Monza");
    assert_eq!(host_screen.host.track_id(), "monza");
    assert_eq!(host_screen.track_title, "Autodromo Nazionale Monza");
    assert_eq!(host_screen.host.laps(), 10, "set_track keeps the lap rule");

    host_screen.set_local_car("gt_vandorn_arrowhead_t2");
    let host_slot = host_screen.host.slots()[0].clone().expect("host slot");
    assert_eq!(host_slot.car_model_id, "gt_vandorn_arrowhead_t2");

    host_screen.cycle_livery();
    let host_slot = host_screen.host.slots()[0].clone().expect("host slot");
    assert_eq!(host_slot.car_model_id, "gt_vandorn_arrowhead_t2", "livery change keeps the car");
    assert_eq!(host_slot.color_scheme_id, cabinet::net::LAN_LIVERIES[host_screen.selected_livery_idx].0);
    assert!(host_screen.take_request().is_none());

    host_screen.copy_address_to_clipboard();
    assert!(host_screen.copied_timer > 0.0);

    // 2. Join Screen & Keypad setup
    let mut join_screen = CabinetLanJoinScreen::new("GuestDriver", "ESP", "gt_valente_corsa_t2", "red");
    join_screen.keypad.set_text("127.0.0.1:7777");
    join_screen.connect_via_keypad();
    assert!(join_screen.pending_client.is_some());

    // 3. Client Lobby Screen setup & loadout cycling
    let client = join_screen.pending_client.take().unwrap();
    let mut client_lobby = CabinetLanClientLobbyScreen::new(client);

    assert!(!client_lobby.is_ready);
    client_lobby.toggle_ready();
    assert!(client_lobby.is_ready);

    client_lobby.set_local_car("gt_bmr_bavaria_t1");
    assert_eq!(client_lobby.car_model_id, "gt_bmr_bavaria_t1");

    client_lobby.set_ready(false);
    assert!(!client_lobby.is_ready);
    assert!(client_lobby.take_request().is_none());
}


// ---------------------------------------------------------------------------
// Spec 044 reproductions of the sync defects found on 2026-09-28.
// ---------------------------------------------------------------------------

use cabinet::net::{NetCarState, SimLinkConfig, SimNetwork, WorldState, MAX_DATAGRAM_SIZE};

fn sim_pump(net: &SimNetwork, host: &mut LanHost, clients: &mut [&mut LanClient], frames: usize) {
    for _ in 0..frames {
        net.advance(0.016);
        host.update(0.016);
        for c in clients.iter_mut() {
            c.update(0.016);
        }
    }
}

fn sim_host_with_clients(net: &SimNetwork, names: &[&str]) -> (LanHost, Vec<LanClient>) {
    let mut host = LanHost::with_transport(Box::new(net.endpoint()), "Sim Room", "Host").unwrap();
    let host_addr = host.local_addr().unwrap();
    let mut clients = Vec::new();
    for name in names {
        let mut c = LanClient::connect_with_transport(Box::new(net.endpoint()), host_addr, *name, "ESP", "gt_valente_corsa_t2", "red").unwrap();
        for _ in 0..30 {
            net.advance(0.016);
            host.update(0.016);
            for other in clients.iter_mut() {
                let other: &mut LanClient = other;
                other.update(0.016);
            }
            c.update(0.016);
            if c.is_connected() {
                break;
            }
        }
        assert!(c.is_connected(), "{name} must join");
        clients.push(c);
    }
    (host, clients)
}

#[test]
fn test_d1_eight_car_world_state_fits_one_datagram() {
    let cars = (0..8u8)
        .map(|slot| NetCarState {
            slot,
            time_ms: 95_123,
            pos_x: -1234.5678,
            pos_y: 876.54321,
            vel_x: -45.123456,
            vel_y: 12.345678,
            angle: -2.3456789,
            angular_velocity: 0.12345678,
            steer_angle: -0.0345678,
            lap: 2,
            checkpoint: 17,
            progress: 0.5,
            ..Default::default()
        })
        .collect();
    let world = WorldState { host_time_ms: 95_130, cars };
    let encoded = world.encode().expect("8-car world state must encode");
    assert!(encoded.len() < MAX_DATAGRAM_SIZE, "{} bytes", encoded.len());
}

#[test]
fn test_d4_client_in_slot_three_keeps_its_slot_after_launch() {
    let net = SimNetwork::new(SimLinkConfig::default(), 1);
    let (mut host, mut clients) = sim_host_with_clients(&net, &["A", "B", "C"]);
    assert_eq!(clients[2].assigned_slot_id(), Some(3));

    host.launch_race().unwrap();
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 5);
    }
    assert_eq!(clients[2].assigned_slot_id(), Some(3), "slot after launch");
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 250);
    }
    assert_eq!(clients[2].assigned_slot_id(), Some(3), "slot in race");
}

#[test]
fn test_d5_lost_state_sync_does_not_change_the_client_roster() {
    let net = SimNetwork::new(SimLinkConfig::default(), 2);
    let (mut host, mut clients) = sim_host_with_clients(&net, &["A"]);
    // Client A is the only client, so the next StateSync goes to A. Drop it.
    let first = std::sync::Arc::new(std::sync::Mutex::new(true));
    net.set_drop_filter(Some(Box::new(move |bytes, _from, _to| {
        let mut first = first.lock().unwrap();
        if *first && bytes.windows(9).any(|w| w == b"StateSync") {
            *first = false;
            return true;
        }
        false
    })));

    let host_addr = host.local_addr().unwrap();
    let mut b = LanClient::connect_with_transport(Box::new(net.endpoint()), host_addr, "B", "ESP", "gt_valente_corsa_t2", "red").unwrap();
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        refs.push(&mut b);
        sim_pump(&net, &mut host, &mut refs, 60);
    }
    let host_names: Vec<String> = host.active_slots().iter().map(|s| s.player_name.clone()).collect();
    let a_names: Vec<String> = clients[0].slots().iter().map(|s| s.player_name.clone()).collect();
    assert_eq!(a_names, host_names, "client A must see the same roster as the host");
}

#[test]
fn test_v1_client_gets_a_v1_version_mismatch_reply() {
    use cabinet::net::Transport;
    let net = SimNetwork::new(SimLinkConfig::default(), 9);
    let mut host = LanHost::with_transport(Box::new(net.endpoint()), "New Room", "Host").unwrap();
    let old_client = net.endpoint();
    // A protocol v1 JoinRequest: magic, version 1, then JSON (no kind byte).
    let mut v1 = b"TDLN".to_vec();
    v1.push(1);
    v1.extend_from_slice(br#"{"Lobby":{"JoinRequest":{"protocol_version":1,"player_name":"Old","country_code":"ESP","car_model_id":"x","color_scheme_id":"red"}}}"#);
    old_client.send_to(&v1, host.local_addr().unwrap()).unwrap();
    net.advance(0.01);
    host.update(0.01);
    net.advance(0.01);

    let mut buf = [0u8; 1400];
    let (n, _) = old_client.recv_from(&mut buf).expect("host must answer the old client");
    assert_eq!(&buf[..4], b"TDLN");
    assert_eq!(buf[4], 1, "reply uses the old client's version byte");
    let body: serde_json::Value = serde_json::from_slice(&buf[5..n]).expect("v1 JSON body");
    assert_eq!(body["Lobby"]["JoinResponse"]["result"], "RejectedVersionMismatch");
    assert!(host.active_slots().len() == 1, "old client gets no slot");
}

/// Spec 082, scenario "LAN results carry the penalty": standings order by finish time plus joker penalty.
#[test]
fn test_lan_standings_order_by_finish_time_plus_penalty() {
    let net = SimNetwork::new(SimLinkConfig::default(), 11);
    let (mut host, mut clients) = sim_host_with_clients(&net, &["A"]);
    host.launch_race().unwrap();
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 5);
    }
    // Client A crosses first (120.0 s) but missed the joker; the host crosses at 125.0 s.
    clients[0].report_finish(120_000, None, 30_000).unwrap();
    let _ = host.report_finish(125_000, None, 0);
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 10);
    }
    let order: Vec<(u8, u32, u32)> = host.standings().iter().map(|f| (f.slot_id, f.finish_ms, f.penalty_ms)).collect();
    assert_eq!(order, vec![(0, 125_000, 0), (1, 120_000, 30_000)]);
    let client_view: Vec<u8> = clients[0].standings().iter().map(|f| f.slot_id).collect();
    assert_eq!(client_view, vec![0, 1], "the client gets the same order");
}
