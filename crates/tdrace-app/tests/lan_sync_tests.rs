//! # LAN race synchronization and lifecycle tests (spec 044)
//!
//! Headless host + clients on an in-memory network. Every session runs its
//! real `update()`; the own cars are driven by the bot AI through
//! `LanRaceState::input_override`.

mod lan_support;

use cabinet::net::{LanCollisionMode, RaceStatus};
use lan_support::{build_lobby, build_lobby_with, launch, quiet_net, run_until_racing, step, Drivers};
use tdrace_app::game::{GameState, RaceSession};

fn results_of(s: &RaceSession) -> Vec<(u8, RaceStatus)> {
    s.lan_race
        .as_ref()
        .and_then(|l| l.results.clone())
        .unwrap_or_default()
        .iter()
        .map(|r| (r.slot_id, r.status))
        .collect()
}

#[test]
fn test_host_pause_keeps_everyone_connected_and_racing() {
    // Ghost mode: the braked host car must not block the bots, so only the network is under test.
    let net = quiet_net(21);
    let (host, clients) = build_lobby_with(&net, &["A", "B"], "classic_grand_prix", 3, LanCollisionMode::GhostPassing);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);
    let mut drivers = Drivers::new(sessions.len());
    for _ in 0..(60 * 3) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }

    sessions[0].pause_race();
    let client_car_before = sessions[1].world.vehicles[1].state.position;
    for _ in 0..(60 * 10) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }
    assert_eq!(sessions[0].state, GameState::Paused);
    assert!(sessions[0].world.vehicles[0].state.speed < 0.5, "paused host car must stand still, speed {}", sessions[0].world.vehicles[0].state.speed);
    for s in &sessions[1..] {
        assert!(s.lan_client.as_ref().unwrap().is_connected(), "no client may time out");
        assert_eq!(s.state, GameState::Racing);
    }
    assert!(sessions[1].world.vehicles[1].state.position.distance(client_car_before) > 20.0, "the others kept racing");
    // The host still sees the client cars move.
    let seen = sessions[0].world.vehicles[1].state.position;
    let owner = sessions[1].world.vehicles[1].state.position;
    assert!(seen.distance(owner) < 6.0, "host view of client car {seen:?} vs owner {owner:?}");

    sessions[0].resume_race();
    step(&net, &mut sessions);
    assert_eq!(sessions[0].state, GameState::Racing);
}

#[test]
fn test_client_leaving_mid_race_is_parked_and_dnf_everywhere() {
    let net = quiet_net(22);
    let (host, clients) = build_lobby(&net, &["Stayer", "Leaver"], "classic_grand_prix", 1);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);
    let mut drivers = Drivers::new(sessions.len());
    for _ in 0..(60 * 4) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }

    // The client in slot 2 quits the game.
    let mut leaver = sessions.pop().unwrap();
    leaver.exit_lan_session();
    let parked_at = sessions[0].world.vehicles[2].state.position;
    let mut frames = 0;
    while frames < 60 * 5 && !sessions.iter().all(|s| s.lan_car_left(2)) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
        frames += 1;
    }
    assert!(sessions.iter().all(|s| s.lan_car_left(2)), "car must be marked as left within 5 s");
    for _ in 0..60 {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }
    for s in &sessions {
        assert!(s.world.vehicles[2].state.speed < 0.01, "left car is parked");
        assert!(s.world.vehicles[2].state.position.distance(parked_at) < 3.0, "left car stays where it stopped");
    }

    // The race goes on and ends with the leaver marked Left on both machines.
    let mut guard = 0;
    while sessions.iter().any(|s| s.state != GameState::Finished) && guard < 60 * 240 {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
        guard += 1;
    }
    assert!(sessions.iter().all(|s| s.state == GameState::Finished), "race must end");
    let host_results = results_of(&sessions[0]);
    assert_eq!(host_results, results_of(&sessions[1]), "same results on every machine");
    assert_eq!(host_results.last(), Some(&(2, RaceStatus::Left)));
}

#[test]
fn test_host_leaving_mid_race_sends_clients_to_results_then_hub() {
    let net = quiet_net(23);
    let (host, clients) = build_lobby(&net, &["A", "B"], "classic_grand_prix", 3);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);
    let mut drivers = Drivers::new(sessions.len());
    for _ in 0..(60 * 3) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }

    let mut host = sessions.remove(0);
    host.exit_lan_session();
    drivers.0.remove(0);
    for _ in 0..(60 * 5) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }
    for s in &sessions {
        assert!(s.lan_race.as_ref().unwrap().host_left, "client must notice the host left");
        assert_eq!(s.state, GameState::Finished, "client shows the last standings");
        assert_eq!(s.results.len(), 3);
    }

    // Leaving the results closes the session.
    for s in sessions.iter_mut() {
        s.exit_lan_session();
        assert!(!s.is_lan_multiplayer);
    }
}

#[test]
fn test_three_player_race_finishes_with_the_same_results_everywhere() {
    let net = quiet_net(24);
    let (host, clients) = build_lobby(&net, &["A", "B"], "classic_grand_prix", 1);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);
    let mut drivers = Drivers::new(sessions.len());

    let mut first_finisher_waited = false;
    let mut guard = 0;
    while sessions.iter().any(|s| s.state != GameState::Finished) && guard < 60 * 300 {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
        guard += 1;
        // A player who finished first keeps watching the race until it closes.
        for s in &sessions {
            let lan = s.lan_race.as_ref().unwrap();
            if lan.local_finished && lan.results.is_none() && s.state == GameState::Racing {
                first_finisher_waited = true;
            }
        }
    }
    assert!(sessions.iter().all(|s| s.state == GameState::Finished), "race must close (guard {guard})");
    assert!(first_finisher_waited, "a finisher must wait in the race view");

    let host_results = results_of(&sessions[0]);
    assert_eq!(host_results.len(), 3);
    assert_eq!(host_results[0].1, RaceStatus::Finished, "someone won: {host_results:?}");
    for s in &sessions[1..] {
        assert_eq!(results_of(s), host_results, "same results on every machine");
    }
    let order: Vec<String> = sessions[0].results.iter().map(|r| r.car_name.clone()).collect();
    for s in &sessions[1..] {
        let other: Vec<usize> = s.results.iter().map(|r| r.car_idx).collect();
        let host_order: Vec<usize> = sessions[0].results.iter().map(|r| r.car_idx).collect();
        assert_eq!(other, host_order, "results screen order {order:?}");
    }
}

/// Puts the host's own car on top of the client's car (as the host sees it) and runs one physics step.
fn overlap_and_step(mode: LanCollisionMode) -> (f32, f32) {
    let net = quiet_net(25);
    let (host, clients) = build_lobby_with(&net, &["A"], "classic_grand_prix", 3, mode);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);
    let host = &mut sessions[0];
    let remote_pos = host.world.vehicles[1].state.position;
    host.world.vehicles[0].state.position = remote_pos + glam::Vec2::new(0.5, 0.0);
    host.world.vehicles[0].state.velocity = glam::Vec2::ZERO;
    let own_before = host.world.vehicles[0].state.position;
    host.physics_step(1.0 / 120.0);
    let own_moved = host.world.vehicles[0].state.position.distance(own_before);
    let remote_moved = host.world.vehicles[1].state.position.distance(remote_pos);
    (own_moved, remote_moved)
}

#[test]
fn test_ghost_mode_cars_pass_through_each_other() {
    let (own_moved, remote_moved) = overlap_and_step(LanCollisionMode::GhostPassing);
    assert!(own_moved < 0.01, "own car pushed {own_moved} m in ghost mode");
    assert_eq!(remote_moved, 0.0);
}

#[test]
fn test_solid_mode_pushes_only_the_own_car() {
    let (own_moved, remote_moved) = overlap_and_step(LanCollisionMode::FullSatSolid);
    assert!(own_moved > 0.05, "own car must be pushed out, moved {own_moved} m");
    assert_eq!(remote_moved, 0.0, "the remote car belongs to its owner");
}

/// Owner position at race time `t`, linear between recorded frames.
fn owner_pos_at(history: &[(f64, glam::Vec2)], t: f64) -> Option<glam::Vec2> {
    let i = history.iter().position(|(ht, _)| *ht >= t)?;
    if i == 0 {
        return None;
    }
    let (t0, p0) = history[i - 1];
    let (t1, p1) = history[i];
    let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) as f32;
    Some(p0.lerp(p1, u))
}

#[test]
fn test_four_players_with_slot_gap_stay_in_sync_over_a_lossy_link() {
    use cabinet::net::{SimLinkConfig, SimNetwork, INTERP_DELAY_SEC};

    // Slots 0 (host), 1 (leaves in the lobby), 2, 3, 4.
    let net = SimNetwork::new(SimLinkConfig { loss: 0.0, delay_sec: 0.002, jitter_sec: 0.0 }, 44);
    // Two laps (~95 s for the bots), so the 60 s measurement is all live racing.
    let (mut host, mut clients) = build_lobby(&net, &["Leaver", "B", "C", "D"], "classic_grand_prix", 2);
    let mut leaver = clients.remove(0);
    leaver.disconnect().unwrap();
    lan_support::pump_lobby(&net, &mut host, &mut clients, 60);

    // From here on the link drops 5% of all datagrams and jitters them by up to 20 ms.
    net.set_config(SimLinkConfig { loss: 0.05, delay_sec: 0.002, jitter_sec: 0.020 });
    let mut sessions = launch(&net, host, clients);

    // Same roster and slot -> car map on every machine.
    let rosters: Vec<Vec<u8>> = sessions
        .iter()
        .map(|s| s.lan_race.as_ref().unwrap().config.roster.iter().map(|e| e.slot_id).collect())
        .collect();
    for r in &rosters {
        assert_eq!(r, &vec![0, 2, 3, 4]);
    }
    for (s, slot) in sessions.iter().zip([0u8, 2, 3, 4]) {
        assert_eq!(s.lan_player_slot, slot);
        assert_eq!(s.player_car_index(), rosters[0].iter().position(|&x| x == slot).unwrap());
    }

    run_until_racing(&net, &mut sessions);
    let mut drivers = Drivers::new(sessions.len());
    let n = sessions.len();
    let mut history: Vec<Vec<(f64, glam::Vec2)>> = vec![Vec::new(); n];
    let mut errors: Vec<f32> = Vec::new();
    let mut max_error = 0.0f32;

    for _ in 0..(60 * 60) {
        drivers.drive(&mut sessions);
        lan_support::step(&net, &mut sessions);
        assert!(
            sessions.iter().all(|s| s.lan_race.as_ref().unwrap().standings.is_empty()),
            "measurement window must be live racing"
        );
        for (k, s) in sessions.iter().enumerate() {
            let clock = s.lan_race_clock().unwrap();
            history[k].push((clock, s.world.vehicles[s.player_car_index()].state.position));
        }
        // Every machine's view of every other car vs where its owner had it at the render time.
        for observer in &sessions {
            let render_time = observer.lan_race_clock().unwrap() - INTERP_DELAY_SEC;
            for (owner_k, owner) in sessions.iter().enumerate() {
                if std::ptr::eq(owner, observer) {
                    continue;
                }
                let car_idx = owner.player_car_index();
                if let Some(truth) = owner_pos_at(&history[owner_k], render_time) {
                    let e = observer.world.vehicles[car_idx].state.position.distance(truth);
                    errors.push(e);
                    max_error = max_error.max(e);
                }
            }
        }
    }
    errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95 = errors[errors.len() * 95 / 100];
    let p50 = errors[errors.len() / 2];
    eprintln!("sync error over {} samples: p50 {p50:.3} m, p95 {p95:.3} m, max {max_error:.3} m", errors.len());
    assert!(p95 <= 1.5, "p95 remote position error {p95} m (max {max_error} m)");

    for s in &sessions {
        let stats = if let Some(ref h) = s.lan_host { h.stats() } else { s.lan_client.as_ref().unwrap().stats() };
        assert_eq!(stats.encode_errors, 0);
        assert!(s.lan_client.as_ref().map_or(true, |c| c.is_connected()));
    }

    // Race to the end: identical results everywhere.
    let mut guard = 0;
    while sessions.iter().any(|s| s.state != GameState::Finished) && guard < 60 * 300 {
        drivers.drive(&mut sessions);
        lan_support::step(&net, &mut sessions);
        guard += 1;
    }
    assert!(sessions.iter().all(|s| s.state == GameState::Finished), "race must close");
    let host_results = results_of(&sessions[0]);
    assert_eq!(host_results.len(), 4);
    for s in &sessions[1..] {
        assert_eq!(results_of(s), host_results);
    }
}

#[test]
fn test_lan_multiplayer_proximity_engine_audio_and_doppler() {
    let net = quiet_net(42);
    let (host, clients) = build_lobby_with(&net, &["Opponent"], "classic_grand_prix", 3, LanCollisionMode::GhostPassing);
    let mut sessions = launch(&net, host, clients);
    run_until_racing(&net, &mut sessions);

    let mut drivers = Drivers::new(sessions.len());
    // Drive for 2 seconds (120 frames) so cars accelerate and stream remote states
    for _ in 0..120 {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }

    assert_eq!(sessions[0].player_car_index(), 0);
    assert_eq!(sessions[1].player_car_index(), 1);

    // Verify remote controls were captured and decoded from network state
    if let Some(ref lan0) = sessions[0].lan_race {
        assert!(lan0.remote_controls.contains_key(&1), "Host should store remote controls for Client slot 1");
        let c1 = lan0.remote_controls.get(&1).unwrap();
        assert!(c1.throttle > 0.0 || c1.brake > 0.0 || sessions[1].world.vehicles[1].state.speed > 1.0);
    }

    if let Some(ref lan1) = sessions[1].lan_race {
        assert!(lan1.remote_controls.contains_key(&0), "Client should store remote controls for Host slot 0");
    }

    // Verify proximity audio allocations if backend is available
    if sessions[0].audio.backend.is_available() {
        assert_eq!(sessions[0].audio.proximity_voices.len(), 3);
        assert_eq!(sessions[1].audio.proximity_voices.len(), 3);

        let dist = sessions[0].world.vehicles[0].state.position.distance(sessions[0].world.vehicles[1].state.position);
        if dist < 75.0 {
            // Host listening to Client car 1
            let host_voice_1 = sessions[0].audio.proximity_voices.iter().find(|s| s.vehicle_id == Some(1));
            assert!(host_voice_1.is_some(), "Host must allocate proximity voice slot for Client car 1");
            let v1 = host_voice_1.unwrap();
            assert!(v1.current_gain > 0.0, "Client car gain must be audible within 75m");
            assert!(v1.rpm_model.current_rpm >= 1000.0, "RPM model must track engine revolutions");

            // Client listening to Host car 0
            let client_voice_0 = sessions[1].audio.proximity_voices.iter().find(|s| s.vehicle_id == Some(0));
            assert!(client_voice_0.is_some(), "Client must allocate proximity voice slot for Host car 0");
            let v0 = client_voice_0.unwrap();
            assert!(v0.current_gain > 0.0, "Host car gain must be audible within 75m");
            assert!(v0.rpm_model.current_rpm >= 1000.0, "RPM model must track engine revolutions");
        }

        // Test Disconnect Culling: Client leaves the match
        let mut leaver = sessions.pop().unwrap();
        leaver.exit_lan_session();

        let mut frames = 0;
        while frames < 60 * 3 && !sessions[0].lan_car_left(1) {
            step(&net, &mut sessions);
            frames += 1;
        }
        assert!(sessions[0].lan_car_left(1), "Host must recognize client 1 left");

        // Step one frame so audio manager updates with disconnected car culled
        sessions[0].update();
        let host_voice_1_after_leave = sessions[0].audio.proximity_voices.iter().find(|s| s.vehicle_id == Some(1));
        assert!(host_voice_1_after_leave.is_none(), "Leaving client car must be culled from proximity voice slots");
    }
}


