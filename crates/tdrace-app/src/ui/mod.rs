pub mod hud;
pub mod menu;

pub use hud::{format_lap_time, render_hud};
pub use menu::{
    render_pause_menu, render_results_screen, render_track_select_menu, CarChoice,
    GameModeChoice, RaceResultEntry, TrackChoice,
};
