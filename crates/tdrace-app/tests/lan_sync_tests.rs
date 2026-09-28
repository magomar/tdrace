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
    let client_car_before = sessions[1].cars[1].state.position;
    for _ in 0..(60 * 10) {
        drivers.drive(&mut sessions);
        step(&net, &mut sessions);
    }
    assert_eq!(sessions[0].state, GameState::Paused);
    assert!(sessions[0].cars[0].state.speed < 0.5, "paused host car must stand still, speed {}", sessions[0].cars[0].state.speed);
    for s in &sessions[1..] {
        assert!(s.lan_client.as_ref().unwrap().is_connected(), "no client may time out");
        assert_eq!(s.state, GameState::Racing);
    }
    assert!(sessions[1].cars[1].state.position.distance(client_car_before) > 20.0, "the others kept racing");
    // The host still sees the client cars move.
    let seen = sessions[0].cars[1].state.position;
    let owner = sessions[1].cars[1].state.position;
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
    let parked_at = sessions[0].cars[2].state.position;
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
        assert!(s.cars[2].state.speed < 0.01, "left car is parked");
        assert!(s.cars[2].state.position.distance(parked_at) < 3.0, "left car stays where it stopped");
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
    let remote_pos = host.cars[1].state.position;
    host.cars[0].state.position = remote_pos + glam::Vec2::new(0.5, 0.0);
    host.cars[0].state.velocity = glam::Vec2::ZERO;
    let own_before = host.cars[0].state.position;
    host.physics_step(1.0 / 120.0);
    let own_moved = host.cars[0].state.position.distance(own_before);
    let remote_moved = host.cars[1].state.position.distance(remote_pos);
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
