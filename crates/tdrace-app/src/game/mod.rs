use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::KeyCode;
use macroquad::prelude::{get_frame_time, screen_height, screen_width};

#[inline]
fn is_key_pressed(k: KeyCode) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_key_pressed(k)).unwrap_or(false)
}

#[inline]
fn is_key_down(k: KeyCode) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_key_down(k)).unwrap_or(false)
}

#[inline]
fn is_mouse_button_pressed(btn: macroquad::input::MouseButton) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_mouse_button_pressed(btn)).unwrap_or(false)
}

#[inline]
fn is_mouse_button_down(btn: macroquad::input::MouseButton) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_mouse_button_down(btn)).unwrap_or(false)
}

#[inline]
fn is_mouse_button_released(btn: macroquad::input::MouseButton) -> bool {
    std::panic::catch_unwind(|| macroquad::input::is_mouse_button_released(btn)).unwrap_or(false)
}

#[inline]
fn mouse_position_safe() -> (f32, f32) {
    std::panic::catch_unwind(macroquad::input::mouse_position).unwrap_or((0.0, 0.0))
}

#[inline]
fn mouse_wheel_safe() -> (f32, f32) {
    std::panic::catch_unwind(macroquad::input::mouse_wheel).unwrap_or((0.0, 0.0))
}

#[inline]
fn get_char_pressed() -> Option<char> {
    std::panic::catch_unwind(macroquad::input::get_char_pressed).unwrap_or(None)
}

#[inline]
fn get_frame_time_safe() -> f32 {
    std::panic::catch_unwind(get_frame_time).unwrap_or(1.0 / 60.0)
}

#[inline]
fn screen_width_safe() -> f32 {
    std::panic::catch_unwind(screen_width).unwrap_or(1280.0)
}

#[inline]
fn screen_height_safe() -> f32 {
    std::panic::catch_unwind(screen_height).unwrap_or(720.0)
}
use tdrace_core::collision::car_collision::resolve_multi_car_collisions;
use tdrace_core::collision::wall::resolve_all_wall_collisions;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::AssistProfile;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::SpawnPose;
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::Track;

use crate::ai::{BotAiDriver, DriverCharacter};
use crate::audio::{AudioManager, MusicTrack, SfxType};
use crate::camera::RaceCamera;
use crate::config::GameConfig;
use crate::db::{HallOfFameDb, HallOfFameEntry};
use crate::fx::EffectsManager;
use crate::input::touch::TouchController;
use crate::input::InputController;
use crate::profile::{CountryRegistry, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::color::{CarColorScheme, Palette};
use crate::editor::{
    render_editor_grid, render_editor_gizmos, render_editor_ui, EditorAction, EditorCamera,
    EditorModal, EditorState, EditorToolType, ToolSettings,
};
use crate::render::ghost::{render_ghost_car, GhostRecorder};
use crate::render::{render_barriers_and_obstacles, render_car, render_track};
use crate::replay::{ReplayPlayer, ReplayRecorder};
use crate::track_manager::TrackManager;
use crate::ui::driver_card::render_driver_cards_screen;
use crate::ui::font::Fonts;
use crate::ui::hall_of_fame::{render_hall_of_fame_screen, PlayerCongrats};
use crate::ui::hud::render_hud;
use crate::ui::menu::{
    render_controls_screen, render_pause_menu, render_results_screen, render_track_select_menu,
    CarChoice, RaceResultEntry, TrackChoice,
};
use crate::ui::profile_ui::{render_profile_create_screen, render_profile_manager_screen};
use crate::ui::starting_grid::render_starting_grid_screen;
use crate::ui::UiScaler;

/// Source screen that launched the DriverCards dossier view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriverCardsOrigin {
    Menu,
    StartingGrid,
    Paused,
}

/// High-level game flow state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Menu,
    StartingGrid,
    Countdown(f32),
    Racing,
    Paused,
    Finished,
    ControlsHelp(bool),
    DriverCards(DriverCardsOrigin),
    ProfileManager {
        selected_idx: usize,
    },
    ProfileCreate {
        editing_id: Option<i64>,
        field_idx: usize,
        input_name: String,
        input_alias: String,
        country_idx: usize,
        livery_idx: usize,
        cursor_timer: f32,
    },
    TrackEditor,
    EditorTestDrive,
}


/// Root controller orchestrating track geometry, cars, physics, UI, and audio.
pub struct RaceSession {
    pub state: GameState,
    pub track: Track,
    pub track_choice: TrackChoice,
    pub track_manager: TrackManager,
    pub car_choice: CarChoice,
    pub free_car_selection: bool,
    pub is_time_attack: bool,
    pub num_bots: usize,
    pub total_laps: u32,
    pub config: GameConfig,

    pub cars: Vec<Car>,
    pub color_schemes: Vec<CarColorScheme>,
    pub trackers: Vec<TrackProgressTracker>,
    pub ai_drivers: Vec<BotAiDriver>,
    pub opponent_drivers: Vec<DriverCharacter>,
    pub driver_cards_idx: usize,

    // Active Player Profile & Career History
    pub active_profile: PlayerProfile,
    pub active_profile_stats: ProfileCareerStats,
    pub profile_list: Vec<PlayerProfile>,
    pub profile_history: Vec<RaceHistoryEntry>,

    pub fx: EffectsManager,
    pub camera: RaceCamera,
    pub input: InputController,
    pub touch: TouchController,
    pub fonts: Fonts,

    // Ghost vehicle recording and playback (Time Attack)
    pub ghost_recorder: GhostRecorder,

    // Replay recording and playback
    pub replay_recorder: Option<ReplayRecorder>,
    pub replay_player: Option<ReplayPlayer>,
    pub is_replay_mode: bool,

    pub session_time: f32,
    pub accumulator: f32,
    pub results: Vec<RaceResultEntry>,

    // Hall of Fame
    pub hof_db: Option<HallOfFameDb>,
    pub hof_entries: Vec<HallOfFameEntry>,
    pub recent_hof_id: Option<i64>,
    pub recent_congrats: Option<PlayerCongrats>,
    pub show_hall_of_fame: bool,

    // Menu selection cursor state
    pub menu_track_idx: usize,
    pub menu_car_idx: usize,
    pub assist_profile: AssistProfile,

    // Track Editor & Test Drive state
    pub editor_state: Option<EditorState>,
    pub editor_camera: EditorCamera,
    pub editor_tools: ToolSettings,
    pub editor_modal: EditorModal,
    pub test_drive_car: Option<Car>,
    pub test_drive_tracker: Option<TrackProgressTracker>,
    pub test_drive_time: f32,

    // Audio System
    pub audio: AudioManager,
    pub engine_rpm: EngineRpmModel,
    prev_countdown_sec: i32,
    prev_player_sector: usize,
    curb_sound_cooldown: f32,
    offroad_sound_cooldown: f32,

    // Internal trackers
    pub prev_player_lap: u32,
}



/// Dynamic vehicle transmission gear and engine RPM simulation model.
#[derive(Debug, Clone)]
pub struct EngineRpmModel {
    pub current_rpm: f32,
    pub current_gear: usize,
    pub shift_cooldown: f32,
}

impl Default for EngineRpmModel {
    fn default() -> Self {
        Self {
            current_rpm: 1100.0,
            current_gear: 1,
            shift_cooldown: 0.0,
        }
    }
}

impl EngineRpmModel {
    pub fn update(&mut self, forward_speed: f32, throttle: f32, max_slip: f32, dt: f32) -> (f32, bool) {
        self.shift_cooldown = (self.shift_cooldown - dt).max(0.0);
        let speed_abs = forward_speed.abs();
        let is_reverse = forward_speed < -0.5 && throttle < 0.0;

        let (new_gear, target_rpm) = if is_reverse {
            let rpm = (1100.0 + (speed_abs / 12.0) * 5500.0).clamp(1100.0, 7200.0);
            (0, rpm)
        } else if speed_abs < 1.0 {
            // Stationary launch revs / idle
            let throttle_revs = if throttle > 0.05 {
                1100.0 + throttle * 5500.0
            } else {
                1100.0
            };
            (1, throttle_revs)
        } else {
            // 5-speed forward sequential transmission
            let (gear, base_rpm) = if speed_abs < 12.5 {
                (1, 1200.0 + (speed_abs / 12.5) * 5800.0)
            } else if speed_abs < 23.5 {
                (2, 3800.0 + ((speed_abs - 12.5) / 11.0) * 3400.0)
            } else if speed_abs < 35.5 {
                (3, 4200.0 + ((speed_abs - 23.5) / 12.0) * 3000.0)
            } else if speed_abs < 47.5 {
                (4, 4600.0 + ((speed_abs - 35.5) / 12.0) * 2600.0)
            } else {
                (5, 5000.0 + ((speed_abs - 47.5) / 16.0) * 2500.0)
            };

            // Wheelspin rev-flare (power drift / burnout)
            let slip_flare = if max_slip > 0.3 { (max_slip - 0.3) * 2500.0 } else { 0.0 };
            (gear, (base_rpm + slip_flare).clamp(1100.0, 7800.0))
        };

        let is_upshift = new_gear > self.current_gear && self.current_gear > 0 && self.shift_cooldown <= 0.0;
        if is_upshift {
            self.shift_cooldown = 0.22;
        }
        self.current_gear = new_gear;

        // Smooth RPM interpolation with realistic engine inertia
        let responsiveness = if target_rpm > self.current_rpm { 16.0 } else { 10.0 };
        self.current_rpm += (target_rpm - self.current_rpm) * (dt * responsiveness).min(1.0);

        (self.current_rpm, is_upshift)
    }
}

impl Default for RaceSession {
    fn default() -> Self {
        Self::new()
    }
}

impl RaceSession {
    pub const FIXED_DT: f32 = 1.0 / 120.0;

    pub fn new() -> Self {
        Self::new_with_config(GameConfig::load_or_default())
    }

    pub fn new_with_config(config: GameConfig) -> Self {
        let track_choice = match config.gameplay.default_track.as_str() {
            "oval_speedway" => TrackChoice::OvalSpeedway,
            "drift_park" => TrackChoice::DriftPark,
            "kart_arena" => TrackChoice::KartArena,
            "ramp_raceway" => TrackChoice::RampRaceway,
            "oasis_rally" | "oasis" | "dune_raid" | "sahara_dunes" => TrackChoice::OasisRally,
            "outlaw_pass" => TrackChoice::OutlawPass,
            _ => TrackChoice::ClassicGrandPrix,
        };
        let car_choice = match config.gameplay.default_car.as_str() {
            "drift_car" => CarChoice::DriftCar,
            "kart" => CarChoice::Kart,
            "rally_car" => CarChoice::RallyCar,
            _ => CarChoice::SportsCar,
        };
        let assist_profile = match config.gameplay.default_assist_profile.to_lowercase().as_str() {
            "sport" => AssistProfile::Sport,
            "pro" => AssistProfile::Pro,
            _ => AssistProfile::Arcade,
        };

        let track_manager = TrackManager::default();
        let track = track_manager.load_track(&track_choice).unwrap_or_else(|_| classic_grand_prix());

        let mut audio = AudioManager::new();
        audio.settings.master_volume = config.audio.master_volume;
        audio.settings.sfx_volume = config.audio.sfx_volume;
        audio.settings.music_volume = config.audio.music_volume;

        let mut input = InputController::new();
        input.filter.config.steer_rise_rate = config.input.steer_rise_rate;
        input.filter.config.steer_return_rate = config.input.steer_return_rate;
        input.filter.config.steer_exponent = config.input.steer_exponent;
        input.filter.config.speed_sensitive_factor = config.input.speed_sensitive_factor;
        input.filter.config.min_speed_steer_limit = config.input.min_speed_steer_limit;
        input.filter.config.throttle_rise_rate = config.input.throttle_rise_rate;
        input.filter.config.brake_rise_rate = config.input.brake_rise_rate;

        let camera = RaceCamera::from_config(&config.camera);
        let editor_camera = EditorCamera::from_config(&config.camera);

        let mut session = Self {
            state: GameState::Menu,
            track,
            track_choice,
            track_manager,
            car_choice,
            free_car_selection: false,
            assist_profile,
            is_time_attack: false,
            num_bots: config.gameplay.default_num_bots,
            total_laps: config.gameplay.default_laps,
            config,

            cars: Vec::new(),
            color_schemes: Vec::new(),
            trackers: Vec::new(),
            ai_drivers: Vec::new(),
            opponent_drivers: Vec::new(),
            driver_cards_idx: 0,

            active_profile: PlayerProfile::default(),
            active_profile_stats: ProfileCareerStats::default(),
            profile_list: Vec::new(),
            profile_history: Vec::new(),

            fx: EffectsManager::new(8000, 1500),
            camera,
            input,
            touch: TouchController::new(),
            fonts: Fonts::load_embedded(),

            ghost_recorder: GhostRecorder::new(),
            replay_recorder: None,
            replay_player: None,
            is_replay_mode: false,

            session_time: 0.0,
            accumulator: 0.0,
            results: Vec::new(),

            hof_db: HallOfFameDb::open_default().ok(),
            hof_entries: Vec::new(),
            recent_hof_id: None,
            recent_congrats: None,
            show_hall_of_fame: true,

            menu_track_idx: 0,
            menu_car_idx: 0,
            editor_state: None,
            editor_camera,
            editor_tools: ToolSettings::default(),
            editor_modal: EditorModal::None,
            test_drive_car: None,
            test_drive_tracker: None,
            test_drive_time: 0.0,
            audio,
            engine_rpm: EngineRpmModel::default(),
            prev_countdown_sec: 4,
            prev_player_sector: 0,
            curb_sound_cooldown: 0.0,
            offroad_sound_cooldown: 0.0,
            prev_player_lap: 1,
        };

        session.refresh_profiles_and_stats();
        session.refresh_hof_entries();
        session.init_race();
        session.state = GameState::Menu; // Start in main menu
        session
    }

    /// Synchronizes active profile, all profiles list, career stats, and race history with the database.
    pub fn refresh_profiles_and_stats(&mut self) {
        if let Some(db) = &self.hof_db {
            let _ = db.seed_default_profile_if_empty();
            if let Ok(active) = db.get_active_profile() {
                self.active_profile = active;
            }
            if let Ok(all) = db.get_all_profiles() {
                self.profile_list = all;
            }
            if let Some(pid) = self.active_profile.id {
                if let Ok(stats) = db.get_stats_for_profile(pid) {
                    self.active_profile_stats = stats;
                }
                if let Ok(hist) = db.get_history_for_profile(pid, 20) {
                    self.profile_history = hist;
                }
            }
        }
    }

    /// Sets the active driver profile by ID and updates active session data.
    pub fn set_active_profile_by_id(&mut self, profile_id: i64) {
        if let Some(db) = &self.hof_db {
            let _ = db.set_active_profile(profile_id);
        }
        self.refresh_profiles_and_stats();
    }

    /// Track identifier string used for Hall of Fame records.
    pub fn track_choice_id(&self) -> &str {
        self.track_choice.track_id()
    }

    /// Refreshes the cached Top 10 Hall of Fame list for the current track.
    pub fn refresh_hof_entries(&mut self) {
        let track_id = self.track_choice_id();
        if let Some(db) = &self.hof_db {
            let _ = db.seed_defaults_if_empty(track_id);
            if let Ok(entries) = db.get_top_10(track_id) {
                self.hof_entries = entries;
            }
        }
    }

    /// Asynchronously initializes audio banks and plays the synthwave menu theme.
    pub async fn init_audio(&mut self) {
        self.audio.init_async().await;
        self.audio.play_music(MusicTrack::NeonMenu);
    }

    /// Resolves the track's predefined vehicle model as a `CarChoice`.
    pub fn resolve_predefined_car(&self) -> CarChoice {
        match self.track.predefined_car.as_deref() {
            Some("drift_car") => CarChoice::DriftCar,
            Some("kart") => CarChoice::Kart,
            Some("rally_car") => CarChoice::RallyCar,
            Some("sports_car") | None | _ => CarChoice::SportsCar,
        }
    }

    /// Returns the active vehicle model for the player, respecting track enforcement or free selection.
    pub fn active_player_car_choice(&self) -> CarChoice {
        if self.free_car_selection {
            self.car_choice
        } else {
            self.resolve_predefined_car()
        }
    }

    /// Reconstructs participant cars, trackers, and AI drivers for the active roster.
    pub fn rebuild_roster_participants(&mut self) {
        let total_cars = if self.is_time_attack {
            1
        } else {
            (1 + self.num_bots).min(self.track.grid_positions.len()).min(8)
        };

        self.cars.clear();
        self.color_schemes.clear();
        self.trackers.clear();
        self.ai_drivers.clear();
        self.opponent_drivers.clear();

        let num_cps = self.track.checkpoints.len();
        let num_sectors = 3;

        let seed = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42))
            .wrapping_add((self.num_bots as u64) * 101);

        if !self.is_time_attack && total_cars > 1 {
            self.opponent_drivers = DriverCharacter::sample_opponents(total_cars - 1, seed);
        }

        let player_car_choice = self.active_player_car_choice();
        let mut base_config = self.config.get_car_config(player_car_choice);
        base_config.assists = self.assist_profile.to_config();

        let grid_pose_0 = self
            .track
            .grid_positions
            .first()
            .copied()
            .unwrap_or(SpawnPose {
                position: Vec2::ZERO,
                angle: 0.0,
                grid_slot: 0,
            });
        let player_car = Car::new(base_config).with_pose(grid_pose_0.position, grid_pose_0.angle);
        self.cars.push(player_car);
        self.color_schemes.push(self.active_profile.color_scheme);
        self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));

        for (bot_idx, character) in self.opponent_drivers.iter().enumerate() {
            let car_idx = bot_idx + 1;
            let grid_pose = self
                .track
                .grid_positions
                .get(car_idx)
                .copied()
                .unwrap_or(SpawnPose {
                    position: Vec2::ZERO,
                    angle: 0.0,
                    grid_slot: car_idx,
                });

            let bot_car_choice = if self.free_car_selection {
                character.preferred_car
            } else {
                player_car_choice
            };
            let bot_config = self.config.get_car_config(bot_car_choice);
            let bot_car = Car::new(bot_config).with_pose(grid_pose.position, grid_pose.angle);

            self.cars.push(bot_car);
            self.color_schemes.push(character.color_scheme);
            self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));
            self.ai_drivers.push(BotAiDriver::new(character.profile));
        }
    }

    /// Initializes or resets the racing circuit, cars, grid spawns, AI drivers, and camera.
    pub fn init_race(&mut self) {
        self.recent_hof_id = None;
        self.recent_congrats = None;
        self.show_hall_of_fame = true;
        self.refresh_hof_entries();

        // 1. Build selected track
        self.track = self
            .track_manager
            .load_track(&self.track_choice)
            .unwrap_or_else(|_| classic_grand_prix());

        // Predefined balanced lap count from track
        self.total_laps = self.track.default_laps;

        // 2. Setup camera
        self.camera.setup_for_track(&self.track);

        // 3. Build participants & cars
        self.rebuild_roster_participants();

        self.fx.clear();
        self.results.clear();
        self.session_time = 0.0;
        self.accumulator = 0.0;
        self.prev_player_lap = 1;

        // Reset ghost active lap samples
        self.ghost_recorder.on_lap_invalidated();

        // Reset input filter smoothing state
        self.input.reset();

        // Start new replay recording
        let active_car = self.active_player_car_choice();
        self.replay_recorder = Some(ReplayRecorder::new(
            self.track_choice.clone(),
            active_car,
            42,
            Self::FIXED_DT,
        ));

        // Audio initialization for new race
        self.prev_countdown_sec = 4;
        self.prev_player_sector = 0;
        self.curb_sound_cooldown = 0.0;
        self.offroad_sound_cooldown = 0.0;
        self.audio.stop_all_loops();
        self.audio.stop_music(); // In-game music muted

        // Show Starting Grid with selected race participants (or directly countdown if time attack)
        let total_cars = self.cars.len();
        if !self.is_time_attack && total_cars > 1 {
            self.state = GameState::StartingGrid;
        } else {
            self.state = GameState::Countdown(3.5);
        }
    }

    /// Master update tick called once per frame.
    pub fn update(&mut self) {
        let frame_dt = get_frame_time_safe().min(0.1);
        let sw = screen_width_safe();
        let sh = screen_height_safe();

        // Handle gamepad input updates
        self.input.gamepad.update();

        // Handle debug toggles
        self.input.update_debug_toggles();

        // Handle touch controls update
        self.touch.update_from_macroquad(sw, sh, frame_dt);

        // Toggle audio mute (M key)
        if is_key_pressed(KeyCode::M) {
            self.audio.toggle_mute();
        }

        // Adjust Master Volume (LeftBracket / RightBracket)
        if is_key_pressed(KeyCode::LeftBracket) {
            let v = (self.audio.settings.master_volume - 0.1).clamp(0.0, 1.0);
            self.audio.set_master_volume(v);
        }
        if is_key_pressed(KeyCode::RightBracket) {
            let v = (self.audio.settings.master_volume + 0.1).clamp(0.0, 1.0);
            self.audio.set_master_volume(v);
        }

        // Toggle touch overlay on desktop (F6 or Z key)
        if is_key_pressed(KeyCode::F6) {
            self.touch.enabled = !self.touch.enabled;
        }

        // Toggle touch layout (L key)
        if is_key_pressed(KeyCode::L) {
            self.touch.toggle_layout();
        }

        // Handle camera toggle / zoom cycle (Tab key or Gamepad Left Stick Click)
        if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_cam_toggle_pressed {
            if self.state == GameState::TrackEditor {
                let bounds = self.editor_state.as_ref().and_then(|s| {
                    let mut min = Vec2::splat(f32::MAX);
                    let mut max = Vec2::splat(f32::MIN);
                    for wp in &s.track.spline.waypoints {
                        min = min.min(wp.point);
                        max = max.max(wp.point);
                    }
                    if min.x <= max.x {
                        Some((min, max))
                    } else {
                        None
                    }
                });
                self.editor_camera.cycle_zoom_level_with_bounds(bounds, sw, sh);
                self.audio.play_sfx(SfxType::UiMove);
            } else {
                let lvl = self.camera.cycle_zoom_level().clone();
                self.audio.play_sfx(SfxType::UiMove);
                if let Some(player_car) = self.cars.first() {
                    let lvl_idx = self.camera.current_level_idx + 1;
                    let total_lvls = self.camera.levels.len();
                    self.fx.drift_popups.spawn_text(
                        player_car.state.position,
                        &format!("CAMERA: {} ({}/{})", lvl.name.to_uppercase(), lvl_idx, total_lvls),
                        Color::new(0.3, 0.9, 1.0, 1.0),
                    );
                }
            }
        }

        // Open Controls & Driving Assists Screen (C or K key)
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) {
            self.audio.play_sfx(SfxType::UiSelect);
            let from_paused = matches!(self.state, GameState::Racing | GameState::Paused | GameState::Countdown(_));
            self.state = GameState::ControlsHelp(from_paused);
            return;
        }

        if self.state == GameState::TrackEditor {
            self.update_track_editor(frame_dt);
            return;
        }

        if self.state == GameState::EditorTestDrive {
            self.update_editor_test_drive(frame_dt);
            return;
        }

        // Open Driver Cards Dossier Screen (D key)
        if is_key_pressed(KeyCode::D) {
            self.audio.play_sfx(SfxType::UiSelect);
            let origin = match self.state {
                GameState::StartingGrid => DriverCardsOrigin::StartingGrid,
                GameState::Racing | GameState::Paused | GameState::Countdown(_) => DriverCardsOrigin::Paused,
                _ => DriverCardsOrigin::Menu,
            };
            self.state = GameState::DriverCards(origin);
            return;
        }


        // Cycle Driver Assists Profile (H key or Gamepad Right Stick Click / Select)
        if is_key_pressed(KeyCode::H) || self.input.gamepad.snapshot.btn_assist_toggle_pressed {
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

        // Dispatch dedicated profile manager / creation state logic
        if let GameState::ProfileManager { selected_idx } = self.state {
            self.update_profile_manager(selected_idx);
            return;
        }

        if matches!(self.state, GameState::ProfileCreate { .. }) {
            if let GameState::ProfileCreate {
                editing_id,
                field_idx,
                input_name,
                input_alias,
                country_idx,
                livery_idx,
                cursor_timer,
            } = std::mem::replace(&mut self.state, GameState::Menu)
            {
                self.update_profile_create(
                    editing_id,
                    field_idx,
                    input_name,
                    input_alias,
                    country_idx,
                    livery_idx,
                    cursor_timer,
                    frame_dt,
                );
                return;
            }
        }

        match self.state {
            GameState::Menu => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_menu();
            }
            GameState::StartingGrid => {
                // 1. Toggle Free Car Selection (F or C key, or Gamepad X)
                if is_key_pressed(KeyCode::F)
                    || is_key_pressed(KeyCode::C)
                    || self.input.gamepad.snapshot.btn_x_pressed
                {
                    self.free_car_selection = !self.free_car_selection;
                    self.rebuild_roster_participants();
                    self.audio.play_sfx(SfxType::UiSelect);
                }

                // 2. Cycle Selected Car (when free car selection is active)
                if self.free_car_selection {
                    if is_key_pressed(KeyCode::Left)
                        || self.input.gamepad.snapshot.dpad_left_pressed
                        || self.input.gamepad.snapshot.nav_left
                        || is_key_pressed(KeyCode::LeftBracket)
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        if self.menu_car_idx == 0 {
                            self.menu_car_idx = CarChoice::ALL.len() - 1;
                        } else {
                            self.menu_car_idx -= 1;
                        }
                        self.car_choice = CarChoice::ALL[self.menu_car_idx];
                        self.rebuild_roster_participants();
                    }
                    if is_key_pressed(KeyCode::Right)
                        || self.input.gamepad.snapshot.dpad_right_pressed
                        || self.input.gamepad.snapshot.nav_right
                        || is_key_pressed(KeyCode::RightBracket)
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.menu_car_idx = (self.menu_car_idx + 1) % CarChoice::ALL.len();
                        self.car_choice = CarChoice::ALL[self.menu_car_idx];
                        self.rebuild_roster_participants();
                    }
                }

                // 3. Modify Driver Count (B, N, Up/Down, Gamepad D-Pad Up/Down)
                let max_bots = (self.track.grid_positions.len().saturating_sub(1)).clamp(1, 7);

                // Cycle next driver count (B / N)
                if is_key_pressed(KeyCode::B)
                    || is_key_pressed(KeyCode::N)
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.num_bots = (self.num_bots % max_bots) + 1;
                    self.rebuild_roster_participants();
                }

                // Increase driver count (Up / Equal / Gamepad D-pad Up)
                if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Equal)
                    || self.input.gamepad.snapshot.dpad_up_pressed
                    || (self.input.gamepad.snapshot.nav_up && !self.free_car_selection)
                {
                    if self.num_bots < max_bots {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.num_bots += 1;
                        self.rebuild_roster_participants();
                    }
                }

                // Decrease driver count (Down / Minus / Gamepad D-pad Down)
                if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::Minus)
                    || self.input.gamepad.snapshot.dpad_down_pressed
                    || (self.input.gamepad.snapshot.nav_down && !self.free_car_selection)
                {
                    if self.num_bots > 1 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.num_bots -= 1;
                        self.rebuild_roster_participants();
                    }
                }

                // Launch race countdown (Space, Enter, Gamepad Confirm [A / South / Start])
                if is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Countdown(3.5);
                }

                // View Driver Dossiers (D key or Gamepad Y)
                if is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.btn_y_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
                }

                // Return to Main Menu (Escape, M, or Gamepad Cancel [B / East / Back])
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::M)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Menu;
                    self.audio.play_music(MusicTrack::NeonMenu);
                }
            }
            GameState::Countdown(ref mut remaining) => {

                *remaining -= frame_dt;

                // Player launch throttle / revs on grid
                let kb_ctrl = self.input.poll_player_controls(frame_dt, 0.0);
                let touch_ctrl = self.touch.poll_controls();
                let player_ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);
                let (rpm, is_shift) = self.engine_rpm.update(0.0, player_ctrl.throttle, 0.0, frame_dt);
                self.audio.update_engine_rpm(rpm, player_ctrl.throttle, is_shift);

                // Countdown audio beeps (3, 2, 1)
                if *remaining <= 3.0 && self.prev_countdown_sec > 3 {
                    self.audio.play_sfx(SfxType::CountdownLow);
                    self.prev_countdown_sec = 3;
                } else if *remaining <= 2.0 && self.prev_countdown_sec > 2 {
                    self.audio.play_sfx(SfxType::CountdownLow);
                    self.prev_countdown_sec = 2;
                } else if *remaining <= 1.0 && self.prev_countdown_sec > 1 {
                    self.audio.play_sfx(SfxType::CountdownLow);
                    self.prev_countdown_sec = 1;
                }

                // Camera follows player during countdown
                if let Some(player_car) = self.cars.first() {
                    self.camera.update(player_car, frame_dt);
                }

                if *remaining <= 0.0 {
                    self.audio.play_sfx(SfxType::CountdownHigh);
                    self.state = GameState::Racing;
                }
            }
            GameState::Racing => {
                // Pause trigger (Escape / Pause key or Gamepad Start)
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Pause) || self.input.gamepad.snapshot.btn_start_pressed {
                    self.audio.stop_all_loops();
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
                self.audio.stop_all_loops();
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Pause)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_start_pressed
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.state = GameState::Racing;
                }
                if is_key_pressed(KeyCode::M) || self.input.gamepad.snapshot.btn_cancel_pressed || self.input.gamepad.snapshot.btn_back_pressed || self.input.gamepad.snapshot.btn_b_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Menu;
                    self.audio.play_music(MusicTrack::NeonMenu);
                }
                if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::ControlsHelp(true);
                }
            }
            GameState::Finished => {
                self.audio.stop_all_loops();
                if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_x_pressed {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.show_hall_of_fame = !self.show_hall_of_fame;
                }
                if is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.init_race();
                }
                if is_key_pressed(KeyCode::M) || self.input.gamepad.snapshot.btn_cancel_pressed || self.input.gamepad.snapshot.btn_back_pressed || self.input.gamepad.snapshot.btn_b_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Menu;
                    self.audio.play_music(MusicTrack::NeonMenu);
                }
            }

            GameState::ControlsHelp(from_paused) => {
                if is_key_pressed(KeyCode::H) || self.input.gamepad.snapshot.btn_assist_toggle_pressed {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.assist_profile = self.assist_profile.next();
                    if let Some(player_car) = self.cars.first_mut() {
                        player_car.config.assists = self.assist_profile.to_config();
                    }
                }

                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::C)
                    || is_key_pressed(KeyCode::K)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if from_paused {
                        self.state = GameState::Paused;
                    } else {
                        self.state = GameState::Menu;
                    }
                }
            }

            GameState::DriverCards(origin) => {
                let roster_len = DriverCharacter::all().len();
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                    self.audio.play_sfx(SfxType::UiMove);
                    if self.driver_cards_idx == 0 {
                        self.driver_cards_idx = roster_len - 1;
                    } else {
                        self.driver_cards_idx -= 1;
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.driver_cards_idx = (self.driver_cards_idx + 1) % roster_len;
                }

                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::C)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    match origin {
                        DriverCardsOrigin::StartingGrid => self.state = GameState::StartingGrid,
                        DriverCardsOrigin::Paused => self.state = GameState::Paused,
                        DriverCardsOrigin::Menu => self.state = GameState::Menu,
                    }
                }
            }

            GameState::ProfileManager { .. }
            | GameState::ProfileCreate { .. }
            | GameState::TrackEditor
            | GameState::EditorTestDrive => {}

        }
    }

    /// Handles input and actions for the Profile Manager screen.
    fn update_profile_manager(&mut self, selected_idx: usize) {
        let mut current_idx = selected_idx;
        let count = self.profile_list.len();
        if count == 0 {
            self.refresh_profiles_and_stats();
        }

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
            self.audio.play_sfx(SfxType::UiMove);
            if current_idx == 0 {
                current_idx = self.profile_list.len().saturating_sub(1);
            } else {
                current_idx -= 1;
            }
            if let Some(p) = self.profile_list.get(current_idx) {
                if let Some(pid) = p.id {
                    if let Some(db) = &self.hof_db {
                        self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                        self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
            self.audio.play_sfx(SfxType::UiMove);
            if !self.profile_list.is_empty() {
                current_idx = (current_idx + 1) % self.profile_list.len();
            }
            if let Some(p) = self.profile_list.get(current_idx) {
                if let Some(pid) = p.id {
                    if let Some(db) = &self.hof_db {
                        self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                        self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                    }
                }
            }
        }

        // Set Active profile (Enter / Space / Gamepad A / Confirm)
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            if let Some(p) = self.profile_list.get(current_idx) {
                if let Some(pid) = p.id {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.set_active_profile_by_id(pid);
                }
            }
        }

        // Edit Profile (E key)
        if is_key_pressed(KeyCode::E) {
            if let Some(p) = self.profile_list.get(current_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                let country_idx = p.country.as_deref().and_then(|code| {
                    CountryRegistry::ALL.iter().position(|c| c.code.eq_ignore_ascii_case(code)).map(|pos| pos + 1)
                }).unwrap_or(0);

                let livery_idx = Palette::CAR_COLORS.iter().position(|c| {
                    c.0 == p.color_scheme.primary && c.1 == p.color_scheme.secondary
                }).unwrap_or(0);

                self.state = GameState::ProfileCreate {
                    editing_id: p.id,
                    field_idx: 0,
                    input_name: p.name.clone(),
                    input_alias: p.alias.clone(),
                    country_idx,
                    livery_idx,
                    cursor_timer: 0.0,
                };
                return;
            }
        }

        // Create New Profile (N key or Gamepad X)
        if is_key_pressed(KeyCode::N) || self.input.gamepad.snapshot.btn_x_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            let next_livery = self.profile_list.len() % Palette::CAR_COLORS.len();
            self.state = GameState::ProfileCreate {
                editing_id: None,
                field_idx: 0,
                input_name: String::new(),
                input_alias: String::new(),
                country_idx: 1, // Spain default
                livery_idx: next_livery,
                cursor_timer: 0.0,
            };
            return;
        }

        // Delete Profile (Delete / X key or Gamepad Y) - only when more than 1 profile exists
        if (is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::X)) && self.profile_list.len() > 1 {
            if let Some(p) = self.profile_list.get(current_idx) {
                if let Some(pid) = p.id {
                    self.audio.play_sfx(SfxType::UiMove);
                    if let Some(db) = &self.hof_db {
                        let _ = db.delete_profile(pid);
                    }
                    self.refresh_profiles_and_stats();
                    current_idx = current_idx.min(self.profile_list.len().saturating_sub(1));
                }
            }
        }

        // Return to Main Menu (Escape, M, or Gamepad Cancel / B)
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::M)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.refresh_profiles_and_stats();
            self.state = GameState::Menu;
            return;
        }

        self.state = GameState::ProfileManager {
            selected_idx: current_idx,
        };
    }

    /// Handles input and interactive wizard navigation for creating or editing a Driver Profile.
    #[allow(clippy::too_many_arguments)]
    fn update_profile_create(
        &mut self,
        editing_id: Option<i64>,
        mut field_idx: usize,
        mut input_name: String,
        mut input_alias: String,
        mut country_idx: usize,
        mut livery_idx: usize,
        mut cursor_timer: f32,
        frame_dt: f32,
    ) {
        cursor_timer += frame_dt;

        // Field switching (Tab, Up, Down)
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Down) {
            self.audio.play_sfx(SfxType::UiMove);
            field_idx = (field_idx + 1) % 4;
        }
        if is_key_pressed(KeyCode::Up) {
            self.audio.play_sfx(SfxType::UiMove);
            if field_idx == 0 {
                field_idx = 3;
            } else {
                field_idx -= 1;
            }
        }

        // Text typing for fields 0 (Name) and 1 (Alias)
        if field_idx == 0 {
            while let Some(c) = get_char_pressed() {
                if (c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_') && input_name.len() < 16 {
                    input_name.push(c);
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
            if is_key_pressed(KeyCode::Backspace) && !input_name.is_empty() {
                input_name.pop();
                self.audio.play_sfx(SfxType::UiMove);
            }
        } else if field_idx == 1 {
            while let Some(c) = get_char_pressed() {
                if (c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_') && input_alias.len() < 16 {
                    input_alias.push(c);
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
            if is_key_pressed(KeyCode::Backspace) && !input_alias.is_empty() {
                input_alias.pop();
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // Left/Right selection for Country (field 2) and Livery (field 3)
        let total_countries = CountryRegistry::ALL.len() + 1; // 0 = None / International
        let total_liveries = Palette::CAR_COLORS.len();

        if field_idx == 2 {
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                self.audio.play_sfx(SfxType::UiMove);
                if country_idx == 0 {
                    country_idx = total_countries - 1;
                } else {
                    country_idx -= 1;
                }
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                self.audio.play_sfx(SfxType::UiMove);
                country_idx = (country_idx + 1) % total_countries;
            }
        }

        if field_idx == 3 {
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                self.audio.play_sfx(SfxType::UiMove);
                if livery_idx == 0 {
                    livery_idx = total_liveries - 1;
                } else {
                    livery_idx -= 1;
                }
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                self.audio.play_sfx(SfxType::UiMove);
                livery_idx = (livery_idx + 1) % total_liveries;
            }
        }

        // Confirm & Save Profile
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            let final_name = if input_name.trim().is_empty() {
                "New Racer".to_string()
            } else {
                input_name.trim().to_string()
            };

            let final_alias = if input_alias.trim().is_empty() {
                "Speedster".to_string()
            } else {
                input_alias.trim().to_string()
            };

            let country_opt = if country_idx > 0 && country_idx <= CountryRegistry::ALL.len() {
                Some(CountryRegistry::ALL[country_idx - 1].code.to_string())
            } else {
                None
            };

            let scheme = CarColorScheme::from_index(livery_idx);
            let mut target_idx = 0;

            if let Some(db) = &self.hof_db {
                if let Some(edit_id) = editing_id {
                    let mut updated = PlayerProfile::new(&final_name, &final_alias, country_opt.as_deref(), scheme);
                    updated.id = Some(edit_id);
                    if let Some(existing) = self.profile_list.iter().find(|p| p.id == Some(edit_id)) {
                        updated.is_active = existing.is_active;
                    }
                    let _ = db.update_profile(&updated);
                    target_idx = self.profile_list.iter().position(|p| p.id == Some(edit_id)).unwrap_or(0);
                } else {
                    let mut new_profile = PlayerProfile::new(&final_name, &final_alias, country_opt.as_deref(), scheme);
                    new_profile.is_active = true;
                    if let Ok(new_id) = db.create_profile(&new_profile) {
                        let _ = db.set_active_profile(new_id);
                    }
                    target_idx = 0;
                }
            }
            self.refresh_profiles_and_stats();
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::ProfileManager { selected_idx: target_idx };
            return;
        }

        // Cancel (Escape or Gamepad Cancel / B)
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            let return_idx = editing_id.and_then(|id| self.profile_list.iter().position(|p| p.id == Some(id))).unwrap_or(0);
            self.state = GameState::ProfileManager { selected_idx: return_idx };
            return;
        }

        self.state = GameState::ProfileCreate {
            editing_id,
            field_idx,
            input_name,
            input_alias,
            country_idx,
            livery_idx,
            cursor_timer,
        };
    }


    /// Menu input navigation (Keyboard + Gamepad D-pad/Analog Sticks/buttons).
    fn update_menu(&mut self) {
        // Check for gamepad mapping changes on disk when in/reloading the main menu
        self.input.gamepad.check_and_reload_profile();

        // Open Controls & Gamepad Screen (C / K key or Gamepad Select)
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) || self.input.gamepad.snapshot.btn_back_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::ControlsHelp(false);
            return;
        }

        // Open Player Profile & Career History Screen (P key or Gamepad Y)
        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.refresh_profiles_and_stats();
            let current_idx = self
                .profile_list
                .iter()
                .position(|p| p.id == self.active_profile.id)
                .unwrap_or(0);
            self.state = GameState::ProfileManager {
                selected_idx: current_idx,
            };
            return;
        }

        let available_tracks = self.track_manager.all_track_choices();
        if self.menu_track_idx >= available_tracks.len() {
            self.menu_track_idx = 0;
        }

        // Track selection cursor (Up/Down: Arrows / W/S / D-pad / Left Stick Y)
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
            self.audio.play_sfx(SfxType::UiMove);
            if self.menu_track_idx == 0 {
                self.menu_track_idx = available_tracks.len().saturating_sub(1);
            } else {
                self.menu_track_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_track_idx = (self.menu_track_idx + 1) % available_tracks.len();
        }

        // Car selection cursor (Left/Right: Arrows / A/D / D-pad / Left Stick X)
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
            self.audio.play_sfx(SfxType::UiMove);
            if self.menu_car_idx == 0 {
                self.menu_car_idx = CarChoice::ALL.len() - 1;
            } else {
                self.menu_car_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_car_idx = (self.menu_car_idx + 1) % CarChoice::ALL.len();
        }

        // Toggle Mode (Time Attack vs Race vs AI - T key or Gamepad X)
        if is_key_pressed(KeyCode::T) || self.input.gamepad.snapshot.btn_x_pressed {
            self.audio.play_sfx(SfxType::UiMove);
            self.is_time_attack = !self.is_time_attack;
        }

        // Change bot count (B key or Gamepad Y)
        if is_key_pressed(KeyCode::B) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiMove);
            self.num_bots = (self.num_bots % 7) + 1;
        }

        // Toggle Driver Assists Profile (H key or Gamepad Right Stick Click / Select)
        if is_key_pressed(KeyCode::H) || self.input.gamepad.snapshot.btn_assist_toggle_pressed {
            self.audio.play_sfx(SfxType::UiMove);
            self.assist_profile = self.assist_profile.next();
        }

        // Open Track Editor (E key)
        if is_key_pressed(KeyCode::E) {
            self.audio.play_sfx(SfxType::UiSelect);
            let available_tracks = self.track_manager.all_track_choices();
            let chosen = available_tracks
                .get(self.menu_track_idx)
                .cloned()
                .unwrap_or(TrackChoice::ClassicGrandPrix);
            let track = self
                .track_manager
                .load_track(&chosen)
                .unwrap_or_else(|_| classic_grand_prix());
            self.enter_track_editor(track);
            return;
        }

        // Start race (Space, Enter, or Gamepad Confirm [A / South / Start])
        if is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            let available_tracks = self.track_manager.all_track_choices();
            self.track_choice = available_tracks
                .get(self.menu_track_idx)
                .cloned()
                .unwrap_or(TrackChoice::ClassicGrandPrix);
            self.car_choice = CarChoice::ALL[self.menu_car_idx];
            self.init_race();
        }
    }

    /// High-performance deterministic fixed physics simulation step.
    pub fn physics_step(&mut self, dt: f32) {
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

        // Trigger Jump Ramps & Landing SFX/FX
        for (i, car) in self.cars.iter_mut().enumerate() {
            for ramp in &self.track.geometry.jump_ramps {
                if car.try_trigger_jump_ramp(ramp) {
                    if i == 0 {
                        self.audio.play_sfx(SfxType::JumpLaunch);
                    }
                    break;
                }
            }
            if car.state.just_landed {
                if i == 0 {
                    self.audio.play_sfx(SfxType::Landing);
                    self.camera.add_trauma(0.25);
                }
                let surf = wheel_surfaces.get(i).map(|s| s[0]).unwrap_or(SurfaceType::Asphalt);
                self.fx.particles.emit_landing_dust(car.state.position, car.state.speed, surf);
            }
        }

        // Water splash sound effect on player car
        if let Some(player_surfaces) = wheel_surfaces.first() {
            let in_water = player_surfaces.iter().any(|&s| s == SurfaceType::Water);
            if in_water {
                if let Some(player_car) = self.cars.first() {
                    if player_car.state.speed > 3.0 && self.session_time.fract() < dt * 4.0 {
                        let gain = (player_car.state.speed / 18.0).clamp(0.35, 0.85);
                        self.audio.play_sfx_with_gain(SfxType::WaterSplash, gain);
                    }
                }
            }
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

        // Camera screen shake and audio on player impacts
        for wev in &wall_collision_events {
            if wev.impact_speed > 3.0 {
                self.camera.add_trauma(wev.impact_speed * 0.08);
            }
            if wev.impact_speed > 2.2 {
                let gain = (wev.impact_speed / 16.0).clamp(0.3, 0.9);
                self.audio.play_sfx_with_gain(SfxType::WallCrash, gain);
            }
        }
        for cev in &car_collision_events {
            if cev.car_a_idx == 0 || cev.car_b_idx == 0 {
                if cev.closing_speed > 3.0 {
                    self.camera.add_trauma(cev.closing_speed * 0.06);
                }
                if cev.closing_speed > 2.0 {
                    let gain = (cev.closing_speed / 14.0).clamp(0.25, 0.85);
                    self.audio.play_sfx_with_gain(SfxType::CarHit, gain);
                }
            }
        }

        // Dynamic Arcade Tire Skid / Drift Chirps & Accelerating Engine Audio
        if let Some(player_car) = self.cars.first() {
            let max_slip_angle = player_car.state.wheels.iter().map(|w| w.slip_angle.abs()).fold(0.0f32, f32::max);
            let max_slip_ratio = player_car.state.wheels.iter().map(|w| w.slip_ratio.abs()).fold(0.0f32, f32::max);
            let slip_intensity = (max_slip_angle * 1.5).max(max_slip_ratio);
            self.audio.update_skid_chirp(slip_intensity, dt);

            let forward_speed = player_car.state.local_velocity.x;
            let throttle = player_ctrl.throttle - player_ctrl.brake;
            let (rpm, is_shift) = self.engine_rpm.update(forward_speed, throttle, slip_intensity, dt);
            self.audio.update_engine_rpm(rpm, throttle, is_shift);
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

        // Lap and sector split audio feedback
        if let Some(tracker) = self.trackers.first() {
            let lap_changed = tracker.current_lap > self.prev_player_lap;

            if tracker.current_sector != self.prev_player_sector {
                if tracker.current_sector > 0 {
                    self.audio.play_sfx(SfxType::SectorPing);
                }
                self.prev_player_sector = tracker.current_sector;
            }
            if lap_changed && (self.is_time_attack || tracker.current_lap <= self.total_laps) {
                self.audio.play_sfx(SfxType::LapChime);
            }

            // 7. Ghost lap telemetry recording (Time Attack)
            if self.is_time_attack {
                if let Some(player_car) = self.cars.first() {
                    self.ghost_recorder.record_frame(tracker.lap_time, player_car, dt);

                    if lap_changed {
                        if let Some(last_lap_time) = tracker.last_lap_time {
                            self.ghost_recorder.on_lap_completed(
                                last_lap_time,
                                self.track_choice.clone(),
                                self.car_choice,
                            );
                        }
                    }
                }
            }

            if lap_changed {
                self.prev_player_lap = tracker.current_lap;
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
    pub fn check_race_finish(&mut self) {
        let player_done = self
            .trackers
            .first()
            .is_some_and(|t| t.current_lap > self.total_laps);

        if player_done && !self.is_time_attack {
            self.build_results();
            self.audio.stop_all_loops();
            self.audio.play_sfx(SfxType::RaceFinish);

            let track_id = self.track_choice_id().to_string();
            let player_time = self.session_time;
            let player_best_lap = self.trackers.first().and_then(|t| t.best_lap_time);

            // 1. Check personal best lap against active profile stats before updating
            let prev_best_lap = self.active_profile_stats.best_times.get(&track_id).copied();
            let is_pb = player_best_lap.is_some_and(|lap| prev_best_lap.map_or(true, |prev| lap < prev));

            // 2. Log player race result to persistent history
            let player_pos = self.results.iter().position(|r| r.is_player).map(|p| p + 1).unwrap_or(1);
            let active_player_car = self.active_player_car_choice();
            if let Some(pid) = self.active_profile.id {
                let history_record = RaceHistoryEntry {
                    id: None,
                    profile_id: pid,
                    track_id: track_id.clone(),
                    car_name: active_player_car.title().to_string(),
                    position: player_pos,
                    total_cars: self.cars.len(),
                    total_time: player_time,
                    best_lap: player_best_lap,
                    laps: self.total_laps,
                    is_time_attack: self.is_time_attack,
                    created_at: String::new(),
                };
                if let Some(db) = &self.hof_db {
                    let _ = db.insert_race_history(&history_record);
                }
                self.refresh_profiles_and_stats();
            }

            // 3. Automatically record all race finishers (player + bots) into the Hall of Fame
            let mut player_hof_id: Option<i64> = None;
            if let Some(db) = &self.hof_db {
                let standings = self.compute_standings();
                for (rank, &car_idx) in standings.iter().enumerate() {
                    let is_player = car_idx == 0;
                    let (driver_name, vehicle_name) = if is_player {
                        (self.active_profile.alias.clone(), active_player_car.title().to_string())
                    } else if let Some(character) = self.opponent_drivers.get(car_idx - 1) {
                        let bot_car = if self.free_car_selection {
                            character.preferred_car.title().to_string()
                        } else {
                            active_player_car.title().to_string()
                        };
                        (character.alias.to_string(), bot_car)
                    } else {
                        (format!("Driver {}", car_idx), active_player_car.title().to_string())
                    };

                    let tracker = &self.trackers[car_idx];
                    let total_time = self.session_time + (rank as f32 * 0.65);
                    let entry = HallOfFameEntry {
                        id: None,
                        track_id: track_id.clone(),
                        player_name: driver_name,
                        car_name: vehicle_name,
                        total_time,
                        best_lap: tracker.best_lap_time,
                        laps: self.total_laps,
                        created_at: String::new(),
                    };

                    if let Ok(inserted_id) = db.insert_entry(&entry) {
                        if is_player {
                            player_hof_id = Some(inserted_id);
                        }
                    }
                }
            }

            self.refresh_hof_entries();

            // 4. Determine player rank in Top 10 Hall of Fame
            let hof_rank = player_hof_id.and_then(|id| {
                self.hof_entries.iter().position(|e| e.id == Some(id)).map(|p| p + 1)
            });

            if hof_rank.is_some() {
                self.recent_hof_id = player_hof_id;
            } else {
                self.recent_hof_id = None;
            }

            // 5. Determine podium finish (top 3 in race)
            let race_position = if player_pos <= 3 { Some(player_pos) } else { None };

            // 6. Build congratulations metadata
            let congrats = PlayerCongrats {
                is_personal_best: is_pb,
                personal_best_lap: player_best_lap,
                hof_rank,
                race_position,
            };
            self.recent_congrats = if congrats.has_achievements() {
                Some(congrats)
            } else {
                None
            };

            self.show_hall_of_fame = true;
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
                format!("{} (You)", self.active_profile.alias)
            } else if let Some(character) = self.opponent_drivers.get(car_idx - 1) {
                character.alias.to_string()
            } else {
                format!("Driver {}", car_idx)
            };

            let tracker = &self.trackers[car_idx];
            let total_time = self.session_time + (rank as f32 * 0.65);
            let leader_time = self.session_time;
            let delta = if rank == 0 { 0.0 } else { total_time - leader_time };

            self.results.push(RaceResultEntry {
                position: rank + 1,
                car_name,
                is_player,
                total_time,
                best_lap: tracker.best_lap_time,
                delta_to_leader: delta,
            });
        }
    }

    /// Renders current UI state, HUD, or pause screen.
    pub fn render(&self) {
        match self.state {
            GameState::Menu => {
                let available_tracks = self.track_manager.all_track_choices();
                render_track_select_menu(
                    &self.fonts,
                    &available_tracks,
                    self.menu_track_idx,
                    self.menu_car_idx,
                    self.num_bots,
                    self.is_time_attack,
                    self.assist_profile,
                    &self.audio.settings,
                    &self.active_profile,
                    &self.active_profile_stats,
                );
            }
            GameState::StartingGrid => {
                self.render_world();
                let player_car_title = self.active_player_car_choice().title();
                let predefined_car_title = self.resolve_predefined_car().title();
                let max_grid_size = self.track.grid_positions.len().min(8);
                render_starting_grid_screen(
                    &self.fonts,
                    &self.track,
                    player_car_title,
                    &self.active_profile,
                    &self.opponent_drivers,
                    self.total_laps,
                    self.is_time_attack,
                    self.input.gamepad.snapshot.is_connected,
                    self.free_car_selection,
                    predefined_car_title,
                    self.cars.len(),
                    max_grid_size,
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
                render_pause_menu(&self.fonts, self.assist_profile, &self.audio.settings);
            }
            GameState::Finished => {
                self.render_world();
                if self.show_hall_of_fame {
                    render_hall_of_fame_screen(
                        &self.fonts,
                        &self.track.name,
                        &self.hof_entries,
                        self.recent_hof_id,
                        self.recent_congrats.as_ref(),
                    );
                } else {
                    render_results_screen(&self.fonts, &self.track.name, &self.results, self.is_time_attack);
                }
            }
            GameState::ControlsHelp(from_paused) => {
                if from_paused {
                    self.render_world();
                }
                render_controls_screen(
                    &self.fonts,
                    self.assist_profile,
                    self.input.gamepad.snapshot.is_connected,
                    &self.input.gamepad.snapshot.gamepad_name,
                );
            }
            GameState::DriverCards(_) => {
                render_driver_cards_screen(&self.fonts, self.driver_cards_idx);
            }
            GameState::ProfileManager { selected_idx } => {
                render_profile_manager_screen(
                    &self.fonts,
                    &self.profile_list,
                    selected_idx,
                    &self.profile_history,
                    &self.active_profile_stats,
                );
            }
            GameState::ProfileCreate {
                editing_id,
                field_idx,
                ref input_name,
                ref input_alias,
                country_idx,
                livery_idx,
                cursor_timer,
            } => {
                render_profile_create_screen(
                    &self.fonts,
                    field_idx,
                    input_name,
                    input_alias,
                    country_idx,
                    livery_idx,
                    cursor_timer,
                    editing_id.is_some(),
                );
            }
            GameState::TrackEditor => {
                self.render_track_editor();
            }
            GameState::EditorTestDrive => {
                self.render_editor_test_drive();
            }
        }
    }

    /// Transitions cleanly into the in-game Track Studio editor with specified circuit.
    pub fn enter_track_editor(&mut self, track: Track) {
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for wp in &track.spline.waypoints {
            min = min.min(wp.point);
            max = max.max(wp.point);
        }
        if min.x > max.x {
            min = Vec2::new(-100.0, -100.0);
            max = Vec2::new(100.0, 100.0);
        }
        self.editor_camera.focus_bounds(min, max, sw, sh);
        self.editor_state = Some(EditorState::new(track));
        self.editor_tools = ToolSettings::default();
        self.editor_modal = EditorModal::None;
        self.state = GameState::TrackEditor;
    }

    /// Spawns vehicle on the circuit starting grid for zero-latency test driving.
    pub fn start_editor_test_drive(&mut self) {
        if let Some(state) = &self.editor_state {
            let mut base_config = self.config.get_car_config(self.car_choice);
            base_config.assists = self.assist_profile.to_config();
            let init_pose = state
                .track
                .grid_positions
                .first()
                .cloned()
                .unwrap_or_else(|| {
                    let p = state.track.spline.waypoints.first().map(|w| w.point).unwrap_or(Vec2::ZERO);
                    tdrace_core::track::geometry::SpawnPose::new(p, 0.0, 0)
                });
            self.test_drive_car = Some(Car::new(base_config).with_pose(init_pose.position, init_pose.angle));
            self.test_drive_tracker = Some(TrackProgressTracker::new(state.track.checkpoints.len(), 100));
            self.test_drive_time = 0.0;
            self.camera.setup_for_track(&state.track);
            self.audio.play_sfx(SfxType::CountdownHigh);
            self.state = GameState::EditorTestDrive;
        }
    }

    /// Frame update tick for Track Studio editing mode.
    pub fn update_track_editor(&mut self, dt: f32) {
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        self.editor_camera.update(dt);

        let (mx, my) = mouse_position_safe();
        let mouse_pos = Vec2::new(mx, my);
        let world_mouse = self.editor_camera.screen_to_world(mouse_pos, sw, sh);

        // Check if cursor is over floating UI palettes or modal
        let in_top = my < 48.0;
        let in_bot = my > sh - 34.0;
        let in_left = mx < 185.0 && my > 48.0 && my < 480.0;
        let in_right = mx > sw - 260.0 && my > 48.0;
        let over_ui = in_top || in_bot || in_left || in_right || self.editor_modal != EditorModal::None;

        if !over_ui {
            if is_mouse_button_pressed(macroquad::input::MouseButton::Middle)
                || is_mouse_button_pressed(macroquad::input::MouseButton::Right)
            {
                self.editor_camera.start_pan(mouse_pos);
            }
            if is_mouse_button_down(macroquad::input::MouseButton::Middle)
                || is_mouse_button_down(macroquad::input::MouseButton::Right)
            {
                self.editor_camera.update_pan(mouse_pos);
            }
            if is_mouse_button_released(macroquad::input::MouseButton::Middle)
                || is_mouse_button_released(macroquad::input::MouseButton::Right)
            {
                self.editor_camera.end_pan();
            }

            let mouse_wheel_y = mouse_wheel_safe().1;
            if mouse_wheel_y.abs() > 0.01 {
                let factor = if mouse_wheel_y > 0.0 { 1.15 } else { 0.85 };
                self.editor_camera.zoom_at(mouse_pos, factor, sw, sh);
            }

            if let Some(state) = &mut self.editor_state {
                if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    self.editor_tools.handle_mouse_down(state, world_mouse);
                }
                if is_mouse_button_down(macroquad::input::MouseButton::Left) {
                    self.editor_tools.handle_mouse_drag(state, world_mouse);
                }
                if is_mouse_button_released(macroquad::input::MouseButton::Left) {
                    self.editor_tools.handle_mouse_up(state, world_mouse);
                }
            }
        }

        // Shortcuts
        if is_key_pressed(KeyCode::Key1) { self.editor_tools.active_tool = EditorToolType::Select; }
        if is_key_pressed(KeyCode::Key2) { self.editor_tools.active_tool = EditorToolType::RoadSpline; }
        if is_key_pressed(KeyCode::Key3) { self.editor_tools.active_tool = EditorToolType::SurfaceZone; }
        if is_key_pressed(KeyCode::Key4) { self.editor_tools.active_tool = EditorToolType::JumpRamp; }
        if is_key_pressed(KeyCode::Key5) { self.editor_tools.active_tool = EditorToolType::Obstacle; }
        if is_key_pressed(KeyCode::Key6) { self.editor_tools.active_tool = EditorToolType::Checkpoint; }
        if is_key_pressed(KeyCode::Key7) { self.editor_tools.active_tool = EditorToolType::StartingGrid; }
        if is_key_pressed(KeyCode::Key8) { self.editor_tools.active_tool = EditorToolType::PitLane; }

        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftSuper))
            && is_key_pressed(KeyCode::Z)
        {
            if let Some(state) = &mut self.editor_state {
                state.undo();
            }
        }
        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftSuper))
            && is_key_pressed(KeyCode::Y)
        {
            if let Some(state) = &mut self.editor_state {
                state.redo();
            }
        }

        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            if let Some(state) = &mut self.editor_state {
                self.editor_tools.delete_selected(state);
            }
        }

        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::P) {
            if self.editor_modal == EditorModal::None {
                self.start_editor_test_drive();
                return;
            }
        }

        if is_key_pressed(KeyCode::F) {
            if let Some(state) = &self.editor_state {
                let mut min = Vec2::splat(f32::MAX);
                let mut max = Vec2::splat(f32::MIN);
                for wp in &state.track.spline.waypoints {
                    min = min.min(wp.point);
                    max = max.max(wp.point);
                }
                if min.x <= max.x {
                    self.editor_camera.focus_bounds(min, max, sw, sh);
                }
            }
        }

        if is_key_pressed(KeyCode::G) {
            if let Some(state) = &mut self.editor_state {
                state.grid_snap = state.grid_snap.next();
            }
        }

        // Action dispatching from UI
        let mut dispatched = EditorAction::None;
        if let Some(state) = &mut self.editor_state {
            dispatched = render_editor_ui(
                &self.fonts,
                state,
                &mut self.editor_tools,
                &mut self.editor_camera,
                &mut self.track_manager,
                &mut self.editor_modal,
            );
        }

        match dispatched {
            EditorAction::StartTestDrive => {
                self.start_editor_test_drive();
            }
            EditorAction::ExitToMenu => {
                self.state = GameState::Menu;
            }
            EditorAction::NewFromTemplate(preset) => {
                let track = match preset.as_str() {
                    "Oval Speedway" => tdrace_core::track::presets::oval_speedway(),
                    "Oasis Rally" => tdrace_core::track::presets::oasis_rally(),
                    "Classic Grand Prix" => tdrace_core::track::presets::classic_grand_prix(),
                    _ => {
                        let mut blank = tdrace_core::track::presets::classic_grand_prix();
                        blank.name = "New Custom Circuit".to_string();
                        blank
                    }
                };
                self.enter_track_editor(track);
            }
            EditorAction::OpenTrack(slug) => {
                let choice = TrackChoice::Custom {
                    id: slug.clone(),
                    title: slug.clone(),
                    path: format!("tracks/{}.json", slug),
                };
                if let Ok(track) = self.track_manager.load_track(&choice) {
                    self.enter_track_editor(track);
                }
            }
            EditorAction::SaveTrack(name) => {
                if let Some(state) = &mut self.editor_state {
                    state.track.name = name;
                    let _ = self.track_manager.save_custom_track(&state.track, None);
                }
            }
            _ => {}
        }
    }

    /// Frame update tick for instant test drive playtesting mode.
    pub fn update_editor_test_drive(&mut self, dt: f32) {
        // [Esc] returns cleanly to Track Studio
        if is_key_pressed(KeyCode::Escape) {
            self.audio.stop_all_loops();
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::TrackEditor;
            return;
        }

        // [R] resets car back to starting pose
        if is_key_pressed(KeyCode::R) {
            if let Some(state) = &self.editor_state {
                let init_pose = state
                    .track
                    .grid_positions
                    .first()
                    .cloned()
                    .unwrap_or_else(|| {
                        let p = state.track.spline.waypoints.first().map(|w| w.point).unwrap_or(Vec2::ZERO);
                        tdrace_core::track::geometry::SpawnPose::new(p, 0.0, 0)
                    });
                if let Some(car) = &mut self.test_drive_car {
                    *car = Car::new(car.config).with_pose(init_pose.position, init_pose.angle);
                }
                if let Some(tracker) = &mut self.test_drive_tracker {
                    *tracker = TrackProgressTracker::new(state.track.checkpoints.len(), 100);
                }
                self.test_drive_time = 0.0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // Physics step on test drive car
        if let (Some(state), Some(car)) = (&self.editor_state, &mut self.test_drive_car) {
            let kb_ctrl = self.input.poll_player_controls(dt, car.state.local_velocity.x);
            let touch_ctrl = self.touch.poll_controls();
            let player_ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);

            // Sample surface per wheel and step vehicle dynamics
            let surfaces = state.track.sample_car_surfaces(car);
            car.step_per_wheel(&player_ctrl, surfaces, dt);

            // Resolve wall & obstacle collisions
            resolve_all_wall_collisions(car, &state.track.geometry.inner_walls, &state.track.geometry.obstacles);
            resolve_all_wall_collisions(car, &state.track.geometry.outer_walls, &[]);

            // Jump ramps
            for ramp in &state.track.geometry.jump_ramps {
                if car.try_trigger_jump_ramp(ramp) {
                    self.audio.play_sfx(SfxType::JumpLaunch);
                    break;
                }
            }
            if car.state.just_landed {
                self.audio.play_sfx(SfxType::Landing);
                self.camera.add_trauma(0.25);
            }

            // Update lap tracker
            if let Some(tracker) = &mut self.test_drive_tracker {
                tracker.update(car, &state.track.spline, &state.track.checkpoints, dt);
            }
            self.test_drive_time += dt;

            // Audio & Camera
            let max_slip = car.state.wheels.iter().map(|w| w.slip_angle.abs()).fold(0.0f32, f32::max);
            let (rpm, is_shift) = self.engine_rpm.update(car.state.local_velocity.x, player_ctrl.throttle, max_slip, dt);
            self.audio.update_engine_rpm(rpm, player_ctrl.throttle, is_shift);

            self.camera.update(car, dt);
        }
    }

    /// Renders the Track Studio viewport pass.
    pub fn render_track_editor(&self) {
        if let Some(state) = &self.editor_state {
            let sw = screen_width_safe();
            let sh = screen_height_safe();

            // 1. World Pass with EditorCamera
            self.editor_camera.apply(sw, sh);

            // Metric grid in background
            render_editor_grid(&self.editor_camera, sw, sh, state.grid_snap);

            // Render track geometry
            render_track(&state.track);

            // Render barriers and obstacles
            render_barriers_and_obstacles(&state.track);

            // Render interactive gizmos, selection handles, and previews
            render_editor_gizmos(state, &self.editor_tools, &self.editor_camera);

            self.editor_camera.reset_to_screen();
        }
    }

    /// Renders the instant Test Drive view with HUD and return banner.
    pub fn render_editor_test_drive(&self) {
        if let (Some(state), Some(car)) = (&self.editor_state, &self.test_drive_car) {
            self.camera.apply();

            // 1. Render Track
            render_track(&state.track);

            // 2. Persistent Skidmarks
            self.fx.render_ground_fx();

            // 3. Barriers and Obstacles
            render_barriers_and_obstacles(&state.track);

            // 4. Test Car
            let is_braking = car.state.local_velocity.x > 1.0 && car.state.wheels[2].slip_ratio < -0.15;
            let scheme = CarColorScheme::default();
            render_car(car, &scheme, is_braking);

            self.camera.reset_to_screen();

            // Screen pass: Top Test Drive Banner + Speedometer
            let sw = screen_width_safe();
            let sh = screen_height_safe();
            let scaler = UiScaler::new(sw, sh);

            // Top Banner
            let banner_h = scaler.s(36.0);
            macroquad::shapes::draw_rectangle(0.0, 0.0, sw, banner_h, Color::new(0.08, 0.10, 0.15, 0.92));
            macroquad::shapes::draw_rectangle_lines(0.0, 0.0, sw, banner_h, 1.5, Palette::NEON_GREEN);

            let banner_text = "TEST DRIVE MODE — [ESC] Return to Circuit Studio | [R] Reset to Grid";
            self.fonts.draw_ui_bold_centered(
                banner_text,
                sw * 0.5,
                scaler.s(24.0),
                scaler.font_s(14.0),
                Palette::NEON_GREEN,
            );

            // Bottom Right Speedometer Badge
            let spd_kmh = (car.state.local_velocity.x * 3.6).abs().round() as i32;
            let lap_str = if let Some(tracker) = &self.test_drive_tracker {
                format!("Lap {} | {:.2}s", tracker.current_lap, tracker.lap_time)
            } else {
                format!("{:.2}s", self.test_drive_time)
            };

            let badge_w = scaler.s(220.0);
            let badge_h = scaler.s(60.0);
            let bx = sw - badge_w - scaler.s(16.0);
            let by = sh - badge_h - scaler.s(16.0);
            scaler.draw_glass_card(bx, by, badge_w, badge_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.5);

            self.fonts.draw_display(
                &format!("{} KM/H", spd_kmh),
                bx + scaler.s(14.0),
                by + scaler.s(32.0),
                scaler.font_s(22.0),
                Palette::WHITE,
            );
            self.fonts.draw_ui_bold(
                &lap_str,
                bx + scaler.s(14.0),
                by + scaler.s(50.0),
                scaler.font_s(13.0),
                Palette::NEON_GOLD,
            );
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
                &self.fonts,
                &self.track,
                &self.cars,
                &self.color_schemes,
                player_car,
                player_tracker,
                player_pos,
                self.cars.len(),
                self.total_laps,
                self.is_time_attack,
                countdown,
                self.input.gamepad.snapshot.is_connected,
            );

            // F5: Telemetry Panel
            self.input.render_screen_debug(player_car);

            // Mobile Touch Controls Overlay (Virtual Joystick / Buttons + Pedals)
            self.touch.render(&self.fonts, sw, sh);
        }
    }
}
