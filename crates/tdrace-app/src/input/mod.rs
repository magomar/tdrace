pub mod filter;
pub mod gamepad;
pub mod touch;

use macroquad::color::Color;
use macroquad::input::KeyCode;

#[inline]
fn is_key_pressed(k: KeyCode) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_key_pressed(k)).unwrap_or(false)
}

#[inline]
fn is_key_down(k: KeyCode) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_key_down(k)).unwrap_or(false)
}
use macroquad::shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_lines};
use macroquad::text::draw_text;
use tdrace_core::collision::sat::OrientedBox;
use tdrace_core::lidar::{LidarConfig, LidarHitType, LidarScanner};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;

use crate::ai::BotAiDriver;
use crate::render::color::Palette;
pub use filter::{DigitalInputConfig, DigitalInputFilter};
pub use gamepad::{GamepadConfig, GamepadController, GamepadSnapshot};
pub use touch::{RawTouchPhase, RawTouchPoint, TouchButtonState, TouchController, TouchLayout};

/// Active debug overlay visibility flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DebugOverlays {
    pub lidar: bool,        // F1
    pub checkpoints: bool,  // F2
    pub collision_obb: bool,// F3
    pub ai_paths: bool,     // F4
    pub telemetry: bool,    // F5
}

/// Unified Input and Debug Overlay Controller.
pub struct InputController {
    pub debug: DebugOverlays,
    pub lidar_scanner: LidarScanner,
    pub filter: DigitalInputFilter,
    pub gamepad: GamepadController,
}

impl Default for InputController {
    fn default() -> Self {
        Self::new()
    }
}

impl InputController {
    pub fn new() -> Self {
        Self {
            debug: DebugOverlays::default(),
            lidar_scanner: LidarScanner::new(LidarConfig::surround_32()),
            filter: DigitalInputFilter::default(),
            gamepad: GamepadController::new(),
        }
    }

    /// Resets smoothed input filter state (e.g. on race start / restart).
    pub fn reset(&mut self) {
        self.filter.reset();
    }

    /// Polls player driving controls (Keyboard + Gamepad with progressive smoothing & analog precision).
    pub fn poll_player_controls(&mut self, dt: f32, current_speed_fwd: f32) -> CarControls {
        // 1. Update Gamepad inputs & events
        self.gamepad.update();

        // 2. Poll keyboard raw inputs
        let mut raw_steer = 0.0f32;
        let mut raw_throttle = 0.0f32;
        let mut raw_brake = 0.0f32;
        let mut raw_reverse = false;

        let is_down = is_key_down;

        // Steering: O / Left = Steer Left (-1.0), P / Right = Steer Right (+1.0)
        if is_down(KeyCode::O) || is_down(KeyCode::Left) {
            raw_steer -= 1.0;
        }
        if is_down(KeyCode::P) || is_down(KeyCode::Right) {
            raw_steer += 1.0;
        }

        // Throttle: Q / Up
        if is_down(KeyCode::Q) || is_down(KeyCode::Up) {
            raw_throttle = 1.0;
        }

        // Brake: A / Down
        if is_down(KeyCode::A) || is_down(KeyCode::Down) {
            raw_brake = 1.0;
        }

        // Reverse: Z
        if is_down(KeyCode::Z) {
            raw_throttle = 1.0;
            raw_reverse = true;
        }

        // Handbrake: Space
        let kb_handbrake = is_down(KeyCode::Space);

        self.process_inputs((raw_steer, raw_throttle, raw_brake, kb_handbrake, raw_reverse), dt, current_speed_fwd)
    }

    /// Pure input processing & blending pipeline between keyboard digital ramps and gamepad analog axes.
    pub fn process_inputs(
        &mut self,
        raw_kb: (f32, f32, f32, bool, bool),
        dt: f32,
        current_speed_fwd: f32,
    ) -> CarControls {
        let (raw_steer, raw_throttle, raw_brake, kb_handbrake, raw_reverse) = raw_kb;

        // Apply digital input smoothing, progressive ramps & speed-sensitive attenuation to keyboard
        let speed_abs = current_speed_fwd.abs();
        let (kb_steer, kb_throttle, kb_brake) = self.filter.update(raw_steer, raw_throttle, raw_brake, speed_abs, dt);

        // Blend Keyboard and Analog Gamepad controls seamlessly
        let gp = &self.gamepad.snapshot;
        let steer = if gp.steer.abs() > 0.001 {
            (kb_steer + gp.steer).clamp(-1.0, 1.0)
        } else {
            kb_steer
        };
        let throttle = kb_throttle.max(gp.throttle).clamp(0.0, 1.0);
        let brake = kb_brake.max(gp.brake).clamp(0.0, 1.0);
        let handbrake = kb_handbrake || gp.handbrake;
        let reverse = raw_reverse || gp.reverse;

        CarControls {
            throttle,
            steer,
            brake,
            handbrake,
            reverse,
        }
    }

    /// Combines keyboard/gamepad controls with touch input controls.
    pub fn combine_controls(kb: CarControls, touch: CarControls) -> CarControls {
        let throttle = (kb.throttle + touch.throttle).clamp(0.0, 1.0);
        let steer = (kb.steer + touch.steer).clamp(-1.0, 1.0);
        let brake = (kb.brake + touch.brake).clamp(0.0, 1.0);
        let handbrake = kb.handbrake || touch.handbrake;
        let reverse = kb.reverse || touch.reverse;

        CarControls {
            throttle,
            steer,
            brake,
            handbrake,
            reverse,
        }
    }

    /// Updates debug overlay toggle keys (F1, F2, F3, F4, F5).
    pub fn update_debug_toggles(&mut self) {
        if is_key_pressed(KeyCode::F1) {
            self.debug.lidar = !self.debug.lidar;
        }
        if is_key_pressed(KeyCode::F2) {
            self.debug.checkpoints = !self.debug.checkpoints;
        }
        if is_key_pressed(KeyCode::F3) {
            self.debug.collision_obb = !self.debug.collision_obb;
        }
        if is_key_pressed(KeyCode::F4) {
            self.debug.ai_paths = !self.debug.ai_paths;
        }
        if is_key_pressed(KeyCode::F5) {
            self.debug.telemetry = !self.debug.telemetry;
        }
    }

    /// Renders all active debug overlays in world coordinate space.
    pub fn render_world_debug(
        &self,
        player_car: &Car,
        all_cars: &[Car],
        track: &Track,
        progress: &TrackProgressTracker,
        ai_drivers: &[BotAiDriver],
    ) {
        // F1: LIDAR Beams
        if self.debug.lidar {
            self.render_lidar(player_car, track, all_cars);
        }

        // F2: Checkpoint gates & racing direction
        if self.debug.checkpoints {
            self.render_checkpoints(track, progress);
        }

        // F3: Collision OBBs & Wheel Vectors
        if self.debug.collision_obb {
            self.render_collision_obbs(all_cars, track);
        }

        // F4: AI Paths and Target Waypoints
        if self.debug.ai_paths {
            self.render_ai_paths(all_cars, track, ai_drivers);
        }
    }

    /// Renders screen-space debug telemetry panel (F5).
    pub fn render_screen_debug(&self, player_car: &Car) {
        if self.debug.telemetry {
            self.render_telemetry_panel(player_car);
        }
    }

    /// F1: LIDAR Raycast Visualization.
    fn render_lidar(&self, car: &Car, track: &Track, opponents: &[Car]) {
        let fwd = car.forward_vector();
        let sensor_pos = car.state.position + fwd * self.lidar_scanner.config.offset_forward;
        let hits = self.lidar_scanner.scan(car, track, opponents);

        for hit in &hits {
            let (beam_col, hit_col) = match hit.hit_type {
                LidarHitType::TrackWall => (Color::new(0.95, 0.2, 0.2, 0.45), Color::new(1.0, 0.1, 0.1, 0.9)),
                LidarHitType::Obstacle => (Color::new(1.0, 0.8, 0.1, 0.45), Color::new(1.0, 0.8, 0.1, 0.9)),
                LidarHitType::OpponentCar => (Color::new(0.2, 0.8, 1.0, 0.55), Color::new(0.2, 0.8, 1.0, 0.95)),
                LidarHitType::None => (Color::new(0.2, 0.9, 0.2, 0.20), Color::new(0.2, 0.9, 0.2, 0.3)),
            };

            let end_pt = hit.hit_point;

            draw_line(sensor_pos.x, sensor_pos.y, end_pt.x, end_pt.y, 0.08, beam_col);

            if hit.hit_type != LidarHitType::None {
                draw_circle(hit.hit_point.x, hit.hit_point.y, 0.20, hit_col);
                // Hit normal indicator
                let norm_end = hit.hit_point + hit.hit_normal * 0.6;
                draw_line(hit.hit_point.x, hit.hit_point.y, norm_end.x, norm_end.y, 0.06, Palette::WHITE);
            }
        }

        // Sensor emitter circle
        draw_circle(sensor_pos.x, sensor_pos.y, 0.25, Color::new(0.2, 1.0, 0.2, 0.9));
    }

    /// F2: Checkpoint Gates & Race Direction.
    fn render_checkpoints(&self, track: &Track, progress: &TrackProgressTracker) {
        for cp in &track.checkpoints {
            let is_next = cp.id == progress.next_checkpoint_idx;
            let gate_col = if cp.is_finish_line {
                Color::new(1.0, 0.9, 0.1, 0.9)
            } else if is_next {
                Color::new(0.1, 0.95, 0.95, 0.95)
            } else {
                Color::new(0.4, 0.8, 0.4, 0.55)
            };

            let thickness = if is_next { 0.35 } else { 0.18 };
            draw_line(cp.gate.start.x, cp.gate.start.y, cp.gate.end.x, cp.gate.end.y, thickness, gate_col);

            // Forward direction arrow from gate center
            let center = (cp.gate.start + cp.gate.end) * 0.5;
            let arrow_tip = center + cp.direction * 2.2;
            draw_line(center.x, center.y, arrow_tip.x, arrow_tip.y, 0.20, gate_col);

            // Gate ID label
            let label = format!("CP{} (S{})", cp.id, cp.sector);
            draw_text(&label, center.x - 1.0, center.y - 0.5, 0.9, Palette::WHITE);
        }
    }

    /// F3: Collision OBBs, Velocities, and Wheel Contacts.
    fn render_collision_obbs(&self, cars: &[Car], _track: &Track) {
        for car in cars {
            let obb = OrientedBox::from_car(car);
            let corners = obb.corners();

            // Draw OBB rectangle edges
            let obb_col = if car.state.is_drifting {
                Color::new(1.0, 0.3, 0.8, 0.9)
            } else {
                Color::new(0.2, 0.9, 0.3, 0.9)
            };

            for i in 0..4 {
                let next = (i + 1) % 4;
                draw_line(corners[i].x, corners[i].y, corners[next].x, corners[next].y, 0.12, obb_col);
            }

            // Linear velocity vector
            let v_end = car.state.position + car.state.velocity * 0.25;
            draw_line(car.state.position.x, car.state.position.y, v_end.x, v_end.y, 0.15, Color::new(0.2, 0.6, 1.0, 0.9));

            // Wheel contact vectors
            let wheel_pos = car.wheel_positions_world();
            for (w, &p) in wheel_pos.iter().enumerate() {
                let slip_col = if car.state.wheels[w].skid_intensity > 0.1 {
                    Color::new(1.0, 0.2, 0.2, 0.9)
                } else {
                    Color::new(0.8, 0.8, 0.8, 0.6)
                };
                draw_circle(p.x, p.y, 0.15, slip_col);
            }
        }
    }

    /// F4: AI Lookahead Targets & Planned Trajectory.
    fn render_ai_paths(&self, cars: &[Car], track: &Track, ai_drivers: &[BotAiDriver]) {
        for (i, ai) in ai_drivers.iter().enumerate() {
            if let Some(bot_car) = cars.get(i + 1) {
                let target_sample = track.spline.sample_at_distance(ai.current_target_dist);
                let target_pos = target_sample.point;

                // Draw line from bot car to its target waypoint
                draw_line(
                    bot_car.state.position.x,
                    bot_car.state.position.y,
                    target_pos.x,
                    target_pos.y,
                    0.15,
                    Color::new(1.0, 0.6, 0.1, 0.8),
                );
                draw_circle(target_pos.x, target_pos.y, 0.40, Color::new(1.0, 0.6, 0.1, 0.9));
            }
        }
    }

    /// F5: Screen-space detailed physics telemetry HUD overlay.
    fn render_telemetry_panel(&self, car: &Car) {
        let x = 18.0;
        let y = 140.0;
        let w = 290.0;
        let h = 260.0;

        draw_rectangle(x, y, w, h, Color::new(0.05, 0.06, 0.08, 0.88));
        draw_rectangle_lines(x, y, w, h, 1.5, Color::new(0.3, 0.6, 0.9, 0.8));

        let mut row_y = y + 22.0;
        let line_h = 17.0;

        draw_text("=== VEHICLE TELEMETRY ===", x + 10.0, row_y, 16.0, Color::new(0.3, 0.85, 1.0, 1.0));
        row_y += line_h * 1.2;

        let texts = [
            format!("Speed: {:.1} km/h ({:.1} m/s)", car.speed_kmh(), car.state.speed),
            format!("Heading: {:.1} deg ({:.2} rad)", car.state.angle.to_degrees(), car.state.angle),
            format!("Yaw Rate: {:.2} rad/s", car.state.angular_velocity),
            format!("Steer Angle: {:.1} deg", car.state.steer_angle.to_degrees()),
            format!("Sideslip (Drift): {:.1} deg", car.state.sideslip_angle.to_degrees()),
            format!("Is Drifting: {}", if car.state.is_drifting { "YES" } else { "NO" }),
            format!("Drift Score: {:.0}", car.state.drift_score),
            format!("Local Accel: X={:.1} Y={:.1} m/s²", car.state.acceleration_local.x, car.state.acceleration_local.y),
            format!("FL Skid: {:.0}% | FR Skid: {:.0}%", car.state.wheels[0].skid_intensity * 100.0, car.state.wheels[1].skid_intensity * 100.0),
            format!("RL Skid: {:.0}% | RR Skid: {:.0}%", car.state.wheels[2].skid_intensity * 100.0, car.state.wheels[3].skid_intensity * 100.0),
        ];

        for t in &texts {
            draw_text(t, x + 10.0, row_y, 14.0, Color::new(0.9, 0.92, 0.95, 1.0));
            row_y += line_h;
        }
    }
}
