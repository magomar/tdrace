use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
};
use tdrace_core::track::Track;

use crate::ui::menu::TrackChoice;

/// Metadata for a custom user-created track stored on disk or in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomTrackInfo {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub length_m: f32,
    pub waypoint_count: usize,
    pub checkpoint_count: usize,
}

/// Manages discovery, loading, saving, and cataloging of custom and preset tracks.
#[derive(Debug, Clone)]
pub struct TrackManager {
    pub tracks_dir: PathBuf,
    pub custom_tracks: Vec<CustomTrackInfo>,
}

impl Default for TrackManager {
    fn default() -> Self {
        Self::new("tracks")
    }
}

impl TrackManager {
    pub fn new(tracks_dir: impl AsRef<Path>) -> Self {
        let mut manager = Self {
            tracks_dir: tracks_dir.as_ref().to_path_buf(),
            custom_tracks: Vec::new(),
        };
        let _ = manager.scan_custom_tracks();
        manager
    }

    /// Scans the tracks directory for `.json` and `.tdtrack` files.
    pub fn scan_custom_tracks(&mut self) -> Result<usize, String> {
        self.custom_tracks.clear();

        if !self.tracks_dir.exists() {
            let _ = fs::create_dir_all(&self.tracks_dir);
            return Ok(0);
        }

        let entries = fs::read_dir(&self.tracks_dir)
            .map_err(|e| format!("Failed to read tracks directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("tdtrack") {
                        if let Ok(track) = Track::load_from_file(&path) {
                            let stem = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("custom_track")
                                .to_string();

                            self.custom_tracks.push(CustomTrackInfo {
                                id: stem,
                                title: track.name,
                                file_path: path.to_string_lossy().to_string(),
                                length_m: track.spline.total_length(),
                                waypoint_count: track.spline.waypoints.len(),
                                checkpoint_count: track.checkpoints.len(),
                            });
                        }
                    }
                }
            }
        }

        // Sort alphabetically by title
        self.custom_tracks.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(self.custom_tracks.len())
    }

    /// Returns all available track choices: Presets followed by Custom tracks.
    pub fn all_track_choices(&self) -> Vec<TrackChoice> {
        let mut choices = Vec::with_capacity(TrackChoice::ALL.len() + self.custom_tracks.len());
        choices.extend(TrackChoice::ALL);

        for custom in &self.custom_tracks {
            choices.push(TrackChoice::Custom {
                id: custom.id.clone(),
                title: custom.title.clone(),
                path: custom.file_path.clone(),
            });
        }

        choices
    }

    /// Loads a `Track` from a `TrackChoice`.
    pub fn load_track(&self, choice: &TrackChoice) -> Result<Track, String> {
        match choice {
            TrackChoice::ClassicGrandPrix => Ok(classic_grand_prix()),
            TrackChoice::OvalSpeedway => Ok(oval_speedway()),
            TrackChoice::DriftPark => Ok(drift_park()),
            TrackChoice::KartArena => Ok(kart_arena()),
            TrackChoice::RampRaceway => Ok(ramp_raceway()),
            TrackChoice::OasisRally => Ok(oasis_rally()),
            TrackChoice::OutlawPass => Ok(outlaw_pass()),
            TrackChoice::Custom { path, .. } => Track::load_from_file(path)
                .map_err(|e| format!("Failed to load custom track from {}: {}", path, e)),
        }
    }

    /// Saves a track to disk in the tracks directory.
    pub fn save_custom_track(&mut self, track: &Track, slug: Option<&str>) -> Result<String, String> {
        let file_slug = if let Some(s) = slug {
            s.to_string()
        } else {
            let sanitized = track
                .name
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
            if sanitized.trim().is_empty() {
                "custom_track".to_string()
            } else {
                sanitized
            }
        };

        let file_name = format!("{}.json", file_slug);
        let path = self.tracks_dir.join(file_name);

        track
            .save_to_file(&path)
            .map_err(|e| format!("Failed to save custom track: {}", e))?;

        let _ = self.scan_custom_tracks();
        Ok(path.to_string_lossy().to_string())
    }

    /// Deletes a custom track by ID.
    pub fn delete_custom_track(&mut self, id: &str) -> Result<bool, String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let _ = fs::remove_file(&path_str);
            self.custom_tracks.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_manager_presets_and_custom_save() {
        let temp_dir = std::env::temp_dir().join("tdrace_test_tracks");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = TrackManager::new(&temp_dir);
        let choices = manager.all_track_choices();
        assert_eq!(choices.len(), 7); // 7 presets initially

        let gp = classic_grand_prix();
        let saved_path = manager
            .save_custom_track(&gp, Some("test_custom_gp"))
            .expect("Must save custom track");
        assert!(Path::new(&saved_path).exists());

        let choices_after = manager.all_track_choices();
        assert_eq!(choices_after.len(), 8);

        let custom_choice = &choices_after[7];
        assert_eq!(custom_choice.title(), "Classic Grand Prix");
        assert!(custom_choice.is_custom());

        let loaded = manager.load_track(custom_choice).expect("Must load track");
        assert_eq!(loaded.name, "Classic Grand Prix");

        // Clean up
        assert!(manager.delete_custom_track("test_custom_gp").unwrap());
        assert_eq!(manager.all_track_choices().len(), 7);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
