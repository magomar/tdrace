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

/// Loads an official circuit for menu preview/metadata using [`Track::load_from_file_menu`] / [`catalog::EmbeddedCircuit::load_menu`],
/// skipping heavy runtime wall trimming and scenery obstacle synthesis.
pub fn load_menu(slug: &str, module_hint: Option<&str>) -> Option<Result<Track, String>> {
    let circuit = catalog::find(slug, module_hint)?;
    if crate::storage::is_dev_mode() {
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
            let path = git_tracks_dir.join(circuit.module).join(format!("{}.json", circuit.id));
            if path.exists() {
                match Track::load_from_file_menu(&path) {
                    Ok(track) => return Some(Ok(track)),
                    Err(e) => eprintln!("official circuit {}: {}; using the embedded copy", path.display(), e),
                }
            }
        }
    }
    Some(
        circuit
            .load_menu()
            .map_err(|e| format!("official circuit '{}/{}': {}", circuit.module, circuit.id, e)),
    )
}

/// Returns compile-time [`catalog::EmbeddedCircuit`] metadata and thumbnail geometry for menu list cards,
/// or `None` when the slug is not an official circuit or an explicit `TDRACE_GIT_TRACKS_DIR` override is active in dev mode.
pub fn embedded_for_menu(slug: &str, module_hint: Option<&str>) -> Option<&'static catalog::EmbeddedCircuit> {
    let circuit = catalog::find(slug, module_hint)?;
    if crate::storage::is_dev_mode() && std::env::var(crate::storage::ENV_GIT_TRACKS_DIR).is_ok() {
        return None;
    }
    Some(circuit)
}

/// The circuit used when a requested circuit cannot be loaded.
pub fn fallback_track() -> Track {
    catalog::official_track("classic", "gt_coastal_grand_prix")
}

