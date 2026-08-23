use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{is_key_pressed, KeyCode};
use macroquad::prelude::{get_frame_time, screen_height, screen_width};
use tdrace_core::collision::car_collision::resolve_multi_car_collisions;
use tdrace_core::collision::wall::resolve_all_wall_collisions;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::AssistProfile;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::SpawnPose;
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};
use tdrace_core::track::Track;
use tdrace_core::CarConfig;

use crate::ai::{BotAiDriver, BotProfile};
use crate::camera::RaceCamera;
use crate::fx::EffectsManager;
use crate::input::touch::TouchController;
use crate::input::InputController;
use crate::render::color::CarColorScheme;
use crate::render::ghost::{render_ghost_car, GhostRecorder};
use crate::render::{render_barriers_and_obstacles, render_car, render_track};
use crate::replay::{ReplayPlayer, ReplayRecorder};
use crate::ui::hud::render_hud;
use crate::ui::menu::{
    render_pause_menu, render_results_screen, render_track_select_menu, CarChoice,
    RaceResultEntry, TrackChoice,
};

/// High-level game flow state machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Countdown(f32),
    Racing,
    Paused,
    Finished,
}

/// Main Racing Session and Simulation Coordinator.
pub struct RaceSession {
    pub state: GameState,
    pub track: Track,
    pub track_choice: TrackChoice,
    pub car_choice: CarChoice,
    pub is_time_attack: bool,
    pub num_bots: usize,
    pub total_laps: u32,

    pub cars: Vec<Car>,
    pub color_schemes: Vec<CarColorScheme>,
    pub trackers: Vec<TrackProgressTracker>,
    pub ai_drivers: Vec<BotAiDriver>,

    pub fx: EffectsManager,
    pub camera: RaceCamera,
    pub input: InputController,
    pub touch: TouchController,

    // Ghost vehicle recording and playback (Time Attack)
    pub ghost_recorder: GhostRecorder,

    // Replay recording and playback
    pub replay_recorder: Option<ReplayRecorder>,
    pub replay_player: Option<ReplayPlayer>,
    pub is_replay_mode: bool,

    pub session_time: f32,
    pub accumulator: f32,
    pub results: Vec<RaceResultEntry>,

    // Menu selection cursor state
    pub menu_track_idx: usize,
    pub menu_car_idx: usize,
    pub assist_profile: AssistProfile,

    // Internal trackers
    prev_player_lap: u32,
}

impl Default for RaceSession {
    fn default() -> Self {
        Self::new()
    }
}

impl RaceSession {
    pub const FIXED_DT: f32 = 1.0 / 120.0;

    pub fn new() -> Self {
        let track = classic_grand_prix();
        let mut session = Self {
            state: GameState::Menu,
            track,
            track_choice: TrackChoice::ClassicGrandPrix,
            car_choice: CarChoice::SportsCar,
            assist_profile: AssistProfile::Arcade,
            is_time_attack: false,
            num_bots: 3,
            total_laps: 3,

            cars: Vec::new(),
            color_schemes: Vec::new(),
            trackers: Vec::new(),
            ai_drivers: Vec::new(),

            fx: EffectsManager::new(8000, 1500),
            camera: RaceCamera::new(),
            input: InputController::new(),
            touch: TouchController::new(),

            ghost_recorder: GhostRecorder::new(),
            replay_recorder: None,
            replay_player: None,
            is_replay_mode: false,

            session_time: 0.0,
            accumulator: 0.0,
            results: Vec::new(),

            menu_track_idx: 0,
            menu_car_idx: 0,
            prev_player_lap: 1,
        };

        session.init_race();
        session.state = GameState::Menu; // Start in main menu
        session
    }

    /// Initializes or resets the racing circuit, cars, grid spawns, AI drivers, and camera.
    pub fn init_race(&mut self) {
        // 1. Build selected track
        self.track = match self.track_choice {
            TrackChoice::ClassicGrandPrix => classic_grand_prix(),
            TrackChoice::OvalSpeedway => oval_speedway(),
            TrackChoice::DriftPark => drift_park(),
            TrackChoice::KartArena => kart_arena(),
        };

        // 2. Setup camera
        self.camera.setup_for_track(&self.track);

        // 3. Player car configuration
        let mut base_config = match self.car_choice {
            CarChoice::SportsCar => CarConfig::sports_car(),
            CarChoice::DriftCar => CarConfig::drift_car(),
            CarChoice::Kart => CarConfig::kart(),
            CarChoice::RallyCar => CarConfig::rally_car(),
        };
        base_config.assists = self.assist_profile.to_config();

        // 4. Determine total participant count
        let total_cars = if self.is_time_attack {
            1
        } else {
            (1 + self.num_bots).min(self.track.grid_positions.len()).min(8)
        };

        self.cars.clear();
        self.color_schemes.clear();
        self.trackers.clear();
        self.ai_drivers.clear();
        self.fx.clear();
        self.results.clear();
        self.session_time = 0.0;
        self.accumulator = 0.0;
        self.prev_player_lap = 1;

        let num_cps = self.track.checkpoints.len();
        let num_sectors = 3;

        let bot_profiles = [
            BotProfile::pro(),
            BotProfile::aggressive(),
            BotProfile::balanced(),
            BotProfile::rookie(),
            BotProfile::pro(),
            BotProfile::aggressive(),
            BotProfile::balanced(),
        ];

        for i in 0..total_cars {
            let grid_pose = self
                .track
                .grid_positions
                .get(i)
                .copied()
                .unwrap_or(SpawnPose {
                    position: Vec2::ZERO,
                    angle: 0.0,
                    grid_slot: i,
                });
            let car = Car::new(base_config).with_pose(grid_pose.position, grid_pose.angle);

            self.cars.push(car);
            self.color_schemes.push(CarColorScheme::from_index(i));
            self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));

            if i > 0 {
                let prof = bot_profiles[(i - 1) % bot_profiles.len()];
                self.ai_drivers.push(BotAiDriver::new(prof));
            }
        }

        // Reset ghost active lap samples
        self.ghost_recorder.on_lap_invalidated();

        // Reset input filter smoothing state
        self.input.reset();

        // Start new replay recording
        self.replay_recorder = Some(ReplayRecorder::new(
            self.track_choice,
            self.car_choice,
            42,
            Self::FIXED_DT,
        ));

        // Start with countdown
        self.state = GameState::Countdown(3.5);
    }

    /// Master update tick called once per frame.
    pub fn update(&mut self) {
        let frame_dt = get_frame_time().min(0.1);
        let sw = screen_width();
        let sh = screen_height();

        // Handle debug toggles
        self.input.update_debug_toggles();

        // Handle touch controls update
        self.touch.update_from_macroquad(sw, sh, frame_dt);

        // Toggle touch overlay on desktop (F6 or Z key)
        if is_key_pressed(KeyCode::F6) {
            self.touch.enabled = !self.touch.enabled;
        }

        // Toggle touch layout (L key)
        if is_key_pressed(KeyCode::L) {
            self.touch.toggle_layout();
        }

        // Handle camera toggle (Tab or C key)
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::C) {
            self.camera.toggle_mode();
        }

        // Cycle Driver Assists Profile (H key)
        if is_key_pressed(KeyCode::H) {
            self.assist_profile = self.assist_profile.next();
            if let Some(player_car) = self.cars.first_mut() {
                player_car.config.assists = self.assist_profile.to_config();
                self.fx.drift_popups.spawn_text(
                    player_car.state.position,
                    &format!("ASSISTS: {}", self.assist_profile.short_name()),
                    Color::new(0.3, 0.9, 1.0, 1.0),
                );
            }
        }

        // Global restart shortcut (R key)
        if is_key_pressed(KeyCode::R) {
            self.init_race();
            return;
        }

        match self.state {
            GameState::Menu => {
                self.update_menu();
            }
            GameState::Countdown(ref mut remaining) => {
                *remaining -= frame_dt;
                // Camera follows player during countdown
                if let Some(player_car) = self.cars.first() {
                    self.camera.update(player_car, frame_dt);
                }

                if *remaining <= 0.0 {
                    self.state = GameState::Racing;
                }
            }
            GameState::Racing => {
                // Pause trigger (Escape / Pause key)
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Pause) {
                    self.state = GameState::Paused;
                    return;
                }

                self.session_time += frame_dt;
                self.accumulator += frame_dt;

                // Fixed physics substepping
                let max_substeps = 8;
                let mut substeps = 0;

                while self.accumulator >= Self::FIXED_DT && substeps < max_substeps {
                    self.physics_step(Self::FIXED_DT);
                    self.accumulator -= Self::FIXED_DT;
                    substeps += 1;
                }

                // Update Camera
                if let Some(player_car) = self.cars.first() {
                    self.camera.update(player_car, frame_dt);
                }

                // Check for race finish conditions
                self.check_race_finish();
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Pause) {
                    self.state = GameState::Racing;
                }
                if is_key_pressed(KeyCode::M) {
                    self.state = GameState::Menu;
                }
            }
            GameState::Finished => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    self.init_race();
                }
                if is_key_pressed(KeyCode::M) {
                    self.state = GameState::Menu;
                }
            }
        }
    }

    /// Menu input navigation.
    fn update_menu(&mut self) {
        // Track selection cursor (Up/Down)
        if is_key_pressed(KeyCode::Up) {
            if self.menu_track_idx == 0 {
                self.menu_track_idx = TrackChoice::ALL.len() - 1;
            } else {
                self.menu_track_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down) {
            self.menu_track_idx = (self.menu_track_idx + 1) % TrackChoice::ALL.len();
        }

        // Car selection cursor (Left/Right)
        if is_key_pressed(KeyCode::Left) {
            if self.menu_car_idx == 0 {
                self.menu_car_idx = CarChoice::ALL.len() - 1;
            } else {
                self.menu_car_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Right) {
            self.menu_car_idx = (self.menu_car_idx + 1) % CarChoice::ALL.len();
        }

        // Toggle Mode (Time Attack vs Race vs AI)
        if is_key_pressed(KeyCode::T) {
            self.is_time_attack = !self.is_time_attack;
        }

        // Change bot count
        if is_key_pressed(KeyCode::B) {
            self.num_bots = (self.num_bots % 7) + 1;
        }

        // Toggle Driver Assists Profile
        if is_key_pressed(KeyCode::H) {
            self.assist_profile = self.assist_profile.next();
        }

        // Start race
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            self.track_choice = TrackChoice::ALL[self.menu_track_idx];
            self.car_choice = CarChoice::ALL[self.menu_car_idx];
            self.init_race();
        }
    }

    /// High-performance deterministic fixed physics simulation step.
    fn physics_step(&mut self, dt: f32) {
        let n_cars = self.cars.len();
        if n_cars == 0 {
            return;
        }

        // 1. Gather driver controls (Player keyboard with smoothing + Touch combined, and AI bots)
        let mut controls_all = Vec::with_capacity(n_cars);
        let player_speed = self.cars.first().map(|c| c.state.local_velocity.x).unwrap_or(0.0);
        let kb_ctrl = self.input.poll_player_controls(dt, player_speed);
        let touch_ctrl = self.touch.poll_controls();
        let player_ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);
        controls_all.push(player_ctrl);

        for i in 1..n_cars {
            let ai_idx = i - 1;
            let other_cars_refs: Vec<&Car> = self
                .cars
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != i)
                .map(|(_, c)| c)
                .collect();

            let bot_ctrl = self.ai_drivers[ai_idx].compute_controls(
                &self.cars[i],
                &self.track,
                &other_cars_refs,
                dt,
            );
            controls_all.push(bot_ctrl);
        }

        // 2. Sample surfaces under all wheels of all cars
        let mut wheel_surfaces = Vec::with_capacity(n_cars);
        for car in &self.cars {
            wheel_surfaces.push(self.track.sample_car_surfaces(car));
        }

        // 3. Step individual vehicle dynamics
        for i in 0..n_cars {
            self.cars[i].step_per_wheel(&controls_all[i], wheel_surfaces[i], dt);
        }

        // 4. Resolve Car-to-Car collisions with momentum exchange and penetration pushback
        let car_collision_events = if n_cars > 1 {
            resolve_multi_car_collisions(&mut self.cars, 0.45, 0.35, 3)
        } else {
            Vec::new()
        };

        // 5. Resolve Wall and Obstacle boundary collisions for each car
        let mut wall_collision_events = Vec::new();
        for car in &mut self.cars {
            let mut wall_events = resolve_all_wall_collisions(
                car,
                &self.track.geometry.inner_walls,
                &self.track.geometry.obstacles,
            );
            let outer_events =
                resolve_all_wall_collisions(car, &self.track.geometry.outer_walls, &[]);
            wall_events.extend(outer_events);
            wall_collision_events.extend(wall_events);
        }

        // Camera screen shake on player impact
        for wev in &wall_collision_events {
            if wev.impact_speed > 3.0 {
                self.camera.add_trauma(wev.impact_speed * 0.08);
            }
        }
        for cev in &car_collision_events {
            if (cev.car_a_idx == 0 || cev.car_b_idx == 0) && cev.closing_speed > 3.0 {
                self.camera.add_trauma(cev.closing_speed * 0.06);
            }
        }

        // 6. Update race progression, lap tracking, sector splits, anti-cheat
        for i in 0..n_cars {
            self.trackers[i].update(
                &self.cars[i],
                &self.track.spline,
                &self.track.checkpoints,
                dt,
            );
        }

        // 7. Ghost lap telemetry recording (Time Attack)
        if self.is_time_attack {
            if let (Some(player_car), Some(tracker)) = (self.cars.first(), self.trackers.first()) {
                self.ghost_recorder.record_frame(tracker.lap_time, player_car, dt);

                if tracker.current_lap > self.prev_player_lap {
                    if let Some(last_lap_time) = tracker.last_lap_time {
                        self.ghost_recorder.on_lap_completed(
                            last_lap_time,
                            self.track_choice,
                            self.car_choice,
                        );
                    }
                    self.prev_player_lap = tracker.current_lap;
                }
            }
        }

        // 8. Replay frame recording
        if let Some(rec) = &mut self.replay_recorder {
            if let (Some(player_car), Some(tracker)) = (self.cars.first(), self.trackers.first()) {
                rec.record_frame(player_ctrl, player_car, tracker);
            }
        }

        // 9. Update visual effects (skidmarks, tire smoke, roost, collision sparks, drift popups)
        self.fx.update(
            &self.cars,
            &wheel_surfaces,
            &wall_collision_events,
            &car_collision_events,
            dt,
        );
    }

    /// Evaluates current positions and checks for checkered flag completion.
    fn check_race_finish(&mut self) {
        let player_done = self
            .trackers
            .first()
            .is_some_and(|t| t.current_lap > self.total_laps);

        if player_done && !self.is_time_attack {
            self.build_results();
            self.state = GameState::Finished;
        }
    }

    /// Computes real-time race standings.
    pub fn compute_standings(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.cars.len()).collect();
        indices.sort_by(|&a, &b| {
            let tr_a = &self.trackers[a];
            let tr_b = &self.trackers[b];

            // Primary: Lap number descending
            if tr_a.current_lap != tr_b.current_lap {
                return tr_b.current_lap.cmp(&tr_a.current_lap);
            }
            // Secondary: Normalized track progress descending
            tr_b.normalized_progress
                .partial_cmp(&tr_a.normalized_progress)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    /// Builds the final results standings table.
    fn build_results(&mut self) {
        let standings = self.compute_standings();
        self.results.clear();

        for (rank, &car_idx) in standings.iter().enumerate() {
            let is_player = car_idx == 0;
            let car_name = if is_player {
                "Player 1 (You)".to_string()
            } else {
                format!("AI Driver {}", car_idx)
            };

            let tracker = &self.trackers[car_idx];
            let best_lap = tracker.best_lap_time;
            let total_time = self.session_time + (rank as f32 * 0.65); // Simulated finish gap
            let delta = if rank == 0 { 0.0 } else { rank as f32 * 0.65 };

            self.results.push(RaceResultEntry {
                position: rank + 1,
                car_name,
                is_player,
                total_time,
                best_lap,
                delta_to_leader: delta,
            });
        }
    }

    /// Master render pipeline:
    /// 1. World pass (Track, Skidmarks, Walls, Ghost Car, Cars, Sparks/Smoke, Debug)
    /// 2. Screen pass (HUD, Mini-map, Touch Controls, Popups, Menus)
    pub fn render(&self) {
        match self.state {
            GameState::Menu => {
                render_track_select_menu(
                    self.menu_track_idx,
                    self.menu_car_idx,
                    self.num_bots,
                    self.is_time_attack,
                    self.assist_profile,
                );
            }
            GameState::Countdown(remaining) => {
                self.render_world();
                self.render_screen(Some(remaining));
            }
            GameState::Racing => {
                self.render_world();
                self.render_screen(None);
            }
            GameState::Paused => {
                self.render_world();
                self.render_screen(None);
                render_pause_menu(self.assist_profile);
            }
            GameState::Finished => {
                self.render_world();
                render_results_screen(&self.track.name, &self.results, self.is_time_attack);
            }
        }
    }

    /// Renders world-space entities under active camera.
    fn render_world(&self) {
        self.camera.apply();

        // 1. Ground & Track
        render_track(&self.track);

        // 2. Persistent Skidmarks
        self.fx.render_ground_fx();

        // 3. Walls, Barriers, and Obstacles with 2.5D Drop Shadows
        render_barriers_and_obstacles(&self.track);

        // 4. Ghost Vehicle (Semi-transparent during Time Attack)
        if self.is_time_attack {
            if let Some(best_ghost) = &self.ghost_recorder.best_ghost_lap {
                if let Some(player_tracker) = self.trackers.first() {
                    if let Some(ghost_frame) = best_ghost.sample_at_time(player_tracker.lap_time) {
                        if let Some(player_car) = self.cars.first() {
                            render_ghost_car(&ghost_frame, &player_car.config, 0.60);
                        }
                    }
                }
            }
        }

        // 5. Vehicles
        for (i, car) in self.cars.iter().enumerate() {
            let scheme = &self.color_schemes[i];
            let is_braking =
                car.state.local_velocity.x > 1.0 && car.state.wheels[2].slip_ratio < -0.15;
            render_car(car, scheme, is_braking);
        }

        // 6. Airborne Particles (Smoke, Dirt roost, Sparks, Drift text)
        self.fx.render_airborne_fx();

        // 7. Debug Overlays (F1: LIDAR, F2: Checkpoints, F3: OBBs, F4: AI Lines)
        if let Some(player_car) = self.cars.first() {
            let player_tracker = &self.trackers[0];
            self.input.render_world_debug(
                player_car,
                &self.cars,
                &self.track,
                player_tracker,
                &self.ai_drivers,
            );
        }

        self.camera.reset_to_screen();
    }

    /// Renders screen-space HUD, UI overlays, and Mobile Touch Controls.
    fn render_screen(&self, countdown: Option<f32>) {
        let sw = screen_width();
        let sh = screen_height();

        if let Some(player_car) = self.cars.first() {
            let player_tracker = &self.trackers[0];
            let standings = self.compute_standings();
            let player_pos = standings.iter().position(|&idx| idx == 0).unwrap_or(0) + 1;

            render_hud(
                player_car,
                &self.cars,
                &self.color_schemes,
                &self.track,
                player_tracker,
                &self.trackers,
                player_pos,
                self.cars.len(),
                self.total_laps,
                self.is_time_attack,
                countdown,
            );

            // F5: Telemetry Panel
            self.input.render_screen_debug(player_car);

            // Mobile Touch Controls Overlay (Virtual Joystick / Buttons + Pedals)
            self.touch.render(sw, sh);
        }
    }
}
