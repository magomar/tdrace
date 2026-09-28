//! # Cabinet LAN Client Subsystem
//!
//! Manages the client-side transport, handshake with the host, lobby state
//! caching, the reliable control channel, clock synchronization with the
//! host, and the stream of owner car states.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md`.

use std::io;
use std::net::SocketAddr;

use super::clock::ClockSync;
use super::protocol::{
    sanitize_string, ControlMessage, FinishRecord, JoinResult,
    LanCollisionMode, LobbyPacket, LobbySlot, Packet, ProtocolError, RaceConfig, RaceResult,
    MAX_DATAGRAM_SIZE, MAX_NAME_LENGTH, PROTOCOL_VERSION,
};
use super::reliable::ReliableChannel;
use super::transport::{Transport, UdpTransport};
use super::wire::{self, NetCarState, WorldState};

/// Connection lifecycle state of the LAN client.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    /// Not connected to any session.
    Disconnected(Option<String>),
    /// Handshake sent to host, waiting for response.
    Connecting {
        host_addr: SocketAddr,
        elapsed_sec: f32,
        retries: u8,
    },
    /// Admitted into lobby waiting room.
    InLobby {
        assigned_slot_id: u8,
        room_name: String,
        track_id: String,
        laps: u8,
        collision_mode: LanCollisionMode,
        slots: Vec<LobbySlot>,
    },
    /// The host launched the race; see `LanClient::race_config` and `race_clock`.
    InRace {
        assigned_slot_id: u8,
    },
}

/// Events produced by the client during state updates.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// Successfully admitted into the host lobby.
    Connected {
        slot_id: u8,
        room_name: String,
        track_id: String,
    },
    /// Host broadcast an updated player list or race settings.
    LobbyUpdated {
        track_id: String,
        laps: u8,
        slots: Vec<LobbySlot>,
    },
    /// Host launched the race with this frozen setup.
    RaceLaunched(RaceConfig),
    /// Host scheduled the green light at `start_at` on the host clock.
    RaceStartScheduled { start_at: f64 },
    /// Newer states of other players' cars (never this client's own car).
    CarStates(Vec<NetCarState>),
    /// Finish order so far.
    Standings(Vec<FinishRecord>),
    /// A player left the race.
    PlayerLeft { slot_id: u8, reason: String },
    /// The race is closed with these results.
    RaceOver(Vec<RaceResult>),
    /// Client disconnected or was rejected/kicked by host.
    Disconnected(String),
}

/// Client endpoint for connecting to and participating in LAN games.
pub struct LanClient {
    transport: Box<dyn Transport + Send>,
    host_addr: SocketAddr,
    state: ClientState,
    player_name: String,
    country_code: String,
    car_model_id: String,
    color_scheme_id: String,
    slot_id: Option<u8>,
    room_name: String,
    last_seen_sec: f32,
    ping_timer_sec: f32,
    ping_interval_sec: f32,
    ping_ms: u16,
    timeout_sec: f32,
    recv_buf: [u8; MAX_DATAGRAM_SIZE],
    last_known_track_id: String,
    last_known_laps: u8,
    last_known_collision_mode: LanCollisionMode,
    last_known_slots: Vec<LobbySlot>,
    roster_rev: u32,
    channel: ReliableChannel,
    clock: ClockSync,
    race: Option<RaceConfig>,
    start_at_host: Option<f64>,
    latest_time_ms: Vec<Option<u32>>,
    standings: Vec<FinishRecord>,
    results: Option<Vec<RaceResult>>,
}

impl LanClient {
    /// Binds an ephemeral client UDP socket and sends an initial join handshake.
    pub fn connect(
        host_addr: SocketAddr,
        player_name: impl Into<String>,
        country_code: impl Into<String>,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
    ) -> Result<Self, io::Error> {
        let transport = Box::new(UdpTransport::bind("0.0.0.0:0")?);
        Self::connect_with_transport(transport, host_addr, player_name, country_code, car_model_id, color_scheme_id)
    }

    /// Creates a client on a given transport and sends the join handshake.
    pub fn connect_with_transport(
        transport: Box<dyn Transport + Send>,
        host_addr: SocketAddr,
        player_name: impl Into<String>,
        country_code: impl Into<String>,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
    ) -> Result<Self, io::Error> {
        let player_name = sanitize_string(&player_name.into(), MAX_NAME_LENGTH);
        let country_code = country_code.into();
        let car_model_id = car_model_id.into();
        let color_scheme_id = color_scheme_id.into();

        let mut client = Self {
            transport,
            host_addr,
            state: ClientState::Connecting {
                host_addr,
                elapsed_sec: 0.0,
                retries: 0,
            },
            player_name,
            country_code,
            car_model_id,
            color_scheme_id,
            slot_id: None,
            room_name: String::new(),
            last_seen_sec: 0.0,
            ping_timer_sec: 0.0,
            ping_interval_sec: 0.5,
            ping_ms: 0,
            timeout_sec: 4.5,
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
            last_known_track_id: "monza".to_string(),
            last_known_laps: 5,
            last_known_collision_mode: LanCollisionMode::default(),
            last_known_slots: Vec::new(),
            roster_rev: 0,
            channel: ReliableChannel::new(),
            clock: ClockSync::new(),
            race: None,
            start_at_host: None,
            latest_time_ms: vec![None; 256],
            standings: Vec::new(),
            results: None,
        };

        client.send_join_request()?;
        Ok(client)
    }

    /// Current connection state.
    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// Whether this client is actively connected (InLobby or InRace).
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ClientState::InLobby { .. } | ClientState::InRace { .. })
    }

    /// Slot assigned by the host at join, while connected.
    pub fn assigned_slot_id(&self) -> Option<u8> {
        if self.is_connected() {
            self.slot_id
        } else {
            None
        }
    }

    /// Last measured round-trip ping time in milliseconds.
    pub fn ping_ms(&self) -> u16 {
        self.ping_ms
    }

    /// Host socket address.
    pub fn host_addr(&self) -> SocketAddr {
        self.host_addr
    }

    /// Active track identifier synced from host.
    pub fn track_id(&self) -> &str {
        &self.last_known_track_id
    }

    /// Total laps synced from host.
    pub fn laps(&self) -> u8 {
        self.last_known_laps
    }

    /// Participant slots synced from host.
    pub fn slots(&self) -> &[LobbySlot] {
        &self.last_known_slots
    }

    /// Selected car model identifier.
    pub fn car_model_id(&self) -> &str {
        &self.car_model_id
    }

    /// Selected livery color scheme identifier.
    pub fn color_scheme_id(&self) -> &str {
        &self.color_scheme_id
    }

    /// Frozen race setup, once the host launched the race.
    pub fn race_config(&self) -> Option<&RaceConfig> {
        self.race.as_ref()
    }

    /// Local monotonic clock in seconds.
    pub fn now(&self) -> f64 {
        self.transport.now_sec()
    }

    /// Estimated host clock minus local clock, once a ping round trip completed.
    pub fn clock_offset(&self) -> Option<f64> {
        self.clock.offset()
    }

    /// Shared race clock in seconds since the green light (negative during the countdown).
    pub fn race_clock(&self) -> Option<f64> {
        let start_at = self.start_at_host?;
        Some(self.clock.to_host(self.now())? - start_at)
    }

    /// Remaining countdown in seconds, once the start is scheduled.
    pub fn countdown_remaining_sec(&self) -> Option<f32> {
        self.race_clock().map(|t| (-t).max(0.0) as f32)
    }

    /// Finish order received from the host.
    pub fn standings(&self) -> &[FinishRecord] {
        &self.standings
    }

    /// Final results, once the host closed the race.
    pub fn results(&self) -> Option<&[RaceResult]> {
        self.results.as_deref()
    }

    /// Sends a vehicle selection or ready toggle update to the host.
    pub fn send_slot_update(
        &mut self,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
        is_ready: bool,
    ) -> Result<(), ProtocolError> {
        let slot_id = match self.assigned_slot_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let car_model_id = car_model_id.into();
        let color_scheme_id = color_scheme_id.into();
        self.car_model_id = car_model_id.clone();
        self.color_scheme_id = color_scheme_id.clone();

        self.send_control(&ControlMessage::ClientSlotUpdate {
            slot_id,
            car_model_id,
            color_scheme_id,
            is_ready,
        })
    }

    /// Tells the host that the race session is loaded.
    pub fn send_loaded(&mut self) -> Result<(), ProtocolError> {
        self.send_control(&ControlMessage::Loaded)
    }

    /// Sends this client's own car state to the host (unreliable, newest wins).
    pub fn send_car_state(&mut self, state: NetCarState) -> Result<(), ProtocolError> {
        let Some(slot) = self.assigned_slot_id() else {
            return Ok(());
        };
        let mut state = state;
        state.slot = slot;
        let _ = self.transport.send_to(&state.encode(), self.host_addr);
        Ok(())
    }

    /// Tells the host that this client's car crossed the finish line.
    pub fn report_finish(&mut self, finish_ms: u32, best_lap_ms: Option<u32>) -> Result<(), ProtocolError> {
        self.send_control(&ControlMessage::Finished { finish_ms, best_lap_ms })
    }

    /// Sends a graceful disconnect notice to the host and resets state.
    pub fn disconnect(&mut self) -> Result<(), ProtocolError> {
        let notice = LobbyPacket::DisconnectNotice {
            reason: "Player left room".to_string(),
        };
        // The client stops listening after this, so send it a few times instead of reliably.
        for _ in 0..3 {
            let _ = self.send_to_host(&notice);
        }
        self.state = ClientState::Disconnected(Some("Disconnected by user".to_string()));
        Ok(())
    }

    /// Updates internal timers, processes incoming datagrams, and checks timeouts.
    pub fn update(&mut self, dt: f32) -> Vec<ClientEvent> {
        let mut events = Vec::new();
        if matches!(self.state, ClientState::Disconnected(_)) {
            return events;
        }

        self.last_seen_sec += dt;

        // 1. Connection retry logic
        if let ClientState::Connecting {
            host_addr: _,
            ref mut elapsed_sec,
            ref mut retries,
        } = self.state
        {
            *elapsed_sec += dt;
            if *elapsed_sec >= 0.75 {
                *elapsed_sec = 0.0;
                *retries += 1;
                if *retries > 6 {
                    self.state = ClientState::Disconnected(Some("Host connection timed out".to_string()));
                    events.push(ClientEvent::Disconnected("Host did not respond".to_string()));
                    return events;
                } else {
                    let _ = self.send_join_request();
                }
            }
        }

        // 2. Heartbeat and clock probe
        if self.is_connected() {
            self.ping_timer_sec += dt;
            if self.ping_timer_sec >= self.ping_interval_sec {
                self.ping_timer_sec = 0.0;
                let ping = LobbyPacket::Ping { sent_at: self.now() };
                let _ = self.send_to_host(&ping);
            }
        }

        // 3. Pump incoming datagrams
        loop {
            match self.transport.recv_from(&mut self.recv_buf) {
                Ok((bytes_read, src_addr)) => {
                    if src_addr == self.host_addr {
                        self.last_seen_sec = 0.0;
                        let datagram = self.recv_buf[..bytes_read].to_vec();
                        self.handle_datagram(&datagram, &mut events);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
            if matches!(self.state, ClientState::Disconnected(_)) {
                return events;
            }
        }

        // 4. Resend unacknowledged reliable fragments
        let now = self.now();
        for d in self.channel.poll_resend(now) {
            let _ = self.transport.send_to(&d, self.host_addr);
        }

        // 5. Host silence timeout
        if self.is_connected() && (self.last_seen_sec > self.timeout_sec || self.channel.has_failed()) {
            self.state = ClientState::Disconnected(Some("Lost connection to host".to_string()));
            events.push(ClientEvent::Disconnected("Host timed out".to_string()));
        }

        events
    }

    fn handle_datagram(&mut self, datagram: &[u8], events: &mut Vec<ClientEvent>) {
        let (kind, payload) = match wire::split(datagram) {
            Ok(split) => split,
            Err(ProtocolError::VersionMismatch(_)) => {
                self.state = ClientState::Disconnected(Some("Protocol version mismatch".to_string()));
                events.push(ClientEvent::Disconnected("Version mismatch with host".to_string()));
                return;
            }
            Err(_) => return,
        };
        match kind {
            wire::KIND_JSON => {
                if let Ok(packet) = Packet::decode_json(payload) {
                    self.handle_packet(packet, events);
                }
            }
            wire::KIND_WORLD_STATE => {
                let Ok(world) = WorldState::decode_payload(payload) else {
                    return;
                };
                let own = self.slot_id;
                let mut fresh = Vec::new();
                for car in world.cars {
                    if Some(car.slot) == own {
                        continue;
                    }
                    let seen = &mut self.latest_time_ms[car.slot as usize];
                    if seen.is_some_and(|t| car.time_ms <= t) {
                        continue;
                    }
                    *seen = Some(car.time_ms);
                    fresh.push(car);
                }
                if !fresh.is_empty() {
                    events.push(ClientEvent::CarStates(fresh));
                }
            }
            wire::KIND_RELIABLE => {
                let Ok((ack, messages)) = self.channel.on_reliable(payload) else {
                    return;
                };
                if !ack.is_empty() {
                    let _ = self.transport.send_to(&ack, self.host_addr);
                }
                for bytes in messages {
                    if let Ok(message) = ControlMessage::from_bytes(&bytes) {
                        self.handle_control(message, events);
                    }
                }
            }
            wire::KIND_ACK => {
                let _ = self.channel.on_ack(payload);
            }
            _ => {}
        }
    }

    fn handle_packet(&mut self, packet: Packet, events: &mut Vec<ClientEvent>) {
        match packet {
            Packet::Lobby(LobbyPacket::JoinResponse {
                result,
                room_name,
                track_id,
                laps,
            }) => match result {
                JoinResult::Accepted { slot_id } => {
                    if self.is_connected() {
                        return; // duplicate answer to a retried request
                    }
                    self.slot_id = Some(slot_id);
                    self.room_name = room_name.clone();
                    if self.roster_rev == 0 {
                        self.last_known_track_id = track_id.clone();
                        self.last_known_laps = laps;
                    }
                    self.state = ClientState::InLobby {
                        assigned_slot_id: slot_id,
                        room_name: room_name.clone(),
                        track_id: self.last_known_track_id.clone(),
                        laps: self.last_known_laps,
                        collision_mode: self.last_known_collision_mode,
                        slots: self.last_known_slots.clone(),
                    };
                    events.push(ClientEvent::Connected {
                        slot_id,
                        room_name,
                        track_id,
                    });
                }
                JoinResult::RejectedFull => {
                    self.state = ClientState::Disconnected(Some("Room is full".to_string()));
                    events.push(ClientEvent::Disconnected("Lobby room is full".to_string()));
                }
                JoinResult::RejectedVersionMismatch => {
                    self.state = ClientState::Disconnected(Some("Protocol version mismatch".to_string()));
                    events.push(ClientEvent::Disconnected("Version mismatch with host".to_string()));
                }
                JoinResult::RejectedGameInProgress => {
                    self.state = ClientState::Disconnected(Some("Race already in progress".to_string()));
                    events.push(ClientEvent::Disconnected("Race is already running".to_string()));
                }
                JoinResult::RejectedNameTaken => {
                    self.state = ClientState::Disconnected(Some("Driver name already taken".to_string()));
                    events.push(ClientEvent::Disconnected("Driver name already taken in room".to_string()));
                }
            },

            Packet::Lobby(LobbyPacket::Ping { sent_at }) => {
                let pong = LobbyPacket::Pong { sent_at, responder_time: self.now() };
                let _ = self.send_to_host(&pong);
            }

            Packet::Lobby(LobbyPacket::Pong { sent_at, responder_time }) => {
                let now = self.now();
                self.clock.add_sample(sent_at, responder_time, now);
                self.ping_ms = ((now - sent_at).max(0.0) * 1000.0).min(999.0) as u16;
            }

            Packet::Lobby(LobbyPacket::DisconnectNotice { reason }) => {
                self.state = ClientState::Disconnected(Some(reason.clone()));
                events.push(ClientEvent::Disconnected(reason));
            }

            _ => {}
        }
    }

    fn handle_control(&mut self, message: ControlMessage, events: &mut Vec<ClientEvent>) {
        match message {
            ControlMessage::StateSync { roster_rev, track_id, laps, collision_mode, slots } => {
                if roster_rev <= self.roster_rev {
                    return;
                }
                self.roster_rev = roster_rev;
                self.last_known_track_id = track_id.clone();
                self.last_known_laps = laps;
                self.last_known_collision_mode = collision_mode;
                self.last_known_slots = slots.clone();
                if let ClientState::InLobby {
                    track_id: ref mut state_track,
                    laps: ref mut state_laps,
                    collision_mode: ref mut state_collision,
                    slots: ref mut state_slots,
                    ..
                } = self.state
                {
                    *state_track = track_id.clone();
                    *state_laps = laps;
                    *state_collision = collision_mode;
                    *state_slots = slots.clone();
                    events.push(ClientEvent::LobbyUpdated { track_id, laps, slots });
                }
            }
            ControlMessage::RaceLaunch(config) => {
                if self.race.is_some() {
                    return;
                }
                if let Some(slot) = self.slot_id {
                    self.state = ClientState::InRace { assigned_slot_id: slot };
                }
                self.last_known_track_id = config.track_id.clone();
                self.last_known_laps = config.laps;
                self.race = Some(config.clone());
                events.push(ClientEvent::RaceLaunched(config));
            }
            ControlMessage::RaceStart { start_at } => {
                self.start_at_host = Some(start_at);
                events.push(ClientEvent::RaceStartScheduled { start_at });
            }
            ControlMessage::Standings { finishers } => {
                self.standings = finishers.clone();
                events.push(ClientEvent::Standings(finishers));
            }
            ControlMessage::PlayerLeft { slot_id, reason } => {
                events.push(ClientEvent::PlayerLeft { slot_id, reason });
            }
            ControlMessage::RaceOver { results } => {
                self.results = Some(results.clone());
                events.push(ClientEvent::RaceOver(results));
            }
            _ => {}
        }
    }

    fn send_join_request(&mut self) -> Result<(), io::Error> {
        let packet = LobbyPacket::JoinRequest {
            protocol_version: PROTOCOL_VERSION,
            player_name: self.player_name.clone(),
            country_code: self.country_code.clone(),
            car_model_id: self.car_model_id.clone(),
            color_scheme_id: self.color_scheme_id.clone(),
        };

        self.send_to_host(&packet).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }

    fn send_control(&mut self, message: &ControlMessage) -> Result<(), ProtocolError> {
        let bytes = message.to_bytes()?;
        let now = self.now();
        for d in self.channel.send(&bytes, now)? {
            let _ = self.transport.send_to(&d, self.host_addr);
        }
        Ok(())
    }

    fn send_to_host(&self, packet: &LobbyPacket) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        let _ = self.transport.send_to(&encoded, self.host_addr);
        Ok(())
    }
}
