//! # Cabinet LAN Network Statistics
//!
//! Datagram counters (via a counting `Transport` wrapper) and per-second
//! rates for the dev net HUD and for tests.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.9.

use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::transport::Transport;

/// Shared raw counters of one endpoint.
#[derive(Debug, Default)]
pub struct NetCounters {
    pub datagrams_in: AtomicU64,
    pub datagrams_out: AtomicU64,
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
}

/// `Transport` wrapper that counts every datagram and byte.
pub struct CountingTransport {
    inner: Box<dyn Transport + Send>,
    counters: Arc<NetCounters>,
}

impl CountingTransport {
    pub fn new(inner: Box<dyn Transport + Send>) -> (Self, Arc<NetCounters>) {
        let counters = Arc::new(NetCounters::default());
        (Self { inner, counters: counters.clone() }, counters)
    }
}

impl Transport for CountingTransport {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        self.counters.datagrams_out.fetch_add(1, Ordering::Relaxed);
        self.counters.bytes_out.fetch_add(buf.len() as u64, Ordering::Relaxed);
        self.inner.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let result = self.inner.recv_from(buf);
        if let Ok((n, _)) = result {
            self.counters.datagrams_in.fetch_add(1, Ordering::Relaxed);
            self.counters.bytes_in.fetch_add(n as u64, Ordering::Relaxed);
        }
        result
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    fn now_sec(&self) -> f64 {
        self.inner.now_sec()
    }
}

/// Snapshot of one endpoint's network health.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NetStats {
    pub datagrams_in: u64,
    pub datagrams_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// Datagrams per second over the last full second.
    pub in_per_sec: f32,
    pub out_per_sec: f32,
    /// Datagrams that failed header or payload checks.
    pub decode_errors: u64,
    /// Outgoing packets that could not be encoded.
    pub encode_errors: u64,
    /// Car states dropped as older than one already received.
    pub stale_dropped: u64,
    /// Reliable fragments waiting for an ack.
    pub reliable_pending: usize,
    /// Latest round trip in milliseconds (client: to the host; host: worst client).
    pub rtt_ms: Option<u16>,
    /// Host clock minus local clock in milliseconds (clients only).
    pub clock_offset_ms: Option<f64>,
}

/// Per-second rate meter over the shared counters.
#[derive(Debug, Default)]
pub struct RateMeter {
    window_start: Option<f64>,
    in_at_start: u64,
    out_at_start: u64,
    in_per_sec: f32,
    out_per_sec: f32,
}

impl RateMeter {
    /// Updates the rates once a full second has passed.
    pub fn tick(&mut self, now: f64, counters: &NetCounters) {
        let din = counters.datagrams_in.load(Ordering::Relaxed);
        let dout = counters.datagrams_out.load(Ordering::Relaxed);
        match self.window_start {
            None => {
                self.window_start = Some(now);
                self.in_at_start = din;
                self.out_at_start = dout;
            }
            Some(start) if now - start >= 1.0 => {
                let span = (now - start) as f32;
                self.in_per_sec = (din - self.in_at_start) as f32 / span;
                self.out_per_sec = (dout - self.out_at_start) as f32 / span;
                self.window_start = Some(now);
                self.in_at_start = din;
                self.out_at_start = dout;
            }
            _ => {}
        }
    }

    /// Fills the counter and rate fields of a snapshot.
    pub fn fill(&self, counters: &NetCounters, stats: &mut NetStats) {
        stats.datagrams_in = counters.datagrams_in.load(Ordering::Relaxed);
        stats.datagrams_out = counters.datagrams_out.load(Ordering::Relaxed);
        stats.bytes_in = counters.bytes_in.load(Ordering::Relaxed);
        stats.bytes_out = counters.bytes_out.load(Ordering::Relaxed);
        stats.in_per_sec = self.in_per_sec;
        stats.out_per_sec = self.out_per_sec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::transport::{SimLinkConfig, SimNetwork};

    #[test]
    fn test_counting_transport_and_rates() {
        let net = SimNetwork::new(SimLinkConfig::default(), 1);
        let (a, counters) = CountingTransport::new(Box::new(net.endpoint()));
        let b = net.endpoint();
        let b_addr = b.local_addr().unwrap();
        let mut meter = RateMeter::default();
        meter.tick(net.now(), &counters);
        for _ in 0..60 {
            a.send_to(&[0u8; 10], b_addr).unwrap();
            net.advance(1.0 / 60.0);
            meter.tick(net.now(), &counters);
        }
        net.advance(0.01);
        meter.tick(net.now(), &counters);
        let mut stats = NetStats::default();
        meter.fill(&counters, &mut stats);
        assert_eq!(stats.datagrams_out, 60);
        assert_eq!(stats.bytes_out, 600);
        assert!((stats.out_per_sec - 60.0).abs() < 2.0, "rate {}", stats.out_per_sec);
    }
}
