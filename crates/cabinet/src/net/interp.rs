//! # Cabinet LAN Remote Car Interpolation
//!
//! Buffers the owner states of one remote car on the shared race clock and
//! produces a smooth pose for any render time: Hermite interpolation between
//! two states (using their velocities), bounded extrapolation when the buffer
//! runs dry, and a short blend instead of a snap when fresh states resume.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md` §2.3.

use std::collections::VecDeque;

use super::wire::NetCarState;

/// Remote cars are shown this far behind the shared race clock.
pub const INTERP_DELAY_SEC: f64 = 0.100;
/// Longest time a remote car is moved past its newest state.
pub const MAX_EXTRAPOLATION_SEC: f64 = 0.250;
/// Duration of the correction blend after an extrapolation.
pub const RECOVERY_BLEND_SEC: f64 = 0.100;
/// States kept per remote car.
pub const BUFFER_LEN: usize = 16;

fn secs(state: &NetCarState) -> f64 {
    state.time_ms as f64 / 1000.0
}

fn lerp(a: f32, b: f32, u: f32) -> f32 {
    a + (b - a) * u
}

fn lerp_angle(a: f32, b: f32, u: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut d = (b - a) % tau;
    if d > std::f32::consts::PI {
        d -= tau;
    } else if d < -std::f32::consts::PI {
        d += tau;
    }
    a + d * u
}

/// Blend correction left over from an extrapolation.
#[derive(Debug, Clone, Copy)]
struct Correction {
    dx: f32,
    dy: f32,
    dangle: f32,
    started_at: f64,
}

/// State buffer and interpolator for one remote car.
#[derive(Debug, Clone, Default)]
pub struct RemoteCarBuffer {
    states: VecDeque<NetCarState>,
    last_output: Option<NetCarState>,
    was_extrapolating: bool,
    extrapolation_sec: f64,
    correction: Option<Correction>,
    dropped: u64,
}

impl RemoteCarBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a state. Returns false (and drops it) when it is not newer than the newest state.
    pub fn push(&mut self, state: NetCarState) -> bool {
        if self.states.back().is_some_and(|b| state.time_ms <= b.time_ms) {
            self.dropped += 1;
            return false;
        }
        self.states.push_back(state);
        while self.states.len() > BUFFER_LEN {
            self.states.pop_front();
        }
        true
    }

    /// Newest received state.
    pub fn latest(&self) -> Option<&NetCarState> {
        self.states.back()
    }

    /// Number of buffered states.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Time the last sample was moved past the newest state, in seconds.
    pub fn extrapolation_sec(&self) -> f64 {
        self.extrapolation_sec
    }

    /// States dropped as older than or equal to the newest one.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Pose at `render_time` on the shared race clock (seconds).
    pub fn sample(&mut self, render_time: f64) -> Option<NetCarState> {
        let first = *self.states.front()?;
        let newest = *self.states.back()?;

        let (mut out, extrapolating) = if render_time <= secs(&first) {
            (first, false)
        } else if render_time >= secs(&newest) {
            let ahead = (render_time - secs(&newest)).min(MAX_EXTRAPOLATION_SEC);
            self.extrapolation_sec = render_time - secs(&newest);
            let dt = ahead as f32;
            let mut s = newest;
            s.pos_x += s.vel_x * dt;
            s.pos_y += s.vel_y * dt;
            s.angle += s.angular_velocity * dt;
            (s, ahead > 0.0)
        } else {
            let idx = self.states.iter().position(|s| secs(s) > render_time).unwrap_or(self.states.len() - 1);
            let a = self.states[idx - 1];
            let b = self.states[idx];
            (Self::hermite(&a, &b, render_time), false)
        };
        if !extrapolating {
            self.extrapolation_sec = 0.0;
        }

        // Fresh data after an extrapolation: blend from the shown pose instead of snapping.
        if self.was_extrapolating && !extrapolating {
            if let Some(prev) = self.last_output {
                self.correction = Some(Correction {
                    dx: prev.pos_x - out.pos_x,
                    dy: prev.pos_y - out.pos_y,
                    dangle: lerp_angle(out.angle, prev.angle, 1.0) - out.angle,
                    started_at: render_time,
                });
            }
        }
        if let Some(c) = self.correction {
            let remaining = 1.0 - ((render_time - c.started_at) / RECOVERY_BLEND_SEC).clamp(0.0, 1.0);
            if remaining <= 0.0 {
                self.correction = None;
            } else {
                let k = remaining as f32;
                out.pos_x += c.dx * k;
                out.pos_y += c.dy * k;
                out.angle += c.dangle * k;
            }
        }

        // Keep one state at or before the render time; older ones are no longer needed.
        while self.states.len() > 2 && secs(&self.states[1]) <= render_time {
            self.states.pop_front();
        }

        self.was_extrapolating = extrapolating;
        self.last_output = Some(out);
        Some(out)
    }

    fn hermite(a: &NetCarState, b: &NetCarState, t: f64) -> NetCarState {
        let span = (secs(b) - secs(a)) as f32;
        let u = (((t - secs(a)) as f32) / span).clamp(0.0, 1.0);
        let (u2, u3) = (u * u, u * u * u);
        let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
        let h10 = u3 - 2.0 * u2 + u;
        let h01 = -2.0 * u3 + 3.0 * u2;
        let h11 = u3 - u2;
        // Derivatives of the basis, for a velocity that matches the curve.
        let d00 = 6.0 * u2 - 6.0 * u;
        let d10 = 3.0 * u2 - 4.0 * u + 1.0;
        let d01 = -6.0 * u2 + 6.0 * u;
        let d11 = 3.0 * u2 - 2.0 * u;

        let pos = |p0: f32, v0: f32, p1: f32, v1: f32| h00 * p0 + h10 * span * v0 + h01 * p1 + h11 * span * v1;
        let vel = |p0: f32, v0: f32, p1: f32, v1: f32| (d00 * p0 + d10 * span * v0 + d01 * p1 + d11 * span * v1) / span;

        // Discrete fields come from the nearer state in time.
        let near = if u < 0.5 { a } else { b };
        NetCarState {
            slot: b.slot,
            time_ms: (t * 1000.0) as u32,
            pos_x: pos(a.pos_x, a.vel_x, b.pos_x, b.vel_x),
            pos_y: pos(a.pos_y, a.vel_y, b.pos_y, b.vel_y),
            vel_x: vel(a.pos_x, a.vel_x, b.pos_x, b.vel_x),
            vel_y: vel(a.pos_y, a.vel_y, b.pos_y, b.vel_y),
            angle: lerp_angle(a.angle, b.angle, u),
            angular_velocity: lerp(a.angular_velocity, b.angular_velocity, u),
            steer_angle: lerp(a.steer_angle, b.steer_angle, u),
            elevation: lerp(a.elevation, b.elevation, u),
            vertical_velocity: lerp(a.vertical_velocity, b.vertical_velocity, u),
            throttle: near.throttle,
            brake: near.brake,
            flags: near.flags,
            lap: near.lap,
            checkpoint: near.checkpoint,
            progress: near.progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// True pose of a car driving a circle of radius `r` at `speed`.
    fn circle(t: f64, r: f32, speed: f32) -> NetCarState {
        let w = speed / r;
        let a = w * t as f32;
        NetCarState {
            time_ms: (t * 1000.0).round() as u32,
            pos_x: r * a.cos(),
            pos_y: r * a.sin(),
            vel_x: -speed * a.sin(),
            vel_y: speed * a.cos(),
            angle: a + std::f32::consts::FRAC_PI_2,
            angular_velocity: w,
            ..Default::default()
        }
    }

    #[test]
    fn test_circle_with_loss_and_jitter_stays_within_one_metre() {
        let mut rng = 99u64;
        let mut next = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % 10_000) as f64 / 10_000.0
        };

        // States sent at 60 Hz with 5% loss and 0..20 ms jitter; arrival order may change.
        let mut in_flight: Vec<(f64, NetCarState)> = Vec::new();
        for i in 0..(60 * 20) {
            let t = i as f64 / 60.0;
            if next() < 0.05 {
                continue;
            }
            in_flight.push((t + 0.002 + next() * 0.020, circle(t, 60.0, 30.0)));
        }
        in_flight.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut buf = RemoteCarBuffer::new();
        let mut errors = Vec::new();
        let mut k = 0;
        let mut frame = 0.5;
        while frame < 19.5 {
            while k < in_flight.len() && in_flight[k].0 <= frame {
                buf.push(in_flight[k].1);
                k += 1;
            }
            let render = frame - INTERP_DELAY_SEC;
            let shown = buf.sample(render).unwrap();
            let truth = circle(render, 60.0, 30.0);
            errors.push(((shown.pos_x - truth.pos_x).powi(2) + (shown.pos_y - truth.pos_y).powi(2)).sqrt());
            assert!(buf.extrapolation_sec() <= MAX_EXTRAPOLATION_SEC);
            frame += 1.0 / 144.0;
        }
        errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = errors[errors.len() * 95 / 100];
        assert!(p95 <= 1.0, "p95 position error {p95} m");
        assert!(buf.dropped() > 0 || k == in_flight.len());
    }

    #[test]
    fn test_old_and_duplicate_states_are_dropped() {
        let mut buf = RemoteCarBuffer::new();
        assert!(buf.push(circle(0.10, 50.0, 20.0)));
        assert!(!buf.push(circle(0.10, 50.0, 20.0)), "duplicate");
        assert!(!buf.push(circle(0.05, 50.0, 20.0)), "older");
        assert!(buf.push(circle(0.12, 50.0, 20.0)));
        assert_eq!(buf.dropped(), 2);
    }

    #[test]
    fn test_extrapolation_is_bounded_then_holds() {
        let mut buf = RemoteCarBuffer::new();
        buf.push(NetCarState { time_ms: 0, vel_x: 10.0, ..Default::default() });
        buf.push(NetCarState { time_ms: 100, pos_x: 1.0, vel_x: 10.0, ..Default::default() });
        let a = buf.sample(0.2).unwrap();
        assert!((a.pos_x - 2.0).abs() < 1e-4, "100 ms ahead: {}", a.pos_x);
        let b = buf.sample(2.0).unwrap();
        let limit = 1.0 + 10.0 * MAX_EXTRAPOLATION_SEC as f32;
        assert!((b.pos_x - limit).abs() < 1e-4, "held at the limit: {}", b.pos_x);
    }

    #[test]
    fn test_fresh_state_after_extrapolation_blends_without_snap() {
        let mut buf = RemoteCarBuffer::new();
        buf.push(NetCarState { time_ms: 0, vel_x: 10.0, ..Default::default() });
        buf.push(NetCarState { time_ms: 100, pos_x: 1.0, vel_x: 10.0, ..Default::default() });
        let shown = buf.sample(0.3).unwrap(); // extrapolated to x = 3.0
        // The car actually stopped at x = 1.5.
        buf.push(NetCarState { time_ms: 400, pos_x: 1.5, ..Default::default() });
        let next = buf.sample(0.3 + 1.0 / 144.0).unwrap();
        assert!((next.pos_x - shown.pos_x).abs() < 0.3, "no snap: {} -> {}", shown.pos_x, next.pos_x);
        let settled = buf.sample(0.3 + RECOVERY_BLEND_SEC + 0.01).unwrap();
        let raw = RemoteCarBuffer::hermite(
            &NetCarState { time_ms: 100, pos_x: 1.0, vel_x: 10.0, ..Default::default() },
            &NetCarState { time_ms: 400, pos_x: 1.5, ..Default::default() },
            0.3 + RECOVERY_BLEND_SEC + 0.01,
        );
        assert!((settled.pos_x - raw.pos_x).abs() < 1e-4, "blend finished");
    }

    #[test]
    fn test_angle_takes_the_short_way_round() {
        let a = NetCarState { time_ms: 0, angle: 3.1, ..Default::default() };
        let b = NetCarState { time_ms: 100, angle: -3.1, ..Default::default() };
        let mid = RemoteCarBuffer::hermite(&a, &b, 0.05);
        assert!(mid.angle.abs() > 3.0, "must cross PI, not zero: {}", mid.angle);
    }
}
