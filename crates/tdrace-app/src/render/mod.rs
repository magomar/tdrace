pub mod barrier;
pub mod car;
pub mod color;
pub mod ghost;
pub mod marker;
pub mod track;

pub use barrier::{
    render_barriers_and_obstacles, render_elevated_barriers_and_obstacles,
    render_ground_barriers_and_obstacles,
};
pub use car::render_car;
pub use color::{CarColorScheme, Palette};
pub use ghost::{lerp_angle, render_ghost_car, GhostFrame, GhostLap, GhostRecorder};
pub use marker::{
    compute_adaptive_alpha, render_player_ground_aura, render_player_overhead_chevron,
    render_player_roof_beacon, PlayerVisibilityOptions,
};
pub use track::{get_track_backdrop_color, render_elevated_track, render_ground_track, render_track};

