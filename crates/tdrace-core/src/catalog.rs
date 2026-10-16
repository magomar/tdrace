//! Official circuit catalog, embedded at build time from `tracks/<module>/<id>.json`.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md` and
//! `specs/121_two_stage_lazy_circuit_menu_loading_and_compile_time_card_metadata.md`.
//! The build script stores each circuit DEFLATE-compressed along with static card metadata
//! and downsampled thumbnail polylines; a circuit is decompressed only when its dossier
//! preview or full race geometry is requested. List order per module follows `tracks/.track_order.json`.

use crate::physics::surface::SurfaceType;
use crate::track::{Track, TrackError};

/// One official circuit as embedded in the binary.
#[derive(Debug)]
pub struct EmbeddedCircuit {
    pub module: &'static str,
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub tag: &'static str,
    pub category_label: &'static str,
    /// ISO 3166-1 alpha-2 code of a real venue, or "" for a fictional circuit.
    pub country_code: &'static str,
    pub default_laps: u32,
    /// Precomputed centerline length in meters (matching `Track::total_length_m()`).
    pub total_length_m: f32,
    /// Precomputed surface composition summary (matching `Track::surface_summary_string()`).
    pub surface_summary: &'static str,
    /// Circuit scale string (e.g. `"1:1"` or `"0.75:1"`).
    pub scale: &'static str,
    /// Whether this circuit displays the `INSPIRED` badge.
    pub is_inspired: bool,
    /// Off-track terrain surface type used for thumbnail card backdrop tinting.
    pub default_surface: SurfaceType,
    /// Whether the main track spline forms a closed loop.
    pub closed: bool,
    /// Precomputed 2D bounding box minimum `[min_x, min_y]` across main spline and branches.
    pub bounds_min: [f32; 2],
    /// Precomputed 2D bounding box maximum `[max_x, max_y]` across main spline and branches.
    pub bounds_max: [f32; 2],
    /// Downsampled main spline points and surfaces for zero-allocation `58x44` card thumbnails.
    pub thumbnail_samples: &'static [([f32; 2], SurfaceType)],
    /// Downsampled joker detour and launch chute segments for zero-allocation card thumbnails.
    pub thumbnail_branches: &'static [&'static [([f32; 2], SurfaceType)]],
    data: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/official_catalog.rs"));

impl EmbeddedCircuit {
    /// Decompresses and parses this circuit with full runtime geometry (for races, validation, and editor).
    pub fn load(&self) -> Result<Track, TrackError> {
        let json = miniz_oxide::inflate::decompress_to_vec(self.data)
            .map_err(|e| TrackError::Json(format!("{}/{}: corrupt embedded data: {:?}", self.module, self.id, e)))?;
        let text = std::str::from_utf8(&json).map_err(|e| TrackError::Json(e.to_string()))?;
        Track::from_json(text)
    }

    /// Decompresses and parses this circuit for lightweight menu/preview inspection (Spec 121).
    pub fn load_menu(&self) -> Result<Track, TrackError> {
        let json = miniz_oxide::inflate::decompress_to_vec(self.data)
            .map_err(|e| TrackError::Json(format!("{}/{}: corrupt embedded data: {:?}", self.module, self.id, e)))?;
        let text = std::str::from_utf8(&json).map_err(|e| TrackError::Json(e.to_string()))?;
        Track::from_json_menu(text)
    }
}

#[inline]
fn module_matches(c_module: &str, requested: &str) -> bool {
    c_module == requested
        || (c_module == "rally" && requested == "rallycross")
        || (c_module == "rallycross" && requested == "rally")
}

/// All official circuits, grouped by module in directory-name order, each module in list order.
pub fn circuits() -> &'static [EmbeddedCircuit] {
    CIRCUITS
}

/// Official circuits of one module, in list order.
pub fn module_circuits(module: &str) -> impl Iterator<Item = &'static EmbeddedCircuit> + '_ {
    CIRCUITS.iter().filter(move |c| module_matches(c.module, module))
}

/// Old or short circuit names and the catalog id each maps to (`tracks/.aliases.json`).
pub fn aliases() -> &'static [(&'static str, &'static str)] {
    ALIASES
}

/// Maps an old or short circuit name to its catalog id. Returns the input when it is already an id.
pub fn canonical_id(slug: &str) -> Option<&'static str> {
    if let Some(c) = CIRCUITS.iter().find(|c| c.id == slug) {
        return Some(c.id);
    }
    ALIASES.iter().find(|(alias, _)| *alias == slug).map(|(_, id)| *id)
}

/// Finds an official circuit by id or alias. `module_hint` picks the module when an id exists in several.
pub fn find(slug: &str, module_hint: Option<&str>) -> Option<&'static EmbeddedCircuit> {
    let id = canonical_id(slug)?;
    module_hint
        .and_then(|m| CIRCUITS.iter().find(|c| c.id == id && module_matches(c.module, m)))
        .or_else(|| CIRCUITS.iter().find(|c| c.id == id))
}

/// Loads an official circuit by id or alias, for tests, benches and tools.
///
/// # Panics
/// Panics when the circuit is not in the catalog or cannot be parsed.
pub fn official_track(module: &str, slug: &str) -> Track {
    let circuit = find(slug, Some(module)).unwrap_or_else(|| panic!("no official circuit '{}/{}'", module, slug));
    circuit
        .load()
        .unwrap_or_else(|e| panic!("official circuit '{}/{}': {}", module, slug, e))
}
