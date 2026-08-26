pub mod ai;
pub mod audio;
pub mod camera;
pub mod config;
pub mod db;
pub mod fx;
pub mod game;
pub mod input;
pub mod render;
pub mod replay;
pub mod ui;

pub use ai::{BotAiDriver, BotProfile, DriverCharacter, DriverStats};
pub use audio::{AudioManager, AudioSettings, MusicTrack, SfxType};
pub use camera::{CameraMode, RaceCamera};
pub use config::{AudioConfig, CameraConfig, GameConfig, GameplayConfig, InputConfig, ZoomLevelConfig};
pub use db::{HallOfFameDb, HallOfFameEntry};
pub use fx::{DriftPopup, EffectsManager, ParticleSystem, SkidmarkBuffer};
pub use game::{GameState, RaceSession};

pub use input::touch::{RawTouchPhase, RawTouchPoint, TouchButtonState, TouchController, TouchLayout};
pub use input::{DebugOverlays, InputController};
pub use render::color::{CarColorScheme, Palette};
pub use render::ghost::{lerp_angle, render_ghost_car, GhostFrame, GhostLap, GhostRecorder};
pub use replay::{PlaybackSpeed, Replay, ReplayHeader, ReplayInputFrame, ReplayKeyframe, ReplayPlayer, ReplayRecorder};
pub use ui::hud::render_hud;
pub use ui::menu::{CarChoice, GameModeChoice, RaceResultEntry, TrackChoice};
