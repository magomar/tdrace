use std::path::PathBuf;
use tdrace_core::track::Track;
use crate::module::{
    classic::ClassicGameModule,
    f1::F1GameModule,
    kart::KartGameModule,
    nascar::NascarGameModule,
    rally::RallyGameModule,
    GameModule,
};
use crate::ui::menu::TrackChoice;

/// Read-only catalog of official presets.
/// Provides safe resolution from Git presets (if available) or built-in procedural generators.
#[derive(Debug, Clone, Default)]
pub struct PresetCatalog;

impl PresetCatalog {
    pub fn new() -> Self {
        Self
    }

    /// Resolves canonical file in the repository's `tracks/` directory for a preset slug, if present.
    pub fn resolve_git_preset_file(slug: &str, module_hint: Option<&str>) -> Option<PathBuf> {
        let git_dir = crate::storage::resolve_git_tracks_dir()?;
        let mut modules: Vec<&str> = Vec::new();
        if let Some(hint) = module_hint {
            modules.push(hint);
        }
        for m in ["classic", "f1", "gt", "rally", "kart", "nascar"] {
            if !modules.contains(&m) {
                modules.push(m);
            }
        }

        let aliases = crate::track_manager::TrackManager::preset_slug_aliases(slug);
        let mut candidates: Vec<&str> = vec![slug];
        for a in aliases {
            if !candidates.contains(a) {
                candidates.push(a);
            }
        }

        for m in modules {
            for c in &candidates {
                let p = git_dir.join(m).join(format!("{}.json", c));
                if p.exists() {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Loads a preset track instance by slug.
    pub fn load_preset(slug: &str) -> Option<Track> {
        if let Some(git_file) = Self::resolve_git_preset_file(slug, None) {
            if let Ok(t) = Track::load_from_file(&git_file) {
                return Some(t);
            }
        }
        TrackChoice::resolve_procedural_preset_by_slug(slug)
    }

    /// Returns the official preset choices for a given motorsport module.
    pub fn preset_choices_for_module(module_id: &str) -> Vec<TrackChoice> {
        match module_id {
            "gt" | "gt_challenge" | "f1" => {
                let m = F1GameModule::new();
                m.tracks()
                    .iter()
                    .map(|def| crate::track_manager::TrackManager::track_choice_from_def(def, "gt"))
                    .collect()
            }
            "rally" => {
                let m = RallyGameModule::new();
                m.tracks()
                    .iter()
                    .map(|def| crate::track_manager::TrackManager::track_choice_from_def(def, "rally"))
                    .collect()
            }
            "kart" => {
                let m = KartGameModule::new();
                m.tracks()
                    .iter()
                    .map(|def| crate::track_manager::TrackManager::track_choice_from_def(def, "kart"))
                    .collect()
            }
            "nascar" => {
                let m = NascarGameModule::new();
                m.tracks()
                    .iter()
                    .map(|def| crate::track_manager::TrackManager::track_choice_from_def(def, "nascar"))
                    .collect()
            }
            _ => {
                let m = ClassicGameModule::new();
                m.tracks()
                    .iter()
                    .map(|def| crate::track_manager::TrackManager::track_choice_from_def(def, "classic"))
                    .collect()
            }
        }
    }
}
