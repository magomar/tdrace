//! # Cabinet LAN Binary Wire Format
//!
//! Every datagram starts with `magic (4) + version (1) + kind (1)`.
//! `KIND_JSON` carries a serde JSON `Packet` (beacon, lobby handshake).
//! The in-race car states use a fixed little-endian binary layout so that
//! an 8-car world packet stays far below `MAX_DATAGRAM_SIZE`.
//!
//! All decoders treat input as untrusted: they check every length, reject
//! non-finite floats, and never panic.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.1.

use super::protocol::{ProtocolError, MAGIC_BYTES, MAX_DATAGRAM_SIZE, PROTOCOL_VERSION};

/// JSON `Packet` payload.
pub const KIND_JSON: u8 = 0;
/// One owner-authoritative `NetCarState` (owner → host).
pub const KIND_CAR_STATE: u8 = 1;
/// A `WorldState` with the latest state of every car (host → clients).
pub const KIND_WORLD_STATE: u8 = 2;
/// A fragment of a reliable control message (see `reliable.rs`).
pub const KIND_RELIABLE: u8 = 3;
/// Acknowledgement of one reliable fragment.
pub const KIND_ACK: u8 = 4;

/// Length of the common datagram header.
pub const HEADER_LEN: usize = 6;

/// `NetCarState::flags` bits.
pub mod flags {
    pub const BRAKING: u8 = 1 << 0;
    pub const HANDBRAKE: u8 = 1 << 1;
    pub const DRIFTING: u8 = 1 << 2;
    pub const AIRBORNE: u8 = 1 << 3;
    pub const LIGHTS: u8 = 1 << 4;
    pub const REVERSE: u8 = 1 << 5;
}

/// Writes the common header for `kind` into a new buffer.
pub fn begin(kind: u8, capacity: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_LEN + capacity);
    buf.extend_from_slice(&MAGIC_BYTES);
    buf.push(PROTOCOL_VERSION);
    buf.push(kind);
    buf
}

/// Checks size, magic and version, and returns `(kind, payload)`.
pub fn split(bytes: &[u8]) -> Result<(u8, &[u8]), ProtocolError> {
    if bytes.len() > MAX_DATAGRAM_SIZE {
        return Err(ProtocolError::PacketTooLarge(bytes.len()));
    }
    if bytes.len() < HEADER_LEN {
        return Err(ProtocolError::PacketTooShort(bytes.len()));
    }
    if bytes[0..4] != MAGIC_BYTES {
        return Err(ProtocolError::InvalidMagic([bytes[0], bytes[1], bytes[2], bytes[3]]));
    }
    if bytes[4] != PROTOCOL_VERSION {
        return Err(ProtocolError::VersionMismatch(bytes[4]));
    }
    Ok((bytes[5], &bytes[HEADER_LEN..]))
}

/// Bounds-checked little-endian reader.
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ProtocolError> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.bytes.len()).ok_or_else(|| {
            ProtocolError::DeserializationFailed(format!("truncated at byte {}", self.pos))
        })?;
        let out = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    pub fn u8(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, ProtocolError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> Result<u32, ProtocolError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Reads an `f32` and rejects NaN and infinity.
    pub fn f32(&mut self) -> Result<f32, ProtocolError> {
        let b = self.take(4)?;
        let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        if v.is_finite() {
            Ok(v)
        } else {
            Err(ProtocolError::DeserializationFailed("non-finite float".to_string()))
        }
    }

    /// Remaining unread bytes.
    pub fn rest(&mut self) -> &'a [u8] {
        let out = &self.bytes[self.pos..];
        self.pos = self.bytes.len();
        out
    }

    pub fn is_empty(&self) -> bool {
        self.pos == self.bytes.len()
    }
}

/// Owner-authoritative kinematic state of one car at one moment of the shared race clock.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NetCarState {
    /// Lobby slot of the owner.
    pub slot: u8,
    /// Shared race clock in milliseconds since the green light.
    pub time_ms: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    /// Heading in radians.
    pub angle: f32,
    pub angular_velocity: f32,
    pub steer_angle: f32,
    pub elevation: f32,
    pub vertical_velocity: f32,
    /// Throttle `0..=255` (for engine sound and lights).
    pub throttle: u8,
    /// Brake `0..=255`.
    pub brake: u8,
    /// See `flags`.
    pub flags: u8,
    /// Owner's lap counter.
    pub lap: u8,
    /// Owner's next checkpoint index.
    pub checkpoint: u16,
    /// Owner's normalized lap progress `0.0..=1.0`.
    pub progress: f32,
}

impl NetCarState {
    /// Encoded size in bytes.
    pub const ENCODED_LEN: usize = 51;

    pub fn write(&self, out: &mut Vec<u8>) {
        out.push(self.slot);
        out.extend_from_slice(&self.time_ms.to_le_bytes());
        for v in [
            self.pos_x,
            self.pos_y,
            self.vel_x,
            self.vel_y,
            self.angle,
            self.angular_velocity,
            self.steer_angle,
            self.elevation,
            self.vertical_velocity,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.push(self.throttle);
        out.push(self.brake);
        out.push(self.flags);
        out.push(self.lap);
        out.extend_from_slice(&self.checkpoint.to_le_bytes());
        out.extend_from_slice(&self.progress.to_le_bytes());
    }

    pub fn read(r: &mut Reader) -> Result<Self, ProtocolError> {
        Ok(Self {
            slot: r.u8()?,
            time_ms: r.u32()?,
            pos_x: r.f32()?,
            pos_y: r.f32()?,
            vel_x: r.f32()?,
            vel_y: r.f32()?,
            angle: r.f32()?,
            angular_velocity: r.f32()?,
            steer_angle: r.f32()?,
            elevation: r.f32()?,
            vertical_velocity: r.f32()?,
            throttle: r.u8()?,
            brake: r.u8()?,
            flags: r.u8()?,
            lap: r.u8()?,
            checkpoint: r.u16()?,
            progress: r.f32()?,
        })
    }

    /// Encodes one `KIND_CAR_STATE` datagram.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = begin(KIND_CAR_STATE, Self::ENCODED_LEN);
        self.write(&mut buf);
        buf
    }

    /// Decodes a `KIND_CAR_STATE` payload (after the header).
    pub fn decode_payload(payload: &[u8]) -> Result<Self, ProtocolError> {
        if payload.len() != Self::ENCODED_LEN {
            return Err(ProtocolError::DeserializationFailed(format!(
                "car state is {} bytes, expected {}",
                payload.len(),
                Self::ENCODED_LEN
            )));
        }
        Self::read(&mut Reader::new(payload))
    }

    pub fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

/// Latest state of every car, relayed by the host.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorldState {
    /// Shared race clock of the host when it sent this packet.
    pub host_time_ms: u32,
    pub cars: Vec<NetCarState>,
}

impl WorldState {
    /// Largest number of cars that fits in one datagram.
    pub const MAX_CARS: usize = (MAX_DATAGRAM_SIZE - HEADER_LEN - 5) / NetCarState::ENCODED_LEN;

    /// Encodes one `KIND_WORLD_STATE` datagram.
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.cars.len() > Self::MAX_CARS {
            return Err(ProtocolError::PacketTooLarge(
                HEADER_LEN + 5 + self.cars.len() * NetCarState::ENCODED_LEN,
            ));
        }
        let mut buf = begin(KIND_WORLD_STATE, 5 + self.cars.len() * NetCarState::ENCODED_LEN);
        buf.extend_from_slice(&self.host_time_ms.to_le_bytes());
        buf.push(self.cars.len() as u8);
        for car in &self.cars {
            car.write(&mut buf);
        }
        Ok(buf)
    }

    /// Decodes a `KIND_WORLD_STATE` payload (after the header).
    pub fn decode_payload(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut r = Reader::new(payload);
        let host_time_ms = r.u32()?;
        let count = r.u8()? as usize;
        if payload.len() != 5 + count * NetCarState::ENCODED_LEN {
            return Err(ProtocolError::DeserializationFailed(format!(
                "world state is {} bytes for {} cars",
                payload.len(),
                count
            )));
        }
        let mut cars = Vec::with_capacity(count);
        for _ in 0..count {
            cars.push(NetCarState::read(&mut r)?);
        }
        Ok(Self { host_time_ms, cars })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(slot: u8) -> NetCarState {
        NetCarState {
            slot,
            time_ms: 95_123,
            pos_x: -1234.5678,
            pos_y: 876.54321,
            vel_x: -45.123456,
            vel_y: 12.345678,
            angle: -2.3456789,
            angular_velocity: 0.12345678,
            steer_angle: -0.0345678,
            elevation: 0.25,
            vertical_velocity: -1.5,
            throttle: 255,
            brake: 12,
            flags: flags::BRAKING | flags::LIGHTS,
            lap: 3,
            checkpoint: 17,
            progress: 0.4567,
        }
    }

    #[test]
    fn test_car_state_roundtrip_is_exact() {
        let s = sample(5);
        let bytes = s.encode();
        assert_eq!(bytes.len(), HEADER_LEN + NetCarState::ENCODED_LEN);
        let (kind, payload) = split(&bytes).unwrap();
        assert_eq!(kind, KIND_CAR_STATE);
        assert_eq!(NetCarState::decode_payload(payload).unwrap(), s);
    }

    #[test]
    fn test_eight_car_world_state_fits_and_roundtrips() {
        let w = WorldState { host_time_ms: 7, cars: (0..8).map(sample).collect() };
        let bytes = w.encode().unwrap();
        assert_eq!(bytes.len(), 419);
        assert!(bytes.len() < MAX_DATAGRAM_SIZE);
        let (kind, payload) = split(&bytes).unwrap();
        assert_eq!(kind, KIND_WORLD_STATE);
        assert_eq!(WorldState::decode_payload(payload).unwrap(), w);
        assert!(WorldState::MAX_CARS >= 8);
    }

    #[test]
    fn test_decoders_reject_bad_input_without_panic() {
        let good = WorldState { host_time_ms: 1, cars: vec![sample(0), sample(1)] }.encode().unwrap();
        let payload = &good[HEADER_LEN..];
        // Every truncation is rejected.
        for n in 0..payload.len() {
            assert!(WorldState::decode_payload(&payload[..n]).is_err(), "truncated to {n}");
        }
        // Extra trailing bytes are rejected.
        let mut long = payload.to_vec();
        long.push(0);
        assert!(WorldState::decode_payload(&long).is_err());
        // A count larger than the data is rejected.
        let mut lying = payload.to_vec();
        lying[4] = 200;
        assert!(WorldState::decode_payload(&lying).is_err());
        // NaN is rejected.
        let mut nan = sample(0);
        nan.pos_x = f32::NAN;
        let bytes = nan.encode();
        assert!(NetCarState::decode_payload(&bytes[HEADER_LEN..]).is_err());
        // Header checks.
        assert!(matches!(split(&good[..3]), Err(ProtocolError::PacketTooShort(3))));
        let mut bad_version = good.clone();
        bad_version[4] = 1;
        assert!(matches!(split(&bad_version), Err(ProtocolError::VersionMismatch(1))));
        assert!(matches!(split(&vec![0u8; MAX_DATAGRAM_SIZE + 1]), Err(ProtocolError::PacketTooLarge(_))));
    }

    #[test]
    fn test_world_state_refuses_too_many_cars() {
        let w = WorldState { host_time_ms: 0, cars: vec![sample(0); WorldState::MAX_CARS + 1] };
        assert!(matches!(w.encode(), Err(ProtocolError::PacketTooLarge(_))));
    }
}
