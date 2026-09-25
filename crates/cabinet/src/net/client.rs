//! # Cabinet LAN Client Subsystem
//!
//! Manages the client-side socket, handshake with authoritative host,
//! lobby state caching, input datagram streaming, and snapshot reception.

use std::io;
use std::net::{SocketAddr, UdpSocket};

use super::protocol::{
    sanitize_string, ClientInputPacket, JoinResult, LanCollisionMode,
    LobbyPacket, LobbySlot, Packet, ProtocolError, WorldSnapshotPacket,
    MAX_DATAGRAM_SIZE, MAX_NAME_LENGTH, PROTOCOL_VERSION,
};

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
    /// Host initiated 3-2-1 countdown.
    StartingCountdown {
        starts_in_millis: u32,
        remaining_sec: f32,
        grid_positions: Vec<u8>,
    },
    /// Active in-race simulation.
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
    /// Host started the launch countdown.
    CountdownStarted {
        starts_in_millis: u32,
        grid_positions: Vec<u8>,
    },
    /// In-race authoritative world snapshot received from host.
    WorldSnapshot(WorldSnapshotPacket),
    /// Client disconnected or was rejected/kicked by host.
    Disconnected(String),
}

/// Client endpoint for connecting to and participating in LAN games.
pub struct LanClient {
    socket: UdpSocket,
    host_addr: SocketAddr,
    state: ClientState,
    player_name: String,
    country_code: String,
    car_model_id: String,
    color_scheme_id: String,
    last_seen_sec: f32,
    ping_timer_sec: f32,
    ping_interval_sec: f32,
    ping_ms: u16,
    input_seq: u32,
    timeout_sec: f32,
    recv_buf: [u8; MAX_DATAGRAM_SIZE],
    last_known_track_id: String,
    last_known_laps: u8,
    last_known_slots: Vec<LobbySlot>,
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
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        let player_name = sanitize_string(&player_name.into(), MAX_NAME_LENGTH);
        let country_code = country_code.into();
        let car_model_id = car_model_id.into();
        let color_scheme_id = color_scheme_id.into();

        let mut client = Self {
            socket,
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
            last_seen_sec: 0.0,
            ping_timer_sec: 0.0,
            ping_interval_sec: 1.0,
            ping_ms: 0,
            input_seq: 0,
            timeout_sec: 4.5,
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
            last_known_track_id: "monza".to_string(),
            last_known_laps: 5,
            last_known_slots: Vec::new(),
        };

        client.send_join_request()?;
        Ok(client)
    }

    /// Current connection state.
    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// Whether this client is actively connected (InLobby, StartingCountdown, or InRace).
    pub fn is_connected(&self) -> bool {
        matches!(
            self.state,
            ClientState::InLobby { .. }
                | ClientState::StartingCountdown { .. }
                | ClientState::InRace { .. }
        )
    }

    /// Assigned slot ID in the room, if connected.
    pub fn assigned_slot_id(&self) -> Option<u8> {
        match self.state {
            ClientState::InLobby { assigned_slot_id, .. } => Some(assigned_slot_id),
            ClientState::StartingCountdown { .. } => {
                // Preserved from lobby
                Some(1)
            }
            ClientState::InRace { assigned_slot_id } => Some(assigned_slot_id),
            _ => None,
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

        let packet = LobbyPacket::ClientSlotUpdate {
            slot_id,
            car_model_id,
            color_scheme_id,
            is_ready,
        };

        self.send_to_host(&packet)
    }

    /// Streams an in-race 60 Hz input frame to the authoritative host.
    pub fn send_input(
        &mut self,
        steering: f32,
        throttle: f32,
        brake: f32,
        handbrake: bool,
        reverse: bool,
    ) -> Result<(), ProtocolError> {
        let slot_id = match self.assigned_slot_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        self.input_seq = self.input_seq.wrapping_add(1);
        let packet = ClientInputPacket {
            sequence_num: self.input_seq,
            slot_id,
            steering,
            throttle,
            brake,
            handbrake,
            reverse,
        };

        let encoded = packet.encode()?;
        let _ = self.socket.send_to(&encoded, self.host_addr);
        Ok(())
    }

    /// Sends a graceful disconnect notice to the host and resets state.
    pub fn disconnect(&mut self) -> Result<(), ProtocolError> {
        let notice = LobbyPacket::DisconnectNotice {
            reason: "Player left room".to_string(),
        };
        let _ = self.send_to_host(&notice);
        self.state = ClientState::Disconnected(Some("Disconnected by user".to_string()));
        Ok(())
    }

    /// Updates internal timers, processes incoming datagrams, and checks timeouts.
    pub fn update(&mut self, dt: f32) -> Vec<ClientEvent> {
        let mut events = Vec::new();

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
                if *retries > 4 {
                    self.state = ClientState::Disconnected(Some("Host connection timed out".to_string()));
                    events.push(ClientEvent::Disconnected("Host did not respond".to_string()));
                    return events;
                } else {
                    let _ = self.send_join_request();
                }
            }
        }

        // 2. Countdown timer advancement
        if let ClientState::StartingCountdown {
            starts_in_millis: _,
            ref mut remaining_sec,
            grid_positions: _,
        } = self.state
        {
            *remaining_sec -= dt;
            if *remaining_sec <= 0.0 {
                let slot = self.assigned_slot_id().unwrap_or(1);
                self.state = ClientState::InRace { assigned_slot_id: slot };
            }
        }

        // 3. Heartbeat ping transmission
        if self.is_connected() {
            self.ping_timer_sec += dt;
            if self.ping_timer_sec >= self.ping_interval_sec {
                self.ping_timer_sec = 0.0;
                let now_ms = current_time_ms();
                let ping = LobbyPacket::Ping { timestamp_ms: now_ms };
                let _ = self.send_to_host(&ping);
            }
        }

        // 4. Pump incoming packets
        loop {
            match self.socket.recv_from(&mut self.recv_buf) {
                Ok((bytes_read, src_addr)) => {
                    if src_addr == self.host_addr {
                        self.last_seen_sec = 0.0;
                        if let Ok(packet) = Packet::decode(&self.recv_buf[..bytes_read]) {
                            self.handle_packet(packet, &mut events);
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // 5. Host silence timeout
        if self.is_connected() && self.last_seen_sec > self.timeout_sec {
            self.state = ClientState::Disconnected(Some("Lost connection to host".to_string()));
            events.push(ClientEvent::Disconnected("Host timed out".to_string()));
        }

        events
    }

    fn handle_packet(&mut self, packet: Packet, events: &mut Vec<ClientEvent>) {
        match packet {
            Packet::Lobby(LobbyPacket::JoinResponse {
                result,
                room_name,
                track_id,
                laps,
                slots,
            }) => match result {
                JoinResult::Accepted { slot_id } => {
                    self.last_known_track_id = track_id.clone();
                    self.last_known_laps = laps;
                    self.last_known_slots = slots.clone();
                    self.state = ClientState::InLobby {
                        assigned_slot_id: slot_id,
                        room_name: room_name.clone(),
                        track_id: track_id.clone(),
                        laps,
                        collision_mode: LanCollisionMode::default(),
                        slots,
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

            Packet::Lobby(LobbyPacket::StateSync {
                track_id,
                laps,
                collision_mode,
                slots,
            }) => {
                self.last_known_track_id = track_id.clone();
                self.last_known_laps = laps;
                self.last_known_slots = slots.clone();
                if let ClientState::InLobby {
                    assigned_slot_id,
                    ref room_name,
                    ..
                } = self.state
                {
                    self.state = ClientState::InLobby {
                        assigned_slot_id,
                        room_name: room_name.clone(),
                        track_id: track_id.clone(),
                        laps,
                        collision_mode,
                        slots: slots.clone(),
                    };
                    events.push(ClientEvent::LobbyUpdated {
                        track_id,
                        laps,
                        slots,
                    });
                }
            }

            Packet::Lobby(LobbyPacket::LaunchCountdown {
                starts_in_millis,
                grid_positions,
            }) => {
                let remaining_sec = starts_in_millis as f32 / 1000.0;
                self.state = ClientState::StartingCountdown {
                    starts_in_millis,
                    remaining_sec,
                    grid_positions: grid_positions.clone(),
                };
                events.push(ClientEvent::CountdownStarted {
                    starts_in_millis,
                    grid_positions,
                });
            }

            Packet::Lobby(LobbyPacket::Ping { timestamp_ms }) => {
                let pong = LobbyPacket::Pong { timestamp_ms };
                let _ = self.send_to_host(&pong);
            }

            Packet::Lobby(LobbyPacket::Pong { timestamp_ms }) => {
                let now = current_time_ms();
                self.ping_ms = now.saturating_sub(timestamp_ms).min(999) as u16;
            }

            Packet::Lobby(LobbyPacket::DisconnectNotice { reason }) => {
                self.state = ClientState::Disconnected(Some(reason.clone()));
                events.push(ClientEvent::Disconnected(reason));
            }

            Packet::Snapshot(snapshot) => {
                events.push(ClientEvent::WorldSnapshot(snapshot));
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

    fn send_to_host(&self, packet: &LobbyPacket) -> Result<(), ProtocolError> {
        let encoded = packet.encode()?;
        let _ = self.socket.send_to(&encoded, self.host_addr);
        Ok(())
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
