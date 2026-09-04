use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
};
use tdrace_core::track::{Track, TrackCategory};

use crate::module::classic::ClassicGameModule;
use crate::module::f1::F1GameModule;
use crate::module::kart::KartGameModule;
use crate::module::rally::RallyGameModule;
use crate::module::{GameModule, TrackDefinition};
use crate::ui::menu::TrackChoice;

/// Filter for selecting tracks by motorsport module in the Track Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleFilter {
    #[default]
    All,
    Classic,
    F1,
    Rally,
    Kart,
}

impl ModuleFilter {
    pub const ALL: [Self; 5] = [
        Self::All,
        Self::Classic,
        Self::Rally,
        Self::Kart,
        Self::F1,
    ];

    pub fn id(&self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Classic => Some("classic"),
            Self::Rally => Some("rally"),
            Self::Kart => Some("kart"),
            Self::F1 => Some("f1"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "ALL MODULES",
            Self::Classic => "CLASSIC",
            Self::Rally => "RALLY",
            Self::Kart => "KARTING",
            Self::F1 => "FORMULA 1",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::Classic,
            Self::Classic => Self::Rally,
            Self::Rally => Self::Kart,
            Self::Kart => Self::F1,
            Self::F1 => Self::All,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::All => Self::F1,
            Self::Classic => Self::All,
            Self::Rally => Self::Classic,
            Self::Kart => Self::Rally,
            Self::F1 => Self::Kart,
        }
    }

    pub fn for_module(mod_id: &str) -> Self {
        match mod_id {
            "f1" => Self::F1,
            "rally" => Self::Rally,
            "kart" => Self::Kart,
            _ => Self::Classic,
        }
    }
}

/// Metadata for a custom user-created track stored on disk or in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomTrackInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: TrackCategory,
    pub module_id: Option<String>,
    pub modules: Vec<String>,
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

impl CustomTrackInfo {
    pub fn belongs_to_module(&self, mod_id: &str) -> bool {
        if self.modules.iter().any(|m| m.eq_ignore_ascii_case(mod_id)) {
            return true;
        }
        if let Some(ref m) = self.module_id {
            if m.eq_ignore_ascii_case(mod_id) {
                return true;
            }
        }
        false
    }

    pub fn module_name(&self) -> &'static str {
        if let Some(ref m) = self.module_id {
            match m.to_lowercase().as_str() {
                "f1" => "Formula 1",
                "rally" => "Rally Cross",
                "kart" => "Karting",
                _ => "Classic",
            }
        } else if self.belongs_to_module("f1") {
            "Formula 1"
        } else if self.belongs_to_module("rally") {
            "Rally Cross"
        } else if self.belongs_to_module("kart") {
            "Karting"
        } else {
            "Classic"
        }
    }
}

/// Manages discovery, loading, saving, and cataloging of custom and preset tracks.
#[derive(Debug, Clone)]
pub struct TrackManager {
    pub tracks_dir: PathBuf,
    pub custom_tracks: Vec<CustomTrackInfo>,
    pub deleted_presets: Vec<String>,
}

impl Default for TrackManager {
    fn default() -> Self {
        Self::new(crate::storage::resolve_user_tracks_dir())
    }
}

impl TrackManager {
    pub fn new(tracks_dir: impl AsRef<Path>) -> Self {
        let dir = tracks_dir.as_ref().to_path_buf();
        let deleted_presets = Self::load_deleted_presets(&dir);
        let mut manager = Self {
            tracks_dir: dir,
            custom_tracks: Vec::new(),
            deleted_presets,
        };
        let _ = manager.scan_custom_tracks();
        manager
    }

    fn load_deleted_presets(tracks_dir: &Path) -> Vec<String> {
        let path = tracks_dir.join(".deleted_tracks.json");
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&data) {
                    return list;
                }
            }
        }
        Vec::new()
    }

    pub fn save_deleted_presets(&self) {
        let path = self.tracks_dir.join(".deleted_tracks.json");
        if let Ok(data) = serde_json::to_string_pretty(&self.deleted_presets) {
            let _ = fs::write(path, data);
        }
    }

    /// Scans the tracks directory and any subdirectories for `.json` and `.tdtrack` files.
    pub fn scan_custom_tracks(&mut self) -> Result<usize, String> {
        self.custom_tracks.clear();

        if !self.tracks_dir.exists() {
            let _ = fs::create_dir_all(&self.tracks_dir);
            return Ok(0);
        }

        let mut files_to_scan = Vec::new();

        // 1. Scan root tracks_dir and immediate subdirectories
        if let Ok(entries) = fs::read_dir(&self.tracks_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files_to_scan.push((path, None));
                } else if path.is_dir() {
                    let subdir_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                files_to_scan.push((sub_path, Some(subdir_name.clone())));
                            }
                        }
                    }
                }
            }
        }

        for (path, subdir) in files_to_scan {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("tdtrack") {
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name.starts_with('.') {
                        continue;
                    }
                    if let Ok(mut track) = Track::load_from_file(&path) {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("custom_track")
                            .to_string();

                        let mut modules = track.modules.clone();
                        // Infer category and module_id from subdirectory if file lacks explicit metadata
                        if let Some(ref dir) = subdir {
                            if modules.is_empty() && track.module_id.is_none() {
                                if dir == "drafts" {
                                    track.category = TrackCategory::Draft;
                                    track.module_id = None;
                                    modules.clear();
                                } else {
                                    track.category = TrackCategory::Main;
                                    track.module_id = Some(dir.clone());
                                    modules = vec![dir.clone()];
                                }
                            }
                        } else if track.category == TrackCategory::Main {
                            let mod_id = track.module_id.clone().unwrap_or_else(|| "classic".to_string());
                            if track.module_id.is_none() {
                                track.module_id = Some(mod_id.clone());
                            }
                            if modules.is_empty() {
                                modules = vec![mod_id];
                            }
                        }

                        let surface_summary = track.surface_summary_string();
                        let length_m = track.spline.total_length();
                        let waypoint_count = track.spline.waypoints.len();
                        let checkpoint_count = track.checkpoints.len();
                        let jump_ramp_count = track.geometry.jump_ramps.len();
                        let obstacle_count = track.geometry.obstacles.len();
                        let default_surface = track.default_surface;
                        let default_laps = track.default_laps;
                        let category = track.category;
                        let module_id = if category == TrackCategory::Main && track.module_id.is_none() {
                            Some("classic".to_string())
                        } else {
                            track.module_id.clone()
                        };

                        self.custom_tracks.push(CustomTrackInfo {
                            id: stem,
                            title: track.name,
                            description: track.description,
                            category,
                            module_id,
                            modules,
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

        // Deduplicate in case a file was scanned twice
        self.custom_tracks.sort_by(|a, b| a.id.cmp(&b.id));
        self.custom_tracks.dedup_by(|a, b| a.file_path == b.file_path);

        // Sort alphabetically by title
        self.custom_tracks.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(self.custom_tracks.len())
    }

    /// Returns Main category tracks: Tested & approved built-in presets + promoted custom tracks across all modules.
    pub fn main_track_choices(&self) -> Vec<TrackChoice> {
        self.module_catalog_tracks("all")
    }

    /// Returns Main category tracks filtered by a specific ModuleFilter.
    pub fn filtered_main_track_choices(&self, filter: ModuleFilter) -> Vec<TrackChoice> {
        match filter {
            ModuleFilter::All => self.main_track_choices(),
            ModuleFilter::Classic => self.module_catalog_tracks("classic"),
            ModuleFilter::F1 => self.module_catalog_tracks("f1"),
            ModuleFilter::Rally => self.module_catalog_tracks("rally"),
            ModuleFilter::Kart => self.module_catalog_tracks("kart"),
        }
    }

    /// Returns Draft / Testing category tracks: Work in progress and experimental prototypes.
    pub fn draft_track_choices(&self) -> Vec<TrackChoice> {
        let mut choices = Vec::new();

        for custom in &self.custom_tracks {
            if custom.category == TrackCategory::Draft && !self.deleted_presets.iter().any(|d| d == &custom.id) {
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

    fn track_choice_from_def(def: &TrackDefinition, mod_id: &str) -> TrackChoice {
        match def.id {
            "classic_grand_prix" => TrackChoice::ClassicGrandPrix,
            "oval_speedway" => TrackChoice::OvalSpeedway,
            "drift_park" => TrackChoice::DriftPark,
            "kart_arena" => TrackChoice::KartArena,
            "ramp_raceway" => TrackChoice::RampRaceway,
            "oasis_rally" => TrackChoice::OasisRally,
            "outlaw_pass" => TrackChoice::OutlawPass,
            other => TrackChoice::Custom {
                id: other.to_string(),
                title: def.title.to_string(),
                description: def.description.to_string(),
                path: format!("{}/{}", mod_id, other),
            },
        }
    }

    /// Returns all circuits for a specific registered game module or draft collection.
    pub fn module_catalog_tracks(&self, module_id: &str) -> Vec<TrackChoice> {
        if module_id == "drafts" {
            return self.draft_track_choices();
        }

        let draft_ids: std::collections::HashSet<&str> = self
            .custom_tracks
            .iter()
            .filter(|t| t.category == TrackCategory::Draft)
            .map(|t| t.id.as_str())
            .collect();

        let raw_list = match module_id {
            "f1" => {
                let f1_module = F1GameModule::new();
                let mut list: Vec<TrackChoice> = f1_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "f1"))
                    .collect();
                for custom in self.module_custom_tracks("f1") {
                    if let Some(pos) = list.iter().position(|c| c.track_id() == custom.track_id()) {
                        list[pos] = custom;
                    } else {
                        list.push(custom);
                    }
                }
                list
            }
            "rally" => {
                let rally_module = RallyGameModule::new();
                let mut list: Vec<TrackChoice> = rally_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "rally"))
                    .collect();
                for custom in self.module_custom_tracks("rally") {
                    if let Some(pos) = list.iter().position(|c| c.track_id() == custom.track_id()) {
                        list[pos] = custom;
                    } else {
                        list.push(custom);
                    }
                }
                list
            }
            "kart" => {
                let kart_module = KartGameModule::new();
                let mut list: Vec<TrackChoice> = kart_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "kart"))
                    .collect();
                for custom in self.module_custom_tracks("kart") {
                    if let Some(pos) = list.iter().position(|c| c.track_id() == custom.track_id()) {
                        list[pos] = custom;
                    } else {
                        list.push(custom);
                    }
                }
                list
            }
            "classic" => {
                let classic_module = ClassicGameModule::new();
                let mut list: Vec<TrackChoice> = classic_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "classic"))
                    .collect();
                for custom in self.module_custom_tracks("classic") {
                    if let Some(pos) = list.iter().position(|c| c.track_id() == custom.track_id()) {
                        list[pos] = custom;
                    } else {
                        list.push(custom);
                    }
                }
                list
            }
            _ => {
                // "all"
                let classic_module = ClassicGameModule::new();
                let f1_module = F1GameModule::new();
                let rally_module = RallyGameModule::new();
                let kart_module = KartGameModule::new();

                let mut list: Vec<TrackChoice> = Vec::new();
                let mut seen_ids = std::collections::HashSet::new();

                for def in classic_module.tracks() {
                    if seen_ids.insert(def.id) {
                        list.push(Self::track_choice_from_def(&def, "classic"));
                    }
                }
                for def in f1_module.tracks() {
                    if seen_ids.insert(def.id) {
                        list.push(Self::track_choice_from_def(&def, "f1"));
                    }
                }
                for def in rally_module.tracks() {
                    if seen_ids.insert(def.id) {
                        list.push(Self::track_choice_from_def(&def, "rally"));
                    }
                }
                for def in kart_module.tracks() {
                    if seen_ids.insert(def.id) {
                        list.push(Self::track_choice_from_def(&def, "kart"));
                    }
                }

                for custom in &self.custom_tracks {
                    if custom.category == TrackCategory::Main {
                        let custom_choice = TrackChoice::Custom {
                            id: custom.id.clone(),
                            title: custom.title.clone(),
                            description: custom.description.clone(),
                            path: custom.file_path.clone(),
                        };
                        if let Some(pos) = list.iter().position(|c| c.track_id() == custom.id) {
                            list[pos] = custom_choice;
                        } else {
                            list.push(custom_choice);
                        }
                    }
                }
                list
            }
        };

        raw_list
            .into_iter()
            .filter(|c| {
                let tid = c.track_id();
                !self.is_preset_deleted_for_module(tid, module_id) && !draft_ids.contains(tid)
            })
            .collect()
    }

    /// Checks if a preset or custom circuit is marked as deleted in the target module or globally.
    pub fn is_preset_deleted_for_module(&self, id: &str, module_id: &str) -> bool {
        let scoped_key = format!("{}:{}", module_id, id);
        if self.deleted_presets.iter().any(|d| d == &scoped_key) {
            return true;
        }

        // Global un-prefixed deletion (legacy or deleted from All Modules)
        if self.deleted_presets.iter().any(|d| d == id) {
            return true;
        }

        if module_id == "all" {
            let has_scoped_deletion = self.deleted_presets.iter().any(|d| d.ends_with(&format!(":{}", id)));
            if has_scoped_deletion {
                let active_in_any = ["classic", "f1", "rally", "kart"].iter().any(|m| {
                    let m_scoped = format!("{}:{}", m, id);
                    if self.deleted_presets.iter().any(|d| d == &m_scoped) {
                        return false;
                    }
                    let in_custom = self.custom_tracks.iter().any(|t| {
                        t.id == id && t.category == TrackCategory::Main && t.belongs_to_module(m)
                    });
                    let in_presets = Self::preset_module(id) == Some(*m);
                    in_custom || in_presets
                });
                return !active_in_any;
            }
        }

        false
    }

    /// Resolves a track instance by slug ID from disk files or built-in presets.
    pub fn load_track_by_slug(&self, slug: &str) -> Result<Track, String> {
        let choice = match slug {
            "classic_grand_prix" => TrackChoice::ClassicGrandPrix,
            "oval_speedway" => TrackChoice::OvalSpeedway,
            "drift_park" => TrackChoice::DriftPark,
            "kart_arena" => TrackChoice::KartArena,
            "ramp_raceway" => TrackChoice::RampRaceway,
            "oasis_rally" => TrackChoice::OasisRally,
            "outlaw_pass" => TrackChoice::OutlawPass,
            custom_id => {
                let path = self.track_path_for_slug(custom_id).to_string_lossy().to_string();
                TrackChoice::Custom {
                    id: custom_id.to_string(),
                    title: custom_id.to_string(),
                    description: String::new(),
                    path,
                }
            }
        };
        self.load_track(&choice)
    }

    /// Loads a `Track` from a `TrackChoice`.
    pub fn load_track(&self, choice: &TrackChoice) -> Result<Track, String> {
        match choice {
            TrackChoice::ClassicGrandPrix => {
                let p = self.track_path_for_slug("classic_grand_prix");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(classic_grand_prix()))
                } else {
                    Ok(classic_grand_prix())
                }
            }
            TrackChoice::OvalSpeedway => {
                let p = self.track_path_for_slug("oval_speedway");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(oval_speedway()))
                } else {
                    Ok(oval_speedway())
                }
            }
            TrackChoice::DriftPark => {
                let p = self.track_path_for_slug("drift_park");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(drift_park()))
                } else {
                    Ok(drift_park())
                }
            }
            TrackChoice::KartArena => {
                let p = self.track_path_for_slug("kart_arena");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(kart_arena()))
                } else {
                    Ok(kart_arena())
                }
            }
            TrackChoice::RampRaceway => {
                let p = self.track_path_for_slug("ramp_raceway");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(ramp_raceway()))
                } else {
                    Ok(ramp_raceway())
                }
            }
            TrackChoice::OasisRally => {
                let p = self.track_path_for_slug("oasis_rally");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(oasis_rally()))
                } else {
                    Ok(oasis_rally())
                }
            }
            TrackChoice::OutlawPass => {
                let p = self.track_path_for_slug("outlaw_pass");
                if p.exists() {
                    Track::load_from_file(&p).or_else(|_| Ok(outlaw_pass()))
                } else {
                    Ok(outlaw_pass())
                }
            }
            TrackChoice::Custom { id, path, .. } => {
                let file_path = Path::new(path);
                if file_path.exists() {
                    if let Ok(t) = Track::load_from_file(file_path) {
                        return Ok(t);
                    }
                }
                let alt_path = self.track_path_for_slug(id);
                if alt_path.exists() {
                    if let Ok(t) = Track::load_from_file(&alt_path) {
                        return Ok(t);
                    }
                }
                match id.as_str() {
                    "dirty_oval_speedway" => Ok(tdrace_core::track::presets::dirty_oval_speedway()),
                    "figure_eight" => Ok(tdrace_core::track::presets::figure_eight()),
                    "monza" => Ok(crate::module::f1::F1GameModule::track_monza()),
                    "spa" => Ok(crate::module::f1::F1GameModule::track_spa()),
                    "silverstone" => Ok(crate::module::f1::F1GameModule::track_silverstone()),
                    "monaco" => Ok(crate::module::f1::F1GameModule::track_monaco()),
                    "suzuka" => Ok(crate::module::f1::F1GameModule::track_suzuka()),
                    "interlagos" => Ok(crate::module::f1::F1GameModule::track_interlagos()),
                    "montreal" => Ok(crate::module::f1::F1GameModule::track_montreal()),
                    "red_bull_ring" => Ok(crate::module::f1::F1GameModule::track_red_bull_ring()),
                    "catalunya" => Ok(crate::module::f1::F1GameModule::track_catalunya()),
                    "zandvoort" => Ok(crate::module::f1::F1GameModule::track_zandvoort()),
                    "bahrain" => Ok(crate::module::f1::F1GameModule::track_bahrain()),
                    "marina_bay" => Ok(crate::module::f1::F1GameModule::track_marina_bay()),
                    "cota" => Ok(crate::module::f1::F1GameModule::track_cota()),
                    "sahara" | "sahara_dunes" => Ok(tdrace_core::track::presets::sahara_dunes()),
                    "dirt_figure_eight" | "dirt_eight" => Ok(tdrace_core::track::presets::dirt_figure_eight()),
                    "holjes_rx" | "holjes" => Ok(tdrace_core::track::presets::holjes_rx()),
                    "lydden_hill" | "lydden" => Ok(tdrace_core::track::presets::lydden_hill()),
                    "hell_rx" | "hell" => Ok(tdrace_core::track::presets::hell_rx()),
                    "loheac_rx" | "loheac" => Ok(tdrace_core::track::presets::loheac_rx()),
                    "lonato" => Ok(crate::module::kart::KartGameModule::track_lonato()),
                    "sarno" => Ok(crate::module::kart::KartGameModule::track_sarno()),
                    "genk" => Ok(crate::module::kart::KartGameModule::track_genk()),
                    "pfi" => Ok(crate::module::kart::KartGameModule::track_pfi()),
                    "zuera" => Ok(crate::module::kart::KartGameModule::track_zuera()),
                    "le_mans_kart" => Ok(crate::module::kart::KartGameModule::track_le_mans()),
                    "portimao_kart" => Ok(crate::module::kart::KartGameModule::track_portimao()),
                    "franciacorta" => Ok(crate::module::kart::KartGameModule::track_franciacorta()),
                    _ => Err(format!("Track file not found: {}", path)),
                }
            }
        }
    }

    /// Returns the built-in motorsport module ID for a preset track slug, if applicable.
    pub fn preset_module(slug: &str) -> Option<&'static str> {
        match slug {
            "classic_grand_prix" | "oval_speedway" | "dirty_oval_speedway" | "figure_eight" | "dirt_figure_eight" | "dirt_eight" | "drift_park" | "ramp_raceway" => Some("classic"),
            "oasis_rally" | "outlaw_pass" | "holjes_rx" | "holjes" | "lydden_hill" | "lydden" | "hell_rx" | "hell" | "loheac_rx" | "loheac" | "sahara" | "sahara_dunes" => Some("rally"),
            "kart_arena" | "lonato" | "sarno" | "genk" | "pfi" | "zuera" | "le_mans_kart" | "portimao_kart" | "franciacorta" => Some("kart"),
            "monza" | "spa" | "silverstone" | "monaco" | "suzuka" | "interlagos" | "montreal" | "red_bull_ring" | "catalunya" | "zandvoort" | "bahrain" | "marina_bay" | "cota" => Some("f1"),
            _ => None,
        }
    }

    /// Checks if the given slug corresponds to a known preset circuit.
    pub fn is_preset_slug(slug: &str) -> bool {
        Self::preset_module(slug).is_some()
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

    /// Returns the target directory for saving tracks. In decoupled storage, all user tracks reside in tracks_dir.
    pub fn target_dir_for_track(&self, _category: TrackCategory, _module_id: Option<&str>) -> PathBuf {
        self.tracks_dir.clone()
    }

    /// Checks if a custom track file with the given slug already exists on disk in any directory.
    pub fn track_file_exists(&self, slug: &str) -> bool {
        let file_name = format!("{}.json", slug);
        self.tracks_dir.join(&file_name).exists()
            || self.tracks_dir.join("drafts").join(&file_name).exists()
            || self.tracks_dir.join("classic").join(&file_name).exists()
            || self.tracks_dir.join("f1").join(&file_name).exists()
            || self.tracks_dir.join("rally").join(&file_name).exists()
            || self.tracks_dir.join("kart").join(&file_name).exists()
    }

    /// Resolves the destination path for a given slug, checking existing files first.
    pub fn track_path_for_slug(&self, slug: &str) -> PathBuf {
        let file_name = format!("{}.json", slug);
        let flat_path = self.tracks_dir.join(&file_name);
        if flat_path.exists() {
            return flat_path;
        }
        let candidates = [
            self.tracks_dir.join("classic").join(&file_name),
            self.tracks_dir.join("f1").join(&file_name),
            self.tracks_dir.join("rally").join(&file_name),
            self.tracks_dir.join("kart").join(&file_name),
            self.tracks_dir.join("drafts").join(&file_name),
        ];
        for cand in &candidates {
            if cand.exists() {
                return cand.clone();
            }
        }
        flat_path
    }

    /// Saves a track to disk with overwrite control in the user circuits storage.
    /// If `overwrite` is false and a file with `slug` exists, appends numbers (`_1`, `_2`, etc.) to find an unused file path.
    pub fn save_custom_track_with_options(
        &mut self,
        track: &Track,
        slug: Option<&str>,
        overwrite: bool,
    ) -> Result<String, String> {
        let mut track_to_save = track.clone();

        let base_slug = if let Some(s) = slug {
            Self::sanitize_slug(s)
        } else {
            Self::sanitize_slug(&track_to_save.name)
        };

        // If file already exists and was Main category, keep its category and module when overwriting.
        // If track is a known preset or Main track with module being overwritten, preserve Main category and module.
        // Otherwise, newly created custom tracks land in Drafts by default.
        let existing_path = self.track_path_for_slug(&base_slug);
        if overwrite && existing_path.exists() {
            if let Ok(existing) = Track::load_from_file(&existing_path) {
                track_to_save.category = existing.category;
                track_to_save.module_id = existing.module_id;
                track_to_save.modules = existing.modules;
            }
        } else if overwrite && Self::is_preset_slug(&base_slug) {
            track_to_save.category = TrackCategory::Main;
            let mod_id = Self::preset_module(&base_slug)
                .map(|s| s.to_string())
                .or_else(|| track_to_save.module_id.clone())
                .unwrap_or_else(|| "classic".to_string());
            track_to_save.module_id = Some(mod_id.clone());
            if track_to_save.modules.is_empty() {
                track_to_save.modules = vec![mod_id];
            }
        } else {
            track_to_save.category = TrackCategory::Draft;
            track_to_save.module_id = None;
            track_to_save.modules.clear();
        }

        let _ = fs::create_dir_all(&self.tracks_dir);

        let file_slug = if overwrite {
            base_slug
        } else {
            let mut candidate = base_slug.clone();
            let mut counter = 1;
            while self.track_file_exists(&candidate) {
                candidate = format!("{}_{}", base_slug, counter);
                counter += 1;
            }
            candidate
        };

        let target_path = if overwrite && existing_path.exists() {
            existing_path
        } else {
            self.tracks_dir.join(format!("{}.json", file_slug))
        };

        track_to_save
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save custom track: {}", e))?;

        let _ = self.scan_custom_tracks();
        Ok(target_path.to_string_lossy().to_string())
    }

    /// Saves a track to disk in the tracks directory. Overwrites by default.
    pub fn save_custom_track(&mut self, track: &Track, slug: Option<&str>) -> Result<String, String> {
        self.save_custom_track_with_options(track, slug, true)
    }

    /// Returns custom tracks promoted to a specific module.
    pub fn module_custom_tracks(&self, module_id: &str) -> Vec<TrackChoice> {
        let mut choices = Vec::new();
        for custom in &self.custom_tracks {
            if custom.category == TrackCategory::Main && custom.belongs_to_module(module_id) {
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

    /// Returns the list of motorsport module IDs ("classic", "rally", "kart", "f1") where a track is currently promoted / available.
    pub fn track_promoted_modules(&self, id: &str) -> Vec<String> {
        let mut mods = Vec::new();
        for mod_id in ["classic", "rally", "kart", "f1"] {
            if self.is_track_in_module(id, mod_id) {
                mods.push(mod_id.to_string());
            }
        }
        mods
    }

    /// Checks if a track is promoted / available in a specific module.
    pub fn is_track_in_module(&self, id: &str, module_id: &str) -> bool {
        self.module_catalog_tracks(module_id)
            .iter()
            .any(|c| c.track_id() == id)
    }

    /// Promotes a track from Draft to Main category (Approved circuit) assigned to multiple modules.
    /// Updates the single track file's metadata (`category = Main`, `modules = module_ids`).
    pub fn promote_track_to_modules(&mut self, id: &str, module_ids: &[&str]) -> Result<(), String> {
        if module_ids.is_empty() {
            return self.demote_track(id);
        }

        let (mut track, target_path) = if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let path = PathBuf::from(&path_str);
            let t = Track::load_from_file(&path)
                .map_err(|e| format!("Failed to load track to promote: {}", e))?;
            (t, path)
        } else {
            let t = self.load_track_by_slug(id).map_err(|e| format!("Track '{}' not found: {}", id, e))?;
            let path = self.track_path_for_slug(id);
            (t, path)
        };

        track.category = TrackCategory::Main;
        track.modules = module_ids.iter().map(|s| s.to_string()).collect();
        track.module_id = module_ids.first().map(|s| s.to_string());

        track
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save promoted track: {}", e))?;

        // Clean up any duplicate legacy files across subdirectories if they differ from target_path
        let file_name = format!("{}.json", id);
        for sub in &["classic", "f1", "rally", "kart", "drafts"] {
            let legacy_p = self.tracks_dir.join(sub).join(&file_name);
            if legacy_p.exists() && legacy_p != target_path {
                let _ = fs::remove_file(legacy_p);
            }
        }

        self.deleted_presets.retain(|d| d != id && !module_ids.iter().any(|m| d == &format!("{}:{}", m, id)));
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        Ok(())
    }

    /// Promotes a track from Draft to Main category (Approved circuit) assigned to a specific module.
    pub fn promote_track_to_module(&mut self, id: &str, module_id: &str) -> Result<(), String> {
        self.promote_track_to_modules(id, &[module_id])
    }

    /// Promotes a track from Draft to Main category with default "classic" module.
    pub fn promote_track(&mut self, id: &str) -> Result<(), String> {
        self.promote_track_to_modules(id, &["classic"])
    }

    /// Demotes a track from Main category back to Draft / Testing, updating the single file's metadata.
    pub fn demote_track(&mut self, id: &str) -> Result<(), String> {
        let (mut track, target_path) = if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let path = PathBuf::from(&path_str);
            let t = Track::load_from_file(&path)
                .map_err(|e| format!("Failed to load track to demote: {}", e))?;
            (t, path)
        } else {
            let t = self.load_track_by_slug(id).map_err(|e| format!("Track '{}' not found: {}", id, e))?;
            let path = self.track_path_for_slug(id);
            (t, path)
        };

        track.category = TrackCategory::Draft;
        track.module_id = None;
        track.modules.clear();

        track
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save demoted track: {}", e))?;

        // Clean up legacy files across subdirectories
        let file_name = format!("{}.json", id);
        for sub in &["classic", "f1", "rally", "kart", "drafts"] {
            let legacy_p = self.tracks_dir.join(sub).join(&file_name);
            if legacy_p.exists() && legacy_p != target_path {
                let _ = fs::remove_file(legacy_p);
            }
        }

        self.deleted_presets.retain(|d| d != id && !d.ends_with(&format!(":{}", id)));
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        Ok(())
    }

    /// Updates the display name and description of a custom track and writes changes to disk.
    pub fn update_track_metadata(&mut self, id: &str, new_title: String, new_description: String) -> Result<(), String> {
        let (mut track, target_path) = if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let path = PathBuf::from(&path_str);
            let t = Track::load_from_file(&path)
                .map_err(|e| format!("Failed to load track for metadata update: {}", e))?;
            (t, path)
        } else {
            let t = self.load_track_by_slug(id).map_err(|e| format!("Track '{}' not found: {}", id, e))?;
            let path = self.track_path_for_slug(id);
            (t, path)
        };

        track.name = new_title;
        track.description = new_description;

        track
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save updated track metadata: {}", e))?;

        let _ = self.scan_custom_tracks();
        Ok(())
    }

    /// Creates a new starter draft circuit in the tracks directory.
    pub fn create_new_draft_track(
        &mut self,
        name: &str,
        description: &str,
    ) -> Result<String, String> {
        self.create_new_draft_track_with_template(
            name,
            description,
            "classic",
            tdrace_core::track::presets::TrackShape::Oval,
            tdrace_core::track::presets::RaceDirection::Right,
        )
    }

    /// Creates a new starter draft circuit using a prototypical template for the specified module.
    pub fn create_new_draft_track_with_template(
        &mut self,
        name: &str,
        description: &str,
        module_id: &str,
        shape: tdrace_core::track::presets::TrackShape,
        direction: tdrace_core::track::presets::RaceDirection,
    ) -> Result<String, String> {
        let mut track = tdrace_core::track::presets::create_prototypical_track(module_id, shape, direction);
        track.name = name.to_string();
        track.description = description.to_string();
        track.category = TrackCategory::Draft;

        self.save_custom_track(&track, None)
    }

    /// Clones an existing circuit (preset or custom), creating an exact duplicate in the user circuits storage.
    /// Appends "(clone)" to the track name, sets category to Draft, and writes to `<slug>_clone.json`.
    /// Returns the cloned Track instance and its saved file path.
    pub fn clone_track(&mut self, choice: &TrackChoice) -> Result<(Track, String), String> {
        let original_track = self.load_track(choice)?;
        let mut cloned_track = original_track.clone();

        let name_base = if !original_track.name.trim().is_empty() {
            original_track.name.trim()
        } else {
            choice.title().trim()
        };
        cloned_track.name = format!("{} (clone)", name_base);
        cloned_track.category = TrackCategory::Draft;
        cloned_track.module_id = None;
        cloned_track.modules.clear();

        let base_slug = format!("{}_clone", Self::sanitize_slug(choice.track_id()));
        let _ = fs::create_dir_all(&self.tracks_dir);

        let mut file_slug = base_slug.clone();
        let mut counter = 1;
        while self.track_file_exists(&file_slug) {
            file_slug = format!("{}_{}", base_slug, counter);
            counter += 1;
        }

        let file_name = format!("{}.json", file_slug);
        let path = self.tracks_dir.join(file_name);

        cloned_track
            .save_to_file(&path)
            .map_err(|e| format!("Failed to save cloned track: {}", e))?;

        self.deleted_presets.retain(|d| d != &file_slug);
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        Ok((cloned_track, path.to_string_lossy().to_string()))
    }

    /// Clones an existing circuit identified by its slug into the drafts group.
    pub fn clone_track_by_slug(&mut self, slug: &str) -> Result<(Track, String), String> {
        let track = self.load_track_by_slug(slug)?;
        let choice = if let Some(custom) = self.custom_tracks.iter().find(|t| t.id == slug) {
            TrackChoice::Custom {
                id: custom.id.clone(),
                title: custom.title.clone(),
                description: custom.description.clone(),
                path: custom.file_path.clone(),
            }
        } else {
            match slug {
                "classic_grand_prix" => TrackChoice::ClassicGrandPrix,
                "oval_speedway" => TrackChoice::OvalSpeedway,
                "drift_park" => TrackChoice::DriftPark,
                "kart_arena" => TrackChoice::KartArena,
                "ramp_raceway" => TrackChoice::RampRaceway,
                "oasis_rally" => TrackChoice::OasisRally,
                "outlaw_pass" => TrackChoice::OutlawPass,
                custom_id => {
                    let path = self.track_path_for_slug(custom_id).to_string_lossy().to_string();
                    TrackChoice::Custom {
                        id: custom_id.to_string(),
                        title: track.name.clone(),
                        description: track.description.clone(),
                        path,
                    }
                }
            }
        };
        self.clone_track(&choice)
    }

    /// Deletes a track specifically from the active module (or drafts).
    /// If `module_id` is None, deletes the track globally across all modules.
    pub fn delete_track_from_module(&mut self, id: &str, module_id: Option<&str>) -> Result<bool, String> {
        let mut deleted_any = false;

        match module_id {
            Some("drafts") => {
                let file_name = format!("{}.json", id);
                let candidates = [
                    self.tracks_dir.join(&file_name),
                    self.tracks_dir.join("drafts").join(&file_name),
                ];
                for cand in &candidates {
                    if cand.exists() {
                        let _ = fs::remove_file(cand);
                        deleted_any = true;
                    }
                }
                let tdtrack_candidates = [
                    self.tracks_dir.join(format!("{}.tdtrack", id)),
                    self.tracks_dir.join("drafts").join(format!("{}.tdtrack", id)),
                ];
                for cand in &tdtrack_candidates {
                    if cand.exists() {
                        let _ = fs::remove_file(cand);
                        deleted_any = true;
                    }
                }
                self.deleted_presets.retain(|d| d != id && d != &format!("drafts:{}", id));
                self.save_deleted_presets();
            }
            Some(mod_id) => {
                let file_name = format!("{}.json", id);
                // 1. If custom track belongs to this module, remove mod_id from track.modules
                for custom in &self.custom_tracks {
                    if custom.id == id && custom.belongs_to_module(mod_id) {
                        let path = PathBuf::from(&custom.file_path);
                        if path.exists() {
                            if let Ok(mut track) = Track::load_from_file(&path) {
                                track.modules.retain(|m| !m.eq_ignore_ascii_case(mod_id));
                                if track.module_id.as_deref().map(|m| m.eq_ignore_ascii_case(mod_id)).unwrap_or(false) {
                                    track.module_id = track.modules.first().cloned();
                                }
                                if track.modules.is_empty() {
                                    track.category = TrackCategory::Draft;
                                    track.module_id = None;
                                }
                                let _ = track.save_to_file(&path);
                                deleted_any = true;
                            }
                        }
                    }
                }

                // If legacy file in <mod_id>/<id>.json exists, remove it
                let mod_path = self.tracks_dir.join(mod_id).join(&file_name);
                if mod_path.exists() {
                    let _ = fs::remove_file(&mod_path);
                    deleted_any = true;
                }
                let mod_tdtrack = self.tracks_dir.join(mod_id).join(format!("{}.tdtrack", id));
                if mod_tdtrack.exists() {
                    let _ = fs::remove_file(&mod_tdtrack);
                    deleted_any = true;
                }

                // 2. Mark preset/track deleted specifically for this module
                let scoped_key = format!("{}:{}", mod_id, id);
                if !self.deleted_presets.iter().any(|d| d == &scoped_key) {
                    self.deleted_presets.push(scoped_key.clone());
                    self.save_deleted_presets();
                }

                self.deleted_presets.retain(|d| d != id);
                self.save_deleted_presets();
            }
            None => {
                // Delete across all modules
                let file_name = format!("{}.json", id);
                let candidates = [
                    self.tracks_dir.join(&file_name),
                    self.tracks_dir.join("drafts").join(&file_name),
                    self.tracks_dir.join("classic").join(&file_name),
                    self.tracks_dir.join("f1").join(&file_name),
                    self.tracks_dir.join("rally").join(&file_name),
                    self.tracks_dir.join("kart").join(&file_name),
                ];
                for cand in &candidates {
                    if cand.exists() {
                        let _ = fs::remove_file(cand);
                        deleted_any = true;
                    }
                }
                let tdtrack = self.tracks_dir.join(format!("{}.tdtrack", id));
                if tdtrack.exists() {
                    let _ = fs::remove_file(&tdtrack);
                    deleted_any = true;
                }

                self.deleted_presets.retain(|d| {
                    if let Some((_, slug)) = d.split_once(':') {
                        slug != id
                    } else {
                        d != id
                    }
                });
                self.deleted_presets.push(id.to_string());
                self.save_deleted_presets();
            }
        }

        let _ = self.scan_custom_tracks();
        let was_deleted = deleted_any || match module_id {
            Some(m) => self.deleted_presets.iter().any(|d| d == &format!("{}:{}", m, id) || d == id),
            None => self.deleted_presets.iter().any(|d| d == id),
        };
        Ok(was_deleted)
    }

    /// Deletes a custom or preset track file from disk and records it in deleted presets list globally.
    pub fn delete_custom_track(&mut self, id: &str) -> Result<bool, String> {
        self.delete_track_from_module(id, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_manager_presets_and_custom_save() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_test_tracks_mgr_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = TrackManager::new(&temp_dir);
        let choices = manager.all_track_choices();
        assert_eq!(choices.len(), 36); // 10 classic + 13 f1 + 5 rally unique + 8 famous kart

        let mut gp = classic_grand_prix();
        gp.name = "My Custom GP".to_string();
        gp.description = "A custom testing GP".to_string();
        gp.category = TrackCategory::Draft;

        let saved_path = manager
            .save_custom_track(&gp, Some("test_custom_gp"))
            .expect("Must save custom track");
        assert!(Path::new(&saved_path).exists());

        // Since gp was saved as Draft, main choices is still 36, but draft choices has 1
        assert_eq!(manager.main_track_choices().len(), 36);
        assert_eq!(manager.draft_track_choices().len(), 1);

        let draft_choice = &manager.draft_track_choices()[0];
        assert_eq!(draft_choice.title(), "My Custom GP");
        assert_eq!(draft_choice.description(), "A custom testing GP");

        // Promote track to Main
        manager.promote_track("test_custom_gp").expect("Must promote");
        assert_eq!(manager.main_track_choices().len(), 37);
        assert_eq!(manager.draft_track_choices().len(), 0);

        // Edit metadata
        manager
            .update_track_metadata(
                "test_custom_gp",
                "Renamed Grand Prix".to_string(),
                "Updated description text".to_string(),
            )
            .expect("Must update metadata");
        let loaded = manager.load_track(&manager.main_track_choices()[36]).expect("Must load");
        assert_eq!(loaded.name, "Renamed Grand Prix");
        assert_eq!(loaded.description, "Updated description text");

        // Demote back to draft
        manager.demote_track("test_custom_gp").expect("Must demote");
        assert_eq!(manager.main_track_choices().len(), 36);
        assert_eq!(manager.draft_track_choices().len(), 1);

        // Clean up
        assert!(manager.delete_custom_track("test_custom_gp").unwrap());
        assert_eq!(manager.main_track_choices().len(), 36);
        assert_eq!(manager.draft_track_choices().len(), 0);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_track_manager_overwrite_options() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_test_tracks_overwrite_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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

    #[test]
    fn test_module_catalog_tracks() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_test_catalog_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut manager = TrackManager::new(temp_dir.clone());

        // Classic tracks
        let classic_tracks = manager.module_catalog_tracks("classic");
        assert_eq!(classic_tracks.len(), 10);

        // F1 tracks
        let f1_tracks = manager.module_catalog_tracks("f1");
        assert_eq!(f1_tracks.len(), 14);
        assert!(f1_tracks.iter().any(|t| t.title().contains("Monza")));
        assert!(f1_tracks.iter().any(|t| t.title().contains("Spa")));
        assert!(f1_tracks.iter().any(|t| t.title().contains("Silverstone")));

        // Rally tracks
        let rally_tracks = manager.module_catalog_tracks("rally");
        assert_eq!(rally_tracks.len(), 7);
        assert!(rally_tracks.iter().any(|t| t.title().contains("Sahara")));
        assert!(rally_tracks.iter().any(|t| t.title().contains("Höljes")));

        // Kart tracks
        let kart_tracks = manager.module_catalog_tracks("kart");
        assert_eq!(kart_tracks.len(), 10);
        assert!(kart_tracks.iter().any(|t| t.title().contains("Lonato")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Sarno")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Genk")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("PFI") || t.title().contains("PF International")));

        // All tracks
        let all_tracks = manager.module_catalog_tracks("all");
        assert_eq!(all_tracks.len(), 36);

        // Save a draft
        let mut draft = classic_grand_prix();
        draft.name = "My Draft Circuit".to_string();
        let _ = manager.save_custom_track_with_options(&draft, Some("my_draft"), false);

        let drafts = manager.module_catalog_tracks("drafts");
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].title(), "My Draft Circuit");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clone_track_presets_and_drafts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_test_clone_mgr_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        let mut manager = TrackManager::new(&temp_dir);

        // 1. Clone a preset track (ClassicGrandPrix)
        let (cloned_gp, path_gp) = manager
            .clone_track(&TrackChoice::ClassicGrandPrix)
            .expect("Must clone ClassicGrandPrix");

        assert_eq!(cloned_gp.name, "Classic Grand Prix (clone)");
        assert_eq!(cloned_gp.category, TrackCategory::Draft);
        assert!(cloned_gp.module_id.is_none());
        assert!(cloned_gp.modules.is_empty());
        assert!(Path::new(&path_gp).exists());

        // Cloned track must appear in drafts, and main count stays 36
        assert_eq!(manager.draft_track_choices().len(), 1);
        assert_eq!(manager.main_track_choices().len(), 36);
        assert_eq!(manager.draft_track_choices()[0].title(), "Classic Grand Prix (clone)");

        // 2. Clone a module preset by slug
        let (cloned_monza, path_monza) = manager
            .clone_track_by_slug("monza")
            .expect("Must clone Monza by slug");

        assert_eq!(cloned_monza.name, "Monza Autodromo Nazionale (clone)");
        assert_eq!(cloned_monza.category, TrackCategory::Draft);
        assert!(cloned_monza.module_id.is_none());
        assert!(cloned_monza.modules.is_empty());
        assert!(Path::new(&path_monza).exists());
        assert_eq!(manager.draft_track_choices().len(), 2);

        // 3. Clone again to verify collision handling (appends _1)
        let (cloned_monza_2, path_monza_2) = manager
            .clone_track_by_slug("monza")
            .expect("Must clone Monza a second time");

        assert_eq!(cloned_monza_2.name, "Monza Autodromo Nazionale (clone)");
        assert!(path_monza_2.ends_with("monza_clone_1.json"));
        assert!(Path::new(&path_monza_2).exists());
        assert_eq!(manager.draft_track_choices().len(), 3);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
