use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::{Track, TrackCategory};

use crate::module::classic::ClassicGameModule;
use crate::module::extreme_offroad::ExtremeOffRoadModule;
use crate::module::gt::GtWorldChallengeModule;
use crate::module::kart::KartGameModule;
use crate::module::nascar::NascarGameModule;
use crate::module::rally::RallyGameModule;
use crate::module::{GameModule, TrackDefinition};
use crate::ui::menu::TrackChoice;

/// Filter for selecting tracks by motorsport module or drafts category in the Track Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleFilter {
    #[default]
    Classic,
    Rally,
    Kart,
    Gt,
    Nascar,
    ExtremeOffRoad,
    Drafts,
}

impl ModuleFilter {
    pub const ALL: [Self; 7] = [
        Self::Classic,
        Self::Rally,
        Self::Kart,
        Self::Gt,
        Self::Nascar,
        Self::ExtremeOffRoad,
        Self::Drafts,
    ];

    pub fn id(&self) -> Option<&'static str> {
        match self {
            Self::Classic => Some("classic"),
            Self::Rally => Some("rally"),
            Self::Kart => Some("kart"),
            Self::Gt => Some("gt"),
            Self::Nascar => Some("nascar"),
            Self::ExtremeOffRoad => Some("extreme_offroad"),
            Self::Drafts => Some("drafts"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Classic => "CLASSIC",
            Self::Rally => "RALLYCROSS",
            Self::Kart => "KARTING",
            Self::Gt => "GT WORLD CHALLENGE",
            Self::Nascar => "NASCAR",
            Self::ExtremeOffRoad => "EXTREME OFF-ROAD",
            Self::Drafts => "DRAFTS",
        }
    }

    /// Compact label for narrow tabs.
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Gt => "GT",
            Self::ExtremeOffRoad => "OFF-ROAD",
            other => other.label(),
        }
    }

    pub fn shortcut_number(&self) -> u8 {
        match self {
            Self::Classic => 1,
            Self::Rally => 2,
            Self::Kart => 3,
            Self::Gt => 4,
            Self::Nascar => 5,
            Self::ExtremeOffRoad => 6,
            Self::Drafts => 9,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Classic => Self::Rally,
            Self::Rally => Self::Kart,
            Self::Kart => Self::Gt,
            Self::Gt => Self::Nascar,
            Self::Nascar => Self::ExtremeOffRoad,
            Self::ExtremeOffRoad => Self::Drafts,
            Self::Drafts => Self::Classic,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Classic => Self::Drafts,
            Self::Rally => Self::Classic,
            Self::Kart => Self::Rally,
            Self::Gt => Self::Kart,
            Self::Nascar => Self::Gt,
            Self::ExtremeOffRoad => Self::Nascar,
            Self::Drafts => Self::ExtremeOffRoad,
        }
    }

    pub fn for_module(mod_id: &str) -> Self {
        match mod_id {
            "gt" | "gt_challenge" => Self::Gt,
            "rally" => Self::Rally,
            "kart" => Self::Kart,
            "nascar" => Self::Nascar,
            "extreme_offroad" | "offroad" => Self::ExtremeOffRoad,
            "drafts" => Self::Drafts,
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
        if mod_id.eq_ignore_ascii_case("all") {
            return true;
        }
        let target_ids = if mod_id.eq_ignore_ascii_case("gt") {
            vec!["gt", "gt_challenge"]
        } else {
            vec![mod_id]
        };
        if self.modules.iter().any(|m| target_ids.iter().any(|&tid| m.eq_ignore_ascii_case(tid))) {
            return true;
        }
        if let Some(ref m) = self.module_id {
            if target_ids.iter().any(|&tid| m.eq_ignore_ascii_case(tid)) {
                return true;
            }
        }
        // If no explicit module restriction is specified, the custom circuit is open to all motorsport modules.
        self.modules.is_empty() && self.module_id.is_none()
    }

    pub fn module_name(&self) -> &'static str {
        if let Some(ref m) = self.module_id {
            match m.to_lowercase().as_str() {
                "gt" | "gt_challenge" => "GT World Challenge",
                "rally" => "Rallycross",
                "kart" => "Karting",
                "nascar" => "NASCAR Cup",
                "extreme_offroad" | "offroad" => "Extreme Off-Road",
                _ => "Classic",
            }
        } else if self.belongs_to_module("gt") {
            "GT World Challenge"
        } else if self.belongs_to_module("rally") {
            "Rallycross"
        } else if self.belongs_to_module("kart") {
            "Karting"
        } else if self.belongs_to_module("nascar") {
            "NASCAR Cup"
        } else if self.belongs_to_module("extreme_offroad") {
            "Extreme Off-Road"
        } else {
            "Classic"
        }
    }
}

/// Logs, once per file and run, a user-folder file that has an official circuit's id.
/// Such a file is ignored: official circuits come only from the official catalog (spec 042).
fn log_ignored_official_shadow(path: &Path) {
    static LOGGED: std::sync::Mutex<Option<std::collections::HashSet<PathBuf>>> = std::sync::Mutex::new(None);
    if let Ok(mut guard) = LOGGED.lock() {
        if guard.get_or_insert_with(Default::default).insert(path.to_path_buf()) {
            eprintln!(
                "user track {} has the id of an official circuit and is ignored; rename it to keep it as a custom circuit",
                path.display()
            );
        }
    }
}

/// Manages discovery, loading, saving, and cataloging of custom and preset tracks.
#[derive(Debug, Clone)]
pub struct TrackManager {
    pub tracks_dir: PathBuf,
    pub custom_tracks: Vec<CustomTrackInfo>,
    pub deleted_presets: Vec<String>,
    pub preset_order: HashMap<String, Vec<String>>,
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
        let preset_order = Self::load_preset_order(&dir);
        let mut manager = Self {
            tracks_dir: dir,
            custom_tracks: Vec::new(),
            deleted_presets,
            preset_order,
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

    fn load_preset_order(tracks_dir: &Path) -> HashMap<String, Vec<String>> {
        let path = tracks_dir.join(".track_order.json");
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(order) = serde_json::from_str::<HashMap<String, Vec<String>>>(&data) {
                    return order;
                }
            }
        }
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir().filter(|_| crate::storage::is_dev_mode()) {
            let git_path = git_tracks_dir.join(".track_order.json");
            if git_path.exists() {
                if let Ok(data) = fs::read_to_string(&git_path) {
                    if let Ok(order) = serde_json::from_str::<HashMap<String, Vec<String>>>(&data) {
                        return order;
                    }
                }
            }
        }
        HashMap::new()
    }

    pub fn save_preset_order(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.preset_order) {
            let path = self.tracks_dir.join(".track_order.json");
            let _ = fs::write(&path, &data);
            if crate::storage::is_dev_mode() {
                if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
                    let git_path = git_tracks_dir.join(".track_order.json");
                    if git_path != path
                        && (std::env::var(crate::storage::ENV_GIT_TRACKS_DIR).is_ok()
                            || self.tracks_dir == crate::storage::resolve_user_tracks_dir())
                    {
                        let _ = fs::write(git_path, &data);
                    }
                }
            }
        }
    }

    /// Checks if an official preset circuit was demoted to a custom circuit.
    pub fn is_preset_demoted(&self, id: &str) -> bool {
        let demoted_marker = format!("demoted:{}", id);
        self.deleted_presets.iter().any(|d| d == &demoted_marker)
    }

    /// Checks if a preset track was marked as demoted, inspecting either the parent directory of path or user tracks dir.
    pub fn is_preset_slug_demoted_in_path(slug: &str, path: &str) -> bool {
        let marker = format!("demoted:{}", slug);
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            let del_file = parent.join(".deleted_tracks.json");
            if del_file.exists() {
                if let Ok(data) = fs::read_to_string(&del_file) {
                    if let Ok(list) = serde_json::from_str::<Vec<String>>(&data) {
                        if list.iter().any(|d| d == &marker) {
                            return true;
                        }
                    }
                }
            }
        }
        let user_del = crate::storage::resolve_user_tracks_dir().join(".deleted_tracks.json");
        if user_del.exists() {
            if let Ok(data) = fs::read_to_string(&user_del) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&data) {
                    if list.iter().any(|d| d == &marker) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Checks if a custom track is marked as deleted in the target module or globally.
    /// Note: `demoted:<id>` marks the official preset as demoted, NOT the custom track as deleted.
    pub fn is_custom_track_deleted_for_module(&self, id: &str, module_id: &str) -> bool {
        let scoped_key = format!("{}:{}", module_id, id);
        if self.deleted_presets.iter().any(|d| d == &scoped_key) {
            return true;
        }

        if self.deleted_presets.iter().any(|d| d == id) {
            return true;
        }

        if module_id == "all" {
            let has_scoped_deletion = self.deleted_presets.iter().any(|d| d.ends_with(&format!(":{}", id)));
            if has_scoped_deletion {
                let active_in_any = ["classic", "gt", "rally", "kart", "nascar", "extreme_offroad"].iter().any(|m| {
                    let m_scoped = format!("{}:{}", m, id);
                    if self.deleted_presets.iter().any(|d| d == &m_scoped) {
                        return false;
                    }
                    self.custom_tracks.iter().any(|t| {
                        t.id == id && t.category == TrackCategory::Main && t.belongs_to_module(m)
                    })
                });
                return !active_in_any;
            }
        }

        false
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
                    if subdir_name.starts_with('.') {
                        continue;
                    }
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

                        let is_demoted = self.is_preset_demoted(&stem);
                        let mut modules = track.modules.clone();
                        let mut category = track.category;
                        let mut module_id = track.module_id.clone();

                        if category != TrackCategory::Draft && !is_demoted && Self::is_preset_slug(&stem) {
                            log_ignored_official_shadow(&path);
                            continue;
                        }

                        if is_demoted {
                            category = TrackCategory::Main;
                            if stem == "sahara_dunes" || stem == "sahara" {
                                modules = vec!["rally".to_string()];
                                module_id = Some("rally".to_string());
                            } else if modules.is_empty() && module_id.is_none() {
                                if let Some(m) = Self::preset_module(&stem) {
                                    modules = vec![m.to_string()];
                                    module_id = Some(m.to_string());
                                }
                            }
                            if track.category != category || track.modules != modules || track.module_id != module_id {
                                track.category = category;
                                track.modules = modules.clone();
                                track.module_id = module_id.clone();
                                let _ = track.save_to_file(&path);
                            }
                        } else if let Some(ref dir) = subdir {
                            if modules.is_empty() && track.module_id.is_none() {
                                if dir == "drafts" {
                                    category = TrackCategory::Draft;
                                    module_id = None;
                                    modules.clear();
                                } else {
                                    category = TrackCategory::Main;
                                    module_id = Some(dir.clone());
                                    modules = vec![dir.clone()];
                                }
                            }
                        } else if category == TrackCategory::Main {
                            let mod_id = module_id.clone().unwrap_or_else(|| "classic".to_string());
                            if module_id.is_none() {
                                module_id = Some(mod_id.clone());
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
                        let module_id = if category == TrackCategory::Main && module_id.is_none() {
                            Some("classic".to_string())
                        } else {
                            module_id
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
            ModuleFilter::Classic => self.module_catalog_tracks("classic"),
            ModuleFilter::Gt => self.module_catalog_tracks("gt"),
            ModuleFilter::Rally => self.module_catalog_tracks("rally"),
            ModuleFilter::Kart => self.module_catalog_tracks("kart"),
            ModuleFilter::Nascar => self.module_catalog_tracks("nascar"),
            ModuleFilter::ExtremeOffRoad => self.module_catalog_tracks("extreme_offroad"),
            ModuleFilter::Drafts => self.draft_track_choices(),
        }
    }

    /// Returns metadata for a custom track by ID, if present.
    pub fn custom_track_info(&self, id: &str) -> Option<&CustomTrackInfo> {
        self.custom_tracks.iter().find(|t| t.id == id)
    }

    /// Returns all user-created / custom circuits.
    pub fn custom_track_choices(&self) -> Vec<TrackChoice> {
        let mut choices = Vec::new();

        for custom in &self.custom_tracks {
            let is_demoted = self.is_preset_demoted(&custom.id);
            if (!Self::is_preset_slug(&custom.id) || is_demoted)
                && !self.is_custom_track_deleted_for_module(&custom.id, "all")
            {
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
            let is_demoted = self.is_preset_demoted(&custom.id);
            if (custom.category == TrackCategory::Draft || is_demoted)
                && !self.is_custom_track_deleted_for_module(&custom.id, "all")
            {
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

    /// Normalizes a motorsport module string identifier into a canonical module key.
    pub fn normalize_module_id(module_id: &str) -> &'static str {
        match module_id.to_ascii_lowercase().as_str() {
            "gt" | "gt_challenge" => "gt",
            "rally" => "rally",
            "kart" => "kart",
            "nascar" => "nascar",
            "extreme_offroad" | "offroad" => "extreme_offroad",
            _ => "classic",
        }
    }

    fn sort_preset_choices(list: &mut Vec<TrackChoice>, order: Option<&Vec<String>>) {
        if let Some(order_ids) = order {
            if !order_ids.is_empty() {
                list.sort_by_key(|c| {
                    let tid = c.track_id();
                    order_ids
                        .iter()
                        .position(|id| id == tid || Self::canonical_preset_id(id) == Self::canonical_preset_id(tid))
                        .unwrap_or(usize::MAX)
                });
            }
        }
    }

    /// Returns official built-in preset circuits for a given module.
    pub fn preset_track_choices(&self, module_id: &str) -> Vec<TrackChoice> {
        if module_id == "all" {
            let mut list: Vec<TrackChoice> = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();
            for m in ["classic", "gt", "rally", "kart", "nascar", "extreme_offroad"] {
                for choice in self.preset_track_choices(m) {
                    if seen_ids.insert(choice.track_id().to_string()) {
                        list.push(choice);
                    }
                }
            }
            if let Some(order) = self.preset_order.get("all") {
                Self::sort_preset_choices(&mut list, Some(order));
            }
            return list;
        }

        let raw: Vec<TrackChoice> = match module_id {
            "gt" | "gt_challenge" => {
                let gt_module = GtWorldChallengeModule::new();
                gt_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "gt"))
                    .collect()
            }
            "rally" => {
                let rally_module = RallyGameModule::new();
                rally_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "rally"))
                    .collect()
            }
            "kart" => {
                let kart_module = KartGameModule::new();
                kart_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "kart"))
                    .collect()
            }
            "nascar" => {
                let nascar_module = NascarGameModule::new();
                nascar_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "nascar"))
                    .collect()
            }
            "extreme_offroad" => {
                let offroad_module = ExtremeOffRoadModule::new();
                offroad_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "extreme_offroad"))
                    .collect()
            }
            _ => {
                let classic_module = ClassicGameModule::new();
                classic_module
                    .tracks()
                    .iter()
                    .map(|def| Self::track_choice_from_def(def, "classic"))
                    .collect()
            }
        };

        let mut list = raw;

        // In dev mode, discover presets promoted to tracks/ that are not embedded yet
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir().filter(|_| crate::storage::is_dev_mode()) {
            let scan_modules: Vec<&str> = match module_id {
                "gt" | "gt_challenge" => vec!["gt"],
                _ => vec![module_id],
            };

            for mod_name in scan_modules {
                let mod_dir = git_tracks_dir.join(mod_name);
                if let Ok(entries) = fs::read_dir(&mod_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if ext.eq_ignore_ascii_case("json") {
                                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                        let canonical_stem = Self::canonical_preset_id(stem);
                                        if !list.iter().any(|c| Self::canonical_preset_id(c.track_id()) == canonical_stem) {
                                            if let Ok(t) = Track::load_from_file(&path) {
                                                list.push(TrackChoice::Custom {
                                                    id: stem.to_string(),
                                                    title: t.name,
                                                    description: t.description,
                                                    path: format!("{}/{}", mod_name, stem),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut filtered: Vec<TrackChoice> = list
            .into_iter()
            .filter(|choice| !self.is_preset_deleted_for_module(choice.track_id(), module_id))
            .collect();

        let norm_mod = Self::normalize_module_id(module_id);
        let order = self
            .preset_order
            .get(norm_mod)
            .or_else(|| self.preset_order.get(module_id));
        Self::sort_preset_choices(&mut filtered, order);

        filtered
    }

    /// Returns all available track choices appearing in main menu (Main category tracks).
    pub fn all_track_choices(&self) -> Vec<TrackChoice> {
        self.main_track_choices()
    }

    pub fn track_choice_from_def(def: &TrackDefinition, mod_id: &str) -> TrackChoice {
        match def.id {
            "classic_grand_prix" => TrackChoice::ClassicGrandPrix,
            "oval_speedway" => TrackChoice::OvalSpeedway,
            "drift_park" => TrackChoice::DriftPark,
            "kart_arena" => TrackChoice::KartArena,
            "ramp_raceway" => TrackChoice::RampRaceway,
            "classic_rallycross" => TrackChoice::ClassicRallycross,
            "oasis_rally" => TrackChoice::OasisRally,
            other => TrackChoice::Custom {
                id: other.to_string(),
                title: def.title.to_string(),
                description: def.description.to_string(),
                path: format!("{}/{}", mod_id, other),
            },
        }
    }

    /// Returns all circuits for a specific registered game module or draft collection.
    /// Presets for the module are returned first, followed by custom circuits belonging to that category.
    pub fn module_catalog_tracks(&self, module_id: &str) -> Vec<TrackChoice> {
        if module_id == "drafts" {
            return self.draft_track_choices();
        }

        let draft_ids: std::collections::HashSet<&str> = self
            .custom_tracks
            .iter()
            .filter(|t| t.category == TrackCategory::Draft && !self.is_preset_demoted(&t.id))
            .map(|t| t.id.as_str())
            .collect();

        let mut list: Vec<TrackChoice> = self.preset_track_choices(module_id);
        let custom_tracks = self.module_custom_tracks(module_id);

        for custom in custom_tracks {
            if let Some(pos) = list.iter().position(|c| c.track_id() == custom.track_id()) {
                if self.is_preset_demoted(custom.track_id()) {
                    list[pos] = custom;
                }
            } else {
                list.push(custom);
            }
        }

        list.into_iter()
            .filter(|c| {
                let tid = c.track_id();
                let is_deleted = if c.is_official_preset() {
                    self.is_preset_deleted_for_module(tid, module_id)
                } else {
                    self.is_custom_track_deleted_for_module(tid, module_id)
                };
                !is_deleted && !draft_ids.contains(tid)
            })
            .collect()
    }

    /// Checks if a preset or custom circuit is marked as deleted in the target module or globally.
    pub fn is_preset_deleted_for_module(&self, id: &str, module_id: &str) -> bool {
        let demoted_key = format!("demoted:{}", id);
        if self.deleted_presets.iter().any(|d| d == &demoted_key) {
            return true;
        }

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
                let active_in_any = ["classic", "gt", "rally", "kart", "nascar", "extreme_offroad"].iter().any(|m| {
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
            "classic_rallycross" => TrackChoice::ClassicRallycross,
            "oasis_rally" => TrackChoice::OasisRally,
            custom_id => {
                if !self.is_preset_demoted(custom_id) {
                    if let Some(result) = crate::tracks::official::load(custom_id, None) {
                        return result;
                    }
                    // Dev mode: a circuit promoted to tracks/ in this session is not embedded yet.
                    if crate::storage::is_dev_mode() {
                        if let Some(git_file) = self.resolve_preset_git_file(custom_id, None) {
                            return Track::load_from_file(&git_file)
                                .map_err(|e| format!("Failed to load git preset '{}': {}", git_file.display(), e));
                        }
                    }
                }
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
        let choice_module = match choice {
            TrackChoice::ClassicGrandPrix
            | TrackChoice::OvalSpeedway
            | TrackChoice::DriftPark
            | TrackChoice::KartArena
            | TrackChoice::RampRaceway
            | TrackChoice::ClassicRallycross
            | TrackChoice::OasisRally => Some("classic"),
            TrackChoice::Custom { path, .. } => {
                if path.starts_with("gt/") {
                    Some("gt")
                } else if path.starts_with("nascar/") {
                    Some("nascar")
                } else if path.starts_with("rally/") {
                    Some("rally")
                } else if path.starts_with("kart/") {
                    Some("kart")
                } else if path.starts_with("classic/") {
                    Some("classic")
                } else if path.starts_with("extreme_offroad/") {
                    Some("extreme_offroad")
                } else {
                    None
                }
            }
        };

        // Official presets load only from the official catalog (spec 042); the user folder never shadows them.
        if choice.is_official_preset() {
            if let Some(result) = crate::tracks::official::load(choice.track_id(), choice_module) {
                return result;
            }
            // Dev mode: a circuit promoted to tracks/ in this session is not embedded yet.
            if crate::storage::is_dev_mode() {
                if let Some(git_file) = self.resolve_preset_git_file(choice.track_id(), choice_module) {
                    return Track::load_from_file(&git_file)
                        .map_err(|e| format!("Failed to load git preset '{}': {}", git_file.display(), e));
                }
            }
            return Err(format!("Official circuit not found: {}", choice.track_id()));
        }

        match choice {
            TrackChoice::Custom { id, path, .. } => {
                let file_path = Path::new(path);
                if file_path.exists() {
                    if let Ok(t) = Track::load_from_file(file_path) {
                        return Ok(t);
                    }
                }
                if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir().filter(|_| crate::storage::is_dev_mode()) {
                    let git_rel = git_tracks_dir.join(path);
                    if git_rel.exists() {
                        if let Ok(t) = Track::load_from_file(&git_rel) {
                            return Ok(t);
                        }
                    }
                    let git_rel_json = git_tracks_dir.join(format!("{}.json", path));
                    if git_rel_json.exists() {
                        if let Ok(t) = Track::load_from_file(&git_rel_json) {
                            return Ok(t);
                        }
                    }
                }
                if crate::storage::is_dev_mode() {
                    if let Some(git_file) = self.resolve_preset_git_file(id, choice_module) {
                        if let Ok(t) = Track::load_from_file(&git_file) {
                            return Ok(t);
                        }
                    }
                }
                let alt_path = self.track_path_for_slug(id);
                if alt_path.exists() {
                    if let Ok(t) = Track::load_from_file(&alt_path) {
                        return Ok(t);
                    }
                }
                let backup_path = self.tracks_dir.join(".backup").join(format!("{}.json", id));
                if backup_path.exists() {
                    if let Ok(t) = Track::load_from_file(&backup_path) {
                        return Ok(t);
                    }
                }
                crate::tracks::official::load(id, choice_module)
                    .unwrap_or_else(|| Err(format!("Track file not found: {}", path)))
            }
            // The classic variants are official circuits and were resolved above.
            other => Err(format!("Official circuit not found: {}", other.track_id())),
        }
    }

    /// Returns the motorsport module of an official circuit id or alias, if it is one.
    pub fn preset_module(slug: &str) -> Option<&'static str> {
        if let Some(circuit) = tdrace_core::catalog::find(slug, None) {
            return Some(circuit.module);
        }
        // Dev mode: a circuit promoted to tracks/ in this session is not embedded yet.
        let git_tracks_dir = crate::storage::resolve_git_tracks_dir().filter(|_| crate::storage::is_dev_mode())?;
        ["classic", "rally", "kart", "gt", "nascar", "extreme_offroad"]
            .into_iter()
            .find(|m| git_tracks_dir.join(m).join(format!("{}.json", slug)).exists())
    }

    /// Returns the catalog id for an official circuit alias (e.g. "daytona" -> "daytona_superspeedway").
    pub fn canonical_preset_id(slug: &str) -> &str {
        tdrace_core::catalog::canonical_id(slug).unwrap_or(slug)
    }

    /// Returns the catalog id and every alias of an official circuit (empty for other slugs).
    pub fn preset_slug_aliases(slug: &str) -> Vec<&'static str> {
        let Some(id) = tdrace_core::catalog::canonical_id(slug) else {
            return Vec::new();
        };
        std::iter::once(id)
            .chain(tdrace_core::catalog::aliases().iter().filter(|(_, target)| *target == id).map(|(alias, _)| *alias))
            .collect()
    }

    /// Resolves the canonical file path in `git_tracks_dir` for a preset track slug,
    /// checking module directories and slug aliases.
    pub fn resolve_preset_git_file_with_dir(
        git_tracks_dir: &Path,
        slug: &str,
        module_hint: Option<&str>,
    ) -> Option<PathBuf> {
        let mut modules: Vec<&str> = Vec::new();
        if let Some(hint) = module_hint {
            modules.push(hint);
        }
        if let Some(m) = Self::preset_module(slug) {
            if !modules.contains(&m) {
                modules.push(m);
            }
        }
        for m in ["classic", "gt", "rally", "kart", "nascar", "extreme_offroad"] {
            if !modules.contains(&m) {
                modules.push(m);
            }
        }

        let aliases = Self::preset_slug_aliases(slug);
        let mut candidates: Vec<&str> = Vec::new();
        candidates.push(slug);
        for a in aliases {
            if !candidates.contains(&a) {
                candidates.push(a);
            }
        }

        for m in modules {
            for c in &candidates {
                let p = git_tracks_dir.join(m).join(format!("{}.json", c));
                if p.exists() {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Resolves the canonical file path in the repository's `tracks/` directory for a preset track slug, if present.
    pub fn resolve_preset_git_file(&self, slug: &str, module_hint: Option<&str>) -> Option<PathBuf> {
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
            Self::resolve_preset_git_file_with_dir(&git_tracks_dir, slug, module_hint)
        } else {
            None
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
            || self.tracks_dir.join("gt").join(&file_name).exists()
            || self.tracks_dir.join("rally").join(&file_name).exists()
            || self.tracks_dir.join("kart").join(&file_name).exists()
            || self.tracks_dir.join("nascar").join(&file_name).exists()
            || self.tracks_dir.join("extreme_offroad").join(&file_name).exists()
            || self.resolve_preset_git_file(slug, None).is_some()
    }

    /// Checks whether a track slug represents an already existing track (either official preset or user circuit on disk).
    pub fn is_existing_track(&self, slug: &str) -> bool {
        (Self::is_preset_slug(slug) && !self.is_preset_demoted(slug))
            || self.track_file_exists(slug)
            || self.track_path_for_slug(slug).exists()
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
            self.tracks_dir.join("gt").join(&file_name),
            self.tracks_dir.join("rally").join(&file_name),
            self.tracks_dir.join("kart").join(&file_name),
            self.tracks_dir.join("nascar").join(&file_name),
            self.tracks_dir.join("extreme_offroad").join(&file_name),
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

        // Official presets are immutable for standard users.
        // In dev mode, we dual-persist to user storage (`self.tracks_dir`) AND the repository's git-tracked directory.
        // Saving to user storage ensures edits are immune to git branch changes, checkouts, and test runs.
        if Self::is_preset_slug(&base_slug) && !self.is_preset_demoted(&base_slug) {
            if !crate::storage::is_dev_mode() {
                // Standard mode never overwrites an official circuit: save the edit as a new custom copy (spec 042 §2.5).
                return self.save_official_edit_as_copy(&track_to_save, &base_slug);
            }

            let mod_hint = track_to_save.module_id.clone().or_else(|| {
                if track_to_save.modules.iter().any(|m| m == "gt") {
                    Some("gt".to_string())
                } else {
                    None
                }
            });
            let mod_id = mod_hint
                .as_deref()
                .or_else(|| Self::preset_module(&base_slug))
                .unwrap_or("classic")
                .to_string();

            track_to_save.category = TrackCategory::Main;
            track_to_save.module_id = Some(mod_id.clone());
            if !track_to_save.modules.contains(&mod_id) {
                track_to_save.modules.push(mod_id.clone());
            }

            // Dev mode: the official circuit has one copy, tracks/<module>/<id>.json (spec 042 §2.5).
            let git_tracks_dir = crate::storage::resolve_git_tracks_dir()
                .ok_or_else(|| "Git repository tracks directory not found.".to_string())?;
            let canonical_slug = tdrace_core::catalog::canonical_id(&base_slug).unwrap_or(&base_slug);
            let target_git_file = self.resolve_preset_git_file(canonical_slug, Some(&mod_id)).unwrap_or_else(|| {
                let git_dir = git_tracks_dir.join(&mod_id);
                let _ = fs::create_dir_all(&git_dir);
                git_dir.join(format!("{}.json", canonical_slug))
            });
            track_to_save
                .save_to_file(&target_git_file)
                .map_err(|e| format!("Failed to save git-tracked preset: {}", e))?;

            let _ = self.scan_custom_tracks();
            crate::ui::menu::clear_menu_track_cache();
            return Ok(target_git_file.to_string_lossy().to_string());
        }

        // If file already exists and was Main category, keep its category and module when overwriting.
        let existing_path = self.track_path_for_slug(&base_slug);
        if overwrite && existing_path.exists() {
            if let Ok(existing) = Track::load_from_file(&existing_path) {
                track_to_save.category = existing.category;
                if track_to_save.modules.is_empty() {
                    track_to_save.module_id = existing.module_id;
                    track_to_save.modules = existing.modules;
                }
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
        } else if track_to_save.category == TrackCategory::Main {
            if track_to_save.modules.is_empty() {
                let mod_id = track_to_save.module_id.clone().unwrap_or_else(|| "classic".to_string());
                track_to_save.module_id = Some(mod_id.clone());
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
            Self::backup_track_file(&existing_path, &self.tracks_dir);
            existing_path
        } else {
            self.tracks_dir.join(format!("{}.json", file_slug))
        };

        track_to_save
            .save_to_file(&target_path)
            .map_err(|e| format!("Failed to save custom track: {}", e))?;

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
        Ok(target_path.to_string_lossy().to_string())
    }

    /// Saves a track to disk in the tracks directory. Overwrites by default.
    pub fn save_custom_track(&mut self, track: &Track, slug: Option<&str>) -> Result<String, String> {
        self.save_custom_track_with_options(track, slug, true)
    }

    /// Returns custom tracks assigned to a specific module.
    pub fn module_custom_tracks(&self, module_id: &str) -> Vec<TrackChoice> {
        if module_id == "drafts" {
            return self.draft_track_choices();
        }
        let mut choices = Vec::new();
        for custom in &self.custom_tracks {
            let is_demoted = self.is_preset_demoted(&custom.id);
            let is_active = custom.category == TrackCategory::Main || is_demoted;
            if is_active
                && (!Self::is_preset_slug(&custom.id) || is_demoted)
                && custom.belongs_to_module(module_id)
                && !self.is_custom_track_deleted_for_module(&custom.id, module_id)
            {
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

    /// Returns the list of motorsport module IDs ("classic", "rally", "kart", "gt") where a track is currently promoted / available.
    pub fn track_promoted_modules(&self, id: &str) -> Vec<String> {
        let mut mods = Vec::new();
        for mod_id in ["classic", "rally", "kart", "gt", "nascar", "extreme_offroad"] {
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

        if Self::is_preset_slug(id) && !self.is_preset_demoted(id) {
            if !crate::storage::is_dev_mode() {
                return Err(format!(
                    "'{}' is an official preset circuit and its categories cannot be modified in standard mode.",
                    id
                ));
            }
            if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
                let path_str = self.custom_tracks[pos].file_path.clone();
                let path = PathBuf::from(&path_str);
                if let Ok(mut t) = Track::load_from_file(&path) {
                    t.category = TrackCategory::Main;
                    t.modules = module_ids.iter().map(|s| s.to_string()).collect();
                    t.module_id = module_ids.first().map(|s| s.to_string());
                    let _ = t.save_to_file(&path);
                }
            }
            if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
                let mut track = self.load_track_by_slug(id)?;
                track.category = TrackCategory::Main;
                track.modules = module_ids.iter().map(|s| s.to_string()).collect();
                track.module_id = module_ids.first().map(|s| s.to_string());
                let orig_mod = Self::preset_module(id).unwrap_or("classic");
                let file_name = format!("{}.json", id);
                let target_p = git_tracks_dir.join(orig_mod).join(&file_name);
                let _ = fs::create_dir_all(git_tracks_dir.join(orig_mod));
                track
                    .save_to_file(&target_p)
                    .map_err(|e| format!("Failed to save git preset: {}", e))?;

                self.deleted_presets.retain(|d| {
                    d != id
                        && !module_ids.iter().any(|m| d == &format!("{}:{}", m, id))
                        && d != &format!("demoted:{}", id)
                });
                self.save_deleted_presets();

                let _ = self.scan_custom_tracks();
                crate::ui::menu::clear_menu_track_cache();
                return Ok(());
            }
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
        for sub in &["classic", "gt", "rally", "kart", "nascar", "extreme_offroad", "drafts"] {
            let legacy_p = self.tracks_dir.join(sub).join(&file_name);
            if legacy_p.exists() && legacy_p != target_path {
                let _ = fs::remove_file(legacy_p);
            }
        }

        self.deleted_presets.retain(|d| d != id && !module_ids.iter().any(|m| d == &format!("{}:{}", m, id)));
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
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
        if Self::is_preset_slug(id) && !crate::storage::is_dev_mode() {
            return Err(format!(
                "'{}' is an official preset circuit and cannot be demoted in standard mode.",
                id
            ));
        }

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
        for sub in &["classic", "gt", "rally", "kart", "nascar", "extreme_offroad", "drafts"] {
            let legacy_p = self.tracks_dir.join(sub).join(&file_name);
            if legacy_p.exists() && legacy_p != target_path {
                let _ = fs::remove_file(legacy_p);
            }
        }

        self.deleted_presets.retain(|d| d != id && !d.ends_with(&format!(":{}", id)));
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
        Ok(())
    }

    /// Updates the display name and description of a custom track and writes changes to disk.
    pub fn update_track_metadata(&mut self, id: &str, new_title: String, new_description: String) -> Result<(), String> {
        if Self::is_preset_slug(id) && !self.is_preset_demoted(id) {
            if !crate::storage::is_dev_mode() {
                return Err(format!(
                    "'{}' is an official preset circuit and its metadata cannot be modified in standard mode.",
                    id
                ));
            }
            if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
                let mut track = self.load_track_by_slug(id)?;
                track.name = new_title;
                track.description = new_description;
                let orig_mod = Self::preset_module(id).unwrap_or("classic");
                let file_name = format!("{}.json", id);
                let target_p = git_tracks_dir.join(orig_mod).join(&file_name);
                let _ = fs::create_dir_all(git_tracks_dir.join(orig_mod));
                track
                    .save_to_file(&target_p)
                    .map_err(|e| format!("Failed to save git preset: {}", e))?;

                let _ = self.scan_custom_tracks();
                crate::ui::menu::clear_menu_track_cache();
                return Ok(());
            }
        }

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
        crate::ui::menu::clear_menu_track_cache();
        Ok(())
    }

    /// Creates a new starter custom circuit assigned to a specific module.
    pub fn create_new_custom_track_with_template(
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
        track.category = TrackCategory::Main;
        track.module_id = Some(module_id.to_string());
        track.modules = vec![module_id.to_string()];

        let slug = Self::sanitize_slug(name);
        self.save_custom_track_with_options(&track, Some(&slug), false)
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
    /// Saves a standard-mode edit of an official circuit as a new draft in the user folder.
    fn save_official_edit_as_copy(&mut self, track: &Track, official_slug: &str) -> Result<String, String> {
        let mut copy = track.clone();
        copy.name = format!("{} (copy)", track.name.trim());
        copy.category = TrackCategory::Draft;
        copy.module_id = None;
        copy.modules.clear();

        let base_slug = format!("{}_copy", Self::sanitize_slug(official_slug));
        let mut file_slug = base_slug.clone();
        let mut counter = 1;
        while self.track_file_exists(&file_slug) {
            file_slug = format!("{}_{}", base_slug, counter);
            counter += 1;
        }
        let _ = fs::create_dir_all(&self.tracks_dir);
        let path = self.tracks_dir.join(format!("{}.json", file_slug));
        copy.save_to_file(&path)
            .map_err(|e| format!("Failed to save copy of official circuit: {}", e))?;

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
        Ok(path.to_string_lossy().to_string())
    }

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

    /// Resolves the corresponding TrackChoice for any track slug (preset or custom).
    pub fn track_choice_for_slug(&self, slug: &str) -> TrackChoice {
        if let Some(custom) = self.custom_tracks.iter().find(|t| t.id == slug) {
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
                "classic_rallycross" => TrackChoice::ClassicRallycross,
                "oasis_rally" => TrackChoice::OasisRally,
                custom_id => {
                    let path = self.track_path_for_slug(custom_id).to_string_lossy().to_string();
                    let (title, description) = self
                        .load_track_by_slug(custom_id)
                        .map(|t| (t.name, t.description))
                        .unwrap_or_else(|_| (custom_id.replace('_', " ").to_uppercase(), String::new()));
                    TrackChoice::Custom {
                        id: custom_id.to_string(),
                        title,
                        description,
                        path,
                    }
                }
            }
        }
    }

    /// Clones an existing circuit identified by its slug into the drafts group.
    pub fn clone_track_by_slug(&mut self, slug: &str) -> Result<(Track, String), String> {
        let _ = self.load_track_by_slug(slug)?;
        let choice = self.track_choice_for_slug(slug);
        self.clone_track(&choice)
    }

    /// Deletes a track specifically from the active module (or drafts).
    /// If `module_id` is None, deletes the track globally across all modules.
    pub fn delete_track_from_module(&mut self, id: &str, module_id: Option<&str>) -> Result<bool, String> {
        if Self::is_preset_slug(id) && !self.is_preset_demoted(id) && !crate::storage::is_dev_mode() {
            return Err(format!(
                "'{}' is an official preset circuit and cannot be deleted in standard mode.",
                id
            ));
        }

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

                // If legacy file in <mod_id>/<id>.json exists, backup and remove it
                let mod_path = self.tracks_dir.join(mod_id).join(&file_name);
                if mod_path.exists() {
                    Self::backup_track_file(&mod_path, &self.tracks_dir);
                    let _ = fs::remove_file(&mod_path);
                    deleted_any = true;
                }
                let mod_tdtrack = self.tracks_dir.join(mod_id).join(format!("{}.tdtrack", id));
                if mod_tdtrack.exists() {
                    Self::backup_track_file(&mod_tdtrack, &self.tracks_dir);
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
                    self.tracks_dir.join("gt").join(&file_name),
                    self.tracks_dir.join("rally").join(&file_name),
                    self.tracks_dir.join("kart").join(&file_name),
                    self.tracks_dir.join("nascar").join(&file_name),
                    self.tracks_dir.join("extreme_offroad").join(&file_name),
                ];
                for cand in &candidates {
                    if cand.exists() {
                        Self::backup_track_file(cand, &self.tracks_dir);
                        let _ = fs::remove_file(cand);
                        deleted_any = true;
                    }
                }
                let tdtrack = self.tracks_dir.join(format!("{}.tdtrack", id));
                if tdtrack.exists() {
                    Self::backup_track_file(&tdtrack, &self.tracks_dir);
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
        crate::ui::menu::clear_menu_track_cache();
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

    /// Safely archives a track file into `.backup/` before deletion or overwrite.
    /// Preserves both `<filename>` (latest backup) and a timestamped `<stem>_<unix_secs>.<ext>`.
    pub fn backup_track_file(file: &Path, tracks_dir: &Path) {
        let store = crate::tracks::UserTrackStore::new(tracks_dir);
        store.backup_file(file);
    }

    /// Promotes a custom track to an official git-tracked preset (dev mode only).
    /// Moves the track JSON to `tracks/<module>/<slug>.json`; the user copy is backed up and removed (spec 042 §2.5).
    /// `target_module` is the module picked in the promote dialog; without it the track's own module is used.
    pub fn promote_custom_track_to_git_preset(&mut self, id: &str, target_module: Option<&str>) -> Result<PathBuf, String> {
        if !crate::storage::is_dev_mode() {
            return Err("Promoting tracks to preset circuits is only allowed in developer mode.".to_string());
        }
        let git_tracks_dir = crate::storage::resolve_git_tracks_dir()
            .ok_or_else(|| "Git repository tracks directory not found.".to_string())?;

        let (mut track, local_path) = if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let p_str = self.custom_tracks[pos].file_path.clone();
            let p = PathBuf::from(&p_str);
            let t = Track::load_from_file(&p)
                .map_err(|e| format!("Failed to load custom track '{}': {}", id, e))?;
            (t, Some(p))
        } else {
            let t = self.load_track_by_slug(id)
                .map_err(|e| format!("Track '{}' not found: {}", id, e))?;
            (t, None)
        };

        let target_module = target_module
            .map(|m| Self::normalize_module_id(m).to_string())
            .or_else(|| track.module_id.clone())
            .or_else(|| track.modules.first().cloned())
            .unwrap_or_else(|| "classic".to_string());

        track.category = TrackCategory::Main;
        if !track.modules.contains(&target_module) {
            track.modules.insert(0, target_module.clone());
        }
        track.module_id = Some(target_module.clone());

        let target_dir = git_tracks_dir.join(&target_module);
        let _ = fs::create_dir_all(&target_dir);
        let target_path = target_dir.join(format!("{}.json", id));

        track.save_to_file(&target_path)
            .map_err(|e| format!("Failed to save git preset '{}': {}", target_path.display(), e))?;

        // The official circuit now lives only in tracks/. Back up and remove the user copies,
        // so a custom copy never sits next to the official one (recoverable from `.backup/`).
        let user_file = self.tracks_dir.join(format!("{}.json", id));
        let draft_cand = self.tracks_dir.join("drafts").join(format!("{}.json", id));
        for p in [Some(user_file), Some(draft_cand), local_path].into_iter().flatten() {
            if p.exists() && p.starts_with(&self.tracks_dir) {
                Self::backup_track_file(&p, &self.tracks_dir);
                let _ = fs::remove_file(p);
            }
        }

        // Clean up any deleted_presets marker for this track
        self.deleted_presets.retain(|d| {
            d != id && !d.ends_with(&format!(":{}", id)) && d != &format!("demoted:{}", id)
        });
        self.save_deleted_presets();

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
        Ok(target_path)
    }

    /// Demotes an official preset circuit to a custom draft circuit in local storage (dev mode only).
    pub fn demote_preset_to_custom_track(&mut self, id: &str) -> Result<PathBuf, String> {
        if !crate::storage::is_dev_mode() {
            return Err("Demoting preset circuits is only allowed in developer mode.".to_string());
        }

        let mut track = self.load_track_by_slug(id)?;
        track.category = TrackCategory::Main;
        let orig_mod = Self::preset_module(id).unwrap_or("classic");
        if track.module_id.is_none() {
            track.module_id = Some(orig_mod.to_string());
        }
        if track.modules.is_empty() {
            track.modules = vec![orig_mod.to_string()];
        }

        let _ = fs::create_dir_all(&self.tracks_dir);
        let target_path = self.tracks_dir.join(format!("{}.json", id));
        track.save_to_file(&target_path)
            .map_err(|e| format!("Failed to save custom circuit: {}", e))?;

        // If it was a git preset, remove it from git_tracks_dir
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
            for m in ["classic", "rally", "kart", "gt", "nascar", "extreme_offroad"] {
                let git_file = git_tracks_dir.join(m).join(format!("{}.json", id));
                if git_file.exists() {
                    let _ = fs::remove_file(git_file);
                }
            }
        }

        // Add demoted marker to hide procedural preset from preset lists
        let demoted_marker = format!("demoted:{}", id);
        if !self.deleted_presets.iter().any(|d| d == &demoted_marker) {
            self.deleted_presets.push(demoted_marker);
            self.save_deleted_presets();
        }

        let _ = self.scan_custom_tracks();
        crate::ui::menu::clear_menu_track_cache();
        Ok(target_path)
    }

    /// Manually moves a preset track up or down within its module's preset order (developer mode only).
    pub fn reorder_preset_track(
        &mut self,
        track_id: &str,
        module_id: &str,
        move_up: bool,
    ) -> Result<bool, String> {
        if !crate::storage::is_dev_mode() {
            return Err("Reordering preset tracks is only allowed in developer mode.".to_string());
        }

        let norm_mod = Self::normalize_module_id(module_id);
        let choices = self.preset_track_choices(norm_mod);
        let current_ids: Vec<String> = choices.iter().map(|c| c.track_id().to_string()).collect();

        let pos = current_ids
            .iter()
            .position(|id| id == track_id || Self::canonical_preset_id(id) == Self::canonical_preset_id(track_id))
            .ok_or_else(|| format!("Preset track '{}' not found in module '{}'.", track_id, norm_mod))?;

        let target_pos = if move_up {
            if pos == 0 {
                return Ok(false); // Already at top
            }
            pos - 1
        } else {
            if pos + 1 >= current_ids.len() {
                return Ok(false); // Already at bottom
            }
            pos + 1
        };

        let mut new_order = current_ids;
        new_order.swap(pos, target_pos);

        self.preset_order.insert(norm_mod.to_string(), new_order);
        self.save_preset_order();
        crate::ui::menu::clear_menu_track_cache();

        Ok(true)
    }

    /// Resets the preset track order for a specific module back to default (developer mode only).
    pub fn reset_preset_order(&mut self, module_id: &str) -> Result<bool, String> {
        if !crate::storage::is_dev_mode() {
            return Err("Resetting preset track order is only allowed in developer mode.".to_string());
        }
        let norm_mod = Self::normalize_module_id(module_id);
        let removed = self.preset_order.remove(norm_mod).is_some();
        if removed {
            self.save_preset_order();
            crate::ui::menu::clear_menu_track_cache();
        }
        Ok(removed)
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
        assert_eq!(choices.len(), 101); // 13 classic + 18 gt + 17 rally + 17 kart + 17 nascar + 19 unique extreme off-road

        let mut gp = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
        gp.name = "My Custom GP".to_string();
        gp.description = "A custom testing GP".to_string();
        gp.category = TrackCategory::Draft;

        let saved_path = manager
            .save_custom_track(&gp, Some("test_custom_gp"))
            .expect("Must save custom track");
        assert!(Path::new(&saved_path).exists());

        // Since gp was saved as Draft, main choices is still 98, but draft choices has 1
        assert_eq!(manager.main_track_choices().len(), 101);
        assert_eq!(manager.draft_track_choices().len(), 1);

        let draft_choice = &manager.draft_track_choices()[0];
        assert_eq!(draft_choice.title(), "My Custom GP");
        assert_eq!(draft_choice.description(), "A custom testing GP");

        // Promote track to Main
        manager.promote_track("test_custom_gp").expect("Must promote");
        assert_eq!(manager.main_track_choices().len(), 102);
        assert_eq!(manager.draft_track_choices().len(), 0);

        // Edit metadata
        manager
            .update_track_metadata(
                "test_custom_gp",
                "Renamed Grand Prix".to_string(),
                "Updated description text".to_string(),
            )
            .expect("Must update metadata");
        let loaded = manager.load_track(&manager.main_track_choices()[101]).expect("Must load");
        assert_eq!(loaded.name, "Renamed Grand Prix");
        assert_eq!(loaded.description, "Updated description text");

        // Demote back to draft
        manager.demote_track("test_custom_gp").expect("Must demote");
        assert_eq!(manager.main_track_choices().len(), 101);
        assert_eq!(manager.draft_track_choices().len(), 1);

        // Clean up
        assert!(manager.delete_custom_track("test_custom_gp").unwrap());
        assert_eq!(manager.main_track_choices().len(), 101);
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

        let mut track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
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
        assert_eq!(classic_tracks.len(), 13);

        // GT tracks
        let gt_tracks = manager.module_catalog_tracks("gt");
        assert_eq!(gt_tracks.len(), 18);
        assert!(gt_tracks.iter().any(|t| t.title().contains("Monza")));
        assert!(gt_tracks.iter().any(|t| t.title().contains("Spa")));
        assert!(gt_tracks.iter().any(|t| t.title().contains("Silverstone")));
        assert!(gt_tracks.iter().any(|t| t.title().contains("MadRing")));

        // Rally tracks
        let rally_tracks = manager.module_catalog_tracks("rally");
        assert_eq!(rally_tracks.len(), 17);
        assert!(rally_tracks.iter().any(|t| t.title().contains("Circuit des Ducs")));
        assert!(rally_tracks.iter().any(|t| t.title().contains("Höljes")));

        // Kart tracks
        let kart_tracks = manager.module_catalog_tracks("kart");
        assert_eq!(kart_tracks.len(), 17);
        assert!(kart_tracks.iter().any(|t| t.title().contains("Lonato")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Sarno")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Genk")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("PFI") || t.title().contains("PF International")));

        // Nascar tracks
        let nascar_tracks = manager.module_catalog_tracks("nascar");
        assert_eq!(nascar_tracks.len(), 17);
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Daytona")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Talladega")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Watkins Glen")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Bristol")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Martinsville")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Darlington")));
        assert!(nascar_tracks.iter().any(|t| t.title().contains("Charlotte")));

        // Extreme Off-Road tracks
        let offroad_tracks = manager.module_catalog_tracks("extreme_offroad");
        assert_eq!(offroad_tracks.len(), 20);
        assert!(offroad_tracks.iter().any(|t| t.title().contains("Sahara")));
        assert!(offroad_tracks.iter().any(|t| t.title().contains("Baja")));

        // All tracks
        let all_tracks = manager.module_catalog_tracks("all");
        assert_eq!(all_tracks.len(), 101);

        // Save a custom circuit assigned to classic and rally
        let mut custom_circuit = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
        custom_circuit.name = "Custom Category Circuit".to_string();
        custom_circuit.modules = vec!["classic".to_string(), "rally".to_string()];
        let _ = manager.save_custom_track_with_options(&custom_circuit, Some("custom_cat_circuit"), false);

        // Classic category: 10 presets first, then 1 custom track
        let classic_after = manager.module_catalog_tracks("classic");
        assert_eq!(classic_after.len(), 14);
        for track in &classic_after[..10] {
            assert!(track.is_official_preset(), "Presets must appear first in catalog: {}", track.title());
        }
        assert!(classic_after[13].is_user_custom(), "Custom circuit must appear after presets");
        assert_eq!(classic_after[13].title(), "Custom Category Circuit");

        // Rally category: 17 presets first, then 1 custom track
        let rally_after = manager.module_catalog_tracks("rally");
        assert_eq!(rally_after.len(), 18);
        for track in &rally_after[..17] {
            assert!(track.is_official_preset(), "Presets must appear first in rally: {}", track.title());
        }
        assert!(rally_after[17].is_user_custom(), "Custom circuit must appear after presets");
        assert_eq!(rally_after[17].title(), "Custom Category Circuit");

        // GT and Kart: Must not contain this custom track
        let gt_after = manager.module_catalog_tracks("gt");
        assert_eq!(gt_after.len(), 18);
        assert!(!gt_after.iter().any(|t| t.title() == "Custom Category Circuit"));

        let kart_after = manager.module_catalog_tracks("kart");
        assert_eq!(kart_after.len(), 17);
        assert!(!kart_after.iter().any(|t| t.title() == "Custom Category Circuit"));

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

        // Cloned track must appear in drafts, and main count stays 98
        assert_eq!(manager.draft_track_choices().len(), 1);
        assert_eq!(manager.main_track_choices().len(), 101);
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
