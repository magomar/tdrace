//! Official circuit catalog, embedded at build time from `tracks/<module>/<id>.json`.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md`.
//! The build script stores each circuit DEFLATE-compressed; a circuit is decompressed
//! only when it is loaded. List order per module follows `tracks/.track_order.json`.

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
    pub default_laps: u32,
    data: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/official_catalog.rs"));

impl EmbeddedCircuit {
    /// Decompresses and parses this circuit.
    pub fn load(&self) -> Result<Track, TrackError> {
        let json = miniz_oxide::inflate::decompress_to_vec(self.data)
            .map_err(|e| TrackError::Json(format!("{}/{}: corrupt embedded data: {:?}", self.module, self.id, e)))?;
        let text = std::str::from_utf8(&json).map_err(|e| TrackError::Json(e.to_string()))?;
        Track::from_json(text)
    }
}

/// All official circuits, grouped by module in directory-name order, each module in list order.
pub fn circuits() -> &'static [EmbeddedCircuit] {
    CIRCUITS
}

/// Official circuits of one module, in list order.
pub fn module_circuits(module: &str) -> impl Iterator<Item = &'static EmbeddedCircuit> + '_ {
    CIRCUITS.iter().filter(move |c| c.module == module)
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
        .and_then(|m| CIRCUITS.iter().find(|c| c.id == id && c.module == m))
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
