use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::track::TrackCategory;
use super::user_store::UserTrackStore;

/// Developer tooling for working directly with repository tracks (`tracks/<module>/*.json`).
///
/// Strictly guarded by `crate::storage::is_dev_mode()`.
/// Guarantees:
/// 1. Promoting a track to a Git preset NEVER deletes the user's local copy.
/// 2. Dual persistence ensures the user copy remains valid in user storage even if Git files are cleaned.
/// 3. Backups are recorded in `.backup/`.
#[derive(Debug, Clone, Default)]
pub struct DevTrackStore;

impl DevTrackStore {
    pub fn new() -> Self {
        Self
    }

    /// Checks if developer mode is active.
    pub fn is_dev() -> bool {
        crate::storage::is_dev_mode()
    }

    /// Resolves the Git repository tracks root directory if present.
    pub fn git_tracks_dir() -> Option<PathBuf> {
        crate::storage::resolve_git_tracks_dir()
    }

    /// Promotes a user custom track to an official git-tracked preset.
    ///
    /// Writes the track to `tracks/<target_module>/<slug>.json`.
    /// Preserves a dual-persistence copy in user storage with `category: Main`.
    /// Archives previous draft copies to `.backup/`.
    pub fn promote_to_git_preset(
        &self,
        user_store: &UserTrackStore,
        slug: &str,
        target_module: &str,
    ) -> Result<PathBuf, String> {
        if !Self::is_dev() {
            return Err("Promoting tracks to preset circuits is only allowed in developer mode.".to_string());
        }

        let git_dir = Self::git_tracks_dir()
            .ok_or_else(|| "Git repository tracks directory not found.".to_string())?;

        let mut track = user_store
            .load_track(slug)
            .ok_or_else(|| format!("Track '{}' not found in user storage.", slug))?;

        track.category = TrackCategory::Main;
        track.module_id = Some(target_module.to_string());
        track.modules = vec![target_module.to_string()];

        let target_dir = git_dir.join(target_module);
        let _ = fs::create_dir_all(&target_dir);
        let target_git_file = target_dir.join(format!("{}.json", slug));

        track
            .save_to_file(&target_git_file)
            .map_err(|e| format!("Failed to save git preset '{}': {}", target_git_file.display(), e))?;

        // DUAL PERSISTENCE: Save/update the user storage copy as an approved Main track
        let _ = user_store.save_track(&track, slug, true);

        // If a copy existed in drafts/, back it up and clean the drafts folder
        let draft_path = user_store.root_dir().join("drafts").join(format!("{}.json", slug));
        if draft_path.exists() {
            user_store.backup_file(&draft_path);
            let _ = fs::remove_file(draft_path);
        }

        Ok(target_git_file)
    }

    /// Demotes an official preset circuit to a custom draft in user storage.
    pub fn demote_git_preset(
        &self,
        user_store: &UserTrackStore,
        slug: &str,
    ) -> Result<PathBuf, String> {
        if !Self::is_dev() {
            return Err("Demoting preset circuits is only allowed in developer mode.".to_string());
        }

        let git_dir = Self::git_tracks_dir();
        if let Some(ref gd) = git_dir {
            for m in ["classic", "rally", "kart", "gt", "nascar"] {
                let p = gd.join(m).join(format!("{}.json", slug));
                if p.exists() {
                    user_store.backup_file(&p);
                    let _ = fs::remove_file(p);
                }
            }
        }

        let mut track = user_store
            .load_track(slug)
            .or_else(|| crate::ui::menu::TrackChoice::resolve_procedural_preset_by_slug(slug))
            .ok_or_else(|| format!("Preset '{}' not found.", slug))?;

        track.category = TrackCategory::Draft;
        track.module_id = None;
        track.modules.clear();

        user_store.save_track(&track, slug, true)
    }

    /// Loads preset order from `.track_order.json` (checking user storage then git repo).
    pub fn load_preset_order(user_dir: &Path) -> HashMap<String, Vec<String>> {
        let path = user_dir.join(".track_order.json");
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(order) = serde_json::from_str::<HashMap<String, Vec<String>>>(&data) {
                    return order;
                }
            }
        }
        if let Some(git_dir) = Self::git_tracks_dir() {
            let gp = git_dir.join(".track_order.json");
            if gp.exists() {
                if let Ok(data) = fs::read_to_string(&gp) {
                    if let Ok(order) = serde_json::from_str::<HashMap<String, Vec<String>>>(&data) {
                        return order;
                    }
                }
            }
        }
        HashMap::new()
    }

    /// Saves preset order to `.track_order.json` in user storage and git repo.
    pub fn save_preset_order(
        user_dir: &Path,
        order: &HashMap<String, Vec<String>>,
    ) {
        if let Ok(data) = serde_json::to_string_pretty(order) {
            let path = user_dir.join(".track_order.json");
            let _ = fs::write(&path, &data);

            if Self::is_dev() {
                if let Some(git_dir) = Self::git_tracks_dir() {
                    let gp = git_dir.join(".track_order.json");
                    let _ = fs::write(gp, &data);
                }
            }
        }
    }
}
