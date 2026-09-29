//! # Cabinet LAN Reliable Control Channel
//!
//! Ordered, exactly-once delivery of control messages over UDP, one channel
//! per peer. A message is split into fragments; each fragment has a `u16`
//! sequence number, is acknowledged by the receiver, and is resent every
//! `RESEND_INTERVAL_SEC` until acknowledged. After `MAX_TRIES` sends
//! without an ack the channel reports the peer as lost.
//!
//! Fragment datagram: header (`KIND_RELIABLE`) + `seq u16` + `frag u8` + `count u8` + bytes.
//! Ack datagram: header (`KIND_ACK`) + `seq u16`.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.6.

use std::collections::{HashMap, VecDeque};

use super::protocol::{ProtocolError, MAX_DATAGRAM_SIZE};
use super::wire::{self, Reader, HEADER_LEN, KIND_ACK, KIND_RELIABLE};

/// Time between resends of an unacknowledged fragment.
pub const RESEND_INTERVAL_SEC: f64 = 0.15;
/// Sends of one fragment before the peer is treated as lost.
pub const MAX_TRIES: u32 = 20;
/// Largest message payload carried by one fragment.
pub const MAX_FRAGMENT_PAYLOAD: usize = MAX_DATAGRAM_SIZE - HEADER_LEN - 4;
/// Largest message the channel accepts (255 fragments).
pub const MAX_MESSAGE_LEN: usize = MAX_FRAGMENT_PAYLOAD * 255;
/// Fragments buffered ahead of the next expected one.
const RECEIVE_WINDOW: u16 = 1024;

struct Pending {
    seq: u16,
    datagram: Vec<u8>,
    last_sent: f64,
    tries: u32,
}

/// One direction-pair of reliable traffic with a single peer.
#[derive(Default)]
pub struct ReliableChannel {
    next_seq: u16,
    pending: VecDeque<Pending>,
    next_expected: u16,
    early: HashMap<u16, (u8, Vec<u8>)>,
    assembling: Vec<u8>,
    failed: bool,
}

impl ReliableChannel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `message` and returns the fragment datagrams to send now.
    pub fn send(&mut self, message: &[u8], now: f64) -> Result<Vec<Vec<u8>>, ProtocolError> {
        if message.len() > MAX_MESSAGE_LEN {
            return Err(ProtocolError::PacketTooLarge(message.len()));
        }
        let chunks: Vec<&[u8]> = if message.is_empty() {
            vec![&[][..]]
        } else {
            message.chunks(MAX_FRAGMENT_PAYLOAD).collect()
        };
        let count = chunks.len() as u8;
        let mut out = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.into_iter().enumerate() {
            let seq = self.next_seq;
            self.next_seq = self.next_seq.wrapping_add(1);
            let mut datagram = wire::begin(KIND_RELIABLE, 4 + chunk.len());
            datagram.extend_from_slice(&seq.to_le_bytes());
            datagram.push(i as u8);
            datagram.push(count);
            datagram.extend_from_slice(chunk);
            self.pending.push_back(Pending { seq, datagram: datagram.clone(), last_sent: now, tries: 1 });
            out.push(datagram);
        }
        Ok(out)
    }

    /// Returns the fragment datagrams that are due for a resend.
    pub fn poll_resend(&mut self, now: f64) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for p in self.pending.iter_mut() {
            if now - p.last_sent >= RESEND_INTERVAL_SEC {
                if p.tries >= MAX_TRIES {
                    self.failed = true;
                    continue;
                }
                p.tries += 1;
                p.last_sent = now;
                out.push(p.datagram.clone());
            }
        }
        out
    }

    /// Handles a `KIND_RELIABLE` payload. Returns the ack datagram (empty when
    /// the fragment is not accepted) and the complete messages that are now
    /// deliverable, in send order.
    pub fn on_reliable(&mut self, payload: &[u8]) -> Result<(Vec<u8>, Vec<Vec<u8>>), ProtocolError> {
        let mut r = Reader::new(payload);
        let seq = r.u16()?;
        let frag = r.u8()?;
        let count = r.u8()?;
        if count == 0 || frag >= count {
            return Err(ProtocolError::DeserializationFailed("bad fragment index".to_string()));
        }
        let data = r.rest().to_vec();

        // Distance ahead of the next expected fragment; >= 0x8000 means already delivered.
        let ahead = seq.wrapping_sub(self.next_expected);
        if (RECEIVE_WINDOW..0x8000).contains(&ahead) {
            // Too far ahead to buffer: no ack, so the sender resends it later.
            return Ok((Vec::new(), Vec::new()));
        }
        if ahead < RECEIVE_WINDOW {
            self.early.entry(seq).or_insert((if frag + 1 == count { 1 } else { 0 }, data));
        }

        let mut ack = wire::begin(KIND_ACK, 2);
        ack.extend_from_slice(&seq.to_le_bytes());

        let mut delivered = Vec::new();
        while let Some((is_last, bytes)) = self.early.remove(&self.next_expected) {
            self.next_expected = self.next_expected.wrapping_add(1);
            self.assembling.extend_from_slice(&bytes);
            if is_last == 1 {
                delivered.push(std::mem::take(&mut self.assembling));
            }
        }
        Ok((ack, delivered))
    }

    /// Handles a `KIND_ACK` payload.
    pub fn on_ack(&mut self, payload: &[u8]) -> Result<(), ProtocolError> {
        let seq = Reader::new(payload).u16()?;
        self.pending.retain(|p| p.seq != seq);
        Ok(())
    }

    /// True once a fragment was sent `MAX_TRIES` times without an ack.
    pub fn has_failed(&self) -> bool {
        self.failed
    }

    /// Number of fragments waiting for an ack.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::wire::split;

    /// Delivers datagrams between two channels with a deterministic drop pattern.
    fn exchange(a: &mut ReliableChannel, b: &mut ReliableChannel, sent: Vec<Vec<u8>>, drop: &mut impl FnMut() -> bool) -> Vec<Vec<u8>> {
        let mut delivered = Vec::new();
        for d in sent {
            if drop() {
                continue;
            }
            let (_, payload) = split(&d).unwrap();
            let (ack, msgs) = b.on_reliable(payload).unwrap();
            delivered.extend(msgs);
            if !drop() {
                let (_, ack_payload) = split(&ack).unwrap();
                a.on_ack(ack_payload).unwrap();
            }
        }
        delivered
    }

    #[test]
    fn test_every_message_arrives_once_in_order_with_30_percent_loss() {
        let mut a = ReliableChannel::new();
        let mut b = ReliableChannel::new();
        let mut rng = 0x1234_5678u64;
        let mut drop = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % 100) < 30
        };

        let messages: Vec<Vec<u8>> = (0..200u32)
            .map(|i| {
                // Some messages need several fragments.
                let len = if i % 17 == 0 { MAX_FRAGMENT_PAYLOAD * 2 + 5 } else { (i % 50) as usize };
                (0..len).map(|k| (i as usize + k) as u8).collect()
            })
            .collect();

        let mut now = 0.0;
        let mut delivered = Vec::new();
        for m in &messages {
            let sent = a.send(m, now).unwrap();
            delivered.extend(exchange(&mut a, &mut b, sent, &mut drop));
            now += 0.016;
            let resent = a.poll_resend(now);
            delivered.extend(exchange(&mut a, &mut b, resent, &mut drop));
        }
        for _ in 0..400 {
            now += 0.05;
            let resent = a.poll_resend(now);
            delivered.extend(exchange(&mut a, &mut b, resent, &mut drop));
            if a.pending_count() == 0 {
                break;
            }
        }

        assert!(!a.has_failed());
        assert_eq!(a.pending_count(), 0);
        assert_eq!(delivered, messages, "exactly once and in order");
    }

    #[test]
    fn test_duplicates_are_acked_but_not_delivered_twice() {
        let mut a = ReliableChannel::new();
        let mut b = ReliableChannel::new();
        let d = a.send(b"hello", 0.0).unwrap().remove(0);
        let (_, payload) = split(&d).unwrap();
        let (_, first) = b.on_reliable(payload).unwrap();
        let (ack, second) = b.on_reliable(payload).unwrap();
        assert_eq!(first, vec![b"hello".to_vec()]);
        assert!(second.is_empty());
        let (kind, _) = split(&ack).unwrap();
        assert_eq!(kind, KIND_ACK);
    }

    #[test]
    fn test_channel_fails_after_max_tries_without_ack() {
        let mut a = ReliableChannel::new();
        a.send(b"x", 0.0).unwrap();
        let mut now = 0.0;
        for _ in 0..MAX_TRIES + 2 {
            now += RESEND_INTERVAL_SEC + 0.001;
            a.poll_resend(now);
        }
        assert!(a.has_failed());
    }

    #[test]
    fn test_sequence_wraps_around() {
        let mut a = ReliableChannel::new();
        let mut b = ReliableChannel::new();
        a.next_seq = u16::MAX - 2;
        b.next_expected = u16::MAX - 2;
        for i in 0..6u8 {
            let d = a.send(&[i], 0.0).unwrap().remove(0);
            let (_, payload) = split(&d).unwrap();
            let (_, msgs) = b.on_reliable(payload).unwrap();
            assert_eq!(msgs, vec![vec![i]]);
        }
    }

    #[test]
    fn test_rejects_malformed_fragment() {
        let mut b = ReliableChannel::new();
        assert!(b.on_reliable(&[0]).is_err());
        assert!(b.on_reliable(&[0, 0, 2, 1]).is_err(), "frag >= count");
        assert!(b.on_reliable(&[0, 0, 0, 0]).is_err(), "count == 0");
    }
}
