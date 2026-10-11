//! Track, barrier, scenery and surface material renderers, and the colour palette.

pub mod barrier;
pub mod color;
pub mod optimal_line;
pub mod scenery;
pub mod surface_material;
pub mod track;

pub use optimal_line::render_optimal_racing_line;
pub use surface_material::set_asset_root;
