//! Shared helpers for headless LAN race tests on an in-memory network (spec 044).
#![allow(dead_code)]

use cabinet::net::{LanClient, LanCollisionMode, LanHost, SimLinkConfig, SimNetwork};
use tdrace_app::ai::{BotAiDriver, BotProfile};
use tdrace_app::game::{GameState, RaceSession};
use tdrace_core::physics::car::Car;

/// Frame time used by every headless session (`get_frame_time_safe` without a window).
pub const FRAME_DT: f64 = 1.0 / 60.0;

/// Host plus clients that joined in order. Slots follow join order unless one left.
pub fn build_lobby(net: &SimNetwork, names: &[&str], track_id: &str, laps: u8) -> (LanHost, Vec<LanClient>) {
    build_lobby_with(net, names, track_id, laps, LanCollisionMode::FullSatSolid)
}

pub fn build_lobby_with(
    net: &SimNetwork,
    names: &[&str],
    track_id: &str,
    laps: u8,
    collision_mode: LanCollisionMode,
) -> (LanHost, Vec<LanClient>) {
    let mut host = LanHost::with_transport(Box::new(net.endpoint()), "Sim GP", "Host")
        .unwrap()
        .with_max_players(8);
    host.set_track_and_rules(track_id, laps, collision_mode);
    let host_addr = host.local_addr().unwrap();
    let mut clients: Vec<LanClient> = Vec::new();
    for name in names {
        let c = LanClient::connect_with_transport(
            Box::new(net.endpoint()),
            host_addr,
            *name,
            "ESP",
            "gt_porsche_911_gt3r",
            "viper_green",
        )
        .unwrap();
        clients.push(c);
        for _ in 0..200 {
            pump_lobby(net, &mut host, &mut clients, 1);
            if clients.last().unwrap().is_connected() {
                break;
            }
        }
        assert!(clients.last().unwrap().is_connected(), "{name} must join");
    }
    pump_lobby(net, &mut host, &mut clients, 60);
    (host, clients)
}

pub fn pump_lobby(net: &SimNetwork, host: &mut LanHost, clients: &mut [LanClient], frames: usize) {
    for _ in 0..frames {
        net.advance(FRAME_DT);
        host.update(FRAME_DT as f32);
        for c in clients.iter_mut() {
            c.update(FRAME_DT as f32);
        }
    }
}

/// Launches the race and returns the game sessions: host first, then clients.
pub fn launch(net: &SimNetwork, mut host: LanHost, mut clients: Vec<LanClient>) -> Vec<RaceSession> {
    host.launch_race().expect("launch");
    for _ in 0..600 {
        pump_lobby(net, &mut host, &mut clients, 1);
        if clients.iter().all(|c| c.race_config().is_some()) {
            break;
        }
    }
    assert!(clients.iter().all(|c| c.race_config().is_some()), "every client must get RaceLaunch");

    let mut sessions = Vec::new();
    let mut hs = RaceSession::new();
    hs.launch_lan_race_session(Some(host), None, 0);
    sessions.push(hs);
    for c in clients {
        let slot = c.assigned_slot_id().expect("client slot");
        let mut s = RaceSession::new();
        s.launch_lan_race_session(None, Some(c), slot);
        sessions.push(s);
    }
    sessions
}

/// Advances the shared network clock by one frame and runs one full `update()` on every session.
pub fn step(net: &SimNetwork, sessions: &mut [RaceSession]) {
    net.advance(FRAME_DT);
    for s in sessions.iter_mut() {
        s.update();
    }
}

/// Steps until every session is racing (past the shared green light).
pub fn run_until_racing(net: &SimNetwork, sessions: &mut [RaceSession]) {
    for _ in 0..(60 * 20) {
        step(net, sessions);
        if sessions.iter().all(|s| s.state == GameState::Racing) {
            return;
        }
    }
    panic!("sessions never reached Racing: {:?}", sessions.iter().map(|s| s.state.clone()).collect::<Vec<_>>());
}

/// Per-session AI that drives the own car through `LanRaceState::input_override`.
pub struct Drivers(pub Vec<BotAiDriver>);

impl Drivers {
    pub fn new(n: usize) -> Self {
        Self((0..n).map(|_| BotAiDriver::new(BotProfile::default())).collect())
    }

    /// Sets this frame's scripted controls of every session's own car.
    pub fn drive(&mut self, sessions: &mut [RaceSession]) {
        for (s, ai) in sessions.iter_mut().zip(self.0.iter_mut()) {
            let idx = s.player_car_index();
            let others: Vec<&Car> = s.cars.iter().enumerate().filter(|(i, _)| *i != idx).map(|(_, c)| c).collect();
            let ctrl = ai.compute_controls(&s.cars[idx], &s.track, &others, FRAME_DT as f32);
            if let Some(lan) = s.lan_race.as_mut() {
                lan.input_override = Some(ctrl);
            }
        }
    }
}

pub fn quiet_net(seed: u64) -> SimNetwork {
    SimNetwork::new(SimLinkConfig { loss: 0.0, delay_sec: 0.002, jitter_sec: 0.0 }, seed)
}
