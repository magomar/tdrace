use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tdrace_core::track::{Track, TrackCategory};
use crate::track_manager::CustomTrackInfo;

/// Safe user storage for custom, cloned, and imported tracks.
/// Located at `~/.local/share/tdrace/tracks/` (or test/env overrides).
///
/// Safety guarantees:
/// 1. Every delete or overwrite automatically archives a copy into `.backup/`.
/// 2. User tracks are NEVER deleted when promoted to presets.
/// 3. If a track file goes missing, `.backup/` serves as an automated fallback recovery.
#[derive(Debug, Clone)]
pub struct UserTrackStore {
    tracks_dir: PathBuf,
}

impl UserTrackStore {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let p = dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&p);
        Self { tracks_dir: p }
    }

    pub fn default_store() -> Self {
        Self::new(crate::storage::resolve_user_tracks_dir())
    }

    pub fn root_dir(&self) -> &Path {
        &self.tracks_dir
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.tracks_dir.join(".backup")
    }

    /// Automatically backs up a track file into `.backup/`.
    /// Saves both `<filename>` (latest snapshot) and `<stem>_<unix_secs>.<ext>` (timestamped history).
    pub fn backup_file(&self, file: &Path) {
        if !file.exists() || !file.is_file() {
            return;
        }
        let bdir = self.backup_dir();
        let _ = fs::create_dir_all(&bdir);

        if let Some(file_name) = file.file_name().and_then(|s| s.to_str()) {
            let latest = bdir.join(file_name);
            let _ = fs::copy(file, &latest);

            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("json");
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let timestamped = bdir.join(format!("{}_{}.{}", stem, ts, ext));
                let _ = fs::copy(file, timestamped);
            }
        }
    }

    /// Resolves the file path for a track slug.
    pub fn path_for_slug(&self, slug: &str) -> PathBuf {
        let file_name = format!("{}.json", slug);
        let flat = self.tracks_dir.join(&file_name);
        if flat.exists() {
            return flat;
        }
        for sub in &["drafts", "classic", "gt", "rally", "kart", "nascar"] {
            let cand = self.tracks_dir.join(sub).join(&file_name);
            if cand.exists() {
                return cand;
            }
        }
        flat
    }

    /// Checks if a track file exists in user storage.
    pub fn track_exists(&self, slug: &str) -> bool {
        let file_name = format!("{}.json", slug);
        self.tracks_dir.join(&file_name).exists()
            || ["drafts", "classic", "gt", "rally", "kart", "nascar"]
                .iter()
                .any(|sub| self.tracks_dir.join(sub).join(&file_name).exists())
    }

    /// Loads a track by slug, checking user storage first, then falling back to `.backup/`.
    pub fn load_track(&self, slug: &str) -> Option<Track> {
        let p = self.path_for_slug(slug);
        if p.exists() {
            if let Ok(t) = Track::load_from_file(&p) {
                return Some(t);
            }
        }
        // Safety recovery fallback
        let backup_p = self.backup_dir().join(format!("{}.json", slug));
        if backup_p.exists() {
            if let Ok(t) = Track::load_from_file(&backup_p) {
                return Some(t);
            }
        }
        None
    }

    /// Saves a track into user storage.
    /// If `overwrite` is true and an existing file is present, it is backed up automatically before writing.
    pub fn save_track(
        &self,
        track: &Track,
        slug: &str,
        overwrite: bool,
    ) -> Result<PathBuf, String> {
        let _ = fs::create_dir_all(&self.tracks_dir);
        let mut final_slug = slug.to_string();

        if !overwrite {
            let mut counter = 1;
            while self.track_exists(&final_slug) {
                final_slug = format!("{}_{}", slug, counter);
                counter += 1;
            }
        }

        let target_path = if overwrite {
            let existing = self.path_for_slug(&final_slug);
            if existing.exists() {
                self.backup_file(&existing);
                existing
            } else {
                self.tracks_dir.join(format!("{}.json", final_slug))
            }
        } else {
            self.tracks_dir.join(format!("{}.json", final_slug))
        };

        track
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save track to '{}': {}", target_path.display(), e))?;

        // Also create a fresh backup snapshot of the newly saved version
        self.backup_file(&target_path);

        Ok(target_path)
    }

    /// Deletes a user track by slug from all user storage locations.
    /// Files are backed up to `.backup/` before deletion so they can be recovered.
    pub fn delete_track(&self, slug: &str) -> Result<bool, String> {
        let file_name = format!("{}.json", slug);
        let tdtrack_name = format!("{}.tdtrack", slug);

        let candidates = [
            self.tracks_dir.join(&file_name),
            self.tracks_dir.join(&tdtrack_name),
            self.tracks_dir.join("drafts").join(&file_name),
            self.tracks_dir.join("drafts").join(&tdtrack_name),
            self.tracks_dir.join("classic").join(&file_name),
            self.tracks_dir.join("gt").join(&file_name),
            self.tracks_dir.join("rally").join(&file_name),
            self.tracks_dir.join("kart").join(&file_name),
            self.tracks_dir.join("nascar").join(&file_name),
        ];

        let mut deleted_any = false;
        for cand in &candidates {
            if cand.exists() {
                self.backup_file(cand);
                let _ = fs::remove_file(cand);
                deleted_any = true;
            }
        }

        Ok(deleted_any)
    }

    /// Scans user storage for custom tracks, skipping dot-directories (such as `.backup`).
    pub fn scan_tracks(&self) -> Vec<CustomTrackInfo> {
        let mut tracks = Vec::new();
        if !self.tracks_dir.exists() {
            return tracks;
        }

        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.tracks_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.starts_with('.') {
                    continue;
                }
                if p.is_file() {
                    files.push((p, None));
                } else if p.is_dir() {
                    let subdir_name = name.to_string();
                    if let Ok(sub_entries) = fs::read_dir(&p) {
                        for sub_entry in sub_entries.flatten() {
                            let sp = sub_entry.path();
                            let sname = sp.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if !sname.starts_with('.') && sp.is_file() {
                                files.push((sp, Some(subdir_name.clone())));
                            }
                        }
                    }
                }
            }
        }

        for (path, subdir) in files {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("tdtrack") {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("custom_track")
                    .to_string();

                if let Ok(track) = Track::load_from_file(&path) {
                    let mut category = track.category;
                    let mut module_id = track.module_id.clone();
                    let mut modules = track.modules.clone();

                    if let Some(ref dir) = subdir {
                        if dir == "drafts" {
                            category = TrackCategory::Draft;
                            module_id = None;
                            modules.clear();
                        } else if modules.is_empty() && module_id.is_none() {
                            category = TrackCategory::Main;
                            module_id = Some(dir.clone());
                            modules = vec![dir.clone()];
                        }
                    }

                    tracks.push(CustomTrackInfo {
                        id: stem,
                        title: track.name.clone(),
                        description: track.description.clone(),
                        category,
                        module_id,
                        modules,
                        file_path: path.to_string_lossy().to_string(),
                        length_m: track.spline.total_length(),
                        waypoint_count: track.spline.waypoints.len(),
                        checkpoint_count: track.checkpoints.len(),
                        jump_ramp_count: track.geometry.jump_ramps.len(),
                        obstacle_count: track.geometry.obstacles.len(),
                        default_surface: track.default_surface,
                        surface_summary: track.surface_summary_string(),
                        default_laps: track.default_laps,
                    });
                }
            }
        }

        tracks.sort_by(|a, b| a.title.cmp(&b.title));
        tracks
    }
}
