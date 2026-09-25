//! # Local IPv4 Address Resolution
//!
//! Provides utilities to determine the local machine's primary non-loopback
//! IPv4 network address and format addresses for arcade UI presentation.

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

/// Utility for discovering, parsing, and formatting the local machine's IPv4 address.
pub struct LocalIpResolver;

impl LocalIpResolver {
    /// Resolves the primary outward-facing non-loopback local IPv4 address.
    ///
    /// Utilizes kernel routing table lookup via a non-blocking UDP socket connect.
    /// This operation is completely local, instantaneous, and does not transmit any
    /// network datagrams across the wire.
    ///
    /// If no outward network route is available, safely falls back to `127.0.0.1`.
    pub fn resolve_local_ipv4() -> Ipv4Addr {
        // Attempt kernel route resolution to a well-known public IP
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(SocketAddr::V4(addr)) = socket.local_addr() {
                    let ip = *addr.ip();
                    if !ip.is_loopback() && !ip.is_unspecified() {
                        return ip;
                    }
                }
            }
            // Fallback route lookup targeting private subnet broadcast
            if socket.connect("10.255.255.255:80").is_ok() {
                if let Ok(SocketAddr::V4(addr)) = socket.local_addr() {
                    let ip = *addr.ip();
                    if !ip.is_loopback() && !ip.is_unspecified() {
                        return ip;
                    }
                }
            }
        }
        Ipv4Addr::new(127, 0, 0, 1)
    }

    /// Formats an IPv4 address and port number into an arcade-styled connection string.
    /// Example: `"192.168.1.105:7777"`
    pub fn format_address(ip: Ipv4Addr, port: u16) -> String {
        format!("{}:{}", ip, port)
    }

    /// Parses an IP:Port string (e.g. `"192.168.1.105:7777"` or `"192.168.1.105"` with default port).
    pub fn parse_address(input: &str, default_port: u16) -> Option<SocketAddr> {
        let trimmed = input.trim();
        if let Ok(addr) = trimmed.parse::<SocketAddr>() {
            return Some(addr);
        }
        if let Ok(ip) = trimmed.parse::<Ipv4Addr>() {
            return Some(SocketAddr::new(std::net::IpAddr::V4(ip), default_port));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_local_ipv4_returns_valid_ip() {
        let ip = LocalIpResolver::resolve_local_ipv4();
        // Should not be 0.0.0.0
        assert!(!ip.is_unspecified());
    }

    #[test]
    fn test_format_address() {
        let ip = Ipv4Addr::new(192, 168, 1, 105);
        assert_eq!(LocalIpResolver::format_address(ip, 7777), "192.168.1.105:7777");
    }

    #[test]
    fn test_parse_address_with_port() {
        let addr = LocalIpResolver::parse_address("192.168.1.50:7777", 9999).unwrap();
        assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 50));
        assert_eq!(addr.port(), 7777);
    }

    #[test]
    fn test_parse_address_without_port_uses_default() {
        let addr = LocalIpResolver::parse_address("10.0.0.12", 7777).unwrap();
        assert_eq!(addr.ip(), Ipv4Addr::new(10, 0, 0, 12));
        assert_eq!(addr.port(), 7777);
    }

    #[test]
    fn test_parse_address_invalid() {
        assert!(LocalIpResolver::parse_address("not_an_ip", 7777).is_none());
        assert!(LocalIpResolver::parse_address("999.999.999.999", 7777).is_none());
    }
}
