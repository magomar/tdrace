//! # Protocol & Core Datagrams for Cabinet LAN Multiplayer
//!
//! Defines the wire protocol constants, packet schemas, error handling,
//! sanitization utilities, and serialization / deserialization codec
//! for local area network (LAN) arcade play.

use serde::{Deserialize, Serialize};

/// Magic header bytes identifying TdRace / Cabinet LAN packets: "TDLN" (0x54, 0x44, 0x4C, 0x4E).
pub const MAGIC_BYTES: [u8; 4] = [0x54, 0x44, 0x4C, 0x4E];

/// Alias for [`MAGIC_BYTES`].
pub const LAN_MAGIC: [u8; 4] = MAGIC_BYTES;

/// Current supported wire protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Default UDP port for local subnet discovery beacons.
pub const DEFAULT_BEACON_PORT: u16 = 7776;

/// Default UDP port for authoritative game session traffic.
pub const DEFAULT_GAME_PORT: u16 = 7777;

/// Maximum safe datagram size in bytes (safely below 1,500 Ethernet MTU).
pub const MAX_DATAGRAM_SIZE: usize = 1400;

/// Maximum length for player names and room titles.
pub const MAX_NAME_LENGTH: usize = 24;

/// Errors that can occur during datagram serialization, deserialization, or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// Packet size exceeds maximum MTU bound.
    PacketTooLarge(usize),
    /// Packet is too short to contain the magic header and protocol version.
    PacketTooShort(usize),
    /// Packet magic header does not match expected [`MAGIC_BYTES`].
    InvalidMagic([u8; 4]),
    /// Packet protocol version does not match [`PROTOCOL_VERSION`].
    VersionMismatch(u8),
    /// Serialization failed.
    SerializationFailed(String),
    /// Deserialization failed.
    DeserializationFailed(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PacketTooLarge(sz) => write!(
                f,
                "Packet size {} exceeds maximum datagram bound ({})",
                sz, MAX_DATAGRAM_SIZE
            ),
            Self::PacketTooShort(sz) => {
                write!(f, "Packet size {} is too short to contain header", sz)
            }
            Self::InvalidMagic(m) => write!(
                f,
                "Invalid packet magic header: {:?} (expected {:?})",
                m, MAGIC_BYTES
            ),
            Self::VersionMismatch(v) => write!(
                f,
                "Protocol version mismatch: {} (expected {})",
                v, PROTOCOL_VERSION
            ),
            Self::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            Self::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Clamps string to `max_len`, strips control characters and ANSI escape sequences,
/// and trims leading/trailing whitespace.
pub fn sanitize_string(input: &str, max_len: usize) -> String {
    let filtered: String = input
        .chars()
        .filter(|c| !c.is_control() && *c != '\u{001b}')
        .collect();
    let trimmed = filtered.trim();
    if trimmed.chars().count() > max_len {
        trimmed.chars().take(max_len).collect()
    } else {
        trimmed.to_string()
    }
}

/// Periodic discovery beacon packet broadcasted across UDP port 7776.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanBeacon {
    /// Magic header identifier (`"TDLN"`).
    pub magic: [u8; 4],
    /// Protocol version.
    pub protocol_version: u8,
    /// Human-readable server or lobby room title.
    pub server_name: String,
    /// Host player display name.
    pub host_player_name: String,
    /// Port number the game session is listening on (e.g. 7777).
    pub game_port: u16,
    /// Selected track identifier.
    pub track_id: String,
    /// Racing discipline / car class.
    pub discipline: String,
    /// Current number of joined players.
    pub current_players: u8,
    /// Maximum allowed players in this lobby.
    pub max_players: u8,
    /// Whether the lobby is password-protected or locked.
    pub is_locked: bool,
}

impl LanBeacon {
    /// Creates a new sanitized beacon with standard magic header and protocol version.
    pub fn new(
        server_name: String,
        host_player_name: String,
        game_port: u16,
        track_id: String,
        discipline: String,
        current_players: u8,
        max_players: u8,
        is_locked: bool,
    ) -> Self {
        Self {
            magic: MAGIC_BYTES,
            protocol_version: PROTOCOL_VERSION,
            server_name: sanitize_string(&server_name, MAX_NAME_LENGTH),
            host_player_name: sanitize_string(&host_player_name, MAX_NAME_LENGTH),
            game_port,
            track_id,
            discipline,
            current_players,
            max_players,
            is_locked,
        }
    }

    /// Encodes this beacon into a wire-formatted datagram byte buffer.
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        Packet::Beacon(self.clone()).encode()
    }

    /// Decodes a beacon from a wire-formatted datagram byte buffer.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match Packet::decode(bytes)? {
            Packet::Beacon(beacon) => Ok(beacon),
            _ => Err(ProtocolError::DeserializationFailed(
                "Packet is not a beacon".to_string(),
            )),
        }
    }
}

/// Result code for a player join request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinResult {
    /// Join request accepted, assigned to given slot.
    Accepted { slot_id: u8 },
    /// Lobby is full.
    RejectedFull,
    /// Protocol version does not match host.
    RejectedVersionMismatch,
    /// Race is already in progress.
    RejectedGameInProgress,
    /// Player display name is already taken in this lobby.
    RejectedNameTaken,
}

/// Collision resolution setting for a LAN race session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LanCollisionMode {
    /// Full rigid body Separating Axis Theorem (SAT) collision resolution.
    #[default]
    FullSatSolid,
    /// Ghost passing without vehicle-to-vehicle contact.
    GhostPassing,
    /// Collisions enabled only against track verges and barriers.
    VergeOnly,
}

/// Represents one participant slot in the LAN lobby room.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LobbySlot {
    /// Slot index (0..7).
    pub slot_id: u8,
    /// Player display name.
    pub player_name: String,
    /// Three-letter ISO country code or regional flag identifier (e.g. "ESP", "FRA").
    pub country_code: String,
    /// Vehicle model identifier.
    pub car_model_id: String,
    /// Livery / color scheme identifier.
    pub color_scheme_id: String,
    /// Whether this player is ready to start.
    pub is_ready: bool,
    /// Whether this slot belongs to the session host.
    pub is_host: bool,
    /// Measured round-trip ping in milliseconds.
    pub ping_ms: u16,
}

impl LobbySlot {
    /// Creates a new lobby slot with default vehicle choices.
    pub fn new(slot_id: u8, player_name: String, country_code: String, is_host: bool) -> Self {
        Self {
            slot_id,
            player_name: sanitize_string(&player_name, MAX_NAME_LENGTH),
            country_code,
            car_model_id: "scuderia_gt".to_string(),
            color_scheme_id: "red".to_string(),
            is_ready: is_host,
            is_host,
            ping_ms: 0,
        }
    }
}

/// Packets exchanged during the lobby staging and synchronization phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyPacket {
    /// Sent by connecting client to request admission into the lobby.
    JoinRequest {
        protocol_version: u8,
        player_name: String,
        country_code: String,
        car_model_id: String,
        color_scheme_id: String,
    },
    /// Host response to client join request.
    JoinResponse {
        result: JoinResult,
        room_name: String,
        track_id: String,
        laps: u8,
        slots: Vec<LobbySlot>,
    },
    /// Periodic or event-triggered full lobby state broadcast from host.
    StateSync {
        track_id: String,
        laps: u8,
        collision_mode: LanCollisionMode,
        slots: Vec<LobbySlot>,
    },
    /// Client-to-host update when changing vehicle, livery, or ready flag.
    ClientSlotUpdate {
        slot_id: u8,
        car_model_id: String,
        color_scheme_id: String,
        is_ready: bool,
    },
    /// Host notification that synchronized grid launch is starting.
    LaunchCountdown {
        starts_in_millis: u32,
        grid_positions: Vec<u8>,
    },
    /// Latency measurement probe.
    Ping { timestamp_ms: u64 },
    /// Latency measurement response.
    Pong { timestamp_ms: u64 },
    /// Notification that client has disconnected or been kicked.
    DisconnectNotice { reason: String },
}

impl LobbyPacket {
    /// Encodes this lobby packet into a wire-formatted datagram byte buffer.
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        Packet::Lobby(self.clone()).encode()
    }

    /// Decodes a lobby packet from a wire-formatted datagram byte buffer.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match Packet::decode(bytes)? {
            Packet::Lobby(packet) => Ok(packet),
            _ => Err(ProtocolError::DeserializationFailed(
                "Packet is not a lobby packet".to_string(),
            )),
        }
    }
}

/// 60 Hz input frame streamed from client to authoritative host during race.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInputPacket {
    /// Monotonically increasing client input sequence number.
    pub sequence_num: u32,
    /// Assigned player slot index.
    pub slot_id: u8,
    /// Steering command: -1.0 (full left) to +1.0 (full right).
    pub steering: f32,
    /// Throttle pedal: 0.0 (idle) to 1.0 (full throttle).
    pub throttle: f32,
    /// Brake pedal: 0.0 (idle) to 1.0 (full brake).
    pub brake: f32,
    /// Handbrake flag.
    pub handbrake: bool,
    /// Reverse gear flag.
    #[serde(default)]
    pub reverse: bool,
}

impl ClientInputPacket {
    /// Encodes this input packet into a wire-formatted datagram byte buffer.
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        Packet::Input(self.clone()).encode()
    }

    /// Decodes an input packet from a wire-formatted datagram byte buffer.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match Packet::decode(bytes)? {
            Packet::Input(packet) => Ok(packet),
            _ => Err(ProtocolError::DeserializationFailed(
                "Packet is not an input packet".to_string(),
            )),
        }
    }
}

/// Kinematic snapshot of a single car transmitted by the authoritative host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarStateSnapshot {
    /// Participant slot index.
    pub slot_id: u8,
    /// World position X coordinate in meters.
    pub pos_x: f32,
    /// World position Y coordinate in meters.
    pub pos_y: f32,
    /// Linear velocity X in m/s.
    pub velocity_x: f32,
    /// Linear velocity Y in m/s.
    pub velocity_y: f32,
    /// Heading orientation angle in radians.
    pub heading_rad: f32,
    /// Angular yaw velocity in rad/s.
    pub angular_velocity: f32,
    /// Current front steer angle in radians.
    pub steer_angle_rad: f32,
    /// Current completed lap count.
    pub current_lap: u16,
    /// Most recent track checkpoint index passed.
    pub checkpoint_idx: u16,
    /// Best lap time in milliseconds, if recorded.
    pub best_lap_time_ms: Option<u32>,
    /// Last completed lap time in milliseconds, if recorded.
    pub last_lap_time_ms: Option<u32>,
    /// Whether the car has completed the race distance.
    pub is_finished: bool,
}

/// 60 Hz authoritative world snapshot broadcasted from host to all peers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshotPacket {
    /// Monotonically increasing server simulation tick.
    pub tick: u32,
    /// Elapsed race session time in seconds.
    pub session_elapsed_sec: f32,
    /// Kinematic states for all active cars.
    pub cars: Vec<CarStateSnapshot>,
}

impl WorldSnapshotPacket {
    /// Encodes this world snapshot packet into a wire-formatted datagram byte buffer.
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        Packet::Snapshot(self.clone()).encode()
    }

    /// Decodes a world snapshot packet from a wire-formatted datagram byte buffer.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match Packet::decode(bytes)? {
            Packet::Snapshot(packet) => Ok(packet),
            _ => Err(ProtocolError::DeserializationFailed(
                "Packet is not a world snapshot packet".to_string(),
            )),
        }
    }
}

/// Top-level wire packet envelope for Cabinet LAN datagrams.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    /// Discovery beacon packet.
    Beacon(LanBeacon),
    /// Staging lobby coordination packet.
    Lobby(LobbyPacket),
    /// Client-to-host driver input packet.
    Input(ClientInputPacket),
    /// Authoritative host-to-client world snapshot packet.
    Snapshot(WorldSnapshotPacket),
    /// Direct peer disconnection packet.
    Disconnect {
        /// Disconnecting slot index.
        slot_id: u8,
        /// Reason for disconnect.
        reason: String,
    },
}

impl Packet {
    /// Encodes this packet into a wire-formatted datagram byte buffer.
    ///
    /// The wire format consists of:
    /// - 4 bytes: Magic header [`MAGIC_BYTES`] (`b"TDLN"`)
    /// - 1 byte:  Protocol version [`PROTOCOL_VERSION`]
    /// - N bytes: Serialized payload
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let payload = serde_json::to_vec(self)
            .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        let total_len = 5 + payload.len();
        if total_len > MAX_DATAGRAM_SIZE {
            return Err(ProtocolError::PacketTooLarge(total_len));
        }
        let mut buf = Vec::with_capacity(total_len);
        buf.extend_from_slice(&MAGIC_BYTES);
        buf.push(PROTOCOL_VERSION);
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Decodes and validates a packet from raw received UDP datagram bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_DATAGRAM_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        if bytes.len() < 5 {
            return Err(ProtocolError::PacketTooShort(bytes.len()));
        }
        if bytes[0..4] != MAGIC_BYTES {
            return Err(ProtocolError::InvalidMagic([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]));
        }
        if bytes[4] != PROTOCOL_VERSION {
            return Err(ProtocolError::VersionMismatch(bytes[4]));
        }
        serde_json::from_slice(&bytes[5..])
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }

    /// Returns a reference to the inner [`LobbyPacket`] if this is a lobby packet.
    pub fn as_lobby(&self) -> Option<&LobbyPacket> {
        match self {
            Self::Lobby(p) => Some(p),
            _ => None,
        }
    }

    /// Returns a reference to the inner [`ClientInputPacket`] if this is an input packet.
    pub fn as_input(&self) -> Option<&ClientInputPacket> {
        match self {
            Self::Input(p) => Some(p),
            _ => None,
        }
    }

    /// Returns a reference to the inner [`WorldSnapshotPacket`] if this is a snapshot packet.
    pub fn as_snapshot(&self) -> Option<&WorldSnapshotPacket> {
        match self {
            Self::Snapshot(p) => Some(p),
            _ => None,
        }
    }

    /// Returns a reference to the inner [`LanBeacon`] if this is a beacon packet.
    pub fn as_beacon(&self) -> Option<&LanBeacon> {
        match self {
            Self::Beacon(b) => Some(b),
            _ => None,
        }
    }
}

impl From<LanBeacon> for Packet {
    fn from(b: LanBeacon) -> Self {
        Self::Beacon(b)
    }
}

impl From<LobbyPacket> for Packet {
    fn from(p: LobbyPacket) -> Self {
        Self::Lobby(p)
    }
}

impl From<ClientInputPacket> for Packet {
    fn from(p: ClientInputPacket) -> Self {
        Self::Input(p)
    }
}

impl From<WorldSnapshotPacket> for Packet {
    fn from(p: WorldSnapshotPacket) -> Self {
        Self::Snapshot(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_and_magic_header() {
        assert_eq!(MAGIC_BYTES, [0x54, 0x44, 0x4C, 0x4E]);
        assert_eq!(LAN_MAGIC, MAGIC_BYTES);
        assert_eq!(&MAGIC_BYTES, b"TDLN");
        assert_eq!(PROTOCOL_VERSION, 1);
        assert_eq!(DEFAULT_BEACON_PORT, 7776);
        assert_eq!(DEFAULT_GAME_PORT, 7777);
        assert_eq!(MAX_DATAGRAM_SIZE, 1400);
        assert_eq!(MAX_NAME_LENGTH, 24);
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  Mario  ", 24), "Mario");
        assert_eq!(
            sanitize_string("SuperLongPlayerNameExceedingTwentyFourChars", 24),
            "SuperLongPlayerNameExcee"
        );
        assert_eq!(sanitize_string("Player\x1b[31mRed\x1b[0m", 24), "Player[31mRed[0m");
        assert_eq!(sanitize_string("Line1\nLine2\tTab", 24), "Line1Line2Tab");
        assert_eq!(sanitize_string("", 24), "");
    }

    #[test]
    fn test_lan_beacon_roundtrip() {
        let beacon = LanBeacon::new(
            "Mario's Grand Prix".to_string(),
            "Mario".to_string(),
            7777,
            "spa_francorchamps".to_string(),
            "GT3".to_string(),
            3,
            8,
            false,
        );

        let encoded = beacon.encode().expect("Failed to encode beacon");
        assert_eq!(&encoded[0..4], b"TDLN");
        assert_eq!(encoded[4], 1);

        let decoded = LanBeacon::decode(&encoded).expect("Failed to decode beacon");
        assert_eq!(beacon, decoded);
    }

    #[test]
    fn test_lobby_packet_roundtrips() {
        let packets = vec![
            LobbyPacket::JoinRequest {
                protocol_version: 1,
                player_name: "Alex".to_string(),
                country_code: "FRA".to_string(),
                car_model_id: "gt3_viper".to_string(),
                color_scheme_id: "green".to_string(),
            },
            LobbyPacket::JoinResponse {
                result: JoinResult::Accepted { slot_id: 1 },
                room_name: "Mario GP".to_string(),
                track_id: "monza".to_string(),
                laps: 5,
                slots: vec![
                    LobbySlot::new(0, "Mario".to_string(), "ESP".to_string(), true),
                    LobbySlot::new(1, "Alex".to_string(), "FRA".to_string(), false),
                ],
            },
            LobbyPacket::StateSync {
                track_id: "spa".to_string(),
                laps: 3,
                collision_mode: LanCollisionMode::FullSatSolid,
                slots: vec![LobbySlot::new(
                    0,
                    "Mario".to_string(),
                    "ESP".to_string(),
                    true,
                )],
            },
            LobbyPacket::ClientSlotUpdate {
                slot_id: 1,
                car_model_id: "m4_gt3".to_string(),
                color_scheme_id: "black".to_string(),
                is_ready: true,
            },
            LobbyPacket::LaunchCountdown {
                starts_in_millis: 3000,
                grid_positions: vec![0, 1],
            },
            LobbyPacket::Ping {
                timestamp_ms: 123456789,
            },
            LobbyPacket::Pong {
                timestamp_ms: 123456789,
            },
            LobbyPacket::DisconnectNotice {
                reason: "Host disbanded lobby".to_string(),
            },
        ];

        for pkt in packets {
            let encoded = pkt.encode().expect("Encode should succeed");
            let decoded = LobbyPacket::decode(&encoded).expect("Decode should succeed");
            assert_eq!(pkt, decoded);
        }
    }

    #[test]
    fn test_client_input_packet_roundtrip() {
        let input = ClientInputPacket {
            sequence_num: 42,
            slot_id: 2,
            steering: -0.75,
            throttle: 1.0,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        };

        let encoded = input.encode().expect("Encode input");
        let decoded = ClientInputPacket::decode(&encoded).expect("Decode input");
        assert_eq!(input, decoded);
    }

    #[test]
    fn test_world_snapshot_packet_roundtrip() {
        let snapshot = WorldSnapshotPacket {
            tick: 1800,
            session_elapsed_sec: 30.0,
            cars: vec![
                CarStateSnapshot {
                    slot_id: 0,
                    pos_x: 100.5,
                    pos_y: 250.2,
                    velocity_x: 45.0,
                    velocity_y: 12.0,
                    heading_rad: 1.57,
                    angular_velocity: 0.02,
                    steer_angle_rad: -0.1,
                    current_lap: 2,
                    checkpoint_idx: 15,
                    best_lap_time_ms: Some(82500),
                    last_lap_time_ms: Some(83100),
                    is_finished: false,
                },
                CarStateSnapshot {
                    slot_id: 1,
                    pos_x: 95.0,
                    pos_y: 248.0,
                    velocity_x: 44.5,
                    velocity_y: 11.8,
                    heading_rad: 1.55,
                    angular_velocity: -0.01,
                    steer_angle_rad: 0.05,
                    current_lap: 2,
                    checkpoint_idx: 14,
                    best_lap_time_ms: Some(83200),
                    last_lap_time_ms: None,
                    is_finished: false,
                },
            ],
        };

        let encoded = snapshot.encode().expect("Encode snapshot");
        let decoded = WorldSnapshotPacket::decode(&encoded).expect("Decode snapshot");
        assert_eq!(snapshot, decoded);
    }

    #[test]
    fn test_reject_invalid_magic() {
        let mut encoded = Packet::Disconnect {
            slot_id: 1,
            reason: "Leave".to_string(),
        }
        .encode()
        .expect("Encode");

        encoded[0] = b'X';
        encoded[1] = b'X';
        let err = Packet::decode(&encoded).unwrap_err();
        match err {
            ProtocolError::InvalidMagic(m) => assert_eq!(&m[0..2], b"XX"),
            _ => panic!("Expected InvalidMagic error"),
        }
    }

    #[test]
    fn test_reject_version_mismatch() {
        let mut encoded = Packet::Disconnect {
            slot_id: 1,
            reason: "Leave".to_string(),
        }
        .encode()
        .expect("Encode");

        encoded[4] = 99; // corrupt version
        let err = Packet::decode(&encoded).unwrap_err();
        assert_eq!(err, ProtocolError::VersionMismatch(99));
    }

    #[test]
    fn test_reject_short_packet() {
        let short_bytes = vec![b'T', b'D', b'L'];
        let err = Packet::decode(&short_bytes).unwrap_err();
        assert_eq!(err, ProtocolError::PacketTooShort(3));
    }

    #[test]
    fn test_reject_oversized_packet() {
        let oversized = vec![0u8; MAX_DATAGRAM_SIZE + 1];
        let err = Packet::decode(&oversized).unwrap_err();
        assert_eq!(err, ProtocolError::PacketTooLarge(MAX_DATAGRAM_SIZE + 1));
    }

    #[test]
    fn test_packet_type_accessors_and_conversions() {
        let beacon = LanBeacon::new(
            "Test".to_string(),
            "Mario".to_string(),
            7777,
            "track1".to_string(),
            "GT".to_string(),
            1,
            4,
            false,
        );
        let pkt: Packet = beacon.clone().into();
        assert_eq!(pkt.as_beacon(), Some(&beacon));
        assert!(pkt.as_lobby().is_none());
        assert!(pkt.as_input().is_none());
        assert!(pkt.as_snapshot().is_none());
    }
}
