//! # Cabinet Networking Subsystem
//!
//! Provides zero-configuration local area network (LAN) arcade multiplayer components,
//! discovery beacons, authoritative host/client networking, and lobby synchronization.

pub mod beacon;
pub mod client;
pub mod host;
pub mod ip;
pub mod protocol;
pub mod ui;

pub use beacon::{DiscoveredHost, LanBeaconBroadcaster, LanBeaconScanner};
pub use client::{ClientEvent, ClientState, LanClient};
pub use host::{HostEvent, LanHost};
pub use ip::LocalIpResolver;
pub use protocol::{
    sanitize_string, CarStateSnapshot, ClientInputPacket, JoinResult, LanBeacon,
    LanCollisionMode, LobbyPacket, LobbySlot, Packet, ProtocolError, WorldSnapshotPacket,
    DEFAULT_BEACON_PORT, DEFAULT_GAME_PORT, LAN_MAGIC, MAGIC_BYTES, MAX_DATAGRAM_SIZE,
    MAX_NAME_LENGTH, PROTOCOL_VERSION,
};
pub use ui::{CabinetLanClientLobbyScreen, CabinetLanHostScreen, CabinetLanJoinScreen, IpKeypad, IpKeypadAction};

