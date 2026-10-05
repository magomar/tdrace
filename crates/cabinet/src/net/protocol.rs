//! # Protocol & Core Datagrams for Cabinet LAN Multiplayer
//!
//! Defines the wire protocol constants, packet schemas, error handling,
//! sanitization utilities, and serialization / deserialization codec
//! for local area network (LAN) arcade play.

use serde::{Deserialize, Serialize};

use super::wire;

/// Magic header bytes identifying TdRace / Cabinet LAN packets: "TDLN" (0x54, 0x44, 0x4C, 0x4E).
pub const MAGIC_BYTES: [u8; 4] = [0x54, 0x44, 0x4C, 0x4E];

/// Alias for [`MAGIC_BYTES`].
pub const LAN_MAGIC: [u8; 4] = MAGIC_BYTES;

/// Current supported wire protocol version.
pub const PROTOCOL_VERSION: u8 = 3;

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
            car_model_id: "gt_valente_corsa_t2".to_string(),
            color_scheme_id: "red".to_string(),
            is_ready: is_host,
            is_host,
            ping_ms: 0,
        }
    }
}

/// Unreliable handshake and heartbeat packets. Lobby and race state changes
/// travel as reliable [`ControlMessage`]s instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LobbyPacket {
    /// Sent by connecting client to request admission into the lobby.
    JoinRequest {
        protocol_version: u8,
        player_name: String,
        country_code: String,
        car_model_id: String,
        color_scheme_id: String,
    },
    /// Host response to client join request. The roster follows as a reliable `StateSync`.
    JoinResponse {
        result: JoinResult,
        room_name: String,
        track_id: String,
        laps: u8,
    },
    /// Latency and clock probe; `sent_at` is the sender's monotonic clock in seconds.
    Ping { sent_at: f64 },
    /// Probe answer; `responder_time` is the responder's monotonic clock in seconds.
    Pong { sent_at: f64, responder_time: f64 },
    /// Notification that client has disconnected or been kicked.
    DisconnectNotice { reason: String },
}

/// One car of the frozen race roster sent in [`ControlMessage::RaceLaunch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterEntry {
    pub slot_id: u8,
    /// Starting grid position, `0` = pole. Also the car index in the race.
    pub grid_index: u8,
    pub player_name: String,
    pub country_code: String,
    pub car_model_id: String,
    pub color_scheme_id: String,
}

/// Complete race setup. Every machine builds its car list only from this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceConfig {
    pub track_id: String,
    pub laps: u8,
    pub collision_mode: LanCollisionMode,
    /// Sorted by `grid_index`.
    pub roster: Vec<RosterEntry>,
}

impl RaceConfig {
    /// Car index of `slot_id`, if it is in the roster.
    pub fn car_index_of(&self, slot_id: u8) -> Option<usize> {
        self.roster.iter().position(|e| e.slot_id == slot_id)
    }
}

/// One finisher as recorded by the host referee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishRecord {
    pub slot_id: u8,
    /// Shared race clock at the finish line, in milliseconds.
    pub finish_ms: u32,
    pub best_lap_ms: Option<u32>,
    /// Time penalty the car's owner added at the finish (a missed joker lap), in milliseconds. Results are
    /// ordered by `finish_ms + penalty_ms`.
    pub penalty_ms: u32,
}

/// Final status of one car.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceStatus {
    Finished,
    /// Still racing when the race closed.
    Dnf,
    /// Left or lost connection during the race.
    Left,
}

/// One row of the host's final results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceResult {
    pub slot_id: u8,
    /// `1` = winner.
    pub position: u8,
    pub status: RaceStatus,
    pub finish_ms: Option<u32>,
    pub best_lap_ms: Option<u32>,
    /// See [`FinishRecord::penalty_ms`]; not included in `finish_ms`.
    pub penalty_ms: u32,
}

/// Lobby and race-control messages, always sent on the reliable channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Host → clients: full lobby state. `roster_rev` grows with every change.
    StateSync {
        roster_rev: u32,
        track_id: String,
        laps: u8,
        collision_mode: LanCollisionMode,
        slots: Vec<LobbySlot>,
    },
    /// Client → host: vehicle, livery or ready change.
    ClientSlotUpdate {
        slot_id: u8,
        car_model_id: String,
        color_scheme_id: String,
        is_ready: bool,
    },
    /// Host → clients: the race is launched with this frozen setup.
    RaceLaunch(RaceConfig),
    /// Client → host: track loaded, ready for the start.
    Loaded,
    /// Host → clients: the green light is at `start_at` on the host clock (seconds).
    RaceStart { start_at: f64 },
    /// Owner → host: the owner's car crossed the finish line.
    Finished { finish_ms: u32, best_lap_ms: Option<u32>, penalty_ms: u32 },
    /// Host → clients: finish order so far.
    Standings { finishers: Vec<FinishRecord> },
    /// Host → clients: a player left the race.
    PlayerLeft { slot_id: u8, reason: String },
    /// Host → clients: the race is closed; final results.
    RaceOver { results: Vec<RaceResult> },
}

impl ControlMessage {
    /// Serializes the message for the reliable channel.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(self).map_err(|e| ProtocolError::SerializationFailed(e.to_string()))
    }

    /// Parses a message delivered by the reliable channel.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        serde_json::from_slice(bytes).map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
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

/// Top-level wire packet envelope for Cabinet LAN datagrams.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    /// Discovery beacon packet.
    Beacon(LanBeacon),
    /// Staging lobby coordination packet.
    Lobby(LobbyPacket),
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
    /// - 1 byte:  Datagram kind [`wire::KIND_JSON`]
    /// - N bytes: Serialized JSON payload
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let payload = serde_json::to_vec(self)
            .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        let total_len = wire::HEADER_LEN + payload.len();
        if total_len > MAX_DATAGRAM_SIZE {
            return Err(ProtocolError::PacketTooLarge(total_len));
        }
        let mut buf = wire::begin(wire::KIND_JSON, payload.len());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Decodes and validates a packet from raw received UDP datagram bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (kind, payload) = wire::split(bytes)?;
        if kind != wire::KIND_JSON {
            return Err(ProtocolError::DeserializationFailed(format!(
                "datagram kind {} is not a JSON packet",
                kind
            )));
        }
        Self::decode_json(payload)
    }

    /// Decodes the JSON payload of a [`wire::KIND_JSON`] datagram (after the header).
    pub fn decode_json(payload: &[u8]) -> Result<Self, ProtocolError> {
        serde_json::from_slice(payload)
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }

    /// Returns a reference to the inner [`LobbyPacket`] if this is a lobby packet.
    pub fn as_lobby(&self) -> Option<&LobbyPacket> {
        match self {
            Self::Lobby(p) => Some(p),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_and_magic_header() {
        assert_eq!(MAGIC_BYTES, [0x54, 0x44, 0x4C, 0x4E]);
        assert_eq!(LAN_MAGIC, MAGIC_BYTES);
        assert_eq!(&MAGIC_BYTES, b"TDLN");
        assert_eq!(PROTOCOL_VERSION, 3);
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
        assert_eq!(encoded[4], PROTOCOL_VERSION);
        assert_eq!(encoded[5], wire::KIND_JSON);

        let decoded = LanBeacon::decode(&encoded).expect("Failed to decode beacon");
        assert_eq!(beacon, decoded);
    }

    #[test]
    fn test_lobby_packet_roundtrips() {
        let packets = vec![
            LobbyPacket::JoinRequest {
                protocol_version: PROTOCOL_VERSION,
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
            },
            LobbyPacket::Ping { sent_at: 12.5 },
            LobbyPacket::Pong { sent_at: 12.5, responder_time: 1003.25 },
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
    fn test_control_message_roundtrips() {
        let roster = vec![RosterEntry {
            slot_id: 2,
            grid_index: 0,
            player_name: "Alex".to_string(),
            country_code: "FRA".to_string(),
            car_model_id: "gt3_viper".to_string(),
            color_scheme_id: "green".to_string(),
        }];
        let messages = vec![
            ControlMessage::StateSync {
                roster_rev: 4,
                track_id: "spa".to_string(),
                laps: 3,
                collision_mode: LanCollisionMode::GhostPassing,
                slots: vec![LobbySlot::new(0, "Mario".to_string(), "ESP".to_string(), true)],
            },
            ControlMessage::ClientSlotUpdate {
                slot_id: 1,
                car_model_id: "m4_gt3".to_string(),
                color_scheme_id: "black".to_string(),
                is_ready: true,
            },
            ControlMessage::RaceLaunch(RaceConfig {
                track_id: "monza".to_string(),
                laps: 5,
                collision_mode: LanCollisionMode::FullSatSolid,
                roster,
            }),
            ControlMessage::Loaded,
            ControlMessage::RaceStart { start_at: 42.125 },
            ControlMessage::Finished { finish_ms: 90_000, best_lap_ms: Some(29_500), penalty_ms: 30_000 },
            ControlMessage::Standings {
                finishers: vec![FinishRecord { slot_id: 2, finish_ms: 90_000, best_lap_ms: None, penalty_ms: 30_000 }],
            },
            ControlMessage::PlayerLeft { slot_id: 3, reason: "Heartbeat timeout".to_string() },
            ControlMessage::RaceOver {
                results: vec![RaceResult {
                    slot_id: 2,
                    position: 1,
                    status: RaceStatus::Finished,
                    finish_ms: Some(90_000),
                    best_lap_ms: Some(29_500),
                    penalty_ms: 30_000,
                }],
            },
        ];
        for m in messages {
            assert_eq!(ControlMessage::from_bytes(&m.to_bytes().unwrap()).unwrap(), m);
        }
        let cfg = RaceConfig {
            track_id: String::new(),
            laps: 1,
            collision_mode: LanCollisionMode::default(),
            roster: vec![],
        };
        assert_eq!(cfg.car_index_of(0), None);
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
    }
}
