//! # Cabinet LAN Datagram Transport
//!
//! Abstracts the UDP socket used by `LanHost` and `LanClient` so that the
//! same session code runs on a real socket (`UdpTransport`) or on an
//! in-memory network with loss, delay, jitter and reordering
//! (`SimNetwork` / `SimTransport`) for deterministic tests.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.1.

use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Non-blocking datagram endpoint plus a monotonic clock.
pub trait Transport {
    /// Sends one datagram to `addr`.
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize>;
    /// Receives one datagram. Returns `ErrorKind::WouldBlock` when none is waiting.
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
    /// Local address of this endpoint.
    fn local_addr(&self) -> io::Result<SocketAddr>;
    /// Monotonic time in seconds. The origin is arbitrary and differs per endpoint.
    fn now_sec(&self) -> f64;
}

/// Real non-blocking UDP socket.
pub struct UdpTransport {
    socket: UdpSocket,
    epoch: Instant,
}

impl UdpTransport {
    /// Binds a non-blocking UDP socket on `addr` (e.g. `"0.0.0.0:7777"`).
    pub fn bind(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket, epoch: Instant::now() })
    }
}

impl Transport for UdpTransport {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    fn now_sec(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64()
    }
}

/// Link behaviour of a `SimNetwork`, applied to every datagram.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimLinkConfig {
    /// Probability in `0.0..=1.0` that a datagram is dropped.
    pub loss: f32,
    /// Fixed one-way delay in seconds.
    pub delay_sec: f64,
    /// Extra random one-way delay in `0.0..jitter_sec`. Jitter reorders datagrams.
    pub jitter_sec: f64,
}

impl Default for SimLinkConfig {
    fn default() -> Self {
        Self { loss: 0.0, delay_sec: 0.001, jitter_sec: 0.0 }
    }
}

/// Decides per datagram `(bytes, from, to)` whether to drop it, in addition to random loss.
pub type SimDropFilter = Box<dyn FnMut(&[u8], SocketAddr, SocketAddr) -> bool + Send>;

struct InFlight {
    deliver_at: f64,
    order: u64,
    from: SocketAddr,
    data: Vec<u8>,
}

struct SimInner {
    now: f64,
    rng: u64,
    order: u64,
    next_port: u16,
    config: SimLinkConfig,
    queues: HashMap<SocketAddr, Vec<InFlight>>,
    drop_filter: Option<SimDropFilter>,
    sent: u64,
    dropped: u64,
}

impl SimInner {
    fn next_unit(&mut self) -> f64 {
        // xorshift64*: deterministic for a given seed.
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let v = self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// In-memory datagram network with a manually advanced clock.
#[derive(Clone)]
pub struct SimNetwork {
    inner: Arc<Mutex<SimInner>>,
}

impl SimNetwork {
    /// Creates a network with the given link behaviour and RNG seed.
    pub fn new(config: SimLinkConfig, seed: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SimInner {
                now: 0.0,
                rng: seed.max(1),
                order: 0,
                next_port: 40_000,
                config,
                queues: HashMap::new(),
                drop_filter: None,
                sent: 0,
                dropped: 0,
            })),
        }
    }

    /// Creates a new endpoint with a unique loopback address.
    pub fn endpoint(&self) -> SimTransport {
        let mut inner = self.inner.lock().unwrap();
        let port = inner.next_port;
        inner.next_port += 1;
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        inner.queues.insert(addr, Vec::new());
        SimTransport { net: self.clone(), addr }
    }

    /// Advances the shared clock by `dt` seconds.
    pub fn advance(&self, dt: f64) {
        self.inner.lock().unwrap().now += dt;
    }

    /// Current shared clock in seconds.
    pub fn now(&self) -> f64 {
        self.inner.lock().unwrap().now
    }

    /// Replaces the link behaviour.
    pub fn set_config(&self, config: SimLinkConfig) {
        self.inner.lock().unwrap().config = config;
    }

    /// Installs a filter that can drop chosen datagrams.
    pub fn set_drop_filter(&self, filter: Option<SimDropFilter>) {
        self.inner.lock().unwrap().drop_filter = filter;
    }

    /// Removes an endpoint. Datagrams sent to it afterwards are lost.
    pub fn disconnect(&self, addr: SocketAddr) {
        self.inner.lock().unwrap().queues.remove(&addr);
    }

    /// `(sent, dropped)` datagram counters.
    pub fn counters(&self) -> (u64, u64) {
        let inner = self.inner.lock().unwrap();
        (inner.sent, inner.dropped)
    }
}

/// One endpoint of a `SimNetwork`.
pub struct SimTransport {
    net: SimNetwork,
    addr: SocketAddr,
}

impl Transport for SimTransport {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let mut guard = self.net.inner.lock().unwrap();
        let inner = &mut *guard;
        inner.sent += 1;
        let filtered = match inner.drop_filter.as_mut() {
            Some(filter) => filter(buf, self.addr, addr),
            None => false,
        };
        let lost = inner.config.loss > 0.0 && inner.next_unit() < inner.config.loss as f64;
        if filtered || lost || !inner.queues.contains_key(&addr) {
            inner.dropped += 1;
            return Ok(buf.len());
        }
        let jitter = if inner.config.jitter_sec > 0.0 {
            inner.next_unit() * inner.config.jitter_sec
        } else {
            0.0
        };
        let deliver_at = inner.now + inner.config.delay_sec + jitter;
        inner.order += 1;
        let order = inner.order;
        if let Some(queue) = inner.queues.get_mut(&addr) {
            queue.push(InFlight { deliver_at, order, from: self.addr, data: buf.to_vec() });
        }
        Ok(buf.len())
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut inner = self.net.inner.lock().unwrap();
        let now = inner.now;
        let queue = inner
            .queues
            .get_mut(&self.addr)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotConnected))?;
        let next = queue
            .iter()
            .enumerate()
            .filter(|(_, p)| p.deliver_at <= now)
            .min_by(|(_, a), (_, b)| {
                a.deliver_at.partial_cmp(&b.deliver_at).unwrap().then(a.order.cmp(&b.order))
            })
            .map(|(i, _)| i);
        match next {
            Some(i) => {
                let packet = queue.swap_remove(i);
                let n = packet.data.len().min(buf.len());
                buf[..n].copy_from_slice(&packet.data[..n]);
                Ok((n, packet.from))
            }
            None => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.addr)
    }

    fn now_sec(&self) -> f64 {
        self.net.now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_delivers_after_delay_in_send_order() {
        let net = SimNetwork::new(SimLinkConfig { loss: 0.0, delay_sec: 0.010, jitter_sec: 0.0 }, 7);
        let a = net.endpoint();
        let b = net.endpoint();
        let b_addr = b.local_addr().unwrap();
        a.send_to(b"one", b_addr).unwrap();
        a.send_to(b"two", b_addr).unwrap();

        let mut buf = [0u8; 16];
        assert_eq!(b.recv_from(&mut buf).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        net.advance(0.011);
        let (n, from) = b.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"one");
        assert_eq!(from, a.local_addr().unwrap());
        let (n, _) = b.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"two");
        assert_eq!(b.recv_from(&mut buf).unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn test_sim_loss_is_deterministic_and_close_to_rate() {
        let run = || {
            let net = SimNetwork::new(SimLinkConfig { loss: 0.3, delay_sec: 0.0, jitter_sec: 0.0 }, 42);
            let a = net.endpoint();
            let b = net.endpoint();
            for _ in 0..2000 {
                a.send_to(b"x", b.local_addr().unwrap()).unwrap();
            }
            net.counters()
        };
        let (sent, dropped) = run();
        assert_eq!((sent, dropped), run(), "same seed must give the same losses");
        let rate = dropped as f64 / sent as f64;
        assert!((0.25..0.35).contains(&rate), "loss rate {rate}");
    }

    #[test]
    fn test_sim_jitter_reorders() {
        let net = SimNetwork::new(SimLinkConfig { loss: 0.0, delay_sec: 0.0, jitter_sec: 0.050 }, 3);
        let a = net.endpoint();
        let b = net.endpoint();
        for i in 0..50u8 {
            a.send_to(&[i], b.local_addr().unwrap()).unwrap();
        }
        net.advance(0.1);
        let mut got = Vec::new();
        let mut buf = [0u8; 4];
        while let Ok((_, _)) = b.recv_from(&mut buf) {
            got.push(buf[0]);
        }
        assert_eq!(got.len(), 50);
        assert!(got.windows(2).any(|w| w[0] > w[1]), "jitter must reorder some datagrams");
    }
}
