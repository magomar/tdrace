use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::{normalize_angle, Car};
use tdrace_core::CarConfig;

use crate::render::track::draw_quad;
use crate::ui::menu::{CarChoice, TrackChoice};

/// Shortest angular interpolation handling wrapping around (-PI, PI].
#[inline]
pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = normalize_angle(b - a);
    normalize_angle(a + diff * t)
}

/// Instantaneous telemetry sample captured during a ghost lap.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GhostFrame {
    /// Lap elapsed time in seconds from start line crossing.
    pub time: f32,
    /// World coordinate position.
    pub position: Vec2,
    /// Yaw orientation angle in radians.
    pub angle: f32,
    /// Front wheel steering angle in radians.
    pub steer_angle: f32,
    /// Scalar vehicle speed in m/s.
    pub speed: f32,
}

/// Complete telemetry recording of a single personal best lap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostLap {
    pub track_choice: TrackChoice,
    pub car_choice: CarChoice,
    pub lap_time: f32,
    pub frames: Vec<GhostFrame>,
}

impl GhostLap {
    pub fn new(track_choice: TrackChoice, car_choice: CarChoice, lap_time: f32, frames: Vec<GhostFrame>) -> Self {
        Self {
            track_choice,
            car_choice,
            lap_time,
            frames,
        }
    }

    /// Evaluates the interpolated ghost pose at any arbitrary lap time.
    pub fn sample_at_time(&self, lap_time: f32) -> Option<GhostFrame> {
        if self.frames.is_empty() {
            return None;
        }

        let first = self.frames.first().unwrap();
        if lap_time <= first.time {
            return Some(*first);
        }

        let last = self.frames.last().unwrap();
        if lap_time >= last.time {
            return Some(*last);
        }

        // Binary search for neighboring frames
        let idx = match self.frames.binary_search_by(|f| f.time.partial_cmp(&lap_time).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(i) => return Some(self.frames[i]),
            Err(i) => i.saturating_sub(1),
        };

        if idx + 1 >= self.frames.len() {
            return Some(*last);
        }

        let f0 = &self.frames[idx];
        let f1 = &self.frames[idx + 1];

        let dt = (f1.time - f0.time).max(1e-5);
        let t = ((lap_time - f0.time) / dt).clamp(0.0, 1.0);

        let pos = f0.position.lerp(f1.position, t);
        let angle = lerp_angle(f0.angle, f1.angle, t);
        let steer_angle = f0.steer_angle + (f1.steer_angle - f0.steer_angle) * t;
        let speed = f0.speed + (f1.speed - f0.speed) * t;

        Some(GhostFrame {
            time: lap_time,
            position: pos,
            angle,
            steer_angle,
            speed,
        })
    }
}

/// Ghost telemetry recorder and personal best lap tracker.
#[derive(Debug, Clone, PartialEq)]
pub struct GhostRecorder {
    pub current_lap_frames: Vec<GhostFrame>,
    pub best_ghost_lap: Option<GhostLap>,
    pub sample_interval: f32,
    pub time_since_last_sample: f32,
}

impl Default for GhostRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostRecorder {
    pub fn new() -> Self {
        Self {
            current_lap_frames: Vec::with_capacity(3600), // preallocate ~60s @ 60Hz
            best_ghost_lap: None,
            sample_interval: 1.0 / 60.0,
            time_since_last_sample: 0.0,
        }
    }

    /// Records car telemetry at regular fixed intervals during active lap.
    pub fn record_frame(&mut self, lap_time: f32, car: &Car, dt: f32) {
        self.time_since_last_sample += dt;

        if self.time_since_last_sample >= self.sample_interval || self.current_lap_frames.is_empty() {
            self.time_since_last_sample = 0.0;
            self.current_lap_frames.push(GhostFrame {
                time: lap_time,
                position: car.state.position,
                angle: car.state.angle,
                steer_angle: car.state.steer_angle,
                speed: car.state.speed,
            });
        }
    }

    /// Called when a lap is completed. If the lap is faster than current personal best,
    /// it becomes the new active ghost lap.
    pub fn on_lap_completed(
        &mut self,
        lap_time: f32,
        track_choice: TrackChoice,
        car_choice: CarChoice,
    ) -> bool {
        let is_new_best = match &self.best_ghost_lap {
            Some(best) => lap_time < best.lap_time,
            None => lap_time > 0.0,
        };

        if is_new_best && !self.current_lap_frames.is_empty() {
            // Append final frame at exact finish time
            if let Some(last) = self.current_lap_frames.last() {
                if last.time < lap_time {
                    let mut final_frame = *last;
                    final_frame.time = lap_time;
                    self.current_lap_frames.push(final_frame);
                }
            }

            self.best_ghost_lap = Some(GhostLap::new(
                track_choice,
                car_choice,
                lap_time,
                std::mem::take(&mut self.current_lap_frames),
            ));
            self.current_lap_frames = Vec::with_capacity(3600);
            self.time_since_last_sample = 0.0;
            true
        } else {
            self.current_lap_frames.clear();
            self.time_since_last_sample = 0.0;
            false
        }
    }

    /// Resets current lap samples when a lap is invalidated (e.g. restart / wrong way).
    pub fn on_lap_invalidated(&mut self) {
        self.current_lap_frames.clear();
        self.time_since_last_sample = 0.0;
    }

    /// Clears all ghost data including personal best.
    pub fn clear_all(&mut self) {
        self.current_lap_frames.clear();
        self.best_ghost_lap = None;
        self.time_since_last_sample = 0.0;
    }
}

/// Renders a semi-transparent ghost vehicle during Time Attack.
pub fn render_ghost_car(frame: &GhostFrame, config: &CarConfig, base_opacity: f32) {
    let pos = frame.position;
    let angle = frame.angle;
    let fwd = Vec2::new(angle.cos(), angle.sin());
    let right = Vec2::new(-angle.sin(), angle.cos());

    let opacity = base_opacity.clamp(0.0, 1.0);

    // Ethereal Spectral Color Scheme
    let ghost_primary = Color::new(0.25, 0.85, 1.0, opacity * 0.45);
    let ghost_glow = Color::new(0.40, 0.95, 1.0, opacity * 0.85);
    let ghost_cockpit = Color::new(0.60, 0.95, 1.0, opacity * 0.60);
    let ghost_wheel = Color::new(0.30, 0.70, 0.90, opacity * 0.40);

    // Car dimensions
    let half_w = config.track_width * 0.5;
    let lf = config.cg_to_front;
    let lr = config.cg_to_rear;
    let total_len = lf + lr + 0.50;
    let body_half_len = total_len * 0.5;
    let body_half_w = half_w + 0.12;

    // 1. Ghost Wheels
    let ackermann_ratio = 1.0;
    let steer_fl = frame.steer_angle * ackermann_ratio;
    let steer_fr = frame.steer_angle * ackermann_ratio;

    let wheel_offsets = [
        fwd * lf - right * half_w,
        fwd * lf + right * half_w,
        -fwd * lr - right * half_w,
        -fwd * lr + right * half_w,
    ];
    let wheel_steers = [steer_fl, steer_fr, 0.0, 0.0];

    for i in 0..4 {
        let w_pos = pos + wheel_offsets[i];
        let w_angle = angle + wheel_steers[i];
        render_ghost_wheel(w_pos, w_angle, ghost_wheel, ghost_glow);
    }

    // 2. Ghost Chassis Hull
    let nose_w = body_half_w * 0.70;
    let tail_w = body_half_w * 0.85;

    let p_nose_l = pos + fwd * body_half_len - right * nose_w;
    let p_nose_r = pos + fwd * body_half_len + right * nose_w;
    let p_side_fl = pos + fwd * (body_half_len * 0.5) - right * body_half_w;
    let p_side_fr = pos + fwd * (body_half_len * 0.5) + right * body_half_w;
    let p_side_rl = pos - fwd * (body_half_len * 0.5) - right * body_half_w;
    let p_side_rr = pos - fwd * (body_half_len * 0.5) + right * body_half_w;
    let p_tail_l = pos - fwd * body_half_len - right * tail_w;
    let p_tail_r = pos - fwd * body_half_len + right * tail_w;

    // Translucent body panels
    draw_quad(p_nose_l, p_nose_r, p_side_fr, p_side_fl, ghost_primary);
    draw_quad(p_side_fl, p_side_fr, p_side_rr, p_side_rl, ghost_primary);
    draw_quad(p_side_rl, p_side_rr, p_tail_r, p_tail_l, ghost_primary);

    // Glowing spectral outlines
    let th = 0.08;
    draw_line(p_nose_l.x, p_nose_l.y, p_nose_r.x, p_nose_r.y, th, ghost_glow);
    draw_line(p_nose_l.x, p_nose_l.y, p_side_fl.x, p_side_fl.y, th, ghost_glow);
    draw_line(p_nose_r.x, p_nose_r.y, p_side_fr.x, p_side_fr.y, th, ghost_glow);
    draw_line(p_side_fl.x, p_side_fl.y, p_side_rl.x, p_side_rl.y, th, ghost_glow);
    draw_line(p_side_fr.x, p_side_fr.y, p_side_rr.x, p_side_rr.y, th, ghost_glow);
    draw_line(p_tail_l.x, p_tail_l.y, p_tail_r.x, p_tail_r.y, th, ghost_glow);

    // 3. Ghost Cockpit
    let cockpit_center = pos - fwd * 0.10;
    let glass_half_len = 0.65;
    let glass_front_w = 0.42;
    let glass_rear_w = 0.48;

    let g_fl = cockpit_center + fwd * glass_half_len - right * glass_front_w;
    let g_fr = cockpit_center + fwd * glass_half_len + right * glass_front_w;
    let g_rr = cockpit_center - fwd * glass_half_len + right * glass_rear_w;
    let g_rl = cockpit_center - fwd * glass_half_len - right * glass_rear_w;

    draw_quad(g_fl, g_fr, g_rr, g_rl, ghost_cockpit);

    // Glowing driver helmet
    let helmet_pos = cockpit_center - fwd * 0.05;
    let helmet_radius = 0.22;
    draw_circle(helmet_pos.x, helmet_pos.y, helmet_radius, ghost_glow);
    draw_circle_lines(helmet_pos.x, helmet_pos.y, helmet_radius, 0.04, Color::new(1.0, 1.0, 1.0, opacity));
}

fn render_ghost_wheel(pos: Vec2, angle: f32, fill: Color, outline: Color) {
    let tire_fwd = Vec2::new(angle.cos(), angle.sin());
    let tire_right = Vec2::new(-angle.sin(), angle.cos());

    let tire_half_len = 0.30;
    let tire_half_w = 0.11;

    let p0 = pos + tire_fwd * tire_half_len - tire_right * tire_half_w;
    let p1 = pos + tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p2 = pos - tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p3 = pos - tire_fwd * tire_half_len - tire_right * tire_half_w;

    draw_quad(p0, p1, p2, p3, fill);
    draw_line(p0.x, p0.y, p1.x, p1.y, 0.05, outline);
    draw_line(p1.x, p1.y, p2.x, p2.y, 0.05, outline);
    draw_line(p2.x, p2.y, p3.x, p3.y, 0.05, outline);
    draw_line(p3.x, p3.y, p0.x, p0.y, 0.05, outline);
}
