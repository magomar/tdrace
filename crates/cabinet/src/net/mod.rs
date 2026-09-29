//! # Cabinet Networking Subsystem
//!
//! Provides zero-configuration local area network (LAN) arcade multiplayer components,
//! discovery beacons, authoritative host/client networking, and lobby synchronization.

pub mod beacon;
pub mod client;
pub mod clock;
pub mod host;
pub mod interp;
pub mod ip;
pub mod protocol;
pub mod reliable;
pub mod stats;
pub mod transport;
pub mod wire;
pub mod ui;

pub use beacon::{DiscoveredHost, LanBeaconBroadcaster, LanBeaconScanner};
pub use client::{ClientEvent, ClientState, LanClient};
pub use clock::ClockSync;
pub use host::{HostEvent, LanHost, COUNTDOWN_SEC, FINISH_TIMEOUT_SEC, LOAD_TIMEOUT_SEC};
pub use interp::{RemoteCarBuffer, INTERP_DELAY_SEC, MAX_EXTRAPOLATION_SEC};
pub use ip::LocalIpResolver;
pub use protocol::{
    sanitize_string, ControlMessage, FinishRecord, JoinResult, LanBeacon, LanCollisionMode,
    LobbyPacket, LobbySlot, Packet, ProtocolError, RaceConfig, RaceResult, RaceStatus,
    RosterEntry,
    DEFAULT_BEACON_PORT, DEFAULT_GAME_PORT, LAN_MAGIC, MAGIC_BYTES, MAX_DATAGRAM_SIZE,
    MAX_NAME_LENGTH, PROTOCOL_VERSION,
};
pub use transport::{SimDropFilter, SimLinkConfig, SimNetwork, SimTransport, Transport, UdpTransport};
pub use reliable::ReliableChannel;
pub use stats::NetStats;
pub use wire::{NetCarState, WorldState};
pub use ui::{
    CabinetLanClientLobbyScreen, CabinetLanHostScreen, CabinetLanJoinScreen, IpKeypad, IpKeypadAction, LanLobbyRequest,
    LAN_LIVERIES,
};

