use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::KeyCode;
use macroquad::prelude::{get_frame_time, screen_height, screen_width};
use serde::{Deserialize, Serialize};

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
    std::panic::catch_unwind(screen_width).unwrap_or(1920.0)
}

#[inline]
fn screen_height_safe() -> f32 {
    std::panic::catch_unwind(screen_height).unwrap_or(1080.0)
}
use tdrace_core::collision::car_collision::resolve_multi_car_collisions;
use tdrace_core::collision::wall::resolve_all_wall_collisions;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::AssistProfile;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::{JumpRampCarExt, SpawnPose};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::{Track, TrackCategory};

use crate::ai::{BotAiDriver, CareerRivalEntry, DriverCharacter, DriverPersonalityOffsets, DriverTier, DrivingStyle};
use crate::audio::{AudioManager, EngineSoundType, MusicTrack, SfxType};
use crate::camera::{RaceCamera, SplitLayout, ZoomLevelConfig};
use crate::config::GameConfig;
use crate::db::{HallOfFameDb, HallOfFameEntry};
use crate::fx::EffectsManager;
use crate::input::touch::TouchController;
use crate::input::{DigitalInputFilter, InputController, NavGrid2D};
pub use crate::module::VehicleVisualType;
use crate::module::{
    ClassicGameModule, ExtremeOffRoadModule, GameModule, GtWorldChallengeModule, KartGameModule,
    NascarGameModule, RallyGameModule,
};
use crate::profile::{
    ChampionshipAward, CountryRegistry, ModuleCareerProgress, PlayerProfile, ProfileCareerStats,
    RaceHistoryEntry,
};
use crate::render::car::render_car_with_visual_type_model_and_shadows;
use crate::render::color::{CarColorScheme, Palette};
use crate::editor::{
    is_mouse_over_editor_ui, render_editor_grid, render_editor_gizmos, render_editor_ui,
    EditorAction, EditorCamera, EditorModal, EditorState, EditorToolType, SurfaceShapeType,
    ToolSettings,
};
use crate::render::ghost::{render_ghost_car, GhostRecorder};
use crate::render::{
    compute_adaptive_alpha, render_elevated_barriers_and_obstacles,
    render_elevated_barriers_and_obstacles_culled, render_elevated_track,
    render_elevated_track_culled, render_floating_bot_nameplates,
    render_grandstand_shadows_culled, render_grandstands_culled,
    render_ground_barriers_and_obstacles, render_ground_barriers_and_obstacles_culled,
    render_ground_track, render_ground_track_culled, render_player_ground_aura,
    render_player_overhead_chevron, render_player_roof_beacon, render_tree_canopies_culled,
    render_tree_shadows_culled, render_tree_trunks_culled, NAMEPLATE_OUTER_RADIUS,
    PlayerVisibilityOptions, VehicleNameplateItem,
};
use crate::replay::{ReplayPlayer, ReplayRecorder};
use crate::series::format::{ChampionshipDefinition, SeriesDefinition};
use crate::series::{
    ChampionshipManager, ChampionshipSession, PointSystem, RoundDriverResult,
};
use crate::track_manager::TrackManager;
use crate::ui::series_editor::{
    handle_championship_editor_input, render_championship_editor, ChampionshipEditorAction,
    ChampionshipEditorState,
};
use crate::ui::driver_card::render_driver_cards_screen;
use crate::ui::font::Fonts;
use crate::ui::hall_of_fame::{render_hall_of_fame_screen, PlayerCongrats};
use crate::ui::race_stats::render_race_stats_screen;
use crate::ui::hud::{format_lap_time, render_hud, render_split_hud, PersonalBestNotification, VisibilityToast};
use crate::ui::menu::{
    render_championship_standings_screen, render_controls_screen, render_exit_confirm_modal,
    render_modality_select_screen, render_module_select_menu, render_pause_menu,
    render_results_screen, render_track_select_menu, resolve_predefined_car_for_track,
    resolve_track_for_menu, CarChoice, GameMode, MenuPanelFocus, ModalityCategory, ModalityItem,
    ModalityModal, RaceResultEntry, TrackCatalogFilter, TrackChoice,
};
use crate::ui::profile_ui::{
    render_player_roster_manager_screen, render_profile_create_screen, render_profile_manager_screen,
    ProfileFocusArea,
};
use crate::ui::starting_grid::render_starting_grid_screen;
pub use crate::ui::starting_grid::StartingGridFocus;
use crate::ui::track_manager_ui::{
    render_track_manager_screen, ModuleFilter, TrackManagerModal, TrackManagerTab, PROMOTION_MODULES,
};
use crate::ui::{
    confirm_modal_layout,
    render_curve_indicator, ArcadeSettingsModal, CabinetContext, CabinetScreen, CabinetTheme,
    CareerHubFocus, CircuitViewerOrigin, CircuitViewerState, ScreenAction, UiScaler, UniversalConfirmModal,
};
pub use cabinet::fx::crt::{CrtConfig, CrtOverlay, ScanlineMode};
pub use cabinet::fx::floating_text::{FloatingTextItem, FloatingTextManager};
pub use cabinet::fx::transition::{ScreenTransition, TransitionConfig, TransitionPhase, TransitionType};

/// Source screen that launched the DriverCards dossier view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriverCardsOrigin {
    Menu,
    StartingGrid,
    Paused,
}

/// Source screen that launched the Garage showroom view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageOrigin {
    ModalitySelect,
    Menu,
    StartingGrid,
    CareerHub,
}

/// Source screen that launched the Profile Manager view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProfileOrigin {
    #[default]
    Menu,
    ModuleSelect,
    ModalitySelect,
    StartingGrid,
}

/// Source screen that launched the Track Studio editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorOrigin {
    #[default]
    TrackManager,
    ModalitySelect,
    Menu,
}

/// Source screen that launched the Track Selection Menu / Circuit Explorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuOrigin {
    #[default]
    ModalitySelect,
    StartingGrid,
}

/// High-level game flow state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Menu,
    ModuleSelect {
        selected_idx: usize,
    },
    ModalitySelect {
        category: ModalityCategory,
        selected_idx: usize,
        modal: Option<ModalityModal>,
    },
    CareerHub {
        selected_tier: u32,
        selected_slot: usize,
        calendar_tracks: Vec<String>,
        showing_standings: bool,
    },
    CareerSelect {
        selected_idx: usize,
    },
    Garage(GarageOrigin),
    CircuitViewer(CircuitViewerOrigin),
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
    PlayerRosterManager {
        selected_idx: usize,
        active_column: usize,
        field_idx: usize,
        input_name: String,
        input_alias: String,
        country_idx: usize,
        livery_idx: usize,
        assist_mode: AssistProfile,
        cursor_timer: f32,
        status_msg: Option<String>,
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
    ChampionshipEditor,
}

/// Active view within the post-race Finished state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FinishedScreenView {
    #[default]
    Results,
    HallOfFame,
    Statistics,
}

/// Telemetry metrics for a single completed lap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LapTelemetry {
    pub lap_number: u32,
    pub lap_time: f32,
    pub sector_times: Vec<f32>,
    pub is_personal_best: bool,
}

/// Acrobatic stunt performance metrics accumulated during a race.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AcrobaticStats {
    pub total_drift_points: u32,
    pub drift_count: u32,
    pub max_single_drift_score: f32,
    pub total_air_time: f32,
    pub jump_count: u32,
    pub longest_jump_time: f32,
    pub jump_points: u32,
    pub max_combo: u32,
    pub total_stunt_score: u32,
}

/// Comprehensive telemetry summary for the human player in a race session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerRaceTelemetry {
    pub laps: Vec<LapTelemetry>,
    pub best_lap_idx: Option<usize>,
    pub top_speed_mps: f32,
    pub stunt_stats: AcrobaticStats,
    pub collision_count: u32,
}

/// Detailed breakdown of XP awarded after completing a race.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct XpAwardReceipt {
    pub per_lap_xp: u64,
    pub completed_laps: u32,
    pub lap_xp: u64,
    pub completion_bonus: u64,
    pub first_time_bonus: u64,
    pub total_xp: u64,
    pub new_balance: u64,
    pub is_first_time: bool,
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
    /// Vehicle model chosen for this participant.
    pub car_choice: CarChoice,
    /// Livery color scheme.
    pub color_scheme: CarColorScheme,
    /// Model ID for authentic vehicle sprite and specification if elected.
    pub model_id: Option<&'static str>,
    /// Best historical single lap time in seconds on this track.
    pub best_lap: Option<f32>,
    /// Best historical total circuit race time in seconds on this track.
    pub best_circuit_time: Option<f32>,
    /// Pseudo-random tiebreaker hash used when times tie or no data is recorded.
    pub random_seed: u64,
    /// Dynamic skill tier of this driver (from difficulty bell curve or series configuration).
    pub driver_tier: Option<DriverTier>,
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

/// User preference memory for a specific casual race modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModalityPreference {
    pub racer_count: usize,
    pub difficulty: DriverTier,
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
    pub pending_championship_results: Option<Vec<RoundDriverResult>>,
    pub championship_manager: ChampionshipManager,
    pub championship_editor_state: Option<ChampionshipEditorState>,
    pub random_car_assignment: bool,
    pub roster_seed: u64,
    pub current_visual_type: VehicleVisualType,
    pub car_visual_types: Vec<VehicleVisualType>,
    pub selected_car_model_id: Option<&'static str>,
    pub car_model_ids: Vec<Option<&'static str>>,

    pub cars: Vec<Car>,
    pub color_schemes: Vec<CarColorScheme>,
    pub trackers: Vec<TrackProgressTracker>,
    pub ai_drivers: Vec<BotAiDriver>,
    pub opponent_drivers: Vec<DriverCharacter>,
    pub opponent_tiers: Vec<DriverTier>,
    pub casual_ai_difficulty: DriverTier,
    pub modality_preferences: std::collections::HashMap<GameMode, ModalityPreference>,
    pub prev_num_bots: usize,
    pub prev_casual_ai_difficulty: DriverTier,
    pub grid_participants: Vec<GridParticipant>,
    pub driver_cards_idx: usize,

    // Interactive Garage Showroom state
    pub garage_origin: GarageOrigin,
    pub garage_tier: u8,
    pub garage_car_idx: usize,
    pub garage_view_mode: crate::ui::GarageViewMode,
    pub garage_turntable_angle: f32,
    pub garage_brake_heat: f32,
    pub garage_rev_rpm: f32,
    pub garage_revving: bool,
    pub garage_gallery_mode: bool,
    pub garage_gallery_filter: usize,
    pub garage_gallery_sel: usize,
    pub in_garage_state: bool,

    // Active Player Profile & Career History
    pub profile_origin: ProfileOrigin,
    pub active_profile: PlayerProfile,
    pub active_profile_stats: ProfileCareerStats,
    pub active_career_progress: ModuleCareerProgress,
    pub profile_list: Vec<PlayerProfile>,
    pub profile_history: Vec<RaceHistoryEntry>,
    pub profile_module_progress: std::collections::HashMap<String, ModuleCareerProgress>,
    pub profile_manager_tab: usize,
    pub profile_telemetry_filter_idx: usize,
    pub profile_focus_card: bool,
    pub profile_champ_scroll: usize,
    pub profile_champ_selected_idx: usize,
    pub profile_focus_area: ProfileFocusArea,
    pub career_hub_focus: CareerHubFocus,
    pub profile_awards: Vec<ChampionshipAward>,
    pub profile_cabinet_disc_idx: usize,
    pub profile_cabinet_tier_idx: usize,

    pub fx: EffectsManager,
    pub camera: RaceCamera,
    pub camera_p2: RaceCamera,
    pub split_layout: SplitLayout,
    pub filter_p2: DigitalInputFilter,
    pub input: InputController,
    pub touch: TouchController,
    pub fonts: Fonts,
    pub visibility_options: PlayerVisibilityOptions,
    pub visibility_toast: Option<VisibilityToast>,

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
    pub finished_view: FinishedScreenView,
    pub finished_prev_view: FinishedScreenView,
    pub player_race_stats: PlayerRaceTelemetry,
    pub last_xp_receipt: Option<XpAwardReceipt>,

    // Menu selection cursor & 2D navigation state
    pub menu_origin: MenuOrigin,
    pub menu_focused_panel: MenuPanelFocus,
    pub menu_track_filter: TrackCatalogFilter,
    pub menu_track_idx: usize,
    pub menu_car_idx: usize,
    pub starting_grid_focus: StartingGridFocus,
    pub starting_grid_card_idx: usize,
    pub starting_grid_roster_idx: usize,
    pub pause_nav: NavGrid2D,
    pub pause_selected_btn: usize,
    pub assist_profile: AssistProfile,
    pub assist_profile_p2: AssistProfile,
    pub show_exit_confirm: bool,
    pub exit_confirm_modal: Option<UniversalConfirmModal>,
    pub settings_modal: Option<ArcadeSettingsModal>,
    pub circuit_viewer_state: Option<CircuitViewerState>,

    // Track Editor & Test Drive state
    pub editor_state: Option<EditorState>,
    pub editor_camera: EditorCamera,
    pub editor_tools: ToolSettings,
    pub editor_modal: EditorModal,
    pub editor_save_toast_timer: f32,
    pub editor_save_toast_msg: String,
    pub return_to_editor_on_exit: bool,
    pub editor_return_track_manager: Option<(TrackManagerTab, ModuleFilter, usize)>,
    pub editor_origin: EditorOrigin,

    // Audio System
    pub audio: AudioManager,
    pub engine_rpm: EngineRpmModel,
    pub engine_rpm_p2: EngineRpmModel,
    prev_countdown_sec: i32,
    pub prev_player_sector: usize,
    pub prev_p2_sector: usize,
    curb_sound_cooldown: f32,
    offroad_sound_cooldown: f32,

    // Internal trackers
    pub prev_player_lap: u32,
    pub prev_p2_lap: u32,
    pub pb_notification: Option<PersonalBestNotification>,

    // Screen transitions (Cabinet FX)
    pub transition: Option<ScreenTransition>,
    pub pending_state: Option<GameState>,

    // CRT & Retro Scanline post-processing overlay (Cabinet FX)
    pub crt_overlay: CrtOverlay,

    // Floating Text Popups for HUD & Alerts (Cabinet FX)
    pub floating_text: FloatingTextManager,
    pub prev_best_sectors: Vec<Option<f32>>,
    pub drift_combo_count: u32,
    pub drift_combo_timer: f32,
    pub prev_player_drifting: bool,
    pub stunt_scoring_override: Option<bool>,
    pub player_collision_stunt_lockout: f32,
    pub player2_collision_stunt_lockout: f32,
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
            _ => TrackChoice::ClassicGrandPrix,
        };
        let car_choice = match config.gameplay.default_car.as_str() {
            "drift_car" => CarChoice::DriftCar,
            "kart" => CarChoice::Kart,
            "rally_car" => CarChoice::RallyCar,
            _ => CarChoice::SportsCar,
        };
        let assist_profile = AssistProfile::Arcade;
        let assist_profile_p2 = AssistProfile::Arcade;

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

        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let camera = RaceCamera::from_config_with_viewport(&config.camera, sw, sh);
        let camera_p2 = RaceCamera::from_config_with_viewport(&config.camera, sw, sh);
        let editor_camera = EditorCamera::from_config_with_viewport(&config.camera, sw, sh);
        let crt_overlay = config.display.to_crt_overlay();
        let default_num_bots = config.gameplay.default_num_bots;
        let bot_nameplates_pref = config.display.bot_nameplates;
        crate::render::track::set_surface_texture_quality(config.display.surface_texture_quality);

        let mut session = Self {
            state: GameState::ModuleSelect { selected_idx: 0 },
            track,
            track_choice,
            track_manager,
            car_choice,
            free_car_selection: false,
            game_mode: GameMode::StandardRace,
            assist_profile,
            assist_profile_p2,
            is_time_attack: false,
            num_bots: default_num_bots,
            total_laps: config.gameplay.default_laps,
            config: config.clone(),
            base_config: config,

            active_module_id: "classic",
            championship_session: None,
            pending_championship_results: None,
            championship_manager: ChampionshipManager::new(),
            championship_editor_state: None,
            random_car_assignment: true,
            roster_seed: 42,
            current_visual_type: VehicleVisualType::TouringGT {
                widebody: true,
                gt_wing: true,
                diffuser: true,
            },
            car_visual_types: Vec::new(),
            selected_car_model_id: None,
            car_model_ids: Vec::new(),

            cars: Vec::new(),
            color_schemes: Vec::new(),
            trackers: Vec::new(),
            ai_drivers: Vec::new(),
            opponent_drivers: Vec::new(),
            opponent_tiers: Vec::new(),
            casual_ai_difficulty: DriverTier::Rookie,
            modality_preferences: std::collections::HashMap::new(),
            prev_num_bots: default_num_bots,
            prev_casual_ai_difficulty: DriverTier::Rookie,
            grid_participants: Vec::new(),
            driver_cards_idx: 0,

            garage_origin: GarageOrigin::ModalitySelect,
            garage_tier: 1,
            garage_car_idx: 0,
            garage_view_mode: crate::ui::GarageViewMode::Lateral,
            garage_turntable_angle: 0.0,
            garage_brake_heat: 0.0,
            garage_rev_rpm: 0.0,
            garage_revving: false,
            garage_gallery_mode: false,
            garage_gallery_filter: 0,
            garage_gallery_sel: 0,
            in_garage_state: false,

            profile_origin: ProfileOrigin::Menu,
            active_profile: PlayerProfile::default(),
            active_profile_stats: ProfileCareerStats::default(),
            active_career_progress: ModuleCareerProgress::default_for_gt(1),
            profile_list: Vec::new(),
            profile_history: Vec::new(),
            profile_module_progress: std::collections::HashMap::new(),
            profile_manager_tab: 0,
            profile_telemetry_filter_idx: 0,
            profile_focus_card: false,
            profile_champ_scroll: 0,
            profile_champ_selected_idx: 0,
            profile_focus_area: ProfileFocusArea::Tabs,
            career_hub_focus: CareerHubFocus::Tabs,
            profile_awards: Vec::new(),
            profile_cabinet_disc_idx: 0,
            profile_cabinet_tier_idx: 0,

            fx: EffectsManager::new_persistent(1500),
            camera,
            camera_p2,
            split_layout: SplitLayout::Vertical,
            filter_p2: DigitalInputFilter::default(),
            input,
            touch: TouchController::new(),
            fonts: Fonts::load_embedded(),
            visibility_options: {
                let mut opts = PlayerVisibilityOptions::default();
                opts.bot_nameplates = bot_nameplates_pref;
                opts
            },
            visibility_toast: None,

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
            show_hall_of_fame: false,
            finished_view: FinishedScreenView::Results,
            finished_prev_view: FinishedScreenView::Results,
            player_race_stats: PlayerRaceTelemetry::default(),
            last_xp_receipt: None,

            menu_origin: MenuOrigin::ModalitySelect,
            menu_focused_panel: MenuPanelFocus::LeftTracks,
            menu_track_filter: TrackCatalogFilter::Presets,
            menu_track_idx: 0,
            menu_car_idx: 0,
            starting_grid_focus: StartingGridFocus::LeftSetup,
            starting_grid_card_idx: 0,
            starting_grid_roster_idx: 0,
            pause_nav: NavGrid2D::new(vec![1, 1]),
            pause_selected_btn: 0,
            show_exit_confirm: false,
            exit_confirm_modal: None,
            settings_modal: None,
            circuit_viewer_state: None,
            editor_state: None,
            editor_camera,
            editor_tools: ToolSettings::default(),
            editor_modal: EditorModal::None,
            editor_save_toast_timer: 0.0,
            editor_save_toast_msg: String::new(),
            return_to_editor_on_exit: false,
            editor_return_track_manager: None,
            editor_origin: EditorOrigin::TrackManager,
            audio,
            engine_rpm: EngineRpmModel::default(),
            engine_rpm_p2: EngineRpmModel::default(),
            prev_countdown_sec: 4,
            prev_player_sector: 0,
            prev_p2_sector: 0,
            curb_sound_cooldown: 0.0,
            offroad_sound_cooldown: 0.0,
            prev_player_lap: 1,
            prev_p2_lap: 1,
            pb_notification: None,
            transition: None,
            pending_state: None,
            crt_overlay,
            floating_text: FloatingTextManager::new(64),
            prev_best_sectors: Vec::new(),
            drift_combo_count: 0,
            drift_combo_timer: 0.0,
            prev_player_drifting: false,
            stunt_scoring_override: None,
            player_collision_stunt_lockout: 0.0,
            player2_collision_stunt_lockout: 0.0,
        };

        session.refresh_profiles_and_stats();
        session.refresh_hof_entries();
        session.init_race();
        session.state = GameState::ModuleSelect { selected_idx: 0 }; // Start in Grand Hub module select screen
        session
    }

    /// Whether acrobatic stunt scoring and floating HUD stunt alerts are active.
    ///
    /// Enabled for the Classic Arcade module (`"classic"`), and disabled for
    /// realistic motorsport disciplines (`"gt"`, `"nascar"`, `"rally"`, `"kart"`, etc.),
    /// unless an explicit manual override has been set.
    #[inline]
    pub fn is_stunt_scoring_enabled(&self) -> bool {
        if let Some(manual) = self.stunt_scoring_override {
            return manual;
        }
        self.active_module_id == "classic"
    }

    /// Sets or clears the manual override for acrobatic stunt scoring.
    #[inline]
    pub fn set_stunt_scoring_enabled(&mut self, enabled: Option<bool>) {
        self.stunt_scoring_override = enabled;
    }

    /// Whether vehicle collisions are intended to be scored as stunts (e.g. Demolition Derby / Car Crush).
    ///
    /// Currently returns `false` for standard circuit and arcade racing, ensuring vehicle
    /// collisions penalize and void acrobatic stunts rather than awarding drift points.
    #[inline]
    pub fn is_demolition_scoring_enabled(&self) -> bool {
        false
    }

    /// Whether the active game session is in 2-Player Split Screen mode.
    #[inline]
    pub fn is_split_screen(&self) -> bool {
        self.game_mode.is_split_screen()
    }

    /// Cycles to the next camera zoom level for player 1 (and synchronizes player 2 if in split-screen mode).
    pub fn cycle_camera_zoom(&mut self) -> ZoomLevelConfig {
        let lvl = self.camera.cycle_zoom_level();
        if self.is_split_screen() {
            self.camera_p2.set_zoom_level(self.camera.current_level_idx);
        }
        lvl
    }

    /// Zooms in one level closer for player 1 (and synchronizes player 2 if in split-screen mode).
    pub fn zoom_in(&mut self) -> Option<ZoomLevelConfig> {
        let lvl = self.camera.zoom_in();
        if self.is_split_screen() {
            self.camera_p2.set_zoom_level(self.camera.current_level_idx);
        }
        lvl
    }

    /// Zooms out one level farther for player 1 (and synchronizes player 2 if in split-screen mode).
    pub fn zoom_out(&mut self) -> Option<ZoomLevelConfig> {
        let lvl = self.camera.zoom_out();
        if self.is_split_screen() {
            self.camera_p2.set_zoom_level(self.camera.current_level_idx);
        }
        lvl
    }

    /// Applies progressive zoom adjustment to active cameras (both players in split-screen mode).
    pub fn zoom_progressive(&mut self, zoom_dir: f32, speed_multiplier: f32, dt: f32) {
        self.camera.zoom_progressive(zoom_dir, speed_multiplier, dt);
        if self.is_split_screen() {
            self.camera_p2.zoom_progressive(zoom_dir, speed_multiplier, dt);
        }
    }

    /// Synchronizes active profile, all profiles list, career stats, and race history with the database.
    pub fn refresh_profiles_and_stats(&mut self) {
        if let Some(db) = &self.hof_db {
            let _ = db.seed_default_profile_if_empty();
            if let Ok(active) = db.get_active_profile() {
                self.active_profile = active;
                self.assist_profile = self.active_profile.last_mode;
                if let Some(player_car) = self.cars.first_mut() {
                    player_car.config.assists = self.assist_profile.to_config();
                }
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
                if let Ok(progress) = db.get_or_create_module_progress(pid, self.active_module_id) {
                    self.active_career_progress = progress;
                }
                if let Ok(all_prog) = db.get_all_module_progress(pid) {
                    self.profile_module_progress = all_prog;
                }
                if let Ok(awards) = db.get_championship_awards(pid) {
                    self.profile_awards = awards;
                }
            }
        }
    }

    /// Refreshes championship podium awards for the active profile from persistent storage.
    pub fn refresh_profile_awards(&mut self) {
        if let Some(db) = &self.hof_db {
            if let Some(pid) = self.active_profile.id {
                if let Ok(awards) = db.get_championship_awards(pid) {
                    self.profile_awards = awards;
                }
            }
        }
    }

    /// Sets the active driver assist profile, updating the car configuration, active profile state, and database.
    pub fn set_assist_profile(&mut self, profile: AssistProfile) {
        self.assist_profile = profile;
        self.active_profile.last_mode = profile;
        if let Some(pid) = self.active_profile.id {
            if let Some(db) = &self.hof_db {
                let _ = db.update_profile_last_mode(pid, profile);
            }
        }
        if let Some(player_car) = self.cars.first_mut() {
            player_car.config.assists = profile.to_config();
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

    /// Clears all historical records (Hall of Fame leaderboards, race history, personal bests, ghost lap)
    /// for a specific circuit when it is modified or reset.
    pub fn clear_circuit_history(&mut self, track_id: &str) {
        if let Some(db) = &self.hof_db {
            let _ = db.clear_track_history(track_id);
        }

        // Clean in-memory caches
        self.hof_entries.retain(|e| e.track_id != track_id);
        self.profile_history.retain(|r| r.track_id != track_id);
        self.active_profile_stats.best_times.remove(track_id);
        self.active_profile_stats.best_circuit_times.remove(track_id);

        if self.ghost_recorder.best_ghost_lap.as_ref().map_or(false, |g| g.track_choice.track_id() == track_id) {
            self.ghost_recorder.best_ghost_lap = None;
        }

        if let Some(id) = self.recent_hof_id {
            if !self.hof_entries.iter().any(|e| e.id == Some(id)) {
                self.recent_hof_id = None;
            }
        }

        // Re-sync active Hall of Fame and profile stats with database if available
        self.refresh_hof_entries();
        self.refresh_profiles_and_stats();
    }

    /// Clears all historical records (race history logs, career statistics, Hall of Fame entries, and ghost laps)
    /// for a specific driver profile without deleting the profile identity itself.
    pub fn clear_profile_history(&mut self, profile_id: i64) {
        let profile_alias = self
            .profile_list
            .iter()
            .find(|p| p.id == Some(profile_id))
            .map(|p| p.alias.clone())
            .or_else(|| {
                if self.active_profile.id == Some(profile_id) {
                    Some(self.active_profile.alias.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        if let Some(db) = &self.hof_db {
            let _ = db.clear_profile_historical_data(profile_id, &profile_alias);
        }

        // Clean in-memory caches if this profile is currently active
        let is_active = self.active_profile.id == Some(profile_id);
        if is_active {
            self.profile_history.clear();
            self.active_profile_stats = crate::profile::ProfileCareerStats::default();
            self.ghost_recorder.best_ghost_lap = None;
        } else {
            self.profile_history.retain(|r| r.profile_id != profile_id);
        }

        let p1_suffix = format!("{} (P1)", profile_alias);
        self.hof_entries
            .retain(|e| e.player_name != profile_alias && e.player_name != p1_suffix);

        if let Some(id) = self.recent_hof_id {
            if !self.hof_entries.iter().any(|e| e.id == Some(id)) {
                self.recent_hof_id = None;
            }
        }

        self.refresh_hof_entries();
        self.refresh_profiles_and_stats();
    }

    /// Clears all historical leaderboard records generated by AI bot drivers across all tracks.
    pub fn clear_bot_history(&mut self) {
        if let Some(db) = &self.hof_db {
            let _ = db.clear_bot_hall_of_fame();
        }

        let mut human_names = std::collections::HashSet::new();
        for p in &self.profile_list {
            human_names.insert(p.name.to_lowercase());
            human_names.insert(p.alias.to_lowercase());
            human_names.insert(format!("{} (p1)", p.alias).to_lowercase());
        }
        human_names.insert(self.active_profile.name.to_lowercase());
        human_names.insert(self.active_profile.alias.to_lowercase());
        human_names.insert(format!("{} (p1)", self.active_profile.alias).to_lowercase());
        human_names.insert("player 2 (p2)".to_string());

        self.hof_entries
            .retain(|e| human_names.contains(&e.player_name.to_lowercase()));

        if let Some(id) = self.recent_hof_id {
            if !self.hof_entries.iter().any(|e| e.id == Some(id)) {
                self.recent_hof_id = None;
            }
        }

        self.refresh_hof_entries();
    }

    /// Asynchronously initializes audio banks and plays the synthwave menu theme.
    pub async fn init_audio(&mut self) {
        self.audio.init_async().await;
        self.audio.play_music(MusicTrack::NeonMenu);
    }

    /// Opens the Arcade Settings Modal, pre-populating it with current audio, gamepad, assists, and display resolution.
    pub fn open_settings_modal(&mut self) {
        let mut modal = ArcadeSettingsModal::new(&self.audio.settings, &self.input.gamepad.config);
        let assist_idx = match self.assist_profile {
            AssistProfile::Arcade => 0,
            AssistProfile::Sport => 1,
            AssistProfile::Pro => 2,
        };
        modal.assist_dropdown.set_selected(assist_idx);
        modal.scanlines_dropdown.set_selected(self.crt_overlay.config.mode.to_index());
        modal.set_vehicle_shadows(self.config.display.vehicle_shadows);

        let (w, h) = if self.config.display.window_width > 0 && self.config.display.window_height > 0 {
            (self.config.display.window_width, self.config.display.window_height)
        } else {
            (screen_width_safe().round() as u32, screen_height_safe().round() as u32)
        };
        let is_fs = self.config.display.fullscreen;
        modal.set_display_state(w, h, is_fs);

        modal.snapshot_initial();

        self.settings_modal = Some(modal);
    }

    /// Returns true if the settings modal overlay is currently open.
    pub fn is_settings_modal_open(&self) -> bool {
        self.settings_modal.is_some()
    }

    /// Closes the settings modal, optionally applying the modified settings to audio, gamepad, assists, and display resolution.
    pub fn close_settings_modal(&mut self, save: bool) {
        if let Some(modal) = self.settings_modal.take() {
            if save {
                modal.apply_to_audio(&mut self.audio.settings);
                modal.apply_to_gamepad(&mut self.input.gamepad.config);
                let selected_mode = ScanlineMode::from_index(modal.scanlines_dropdown.selected_index);
                self.set_scanline_mode(selected_mode);
                let chosen_assist = match modal.assist_dropdown.selected_index {
                    0 => AssistProfile::Arcade,
                    1 => AssistProfile::Sport,
                    _ => AssistProfile::Pro,
                };
                self.set_assist_profile(chosen_assist);

                // Apply and persist display settings (resolution & fullscreen)
                modal.apply_display_settings();
                let (sel_w, sel_h) = modal.selected_resolution();
                let is_fs = modal.is_fullscreen();
                self.config.display.window_width = sel_w;
                self.config.display.window_height = sel_h;
                self.config.display.fullscreen = is_fs;
                self.config.display.vehicle_shadows = modal.vehicle_shadows();
                self.camera.set_screen_height(sel_h as f32);
                self.camera_p2.set_screen_height(sel_h as f32);
                self.editor_camera.set_screen_height(sel_h as f32);
                self.config.display.scanline_mode = match selected_mode {
                    ScanlineMode::Subtle => "subtle".to_string(),
                    ScanlineMode::ArcadeCrt => "arcade_crt".to_string(),
                    ScanlineMode::RetroGlow => "retro_glow".to_string(),
                    ScanlineMode::Disabled => "disabled".to_string(),
                };
                self.base_config.display = self.config.display.clone();
                self.base_config.audio = self.config.audio.clone();

                let _ = self.config.save_to_first_existing_or_default();
            }
        }
    }

    /// Sets active CRT scanline mode.
    pub fn set_scanline_mode(&mut self, mode: ScanlineMode) {
        self.crt_overlay.config.mode = mode;
        self.crt_overlay.config.vignette_intensity = if mode == ScanlineMode::Disabled {
            0.0
        } else if self.config.display.vignette_intensity > 0.0 {
            self.config.display.vignette_intensity
        } else {
            0.25
        };
        self.config.display.scanline_mode = match mode {
            ScanlineMode::Disabled => "disabled".to_string(),
            ScanlineMode::Subtle => "subtle".to_string(),
            ScanlineMode::ArcadeCrt => "arcade_crt".to_string(),
            ScanlineMode::RetroGlow => "retro_glow".to_string(),
        };
        self.base_config.display.scanline_mode = self.config.display.scanline_mode.clone();
    }

    /// Cycles CRT scanline intensity modes (Disabled -> Subtle -> Arcade CRT -> Retro Glow -> Disabled).
    pub fn cycle_scanline_mode(&mut self) -> ScanlineMode {
        let next = match self.crt_overlay.config.mode {
            ScanlineMode::Disabled => ScanlineMode::Subtle,
            ScanlineMode::Subtle => ScanlineMode::ArcadeCrt,
            ScanlineMode::ArcadeCrt => ScanlineMode::RetroGlow,
            ScanlineMode::RetroGlow => ScanlineMode::Disabled,
        };
        self.set_scanline_mode(next);
        next
    }

    /// Returns true if developer mode is enabled via config or environment / CLI flags.
    #[inline]
    pub fn is_dev_mode(&self) -> bool {
        self.config.gameplay.dev_mode || crate::storage::is_dev_mode()
    }

    /// Resolves the track's predefined vehicle model as a `CarChoice`.
    pub fn resolve_predefined_car(&self) -> CarChoice {
        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        resolve_predefined_car_for_track(Some(&self.track), effective_module)
    }

    /// Checks whether the specified car is unlocked under the active profile's career progress.
    pub fn is_car_unlocked(&self, car: CarChoice) -> bool {
        if self.is_dev_mode() || self.active_module_id == "classic" {
            return true;
        }
        let car_id = match car {
            CarChoice::GT4Clubsport => "gt4_clubsport",
            CarChoice::GT3Car => "gt3_evo",
            CarChoice::GT2Biturbo => "gt2_biturbo",
            CarChoice::GT1Legend => "gt1_legend",
            CarChoice::HypercarPrototype => "hypercar_prototype",
            _ => return true,
        };
        self.active_career_progress.is_car_unlocked(car_id, self.is_dev_mode())
    }

    /// Returns the motorsport tier (1..=5) of the player's active vehicle,
    /// respecting authentic catalog selection (selected_car_model_id) if active.
    pub fn active_player_car_tier(&self) -> u8 {
        if let Some(model_id) = self.selected_car_model_id {
            if let Some(model) = crate::catalog::find_model_by_id(model_id) {
                return model.tier;
            }
        }
        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        if effective_module != "classic" {
            if let Some(model) = crate::catalog::get_models_for_module(effective_module).first() {
                return model.tier;
            }
        }
        self.active_player_car_choice().tier()
    }

    /// Returns the career unlock level required for the player's active vehicle.
    pub fn active_player_car_unlock_level(&self) -> u32 {
        if let Some(model_id) = self.selected_car_model_id {
            if let Some(model) = crate::catalog::find_model_by_id(model_id) {
                return model.tier as u32;
            }
        }
        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        if effective_module != "classic" {
            if let Some(model) = crate::catalog::get_models_for_module(effective_module).first() {
                return model.tier as u32;
            }
        }
        self.active_player_car_choice().unlock_level()
    }

    /// Checks whether the player's active vehicle is eligible for a race with `required_tier` and the track surface.
    pub fn is_active_player_car_eligible(&self, required_tier: u8) -> bool {
        if self.is_dev_mode() {
            return true;
        }
        if self.active_player_car_tier() > required_tier {
            return false;
        }
        let track_surface = self.track.default_surface;
        if let Some(model_id) = self.selected_car_model_id {
            if let Some(model) = crate::catalog::find_model_by_id(model_id) {
                return model.is_eligible_for_surface(track_surface, self.is_dev_mode());
            }
        }
        self.active_player_car_choice().is_eligible_for_surface(track_surface, self.is_dev_mode())
    }

    /// Returns a warning advisory if the player's active vehicle has a severe mismatch with the track surface.
    pub fn active_player_surface_warning(&self) -> Option<&'static str> {
        let track_surface = self.track.default_surface;
        if let Some(model_id) = self.selected_car_model_id {
            if let Some(model) = crate::catalog::find_model_by_id(model_id) {
                return model.surface_warning(track_surface);
            }
        }
        self.active_player_car_choice().surface_warning(track_surface)
    }

    /// Checks whether the player's active vehicle is unlocked under career progression.
    pub fn is_active_player_car_unlocked(&self) -> bool {
        if self.is_dev_mode() || self.active_module_id == "classic" || self.game_mode == GameMode::StandardRace {
            return true;
        }
        if let Some(model_id) = self.selected_car_model_id {
            return self.active_career_progress.is_car_unlocked(model_id, self.is_dev_mode());
        }
        self.is_car_unlocked(self.active_player_car_choice())
    }

    /// Synchronizes active_career_progress with the current active_module_id from the database or default fallback.
    pub fn sync_career_progress_for_active_module(&mut self) {
        if let Some(existing) = self.profile_module_progress.get(self.active_module_id) {
            self.active_career_progress = existing.clone();
            return;
        }
        if let Some(db) = &self.hof_db {
            if let Some(pid) = self.active_profile.id {
                if let Ok(progress) = db.get_or_create_module_progress(pid, self.active_module_id) {
                    self.active_career_progress = progress;
                    return;
                }
            }
        }
        if self.active_career_progress.module_id != self.active_module_id {
            self.active_career_progress = crate::profile::ModuleCareerProgress::default_for_module(
                self.active_profile.id.unwrap_or(1),
                self.active_module_id,
            );
        }
    }

    /// Checks whether the specified track is unlocked under the active profile's career progress.
    pub fn is_track_unlocked(&self, track_id: &str) -> bool {
        if self.is_dev_mode() {
            return true;
        }
        if self.active_module_id == "gt" {
            self.active_career_progress.is_track_unlocked(track_id, self.is_dev_mode())
        } else {
            true
        }
    }

    /// Checks if a championship series is unlocked for the current profile.
    pub fn is_championship_unlocked(&self, champ: &SeriesDefinition) -> bool {
        if self.is_dev_mode() || champ.series.tier <= 1 {
            return true;
        }
        if self.active_career_progress.module_id.eq_ignore_ascii_case(&champ.series.module_id)
            || self.active_module_id.eq_ignore_ascii_case(&champ.series.module_id)
        {
            return self.active_career_progress.level >= champ.series.tier;
        }
        if let Some(db) = &self.hof_db {
            let pid = self.active_career_progress.profile_id;
            if let Ok(Some(prog)) = db.get_module_progress(pid, &champ.series.module_id) {
                return prog.level >= champ.series.tier;
            }
        }
        false
    }

    /// Returns the required motorsport category tier (1..=5) for the current race.
    pub fn current_race_required_tier(&self) -> u8 {
        if self.active_module_id == "classic" {
            5
        } else if let Some(champ) = &self.championship_session {
            (champ.tier as u8).clamp(1, 5)
        } else if self.game_mode == GameMode::Career {
            (self.active_career_progress.level as u8).clamp(1, 5)
        } else if self.free_car_selection {
            self.active_player_car_tier()
        } else {
            self.resolve_predefined_car().tier()
        }
    }

    /// Returns available vehicle choices for the active motorsport game module,
    /// filtered by the race's category requirement (tier <= required_tier || dev_mode).
    pub fn active_module_car_choices(&self) -> Vec<CarChoice> {
        let base_choices = match self.active_module_id {
            "gt" | "gt_challenge" => vec![
                CarChoice::GT4Clubsport,
                CarChoice::GT3Car,
                CarChoice::GT2Biturbo,
                CarChoice::GT1Legend,
                CarChoice::HypercarPrototype,
            ],
            "nascar" => vec![CarChoice::StockCar],
            "extreme_offroad" => vec![CarChoice::SandRail],
            "rally" => vec![CarChoice::RallyCar],
            "kart" => vec![CarChoice::Kart],
            "classic" => vec![
                CarChoice::SportsCar,
                CarChoice::StockCar,
                CarChoice::RallyCar,
                CarChoice::Kart,
                CarChoice::SandRail,
            ],
            _ => vec![
                CarChoice::SportsCar,
                CarChoice::DriftCar,
                CarChoice::Kart,
                CarChoice::RallyCar,
            ],
        };
        if self.active_module_id == "classic" {
            return base_choices;
        }
        if self.active_module_id != "gt" && self.active_module_id != "gt_challenge" {
            return base_choices;
        }
        let req_tier = self.current_race_required_tier();
        base_choices
            .into_iter()
            .filter(|c| c.is_eligible_for_race_tier(req_tier, self.is_dev_mode()))
            .collect()
    }

    /// Returns the active vehicle model for the player, respecting track enforcement or free selection.
    pub fn active_player_car_choice(&self) -> CarChoice {
        if self.free_car_selection {
            self.car_choice
        } else {
            self.resolve_predefined_car()
        }
    }

    /// Resolves the engine sound archetype for the active session, respecting vehicle selection and tier banks.
    pub fn resolve_active_sound_type(&self) -> EngineSoundType {
        // 1. In Garage showroom, use the focused car model from the catalog
        if matches!(self.state, GameState::Garage(_)) {
            let models = crate::catalog::get_models_for_module_and_tier(self.active_module_id, self.garage_tier);
            if let Some(active_car) = models.get(self.garage_car_idx) {
                return active_car.sound_type();
            }
            return self.car_choice.sound_type();
        }

        // 2. If a specific authentic car model was selected, dispatch by that model
        if let Some(model_id) = self.selected_car_model_id {
            if let Some(model) = crate::catalog::find_model_by_id(model_id) {
                return model.sound_type();
            }
        }

        // 3. In Classic mode, each vehicle uses the Tier 1 sound bank of its discipline
        let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        if effective_module == "classic" {
            return self.active_player_car_choice().sound_type();
        }

        // 4. Specialized modules default to their primary sound bank, falling back to car choice
        match effective_module {
            "gt" | "gt_challenge" => EngineSoundType::SportGT,
            "nascar" => EngineSoundType::NascarV8,
            "rally" => EngineSoundType::RallyTurbo,
            "kart" => EngineSoundType::Kart125cc,
            "extreme_offroad" => EngineSoundType::SandRailBoxer,
            _ => self.active_player_car_choice().sound_type(),
        }
    }

    /// Returns the pool of eligible car models for opponents based on the active motorsport category or track.
    pub fn eligible_opponent_cars(&self) -> Vec<CarChoice> {
        let cat = self.track.car_category;
        if self.active_module_id == "classic" {
            vec![CarChoice::classic_car_for_category(cat)]
        } else {
            let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
            match effective_module {
                "gt" | "gt_challenge" => vec![
                    CarChoice::GT4Clubsport,
                    CarChoice::GT3Car,
                    CarChoice::GT2Biturbo,
                    CarChoice::GT1Legend,
                    CarChoice::HypercarPrototype,
                ],
                "nascar" => vec![CarChoice::StockCar],
                "extreme_offroad" => vec![CarChoice::SandRail],
                "rally" => vec![CarChoice::RallyCar],
                "kart" => vec![CarChoice::Kart],
                _ => vec![CarChoice::classic_car_for_category(cat)],
            }
        }
    }

    /// Selects a pseudo-random opponent car from the eligible pool deterministically using bot index and seed.
    pub fn sample_random_opponent_car(&self, bot_idx: usize, bot_seed: u64) -> CarChoice {
        let pool = self.eligible_opponent_cars();
        if pool.is_empty() {
            return self.resolve_predefined_car();
        }
        if pool.len() == 1 {
            return pool[0];
        }
        let h = bot_seed
            .wrapping_mul(0x517CC1B727220A95)
            .wrapping_add((bot_idx as u64).wrapping_mul(0x9E3779B97F4A7C15));
        let idx = ((h >> 32) as usize) % pool.len();
        pool[idx]
    }

    /// Transitions from Starting Grid to Garage Showroom, focusing on the circuit's category vehicle.
    pub fn open_garage_from_starting_grid(&mut self) {
        self.audio.play_sfx(SfxType::UiSelect);
        self.garage_origin = GarageOrigin::StartingGrid;
        self.garage_tier = self.current_race_required_tier();
        if self.active_module_id == "classic" {
            let target_model = crate::catalog::get_classic_model_for_category(self.track.car_category);
            let models = crate::catalog::get_models_for_module("classic");
            self.garage_car_idx = models.iter().position(|m| m.id == target_model.id).unwrap_or(0);
        } else {
            let tier_models = crate::catalog::get_models_for_module_and_tier(self.active_module_id, self.garage_tier);
            if let Some(selected_id) = self.selected_car_model_id {
                self.garage_car_idx = tier_models.iter().position(|m| m.id == selected_id).unwrap_or(0);
            } else {
                self.garage_car_idx = 0;
            }
        }
        self.state = GameState::Garage(GarageOrigin::StartingGrid);
    }

    /// Transitions from Starting Grid to Profile Manager screen.
    pub fn open_profile_from_starting_grid(&mut self) {
        self.audio.play_sfx(SfxType::UiSelect);
        self.profile_origin = ProfileOrigin::StartingGrid;
        self.refresh_profiles_and_stats();
        let current_idx = self
            .profile_list
            .iter()
            .position(|p| p.id == self.active_profile.id)
            .unwrap_or(0);
        self.state = GameState::ProfileManager {
            selected_idx: current_idx,
        };
    }

    /// Transitions from Starting Grid to Track Selection Menu / Circuit Explorer.
    pub fn open_circuit_selector_from_starting_grid(&mut self) {
        self.audio.play_sfx(SfxType::UiSelect);
        self.menu_origin = MenuOrigin::StartingGrid;
        if self.track_choice.is_user_custom() {
            self.menu_track_filter = TrackCatalogFilter::Custom;
        } else {
            self.menu_track_filter = TrackCatalogFilter::Presets;
        }
        let tracks = self.filtered_menu_tracks();
        if let Some(pos) = tracks.iter().position(|t| t.track_id() == self.track_choice.track_id()) {
            self.menu_track_idx = pos;
        }
        self.state = GameState::Menu;
    }

    /// Returns available circuits for the active motorsport game module (including both presets and custom circuits).
    pub fn active_module_tracks(&self) -> Vec<TrackChoice> {
        let mut tracks = self.track_manager.preset_track_choices(self.active_module_id);
        let custom_tracks = self.track_manager.custom_track_choices();
        for custom in custom_tracks {
            let matches_mod = if let Some(custom_info) = self.track_manager.custom_track_info(custom.track_id()) {
                custom_info.belongs_to_module(self.active_module_id)
            } else {
                true
            };
            if matches_mod && !tracks.iter().any(|t| t.track_id() == custom.track_id()) {
                tracks.push(custom);
            }
        }
        tracks
    }

    /// Returns available circuits filtered by the active menu catalog filter tab.
    pub fn filtered_menu_tracks(&self) -> Vec<TrackChoice> {
        let all = self.active_module_tracks();
        match self.menu_track_filter {
            TrackCatalogFilter::Presets => all.into_iter().filter(|t| t.is_official_preset()).collect(),
            TrackCatalogFilter::Custom => all.into_iter().filter(|t| t.is_user_custom()).collect(),
        }
    }

    /// Returns counts of (presets, custom) tracks for the active motorsport module.
    pub fn menu_track_filter_counts(&self) -> (usize, usize) {
        let all = self.active_module_tracks();
        let presets = all.iter().filter(|t| t.is_official_preset()).count();
        let custom = all.iter().filter(|t| t.is_user_custom()).count();
        (presets, custom)
    }

    /// Returns available driver characters for the active motorsport game module.
    pub fn active_module_drivers(&self) -> Vec<DriverCharacter> {
        let effective_mod = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
        match effective_mod {
            "gt" | "gt_challenge" => GtWorldChallengeModule::new().drivers(),
            "rally" => RallyGameModule::new().drivers(),
            "kart" => KartGameModule::new().drivers(),
            "nascar" => NascarGameModule::new().drivers(),
            "extreme_offroad" => ExtremeOffRoadModule::new().drivers(),
            _ => DriverCharacter::all().to_vec(),
        }
    }

    /// Returns available vehicles for the active motorsport game module.
    pub fn active_module_vehicles(&self) -> Vec<(&'static str, &'static str, &'static str, (f32, f32, f32, f32))> {
        match self.active_module_id {
            "gt" | "gt_challenge" => vec![
                (CarChoice::GT4Clubsport.title(), CarChoice::GT4Clubsport.tag(), CarChoice::GT4Clubsport.description(), CarChoice::GT4Clubsport.stats()),
                (CarChoice::GT3Car.title(), CarChoice::GT3Car.tag(), CarChoice::GT3Car.description(), CarChoice::GT3Car.stats()),
                (CarChoice::GT2Biturbo.title(), CarChoice::GT2Biturbo.tag(), CarChoice::GT2Biturbo.description(), CarChoice::GT2Biturbo.stats()),
                (CarChoice::GT1Legend.title(), CarChoice::GT1Legend.tag(), CarChoice::GT1Legend.description(), CarChoice::GT1Legend.stats()),
                (CarChoice::HypercarPrototype.title(), CarChoice::HypercarPrototype.tag(), CarChoice::HypercarPrototype.description(), CarChoice::HypercarPrototype.stats()),
            ],
            "rally" => vec![
                (CarChoice::RallyCar.title(), CarChoice::RallyCar.tag(), CarChoice::RallyCar.description(), CarChoice::RallyCar.stats()),
                ("Group B Turbo Monster", "GROUP B LEGEND", "520 BHP 1980s twin-charge monster with flame-spitting anti-lag.", (0.95, 0.98, 0.75, 0.98)),
            ],
            "kart" => vec![
                (CarChoice::Kart.title(), CarChoice::Kart.tag(), CarChoice::Kart.description(), CarChoice::Kart.stats()),
                ("100cc Direct Drive Sprint", "100cc CLUTCHLESS", "High-revving direct-drive kart with ultra sharp throttle response.", (0.65, 0.95, 0.99, 0.45)),
            ],
            "nascar" => vec![
                (CarChoice::StockCar.title(), CarChoice::StockCar.tag(), CarChoice::StockCar.description(), CarChoice::StockCar.stats()),
                ("Trans-Am TA1 Spaceframe V8", "850 BHP SPACEFRAME", "Pure American road racing silhouette monster: tube-frame chassis, high-mount carbon GT wing, side boom tubes.", (0.95, 0.92, 0.91, 0.85)),
            ],
            "extreme_offroad" => vec![
                (CarChoice::SandRail.title(), CarChoice::SandRail.tag(), CarChoice::SandRail.description(), CarChoice::SandRail.stats()),
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
        self.camera_p2.position_smoothing = self.config.camera.position_smoothing;
        self.camera_p2.zoom_smoothing = self.config.camera.zoom_smoothing;
        self.camera_p2.velocity_lookahead_time = self.config.camera.velocity_lookahead_time;
        self.camera_p2.trauma_decay = self.config.camera.trauma_decay;
        self.camera_p2.max_shake_offset = self.config.camera.max_shake_offset;
        if !self.config.camera.levels.is_empty() {
            self.camera.levels = self.config.camera.levels.clone();
            self.camera.current_level_idx = self
                .config
                .camera
                .default_level_index
                .min(self.camera.levels.len().saturating_sub(1));
            self.camera_p2.levels = self.config.camera.levels.clone();
            self.camera_p2.current_level_idx = self.camera.current_level_idx;
        }

        // Apply gameplay settings
        self.num_bots = self.config.gameplay.default_num_bots;
        self.total_laps = self.config.gameplay.default_laps;
        // Driving assist mode belongs to the player: any player starts in Arcade mode,
        // and for any new race it starts with the last mode the player used.
        // We preserve self.assist_profile across module changes.
    }

    /// Switches the active motorsport game module (nascar, gt, rally, kart, classic, extreme_offroad).
    pub fn switch_to_module(&mut self, mod_id: &str) {
        match mod_id {
            "nascar" => self.switch_to_nascar(),
            "gt" | "gt_challenge" => self.switch_to_gt(),
            "rally" => self.switch_to_rally(),
            "kart" => self.switch_to_kart(),
            "extreme_offroad" | "offroad" => self.switch_to_extreme_offroad(),
            _ => self.switch_to_classic(),
        }
        self.sync_career_progress_for_active_module();
    }

    /// Activates the NASCAR Cup Series & Trans-Am TA1 module.
    pub fn switch_to_nascar(&mut self) {
        self.apply_module_config("nascar");
        self.active_module_id = "nascar";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::StockCar {
            tall_wing: false,
            roof_fins: true,
            window_net: true,
        };
        self.selected_car_model_id = Some("nascar_monte_carlo_ss");
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
                id: "daytona_superspeedway".to_string(),
                title: "Daytona International Speedway".to_string(),
                description: "The World Center of Racing. 2.5-mile tri-oval with 31-degree banking.".to_string(),
                path: "nascar/daytona_superspeedway".to_string(),
            });
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::StockCar;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 3;
        }
        self.camera.setup_for_track(&self.track);
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Extreme Off-Road & Stunt Arenas module.
    pub fn switch_to_extreme_offroad(&mut self) {
        self.apply_module_config("extreme_offroad");
        self.active_module_id = "extreme_offroad";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::SandRail {
            lightbar: true,
            whip_antenna: true,
            paddle_tires: true,
        };
        self.selected_car_model_id = Some("offroad_sand_rail_buggy");
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
                id: "sahara_dune_crossing".to_string(),
                title: "Sahara Dune Crossing".to_string(),
                description: "High-speed sweeping desert crossing over cresting sand dunes.".to_string(),
                path: "extreme_offroad/sahara_dune_crossing".to_string(),
            });
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = CarChoice::SandRail;
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 3;
        }
        self.camera.setup_for_track(&self.track);
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Returns the player's effective color scheme: factory livery colors when
    /// a real car model is selected, otherwise the profile's abstract scheme.
    fn player_effective_color_scheme(&self) -> CarColorScheme {
        if self.game_mode == GameMode::Career {
            let model_opt = self
                .selected_car_model_id
                .and_then(crate::catalog::find_model_by_id)
                .or_else(|| {
                    if self.active_module_id == "classic" {
                        Some(crate::catalog::get_classic_model_for_category(self.track.car_category))
                    } else {
                        None
                    }
                });
            if let Some(model) = model_opt {
                return CarColorScheme {
                    primary: model.primary_color,
                    secondary: model.secondary_color,
                    helmet: self.active_profile.color_scheme.helmet,
                };
            }
        }
        if self.active_module_id == "classic" {
            return self.active_profile.color_scheme;
        }
        if let Some(model) = self
            .selected_car_model_id
            .and_then(crate::catalog::find_model_by_id)
        {
            CarColorScheme {
                primary: model.primary_color,
                secondary: model.secondary_color,
                helmet: self.active_profile.color_scheme.helmet,
            }
        } else {
            self.active_profile.color_scheme
        }
    }

    /// Resolves the color scheme for an AI bot driver across Career, Quick, and Custom races.
    /// Bots must use masked colors (must not match the vehicle model's factory livery,
    /// triggering dynamic mask-based tinting) and must have primary colors visually distinct
    /// from the player's primary color and other cars on the grid, maximizing color diversity.
    pub fn resolve_bot_color_scheme(
        base_scheme: CarColorScheme,
        bot_model: Option<&crate::catalog::RealCarModel>,
        player_primary: Option<Color>,
        existing_participants: &[GridParticipant],
        bot_idx: usize,
    ) -> CarColorScheme {
        let pp = player_primary.unwrap_or(Palette::CAR_COLORS[0].0);
        let bot_mid = bot_model.map(|m| m.id);

        let color_dist = |c1: Color, c2: Color| -> f32 {
            let dr = c1.r - c2.r;
            let dg = c1.g - c2.g;
            let db = c1.b - c2.b;
            (dr * dr + dg * dg + db * db).sqrt()
        };

        let is_factory = |c: Color| -> bool {
            if let Some(m) = bot_model {
                let dr = (c.r - m.primary_color.r).abs();
                let dg = (c.g - m.primary_color.g).abs();
                let db = (c.b - m.primary_color.b).abs();
                dr < 0.05 && dg < 0.05 && db < 0.05
            } else {
                false
            }
        };

        // Hard conflict: matches factory livery, too close to player (< 0.20),
        // or too close (< 0.20) to another car sharing the same vehicle model.
        let is_hard_conflict = |s: &CarColorScheme| -> bool {
            if is_factory(s.primary) {
                return true;
            }
            if color_dist(s.primary, pp) < 0.20 {
                return true;
            }
            if let Some(mid) = bot_mid {
                for other in existing_participants {
                    if other.model_id == Some(mid) && color_dist(s.primary, other.color_scheme.primary) < 0.20 {
                        return true;
                    }
                }
            }
            false
        };

        // Grid-wide diversity conflict: also checks if any existing car on the grid has the same/close primary color (< 0.20).
        let is_grid_conflict = |s: &CarColorScheme| -> bool {
            if is_hard_conflict(s) {
                return true;
            }
            for other in existing_participants {
                if color_dist(s.primary, other.color_scheme.primary) < 0.20 {
                    return true;
                }
            }
            false
        };

        // 1. If base scheme has no conflict across the grid, retain driver's signature livery.
        if !is_grid_conflict(&base_scheme) {
            return base_scheme;
        }

        // 2. Try candidate color presets from Palette::CAR_COLORS (indices 1..=8, excluding index 0 player red)
        let num_candidates = Palette::CAR_COLORS.len() - 1;
        for offset in 0..num_candidates {
            let candidate_idx = (bot_idx + offset) % num_candidates + 1;
            let candidate = CarColorScheme::from_index(candidate_idx);
            if !is_grid_conflict(&candidate) {
                return CarColorScheme {
                    primary: candidate.primary,
                    secondary: candidate.secondary,
                    helmet: base_scheme.helmet,
                };
            }
        }

        // 3. If all presets have grid-wide conflicts (e.g. grids > 8 cars), relax grid-wide check
        // but strictly enforce hard constraints (not factory, distinct from player, distinct from same model).
        for offset in 0..num_candidates {
            let candidate_idx = (bot_idx + offset) % num_candidates + 1;
            let candidate = CarColorScheme::from_index(candidate_idx);
            if !is_hard_conflict(&candidate) {
                return CarColorScheme {
                    primary: candidate.primary,
                    secondary: candidate.secondary,
                    helmet: base_scheme.helmet,
                };
            }
        }

        // 4. Fallback: generate a deterministic hue-shifted color that avoids player & factory.
        let mut best_color = base_scheme.primary;
        let mut best_dist = 0.0f32;
        for step in 1..=8 {
            let shift = (step as f32 * 0.125 + (bot_idx as f32 * 0.07)) % 1.0;
            let cand_c = Color::new(
                (base_scheme.primary.r + shift) % 1.0,
                (base_scheme.primary.g + shift) % 1.0,
                (base_scheme.primary.b + shift) % 1.0,
                1.0,
            );
            if !is_factory(cand_c) {
                let d = color_dist(cand_c, pp);
                if d > best_dist {
                    best_dist = d;
                    best_color = cand_c;
                }
            }
        }

        CarColorScheme {
            primary: best_color,
            secondary: base_scheme.secondary,
            helmet: base_scheme.helmet,
        }
    }

    /// Resolves the color scheme for a bot in Career mode (backward-compatible delegate).
    pub fn resolve_bot_career_color_scheme(
        base_scheme: CarColorScheme,
        bot_model: Option<&crate::catalog::RealCarModel>,
        player_model: Option<&crate::catalog::RealCarModel>,
        bot_idx: usize,
    ) -> CarColorScheme {
        let player_primary = player_model.map(|m| m.primary_color);
        Self::resolve_bot_color_scheme(base_scheme, bot_model, player_primary, &[], bot_idx)
    }

    /// Activates the GT World Challenge module.
    pub fn switch_to_gt(&mut self) {
        self.apply_module_config("gt");
        self.active_module_id = "gt";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        };
        self.selected_car_model_id = Some("gt_toyota_supra_gt4");
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
                path: "gt/monza".to_string(),
            });
        }
        self.track = self.load_track_for_session(&self.track_choice);
        self.car_choice = self.resolve_predefined_car();
        if self.config.gameplay.default_laps == self.base_config.gameplay.default_laps {
            self.total_laps = 5;
        }
        self.camera.setup_for_track(&self.track);
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Rallycross World Cup module.
    pub fn switch_to_rally(&mut self) {
        self.apply_module_config("rally");
        self.active_module_id = "rally";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::RallyHatch {
            roof_scoop: true,
            mudflaps: true,
            large_wing: true,
        };
        self.selected_car_model_id = Some("rally_peugeot_208_rally4");
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
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Karting World Cup module.
    pub fn switch_to_kart(&mut self) {
        self.apply_module_config("kart");
        self.active_module_id = "kart";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::GoKart {
            exposed_driver: true,
            side_bumpers: true,
        };
        self.selected_car_model_id = Some("kart_crg_hero_60");
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
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Activates the Classic Arcade Motorsport module.
    pub fn switch_to_classic(&mut self) {
        self.apply_module_config("classic");
        self.active_module_id = "classic";
        self.sync_career_progress_for_active_module();
        self.menu_track_idx = 0;
        self.menu_car_idx = 0;
        self.current_visual_type = VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        };
        self.selected_car_model_id = None;
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
        self.camera_p2.setup_for_track(&self.track);
        self.rebuild_roster_participants();
        self.state = GameState::Menu;
    }

    /// Starts a full GT World Challenge Championship Season.
    pub fn start_gt_championship(&mut self) {
        let champ = ChampionshipSession::new(
            "GT World Challenge Championship 2026",
            PointSystem::FiaStandard { fastest_lap_bonus: true },
            vec!["monza".to_string(), "spa".to_string(), "silverstone".to_string(), "classic_grand_prix".to_string()],
            3,
            &[
                ("player", "Player", "Apex GT Racing"),
                ("max_hunter", "Max Hunter", "Red Bull GT"),
                ("charles_laurent", "Charles Laurent", "Scuderia GT"),
                ("lewis_vance", "Lewis Vance", "Scuderia GT"),
                ("fernando_toro", "Fernando Toro", "Aston GT"),
                ("george_speed", "George Speed", "Mercedes-AMG GT"),
                ("lando_vance", "Lando Vance", "McLaren GT"),
                ("oscar_rocket", "Oscar Rocket", "McLaren GT"),
            ],
        );
        self.switch_to_gt();
        self.championship_session = Some(champ.with_tier(2));
        self.init_race();
    }

    /// Launches a GT Career Championship Cup for the given tier (1..=5).
    pub fn start_gt_career_tier(&mut self, tier: u32) {
        self.start_gt_career_tier_with_calendar(tier, None);
    }

    /// Launches a GT Career Championship Cup with an optional custom calendar.
    pub fn start_gt_career_tier_with_calendar(&mut self, tier: u32, custom_tracks: Option<Vec<String>>) {
        let default_tracks = crate::ui::gt_default_calendar(tier);
        let mandatory_tracks = crate::ui::gt_mandatory_tracks(tier);
        let track_ids = match custom_tracks {
            Some(tracks) => {
                if tracks.len() == default_tracks.len()
                    && mandatory_tracks.iter().all(|m| tracks.iter().any(|t| t == m))
                {
                    tracks
                } else {
                    default_tracks
                }
            }
            None => default_tracks,
        };
        let cup_name = crate::ui::gt_tier_title(tier);
        let car_choice = match tier {
            1 => CarChoice::GT4Clubsport,
            2 => CarChoice::GT3Car,
            3 => CarChoice::GT2Biturbo,
            4 => CarChoice::GT1Legend,
            _ => CarChoice::HypercarPrototype,
        };

        let active_session = self.active_career_progress.active_championship.clone().filter(|s| {
            !s.is_completed && s.tier == tier
        });

        let champ = if let Some(mut existing) = active_session {
            if !self.active_career_progress.career_rivals.is_empty() {
                existing.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            }
            existing
        } else {
            let mut c = ChampionshipSession::new(
                cup_name,
                PointSystem::FiaStandard { fastest_lap_bonus: true },
                track_ids,
                3,
                &[
                    ("player", "Player", "Apex GT Racing"),
                    ("max_hunter", "Max Hunter", "Red Bull GT"),
                    ("charles_laurent", "Charles Laurent", "Scuderia GT"),
                    ("lewis_vance", "Lewis Vance", "Scuderia GT"),
                    ("fernando_toro", "Fernando Toro", "Aston GT"),
                    ("george_speed", "George Speed", "Mercedes-AMG GT"),
                    ("lando_vance", "Lando Vance", "McLaren GT"),
                    ("oscar_rocket", "Oscar Rocket", "McLaren GT"),
                ],
            );
            if !self.active_career_progress.career_rivals.is_empty() {
                c.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            } else {
                let rivals = c
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .map(|s| {
                        let char_def = DriverCharacter::find_global(&s.driver_id);
                        let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                        CareerRivalEntry {
                            driver_id: s.driver_id.clone(),
                            driver_name: s.driver_name.clone(),
                            style,
                            tier: DriverTier::from_u8(tier.clamp(1, 5) as u8),
                        }
                    })
                    .collect();
                self.active_career_progress.career_rivals = rivals;
            }
            c
        };
        let prev_selected = self.selected_car_model_id;
        self.switch_to_gt();
        self.game_mode = GameMode::Career;

        let selected_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == "gt" && m.tier == tier as u8 && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("gt", tier as u8)
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("gt", tier as u8)
                    .into_iter()
                    .next()
            });

        if let Some(model) = selected_model {
            self.active_career_progress.ensure_car(model.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(model.id);
            self.car_choice = model.base_car_choice;
            self.current_visual_type = model.visual_type;
            self.free_car_selection = true;
        } else {
            self.car_choice = car_choice;
        }
        self.championship_session = Some(champ.with_tier(tier));
        self.active_career_progress.active_championship = self.championship_session.clone();
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert("gt".to_string(), self.active_career_progress.clone());

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }
        self.init_race();
    }

    /// Launches a NASCAR Career Championship Cup for the given tier (1..=5).
    pub fn start_nascar_career_tier(&mut self, tier: u32) {
        let (cup_name, track_ids) = match tier {
            1 => (
                "NASCAR Weekly Short Track Series (Tier 1)",
                vec![
                    "martinsville_speedway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "eldora_speedway".to_string(),
                    "bowman_gray_stadium".to_string(),
                    "lucas_oil_irp".to_string(),
                ],
            ),
            2 => (
                "NASCAR Intermediate Oval Challenge (Tier 2)",
                vec![
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "north_wilkesboro_speedway".to_string(),
                    "martinsville_speedway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "eldora_speedway".to_string(),
                    "lucas_oil_irp".to_string(),
                ],
            ),
            3 => (
                "NASCAR National Road & Oval Tour (Tier 3)",
                vec![
                    "iowa_speedway".to_string(),
                    "watkins_glen_nascar".to_string(),
                    "road_america".to_string(),
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "north_wilkesboro_speedway".to_string(),
                    "martinsville_speedway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "lucas_oil_irp".to_string(),
                ],
            ),
            4 => (
                "NASCAR Premier Speedway Trophy (Tier 4)",
                vec![
                    "indianapolis_motor_speedway".to_string(),
                    "pocono_raceway".to_string(),
                    "chicago_street_course".to_string(),
                    "iowa_speedway".to_string(),
                    "watkins_glen_nascar".to_string(),
                    "road_america".to_string(),
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "bristol_motor_speedway".to_string(),
                    "martinsville_speedway".to_string(),
                ],
            ),
            _ => (
                "NASCAR Cup Series Championship (Tier 5)",
                vec![
                    "daytona_superspeedway".to_string(),
                    "talladega_superspeedway".to_string(),
                    "phoenix_raceway".to_string(),
                    "indianapolis_motor_speedway".to_string(),
                    "pocono_raceway".to_string(),
                    "chicago_street_course".to_string(),
                    "iowa_speedway".to_string(),
                    "watkins_glen_nascar".to_string(),
                    "road_america".to_string(),
                    "charlotte_motor_speedway".to_string(),
                    "darlington_raceway".to_string(),
                    "martinsville_speedway".to_string(),
                ],
            ),
        };

        let active_session = self.active_career_progress.active_championship.clone().filter(|s| {
            !s.is_completed && s.tier == tier
        });

        let champ = if let Some(mut existing) = active_session {
            if !self.active_career_progress.career_rivals.is_empty() {
                existing.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            }
            existing
        } else {
            let mut c = ChampionshipSession::new(
                cup_name,
                PointSystem::NascarCup { stage_win_bonus: true },
                track_ids,
                4,
                &[
                    ("player", "Player", "Apex Stock Car"),
                    ("dale_vance", "Dale 'The Intimidator' Vance", "Richard Childress Racing"),
                    ("chase_gordon", "Chase 'Rainbow' Gordon", "Hendrick Motorsports"),
                    ("richard_pettyfield", "Richard 'The King' Pettyfield", "Petty Enterprises"),
                    ("rowdy_busch", "Rowdy 'Wild Thing' Busch", "Joe Gibbs Racing"),
                    ("jimmie_johnson", "Jimmie 'Seven-Time' Johnson", "Hendrick Motorsports"),
                    ("tony_stewart", "Tony 'Smoke' Stewart", "Stewart-Haas Racing"),
                    ("bobby_allison", "Bobby 'Alabama' Allison", "Alabama Gang"),
                    ("bubba_wallace", "Bubba 'The Rocket' Wallace", "23XI Racing"),
                    ("joey_logano", "Joey 'Sliced Bread' Logano", "Team Penske"),
                    ("bill_elliott", "Bill 'Awesome Bill' Elliott", "Melling Racing"),
                    ("cale_yarborough", "Cale 'The Iron Man' Yarborough", "Junior Johnson Racing"),
                ],
            );
            if !self.active_career_progress.career_rivals.is_empty() {
                c.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            } else {
                let rivals = c
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .map(|s| {
                        let char_def = DriverCharacter::find_global(&s.driver_id);
                        let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                        CareerRivalEntry {
                            driver_id: s.driver_id.clone(),
                            driver_name: s.driver_name.clone(),
                            style,
                            tier: DriverTier::from_u8(tier.clamp(1, 5) as u8),
                        }
                    })
                    .collect();
                self.active_career_progress.career_rivals = rivals;
            }
            c
        };
        let prev_selected = self.selected_car_model_id;
        self.switch_to_nascar();
        self.game_mode = GameMode::Career;

        let selected_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == "nascar" && m.tier == tier as u8 && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("nascar", tier as u8)
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("nascar", tier as u8)
                    .into_iter()
                    .next()
            });

        if let Some(model) = selected_model {
            self.active_career_progress.ensure_car(model.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(model.id);
            self.car_choice = model.base_car_choice;
            self.current_visual_type = model.visual_type;
            self.free_car_selection = true;
        } else {
            self.car_choice = CarChoice::StockCar;
        }
        self.championship_session = Some(champ.with_tier(tier));
        self.active_career_progress.active_championship = self.championship_session.clone();
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert("nascar".to_string(), self.active_career_progress.clone());

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }
        self.init_race();
    }

    /// Launches a Rallycross Career Championship Cup for the given tier (1..=5).
    pub fn start_rally_career_tier(&mut self, tier: u32) {
        let (cup_name, track_ids) = match tier {
            1 => (
                "Rallycross Grassroots Cup (Tier 1)",
                vec![
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "mettet_rx".to_string(),
                    "dreux_rx".to_string(),
                    "blyton_rx".to_string(),
                ],
            ),
            2 => (
                "World Rallycross Challenge (Tier 2)",
                vec![
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "silverstone_rx".to_string(),
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "mettet_rx".to_string(),
                    "blyton_rx".to_string(),
                ],
            ),
            3 => (
                "Group B Masters Series (Tier 3)",
                vec![
                    "estering_rx".to_string(),
                    "montalegre_rx".to_string(),
                    "riga_rx".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "silverstone_rx".to_string(),
                    "holjes_rx".to_string(),
                    "lydden_hill".to_string(),
                    "mettet_rx".to_string(),
                ],
            ),
            4 => (
                "Dakar Rally Raid Trophy (Tier 4)",
                vec![
                    "nyirad_rx".to_string(),
                    "kouvola_rx".to_string(),
                    "killarney_rx".to_string(),
                    "estering_rx".to_string(),
                    "montalegre_rx".to_string(),
                    "riga_rx".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "silverstone_rx".to_string(),
                    "holjes_rx".to_string(),
                ],
            ),
            _ => (
                "Stadium Super Trucks World Series (Tier 5)",
                vec![
                    "catalunya_rx".to_string(),
                    "yas_marina_rx".to_string(),
                    "essay_rx".to_string(),
                    "nyirad_rx".to_string(),
                    "kouvola_rx".to_string(),
                    "killarney_rx".to_string(),
                    "estering_rx".to_string(),
                    "montalegre_rx".to_string(),
                    "riga_rx".to_string(),
                    "hell_rx".to_string(),
                    "loheac_rx".to_string(),
                    "holjes_rx".to_string(),
                ],
            ),
        };

        let active_session = self.active_career_progress.active_championship.clone().filter(|s| {
            !s.is_completed && s.tier == tier
        });

        let champ = if let Some(mut existing) = active_session {
            if !self.active_career_progress.career_rivals.is_empty() {
                existing.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            }
            existing
        } else {
            let mut c = ChampionshipSession::new(
                cup_name,
                PointSystem::FiaStandard { fastest_lap_bonus: true },
                track_ids,
                5,
                &[
                    ("player", "Player", "Apex Rally Team"),
                    ("johan_vance", "Johan Vance", "KMS Motorsport"),
                    ("mattias_storm", "Mattias Storm", "EKS RX"),
                    ("timmy_hansenfield", "Timmy Hansenfield", "Hansen Motorsport"),
                    ("kevin_hansenfield", "Kevin Hansenfield", "Hansen Motorsport"),
                    ("niclas_gron", "Niclas Gron", "GRX Taneco"),
                    ("anton_mark", "Anton Mark", "GCK Motorsport"),
                    ("timo_scheider", "Timo Scheider", "All-Inkl Racing"),
                ],
            );
            if !self.active_career_progress.career_rivals.is_empty() {
                c.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            } else {
                let rivals = c
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .map(|s| {
                        let char_def = DriverCharacter::find_global(&s.driver_id);
                        let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                        CareerRivalEntry {
                            driver_id: s.driver_id.clone(),
                            driver_name: s.driver_name.clone(),
                            style,
                            tier: DriverTier::from_u8(tier.clamp(1, 5) as u8),
                        }
                    })
                    .collect();
                self.active_career_progress.career_rivals = rivals;
            }
            c
        };
        let prev_selected = self.selected_car_model_id;
        self.switch_to_rally();
        self.game_mode = GameMode::Career;

        let selected_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == "rally" && m.tier == tier as u8 && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("rally", tier as u8)
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("rally", tier as u8)
                    .into_iter()
                    .next()
            });

        if let Some(model) = selected_model {
            self.active_career_progress.ensure_car(model.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(model.id);
            self.car_choice = model.base_car_choice;
            self.current_visual_type = model.visual_type;
            self.free_car_selection = true;
        } else {
            self.car_choice = CarChoice::RallyCar;
        }
        self.championship_session = Some(champ.with_tier(tier));
        self.active_career_progress.active_championship = self.championship_session.clone();
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert("rally".to_string(), self.active_career_progress.clone());

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }
        self.init_race();
    }

    /// Launches a Karting Career Championship Cup for the given tier (1..=5).
    pub fn start_kart_career_tier(&mut self, tier: u32) {
        let (cup_name, track_ids) = match tier {
            1 => (
                "Rotax Junior Academy (Tier 1)",
                vec![
                    "lonato".to_string(),
                    "genk".to_string(),
                    "wackersdorf".to_string(),
                    "laval_kart".to_string(),
                    "whilton_mill".to_string(),
                ],
            ),
            2 => (
                "National Kart Championship (Tier 2)",
                vec![
                    "sarno".to_string(),
                    "kristianstad".to_string(),
                    "seven_laghi".to_string(),
                    "lonato".to_string(),
                    "genk".to_string(),
                    "wackersdorf".to_string(),
                    "whilton_mill".to_string(),
                ],
            ),
            3 => (
                "Continental Rotax Trophy (Tier 3)",
                vec![
                    "pfi".to_string(),
                    "franciacorta".to_string(),
                    "ampfing".to_string(),
                    "sarno".to_string(),
                    "kristianstad".to_string(),
                    "seven_laghi".to_string(),
                    "lonato".to_string(),
                    "genk".to_string(),
                    "wackersdorf".to_string(),
                ],
            ),
            4 => (
                "FIA Karting European Championship (Tier 4)",
                vec![
                    "zuera".to_string(),
                    "silverstone_national_kart".to_string(),
                    "le_mans_kart".to_string(),
                    "pfi".to_string(),
                    "franciacorta".to_string(),
                    "ampfing".to_string(),
                    "sarno".to_string(),
                    "kristianstad".to_string(),
                    "seven_laghi".to_string(),
                    "lonato".to_string(),
                ],
            ),
            _ => (
                "FIA Karting World Championship (Tier 5)",
                vec![
                    "portimao_kart".to_string(),
                    "valencia_kart".to_string(),
                    "campillos".to_string(),
                    "zuera".to_string(),
                    "silverstone_national_kart".to_string(),
                    "le_mans_kart".to_string(),
                    "pfi".to_string(),
                    "franciacorta".to_string(),
                    "ampfing".to_string(),
                    "sarno".to_string(),
                    "kristianstad".to_string(),
                    "lonato".to_string(),
                ],
            ),
        };

        let active_session = self.active_career_progress.active_championship.clone().filter(|s| {
            !s.is_completed && s.tier == tier
        });

        let champ = if let Some(mut existing) = active_session {
            if !self.active_career_progress.career_rivals.is_empty() {
                existing.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            }
            existing
        } else {
            let mut c = ChampionshipSession::new(
                cup_name,
                PointSystem::FiaStandard { fastest_lap_bonus: true },
                track_ids,
                5,
                &[
                    ("player", "Player", "Apex Kart Racing"),
                    ("marco_armani", "Marco Armani", "Tony Kart Racing"),
                    ("lucas_vance", "Lucas Vance", "CRG Factory Team"),
                    ("alex_rossi", "Alex Rossi", "Birel ART"),
                    ("sofia_lind", "Sofia Lind", "Kosmic Racing"),
                    ("finn_korhonen", "Finn Korhonen", "Sodi Kart"),
                    ("leo_dupont", "Leo Dupont", "Energy Corse"),
                    ("mateo_silva", "Mateo Silva", "Parolin Motorsport"),
                ],
            );
            if !self.active_career_progress.career_rivals.is_empty() {
                c.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            } else {
                let rivals = c
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .map(|s| {
                        let char_def = DriverCharacter::find_global(&s.driver_id);
                        let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                        CareerRivalEntry {
                            driver_id: s.driver_id.clone(),
                            driver_name: s.driver_name.clone(),
                            style,
                            tier: DriverTier::from_u8(tier.clamp(1, 5) as u8),
                        }
                    })
                    .collect();
                self.active_career_progress.career_rivals = rivals;
            }
            c
        };
        let prev_selected = self.selected_car_model_id;
        self.switch_to_kart();
        self.game_mode = GameMode::Career;

        let selected_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == "kart" && m.tier == tier as u8 && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("kart", tier as u8)
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("kart", tier as u8)
                    .into_iter()
                    .next()
            });

        if let Some(model) = selected_model {
            self.active_career_progress.ensure_car(model.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(model.id);
            self.car_choice = model.base_car_choice;
            self.current_visual_type = model.visual_type;
            self.free_car_selection = true;
        } else {
            self.car_choice = CarChoice::Kart;
        }
        self.championship_session = Some(champ.with_tier(tier));
        self.active_career_progress.active_championship = self.championship_session.clone();
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert("kart".to_string(), self.active_career_progress.clone());

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }
        self.init_race();
    }

    /// Launches an Extreme Off-Road Career Championship Cup for the given tier (1..=5).
    pub fn start_extreme_offroad_career_tier(&mut self, tier: u32) {
        let (cup_name, track_ids) = match tier {
            1 => (
                "Desert Sand Sprint Series (Tier 1)",
                vec![
                    "sahara_dune_crossing".to_string(),
                    "dirt_figure_eight".to_string(),
                    "atacama_sand_basin".to_string(),
                    "glamis_dunes".to_string(),
                    "crandon_short_course".to_string(),
                ],
            ),
            2 => (
                "Red Rock Canyon Raid (Tier 2)",
                vec![
                    "red_rock_canyon".to_string(),
                    "mud_slough_arena".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "sahara_dune_crossing".to_string(),
                    "dirt_figure_eight".to_string(),
                    "atacama_sand_basin".to_string(),
                    "crandon_short_course".to_string(),
                ],
            ),
            3 => (
                "Arctic Glacial Challenge (Tier 3)",
                vec![
                    "arctic_frozen_lake".to_string(),
                    "alpine_snow_ridge".to_string(),
                    "rovaniemi_ice_ring".to_string(),
                    "red_rock_canyon".to_string(),
                    "mud_slough_arena".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "sahara_dune_crossing".to_string(),
                    "dirt_figure_eight".to_string(),
                    "crandon_short_course".to_string(),
                ],
            ),
            4 => (
                "Supercross & Mud Masters (Tier 4)",
                vec![
                    "supercross_stadium_arena".to_string(),
                    "gravel_quarry_chasm".to_string(),
                    "louisiana_mud_swampland".to_string(),
                    "arctic_frozen_lake".to_string(),
                    "alpine_snow_ridge".to_string(),
                    "red_rock_canyon".to_string(),
                    "mud_slough_arena".to_string(),
                    "rovaniemi_ice_ring".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "sahara_dune_crossing".to_string(),
                ],
            ),
            _ => (
                "Extreme Off-Road Ultimate Championship (Tier 5)",
                vec![
                    "monster_colosseum".to_string(),
                    "glacier_crest_pass".to_string(),
                    "stunt_city_megastructure".to_string(),
                    "supercross_stadium_arena".to_string(),
                    "gravel_quarry_chasm".to_string(),
                    "arctic_frozen_lake".to_string(),
                    "alpine_snow_ridge".to_string(),
                    "louisiana_mud_swampland".to_string(),
                    "red_rock_canyon".to_string(),
                    "rovaniemi_ice_ring".to_string(),
                    "baja_500_desert_scrub".to_string(),
                    "sahara_dune_crossing".to_string(),
                ],
            ),
        };

        let round_laps: Vec<Option<u32>> = track_ids
            .iter()
            .map(|id| {
                Some(match id.as_str() {
                    "supercross_stadium_arena" | "arctic_frozen_lake" | "dirt_figure_eight" => 5,
                    "rovaniemi_ice_ring"
                    | "louisiana_mud_swampland"
                    | "red_rock_canyon"
                    | "gravel_quarry_chasm"
                    | "alpine_snow_ridge"
                    | "glacier_crest_pass" => 4,
                    _ => 3,
                })
            })
            .collect();

        let active_session = self.active_career_progress.active_championship.clone().filter(|s| {
            !s.is_completed && s.tier == tier
        });

        let champ = if let Some(mut existing) = active_session {
            if !self.active_career_progress.career_rivals.is_empty() {
                existing.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            }
            existing
        } else {
            let mut c = ChampionshipSession::new(
                cup_name,
                PointSystem::FiaStandard { fastest_lap_bonus: false },
                track_ids,
                3,
                &[
                    ("player", "Player", "Sand Rail Dynamics"),
                    ("wyatt_cole", "Wyatt 'Dust Devil' Cole", "Mojave Sandworks"),
                    ("jaxson_rivera", "Jaxson 'Baja King' Rivera", "Baja Trophy Racing"),
                    ("astrid_lindholm", "Astrid 'Ice Queen' Lindholm", "Nordic Glacier Works"),
                    ("bubba_beauregard", "Bubba 'Mud Slinger' Beauregard", "Bayou Heavy Traction"),
                    ("travis_mcgrath", "Travis 'Nitro' McGrath", "Redline Freestyle"),
                    ("roxie_vance", "Roxie 'Rock Hound' Vance", "Canyon Crawler Team"),
                    ("sven_lindqvist", "Sven 'Blizzard' Lindqvist", "Arctic Circle Rally"),
                    ("cruz_morales", "Cruz 'Chasm Jumper' Morales", "Quarry Stunt Squad"),
                ],
            )
            .with_round_laps(round_laps);
            if !self.active_career_progress.career_rivals.is_empty() {
                c.update_from_career_rivals(tier, &self.active_career_progress.career_rivals);
            } else {
                let rivals = c
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .map(|s| {
                        let char_def = DriverCharacter::find_global(&s.driver_id);
                        let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                        CareerRivalEntry {
                            driver_id: s.driver_id.clone(),
                            driver_name: s.driver_name.clone(),
                            style,
                            tier: DriverTier::from_u8(tier.clamp(1, 5) as u8),
                        }
                    })
                    .collect();
                self.active_career_progress.career_rivals = rivals;
            }
            c
        };
        let prev_selected = self.selected_car_model_id;
        self.switch_to_extreme_offroad();
        self.game_mode = GameMode::Career;

        let selected_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == "extreme_offroad" && m.tier == tier as u8 && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("extreme_offroad", tier as u8)
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                crate::catalog::get_models_for_module_and_tier("extreme_offroad", tier as u8)
                    .into_iter()
                    .next()
            });

        if let Some(model) = selected_model {
            self.active_career_progress.ensure_car(model.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(model.id);
            self.car_choice = model.base_car_choice;
            self.current_visual_type = model.visual_type;
            self.free_car_selection = true;
        } else {
            self.car_choice = CarChoice::SandRail;
        }
        self.championship_session = Some(champ.with_tier(tier));
        self.active_career_progress.active_championship = self.championship_session.clone();
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert("extreme_offroad".to_string(), self.active_career_progress.clone());

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }
        self.init_race();
    }

    /// Starts a full NASCAR Cup Series Championship Season.
    pub fn start_nascar_championship(&mut self) {
        let champ = ChampionshipSession::new(
            "NASCAR Cup Series Championship 2026",
            PointSystem::NascarCup { stage_win_bonus: true },
            vec![
                "daytona_superspeedway".to_string(),
                "talladega_superspeedway".to_string(),
                "eldora_speedway".to_string(),
                "iowa_speedway".to_string(),
                "indianapolis_motor_speedway".to_string(),
                "charlotte_motor_speedway".to_string(),
                "darlington_raceway".to_string(),
                "bristol_motor_speedway".to_string(),
                "martinsville_speedway".to_string(),
                "road_america".to_string(),
                "chicago_street_course".to_string(),
                "watkins_glen_nascar".to_string(),
            ],
            4,
            &[
                ("player", "Player", "Apex Stock Car"),
                ("dale_vance", "Dale 'The Intimidator' Vance", "Richard Childress Racing"),
                ("chase_gordon", "Chase 'Rainbow' Gordon", "Hendrick Motorsports"),
                ("richard_pettyfield", "Richard 'The King' Pettyfield", "Petty Enterprises"),
                ("rowdy_busch", "Rowdy 'Wild Thing' Busch", "Joe Gibbs Racing"),
                ("jimmie_johnson", "Jimmie 'Seven-Time' Johnson", "Hendrick Motorsports"),
                ("tony_stewart", "Tony 'Smoke' Stewart", "Stewart-Haas Racing"),
                ("bobby_allison", "Bobby 'Alabama' Allison", "Alabama Gang"),
                ("bubba_wallace", "Bubba 'The Rocket' Wallace", "23XI Racing"),
                ("joey_logano", "Joey 'Sliced Bread' Logano", "Team Penske"),
                ("bill_elliott", "Bill 'Awesome Bill' Elliott", "Melling Racing"),
                ("cale_yarborough", "Cale 'The Iron Man' Yarborough", "Junior Johnson Racing"),
            ],
        );
        self.switch_to_nascar();
        self.championship_session = Some(champ.with_tier(5));
        self.selected_car_model_id = Some("nascar_corvette_ta1");
        self.active_career_progress.ensure_car("nascar_corvette_ta1");
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.free_car_selection = true;
        self.init_race();
    }

    /// Starts a full Extreme Off-Road & Stunt Arenas Championship Season.
    pub fn start_extreme_offroad_championship(&mut self) {
        let track_ids = vec![
            "sahara_dune_crossing".to_string(),
            "dirt_figure_eight".to_string(),
            "atacama_sand_basin".to_string(),
            "red_rock_canyon".to_string(),
            "mud_slough_arena".to_string(),
            "baja_500_desert_scrub".to_string(),
            "arctic_frozen_lake".to_string(),
            "alpine_snow_ridge".to_string(),
            "rovaniemi_ice_ring".to_string(),
            "supercross_stadium_arena".to_string(),
            "gravel_quarry_chasm".to_string(),
            "louisiana_mud_swampland".to_string(),
            "monster_colosseum".to_string(),
            "glacier_crest_pass".to_string(),
            "stunt_city_megastructure".to_string(),
        ];
        let round_laps: Vec<Option<u32>> = track_ids
            .iter()
            .map(|id| {
                Some(match id.as_str() {
                    "supercross_stadium_arena" | "arctic_frozen_lake" | "dirt_figure_eight" => 5,
                    "rovaniemi_ice_ring"
                    | "louisiana_mud_swampland"
                    | "red_rock_canyon"
                    | "gravel_quarry_chasm"
                    | "alpine_snow_ridge"
                    | "glacier_crest_pass" => 4,
                    _ => 3,
                })
            })
            .collect();

        let champ = ChampionshipSession::new(
            "Extreme Off-Road World Series 2026",
            PointSystem::FiaStandard { fastest_lap_bonus: false },
            track_ids,
            3,
            &[
                ("player", "Player", "Sand Rail Dynamics"),
                ("wyatt_cole", "Wyatt 'Dust Devil' Cole", "Mojave Sandworks"),
                ("jaxson_rivera", "Jaxson 'Baja King' Rivera", "Baja Trophy Racing"),
                ("astrid_lindholm", "Astrid 'Ice Queen' Lindholm", "Nordic Glacier Works"),
                ("bubba_beauregard", "Bubba 'Mud Slinger' Beauregard", "Bayou Heavy Traction"),
                ("travis_mcgrath", "Travis 'Nitro' McGrath", "Redline Freestyle"),
                ("roxie_vance", "Roxie 'Rock Hound' Vance", "Canyon Crawler Team"),
                ("sven_lindqvist", "Sven 'Blizzard' Lindqvist", "Arctic Circle Rally"),
                ("cruz_morales", "Cruz 'Chasm Jumper' Morales", "Quarry Stunt Squad"),
            ],
        )
        .with_round_laps(round_laps);
        self.switch_to_extreme_offroad();
        self.championship_session = Some(champ.with_tier(1));
        self.init_race();
    }

    pub fn advance_championship_round(&mut self) {
        let next_track_id = self
            .championship_session
            .as_ref()
            .and_then(|champ| champ.current_track_id().map(|s| s.to_string()));
        let has_session = self.championship_session.is_some();

        if has_session {
            if let Some(track_id) = next_track_id {
                self.track_choice = self.track_manager.track_choice_for_slug(&track_id);
                self.track = self
                    .track_manager
                    .load_track_by_slug(&track_id)
                    .unwrap_or_else(|_| tdrace_core::track::presets::classic_grand_prix());
                self.init_race();
            } else {
                if self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                    let tier = self.active_career_progress.level.clamp(1, 5);
                    let calendar = crate::ui::gt_default_calendar(tier);
                    self.career_hub_focus = CareerHubFocus::Tabs;
                    self.state = GameState::CareerHub {
                        selected_tier: tier,
                        selected_slot: 0,
                        calendar_tracks: calendar,
                        showing_standings: false,
                    };
                } else {
                    self.championship_session = None;
                    self.state = GameState::Menu;
                }
            }
        } else {
            if self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                let tier = self.active_career_progress.level.clamp(1, 5);
                let calendar = crate::ui::gt_default_calendar(tier);
                self.career_hub_focus = CareerHubFocus::Tabs;
                self.state = GameState::CareerHub {
                    selected_tier: tier,
                    selected_slot: 0,
                    calendar_tracks: calendar,
                    showing_standings: false,
                };
            } else {
                self.state = GameState::Menu;
            }
        }
    }

    /// Returns the maximum allowed participants based on the circuit's starting grid slots.
    pub fn max_grid_participants(&self) -> usize {
        if self.track.grid_positions.is_empty() {
            self.track.default_grid_count()
        } else {
            self.track.grid_positions.len()
        }
    }

    /// Returns the maximum allowed bot count for the active game mode.
    pub fn max_bots(&self) -> usize {
        let human_count = if self.is_split_screen() { 2 } else { 1 };
        self.max_grid_participants().saturating_sub(human_count).max(1)
    }

    /// Cycles the casual AI difficulty tier between Rookie, Amateur, Contender, Pro, and Legend.
    pub fn cycle_casual_ai_difficulty(&mut self) {
        let next_tier = match self.casual_ai_difficulty {
            DriverTier::Rookie => DriverTier::Amateur,
            DriverTier::Amateur => DriverTier::Contender,
            DriverTier::Contender => DriverTier::Pro,
            DriverTier::Pro => DriverTier::Legend,
            DriverTier::Legend => DriverTier::Rookie,
        };
        self.set_casual_ai_difficulty(next_tier);
    }

    /// Sets the target difficulty tier for casual races and rebuilds grid tiers.
    pub fn set_casual_ai_difficulty(&mut self, tier: DriverTier) {
        self.casual_ai_difficulty = tier;
        self.prev_casual_ai_difficulty = tier;
        if self.game_mode != GameMode::Career && self.championship_session.is_none() {
            let human_count = if self.is_split_screen() { 2 } else { 1 };
            let racer_count = human_count + self.num_bots;
            if let Some(pref) = self.modality_preferences.get_mut(&self.game_mode) {
                pref.difficulty = tier;
            } else {
                self.modality_preferences.insert(
                    self.game_mode,
                    ModalityPreference {
                        racer_count,
                        difficulty: tier,
                    },
                );
            }
        }
        self.rebuild_roster_participants();
    }

    /// Sets bot count and records preference for active non-career modality.
    pub fn set_num_bots(&mut self, bots: usize) {
        self.num_bots = bots;
        self.prev_num_bots = bots;
        self.update_active_modality_racer_count();
        self.rebuild_roster_participants();
    }

    /// Records the current racer count into modality preferences.
    pub fn update_active_modality_racer_count(&mut self) {
        if self.game_mode != GameMode::Career && self.championship_session.is_none() {
            let human_count = if self.is_split_screen() { 2 } else { 1 };
            let racer_count = human_count + self.num_bots;
            if let Some(pref) = self.modality_preferences.get_mut(&self.game_mode) {
                pref.racer_count = racer_count;
            } else {
                self.modality_preferences.insert(
                    self.game_mode,
                    ModalityPreference {
                        racer_count,
                        difficulty: self.casual_ai_difficulty,
                    },
                );
            }
            self.prev_num_bots = self.num_bots;
        }
    }

    /// Explicitly updates saved preference for a game mode.
    pub fn update_modality_preference(&mut self, mode: GameMode, racer_count: usize, difficulty: DriverTier) {
        self.modality_preferences.insert(
            mode,
            ModalityPreference {
                racer_count,
                difficulty,
            },
        );
        if self.game_mode == mode {
            let human_count = if self.is_split_screen() { 2 } else { 1 };
            let max_grid = self.max_grid_participants();
            let clamped = racer_count.clamp(human_count, max_grid);
            let min_bots = if self.is_split_screen() { 0 } else { 1 };
            self.num_bots = clamped.saturating_sub(human_count).max(min_bots);
            self.casual_ai_difficulty = difficulty;
            self.prev_num_bots = self.num_bots;
            self.prev_casual_ai_difficulty = difficulty;
            self.rebuild_roster_participants();
        }
    }

    /// Returns the remembered preference for a game mode, if any.
    pub fn modality_preference(&self, mode: GameMode) -> Option<ModalityPreference> {
        self.modality_preferences.get(&mode).copied()
    }

    /// Reconstructs participant cars, trackers, and AI drivers for the active roster.
    pub fn rebuild_roster_participants(&mut self) {
        if let Some(champ) = &self.championship_session {
            let max_grid = self.max_grid_participants();
            let human_count = if self.is_split_screen() { 2 } else { 1 };
            let bot_count = champ.standings.iter().filter(|s| s.driver_id != "player").count();
            self.num_bots = bot_count.min(max_grid.saturating_sub(human_count));
        }

        let total_cars = if self.is_time_attack {
            1
        } else if self.is_split_screen() {
            (2 + self.num_bots).min(self.max_grid_participants())
        } else {
            (1 + self.num_bots).min(self.max_grid_participants())
        };

        self.cars.clear();
        self.car_visual_types.clear();
        self.car_model_ids.clear();
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

        let player_car_choice = self.active_player_car_choice();

        let human_count = if self.is_split_screen() { 2 } else { 1 };
        if !self.is_time_attack && total_cars > human_count {
            let target_opponents = total_cars - human_count;
            let module_opponents: Vec<DriverCharacter> = match effective_module {
                "classic" => ClassicGameModule::new().drivers(),
                "gt" | "gt_challenge" => GtWorldChallengeModule::new().drivers(),
                "rally" => RallyGameModule::new().drivers(),
                "kart" => KartGameModule::new().drivers(),
                "nascar" => NascarGameModule::new().drivers(),
                "extreme_offroad" => ExtremeOffRoadModule::new().drivers(),
                _ => Vec::new(),
            };

            if let Some(champ) = &self.championship_session {
                let mut champ_opponents = Vec::new();
                for (idx, entry) in champ
                    .standings
                    .iter()
                    .filter(|s| s.driver_id != "player")
                    .enumerate()
                {
                    if let Some(d) = module_opponents.iter().find(|d| d.id == entry.driver_id) {
                        champ_opponents.push(d.clone());
                    } else if let Some(d) = DriverCharacter::find_global(&entry.driver_id) {
                        champ_opponents.push(d);
                    } else {
                        let static_id: &'static str = Box::leak(entry.driver_id.clone().into_boxed_str());
                        let static_name: &'static str = Box::leak(entry.driver_name.clone().into_boxed_str());
                        let scheme = CarColorScheme::from_index((idx + 1) % 9);
                        let style = if let Some(style_str) = entry.ai_style.as_deref().or(entry.ai_character.as_deref()) {
                            DrivingStyle::from_str_lossy(style_str)
                        } else {
                            DrivingStyle::ALL[idx % DrivingStyle::ALL.len()]
                        };
                        champ_opponents.push(DriverCharacter {
                            id: static_id,
                            name: static_name,
                            alias: static_name,
                            bio: "Championship contender battling for the season crown.",
                            preferred_car: player_car_choice,
                            color_scheme: scheme,
                            offsets: DriverPersonalityOffsets::ZERO,
                            style,
                            favorite_cars: &[],
                        });
                    }
                }
                if !champ_opponents.is_empty() {
                    self.opponent_drivers = champ_opponents.into_iter().take(target_opponents).collect();
                }
            } else if self.game_mode == GameMode::Career {
                let rivals = self.active_career_progress.ensure_career_rivals_with_pool(&module_opponents, target_opponents, seed).to_vec();
                let mut resolved_drivers = Vec::new();
                let mut resolved_tiers = Vec::new();
                for r in rivals.iter().take(target_opponents) {
                    if let Some(c) = r.resolve_character() {
                        resolved_drivers.push(c);
                        resolved_tiers.push(r.tier);
                    }
                }
                self.opponent_drivers = resolved_drivers;
                self.opponent_tiers = resolved_tiers;
            } else {
                // Casual races (Quick Race / StandardRace, Custom Race / ExperimentalRace, SplitScreen, Multiplayer):
                // Sample across the entire global 72-driver pool with uniform 1/6 driving style distribution.
                self.opponent_drivers = DriverCharacter::sample_casual_race_opponents(target_opponents, seed);
            }
        }

        let casual_tiers = if self.championship_session.is_none() && self.game_mode != GameMode::Career {
            self.casual_ai_difficulty.sample_grid_tiers(self.opponent_drivers.len(), seed.wrapping_add(888))
        } else {
            Vec::new()
        };
        if self.game_mode != GameMode::Career && self.championship_session.is_none() {
            self.opponent_tiers = casual_tiers.clone();
        }

        // Resolve the real or fantasy car model
        let mut player_model = if effective_module != "classic" {
            self.selected_car_model_id
                .and_then(crate::catalog::find_model_by_id)
                .filter(|m| m.module_id == effective_module && m.base_car_choice == player_car_choice)
                .or_else(|| {
                    crate::catalog::get_models_for_module(effective_module)
                        .into_iter()
                        .find(|m| {
                            m.base_car_choice == player_car_choice
                                && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode())
                        })
                        .or_else(|| {
                            crate::catalog::get_models_for_module(effective_module)
                                .into_iter()
                                .find(|m| m.base_car_choice == player_car_choice)
                        })
                })
        } else {
            if self.free_car_selection {
                self.selected_car_model_id
                    .and_then(crate::catalog::find_model_by_id)
                    .filter(|m| m.module_id == "classic" && m.base_car_choice == player_car_choice)
                    .or_else(|| Some(crate::catalog::get_classic_model_for_category(player_car_choice.category())))
            } else {
                Some(crate::catalog::get_classic_model_for_category(self.track.car_category))
            }
        };

        // Fallback guarantee: In standard race / non-free selection, never allow a None model if a catalog model exists
        if player_model.is_none() && !self.free_car_selection {
            player_model = crate::catalog::get_models_for_module(effective_module)
                .into_iter()
                .find(|m| m.base_car_choice == player_car_choice || m.base_car_choice.category() == self.track.car_category)
                .or_else(|| crate::catalog::get_models_for_module(effective_module).into_iter().next())
                .or_else(|| Some(crate::catalog::get_classic_model_for_category(self.track.car_category)));
        }

        let mut base_config = match player_car_choice {
            CarChoice::GT4Clubsport => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: false,
                    gt_wing: true,
                    diffuser: false,
                };
                GtWorldChallengeModule::car_gt4_clubsport()
            }
            CarChoice::GT3Car => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                GtWorldChallengeModule::car_gt3_evo()
            }
            CarChoice::GT2Biturbo => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                GtWorldChallengeModule::car_gt2_biturbo()
            }
            CarChoice::GT1Legend => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                GtWorldChallengeModule::car_gt1_legend()
            }
            CarChoice::HypercarPrototype => {
                self.current_visual_type = VehicleVisualType::TouringGT {
                    widebody: true,
                    gt_wing: true,
                    diffuser: true,
                };
                GtWorldChallengeModule::car_hypercar_prototype()
            }
            CarChoice::RallyCar => {
                self.current_visual_type = VehicleVisualType::RallyHatch {
                    roof_scoop: true,
                    mudflaps: true,
                    large_wing: true,
                };
                if self.active_module_id == "classic" {
                    ClassicGameModule::car_classic_rally()
                } else {
                    RallyGameModule::car_wrc_rally()
                }
            }
            CarChoice::Kart => {
                self.current_visual_type = VehicleVisualType::GoKart {
                    exposed_driver: true,
                    side_bumpers: true,
                };
                if self.active_module_id == "classic" {
                    ClassicGameModule::car_classic_kart()
                } else {
                    KartGameModule::car_shifter_kart()
                }
            }
            CarChoice::StockCar => {
                self.current_visual_type = VehicleVisualType::StockCar {
                    tall_wing: self.active_module_id == "classic",
                    roof_fins: true,
                    window_net: true,
                };
                if self.active_module_id == "classic" {
                    ClassicGameModule::car_classic_nascar()
                } else {
                    NascarGameModule::car_stock_car()
                }
            }
            CarChoice::SandRail => {
                self.current_visual_type = VehicleVisualType::SandRail {
                    lightbar: true,
                    whip_antenna: true,
                    paddle_tires: true,
                };
                if self.active_module_id == "classic" {
                    ClassicGameModule::car_classic_offroad()
                } else {
                    ExtremeOffRoadModule::car_sand_rail()
                }
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
                if self.active_module_id == "classic" {
                    ClassicGameModule::car_classic_gt()
                } else {
                    self.config.get_car_config(player_car_choice)
                }
            }
        };

        let (player_car_title, player_model_id, player_visual_type) = if let Some(pm) = player_model {
            self.selected_car_model_id = Some(pm.id);
            self.current_visual_type = pm.visual_type;
            base_config = pm.to_car_config();
            (pm.name.to_string(), Some(pm.id), pm.visual_type)
        } else {
            (player_car_choice.title().to_string(), None, self.current_visual_type)
        };
        base_config.assists = self.assist_profile.to_config();

        let current_tier: u8 = if effective_module == "classic" {
            1
        } else if let Some(champ) = &self.championship_session {
            (champ.tier as u8).clamp(1, 5)
        } else if let Some(pm) = player_model {
            pm.tier.clamp(1, 5)
        } else if self.game_mode == GameMode::Career {
            (self.active_career_progress.level as u8).clamp(1, 5)
        } else {
            self.active_player_car_tier().clamp(1, 5)
        };

        if self.championship_session.is_some() {
            self.opponent_tiers = self
                .opponent_drivers
                .iter()
                .map(|character| {
                    self.championship_session
                        .as_ref()
                        .and_then(|champ| {
                            champ
                                .standings
                                .iter()
                                .find(|s| s.driver_id == character.id)
                                .and_then(|s| s.ai_tier)
                                .map(DriverTier::from_u8)
                        })
                        .unwrap_or_else(|| DriverTier::from_u8(current_tier))
                })
                .collect();
        }

        let category_models = if effective_module == "classic" {
            if self.game_mode == GameMode::ExperimentalRace {
                if let Some(pm) = player_model {
                    vec![pm]
                } else {
                    vec![crate::catalog::get_classic_model_for_category(player_car_choice.category())]
                }
            } else if self.free_car_selection {
                crate::catalog::get_models_for_module("classic")
            } else if let Some(pm) = player_model {
                vec![pm]
            } else {
                vec![crate::catalog::get_classic_model_for_category(self.track.car_category)]
            }
        } else if let Some(pm) = player_model {
            let models = crate::catalog::get_models_for_category(pm.module_id, pm.category_name);
            if !models.is_empty() {
                models
            } else {
                let tier_models = crate::catalog::get_models_for_module_and_tier(effective_module, current_tier);
                if !tier_models.is_empty() {
                    tier_models
                } else {
                    vec![pm]
                }
            }
        } else if !self.free_car_selection {
            let fallback_mod = crate::catalog::get_models_for_module(effective_module);
            if !fallback_mod.is_empty() {
                fallback_mod
            } else {
                vec![crate::catalog::get_classic_model_for_category(self.track.car_category)]
            }
        } else {
            Vec::new()
        };

        let track_id = self.track_choice_id();

        let mut participants = Vec::new();

        // 1. Player participant
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

        let player_car_title_clone = player_car_title.clone();
        participants.push(GridParticipant {
            is_player: true,
            bot_index: None,
            name: self.active_profile.name.clone(),
            alias: self.active_profile.alias.clone(),
            country: self.active_profile.country.clone(),
            car_title: player_car_title,
            car_choice: player_car_choice,
            model_id: player_model_id,
            color_scheme: self.player_effective_color_scheme(),
            best_lap: player_best_lap,
            best_circuit_time: player_best_circuit,
            random_seed: player_seed,
            driver_tier: None,
        });

        // 1b. Player 2 participant for Split Screen
        if self.is_split_screen() {
            let p2_seed = seed.wrapping_mul(0x517CC1B727220A95);
            let p2_scheme = if self.active_profile.color_scheme == CarColorScheme::from_index(1) {
                CarColorScheme::from_index(0)
            } else {
                CarColorScheme::from_index(1)
            };
            participants.push(GridParticipant {
                is_player: true,
                bot_index: None,
                name: "PLAYER 2".to_string(),
                alias: "P2".to_string(),
                country: Some("ARC".to_string()),
                car_title: player_car_title_clone,
                car_choice: player_car_choice,
                model_id: player_model_id,
                color_scheme: p2_scheme,
                best_lap: None,
                best_circuit_time: None,
                random_seed: p2_seed,
                driver_tier: None,
            });
        }

        let player_primary = participants.first().map(|p| p.color_scheme.primary);

        // 2. AI Opponent participants
        for (bot_idx, character) in self.opponent_drivers.iter().enumerate() {
            let bot_hof = self
                .hof_entries
                .iter()
                .find(|e| e.player_name.eq_ignore_ascii_case(character.name));
            let bot_best_lap = bot_hof.and_then(|e| e.best_lap);
            let bot_best_circuit = bot_hof.map(|e| e.total_time);
            let bot_seed = seed.wrapping_add((bot_idx as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15));

            let bot_tier = if let Some(champ) = &self.championship_session {
                champ
                    .standings
                    .iter()
                    .find(|s| s.driver_id == character.id)
                    .and_then(|s| s.ai_tier)
                    .map(DriverTier::from_u8)
                    .unwrap_or_else(|| DriverTier::from_u8(current_tier))
            } else if self.game_mode == GameMode::Career {
                self.opponent_tiers.get(bot_idx).copied().unwrap_or_else(|| DriverTier::from_u8(current_tier))
            } else {
                casual_tiers.get(bot_idx).copied().unwrap_or(self.casual_ai_difficulty)
            };

            let (bot_car_choice, bot_car_title, bot_model_id, bot_scheme) = if !category_models.is_empty() {
                let favorite_model_opt = character
                    .favorite_car_for_discipline_and_tier(effective_module, bot_tier.to_u8())
                    .and_then(|fav_id| category_models.iter().copied().find(|m| m.id == fav_id));

                let bot_model = favorite_model_opt.unwrap_or_else(|| {
                    category_models[bot_idx % category_models.len()]
                });
                let scheme = Self::resolve_bot_color_scheme(
                    character.color_scheme,
                    Some(bot_model),
                    player_primary,
                    &participants,
                    bot_idx,
                );
                (bot_model.base_car_choice, bot_model.name.to_string(), Some(bot_model.id), scheme)
            } else {
                let choice = match self.game_mode {
                    GameMode::ExperimentalRace => player_car_choice,
                    GameMode::Career => self.resolve_predefined_car(),
                    _ => {
                        if self.championship_session.is_some() {
                            self.resolve_predefined_car()
                        } else if self.free_car_selection {
                            character.preferred_car
                        } else if self.random_car_assignment {
                            self.sample_random_opponent_car(bot_idx, bot_seed)
                        } else {
                            self.resolve_predefined_car()
                        }
                    }
                };
                let scheme = Self::resolve_bot_color_scheme(
                    character.color_scheme,
                    None,
                    player_primary,
                    &participants,
                    bot_idx,
                );
                (choice, choice.title().to_string(), None, scheme)
            };

            participants.push(GridParticipant {
                is_player: false,
                bot_index: Some(bot_idx),
                name: character.name.to_string(),
                alias: character.alias.to_string(),
                country: None,
                car_title: bot_car_title,
                car_choice: bot_car_choice,
                model_id: bot_model_id,
                color_scheme: bot_scheme,
                best_lap: bot_best_lap,
                best_circuit_time: bot_best_circuit,
                random_seed: bot_seed,
                driver_tier: Some(bot_tier),
            });
        }

        if !self.is_time_attack && total_cars > 1 {
            if !self.is_split_screen() {
                let is_successive_championship_round = self
                    .championship_session
                    .as_ref()
                    .map(|champ| !champ.is_new())
                    .unwrap_or(false);

                if is_successive_championship_round {
                    let champ = self.championship_session.as_ref().unwrap();
                    participants.sort_by(|a, b| {
                        let id_a = if a.is_player {
                            "player"
                        } else {
                            a.bot_index
                                .and_then(|idx| self.opponent_drivers.get(idx))
                                .map(|d| d.id)
                                .unwrap_or("")
                        };
                        let id_b = if b.is_player {
                            "player"
                        } else {
                            b.bot_index
                                .and_then(|idx| self.opponent_drivers.get(idx))
                                .map(|d| d.id)
                                .unwrap_or("")
                        };
                        let rank_a = champ
                            .standings
                            .iter()
                            .position(|s| s.driver_id == id_a)
                            .unwrap_or(usize::MAX);
                        let rank_b = champ
                            .standings
                            .iter()
                            .position(|s| s.driver_id == id_b)
                            .unwrap_or(usize::MAX);

                        rank_a.cmp(&rank_b).then_with(|| a.cmp_grid_priority(b))
                    });
                } else {
                    participants.sort_by(|a, b| a.cmp_grid_priority(b));
                }
            }
        }
        self.grid_participants = participants;

        let player_slot = if self.is_split_screen() {
            0
        } else {
            self.grid_participants
                .iter()
                .position(|p| p.is_player)
                .unwrap_or(0)
        };

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
        self.car_visual_types.push(player_visual_type);
        self.color_schemes.push(self.player_effective_color_scheme());
        self.car_model_ids.push(player_model_id);
        self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));

        if self.is_split_screen() {
            let p2_slot = 1;
            let grid_pose_p2 = self
                .track
                .grid_positions
                .get(p2_slot)
                .copied()
                .unwrap_or(SpawnPose {
                    position: Vec2::ZERO,
                    angle: 0.0,
                    grid_slot: p2_slot,
                });
            let p2_scheme = if self.active_profile.color_scheme == CarColorScheme::from_index(1) {
                CarColorScheme::from_index(0)
            } else {
                CarColorScheme::from_index(1)
            };
            let mut p2_config = base_config;
            p2_config.assists = self.assist_profile_p2.to_config();
            let p2_car = Car::new(p2_config).with_pose(grid_pose_p2.position, grid_pose_p2.angle);
            self.cars.push(p2_car);
            self.car_visual_types.push(player_visual_type);
            self.color_schemes.push(p2_scheme);
            self.car_model_ids.push(player_model_id);
            self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));
        }

        for (bot_idx, character) in self.opponent_drivers.iter().enumerate() {
            let bot_slot = self
                .grid_participants
                .iter()
                .position(|p| p.bot_index == Some(bot_idx))
                .unwrap_or(if self.is_split_screen() { bot_idx + 2 } else { bot_idx + 1 });

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

            let bot_participant = self
                .grid_participants
                .iter()
                .find(|p| p.bot_index == Some(bot_idx));

            let (bot_config, bot_visual_type, bot_scheme, bot_model_id) = if let Some(p) = bot_participant {
                if let Some(m) = p.model_id.and_then(crate::catalog::find_model_by_id) {
                    (m.to_car_config(), m.visual_type, p.color_scheme, Some(m.id))
                } else {
                    let cfg = match p.car_choice {
                        CarChoice::SportsCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_gt(),
                        CarChoice::StockCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_nascar(),
                        CarChoice::SandRail if self.active_module_id == "classic" => ClassicGameModule::car_classic_offroad(),
                        CarChoice::Kart if self.active_module_id == "classic" => ClassicGameModule::car_classic_kart(),
                        CarChoice::RallyCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_rally(),
                        CarChoice::SportsCar | CarChoice::DriftCar => self.config.get_car_config(p.car_choice),
                        _ => p.car_choice.config(),
                    };
                    let visual = match p.car_choice {
                        CarChoice::StockCar if self.active_module_id == "classic" => VehicleVisualType::StockCar {
                            tall_wing: true,
                            roof_fins: true,
                            window_net: true,
                        },
                        _ => p.car_choice.visual_type(),
                    };
                    (cfg, visual, p.color_scheme, None)
                }
            } else {
                let cfg = match player_car_choice {
                    CarChoice::SportsCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_gt(),
                    CarChoice::StockCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_nascar(),
                    CarChoice::SandRail if self.active_module_id == "classic" => ClassicGameModule::car_classic_offroad(),
                    CarChoice::Kart if self.active_module_id == "classic" => ClassicGameModule::car_classic_kart(),
                    CarChoice::RallyCar if self.active_module_id == "classic" => ClassicGameModule::car_classic_rally(),
                    CarChoice::SportsCar | CarChoice::DriftCar => self.config.get_car_config(player_car_choice),
                    _ => player_car_choice.config(),
                };
                let visual = match player_car_choice {
                    CarChoice::StockCar if self.active_module_id == "classic" => VehicleVisualType::StockCar {
                        tall_wing: true,
                        roof_fins: true,
                        window_net: true,
                    },
                    _ => player_car_choice.visual_type(),
                };
                (cfg, visual, character.color_scheme, None)
            };

            let bot_car = Car::new(bot_config).with_pose(grid_pose_bot.position, grid_pose_bot.angle);

            self.cars.push(bot_car);
            self.car_visual_types.push(bot_visual_type);
            self.color_schemes.push(bot_scheme);
            self.car_model_ids.push(bot_model_id);
            self.trackers.push(TrackProgressTracker::new(num_cps, num_sectors));
            let bot_tier = if let Some(champ) = &self.championship_session {
                champ
                    .standings
                    .iter()
                    .find(|s| s.driver_id == character.id)
                    .and_then(|s| s.ai_tier)
                    .map(DriverTier::from_u8)
                    .unwrap_or_else(|| DriverTier::from_u8(current_tier))
            } else if self.game_mode == GameMode::Career {
                self.opponent_tiers.get(bot_idx).copied().unwrap_or_else(|| DriverTier::from_u8(current_tier))
            } else {
                self.opponent_tiers.get(bot_idx).copied().unwrap_or(self.casual_ai_difficulty)
            };
            let bot_profile = character.resolve_profile(bot_tier);
            self.ai_drivers.push(BotAiDriver::new(bot_profile));
        }
    }

    /// Initializes or resets the racing circuit, cars, grid spawns, AI drivers, and camera.
    pub fn init_race(&mut self) {
        self.recent_hof_id = None;
        self.recent_congrats = None;
        self.show_hall_of_fame = false;
        self.finished_view = FinishedScreenView::Results;
        self.finished_prev_view = FinishedScreenView::Results;
        self.player_race_stats = PlayerRaceTelemetry::default();
        self.refresh_hof_entries();

        // 1. Build selected track (preserve in-memory track if launched from editor)
        if !self.return_to_editor_on_exit {
            if let Some(champ) = &self.championship_session {
                if let Some(track_id) = champ.current_track_id() {
                    if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                        self.track = t;
                    } else {
                        self.track = self.load_track_for_session(&self.track_choice);
                    }
                } else {
                    self.track = self.load_track_for_session(&self.track_choice);
                }
            } else {
                self.track = self.load_track_for_session(&self.track_choice);
            }
        }

        // If starting a brand new championship, synchronize the starting roster size to the circuit grid slots
        let is_new_championship = self.championship_session.as_ref().map(|c| c.is_new()).unwrap_or(false);
        if is_new_championship {
            let circuit_slots = self.max_grid_participants();
            let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
            let mut fallback_pool = match effective_module {
                "classic" => ClassicGameModule::new().drivers(),
                "gt" | "gt_challenge" => GtWorldChallengeModule::new().drivers(),
                "rally" => RallyGameModule::new().drivers(),
                "kart" => KartGameModule::new().drivers(),
                "nascar" => NascarGameModule::new().drivers(),
                "extreme_offroad" => ExtremeOffRoadModule::new().drivers(),
                _ => Vec::new(),
            };
            let global_all = DriverCharacter::all_across_modules();
            for d in global_all {
                if !fallback_pool.iter().any(|existing| existing.id == d.id) {
                    fallback_pool.push(d);
                }
            }
            if let Some(champ) = &mut self.championship_session {
                champ.sync_initial_grid_slots(circuit_slots, &fallback_pool);
            }
            if self.game_mode == GameMode::Career {
                if let Some(champ) = &self.championship_session {
                    let rivals = champ
                        .standings
                        .iter()
                        .filter(|s| s.driver_id != "player")
                        .map(|s| {
                            let char_def = DriverCharacter::find_global(&s.driver_id);
                            let style = char_def.as_ref().map(|c| c.style).unwrap_or(DrivingStyle::Balanced);
                            CareerRivalEntry {
                                driver_id: s.driver_id.clone(),
                                driver_name: s.driver_name.clone(),
                                style,
                                tier: s.ai_tier.map(DriverTier::from_u8).unwrap_or(DriverTier::Rookie),
                            }
                        })
                        .collect();
                    self.active_career_progress.career_rivals = rivals;
                }
            }
        }

        // Predefined balanced lap count from track (or championship override)
        self.total_laps = if let Some(champ) = &self.championship_session {
            if let Some(round_laps) = champ.current_round_laps() {
                round_laps
            } else if champ.laps_per_round > 0 {
                champ.laps_per_round
            } else {
                self.track.default_laps
            }
        } else {
            self.track.default_laps
        };

        // 2. Setup camera
        self.camera.setup_for_track(&self.track);
        self.camera_p2.setup_for_track(&self.track);

        // Apply or initialize modality user preferences (for non-career races with grid participants)
        let is_career = self.game_mode == GameMode::Career || self.championship_session.is_some();
        if !is_career && !self.is_time_attack {
            let human_count = if self.is_split_screen() { 2 } else { 1 };
            let max_grid = self.max_grid_participants();

            // Detect direct explicit assignment to num_bots or casual_ai_difficulty (e.g. from tests or prior calls)
            let explicit_bots = (self.num_bots != self.prev_num_bots).then_some(self.num_bots);
            let explicit_difficulty = (self.casual_ai_difficulty != self.prev_casual_ai_difficulty).then_some(self.casual_ai_difficulty);

            let min_bots = if self.is_split_screen() { 0 } else { 1 };

            if let Some(pref) = self.modality_preferences.get_mut(&self.game_mode) {
                if let Some(bots) = explicit_bots {
                    pref.racer_count = human_count + bots;
                }
                if let Some(diff) = explicit_difficulty {
                    pref.difficulty = diff;
                }
                let target_racers = pref.racer_count.clamp(human_count, max_grid);
                self.num_bots = target_racers.saturating_sub(human_count).max(min_bots);
                self.casual_ai_difficulty = pref.difficulty;
            } else {
                // First initialization of this modality: initialize roster with slots available in grid and difficulty T1 (Rookie)
                let target_difficulty = explicit_difficulty.unwrap_or(DriverTier::Rookie);
                let target_racers = explicit_bots
                    .map(|b| human_count + b)
                    .unwrap_or(max_grid)
                    .clamp(human_count, max_grid);
                self.num_bots = target_racers.saturating_sub(human_count).max(min_bots);
                self.casual_ai_difficulty = target_difficulty;

                self.modality_preferences.insert(
                    self.game_mode,
                    ModalityPreference {
                        racer_count: target_racers,
                        difficulty: target_difficulty,
                    },
                );
            }

            self.prev_num_bots = self.num_bots;
            self.prev_casual_ai_difficulty = self.casual_ai_difficulty;
        }

        // 3. Build participants & cars
        self.rebuild_roster_participants();

        self.fx.clear();
        self.floating_text.clear();
        self.prev_best_sectors.clear();
        self.drift_combo_count = 0;
        self.drift_combo_timer = 0.0;
        self.prev_player_drifting = false;
        self.player_collision_stunt_lockout = 0.0;
        self.player2_collision_stunt_lockout = 0.0;
        self.results.clear();
        self.session_time = 0.0;
        self.accumulator = 0.0;
        self.prev_player_lap = 1;
        self.prev_player_sector = 0;
        self.prev_p2_lap = 1;
        self.prev_p2_sector = 0;
        self.engine_rpm = EngineRpmModel::default();
        self.engine_rpm_p2 = EngineRpmModel::default();
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

        let sound_type = self.resolve_active_sound_type();
        self.audio.set_engine_type(sound_type);

        // Show Starting Grid with selected race participants
        self.starting_grid_focus = StartingGridFocus::LeftSetup;
        self.starting_grid_card_idx = 0;
        self.starting_grid_roster_idx = 0;
        self.state = GameState::StartingGrid;
    }

    /// Pauses the race session and activates the static full-circuit overview camera.
    pub fn pause_race(&mut self) {
        self.audio.stop_all_loops();
        self.state = GameState::Paused;
        self.camera.set_paused_overview();
        if self.is_split_screen() {
            self.camera_p2.set_paused_overview();
        }
    }

    /// Resumes the race session from pause, restoring the active follow driving camera.
    pub fn resume_race(&mut self) {
        self.state = GameState::Racing;
        let player_car = self.cars.first();
        self.camera.resume_from_pause(player_car);
        if self.is_split_screen() {
            let p2_car = self.cars.get(1);
            self.camera_p2.resume_from_pause(p2_car);
        }
    }

    /// Starts an animated screen transition towards a target `GameState`.
    /// When the transition reaches its midpoint (Holding phase, full coverage),
    /// `self.state` is swapped to `target_state`.
    pub fn transition_to(&mut self, target_state: GameState, transition: ScreenTransition) {
        self.pending_state = Some(target_state);
        self.transition = Some(transition);
    }

    /// Helper for a smooth fade transition to target state.
    pub fn transition_fade_to(&mut self, target_state: GameState, duration: f32) {
        self.transition_to(target_state, ScreenTransition::fade(duration));
    }

    /// Helper for an iris wipe transition to target state.
    pub fn transition_iris_to(&mut self, target_state: GameState, duration: f32) {
        self.transition_to(target_state, ScreenTransition::iris(duration));
    }

    /// Helper for a curtain wipe transition to target state.
    pub fn transition_curtain_to(&mut self, target_state: GameState, duration: f32) {
        self.transition_to(target_state, ScreenTransition::curtain(duration));
    }

    /// Helper for a scanline wipe transition to target state.
    pub fn transition_scanline_to(&mut self, target_state: GameState, duration: f32) {
        self.transition_to(target_state, ScreenTransition::scanline(duration));
    }

    /// Returns whether a screen transition is currently active.
    pub fn is_transitioning(&self) -> bool {
        self.transition.as_ref().is_some_and(|t| t.is_active())
    }

    /// Steps any active screen transition by `dt`. Returns `true` if a state swap occurred on this tick.
    pub fn update_transition(&mut self, dt: f32) -> bool {
        let reached_hold = if let Some(ref mut trans) = self.transition {
            trans.update(dt)
        } else {
            false
        };

        let mut swapped = false;
        if reached_hold {
            if let Some(target) = self.pending_state.take() {
                self.apply_transition_target(target);
                swapped = true;
            }
        }

        if self.transition.as_ref().is_some_and(|t| t.is_complete()) {
            self.transition = None;
            self.pending_state = None;
        }

        swapped
    }

    /// Applies the state swap when transition reaches holding point.
    fn apply_transition_target(&mut self, target: GameState) {
        if matches!(self.state, GameState::Garage(_)) {
            self.audio.stop_all_loops();
        }
        match &target {
            GameState::Garage(_) => {
                self.audio.stop_music();
            }
            GameState::Countdown(_) => {
                self.audio.stop_all_loops();
                self.audio.play_sfx(SfxType::UiSelect);
            }
            GameState::ModalitySelect { .. } => {
                if let GameState::ModuleSelect { selected_idx } = self.state {
                    match selected_idx {
                        1 => self.switch_to_classic(),
                        2 => self.switch_to_rally(),
                        3 => self.switch_to_kart(),
                        4 => self.switch_to_gt(),
                        5 => self.switch_to_nascar(),
                        6 => self.switch_to_extreme_offroad(),
                        _ => self.switch_to_classic(),
                    }
                }
                self.audio.play_music(MusicTrack::NeonMenu);
            }
            GameState::Menu => {
                if let GameState::ModuleSelect { selected_idx } = self.state {
                    match selected_idx {
                        1 => self.switch_to_classic(),
                        2 => self.switch_to_rally(),
                        3 => self.switch_to_kart(),
                        4 => self.switch_to_gt(),
                        5 => self.switch_to_nascar(),
                        6 => self.switch_to_extreme_offroad(),
                        _ => self.switch_to_classic(),
                    }
                }
                self.audio.play_music(MusicTrack::NeonMenu);
            }
            GameState::ModuleSelect { .. } => {
                self.audio.play_music(MusicTrack::NeonMenu);
            }
            _ => {}
        }
        self.state = target;
    }

    /// Master update tick called once per frame.
    pub fn update(&mut self) {
        let frame_dt = get_frame_time_safe().min(0.1);
        let sw = screen_width_safe();
        let sh = screen_height_safe();

        // Handle gamepad input updates
        self.input.gamepad.update();

        // Step active screen transition
        self.update_transition(frame_dt);

        // Step active CRT overlay animation
        self.crt_overlay.update(frame_dt);

        // Step active floating text popups
        self.floating_text.update(frame_dt);

        // Step active drift combo decay timer
        if self.drift_combo_timer > 0.0 {
            self.drift_combo_timer -= frame_dt;
            if self.drift_combo_timer <= 0.0 {
                self.drift_combo_count = 0;
            }
        }

        // If a transition is actively covering or holding before the state swap,
        // suppress UI navigation and game interaction.
        if self.transition.as_ref().is_some_and(|t| {
            t.phase == TransitionPhase::Covering || t.phase == TransitionPhase::Holding
        }) {
            return;
        }

        // Handle debug toggles
        self.input.update_debug_toggles();

        // Handle touch controls update
        self.touch.update_from_macroquad(sw, sh, frame_dt);

        // Handle entering garage state: stop music once on entry so engine sound is clear
        let is_in_garage = matches!(self.state, GameState::Garage(_));
        if is_in_garage && !self.in_garage_state {
            self.in_garage_state = true;
            self.audio.stop_music();
        } else if !is_in_garage {
            self.in_garage_state = false;
        }

        // Toggle audio: M switches music, S switches other sounds (SFX / engine)
        // (only when not typing, not in Track Manager, and not Ctrl/Cmd hotkey)
        let ctrl_down = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);
        let is_typing_or_tm = ctrl_down
            || matches!(
                self.state,
                GameState::ProfileCreate { .. }
                    | GameState::PlayerRosterManager { active_column: 1, field_idx: 0..=1, .. }
                    | GameState::TrackManager { .. }
                    | GameState::ChampionshipEditor
            )
            || (matches!(self.state, GameState::TrackEditor) && self.editor_modal != EditorModal::None);

        if !is_typing_or_tm && is_key_pressed(KeyCode::M) {
            self.audio.toggle_music();
            if !self.audio.settings.is_music_muted && !self.audio.settings.is_muted && self.audio.current_music.is_none() {
                let track = match self.state {
                    GameState::Racing | GameState::Countdown(_) | GameState::Paused | GameState::Finished => {
                        MusicTrack::NightcallRace
                    }
                    _ => MusicTrack::NeonMenu,
                };
                self.audio.play_music(track);
            }
        }

        let is_wasd_racing = matches!(self.state, GameState::Racing | GameState::Countdown(_))
            && self.input.input_map == cabinet::input::InputMap::wasd_racing();
        if !is_typing_or_tm && !is_wasd_racing && is_key_pressed(KeyCode::S) {
            self.audio.toggle_sfx();
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

        // Toggle Split-Screen Layout (F8 key)
        if self.is_split_screen() && is_key_pressed(KeyCode::F8) {
            self.split_layout = match self.split_layout {
                SplitLayout::Vertical => SplitLayout::Horizontal,
                SplitLayout::Horizontal => SplitLayout::Vertical,
            };
            self.audio.play_sfx(SfxType::UiSelect);
        }

        // Open Championship Editor studio (F11 key)
        if is_key_pressed(KeyCode::F11) {
            self.enter_championship_editor(None);
            return;
        }

        // Handle camera toggle / zoom cycle (Tab key for P1, Gamepad Cam Toggle for P2 in Split Screen, or either in Single Player)
        let is_camera_state = matches!(
            self.state,
            GameState::Racing
                | GameState::Countdown(_)
                | GameState::TrackEditor
        );
        if is_camera_state {
            if self.is_split_screen() {
                if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_cam_toggle_pressed {
                    let lvl = self.cycle_camera_zoom();
                    self.audio.play_sfx(SfxType::UiMove);
                    let lvl_idx = self.camera.current_level_idx + 1;
                    let total_lvls = self.camera.levels.iter().filter(|l| !l.is_overview()).count().max(1);
                    let text = format!("CAMERA: {} ({}/{})", lvl.name.to_uppercase(), lvl_idx, total_lvls);

                    if let Some(pos1) = self.cars.first().map(|c| c.state.position) {
                        self.fx.drift_popups.spawn_text(pos1, &text, Color::new(0.3, 0.9, 1.0, 1.0));
                    }
                    if let Some(pos2) = self.cars.get(1).map(|c| c.state.position) {
                        self.fx.drift_popups.spawn_text(pos2, &text, Color::new(0.3, 0.9, 1.0, 1.0));
                    }
                }
            } else if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_cam_toggle_pressed {
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
                    let lvl = self.cycle_camera_zoom();
                    self.audio.play_sfx(SfxType::UiMove);
                    let car_pos = self
                        .cars
                        .first()
                        .map(|c| c.state.position);
                    if let Some(pos) = car_pos {
                        let lvl_idx = self.camera.current_level_idx + 1;
                        let total_lvls = self.camera.levels.iter().filter(|l| !l.is_overview()).count().max(1);
                        self.fx.drift_popups.spawn_text(
                            pos,
                            &format!("CAMERA: {} ({}/{})", lvl.name.to_uppercase(), lvl_idx, total_lvls),
                            Color::new(0.3, 0.9, 1.0, 1.0),
                        );
                    }
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
            // [1] Toggle Overhead Chevron Indicator
            if is_key_pressed(KeyCode::Key1) {
                self.visibility_options.overhead_chevron = !self.visibility_options.overhead_chevron;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.overhead_chevron { "ON" } else { "OFF" };
                let col = if self.visibility_options.overhead_chevron { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[1] CHEVRON: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[1] OVERHEAD CHEVRON: {}", state_str),
                    is_on: self.visibility_options.overhead_chevron,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [2] Toggle Player Ground Aura / Underglow Disc
            if is_key_pressed(KeyCode::Key2) {
                self.visibility_options.ground_aura = !self.visibility_options.ground_aura;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.ground_aura { "ON" } else { "OFF" };
                let col = if self.visibility_options.ground_aura { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[2] GROUND AURA: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[2] GROUND AURA: {}", state_str),
                    is_on: self.visibility_options.ground_aura,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [3] Toggle Context-Aware Adaptive Visibility
            if is_key_pressed(KeyCode::Key3) {
                self.visibility_options.adaptive_visibility = !self.visibility_options.adaptive_visibility;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.adaptive_visibility { "ON" } else { "OFF" };
                let col = if self.visibility_options.adaptive_visibility { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[3] ADAPTIVE: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[3] ADAPTIVE SCALING: {}", state_str),
                    is_on: self.visibility_options.adaptive_visibility,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [4] Toggle High-Visibility Roof Beacon
            if is_key_pressed(KeyCode::Key4) {
                self.visibility_options.roof_beacon = !self.visibility_options.roof_beacon;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.roof_beacon { "ON" } else { "OFF" };
                let col = if self.visibility_options.roof_beacon { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[4] ROOF BEACON: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[4] ROOF BEACON: {}", state_str),
                    is_on: self.visibility_options.roof_beacon,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [5] Toggle Approaching Curve & Dynamic Braking Helper
            if is_key_pressed(KeyCode::Key5) {
                self.visibility_options.curve_helper = !self.visibility_options.curve_helper;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.curve_helper { "ON" } else { "OFF" };
                let col = if self.visibility_options.curve_helper { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[5] CORNER ASSIST: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[5] CORNER ASSIST: {}", state_str),
                    is_on: self.visibility_options.curve_helper,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [6] Cycle Curve Helper Color Scheme (Traffic -> Synthwave -> Contrast -> Rally)
            if is_key_pressed(KeyCode::Key6) {
                self.visibility_options.curve_color_scheme = self.visibility_options.curve_color_scheme.next();
                self.audio.play_sfx(SfxType::UiMove);
                let scheme_str = self.visibility_options.curve_color_scheme.as_str();
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[6] COLOR: {}", scheme_str), Palette::NEON_GOLD);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[6] COLOR: {}", scheme_str),
                    is_on: true,
                    timer: 2.2,
                    duration: 2.2,
                });
            }

            // [7] Toggle / Cycle CRT Scanlines Post-Processing Overlay
            if is_key_pressed(KeyCode::Key7) || is_key_pressed(KeyCode::F7) {
                let mode = self.cycle_scanline_mode();
                self.audio.play_sfx(SfxType::UiMove);
                let is_on = mode != ScanlineMode::Disabled;
                let col = if is_on { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[7] CRT: {}", mode.label()), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[7] CRT SCANLINES: {}", mode.label().to_uppercase()),
                    is_on,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // [ALT] Toggle In-Race Floating Bot Nameplates (Spec 029)
            if is_key_pressed(KeyCode::LeftAlt) || is_key_pressed(KeyCode::RightAlt) {
                self.visibility_options.bot_nameplates = !self.visibility_options.bot_nameplates;
                self.config.display.bot_nameplates = self.visibility_options.bot_nameplates;
                self.audio.play_sfx(SfxType::UiMove);
                let state_str = if self.visibility_options.bot_nameplates { "ON" } else { "OFF" };
                let col = if self.visibility_options.bot_nameplates { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED };
                if let Some(pos) = self.cars.first().map(|c| c.state.position) {
                    self.fx.drift_popups.spawn_text(pos, &format!("[ALT] NAMES: {}", state_str), col);
                }
                self.visibility_toast = Some(VisibilityToast {
                    text: format!("[ALT] BOT NAMEPLATES: {}", state_str),
                    is_on: self.visibility_options.bot_nameplates,
                    timer: 1.8,
                    duration: 1.8,
                });
            }

            // Update visibility toast timer
            if let Some(toast) = &mut self.visibility_toast {
                toast.timer -= frame_dt;
                if toast.timer <= 0.0 {
                    self.visibility_toast = None;
                }
            }

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
                self.zoom_progressive(zoom_dir, zoom_speed_mult, frame_dt);
            }
        }

        if self.state == GameState::TrackEditor {
            self.update_track_editor(frame_dt);
            return;
        }

        if self.state == GameState::ChampionshipEditor {
            self.update_championship_editor(frame_dt);
            return;
        }

        // Dispatch dedicated profile manager / creation state logic
        if let GameState::ProfileManager { selected_idx } = self.state {
            self.update_profile_manager(selected_idx);
            return;
        }

        if matches!(self.state, GameState::PlayerRosterManager { .. }) {
            if let GameState::PlayerRosterManager {
                selected_idx,
                active_column,
                field_idx,
                input_name,
                input_alias,
                country_idx,
                livery_idx,
                assist_mode,
                cursor_timer,
                status_msg,
            } = std::mem::replace(&mut self.state, GameState::Menu)
            {
                self.update_player_roster_manager(
                    selected_idx,
                    active_column,
                    field_idx,
                    input_name,
                    input_alias,
                    country_idx,
                    livery_idx,
                    assist_mode,
                    cursor_timer,
                    status_msg,
                    frame_dt,
                );
                return;
            }
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

        // Open Controls & Driving Assists Screen (K key)
        if is_key_pressed(KeyCode::K) {
            self.audio.play_sfx(SfxType::UiSelect);
            let from_paused = matches!(self.state, GameState::Racing | GameState::Paused | GameState::Countdown(_));
            self.state = GameState::ControlsHelp(from_paused);
            return;
        }

        // Open Driver Cards Dossier Screen (D key)
        if is_key_pressed(KeyCode::D) && !matches!(self.state, GameState::Garage(_) | GameState::ModalitySelect { .. } | GameState::CareerHub { .. }) {
            self.audio.play_sfx(SfxType::UiSelect);
            let origin = match self.state {
                GameState::StartingGrid => DriverCardsOrigin::StartingGrid,
                GameState::Racing | GameState::Paused | GameState::Countdown(_) => DriverCardsOrigin::Paused,
                _ => DriverCardsOrigin::Menu,
            };
            self.state = GameState::DriverCards(origin);
            return;
        }

        // Cycle Driver Assists Profile (H key for P1, Gamepad Right Stick Click for P1 in single-player or P2 in split-screen)
        if is_key_pressed(KeyCode::H) || (!self.is_split_screen() && self.input.gamepad.snapshot.btn_assist_toggle_pressed) {
            let next_mode = self.assist_profile.next();
            self.set_assist_profile(next_mode);
            self.audio.play_sfx(SfxType::UiMove);
            if let Some(player_car) = self.cars.first() {
                self.fx.drift_popups.spawn_text(
                    player_car.state.position,
                    &format!("ASSISTS: {}", self.assist_profile.short_name()),
                    Color::new(0.3, 0.9, 1.0, 1.0),
                );
            }
        }
        if self.is_split_screen() && self.input.gamepad.snapshot.btn_assist_toggle_pressed {
            self.assist_profile_p2 = self.assist_profile_p2.next();
            self.audio.play_sfx(SfxType::UiMove);
            if let Some(p2_car) = self.cars.get_mut(1) {
                p2_car.config.assists = self.assist_profile_p2.to_config();
                self.fx.drift_popups.spawn_text(
                    p2_car.state.position,
                    &format!("P2 ASSISTS: {}", self.assist_profile_p2.short_name()),
                    Color::new(0.3, 0.9, 1.0, 1.0),
                );
            }
        }

        // Global restart shortcut (R key) during active racing / paused sessions
        if matches!(self.state, GameState::Racing | GameState::Paused | GameState::Countdown(_))
            && is_key_pressed(KeyCode::R)
        {
            self.init_race();
            return;
        }

        match self.state {
            GameState::Menu => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_menu();
            }
            GameState::ModalitySelect { .. } => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_modality_select();
            }
            GameState::CareerHub { .. } => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_career_hub();
            }
            GameState::CareerSelect { selected_idx } => {
                self.audio.play_music(MusicTrack::NeonMenu);
                self.update_career_select(selected_idx);
            }
            GameState::Garage(origin) => {
                self.update_garage(origin, frame_dt);
            }
            GameState::CircuitViewer(origin) => {
                self.update_circuit_viewer(origin, frame_dt);
            }
            GameState::ModuleSelect { .. } => {
                self.update_module_select();
            }
            GameState::ChampionshipStandings => {
                self.update_championship_standings();
            }
            GameState::StartingGrid => {
                self.update_starting_grid();
            }
            GameState::Countdown(ref mut remaining) => {

                *remaining -= frame_dt;

                // Player launch throttle / revs on grid
                if self.game_mode.is_split_screen() {
                    let (p1_ctrl, p2_ctrl) = self.input.poll_split_player_controls(&mut self.filter_p2, frame_dt, 0.0, 0.0);
                    let (rpm1, is_shift1) = self.engine_rpm.update(0.0, p1_ctrl.throttle, 0.0, frame_dt);
                    self.audio.update_engine_telemetry(rpm1, p1_ctrl.throttle, is_shift1, 0.0, self.engine_rpm.current_gear, frame_dt);
                    let (rpm2, is_shift2) = self.engine_rpm_p2.update(0.0, p2_ctrl.throttle, 0.0, frame_dt);
                    self.audio.update_engine_telemetry_p2(rpm2, p2_ctrl.throttle, is_shift2, 0.0, self.engine_rpm_p2.current_gear, frame_dt);
                } else {
                    let kb_ctrl = self.input.poll_player_controls(frame_dt, 0.0);
                    let touch_ctrl = self.touch.poll_controls();
                    let player_ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);
                    let (rpm, is_shift) = self.engine_rpm.update(0.0, player_ctrl.throttle, 0.0, frame_dt);
                    self.audio.update_engine_telemetry(rpm, player_ctrl.throttle, is_shift, 0.0, self.engine_rpm.current_gear, frame_dt);
                }

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
                if self.game_mode.is_split_screen() {
                    if let Some(p2_car) = self.cars.get(1) {
                        self.camera_p2.update(p2_car, frame_dt);
                    }
                }

                if *remaining <= 0.0 {
                    self.audio.play_sfx(SfxType::CountdownHigh);
                    self.state = GameState::Racing;
                }
            }
            GameState::Racing => {
                // If resuming race after paused overview was active, restore driving follow camera
                if self.camera.paused_from_follow.is_some() {
                    let player_car = self.cars.first();
                    self.camera.resume_from_pause(player_car);
                }
                if self.is_split_screen() && self.camera_p2.paused_from_follow.is_some() {
                    let p2_car = self.cars.get(1);
                    self.camera_p2.resume_from_pause(p2_car);
                }

                // Pause trigger (Escape / Pause key or Gamepad Start)
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Pause) || self.input.gamepad.snapshot.btn_start_pressed {
                    self.pause_race();
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
                if self.is_split_screen() {
                    if let Some(p2_car) = self.cars.get(1) {
                        self.camera_p2.update(p2_car, frame_dt);
                    }
                }

                // Check for race finish conditions
                self.check_race_finish();
            }
            GameState::Paused => {
                self.audio.stop_all_loops();
                if self.camera.paused_from_follow.is_none() {
                    self.camera.set_paused_overview();
                }
                if self.is_split_screen() && self.camera_p2.paused_from_follow.is_none() {
                    self.camera_p2.set_paused_overview();
                }

                // If Arcade Settings Modal is open, handle its updates and return
                if let Some(ref mut modal) = self.settings_modal {
                    let (sw, sh) = (screen_width_safe(), screen_height_safe());
                    let scaler = UiScaler::new(sw, sh);
                    let theme = CabinetTheme::default();
                    let mut ctx = CabinetContext {
                        scaler: &scaler,
                        fonts: &self.fonts,
                        theme: &theme,
                        gamepad: &self.input.gamepad.snapshot,
                        dt: self.accumulator.min(0.1),
                        audio: Some(&self.audio),
                    };

                    let action = modal.update(&mut ctx);
                    if matches!(action, ScreenAction::Pop) {
                        let saved = modal.is_saved;
                        self.close_settings_modal(saved);
                        if saved {
                            self.audio.play_sfx(SfxType::UiSelect);
                        }
                    }
                    return;
                }

                // Open Arcade Settings Modal via O key or Gamepad Y button
                if is_key_pressed(KeyCode::O) || self.input.gamepad.snapshot.btn_y_pressed {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.open_settings_modal();
                    return;
                }

                let (sw, sh) = (screen_width_safe(), screen_height_safe());
                let (_, _, _, _, btn_layout) = crate::ui::pause_menu_layout(sw, sh);

                if NavGrid2D::check_mouse_hover(btn_layout.resume_rect) {
                    if self.pause_nav.focused_col != 0 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.pause_nav.set_focus(0, 0);
                    }
                }
                if NavGrid2D::check_mouse_hover(btn_layout.exit_rect) {
                    if self.pause_nav.focused_col != 1 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.pause_nav.set_focus(1, 0);
                    }
                }

                let resume_clicked = NavGrid2D::check_mouse_click(btn_layout.resume_rect);
                let exit_clicked = NavGrid2D::check_mouse_click(btn_layout.exit_rect);

                // Button cursor navigation (Left / Right / A / D / Up / Down / Gamepad D-pad)
                if is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A)
                    || self.input.gamepad.snapshot.dpad_left_pressed
                    || self.input.gamepad.snapshot.nav_left
                {
                    if self.pause_nav.focused_col != 0 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.pause_nav.set_focus(0, 0);
                    }
                }
                if is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D)
                    || self.input.gamepad.snapshot.dpad_right_pressed
                    || self.input.gamepad.snapshot.nav_right
                {
                    if self.pause_nav.focused_col != 1 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.pause_nav.set_focus(1, 0);
                    }
                }
                if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::W)
                    || self.input.gamepad.snapshot.nav_up
                    || self.input.gamepad.snapshot.nav_down
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    let next_col = 1 - self.pause_nav.focused_col;
                    self.pause_nav.set_focus(next_col, 0);
                }

                self.pause_selected_btn = self.pause_nav.focused_col;

                // Action confirmation on highlighted button or click
                let is_confirmed = is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || self.pause_nav.is_confirmed(
                        self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed,
                    );

                if is_confirmed || resume_clicked || exit_clicked {
                    self.audio.play_sfx(SfxType::UiSelect);
                    let action_idx = if resume_clicked {
                        0
                    } else if exit_clicked {
                        1
                    } else {
                        self.pause_selected_btn
                    };

                    if action_idx == 0 {
                        self.resume_race();
                        return;
                    } else {
                        self.camera.resume_from_pause(None);
                        if self.is_split_screen() {
                            self.camera_p2.resume_from_pause(None);
                        }
                        if self.return_to_editor_on_exit {
                            self.return_to_editor_on_exit = false;
                            self.transition_fade_to(GameState::TrackEditor, 0.35);
                        } else {
                            self.transition_fade_to(GameState::Menu, 0.35);
                        }
                        return;
                    }
                }

                // Direct shortcut triggers
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Pause)
                    || self.input.gamepad.snapshot.btn_start_pressed
                    || resume_clicked
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.resume_race();
                    return;
                } else if is_key_pressed(KeyCode::E)
                    || self.input.gamepad.snapshot.btn_cancel_pressed
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                    || exit_clicked
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.camera.resume_from_pause(None);
                    if self.is_split_screen() {
                        self.camera_p2.resume_from_pause(None);
                    }
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.transition_fade_to(GameState::TrackEditor, 0.35);
                    } else {
                        self.transition_fade_to(GameState::Menu, 0.35);
                    }
                    return;
                }
                if is_key_pressed(KeyCode::K) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::ControlsHelp(true);
                }
            }
            GameState::Finished => {
                self.update_finished_screen();
            }

            GameState::ControlsHelp(from_paused) => {
                if is_key_pressed(KeyCode::Tab)
                    || is_key_pressed(KeyCode::C)
                    || self.input.gamepad.snapshot.btn_x_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.input.cycle_control_preset();
                    let _ = self.input.save_bindings();
                }

                if is_key_pressed(KeyCode::Escape)
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
            | GameState::PlayerRosterManager { .. }
            | GameState::TrackManager { .. }
            | GameState::TrackEditor
            | GameState::ChampionshipEditor => {}

        }
    }

    /// Cycles the selected Starting Grid participant's car model (forward or backward) in modes that allow roster customization,
    /// updating their model, car choice, title, and re-resolving bot color scheme against the rest of the grid.
    pub fn cycle_starting_grid_participant_model(&mut self, next: bool) {
        if !self.game_mode.allows_roster_customization() {
            return;
        }
        let models = crate::catalog::get_models_for_module(self.active_module_id);
        if models.is_empty() {
            return;
        }
        if let Some(p) = self.grid_participants.get(self.starting_grid_roster_idx) {
            let cur_idx = p.model_id
                .and_then(|id| models.iter().position(|m| m.id == id))
                .unwrap_or(0);
            let next_idx = if next {
                (cur_idx + 1) % models.len()
            } else if cur_idx == 0 {
                models.len() - 1
            } else {
                cur_idx - 1
            };
            let next_m = models[next_idx];
            let is_player = p.is_player;
            let bot_idx_opt = p.bot_index;
            let old_scheme = p.color_scheme;
            let other_parts: Vec<GridParticipant> = self.grid_participants
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != self.starting_grid_roster_idx)
                .map(|(_, part)| part.clone())
                .collect();
            let player_primary = other_parts
                .iter()
                .find(|part| part.is_player)
                .map(|part| part.color_scheme.primary);
            let new_scheme = if !is_player {
                Self::resolve_bot_color_scheme(
                    old_scheme,
                    Some(next_m),
                    player_primary,
                    &other_parts,
                    self.starting_grid_roster_idx,
                )
            } else {
                old_scheme
            };

            if let Some(p_mut) = self.grid_participants.get_mut(self.starting_grid_roster_idx) {
                p_mut.model_id = Some(next_m.id);
                p_mut.car_title = next_m.name.to_string();
                p_mut.car_choice = next_m.base_car_choice;
                if !is_player {
                    p_mut.color_scheme = new_scheme;
                }
            }
            self.audio.play_sfx(SfxType::UiMove);
            if is_player {
                self.selected_car_model_id = Some(next_m.id);
                self.car_choice = next_m.base_car_choice;
                self.current_visual_type = next_m.visual_type;
                if let Some(cs) = self.color_schemes.get_mut(0) {
                    *cs = new_scheme;
                }
                if let Some(mid) = self.car_model_ids.get_mut(0) {
                    *mid = Some(next_m.id);
                }
                if let Some(vt) = self.car_visual_types.get_mut(0) {
                    *vt = next_m.visual_type;
                }
                if let Some(car) = self.cars.get_mut(0) {
                    car.config = next_m.to_car_config();
                }
            } else {
                let car_idx = bot_idx_opt
                    .map(|b| if self.is_split_screen() { b + 2 } else { b + 1 })
                    .unwrap_or(self.starting_grid_roster_idx);
                if let Some(cs) = self.color_schemes.get_mut(car_idx) {
                    *cs = new_scheme;
                }
                if let Some(mid) = self.car_model_ids.get_mut(car_idx) {
                    *mid = Some(next_m.id);
                }
                if let Some(vt) = self.car_visual_types.get_mut(car_idx) {
                    *vt = next_m.visual_type;
                }
                if let Some(car) = self.cars.get_mut(car_idx) {
                    car.config = next_m.to_car_config();
                }
            }
        }
    }

    /// Updates input and state progression when in the StartingGrid screen.
    pub fn update_starting_grid(&mut self) {
        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let (btn_x, btn_y, btn_w, btn_h) = crate::ui::starting_grid_launch_button_rect(sw, sh);
        let (g_btn_x, g_btn_y, g_btn_w, g_btn_h) = crate::ui::starting_grid_garage_button_rect(sw, sh);
        let (grid_btn_x, grid_btn_y, grid_btn_w, grid_btn_h) = crate::ui::starting_grid_grid_button_rect(sw, sh);
        let (p_btn_x, p_btn_y, p_btn_w, p_btn_h) = crate::ui::starting_grid_player_card_rect(sw, sh);
        let (c_btn_x, c_btn_y, c_btn_w, c_btn_h) = crate::ui::starting_grid_circuit_card_rect(sw, sh);
        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);
        let launch_btn_clicked = mouse_clicked
            && mx >= btn_x
            && mx <= btn_x + btn_w
            && my >= btn_y
            && my <= btn_y + btn_h;
        let garage_btn_clicked = mouse_clicked
            && mx >= g_btn_x
            && mx <= g_btn_x + g_btn_w
            && my >= g_btn_y
            && my <= g_btn_y + g_btn_h;
        let grid_btn_clicked = mouse_clicked
            && mx >= grid_btn_x
            && mx <= grid_btn_x + grid_btn_w
            && my >= grid_btn_y
            && my <= grid_btn_y + grid_btn_h;
        let player_card_clicked = mouse_clicked
            && mx >= p_btn_x
            && mx <= p_btn_x + p_btn_w
            && my >= p_btn_y
            && my <= p_btn_y + p_btn_h;
        let circuit_card_clicked = mouse_clicked
            && mx >= c_btn_x
            && mx <= c_btn_x + c_btn_w
            && my >= c_btn_y
            && my <= c_btn_y + c_btn_h;

        if player_card_clicked {
            self.starting_grid_focus = StartingGridFocus::LeftSetup;
            self.starting_grid_card_idx = 3;
            self.open_profile_from_starting_grid();
            return;
        }

        if circuit_card_clicked {
            self.starting_grid_focus = StartingGridFocus::LeftSetup;
            self.starting_grid_card_idx = 4;
            self.open_circuit_selector_from_starting_grid();
            return;
        }

        if garage_btn_clicked {
            self.open_garage_from_starting_grid();
            return;
        }

        if grid_btn_clicked {
            self.starting_grid_focus = StartingGridFocus::RightRoster;
            self.starting_grid_card_idx = 1;
            if self.game_mode.has_bots() && self.game_mode.allows_grid_customization() {
                let max_bots = self.max_bots();
                self.audio.play_sfx(SfxType::UiMove);
                if self.num_bots < max_bots {
                    self.num_bots += 1;
                } else {
                    self.num_bots = 1;
                }
                self.rebuild_roster_participants();
                self.update_active_modality_racer_count();
            }
            return;
        }

        // 1. Panel Switching (Left / Right / A / D / D-pad Left/Right / Nav Left/Right)
        if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::A)
            || self.input.gamepad.snapshot.dpad_left_pressed
            || self.input.gamepad.snapshot.nav_left
        {
            if self.starting_grid_focus != StartingGridFocus::LeftSetup {
                self.audio.play_sfx(SfxType::UiMove);
                self.starting_grid_focus = StartingGridFocus::LeftSetup;
                if self.starting_grid_card_idx == 1 {
                    self.starting_grid_card_idx = 0;
                }
            }
        }
        if is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || self.input.gamepad.snapshot.dpad_right_pressed
            || self.input.gamepad.snapshot.nav_right
        {
            if self.starting_grid_focus != StartingGridFocus::RightRoster {
                self.audio.play_sfx(SfxType::UiMove);
                self.starting_grid_focus = StartingGridFocus::RightRoster;
            }
        }

        // 2. Navigation & Actions within Active Panel
        match self.starting_grid_focus {
            StartingGridFocus::LeftSetup => {
                // Up / Down to navigate between setup cards (Player Card vs Circuit Card vs Garage vs Launch Race button)
                if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::W)
                    || self.input.gamepad.snapshot.dpad_up_pressed
                    || self.input.gamepad.snapshot.nav_up
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.starting_grid_card_idx = match self.starting_grid_card_idx {
                        2 => 0, // From Launch Button up to Garage
                        0 => 4, // From Garage up to Circuit Card
                        4 => 3, // From Circuit Card up to Player Card
                        3 => 2, // From Player Card up (wrap) to Launch Button
                        _ => 0,
                    };
                }
                if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::S)
                    || self.input.gamepad.snapshot.dpad_down_pressed
                    || self.input.gamepad.snapshot.nav_down
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.starting_grid_card_idx = match self.starting_grid_card_idx {
                        3 => 4, // From Player Card down to Circuit Card
                        4 => 0, // From Circuit Card down to Garage
                        0 => 2, // From Garage down to Launch Button
                        2 => 3, // From Launch Button down (wrap) to Player Card
                        _ => 2,
                    };
                }

                // Modify active card setting on Enter / Space / bracket / etc.
                match self.starting_grid_card_idx {
                    0 => {
                        // Card 0: Combined Motorsport Garage & Active Car Card
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed
                        {
                            self.open_garage_from_starting_grid();
                            return;
                        }
                        if self.game_mode.allows_car_change() {
                            let models = crate::catalog::get_models_for_module(self.active_module_id);
                            if !models.is_empty() {
                                let current_idx = self.selected_car_model_id
                                    .and_then(|id| models.iter().position(|m| m.id == id))
                                    .unwrap_or(0);
                                if is_key_pressed(KeyCode::RightBracket) {
                                    self.audio.play_sfx(SfxType::UiMove);
                                    let next_idx = (current_idx + 1) % models.len();
                                    let chosen = models[next_idx];
                                    self.selected_car_model_id = Some(chosen.id);
                                    self.car_choice = chosen.base_car_choice;
                                    self.current_visual_type = chosen.visual_type;
                                    self.free_car_selection = true;
                                    self.rebuild_roster_participants();
                                }
                                if is_key_pressed(KeyCode::LeftBracket) {
                                    self.audio.play_sfx(SfxType::UiMove);
                                    let next_idx = if current_idx == 0 { models.len() - 1 } else { current_idx - 1 };
                                    let chosen = models[next_idx];
                                    self.selected_car_model_id = Some(chosen.id);
                                    self.car_choice = chosen.base_car_choice;
                                    self.current_visual_type = chosen.visual_type;
                                    self.free_car_selection = true;
                                    self.rebuild_roster_participants();
                                }
                            } else {
                                let choices = self.active_module_car_choices();
                                if !choices.is_empty() {
                                    if is_key_pressed(KeyCode::RightBracket) {
                                        self.audio.play_sfx(SfxType::UiMove);
                                        self.menu_car_idx = (self.menu_car_idx + 1) % choices.len();
                                        self.car_choice = choices[self.menu_car_idx];
                                        self.free_car_selection = true;
                                        self.rebuild_roster_participants();
                                    }
                                    if is_key_pressed(KeyCode::LeftBracket) {
                                        self.audio.play_sfx(SfxType::UiMove);
                                        if self.menu_car_idx == 0 {
                                            self.menu_car_idx = choices.len() - 1;
                                        } else {
                                            self.menu_car_idx -= 1;
                                        }
                                        self.car_choice = choices[self.menu_car_idx];
                                        self.free_car_selection = true;
                                        self.rebuild_roster_participants();
                                    }
                                }
                            }
                        }
                    }
                    1 => {
                        // Backward-compatibility / direct test adjustment of Grid Configuration
                        if self.game_mode.has_bots() && self.game_mode.allows_grid_customization() {
                            let max_bots = self.max_bots();
                            if is_key_pressed(KeyCode::Enter)
                                || is_key_pressed(KeyCode::KpEnter)
                                || is_key_pressed(KeyCode::RightBracket)
                                || is_key_pressed(KeyCode::Equal)
                            {
                                self.audio.play_sfx(SfxType::UiMove);
                                if self.num_bots < max_bots {
                                    self.num_bots += 1;
                                } else {
                                    self.num_bots = 1;
                                }
                                self.rebuild_roster_participants();
                                self.update_active_modality_racer_count();
                            }
                            if is_key_pressed(KeyCode::LeftBracket) || is_key_pressed(KeyCode::Minus) {
                                self.audio.play_sfx(SfxType::UiMove);
                                if self.num_bots > 1 {
                                    self.num_bots -= 1;
                                } else {
                                    self.num_bots = max_bots;
                                }
                                self.rebuild_roster_participants();
                                self.update_active_modality_racer_count();
                            }
                        }
                    }
                    2 => {
                        // Card 2: Launch Race Button
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed
                        {
                            let req_tier = self.current_race_required_tier();
                            if !self.is_active_player_car_eligible(req_tier) || !self.is_active_player_car_unlocked() {
                                self.audio.play_sfx(SfxType::UiMove);
                                return;
                            }
                            self.audio.play_sfx(SfxType::UiSelect);
                            self.transition_iris_to(GameState::Countdown(3.5), 0.45);
                            return;
                        }
                    }
                    3 => {
                        // Card 3: Player Profile Card
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed
                        {
                            self.open_profile_from_starting_grid();
                            return;
                        }
                    }
                    4 => {
                        // Card 4: Circuit Explorer / Selector Card
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed
                        {
                            self.open_circuit_selector_from_starting_grid();
                            return;
                        }
                    }
                    _ => {
                        // Fallback: Launch Race Button
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || self.input.gamepad.snapshot.btn_confirm_pressed
                            || self.input.gamepad.snapshot.btn_a_pressed
                        {
                            let req_tier = self.current_race_required_tier();
                            if !self.is_active_player_car_eligible(req_tier) || !self.is_active_player_car_unlocked() {
                                self.audio.play_sfx(SfxType::UiMove);
                                return;
                            }
                            self.audio.play_sfx(SfxType::UiSelect);
                            self.transition_iris_to(GameState::Countdown(3.5), 0.45);
                            return;
                        }
                    }
                }
            }
            StartingGridFocus::RightRoster => {
                let roster_len = self.grid_participants.len().max(1);
                if self.starting_grid_card_idx == 1 {
                    // Top Grid Config card is active in right column
                    if is_key_pressed(KeyCode::Up)
                        || is_key_pressed(KeyCode::W)
                        || self.input.gamepad.snapshot.dpad_up_pressed
                        || self.input.gamepad.snapshot.nav_up
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.starting_grid_card_idx = 0;
                        self.starting_grid_roster_idx = roster_len - 1;
                    }
                    if is_key_pressed(KeyCode::Down)
                        || is_key_pressed(KeyCode::S)
                        || self.input.gamepad.snapshot.dpad_down_pressed
                        || self.input.gamepad.snapshot.nav_down
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.starting_grid_card_idx = 0;
                        self.starting_grid_roster_idx = 0;
                    }

                    // Adjust bots on Enter / + / - / [ / ]
                    if self.game_mode.has_bots() && self.game_mode.allows_grid_customization() {
                        let max_bots = self.max_bots();
                        if is_key_pressed(KeyCode::Enter)
                            || is_key_pressed(KeyCode::KpEnter)
                            || is_key_pressed(KeyCode::RightBracket)
                            || is_key_pressed(KeyCode::Equal)
                        {
                            self.audio.play_sfx(SfxType::UiMove);
                            if self.num_bots < max_bots {
                                self.num_bots += 1;
                            } else {
                                self.num_bots = 1;
                            }
                            self.rebuild_roster_participants();
                            self.update_active_modality_racer_count();
                        }
                        if is_key_pressed(KeyCode::LeftBracket) || is_key_pressed(KeyCode::Minus) {
                            self.audio.play_sfx(SfxType::UiMove);
                            if self.num_bots > 1 {
                                self.num_bots -= 1;
                            } else {
                                self.num_bots = max_bots;
                            }
                            self.rebuild_roster_participants();
                            self.update_active_modality_racer_count();
                        }
                    }
                } else {
                    // Roster entries are active
                    if is_key_pressed(KeyCode::Up)
                        || is_key_pressed(KeyCode::W)
                        || self.input.gamepad.snapshot.dpad_up_pressed
                        || self.input.gamepad.snapshot.nav_up
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        if self.starting_grid_roster_idx == 0 {
                            self.starting_grid_card_idx = 1; // Move up to Grid Config card
                        } else {
                            self.starting_grid_roster_idx -= 1;
                        }
                    }
                    if is_key_pressed(KeyCode::Down)
                        || is_key_pressed(KeyCode::S)
                        || self.input.gamepad.snapshot.dpad_down_pressed
                        || self.input.gamepad.snapshot.nav_down
                    {
                        self.audio.play_sfx(SfxType::UiMove);
                        if self.starting_grid_roster_idx >= roster_len - 1 {
                            self.starting_grid_card_idx = 1; // Wrap down to Grid Config card
                        } else {
                            self.starting_grid_roster_idx += 1;
                        }
                    }

                    // In Custom Race mode, allow customizing vehicle of selected participant with [ / ]
                    if self.game_mode.allows_roster_customization() {
                        if is_key_pressed(KeyCode::RightBracket) {
                            self.cycle_starting_grid_participant_model(true);
                        }
                        if is_key_pressed(KeyCode::LeftBracket) {
                            self.cycle_starting_grid_participant_model(false);
                        }
                    }

                    // Open Driver Dossier for selected slot (Enter / D / Gamepad Y)
                    if is_key_pressed(KeyCode::Enter)
                        || is_key_pressed(KeyCode::KpEnter)
                        || is_key_pressed(KeyCode::D)
                        || self.input.gamepad.snapshot.btn_y_pressed
                    {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.driver_cards_idx = self.starting_grid_roster_idx;
                        self.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
                        return;
                    }
                }
            }
        }

        // Global Launch race countdown (Space, Launch button clicked, Gamepad Start)
        if is_key_pressed(KeyCode::Space)
            || self.input.gamepad.snapshot.btn_start_pressed
            || launch_btn_clicked
            || (self.starting_grid_focus == StartingGridFocus::LeftSetup && (self.starting_grid_card_idx == 1 || self.starting_grid_card_idx == 2) && (self.input.gamepad.snapshot.btn_confirm_pressed || self.input.gamepad.snapshot.btn_a_pressed))
        {
            let req_tier = self.current_race_required_tier();
            if !self.is_active_player_car_eligible(req_tier) || !self.is_active_player_car_unlocked() {
                self.audio.play_sfx(SfxType::UiMove);
                return;
            }
            self.audio.play_sfx(SfxType::UiSelect);
            self.transition_iris_to(GameState::Countdown(3.5), 0.45);
            return;
        }

        // View Interactive Garage direct key shortcut (G key)
        if is_key_pressed(KeyCode::G) {
            self.open_garage_from_starting_grid();
            return;
        }

        // Cycle Casual AI Difficulty shortcut (T key)
        if is_key_pressed(KeyCode::T) && self.game_mode.allows_grid_customization() {
            self.audio.play_sfx(SfxType::UiMove);
            self.cycle_casual_ai_difficulty();
        }

        // View Driver Dossiers direct key shortcut (D key or Gamepad Y)
        if is_key_pressed(KeyCode::D) || (self.starting_grid_focus == StartingGridFocus::LeftSetup && self.input.gamepad.snapshot.btn_y_pressed) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
            return;
        }

        // View Player Profile direct key shortcut (P key)
        if is_key_pressed(KeyCode::P) {
            self.open_profile_from_starting_grid();
            return;
        }

        // View Circuit Explorer direct key shortcut (C key)
        if is_key_pressed(KeyCode::C) {
            self.open_circuit_selector_from_starting_grid();
            return;
        }

        // Return to Main Menu or Track Editor (Escape, or Gamepad Cancel [B / East / Back])
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.return_to_editor_on_exit {
                self.return_to_editor_on_exit = false;
                self.transition_fade_to(GameState::TrackEditor, 0.35);
            } else if self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                let tier = self.active_career_progress.level.clamp(1, 5);
                let calendar = if let Some(c) = &self.championship_session {
                    c.track_ids.clone()
                } else {
                    crate::ui::gt_default_calendar(tier)
                };
                self.career_hub_focus = CareerHubFocus::Tabs;
                self.transition_fade_to(
                    GameState::CareerHub {
                        selected_tier: tier,
                        selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                        calendar_tracks: calendar,
                        showing_standings: false,
                    },
                    0.35,
                );
            } else {
                self.transition_fade_to(GameState::Menu, 0.35);
            }
            return;
        }
    }

    /// Updates input and state progression when in the post-race Finished state.
    pub fn update_finished_screen(&mut self) {
        self.audio.stop_all_loops();

        // Synchronize legacy `show_hall_of_fame` boolean if modified externally
        let current_view = if self.finished_view == FinishedScreenView::Statistics {
            FinishedScreenView::Statistics
        } else if self.show_hall_of_fame || self.finished_view == FinishedScreenView::HallOfFame {
            FinishedScreenView::HallOfFame
        } else {
            FinishedScreenView::Results
        };

        // 1. [R] Key / Gamepad Y: Restart Race / Re-run Round
        if is_key_pressed(KeyCode::R) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.pending_championship_results = None;
            self.init_race();
            self.transition_iris_to(GameState::Countdown(3.5), 0.45);
            return;
        }

        // 2. [TAB] Key / Gamepad X: Toggle Detailed Race Statistics
        if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_x_pressed {
            if current_view == FinishedScreenView::Statistics {
                self.audio.play_sfx(SfxType::UiMove);
                self.finished_view = self.finished_prev_view;
                self.show_hall_of_fame = self.finished_view == FinishedScreenView::HallOfFame;
            } else {
                self.audio.play_sfx(SfxType::UiSelect);
                self.finished_prev_view = current_view;
                self.finished_view = FinishedScreenView::Statistics;
                self.show_hall_of_fame = false;
            }
            return;
        }

        // 3. [SPACE] / [ENTER] / Gamepad Confirm / A: Sequential forward progression
        // In Championships: Race Results -> Championship Standings / Career Hub
        // In Quick Race: Race Results -> Hall of Fame -> Main Menu
        if is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            match current_view {
                FinishedScreenView::Results => {
                    if self.championship_session.is_some() {
                        let mut awarded_trophy: Option<ChampionshipAward> = None;
                        if let Some(round_results) = self.pending_championship_results.take() {
                            let car_model_id = self.selected_car_model_id
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| self.active_player_car_choice().title().to_string());
                            let profile_id = self.active_profile.id.unwrap_or(1);
                            let module_id = self.active_module_id.to_string();

                            if let Some(champ) = &mut self.championship_session {
                                champ.submit_round_results(&self.track.name, round_results);
                                if champ.is_completed {
                                    if let Some(pos) = champ.standings.iter().position(|s| s.driver_id == "player") {
                                        let finish_pos = (pos + 1) as u32;
                                        match pos {
                                            0 => self.active_career_progress.trophies_gold += 1,
                                            1 => self.active_career_progress.trophies_silver += 1,
                                            2 => self.active_career_progress.trophies_bronze += 1,
                                            _ => {}
                                        }
                                        if finish_pos <= 3 {
                                            awarded_trophy = Some(ChampionshipAward {
                                                profile_id,
                                                championship_id: champ.name.to_lowercase().replace(' ', "_"),
                                                module_id,
                                                tier: champ.tier,
                                                position: finish_pos,
                                                points: champ.standings[pos].points,
                                                car_model_id,
                                                achieved_at: chrono::Utc::now().to_rfc3339(),
                                            });
                                        }
                                    }
                                    self.active_career_progress.active_championship = None;
                                } else {
                                    self.active_career_progress.active_championship = Some(champ.clone());
                                }
                                if let Some(db) = &self.hof_db {
                                    let _ = db.save_module_progress(&self.active_career_progress);
                                }
                                self.profile_module_progress.insert(self.active_career_progress.module_id.clone(), self.active_career_progress.clone());
                            }
                        }
                        if let Some(award) = awarded_trophy {
                            if let Some(db) = &self.hof_db {
                                let _ = db.save_championship_award(&award);
                            }
                            self.refresh_profile_awards();
                        }

                        self.audio.play_sfx(SfxType::UiSelect);
                        if self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                            let tier = self.active_career_progress.level.clamp(1, 5);
                            let calendar = if let Some(c) = &self.championship_session {
                                c.track_ids.clone()
                            } else {
                                crate::ui::gt_default_calendar(tier)
                            };
                            self.career_hub_focus = CareerHubFocus::Tabs;
                            self.state = GameState::CareerHub {
                                selected_tier: tier,
                                selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                                calendar_tracks: calendar,
                                showing_standings: true,
                            };
                        } else {
                            self.state = GameState::ChampionshipStandings;
                        }
                        return;
                    }

                    self.audio.play_sfx(SfxType::UiMove);
                    self.finished_view = FinishedScreenView::HallOfFame;
                    self.show_hall_of_fame = true;
                }
                FinishedScreenView::HallOfFame => {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.transition_fade_to(GameState::TrackEditor, 0.35);
                    } else {
                        self.transition_fade_to(GameState::Menu, 0.35);
                    }
                    return;
                }
                FinishedScreenView::Statistics => {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.finished_view = self.finished_prev_view;
                    self.show_hall_of_fame = self.finished_view == FinishedScreenView::HallOfFame;
                }
            }
            return;
        }

        // 4. [ESC] / Gamepad Cancel / Back / B: Step backward one screen
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            match current_view {
                FinishedScreenView::Statistics => {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.finished_view = self.finished_prev_view;
                    self.show_hall_of_fame = self.finished_view == FinishedScreenView::HallOfFame;
                }
                FinishedScreenView::HallOfFame => {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.finished_view = FinishedScreenView::Results;
                    self.show_hall_of_fame = false;
                }
                FinishedScreenView::Results => {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.pending_championship_results = None;
                    if self.return_to_editor_on_exit {
                        self.return_to_editor_on_exit = false;
                        self.transition_fade_to(GameState::TrackEditor, 0.35);
                    } else if self.championship_session.is_some() && self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                        let tier = self.active_career_progress.level.clamp(1, 5);
                        let calendar = if let Some(c) = &self.championship_session {
                            c.track_ids.clone()
                        } else {
                            crate::ui::gt_default_calendar(tier)
                        };
                        self.career_hub_focus = CareerHubFocus::Tabs;
                        self.state = GameState::CareerHub {
                            selected_tier: tier,
                            selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                            calendar_tracks: calendar,
                            showing_standings: false,
                        };
                    } else {
                        self.transition_fade_to(GameState::Menu, 0.35);
                    }
                    return;
                }
            }
            return;
        }
    }

    /// Updates input and state progression when viewing the Championship Standings screen.
    pub fn update_championship_standings(&mut self) {
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.advance_championship_round();
        }
        // Re-run latest round (R key or Gamepad Y)
        if is_key_pressed(KeyCode::R) || self.input.gamepad.snapshot.btn_y_pressed {
            if let Some(champ) = &mut self.championship_session {
                if let Some(track_id) = champ.cancel_latest_round() {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if let Some(db) = &self.hof_db {
                        let _ = db.delete_latest_race_history_entry_for_championship(&champ.name);
                    }
                    self.active_career_progress.active_championship = Some(champ.clone());
                    if let Some(db) = &self.hof_db {
                        let _ = db.save_module_progress(&self.active_career_progress);
                    }
                    self.profile_module_progress.insert(self.active_career_progress.module_id.clone(), self.active_career_progress.clone());
                    self.spawn_hud_alert("LATEST ROUND RESULTS CANCELLED — RE-RUNNING".to_string(), Palette::NEON_GOLD);
                    self.track_choice = self.track_manager.track_choice_for_slug(&track_id);
                    self.track = self
                        .track_manager
                        .load_track_by_slug(&track_id)
                        .unwrap_or_else(|_| tdrace_core::track::presets::classic_grand_prix());
                    self.init_race();
                    self.transition_iris_to(GameState::Countdown(3.5), 0.45);
                    return;
                }
            }
        }
        if is_key_pressed(KeyCode::Escape) || self.input.gamepad.snapshot.btn_cancel_pressed || self.input.gamepad.snapshot.btn_b_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.game_mode == GameMode::Career && (self.active_module_id == "gt" || self.active_module_id == "gt_challenge") {
                let tier = self.active_career_progress.level.clamp(1, 5);
                let calendar = if let Some(c) = &self.championship_session {
                    c.track_ids.clone()
                } else {
                    crate::ui::gt_default_calendar(tier)
                };
                self.career_hub_focus = CareerHubFocus::Tabs;
                self.state = GameState::CareerHub {
                    selected_tier: tier,
                    selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                    calendar_tracks: calendar,
                    showing_standings: false,
                };
            } else {
                self.state = GameState::Menu;
            }
        }
    }

    /// Handles input and actions for the Profile Manager screen.
    pub fn update_profile_manager(&mut self, selected_idx: usize) {
        let mut current_idx = selected_idx;
        let count = self.profile_list.len();
        if count == 0 {
            self.refresh_profiles_and_stats();
        }

        // Synchronize legacy profile_focus_card with profile_focus_area if modified externally
        if self.profile_focus_card && self.profile_focus_area != ProfileFocusArea::HeroCard {
            self.profile_focus_area = ProfileFocusArea::HeroCard;
        } else if !self.profile_focus_card && self.profile_focus_area == ProfileFocusArea::HeroCard {
            self.profile_focus_area = ProfileFocusArea::Tabs;
        }

        // Navigation state machine across HeroCard, Tabs, Module Filters, and Content
        match self.profile_focus_area {
            ProfileFocusArea::HeroCard => {
                // Pressing Down returns focus to Tabs
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_focus_area = ProfileFocusArea::Tabs;
                }

                // Enter on focused hero card opens Player Roster Manager
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.open_player_roster_manager(current_idx);
                    return;
                }

                // Inline driver cycling while focused on card
                let cycle_prev = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Q);
                let cycle_next = is_key_pressed(KeyCode::Right);

                if cycle_prev {
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

                if cycle_next {
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
            }
            ProfileFocusArea::Tabs => {
                // Focus hero card when pressing Up
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_focus_area = ProfileFocusArea::HeroCard;
                }

                // Focus module filters (or content) when pressing Down
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                    self.audio.play_sfx(SfxType::UiMove);
                    if self.profile_manager_tab == 3 || self.profile_manager_tab == 4 {
                        self.profile_focus_area = ProfileFocusArea::Filters;
                    } else {
                        self.profile_focus_area = ProfileFocusArea::Content;
                    }
                }

                // Tab Switching (Left/Right Arrow Keys, Tab, Numbers 1-5)
                let tab_prev = is_key_pressed(KeyCode::Left) || self.input.gamepad.snapshot.nav_left;
                let tab_next = is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::Tab)
                    || self.input.gamepad.snapshot.nav_right;

                if tab_prev {
                    self.audio.play_sfx(SfxType::UiMove);
                    if self.profile_manager_tab == 0 {
                        self.profile_manager_tab = 4;
                    } else {
                        self.profile_manager_tab -= 1;
                    }
                    self.profile_champ_scroll = 0;
                    self.profile_champ_selected_idx = 0;
                }
                if tab_next {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_manager_tab = (self.profile_manager_tab + 1) % 5;
                    self.profile_champ_scroll = 0;
                    self.profile_champ_selected_idx = 0;
                }
            }
            ProfileFocusArea::Filters => {
                // Return focus to Tabs when pressing Up
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_focus_area = ProfileFocusArea::Tabs;
                }

                // Select previous module filter with Left arrow or A key
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                    self.audio.play_sfx(SfxType::UiMove);
                    if self.profile_telemetry_filter_idx == 0 {
                        self.profile_telemetry_filter_idx = 6;
                    } else {
                        self.profile_telemetry_filter_idx -= 1;
                    }
                    self.profile_champ_scroll = 0;
                    self.profile_champ_selected_idx = 0;
                }

                // Select next module filter with Right arrow or D key
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_telemetry_filter_idx = (self.profile_telemetry_filter_idx + 1) % 7;
                    self.profile_champ_scroll = 0;
                    self.profile_champ_selected_idx = 0;
                }

                // Move down into Content list when pressing Down
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_focus_area = ProfileFocusArea::Content;
                }

                // Tab key cycles tabs
                if is_key_pressed(KeyCode::Tab) {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_manager_tab = (self.profile_manager_tab + 1) % 5;
                    self.profile_focus_area = ProfileFocusArea::Tabs;
                    self.profile_champ_scroll = 0;
                    self.profile_champ_selected_idx = 0;
                }
            }
            ProfileFocusArea::Content => {
                if self.profile_manager_tab == 2 {
                    // Trophy Cabinet grid navigation
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                        if self.profile_cabinet_disc_idx + 1 < 5 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_cabinet_disc_idx += 1;
                        }
                    }
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                        if self.profile_cabinet_disc_idx > 0 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_cabinet_disc_idx -= 1;
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_focus_area = ProfileFocusArea::Tabs;
                        }
                    }
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                        if self.profile_cabinet_tier_idx > 0 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_cabinet_tier_idx -= 1;
                        }
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                        if self.profile_cabinet_tier_idx + 1 < 5 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_cabinet_tier_idx += 1;
                        }
                    }
                } else if self.profile_manager_tab == 3 {
                    let active_filter = crate::ui::profile_ui::TELEMETRY_CATEGORY_FILTERS
                        .get(self.profile_telemetry_filter_idx)
                        .copied()
                        .unwrap_or(crate::ui::profile_ui::TELEMETRY_CATEGORY_FILTERS[0]);
                    let filtered_champs = crate::ui::profile_ui::get_sorted_championships(&self.championship_manager, active_filter.1);
                    let total_champs = filtered_champs.len();

                    let sw = screen_width_safe();
                    let sh = screen_height_safe();
                    let scaler = UiScaler::new(sw, sh);
                    let cur_y = scaler.s(16.0) + scaler.s(74.0) + scaler.s(8.0) + scaler.s(34.0) + scaler.s(8.0);
                    let footer_h = scaler.s(36.0);
                    let content_h = (sh - cur_y - footer_h - scaler.s(10.0)).max(scaler.s(360.0));
                    let visible_count = crate::ui::profile_ui::championship_visible_count(&scaler, content_h);

                    // Navigating down through championships
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down {
                        if total_champs > 0 && self.profile_champ_selected_idx + 1 < total_champs {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_champ_selected_idx += 1;
                            if self.profile_champ_selected_idx >= self.profile_champ_scroll + visible_count {
                                self.profile_champ_scroll = self.profile_champ_selected_idx + 1 - visible_count;
                            }
                        }
                    }

                    // Navigating up through championships
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                        if self.profile_champ_selected_idx > 0 {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_champ_selected_idx -= 1;
                            if self.profile_champ_selected_idx < self.profile_champ_scroll {
                                self.profile_champ_scroll = self.profile_champ_selected_idx;
                            }
                        } else {
                            // At top of championship list, Up returns focus to Filters
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_focus_area = ProfileFocusArea::Filters;
                        }
                    }

                    // Enter on selected championship launches or resumes it if unlocked
                    if is_key_pressed(KeyCode::Enter)
                        || is_key_pressed(KeyCode::KpEnter)
                        || is_key_pressed(KeyCode::Space)
                        || self.input.gamepad.snapshot.btn_confirm_pressed
                        || self.input.gamepad.snapshot.btn_a_pressed
                    {
                        if let Some(champ) = filtered_champs.get(self.profile_champ_selected_idx) {
                            if self.is_championship_unlocked(champ) {
                                let def = (*champ).clone();
                                self.launch_or_resume_championship(&def);
                                return;
                            } else {
                                self.audio.play_sfx(SfxType::UiMove);
                            }
                        }
                    }

                    // Reset/restart championship shortcut (R, X, Backspace, Delete, Gamepad X)
                    if is_key_pressed(KeyCode::R)
                        || is_key_pressed(KeyCode::X)
                        || is_key_pressed(KeyCode::Backspace)
                        || is_key_pressed(KeyCode::Delete)
                        || self.input.gamepad.snapshot.btn_x_pressed
                    {
                        let target_series = filtered_champs
                            .get(self.profile_champ_selected_idx)
                            .map(|c| (c.series.name.clone(), c.series.id.clone()));
                        drop(filtered_champs);
                        if let Some((name, id)) = target_series {
                            self.reset_championship(&name, &id);
                        }
                    }
                } else if self.profile_manager_tab == 4 {
                    // Telemetry tab: Up returns focus to Filters
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.profile_focus_area = ProfileFocusArea::Filters;
                    }
                } else if self.profile_manager_tab == 0 {
                    // Overview tab: Enter or Space jumps to Trophy Cabinet
                    if is_key_pressed(KeyCode::Enter)
                        || is_key_pressed(KeyCode::KpEnter)
                        || is_key_pressed(KeyCode::Space)
                        || self.input.gamepad.snapshot.btn_confirm_pressed
                        || self.input.gamepad.snapshot.btn_a_pressed
                    {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.profile_manager_tab = 2;
                        self.profile_focus_area = ProfileFocusArea::Content;
                    } else if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.profile_focus_area = ProfileFocusArea::Tabs;
                    }
                } else {
                    // Tab 1: Up returns focus to Tabs
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.profile_focus_area = ProfileFocusArea::Tabs;
                    }
                }

                // Tab key cycles tabs
                if is_key_pressed(KeyCode::Tab) {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.profile_manager_tab = (self.profile_manager_tab + 1) % 5;
                    self.profile_focus_area = ProfileFocusArea::Tabs;
                    self.profile_champ_scroll = 0;
                }
            }
        }

        // Direct tab jump hotkeys 1-5
        if is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::Kp1) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_manager_tab = 0;
            self.profile_focus_area = ProfileFocusArea::Tabs;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }
        if is_key_pressed(KeyCode::Key2) || is_key_pressed(KeyCode::Kp2) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_manager_tab = 1;
            self.profile_focus_area = ProfileFocusArea::Tabs;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }
        if is_key_pressed(KeyCode::Key3) || is_key_pressed(KeyCode::Kp3) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_manager_tab = 2;
            self.profile_focus_area = ProfileFocusArea::Tabs;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }
        if is_key_pressed(KeyCode::Key4) || is_key_pressed(KeyCode::Kp4) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_manager_tab = 3;
            self.profile_focus_area = ProfileFocusArea::Tabs;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }
        if is_key_pressed(KeyCode::Key5) || is_key_pressed(KeyCode::Kp5) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_manager_tab = 4;
            self.profile_focus_area = ProfileFocusArea::Tabs;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }

        // Sync legacy boolean
        self.profile_focus_card = self.profile_focus_area == ProfileFocusArea::HeroCard;

        // Mouse click on Hero Card, Tab bar, Module Filter Pills, or Championship Cards
        if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            let (mx, my) = macroquad::input::mouse_position();
            let sw = screen_width();
            let sh = screen_height();
            let scaler = UiScaler::new(sw, sh);
            let full_w = (sw * 0.96).max(scaler.s(720.0));
            let px = (sw - full_w) * 0.5;
            let py = scaler.s(16.0);
            let hero_h = scaler.s(74.0);

            // Clicked hero card -> open roster manager
            if mx >= px && mx <= px + full_w && my >= py && my <= py + hero_h {
                self.open_player_roster_manager(current_idx);
                return;
            }

            let tab_y = py + hero_h + scaler.s(8.0);
            let tab_bar_h = scaler.s(34.0);
            if my >= tab_y && my <= tab_y + tab_bar_h {
                let tab_gap = scaler.s(8.0);
                let total_gaps = tab_gap * 4.0;
                let tab_w = ((full_w - scaler.s(220.0) - total_gaps) / 5.0).max(scaler.s(96.0));
                for i in 0..5 {
                    let tx = px + i as f32 * (tab_w + tab_gap);
                    if mx >= tx && mx <= tx + tab_w {
                        if self.profile_manager_tab != i {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_manager_tab = i;
                            self.profile_champ_scroll = 0;
                            self.profile_champ_selected_idx = 0;
                        }
                        self.profile_focus_area = ProfileFocusArea::Tabs;
                        self.profile_focus_card = false;
                        break;
                    }
                }
            }

            // Clicked Top Honors Shelf (Tab 0)
            if self.profile_manager_tab == 0 {
                let content_y = tab_y + tab_bar_h + scaler.s(8.0);
                let pad = scaler.s(16.0);
                let inner_w = full_w - pad * 2.0;
                let col_gap = scaler.s(16.0);
                let col_w = (inner_w - col_gap) * 0.5;
                let right_x = px + pad + col_w + col_gap;
                let shelf_x = right_x + scaler.s(14.0);
                let shelf_w = col_w - scaler.s(28.0);
                let shelf_y = content_y + pad + scaler.s(54.0) + scaler.s(14.0) + scaler.s(44.0) + scaler.s(24.0) * 4.0 + scaler.s(8.0);
                if mx >= shelf_x && mx <= shelf_x + shelf_w && my >= shelf_y {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.profile_manager_tab = 2;
                    self.profile_focus_area = ProfileFocusArea::Content;
                }
            }

            // Clicked Trophy Cabinet Grid (Tab 2)
            if self.profile_manager_tab == 2 {
                let content_y = tab_y + tab_bar_h + scaler.s(8.0);
                let pad = scaler.s(16.0);
                let inner_w = full_w - pad * 2.0;
                let cy = content_y + pad + scaler.s(34.0) + scaler.s(10.0);
                let col_gap = scaler.s(14.0);
                let grid_w = (inner_w - col_gap) * 0.58;
                let body_h = (sh - cy - scaler.s(36.0) - scaler.s(24.0)).max(scaler.s(280.0));
                let left_x = px + pad;
                let disc_label_w = scaler.s(108.0);
                let tier_col_w = (grid_w - disc_label_w - scaler.s(16.0)) / 5.0;
                let grid_top_pad = scaler.s(12.0);
                let grid_rows_y = cy + grid_top_pad + scaler.s(20.0);
                let row_h = (body_h - grid_top_pad - scaler.s(20.0) - scaler.s(22.0)) / 5.0;

                if mx >= left_x + disc_label_w && mx <= left_x + grid_w && my >= grid_rows_y && my <= grid_rows_y + row_h * 5.0 {
                    let col = ((mx - (left_x + disc_label_w)) / tier_col_w).floor() as usize;
                    let row = ((my - grid_rows_y) / row_h).floor() as usize;
                    if col < 5 && row < 5 {
                        self.audio.play_sfx(SfxType::UiMove);
                        self.profile_cabinet_disc_idx = row;
                        self.profile_cabinet_tier_idx = col;
                        self.profile_focus_area = ProfileFocusArea::Content;
                    }
                }
            }

            // Clicked Module Filter Pills (Tab 3 or Tab 4)
            if self.profile_manager_tab == 3 || self.profile_manager_tab == 4 {
                let content_y = tab_y + tab_bar_h + scaler.s(8.0);
                let pad = if self.profile_manager_tab == 3 { scaler.s(16.0) } else { scaler.s(14.0) };
                let filter_y = if self.profile_manager_tab == 3 {
                    content_y + pad + scaler.s(40.0)
                } else {
                    content_y + pad
                };
                let pill_h = scaler.s(24.0);
                let pill_gap = scaler.s(6.0);
                let label_w = scaler.s(if self.profile_focus_area == ProfileFocusArea::Filters { 84.0 } else { 55.0 });
                let mut pill_x = px + pad + label_w;

                if my >= filter_y && my <= filter_y + pill_h {
                    for (f_idx, (f_name, _)) in crate::ui::profile_ui::TELEMETRY_CATEGORY_FILTERS.iter().enumerate() {
                        let pill_w = scaler.s(if *f_name == "ALL" { 48.0 } else { 75.0 });
                        if mx >= pill_x && mx <= pill_x + pill_w {
                            if self.profile_telemetry_filter_idx != f_idx {
                                self.audio.play_sfx(SfxType::UiMove);
                                self.profile_telemetry_filter_idx = f_idx;
                                self.profile_champ_scroll = 0;
                                self.profile_champ_selected_idx = 0;
                            }
                            self.profile_focus_area = ProfileFocusArea::Filters;
                            break;
                        }
                        pill_x += pill_w + pill_gap;
                    }
                }
            }

            // Clicked Championship Cards (Tab 3)
            if self.profile_manager_tab == 3 {
                let content_y = tab_y + tab_bar_h + scaler.s(8.0);
                let pad = scaler.s(16.0);
                let inner_w = full_w - pad * 2.0;
                let item_h = scaler.s(64.0);
                let item_gap = scaler.s(7.0);
                let list_top_y = content_y + pad + scaler.s(40.0) + scaler.s(24.0) + scaler.s(10.0);

                let active_filter = crate::ui::profile_ui::TELEMETRY_CATEGORY_FILTERS
                    .get(self.profile_telemetry_filter_idx)
                    .copied()
                    .unwrap_or(crate::ui::profile_ui::TELEMETRY_CATEGORY_FILTERS[0]);
                let filtered_champs = crate::ui::profile_ui::get_sorted_championships(&self.championship_manager, active_filter.1);
                let content_h = (sh - content_y - scaler.s(36.0) - scaler.s(10.0)).max(scaler.s(360.0));
                let visible_count = crate::ui::profile_ui::championship_visible_count(&scaler, content_h);

                for (rel_i, champ) in filtered_champs.iter().skip(self.profile_champ_scroll).take(visible_count).enumerate() {
                    let card_y = list_top_y + rel_i as f32 * (item_h + item_gap);
                    let card_x = px + pad;
                    if mx >= card_x && mx <= card_x + inner_w && my >= card_y && my <= card_y + item_h {
                        let clicked_idx = self.profile_champ_scroll + rel_i;
                        if self.profile_focus_area == ProfileFocusArea::Content && self.profile_champ_selected_idx == clicked_idx {
                            if self.is_championship_unlocked(champ) {
                                let def = (*champ).clone();
                                self.launch_or_resume_championship(&def);
                                return;
                            } else {
                                self.audio.play_sfx(SfxType::UiMove);
                            }
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                            self.profile_champ_selected_idx = clicked_idx;
                            self.profile_focus_area = ProfileFocusArea::Content;
                        }
                        break;
                    }
                }
            }
        }

        // Mouse wheel and PageUp/PageDown scrolling for Tab 2
        if self.profile_manager_tab == 2 {
            let wheel_y = mouse_wheel_safe().1;
            if wheel_y < -0.01 || is_key_pressed(KeyCode::PageDown) {
                self.profile_champ_scroll = self.profile_champ_scroll.saturating_add(1);
                self.profile_focus_area = ProfileFocusArea::Content;
            } else if wheel_y > 0.01 || is_key_pressed(KeyCode::PageUp) {
                self.profile_champ_scroll = self.profile_champ_scroll.saturating_sub(1);
            }
        }

        // Category Filter Cycling (F key shortcut anywhere on Tab 2 or Tab 3)
        if (self.profile_manager_tab == 2 || self.profile_manager_tab == 3) && is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiMove);
            self.profile_telemetry_filter_idx = (self.profile_telemetry_filter_idx + 1) % 7;
            self.profile_champ_scroll = 0;
            self.profile_champ_selected_idx = 0;
        }

        // Q key cycles driver prev anywhere in ProfileManager
        if !self.profile_focus_card && is_key_pressed(KeyCode::Q) {
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

        // Open Player Roster Manager (E or F2 key)
        if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::F2) {
            self.open_player_roster_manager(current_idx);
            return;
        }

        // Create New Profile (N key or Gamepad X)
        if is_key_pressed(KeyCode::N) || self.input.gamepad.snapshot.btn_x_pressed {
            self.open_player_roster_manager(current_idx);
            return;
        }

        // Clear Profile History (C key)
        if is_key_pressed(KeyCode::C) {
            if let Some(p) = self.profile_list.get(current_idx) {
                if let Some(pid) = p.id {
                    self.audio.play_sfx(SfxType::UiMove);
                    self.clear_profile_history(pid);
                    if let Some(db) = &self.hof_db {
                        self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                        self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                    }
                }
            }
        }

        // Return to Origin Screen (Escape, or Gamepad Cancel / B)
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.profile_focus_card = false;
            self.refresh_profiles_and_stats();
            match self.profile_origin {
                ProfileOrigin::ModalitySelect => {
                    self.state = GameState::ModalitySelect {
                        category: ModalityCategory::Options,
                        selected_idx: 0,
                        modal: None,
                    };
                }
                ProfileOrigin::ModuleSelect => {
                    self.state = GameState::ModuleSelect { selected_idx: 0 };
                }
                ProfileOrigin::Menu => {
                    self.state = GameState::Menu;
                }
                ProfileOrigin::StartingGrid => {
                    self.rebuild_roster_participants();
                    self.state = GameState::StartingGrid;
                }
            }
            return;
        }

        self.state = GameState::ProfileManager {
            selected_idx: current_idx,
        };
    }

    /// Opens the Two-Column Driver Roster & Profile Manager screen.
    pub fn open_player_roster_manager(&mut self, selected_idx: usize) {
        self.audio.play_sfx(SfxType::UiSelect);
        while get_char_pressed().is_some() {}
        let sel = selected_idx.min(self.profile_list.len().saturating_sub(1));
        if let Some(p) = self.profile_list.get(sel) {
            let country_idx = p.country.as_deref().and_then(|code| {
                CountryRegistry::ALL.iter().position(|c| c.code.eq_ignore_ascii_case(code)).map(|pos| pos + 1)
            }).unwrap_or(0);

            let livery_idx = Palette::CAR_COLORS.iter().position(|c| {
                c.0 == p.color_scheme.primary && c.1 == p.color_scheme.secondary
            }).unwrap_or(0);

            if let Some(pid) = p.id {
                if let Some(db) = &self.hof_db {
                    self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                    self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                }
            }

            self.state = GameState::PlayerRosterManager {
                selected_idx: sel,
                active_column: 0,
                field_idx: 0,
                input_name: p.name.clone(),
                input_alias: p.alias.clone(),
                country_idx,
                livery_idx,
                assist_mode: p.last_mode,
                cursor_timer: 0.0,
                status_msg: None,
            };
        }
    }

    /// Handles input and navigation for the Two-Column Driver Roster & Profile Manager screen.
    #[allow(clippy::too_many_arguments)]
    fn update_player_roster_manager(
        &mut self,
        mut selected_idx: usize,
        mut active_column: usize,
        mut field_idx: usize,
        mut input_name: String,
        mut input_alias: String,
        mut country_idx: usize,
        mut livery_idx: usize,
        mut assist_mode: AssistProfile,
        mut cursor_timer: f32,
        mut status_msg: Option<String>,
        frame_dt: f32,
    ) {
        cursor_timer += frame_dt;

        // Escape: Back to ProfileManager
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.refresh_profiles_and_stats();
            self.state = GameState::ProfileManager { selected_idx };
            return;
        }

        // New Driver [N key]
        if is_key_pressed(KeyCode::N) || (active_column == 0 && self.input.gamepad.snapshot.btn_x_pressed) {
            self.audio.play_sfx(SfxType::UiSelect);
            while get_char_pressed().is_some() {}
            if let Some(db) = &self.hof_db {
                let new_num = self.profile_list.len() + 1;
                let new_name = format!("Racer {}", new_num);
                let new_alias = format!("Apex {}", new_num);
                let next_livery = self.profile_list.len() % Palette::CAR_COLORS.len();
                let scheme = CarColorScheme::from_index(next_livery);
                let mut new_profile = PlayerProfile::new(&new_name, &new_alias, Some("ESP"), scheme);
                new_profile.is_active = false;
                if let Ok(new_id) = db.create_profile(&new_profile) {
                    self.refresh_profiles_and_stats();
                    if let Some(pos) = self.profile_list.iter().position(|p| p.id == Some(new_id)) {
                        selected_idx = pos;
                    }
                    input_name = new_name;
                    input_alias = new_alias;
                    country_idx = 1;
                    livery_idx = next_livery;
                    assist_mode = AssistProfile::Arcade;
                    active_column = 1;
                    field_idx = 0;
                    status_msg = Some("New driver added. Enter details and press Save.".to_string());
                    self.state = GameState::PlayerRosterManager {
                        selected_idx,
                        active_column,
                        field_idx,
                        input_name,
                        input_alias,
                        country_idx,
                        livery_idx,
                        assist_mode,
                        cursor_timer,
                        status_msg,
                    };
                    return;
                }
            }
        }

        // Delete Driver [DEL or X key] - only when more than 1 profile exists
        if (is_key_pressed(KeyCode::Delete) || (active_column == 0 && is_key_pressed(KeyCode::X)))
            && self.profile_list.len() > 1
        {
            if let Some(p) = self.profile_list.get(selected_idx) {
                if let Some(pid) = p.id {
                    self.audio.play_sfx(SfxType::UiMove);
                    if let Some(db) = &self.hof_db {
                        let _ = db.delete_profile(pid);
                    }
                    self.refresh_profiles_and_stats();
                    selected_idx = selected_idx.min(self.profile_list.len().saturating_sub(1));
                    if let Some(np) = self.profile_list.get(selected_idx) {
                        input_name = np.name.clone();
                        input_alias = np.alias.clone();
                        country_idx = np.country.as_deref().and_then(|c| {
                            CountryRegistry::ALL.iter().position(|r| r.code.eq_ignore_ascii_case(c)).map(|pos| pos + 1)
                        }).unwrap_or(0);
                        livery_idx = Palette::CAR_COLORS.iter().position(|c| {
                            c.0 == np.color_scheme.primary && c.1 == np.color_scheme.secondary
                        }).unwrap_or(0);
                        assist_mode = np.last_mode;
                        if let Some(npid) = np.id {
                            if let Some(db) = &self.hof_db {
                                self.active_profile_stats = db.get_stats_for_profile(npid).unwrap_or_default();
                                self.profile_history = db.get_history_for_profile(npid, 20).unwrap_or_default();
                            }
                        }
                    }
                    status_msg = Some("Driver removed from roster.".to_string());
                    self.state = GameState::PlayerRosterManager {
                        selected_idx,
                        active_column,
                        field_idx,
                        input_name,
                        input_alias,
                        country_idx,
                        livery_idx,
                        assist_mode,
                        cursor_timer,
                        status_msg,
                    };
                    return;
                }
            }
        }

        // Column switching (Tab key, or Right Arrow from left column)
        if is_key_pressed(KeyCode::Tab) {
            self.audio.play_sfx(SfxType::UiMove);
            active_column = 1 - active_column;
            while get_char_pressed().is_some() {}
        } else if active_column == 0 && (is_key_pressed(KeyCode::Right) || self.input.gamepad.snapshot.nav_right) {
            self.audio.play_sfx(SfxType::UiMove);
            active_column = 1;
            while get_char_pressed().is_some() {}
        }

        if active_column == 0 {
            // Left Column: Browsing Roster
            let prev_driver = is_key_pressed(KeyCode::Up)
                || is_key_pressed(KeyCode::W)
                || self.input.gamepad.snapshot.nav_up;
            let next_driver = is_key_pressed(KeyCode::Down)
                || is_key_pressed(KeyCode::S)
                || self.input.gamepad.snapshot.nav_down;

            if prev_driver && !self.profile_list.is_empty() {
                self.audio.play_sfx(SfxType::UiMove);
                if selected_idx == 0 {
                    selected_idx = self.profile_list.len() - 1;
                } else {
                    selected_idx -= 1;
                }
                if let Some(p) = self.profile_list.get(selected_idx) {
                    input_name = p.name.clone();
                    input_alias = p.alias.clone();
                    country_idx = p.country.as_deref().and_then(|c| {
                        CountryRegistry::ALL.iter().position(|r| r.code.eq_ignore_ascii_case(c)).map(|pos| pos + 1)
                    }).unwrap_or(0);
                    livery_idx = Palette::CAR_COLORS.iter().position(|c| {
                        c.0 == p.color_scheme.primary && c.1 == p.color_scheme.secondary
                    }).unwrap_or(0);
                    assist_mode = p.last_mode;
                    if let Some(pid) = p.id {
                        if let Some(db) = &self.hof_db {
                            self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                            self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                        }
                    }
                }
            } else if next_driver && !self.profile_list.is_empty() {
                self.audio.play_sfx(SfxType::UiMove);
                selected_idx = (selected_idx + 1) % self.profile_list.len();
                if let Some(p) = self.profile_list.get(selected_idx) {
                    input_name = p.name.clone();
                    input_alias = p.alias.clone();
                    country_idx = p.country.as_deref().and_then(|c| {
                        CountryRegistry::ALL.iter().position(|r| r.code.eq_ignore_ascii_case(c)).map(|pos| pos + 1)
                    }).unwrap_or(0);
                    livery_idx = Palette::CAR_COLORS.iter().position(|c| {
                        c.0 == p.color_scheme.primary && c.1 == p.color_scheme.secondary
                    }).unwrap_or(0);
                    assist_mode = p.last_mode;
                    if let Some(pid) = p.id {
                        if let Some(db) = &self.hof_db {
                            self.active_profile_stats = db.get_stats_for_profile(pid).unwrap_or_default();
                            self.profile_history = db.get_history_for_profile(pid, 20).unwrap_or_default();
                        }
                    }
                }
            }

            // Set Active Profile (Enter / Space / Gamepad A)
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::KpEnter)
                || is_key_pressed(KeyCode::Space)
                || self.input.gamepad.snapshot.btn_confirm_pressed
                || self.input.gamepad.snapshot.btn_a_pressed
            {
                let maybe_active = self.profile_list.get(selected_idx).and_then(|p| {
                    p.id.map(|pid| (pid, p.name.clone()))
                });
                if let Some((pid, name)) = maybe_active {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.set_active_profile_by_id(pid);
                    status_msg = Some(format!("'{}' set as active racer!", name));
                }
            }
        } else {
            // Right Column: Editing Profile Details
            if is_key_pressed(KeyCode::Up) || (field_idx > 1 && (is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up)) {
                self.audio.play_sfx(SfxType::UiMove);
                if field_idx == 0 {
                    field_idx = 5;
                } else {
                    field_idx -= 1;
                }
            }
            if is_key_pressed(KeyCode::Down) || (field_idx > 1 && (is_key_pressed(KeyCode::S) || self.input.gamepad.snapshot.nav_down)) {
                self.audio.play_sfx(SfxType::UiMove);
                field_idx = (field_idx + 1) % 6;
            }

            // Text input for fields 0 (Name) and 1 (Alias)
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

            // Cycling options for Country (2), Livery (3), Assist Mode (4)
            let total_countries = CountryRegistry::ALL.len() + 1;
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
            } else if field_idx == 3 {
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
            } else if field_idx == 4 {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) || self.input.gamepad.snapshot.nav_left {
                    self.audio.play_sfx(SfxType::UiMove);
                    assist_mode = match assist_mode {
                        AssistProfile::Arcade => AssistProfile::Pro,
                        AssistProfile::Sport => AssistProfile::Arcade,
                        AssistProfile::Pro => AssistProfile::Sport,
                    };
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || self.input.gamepad.snapshot.nav_right {
                    self.audio.play_sfx(SfxType::UiMove);
                    assist_mode = match assist_mode {
                        AssistProfile::Arcade => AssistProfile::Sport,
                        AssistProfile::Sport => AssistProfile::Pro,
                        AssistProfile::Pro => AssistProfile::Arcade,
                    };
                }
            }

            // Save Changes (Enter on Save button, or Enter on country/livery/assist, or Ctrl+S)
            let save_triggered = if field_idx >= 2 {
                is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
            } else {
                (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) && is_key_pressed(KeyCode::S)
            };

            if save_triggered {
                let final_name = if input_name.trim().is_empty() {
                    "Driver".to_string()
                } else {
                    input_name.trim().to_string()
                };

                let final_alias = if input_alias.trim().is_empty() {
                    "Apex".to_string()
                } else {
                    input_alias.trim().to_string()
                };

                let country_opt = if country_idx > 0 && country_idx <= CountryRegistry::ALL.len() {
                    Some(CountryRegistry::ALL[country_idx - 1].code.to_string())
                } else {
                    None
                };

                let scheme = CarColorScheme::from_index(livery_idx);

                let maybe_updated = self.profile_list.get(selected_idx).and_then(|p| {
                    p.id.map(|pid| PlayerProfile {
                        id: Some(pid),
                        name: final_name,
                        alias: final_alias,
                        country: country_opt,
                        color_scheme: scheme,
                        is_active: p.is_active,
                        created_at: p.created_at.clone(),
                        last_mode: assist_mode,
                    })
                });

                if let Some(updated) = maybe_updated {
                    if let Some(db) = &self.hof_db {
                        let _ = db.update_profile(&updated);
                    }
                    self.refresh_profiles_and_stats();
                    self.audio.play_sfx(SfxType::UiSelect);
                    status_msg = Some("✓ Profile saved successfully!".to_string());
                }
            }
        }

        self.state = GameState::PlayerRosterManager {
            selected_idx,
            active_column,
            field_idx,
            input_name,
            input_alias,
            country_idx,
            livery_idx,
            assist_mode,
            cursor_timer,
            status_msg,
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


    /// Updates input and navigation for Grand Hub Module Selection screen.
    pub fn update_module_select(&mut self) {
        let mut selected_idx = match self.state {
            GameState::ModuleSelect { selected_idx } => selected_idx,
            _ => return,
        };

        // If Arcade Settings Modal is open on the Grand Hub, update it and return:
        if let Some(ref mut modal) = self.settings_modal {
            let (sw, sh) = (screen_width_safe(), screen_height_safe());
            let scaler = UiScaler::new(sw, sh);
            let theme = CabinetTheme::default();
            let mut ctx = CabinetContext {
                scaler: &scaler,
                fonts: &self.fonts,
                theme: &theme,
                gamepad: &self.input.gamepad.snapshot,
                dt: 1.0 / 60.0,
                audio: Some(&self.audio),
            };

            let action = modal.update(&mut ctx);
            if matches!(action, ScreenAction::Pop) {
                let saved = modal.is_saved;
                self.close_settings_modal(saved);
                if saved {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
            return;
        }

        // If exit confirmation modal is currently open:
        if self.show_exit_confirm {
            if self.exit_confirm_modal.is_none() {
                self.exit_confirm_modal = Some(UniversalConfirmModal::quit_game());
            }
            if let Some(ref mut modal) = self.exit_confirm_modal {
                let sw = screen_width_safe();
                let sh = screen_height_safe();
                let scaler = UiScaler::new(sw, sh);
                let theme = CabinetTheme::cyberpunk_neon();
                let mut ctx = CabinetContext::new(
                    &scaler,
                    &self.fonts,
                    &theme,
                    &self.input.gamepad.snapshot,
                    1.0 / 60.0,
                )
                .with_audio(Some(&self.audio));

                // Direct legacy shortcuts for Y / N
                if is_key_pressed(KeyCode::Y) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::N) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.show_exit_confirm = false;
                    self.exit_confirm_modal = None;
                    return;
                }

                let action = modal.update(&mut ctx);
                match action {
                    ScreenAction::Quit => {
                        std::process::exit(0);
                    }
                    ScreenAction::Pop => {
                        self.show_exit_confirm = false;
                        self.exit_confirm_modal = None;
                    }
                    _ => {}
                }
            }
            return;
        }

        let num_items = 7; // 0: Player Profile, 1..=6: Motorsport Modules
        if is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || self.input.gamepad.snapshot.nav_up
            || self.input.gamepad.snapshot.dpad_up_pressed
        {
            self.audio.play_sfx(SfxType::UiMove);
            if selected_idx == 0 {
                selected_idx = num_items - 1;
            } else {
                selected_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::S)
            || self.input.gamepad.snapshot.nav_down
            || self.input.gamepad.snapshot.dpad_down_pressed
        {
            self.audio.play_sfx(SfxType::UiMove);
            selected_idx = (selected_idx + 1) % num_items;
        }

        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);

        let mut mouse_selected_item = None;
        if mouse_clicked {
            let (bx, by, bw, bh) = crate::ui::menu::module_select_badge_rect(sw, sh);
            if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                selected_idx = 0;
                mouse_selected_item = Some(0);
            } else {
                let num_modules = 6;
                for i in 0..num_modules {
                    let (cx, cy, cw, ch) = crate::ui::menu::module_select_card_rect(sw, sh, i, num_modules);
                    if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                        selected_idx = i + 1;
                        mouse_selected_item = Some(i + 1);
                        break;
                    }
                }
            }
        }

        if matches!(self.state, GameState::ModuleSelect { .. }) {
            self.state = GameState::ModuleSelect { selected_idx };
        }

        let confirm_pressed = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
            || mouse_selected_item.is_some();

        if confirm_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            if selected_idx == 0 {
                self.profile_origin = ProfileOrigin::ModuleSelect;
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
            } else {
                self.transition_scanline_to(
                    GameState::ModalitySelect {
                        category: ModalityCategory::SinglePlayer,
                        selected_idx: 0,
                        modal: None,
                    },
                    0.35,
                );
                return;
            }
        }

        // Profile Manager (P key or Gamepad Y)
        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.profile_origin = ProfileOrigin::ModuleSelect;
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

        // Arcade Settings Modal (X key)
        if is_key_pressed(KeyCode::X) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.open_settings_modal();
            return;
        }

        // Controls Help (K key)
        if is_key_pressed(KeyCode::K) {
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

        if matches!(self.state, GameState::ModuleSelect { .. }) {
            self.state = GameState::ModuleSelect { selected_idx };
        }
    }

    /// Updates input and state for the Race Modality Selection stage.
    pub fn update_modality_select(&mut self) {
        let (mut category, mut selected_idx, mut modal) = match self.state {
            GameState::ModalitySelect {
                category,
                selected_idx,
                ref modal,
            } => (category, selected_idx, modal.clone()),
            _ => return,
        };

        // If Arcade Settings Modal is open on the Modality Select screen, update it and return:
        if let Some(ref mut modal) = self.settings_modal {
            let (sw, sh) = (screen_width_safe(), screen_height_safe());
            let scaler = UiScaler::new(sw, sh);
            let theme = CabinetTheme::default();
            let mut ctx = CabinetContext {
                scaler: &scaler,
                fonts: &self.fonts,
                theme: &theme,
                gamepad: &self.input.gamepad.snapshot,
                dt: 1.0 / 60.0,
                audio: Some(&self.audio),
            };

            let action = modal.update(&mut ctx);
            if matches!(action, ScreenAction::Pop) {
                let saved = modal.is_saved;
                self.close_settings_modal(saved);
                if saved {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
            return;
        }

        // If informational coming-soon modal is open, any confirm/back dismisses it
        if modal.is_some() {
            if is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::KpEnter)
                || self.input.gamepad.snapshot.btn_confirm_pressed
                || self.input.gamepad.snapshot.btn_a_pressed
                || self.input.gamepad.snapshot.btn_b_pressed
                || self.input.gamepad.snapshot.btn_back_pressed
            {
                self.audio.play_sfx(SfxType::UiSelect);
                modal = None;
                self.state = GameState::ModalitySelect {
                    category,
                    selected_idx,
                    modal,
                };
            }
            return;
        }

        // Direct Garage Showroom shortcut (G key)
        if is_key_pressed(KeyCode::G) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.garage_origin = GarageOrigin::ModalitySelect;
            self.state = GameState::Garage(GarageOrigin::ModalitySelect);
            return;
        }

        // Direct Player Profile shortcut (P key or Gamepad Y)
        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.profile_origin = ProfileOrigin::ModalitySelect;
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

        // Direct Track Editor shortcut (E key)
        if is_key_pressed(KeyCode::E) {
            self.enter_track_editor_from_modality_select();
            return;
        }

        // Direct Championship Editor shortcut (C key)
        if is_key_pressed(KeyCode::C) {
            self.enter_championship_editor(None);
            return;
        }

        // Direct Settings shortcut (X key)
        if is_key_pressed(KeyCode::X) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.open_settings_modal();
            return;
        }

        // Category Column/Menu Switching (Left / Right / Tab / 1 / 2 / 3 / Gamepad D-pad / Bumpers / Mouse Tab Click)
        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let scaler = UiScaler::new(sw, sh);
        let tab_w = (sw * 0.28).clamp(scaler.s(160.0), scaler.s(260.0));
        let tab_h = scaler.s(30.0);
        let tab_gap = scaler.s(12.0);
        let total_tabs_w = tab_w * 3.0 + tab_gap * 2.0;
        let tabs_start_x = (sw - total_tabs_w) * 0.5;
        let tab_y = scaler.s(64.0);

        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);
        if mouse_clicked && my >= tab_y && my <= tab_y + tab_h {
            for (cat_idx, cat) in ModalityCategory::ALL.iter().enumerate() {
                let tx = tabs_start_x + (cat_idx as f32) * (tab_w + tab_gap);
                if mx >= tx && mx <= tx + tab_w {
                    if category != *cat {
                        category = *cat;
                        selected_idx = 0;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    break;
                }
            }
        }

        if is_key_pressed(KeyCode::Key1) {
            if category != ModalityCategory::SinglePlayer {
                category = ModalityCategory::SinglePlayer;
                selected_idx = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        } else if is_key_pressed(KeyCode::Key2) {
            if category != ModalityCategory::Multiplayer {
                category = ModalityCategory::Multiplayer;
                selected_idx = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        } else if is_key_pressed(KeyCode::Key3) {
            if category != ModalityCategory::Options {
                category = ModalityCategory::Options;
                selected_idx = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        } else if is_key_pressed(KeyCode::Tab)
            || is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || self.input.gamepad.snapshot.dpad_right_pressed
            || self.input.gamepad.snapshot.btn_rb_pressed
        {
            category = match category {
                ModalityCategory::SinglePlayer => ModalityCategory::Multiplayer,
                ModalityCategory::Multiplayer => ModalityCategory::Options,
                ModalityCategory::Options => ModalityCategory::SinglePlayer,
            };
            selected_idx = 0;
            self.audio.play_sfx(SfxType::UiMove);
        } else if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::A)
            || self.input.gamepad.snapshot.dpad_left_pressed
            || self.input.gamepad.snapshot.btn_lb_pressed
        {
            category = match category {
                ModalityCategory::SinglePlayer => ModalityCategory::Options,
                ModalityCategory::Multiplayer => ModalityCategory::SinglePlayer,
                ModalityCategory::Options => ModalityCategory::Multiplayer,
            };
            selected_idx = 0;
            self.audio.play_sfx(SfxType::UiMove);
        }

        // Modality Card Navigation (Up / Down / W / S / Gamepad D-pad)
        let items = category.items();
        let items_len = items.len();
        if is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || self.input.gamepad.snapshot.dpad_up_pressed
            || self.input.gamepad.snapshot.nav_up
        {
            self.audio.play_sfx(SfxType::UiMove);
            if selected_idx == 0 {
                selected_idx = items_len.saturating_sub(1);
            } else {
                selected_idx -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down)
            || self.input.gamepad.snapshot.dpad_down_pressed
            || self.input.gamepad.snapshot.nav_down
        {
            self.audio.play_sfx(SfxType::UiMove);
            if items_len > 0 {
                selected_idx = (selected_idx + 1) % items_len;
            }
        }

        // Mouse click on modality cards
        let mut mouse_confirmed_card = false;
        if modal.is_none() && mouse_clicked {
            for i in 0..items_len {
                let (cx, cy, cw, ch) = crate::ui::menu::modality_card_rect(sw, sh, category, i);
                if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                    selected_idx = i;
                    mouse_confirmed_card = true;
                    break;
                }
            }
        }

        // Confirmation (Enter / Space / Gamepad A / Mouse Click)
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
            || mouse_confirmed_card
        {
            if let Some(&item) = items.get(selected_idx) {
                match item {
                    ModalityItem::PlayerProfile => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.profile_origin = ProfileOrigin::ModalitySelect;
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
                    ModalityItem::Garage => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.garage_origin = GarageOrigin::ModalitySelect;
                        self.state = GameState::Garage(GarageOrigin::ModalitySelect);
                        return;
                    }
                    ModalityItem::TrackEditor => {
                        self.enter_track_editor_from_modality_select();
                        return;
                    }
                    ModalityItem::SeriesEditor => {
                        self.enter_championship_editor(None);
                        return;
                    }
                    ModalityItem::Settings => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.open_settings_modal();
                        return;
                    }
                    ModalityItem::QuickRace => {
                        self.game_mode = GameMode::StandardRace;
                        self.free_car_selection = false;
                        self.is_time_attack = false;
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.transition_scanline_to(GameState::Menu, 0.35);
                        return;
                    }
                    ModalityItem::CustomRace => {
                        self.game_mode = GameMode::ExperimentalRace;
                        self.free_car_selection = true;
                        self.is_time_attack = false;
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.transition_scanline_to(GameState::Menu, 0.35);
                        return;
                    }
                    ModalityItem::CareerMode => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.state = GameState::CareerSelect { selected_idx: 0 };
                        return;
                    }
                    ModalityItem::TimeTrial => {
                        self.game_mode = GameMode::TimeTrial;
                        self.free_car_selection = true;
                        self.is_time_attack = true;
                        self.num_bots = 0;
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.transition_scanline_to(GameState::Menu, 0.35);
                        return;
                    }
                    ModalityItem::FreeRide => {
                        self.game_mode = GameMode::FreeRide;
                        self.free_car_selection = true;
                        self.is_time_attack = true;
                        self.num_bots = 0;
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.transition_scanline_to(GameState::Menu, 0.35);
                        return;
                    }
                    ModalityItem::SplitScreen => {
                        self.game_mode = GameMode::SplitScreen;
                        self.free_car_selection = true;
                        self.is_time_attack = false;
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.transition_scanline_to(GameState::Menu, 0.35);
                        return;
                    }
                    ModalityItem::LanPlay => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        modal = Some(ModalityModal::LanComingSoon);
                        self.state = GameState::ModalitySelect {
                            category,
                            selected_idx,
                            modal,
                        };
                        return;
                    }
                    ModalityItem::CloudPlay => {
                        self.audio.play_sfx(SfxType::UiSelect);
                        modal = Some(ModalityModal::CloudComingSoon);
                        self.state = GameState::ModalitySelect {
                            category,
                            selected_idx,
                            modal,
                        };
                        return;
                    }
                }
            }
        }

        // Return to Grand Hub (Escape / Gamepad B / Back)
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_b_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            let cur_mod_idx = match self.active_module_id {
                "classic" => 1,
                "rally" => 2,
                "kart" => 3,
                "gt" | "gt_challenge" => 4,
                "nascar" => 5,
                "extreme_offroad" => 6,
                _ => 1,
            };
            self.transition_fade_to(GameState::ModuleSelect { selected_idx: cur_mod_idx }, 0.3);
            return;
        }

        if matches!(self.state, GameState::ModalitySelect { .. }) {
            self.state = GameState::ModalitySelect {
                category,
                selected_idx,
                modal,
            };
        }
    }

    /// Updates input and state for the GT Career Hub screen.
    pub fn update_career_hub(&mut self) {
        let (mut selected_tier, mut selected_slot, mut calendar_tracks, mut showing_standings) =
            match self.state {
                GameState::CareerHub {
                    selected_tier,
                    selected_slot,
                    ref calendar_tracks,
                    showing_standings,
                } => (
                    selected_tier,
                    selected_slot,
                    calendar_tracks.clone(),
                    showing_standings,
                ),
                _ => return,
            };

        let mut tier_changed = false;

        // 1. Mouse Interaction: Click tabs to select tier & focus tabs, click calendar to select slot & focus calendar
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        let scaler = UiScaler::new(sw, sh);
        let full_w = (sw * 0.96).max(scaler.s(760.0));
        let x = (sw - full_w) * 0.5;
        let tab_bar_y = scaler.s(14.0) + scaler.s(56.0) + scaler.s(10.0);
        let tab_bar_h = scaler.s(40.0);
        let tier_count = 5;
        let tab_gap = scaler.s(8.0);
        let tab_w = (full_w - tab_gap * (tier_count as f32 - 1.0)) / tier_count as f32;

        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);

        if mouse_clicked && my >= tab_bar_y && my <= tab_bar_y + tab_bar_h {
            for t in 1..=tier_count {
                let tx = x + (t - 1) as f32 * (tab_w + tab_gap);
                if mx >= tx && mx <= tx + tab_w {
                    if selected_tier != t as u32 {
                        selected_tier = t as u32;
                        tier_changed = true;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    self.career_hub_focus = CareerHubFocus::Tabs;
                    break;
                }
            }
        }

        // 2. Direct Number Key Shortcuts (1-5) for immediate tier selection
        if is_key_pressed(KeyCode::Key1) {
            if selected_tier != 1 {
                selected_tier = 1;
                tier_changed = true;
                self.audio.play_sfx(SfxType::UiMove);
            }
            self.career_hub_focus = CareerHubFocus::Tabs;
        } else if is_key_pressed(KeyCode::Key2) {
            if selected_tier != 2 {
                selected_tier = 2;
                tier_changed = true;
                self.audio.play_sfx(SfxType::UiMove);
            }
            self.career_hub_focus = CareerHubFocus::Tabs;
        } else if is_key_pressed(KeyCode::Key3) {
            if selected_tier != 3 {
                selected_tier = 3;
                tier_changed = true;
                self.audio.play_sfx(SfxType::UiMove);
            }
            self.career_hub_focus = CareerHubFocus::Tabs;
        } else if is_key_pressed(KeyCode::Key4) {
            if selected_tier != 4 {
                selected_tier = 4;
                tier_changed = true;
                self.audio.play_sfx(SfxType::UiMove);
            }
            self.career_hub_focus = CareerHubFocus::Tabs;
        } else if is_key_pressed(KeyCode::Key5) {
            if selected_tier != 5 {
                selected_tier = 5;
                tier_changed = true;
                self.audio.play_sfx(SfxType::UiMove);
            }
            self.career_hub_focus = CareerHubFocus::Tabs;
        }

        // 3. Standings Toggle (Tab / Gamepad Y)
        if is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            showing_standings = !showing_standings;
            self.state = GameState::CareerHub {
                selected_tier,
                selected_slot,
                calendar_tracks,
                showing_standings,
            };
            return;
        }

        // 4. Direct Garage Shortcut (G key)
        if is_key_pressed(KeyCode::G) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.garage_origin = GarageOrigin::CareerHub;
            self.state = GameState::Garage(GarageOrigin::CareerHub);
            return;
        }

        // 5. Global Tier Bumper / Key Shortcuts (Q / E, Gamepad LB / RB)
        if is_key_pressed(KeyCode::Q) || self.input.gamepad.snapshot.btn_lb_pressed {
            if selected_tier > 1 {
                self.audio.play_sfx(SfxType::UiMove);
                selected_tier -= 1;
                tier_changed = true;
            }
        }
        if is_key_pressed(KeyCode::E) || self.input.gamepad.snapshot.btn_rb_pressed {
            if selected_tier < 5 {
                self.audio.play_sfx(SfxType::UiMove);
                selected_tier += 1;
                tier_changed = true;
            }
        }

        let season_active = self
            .championship_session
            .as_ref()
            .map(|c| !c.is_completed)
            .unwrap_or(false);

        // 6. Focus Navigation (Tabs vs Calendar)
        match self.career_hub_focus {
            CareerHubFocus::Tabs => {
                // Horizontal navigation across championship tier tabs
                let prev_tab = is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A)
                    || self.input.gamepad.snapshot.dpad_left_pressed
                    || self.input.gamepad.snapshot.nav_left;
                let next_tab = is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D)
                    || self.input.gamepad.snapshot.dpad_right_pressed
                    || self.input.gamepad.snapshot.nav_right;

                if prev_tab && selected_tier > 1 {
                    selected_tier -= 1;
                    tier_changed = true;
                    self.audio.play_sfx(SfxType::UiMove);
                } else if next_tab && selected_tier < 5 {
                    selected_tier += 1;
                    tier_changed = true;
                    self.audio.play_sfx(SfxType::UiMove);
                }

                // Vertical navigation: Down moves focus to calendar slots
                if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::S)
                    || self.input.gamepad.snapshot.dpad_down_pressed
                    || self.input.gamepad.snapshot.nav_down
                {
                    self.career_hub_focus = CareerHubFocus::Calendar;
                    selected_slot = 0;
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
            CareerHubFocus::Calendar => {
                let num_slots = calendar_tracks.len();

                // Mouse selection on calendar rows
                if mouse_clicked {
                    let col_gap = scaler.s(14.0);
                    let left_w = full_w * 0.38;
                    let right_inner_x = x + left_w + col_gap + scaler.s(16.0);
                    let right_inner_w = full_w - left_w - col_gap - scaler.s(32.0);
                    let body_h = (sh - (tab_bar_y + tab_bar_h + scaler.s(12.0)) - scaler.s(60.0)).max(scaler.s(410.0));
                    let slot_h = ((body_h - scaler.s(80.0)) / num_slots as f32).clamp(scaler.s(24.0), scaler.s(36.0));
                    let slot_gap = scaler.s(4.0);
                    let start_ry = tab_bar_y + tab_bar_h + scaler.s(12.0) + scaler.s(16.0) + scaler.s(28.0) + scaler.s(10.0);

                    if mx >= right_inner_x && mx <= right_inner_x + right_inner_w {
                        for idx in 0..num_slots {
                            let sy = start_ry + idx as f32 * (slot_h + slot_gap);
                            if my >= sy && my <= sy + slot_h {
                                if selected_slot != idx {
                                    selected_slot = idx;
                                    self.audio.play_sfx(SfxType::UiMove);
                                }
                                break;
                            }
                        }
                    }
                }

                // Vertical navigation within calendar slots
                if is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::S)
                    || self.input.gamepad.snapshot.dpad_down_pressed
                    || self.input.gamepad.snapshot.nav_down
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    if selected_slot + 1 < num_slots {
                        selected_slot += 1;
                    }
                }

                if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::W)
                    || self.input.gamepad.snapshot.dpad_up_pressed
                    || self.input.gamepad.snapshot.nav_up
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    if selected_slot > 0 {
                        selected_slot -= 1;
                    } else {
                        // Returning to tabs from slot 0
                        self.career_hub_focus = CareerHubFocus::Tabs;
                    }
                }

                // Horizontal inputs while in Calendar
                let left_pressed = is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A)
                    || is_key_pressed(KeyCode::LeftBracket)
                    || is_key_pressed(KeyCode::Comma)
                    || self.input.gamepad.snapshot.dpad_left_pressed
                    || self.input.gamepad.snapshot.nav_left;
                let right_pressed = is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D)
                    || is_key_pressed(KeyCode::RightBracket)
                    || is_key_pressed(KeyCode::Period)
                    || self.input.gamepad.snapshot.dpad_right_pressed
                    || self.input.gamepad.snapshot.nav_right;

                let cur_track_id = calendar_tracks.get(selected_slot).cloned().unwrap_or_default();
                let is_slot_swappable = !season_active
                    && selected_tier > 1
                    && !crate::ui::career_hub::is_slot_mandatory(selected_tier, &cur_track_id);

                if is_slot_swappable {
                    if left_pressed {
                        if crate::ui::cycle_calendar_slot(selected_tier, &mut calendar_tracks, selected_slot, false) {
                            self.audio.play_sfx(SfxType::UiSelect);
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    } else if right_pressed {
                        if crate::ui::cycle_calendar_slot(selected_tier, &mut calendar_tracks, selected_slot, true) {
                            self.audio.play_sfx(SfxType::UiSelect);
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    }
                } else {
                    // On non-swappable slots (e.g. Tier 1 or mandatory tracks), horizontal input navigates tabs
                    if left_pressed && selected_tier > 1 {
                        selected_tier -= 1;
                        tier_changed = true;
                        self.audio.play_sfx(SfxType::UiMove);
                    } else if right_pressed && selected_tier < 5 {
                        selected_tier += 1;
                        tier_changed = true;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                }
            }
        }

        if tier_changed {
            calendar_tracks = if let Some(champ) = &self.championship_session {
                if champ.track_ids.len() == crate::ui::gt_default_calendar(selected_tier).len() {
                    champ.track_ids.clone()
                } else {
                    crate::ui::gt_default_calendar(selected_tier)
                }
            } else {
                crate::ui::gt_default_calendar(selected_tier)
            };
            selected_slot = 0;
            self.state = GameState::CareerHub {
                selected_tier,
                selected_slot,
                calendar_tracks,
                showing_standings,
            };
            return;
        }

        // 7. Advance Tier Gate (P key)
        if is_key_pressed(KeyCode::P) {
            if selected_tier == self.active_career_progress.level
                && self.active_career_progress.can_advance_tier()
            {
                if let Ok(new_tier) = self.active_career_progress.advance_tier() {
                    if let Some(champ) = &mut self.championship_session {
                        champ.update_from_career_rivals(new_tier, &self.active_career_progress.career_rivals);
                    }
                    if let Some(db) = &self.hof_db {
                        let _ = db.save_module_progress(&self.active_career_progress);
                    }
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.spawn_hud_alert(
                        format!("PROMOTED TO TIER {}! NEW CALENDAR UNLOCKED!", new_tier),
                        Palette::NEON_GOLD,
                    );
                    selected_tier = new_tier;
                    calendar_tracks = crate::ui::gt_default_calendar(new_tier);
                    selected_slot = 0;
                    self.career_hub_focus = CareerHubFocus::Tabs;
                    self.state = GameState::CareerHub {
                        selected_tier,
                        selected_slot,
                        calendar_tracks,
                        showing_standings,
                    };
                    return;
                }
            }
        }

        // 8. Re-run Latest Round (R key / Gamepad X)
        if is_key_pressed(KeyCode::R) || self.input.gamepad.snapshot.btn_x_pressed {
            if let Some(champ) = &mut self.championship_session {
                if let Some(track_id) = champ.cancel_latest_round() {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if let Some(db) = &self.hof_db {
                        let _ = db.delete_latest_race_history_entry_for_championship(&champ.name);
                    }
                    self.spawn_hud_alert("ROUND RE-RUN: LATEST RESULTS CANCELLED".to_string(), Palette::NEON_GOLD);
                    self.track = self
                        .track_manager
                        .load_track_by_slug(&track_id)
                        .unwrap_or_else(|_| tdrace_core::track::presets::classic_grand_prix());
                    self.init_race();
                    self.transition_iris_to(GameState::Countdown(3.5), 0.45);
                    return;
                }
            }
        }

        // 9. Confirm / Start / Resume Round (Enter, Space, KpEnter, Gamepad A)
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            if selected_tier <= self.active_career_progress.level {
                self.audio.play_sfx(SfxType::UiSelect);
                let need_new_champ = match &self.championship_session {
                    None => true,
                    Some(c) => c.is_completed || c.total_rounds() != calendar_tracks.len(),
                };

                if need_new_champ {
                    self.start_gt_career_tier_with_calendar(selected_tier, Some(calendar_tracks.clone()));
                } else {
                    // Resume existing round
                    if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
                        if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                            self.track = t;
                        }
                    }
                    self.init_race();
                }
                return;
            } else {
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // 10. Back to Modality Selection (Escape / Gamepad B / Back)
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_b_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.transition_fade_to(
                GameState::ModalitySelect {
                    category: ModalityCategory::SinglePlayer,
                    selected_idx: 2,
                    modal: None,
                },
                0.3,
            );
            return;
        }

        self.state = GameState::CareerHub {
            selected_tier,
            selected_slot,
            calendar_tracks,
            showing_standings,
        };
    }

    /// Updates inputs, vehicle selection, and turntable/rev stage animations for the Interactive Garage (`GameState::Garage`).
    pub fn update_garage(&mut self, origin: GarageOrigin, frame_dt: f32) {
        // 1. Turntable rotation & Revving state
        self.garage_turntable_angle += frame_dt * 0.45;

        self.garage_revving = is_key_down(KeyCode::Space) || self.input.gamepad.snapshot.btn_a_pressed;
        if self.garage_revving {
            self.garage_rev_rpm = (self.garage_rev_rpm + frame_dt * 3.5).min(1.0);
            self.garage_brake_heat = (self.garage_brake_heat + frame_dt * 0.4).min(1.0);
        } else {
            self.garage_rev_rpm = (self.garage_rev_rpm - frame_dt * 2.2).max(0.0);
            self.garage_brake_heat = (self.garage_brake_heat - frame_dt * 0.15).max(0.0);
        }

        // Active vehicle engine acoustic archetype & live telemetry
        let sound_type = self.resolve_active_sound_type();
        self.audio.set_engine_type(sound_type);

        let current_rpm = 1100.0 + self.garage_rev_rpm * 7400.0;
        let throttle = if self.garage_revving { 1.0 } else { 0.0 };
        self.audio.update_engine_telemetry(current_rpm, throttle, false, 0.0, 1, frame_dt);

        // 2. Return to Origin (Escape / Gamepad B / Back)
        if is_key_pressed(KeyCode::Escape)
            || self.input.gamepad.snapshot.btn_b_pressed
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.garage_gallery_mode {
                self.garage_gallery_mode = false;
                return;
            }
            self.audio.stop_all_loops();
            match origin {
                GarageOrigin::ModalitySelect => {
                    self.state = GameState::ModalitySelect {
                        category: ModalityCategory::Options,
                        selected_idx: 1,
                        modal: None,
                    };
                }
                GarageOrigin::Menu => {
                    self.state = GameState::Menu;
                }
                GarageOrigin::StartingGrid => {
                    self.state = GameState::StartingGrid;
                }
                GarageOrigin::CareerHub => {
                    let tier = self.active_career_progress.level.clamp(1, 5);
                    let calendar = if let Some(c) = &self.championship_session {
                        c.track_ids.clone()
                    } else {
                        crate::ui::gt_default_calendar(tier)
                    };
                    self.career_hub_focus = CareerHubFocus::Tabs;
                    self.state = GameState::CareerHub {
                        selected_tier: tier,
                        selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                        calendar_tracks: calendar,
                        showing_standings: false,
                    };
                }
            }
            return;
        }

        // 3. Toggle View Mode (Tab / Gamepad Y) - only when not in gallery mode
        if !self.garage_gallery_mode && (is_key_pressed(KeyCode::Tab) || self.input.gamepad.snapshot.btn_y_pressed) {
            self.audio.play_sfx(SfxType::UiMove);
            self.garage_view_mode = match self.garage_view_mode {
                crate::ui::GarageViewMode::Lateral => crate::ui::GarageViewMode::TopDownTurntable,
                crate::ui::GarageViewMode::TopDownTurntable => crate::ui::GarageViewMode::Lateral,
            };
        }

        // 4. Toggle Fleet Gallery (C / F / Gamepad X)
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::F) || self.input.gamepad.snapshot.btn_x_pressed {
            self.audio.play_sfx(SfxType::UiMove);
            self.garage_gallery_mode = !self.garage_gallery_mode;
            if self.garage_gallery_mode {
                self.garage_gallery_filter = crate::ui::garage::module_to_gallery_filter(self.active_module_id);
                let filtered_models = crate::catalog::get_models_for_module(self.active_module_id);
                let tier_models = crate::catalog::get_models_for_module_and_tier(self.active_module_id, self.garage_tier);
                if let Some(target_car) = tier_models.get(self.garage_car_idx) {
                    if let Some(pos) = filtered_models.iter().position(|m| m.id == target_car.id) {
                        self.garage_gallery_sel = pos;
                    } else {
                        self.garage_gallery_sel = 0;
                    }
                } else {
                    self.garage_gallery_sel = 0;
                }
            }
        }

        if self.garage_gallery_mode {
            let total_tabs = crate::ui::garage::GALLERY_MODULES.len();
            self.garage_gallery_filter = self.garage_gallery_filter.min(total_tabs - 1);
            let mut mod_changed = false;

            // Direct module selection via number keys 1..=5
            if is_key_pressed(KeyCode::Key1) {
                self.garage_gallery_filter = 0;
                mod_changed = true;
            } else if is_key_pressed(KeyCode::Key2) {
                self.garage_gallery_filter = 1;
                mod_changed = true;
            } else if is_key_pressed(KeyCode::Key3) {
                self.garage_gallery_filter = 2;
                mod_changed = true;
            } else if is_key_pressed(KeyCode::Key4) {
                self.garage_gallery_filter = 3;
                mod_changed = true;
            } else if is_key_pressed(KeyCode::Key5) {
                self.garage_gallery_filter = 4;
                mod_changed = true;
            }

            // Tab / Shift-Tab cycle module tabs
            if is_key_pressed(KeyCode::Tab) {
                let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                if shift {
                    self.garage_gallery_filter = (self.garage_gallery_filter + total_tabs - 1) % total_tabs;
                } else {
                    self.garage_gallery_filter = (self.garage_gallery_filter + 1) % total_tabs;
                }
                mod_changed = true;
            }

            // Q / E or Gamepad LB / RB cycle module tabs
            if is_key_pressed(KeyCode::Q) || self.input.gamepad.snapshot.btn_lb_pressed {
                self.garage_gallery_filter = (self.garage_gallery_filter + total_tabs - 1) % total_tabs;
                mod_changed = true;
            } else if is_key_pressed(KeyCode::E) || self.input.gamepad.snapshot.btn_rb_pressed {
                self.garage_gallery_filter = (self.garage_gallery_filter + 1) % total_tabs;
                mod_changed = true;
            }

            let (sw, sh) = (screen_width_safe(), screen_height_safe());
            let (mx, my) = mouse_position_safe();
            let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);

            // Mouse click on module tabs
            if mouse_clicked {
                for i in 0..total_tabs {
                    let (tx, ty, tw, th) = crate::ui::garage::garage_gallery_tab_rect(sw, sh, i);
                    if mx >= tx && mx <= tx + tw && my >= ty && my <= ty + th {
                        if self.garage_gallery_filter != i {
                            self.garage_gallery_filter = i;
                            mod_changed = true;
                        }
                        break;
                    }
                }
            }

            if mod_changed {
                self.garage_gallery_sel = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }

            let target_mod = crate::ui::garage::gallery_filter_to_module(self.garage_gallery_filter);
            let filtered_models = crate::catalog::get_models_for_module(target_mod);
            let n = filtered_models.len();

            let mut confirm_selection = is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::KpEnter)
                || self.input.gamepad.snapshot.btn_confirm_pressed;

            // Mouse click on car cards in gallery
            if mouse_clicked && !mod_changed {
                for i in 0..n {
                    let (cx, cy, cw, ch) = crate::ui::garage::garage_gallery_card_rect(sw, sh, i);
                    if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                        if self.garage_gallery_sel == i {
                            confirm_selection = true;
                        } else {
                            self.garage_gallery_sel = i;
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                        break;
                    }
                }
            }

            if n > 0 {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    self.garage_gallery_sel = (self.garage_gallery_sel + n - 1) % n;
                    self.audio.play_sfx(SfxType::UiMove);
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    self.garage_gallery_sel = (self.garage_gallery_sel + 1) % n;
                    self.audio.play_sfx(SfxType::UiMove);
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.garage_gallery_sel >= 4 {
                        self.garage_gallery_sel -= 4;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                }
                if is_key_pressed(KeyCode::Down) {
                    if self.garage_gallery_sel + 4 < n {
                        self.garage_gallery_sel += 4;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                }
                if confirm_selection {
                    if let Some(car) = filtered_models.get(self.garage_gallery_sel) {
                        self.active_module_id = car.module_id;
                        self.garage_tier = car.tier;
                        let tier_models = crate::catalog::get_models_for_module_and_tier(car.module_id, car.tier);
                        if let Some(pos) = tier_models.iter().position(|m| m.id == car.id) {
                            self.garage_car_idx = pos;
                        }
                        self.garage_gallery_mode = false;
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                }
            }
            return;
        }

        // 5. Switching active module (1..=5)
        let prev_mod = self.active_module_id;
        if is_key_pressed(KeyCode::Key1) {
            self.active_module_id = "gt";
        } else if is_key_pressed(KeyCode::Key2) {
            self.active_module_id = "rally";
        } else if is_key_pressed(KeyCode::Key3) {
            self.active_module_id = "kart";
        } else if is_key_pressed(KeyCode::Key4) {
            self.active_module_id = "nascar";
        } else if is_key_pressed(KeyCode::Key5) {
            self.active_module_id = "extreme_offroad";
        }
        if self.active_module_id != prev_mod {
            self.garage_car_idx = 0;
            self.audio.play_sfx(SfxType::UiMove);
        }

        // 6. Tier Navigation (Q / E or Up / Down or Gamepad D-pad Up / Down or LB / RB)
        if is_key_pressed(KeyCode::Q)
            || is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || self.input.gamepad.snapshot.dpad_up_pressed
            || self.input.gamepad.snapshot.btn_lb_pressed
        {
            if self.garage_tier > 1 {
                self.garage_tier -= 1;
                self.garage_car_idx = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        } else if is_key_pressed(KeyCode::E)
            || is_key_pressed(KeyCode::Down)
            || self.input.gamepad.snapshot.dpad_down_pressed
            || self.input.gamepad.snapshot.btn_rb_pressed
        {
            if self.garage_tier < 5 {
                self.garage_tier += 1;
                self.garage_car_idx = 0;
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // 7. Car Navigation within Tier (A / D or Left / Right or Gamepad D-pad Left / Right)
        let models = crate::catalog::get_models_for_module_and_tier(self.active_module_id, self.garage_tier);
        let num_models = models.len();
        if num_models > 0 {
            if is_key_pressed(KeyCode::Left)
                || is_key_pressed(KeyCode::A)
                || self.input.gamepad.snapshot.dpad_left_pressed
            {
                self.garage_car_idx = (self.garage_car_idx + num_models - 1) % num_models;
                self.audio.play_sfx(SfxType::UiMove);
            } else if is_key_pressed(KeyCode::Right)
                || is_key_pressed(KeyCode::D)
                || self.input.gamepad.snapshot.dpad_right_pressed
            {
                self.garage_car_idx = (self.garage_car_idx + 1) % num_models;
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // 8. Confirm / Select Car for Race (Enter / Gamepad A / Mouse Click)
        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let (sel_bx, sel_by, sel_bw, sel_bh) = crate::ui::garage_select_button_rect(sw, sh);
        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);
        let select_btn_clicked = mouse_clicked && mx >= sel_bx && mx <= sel_bx + sel_bw && my >= sel_by && my <= sel_by + sel_bh;

        // Also allow clicking directly on car cards in bottom carousel
        if mouse_clicked {
            for i in 0..num_models {
                let (cx, cy, cw, ch) = crate::ui::garage_car_card_rect(sw, sh, i, num_models);
                if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                    if self.garage_car_idx != i {
                        self.garage_car_idx = i;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    break;
                }
            }
        }

        let buy_pressed = is_key_pressed(KeyCode::B);
        let confirm_pressed = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || select_btn_clicked;

        if buy_pressed || confirm_pressed {
            if let Some(active_car) = models.get(self.garage_car_idx) {
                let is_unlocked = self.active_module_id == "classic"
                    || self.active_career_progress.is_car_unlocked(active_car.id, self.is_dev_mode())
                    || self.is_dev_mode();

                if is_unlocked {
                    if confirm_pressed {
                        self.car_choice = active_car.base_car_choice;
                        self.current_visual_type = active_car.visual_type;
                        self.selected_car_model_id = Some(active_car.id);
                        self.free_car_selection = true;
                        self.audio.play_sfx(SfxType::UiSelect);
                        match origin {
                            GarageOrigin::StartingGrid => {
                                self.audio.stop_all_loops();
                                self.rebuild_roster_participants();
                                self.state = GameState::StartingGrid;
                            }
                            GarageOrigin::Menu => {
                                self.audio.stop_all_loops();
                                self.state = GameState::Menu;
                            }
                            GarageOrigin::CareerHub => {
                                self.audio.stop_all_loops();
                                let tier = self.active_career_progress.level.clamp(1, 5);
                                let calendar = if let Some(c) = &self.championship_session {
                                    c.track_ids.clone()
                                } else {
                                    crate::ui::gt_default_calendar(tier)
                                };
                                self.career_hub_focus = CareerHubFocus::Tabs;
                                self.state = GameState::CareerHub {
                                    selected_tier: tier,
                                    selected_slot: self.championship_session.as_ref().map(|c| c.current_round).unwrap_or(0),
                                    calendar_tracks: calendar,
                                    showing_standings: false,
                                };
                            }
                            GarageOrigin::ModalitySelect => {
                                // Stay or return
                            }
                        }
                    }
                } else if self.active_career_progress.can_buy_car(active_car.id, active_car.tier) {
                    if let Ok(()) = self.active_career_progress.buy_car(active_car.id, active_car.tier) {
                        if let Some(db) = &self.hof_db {
                            let _ = db.save_module_progress(&self.active_career_progress);
                        }
                        self.spawn_hud_alert(
                            format!(
                                "PURCHASED {} FOR {} XP! BALANCE: {} XP",
                                active_car.name,
                                ModuleCareerProgress::car_cost(active_car.tier),
                                self.active_career_progress.xp
                            ),
                            Palette::NEON_GOLD,
                        );
                        self.audio.play_sfx(SfxType::UiSelect);
                    }
                } else {
                    let cost = ModuleCareerProgress::car_cost(active_car.tier);
                    if self.active_career_progress.level < active_car.tier as u32 {
                        self.spawn_hud_alert(
                            format!(
                                "LOCKED: CAREER TIER {} REQUIRED (CURRENT: TIER {})",
                                active_car.tier, self.active_career_progress.level
                            ),
                            Palette::RED,
                        );
                    } else {
                        self.spawn_hud_alert(
                            format!(
                                "CANNOT AFFORD: REQUIRES {} XP (WALLET: {} XP)",
                                cost, self.active_career_progress.xp
                            ),
                            Palette::RED,
                        );
                    }
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
        }
    }

    /// Returns the active circuit identifier when in Menu or Starting Grid context.
    pub fn active_menu_track_id(&self) -> Option<&str> {
        if self.game_mode == GameMode::Career {
            self.championship_session
                .as_ref()
                .and_then(|c| c.current_track_id())
                .or(Some(self.track_choice.track_id()))
        } else if self.menu_origin == MenuOrigin::StartingGrid {
            Some(self.track_choice.track_id())
        } else {
            None
        }
    }

    /// Transitions cleanly into full circuit top-down viewer with maximum zoom out.
    pub fn open_circuit_viewer(&mut self, track: Track, title: String, origin: CircuitViewerOrigin) {
        self.audio.play_sfx(SfxType::UiSelect);
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        let state = CircuitViewerState::new(
            track,
            title,
            self.active_module_id.to_string(),
            origin,
            sw,
            sh,
        );
        self.circuit_viewer_state = Some(state);
        self.state = GameState::CircuitViewer(origin);
    }

    /// Updates inputs (pan, zoom, reset fit, exit) for the full-circuit topdown viewer.
    pub fn update_circuit_viewer(&mut self, origin: CircuitViewerOrigin, dt: f32) {
        if let Some(ref mut state) = self.circuit_viewer_state {
            let exit_requested = crate::ui::circuit_viewer::handle_circuit_viewer_input(
                state,
                dt,
                &self.input.gamepad.snapshot,
            );
            if exit_requested {
                self.audio.play_sfx(SfxType::UiSelect);
                self.circuit_viewer_state = None;
                match origin {
                    CircuitViewerOrigin::Menu => self.state = GameState::Menu,
                    CircuitViewerOrigin::TrackManager => {
                        let has_module_customs = !self.track_manager.module_custom_tracks(self.active_module_id).is_empty();
                        let has_drafts = !self.track_manager.draft_track_choices().is_empty();
                        let (target_tab, target_filter) = if !has_module_customs && has_drafts {
                            (TrackManagerTab::Drafts, ModuleFilter::Drafts)
                        } else {
                            (TrackManagerTab::Main, ModuleFilter::for_module(self.active_module_id))
                        };
                        self.state = GameState::TrackManager {
                            active_tab: target_tab,
                            module_filter: target_filter,
                            selected_idx: 0,
                            modal: TrackManagerModal::None,
                        };
                    }
                    CircuitViewerOrigin::StartingGrid => self.state = GameState::StartingGrid,
                }
            }
        } else {
            self.state = match origin {
                CircuitViewerOrigin::Menu => GameState::Menu,
                CircuitViewerOrigin::StartingGrid => GameState::StartingGrid,
                CircuitViewerOrigin::TrackManager => GameState::Menu,
            };
        }
    }

    /// Renders the full-circuit topdown viewer screen with complete in-game graphics and HUD overlay.
    pub fn render_circuit_viewer(&self) {
        if let Some(ref state) = self.circuit_viewer_state {
            crate::ui::circuit_viewer::render_circuit_viewer_screen(&self.fonts, state);
        }
    }

    /// Menu input navigation (Keyboard + Gamepad D-pad/Analog Sticks/buttons).
    pub fn update_menu(&mut self) {
        // Check for gamepad mapping changes on disk when in/reloading the main menu
        self.input.gamepad.check_and_reload_profile();

        // If exit confirmation modal is currently open:
        if self.show_exit_confirm {
            if self.exit_confirm_modal.is_none() {
                self.exit_confirm_modal = Some(UniversalConfirmModal::quit_game());
            }
            if let Some(ref mut modal) = self.exit_confirm_modal {
                let sw = screen_width_safe();
                let sh = screen_height_safe();
                let scaler = UiScaler::new(sw, sh);
                let theme = CabinetTheme::cyberpunk_neon();
                let mut ctx = CabinetContext::new(
                    &scaler,
                    &self.fonts,
                    &theme,
                    &self.input.gamepad.snapshot,
                    1.0 / 60.0,
                )
                .with_audio(Some(&self.audio));

                // Direct legacy shortcuts for Y / N
                if is_key_pressed(KeyCode::Y) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::N) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.show_exit_confirm = false;
                    self.exit_confirm_modal = None;
                    return;
                }

                let action = modal.update(&mut ctx);
                match action {
                    ScreenAction::Quit => {
                        std::process::exit(0);
                    }
                    ScreenAction::Pop => {
                        self.show_exit_confirm = false;
                        self.exit_confirm_modal = None;
                    }
                    _ => {}
                }
            }
            return;
        }

        // If Arcade Settings Modal is open in the main menu, update it and return:
        if let Some(ref mut modal) = self.settings_modal {
            let (sw, sh) = (screen_width_safe(), screen_height_safe());
            let scaler = UiScaler::new(sw, sh);
            let theme = CabinetTheme::default();
            let mut ctx = CabinetContext {
                scaler: &scaler,
                fonts: &self.fonts,
                theme: &theme,
                gamepad: &self.input.gamepad.snapshot,
                dt: 1.0 / 60.0,
                audio: Some(&self.audio),
            };

            let action = modal.update(&mut ctx);
            if matches!(action, ScreenAction::Pop) {
                let saved = modal.is_saved;
                self.close_settings_modal(saved);
                if saved {
                    self.audio.play_sfx(SfxType::UiSelect);
                }
            }
            return;
        }

        // Open Arcade Settings Modal (O key)
        if is_key_pressed(KeyCode::O) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.open_settings_modal();
            return;
        }

        // Direct Garage Showroom shortcut (G key)
        if is_key_pressed(KeyCode::G) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.garage_origin = GarageOrigin::Menu;
            self.garage_tier = self.current_race_required_tier();
            self.garage_car_idx = 0;
            self.state = GameState::Garage(GarageOrigin::Menu);
            return;
        }

        // Return to Modality Selection Screen or Starting Grid (Escape key or Gamepad B / Cancel / Back / Tab)
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Tab)
            || self.input.gamepad.snapshot.btn_cancel_pressed
            || self.input.gamepad.snapshot.btn_b_pressed
            || self.input.gamepad.snapshot.btn_back_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            if self.menu_origin == MenuOrigin::StartingGrid {
                self.state = GameState::StartingGrid;
                return;
            }
            let initial_category = match self.game_mode {
                GameMode::SplitScreen => ModalityCategory::Multiplayer,
                _ => ModalityCategory::SinglePlayer,
            };
            let initial_idx = match self.game_mode {
                GameMode::StandardRace => 0,
                GameMode::ExperimentalRace => 1,
                GameMode::Career => 2,
                GameMode::TimeTrial => 3,
                GameMode::FreeRide => 4,
                GameMode::SplitScreen => 0,
            };
            self.transition_fade_to(
                GameState::ModalitySelect {
                    category: initial_category,
                    selected_idx: initial_idx,
                    modal: None,
                },
                0.3,
            );
            return;
        }

        // Open Controls & Gamepad Screen (K key)
        if is_key_pressed(KeyCode::K) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::ControlsHelp(false);
            return;
        }

        // Quick Championship trigger for GT World Challenge (G / F key)
        if self.active_module_id == "gt"
            && (is_key_pressed(KeyCode::F) || is_key_pressed(KeyCode::G))
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.start_gt_championship();
            return;
        }

        // Quick Championship trigger for NASCAR Cup Series (F key)
        if self.active_module_id == "nascar" && is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.start_nascar_championship();
            return;
        }

        // Quick Championship trigger for Extreme Off-Road & Stunt Arenas (F key)
        if self.active_module_id == "extreme_offroad" && is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiSelect);
            self.start_extreme_offroad_championship();
            return;
        }

        // Open Track Manager (T key)
        if is_key_pressed(KeyCode::T) {
            self.audio.play_sfx(SfxType::UiSelect);
            let has_module_customs = !self.track_manager.module_custom_tracks(self.active_module_id).is_empty();
            let has_drafts = !self.track_manager.draft_track_choices().is_empty();
            let (target_tab, target_filter) = if !has_module_customs && has_drafts {
                (TrackManagerTab::Drafts, ModuleFilter::Drafts)
            } else {
                (TrackManagerTab::Main, ModuleFilter::for_module(self.active_module_id))
            };
            self.state = GameState::TrackManager {
                active_tab: target_tab,
                module_filter: target_filter,
                selected_idx: 0,
                modal: TrackManagerModal::None,
            };
            return;
        }

        // Open Player Profile & Career History Screen (P key or Gamepad Y)
        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            self.audio.play_sfx(SfxType::UiSelect);
            self.profile_origin = ProfileOrigin::Menu;
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

        let available_tracks = self.filtered_menu_tracks();
        let has_tm_entry = self.menu_track_filter == TrackCatalogFilter::Custom;
        let total_items = if has_tm_entry {
            available_tracks.len() + 1
        } else {
            available_tracks.len()
        };
        if total_items == 0 {
            self.menu_track_idx = 0;
        } else if self.menu_track_idx >= total_items {
            self.menu_track_idx = 0;
        }

        // 1. Catalog Filter Tab Cycling (Left/Right: Arrows / A/D / Gamepad D-pad / Left Stick X)
        if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::A)
            || self.input.gamepad.snapshot.dpad_left_pressed
            || self.input.gamepad.snapshot.nav_left
        {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_track_filter = self.menu_track_filter.prev();
            self.menu_track_idx = 0;
        }
        if is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || self.input.gamepad.snapshot.dpad_right_pressed
            || self.input.gamepad.snapshot.nav_right
        {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_track_filter = self.menu_track_filter.next();
            self.menu_track_idx = 0;
        }

        // Tab key also cycles filter tabs
        if is_key_pressed(KeyCode::Tab) {
            self.audio.play_sfx(SfxType::UiMove);
            self.menu_track_filter = self.menu_track_filter.next();
            self.menu_track_idx = 0;
        }

        // 2. Active Column Track Navigation (Up/Down: Arrows / W/S / Gamepad D-pad / Left Stick Y)
        if total_items > 0 {
            if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || self.input.gamepad.snapshot.nav_up {
                self.audio.play_sfx(SfxType::UiMove);
                if self.menu_track_idx == 0 {
                    self.menu_track_idx = total_items - 1;
                } else {
                    self.menu_track_idx -= 1;
                }
            }
            if is_key_pressed(KeyCode::Down) || self.input.gamepad.snapshot.nav_down {
                self.audio.play_sfx(SfxType::UiMove);
                self.menu_track_idx = (self.menu_track_idx + 1) % total_items;
            }
        }

        // Launch Championship / Tournament (F key)
        if is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiSelect);
            match self.active_module_id {
                "gt" | "gt_challenge" => {
                    self.start_gt_championship();
                    return;
                }
                "nascar" => {
                    self.start_nascar_championship();
                    return;
                }
                "extreme_offroad" => {
                    self.start_extreme_offroad_championship();
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

        // Direct Circuit Manager shortcut (T key)
        if is_key_pressed(KeyCode::T) {
            self.audio.play_sfx(SfxType::UiSelect);
            let is_on_draft = self.menu_track_filter == TrackCatalogFilter::Custom
                && available_tracks.get(self.menu_track_idx)
                    .and_then(|t| self.track_manager.custom_tracks.iter().find(|c| c.id == t.track_id()))
                    .map(|c| c.category == TrackCategory::Draft)
                    .unwrap_or(false);
            let has_module_customs = !self.track_manager.module_custom_tracks(self.active_module_id).is_empty();
            let has_drafts = !self.track_manager.draft_track_choices().is_empty();
            let (target_tab, target_filter, target_idx) = if self.menu_track_filter == TrackCatalogFilter::Custom && (is_on_draft || (!has_module_customs && has_drafts)) {
                let sel_idx = if is_on_draft {
                    let track_id = available_tracks[self.menu_track_idx].track_id();
                    self.track_manager.draft_track_choices().iter().position(|t| t.track_id() == track_id).unwrap_or(0)
                } else {
                    0
                };
                (TrackManagerTab::Drafts, ModuleFilter::Drafts, sel_idx)
            } else {
                let mod_filter = ModuleFilter::for_module(self.active_module_id);
                let sel_idx = if self.menu_track_idx < available_tracks.len() {
                    let track_id = available_tracks[self.menu_track_idx].track_id();
                    self.track_manager.filtered_main_track_choices(mod_filter)
                        .iter()
                        .position(|t| t.track_id() == track_id)
                        .unwrap_or(0)
                } else {
                    0
                };
                (TrackManagerTab::Main, mod_filter, sel_idx)
            };
            self.state = GameState::TrackManager {
                active_tab: target_tab,
                module_filter: target_filter,
                selected_idx: target_idx,
                modal: TrackManagerModal::None,
            };
            return;
        }

        // Developer Mode: Direct Dev Workbench shortcut (Ctrl+D or F12)
        let ctrl_down = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);
        if crate::storage::is_dev_mode()
            && (is_key_pressed(KeyCode::F12) || (ctrl_down && is_key_pressed(KeyCode::D)))
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::TrackManager {
                active_tab: TrackManagerTab::DevWorkbench,
                module_filter: ModuleFilter::for_module(self.active_module_id),
                selected_idx: 0,
                modal: TrackManagerModal::None,
            };
            return;
        }

        // Full Circuit Top-Down Inspection View ([X], [Z], Gamepad X, or clicking the preview card)
        let (sw, sh) = (screen_width_safe(), screen_height_safe());
        let (mx, my) = mouse_position_safe();
        let mouse_clicked = is_mouse_button_pressed(macroquad::input::MouseButton::Left);
        let clicked_preview = if self.menu_track_idx < available_tracks.len() {
            let track_choice = &available_tracks[self.menu_track_idx];
            let active_tid = self.active_menu_track_id();
            let is_sel_active = active_tid.map_or(false, |aid| aid == track_choice.track_id());
            let is_sel_locked = !self.is_track_unlocked(track_choice.track_id());
            let has_status_banner = self.game_mode == GameMode::Career || is_sel_locked || is_sel_active;
            let (px, py, pw, ph) = crate::ui::track_select_preview_rect(
                sw,
                sh,
                self.active_module_id == "gt",
                has_status_banner,
                track_choice.is_user_custom(),
            );
            mouse_clicked && mx >= px && mx <= px + pw && my >= py && my <= py + ph
        } else {
            false
        };

        if is_key_pressed(KeyCode::X)
            || is_key_pressed(KeyCode::Z)
            || self.input.gamepad.snapshot.btn_x_pressed
            || clicked_preview
        {
            if self.menu_track_idx < available_tracks.len() {
                let track_choice = &available_tracks[self.menu_track_idx];
                if let Some(loaded_track) = resolve_track_for_menu(track_choice) {
                    self.open_circuit_viewer(
                        loaded_track,
                        track_choice.title().to_string(),
                        CircuitViewerOrigin::Menu,
                    );
                    return;
                }
            }
        }

        // Start race or open Track Manager (Space, Enter, or Gamepad Confirm [A / South / Start])
        if is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed
        {
            if self.menu_track_idx < available_tracks.len() {
                let track_choice = &available_tracks[self.menu_track_idx];
                let track_id = track_choice.track_id();
                if self.game_mode == GameMode::Career && self.menu_origin == MenuOrigin::StartingGrid {
                    let active_id = self.championship_session.as_ref()
                        .and_then(|c| c.current_track_id())
                        .unwrap_or_else(|| self.track_choice.track_id());
                    if track_id == active_id {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.state = GameState::StartingGrid;
                        return;
                    } else {
                        self.audio.play_sfx(SfxType::UiMove);
                        return;
                    }
                }
                if !self.is_track_unlocked(track_id) {
                    self.audio.play_sfx(SfxType::UiMove);
                    return;
                }
                self.audio.play_sfx(SfxType::UiSelect);
                self.track_choice = track_choice.clone();
                let loaded = resolve_track_for_menu(&self.track_choice);
                let effective_module = loaded.as_ref().and_then(|t| t.module_id.as_deref()).unwrap_or(self.active_module_id);
                self.car_choice = resolve_predefined_car_for_track(loaded.as_ref(), effective_module);
                self.init_race();
            } else if has_tm_entry {
                let has_module_customs = !self.track_manager.module_custom_tracks(self.active_module_id).is_empty();
                let has_drafts = !self.track_manager.draft_track_choices().is_empty();
                let (target_tab, target_filter) = if !has_module_customs && has_drafts {
                    (TrackManagerTab::Drafts, ModuleFilter::Drafts)
                } else {
                    (TrackManagerTab::Main, ModuleFilter::for_module(self.active_module_id))
                };
                self.state = GameState::TrackManager {
                    active_tab: target_tab,
                    module_filter: target_filter,
                    selected_idx: 0,
                    modal: TrackManagerModal::None,
                };
                return;
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
                ref track_title,
                mut cursor_idx,
            } => {
                let sw = screen_width_safe();
                let sh = screen_height_safe();
                let (_, _, _, _, btn_layout) = confirm_modal_layout(sw, sh);

                if NavGrid2D::check_mouse_hover(btn_layout.cancel_rect) {
                    if cursor_idx != 0 {
                        self.audio.play_sfx(SfxType::UiMove);
                        cursor_idx = 0;
                    }
                }
                if NavGrid2D::check_mouse_hover(btn_layout.confirm_rect) {
                    if cursor_idx != 1 {
                        self.audio.play_sfx(SfxType::UiMove);
                        cursor_idx = 1;
                    }
                }

                let cancel_clicked = NavGrid2D::check_mouse_click(btn_layout.cancel_rect);
                let confirm_clicked = NavGrid2D::check_mouse_click(btn_layout.confirm_rect);

                // Button cursor navigation (Left / Right / A / D / Up / Down / W / S / Tab / Gamepad D-pad & Sticks)
                if is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A)
                    || self.input.gamepad.snapshot.dpad_left_pressed
                    || self.input.gamepad.snapshot.nav_left
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    cursor_idx = if cursor_idx == 0 { 1 } else { 0 };
                } else if is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D)
                    || self.input.gamepad.snapshot.dpad_right_pressed
                    || self.input.gamepad.snapshot.nav_right
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    cursor_idx = if cursor_idx == 1 { 0 } else { 1 };
                } else if is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Down)
                    || is_key_pressed(KeyCode::W)
                    || is_key_pressed(KeyCode::S)
                    || is_key_pressed(KeyCode::Tab)
                    || self.input.gamepad.snapshot.nav_up
                    || self.input.gamepad.snapshot.nav_down
                {
                    self.audio.play_sfx(SfxType::UiMove);
                    cursor_idx = 1 - cursor_idx;
                }

                let is_confirmed = is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed;

                if confirm_clicked || (is_confirmed && cursor_idx == 1) || is_key_pressed(KeyCode::Y) {
                    let tid = track_id.clone();
                    let is_dev_workbench = crate::storage::is_dev_mode() && active_tab == TrackManagerTab::DevWorkbench;
                    if is_dev_workbench {
                        let target_module = module_filter.id();
                        let _ = self.track_manager.delete_track_from_module(&tid, target_module);
                    } else {
                        let _ = self.track_manager.delete_custom_track(&tid);
                    }
                    self.clear_circuit_history(&tid);
                    self.audio.play_sfx(SfxType::UiSelect);
                    let list_len = if is_dev_workbench {
                        self.track_manager.filtered_main_track_choices(module_filter).len()
                    } else {
                        let mod_id = module_filter.id().unwrap_or("classic");
                        self.track_manager.module_custom_tracks(mod_id).len()
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

                if cancel_clicked
                    || (is_confirmed && cursor_idx == 0)
                    || is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::N)
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
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
                    modal: TrackManagerModal::ConfirmDelete {
                        track_id: track_id.clone(),
                        track_title: track_title.clone(),
                        cursor_idx,
                    },
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
                    let target_mods: Vec<&str> = PROMOTION_MODULES
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| selected_mask[*i])
                        .map(|(_, (mod_id, _, _, _))| *mod_id)
                        .collect();
                    if target_mods.is_empty() {
                        let fallback_mod = module_filter.id().unwrap_or("classic");
                        let _ = self.track_manager.promote_track_to_modules(track_id, &[fallback_mod]);
                    } else {
                        let _ = self.track_manager.promote_track_to_modules(track_id, &target_mods);
                    }
                    self.audio.play_sfx(SfxType::UiSelect);
                    let list_len = self.track_manager.filtered_main_track_choices(module_filter).len();
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
            TrackManagerModal::CloneBeforeEdit {
                ref track_choice,
                ref track_title,
                is_dev_mode,
                ref mut dev_choice,
            } => {
                if is_dev_mode {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Key1) {
                        *dev_choice = 0;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Key2) {
                        *dev_choice = 1;
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                }

                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    self.audio.play_sfx(SfxType::UiSelect);
                    if is_dev_mode && *dev_choice == 0 {
                        // Developer mode: Overwrite Git-Tracked Preset directly
                        let track = self
                            .track_manager
                            .load_track(track_choice)
                            .unwrap_or_else(|_| classic_grand_prix());
                        let file_path = if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
                            let mod_id = TrackManager::preset_module(track_choice.track_id()).unwrap_or("classic");
                            Some(git_tracks_dir.join(mod_id).join(format!("{}.json", track_choice.track_id())).to_string_lossy().to_string())
                        } else {
                            None
                        };
                        self.track_choice = track_choice.clone();
                        self.track = track.clone();
                        self.editor_return_track_manager = Some((active_tab, module_filter, selected_idx));
                        self.enter_track_editor_with_path(track, file_path);
                        return;
                    } else {
                        // Clone track to Drafts and open in editor
                        if let Ok((cloned_track, file_path)) = self.track_manager.clone_track(track_choice) {
                            let file_stem = std::path::Path::new(&file_path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("cloned_track")
                                .to_string();
                            self.track_choice = TrackChoice::Custom {
                                id: file_stem.clone(),
                                title: cloned_track.name.clone(),
                                description: cloned_track.description.clone(),
                                path: file_path.clone(),
                            };
                            self.track = cloned_track.clone();
                            let drafts = self.track_manager.draft_track_choices();
                            let sel = drafts.iter().position(|t| t.track_id() == file_stem).unwrap_or(0);
                            self.editor_return_track_manager = Some((TrackManagerTab::Drafts, ModuleFilter::Drafts, sel));
                            self.enter_track_editor_with_path(cloned_track, Some(file_path));
                            return;
                        }
                    }
                }

                if is_key_pressed(KeyCode::Escape)
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
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
                    modal: TrackManagerModal::CloneBeforeEdit {
                        track_choice: track_choice.clone(),
                        track_title: track_title.clone(),
                        is_dev_mode,
                        dev_choice: *dev_choice,
                    },
                };
                return;
            }
            TrackManagerModal::ConfirmPromoteToPreset {
                ref track_id,
                ref track_title,
                ref target_module,
            } => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Y)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    let tid = track_id.clone();
                    if let Ok(_p) = self.track_manager.promote_custom_track_to_git_preset(&tid) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    } else {
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    let list_len = self.track_manager.filtered_main_track_choices(module_filter).len();
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

                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::N)
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
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
                    modal: TrackManagerModal::ConfirmPromoteToPreset {
                        track_id: track_id.clone(),
                        track_title: track_title.clone(),
                        target_module: target_module.clone(),
                    },
                };
                return;
            }
            TrackManagerModal::ConfirmDemoteToCustom {
                ref track_id,
                ref track_title,
            } => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::KpEnter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Y)
                    || self.input.gamepad.snapshot.btn_confirm_pressed
                    || self.input.gamepad.snapshot.btn_a_pressed
                {
                    let tid = track_id.clone();
                    if let Ok(_p) = self.track_manager.demote_preset_to_custom_track(&tid) {
                        self.audio.play_sfx(SfxType::UiSelect);
                    } else {
                        self.audio.play_sfx(SfxType::UiMove);
                    }
                    let list_len = self.track_manager.filtered_main_track_choices(module_filter).len();
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

                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::N)
                    || self.input.gamepad.snapshot.btn_back_pressed
                    || self.input.gamepad.snapshot.btn_b_pressed
                {
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
                    modal: TrackManagerModal::ConfirmDemoteToCustom {
                        track_id: track_id.clone(),
                        track_title: track_title.clone(),
                    },
                };
                return;
            }
            TrackManagerModal::None => {}
        }

        // --- NON-MODAL NAVIGATION AND ACTIONS ---
        // 1. Switch Motorsport Module (Left/Right arrows, A/D, LeftBracket/RightBracket, Gamepad Nav Left/Right)
        if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::A)
            || is_key_pressed(KeyCode::LeftBracket)
            || self.input.gamepad.snapshot.nav_left
        {
            self.audio.play_sfx(SfxType::UiMove);
            module_filter = module_filter.prev();
            active_tab = if module_filter == ModuleFilter::Drafts {
                TrackManagerTab::Drafts
            } else if active_tab == TrackManagerTab::Drafts {
                TrackManagerTab::Main
            } else {
                active_tab
            };
            selected_idx = 0;
        }
        if is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::D)
            || is_key_pressed(KeyCode::RightBracket)
            || self.input.gamepad.snapshot.nav_right
        {
            self.audio.play_sfx(SfxType::UiMove);
            module_filter = module_filter.next();
            active_tab = if module_filter == ModuleFilter::Drafts {
                TrackManagerTab::Drafts
            } else if active_tab == TrackManagerTab::Drafts {
                TrackManagerTab::Main
            } else {
                active_tab
            };
            selected_idx = 0;
        }

        // Direct module/category selection via number keys 1-5, 9
        if is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::Kp1) {
            if active_tab == TrackManagerTab::Drafts || module_filter != ModuleFilter::Classic {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Classic;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key2) || is_key_pressed(KeyCode::Kp2) {
            if active_tab == TrackManagerTab::Drafts || module_filter != ModuleFilter::Rally {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Rally;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key3) || is_key_pressed(KeyCode::Kp3) {
            if active_tab == TrackManagerTab::Drafts || module_filter != ModuleFilter::Kart {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Kart;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key4) || is_key_pressed(KeyCode::Kp4) {
            if active_tab == TrackManagerTab::Drafts || module_filter != ModuleFilter::Gt {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Gt;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key5) || is_key_pressed(KeyCode::Kp5) {
            if active_tab == TrackManagerTab::Drafts || module_filter != ModuleFilter::Nascar {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Nascar;
                selected_idx = 0;
            }
        }
        if is_key_pressed(KeyCode::Key9) || is_key_pressed(KeyCode::Kp9) {
            if active_tab != TrackManagerTab::Drafts || module_filter != ModuleFilter::Drafts {
                self.audio.play_sfx(SfxType::UiMove);
                active_tab = TrackManagerTab::Drafts;
                module_filter = ModuleFilter::Drafts;
                selected_idx = 0;
            }
        }

        // Module cycling shortcuts (M / F)
        if is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::F) {
            self.audio.play_sfx(SfxType::UiMove);
            module_filter = module_filter.next();
            active_tab = if module_filter == ModuleFilter::Drafts {
                TrackManagerTab::Drafts
            } else {
                TrackManagerTab::Main
            };
            selected_idx = 0;
        }

        // Toggle between Approved Modules and Workshop Drafts (Tab key)
        if is_key_pressed(KeyCode::Tab) {
            self.audio.play_sfx(SfxType::UiMove);
            if module_filter == ModuleFilter::Drafts || active_tab == TrackManagerTab::Drafts {
                active_tab = TrackManagerTab::Main;
                module_filter = ModuleFilter::Classic;
            } else {
                active_tab = TrackManagerTab::Drafts;
                module_filter = ModuleFilter::Drafts;
            }
            selected_idx = 0;
        }

        // Re-evaluate list after potential module/tab switch:
        // Re-evaluate list after potential module/tab switch:
        // Returns all circuits for the active module (presets first, then custom), or drafts
        let is_dev = crate::storage::is_dev_mode();
        let is_dev_workbench = is_dev && active_tab == TrackManagerTab::DevWorkbench;

        let current_list = if active_tab == TrackManagerTab::Drafts || module_filter == ModuleFilter::Drafts {
            self.track_manager.draft_track_choices()
        } else {
            self.track_manager.filtered_main_track_choices(module_filter)
        };
        let list_len = current_list.len();

        if list_len > 0 && selected_idx >= list_len {
            selected_idx = list_len.saturating_sub(1);
        }

        // Toggle Dev Workbench via Ctrl+D or F12 (Developer Mode only)
        let ctrl_down = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);

        if is_dev && (is_key_pressed(KeyCode::F12) || (ctrl_down && is_key_pressed(KeyCode::D))) {
            self.audio.play_sfx(SfxType::UiSelect);
            let next_tab = if active_tab == TrackManagerTab::DevWorkbench {
                TrackManagerTab::Main
            } else {
                TrackManagerTab::DevWorkbench
            };
            self.state = GameState::TrackManager {
                active_tab: next_tab,
                module_filter,
                selected_idx: 0,
                modal: TrackManagerModal::None,
            };
            return;
        }

        // 3. Up/Down Track Selection or Dev-Mode Preset Reordering
        let shift_or_alt = is_key_down(KeyCode::LeftShift)
            || is_key_down(KeyCode::RightShift)
            || is_key_down(KeyCode::LeftAlt)
            || is_key_down(KeyCode::RightAlt);

        let want_reorder_up = is_dev
            && ((shift_or_alt && (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)))
                || is_key_pressed(KeyCode::PageUp));

        let want_reorder_down = is_dev
            && ((shift_or_alt && (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)))
                || is_key_pressed(KeyCode::PageDown));

        if want_reorder_up {
            if let Some(track_choice) = current_list.get(selected_idx) {
                if track_choice.is_official_preset() {
                    let tid = track_choice.track_id().to_string();
                    let mod_id = module_filter.id().unwrap_or("classic");
                    if let Ok(moved) = self.track_manager.reorder_preset_track(&tid, mod_id, true) {
                        if moved {
                            self.audio.play_sfx(SfxType::UiSelect);
                            selected_idx = selected_idx.saturating_sub(1);
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    }
                } else {
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
        } else if want_reorder_down {
            if let Some(track_choice) = current_list.get(selected_idx) {
                if track_choice.is_official_preset() {
                    let tid = track_choice.track_id().to_string();
                    let mod_id = module_filter.id().unwrap_or("classic");
                    if let Ok(moved) = self.track_manager.reorder_preset_track(&tid, mod_id, false) {
                        if moved {
                            self.audio.play_sfx(SfxType::UiSelect);
                            selected_idx = (selected_idx + 1).min(list_len.saturating_sub(1));
                        } else {
                            self.audio.play_sfx(SfxType::UiMove);
                        }
                    }
                } else {
                    self.audio.play_sfx(SfxType::UiMove);
                }
            }
        } else {
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
        }

        let is_confirm_pressed = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space)
            || self.input.gamepad.snapshot.btn_confirm_pressed
            || self.input.gamepad.snapshot.btn_a_pressed;

        let is_edit_pressed = is_key_pressed(KeyCode::E) || self.input.gamepad.snapshot.btn_x_pressed;

        // 4. In Dev Workbench, Enter races track
        if is_dev_workbench && is_confirm_pressed {
            if let Some(track_choice) = current_list.get(selected_idx).cloned() {
                self.audio.play_sfx(SfxType::UiSelect);
                self.track_choice = track_choice;
                self.init_race();
                return;
            }
        }

        // 5. Edit in Track Editor (Enter or E)
        if (!is_dev_workbench && (is_confirm_pressed || is_edit_pressed)) || (is_dev_workbench && is_edit_pressed) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                if track_choice.is_official_preset() {
                    if is_dev {
                        // In dev mode, open preset directly in Track Editor
                        let track = self
                            .track_manager
                            .load_track(track_choice)
                            .unwrap_or_else(|_| classic_grand_prix());
                        let file_path = if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
                            let mod_id = TrackManager::preset_module(track_choice.track_id()).unwrap_or("classic");
                            Some(git_tracks_dir.join(mod_id).join(format!("{}.json", track_choice.track_id())).to_string_lossy().to_string())
                        } else {
                            None
                        };
                        self.track_choice = track_choice.clone();
                        self.track = track.clone();
                        self.editor_return_track_manager = Some((active_tab, module_filter, selected_idx));
                        self.enter_track_editor_with_path(track, file_path);
                        return;
                    } else {
                        // Normal player: official presets cannot be edited directly, prompt to clone to Drafts
                        self.state = GameState::TrackManager {
                            active_tab,
                            module_filter,
                            selected_idx,
                            modal: TrackManagerModal::CloneBeforeEdit {
                                track_choice: track_choice.clone(),
                                track_title: track_choice.title().to_string(),
                                is_dev_mode: false,
                                dev_choice: 0,
                            },
                        };
                        return;
                    }
                }

                // If user custom track or draft, open directly in Track Editor
                self.track_choice = track_choice.clone();
                let file_path = match track_choice {
                    TrackChoice::Custom { path, .. } => {
                        let candidate = self.track_manager.track_path_for_slug(track_choice.track_id());
                        if candidate.exists() {
                            Some(candidate.to_string_lossy().to_string())
                        } else if std::path::Path::new(path).exists() {
                            Some(path.clone())
                        } else {
                            Some(candidate.to_string_lossy().to_string())
                        }
                    }
                    _ => None,
                };
                let track = self
                    .track_manager
                    .load_track(track_choice)
                    .unwrap_or_else(|_| classic_grand_prix());
                self.track = track.clone();
                self.editor_return_track_manager = Some((active_tab, module_filter, selected_idx));
                self.enter_track_editor_with_path(track, file_path);
                return;
            }
        }

        // 6. Clone Circuit to Drafts (C key)
        if is_key_pressed(KeyCode::C) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                if let Ok((_cloned_track, file_path)) = self.track_manager.clone_track(track_choice) {
                    self.audio.play_sfx(SfxType::UiSelect);
                    active_tab = TrackManagerTab::Drafts;
                    module_filter = ModuleFilter::Drafts;
                    let file_stem = std::path::Path::new(&file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("cloned_track")
                        .to_string();
                    let drafts = self.track_manager.draft_track_choices();
                    selected_idx = drafts.iter().position(|t| t.track_id() == file_stem).unwrap_or(0);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::None,
                    };
                    return;
                }
            }
        }

        // 7. Promote / Demote Track (P key = Dev Promote/Demote, Ctrl+P = Assign Categories)
        let ctrl_down = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);

        if is_key_pressed(KeyCode::P) || self.input.gamepad.snapshot.btn_y_pressed {
            if let Some(track_choice) = current_list.get(selected_idx) {
                let tid = track_choice.track_id().to_string();
                let is_dev = crate::storage::is_dev_mode();
                let is_preset = track_choice.is_official_preset();

                if ctrl_down {
                    // Ctrl+P: Assign Categories (Modifying categories a circuit belongs to)
                    if is_preset && !is_dev {
                        // Presets are strictly immutable in standard mode!
                        self.audio.play_sfx(SfxType::UiMove);
                    } else {
                        self.audio.play_sfx(SfxType::UiSelect);
                        let mut selected_mask = [false; 4];
                        let mut has_any_selected = false;
                        for (idx, (mod_id, _, _, _)) in PROMOTION_MODULES.iter().enumerate() {
                            if self.track_manager.is_track_in_module(&tid, mod_id) {
                                selected_mask[idx] = true;
                                has_any_selected = true;
                            }
                        }
                        let default_mod_idx = match module_filter.id().unwrap_or(self.active_module_id) {
                            "rally" => 1,
                            "kart" => 2,
                            "gt" => 3,
                            _ => 0,
                        };
                        if !has_any_selected {
                            selected_mask[default_mod_idx] = true;
                        }
                        let cursor_idx = if has_any_selected {
                            selected_mask.iter().position(|&b| b).unwrap_or(default_mod_idx)
                        } else {
                            default_mod_idx
                        };
                        self.state = GameState::TrackManager {
                            active_tab,
                            module_filter,
                            selected_idx,
                            modal: TrackManagerModal::SelectModulePromotion {
                                track_id: tid,
                                track_title: track_choice.title().to_string(),
                                cursor_idx,
                                selected_mask,
                            },
                        };
                        return;
                    }
                } else {
                    // Regular P: Promote / Demote
                    if !is_dev {
                        if is_preset {
                            self.audio.play_sfx(SfxType::UiMove);
                        } else {
                            self.audio.play_sfx(SfxType::UiSelect);
                            let mut selected_mask = [false; 4];
                            let mut has_any_selected = false;
                            for (idx, (mod_id, _, _, _)) in PROMOTION_MODULES.iter().enumerate() {
                                if self.track_manager.is_track_in_module(&tid, mod_id) {
                                    selected_mask[idx] = true;
                                    has_any_selected = true;
                                }
                            }
                            let default_mod_idx = match module_filter.id().unwrap_or(self.active_module_id) {
                                "rally" => 1,
                                "kart" => 2,
                                "gt" => 3,
                                _ => 0,
                            };
                            if !has_any_selected {
                                selected_mask[default_mod_idx] = true;
                            }
                            let cursor_idx = if has_any_selected {
                                selected_mask.iter().position(|&b| b).unwrap_or(default_mod_idx)
                            } else {
                                default_mod_idx
                            };
                            self.state = GameState::TrackManager {
                                active_tab,
                                module_filter,
                                selected_idx,
                                modal: TrackManagerModal::SelectModulePromotion {
                                    track_id: tid,
                                    track_title: track_choice.title().to_string(),
                                    cursor_idx,
                                    selected_mask,
                                },
                            };
                            return;
                        }
                    } else if is_preset {
                        self.audio.play_sfx(SfxType::UiSelect);
                        self.state = GameState::TrackManager {
                            active_tab,
                            module_filter,
                            selected_idx,
                            modal: TrackManagerModal::ConfirmDemoteToCustom {
                                track_id: tid,
                                track_title: track_choice.title().to_string(),
                            },
                        };
                        return;
                    } else {
                        self.audio.play_sfx(SfxType::UiSelect);
                        let target_mod = module_filter.id().unwrap_or(self.active_module_id).to_string();
                        self.state = GameState::TrackManager {
                            active_tab,
                            module_filter,
                            selected_idx,
                            modal: TrackManagerModal::ConfirmPromoteToPreset {
                                track_id: tid,
                                track_title: track_choice.title().to_string(),
                                target_module: target_mod,
                            },
                        };
                        return;
                    }
                }
            }
        }

        // 8. Create New Custom Track / Draft (N key)
        if is_key_pressed(KeyCode::N) {
            self.audio.play_sfx(SfxType::UiSelect);
            if module_filter == ModuleFilter::Drafts || active_tab == TrackManagerTab::Drafts {
                let count = self.track_manager.draft_track_choices().len() + 1;
                let name = format!("Draft Track {}", count);
                let desc = "Work in progress draft circuit.".to_string();
                let _ = self.track_manager.create_new_draft_track(&name, &desc);
                selected_idx = self.track_manager.draft_track_choices().len().saturating_sub(1);
            } else {
                let effective_module = module_filter.id().unwrap_or(self.active_module_id);
                let count = self.track_manager.module_custom_tracks(effective_module).len() + 1;
                let name = format!("Custom Track {}", count);
                let desc = format!("Custom circuit for {} module.", effective_module);
                let _ = self.track_manager.create_new_custom_track_with_template(
                    &name,
                    &desc,
                    effective_module,
                    tdrace_core::track::presets::TrackShape::Oval,
                    tdrace_core::track::presets::RaceDirection::Right,
                );
                selected_idx = self.track_manager.filtered_main_track_choices(module_filter).len().saturating_sub(1);
            }
        }

        // 9. Edit Metadata (I key)
        if is_key_pressed(KeyCode::I) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                if track_choice.is_official_preset() && !crate::storage::is_dev_mode() {
                    self.audio.play_sfx(SfxType::UiMove);
                } else {
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
        }

        // 10. Delete Track (Delete / Backspace key)
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            if let Some(track_choice) = current_list.get(selected_idx) {
                if track_choice.is_official_preset() && !crate::storage::is_dev_mode() {
                    self.audio.play_sfx(SfxType::UiMove);
                } else {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.state = GameState::TrackManager {
                        active_tab,
                        module_filter,
                        selected_idx,
                        modal: TrackManagerModal::ConfirmDelete {
                            track_id: track_choice.track_id().to_string(),
                            track_title: track_choice.title().to_string(),
                            cursor_idx: 0,
                        },
                    };
                    return;
                }
            }
        }

        // 11. Back to Main Menu (Escape / Gamepad Back)
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
        let is_split = self.is_split_screen();

        if self.player_collision_stunt_lockout > 0.0 {
            self.player_collision_stunt_lockout = (self.player_collision_stunt_lockout - dt).max(0.0);
        }
        if self.player2_collision_stunt_lockout > 0.0 {
            self.player2_collision_stunt_lockout = (self.player2_collision_stunt_lockout - dt).max(0.0);
        }

        // 1. Gather driver controls (Player keyboard with smoothing + Touch combined, and AI bots)
        let mut controls_all = Vec::with_capacity(n_cars);
        if is_split {
            let p1_speed = self.cars.first().map(|c| c.state.local_velocity.x).unwrap_or(0.0);
            let p2_speed = self.cars.get(1).map(|c| c.state.local_velocity.x).unwrap_or(0.0);
            let (mut p1_ctrl, mut p2_ctrl) = self.input.poll_split_player_controls(
                &mut self.filter_p2,
                dt,
                p1_speed,
                p2_speed,
            );
            if p1_speed <= 0.25 && p1_ctrl.brake > 0.0 && p1_ctrl.throttle == 0.0 {
                p1_ctrl.reverse = true;
                p1_ctrl.throttle = p1_ctrl.brake;
                p1_ctrl.brake = 0.0;
            }
            if p2_speed <= 0.25 && p2_ctrl.brake > 0.0 && p2_ctrl.throttle == 0.0 {
                p2_ctrl.reverse = true;
                p2_ctrl.throttle = p2_ctrl.brake;
                p2_ctrl.brake = 0.0;
            }
            controls_all.push(p1_ctrl);
            controls_all.push(p2_ctrl);

            for i in 2..n_cars {
                let ai_idx = i - 2;
                let other_cars_refs: Vec<&Car> = self
                    .cars
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != i)
                    .map(|(_, c)| c)
                    .collect();

                let bot_ctrl = if let Some(ai) = self.ai_drivers.get_mut(ai_idx) {
                    ai.compute_controls(
                        &self.cars[i],
                        &self.track,
                        &other_cars_refs,
                        dt,
                    )
                } else {
                    CarControls::default()
                };
                controls_all.push(bot_ctrl);
            }
        } else {
            let player_speed = self.cars.first().map(|c| c.state.local_velocity.x).unwrap_or(0.0);
            let kb_ctrl = self.input.poll_player_controls(dt, player_speed);
            let touch_ctrl = self.touch.poll_controls();
            let mut player_ctrl = InputController::combine_controls(kb_ctrl, touch_ctrl);
            if player_speed <= 0.25 && player_ctrl.brake > 0.0 && player_ctrl.throttle == 0.0 {
                player_ctrl.reverse = true;
                player_ctrl.throttle = player_ctrl.brake;
                player_ctrl.brake = 0.0;
            }
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

                let bot_ctrl = if let Some(ai) = self.ai_drivers.get_mut(ai_idx) {
                    ai.compute_controls(
                        &self.cars[i],
                        &self.track,
                        &other_cars_refs,
                        dt,
                    )
                } else {
                    CarControls::default()
                };
                controls_all.push(bot_ctrl);
            }
        }

        // 2. Sample surfaces under all wheels of all cars
        let mut wheel_surfaces = Vec::with_capacity(n_cars);
        for (i, car) in self.cars.iter().enumerate() {
            let prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            wheel_surfaces.push(self.track.sample_car_surfaces_with_hint(car, prog));
        }

        // 2b. Compute aerodynamic slipstream wake drafting between cars
        let mut drafts = Vec::with_capacity(n_cars);
        for i in 0..n_cars {
            let other_refs: Vec<&Car> = self
                .cars
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != i)
                .map(|(_, c)| c)
                .collect();
            drafts.push(self.cars[i].compute_draft_intensity(&other_refs));
        }
        for i in 0..n_cars {
            self.cars[i].state.draft_intensity = drafts[i];
        }

        // 3. Step individual vehicle dynamics and update road elevation & cross-slope banking
        for i in 0..n_cars {
            let prev_prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            let proj = self.track.spline.project_point_continuity(self.cars[i].state.position, prev_prog, 50.0);
            self.cars[i].state.road_elevation = proj.elevation;
            self.cars[i].state.road_bank_angle = proj.bank_angle;
            self.cars[i].state.road_grade_slope = proj.grade_slope;
            self.cars[i].state.road_vertical_curvature = proj.vertical_curvature;
            self.cars[i].state.track_right = Vec2::new(proj.tangent.y, -proj.tangent.x);
            self.cars[i].state.track_forward = proj.tangent;

            self.cars[i].step_per_wheel(&controls_all[i], wheel_surfaces[i], dt);

            // Soft tree canopy brush interaction: viscous foliage drag & leaf roost particles
            if !self.cars[i].state.is_airborne && self.cars[i].state.elevation < 0.6 {
                for tree in &self.track.geometry.trees {
                    let car_pos = self.cars[i].state.position;
                    if tree.contains_canopy(car_pos) && !tree.contains_trunk(car_pos) {
                        let drag_rate = tree.tree_type.canopy_drag_deceleration();
                        self.cars[i].state.velocity *= (1.0 - drag_rate * dt).max(0.0);
                        self.cars[i].state.speed = self.cars[i].state.velocity.length();

                        if self.cars[i].state.speed > 3.0 {
                            self.fx.particles.emit_foliage_roost(
                                car_pos,
                                tree.tree_type,
                                self.cars[i].state.velocity,
                            );
                        }
                    }
                }
            }
        }

        // Track human player top speed
        if let Some(player_car) = self.cars.first() {
            self.player_race_stats.top_speed_mps = self.player_race_stats.top_speed_mps.max(player_car.state.speed);
        }

        // Continuous Jump Ramp Traversal, Lip Takeoff & Landing SFX/FX
        let mut player_jump_air_time = None;
        for (i, car) in self.cars.iter_mut().enumerate() {
            let was_airborne = car.state.is_airborne;
            let mut on_any_ramp = false;

            if !was_airborne {
                for ramp in &self.track.geometry.jump_ramps {
                    if ramp.contains(car.state.position) {
                        on_any_ramp = true;
                        if car.step_ramp_interaction(ramp, dt) {
                            break;
                        }
                    }
                }
                if !on_any_ramp {
                    if car.state.ramp_elevation > 0.10 {
                        // Rolled off an elevated ramp edge without launching at speed
                        car.state.elevation = car.state.ramp_elevation;
                        car.state.is_airborne = true;
                        car.state.vertical_velocity = 0.0;
                    }
                    car.state.ramp_elevation = 0.0;
                }
            }
            if car.state.just_landed {
                if i == 0 {
                    self.audio.play_sfx(SfxType::Landing);
                    self.camera.add_trauma(0.25);
                    if car.state.last_air_time >= 0.20 {
                        player_jump_air_time = Some(car.state.last_air_time);
                    }
                } else if i == 1 && is_split {
                    self.audio.play_sfx(SfxType::Landing);
                    self.camera_p2.add_trauma(0.25);
                }
                let surf = wheel_surfaces.get(i).map(|s| s[0]).unwrap_or(SurfaceType::Asphalt);
                self.fx.particles.emit_landing_dust(car.state.position, car.state.speed, surf);
            }
        }

        if let Some(air_time) = player_jump_air_time {
            if self.is_stunt_scoring_enabled() && self.player_collision_stunt_lockout <= 0.0 {
                let pts = (air_time * 250.0).round() as u32;
                self.player_race_stats.stunt_stats.jump_count += 1;
                self.player_race_stats.stunt_stats.total_air_time += air_time;
                self.player_race_stats.stunt_stats.longest_jump_time = self.player_race_stats.stunt_stats.longest_jump_time.max(air_time);
                self.player_race_stats.stunt_stats.jump_points += pts;
                self.player_race_stats.stunt_stats.total_stunt_score += pts;

                if let Some(player_car) = self.cars.first() {
                    let sw = screen_width_safe();
                    let sh = screen_height_safe();
                    let screen_pos = self.camera.world_to_screen_with_viewport(player_car.state.position, sw, sh);
                    let anchor = Vec2::new(
                        screen_pos.x.clamp(100.0, sw - 100.0),
                        (screen_pos.y - 45.0).clamp(70.0, sh - 70.0),
                    );

                    self.drift_combo_count += 1;
                    self.drift_combo_timer = 4.0;
                    self.player_race_stats.stunt_stats.max_combo = self.player_race_stats.stunt_stats.max_combo.max(self.drift_combo_count);

                    if air_time >= 1.50 {
                        self.floating_text.spawn_alert(
                            format!("MEGA JUMP! {:.2}s (+{} PTS)", air_time, pts),
                            anchor,
                            Palette::NEON_GOLD,
                        );
                    } else {
                        self.floating_text.spawn_alert(
                            format!("AIR TIME {:.2}s (+{} PTS)", air_time, pts),
                            anchor,
                            Palette::NEON_CYAN,
                        );
                    }

                    if self.drift_combo_count >= 2 {
                        self.floating_text.spawn_combo(
                            self.drift_combo_count,
                            anchor + Vec2::new(0.0, -26.0),
                        );
                    }
                }
            }
        }

        // Water splash sound effect on player cars (P1 and P2 in split screen)
        let player_count = if is_split { 2.min(self.cars.len()) } else { 1.min(self.cars.len()) };
        for p_idx in 0..player_count {
            if let Some(surfaces) = wheel_surfaces.get(p_idx) {
                let in_water = surfaces.iter().any(|&s| s == SurfaceType::Water);
                if in_water {
                    let car = &self.cars[p_idx];
                    if !car.state.is_airborne
                        && car.state.elevation <= 0.0
                        && car.state.speed > 3.0
                        && self.session_time.fract() < dt * 4.0
                    {
                        let gain = (car.state.speed / 18.0).clamp(0.35, 0.85);
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

        // 5. Resolve Wall and Obstacle boundary collisions for each car (including grandstands & tree trunks)
        let scenery_obstacles = self.track.geometry.all_obstacles_with_scenery();
        let demolition_mode = self.is_demolition_scoring_enabled();
        let mut wall_collision_events = Vec::new();
        for (car_idx, car) in self.cars.iter_mut().enumerate() {
            let mut wall_events = resolve_all_wall_collisions(
                car,
                &self.track.geometry.inner_walls,
                &scenery_obstacles,
            );
            let outer_events =
                resolve_all_wall_collisions(car, &self.track.geometry.outer_walls, &[]);
            wall_events.extend(outer_events);

            for wev in &wall_events {
                if wev.impact_speed > 2.0 && !demolition_mode {
                    car.state.drift_score = 0.0;
                    car.state.is_drifting = false;
                    if car_idx == 0 {
                        self.player_collision_stunt_lockout = 1.2;
                        if self.drift_combo_count > 0 || self.prev_player_drifting {
                            if self.drift_combo_count >= 2 || self.prev_player_drifting {
                                let sw = screen_width_safe();
                                let sh = screen_height_safe();
                                let screen_pos = self.camera.world_to_screen_with_viewport(car.state.position, sw, sh);
                                let anchor = Vec2::new(
                                    screen_pos.x.clamp(100.0, sw - 100.0),
                                    (screen_pos.y - 45.0).clamp(70.0, sh - 70.0),
                                );
                                let alert_msg = if self.drift_combo_count >= 2 { "COMBO BROKEN!" } else { "DRIFT VOIDED!" };
                                self.floating_text.spawn_alert(alert_msg, anchor, Palette::RED);
                            }
                            self.drift_combo_count = 0;
                            self.drift_combo_timer = 0.0;
                            self.prev_player_drifting = false;
                        }
                    } else if car_idx == 1 && is_split {
                        self.player2_collision_stunt_lockout = 1.2;
                    }
                }
                if wev.impact_speed > 3.0 {
                    if car_idx == 0 {
                        self.camera.add_trauma(wev.impact_speed * 0.08);
                    } else if car_idx == 1 && is_split {
                        self.camera_p2.add_trauma(wev.impact_speed * 0.08);
                    }
                }
                if wev.impact_speed > 2.2 {
                    if car_idx == 0 {
                        self.player_race_stats.collision_count = self.player_race_stats.collision_count.saturating_add(1);
                    }
                    let gain = (wev.impact_speed / 16.0).clamp(0.3, 0.9);
                    self.audio.play_sfx_with_gain(SfxType::WallCrash, gain);
                }
            }
            wall_collision_events.extend(wall_events);
        }

        for cev in &car_collision_events {
            if cev.closing_speed > 2.0 && !demolition_mode {
                if cev.car_a_idx < self.cars.len() {
                    self.cars[cev.car_a_idx].state.drift_score = 0.0;
                    self.cars[cev.car_a_idx].state.is_drifting = false;
                }
                if cev.car_b_idx < self.cars.len() {
                    self.cars[cev.car_b_idx].state.drift_score = 0.0;
                    self.cars[cev.car_b_idx].state.is_drifting = false;
                }
                if cev.car_a_idx == 0 || cev.car_b_idx == 0 {
                    self.player_collision_stunt_lockout = 1.2;
                    if self.drift_combo_count > 0 || self.prev_player_drifting {
                        if self.drift_combo_count >= 2 || self.prev_player_drifting {
                            let sw = screen_width_safe();
                            let sh = screen_height_safe();
                            let pos = self.cars.first().map(|c| c.state.position).unwrap_or(Vec2::ZERO);
                            let screen_pos = self.camera.world_to_screen_with_viewport(pos, sw, sh);
                            let anchor = Vec2::new(
                                screen_pos.x.clamp(100.0, sw - 100.0),
                                (screen_pos.y - 45.0).clamp(70.0, sh - 70.0),
                            );
                            let alert_msg = if self.drift_combo_count >= 2 { "COMBO BROKEN!" } else { "DRIFT VOIDED!" };
                            self.floating_text.spawn_alert(alert_msg, anchor, Palette::RED);
                        }
                        self.drift_combo_count = 0;
                        self.drift_combo_timer = 0.0;
                        self.prev_player_drifting = false;
                    }
                }
                if is_split && (cev.car_a_idx == 1 || cev.car_b_idx == 1) {
                    self.player2_collision_stunt_lockout = 1.2;
                }
            }
            if cev.car_a_idx == 0 || cev.car_b_idx == 0 {
                if cev.closing_speed > 3.0 {
                    self.camera.add_trauma(cev.closing_speed * 0.06);
                }
                if cev.closing_speed > 2.0 {
                    self.player_race_stats.collision_count = self.player_race_stats.collision_count.saturating_add(1);
                    let gain = (cev.closing_speed / 14.0).clamp(0.25, 0.85);
                    self.audio.play_sfx_with_gain(SfxType::CarHit, gain);
                }
            }
            if is_split && (cev.car_a_idx == 1 || cev.car_b_idx == 1) {
                if cev.closing_speed > 3.0 {
                    self.camera_p2.add_trauma(cev.closing_speed * 0.06);
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

            let p1_ctrl = controls_all[0];
            let forward_speed = player_car.state.local_velocity.x;
            let effective_throttle = if p1_ctrl.reverse {
                -p1_ctrl.throttle
            } else {
                p1_ctrl.throttle - p1_ctrl.brake
            };
            let (rpm, is_shift) = self.engine_rpm.update(forward_speed, effective_throttle, slip_intensity, dt);
            self.audio.update_engine_telemetry(rpm, effective_throttle, is_shift, forward_speed, self.engine_rpm.current_gear, dt);
        }

        if is_split && self.cars.len() >= 2 {
            let p2_car = &self.cars[1];
            let max_slip_angle = p2_car.state.wheels.iter().map(|w| w.slip_angle.abs()).fold(0.0f32, f32::max);
            let max_slip_ratio = p2_car.state.wheels.iter().map(|w| w.slip_ratio.abs()).fold(0.0f32, f32::max);
            let slip_intensity = (max_slip_angle * 1.5).max(max_slip_ratio);

            let p2_ctrl = controls_all[1];
            let forward_speed = p2_car.state.local_velocity.x;
            let effective_throttle = if p2_ctrl.reverse {
                -p2_ctrl.throttle
            } else {
                p2_ctrl.throttle - p2_ctrl.brake
            };
            let (rpm2, is_shift2) = self.engine_rpm_p2.update(forward_speed, effective_throttle, slip_intensity, dt);
            self.audio.update_engine_telemetry_p2(rpm2, effective_throttle, is_shift2, forward_speed, self.engine_rpm_p2.current_gear, dt);
        } else {
            self.audio.stop_player2_engine();
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

                let completed_sector = self.prev_player_sector;
                let sector_time = if lap_changed {
                    tracker.last_lap_sector_times.get(completed_sector).copied().unwrap_or(0.0)
                } else {
                    tracker.sector_times.get(completed_sector).copied().unwrap_or(0.0)
                };

                if sector_time > 0.05 {
                    let sw = screen_width_safe();
                    let sh = screen_height_safe();
                    let popup_pos = Vec2::new(sw * 0.5, sh * 0.20);

                    let num_sectors = tracker.sector_times.len();
                    if self.prev_best_sectors.len() < num_sectors {
                        self.prev_best_sectors.resize(num_sectors, None);
                    }

                    if let Some(prev_best) = self.prev_best_sectors.get(completed_sector).copied().flatten() {
                        let delta = sector_time - prev_best;
                        if delta < -0.005 {
                            // Purple / personal best sector
                            let delta_str = format!("-{:.2}s", -delta);
                            let text = format!("{} SECTOR {}", delta_str, completed_sector + 1);
                            self.floating_text.spawn_alert(text, popup_pos, Palette::NEON_MAGENTA);
                            self.prev_best_sectors[completed_sector] = Some(sector_time);
                        } else {
                            // Slower sector
                            let delta_str = format!("+{:.2}s", delta);
                            let text = format!("{} SECTOR {}", delta_str, completed_sector + 1);
                            self.floating_text.spawn_alert(text, popup_pos, Palette::YELLOW);
                        }
                    } else {
                        // Benchmark sector on first flying lap
                        let text = format!("SECTOR {}: {:.2}s", completed_sector + 1, sector_time);
                        self.floating_text.spawn_alert(text, popup_pos, Palette::NEON_CYAN);
                        self.prev_best_sectors[completed_sector] = Some(sector_time);
                    }
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
                    let completed_lap = tracker.current_lap.saturating_sub(1).max(1);
                    let sec_times = tracker.last_lap_sector_times.clone();
                    let is_fastest_so_far = self.player_race_stats.laps.iter().all(|l| last_lap_time <= l.lap_time);

                    self.player_race_stats.laps.push(LapTelemetry {
                        lap_number: completed_lap,
                        lap_time: last_lap_time,
                        sector_times: sec_times,
                        is_personal_best: is_fastest_so_far,
                    });

                    let mut min_t = f32::MAX;
                    let mut best_idx = None;
                    for (idx, l) in self.player_race_stats.laps.iter().enumerate() {
                        if l.lap_time < min_t {
                            min_t = l.lap_time;
                            best_idx = Some(idx);
                        }
                    }
                    self.player_race_stats.best_lap_idx = best_idx;

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

                            let sw = screen_width_safe();
                            let sh = screen_height_safe();
                            self.floating_text.spawn_alert(
                                &popup_msg,
                                Vec2::new(sw * 0.5, sh * 0.16),
                                Palette::NEON_GOLD,
                            );
                        }
                    }
                }
                self.prev_player_lap = tracker.current_lap;
            }
        }

        // Lap and sector split audio feedback for Player 2
        if is_split && self.trackers.len() >= 2 {
            let p2_tracker = &self.trackers[1];
            let p2_lap_changed = p2_tracker.current_lap > self.prev_p2_lap;
            if p2_tracker.current_sector != self.prev_p2_sector {
                if p2_tracker.current_sector > 0 {
                    self.audio.play_sfx(SfxType::SectorPing);
                }
                self.prev_p2_sector = p2_tracker.current_sector;
            }
            if p2_lap_changed && p2_tracker.current_lap <= self.total_laps {
                self.audio.play_sfx(SfxType::LapChime);
            }
            if p2_lap_changed {
                self.prev_p2_lap = p2_tracker.current_lap;
            }
        }

        // 8. Replay frame recording
        if let Some(rec) = &mut self.replay_recorder {
            if let (Some(player_car), Some(tracker)) = (self.cars.first(), self.trackers.first()) {
                rec.record_frame(controls_all[0], player_car, tracker);
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

        // 9b. Drift Combo & HUD Floating Popups
        let stunt_scoring_enabled = self.is_stunt_scoring_enabled();
        if let Some(player_car) = self.cars.first_mut() {
            let was_drifting = self.prev_player_drifting;
            let is_drifting = player_car.state.is_drifting;

            if self.player_collision_stunt_lockout > 0.0 {
                // In collision recovery lockout: cancel any post-impact slide from scoring as a stunt
                player_car.state.drift_score = 0.0;
                player_car.state.is_drifting = false;
                self.prev_player_drifting = false;
            } else {
                if stunt_scoring_enabled && was_drifting && !is_drifting && player_car.state.drift_score > 50.0 {
                    self.drift_combo_count += 1;
                    self.drift_combo_timer = 4.0;
                    let pts = player_car.state.drift_score.round() as u32;

                    self.player_race_stats.stunt_stats.drift_count += 1;
                    self.player_race_stats.stunt_stats.total_drift_points += pts;
                    self.player_race_stats.stunt_stats.max_single_drift_score = self.player_race_stats.stunt_stats.max_single_drift_score.max(player_car.state.drift_score);
                    self.player_race_stats.stunt_stats.total_stunt_score += pts;
                    self.player_race_stats.stunt_stats.max_combo = self.player_race_stats.stunt_stats.max_combo.max(self.drift_combo_count);

                    let sw = screen_width_safe();
                    let sh = screen_height_safe();
                    let screen_pos = self.camera.world_to_screen_with_viewport(player_car.state.position, sw, sh);
                    let anchor = Vec2::new(
                        screen_pos.x.clamp(100.0, sw - 100.0),
                        (screen_pos.y - 45.0).clamp(70.0, sh - 70.0),
                    );

                    self.floating_text.spawn_score(pts, anchor);

                    if self.drift_combo_count >= 2 {
                        self.floating_text.spawn_combo(
                            self.drift_combo_count,
                            anchor + Vec2::new(0.0, -26.0),
                        );
                    }
                }
                if was_drifting && !is_drifting {
                    // Reset single-maneuver drift score upon ending drift so it doesn't leak into subsequent maneuvers
                    player_car.state.drift_score = 0.0;
                }
                self.prev_player_drifting = is_drifting;
            }
        }

        // Ensure non-player cars also clear drift_score when not drifting
        for car in self.cars.iter_mut().skip(1) {
            if !car.state.is_drifting && car.state.drift_score > 0.0 {
                car.state.drift_score = 0.0;
            }
        }

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
        let player_done = if self.is_split_screen() {
            let p1_done = self.trackers.first().is_some_and(|t| t.current_lap > self.total_laps);
            let p2_done = self.trackers.get(1).is_some_and(|t| t.current_lap > self.total_laps);
            p1_done || p2_done
        } else {
            self.trackers
                .first()
                .is_some_and(|t| t.current_lap > self.total_laps)
        };

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
            let player_car_title = self
                .selected_car_model_id
                .and_then(crate::catalog::find_model_by_id)
                .map(|m| m.name.to_string())
                .unwrap_or_else(|| active_player_car.title().to_string());
            if let Some(pid) = self.active_profile.id {
                let history_record = RaceHistoryEntry {
                    id: None,
                    profile_id: pid,
                    track_id: track_id.clone(),
                    car_name: player_car_title.clone(),
                    position: player_pos,
                    total_cars: self.cars.len(),
                    total_time: player_time,
                    best_lap: player_best_lap,
                    laps: self.total_laps,
                    is_time_attack: self.is_time_attack,
                    created_at: String::new(),
                    category: self.active_module_id.to_string(),
                    championship_name: self.championship_session.as_ref().map(|c| c.name.clone()),
                    stunt_score: self.player_race_stats.stunt_stats.total_stunt_score,
                    collisions: self.player_race_stats.collision_count,
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
                    let is_p1 = car_idx == 0;
                    let is_p2 = self.is_split_screen() && car_idx == 1;
                    let (driver_name, vehicle_name) = if is_p1 {
                        if self.is_split_screen() {
                            (format!("{} (P1)", self.active_profile.alias), player_car_title.clone())
                        } else {
                            (self.active_profile.alias.clone(), player_car_title.clone())
                        }
                    } else if is_p2 {
                        ("Player 2 (P2)".to_string(), player_car_title.clone())
                    } else {
                        let bot_offset = if self.is_split_screen() { 2 } else { 1 };
                        if let Some(character) = self.opponent_drivers.get(car_idx.saturating_sub(bot_offset)) {
                            let bot_car = if let Some(fav) = character.favorite_cars.iter().find(|f| f.discipline == self.active_module_id) {
                                crate::catalog::find_model_by_id(fav.model_id).map(|m| m.name.to_string()).unwrap_or_else(|| character.preferred_car.title().to_string())
                            } else if self.free_car_selection {
                                character.preferred_car.title().to_string()
                            } else {
                                player_car_title.clone()
                            };
                            (character.alias.to_string(), bot_car)
                        } else {
                            (format!("Driver {}", car_idx), player_car_title.clone())
                        }
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
                        if is_p1 {
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

            // 7. Career XP Award (GT World Challenge / Career mode)
            if self.active_module_id == "gt" || self.game_mode == GameMode::Career {
                // Metric distance-based XP: track length / 10, rounded to 10
                let track_len_m = self.track.spline.total_length().max(100.0);
                let per_lap_xp = ModuleCareerProgress::round_to_10((track_len_m / 10.0) as u64);
                let completed_laps = self.total_laps;
                let lap_xp = per_lap_xp * (completed_laps as u64);
                // Completing a race gives an extra bonus duplicating lap points
                let completion_bonus = lap_xp;

                // First-time circuit bonus: 250 XP x tier (Tier 1: 250, Tier 2: 500, Tier 3: 750, etc.)
                let is_first_time = !self.active_career_progress.visited_tracks.iter().any(|t| t == &track_id);
                let first_time_bonus = if is_first_time {
                    self.active_career_progress.visited_tracks.push(track_id.clone());
                    ModuleCareerProgress::first_time_circuit_bonus(self.active_career_progress.level)
                } else {
                    0
                };

                let total_xp = lap_xp + completion_bonus + first_time_bonus;
                self.active_career_progress.add_xp(total_xp);

                let receipt = XpAwardReceipt {
                    per_lap_xp,
                    completed_laps,
                    lap_xp,
                    completion_bonus,
                    first_time_bonus,
                    total_xp,
                    new_balance: self.active_career_progress.xp,
                    is_first_time,
                };
                self.last_xp_receipt = Some(receipt);

                if let Some(db) = &self.hof_db {
                    let _ = db.save_module_progress(&self.active_career_progress);
                }

                if first_time_bonus > 0 {
                    self.spawn_hud_alert(
                        format!(
                            "+{} XP (LAPS: {}, FINISH: {}, 1ST VISIT: +{}) | WALLET: {} XP",
                            total_xp, lap_xp, completion_bonus, first_time_bonus, self.active_career_progress.xp
                        ),
                        Palette::NEON_GOLD,
                    );
                } else {
                    self.spawn_hud_alert(
                        format!(
                            "+{} XP (LAPS: {}, FINISH: {}) | WALLET: {} XP",
                            total_xp, lap_xp, completion_bonus, self.active_career_progress.xp
                        ),
                        Palette::NEON_CYAN,
                    );
                }
            }

            let bot_offset = if self.is_split_screen() { 2 } else { 1 };
            // If an active championship season is underway, prepare round results with points for the Results screen
            if let Some(champ) = &self.championship_session {
                let mut round_results = Vec::new();
                for (pos, res) in self.results.iter().enumerate() {
                    let (driver_id, driver_name, team_name) = if res.is_player {
                        let name = champ
                            .standings
                            .iter()
                            .find(|s| s.driver_id == "player")
                            .map(|s| s.driver_name.clone())
                            .unwrap_or_else(|| self.active_profile.alias.clone());
                        let team = champ
                            .standings
                            .iter()
                            .find(|s| s.driver_id == "player")
                            .map(|s| s.team_name.clone())
                            .unwrap_or_else(|| "Apex Racing Team".to_string());
                        ("player".to_string(), name, team)
                    } else {
                        let bot_idx = res.car_idx.saturating_sub(bot_offset);
                        if let Some(character) = self.opponent_drivers.get(bot_idx) {
                            let name = champ
                                .standings
                                .iter()
                                .find(|s| s.driver_id == character.id)
                                .map(|s| s.driver_name.clone())
                                .unwrap_or_else(|| character.name.to_string());
                            let team = champ
                                .standings
                                .iter()
                                .find(|s| s.driver_id == character.id)
                                .map(|s| s.team_name.clone())
                                .unwrap_or_else(|| "Motorsport Team".to_string());
                            (character.id.to_string(), name, team)
                        } else {
                            (format!("bot_{}", pos), format!("Driver {}", res.car_idx), "Motorsport Team".to_string())
                        }
                    };
                    round_results.push(RoundDriverResult {
                        driver_id,
                        driver_name,
                        team_name,
                        finish_position: pos + 1,
                        total_time: res.total_time,
                        best_lap: res.best_lap,
                        points_awarded: 0,
                        has_fastest_lap: false,
                    });
                }

                // Award fastest lap bonus to the driver with the actual best lap
                let fastest_lap_time = round_results
                    .iter()
                    .filter_map(|r| r.best_lap)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                if let Some(fastest) = fastest_lap_time {
                    for r in &mut round_results {
                        if let Some(bl) = r.best_lap {
                            if (bl - fastest).abs() < 1e-4 {
                                r.has_fastest_lap = true;
                                break;
                            }
                        }
                    }
                }

                // Calculate points awarded for each driver and populate self.results
                for (pos, res) in round_results.iter_mut().enumerate() {
                    let pts = champ.point_system.points_for_position(res.finish_position, res.has_fastest_lap);
                    res.points_awarded = pts;
                    if let Some(ui_res) = self.results.get_mut(pos) {
                        ui_res.points_awarded = pts;
                    }
                }

                self.pending_championship_results = Some(round_results);
            }

            // Populate fallback telemetry if synthetic test or laps empty
            if self.player_race_stats.laps.is_empty() {
                let best = self.trackers.first().and_then(|t| t.best_lap_time).unwrap_or(self.session_time / self.total_laps.max(1) as f32);
                let sectors = self.trackers.first().map(|t| t.last_lap_sector_times.clone()).unwrap_or_default();
                for lap_idx in 1..=self.total_laps {
                    self.player_race_stats.laps.push(LapTelemetry {
                        lap_number: lap_idx,
                        lap_time: best,
                        sector_times: sectors.clone(),
                        is_personal_best: lap_idx == 1,
                    });
                }
                self.player_race_stats.best_lap_idx = Some(0);
            }

            self.show_hall_of_fame = false;
            self.finished_view = FinishedScreenView::Results;
            self.finished_prev_view = FinishedScreenView::Results;
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
            let is_player = car_idx == 0 || (self.is_split_screen() && car_idx == 1);
            let car_name = if car_idx == 0 {
                if self.is_split_screen() {
                    format!("{} (P1 Keys)", self.active_profile.alias)
                } else {
                    format!("{} (You)", self.active_profile.alias)
                }
            } else if self.is_split_screen() && car_idx == 1 {
                "PLAYER 2 (P2 Gamepad)".to_string()
            } else {
                let bot_offset = if self.is_split_screen() { 2 } else { 1 };
                if let Some(character) = self.opponent_drivers.get(car_idx.saturating_sub(bot_offset)) {
                    if self.championship_session.is_some() {
                        character.name.to_string()
                    } else {
                        character.alias.to_string()
                    }
                } else {
                    format!("Driver {}", car_idx)
                }
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
                car_idx,
                points_awarded: 0,
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
            GameState::ModalitySelect {
                category,
                selected_idx,
                ref modal,
            } => {
                let (mod_title, mod_accent) = match self.active_module_id {
                    "gt" | "gt_challenge" => ("GT WORLD CHALLENGE", Palette::RED),
                    "rally" => ("RALLYCROSS WORLD CUP", Palette::NEON_GOLD),
                    "kart" => ("KARTING WORLD CUP", Palette::NEON_GREEN),
                    "nascar" => ("NASCAR CUP SERIES", Palette::YELLOW),
                    "extreme_offroad" => ("EXTREME OFF-ROAD & STUNT ARENAS", Color::new(1.0, 0.40, 0.05, 1.0)),
                    _ => ("CLASSIC ARCADE MOTORSPORT", Palette::NEON_CYAN),
                };
                let active_tracks = self.track_manager.module_catalog_tracks(self.active_module_id);
                render_modality_select_screen(
                    &self.fonts,
                    mod_title,
                    self.active_module_id,
                    mod_accent,
                    category,
                    selected_idx,
                    modal.as_ref(),
                    &self.active_profile,
                    &self.active_profile_stats,
                    self.is_dev_mode(),
                    &active_tracks,
                );
                if let Some(ref modal) = self.settings_modal {
                    let (sw, sh) = (screen_width_safe(), screen_height_safe());
                    let scaler = UiScaler::new(sw, sh);
                    let theme = CabinetTheme::default();
                    let ctx = CabinetContext {
                        scaler: &scaler,
                        fonts: &self.fonts,
                        theme: &theme,
                        gamepad: &self.input.gamepad.snapshot,
                        dt: 0.0,
                        audio: Some(&self.audio),
                    };
                    modal.draw(&ctx);
                }
            }
            GameState::Garage(_) => {
                let unlocked_tier = if self.is_dev_mode() {
                    5
                } else {
                    let wins = self.active_profile_stats.wins;
                    if wins >= 10 {
                        5
                    } else if wins >= 5 {
                        4
                    } else if wins >= 2 {
                        3
                    } else if wins >= 1 {
                        2
                    } else {
                        1
                    }
                };
                crate::ui::render_garage_screen(
                    &self.fonts,
                    self.active_module_id,
                    self.garage_tier,
                    self.garage_car_idx,
                    self.garage_view_mode,
                    self.garage_turntable_angle,
                    self.garage_brake_heat,
                    self.garage_revving,
                    self.garage_rev_rpm,
                    self.garage_gallery_mode,
                    self.garage_gallery_filter,
                    self.garage_gallery_sel,
                    self.is_dev_mode(),
                    unlocked_tier as u32,
                    Some(&self.active_career_progress),
                );
            }
            GameState::CircuitViewer(_) => {
                self.render_circuit_viewer();
            }
            GameState::Menu => {
                let available_tracks = self.filtered_menu_tracks();
                let filter_counts = self.menu_track_filter_counts();
                let (mod_title, mod_sub, mod_accent) = match self.active_module_id {
                    "gt" | "gt_challenge" => ("GT WORLD CHALLENGE", "FIA GT3 & SRO GT2 World Tour", Palette::RED),
                    "rally" => ("RALLYCROSS WORLD CUP", "World RX & Euro RX Mixed Surface Stages", Palette::NEON_GOLD),
                    "kart" => ("KARTING WORLD CUP", "125cc Direct Steering Shifter Karts", Palette::NEON_GREEN),
                    "nascar" => ("NASCAR CUP SERIES", "850 BHP Pushrod V8 High-Banked Superspeedways", Palette::YELLOW),
                    "extreme_offroad" => ("EXTREME OFF-ROAD & STUNT ARENAS", "Baja Deserts, Ice Lakes, Supercross Triples & Stunt Arenas", Color::new(1.0, 0.40, 0.05, 1.0)),
                    _ => ("TDRACE ARCADE RACING", "Modern Cross-Platform 2D Motorsport Simulation & Visuals", Palette::NEON_GOLD),
                };
                let cp_ref = if self.active_module_id == "gt" {
                    Some(&self.active_career_progress)
                } else {
                    None
                };
                let active_track_id = if self.game_mode == GameMode::Career {
                    self.championship_session.as_ref().and_then(|c| c.current_track_id()).or(Some(self.track_choice.track_id()))
                } else if self.menu_origin == MenuOrigin::StartingGrid {
                    Some(self.track_choice.track_id())
                } else {
                    None
                };
                let is_career = self.game_mode == GameMode::Career;
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
                    self.menu_track_filter,
                    filter_counts,
                    cp_ref,
                    self.is_dev_mode(),
                    active_track_id,
                    is_career,
                );
                if self.show_exit_confirm {
                    if let Some(ref modal) = self.exit_confirm_modal {
                        let sw = screen_width_safe();
                        let sh = screen_height_safe();
                        let scaler = UiScaler::new(sw, sh);
                        let theme = CabinetTheme::cyberpunk_neon();
                        let ctx = CabinetContext::new(&scaler, &self.fonts, &theme, &self.input.gamepad.snapshot, 0.0);
                        modal.draw(&ctx);
                    } else {
                        render_exit_confirm_modal(&self.fonts);
                    }
                }
                if let Some(ref modal) = self.settings_modal {
                    let (sw, sh) = (screen_width_safe(), screen_height_safe());
                    let scaler = UiScaler::new(sw, sh);
                    let theme = CabinetTheme::default();
                    let ctx = CabinetContext {
                        scaler: &scaler,
                        fonts: &self.fonts,
                        theme: &theme,
                        gamepad: &self.input.gamepad.snapshot,
                        dt: 0.0,
                        audio: Some(&self.audio),
                    };
                    modal.draw(&ctx);
                }
            }
            GameState::ModuleSelect { selected_idx } => {
                let modules_data = [
                    ("classic", "Classic Arcade Motorsport", "All-in-one arcade racing, time trials & CAD circuit studio workshop", Palette::NEON_CYAN),
                    ("rally", "Rallycross World Cup", "Mixed-surface sprint heats, jumps & high-sliding dirt circuits", Palette::NEON_GOLD),
                    ("kart", "Karting World Cup", "Direct 1:1 steering, tight chicanes & elimination tournament heats", Palette::NEON_GREEN),
                    ("gt", "GT World Challenge", "High-downforce endurance & sprint racing on world grand prix circuits", Palette::RED),
                    ("nascar", "NASCAR Cup Series & Trans-Am TA1", "High-speed pack drafting, banked tri-ovals & iconic road courses", Color::new(1.0, 0.82, 0.08, 1.0)),
                    ("extreme_offroad", "Extreme Off-Road & Stunt Arenas", "Desert dunes, ice lakes, massive stadium jumps & stunt arenas", Color::new(1.0, 0.40, 0.05, 1.0)),
                ];
                render_module_select_menu(
                    &self.fonts,
                    selected_idx,
                    &modules_data,
                    &self.active_profile,
                    &self.active_profile_stats,
                );
                if self.show_exit_confirm {
                    if let Some(ref modal) = self.exit_confirm_modal {
                        let sw = screen_width_safe();
                        let sh = screen_height_safe();
                        let scaler = UiScaler::new(sw, sh);
                        let theme = CabinetTheme::cyberpunk_neon();
                        let ctx = CabinetContext::new(&scaler, &self.fonts, &theme, &self.input.gamepad.snapshot, 0.0);
                        modal.draw(&ctx);
                    } else {
                        render_exit_confirm_modal(&self.fonts);
                    }
                }
                if let Some(ref modal) = self.settings_modal {
                    let (sw, sh) = (screen_width_safe(), screen_height_safe());
                    let scaler = UiScaler::new(sw, sh);
                    let theme = CabinetTheme::default();
                    let ctx = CabinetContext {
                        scaler: &scaler,
                        fonts: &self.fonts,
                        theme: &theme,
                        gamepad: &self.input.gamepad.snapshot,
                        dt: 0.0,
                        audio: Some(&self.audio),
                    };
                    modal.draw(&ctx);
                }
            }
            GameState::CareerHub {
                selected_tier,
                selected_slot,
                ref calendar_tracks,
                showing_standings,
            } => {
                crate::ui::render_career_hub_screen(
                    &self.fonts,
                    &self.active_profile,
                    &self.active_career_progress,
                    selected_tier,
                    selected_slot,
                    calendar_tracks,
                    self.championship_session.as_ref(),
                    showing_standings,
                    self.selected_car_model_id,
                    self.input.gamepad.snapshot.is_connected,
                    self.career_hub_focus,
                );
            }
            GameState::CareerSelect { selected_idx } => {
                self.render_career_select(selected_idx);
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
                let max_grid_size = self.max_grid_participants();
                let active_car = self.active_player_car_choice();
                let best_lap = self
                    .grid_participants
                    .iter()
                    .find(|p| p.is_player)
                    .and_then(|p| p.best_lap)
                    .or_else(|| self.ghost_recorder.best_ghost_lap.as_ref().map(|g| g.lap_time));
                let req_tier = self.current_race_required_tier();
                let is_unlocked = self.is_active_player_car_unlocked();
                let is_eligible = self.is_active_player_car_eligible(req_tier);
                let unlock_level = self.active_player_car_unlock_level();
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
                    self.starting_grid_focus,
                    self.starting_grid_card_idx,
                    self.starting_grid_roster_idx,
                    is_unlocked,
                    is_eligible,
                    req_tier,
                    unlock_level,
                    self.selected_car_model_id,
                    self.casual_ai_difficulty,
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
                render_pause_menu(&self.fonts, self.assist_profile, &self.audio.settings, self.pause_selected_btn);
                if let Some(ref modal) = self.settings_modal {
                    let (sw, sh) = (screen_width_safe(), screen_height_safe());
                    let scaler = UiScaler::new(sw, sh);
                    let theme = CabinetTheme::default();
                    let ctx = CabinetContext {
                        scaler: &scaler,
                        fonts: &self.fonts,
                        theme: &theme,
                        gamepad: &self.input.gamepad.snapshot,
                        dt: 0.0,
                        audio: Some(&self.audio),
                    };

                    modal.draw(&ctx);
                }
            }
            GameState::Finished => {
                self.render_world();
                let current_view = if self.finished_view == FinishedScreenView::Statistics {
                    FinishedScreenView::Statistics
                } else if self.show_hall_of_fame || self.finished_view == FinishedScreenView::HallOfFame {
                    FinishedScreenView::HallOfFame
                } else {
                    FinishedScreenView::Results
                };

                match current_view {
                    FinishedScreenView::Results => {
                        render_results_screen(&self.fonts, &self.track.name, &self.results, self.is_time_attack, self.championship_session.is_some(), self.last_xp_receipt.as_ref());
                    }
                    FinishedScreenView::HallOfFame => {
                        render_hall_of_fame_screen(
                            &self.fonts,
                            &self.track.name,
                            &self.hof_entries,
                            self.recent_hof_id,
                            self.recent_congrats.as_ref(),
                        );
                    }
                    FinishedScreenView::Statistics => {
                        render_race_stats_screen(
                            &self.fonts,
                            &self.track.name,
                            self.car_choice.title(),
                            &self.player_race_stats,
                            self.session_time,
                            self.finished_prev_view == FinishedScreenView::HallOfFame,
                            self.is_stunt_scoring_enabled(),
                        );
                    }
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
                    &self.input.input_map,
                    self.input.active_preset_name(),
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
                    self.profile_manager_tab,
                    self.profile_telemetry_filter_idx,
                    self.profile_focus_area,
                    &self.championship_manager,
                    self.championship_session.as_ref(),
                    self.profile_champ_scroll,
                    self.profile_champ_selected_idx,
                    self.is_dev_mode(),
                    self.active_career_progress.level,
                    &self.profile_module_progress,
                    &self.profile_awards,
                    self.profile_cabinet_disc_idx,
                    self.profile_cabinet_tier_idx,
                );
            }
            GameState::PlayerRosterManager {
                selected_idx,
                active_column,
                field_idx,
                ref input_name,
                ref input_alias,
                country_idx,
                livery_idx,
                assist_mode,
                cursor_timer,
                ref status_msg,
            } => {
                render_player_roster_manager_screen(
                    &self.fonts,
                    &self.profile_list,
                    selected_idx,
                    active_column,
                    field_idx,
                    input_name,
                    input_alias,
                    country_idx,
                    livery_idx,
                    assist_mode,
                    &self.active_profile_stats,
                    cursor_timer,
                    status_msg.as_deref(),
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
            GameState::ChampionshipEditor => {
                self.render_championship_editor();
            }
        }

        // Render CRT & Retro Scanline post-processing overlay
        if self.crt_overlay.is_active() {
            self.crt_overlay.render(0.0, 0.0, screen_width_safe(), screen_height_safe());
        }

        // Render top-most arcade screen transition overlay if active
        if let Some(ref trans) = self.transition {
            trans.render(0.0, 0.0, screen_width_safe(), screen_height_safe());
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
        self.track = track.clone();
        let mut state = EditorState::new(track);
        state.current_file_path = file_path;
        self.editor_state = Some(state);
        self.editor_tools = ToolSettings::default();
        self.editor_modal = EditorModal::None;
        self.state = GameState::TrackEditor;
    }

    /// Transitions from Race Modality Hub into Track Studio editor, setting origin to ModalitySelect.
    pub fn enter_track_editor_from_modality_select(&mut self) {
        self.audio.play_sfx(SfxType::UiSelect);
        self.editor_origin = EditorOrigin::ModalitySelect;
        let track = self.track.clone();
        let file_path = match &self.track_choice {
            TrackChoice::Custom { path, .. } => {
                let candidate = self.track_manager.track_path_for_slug(self.track_choice.track_id());
                if candidate.exists() {
                    Some(candidate.to_string_lossy().to_string())
                } else if std::path::Path::new(path).exists() {
                    Some(path.clone())
                } else {
                    Some(candidate.to_string_lossy().to_string())
                }
            }
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

    /// Transitions into the Championship Editor studio with an optional initial definition.
    pub fn enter_championship_editor(&mut self, def: Option<ChampionshipDefinition>) {
        self.audio.play_sfx(SfxType::UiSelect);
        let initial_def = def.or_else(|| {
            // Default to the championship preset matching active module and garage tier, or module first, or any sorted preset
            self.championship_manager
                .get_by_module_and_tier(&self.active_module_id, self.garage_tier as u32)
                .cloned()
                .or_else(|| self.championship_manager.get_by_module(&self.active_module_id).first().map(|d| (*d).clone()))
                .or_else(|| self.championship_manager.get("gt4_clubman_sprint").cloned())
                .or_else(|| self.championship_manager.all_sorted().first().map(|d| (*d).clone()))
        });
        self.championship_editor_state = Some(ChampionshipEditorState::new(initial_def));
        self.state = GameState::ChampionshipEditor;
    }

    /// Handles frame update & user interaction in the Championship Editor.
    pub fn update_championship_editor(&mut self, dt: f32) {
        if let Some(mut state) = self.championship_editor_state.take() {
            if let Some((_, ref mut timer)) = state.status_msg {
                *timer -= dt;
                if *timer <= 0.0 {
                    state.status_msg = None;
                }
            }

            let tracks = self.track_manager.main_track_choices();
            let available_champs = self.championship_manager.all_sorted();
            let action = handle_championship_editor_input(&mut state, &tracks, &available_champs, self.is_dev_mode());
            match action {
                ChampionshipEditorAction::None => {
                    self.championship_editor_state = Some(state);
                }
                ChampionshipEditorAction::Exit => {
                    self.audio.play_sfx(SfxType::UiSelect);
                    self.championship_editor_state = None;
                    self.state = GameState::ModalitySelect {
                        category: ModalityCategory::Options,
                        selected_idx: 3,
                        modal: None,
                    };
                }
                ChampionshipEditorAction::LoadChampionship(new_def) => {
                    self.audio.play_sfx(SfxType::UiSelect);
                    let name = new_def.series.name.clone();
                    state.def = new_def;
                    state.selected_round_idx = 0;
                    state.selected_driver_idx = 0;
                    state.set_status(format!("LOADED CUP: {}", name), 3.0);
                    self.championship_editor_state = Some(state);
                }
                ChampionshipEditorAction::NewChampionship => {
                    self.audio.play_sfx(SfxType::UiSelect);
                    let mut blank_def = ChampionshipDefinition::default();
                    blank_def.series.module_id = self.active_module_id.to_string();
                    crate::ui::championship_editor::autofill_grid_for_module(&mut blank_def);
                    state.def = blank_def;
                    state.selected_round_idx = 0;
                    state.selected_driver_idx = 0;
                    state.set_status("CREATED NEW CHAMPIONSHIP", 2.5);
                    self.championship_editor_state = Some(state);
                }
                ChampionshipEditorAction::SaveUser => {
                    match self.championship_manager.save_user_championship(&state.def) {
                        Ok(path) => {
                            self.audio.play_sfx(SfxType::UiSelect);
                            let filename = path
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| "custom.toml".to_string());
                            state.status_msg = Some((format!("SAVED USER CUP: {}", filename), 3.0));
                        }
                        Err(e) => {
                            self.audio.play_sfx(SfxType::UiSelect);
                            state.status_msg = Some((format!("SAVE ERROR: {}", e), 4.0));
                        }
                    }
                    self.championship_editor_state = Some(state);
                }
                ChampionshipEditorAction::SavePreset => {
                    if self.is_dev_mode() {
                        match self.championship_manager.save_preset_championship(&state.def) {
                            Ok(path) => {
                                self.audio.play_sfx(SfxType::UiSelect);
                                let filename = path
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "preset.toml".to_string());
                                state.status_msg = Some((format!("SAVED PRESET: {}", filename), 3.0));
                            }
                            Err(e) => {
                                self.audio.play_sfx(SfxType::UiSelect);
                                state.status_msg = Some((format!("SAVE ERROR: {}", e), 4.0));
                            }
                        }
                    }
                    self.championship_editor_state = Some(state);
                }
                ChampionshipEditorAction::LaunchTestCup(def) => {
                    self.launch_championship_test_cup(def);
                }
            }
        }
    }

    /// Converts a ChampionshipDefinition to a runtime ChampionshipSession, switches module, and starts race.
    pub fn launch_championship_test_cup(&mut self, def: ChampionshipDefinition) {
        self.audio.play_sfx(SfxType::UiSelect);
        let champ = def.to_session();
        self.switch_to_module(&def.series.module_id);
        self.championship_session = Some(champ);
        self.championship_editor_state = None;
        self.init_race();
    }

    /// Launches or resumes an official championship from the Profile Manager.
    /// Sets up the championship session, switches motorsport module, selects the appropriate car,
    /// and initializes the race into Starting Grid.
    pub fn launch_or_resume_championship(&mut self, def: &SeriesDefinition) {
        self.audio.play_sfx(SfxType::UiSelect);

        let is_matching_session = |s: &ChampionshipSession| -> bool {
            !s.is_completed
                && (s.name.eq_ignore_ascii_case(&def.series.name)
                    || s.name.eq_ignore_ascii_case(&def.series.id)
                    || s.name.to_lowercase().contains(&def.series.id.to_lowercase())
                    || def.series.id.to_lowercase().contains(&s.name.to_lowercase())
                    || (s.tier == def.series.tier && !def.series.module_id.is_empty()))
        };

        let existing_session = self.profile_module_progress.get(&def.series.module_id)
            .and_then(|p| p.active_championship.as_ref())
            .filter(|s| is_matching_session(s))
            .cloned()
            .or_else(|| self.active_career_progress.active_championship.as_ref().filter(|s| is_matching_session(s)).cloned())
            .or_else(|| self.championship_session.as_ref().filter(|s| is_matching_session(s)).cloned());

        self.switch_to_module(&def.series.module_id);

        let champ = if let Some(existing) = existing_session {
            existing
        } else {
            let mut c = def.to_session();
            let history_rounds = self.profile_history.iter().filter(|h| {
                h.category == def.series.module_id
                    && (!h.track_id.is_empty() || h.championship_name.as_deref() == Some(&def.series.name))
            }).count();
            if history_rounds > 0 && history_rounds < c.track_ids.len() {
                c.current_round = history_rounds;
            }
            c
        };

        self.championship_session = Some(champ.clone());
        self.active_career_progress.active_championship = Some(champ);
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert(def.series.module_id.clone(), self.active_career_progress.clone());

        self.game_mode = GameMode::Career;

        let prev_selected = self.selected_car_model_id;
        let player_car_model_id = def
            .drivers
            .iter()
            .find(|d| d.is_player)
            .and_then(|d| d.car_model_id.as_deref());

        let tier = def.series.tier as u8;
        let chosen_model = prev_selected
            .and_then(crate::catalog::find_model_by_id)
            .filter(|m| m.module_id == def.series.module_id && m.tier == tier && self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            .or_else(|| {
                player_car_model_id
                    .and_then(crate::catalog::find_model_by_id)
                    .filter(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                let models = crate::catalog::get_models_for_module_and_tier(&def.series.module_id, tier);
                models
                    .into_iter()
                    .find(|m| self.active_career_progress.is_car_unlocked(m.id, self.is_dev_mode()))
            })
            .or_else(|| {
                player_car_model_id.and_then(crate::catalog::find_model_by_id)
            });

        if let Some(m) = chosen_model {
            self.active_career_progress.ensure_car(m.id);
            if let Some(db) = &self.hof_db {
                let _ = db.save_module_progress(&self.active_career_progress);
            }
            self.selected_car_model_id = Some(m.id);
            self.car_choice = m.base_car_choice;
            self.current_visual_type = m.visual_type;
            self.free_car_selection = true;
        }

        if let Some(track_id) = self.championship_session.as_ref().and_then(|c| c.current_track_id()) {
            self.track_choice = self.track_manager.track_choice_for_slug(track_id);
            if let Ok(t) = self.track_manager.load_track_by_slug(track_id) {
                self.track = t;
            }
        }

        self.init_race();
    }

    /// Resets an active or saved championship season so the player can restart it afresh.
    pub fn reset_championship(&mut self, series_name: &str, series_id: &str) {
        self.audio.play_sfx(SfxType::UiSelect);
        let matches = |name: &str| -> bool {
            name.eq_ignore_ascii_case(series_name)
                || name.eq_ignore_ascii_case(series_id)
                || name.to_lowercase().contains(&series_name.to_lowercase())
                || series_name.to_lowercase().contains(&name.to_lowercase())
        };

        if self.championship_session.as_ref().is_some_and(|s| matches(&s.name)) {
            self.championship_session = None;
        }

        if let Some(active) = &self.active_career_progress.active_championship {
            if matches(&active.name) {
                self.active_career_progress.active_championship = None;
            }
        }

        for (_, prog) in self.profile_module_progress.iter_mut() {
            if let Some(active) = &prog.active_championship {
                if matches(&active.name) {
                    prog.active_championship = None;
                }
            }
        }

        if let Some(db) = &self.hof_db {
            if let Some(pid) = self.active_profile.id {
                let _ = db.clear_active_championship_for_profile(pid, series_name, series_id);
            }
            let _ = db.clear_race_history_for_championship(series_name);
            let _ = db.clear_race_history_for_championship(series_id);
        }
        self.refresh_profiles_and_stats();
        self.spawn_hud_alert(format!("{} RESET TO ROUND 1", series_name), Palette::NEON_CYAN);
    }

    /// Renders the multi-career selection screen.
    pub fn render_career_select(&self, selected_idx: usize) {
        let (cards, active_count) = crate::ui::career_select::build_career_select_cards(
            &self.championship_manager,
            &self.profile_module_progress,
            &self.active_career_progress,
            self.championship_session.as_ref(),
            &self.profile_history,
        );
        let profile = &self.active_profile;
        crate::ui::career_select::render_career_select_screen(
            &self.fonts,
            &cards,
            active_count,
            selected_idx,
            profile,
            &self.active_profile_stats,
        );
    }

    /// Updates input and interactions for the multi-career selection screen.
    pub fn update_career_select(&mut self, mut selected_idx: usize) {
        let (cards, _active_count) = crate::ui::career_select::build_career_select_cards(
            &self.championship_manager,
            &self.profile_module_progress,
            &self.active_career_progress,
            self.championship_session.as_ref(),
            &self.profile_history,
        );

        if cards.is_empty() {
            self.state = GameState::CareerSelect { selected_idx: 0 };
            return;
        }

        if selected_idx >= cards.len() {
            selected_idx = cards.len() - 1;
        }

        // Keyboard and Gamepad navigation
        if is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || self.input.gamepad.snapshot.nav_up
            || self.input.gamepad.snapshot.dpad_up_pressed
        {
            if selected_idx > 0 {
                selected_idx -= 1;
                self.audio.play_sfx(SfxType::UiMove);
            }
        }
        if is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::S)
            || self.input.gamepad.snapshot.nav_down
            || self.input.gamepad.snapshot.dpad_down_pressed
        {
            if selected_idx + 1 < cards.len() {
                selected_idx += 1;
                self.audio.play_sfx(SfxType::UiMove);
            }
        }

        // Mouse click navigation (click only to select/expand or launch; hover does not fight keys)
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        let (mx, my) = mouse_position_safe();
        let mouse_vec = macroquad::math::Vec2::new(mx, my);
        let mut clicked_card = false;

        if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            for (i, _) in cards.iter().enumerate() {
                let rect = crate::ui::career_select::career_select_card_rect(i, selected_idx, sw, sh);
                if rect.contains(mouse_vec) {
                    if selected_idx != i {
                        selected_idx = i;
                        self.audio.play_sfx(SfxType::UiMove);
                    } else {
                        clicked_card = true;
                    }
                    break;
                }
            }
        }

        // Back / Cancel
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Backspace)
            || self.input.gamepad.snapshot.btn_cancel_pressed
        {
            self.audio.play_sfx(SfxType::UiSelect);
            self.state = GameState::ModalitySelect {
                category: ModalityCategory::SinglePlayer,
                selected_idx: 0,
                modal: None,
            };
            return;
        }

        // Reset selected season
        if is_key_pressed(KeyCode::R) {
            if let Some(card) = cards.get(selected_idx) {
                if card.is_active {
                    self.reset_championship(&card.series_name, &card.series_id);
                    self.state = GameState::CareerSelect { selected_idx };
                    return;
                }
            }
        }

        // Confirm / Launch / Resume
        let confirm_pressed = clicked_card
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space)
            || self.input.gamepad.snapshot.btn_confirm_pressed;

        if confirm_pressed {
            if let Some(card) = cards.get(selected_idx) {
                self.audio.play_sfx(SfxType::UiSelect);
                let mod_id = card.module_id.clone();
                let tier = card.tier;

                self.switch_to_module(&mod_id);

                match mod_id.as_str() {
                    "gt" | "gt_challenge" => {
                        let calendar = if let Some(c) = &self.active_career_progress.active_championship {
                            c.track_ids.clone()
                        } else {
                            crate::ui::gt_default_calendar(tier)
                        };
                        self.career_hub_focus = CareerHubFocus::Tabs;
                        self.state = GameState::CareerHub {
                            selected_tier: tier,
                            selected_slot: self.active_career_progress.active_championship.as_ref().map(|c| c.current_round).unwrap_or(0),
                            calendar_tracks: calendar,
                            showing_standings: false,
                        };
                    }
                    "nascar" => {
                        self.start_nascar_career_tier(tier);
                    }
                    "rally" => {
                        self.start_rally_career_tier(tier);
                    }
                    "kart" => {
                        self.start_kart_career_tier(tier);
                    }
                    "extreme_offroad" => {
                        self.start_extreme_offroad_career_tier(tier);
                    }
                    _ => {
                        self.start_gt_career_tier_with_calendar(tier, Some(crate::ui::gt_default_calendar(tier)));
                    }
                }
                return;
            }
        }

        self.state = GameState::CareerSelect { selected_idx };
    }

    /// Renders the full-screen Championship Editor studio.
    pub fn render_championship_editor(&self) {
        if let Some(state) = &self.championship_editor_state {
            let sw = screen_width_safe();
            let sh = screen_height_safe();
            let scaler = UiScaler::new(sw, sh);
            let tracks = self.track_manager.main_track_choices();
            let available_champs = self.championship_manager.all_sorted();
            render_championship_editor(
                &self.fonts,
                &scaler,
                state,
                &tracks,
                &available_champs,
                self.is_dev_mode(),
            );
        }
    }

    /// Exits Track Studio editor and navigates cleanly back to the Track Manager screen,
    /// restoring the previous active tab, module filter, and track cursor.
    pub fn return_to_track_manager(&mut self) {
        crate::ui::menu::clear_menu_track_cache();
        let _ = self.track_manager.scan_custom_tracks();

        if self.editor_origin == EditorOrigin::ModalitySelect {
            self.editor_origin = EditorOrigin::TrackManager;
            self.state = GameState::ModalitySelect {
                category: ModalityCategory::Options,
                selected_idx: 2,
                modal: None,
            };
            return;
        }

        let (target_tab, target_filter, fallback_idx) = self
            .editor_return_track_manager
            .take()
            .unwrap_or_else(|| {
                // If not previously recorded, deduce sensible defaults from module customs/drafts
                let has_module_customs = !self.track_manager.module_custom_tracks(self.active_module_id).is_empty();
                let has_drafts = !self.track_manager.draft_track_choices().is_empty();
                if !has_module_customs && has_drafts {
                    (TrackManagerTab::Drafts, ModuleFilter::Drafts, 0)
                } else {
                    (TrackManagerTab::Main, ModuleFilter::for_module(self.active_module_id), 0)
                }
            });

        // Determine list for the target tab
        let current_list = if target_tab == TrackManagerTab::Drafts || target_filter == ModuleFilter::Drafts {
            self.track_manager.draft_track_choices()
        } else {
            self.track_manager.filtered_main_track_choices(target_filter)
        };

        // Try to position cursor on the track that was open/saved in the editor
        let mut selected_idx = fallback_idx;
        let track_slug = if let Some(ref state) = self.editor_state {
            if let Some(ref p) = state.current_file_path {
                std::path::Path::new(p)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                Some(TrackManager::sanitize_slug(&state.track.name))
            }
        } else {
            None
        };

        if let Some(ref slug) = track_slug {
            if let Some(pos) = current_list.iter().position(|t| t.track_id() == slug) {
                selected_idx = pos;
            }
        }

        // Clamp selected_idx within current list bounds
        if current_list.is_empty() {
            selected_idx = 0;
        } else if selected_idx >= current_list.len() {
            selected_idx = current_list.len() - 1;
        }

        self.state = GameState::TrackManager {
            active_tab: target_tab,
            module_filter: target_filter,
            selected_idx,
            modal: TrackManagerModal::None,
        };
    }

    /// Launches a Time Trial race session from the Track Studio editor using the circuit's default car.
    pub fn start_editor_test_drive(&mut self) {
        if let Some(state) = &mut self.editor_state {
            state.rebuild_geometry();
            self.track = state.track.clone();
            let effective_module = self.track.module_id.as_deref().unwrap_or(self.active_module_id);
            let default_car = resolve_predefined_car_for_track(Some(&self.track), effective_module);
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
        let over_ui = is_mouse_over_editor_ui(
            mouse_pos,
            sw,
            sh,
            self.editor_tools.active_tool,
            self.editor_modal != EditorModal::None,
        );

        let is_select_tool = self.editor_tools.active_tool == EditorToolType::Select;

        if !over_ui {
            if is_mouse_button_pressed(macroquad::input::MouseButton::Middle)
                || (is_select_tool && is_mouse_button_pressed(macroquad::input::MouseButton::Right))
            {
                self.editor_camera.start_pan(mouse_pos);
            }
        }
        if self.editor_camera.is_panning {
            if is_mouse_button_down(macroquad::input::MouseButton::Middle)
                || (is_select_tool && is_mouse_button_down(macroquad::input::MouseButton::Right))
            {
                self.editor_camera.update_pan(mouse_pos);
            }
            if is_mouse_button_released(macroquad::input::MouseButton::Middle)
                || (is_select_tool && is_mouse_button_released(macroquad::input::MouseButton::Right))
            {
                self.editor_camera.end_pan();
            }
        }

        let mouse_wheel_y = mouse_wheel_safe().1;
        if !over_ui && mouse_wheel_y.abs() > 0.01 {
            let factor = if mouse_wheel_y > 0.0 { 1.15 } else { 0.85 };
            self.editor_camera.zoom_at(mouse_pos, factor, sw, sh);
        }

        if let Some(state) = &mut self.editor_state {
            let is_multi = is_key_down(KeyCode::LeftShift)
                || is_key_down(KeyCode::RightShift)
                || is_key_down(KeyCode::LeftControl)
                || is_key_down(KeyCode::RightControl)
                || is_key_down(KeyCode::LeftSuper)
                || is_key_down(KeyCode::RightSuper);

            // Primary Button (Left Click) -> Select entities, drag selection, marquee area box select
            if !over_ui && is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                self.editor_tools.handle_primary_down(state, world_mouse, is_multi);
            }
            if self.editor_tools.is_dragging {
                if is_mouse_button_down(macroquad::input::MouseButton::Left) {
                    self.editor_tools.handle_primary_drag(state, world_mouse);
                }
                if is_mouse_button_released(macroquad::input::MouseButton::Left) {
                    self.editor_tools.handle_primary_up(state, world_mouse);
                }
            }

            // Secondary Button (Right Click) -> Place elements (for creation tools)
            if !is_select_tool {
                if !over_ui && is_mouse_button_pressed(macroquad::input::MouseButton::Right) {
                    self.editor_tools.handle_secondary_down(state, world_mouse);
                }
                if self.editor_tools.is_placing {
                    if is_mouse_button_down(macroquad::input::MouseButton::Right) {
                        self.editor_tools.handle_secondary_drag(state, world_mouse);
                    }
                    if is_mouse_button_released(macroquad::input::MouseButton::Right) {
                        self.editor_tools.handle_secondary_up(state, world_mouse);
                    }
                }
            }
        }

        // Shortcuts (bypassed while editing text in an inspector input control)
        if self.editor_tools.is_editing_text() {
            return;
        }

        if is_key_pressed(KeyCode::Key1) { self.editor_tools.active_tool = EditorToolType::Select; }
        if is_key_pressed(KeyCode::Key2) { self.editor_tools.active_tool = EditorToolType::RoadSpline; }
        if is_key_pressed(KeyCode::Key3) { self.editor_tools.active_tool = EditorToolType::SurfaceZone; }
        if is_key_pressed(KeyCode::Key4) { self.editor_tools.active_tool = EditorToolType::JumpRamp; }
        if is_key_pressed(KeyCode::Key5) { self.editor_tools.active_tool = EditorToolType::Obstacle; }
        if is_key_pressed(KeyCode::Key6) { self.editor_tools.active_tool = EditorToolType::Checkpoint; }
        if is_key_pressed(KeyCode::Key7) { self.editor_tools.active_tool = EditorToolType::PitLane; }
        if is_key_pressed(KeyCode::Key8) { self.editor_tools.active_tool = EditorToolType::ArenaFloor; }
        if is_key_pressed(KeyCode::Key9) { self.editor_tools.active_tool = EditorToolType::WhoopSection; }
        if is_key_pressed(KeyCode::Key0) { self.editor_tools.active_tool = EditorToolType::StuntRamp; }

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

        // Drain unconsumed characters when no text modal or inline bar editing is open in the editor
        if !self.editor_tools.is_editing_text() && !matches!(self.editor_modal, EditorModal::SaveAs { .. } | EditorModal::SetRampAngle { .. } | EditorModal::SetRampProperty { .. }) {
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
            EditorAction::ExitToMenu | EditorAction::ExitToTrackManager => {
                self.return_to_track_manager();
            }
            EditorAction::NewTrack { shape, direction, module_id } => {
                let track = tdrace_core::track::presets::create_prototypical_track(&module_id, shape, direction);
                self.enter_track_editor_with_path(track, None);
            }
            EditorAction::NewFromTemplate(preset) => {
                let track = match preset.as_str() {
                    "Oval Speedway" => tdrace_core::track::presets::oval_speedway(),
                    "Oasis Rally" => tdrace_core::track::presets::oasis_rally(),
                    "Classic Grand Prix" => tdrace_core::track::presets::classic_grand_prix(),
                    _ => tdrace_core::track::presets::create_prototypical_track(
                        self.active_module_id,
                        tdrace_core::track::presets::TrackShape::Oval,
                        tdrace_core::track::presets::RaceDirection::Right,
                    ),
                };
                self.enter_track_editor_with_path(track, None);
            }
            EditorAction::OpenTrack(choice) => {
                if let Ok(track) = self.track_manager.load_track(&choice) {
                    let file_path = match &choice {
                        TrackChoice::Custom { path, .. } => {
                            let candidate = self.track_manager.track_path_for_slug(choice.track_id());
                            if candidate.exists() {
                                Some(candidate.to_string_lossy().to_string())
                            } else if std::path::Path::new(path).exists() {
                                Some(path.clone())
                            } else {
                                Some(candidate.to_string_lossy().to_string())
                            }
                        }
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
                let save_outcome = if let Some(state) = &mut self.editor_state {
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

                    let was_existing = if overwrite {
                        if let Some(slug) = target_slug.as_deref() {
                            self.track_manager.is_existing_track(slug) || state.current_file_path.is_some()
                        } else {
                            state.current_file_path.is_some()
                        }
                    } else {
                        false
                    };

                    let track_clone = state.track.clone();
                    Some((target_slug, was_existing, track_clone))
                } else {
                    None
                };

                if let Some((target_slug, was_existing, track_to_save)) = save_outcome {
                    let result = self.track_manager.save_custom_track_with_options(
                        &track_to_save,
                        target_slug.as_deref(),
                        overwrite,
                    );
                    match result {
                        Ok(path) => {
                            let saved_slug = std::path::Path::new(&path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                                .or_else(|| target_slug.clone());

                            if was_existing {
                                if let Some(ref slug) = saved_slug {
                                    self.clear_circuit_history(slug);
                                }
                            }

                            if let Some(state) = &mut self.editor_state {
                                state.current_file_path = Some(path.clone());
                                state.is_dirty = false;
                            }
                            self.editor_save_toast_timer = 2.5;
                            if overwrite {
                                self.editor_save_toast_msg = format!("Track overwritten: {}", path);
                            } else {
                                self.editor_save_toast_msg = format!("Track saved: {}", path);
                            }
                            self.audio.play_sfx(SfxType::UiSelect);

                            // Rescan custom tracks & clear thumbnail cache
                            let _ = self.track_manager.scan_custom_tracks();
                            crate::ui::menu::clear_menu_track_cache();

                            self.track = track_to_save;

                            if exit_after {
                                self.return_to_track_manager();
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
                self.clear_circuit_history(&id);
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
            render_grandstand_shadows_culled(&state.track, None);
            render_tree_shadows_culled(&state.track, None);
            render_grandstands_culled(&state.track, None);
            render_ground_barriers_and_obstacles(&state.track);
            render_tree_trunks_culled(&state.track, None);

            // Render elevated overpass bridges & barriers
            render_elevated_track(&state.track);
            render_elevated_barriers_and_obstacles(&state.track);

            // Render tree canopies above track
            render_tree_canopies_culled(&state.track, &[], None);

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



    /// Renders world-space entities for a specific camera viewport and focus car.
    fn render_world_viewport(
        &self,
        camera: &RaceCamera,
        focus_car_idx: usize,
        viewport: Option<(i32, i32, i32, i32)>,
    ) {
        camera.apply_with_viewport(viewport);
        let view_bounds = Some(camera.visible_world_bounds(12.0));

        // 1. Ground Track & Environment (elevation < 0.6m)
        render_ground_track_culled(&self.track, view_bounds);

        // 2. Persistent Ground Skidmarks
        self.fx.render_ground_fx_culled(view_bounds);

        // 3. Ground Scenery Shadows, Barriers & Obstacles (elevation < 0.6m)
        render_grandstand_shadows_culled(&self.track, view_bounds);
        render_tree_shadows_culled(&self.track, view_bounds);
        render_grandstands_culled(&self.track, view_bounds);
        render_ground_barriers_and_obstacles_culled(&self.track, view_bounds);
        render_tree_trunks_culled(&self.track, view_bounds);

        // Separate cars into ground and elevated groups
        let mut ground_cars = Vec::new();
        let mut elevated_cars = Vec::new();
        for i in 0..self.cars.len() {
            let car = &self.cars[i];
            let is_elevated = car.state.elevation > 0.05
                || car.state.ramp_elevation > 0.05
                || self.track.spline.project_point(car.state.position).is_bridge;
            if is_elevated {
                elevated_cars.push(i);
            } else {
                ground_cars.push(i);
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
        let player_alpha = self.cars.get(focus_car_idx).map(|pc| {
            compute_adaptive_alpha(
                self.visibility_options.adaptive_visibility,
                camera.current_zoom,
                pc.state.speed,
                self.session_time,
            )
        }).unwrap_or(1.0);

        if self.visibility_options.ground_aura && ground_cars.contains(&focus_car_idx) {
            if let Some(focus_car) = self.cars.get(focus_car_idx) {
                let scheme = self.color_schemes.get(focus_car_idx).unwrap_or(&self.active_profile.color_scheme);
                render_player_ground_aura(focus_car.state.position, camera.current_zoom, scheme, player_alpha);
            }
        }
        for &i in &ground_cars {
            let car = &self.cars[i];
            let is_player = !self.is_split_screen() && i == 0 || self.is_split_screen() && i < 2;
            let model_id = self.car_model_ids.get(i).copied().flatten();
            let effective_scheme = if (self.active_module_id == "classic" || self.game_mode == GameMode::Career) && is_player {
                if let Some(m) = model_id.and_then(crate::catalog::find_model_by_id) {
                    CarColorScheme {
                        primary: m.primary_color,
                        secondary: m.secondary_color,
                        helmet: self.color_schemes[i].helmet,
                    }
                } else {
                    self.color_schemes[i]
                }
            } else {
                self.color_schemes[i]
            };
            let is_braking = car.state.is_braking;
            let visual_type = self.car_visual_types.get(i).copied().unwrap_or(self.current_visual_type);
            render_car_with_visual_type_model_and_shadows(
                car,
                &effective_scheme,
                is_braking,
                visual_type,
                model_id,
                self.config.display.vehicle_shadows,
            );
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
        render_elevated_track_culled(&self.track, view_bounds);

        // 7. Elevated Bridge Barriers & Guardrails (drawn on top of the bridge deck, touching the track)
        render_elevated_barriers_and_obstacles_culled(&self.track, view_bounds);

        // 8. Elevated Vehicles (drawn on top of the bridge deck)
        if self.visibility_options.ground_aura && elevated_cars.contains(&focus_car_idx) {
            if let Some(focus_car) = self.cars.get(focus_car_idx) {
                let scheme = self.color_schemes.get(focus_car_idx).unwrap_or(&self.active_profile.color_scheme);
                render_player_ground_aura(focus_car.state.position, camera.current_zoom, scheme, player_alpha);
            }
        }
        for &i in &elevated_cars {
            let car = &self.cars[i];
            let is_player = !self.is_split_screen() && i == 0 || self.is_split_screen() && i < 2;
            let model_id = self.car_model_ids.get(i).copied().flatten();
            let effective_scheme = if (self.active_module_id == "classic" || self.game_mode == GameMode::Career) && is_player {
                if let Some(m) = model_id.and_then(crate::catalog::find_model_by_id) {
                    CarColorScheme {
                        primary: m.primary_color,
                        secondary: m.secondary_color,
                        helmet: self.color_schemes[i].helmet,
                    }
                } else {
                    self.color_schemes[i]
                }
            } else {
                self.color_schemes[i]
            };
            let is_braking = car.state.is_braking;
            let visual_type = self.car_visual_types.get(i).copied().unwrap_or(self.current_visual_type);
            render_car_with_visual_type_model_and_shadows(
                car,
                &effective_scheme,
                is_braking,
                visual_type,
                model_id,
                self.config.display.vehicle_shadows,
            );
        }

        // 9. Airborne Particles (Smoke, Dirt roost, Sparks, Drift text)
        self.fx.render_airborne_fx();

        // 10. Player Car Visibility Aids (Overhead Chevron, Roof Beacon)
        if let Some(focus_car) = self.cars.get(focus_car_idx) {
            let scheme = self.color_schemes.get(focus_car_idx).unwrap_or(&self.active_profile.color_scheme);
            if self.visibility_options.roof_beacon {
                render_player_roof_beacon(
                    focus_car.state.position,
                    focus_car.forward_vector(),
                    focus_car.total_elevation(),
                    camera.current_zoom,
                    self.session_time,
                    scheme,
                    player_alpha,
                );
            }
            if self.visibility_options.overhead_chevron {
                render_player_overhead_chevron(
                    focus_car.state.position,
                    focus_car.total_elevation(),
                    camera.current_zoom,
                    self.session_time,
                    scheme,
                    player_alpha,
                );
            }
            if self.visibility_options.curve_helper {
                if let Some(focus_tracker) = self.trackers.get(focus_car_idx) {
                    let max_lookahead = (focus_car.state.speed * 3.5).clamp(130.0, 220.0);
                    if let Some(status) = self.track.spline.upcoming_curve(
                        focus_tracker.progress_distance,
                        focus_car.state.speed,
                        max_lookahead,
                    ) {
                        render_curve_indicator(
                            focus_car,
                            &status,
                            self.visibility_options.curve_color_scheme,
                            camera.current_zoom,
                            self.session_time,
                        );
                    }
                }
            }
        }

        // 11. Tree Foliage Canopies (Above Vehicles with proximity alpha fading)
        render_tree_canopies_culled(&self.track, &self.cars, view_bounds);

        // 12. Debug Overlays (F1: LIDAR, F2: Checkpoints, F3: OBBs, F4: AI Lines)
        if let Some(focus_car) = self.cars.get(focus_car_idx) {
            if let Some(focus_tracker) = self.trackers.get(focus_car_idx) {
                self.input.render_world_debug(
                    focus_car,
                    &self.cars,
                    &self.track,
                    focus_tracker,
                    &self.ai_drivers,
                );
            }
        }

        camera.reset_to_screen();
    }

    /// Collects candidate nameplate items for a given focus player car.
    fn collect_bot_nameplates<'a>(&'a self, focus_car_idx: usize) -> Vec<VehicleNameplateItem<'a>> {
        let focus_pos = match self.cars.get(focus_car_idx) {
            Some(c) => c.state.position,
            None => return Vec::new(),
        };

        let bot_offset = if self.is_split_screen() { 2 } else { 1 };
        let mut items = Vec::new();

        for (i, car) in self.cars.iter().enumerate() {
            if i == focus_car_idx {
                continue;
            }

            let dist = car.state.position.distance(focus_pos);
            if dist > NAMEPLATE_OUTER_RADIUS {
                continue;
            }

            let scheme = self.color_schemes.get(i).unwrap_or(&self.active_profile.color_scheme);
            let accent_color = scheme.secondary;

            if self.is_split_screen() && i == 0 {
                items.push(VehicleNameplateItem {
                    car_idx: i,
                    name: self.active_profile.alias.as_str(),
                    tier_label: None,
                    accent_color,
                    position: car.state.position,
                    elevation: car.total_elevation(),
                    distance_to_player: dist,
                });
            } else if self.is_split_screen() && i == 1 {
                items.push(VehicleNameplateItem {
                    car_idx: i,
                    name: "PLAYER 2",
                    tier_label: None,
                    accent_color,
                    position: car.state.position,
                    elevation: car.total_elevation(),
                    distance_to_player: dist,
                });
            } else {
                let bot_idx = i.saturating_sub(bot_offset);
                let (name, tier_label) = if let Some(character) = self.opponent_drivers.get(bot_idx) {
                    let name = if self.championship_session.is_some() {
                        character.name
                    } else {
                        character.alias
                    };
                    let tier = self.opponent_tiers.get(bot_idx).map(|t| t.tag());
                    (name, tier)
                } else {
                    ("Opponent", None)
                };

                items.push(VehicleNameplateItem {
                    car_idx: i,
                    name,
                    tier_label,
                    accent_color,
                    position: car.state.position,
                    elevation: car.total_elevation(),
                    distance_to_player: dist,
                });
            }
        }

        items
    }

    /// Renders world-space entities under active camera with strict elevation occlusion layering.
    fn render_world(&self) {
        if self.is_split_screen() && self.cars.len() >= 2 {
            let sw = screen_width_safe();
            let sh = screen_height_safe();
            let [vp1, vp2] = self.split_layout.viewports(sw, sh);
            self.render_world_viewport(&self.camera, 0, Some(vp1));
            self.render_world_viewport(&self.camera_p2, 1, Some(vp2));

            // Floating Bot Nameplates (Spec 029)
            if self.visibility_options.bot_nameplates {
                let [s_rect1, s_rect2] = self.split_layout.screen_rects(sw, sh);
                let p1_nameplates = self.collect_bot_nameplates(0);
                render_floating_bot_nameplates(&self.fonts, &self.camera, Some(s_rect1), &p1_nameplates, 1.0);
                let p2_nameplates = self.collect_bot_nameplates(1);
                render_floating_bot_nameplates(&self.fonts, &self.camera_p2, Some(s_rect2), &p2_nameplates, 1.0);
            }
        } else {
            self.render_world_viewport(&self.camera, 0, None);

            // Floating Bot Nameplates (Spec 029)
            if self.visibility_options.bot_nameplates {
                let nameplates = self.collect_bot_nameplates(0);
                render_floating_bot_nameplates(&self.fonts, &self.camera, None, &nameplates, 1.0);
            }
        }
    }

    /// Renders screen-space HUD, UI overlays, and Mobile Touch Controls.
    fn render_screen(&self, countdown: Option<f32>) {
        let sw = screen_width_safe();
        let sh = screen_height_safe();

        if self.is_split_screen() && self.cars.len() >= 2 {
            let standings = self.compute_standings();
            let p1_pos = standings.iter().position(|&idx| idx == 0).unwrap_or(0) + 1;
            let p2_pos = standings.iter().position(|&idx| idx == 1).unwrap_or(0) + 1;

            render_split_hud(
                &self.fonts,
                &self.track,
                &self.cars,
                &self.color_schemes,
                &self.cars[0],
                &self.trackers[0],
                p1_pos,
                &self.cars[1],
                &self.trackers[1],
                p2_pos,
                self.cars.len(),
                self.total_laps,
                countdown,
                self.input.gamepad.snapshot.is_connected,
                self.split_layout,
            );
        } else if let Some(player_car) = self.cars.first() {
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
                self.visibility_toast.as_ref(),
                &self.visibility_options,
                self.session_time,
            );

            // F5: Telemetry Panel
            self.input.render_screen_debug(player_car);

            // Mobile Touch Controls Overlay (Virtual Joystick / Buttons + Pedals)
            self.touch.render(&self.fonts, sw, sh);
        }

        // Render floating text popups (combos, sector splits, alerts) on HUD overlay
        let scaler = UiScaler::new(sw, sh);
        self.floating_text.draw(&self.fonts, &scaler);
    }

    /// Spawns an animated HUD alert or milestone notification banner.
    pub fn spawn_hud_alert(&mut self, text: impl Into<String>, color: Color) {
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        self.floating_text.spawn_alert(text, Vec2::new(sw * 0.5, sh * 0.20), color);
    }

    /// Spawns a floating sector split time delta popup.
    pub fn spawn_sector_split_popup(&mut self, sector_idx: usize, delta: f32) {
        let sw = screen_width_safe();
        let sh = screen_height_safe();
        let pos = Vec2::new(sw * 0.5, sh * 0.20);
        if delta < -0.005 {
            let text = format!("-{:.2}s SECTOR {}", -delta, sector_idx + 1);
            self.floating_text.spawn_alert(text, pos, Palette::NEON_MAGENTA);
        } else {
            let text = format!("+{:.2}s SECTOR {}", delta, sector_idx + 1);
            self.floating_text.spawn_alert(text, pos, Palette::YELLOW);
        }
    }

    /// Spawns an arcade score popup at screen position.
    pub fn spawn_score_popup(&mut self, score: u32, pos: Vec2) {
        self.floating_text.spawn_score(score, pos);
    }

    /// Spawns an arcade combo notification popup at screen position.
    pub fn spawn_combo_popup(&mut self, combo: u32, pos: Vec2) {
        self.floating_text.spawn_combo(combo, pos);
    }
}
