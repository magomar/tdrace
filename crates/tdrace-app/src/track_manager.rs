use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
};
use tdrace_core::track::{Track, TrackCategory};

use crate::ui::menu::TrackChoice;

/// Metadata for a custom user-created track stored on disk or in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomTrackInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: TrackCategory,
    pub file_path: String,
    pub length_m: f32,
    pub waypoint_count: usize,
    pub checkpoint_count: usize,
    pub jump_ramp_count: usize,
    pub obstacle_count: usize,
    pub default_surface: SurfaceType,
    pub default_laps: u32,
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
                                description: track.description,
                                category: track.category,
                                file_path: path.to_string_lossy().to_string(),
                                length_m: track.spline.total_length(),
                                waypoint_count: track.spline.waypoints.len(),
                                checkpoint_count: track.checkpoints.len(),
                                jump_ramp_count: track.geometry.jump_ramps.len(),
                                obstacle_count: track.geometry.obstacles.len(),
                                default_surface: track.default_surface,
                                default_laps: track.default_laps,
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

    /// Returns Main category tracks: Tested & approved built-in presets + promoted custom tracks.
    pub fn main_track_choices(&self) -> Vec<TrackChoice> {
        let mut choices = Vec::with_capacity(TrackChoice::ALL.len() + self.custom_tracks.len());
        choices.extend(TrackChoice::ALL);

        for custom in &self.custom_tracks {
            if custom.category == TrackCategory::Main {
                choices.push(TrackChoice::Custom {
                    id: custom.id.clone(),
                    title: custom.title.clone(),
                    description: custom.description.clone(),
                    path: custom.file_path.clone(),
                });
            }
        }

        choices
    }

    /// Returns Draft / Testing category tracks: Work in progress and experimental prototypes.
    pub fn draft_track_choices(&self) -> Vec<TrackChoice> {
        let mut choices = Vec::new();

        for custom in &self.custom_tracks {
            if custom.category == TrackCategory::Draft {
                choices.push(TrackChoice::Custom {
                    id: custom.id.clone(),
                    title: custom.title.clone(),
                    description: custom.description.clone(),
                    path: custom.file_path.clone(),
                });
            }
        }

        choices
    }

    /// Returns all available track choices appearing in main menu (Main category tracks).
    pub fn all_track_choices(&self) -> Vec<TrackChoice> {
        self.main_track_choices()
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

    /// Saves a track to disk in the tracks directory. Newly saved tracks always start in the Draft category.
    pub fn save_custom_track(&mut self, track: &Track, slug: Option<&str>) -> Result<String, String> {
        let mut track_to_save = track.clone();
        track_to_save.category = TrackCategory::Draft;

        let file_slug = if let Some(s) = slug {
            s.to_string()
        } else {
            let sanitized = track_to_save
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

        track_to_save
            .save_to_file(&path)
            .map_err(|e| format!("Failed to save custom track: {}", e))?;

        let _ = self.scan_custom_tracks();
        Ok(path.to_string_lossy().to_string())
    }

    /// Promotes a track from Draft to Main category (Approved circuit).
    pub fn promote_track(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let mut track = Track::load_from_file(&path_str)
                .map_err(|e| format!("Failed to load track to promote: {}", e))?;
            track.category = TrackCategory::Main;
            track
                .save_to_file(&path_str)
                .map_err(|e| format!("Failed to save promoted track: {}", e))?;
            let _ = self.scan_custom_tracks();
            Ok(())
        } else {
            Err(format!("Custom track '{}' not found", id))
        }
    }

    /// Demotes a track from Main category back to Draft / Testing.
    pub fn demote_track(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let mut track = Track::load_from_file(&path_str)
                .map_err(|e| format!("Failed to load track to demote: {}", e))?;
            track.category = TrackCategory::Draft;
            track
                .save_to_file(&path_str)
                .map_err(|e| format!("Failed to save demoted track: {}", e))?;
            let _ = self.scan_custom_tracks();
            Ok(())
        } else {
            Err(format!("Custom track '{}' not found", id))
        }
    }

    /// Updates track metadata (name and description) on disk.
    pub fn update_track_metadata(
        &mut self,
        id: &str,
        new_name: String,
        new_description: String,
    ) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let mut track = Track::load_from_file(&path_str)
                .map_err(|e| format!("Failed to load track to update: {}", e))?;
            track.name = new_name;
            track.description = new_description;
            track
                .save_to_file(&path_str)
                .map_err(|e| format!("Failed to save updated track: {}", e))?;
            let _ = self.scan_custom_tracks();
            Ok(())
        } else {
            Err(format!("Custom track '{}' not found", id))
        }
    }

    /// Creates a new starter draft circuit in the tracks directory.
    pub fn create_new_draft_track(
        &mut self,
        name: &str,
        description: &str,
    ) -> Result<String, String> {
        let mut track = classic_grand_prix();
        track.name = name.to_string();
        track.description = description.to_string();
        track.category = TrackCategory::Draft;

        self.save_custom_track(&track, None)
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
        let temp_dir = std::env::temp_dir().join("tdrace_test_tracks_mgr");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = TrackManager::new(&temp_dir);
        let choices = manager.all_track_choices();
        assert_eq!(choices.len(), 7); // 7 presets initially

        let mut gp = classic_grand_prix();
        gp.name = "My Custom GP".to_string();
        gp.description = "A custom testing GP".to_string();
        gp.category = TrackCategory::Draft;

        let saved_path = manager
            .save_custom_track(&gp, Some("test_custom_gp"))
            .expect("Must save custom track");
        assert!(Path::new(&saved_path).exists());

        // Since gp was saved as Draft, main choices is still 7, but draft choices has 1
        assert_eq!(manager.main_track_choices().len(), 7);
        assert_eq!(manager.draft_track_choices().len(), 1);

        let draft_choice = &manager.draft_track_choices()[0];
        assert_eq!(draft_choice.title(), "My Custom GP");
        assert_eq!(draft_choice.description(), "A custom testing GP");

        // Promote track to Main
        manager.promote_track("test_custom_gp").expect("Must promote");
        assert_eq!(manager.main_track_choices().len(), 8);
        assert_eq!(manager.draft_track_choices().len(), 0);

        // Edit metadata
        manager
            .update_track_metadata(
                "test_custom_gp",
                "Renamed Grand Prix".to_string(),
                "Updated description text".to_string(),
            )
            .expect("Must update metadata");
        let loaded = manager.load_track(&manager.main_track_choices()[7]).expect("Must load");
        assert_eq!(loaded.name, "Renamed Grand Prix");
        assert_eq!(loaded.description, "Updated description text");

        // Demote back to draft
        manager.demote_track("test_custom_gp").expect("Must demote");
        assert_eq!(manager.main_track_choices().len(), 7);
        assert_eq!(manager.draft_track_choices().len(), 1);

        // Clean up
        assert!(manager.delete_custom_track("test_custom_gp").unwrap());
        assert_eq!(manager.main_track_choices().len(), 7);
        assert_eq!(manager.draft_track_choices().len(), 0);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
