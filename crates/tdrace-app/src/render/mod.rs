pub mod barrier;
pub mod car;
pub mod color;
pub mod ghost;
pub mod lateral;
pub mod marker;
pub mod scenery;
pub mod surface_material;
pub mod track;
pub mod trophy_textures;
pub mod vehicle_assets;

pub use trophy_textures::{draw_trophy_badge, get_trophy_texture, normalize_discipline, trophy_filename};

pub use barrier::{
    render_barriers_and_obstacles, render_elevated_barriers_and_obstacles,
    render_elevated_barriers_and_obstacles_culled, render_ground_barriers_and_obstacles,
    render_ground_barriers_and_obstacles_culled,
};
pub use car::{
    render_car, render_car_with_visual_type, render_car_with_visual_type_and_model,
    render_car_with_visual_type_model_and_shadows,
};
pub use lateral::{render_car_lateral, render_lateral_car, render_real_car_lateral_by_id};
pub use color::{CarColorScheme, Palette};
pub use ghost::{lerp_angle, render_ghost_car, GhostFrame, GhostLap, GhostRecorder};
pub use marker::{
    compute_adaptive_alpha, compute_proximity_alpha, deconflict_nameplates,
    render_floating_bot_nameplates, render_player_ground_aura, render_player_overhead_chevron,
    render_player_roof_beacon, DeconflictedNameplate, PlayerVisibilityOptions,
    VehicleNameplateItem, MAX_VISIBLE_NAMEPLATES, NAMEPLATE_DECONFLICT_H_THRESH,
    NAMEPLATE_DECONFLICT_W_THRESH, NAMEPLATE_HEIGHT_CLEARANCE, NAMEPLATE_INNER_RADIUS,
    NAMEPLATE_OUTER_RADIUS, NAMEPLATE_STACK_NUDGE,
};
pub use scenery::{
    render_grandstand, render_grandstand_shadows_culled, render_grandstands_culled,
    render_tree_canopies_culled, render_tree_shadows_culled, render_tree_trunks_culled,
};
pub use surface_material::{
    evaluate_macro_modulation, generate_curb_image, generate_edge_fringe_mask,
    generate_macro_noise_image, generate_surface_image, generate_tire_rubber_image,
    SurfaceMaterial, SurfaceMaterialRegistry, SurfaceTextureQuality, TrackWearState,
};
pub use track::{
    get_track_backdrop_color, render_elevated_track, render_elevated_track_culled,
    render_ground_track, render_ground_track_culled, render_track, render_track_culled,
};
pub use vehicle_assets::{
    get_vehicle_lateral_texture, get_vehicle_topdown_chassis_texture, get_vehicle_topdown_texture,
};


