//! # LAN race session (owner-authoritative cars)
//!
//! Each machine simulates only its own car and streams its state. The other
//! players' cars are kinematic: they are placed from interpolated owner
//! states, shown `INTERP_DELAY_SEC` behind the shared race clock. The host
//! relays the states and referees the race; every machine shows the host's
//! results.
//!
//! See `specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md`.

use std::collections::HashMap;

use cabinet::net::wire::flags;
use cabinet::net::{
    ClientEvent, FinishRecord, HostEvent, LanCollisionMode, NetCarState, RaceConfig, RaceResult,
    RaceStatus, RemoteCarBuffer, INTERP_DELAY_SEC,
};
use glam::Vec2;
use tdrace_core::physics::car::CarControls;

use macroquad::input::KeyCode;
use macroquad::shapes::draw_rectangle;

use super::{get_frame_time_safe, is_key_pressed, screen_height_safe, screen_width_safe, GameState, RaceSession};
use crate::audio::SfxType;
use crate::input::InputController;
use crate::render::color::Palette;

/// Countdown value shown while the host waits for every racer to load.
pub const LAN_WAITING_COUNTDOWN: f32 = 999.0;
/// Minimum time between two own-car state packets (60 Hz).
const STATE_SEND_INTERVAL_SEC: f64 = 1.0 / 60.0;
/// Time the "host left" message stays up before the results.
const HOST_LEFT_RESULTS_DELAY_SEC: f32 = 3.0;

/// Per-race LAN state kept by the game session.
#[derive(Debug, Clone)]
pub struct LanRaceState {
    /// Frozen roster and rules. Car index `i` is `config.roster[i]`.
    pub config: RaceConfig,
    /// Interpolation buffers of the other players' cars, by slot.
    pub remote: HashMap<u8, RemoteCarBuffer>,
    /// Sampled driver controls received from the other players' cars, by slot.
    pub remote_controls: HashMap<u8, CarControls>,
    /// Slots that left the race.
    pub left: Vec<u8>,
    /// Finish order from the host.
    pub standings: Vec<FinishRecord>,
    /// Final results from the host (or built locally when the host left).
    pub results: Option<Vec<RaceResult>>,
    /// The local car crossed the finish line and was reported.
    pub local_finished: bool,
    /// The host left or timed out (clients only).
    pub host_left: bool,
    /// Seconds left before the results screen after the host left.
    pub host_left_timer: f32,
    /// Race clock of the last own-state packet.
    pub last_send_clock: f64,
    /// Controls applied to the own car in the last physics step.
    pub last_controls: CarControls,
    /// Hold the own car still (brake) this frame: pause menu, other screen, or finished.
    pub hold_input: bool,
    /// Scripted own-car controls (headless tests and demos) instead of the keyboard.
    pub input_override: Option<CarControls>,
    /// Dev net HUD visible (F9, dev mode only).
    pub show_net_hud: bool,
}

impl LanRaceState {
    pub fn new(config: RaceConfig) -> Self {
        Self {
            config,
            remote: HashMap::new(),
            remote_controls: HashMap::new(),
            left: Vec::new(),
            standings: Vec::new(),
            results: None,
            local_finished: false,
            host_left: false,
            host_left_timer: 0.0,
            last_send_clock: f64::NEG_INFINITY,
            last_controls: CarControls::default(),
            hold_input: false,
            input_override: None,
            show_net_hud: false,
        }
    }

    /// Slot that owns car `car_idx`.
    pub fn slot_of(&self, car_idx: usize) -> Option<u8> {
        self.config.roster.get(car_idx).map(|e| e.slot_id)
    }
}

impl RaceSession {
    /// Shared race clock in seconds since the green light, once the start is scheduled.
    pub fn lan_race_clock(&self) -> Option<f64> {
        if let Some(ref host) = self.lan_host {
            host.race_clock()
        } else if let Some(ref client) = self.lan_client {
            client.race_clock()
        } else {
            None
        }
    }

    /// True for a car owned by another player in a LAN race.
    pub fn lan_is_remote_car(&self, car_idx: usize) -> bool {
        self.lan_race
            .as_ref()
            .and_then(|l| l.slot_of(car_idx))
            .is_some_and(|slot| slot != self.lan_player_slot)
    }

    /// True for a car whose player left the race.
    pub fn lan_car_left(&self, car_idx: usize) -> bool {
        self.lan_race
            .as_ref()
            .is_some_and(|l| l.slot_of(car_idx).is_some_and(|s| l.left.contains(&s)))
    }

    /// Car-to-car collisions are off in Ghost mode, and for the own car once it finished.
    pub fn lan_ghost_collisions(&self) -> bool {
        self.lan_race
            .as_ref()
            .is_some_and(|l| l.config.collision_mode == LanCollisionMode::GhostPassing || l.local_finished)
    }

    /// A remote car that no longer collides: its player left, or it already finished.
    pub fn lan_car_passive(&self, car_idx: usize) -> bool {
        self.lan_race.as_ref().is_some_and(|l| {
            l.slot_of(car_idx)
                .is_some_and(|s| l.left.contains(&s) || l.standings.iter().any(|f| f.slot_id == s))
        })
    }

    /// Controls for the own car in a LAN race.
    pub(super) fn lan_own_controls(&mut self, dt: f32, car_idx: usize) -> CarControls {
        let hold = self.lan_race.as_ref().is_some_and(|l| l.hold_input || l.local_finished);
        let scripted = self.lan_race.as_ref().and_then(|l| l.input_override);
        let ctrl = if hold {
            CarControls { brake: 1.0, ..CarControls::default() }
        } else if let Some(ctrl) = scripted {
            ctrl
        } else {
            let speed = self.world.vehicles.get(car_idx).map(|c| c.state.local_velocity.x).unwrap_or(0.0);
            let kb_ctrl = self.input.poll_player_controls(dt, speed);
            let touch_ctrl = self.touch.poll_controls();
            let mut ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);
            if speed <= 0.25 && ctrl.brake > 0.0 && ctrl.throttle == 0.0 {
                ctrl.reverse = true;
                ctrl.throttle = ctrl.brake;
                ctrl.brake = 0.0;
            }
            ctrl
        };
        if let Some(ref mut lan) = self.lan_race {
            lan.last_controls = ctrl;
        }
        ctrl
    }

    /// Countdown value for the HUD: time to the green light, or `LAN_WAITING_COUNTDOWN`.
    pub fn lan_countdown_remaining(&self) -> f32 {
        match self.lan_race_clock() {
            Some(clock) => (-clock) as f32,
            None => LAN_WAITING_COUNTDOWN,
        }
    }

    /// Pumps the host or client and applies its events to the race.
    pub fn pump_lan(&mut self, frame_dt: f32) {
        self.lan_frame_ran = true;
        if let Some(ref mut host) = self.lan_host {
            let events = host.update(frame_dt);
            self.lan_apply_host_events(events);
        } else if let Some(ref mut client) = self.lan_client {
            let events = client.update(frame_dt);
            self.lan_apply_client_events(events);
        }
    }

    fn lan_apply_host_events(&mut self, events: Vec<HostEvent>) {
        for event in events {
            match event {
                HostEvent::CarState(state) => self.lan_push_remote_state(state),
                HostEvent::PlayerLeft { slot_id, reason } => self.lan_player_left(slot_id, &reason),
                HostEvent::StandingsUpdated(finishers) => {
                    if let Some(ref mut lan) = self.lan_race {
                        lan.standings = finishers;
                    }
                }
                HostEvent::RaceOver(results) => {
                    if let Some(ref mut lan) = self.lan_race {
                        lan.results = Some(results);
                    }
                }
                _ => {}
            }
        }
    }

    fn lan_apply_client_events(&mut self, events: Vec<ClientEvent>) {
        for event in events {
            match event {
                ClientEvent::CarStates(states) => {
                    for state in states {
                        self.lan_push_remote_state(state);
                    }
                }
                ClientEvent::PlayerLeft { slot_id, reason } => self.lan_player_left(slot_id, &reason),
                ClientEvent::Standings(finishers) => {
                    if let Some(ref mut lan) = self.lan_race {
                        lan.standings = finishers;
                    }
                }
                ClientEvent::RaceOver(results) => {
                    if let Some(ref mut lan) = self.lan_race {
                        lan.results = Some(results);
                    }
                }
                ClientEvent::Disconnected(_) => self.lan_host_lost(),
                _ => {}
            }
        }
    }

    fn lan_push_remote_state(&mut self, state: NetCarState) {
        let my_slot = self.lan_player_slot;
        if let Some(ref mut lan) = self.lan_race {
            if state.slot != my_slot && lan.config.car_index_of(state.slot).is_some() && !lan.left.contains(&state.slot) {
                lan.remote.entry(state.slot).or_default().push(state);
            }
        }
    }

    fn lan_alert(&mut self, text: String, color: macroquad::color::Color) {
        let anchor = Vec2::new(screen_width_safe() * 0.5, screen_height_safe() * 0.30);
        self.floating_text.spawn_alert(text, anchor, color);
    }

    fn lan_player_left(&mut self, slot_id: u8, reason: &str) {
        let Some(ref mut lan) = self.lan_race else {
            return;
        };
        if lan.left.contains(&slot_id) {
            return;
        }
        lan.left.push(slot_id);
        lan.remote.remove(&slot_id);
        lan.remote_controls.remove(&slot_id);
        let car_idx = lan.config.car_index_of(slot_id);
        let name = car_idx.and_then(|i| lan.config.roster.get(i)).map(|e| e.player_name.clone()).unwrap_or_default();
        if let Some(car) = car_idx.and_then(|i| self.world.vehicles.get_mut(i)) {
            car.state.velocity = Vec2::ZERO;
            car.state.angular_velocity = 0.0;
            car.state.speed = 0.0;
            car.state.local_velocity = Vec2::ZERO;
        }
        self.lan_alert(format!("{} LEFT THE RACE ({})", name.to_uppercase(), reason), Palette::NEON_GOLD);
    }

    /// Client lost the host: keep the last standings and close the race locally.
    fn lan_host_lost(&mut self) {
        let Some(ref mut lan) = self.lan_race else {
            return;
        };
        if lan.host_left || lan.results.is_some() {
            return;
        }
        lan.host_left = true;
        lan.host_left_timer = HOST_LEFT_RESULTS_DELAY_SEC;
        self.lan_alert("HOST LEFT THE RACE".to_string(), Palette::RED);
    }

    /// Results built on this machine when the host is gone: host finishers first, then track order.
    fn lan_local_results(&self) -> Vec<RaceResult> {
        let Some(ref lan) = self.lan_race else {
            return Vec::new();
        };
        let mut results: Vec<RaceResult> = lan
            .standings
            .iter()
            .map(|f| RaceResult {
                slot_id: f.slot_id,
                position: 0,
                status: RaceStatus::Finished,
                finish_ms: Some(f.finish_ms),
                best_lap_ms: f.best_lap_ms,
            })
            .collect();
        for car_idx in self.compute_standings() {
            let Some(slot_id) = lan.slot_of(car_idx) else {
                continue;
            };
            if results.iter().any(|r| r.slot_id == slot_id) {
                continue;
            }
            let status = if lan.left.contains(&slot_id) { RaceStatus::Left } else { RaceStatus::Dnf };
            results.push(RaceResult { slot_id, position: 0, status, finish_ms: None, best_lap_ms: None });
        }
        results.sort_by_key(|r| (r.status == RaceStatus::Left) as u8);
        for (i, r) in results.iter_mut().enumerate() {
            r.position = i as u8 + 1;
        }
        results
    }

    /// Places every remote car at its interpolated pose for `render_time` (race clock seconds).
    pub fn lan_apply_remote_poses(&mut self, render_time: f64) {
        let my_slot = self.lan_player_slot;
        let Some(ref mut lan) = self.lan_race else {
            return;
        };
        for (i, entry) in lan.config.roster.iter().enumerate() {
            if entry.slot_id == my_slot || i >= self.world.vehicles.len() || lan.left.contains(&entry.slot_id) {
                continue;
            }
            let Some(s) = lan.remote.get_mut(&entry.slot_id).and_then(|b| b.sample(render_time)) else {
                continue;
            };
            let car = &mut self.world.vehicles[i];
            let velocity = Vec2::new(s.vel_x, s.vel_y);
            let fwd = Vec2::new(s.angle.cos(), s.angle.sin());
            let right = Vec2::new(-s.angle.sin(), s.angle.cos());
            car.state.position = Vec2::new(s.pos_x, s.pos_y);
            car.state.velocity = velocity;
            car.state.angle = s.angle;
            car.state.angular_velocity = s.angular_velocity;
            car.state.steer_angle = s.steer_angle;
            car.state.elevation = s.elevation;
            car.state.vertical_velocity = s.vertical_velocity;
            car.state.speed = velocity.length();
            car.state.local_velocity = Vec2::new(velocity.dot(fwd), velocity.dot(right));
            car.state.is_airborne = s.has_flag(flags::AIRBORNE);
            car.state.is_braking = s.has_flag(flags::BRAKING);
            car.state.is_drifting = s.has_flag(flags::DRIFTING);
            let ctrl = CarControls {
                throttle: (s.throttle as f32) / 255.0,
                brake: (s.brake as f32) / 255.0,
                steer: s.steer_angle,
                handbrake: s.has_flag(flags::HANDBRAKE),
                reverse: s.has_flag(flags::REVERSE),
            };
            lan.remote_controls.insert(entry.slot_id, ctrl);
            if let Some(lights) = self.car_lights_on.get_mut(i) {
                *lights = s.has_flag(flags::LIGHTS);
            }
            if let Some(tracker) = self.world.trackers.get_mut(i) {
                tracker.current_lap = s.lap as u32;
                tracker.next_checkpoint_idx = s.checkpoint as usize;
                tracker.normalized_progress = s.progress;
            }
        }
    }

    /// Sends the own car state (at most 60 Hz).
    fn lan_send_own_state(&mut self, clock: f64) {
        let my_idx = self.player_car_index();
        let lights = self.is_car_lights_on(my_idx);
        let (Some(lan), Some(car)) = (self.lan_race.as_mut(), self.world.vehicles.get(my_idx)) else {
            return;
        };
        if clock - lan.last_send_clock < STATE_SEND_INTERVAL_SEC - 1e-4 {
            return;
        }
        lan.last_send_clock = clock;
        let tracker = self.world.trackers.get(my_idx);
        let ctrl = lan.last_controls;
        let mut f = 0u8;
        for (on, bit) in [
            (car.state.is_braking, flags::BRAKING),
            (ctrl.handbrake, flags::HANDBRAKE),
            (car.state.is_drifting, flags::DRIFTING),
            (car.state.is_airborne, flags::AIRBORNE),
            (lights, flags::LIGHTS),
            (ctrl.reverse, flags::REVERSE),
        ] {
            if on {
                f |= bit;
            }
        }
        let state = NetCarState {
            slot: self.lan_player_slot,
            time_ms: (clock.max(0.0) * 1000.0) as u32,
            pos_x: car.state.position.x,
            pos_y: car.state.position.y,
            vel_x: car.state.velocity.x,
            vel_y: car.state.velocity.y,
            angle: car.state.angle,
            angular_velocity: car.state.angular_velocity,
            steer_angle: car.state.steer_angle,
            elevation: car.state.elevation,
            vertical_velocity: car.state.vertical_velocity,
            throttle: (ctrl.throttle.clamp(0.0, 1.0) * 255.0) as u8,
            brake: (ctrl.brake.clamp(0.0, 1.0) * 255.0) as u8,
            flags: f,
            lap: tracker.map(|t| t.current_lap.min(255) as u8).unwrap_or(0),
            checkpoint: tracker.map(|t| t.next_checkpoint_idx.min(u16::MAX as usize) as u16).unwrap_or(0),
            progress: tracker.map(|t| t.normalized_progress).unwrap_or(0.0),
        };
        if let Some(ref mut host) = self.lan_host {
            let _ = host.send_car_state(state);
        } else if let Some(ref mut client) = self.lan_client {
            let _ = client.send_car_state(state);
        }
    }

    /// Reports the own finish once, when the own tracker completes the last lap.
    pub(super) fn lan_check_own_finish(&mut self) {
        let my_idx = self.player_car_index();
        let done = self.world.trackers.get(my_idx).is_some_and(|t| t.current_lap > self.total_laps);
        let clock = self.lan_race_clock();
        let Some(ref mut lan) = self.lan_race else {
            return;
        };
        if !done || lan.local_finished {
            return;
        }
        lan.local_finished = true;
        let finish_ms = (clock.unwrap_or(0.0).max(0.0) * 1000.0) as u32;
        let best_lap_ms = self.world.trackers.get(my_idx).and_then(|t| t.best_lap_time).map(|s| (s * 1000.0) as u32);
        if let Some(ref mut host) = self.lan_host {
            let events = host.report_finish(finish_ms, best_lap_ms);
            self.lan_apply_host_events(events);
        } else if let Some(ref mut client) = self.lan_client {
            let _ = client.report_finish(finish_ms, best_lap_ms);
        }
        self.audio.play_sfx(SfxType::RaceFinish);
        self.lan_alert("FINISHED! WAITING FOR THE OTHER RACERS".to_string(), Palette::NEON_GREEN);
    }

    /// One frame of the LAN race: network, remote cars, own-car physics, own state, own finish.
    ///
    /// `hold` brakes the own car (pause menu or another screen is open).
    pub fn lan_race_frame(&mut self, frame_dt: f32, hold: bool) {
        self.pump_lan(frame_dt);
        let Some(clock) = self.lan_race_clock() else {
            return;
        };
        if clock < 0.0 || self.lan_race.as_ref().is_some_and(|l| l.results.is_some() || l.host_left) {
            return;
        }
        if let Some(ref mut lan) = self.lan_race {
            lan.hold_input = hold;
            if !hold && crate::storage::is_dev_mode() && is_key_pressed(KeyCode::F9) {
                lan.show_net_hud = !lan.show_net_hud;
            }
        }
        self.lan_apply_remote_poses(clock - INTERP_DELAY_SEC);

        self.session_time += frame_dt;
        self.accumulator += frame_dt;
        let max_substeps = 8;
        let mut substeps = 0;
        while self.accumulator >= Self::FIXED_DT && substeps < max_substeps {
            self.physics_step(Self::FIXED_DT);
            self.accumulator -= Self::FIXED_DT;
            substeps += 1;
        }

        self.lan_send_own_state(clock);
        self.lan_check_own_finish();
    }

    /// End-of-frame hook: keeps the LAN race running while the player is in a menu,
    /// and closes the race when the results are known.
    pub fn lan_after_frame(&mut self) {
        if self.lan_race.is_none() {
            return;
        }
        let frame_dt = get_frame_time_safe().min(0.1);
        if !self.lan_frame_ran {
            if self.state == GameState::Finished {
                self.pump_lan(frame_dt);
            } else {
                self.lan_race_frame(frame_dt, true);
            }
        }
        if self.state == GameState::Finished {
            return;
        }

        // Host gone: show the message, then close the race with the last standings.
        let mut close_locally = false;
        if let Some(ref mut lan) = self.lan_race {
            if lan.host_left && lan.results.is_none() {
                lan.host_left_timer -= frame_dt;
                close_locally = lan.host_left_timer <= 0.0;
            }
        }
        if close_locally {
            let results = self.lan_local_results();
            if let Some(ref mut lan) = self.lan_race {
                lan.results = Some(results);
            }
        }

        if self.lan_race.as_ref().is_some_and(|l| l.results.is_some()) {
            self.check_race_finish();
        }
    }

    /// Car indices in the host's result order, once the results are known.
    pub(super) fn lan_result_order(&self) -> Option<Vec<usize>> {
        let lan = self.lan_race.as_ref()?;
        let results = lan.results.as_ref()?;
        let mut order: Vec<usize> = results.iter().filter_map(|r| lan.config.car_index_of(r.slot_id)).collect();
        for i in 0..self.world.vehicles.len() {
            if !order.contains(&i) {
                order.push(i);
            }
        }
        Some(order)
    }

    /// Host result row of car `car_idx`.
    pub(super) fn lan_result_of(&self, car_idx: usize) -> Option<RaceResult> {
        let lan = self.lan_race.as_ref()?;
        let slot = lan.slot_of(car_idx)?;
        lan.results.as_ref()?.iter().find(|r| r.slot_id == slot).copied()
    }

    /// Draws the dev net HUD (F9): RTT, clock offset, rates, errors, and each remote buffer.
    pub(super) fn render_lan_net_hud(&self) {
        let Some(ref lan) = self.lan_race else {
            return;
        };
        if !lan.show_net_hud {
            return;
        }
        let (role, stats) = if let Some(ref h) = self.lan_host {
            ("HOST".to_string(), h.stats())
        } else if let Some(ref c) = self.lan_client {
            (format!("CLIENT slot {}", self.lan_player_slot), c.stats())
        } else {
            return;
        };
        let mut lines = vec![
            format!("LAN {role}  clock {:.2}s", self.lan_race_clock().unwrap_or(f64::NAN)),
            format!(
                "rtt {} ms  offset {}",
                stats.rtt_ms.map(|r| r.to_string()).unwrap_or_else(|| "-".into()),
                stats.clock_offset_ms.map(|o| format!("{o:.1} ms")).unwrap_or_else(|| "-".into())
            ),
            format!("in {:.0}/s  out {:.0}/s  reliable pending {}", stats.in_per_sec, stats.out_per_sec, stats.reliable_pending),
            format!("decode err {}  encode err {}  stale {}", stats.decode_errors, stats.encode_errors, stats.stale_dropped),
        ];
        for entry in &lan.config.roster {
            if entry.slot_id == self.lan_player_slot {
                continue;
            }
            let line = match lan.remote.get(&entry.slot_id) {
                _ if lan.left.contains(&entry.slot_id) => format!("slot {}: left", entry.slot_id),
                Some(b) => format!(
                    "slot {}: buf {}  extrap {:.0} ms  dropped {}",
                    entry.slot_id,
                    b.len(),
                    b.extrapolation_sec() * 1000.0,
                    b.dropped()
                ),
                None => format!("slot {}: no data", entry.slot_id),
            };
            lines.push(line);
        }
        let (x, y, line_h) = (12.0, 120.0, 16.0);
        draw_rectangle(x - 6.0, y - 14.0, 360.0, line_h * lines.len() as f32 + 10.0, macroquad::color::Color::new(0.0, 0.0, 0.0, 0.65));
        for (i, line) in lines.iter().enumerate() {
            self.fonts.draw_ui_bold(line, x, y + i as f32 * line_h, 13.0, Palette::NEON_CYAN);
        }
    }
}
