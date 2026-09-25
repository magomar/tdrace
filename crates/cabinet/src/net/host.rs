//! # Cabinet Authoritative LAN Host Subsystem
//!
//! Manages the authoritative UDP socket, participant slot allocation,
//! beacon discovery broadcasting, player readiness, and game snapshot dispatch.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use super::beacon::LanBeaconBroadcaster;
use super::ip::LocalIpResolver;
use super::protocol::{
    sanitize_string, ClientInputPacket, JoinResult, LanBeacon, LanCollisionMode,
    LobbyPacket, LobbySlot, Packet, ProtocolError, WorldSnapshotPacket,
    DEFAULT_BEACON_PORT, MAX_DATAGRAM_SIZE, MAX_NAME_LENGTH, PROTOCOL_VERSION,
};

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
    /// In-race input packet received from a client.
    PlayerInput {
        slot_id: u8,
        input: ClientInputPacket,
    },
    /// Synchronized launch countdown started.
    CountdownStarted {
        starts_in_millis: u32,
    },
}

#[derive(Debug)]
struct ConnectedClient {
    slot_id: u8,
    addr: SocketAddr,
    last_seen_sec: f32,
    ping_ms: u16,
}

/// Authoritative session host managing the socket, slots, and synchronization.
pub struct LanHost {
    socket: UdpSocket,
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

        if let Ok(s) = UdpSocket::bind(format!("0.0.0.0:{}", preferred_port)) {
            chosen_socket = Some(s);
        } else {
            for p in (preferred_port + 1)..=(preferred_port + 10) {
                if let Ok(s) = UdpSocket::bind(format!("0.0.0.0:{}", p)) {
                    chosen_socket = Some(s);
                    bound_port = p;
                    break;
                }
            }
        }

        let socket = match chosen_socket {
            Some(s) => s,
            None => UdpSocket::bind("0.0.0.0:0")?,
        };

        if bound_port == 0 {
            bound_port = socket.local_addr()?.port();
        }

        socket.set_nonblocking(true)?;

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
            socket,
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
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        let port = socket.local_addr()?.port();
        socket.set_nonblocking(true)?;

        let host_slot = LobbySlot {
            slot_id: 0,
            player_name: sanitize_string(&host_player_name.into(), MAX_NAME_LENGTH),
            country_code: "ESP".to_string(),
            car_model_id: "scuderia_gt".to_string(),
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
            socket,
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
            timeout_sec: 3.5,
            ping_timer_sec: 0.0,
            ping_interval_sec: 1.0,
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
        })
    }

    /// Local socket address of this game session.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
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

    /// Whether this host session is actively running an in-race simulation.
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

    /// Starts the synchronized race launch countdown.
    pub fn start_countdown(&mut self, starts_in_millis: u32) -> Result<(), ProtocolError> {
        self.in_race = true;
        self.sync_beacon();

        let grid_positions: Vec<u8> = self.active_slots().iter().map(|s| s.slot_id).collect();
        let packet = LobbyPacket::LaunchCountdown {
            starts_in_millis,
            grid_positions,
        };
        self.broadcast_lobby_packet(&packet)?;
        Ok(())
    }

    /// Broadcasts an authoritative kinematic world snapshot to all clients at 60 Hz.
    pub fn broadcast_snapshot(&mut self, snapshot: &WorldSnapshotPacket) -> Result<(), ProtocolError> {
        let encoded = snapshot.encode()?;
        for client in &self.clients {
            let _ = self.socket.send_to(&encoded, client.addr);
        }
        Ok(())
    }

    /// Kicks a player from the room by slot index.
    pub fn kick_player(&mut self, slot_id: u8, reason: &str) {
        if slot_id == 0 || slot_id as usize >= self.slots.len() {
            return;
        }

        if let Some(idx) = self.clients.iter().position(|c| c.slot_id == slot_id) {
            let client = self.clients.remove(idx);
            let notice = LobbyPacket::DisconnectNotice {
                reason: reason.to_string(),
            };
            if let Ok(encoded) = notice.encode() {
                let _ = self.socket.send_to(&encoded, client.addr);
            }
        }

        self.slots[slot_id as usize] = None;
        self.sync_beacon();
        let _ = self.broadcast_lobby_state();
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

        // 3. Periodic ping transmission
        self.ping_timer_sec += dt;
        if self.ping_timer_sec >= self.ping_interval_sec {
            self.ping_timer_sec = 0.0;
            let now_ms = current_time_ms();
            let ping = LobbyPacket::Ping { timestamp_ms: now_ms };
            let _ = self.broadcast_lobby_packet(&ping);
        }

        // 4. Pump incoming socket packets
        loop {
            match self.socket.recv_from(&mut self.recv_buf) {
                Ok((bytes_read, src_addr)) => {
                    if let Ok(packet) = Packet::decode(&self.recv_buf[..bytes_read]) {
                        self.handle_packet(packet, src_addr, &mut events);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // 5. Prune timed-out clients
        let mut dropped_slots = Vec::new();
        self.clients.retain(|client| {
            if client.last_seen_sec > self.timeout_sec {
                dropped_slots.push(client.slot_id);
                false
            } else {
                true
            }
        });

        for slot_id in dropped_slots {
            if (slot_id as usize) < self.slots.len() {
                self.slots[slot_id as usize] = None;
            }
            self.sync_beacon();
            let _ = self.broadcast_lobby_state();
            events.push(HostEvent::PlayerLeft {
                slot_id,
                reason: "Heartbeat timeout".to_string(),
            });
        }

        events
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
                    let _ = self.send_to_addr(
                        &LobbyPacket::JoinResponse {
                            result: JoinResult::RejectedVersionMismatch,
                            room_name: self.room_name.clone(),
                            track_id: self.track_id.clone(),
                            laps: self.laps,
                            slots: self.active_slots(),
                        },
                        src_addr,
                    );
                    return;
                }

                // Race in progress check
                if self.in_race {
                    let _ = self.send_to_addr(
                        &LobbyPacket::JoinResponse {
                            result: JoinResult::RejectedGameInProgress,
                            room_name: self.room_name.clone(),
                            track_id: self.track_id.clone(),
                            laps: self.laps,
                            slots: self.active_slots(),
                        },
                        src_addr,
                    );
                    return;
                }

                // Reconnection check
                let existing_slot = self.clients.iter_mut().find(|c| c.addr == src_addr).map(|c| {
                    c.last_seen_sec = 0.0;
                    c.slot_id
                });
                if let Some(slot_id) = existing_slot {
                    let _ = self.send_to_addr(
                        &LobbyPacket::JoinResponse {
                            result: JoinResult::Accepted { slot_id },
                            room_name: self.room_name.clone(),
                            track_id: self.track_id.clone(),
                            laps: self.laps,
                            slots: self.active_slots(),
                        },
                        src_addr,
                    );
                    return;
                }

                // Name uniqueness check
                let sanitized_name = sanitize_string(&player_name, MAX_NAME_LENGTH);
                if self.active_slots().iter().any(|s| s.player_name == sanitized_name) {
                    let _ = self.send_to_addr(
                        &LobbyPacket::JoinResponse {
                            result: JoinResult::RejectedNameTaken,
                            room_name: self.room_name.clone(),
                            track_id: self.track_id.clone(),
                            laps: self.laps,
                            slots: self.active_slots(),
                        },
                        src_addr,
                    );
                    return;
                }

                // Find free slot
                let free_slot_idx = self.slots.iter().position(|s| s.is_none());
                match free_slot_idx {
                    Some(idx) => {
                        let slot_id = idx as u8;
                        let new_slot = LobbySlot {
                            slot_id,
                            player_name: sanitized_name.clone(),
                            country_code,
                            car_model_id: car_model_id.clone(),
                            color_scheme_id,
                            is_ready: false,
                            is_host: false,
                            ping_ms: 0,
                        };

                        self.slots[idx] = Some(new_slot);
                        self.clients.push(ConnectedClient {
                            slot_id,
                            addr: src_addr,
                            last_seen_sec: 0.0,
                            ping_ms: 0,
                        });

                        self.sync_beacon();

                        // Respond to client
                        let _ = self.send_to_addr(
                            &LobbyPacket::JoinResponse {
                                result: JoinResult::Accepted { slot_id },
                                room_name: self.room_name.clone(),
                                track_id: self.track_id.clone(),
                                laps: self.laps,
                                slots: self.active_slots(),
                            },
                            src_addr,
                        );

                        // Broadcast new slot to all
                        let _ = self.broadcast_lobby_state();

                        events.push(HostEvent::PlayerJoined {
                            slot_id,
                            player_name: sanitized_name,
                            car_model_id,
                        });
                    }
                    None => {
                        let _ = self.send_to_addr(
                            &LobbyPacket::JoinResponse {
                                result: JoinResult::RejectedFull,
                                room_name: self.room_name.clone(),
                                track_id: self.track_id.clone(),
                                laps: self.laps,
                                slots: self.active_slots(),
                            },
                            src_addr,
                        );
                    }
                }
            }

            Packet::Lobby(LobbyPacket::ClientSlotUpdate {
                slot_id,
                car_model_id,
                color_scheme_id,
                is_ready,
            }) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.addr == src_addr) {
                    client.last_seen_sec = 0.0;
                    if client.slot_id == slot_id && (slot_id as usize) < self.slots.len() {
                        if let Some(Some(ref mut slot)) = self.slots.get_mut(slot_id as usize) {
                            slot.car_model_id = car_model_id.clone();
                            slot.color_scheme_id = color_scheme_id.clone();
                            slot.is_ready = is_ready;

                            let _ = self.broadcast_lobby_state();

                            events.push(HostEvent::PlayerSlotUpdated {
                                slot_id,
                                car_model_id,
                                color_scheme_id,
                                is_ready,
                            });
                        }
                    }
                }
            }

            Packet::Lobby(LobbyPacket::Ping { timestamp_ms }) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.addr == src_addr) {
                    client.last_seen_sec = 0.0;
                }
                let pong = LobbyPacket::Pong { timestamp_ms };
                let _ = self.send_to_addr(&pong, src_addr);
            }

            Packet::Lobby(LobbyPacket::Pong { timestamp_ms }) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.addr == src_addr) {
                    client.last_seen_sec = 0.0;
                    let now = current_time_ms();
                    let rtt = now.saturating_sub(timestamp_ms).min(999) as u16;
                    client.ping_ms = rtt;

                    if let Some(Some(ref mut slot)) = self.slots.get_mut(client.slot_id as usize) {
                        slot.ping_ms = rtt;
                    }
                }
            }

            Packet::Lobby(LobbyPacket::DisconnectNotice { reason }) => {
                if let Some(idx) = self.clients.iter().position(|c| c.addr == src_addr) {
                    let client = self.clients.remove(idx);
                    let slot_id = client.slot_id;
                    if (slot_id as usize) < self.slots.len() {
                        self.slots[slot_id as usize] = None;
                    }
                    self.sync_beacon();
                    let _ = self.broadcast_lobby_state();
                    events.push(HostEvent::PlayerLeft { slot_id, reason });
                }
            }

            Packet::Input(input) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.addr == src_addr) {
                    client.last_seen_sec = 0.0;
                    if client.slot_id == input.slot_id {
                        events.push(HostEvent::PlayerInput {
                            slot_id: input.slot_id,
                            input,
                        });
                    }
                }
            }

            _ => {}
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
        let state_packet = LobbyPacket::StateSync {
            track_id: self.track_id.clone(),
            laps: self.laps,
            collision_mode: self.collision_mode,
            slots: self.active_slots(),
        };
        self.broadcast_lobby_packet(&state_packet)
    }

    fn broadcast_lobby_packet(&self, packet: &LobbyPacket) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        for client in &self.clients {
            let _ = self.socket.send_to(&encoded, client.addr);
        }
        Ok(())
    }

    fn send_to_addr(&self, packet: &LobbyPacket, addr: SocketAddr) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        let _ = self.socket.send_to(&encoded, addr);
        Ok(())
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
