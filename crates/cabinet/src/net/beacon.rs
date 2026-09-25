//! # Local LAN Beacon Discovery System
//!
//! Handles broadcasting UDP advertisement beacons across port 7776 and
//! scanning local subnets for active Cabinet arcade rooms.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use crate::net::protocol::{
    LanBeacon, ProtocolError, DEFAULT_BEACON_PORT, MAX_DATAGRAM_SIZE,
};

/// Represents an active LAN game discovered via UDP broadcast beacons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredHost {
    /// Room advertisement beacon metadata.
    pub beacon: LanBeacon,
    /// Authoritative game session address (IP and game port).
    pub host_address: SocketAddr,
    /// Measured round-trip ping time in milliseconds (0 if unmeasured).
    pub ping_ms: u16,
    /// Internal age timer in seconds since last beacon received.
    pub(crate) age_sec_x1000: u32,
}

impl DiscoveredHost {
    /// Returns the socket address to connect to for the game session.
    pub fn game_socket_addr(&self) -> SocketAddr {
        self.host_address
    }

    /// Elapsed seconds since the last beacon was received from this host.
    pub fn last_seen_elapsed_sec(&self) -> f32 {
        self.age_sec_x1000 as f32 / 1000.0
    }

    /// Returns true if no beacons have been received within `timeout_sec`.
    pub fn is_stale(&self, timeout_sec: f32) -> bool {
        self.last_seen_elapsed_sec() > timeout_sec
    }
}

/// Periodic UDP broadcast beacon transmitter for session hosts.
pub struct LanBeaconBroadcaster {
    socket: UdpSocket,
    beacon: LanBeacon,
    broadcast_port: u16,
    custom_target: Option<SocketAddr>,
    interval_sec: f32,
    timer_sec: f32,
}

impl LanBeaconBroadcaster {
    /// Binds an ephemeral UDP socket configured for subnet broadcasting.
    pub fn new(beacon: LanBeacon, broadcast_port: u16) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let _ = socket.set_broadcast(true);
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            beacon,
            broadcast_port,
            custom_target: None,
            interval_sec: 1.0,
            timer_sec: 1.0, // trigger immediate initial broadcast on first update
        })
    }

    /// Sets a custom target socket address (useful for loopback or targeted testing).
    pub fn set_target(&mut self, target: Option<SocketAddr>) {
        self.custom_target = target;
    }

    /// Updates the beacon contents advertised to the network.
    pub fn set_beacon(&mut self, beacon: LanBeacon) {
        self.beacon = beacon;
    }

    /// Returns a reference to the active beacon metadata.
    pub fn beacon(&self) -> &LanBeacon {
        &self.beacon
    }

    /// Sets the broadcast repetition interval in seconds (default: 1.0s).
    pub fn set_interval(&mut self, interval_sec: f32) {
        self.interval_sec = interval_sec.max(0.1);
    }

    /// Advances the internal timer and broadcasts when the interval expires.
    ///
    /// Returns `Ok(true)` if a broadcast was transmitted this tick, `Ok(false)` otherwise.
    pub fn update(&mut self, dt: f32) -> Result<bool, ProtocolError> {
        self.timer_sec += dt;
        if self.timer_sec >= self.interval_sec {
            self.timer_sec = 0.0;
            self.broadcast()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Forcefully transmits a beacon packet immediately.
    pub fn broadcast(&mut self) -> Result<(), ProtocolError> {
        let encoded = self.beacon.encode()?;
        if let Some(target) = self.custom_target {
            let _ = self.socket.send_to(&encoded, target);
        } else {
            let broadcast_addr = format!("255.255.255.255:{}", self.broadcast_port);
            if self.socket.send_to(&encoded, &broadcast_addr).is_err() {
                // If global subnet broadcast fails (e.g. no default route), fallback to local loopback
                let loopback_addr = format!("127.0.0.1:{}", self.broadcast_port);
                let _ = self.socket.send_to(&encoded, &loopback_addr);
            }
        }
        Ok(())
    }
}

/// Passive UDP listener scanning for local broadcast beacons.
pub struct LanBeaconScanner {
    socket: UdpSocket,
    discovered: Vec<DiscoveredHost>,
    recv_buf: [u8; MAX_DATAGRAM_SIZE],
    timeout_sec: f32,
}

impl LanBeaconScanner {
    /// Binds to `0.0.0.0:port` to receive discovery broadcasts.
    pub fn bind(port: u16) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        let _ = socket.set_broadcast(true);
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            discovered: Vec::new(),
            recv_buf: [0u8; MAX_DATAGRAM_SIZE],
            timeout_sec: 3.5,
        })
    }

    /// Binds using the default discovery port [`DEFAULT_BEACON_PORT`] (7776).
    pub fn bind_default() -> Result<Self, io::Error> {
        Self::bind(DEFAULT_BEACON_PORT)
    }

    /// Returns the local socket address this scanner is bound to.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Sets the timeout duration after which a missing host is purged (default: 3.5s).
    pub fn set_timeout(&mut self, timeout_sec: f32) {
        self.timeout_sec = timeout_sec.max(0.5);
    }

    /// Pumps incoming UDP datagrams, discovers new hosts, and prunes stale rooms.
    pub fn update(&mut self, dt: f32) -> &[DiscoveredHost] {
        let delta_ms = (dt * 1000.0).max(0.0) as u32;
        for host in &mut self.discovered {
            host.age_sec_x1000 = host.age_sec_x1000.saturating_add(delta_ms);
        }

        // Receive pending broadcast datagrams
        loop {
            match self.socket.recv_from(&mut self.recv_buf) {
                Ok((bytes_read, src_addr)) => {
                    if let Ok(beacon) = LanBeacon::decode(&self.recv_buf[..bytes_read]) {
                        let game_addr = SocketAddr::new(src_addr.ip(), beacon.game_port);
                        if let Some(existing) = self.discovered.iter_mut().find(|h| {
                            h.beacon.server_name == beacon.server_name
                                && h.host_address.ip() == game_addr.ip()
                        }) {
                            existing.beacon = beacon;
                            existing.host_address = game_addr;
                            existing.age_sec_x1000 = 0;
                        } else {
                            self.discovered.push(DiscoveredHost {
                                beacon,
                                host_address: game_addr,
                                ping_ms: 0,
                                age_sec_x1000: 0,
                            });
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // Prune stale hosts
        let timeout_ms = (self.timeout_sec * 1000.0) as u32;
        self.discovered.retain(|h| h.age_sec_x1000 <= timeout_ms);

        &self.discovered
    }

    /// Returns currently discovered active hosts.
    pub fn discovered_hosts(&self) -> &[DiscoveredHost] {
        &self.discovered
    }

    /// Clears all discovered hosts.
    pub fn clear(&mut self) {
        self.discovered.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_broadcaster_and_scanner_loopback() {
        // Bind scanner to ephemeral port
        let mut scanner = LanBeaconScanner::bind(0).expect("Bind scanner");
        let scanner_addr = scanner.local_addr().expect("Scanner local addr");

        let beacon = LanBeacon::new(
            "Test Grand Prix".to_string(),
            "Mario".to_string(),
            7777,
            "monza".to_string(),
            "Formula 1".to_string(),
            1,
            8,
            false,
        );

        let mut broadcaster = LanBeaconBroadcaster::new(beacon.clone(), scanner_addr.port())
            .expect("Create broadcaster");

        // Target the scanner directly via loopback
        broadcaster.set_target(Some(SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            scanner_addr.port(),
        )));

        // Broadcast once
        broadcaster.broadcast().expect("Broadcast packet");

        // Pump scanner
        std::thread::sleep(std::time::Duration::from_millis(10));
        let discovered = scanner.update(0.01);

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].beacon.server_name, "Test Grand Prix");
        assert_eq!(discovered[0].beacon.host_player_name, "Mario");
        assert_eq!(discovered[0].game_socket_addr().port(), 7777);
        assert!(!discovered[0].is_stale(1.0));

        // Age out host
        let pruned = scanner.update(4.0);
        assert_eq!(pruned.len(), 0);
    }

    #[test]
    fn test_broadcaster_timer_interval() {
        let beacon = LanBeacon::new(
            "Timer Room".to_string(),
            "Luigi".to_string(),
            7777,
            "spa".to_string(),
            "GT3".to_string(),
            2,
            6,
            false,
        );

        let mut broadcaster = LanBeaconBroadcaster::new(beacon, 7776).expect("Create broadcaster");
        broadcaster.set_interval(0.5);

        // First update should trigger immediate broadcast since timer was initialized to interval
        assert!(broadcaster.update(0.0).expect("Update"));

        // Small dt should not trigger
        assert!(!broadcaster.update(0.2).expect("Update"));
        assert!(!broadcaster.update(0.2).expect("Update"));

        // Crossing 0.5s should trigger
        assert!(broadcaster.update(0.2).expect("Update"));
    }
}
