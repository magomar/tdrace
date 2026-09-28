//! # Cabinet Net Loopback Integration Tests
//!
//! Verifies end-to-end socket handshakes, slot assignments, state sync,
//! input streaming, snapshot broadcasting, and disconnect handling.

use cabinet::net::{
    CarStateSnapshot, ClientEvent, HostEvent,
    LanClient, LanHost, WorldSnapshotPacket,
};

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
fn test_launch_countdown_and_race_streaming() {
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

    for _ in 0..50 {
        host.update(0.016);
        client.update(0.016);
        if client.is_connected() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // Host triggers launch countdown
    host.start_countdown(3000).expect("Start countdown");

    let mut countdown_started = false;
    for _ in 0..50 {
        host.update(0.016);
        let events = client.update(0.016);
        for event in events {
            if let ClientEvent::CountdownStarted { starts_in_millis, .. } = event {
                assert_eq!(starts_in_millis, 3000);
                countdown_started = true;
            }
        }
        if countdown_started {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(countdown_started, "Client should receive countdown start");

    // Client streams input frame
    client
        .send_input(0.75, 1.0, 0.0, false, false)
        .expect("Send input");

    let mut input_received = false;
    for _ in 0..50 {
        let events = host.update(0.016);
        for event in events {
            if let HostEvent::PlayerInput { slot_id, input } = event {
                assert_eq!(slot_id, 1);
                assert_eq!(input.steering, 0.75);
                assert_eq!(input.throttle, 1.0);
                input_received = true;
            }
        }
        client.update(0.016);
        if input_received {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(input_received, "Host should receive streamed input");

    // Host broadcasts world snapshot
    let snapshot = WorldSnapshotPacket {
        tick: 1,
        session_elapsed_sec: 0.016,
        cars: vec![
            CarStateSnapshot {
                slot_id: 0,
                pos_x: 10.0,
                pos_y: 20.0,
                velocity_x: 5.0,
                velocity_y: 0.0,
                heading_rad: 0.0,
                angular_velocity: 0.0,
                steer_angle_rad: 0.0,
                current_lap: 0,
                checkpoint_idx: 0,
                best_lap_time_ms: None,
                last_lap_time_ms: None,
                is_finished: false,
            },
            CarStateSnapshot {
                slot_id: 1,
                pos_x: 10.0,
                pos_y: 25.0,
                velocity_x: 6.0,
                velocity_y: 0.0,
                heading_rad: 0.0,
                angular_velocity: 0.0,
                steer_angle_rad: 0.1,
                current_lap: 0,
                checkpoint_idx: 0,
                best_lap_time_ms: None,
                last_lap_time_ms: None,
                is_finished: false,
            },
        ],
    };

    host.broadcast_snapshot(&snapshot).expect("Broadcast snapshot");

    let mut snapshot_received = false;
    for _ in 0..50 {
        host.update(0.016);
        let events = client.update(0.016);
        for event in events {
            if let ClientEvent::WorldSnapshot(s) = event {
                assert_eq!(s.tick, 1);
                assert_eq!(s.cars.len(), 2);
                snapshot_received = true;
            }
        }
        if snapshot_received {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(snapshot_received, "Client should receive authoritative world snapshot");
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

    host_screen.cycle_track();
    assert_eq!(host_screen.host.track_id(), "Autodromo Nazionale Monza");

    host_screen.copy_address_to_clipboard();
    assert!(host_screen.copied_timer > 0.0);

    // 2. Join Screen & Keypad setup
    let mut join_screen = CabinetLanJoinScreen::new("GuestDriver", "ESP", "gt_ferrari_296_gt3", "red");
    join_screen.keypad.set_text("127.0.0.1:7777");
    join_screen.connect_via_keypad();
    assert!(join_screen.pending_client.is_some());

    // 3. Client Lobby Screen setup & loadout cycling
    let client = join_screen.pending_client.take().unwrap();
    let mut client_lobby = CabinetLanClientLobbyScreen::new(client);

    assert!(!client_lobby.is_ready);
    client_lobby.toggle_ready();
    assert!(client_lobby.is_ready);

    let initial_car = client_lobby.car_models[client_lobby.selected_car_idx].0.clone();
    client_lobby.cycle_car();
    let cycled_car = client_lobby.car_models[client_lobby.selected_car_idx].0.clone();
    assert_ne!(initial_car, cycled_car);
}


// ---------------------------------------------------------------------------
// Spec 044 reproductions of the sync defects found on 2026-09-28.
// ---------------------------------------------------------------------------

use cabinet::net::{SimLinkConfig, SimNetwork, MAX_DATAGRAM_SIZE};

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
        let mut c = LanClient::connect_with_transport(Box::new(net.endpoint()), host_addr, *name, "ESP", "gt_ferrari_296_gt3", "red").unwrap();
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
#[ignore = "spec 044 D1: fixed by the binary wire format (tdrace-6d87.2)"]
fn test_d1_eight_car_world_snapshot_fits_one_datagram() {
    let cars = (0..8u8)
        .map(|i| CarStateSnapshot {
            slot_id: i,
            pos_x: -1234.5678,
            pos_y: 876.54321,
            velocity_x: -45.123456,
            velocity_y: 12.345678,
            heading_rad: -2.3456789,
            angular_velocity: 0.12345678,
            steer_angle_rad: -0.0345678,
            current_lap: 2,
            checkpoint_idx: 17,
            best_lap_time_ms: Some(83456),
            last_lap_time_ms: Some(84567),
            is_finished: false,
        })
        .collect();
    let snapshot = WorldSnapshotPacket { tick: 12345, session_elapsed_sec: 95.1, cars };
    let encoded = snapshot.encode().expect("8-car world state must encode");
    assert!(encoded.len() < MAX_DATAGRAM_SIZE);
}

#[test]
#[ignore = "spec 044 D4: fixed by the stored slot id (tdrace-6d87.3)"]
fn test_d4_client_in_slot_three_keeps_its_slot_after_launch() {
    let net = SimNetwork::new(SimLinkConfig::default(), 1);
    let (mut host, mut clients) = sim_host_with_clients(&net, &["A", "B", "C"]);
    assert_eq!(clients[2].assigned_slot_id(), Some(3));

    host.start_countdown(3000).unwrap();
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 5);
    }
    assert_eq!(clients[2].assigned_slot_id(), Some(3), "slot during countdown");
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        sim_pump(&net, &mut host, &mut refs, 250);
    }
    assert_eq!(clients[2].assigned_slot_id(), Some(3), "slot in race");
}

#[test]
#[ignore = "spec 044 D5: fixed by reliable StateSync and the RaceLaunch roster (tdrace-6d87.3)"]
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
    let mut b = LanClient::connect_with_transport(Box::new(net.endpoint()), host_addr, "B", "ESP", "gt_ferrari_296_gt3", "red").unwrap();
    {
        let mut refs: Vec<&mut LanClient> = clients.iter_mut().collect();
        refs.push(&mut b);
        sim_pump(&net, &mut host, &mut refs, 60);
    }
    let host_names: Vec<String> = host.active_slots().iter().map(|s| s.player_name.clone()).collect();
    let a_names: Vec<String> = clients[0].slots().iter().map(|s| s.player_name.clone()).collect();
    assert_eq!(a_names, host_names, "client A must see the same roster as the host");
}
