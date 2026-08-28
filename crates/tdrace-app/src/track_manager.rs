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
    pub surface_summary: String,
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

                            let surface_summary = track.surface_summary_string();
                            let length_m = track.spline.total_length();
                            let waypoint_count = track.spline.waypoints.len();
                            let checkpoint_count = track.checkpoints.len();
                            let jump_ramp_count = track.geometry.jump_ramps.len();
                            let obstacle_count = track.geometry.obstacles.len();
                            let default_surface = track.default_surface;
                            let default_laps = track.default_laps;
                            let category = track.category;

                            self.custom_tracks.push(CustomTrackInfo {
                                id: stem,
                                title: track.name,
                                description: track.description,
                                category,
                                file_path: path.to_string_lossy().to_string(),
                                length_m,
                                waypoint_count,
                                checkpoint_count,
                                jump_ramp_count,
                                obstacle_count,
                                default_surface,
                                surface_summary,
                                default_laps,
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
            TrackChoice::Custom { id, path, .. } => {
                if id == "monza" {
                    return Ok(crate::module::f1::F1GameModule::track_monza());
                }
                if id == "spa" {
                    return Ok(crate::module::f1::F1GameModule::track_spa());
                }
                if id == "silverstone" {
                    return Ok(crate::module::f1::F1GameModule::track_silverstone());
                }
                if id == "sahara" {
                    return Ok(tdrace_core::track::presets::sahara_dunes());
                }
                Track::load_from_file(path)
                    .map_err(|e| format!("Failed to load custom track from {}: {}", path, e))
            }
        }
    }

    /// Converts a track name into a valid file slug.
    pub fn sanitize_slug(name: &str) -> String {
        let sanitized = name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        let trimmed = sanitized.trim_matches('_');
        if trimmed.is_empty() {
            "custom_track".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Checks if a custom track file with the given slug already exists on disk.
    pub fn track_file_exists(&self, slug: &str) -> bool {
        let file_name = format!("{}.json", slug);
        self.tracks_dir.join(file_name).exists()
    }

    /// Resolves the destination path for a given slug.
    pub fn track_path_for_slug(&self, slug: &str) -> PathBuf {
        self.tracks_dir.join(format!("{}.json", slug))
    }

    /// Saves a track to disk with overwrite control.
    /// If `overwrite` is false and a file with `slug` exists, appends numbers (`_1`, `_2`, etc.) to find an unused file path.
    pub fn save_custom_track_with_options(
        &mut self,
        track: &Track,
        slug: Option<&str>,
        overwrite: bool,
    ) -> Result<String, String> {
        let mut track_to_save = track.clone();
        track_to_save.category = TrackCategory::Draft;

        let base_slug = if let Some(s) = slug {
            Self::sanitize_slug(s)
        } else {
            Self::sanitize_slug(&track_to_save.name)
        };

        let file_slug = if overwrite {
            base_slug
        } else {
            let mut candidate = base_slug.clone();
            let mut counter = 1;
            while self.tracks_dir.join(format!("{}.json", candidate)).exists() {
                candidate = format!("{}_{}", base_slug, counter);
                counter += 1;
            }
            candidate
        };

        let file_name = format!("{}.json", file_slug);
        let path = self.tracks_dir.join(file_name);

        track_to_save
            .save_to_file(&path)
            .map_err(|e| format!("Failed to save custom track: {}", e))?;

        let _ = self.scan_custom_tracks();
        Ok(path.to_string_lossy().to_string())
    }

    /// Saves a track to disk in the tracks directory. Overwrites by default.
    pub fn save_custom_track(&mut self, track: &Track, slug: Option<&str>) -> Result<String, String> {
        self.save_custom_track_with_options(track, slug, true)
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

    #[test]
    fn test_track_manager_overwrite_options() {
        let temp_dir = std::env::temp_dir().join("tdrace_test_tracks_overwrite");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = TrackManager::new(&temp_dir);

        let mut track = classic_grand_prix();
        track.name = "Awesome Track".to_string();

        // 1. Initial save
        let path1 = manager
            .save_custom_track_with_options(&track, Some("awesome_track"), true)
            .expect("First save should succeed");
        assert!(Path::new(&path1).exists());
        assert!(manager.track_file_exists("awesome_track"));

        // 2. Save again with overwrite: false -> should generate awesome_track_1.json
        let path2 = manager
            .save_custom_track_with_options(&track, Some("awesome_track"), false)
            .expect("Second save with overwrite: false should create copy");
        assert!(path2.ends_with("awesome_track_1.json"));
        assert!(Path::new(&path2).exists());

        // 3. Save again with overwrite: false -> should generate awesome_track_2.json
        let path3 = manager
            .save_custom_track_with_options(&track, Some("awesome_track"), false)
            .expect("Third save with overwrite: false should create copy");
        assert!(path3.ends_with("awesome_track_2.json"));
        assert!(Path::new(&path3).exists());

        // 4. Save with overwrite: true -> should overwrite awesome_track.json without error
        track.name = "Awesome Track v2".to_string();
        let path_overwrite = manager
            .save_custom_track_with_options(&track, Some("awesome_track"), true)
            .expect("Overwrite save should succeed");
        assert!(path_overwrite.ends_with("awesome_track.json"));

        let loaded = Track::load_from_file(&path_overwrite).expect("Should load overwritten track");
        assert_eq!(loaded.name, "Awesome Track v2");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sanitize_slug_and_file_existence() {
        assert_eq!(TrackManager::sanitize_slug("My Super Track!"), "my_super_track");
        assert_eq!(TrackManager::sanitize_slug("   __Track--123__  "), "track__123");
        assert_eq!(TrackManager::sanitize_slug(""), "custom_track");
        assert_eq!(TrackManager::sanitize_slug("!!!"), "custom_track");
    }
}
