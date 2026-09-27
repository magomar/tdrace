//! One resolver for official circuits (specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md §2.3).
//!
//! An official circuit is read from the catalog embedded in `tdrace_core::catalog`. In dev mode a file
//! `tracks/<module>/<id>.json` on disk replaces the embedded copy. The user folder is never read here.

use tdrace_core::catalog;
use tdrace_core::track::Track;

/// Loads an official circuit by id or alias, or returns `None` when the slug is not an official circuit.
pub fn load(slug: &str, module_hint: Option<&str>) -> Option<Result<Track, String>> {
    let circuit = catalog::find(slug, module_hint)?;
    if crate::storage::is_dev_mode() {
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
            let path = git_tracks_dir.join(circuit.module).join(format!("{}.json", circuit.id));
            if path.exists() {
                match Track::load_from_file(&path) {
                    Ok(track) => return Some(Ok(track)),
                    Err(e) => eprintln!("official circuit {}: {}; using the embedded copy", path.display(), e),
                }
            }
        }
    }
    Some(
        circuit
            .load()
            .map_err(|e| format!("official circuit '{}/{}': {}", circuit.module, circuit.id, e)),
    )
}

/// The circuit used when a requested circuit cannot be loaded.
pub fn fallback_track() -> Track {
    catalog::official_track("classic", "classic_grand_prix")
}
