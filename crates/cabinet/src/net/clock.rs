//! # Cabinet LAN Clock Synchronization
//!
//! Estimates the offset between the local monotonic clock and the host's
//! clock from ping/pong round trips, so that every machine agrees on the
//! shared race clock (seconds since the green light).
//!
//! `offset = host_time - (local_send + rtt / 2)`, taken from the sample with
//! the lowest round-trip time among the most recent `SAMPLE_WINDOW` samples.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.2.

use std::collections::VecDeque;

/// Number of recent ping samples considered.
pub const SAMPLE_WINDOW: usize = 8;

#[derive(Debug, Clone, Copy)]
struct Sample {
    rtt: f64,
    offset: f64,
}

/// Local-to-host clock offset estimator.
#[derive(Debug, Clone, Default)]
pub struct ClockSync {
    samples: VecDeque<Sample>,
}

impl ClockSync {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one round trip: sent at `local_send`, answered with the peer time
    /// `remote_time`, received at `local_recv` (all in seconds).
    pub fn add_sample(&mut self, local_send: f64, remote_time: f64, local_recv: f64) {
        let rtt = local_recv - local_send;
        if !(rtt.is_finite() && remote_time.is_finite()) || rtt < 0.0 {
            return;
        }
        if self.samples.len() == SAMPLE_WINDOW {
            self.samples.pop_front();
        }
        self.samples.push_back(Sample { rtt, offset: remote_time - (local_send + rtt / 2.0) });
    }

    fn best(&self) -> Option<Sample> {
        self.samples
            .iter()
            .copied()
            .min_by(|a, b| a.rtt.partial_cmp(&b.rtt).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// `host_time - local_time`, once at least one sample exists.
    pub fn offset(&self) -> Option<f64> {
        self.best().map(|s| s.offset)
    }

    /// Most recent round-trip time in seconds.
    pub fn last_rtt(&self) -> Option<f64> {
        self.samples.back().map(|s| s.rtt)
    }

    /// Converts a local clock reading into host clock time.
    pub fn to_host(&self, local: f64) -> Option<f64> {
        self.offset().map(|o| local + o)
    }

    /// Converts a host clock reading into local clock time.
    pub fn to_local(&self, host: f64) -> Option<f64> {
        self.offset().map(|o| host - o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_uses_lowest_rtt_sample() {
        let mut c = ClockSync::new();
        assert!(c.offset().is_none());
        // Host clock is 100 s ahead. Symmetric 10 ms trip.
        c.add_sample(1.000, 101.005, 1.010);
        // Asymmetric slow trip: 200 ms out, 0 ms back (a bad sample).
        c.add_sample(2.000, 102.200, 2.200);
        let off = c.offset().unwrap();
        assert!((off - 100.0).abs() < 1e-9, "offset {off}");
        assert!((c.to_host(5.0).unwrap() - 105.0).abs() < 1e-9);
        assert!((c.to_local(105.0).unwrap() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_old_samples_leave_the_window() {
        let mut c = ClockSync::new();
        c.add_sample(0.0, 50.0, 0.0); // perfect sample, offset 50
        for i in 0..SAMPLE_WINDOW {
            let t = 10.0 + i as f64;
            c.add_sample(t, t + 7.001, t + 0.002);
        }
        assert!((c.offset().unwrap() - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_rejects_negative_or_non_finite_samples() {
        let mut c = ClockSync::new();
        c.add_sample(1.0, 2.0, 0.5);
        c.add_sample(1.0, f64::NAN, 1.1);
        assert!(c.offset().is_none());
    }
}
