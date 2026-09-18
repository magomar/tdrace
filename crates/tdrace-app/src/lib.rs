pub mod ai;
pub mod audio;
pub mod camera;
pub mod catalog;
pub mod config;
pub mod db;
pub mod dev_tools;
pub mod editor;
pub mod fx;
pub mod game;
pub mod input;
pub mod module;
pub mod profile;
pub mod render;
pub mod replay;
pub mod storage;
pub mod tournament;
pub mod track_manager;
pub mod tracks;
pub mod ui;

pub use storage::{resolve_user_data_dir, resolve_user_tracks_dir};
pub use tracks::{DevTrackStore, PresetCatalog, UserTrackStore};

pub use ai::{BotAiDriver, BotProfile, DriverCharacter, DriverStats};
pub use audio::{AudioManager, AudioSettings, EngineSoundConfig, EngineSoundType, MusicTrack, SfxType};
pub use camera::{CameraMode, RaceCamera, SplitLayout};
pub use config::{AudioConfig, CameraConfig, GameConfig, GameplayConfig, InputConfig, ZoomLevelConfig};
pub use db::{HallOfFameDb, HallOfFameEntry};
pub use fx::{DriftPopup, EffectsManager, ParticleSystem, SkidmarkBuffer};
pub use game::{
    AcrobaticStats, DriverCardsOrigin, FinishedScreenView, GameState, GridParticipant, LapTelemetry,
    PlayerRaceTelemetry, RaceSession,
};
pub use module::{
    ClassicGameModule, EngineAudioProfile, ExtremeOffRoadModule, F1GameModule, GameModule,
    KartGameModule, ModuleTheme, NascarGameModule, RallyGameModule, TrackDefinition,
    VehicleModelDefinition, VehicleVisualType,
};
pub use tournament::{
    ChampionshipRoundResult, ChampionshipSession, EliminationSession, PointSystem, QualifyingResult,
    QualifyingSession, RallyStageResult, RoundDriverResult, StageRallySession, TournamentFormat,
    TournamentStandingEntry,
};
pub use track_manager::{CustomTrackInfo, TrackManager};

pub use input::touch::{RawTouchPhase, RawTouchPoint, TouchButtonState, TouchController, TouchLayout};
pub use input::{DebugOverlays, InputController};
pub use profile::{draw_country_banner, CountryInfo, CountryRegistry, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
pub use render::color::{CarColorScheme, Palette};
pub use render::lateral::render_car_lateral;
pub use render::ghost::{lerp_angle, render_ghost_car, GhostFrame, GhostLap, GhostRecorder};
pub use render::{compute_adaptive_alpha, PlayerVisibilityOptions};
pub use replay::{PlaybackSpeed, Replay, ReplayHeader, ReplayInputFrame, ReplayKeyframe, ReplayPlayer, ReplayRecorder};
pub use ui::curve_indicator::CurveColorScheme;
pub use ui::hud::{render_hud, render_split_hud, PersonalBestNotification, VisibilityToast};
pub use ui::menu::{CarChoice, GameMode, GameModeChoice, RaceResultEntry, TrackChoice};
pub use ui::profile_ui::{render_profile_badge, render_profile_create_screen, render_profile_manager_screen};
pub use ui::garage::{garage_select_button_rect, render_garage_screen, GarageViewMode};
pub use ui::starting_grid::{
    render_starting_grid_screen, starting_grid_garage_button_rect, starting_grid_launch_button_rect,
    StartingGridFocus,
};
