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
use crate::audio::{AudioManager, EngineSoundType, MusicTrack, SfxType};
use crate::camera::RaceCamera;
use crate::config::GameConfig;
use crate::db::{HallOfFameDb, HallOfFameEntry};
use crate::fx::EffectsManager;
use crate::input::touch::TouchController;
use crate::input::InputController;
pub use crate::module::VehicleVisualType;
use crate::module::{
    F1GameModule, GameModule, KartGameModule, RallyGameModule,
};
use crate::profile::{CountryRegistry, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::car::render_car_with_visual_type;
use crate::render::color::{CarColorScheme, Palette};
use crate::editor::{
    render_editor_grid, render_editor_gizmos, render_editor_ui, EditorAction, EditorCamera,
    EditorModal, EditorState, EditorToolType, SurfaceShapeType, ToolSettings,
};
use crate::render::ghost::{render_ghost_car, GhostRecorder};
use crate::render::{
    render_elevated_barriers_and_obstacles, render_elevated_track,
    render_ground_barriers_and_obstacles, render_ground_track,
};
use crate::replay::{ReplayPlayer, ReplayRecorder};
use crate::tournament::{ChampionshipSession, PointSystem, RoundDriverResult};
use crate::track_manager::TrackManager;
use crate::ui::driver_card::render_driver_cards_screen;
use crate::ui::font::Fonts;
use crate::ui::hall_of_fame::{render_hall_of_fame_screen, PlayerCongrats};
use crate::ui::hud::{format_lap_time, render_hud, PersonalBestNotification};
use crate::ui::menu::{
    render_championship_standings_screen, render_controls_screen, render_exit_confirm_modal,
    render_module_select_menu, render_pause_menu, render_results_screen, render_track_select_menu,
    resolve_predefined_car_for_track, CarChoice, GameMode, RaceResultEntry, TrackChoice,
};
use crate::ui::profile_ui::{render_profile_create_screen, render_profile_manager_screen};
use crate::ui::starting_grid::render_starting_grid_screen;
use crate::ui::track_manager_ui::{
    render_track_manager_screen, ModuleFilter, TrackManagerModal, TrackManagerTab, PROMOTION_MODULES,
};
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
    ModuleSelect {
        selected_idx: usize,
    },
    ChampionshipStandings,
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
    TrackManager {
        active_tab: TrackManagerTab,
        module_filter: ModuleFilter,
        selected_idx: usize,
        modal: TrackManagerModal,
    },
    TrackEditor,
}


/// Qualified participant on the starting grid, ranked by historical best lap, circuit time, or random draw.
#[derive(Debug, Clone, PartialEq)]
pub struct GridParticipant {
    /// True if this slot belongs to the human player.
    pub is_player: bool,
    /// Optional index into `opponent_drivers` if this is an AI bot.
    pub bot_index: Option<usize>,
    /// Driver's display name.
    pub name: String,
    /// Driver's alias or nickname.
    pub alias: String,
    /// Driver's country code if available.
    pub country: Option<String>,
    /// Display title of the vehicle driven.
    pub car_title: String,
    /// Livery color scheme.
    pub color_scheme: CarColorScheme,
    /// Best historical single lap time in seconds on this track.
    pub best_lap: Option<f32>,
    /// Best historical total circuit race time in seconds on this track.
    pub best_circuit_time: Option<f32>,
    /// Pseudo-random tiebreaker hash used when times tie or no data is recorded.
    pub random_seed: u64,
}

impl GridParticipant {
    /// Three-tier comparison for grid order:
    /// 1. Best lap time (ascending, Some < None)
    /// 2. Best circuit time (ascending, Some < None)
    /// 3. Random seed (ascending)
    pub fn cmp_grid_priority(&self, other: &Self) -> std::cmp::Ordering {
        // Tier 1: Best Lap Time
        match (self.best_lap, other.best_lap) {
            (Some(a), Some(b)) => {
                if (a - b).abs() > 1e-4 {
                    return a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal);
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }

        // Tier 2: Best Circuit Time
        match (self.best_circuit_time, other.best_circuit_time) {
            (Some(a), Some(b)) => {
                if (a - b).abs() > 1e-4 {
                    return a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal);
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }

        // Tier 3: Random Seed Tie-Breaker
        self.random_seed.cmp(&other.random_seed)
    }
}

/// Root controller orchestrating track geometry, cars, physics, UI, and audio.
pub struct RaceSession {
    pub state: GameState,
    pub track: Track,
    pub track_choice: TrackChoice,
    pub track_manager: TrackManager,
    pub car_choice: CarChoice,
    pub free_car_selection: bool,
    pub game_mode: GameMode,
    pub is_time_attack: bool,
    pub num_bots: usize,
    pub total_laps: u32,
    pub config: GameConfig,
    pub base_config: GameConfig,

    pub active_module_id: &'static str,
    pub championship_session: Option<ChampionshipSession>,
    pub current_visual_type: VehicleVisualType,

    pub cars: Vec<Car>,
    pub color_schemes: Vec<CarColorScheme>,
    pub trackers: Vec<TrackProgressTracker>,
    pub ai_drivers: Vec<BotAiDriver>,
    pub opponent_drivers: Vec<DriverCharacter>,
    pub grid_participants: Vec<GridParticipant>,
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
    pub show_exit_confirm: bool,

    // Track Editor & Test Drive state
    pub editor_state: Option<EditorState>,
    pub editor_camera: EditorCamera,
    pub editor_tools: ToolSettings,
    pub editor_modal: EditorModal,
    pub editor_save_toast_timer: f32,
    pub editor_save_toast_msg: String,
    pub return_to_editor_on_exit: bool,

    // Audio System
    pub audio: AudioManager,
    pub engine_rpm: EngineRpmModel,
    prev_countdown_sec: i32,
    prev_player_sector: usize,
    curb_sound_cooldown: f32,
    offroad_sound_cooldown: f32,

    // Internal trackers
    pub prev_player_lap: u32,
    pub pb_notification: Option<PersonalBestNotification>,
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
            state: GameState::ModuleSelect { selected_idx: 0 },
            track,
            track_choice,
            track_manager,
            car_choice,
            free_car_selection: false,
            game_mode: GameMode::StandardRace,
            assist_profile,
            is_time_attack: false,
            num_bots: config.gameplay.default_num_bots,
            total_laps: config.gameplay.default_laps,
            config: config.clone(),
            base_config: config,

            active_module_id: "classic",
            championship_session: None,
            current_visual_type: VehicleVisualType::TouringGT {
                widebody: true,
                gt_wing: true,
                diffuser: true,
            },

            cars: Vec::new(),
            color_schemes: Vec::new(),
            trackers: Vec::new(),
            ai_drivers: Vec::new(),
            opponent_drivers: Vec::new(),
            grid_participants: Vec::new(),
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
            show_exit_confirm: false,
            editor_state: None,
            editor_camera,
            editor_tools: ToolSettings::default(),
            editor_modal: EditorModal::None,
            editor_save_toast_timer: 0.0,
            editor_save_toast_msg: String::new(),
            return_to_editor_on_exit: false,
            audio,
            engine_rpm: EngineRpmModel::default(),
            prev_countdown_sec: 4,
            prev_player_sector: 0,
            curb_sound_cooldown: 0.0,
            offroad_sound_cooldown: 0.0,
            prev_player_lap: 1,
            pb_notification: None,
        };

        session.refresh_profiles_and_stats();
        session.refresh_hof_entries();
        session.init_race();
        session.state = GameState::ModuleSelect { selected_idx: 0 }; // Start in Grand Hub module select screen
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
            Some("f1" | "f1_car" | "f1_hybrid_26" | "open_wheel") => CarChoice::F1Car,
            Some("drift_car") => CarChoice::DriftCar,
            Some("kart" | "shifter_kart" | "shifter_kart_125") => CarChoice::Kart,
            Some("rally_car" | "wrc_turbo_rally" | "rally") => CarChoice::RallyCar,
            Some("sports_car") => CarChoice::SportsCar,
            _ => match self.track.module_id.as_deref().unwrap_or(self.active_module_id) {
                "f1" => CarChoice::F1Car,
                "rally" => CarChoice::RallyCar,
                "kart" => CarChoice::Kart,
                _ => CarChoice::SportsCar,
            },
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

    /// Returns available circuits for the active motorsport game module.
    pub fn active_module_tracks(&self) -> Vec<TrackChoice> {
        self.track_manager.module_catalog_tracks(self.active_module_id)
    }

    /// Returns available driver characters for the active motorsport game module.
    pub fn active_module_drivers(&self) -> Vec<DriverCharacter> {
        let effective_mod = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        match effective_mod {
            "f1" => F1GameModule::new().drivers(),
            "rally" => RallyGameModule::new().drivers(),
            "kart" => KartGameModule::new().drivers(),
            _ => DriverCharacter::all().to_vec(),
        }
    }

    /// Returns available vehicles for the active motorsport game module.
    pub fn active_module_vehicles(&self) -> Vec<(&'static str, &'static str, &'static str, (f32, f32, f32, f32))> {
        match self.active_module_id {
            "f1" => vec![
                (CarChoice::F1Car.title(), CarChoice::F1Car.tag(), CarChoice::F1Car.description(), CarChoice::F1Car.stats()),
                ("Monza Low-Drag Aero Spec", "LOW DRAG VMAX", "Reduced wing angle for 354 km/h straight-line top speed.", (1.00, 0.96, 0.90, 0.35)),
            ],
            "rally" => vec![
                (CarChoice::RallyCar.title(), CarChoice::RallyCar.tag(), CarChoice::RallyCar.description(), CarChoice::RallyCar.stats()),
                ("Group B Turbo Monster", "GROUP B LEGEND", "520 BHP 1980s twin-charge monster with flame-spitting anti-lag.", (0.95, 0.98, 0.75, 0.98)),
            ],
            "kart" => vec![
                (CarChoice::Kart.title(), CarChoice::Kart.tag(), CarChoice::Kart.description(), CarChoice::Kart.stats()),
                ("100cc Direct Drive Sprint", "100cc CLUTCHLESS", "High-revving direct-drive kart with ultra sharp throttle response.", (0.65, 0.95, 0.99, 0.45)),
            ],
            _ => vec![
                (CarChoice::SportsCar.title(), CarChoice::SportsCar.tag(), CarChoice::SportsCar.description(), CarChoice::SportsCar.stats()),
                (CarChoice::DriftCar.title(), CarChoice::DriftCar.tag(), CarChoice::DriftCar.description(), CarChoice::DriftCar.stats()),
                (CarChoice::Kart.title(), CarChoice::Kart.tag(), CarChoice::Kart.description(), CarChoice::Kart.stats()),
                (CarChoice::RallyCar.title(), CarChoice::RallyCar.tag(), CarChoice::RallyCar.description(), CarChoice::RallyCar.stats()),
            ],
        }
    }

    /// Loads the track corresponding to a TrackChoice respecting specialized modules.
    pub fn load_track_for_session(&self, choice: &TrackChoice) -> Track {
        self.track_manager.load_track(choice).unwrap_or_else(|_| classic_grand_prix())
    }

    /// Resolves and applies the effective configuration for the given module ID
    /// (hierarchical merging: general base_config merged with in-file [modules.<id>] and external config files).
    pub fn apply_module_config(&mut self, module_id: &'static str) {
        self.active_module_id = module_id;
        self.config = self.base_config.for_module(module_id);

        // Apply audio settings
        self.audio.settings.master_volume = self.config.audio.master_volume;
        self.audio.settings.sfx_volume = self.config.audio.sfx_volume;
        self.audio.settings.music_volume = self.config.audio.music_volume;

        // Apply input filter settings
        self.input.filter.config.steer_rise_rate = self.config.input.steer_rise_rate;
        self.input.filter.config.steer_return_rate = self.config.input.steer_return_rate;
        self.input.filter.config.steer_exponent = self.config.input.steer_exponent;
        self.input.filter.config.speed_sensitive_factor = self.config.input.speed_sensitive_factor;
        self.input.filter.config.min_speed_steer_limit = self.config.input.min_speed_steer_limit;
        self.input.filter.config.throttle_rise_rate = self.config.input.throttle_rise_rate;
        self.input.filter.config.brake_rise_rate = self.config.input.brake_rise_rate;

        // Apply camera settings
        self.camera.position_smoothing = self.config.camera.position_smoothing;
        self.camera.zoom_smoothing = self.config.camera.zoom_smoothing;
        self.camera.velocity_lookahead_time = self.config.camera.velocity_lookahead_time;
        self.camera.trauma_decay = self.config.camera.trauma_decay;
        self.camera.max_shake_offset = self.config.camera.max_shake_offset;
        if !self.config.camera.levels.is_empty() {
            self.camera.levels = self.config.camera.levels.clone();
            self.camera.current_level_idx = self
                .config
                .camera
                .default_level_index
                .min(self.camera.levels.len().saturating_sub(1));
        }

        // Apply gameplay settings
        self.num_bots = self.config.gameplay.default_num_bots;
        self.total_laps = self.config.gameplay.default_laps;
        self.assist_profile = match self.config.gameplay.default_assist_profile.to_lowercase().as_str() {
            "sport" => AssistProfile::Sport,
            "pro" => AssistProfile::Pro,
            _ => AssistProfile::Arcade,
        };
    }

    /// Switches the active motorsport game module (f1, rally, kart, classic).
    pub fn switch_to_module(&mut self, mod_id: &str) {
        match mod_id {
            "f1" => self.switch_to_f1(),
            "rally" => self.switch_to_rally(),
            "kart" => self.switch_to_kart(),
            _ => self.switch_to_classic(),
        }
    }

    /// Activates the Formula 1 Grand Prix module.
    pub fn switch_to_f1(&mut self) {
        self.apply_module_config("f1");
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::OpenWheel {
            front_wing_span: 1.80,
            rear_wing_height: 0.85,
            halo: true,
        };
        let tracks = self.active_module_tracks();
        if let Some((idx, choice)) = tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.track_id() == self.config.gameplay.default_track)
        {
            self.menu_track_idx = idx;
            self.track_choice = choice.clone();
        } else {
            self.track_choice = tracks.first().cloned().unwrap_or_else(|| TrackChoice::Custom {
                id: "monza".to_string(),
                title: "Monza Autodromo Nazionale".to_string(),
                description: "Temple of Speed. 5.79km high-speed DRS straights & Variante del Rettifilo.".to_string(),
                path: "f1/monza".to_string(),
            });
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::F1Car;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 5;
        }
        self.camera.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the World Rally Championship module.
    pub fn switch_to_rally(&mut self) {
        self.apply_module_config("rally");
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::RallyHatch {
            roof_scoop: true,
            mudflaps: true,
            large_wing: true,
        };
        let tracks = self.active_module_tracks();
        if let Some((idx, choice)) = tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.track_id() == self.config.gameplay.default_track)
        {
            self.menu_track_idx = idx;
            self.track_choice = choice.clone();
        } else {
            self.track_choice = tracks.first().cloned().unwrap_or(TrackChoice::OasisRally);
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::RallyCar;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 3;
        }
        self.camera.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Sprint Karting Cup module.
    pub fn switch_to_kart(&mut self) {
        self.apply_module_config("kart");
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::GoKart {
            exposed_driver: true,
            side_bumpers: true,
        };
        let tracks = self.active_module_tracks();
        if let Some((idx, choice)) = tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.track_id() == self.config.gameplay.default_track)
        {
            self.menu_track_idx = idx;
            self.track_choice = choice.clone();
        } else {
            self.track_choice = tracks.first().cloned().unwrap_or_else(|| TrackChoice::Custom {
                id: "lonato".to_string(),
                title: "South Garda Karting (Lonato)".to_string(),
                description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.".to_string(),
                path: "kart/lonato".to_string(),
            });
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::Kart;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 4;
        }
        self.camera.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Classic Arcade Motorsport module.
    pub fn switch_to_classic(&mut self) {
        self.apply_module_config("classic");
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        };
        let tracks = self.active_module_tracks();
        if let Some((idx, choice)) = tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.track_id() == self.config.gameplay.default_track)
        {
            self.menu_track_idx = idx;
            self.track_choice = choice.clone();
        } else {
            self.track_choice = tracks.first().cloned().unwrap_or(TrackChoice::ClassicGrandPrix);
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::SportsCar;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 3;
        }
        self.camera.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Starts a full Formula 1 World Championship Season.
    pub fn start_f1_championship(&mut self) {
        let champ = ChampionshipSession::new(
            "FIA Formula 1 World Championship 2026",
            PointSystem::F1Standard { fastest_lap_bonus: true },
            vec!["monza".to_string(), "spa".to_string(), "silverstone".to_string(), "classic_grand_prix".to_string()],
            5,
            &[
                ("player", "Player", "Apex GP"),
                ("max_hunter", "Max Hunter", "Red Bull"),
                ("charles_laurent", "Charles Laurent", "Ferrari"),
                ("lewis_vance", "Lewis Vance", "Ferrari"),
                ("fernando_toro", "Fernando Toro", "Aston Martin"),
                ("bot_5", "George Speed", "Mercedes-AMG"),
                ("bot_6", "Lando Vance", "McLaren"),
                ("bot_7", "Oscar Rocket", "McLaren"),
            ],
        );
        self.switch_to_f1();
        self.championship_session = Some(champ);
        self.init_race();
    }

    /// Advances to the next round in an active championship season.
    pub fn advance_championship_round(&mut self) {
        if let Some(champ) = &self.championship_session {
            if let Some(track_id) = champ.current_track_id() {
                match track_id {
                    "monza" => self.track = F1GameModule::track_monza(),
                    "spa" => self.track = F1GameModule::track_spa(),
                    "silverstone" => self.track = F1GameModule::track_silverstone(),
                    "monaco" => self.track = F1GameModule::track_monaco(),
                    "suzuka" => self.track = F1GameModule::track_suzuka(),
                    "interlagos" => self.track = F1GameModule::track_interlagos(),
                    "montreal" => self.track = F1GameModule::track_montreal(),
                    "red_bull_ring" => self.track = F1GameModule::track_red_bull_ring(),
                    "catalunya" => self.track = F1GameModule::track_catalunya(),
                    "zandvoort" => self.track = F1GameModule::track_zandvoort(),
                    "bahrain" => self.track = F1GameModule::track_bahrain(),
                    "marina_bay" => self.track = F1GameModule::track_marina_bay(),
                    "cota" => self.track = F1GameModule::track_cota(),
                    _ => self.track = tdrace_core::track::presets::classic_grand_prix(),
                }
                self.init_race();
            } else {
                self.championship_session = None;
                self.state = GameState::Menu;
            }
        } else {
            self.state = GameState::Menu;
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

        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);

        if !self.is_time_attack && total_cars > 1 {
            let target_opponents = total_cars - 1;
            let mut module_opponents: Vec<DriverCharacter> = match effective_module {
                "f1" => F1GameModule::new().drivers(),
                "rally" => RallyGameModule::new().drivers(),
                "kart" => KartGameModule::new().drivers(),
                _ => Vec::new(),
            };

            if module_opponents.is_empty() {
                self.opponent_drivers = DriverCharacter::sample_opponents(target_opponents, seed);
            } else if module_opponents.len() >= target_opponents {
                self.opponent_drivers = module_opponents.into_iter().take(target_opponents).collect();
            } else {
                let remaining = target_opponents - module_opponents.len();
                let sampled_extra = DriverCharacter::sample_opponents(remaining, seed);
                module_opponents.extend(sampled_extra);
                self.opponent_drivers = module_opponents;
            }
        }

        let player_car_choice = self.active_player_car_choice();
        let mut base_config = match player_car_choice {
            CarChoice::F1Car => {
                self.current_visual_type = VehicleVisualType::OpenWheel {
                    front_wing_span: 1.80,
                    rear_wing_height: 0.85,
                    halo: true,
                };
                F1GameModule::car_f1_hybrid()
            }
            CarChoice::RallyCar => {
                self.current_visual_type = VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                };
                RallyGameModule::car_wrc_rally()
            }
            CarChoice::Kart => {
                self.current_visual_type = VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: true,
                };
                KartGameModule::car_shifter_kart()
            }
            CarChoice::DriftCar => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                self.config.get_car_config(player_car_choice)
            }
            CarChoice::SportsCar => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                self.config.get_car_config(player_car_choice)
            }
        };
        base_config.assists = self.assist_profile.to_config();

        let track_id = self.track_choice_id();

        let mut participants = Vec::new();

        // 1. Player participant
        let player_car_title = player_car_choice.title().to_string();
        let player_best_lap = self
            .active_profile_stats
            .best_times
            .get(track_id)
            .copied()
            .or_else(|| {
                self.profile_history
                    .iter()
                    .filter(|r| r.track_id == track_id)
                    .filter_map(|r| r.best_lap)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            });
        let player_best_circuit = self
            .active_profile_stats
            .best_circuit_times
            .get(track_id)
            .copied()
            .or_else(|| {
                self.profile_history
                    .iter()
                    .filter(|r| r.track_id == track_id && r.total_time > 0.0 && !r.is_time_attack)
                    .map(|r| r.total_time)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            });
        let player_seed = seed.wrapping_mul(0x9E3779B97F4A7C15);

        participants.push(GridParticipant {
            is_player: true,
            bot_index: None,
            name: self.active_profile.name.clone(),
            alias: self.active_profile.alias.clone(),
            country: self.active_profile.country.clone(),
            car_title: player_car_title,
            color_scheme: self.active_profile.color_scheme,
            best_lap: player_best_lap,
            best_circuit_time: player_best_circuit,
            random_seed: player_seed,
        });

        // 2. AI Opponent participants
        for (bot_idx, character) in self.opponent_drivers.iter().enumerate() {
            let bot_hof = self
                .hof_entries
                .iter()
                .find(|e| e.player_name.eq_ignore_ascii_case(character.name));
            let bot_best_lap = bot_hof.and_then(|e| e.best_lap);
            let bot_best_circuit = bot_hof.map(|e| e.total_time);
            let bot_seed = seed.wrapping_add((bot_idx as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15));

            let bot_car_choice = match self.game_mode {
                GameMode::ExperimentalRace => player_car_choice,
                _ => {
                    if self.free_car_selection {
                        character.preferred_car
                    } else {
                        self.resolve_predefined_car()
                    }
                }
            };
            let bot_car_title = bot_car_choice.title().to_string();

            participants.push(GridParticipant {
                is_player: false,
                bot_index: Some(bot_idx),
                name: character.name.to_string(),
                alias: character.alias.to_string(),
                country: None,
                car_title: bot_car_title,
                color_scheme: character.color_scheme,
                best_lap: bot_best_lap,
                best_circuit_time: bot_best_circuit,
                random_seed: bot_seed,
            });
        }

        if !self.is_time_attack && total_cars > 1 {
            participants.sort_by(|a, b| a.cmp_grid_priority(b));
        }
        self.grid_participants = participants;

        let player_slot = self
            .grid_participants
            .iter()
            .position(|p| p.is_player)
            .unwrap_or(0);

        let grid_pose_player = self
            .track
            .grid_positions
            .get(player_slot)
            .copied()
            .unwrap_or(SpawnPose {
                position: Vec2::ZERO,
                angle: 0.0,
                grid_slot: player_slot,
            });
        let player_car = Car::new(base_config).with_pose(grid_pose_player.position, grid_pose_player.angle);
        self.cars.push(player_car);
        self.color_schemes.push(self.active_profile.color_scheme);
        self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));

        for (bot_idx, character) in self.opponent_drivers.iter().enumerate() {
            let bot_slot = self
                .grid_participants
                .iter()
                .position(|p| p.bot_index == Some(bot_idx))
                .unwrap_or(bot_idx + 1);

            let grid_pose_bot = self
                .track
                .grid_positions
                .get(bot_slot)
                .copied()
                .unwrap_or(SpawnPose {
                    position: Vec2::ZERO,
                    angle: 0.0,
                    grid_slot: bot_slot,
                });

            let bot_car_choice = match self.game_mode {
                GameMode::ExperimentalRace => player_car_choice,
                _ => {
                    if self.free_car_selection {
                        character.preferred_car
                    } else {
                        self.resolve_predefined_car()
                    }
                }
            };

            let bot_config = match bot_car_choice {
                CarChoice::F1Car => F1GameModule::car_f1_hybrid(),
                CarChoice::RallyCar => RallyGameModule::car_wrc_rally(),
                CarChoice::Kart => KartGameModule::car_shifter_kart(),
                CarChoice::SportsCar | CarChoice::DriftCar => self.config.get_car_config(bot_car_choice),
            };
            let bot_car = Car::new(bot_config).with_pose(grid_pose_bot.position, grid_pose_bot.angle);

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

        // 1. Build selected track (preserve in-memory track if launched from editor)
        if !self.return_to_editor_on_exit {
            self.track = self.load_track_for_session(&self.track_choice);
        }

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
        self.pb_notification = None;

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

        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        let sound_type = match effective_module {
            "f1" => EngineSoundType::F1V6Turbo,
            "rally" => EngineSoundType::RallyTurbo,
            "kart" => EngineSoundType::Kart125cc,
            _ => match active_car {
                CarChoice::F1Car => EngineSoundType::F1V6Turbo,
                CarChoice::Kart => EngineSoundType::Kart125cc,
                CarChoice::RallyCar => EngineSoundType::RallyTurbo,
                CarChoice::SportsCar | CarChoice::DriftCar => EngineSoundType::SportGT,
            },
        };
        self.audio.set_engine_type(sound_type);

        // Show Starting Grid with selected race participants
        self.state = GameState::StartingGrid;
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

        // Toggle audio mute (M key) - only when not typing and not in Track Manager
        let is_typing_or_tm = matches!(
            self.state,
            GameState::ProfileCreate { .. }
                | GameState::TrackManager { .. }
        );
        if !is_typing_or_tm && is_key_pressed(KeyCode::M) {
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

        // Handle camera toggle / zoom cycle (Tab key or Gamepad Left Stick Click) during driving/editor
        let is_camera_state = matches!(
            self.state,
            GameState::Racing
                | GameState::Countdown(_)
                | GameState::Paused
                | GameState::TrackEditor
        );
        if is_camera_state && (is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_cam_toggle_pressed) {
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
                let car_pos = self
                    .cars
                    .first()
                    .map(|c| c.state.position);
                if let Some(pos) = car_pos {
                    let lvl_idx = self.camera.current_level_idx + 1;
                    let total_lvls = self.camera.levels.len();
                    self.fx.drift_popups.spawn_text(
                        pos,
                        &format!("CAMERA: {} ({}/{})", lvl.name.to_uppercase(), lvl_idx, total_lvls),
                        Color::new(0.3, 0.9, 1.0, 1.0),
                    );
                }
            }
        }

        // Camera Progressive Zoom (+ / - keys) during gameplay
        let is_gameplay_state = matches!(
            self.state,
            GameState::Racing
                | GameState::Countdown(_)
                | GameState::StartingGrid
                | GameState::Paused
                | GameState::Finished
        );
        if is_gameplay_state {
            let mut zoom_dir = 0.0f32;
            if is_key_down(KeyCode::Equal) || is_key_down(KeyCode::KpAdd) {
                zoom_dir += 1.0;
            }
            if is_key_down(KeyCode::Minus) || is_key_down(KeyCode::KpSubtract) {
                zoom_dir -= 1.0;
            }

            if zoom_dir != 0.0 {
                let zoom_speed_mult = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    2.0
                } else {
                    1.0
                };
                self.camera.zoom_progressive(zoom_dir, zoom_speed_mult, frame_dt);
            }
        }

        if self.state == GameState::TrackEditor {
            self.update_track_editor(frame_dt);
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

        if matches!(self.state, GameState::TrackManager { .. }) {
            if let GameState::TrackManager {
                active_tab,
                module_filter,
                selected_idx,
                modal,
            } = std::mem::replace(
                &mut self.state,
                GameState::Menu,
            ) {
                self.update_track_manager(active_tab, module_filter, selected_idx, modal, frame_dt);
                return;
            }
        }

        // Open Controls & Driving Assists Screen (C or K key)
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) {
            self.audio.play_sfx(SfxType::UiSelect);
            let from_paused = matches!(self.state, GameState::Racing | GameState::Paused | GameState::Countdown(_));
            self.state = GameState::ControlsHelp(from_paused);
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

        match self.state {
            GameState::Menu => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_menu();
            }
            GameState::ModuleSelect { ref mut selected_idx } => {
                // If exit confirmation modal is currently open:
                if self.show_exit_confirm {
                    // Confirm Exit: Enter, KpEnter, Space, Y, or Gamepad Confirm (A / Start)
                    if is_key_pressed(KeyCode::Enter)
                        || is_key_pressed(KeyCode::KpEnter)
                        || is_key_pressed(KeyCode::Space)
                        || is_key_pressed(KeyCode::Y)
                        || self.input.gamepad.snapshot.btn_confirm_pressed
                        || self.input.gamepad.snapshot.btn_a_pressed
                        || self.input.gamepad.snapshot.btn_start_pressed
                    {
                        std::process::exit(0);
                    }

                    // Cancel / Dismiss Exit Dialog: Escape, N, or Gamepad Cancel (B / Back / Select)
                    if is_key_pressed(KeyCode::Escape)
                        || is_key_pressed(KeyCode::N)
                        || self.input.gamepad.snapshot.btn_cancel_pressed
                        || self.input.gamepad.snapshot.btn_b_pressed
                        || self.input.gamepad.snapshot.btn_back_pressed
                    {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.show_exit_confirm = false;
                    }

                    return;
                }

                let num_modules = 4;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                    self.audio.play_sfx(SfxType::UiMove);
                    if *selected_idx == 0 {
                        *selected_idx = num_modules - 1;
                    } else {
                        *selected_idx -= 1;
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                    self.audio.play_sfx(SfxType::UiMove);
                    *selected_idx = (*selected_idx + 1) % num_modules;
                }
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    match *selected_idx {
                        0 => self.switch_to_classic(),
                        1 => self.switch_to_rally(),
                        2 => self.switch_to_kart(),
                        _ => self.switch_to_f1(),
                    }
                }

                // Profile Manager (P key or Gamepad Y)
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

                // New Profile (N key or Gamepad X)
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

                // Controls Help (C / K key)
                if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::ControlsHelp(false);
                    return;
                }

                // Escape / Gamepad B / Back to trigger exit dialog
                if is_key_pressed(KeyCode::Escape)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.show_exit_confirm = true;
                }
            }
            GameState::ChampionshipStandings => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.advance_championship_round();
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) || self.input.gamepad.snapshot.btn_cancel_pressed || self.input.gamepad.snapshot.btn_b_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.championship_session = None;
                    self.state = GameState::Menu;
                }
            }
            GameState::StartingGrid => {
                // 1. Cycle Game Mode (Tab or Gamepad X)
                if is_key_pressed(KeyCode::Tab)
                    || self.input.gamepad.snapshot.btn_x_pressed
                {
                    self.game_mode = self.game_mode.next();
                    self.is_time_attack = self.game_mode.is_time_attack();
                    self.free_car_selection = self.game_mode.allows_car_change();
                    self.rebuild_roster_participants();
                    self.audio.play_sfx(SfxType::UiSelect);
                }

                // 2. Cycle Selected Car (when free car selection is active / mode allows car change)
                if self.game_mode.allows_car_change() {
                    if is_key_pressed(KeyCode::Left)
                        || is_key_pressed(KeyCode::A)
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
                        || is_key_pressed(KeyCode::D)
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

                // 3. Modify Driver Count (only in race modes with bots)
                if self.game_mode.has_bots() {
                    let max_bots = (self.track.grid_positions.len().saturating_sub(1)).clamp(1, 7);

                    // Increase driver count (Up / Gamepad D-pad Up)
                    if is_key_pressed(KeyCode::Up)
                        || self.input.gamepad.snapshot.dpad_up_pressed
                    {
                        if self.num_bots < max_bots {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.num_bots += 1;
                            self.rebuild_roster_participants();
                        }
                    }

                    // Decrease driver count (Down / Gamepad D-pad Down)
                    if is_key_pressed(KeyCode::Down)
                        || self.input.gamepad.snapshot.dpad_down_pressed
                    {
                        if self.num_bots > 1 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.num_bots -= 1;
                            self.rebuild_roster_participants();
                        }
                    }
                }

                // 4. Launch race countdown (Space, Enter, Gamepad Confirm [A / South / Start])
                if is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Countdown(3.5);
                }

                // 5. View Driver Dossiers (D key or Gamepad Y)
                if is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.btn_y_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
                }

                // 6. Return to Main Menu or Track Editor (Escape, or Gamepad Cancel [B / East / Back])
                if is_key_pressed(KeyCode::Escape)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.state = GameState::TrackEditor;
                    } else {
                        self.state = GameState::Menu;
                        self.audio.play_music(MusicTrack::NeonMenu);
                    }
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

                let (sw, sh) = (screen_width_safe(), screen_height_safe());
                let (_, _, _, _, btn_layout) = crate::ui::pause_menu_layout(sw, sh);
                let (mx, my) = mouse_position_safe();
                let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);

                let resume_clicked = mouse_clicked
                    && mx >= btn_layout.resume_rect.0
                    && mx <= btn_layout.resume_rect.0 + btn_layout.resume_rect.2
                    && my >= btn_layout.resume_rect.1
                    && my <= btn_layout.resume_rect.1 + btn_layout.resume_rect.3;

                let exit_clicked = mouse_clicked
                    && mx >= btn_layout.exit_rect.0
                    && mx <= btn_layout.exit_rect.0 + btn_layout.exit_rect.2
                    && my >= btn_layout.exit_rect.1
                    && my <= btn_layout.exit_rect.1 + btn_layout.exit_rect.3;

                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Pause)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_start_pressed
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                    || resume_clicked
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::Racing;
                } else if is_key_pressed(KeyCode::E)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                    || exit_clicked
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.state = GameState::TrackEditor;
                    } else {
                        self.state = GameState::Menu;
                        self.audio.play_music(MusicTrack::NeonMenu);
                    }
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
                if is_key_pressed(KeyCode::M)
                    || is_key_pressed(KeyCode::Escape)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.state = GameState::TrackEditor;
                    } else {
                        self.state = GameState::Menu;
                        self.audio.play_music(MusicTrack::NeonMenu);
                    }
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
                let drivers = self.active_module_drivers();
                let roster_len = drivers.len().max(1);
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
            | GameState::TrackManager { .. }
            | GameState::TrackEditor => {}

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
                while get_char_pressed().is_some() {}
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
            while get_char_pressed().is_some() {}
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

        // If exit confirmation modal is currently open:
        if self.show_exit_confirm {
            // Confirm Exit: Enter, KpEnter, Space, Y, or Gamepad Confirm (A / Start)
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::KpEnter)
                || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Y)
                || self.input.gamepad.snapshot.btn_confirm_pressed
                || self.input.gamepad.snapshot.btn_a_pressed
                || self.input.gamepad.snapshot.btn_start_pressed
            {
                std::process::exit(0);
            }

            // Cancel / Dismiss Exit Dialog: Escape, N, or Gamepad Cancel (B / Back / Select)
            if is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::N)
                || self.input.gamepad.snapshot.btn_cancel_pressed
                || self.input.gamepad.snapshot.btn_b_pressed
                || self.input.gamepad.snapshot.btn_back_pressed
            {
                self.audio.play_sfx(SfxType::UiSelect);
                self.show_exit_confirm = false;
            }

            return;
        }

        // Return to Grand Hub Module Selection Screen (Escape key or Gamepad B / Cancel / Back / G / Tab)
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::G)
            || is_key_pressed(KeyCode::Tab)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            let cur_mod_idx = match self.active_module_id {
                "classic" => 0,
                "rally" => 1,
                "kart" => 2,
                _ => 3,
            };
            self.state = GameState::ModuleSelect { selected_idx: cur_mod_idx };
            return;
        }

        // Open Controls & Gamepad Screen (C / K key)
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::K) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::ControlsHelp(false);
            return;
        }

        // Quick Championship trigger for Formula 1 (F key)
        if self.active_module_id == "f1" && is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.start_f1_championship();
            return;
        }

        // Open Track Manager (T key)
        if is_key_pressed(KeyCode::T) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::TrackManager {
                active_tab: TrackManagerTab::Main,
                module_filter: ModuleFilter::for_module(self.active_module_id),
                selected_idx: 0,
                modal: TrackManagerModal::None,
            };
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

        let available_tracks = self.active_module_tracks();
        let available_vehicles = self.active_module_vehicles();
        let total_items = available_tracks.len() + 1; // +1 for the dedicated Track Manager entry
        if self.menu_track_idx >= total_items {
            self.menu_track_idx = 0;
        }

        // Track selection cursor (Up/Down: Arrows / W/S / D-pad / Left Stick Y)
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
            self.audio.play_sfx(SfxType::UiMove);
            if self.menu_track_idx == 0 {
                self.menu_track_idx = total_items.saturating_sub(1);
            } else {
                self.menu_track_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_track_idx = (self.menu_track_idx + 1) % total_items;
        }

        // Car selection cursor (Left/Right: Arrows / A/D / D-pad / Left Stick X)
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
            self.audio.play_sfx(SfxType::UiMove);
            if self.menu_car_idx == 0 {
                self.menu_car_idx = available_vehicles.len().saturating_sub(1);
            } else {
                self.menu_car_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_car_idx = (self.menu_car_idx + 1) % available_vehicles.len();
        }

        // Launch Championship / Tournament (F key)
        if is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiSelect);
            match self.active_module_id {
                "f1" => {
                    self.start_f1_championship();
                    return;
                }
                "rally" => {
                    self.init_race();
                    return;
                }
                "kart" => {
                    self.init_race();
                    return;
                }
                _ => {}
            }
        }

        // Toggle Mode (Time Attack vs Race vs AI - X key or Gamepad X)
        if is_key_pressed(KeyCode::X) || self.input.gamepad.snapshot.btn_x_pressed {
            self.is_time_attack = !self.is_time_attack;
            self.audio.play_sfx(SfxType::UiSelect);
        }

        // Cycle Driving Assists Profile (H key)
        if is_key_pressed(KeyCode::H) {
            self.assist_profile = self.assist_profile.next();
            self.audio.play_sfx(SfxType::UiSelect);
        }

        // Cycle Audio Volume (V key)
        if is_key_pressed(KeyCode::V) {
            let mut vol = self.audio.settings.master_volume + 0.25;
            if vol > 1.05 {
                vol = 0.0;
            }
            self.audio.settings.master_volume = vol;
            self.audio.settings.sfx_volume = vol;
            self.audio.settings.music_volume = vol;
            self.audio.play_sfx(SfxType::UiSelect);
        }

        // Quick Track Editor Launcher (E key)
        if is_key_pressed(KeyCode::E) {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.menu_track_idx < available_tracks.len() {
                let chosen = available_tracks[self.menu_track_idx].clone();
                let file_path = match &chosen {
                    TrackChoice::Custom { path, .. } => Some(path.clone()),
                    preset => {
                        let candidate = self.track_manager.track_path_for_slug(preset.track_id());
                        if candidate.exists() {
                            Some(candidate.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    }
                };
                let track = self.load_track_for_session(&chosen);
                self.enter_track_editor_with_path(track, file_path);
            } else {
                // If cursor is on the Track Manager entry, open Track Manager
                self.state = GameState::TrackManager {
                    active_tab: TrackManagerTab::Main,
                    module_filter: ModuleFilter::for_module(self.active_module_id),
                    selected_idx: 0,
                    modal: TrackManagerModal::None,
                };
            }
            return;
        }

        // Start race or open Track Manager (Space, Enter, or Gamepad Confirm [A / South / Start])
        if is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.menu_track_idx < available_tracks.len() {
                self.track_choice = available_tracks[self.menu_track_idx].clone();
                self.car_choice = match self.active_module_id {
                    "f1" => CarChoice::F1Car,
                    "rally" => CarChoice::RallyCar,
                    "kart" => CarChoice::Kart,
                    _ => CarChoice::ALL[self.menu_car_idx.min(CarChoice::ALL.len() - 1)],
                };
                self.init_race();
            } else {
                // User pressed Confirm on the dedicated "Track Manager" entry!
                self.state = GameState::TrackManager {
                    active_tab: TrackManagerTab::Main,
                    module_filter: ModuleFilter::for_module(self.active_module_id),
                    selected_idx: 0,
                    modal: TrackManagerModal::None,
                };
            }
        }
    }

    /// Handles input and actions for the dedicated Track Manager screen.
    fn update_track_manager(
        &mut self,
        mut active_tab: TrackManagerTab,
        mut module_filter: ModuleFilter,
        mut selected_idx: usize,
        mut modal: TrackManagerModal,
        dt: f32,
    ) {
        if !matches!(modal, TrackManagerModal::EditMetadata { .. }) {
            while get_char_pressed().is_some() {}
        }

        match modal {
            TrackManagerModal::EditMetadata {
                ref track_id,
                ref mut name_input,
                ref mut desc_input,
                ref mut active_field,
                ref mut cursor_timer,
            } => {
                *cursor_timer += dt;

                while let Some(c) = get_char_pressed() {
                    if !c.is_control() {
                        if *active_field == 0 {
                            if name_input.len() < 32 {
                                name_input.push(c);
                            }
                        } else if desc_input.len() < 140 {
                            desc_input.push(c);
                        }
                    }
                }

                if is_key_pressed(KeyCode::Backspace) {
                    if *active_field == 0 {
                        name_input.pop();
                    } else {
                        desc_input.pop();
                    }
                }

                if is_key_pressed(KeyCode::Tab)
                    || is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Down)
                {
                    *active_field = 1 - *active_field;
                    *cursor_timer = 0.0;
                    self.audio.play_sfx(SfxType::UiMove);
                }

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                    let tid = track_id.clone();
                    let new_n = name_input.trim().to_string();
                    let new_d = desc_input.trim().to_string();
                    if !new_n.is_empty() {
                        let _ = self.track_manager.update_track_metadata(&tid, new_n, new_d);
                    }
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                self.state = GameState::TrackManager {
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal,
                };
                return;
            }
            TrackManagerModal::ConfirmDelete {
                ref track_id,
                ..
            } => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Y)
                    || is_key_pressed(KeyCode::Backspace)
                    || is_key_pressed(KeyCode::Delete)
                {
                    let tid = track_id.clone();
                    let _ = self.track_manager.delete_custom_track(&tid);
                    self.audio.play_sfx(SfxType::UiSelect);
                    let list_len = match active_tab {
                        TrackManagerTab::Main => self.track_manager.filtered_main_track_choices(module_filter).len(),
                        TrackManagerTab::Drafts => self.track_manager.draft_track_choices().len(),
                    };
                    if selected_idx >= list_len && list_len > 0 {
                        selected_idx = list_len - 1;
                    }
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::N) {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                self.state = GameState::TrackManager {
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal,
                };
                return;
            }
            TrackManagerModal::SelectModulePromotion {
                ref track_id,
                track_title,
                mut cursor_idx,
                mut selected_mask,
            } => {
                if is_key_pressed(KeyCode::Key1) {
                    selected_mask[0] = !selected_mask[0];
                    cursor_idx = 0;
                    self.audio.play_sfx(SfxType::UiMove);
                } else if is_key_pressed(KeyCode::Key2) {
                    selected_mask[1] = !selected_mask[1];
                    cursor_idx = 1;
                    self.audio.play_sfx(SfxType::UiMove);
                } else if is_key_pressed(KeyCode::Key3) {
                    selected_mask[2] = !selected_mask[2];
                    cursor_idx = 2;
                    self.audio.play_sfx(SfxType::UiMove);
                } else if is_key_pressed(KeyCode::Key4) {
                    selected_mask[3] = !selected_mask[3];
                    cursor_idx = 3;
                    self.audio.play_sfx(SfxType::UiMove);
                } else if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::W)
                    || is_key_pressed(KeyCode::Left)
                    || self.input.gamepad.snapshot.nav_up
                    || self.input.gamepad.snapshot.nav_left
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    cursor_idx = cursor_idx.saturating_sub(1);
                } else if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::S)
                    || is_key_pressed(KeyCode::Right)
                    || self.input.gamepad.snapshot.nav_down
                    || self.input.gamepad.snapshot.nav_right
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    if cursor_idx + 1 < PROMOTION_MODULES.len() {
                        cursor_idx += 1;
                    }
                }

                if is_key_pressed(KeyCode::Space)
                    || self.input.gamepad.snapshot.btn_x_pressed
                    || self.input.gamepad.snapshot.btn_y_pressed
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    selected_mask[cursor_idx] = !selected_mask[cursor_idx];
                }

                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    let mut target_mods: Vec<&str> = PROMOTION_MODULES
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| selected_mask[*i])
                        .map(|(_, (mod_id, _, _, _))| *mod_id)
                        .collect();
                    if target_mods.is_empty() {
                        let (cur_mod, _, _, _) = PROMOTION_MODULES[cursor_idx.min(PROMOTION_MODULES.len() - 1)];
                        target_mods.push(cur_mod);
                    }
                    let _ = self.track_manager.promote_track_to_modules(track_id, &target_mods);
                    self.audio.play_sfx(SfxType::UiSelect);
                    let new_len = self.track_manager.draft_track_choices().len();
                    if selected_idx >= new_len && new_len > 0 {
                        selected_idx = new_len - 1;
                    }
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                if is_key_pressed(KeyCode::Escape) || self.input.gamepad.snapshot.btn_back_pressed || self.input.gamepad.snapshot.btn_b_pressed {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }

                self.state = GameState::TrackManager {
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal: TrackManagerModal::SelectModulePromotion {
                        track_id: track_id.clone(),
                        track_title,
                        cursor_idx,
                        selected_mask,
                    },
                };
                return;
            }
            TrackManagerModal::None => {}
        }

        // --- NON-MODAL NAVIGATION AND ACTIONS ---
        // 1. Switch Tabs between Promoted tracks and Drafts (Tab key, Key 1 / 2)
        if is_key_pressed(KeyCode::Key1) {
            if active_tab != TrackManagerTab::Main {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key2) {
            if active_tab != TrackManagerTab::Drafts {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Drafts;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            self.audio.play_sfx(SfxType::UiMove);
            active_tab = match active_tab {
                TrackManagerTab::Main => TrackManagerTab::Drafts,
                TrackManagerTab::Drafts => TrackManagerTab::Main,
            };
            selected_idx = 0;
        }

        // 2. Switch Module Filter on Main/Promoted Tab (Left / Right arrows, A / D, M key, Gamepad Left/Right)
        if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::A)
            || self.input.gamepad.snapshot.nav_left
        {
            if active_tab == TrackManagerTab::Main {
                self.audio.play_sfx(SfxType::UiMove);
                module_filter = module_filter.prev();
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || self.input.gamepad.snapshot.nav_right
        {
            if active_tab == TrackManagerTab::Main {
                self.audio.play_sfx(SfxType::UiMove);
                module_filter = module_filter.next();
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::M) && active_tab == TrackManagerTab::Main {
            self.audio.play_sfx(SfxType::UiMove);
            module_filter = module_filter.next();
            selected_idx = 0;
        }

        // Re-evaluate list after potential tab/filter switch
        let current_list = match active_tab {
            TrackManagerTab::Main => self.track_manager.filtered_main_track_choices(module_filter),
            TrackManagerTab::Drafts => self.track_manager.draft_track_choices(),
        };
        let list_len = current_list.len();

        if list_len > 0 && selected_idx >= list_len {
            selected_idx = list_len.saturating_sub(1);
        }

        // 3. Up/Down Track Selection
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
            self.audio.play_sfx(SfxType::UiMove);
            if selected_idx == 0 {
                selected_idx = list_len.saturating_sub(1);
            } else {
                selected_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
            self.audio.play_sfx(SfxType::UiMove);
            if list_len > 0 {
                selected_idx = (selected_idx + 1) % list_len;
            }
        }

        // 4. Race Track (Enter / Space / Gamepad A / Confirm)
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            if let Some(track_choice) = current_list.get(selected_idx).cloned() {
                self.audio.play_sfx(SfxType::UiSelect);
                self.track_choice = track_choice;
                self.init_race();
                return;
            }
        }

        // 5. Edit in Track Editor (E key / Gamepad X)
        if is_key_pressed(KeyCode::E) || self.input.gamepad.snapshot.btn_x_pressed {
            if let Some(track_choice) = current_list.get(selected_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                self.track_choice = track_choice.clone();
                let file_path = match track_choice {
                    TrackChoice::Custom { path, .. } => Some(path.clone()),
                    preset => {
                        let candidate = self.track_manager.track_path_for_slug(preset.track_id());
                        if candidate.exists() {
                            Some(candidate.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    }
                };
                let track = self
                    .track_manager
                    .load_track(track_choice)
                    .unwrap_or_else(|_| classic_grand_prix());
                self.track = track.clone();
                self.enter_track_editor_with_path(track, file_path);
                return;
            }
        }

        // 6. Promote / Demote Track (P key / Gamepad Y)
        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            if let Some(track_choice) = current_list.get(selected_idx) {
                let tid = track_choice.track_id().to_string();
                match active_tab {
                    TrackManagerTab::Drafts => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        let default_mod_idx = match self.active_module_id {
                            "classic" => 0,
                            "rally" => 1,
                            "kart" => 2,
                            _ => 3,
                        };
                        let mut selected_mask = [false; 4];
                        selected_mask[default_mod_idx] = true;
                        self.state = GameState::TrackManager {
                            active_tab,
                            module_filter,
                            selected_idx,
                            modal: TrackManagerModal::SelectModulePromotion {
                                track_id: tid,
                                track_title: track_choice.title().to_string(),
                                cursor_idx: default_mod_idx,
                                selected_mask,
                            },
                        };
                        return;
                    }
                    TrackManagerTab::Main => {
                        let _ = self.track_manager.demote_track(&tid);
                        self.audio.play_sfx(SfxType::UiSelect);
                        let new_len = self.track_manager.filtered_main_track_choices(module_filter).len();
                        if selected_idx >= new_len && new_len > 0 {
                            selected_idx = new_len - 1;
                        }
                    }
                }
            }
        }

        // 7. Create New Draft Track (N key)
        if is_key_pressed(KeyCode::N) {
            self.audio.play_sfx(SfxType::UiSelect);
            let count = self.track_manager.draft_track_choices().len() + 1;
            let name = format!("Draft Track {}", count);
            let desc = "Experimental draft circuit layout under testing.".to_string();
            let _ = self.track_manager.create_new_draft_track(&name, &desc);
            active_tab = TrackManagerTab::Drafts;
            selected_idx = self.track_manager.draft_track_choices().len().saturating_sub(1);
        }

        // 8. Edit Metadata (I key)
        if is_key_pressed(KeyCode::I) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                while get_char_pressed().is_some() {}
                self.state = GameState::TrackManager {
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal: TrackManagerModal::EditMetadata {
                        track_id: track_choice.track_id().to_string(),
                        name_input: track_choice.title().to_string(),
                        desc_input: track_choice.description().to_string(),
                        active_field: 0,
                        cursor_timer: 0.0,
                    },
                };
                return;
            }
        }

        // 9. Delete Track (Delete / Backspace / X key)
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::X) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                self.state = GameState::TrackManager {
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal: TrackManagerModal::ConfirmDelete {
                        track_id: track_choice.track_id().to_string(),
                        track_title: track_choice.title().to_string(),
                    },
                };
                return;
            }
        }

        // 10. Back to Main Menu (Escape / Gamepad Back)
        if is_key_pressed(KeyCode::Escape) || self.input.gamepad.snapshot.btn_back_pressed {
            self.audio.play_sfx(SfxType::UiMove);
            self.state = GameState::Menu;
            return;
        }

        self.state = GameState::TrackManager {
            active_tab,
            module_filter,
            selected_idx,
            modal,
        };
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

        // 3. Step individual vehicle dynamics and update road elevation & cross-slope banking
        for i in 0..n_cars {
            let prev_prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            let proj = self.track.spline.project_point_continuity(self.cars[i].state.position, prev_prog, 50.0);
            self.cars[i].state.road_elevation = proj.elevation;
            self.cars[i].state.road_bank_angle = proj.bank_angle;
            self.cars[i].state.track_right = Vec2::new(proj.tangent.y, -proj.tangent.x);

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
                    if !player_car.state.is_airborne
                        && player_car.state.elevation <= 0.0
                        && player_car.state.speed > 3.0
                        && self.session_time.fract() < dt * 4.0
                    {
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

        // Dynamic Accelerating Engine Audio (motor sound only)
        if let Some(player_car) = self.cars.first() {
            let max_slip_angle = player_car.state.wheels.iter().map(|w| w.slip_angle.abs()).fold(0.0f32, f32::max);
            let max_slip_ratio = player_car.state.wheels.iter().map(|w| w.slip_ratio.abs()).fold(0.0f32, f32::max);
            let slip_intensity = (max_slip_angle * 1.5).max(max_slip_ratio);

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

            // 7b. Check and notify Personal Best lap achievement
            if lap_changed {
                if let Some(last_lap_time) = tracker.last_lap_time {
                    let track_id = self.track_choice_id().to_string();
                    let prev_best = self.active_profile_stats.best_times.get(&track_id).copied();
                    let is_pb = prev_best.map_or(true, |best| last_lap_time < best);

                    if is_pb {
                        let delta = prev_best.map(|prev| prev - last_lap_time);
                        self.active_profile_stats.best_times.insert(track_id, last_lap_time);

                        let completed_lap = tracker.current_lap.saturating_sub(1).max(1);
                        self.pb_notification = Some(PersonalBestNotification {
                            completed_lap,
                            lap_time: last_lap_time,
                            delta,
                            timer: 3.5,
                            duration: 3.5,
                        });

                        if let Some(player_car) = self.cars.first() {
                            let popup_msg = if let Some(d) = delta {
                                format!("PERSONAL BEST! {} (-{:.2}s)", format_lap_time(last_lap_time), d)
                            } else {
                                format!("PERSONAL BEST! {}", format_lap_time(last_lap_time))
                            };
                            self.fx.drift_popups.spawn_text(
                                player_car.state.position,
                                &popup_msg,
                                Palette::NEON_GOLD,
                            );
                        }
                    }
                }
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

        // 10. Update active Personal Best notification timer
        if let Some(notif) = &mut self.pb_notification {
            notif.timer -= dt;
            if notif.timer <= 0.0 {
                self.pb_notification = None;
            }
        }
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

            // If an active championship season is underway, submit round results and show standings
            if let Some(champ) = &mut self.championship_session {
                let mut round_results = Vec::new();
                for (pos, res) in self.results.iter().enumerate() {
                    let driver_id = if res.is_player {
                        "player".to_string()
                    } else if let Some(character) = self.opponent_drivers.get(pos.saturating_sub(1)) {
                        character.id.to_string()
                    } else {
                        format!("bot_{}", pos)
                    };
                    round_results.push(RoundDriverResult {
                        driver_id,
                        driver_name: res.car_name.clone(),
                        team_name: "Motorsport Team".to_string(),
                        finish_position: pos + 1,
                        total_time: res.total_time,
                        best_lap: res.best_lap,
                        points_awarded: 0,
                        has_fastest_lap: pos == 0,
                    });
                }
                champ.submit_round_results(&self.track.name, round_results);
                self.state = GameState::ChampionshipStandings;
                return;
            }

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

    /// Returns the active backdrop clear color based on the current track or editor state.
    pub fn active_backdrop_color(&self) -> Color {
        match self.state {
            GameState::TrackEditor => {
                if let Some(ref editor) = self.editor_state {
                    crate::render::get_track_backdrop_color(editor.track.default_surface)
                } else {
                    crate::render::get_track_backdrop_color(self.track.default_surface)
                }
            }
            _ => {
                crate::render::get_track_backdrop_color(self.track.default_surface)
            }
        }
    }

    /// Renders current UI state, HUD, or pause screen.
    pub fn render(&mut self) {
        match self.state {
            GameState::Menu => {
                let available_tracks = self.active_module_tracks();
                let (mod_title, mod_sub, mod_accent) = match self.active_module_id {
                    "f1" => ("FORMULA 1 GRAND PRIX", "FIA Hybrid Turbo Championship", Palette::RED),
                    "rally" => ("WORLD RALLY CHAMPIONSHIP", "WRC AWD Dirt & Gravel Stages", Palette::NEON_GOLD),
                    "kart" => ("SPRINT KARTING CUP", "125cc Direct Steering Shifter Karts", Palette::NEON_GREEN),
                    _ => ("TDRACE ARCADE RACING", "Modern Cross-Platform 2D Motorsport Simulation & Visuals", Palette::NEON_GOLD),
                };
                render_track_select_menu(
                    &self.fonts,
                    self.active_module_id,
                    mod_title,
                    mod_sub,
                    mod_accent,
                    &available_tracks,
                    self.menu_track_idx,
                    &self.active_profile,
                    &self.active_profile_stats,
                );
                if self.show_exit_confirm {
                    render_exit_confirm_modal(&self.fonts);
                }
            }
            GameState::ModuleSelect { selected_idx } => {
                let modules_data = [
                    ("classic", "Classic Arcade Motorsport", "ALL-IN-ONE ARCADE & STUDIO", "GT Coupe, Drift Spec, Shifter Kart, Rally Car & CAD Circuit Studio Workshop.", Palette::NEON_CYAN),
                    ("rally", "World Rally Championship", "WRC LOOSE SURFACE AWD", "AWD stage time trials and Scandinavian flicks on desert sand & mountain passes.", Palette::NEON_GOLD),
                    ("kart", "Sprint Karting Cup", "125CC SHIFTER KARTS", "Direct 1:1 steering, 3.5G cornering bites, and elimination tournament heats.", Palette::NEON_GREEN),
                    ("f1", "Formula 1 Grand Prix", "FIA WORLD CHAMPIONSHIP", "High-downforce 1050 BHP hybrid open-wheelers on Monza, Spa, and Silverstone.", Palette::RED),
                ];
                render_module_select_menu(
                    &self.fonts,
                    selected_idx,
                    &modules_data,
                    &self.active_profile,
                    &self.active_profile_stats,
                );
                if self.show_exit_confirm {
                    render_exit_confirm_modal(&self.fonts);
                }
            }
            GameState::ChampionshipStandings => {
                if let Some(champ) = &self.championship_session {
                    render_championship_standings_screen(&self.fonts, champ);
                } else {
                    self.state = GameState::Menu;
                }
            }
            GameState::StartingGrid => {
                self.render_world();
                let predefined_car = self.resolve_predefined_car();
                let max_grid_size = self.track.grid_positions.len().min(8);
                let active_car = self.active_player_car_choice();
                let best_lap = self
                    .grid_participants
                    .iter()
                    .find(|p| p.is_player)
                    .and_then(|p| p.best_lap)
                    .or_else(|| self.ghost_recorder.best_ghost_lap.as_ref().map(|g| g.lap_time));
                render_starting_grid_screen(
                    &self.fonts,
                    &self.track,
                    &self.active_profile,
                    self.game_mode,
                    active_car,
                    predefined_car,
                    &self.grid_participants,
                    self.total_laps,
                    best_lap,
                    self.cars.len(),
                    max_grid_size,
                    self.input.gamepad.snapshot.is_connected,
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
                let drivers = self.active_module_drivers();
                render_driver_cards_screen(&self.fonts, &drivers, self.driver_cards_idx);
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
            GameState::TrackManager {
                active_tab,
                module_filter,
                selected_idx,
                ref modal,
            } => {
                render_track_manager_screen(
                    &self.fonts,
                    &self.track_manager,
                    active_tab,
                    module_filter,
                    selected_idx,
                    modal,
                );
            }
            GameState::TrackEditor => {
                self.render_track_editor();
            }
        }
    }

    /// Transitions cleanly into the in-game Track Studio editor with specified circuit.
    pub fn enter_track_editor(&mut self, track: Track) {
        self.enter_track_editor_with_path(track, None);
    }

    /// Transitions into Track Studio editor with specified circuit and track source file path.
    pub fn enter_track_editor_with_path(&mut self, track: Track, file_path: Option<String>) {
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
        let mut state = EditorState::new(track);
        state.current_file_path = file_path;
        self.editor_state = Some(state);
        self.editor_tools = ToolSettings::default();
        self.editor_modal = EditorModal::None;
        self.state = GameState::TrackEditor;
    }

    /// Launches a Time Trial race session from the Track Studio editor using the circuit's default car.
    pub fn start_editor_test_drive(&mut self) {
        if let Some(state) = &mut self.editor_state {
            state.rebuild_geometry();
            self.track = state.track.clone();
            let default_car = resolve_predefined_car_for_track(Some(&self.track), self.active_module_id);
            self.car_choice = default_car;
            self.game_mode = GameMode::TimeTrial;
            self.is_time_attack = true;
            self.free_car_selection = false;
            self.return_to_editor_on_exit = true;
            self.init_race();
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
                    let is_multi = is_key_down(KeyCode::LeftShift)
                        || is_key_down(KeyCode::RightShift)
                        || is_key_down(KeyCode::LeftControl)
                        || is_key_down(KeyCode::RightControl)
                        || is_key_down(KeyCode::LeftSuper)
                        || is_key_down(KeyCode::RightSuper);
                    self.editor_tools.handle_mouse_down_with_mods(state, world_mouse, is_multi);
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

        if (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
            && is_key_pressed(KeyCode::Z)
        {
            if let Some(state) = &mut self.editor_state {
                state.undo();
            }
        }
        if (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
            && is_key_pressed(KeyCode::Y)
        {
            if let Some(state) = &mut self.editor_state {
                state.redo();
            }
        }

        if (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
            && is_key_pressed(KeyCode::A)
        {
            if self.editor_modal == EditorModal::None {
                if let Some(state) = &mut self.editor_state {
                    if self.editor_tools.select_all_for_active_tool(state) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }
        }

        if (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
            && is_key_pressed(KeyCode::D)
        {
            if let Some(state) = &mut self.editor_state {
                if self.editor_tools.duplicate_selected(state) {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
        }

        if (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
            && is_key_pressed(KeyCode::S)
        {
            if self.editor_modal == EditorModal::None {
                while get_char_pressed().is_some() {}
                if let Some(state) = &self.editor_state {
                    let is_existing = state.current_file_path.is_some();
                    let initial_filename = if let Some(ref p) = state.current_file_path {
                        std::path::Path::new(p)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string()
                    } else {
                        TrackManager::sanitize_slug(&state.track.name)
                    };
                    self.editor_modal = EditorModal::SaveAs {
                        input_name: state.track.name.clone(),
                        input_filename: initial_filename,
                        input_description: state.track.description.clone(),
                        active_field: 0,
                        overwrite: is_existing,
                        custom_filename_edited: is_existing,
                        exit_on_save: false,
                    };
                }
            }
        }

        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            if let Some(state) = &mut self.editor_state {
                if self.editor_tools.delete_selected(state) {
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            if !self.editor_tools.active_polygon_vertices.is_empty() {
                self.editor_tools.active_polygon_vertices.clear();
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // Surface Zone Layer & Shape Shortcuts (with Ctrl/Cmd modifier)
        let ctrl_down = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);

        if ctrl_down && is_key_pressed(KeyCode::F) {
            if let Some(state) = &mut self.editor_state {
                if self.editor_tools.bring_selected_surface_front(state) {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
        }

        if ctrl_down && is_key_pressed(KeyCode::B) {
            if let Some(state) = &mut self.editor_state {
                if self.editor_tools.send_selected_surface_back(state) {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
        }

        if !ctrl_down && is_key_pressed(KeyCode::B) {
            if let Some(state) = &mut self.editor_state {
                let shift_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                let has_waypoints = !state.selection.selected_waypoint_indices().is_empty();
                if has_waypoints {
                    if shift_down {
                        if self.editor_tools.batch_invert_banking(state) {
                            self.audio.play_sfx(SfxType::UiSelect);
                        }
                    } else if self.editor_tools.cycle_selected_banking(state) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                } else if !self.editor_tools.send_selected_surface_back(state) {
                    let curr = self.editor_tools.new_waypoint_bank_angle;
                    self.editor_tools.new_waypoint_bank_angle = if curr.abs() < 1.0 {
                        10.0
                    } else if (curr - 10.0).abs() < 2.0 {
                        18.0
                    } else if (curr - 18.0).abs() < 2.0 {
                        22.0
                    } else {
                        0.0
                    };
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
        }

        if ctrl_down && is_key_pressed(KeyCode::C) {
            if self.editor_tools.active_tool == EditorToolType::SurfaceZone {
                self.editor_tools.active_surface_shape = SurfaceShapeType::Circle;
                self.audio.play_sfx(SfxType::UiSelect);
            }
        }

        if ctrl_down && is_key_pressed(KeyCode::T) {
            if self.editor_tools.active_tool == EditorToolType::SurfaceZone {
                self.editor_tools.active_surface_shape = SurfaceShapeType::Triangle;
                self.editor_tools.active_polygon_vertices.clear();
                self.audio.play_sfx(SfxType::UiSelect);
            }
        }

        if ctrl_down && is_key_pressed(KeyCode::P) {
            if self.editor_tools.active_tool == EditorToolType::SurfaceZone {
                self.editor_tools.active_surface_shape = SurfaceShapeType::Polygon;
                self.editor_tools.active_polygon_vertices.clear();
                self.audio.play_sfx(SfxType::UiSelect);
            }
        }

        if is_key_pressed(KeyCode::Space) || (!ctrl_down && is_key_pressed(KeyCode::P) && self.editor_tools.active_tool != EditorToolType::SurfaceZone) {
            if self.editor_modal == EditorModal::None {
                self.start_editor_test_drive();
                return;
            }
        }

        if !ctrl_down && is_key_pressed(KeyCode::F) {
            if let Some(state) = &mut self.editor_state {
                // If a surface zone is selected, plain F toggles/brings it to front; otherwise focus camera
                if !self.editor_tools.bring_selected_surface_front(state) {
                    let mut min = Vec2::splat(f32::MAX);
                    let mut max = Vec2::splat(f32::MIN);
                    for wp in &state.track.spline.waypoints {
                        min = min.min(wp.point);
                        max = max.max(wp.point);
                    }
                    if min.x <= max.x {
                        self.editor_camera.focus_bounds(min, max, sw, sh);
                    }
                } else {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
        }

        // Waypoint Banking & Jump Ramp Rotation Shortcuts: [ / ], R / Shift+R
        if self.editor_modal == EditorModal::None {
            let shift_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            if is_key_pressed(KeyCode::R) {
                let delta = if shift_down {
                    -std::f32::consts::PI / 12.0
                } else {
                    std::f32::consts::PI / 12.0
                };
                if let Some(state) = &mut self.editor_state {
                    if self.editor_tools.rotate_selected_jump_ramp(state, delta) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }

            if is_key_pressed(KeyCode::LeftBracket) {
                if let Some(state) = &mut self.editor_state {
                    let has_waypoints = !state.selection.selected_waypoint_indices().is_empty();
                    if has_waypoints {
                        let delta = if shift_down { -5.0 } else { -1.0 };
                        if self.editor_tools.batch_adjust_banking(state, delta) {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    } else if self.editor_tools.rotate_selected_jump_ramp(state, -std::f32::consts::PI / 12.0) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }

            if is_key_pressed(KeyCode::RightBracket) {
                if let Some(state) = &mut self.editor_state {
                    let has_waypoints = !state.selection.selected_waypoint_indices().is_empty();
                    if has_waypoints {
                        let delta = if shift_down { 5.0 } else { 1.0 };
                        if self.editor_tools.batch_adjust_banking(state, delta) {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    } else if self.editor_tools.rotate_selected_jump_ramp(state, std::f32::consts::PI / 12.0) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }

            if is_key_pressed(KeyCode::Comma) {
                if let Some(state) = &mut self.editor_state {
                    if self.editor_tools.rotate_selected_jump_ramp(state, -std::f32::consts::PI / 36.0) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }

            if is_key_pressed(KeyCode::Period) {
                if let Some(state) = &mut self.editor_state {
                    if self.editor_tools.rotate_selected_jump_ramp(state, std::f32::consts::PI / 36.0) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::G) {
            if let Some(state) = &mut self.editor_state {
                state.grid_snap = state.grid_snap.next();
            }
        }

        // Drain unconsumed characters when no text modal is open in the editor
        if !matches!(self.editor_modal, EditorModal::SaveAs { .. } | EditorModal::SetRampAngle { .. }) {
            while get_char_pressed().is_some() {}
        }

        // Arrow Keys (and WASD / Gamepad) Camera Navigation
        if self.editor_modal == EditorModal::None {
            let ctrl_down = is_key_down(KeyCode::LeftControl)
                || is_key_down(KeyCode::RightControl)
                || is_key_down(KeyCode::LeftSuper)
                || is_key_down(KeyCode::RightSuper);

            let mut pan_dir = Vec2::ZERO;
            if is_key_down(KeyCode::Up) || (!ctrl_down && is_key_down(KeyCode::W)) {
                pan_dir.y += 1.0;
            }
            if is_key_down(KeyCode::Down) || (!ctrl_down && is_key_down(KeyCode::S)) {
                pan_dir.y -= 1.0;
            }
            if is_key_down(KeyCode::Left) || (!ctrl_down && is_key_down(KeyCode::A)) {
                pan_dir.x -= 1.0;
            }
            if is_key_down(KeyCode::Right) || (!ctrl_down && is_key_down(KeyCode::D)) {
                pan_dir.x += 1.0;
            }

            // Gamepad navigation support
            let gp = &self.input.gamepad.snapshot;
            if gp.is_connected {
                if gp.nav_up || gp.dpad_up_pressed { pan_dir.y += 1.0; }
                if gp.nav_down || gp.dpad_down_pressed { pan_dir.y -= 1.0; }
                if gp.nav_left || gp.dpad_left_pressed { pan_dir.x -= 1.0; }
                if gp.nav_right || gp.dpad_right_pressed { pan_dir.x += 1.0; }
                if gp.steer.abs() > 0.15 { pan_dir.x += gp.steer; }
                if gp.throttle > 0.15 { pan_dir.y += gp.throttle; }
                if gp.brake > 0.15 { pan_dir.y -= gp.brake; }
            }

            if pan_dir.length_squared() > 0.0 {
                let speed_mult = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    2.5
                } else {
                    1.0
                };
                self.editor_camera.pan_direction(pan_dir, speed_mult, dt);
            }

            // Progressive Zoom (+ / - keys)
            let mut zoom_dir = 0.0f32;
            if is_key_down(KeyCode::Equal) || is_key_down(KeyCode::KpAdd) {
                zoom_dir += 1.0;
            }
            if is_key_down(KeyCode::Minus) || is_key_down(KeyCode::KpSubtract) {
                zoom_dir -= 1.0;
            }

            if zoom_dir != 0.0 {
                let zoom_speed_mult = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    2.0
                } else {
                    1.0
                };
                let zoom_center = if mouse_pos.x >= 0.0 && mouse_pos.x <= sw && mouse_pos.y >= 0.0 && mouse_pos.y <= sh {
                    mouse_pos
                } else {
                    Vec2::new(sw * 0.5, sh * 0.5)
                };
                self.editor_camera.zoom_progressive(zoom_center, zoom_dir, zoom_speed_mult, dt, sw, sh);
            }
        }

        if self.editor_save_toast_timer > 0.0 {
            self.editor_save_toast_timer = (self.editor_save_toast_timer - dt).max(0.0);
        }
    }

    /// Dispatches action events triggered from Editor UI interactions.
    pub fn handle_editor_action(&mut self, action: EditorAction) {
        match action {
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
                        blank.module_id = Some(self.active_module_id.to_string());
                        blank.modules = vec![self.active_module_id.to_string()];
                        blank.default_surface = match self.active_module_id {
                            "rally" => tdrace_core::physics::surface::SurfaceType::Dirt,
                            _ => tdrace_core::physics::surface::SurfaceType::Grass,
                        };
                        blank.predefined_car = match self.active_module_id {
                            "f1" => Some("f1_car".to_string()),
                            "kart" => Some("kart".to_string()),
                            "rally" => Some("rally_car".to_string()),
                            _ => Some("sports_car".to_string()),
                        };
                        blank
                    }
                };
                self.enter_track_editor_with_path(track, None);
            }
            EditorAction::OpenTrack(choice) => {
                if let Ok(track) = self.track_manager.load_track(&choice) {
                    let file_path = match &choice {
                        TrackChoice::Custom { path, .. } => Some(path.clone()),
                        preset => {
                            let candidate = self.track_manager.track_path_for_slug(preset.track_id());
                            if candidate.exists() {
                                Some(candidate.to_string_lossy().to_string())
                            } else {
                                None
                            }
                        }
                    };
                    self.enter_track_editor_with_path(track, file_path);
                }
            }
            EditorAction::SaveTrack { name, filename, description, overwrite, exit_after } => {
                if let Some(state) = &mut self.editor_state {
                    state.track.name = name;
                    state.track.description = description;
                    state.rebuild_geometry();

                    let target_slug = if overwrite {
                        if let Some(ref p) = state.current_file_path {
                            std::path::Path::new(p)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                        } else if !filename.trim().is_empty() {
                            Some(TrackManager::sanitize_slug(&filename))
                        } else {
                            Some(TrackManager::sanitize_slug(&state.track.name))
                        }
                    } else if !filename.trim().is_empty() {
                        Some(TrackManager::sanitize_slug(&filename))
                    } else {
                        Some(TrackManager::sanitize_slug(&state.track.name))
                    };

                    let result = self.track_manager.save_custom_track_with_options(
                        &state.track,
                        target_slug.as_deref(),
                        overwrite,
                    );
                    match result {
                        Ok(path) => {
                            state.current_file_path = Some(path.clone());
                            state.is_dirty = false;
                            self.editor_save_toast_timer = 2.5;
                            if overwrite {
                                self.editor_save_toast_msg = format!("Track overwritten: {}", path);
                            } else {
                                self.editor_save_toast_msg = format!("Track saved: {}", path);
                            }
                            self.audio.play_sfx(SfxType::UiSelect);

                            // Rescan custom tracks
                            let _ = self.track_manager.scan_custom_tracks();

                            if exit_after {
                                self.state = GameState::Menu;
                            }
                        }
                        Err(err) => {
                            self.editor_save_toast_timer = 3.5;
                            self.editor_save_toast_msg = format!("Save error: {}", err);
                        }
                    }
                }
            }
            EditorAction::DeleteTrack(id) => {
                let _ = self.track_manager.delete_custom_track(&id);
                self.audio.play_sfx(SfxType::UiSelect);
            }
            _ => {}
        }
    }



    /// Renders the Track Studio viewport pass.
    pub fn render_track_editor(&mut self) {
        if let Some(state) = &mut self.editor_state {
            let sw = screen_width_safe();
            let sh = screen_height_safe();

            // 1. World Pass with EditorCamera
            self.editor_camera.apply(sw, sh);

            // Metric grid in background
            render_editor_grid(&self.editor_camera, sw, sh, state.grid_snap);

            // Render ground track & barriers
            render_ground_track(&state.track);
            render_ground_barriers_and_obstacles(&state.track);

            // Render elevated overpass bridges & barriers
            render_elevated_track(&state.track);
            render_elevated_barriers_and_obstacles(&state.track);

            // Render interactive gizmos, selection handles, and previews
            render_editor_gizmos(state, &self.editor_tools, &self.editor_camera);

            self.editor_camera.reset_to_screen();

            // 2. Screen Pass: Render Editor UI (toolbars, palettes, inspector, status bar, and modals) ON TOP of the track!
            let dispatched = render_editor_ui(
                &self.fonts,
                state,
                &mut self.editor_tools,
                &mut self.editor_camera,
                &mut self.track_manager,
                &mut self.editor_modal,
            );

            // 3. Render floating Save Confirmation Toast if active
            if self.editor_save_toast_timer > 0.0 {
                let scaler = UiScaler::new(sw, sh);
                let toast_w = scaler.s(360.0);
                let toast_h = scaler.s(36.0);
                let toast_x = (sw - toast_w) * 0.5;
                let toast_y = scaler.s(52.0);

                let alpha = (self.editor_save_toast_timer / 0.4).min(1.0);
                macroquad::shapes::draw_rectangle(
                    toast_x,
                    toast_y,
                    toast_w,
                    toast_h,
                    Color::new(0.04, 0.16, 0.10, 0.95 * alpha),
                );
                macroquad::shapes::draw_rectangle_lines(
                    toast_x,
                    toast_y,
                    toast_w,
                    toast_h,
                    1.5,
                    Color::new(0.25, 0.95, 0.45, 0.98 * alpha),
                );
                self.fonts.draw_ui_bold(
                    &self.editor_save_toast_msg,
                    toast_x + scaler.s(16.0),
                    toast_y + scaler.s(22.0),
                    scaler.font_s(13.0),
                    Color::new(0.3, 0.98, 0.55, alpha),
                );
            }

            self.handle_editor_action(dispatched);
        }
    }



    /// Renders world-space entities under active camera with strict elevation occlusion layering.
    fn render_world(&self) {
        self.camera.apply();

        // 1. Ground Track & Environment (elevation < 0.6m)
        render_ground_track(&self.track);

        // 2. Persistent Ground Skidmarks
        self.fx.render_ground_fx();

        // 3. Ground Barriers & Obstacles (elevation < 0.6m)
        render_ground_barriers_and_obstacles(&self.track);

        // Separate cars into ground and elevated groups
        let mut ground_cars = Vec::new();
        let mut elevated_cars = Vec::new();
        for i in 0..self.cars.len() {
            if self.cars[i].total_elevation() < 0.6 {
                ground_cars.push(i);
            } else {
                elevated_cars.push(i);
            }
        }
        ground_cars.sort_by(|&a, &b| {
            self.cars[a]
                .total_elevation()
                .partial_cmp(&self.cars[b].total_elevation())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        elevated_cars.sort_by(|&a, &b| {
            self.cars[a]
                .total_elevation()
                .partial_cmp(&self.cars[b].total_elevation())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4. Ground-Level Vehicles
        for &i in &ground_cars {
            let car = &self.cars[i];
            let scheme = &self.color_schemes[i];
            let is_braking =
                car.state.local_velocity.x > 1.0 && car.state.wheels[2].slip_ratio < -0.15;
            render_car_with_visual_type(car, scheme, is_braking, self.current_visual_type);
        }

        // 5. Ghost Vehicle (Semi-transparent during Time Trial)
        if self.game_mode.has_ghost() {
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

        // 6. Elevated Overpass Bridges (solid opaque concrete deck + drop shadow + ribbon)
        render_elevated_track(&self.track);

        // 7. Elevated Bridge Barriers & Guardrails (drawn on top of the bridge deck, touching the track)
        render_elevated_barriers_and_obstacles(&self.track);

        // 8. Elevated Vehicles (drawn on top of the bridge deck)
        for &i in &elevated_cars {
            let car = &self.cars[i];
            let scheme = &self.color_schemes[i];
            let is_braking =
                car.state.local_velocity.x > 1.0 && car.state.wheels[2].slip_ratio < -0.15;
            render_car_with_visual_type(car, scheme, is_braking, self.current_visual_type);
        }

        // 9. Airborne Particles (Smoke, Dirt roost, Sparks, Drift text)
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
                self.pb_notification.as_ref(),
            );

            // F5: Telemetry Panel
            self.input.render_screen_debug(player_car);

            // Mobile Touch Controls Overlay (Virtual Joystick / Buttons + Pedals)
            self.touch.render(&self.fonts, sw, sh);
        }
    }
}
