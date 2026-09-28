//! # Cabinet Authoritative LAN Host Subsystem
//!
//! Manages the host UDP transport, participant slot allocation, beacon
//! discovery broadcasting, player readiness, and the race session:
//! the frozen roster, the shared green light, the relay of owner car
//! states, and the referee (finish order, results).
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md`.

use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};

use super::beacon::LanBeaconBroadcaster;
use super::ip::LocalIpResolver;
use super::protocol::{
    sanitize_string, ControlMessage, FinishRecord, JoinResult, LanBeacon,
    LanCollisionMode, LobbyPacket, LobbySlot, Packet, ProtocolError, RaceConfig, RaceResult,
    RaceStatus, RosterEntry, DEFAULT_BEACON_PORT, MAGIC_BYTES,
    MAX_DATAGRAM_SIZE, MAX_NAME_LENGTH, PROTOCOL_VERSION,
};
use super::reliable::ReliableChannel;
use super::transport::{Transport, UdpTransport};
use super::wire::{self, NetCarState, WorldState};

/// Countdown between the scheduled start message and the green light.
pub const COUNTDOWN_SEC: f64 = 3.0;
/// Time the host waits for every client to load the track.
pub const LOAD_TIMEOUT_SEC: f64 = 10.0;
/// Time after the winner finishes before the race is closed.
pub const FINISH_TIMEOUT_SEC: f64 = 30.0;

/// High-level events emitted by the host during state updates.
#[derive(Debug, Clone, PartialEq)]
pub enum HostEvent {
    /// A new client has connected and been assigned a slot.
    PlayerJoined {
        slot_id: u8,
        player_name: String,
        car_model_id: String,
    },
    /// A connected client updated their vehicle choice or ready status.
    PlayerSlotUpdated {
        slot_id: u8,
        car_model_id: String,
        color_scheme_id: String,
        is_ready: bool,
    },
    /// A client left gracefully or timed out.
    PlayerLeft {
        slot_id: u8,
        reason: String,
    },
    /// A client finished loading the race.
    PlayerLoaded { slot_id: u8 },
    /// The green light is scheduled at `start_at` on the host clock.
    RaceStartScheduled { start_at: f64 },
    /// A newer owner state of a client's car.
    CarState(NetCarState),
    /// The finish order changed.
    StandingsUpdated(Vec<FinishRecord>),
    /// The race is closed with these results.
    RaceOver(Vec<RaceResult>),
}

struct ConnectedClient {
    slot_id: u8,
    addr: SocketAddr,
    last_seen_sec: f32,
    ping_ms: u16,
    channel: ReliableChannel,
    loaded: bool,
}

struct HostRace {
    config: RaceConfig,
    launched_at: f64,
    host_loaded: bool,
    start_at: Option<f64>,
    latest: HashMap<u8, NetCarState>,
    finishers: Vec<FinishRecord>,
    left: Vec<u8>,
    first_finish_at: Option<f64>,
    results: Option<Vec<RaceResult>>,
}

/// Authoritative session host managing the socket, slots, and synchronization.
pub struct LanHost {
    transport: Box<dyn Transport + Send>,
    port: u16,
    room_name: String,
    track_id: String,
    discipline: String,
    laps: u8,
    collision_mode: LanCollisionMode,
    max_players: u8,
    slots: Vec<Option<LobbySlot>>,
    clients: Vec<ConnectedClient>,
    broadcaster: Option<LanBeaconBroadcaster>,
    in_race: bool,
    race: Option<HostRace>,
    roster_rev: u32,
    timeout_sec: f32,
    ping_timer_sec: f32,
    ping_interval_sec: f32,
    recv_buf: [u8; MAX_DATAGRAM_SIZE],
}

impl LanHost {
    /// Binds an authoritative host session on the specified port.
    ///
    /// If the default port is busy, automatically attempts fallback ports `7778..7785`.
    /// Also initializes the discovery beacon broadcaster on `DEFAULT_BEACON_PORT` (7776).
    pub fn bind(
        room_name: impl Into<String>,
        host_player_name: impl Into<String>,
        country_code: impl Into<String>,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
        preferred_port: u16,
        max_players: u8,
        track_id: impl Into<String>,
        discipline: impl Into<String>,
        laps: u8,
    ) -> Result<Self, io::Error> {
        let max_players = max_players.clamp(2, 8);
        let room_name_sanitized = sanitize_string(&room_name.into(), MAX_NAME_LENGTH);
        let host_name_sanitized = sanitize_string(&host_player_name.into(), MAX_NAME_LENGTH);
        let country_code = country_code.into();
        let car_model_id = car_model_id.into();
        let color_scheme_id = color_scheme_id.into();
        let track_id = track_id.into();
        let discipline = discipline.into();

        // Bind game socket with fallback
        let mut chosen_socket = None;
        let mut bound_port = preferred_port;

        if let Ok(s) = UdpTransport::bind(&format!("0.0.0.0:{}", preferred_port)) {
            chosen_socket = Some(s);
        } else {
            for p in (preferred_port + 1)..=(preferred_port + 10) {
                if let Ok(s) = UdpTransport::bind(&format!("0.0.0.0:{}", p)) {
                    chosen_socket = Some(s);
                    bound_port = p;
                    break;
                }
            }
        }

        let socket = match chosen_socket {
            Some(s) => s,
            None => UdpTransport::bind("0.0.0.0:0")?,
        };

        if bound_port == 0 {
            bound_port = socket.local_addr()?.port();
        }

        // Slot 0 is reserved for Host
        let host_slot = LobbySlot {
            slot_id: 0,
            player_name: host_name_sanitized.clone(),
            country_code,
            car_model_id,
            color_scheme_id,
            is_ready: true,
            is_host: true,
            ping_ms: 0,
        };

        let mut slots = Vec::with_capacity(max_players as usize);
        slots.push(Some(host_slot));
        for _ in 1..max_players {
            slots.push(None);
        }

        // Initialize discovery beacon broadcaster
        let beacon = LanBeacon::new(
            room_name_sanitized.clone(),
            host_name_sanitized,
            bound_port,
            track_id.clone(),
            discipline.clone(),
            1,
            max_players,
            false,
        );

        let broadcaster = LanBeaconBroadcaster::new(beacon, DEFAULT_BEACON_PORT).ok();

        Ok(Self {
            transport: Box::new(socket),
            port: bound_port,
            room_name: room_name_sanitized,
            track_id,
            discipline,
            laps: laps.max(1),
            collision_mode: LanCollisionMode::default(),
            max_players,
            slots,
            clients: Vec::new(),
            broadcaster,
            in_race: false,
            race: None,
            roster_rev: 0,
            timeout_sec: 3.5,
            ping_timer_sec: 0.0,
            ping_interval_sec: 1.0,
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
        })
    }

    /// Binds an ephemeral host (port 0) for loopback unit tests without port conflicts.
    pub fn bind_ephemeral(
        room_name: impl Into<String>,
        host_player_name: impl Into<String>,
    ) -> Result<Self, io::Error> {
        Self::with_transport(Box::new(UdpTransport::bind("127.0.0.1:0")?), room_name, host_player_name)
    }

    /// Creates a host on a given transport (no beacon), with the same defaults as `bind_ephemeral`.
    pub fn with_transport(
        transport: Box<dyn Transport + Send>,
        room_name: impl Into<String>,
        host_player_name: impl Into<String>,
    ) -> Result<Self, io::Error> {
        let port = transport.local_addr()?.port();

        let host_slot = LobbySlot {
            slot_id: 0,
            player_name: sanitize_string(&host_player_name.into(), MAX_NAME_LENGTH),
            country_code: "ESP".to_string(),
            car_model_id: "gt_ferrari_296_gt3".to_string(),
            color_scheme_id: "red".to_string(),
            is_ready: true,
            is_host: true,
            ping_ms: 0,
        };

        let max_players = 4;
        let mut slots = Vec::with_capacity(max_players);
        slots.push(Some(host_slot));
        for _ in 1..max_players {
            slots.push(None);
        }

        Ok(Self {
            transport,
            port,
            room_name: sanitize_string(&room_name.into(), MAX_NAME_LENGTH),
            track_id: "monza".to_string(),
            discipline: "gt".to_string(),
            laps: 3,
            collision_mode: LanCollisionMode::default(),
            max_players: max_players as u8,
            slots,
            clients: Vec::new(),
            broadcaster: None,
            in_race: false,
            race: None,
            roster_rev: 0,
            timeout_sec: 3.5,
            ping_timer_sec: 0.0,
            ping_interval_sec: 1.0,
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
        })
    }

    /// Changes the room size (2..=8) while no client has joined.
    pub fn with_max_players(mut self, max_players: u8) -> Self {
        let max_players = max_players.clamp(2, 8);
        if self.clients.is_empty() {
            self.max_players = max_players;
            self.slots.resize(max_players as usize, None);
        }
        self
    }

    /// Local socket address of this game session.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.transport.local_addr()
    }

    /// Primary resolved IPv4 address of the local machine.
    pub fn local_ip(&self) -> Ipv4Addr {
        LocalIpResolver::resolve_local_ipv4()
    }

    /// Returns a human-friendly connection string, e.g. `"192.168.1.105:7777"`.
    pub fn formatted_address(&self) -> String {
        LocalIpResolver::format_address(self.local_ip(), self.port)
    }

    /// Bound game session port number.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Active track identifier.
    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    /// Lap count.
    pub fn laps(&self) -> u8 {
        self.laps
    }

    /// Current room name.
    pub fn room_name(&self) -> &str {
        &self.room_name
    }

    /// Collision mode.
    pub fn collision_mode(&self) -> LanCollisionMode {
        self.collision_mode
    }

    /// Slice of all participant slots (Some for occupied, None for open).
    pub fn slots(&self) -> &[Option<LobbySlot>] {
        &self.slots
    }

    /// Whether the race has been launched (the roster is frozen).
    pub fn is_in_race(&self) -> bool {
        self.in_race
    }

    /// Mutable slice of all participant slots.
    pub fn slots_mut(&mut self) -> &mut [Option<LobbySlot>] {
        &mut self.slots
    }

    /// Sets the host's ready status and broadcasts state.
    pub fn set_host_ready(&mut self, is_ready: bool) {
        if let Some(Some(ref mut slot)) = self.slots.get_mut(0) {
            slot.is_ready = is_ready;
        }
        let _ = self.broadcast_lobby_state();
    }

    /// Returns all occupied participant slots.
    pub fn active_slots(&self) -> Vec<LobbySlot> {
        self.slots.iter().filter_map(|s| s.clone()).collect()
    }

    /// Returns true if all active players in the room are marked ready.
    pub fn is_all_ready(&self) -> bool {
        let active = self.active_slots();
        !active.is_empty() && active.iter().all(|s| s.is_ready)
    }

    /// Sets the track and racing rules, broadcasting the change to all connected clients.
    pub fn set_track_and_rules(
        &mut self,
        track_id: impl Into<String>,
        laps: u8,
        collision_mode: LanCollisionMode,
    ) {
        self.track_id = track_id.into();
        self.laps = laps.max(1);
        self.collision_mode = collision_mode;
        self.sync_beacon();
        let _ = self.broadcast_lobby_state();
    }

    /// Updates the host's own vehicle and livery selection.
    pub fn update_host_slot(
        &mut self,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
    ) {
        if let Some(Some(ref mut slot)) = self.slots.get_mut(0) {
            slot.car_model_id = car_model_id.into();
            slot.color_scheme_id = color_scheme_id.into();
        }
        let _ = self.broadcast_lobby_state();
    }

    /// Host monotonic clock in seconds.
    pub fn now(&self) -> f64 {
        self.transport.now_sec()
    }

    /// Freezes the roster and sends the reliable `RaceLaunch` to every client.
    ///
    /// Grid order is slot order. Returns the race setup for the host's own session.
    pub fn launch_race(&mut self) -> Result<RaceConfig, ProtocolError> {
        if let Some(ref race) = self.race {
            return Ok(race.config.clone());
        }
        let roster = self
            .active_slots()
            .into_iter()
            .enumerate()
            .map(|(grid_index, s)| RosterEntry {
                slot_id: s.slot_id,
                grid_index: grid_index as u8,
                player_name: s.player_name,
                country_code: s.country_code,
                car_model_id: s.car_model_id,
                color_scheme_id: s.color_scheme_id,
            })
            .collect();
        let config = RaceConfig {
            track_id: self.track_id.clone(),
            laps: self.laps,
            collision_mode: self.collision_mode,
            roster,
        };
        self.in_race = true;
        self.sync_beacon();
        self.race = Some(HostRace {
            config: config.clone(),
            launched_at: self.now(),
            host_loaded: false,
            start_at: None,
            latest: HashMap::new(),
            finishers: Vec::new(),
            left: Vec::new(),
            first_finish_at: None,
            results: None,
        });
        self.broadcast_control(&ControlMessage::RaceLaunch(config.clone()))?;
        Ok(config)
    }

    /// Frozen race setup, once launched.
    pub fn race_config(&self) -> Option<&RaceConfig> {
        self.race.as_ref().map(|r| &r.config)
    }

    /// Marks the host's own session as loaded. The start is scheduled once every client is loaded.
    pub fn set_local_loaded(&mut self) -> Vec<HostEvent> {
        let mut events = Vec::new();
        if let Some(ref mut race) = self.race {
            race.host_loaded = true;
        }
        self.try_schedule_start(false, &mut events);
        events
    }

    /// Shared race clock in seconds since the green light (negative during the countdown).
    pub fn race_clock(&self) -> Option<f64> {
        let start_at = self.race.as_ref()?.start_at?;
        Some(self.now() - start_at)
    }

    /// Records the host's own car state and relays the latest state of every car to all clients.
    pub fn send_car_state(&mut self, state: NetCarState) -> Result<(), ProtocolError> {
        let Some(ref mut race) = self.race else {
            return Ok(());
        };
        let mut state = state;
        state.slot = 0;
        Self::store_state(race, state);
        let mut cars: Vec<NetCarState> = race.latest.values().copied().collect();
        cars.sort_by_key(|c| c.slot);
        let host_time_ms = self.race_clock().map(|t| (t.max(0.0) * 1000.0) as u32).unwrap_or(0);
        let encoded = WorldState { host_time_ms, cars }.encode()?;
        for client in &self.clients {
            let _ = self.transport.send_to(&encoded, client.addr);
        }
        Ok(())
    }

    /// Records the host's own finish.
    pub fn report_finish(&mut self, finish_ms: u32, best_lap_ms: Option<u32>) -> Vec<HostEvent> {
        let mut events = Vec::new();
        self.record_finish(0, finish_ms, best_lap_ms, &mut events);
        events
    }

    /// Finish order so far.
    pub fn standings(&self) -> &[FinishRecord] {
        self.race.as_ref().map(|r| r.finishers.as_slice()).unwrap_or(&[])
    }

    /// Final results, once the race is closed.
    pub fn results(&self) -> Option<&[RaceResult]> {
        self.race.as_ref()?.results.as_deref()
    }

    /// Kicks a player from the room by slot index.
    pub fn kick_player(&mut self, slot_id: u8, reason: &str) {
        if slot_id == 0 || slot_id as usize >= self.slots.len() {
            return;
        }
        let mut events = Vec::new();
        self.drop_client(slot_id, reason, true, &mut events);
        if (slot_id as usize) < self.slots.len() && !self.in_race {
            self.slots[slot_id as usize] = None;
        }
    }

    /// Tells every client that the session is closing and forgets them.
    ///
    /// The notice is sent a few times instead of reliably, because nobody listens afterwards.
    pub fn shutdown(&mut self, reason: &str) {
        let notice = LobbyPacket::DisconnectNotice { reason: reason.to_string() };
        if let Ok(encoded) = notice.encode() {
            for _ in 0..3 {
                for client in &self.clients {
                    let _ = self.transport.send_to(&encoded, client.addr);
                }
            }
        }
        self.clients.clear();
    }

    /// Updates internal timers, processes incoming datagrams, and checks heartbeat timeouts.
    pub fn update(&mut self, dt: f32) -> Vec<HostEvent> {
        let mut events = Vec::new();

        // 1. Advance beacon broadcaster
        if let Some(ref mut broadcaster) = self.broadcaster {
            let _ = broadcaster.update(dt);
        }

        // 2. Advance client age timers
        for client in &mut self.clients {
            client.last_seen_sec += dt;
        }

        // 3. Periodic ping transmission (RTT for the lobby display)
        self.ping_timer_sec += dt;
        if self.ping_timer_sec >= self.ping_interval_sec {
            self.ping_timer_sec = 0.0;
            let ping = LobbyPacket::Ping { sent_at: self.now() };
            let _ = self.broadcast_lobby_packet(&ping);
        }

        // 4. Pump incoming datagrams
        loop {
            match self.transport.recv_from(&mut self.recv_buf) {
                Ok((bytes_read, src_addr)) => {
                    let datagram = self.recv_buf[..bytes_read].to_vec();
                    self.handle_datagram(&datagram, src_addr, &mut events);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // 5. Resend unacknowledged reliable fragments
        let now = self.now();
        for client in &mut self.clients {
            for d in client.channel.poll_resend(now) {
                let _ = self.transport.send_to(&d, client.addr);
            }
        }

        // 6. Drop silent or unreachable clients
        let lost: Vec<u8> = self
            .clients
            .iter()
            .filter(|c| c.last_seen_sec > self.timeout_sec || c.channel.has_failed())
            .map(|c| c.slot_id)
            .collect();
        for slot_id in lost {
            self.drop_client(slot_id, "Heartbeat timeout", false, &mut events);
        }

        // 7. Race referee timers
        self.referee_tick(&mut events);

        events
    }

    fn handle_datagram(&mut self, datagram: &[u8], src_addr: SocketAddr, events: &mut Vec<HostEvent>) {
        let (kind, payload) = match wire::split(datagram) {
            Ok(split) => split,
            Err(ProtocolError::VersionMismatch(v)) if v < PROTOCOL_VERSION => {
                self.reply_legacy_version_mismatch(datagram, src_addr);
                return;
            }
            Err(_) => return,
        };

        let client_idx = self.clients.iter().position(|c| c.addr == src_addr);
        if let Some(idx) = client_idx {
            self.clients[idx].last_seen_sec = 0.0;
        }

        match kind {
            wire::KIND_JSON => {
                if let Ok(packet) = Packet::decode_json(payload) {
                    self.handle_packet(packet, src_addr, events);
                }
            }
            wire::KIND_CAR_STATE => {
                let (Some(idx), Ok(state)) = (client_idx, NetCarState::decode_payload(payload)) else {
                    return;
                };
                if state.slot != self.clients[idx].slot_id {
                    return;
                }
                if let Some(ref mut race) = self.race {
                    if race.config.car_index_of(state.slot).is_some() && Self::store_state(race, state) {
                        events.push(HostEvent::CarState(state));
                    }
                }
            }
            wire::KIND_RELIABLE => {
                let Some(idx) = client_idx else {
                    return;
                };
                let Ok((ack, messages)) = self.clients[idx].channel.on_reliable(payload) else {
                    return;
                };
                if !ack.is_empty() {
                    let _ = self.transport.send_to(&ack, src_addr);
                }
                let slot_id = self.clients[idx].slot_id;
                for bytes in messages {
                    if let Ok(message) = ControlMessage::from_bytes(&bytes) {
                        self.handle_control(slot_id, message, events);
                    }
                }
            }
            wire::KIND_ACK => {
                if let Some(idx) = client_idx {
                    let _ = self.clients[idx].channel.on_ack(payload);
                }
            }
            _ => {}
        }
    }

    /// Answers a join request from an older build in the format that build understands.
    fn reply_legacy_version_mismatch(&self, datagram: &[u8], src_addr: SocketAddr) {
        if datagram.len() < 5 || datagram[0..4] != MAGIC_BYTES || !datagram.windows(11).any(|w| w == b"JoinRequest") {
            return;
        }
        let body = serde_json::json!({
            "Lobby": { "JoinResponse": {
                "result": "RejectedVersionMismatch",
                "room_name": self.room_name,
                "track_id": self.track_id,
                "laps": self.laps,
                "slots": [],
            }}
        });
        let mut reply = MAGIC_BYTES.to_vec();
        reply.push(datagram[4]);
        reply.extend_from_slice(body.to_string().as_bytes());
        let _ = self.transport.send_to(&reply, src_addr);
    }

    fn handle_packet(&mut self, packet: Packet, src_addr: SocketAddr, events: &mut Vec<HostEvent>) {
        match packet {
            Packet::Lobby(LobbyPacket::JoinRequest {
                protocol_version,
                player_name,
                country_code,
                car_model_id,
                color_scheme_id,
            }) => {
                // Version check
                if protocol_version != PROTOCOL_VERSION {
                    self.send_join_response(JoinResult::RejectedVersionMismatch, src_addr);
                    return;
                }

                // Reconnection check (the first response was lost)
                if let Some(slot_id) = self.clients.iter().find(|c| c.addr == src_addr).map(|c| c.slot_id) {
                    self.send_join_response(JoinResult::Accepted { slot_id }, src_addr);
                    return;
                }

                // Race in progress check
                if self.in_race {
                    self.send_join_response(JoinResult::RejectedGameInProgress, src_addr);
                    return;
                }

                // Name uniqueness check: auto-disambiguate duplicates instead of rejecting
                let mut sanitized_name = sanitize_string(&player_name, MAX_NAME_LENGTH);
                if self.active_slots().iter().any(|s| s.player_name == sanitized_name) {
                    let base = sanitized_name.clone();
                    let mut suffix = 2;
                    while self.active_slots().iter().any(|s| s.player_name == sanitized_name) {
                        sanitized_name = format!("{} ({})", base, suffix);
                        suffix += 1;
                    }
                }

                // Find free slot
                match self.slots.iter().position(|s| s.is_none()) {
                    Some(idx) => {
                        let slot_id = idx as u8;
                        self.slots[idx] = Some(LobbySlot {
                            slot_id,
                            player_name: sanitized_name.clone(),
                            country_code,
                            car_model_id: car_model_id.clone(),
                            color_scheme_id,
                            is_ready: false,
                            is_host: false,
                            ping_ms: 0,
                        });
                        self.clients.push(ConnectedClient {
                            slot_id,
                            addr: src_addr,
                            last_seen_sec: 0.0,
                            ping_ms: 0,
                            channel: ReliableChannel::new(),
                            loaded: false,
                        });

                        self.sync_beacon();
                        self.send_join_response(JoinResult::Accepted { slot_id }, src_addr);
                        let _ = self.broadcast_lobby_state();

                        events.push(HostEvent::PlayerJoined {
                            slot_id,
                            player_name: sanitized_name,
                            car_model_id,
                        });
                    }
                    None => self.send_join_response(JoinResult::RejectedFull, src_addr),
                }
            }

            Packet::Lobby(LobbyPacket::Ping { sent_at }) => {
                let pong = LobbyPacket::Pong { sent_at, responder_time: self.now() };
                let _ = self.send_to_addr(&pong, src_addr);
            }

            Packet::Lobby(LobbyPacket::Pong { sent_at, .. }) => {
                let now = self.now();
                if let Some(client) = self.clients.iter_mut().find(|c| c.addr == src_addr) {
                    let rtt = ((now - sent_at).max(0.0) * 1000.0).min(999.0) as u16;
                    client.ping_ms = rtt;
                    if let Some(Some(ref mut slot)) = self.slots.get_mut(client.slot_id as usize) {
                        slot.ping_ms = rtt;
                    }
                }
            }

            Packet::Lobby(LobbyPacket::DisconnectNotice { reason }) => {
                if let Some(slot_id) = self.clients.iter().find(|c| c.addr == src_addr).map(|c| c.slot_id) {
                    self.drop_client(slot_id, &reason, false, events);
                }
            }

            _ => {}
        }
    }

    fn handle_control(&mut self, slot_id: u8, message: ControlMessage, events: &mut Vec<HostEvent>) {
        match message {
            ControlMessage::ClientSlotUpdate { slot_id: claimed, car_model_id, color_scheme_id, is_ready } => {
                if claimed != slot_id || self.in_race {
                    return;
                }
                if let Some(Some(ref mut slot)) = self.slots.get_mut(slot_id as usize) {
                    slot.car_model_id = car_model_id.clone();
                    slot.color_scheme_id = color_scheme_id.clone();
                    slot.is_ready = is_ready;
                    let _ = self.broadcast_lobby_state();
                    events.push(HostEvent::PlayerSlotUpdated { slot_id, car_model_id, color_scheme_id, is_ready });
                }
            }
            ControlMessage::Loaded => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.slot_id == slot_id) {
                    if !client.loaded {
                        client.loaded = true;
                        events.push(HostEvent::PlayerLoaded { slot_id });
                    }
                }
                self.try_schedule_start(false, events);
            }
            ControlMessage::Finished { finish_ms, best_lap_ms } => {
                self.record_finish(slot_id, finish_ms, best_lap_ms, events);
            }
            _ => {}
        }
    }

    /// Keeps the newest state per slot. Returns false for a stale or backwards state.
    fn store_state(race: &mut HostRace, state: NetCarState) -> bool {
        if let Some(prev) = race.latest.get(&state.slot) {
            if state.time_ms < prev.time_ms || state.lap < prev.lap {
                return false;
            }
        }
        race.latest.insert(state.slot, state);
        true
    }

    fn try_schedule_start(&mut self, load_timed_out: bool, events: &mut Vec<HostEvent>) {
        let all_loaded = self.clients.iter().all(|c| c.loaded);
        let Some(ref race) = self.race else {
            return;
        };
        if race.start_at.is_some() || !race.host_loaded || !(all_loaded || load_timed_out) {
            return;
        }
        let start_at = self.now() + COUNTDOWN_SEC;
        if let Some(ref mut race) = self.race {
            race.start_at = Some(start_at);
        }
        let _ = self.broadcast_control(&ControlMessage::RaceStart { start_at });
        events.push(HostEvent::RaceStartScheduled { start_at });
    }

    fn record_finish(&mut self, slot_id: u8, finish_ms: u32, best_lap_ms: Option<u32>, events: &mut Vec<HostEvent>) {
        let now = self.now();
        let Some(ref mut race) = self.race else {
            return;
        };
        if race.results.is_some()
            || race.config.car_index_of(slot_id).is_none()
            || race.left.contains(&slot_id)
            || race.finishers.iter().any(|f| f.slot_id == slot_id)
        {
            return;
        }
        race.finishers.push(FinishRecord { slot_id, finish_ms, best_lap_ms });
        race.finishers.sort_by_key(|f| f.finish_ms);
        race.first_finish_at.get_or_insert(now);
        let finishers = race.finishers.clone();
        let _ = self.broadcast_control(&ControlMessage::Standings { finishers: finishers.clone() });
        events.push(HostEvent::StandingsUpdated(finishers));
        self.check_race_over(events);
    }

    fn referee_tick(&mut self, events: &mut Vec<HostEvent>) {
        let now = self.now();
        let Some(ref race) = self.race else {
            return;
        };
        if race.start_at.is_none() && race.host_loaded && now - race.launched_at > LOAD_TIMEOUT_SEC {
            let late: Vec<u8> = self.clients.iter().filter(|c| !c.loaded).map(|c| c.slot_id).collect();
            for slot_id in late {
                self.drop_client(slot_id, "Did not load in time", true, events);
            }
            self.try_schedule_start(true, events);
        }
        self.check_race_over(events);
    }

    fn check_race_over(&mut self, events: &mut Vec<HostEvent>) {
        let now = self.now();
        let Some(ref mut race) = self.race else {
            return;
        };
        if race.results.is_some() || race.start_at.is_none() {
            return;
        }
        let racing: Vec<u8> = race
            .config
            .roster
            .iter()
            .map(|e| e.slot_id)
            .filter(|s| !race.left.contains(s))
            .collect();
        let all_done = !racing.is_empty() && racing.iter().all(|s| race.finishers.iter().any(|f| f.slot_id == *s));
        let timed_out = race.first_finish_at.is_some_and(|t| now - t >= FINISH_TIMEOUT_SEC);
        if !(all_done || timed_out || racing.is_empty()) {
            return;
        }

        let mut results = Vec::with_capacity(race.config.roster.len());
        for f in &race.finishers {
            results.push(RaceResult {
                slot_id: f.slot_id,
                position: 0,
                status: RaceStatus::Finished,
                finish_ms: Some(f.finish_ms),
                best_lap_ms: f.best_lap_ms,
            });
        }
        let mut still_racing: Vec<u8> = racing
            .iter()
            .copied()
            .filter(|s| !race.finishers.iter().any(|f| f.slot_id == *s))
            .collect();
        let progress = |s: &u8| race.latest.get(s).map(|c| (c.lap, c.progress)).unwrap_or((0, 0.0));
        still_racing.sort_by(|a, b| {
            let (la, pa) = progress(a);
            let (lb, pb) = progress(b);
            lb.cmp(&la).then(pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal))
        });
        for s in still_racing {
            results.push(RaceResult { slot_id: s, position: 0, status: RaceStatus::Dnf, finish_ms: None, best_lap_ms: None });
        }
        for &s in &race.left {
            results.push(RaceResult { slot_id: s, position: 0, status: RaceStatus::Left, finish_ms: None, best_lap_ms: None });
        }
        for (i, r) in results.iter_mut().enumerate() {
            r.position = i as u8 + 1;
        }
        race.results = Some(results.clone());
        let _ = self.broadcast_control(&ControlMessage::RaceOver { results: results.clone() });
        events.push(HostEvent::RaceOver(results));
    }

    /// Removes a client. In the lobby the slot is freed; in a race the car is marked as left.
    fn drop_client(&mut self, slot_id: u8, reason: &str, notify: bool, events: &mut Vec<HostEvent>) {
        let Some(idx) = self.clients.iter().position(|c| c.slot_id == slot_id) else {
            return;
        };
        let client = self.clients.remove(idx);
        if notify {
            // The client stops listening after this, so send it a few times instead of reliably.
            let notice = LobbyPacket::DisconnectNotice { reason: reason.to_string() };
            if let Ok(encoded) = notice.encode() {
                for _ in 0..3 {
                    let _ = self.transport.send_to(&encoded, client.addr);
                }
            }
        }

        match self.race {
            Some(ref mut race) => {
                if !race.left.contains(&slot_id) && race.config.car_index_of(slot_id).is_some() {
                    race.left.push(slot_id);
                }
                let _ = self.broadcast_control(&ControlMessage::PlayerLeft { slot_id, reason: reason.to_string() });
            }
            None => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize] = None;
                }
                self.sync_beacon();
                let _ = self.broadcast_lobby_state();
            }
        }
        events.push(HostEvent::PlayerLeft { slot_id, reason: reason.to_string() });
        if self.race.is_some() {
            self.try_schedule_start(false, events);
            self.check_race_over(events);
        }
    }

    fn sync_beacon(&mut self) {
        if self.broadcaster.is_some() {
            let active = self.active_slots();
            let host_name = self.slots[0].as_ref().map(|s| s.player_name.clone()).unwrap_or_default();
            let beacon = LanBeacon::new(
                self.room_name.clone(),
                host_name,
                self.port,
                self.track_id.clone(),
                self.discipline.clone(),
                active.len() as u8,
                self.max_players,
                self.in_race,
            );
            if let Some(ref mut broadcaster) = self.broadcaster {
                broadcaster.set_beacon(beacon);
            }
        }
    }

    /// Broadcasts the current room rules and participant slot roster to all connected clients.
    pub fn broadcast_lobby_state(&mut self) -> Result<(), ProtocolError> {
        self.roster_rev += 1;
        let state = ControlMessage::StateSync {
            roster_rev: self.roster_rev,
            track_id: self.track_id.clone(),
            laps: self.laps,
            collision_mode: self.collision_mode,
            slots: self.active_slots(),
        };
        self.broadcast_control(&state)
    }

    fn broadcast_control(&mut self, message: &ControlMessage) -> Result<(), ProtocolError> {
        let bytes = message.to_bytes()?;
        let now = self.transport.now_sec();
        for client in &mut self.clients {
            for d in client.channel.send(&bytes, now)? {
                let _ = self.transport.send_to(&d, client.addr);
            }
        }
        Ok(())
    }

    fn send_join_response(&self, result: JoinResult, addr: SocketAddr) {
        let response = LobbyPacket::JoinResponse {
            result,
            room_name: self.room_name.clone(),
            track_id: self.track_id.clone(),
            laps: self.laps,
        };
        let _ = self.send_to_addr(&response, addr);
    }

    fn broadcast_lobby_packet(&self, packet: &LobbyPacket) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        for client in &self.clients {
            let _ = self.transport.send_to(&encoded, client.addr);
        }
        Ok(())
    }

    fn send_to_addr(&self, packet: &LobbyPacket, addr: SocketAddr) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        let _ = self.transport.send_to(&encoded, addr);
        Ok(())
    }
}
