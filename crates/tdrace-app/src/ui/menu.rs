use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::audio::AudioSettings;
use crate::game::XpAwardReceipt;
use crate::render::color::Palette;
use cabinet::input::GamepadSnapshot;
use cabinet::state::{CabinetContext, CabinetScreen, UniversalConfirmModal};
use cabinet::ui::theme::CabinetTheme;
use tdrace_core::physics::config::{AssistProfile, CarConfig};

/// Available track options in track selection menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackChoice {
    ClassicGrandPrix,
    OvalSpeedway,
    DriftPark,
    KartArena,
    RampRaceway,
    OasisRally,
    ClassicRallycross,
    Custom { id: String, title: String, description: String, path: String },
}

impl TrackChoice {
    pub const ALL: [Self; 7] = [
        Self::ClassicGrandPrix,
        Self::OvalSpeedway,
        Self::DriftPark,
        Self::KartArena,
        Self::RampRaceway,
        Self::OasisRally,
        Self::ClassicRallycross,
    ];

    pub fn title(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "Classic Grand Prix",
            Self::OvalSpeedway => "Oval Speedway",
            Self::DriftPark => "Drift Park",
            Self::KartArena => "Kart Arena",
            Self::RampRaceway => "Ramp Raceway",
            Self::OasisRally => "Oasis Rally",
            Self::ClassicRallycross => "Classic Rallycross",
            Self::Custom { title, .. } => title.as_str(),
        }
    }

    pub fn tag(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "FIA GP CIRCUIT",
            Self::OvalSpeedway => "SUPERSPEEDWAY",
            Self::DriftPark => "TECHNICAL DRIFT",
            Self::KartArena => "AGILE SPRINT",
            Self::RampRaceway => "DIRT STUNT RAMPS",
            Self::OasisRally => "DESERT DIRT RALLY",
            Self::ClassicRallycross => "HYBRID RALLYCROSS",
            Self::Custom { id, path, .. } => {
                if path.contains("/rally/") || path.starts_with("rally/") || matches!(id.as_str(), "essay_rx" | "essay" | "holjes_rx" | "holjes" | "lydden_hill" | "lydden" | "hell_rx" | "hell" | "loheac_rx" | "loheac" | "estering_rx" | "estering" | "montalegre_rx" | "montalegre" | "nyirad_rx" | "nyirad" | "kouvola_rx" | "kouvola" | "catalunya_rx" | "mettet_rx" | "mettet" | "silverstone_rx" | "riga_rx" | "riga" | "bikernieki" | "killarney_rx" | "killarney" | "yas_marina_rx" | "yas_marina") {
                    "RALLY CROSS"
                } else if path.contains("/gt/") || path.starts_with("gt/") || matches!(id.as_str(), "monza" | "spa" | "silverstone" | "monaco" | "suzuka" | "interlagos" | "montreal" | "red_bull_ring" | "catalunya" | "zandvoort" | "bahrain" | "marina_bay" | "singapore" | "singapur" | "cota" | "madring" | "nurburgring_gp" | "nurburgring" | "bathurst" | "mount_panorama" | "portimao_gp" | "portimao" | "le_mans_sarthe" | "le_mans") {
                    "GT WORLD CHALLENGE"
                } else if path.contains("/kart/") || path.starts_with("kart/") || matches!(id.as_str(), "valencia_kart" | "valencia" | "campillos" | "lonato" | "sarno" | "genk" | "pfi" | "zuera" | "le_mans_kart" | "portimao_kart" | "franciacorta" | "wackersdorf" | "prokart_wackersdorf" | "kristianstad" | "asum_ring" | "seven_laghi" | "7laghi" | "castelletto_kart" | "castelletto" | "ampfing" | "schweppermannring" | "silverstone_national_kart" | "silverstone_kart") {
                    "KARTING"
                } else if path.contains("/nascar/") || path.starts_with("nascar/") || matches!(id.as_str(), "daytona" | "daytona_superspeedway" | "talladega" | "talladega_superspeedway" | "watkins_glen" | "watkins_glen_nascar" | "bristol" | "bristol_motor_speedway" | "martinsville" | "martinsville_speedway" | "darlington" | "darlington_raceway" | "charlotte" | "charlotte_motor_speedway" | "indianapolis" | "indianapolis_motor_speedway" | "eldora" | "eldora_speedway" | "iowa" | "iowa_speedway" | "road_america" | "chicago" | "chicago_street_course") {
                    "NASCAR CUP"
                } else {
                    "CLASSIC MOTORSPORT"
                }
            }
        }
    }

    pub fn tag_for_module(&self, mod_id: &str) -> &str {
        match self {
            Self::ClassicGrandPrix => {
                if mod_id == "gt" || mod_id == "gt_challenge" { "GT GP CIRCUIT" } else { "FIA GP CIRCUIT" }
            }
            Self::OvalSpeedway => "SUPERSPEEDWAY",
            Self::DriftPark => "TECHNICAL DRIFT",
            Self::KartArena => "AGILE SPRINT",
            Self::RampRaceway => "DIRT STUNT RAMPS",
            Self::OasisRally => "DESERT DIRT RALLY",
            Self::ClassicRallycross => "HYBRID RALLYCROSS",
            Self::Custom { id, path, .. } => {
                if path.contains("/rally/") || path.starts_with("rally/") || matches!(id.as_str(), "essay_rx" | "essay" | "holjes_rx" | "holjes" | "lydden_hill" | "lydden" | "hell_rx" | "hell" | "loheac_rx" | "loheac" | "estering_rx" | "estering" | "montalegre_rx" | "montalegre" | "nyirad_rx" | "nyirad" | "kouvola_rx" | "kouvola" | "catalunya_rx" | "mettet_rx" | "mettet" | "silverstone_rx" | "riga_rx" | "riga" | "bikernieki" | "killarney_rx" | "killarney" | "yas_marina_rx" | "yas_marina") {
                    "RALLY CROSS"
                } else if path.contains("/gt/") || path.starts_with("gt/") || matches!(id.as_str(), "monza" | "spa" | "silverstone" | "monaco" | "suzuka" | "interlagos" | "montreal" | "red_bull_ring" | "catalunya" | "zandvoort" | "bahrain" | "marina_bay" | "singapore" | "singapur" | "cota" | "madring" | "nurburgring_gp" | "nurburgring" | "bathurst" | "mount_panorama" | "portimao_gp" | "portimao" | "le_mans_sarthe" | "le_mans") {
                    "GT WORLD CHALLENGE"
                } else if path.contains("/kart/") || path.starts_with("kart/") || matches!(id.as_str(), "valencia_kart" | "valencia" | "campillos" | "lonato" | "sarno" | "genk" | "pfi" | "zuera" | "le_mans_kart" | "portimao_kart" | "franciacorta" | "wackersdorf" | "prokart_wackersdorf" | "kristianstad" | "asum_ring" | "seven_laghi" | "7laghi" | "castelletto_kart" | "castelletto" | "ampfing" | "schweppermannring" | "silverstone_national_kart" | "silverstone_kart") {
                    "KARTING"
                } else if path.contains("/nascar/") || path.starts_with("nascar/") || matches!(id.as_str(), "daytona" | "daytona_superspeedway" | "talladega" | "talladega_superspeedway" | "watkins_glen" | "watkins_glen_nascar" | "bristol" | "bristol_motor_speedway" | "martinsville" | "martinsville_speedway" | "darlington" | "darlington_raceway" | "charlotte" | "charlotte_motor_speedway" | "indianapolis" | "indianapolis_motor_speedway" | "eldora" | "eldora_speedway" | "iowa" | "iowa_speedway" | "road_america" | "chicago" | "chicago_street_course") {
                    "NASCAR CUP"
                } else if path.contains("/extreme_offroad/") || path.starts_with("extreme_offroad/") || matches!(id.as_str(), "sahara_dune_crossing" | "atacama_sand_basin" | "atacama" | "red_rock_canyon" | "red_rock" | "baja_500_desert_scrub" | "baja_500" | "baja" | "mud_slough_arena" | "mud_slough" | "gravel_quarry_chasm" | "gravel_quarry" | "louisiana_mud_swampland" | "louisiana_swampland" | "louisiana" | "arctic_frozen_lake" | "frozen_lake" | "alpine_snow_ridge" | "alpine_snow" | "rovaniemi_ice_ring" | "rovaniemi" | "glacier_crest_pass" | "glacier_crest" | "supercross_stadium_arena" | "supercross_stadium" | "supercross" | "monster_colosseum" | "stunt_city_megastructure" | "stunt_city") {
                    "EXTREME OFF-ROAD"
                } else {
                    match mod_id {
                        "gt" | "gt_challenge" => "GT WORLD CHALLENGE",
                        "rally" => "RALLY CROSS",
                        "kart" => "KARTING",
                        "nascar" => "NASCAR CUP",
                        "extreme_offroad" => "EXTREME OFF-ROAD",
                        _ => "CLASSIC MOTORSPORT",
                    }
                }
            }
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.",
            Self::OvalSpeedway => "Full-throttle banked superspeedway surrounded by concrete barriers.",
            Self::DriftPark => "Technical hairpin slides, wide transitions & dynamic apex clipping zones.",
            Self::KartArena => "Tight 90-degree corners, rapid switchbacks & aggressive rumble curbs.",
            Self::RampRaceway => "High-speed dirt stadium circuit with launch ramps, hazard water puddles, gap jumps & banked dirt turns.",
            Self::OasisRally => "Pure dirt desert rally circuit with oasis water hazards, perilous sand traps & high-sliding rally dynamics.",
            Self::ClassicRallycross => "Dynamic 1.0 km mixed-surface rallycross circuit featuring asphalt launch straights, high-grip chicanes, sweeping dirt hairpins & tabletop jump ramps.",
            Self::Custom { description, .. } => {
                if description.trim().is_empty() {
                    "User-created custom racing circuit."
                } else {
                    description.as_str()
                }
            }
        }
    }

    pub fn track_id(&self) -> &str {
        match self {
            Self::ClassicGrandPrix => "classic_grand_prix",
            Self::OvalSpeedway => "oval_speedway",
            Self::DriftPark => "drift_park",
            Self::KartArena => "kart_arena",
            Self::RampRaceway => "ramp_raceway",
            Self::OasisRally => "oasis_rally",
            Self::ClassicRallycross => "classic_rallycross",
            Self::Custom { id, .. } => id.as_str(),
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom { .. })
    }

    /// Returns true if this track choice represents an official built-in preset (from core or a game module).
    pub fn is_official_preset(&self) -> bool {
        match self {
            Self::ClassicGrandPrix
            | Self::OvalSpeedway
            | Self::DriftPark
            | Self::KartArena
            | Self::RampRaceway
            | Self::OasisRally
            | Self::ClassicRallycross => true,
            Self::Custom { id, path, .. } => {
                let is_demoted = crate::track_manager::TrackManager::is_preset_slug_demoted_in_path(id, path);
                !is_demoted
                    && (crate::track_manager::TrackManager::is_preset_slug(id)
                        || path.starts_with("gt/")
                        || path.starts_with("rally/")
                        || path.starts_with("kart/")
                        || path.starts_with("nascar/")
                        || path.starts_with("classic/"))
            }
        }
    }

    /// Returns true if this track choice represents a user-created custom or cloned circuit.
    pub fn is_user_custom(&self) -> bool {
        !self.is_official_preset()
    }
}

static MENU_TRACK_CACHE: std::sync::Mutex<Option<std::collections::HashMap<String, tdrace_core::track::Track>>> =
    std::sync::Mutex::new(None);

/// Clears the cached resolved menu tracks (e.g. after track edit or save).
pub fn clear_menu_track_cache() {
    if let Ok(mut guard) = MENU_TRACK_CACHE.lock() {
        if let Some(cache) = guard.as_mut() {
            cache.clear();
        }
    }
}

/// Resolves a TrackChoice to a concrete Track instance for UI preview rendering.
pub fn resolve_track_for_menu(choice: &TrackChoice) -> Option<tdrace_core::track::Track> {
    resolve_track_for_menu_with_dir(choice, crate::storage::resolve_user_tracks_dir())
}

/// Resolves a TrackChoice to a concrete Track instance for UI preview rendering given a tracks directory, cached.
pub fn resolve_track_for_menu_with_dir(
    choice: &TrackChoice,
    tracks_dir: impl AsRef<std::path::Path>,
) -> Option<tdrace_core::track::Track> {
    let dir = tracks_dir.as_ref();
    let cache_key = format!("{}:{}", choice.track_id(), dir.display());

    if let Ok(guard) = MENU_TRACK_CACHE.lock() {
        if let Some(cache) = guard.as_ref() {
            if let Some(track) = cache.get(&cache_key) {
                return Some(track.clone());
            }
        }
    }

    let loaded = resolve_track_for_menu_with_dir_uncached(choice, dir);
    if let Some(ref t) = loaded {
        if let Ok(mut guard) = MENU_TRACK_CACHE.lock() {
            let cache = guard.get_or_insert_with(std::collections::HashMap::new);
            cache.insert(cache_key, t.clone());
        }
    }
    loaded
}

fn resolve_track_for_menu_with_dir_uncached(
    choice: &TrackChoice,
    dir: &std::path::Path,
) -> Option<tdrace_core::track::Track> {
    let choice_module = match choice {
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
            } else {
                None
            }
        }
        _ => None,
    };

    // 0. Check user storage first: if the user customized this track (preset or custom),
    // their local saved version in `dir` takes highest priority.
    let id = choice.track_id();
    let file_name = format!("{}.json", id);
    let user_candidates = [
        dir.join(&file_name),
        dir.join("classic").join(&file_name),
        dir.join("gt").join(&file_name),
        dir.join("rally").join(&file_name),
        dir.join("kart").join(&file_name),
        dir.join("nascar").join(&file_name),
        dir.join("drafts").join(&file_name),
    ];
    for p in &user_candidates {
        if p.exists() {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(p) {
                return Some(t);
            }
        }
    }

    // 1. If git_tracks_dir exists, official git presets or custom tracks saved into the repository's
    // tracks/ directory take second precedence.
    if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
        if let Some(p) = crate::track_manager::TrackManager::resolve_preset_git_file_with_dir(
            &git_tracks_dir,
            choice.track_id(),
            choice_module,
        ) {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(&p) {
                return Some(t);
            }
        }
    }

    // 1. If custom choice and path directly exists on disk, load it
    if let TrackChoice::Custom { path, .. } = choice {
        let file_path = std::path::Path::new(path);
        if file_path.exists() {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(file_path) {
                return Some(t);
            }
        }
        if file_path.with_extension("json").exists() {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(&file_path.with_extension("json")) {
                return Some(t);
            }
        }
        let rel_in_dir = dir.join(path);
        if rel_in_dir.exists() {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(&rel_in_dir) {
                return Some(t);
            }
        }
        let rel_in_dir_json = dir.join(format!("{}.json", path));
        if rel_in_dir_json.exists() {
            if let Ok(t) = tdrace_core::track::Track::load_from_file(&rel_in_dir_json) {
                return Some(t);
            }
        }
        if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
            let rel_in_git = git_tracks_dir.join(path);
            if rel_in_git.exists() {
                if let Ok(t) = tdrace_core::track::Track::load_from_file(&rel_in_git) {
                    return Some(t);
                }
            }
            let rel_in_git_json = git_tracks_dir.join(format!("{}.json", path));
            if rel_in_git_json.exists() {
                if let Ok(t) = tdrace_core::track::Track::load_from_file(&rel_in_git_json) {
                    return Some(t);
                }
            }
        }
    }

    // 2. Check git fallback candidates across module subdirectories

    if let Some(git_tracks_dir) = crate::storage::resolve_git_tracks_dir() {
        let git_candidates = [
            git_tracks_dir.join("classic").join(&file_name),
            git_tracks_dir.join("gt").join(&file_name),
            git_tracks_dir.join("rally").join(&file_name),
            git_tracks_dir.join("kart").join(&file_name),
            git_tracks_dir.join("nascar").join(&file_name),
            git_tracks_dir.join(&file_name),
        ];
        for p in &git_candidates {
            if p.exists() {
                if let Ok(t) = tdrace_core::track::Track::load_from_file(p) {
                    return Some(t);
                }
            }
        }
    }

    // 3. Fallback to procedural preset definitions
    TrackChoice::resolve_procedural_preset(choice)
}

impl TrackChoice {
    pub fn resolve_procedural_preset_by_slug(slug: &str) -> Option<tdrace_core::track::Track> {
        let choice = match slug {
            "classic_grand_prix" => Self::ClassicGrandPrix,
            "oval_speedway" => Self::OvalSpeedway,
            "drift_park" => Self::DriftPark,
            "kart_arena" => Self::KartArena,
            "ramp_raceway" => Self::RampRaceway,
            "oasis_rally" => Self::OasisRally,
            "classic_rallycross" => Self::ClassicRallycross,
            other => Self::Custom {
                id: other.to_string(),
                title: other.to_string(),
                description: String::new(),
                path: String::new(),
            },
        };
        Self::resolve_procedural_preset(&choice)
    }

    pub fn resolve_procedural_preset(choice: &TrackChoice) -> Option<tdrace_core::track::Track> {
    match choice {
        TrackChoice::ClassicGrandPrix => Some(tdrace_core::track::presets::classic_grand_prix()),
        TrackChoice::OvalSpeedway => Some(tdrace_core::track::presets::oval_speedway()),
        TrackChoice::DriftPark => Some(tdrace_core::track::presets::drift_park()),
        TrackChoice::KartArena => Some(tdrace_core::track::presets::kart_arena()),
        TrackChoice::RampRaceway => Some(tdrace_core::track::presets::ramp_raceway()),
        TrackChoice::OasisRally => Some(tdrace_core::track::presets::oasis_rally()),
        TrackChoice::ClassicRallycross => Some(tdrace_core::track::presets::classic_rallycross()),
        TrackChoice::Custom { id, .. } => match id.as_str() {
            "dirty_oval_speedway" | "dirty_oval" => Some(tdrace_core::track::presets::dirty_oval_speedway()),
            "figure_eight" | "figure_8" => Some(tdrace_core::track::presets::figure_eight()),
            "monza" => Some(crate::module::gt::GtWorldChallengeModule::track_monza()),
            "spa" => Some(crate::module::gt::GtWorldChallengeModule::track_spa()),
            "silverstone" => Some(crate::module::gt::GtWorldChallengeModule::track_silverstone()),
            "monaco" => Some(crate::module::gt::GtWorldChallengeModule::track_monaco()),
            "suzuka" => Some(crate::module::gt::GtWorldChallengeModule::track_suzuka()),
            "interlagos" => Some(crate::module::gt::GtWorldChallengeModule::track_interlagos()),
            "montreal" => Some(crate::module::gt::GtWorldChallengeModule::track_montreal()),
            "red_bull_ring" => Some(crate::module::gt::GtWorldChallengeModule::track_red_bull_ring()),
            "catalunya" => Some(crate::module::gt::GtWorldChallengeModule::track_catalunya()),
            "zandvoort" => Some(crate::module::gt::GtWorldChallengeModule::track_zandvoort()),
            "bahrain" => Some(crate::module::gt::GtWorldChallengeModule::track_bahrain()),
            "marina_bay" | "singapore" | "singapur" => Some(crate::module::gt::GtWorldChallengeModule::track_marina_bay()),
            "cota" => Some(crate::module::gt::GtWorldChallengeModule::track_cota()),
            "madring" => Some(crate::module::gt::GtWorldChallengeModule::track_madring()),
            "nurburgring_gp" | "nurburgring" => Some(crate::module::gt::GtWorldChallengeModule::track_nurburgring_gp()),
            "bathurst" | "mount_panorama" => Some(crate::module::gt::GtWorldChallengeModule::track_bathurst()),
            "portimao_gp" | "portimao" => Some(crate::module::gt::GtWorldChallengeModule::track_portimao_gp()),
            "le_mans_sarthe" | "le_mans" => Some(crate::module::gt::GtWorldChallengeModule::track_le_mans_sarthe()),
            "sahara" | "sahara_dunes" => Some(tdrace_core::track::presets::sahara_dunes()),
            "dirt_figure_eight" | "dirt_eight" => Some(tdrace_core::track::presets::dirt_figure_eight()),
            "holjes_rx" | "holjes" => Some(tdrace_core::track::presets::holjes_rx()),
            "lydden_hill" | "lydden" => Some(tdrace_core::track::presets::lydden_hill()),
            "hell_rx" | "hell" => Some(tdrace_core::track::presets::hell_rx()),
            "loheac_rx" | "loheac" => Some(tdrace_core::track::presets::loheac_rx()),
            "estering_rx" | "estering" => Some(tdrace_core::track::presets::estering_rx()),
            "montalegre_rx" | "montalegre" => Some(tdrace_core::track::presets::montalegre_rx()),
            "nyirad_rx" | "nyirad" => Some(tdrace_core::track::presets::nyirad_rx()),
            "kouvola_rx" | "kouvola" => Some(tdrace_core::track::presets::kouvola_rx()),
            "catalunya_rx" => Some(tdrace_core::track::presets::catalunya_rx()),
            "mettet_rx" | "mettet" => Some(tdrace_core::track::presets::mettet_rx()),
            "silverstone_rx" => Some(tdrace_core::track::presets::silverstone_rx()),
            "riga_rx" | "riga" | "bikernieki" => Some(tdrace_core::track::presets::riga_rx()),
            "killarney_rx" | "killarney" => Some(tdrace_core::track::presets::killarney_rx()),
            "yas_marina_rx" | "yas_marina" => Some(tdrace_core::track::presets::yas_marina_rx()),
            "essay_rx" | "essay" => Some(tdrace_core::track::presets::essay_rx()),
            "lonato" => Some(crate::module::kart::KartGameModule::track_lonato()),
            "sarno" => Some(crate::module::kart::KartGameModule::track_sarno()),
            "genk" => Some(crate::module::kart::KartGameModule::track_genk()),
            "pfi" => Some(crate::module::kart::KartGameModule::track_pfi()),
            "zuera" => Some(crate::module::kart::KartGameModule::track_zuera()),
            "le_mans_kart" => Some(crate::module::kart::KartGameModule::track_le_mans()),
            "portimao_kart" => Some(crate::module::kart::KartGameModule::track_portimao()),
            "franciacorta" => Some(crate::module::kart::KartGameModule::track_franciacorta()),
            "wackersdorf" | "prokart_wackersdorf" => Some(crate::module::kart::KartGameModule::track_wackersdorf()),
            "kristianstad" | "asum_ring" => Some(crate::module::kart::KartGameModule::track_kristianstad()),
            "seven_laghi" | "7laghi" | "castelletto_kart" | "castelletto" => Some(crate::module::kart::KartGameModule::track_seven_laghi()),
            "ampfing" | "schweppermannring" => Some(crate::module::kart::KartGameModule::track_ampfing()),
            "silverstone_national_kart" | "silverstone_kart" => Some(crate::module::kart::KartGameModule::track_silverstone_national_kart()),
            "valencia_kart" | "valencia" => Some(crate::module::kart::KartGameModule::track_valencia_kart()),
            "campillos" => Some(crate::module::kart::KartGameModule::track_campillos()),
            "daytona" | "daytona_superspeedway" => Some(tdrace_core::track::presets::daytona_superspeedway()),
            "talladega" | "talladega_superspeedway" => Some(tdrace_core::track::presets::talladega_superspeedway()),
            "watkins_glen" | "watkins_glen_nascar" => Some(tdrace_core::track::presets::watkins_glen_nascar()),
            "bristol" | "bristol_motor_speedway" => Some(tdrace_core::track::presets::bristol_motor_speedway()),
            "martinsville" | "martinsville_speedway" => Some(tdrace_core::track::presets::martinsville_speedway()),
            "darlington" | "darlington_raceway" => Some(tdrace_core::track::presets::darlington_raceway()),
            "charlotte" | "charlotte_motor_speedway" => Some(tdrace_core::track::presets::charlotte_motor_speedway()),
            "indianapolis" | "indianapolis_motor_speedway" => Some(tdrace_core::track::presets::indianapolis_motor_speedway()),
            "eldora" | "eldora_speedway" => Some(tdrace_core::track::presets::eldora_speedway()),
            "iowa" | "iowa_speedway" => Some(tdrace_core::track::presets::iowa_speedway()),
            "road_america" => Some(tdrace_core::track::presets::road_america()),
            "chicago" | "chicago_street_course" => Some(tdrace_core::track::presets::chicago_street_course()),
            "sahara_dune_crossing" => Some(tdrace_core::track::presets::sahara_dune_crossing()),
            "atacama_sand_basin" | "atacama" => Some(tdrace_core::track::presets::atacama_sand_basin()),
            "red_rock_canyon" | "red_rock" => Some(tdrace_core::track::presets::red_rock_canyon()),
            "baja_500_desert_scrub" | "baja_500" | "baja" => Some(tdrace_core::track::presets::baja_500_desert_scrub()),
            "mud_slough_arena" | "mud_slough" => Some(tdrace_core::track::presets::mud_slough_arena()),
            "gravel_quarry_chasm" | "gravel_quarry" => Some(tdrace_core::track::presets::gravel_quarry_chasm()),
            "louisiana_mud_swampland" | "louisiana_swampland" | "louisiana" => Some(tdrace_core::track::presets::louisiana_mud_swampland()),
            "arctic_frozen_lake" | "frozen_lake" => Some(tdrace_core::track::presets::arctic_frozen_lake()),
            "alpine_snow_ridge" | "alpine_snow" => Some(tdrace_core::track::presets::alpine_snow_ridge()),
            "rovaniemi_ice_ring" | "rovaniemi" => Some(tdrace_core::track::presets::rovaniemi_ice_ring()),
            "glacier_crest_pass" | "glacier_crest" => Some(tdrace_core::track::presets::glacier_crest_pass()),
            "supercross_stadium_arena" | "supercross_stadium" | "supercross" => Some(tdrace_core::track::presets::supercross_stadium_arena()),
            "monster_colosseum" => Some(tdrace_core::track::presets::monster_colosseum()),
            "stunt_city_megastructure" | "stunt_city" => Some(tdrace_core::track::presets::stunt_city_megastructure()),
            _ => None,
        },
    }
}
}

/// Available vehicle model options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CarChoice {
    SportsCar,
    DriftCar,
    Kart,
    RallyCar,
    GT4Clubsport,
    GT3Car,
    GT2Biturbo,
    GT1Legend,
    HypercarPrototype,
    StockCar,
    SandRail,
}

impl CarChoice {
    pub const ALL: [Self; 11] = [
        Self::SportsCar,
        Self::DriftCar,
        Self::Kart,
        Self::RallyCar,
        Self::GT4Clubsport,
        Self::GT3Car,
        Self::GT2Biturbo,
        Self::GT1Legend,
        Self::HypercarPrototype,
        Self::StockCar,
        Self::SandRail,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::SportsCar => "GT Sports Coupe",
            Self::DriftCar => "Tuned Drift Spec",
            Self::Kart => "125cc Shifter Kart",
            Self::RallyCar => "AWD Turbo Rally",
            Self::GT4Clubsport => "420 BHP GT4 Clubsport",
            Self::GT3Car => "600 BHP GT3 Evo Racer",
            Self::GT2Biturbo => "707 BHP GT2 Biturbo Sprint",
            Self::GT1Legend => "650 BHP GT1 Le Mans Legend",
            Self::HypercarPrototype => "800 BHP LMH Hypercar Prototype",
            Self::StockCar => "850 BHP NASCAR Cup V8",
            Self::SandRail => "300 BHP Sand Rail Buggy",
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            Self::SportsCar => "BALANCED RWD",
            Self::DriftCar => "PRO SLIDE",
            Self::Kart => "APEX GRIP",
            Self::RallyCar => "AWD ALL-TERRAIN",
            Self::GT4Clubsport => "GT4 ENTRY SPEC",
            Self::GT3Car => "FIA GT3 SPEC",
            Self::GT2Biturbo => "SRO GT2 SPRINT",
            Self::GT1Legend => "90s GT1 LEGEND",
            Self::HypercarPrototype => "LE MANS HYPERCAR",
            Self::StockCar => "850 BHP SPACEFRAME V8",
            Self::SandRail => "300 BHP RWD ULTRALIGHT",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SportsCar => "Balanced RWD arcade dynamics, responsive rack, 208 km/h top speed.",
            Self::DriftCar => "High-power slide machine with loose rear, wide lock & snappy counter-steer.",
            Self::Kart => "Ultra-lightweight direct steering with extreme apex cornering grip.",
            Self::RallyCar => "All-wheel-drive traction with compliant suspension for mixed surfaces.",
            Self::GT4Clubsport => "Agile 420 BHP lightweight RWD racer, agile cornering, gentle aero (Cl=0.85).",
            Self::GT3Car => "4.0L V8, 600 BHP, high aerodynamic downforce (Cl=2.1), carbon brakes, ABS & TC.",
            Self::GT2Biturbo => "High-power 707 BHP biturbo straight-line missile, 328 km/h top speed, lower downforce (Cl=1.4).",
            Self::GT1Legend => "Raw 650 BHP twin-turbo beast with high downforce (Cl=2.60) and pure analog handling (zero electronic assists).",
            Self::HypercarPrototype => "Cutting-edge 800 BHP hybrid prototype with ground-effect aero tunnels (Cl=3.10) and hybrid boost.",
            Self::StockCar => "High-compression 5.9L pushrod V8, 850 BHP, 1260 kg, quick-ratio steering, 320 km/h superspeedway pack racer.",
            Self::SandRail => "Ultralight chromoly tube chassis, 300 BHP rear turbo boxer, paddle tires, and long-travel off-road suspension.",
        }
    }

    /// Returns normalized stat ratings: (Speed, Acceleration, Grip, Drift) [0.0..1.0]
    pub fn stats(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::SportsCar => (0.85, 0.80, 0.75, 0.65),
            Self::DriftCar => (0.80, 0.85, 0.50, 0.98),
            Self::Kart => (0.65, 0.95, 0.95, 0.40),
            Self::RallyCar => (0.78, 0.90, 0.85, 0.75),
            Self::GT4Clubsport => (0.78, 0.82, 0.85, 0.70),
            Self::GT3Car => (0.92, 0.94, 0.95, 0.50),
            Self::GT2Biturbo => (0.96, 0.97, 0.89, 0.65),
            Self::GT1Legend => (0.98, 0.98, 0.93, 0.40),
            Self::HypercarPrototype => (0.99, 0.99, 0.98, 0.35),
            Self::StockCar => (0.97, 0.90, 0.86, 0.88),
            Self::SandRail => (0.88, 0.96, 0.82, 0.94),
        }
    }

    /// Returns key engineering and dynamic specifications: (Drivetrain, Mass, Top Speed, Aero/Handling)
    pub fn specs(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::SportsCar => ("RWD Drivetrain", "1,180 kg Mass", "208 km/h Top Speed", "Cl 0.65 Downforce"),
            Self::DriftCar => ("RWD Drift Spec", "980 kg Mass", "45° Wide Drift Lock", "High-Slip Balance"),
            Self::Kart => ("Direct Rear Axle", "180 kg Mass", "115 km/h Top Speed", "1:1 Direct Rack"),
            Self::RallyCar => ("AWD 50:50 Split", "1,240 kg Mass", "Long-Travel Setup", "Cl 0.70 Downforce"),
            Self::GT4Clubsport => ("RWD GT4 Spec", "1,320 kg Mass", "272 km/h Top Speed", "Cl 0.85 Downforce"),
            Self::GT3Car => ("RWD GT3 Spec", "1,260 kg Mass", "297 km/h Top Speed", "Cl 2.10 Downforce"),
            Self::GT2Biturbo => ("RWD GT2 Spec", "1,390 kg Mass", "328 km/h Top Speed", "Cl 1.40 Downforce"),
            Self::GT1Legend => ("RWD GT1 Analog", "1,120 kg Mass", "335 km/h Top Speed", "Cl 2.60 Downforce"),
            Self::HypercarPrototype => ("Hybrid Ground-Effect", "1,030 kg Mass", "342 km/h Top Speed", "Cl 3.10 Downforce"),
            Self::StockCar => ("RWD Spaceframe V8", "1,260 kg Mass", "320 km/h Top Speed", "Pack Draft Dynamic"),
            Self::SandRail => ("RWD Long-Travel", "680 kg Mass", "215 km/h Top Speed", "Paddle Sand Tires"),
        }
    }

    /// Returns the GT career unlock level required for this car (Level 1-5).
    pub fn unlock_level(&self) -> u32 {
        match self {
            Self::GT4Clubsport => 1,
            Self::GT3Car => 2,
            Self::GT2Biturbo => 3,
            Self::GT1Legend => 4,
            Self::HypercarPrototype => 5,
            _ => 1,
        }
    }

    /// Returns the motorsport category tier for this vehicle choice (Tier 1..=5).
    pub fn tier(&self) -> u8 {
        match self {
            Self::GT4Clubsport | Self::SportsCar | Self::SandRail => 1,
            Self::GT3Car | Self::RallyCar | Self::DriftCar => 2,
            Self::GT2Biturbo | Self::Kart => 3,
            Self::GT1Legend => 4,
            Self::HypercarPrototype | Self::StockCar => 5,
        }
    }

    /// Checks whether this vehicle is eligible for a race that requires `required_tier`.
    /// Rule: Selectable iff car.tier <= required_tier || dev_mode.
    #[inline]
    pub fn is_eligible_for_race_tier(&self, required_tier: u8, dev_mode: bool) -> bool {
        dev_mode || self.tier() <= required_tier
    }

    /// Returns the vehicle physics specification for this vehicle choice.
    pub fn config(&self) -> CarConfig {
        match self {
            Self::SportsCar => CarConfig::sports_car(),
            Self::DriftCar => CarConfig::drift_car(),
            Self::Kart => CarConfig::kart(),
            Self::RallyCar => CarConfig::rally_car(),
            Self::GT4Clubsport => crate::module::gt::GtWorldChallengeModule::car_gt4_clubsport(),
            Self::GT3Car => crate::module::gt::GtWorldChallengeModule::car_gt3_evo(),
            Self::GT2Biturbo => crate::module::gt::GtWorldChallengeModule::car_gt2_biturbo(),
            Self::GT1Legend => crate::module::gt::GtWorldChallengeModule::car_gt1_legend(),
            Self::HypercarPrototype => crate::module::gt::GtWorldChallengeModule::car_hypercar_prototype(),
            Self::StockCar => CarConfig::stock_car_ta1(),
            Self::SandRail => CarConfig::sand_rail(),
        }
    }

    /// Returns the visual rendering archetype for this vehicle choice.
    pub fn visual_type(&self) -> crate::module::VehicleVisualType {
        match self {
            Self::GT4Clubsport => crate::module::VehicleVisualType::TouringGT {
                widebody: false,
                gt_wing: true,
                diffuser: false,
            },
            Self::GT3Car
            | Self::GT2Biturbo
            | Self::GT1Legend
            | Self::HypercarPrototype => crate::module::VehicleVisualType::TouringGT {
                widebody: true,
                gt_wing: true,
                diffuser: true,
            },
            Self::RallyCar => crate::module::VehicleVisualType::RallyHatch {
                roof_scoop: true,
                mudflaps: true,
                large_wing: true,
            },
            Self::Kart => crate::module::VehicleVisualType::GoKart {
                exposed_driver: true,
                side_bumpers: true,
            },
            Self::StockCar => crate::module::VehicleVisualType::StockCar {
                tall_wing: true,
                roof_fins: true,
                window_net: true,
            },
            Self::SandRail => crate::module::VehicleVisualType::SandRail {
                lightbar: true,
                whip_antenna: true,
                paddle_tires: true,
            },
            Self::SportsCar | Self::DriftCar => crate::module::VehicleVisualType::TouringGT {
                widebody: false,
                gt_wing: false,
                diffuser: false,
            },
        }
    }
}

/// Resolves the authentic predefined car for a specific track and active module context.
pub fn resolve_predefined_car_for_track(track: Option<&tdrace_core::track::Track>, module_id: &str) -> CarChoice {
    if let Some(tr) = track {
        match tr.predefined_car.as_deref() {
            Some("gt4" | "gt4_clubsport") => CarChoice::GT4Clubsport,
            Some("gt3" | "gt3_car" | "gt3_evo") => CarChoice::GT3Car,
            Some("gt2" | "gt2_biturbo") => CarChoice::GT2Biturbo,
            Some("gt1" | "gt1_legend") => CarChoice::GT1Legend,
            Some("hypercar" | "hypercar_prototype" | "lmh" | "lmdh") => CarChoice::HypercarPrototype,
            Some("gt") => CarChoice::GT4Clubsport,
            Some("open_wheel") => CarChoice::Kart,
            Some("drift_car") => CarChoice::DriftCar,
            Some("kart" | "shifter_kart" | "shifter_kart_125" | "classic_kart") => CarChoice::Kart,
            Some("rally_car" | "wrc_turbo_rally" | "rally" | "classic_rally") => CarChoice::RallyCar,
            Some("nascar" | "nascar_cup" | "nascar_cup_v8" | "stock_car" | "trans_am" | "trans_am_ta1" | "ta1" | "classic_nascar") => CarChoice::StockCar,
            Some("sand_rail" | "sand_rail_buggy" | "buggy" | "classic_offroad") => CarChoice::SandRail,
            Some("sports_car" | "classic_gt") => CarChoice::SportsCar,
            _ => match tr.module_id.as_deref().unwrap_or(module_id) {
                "gt" | "gt_challenge" => CarChoice::GT4Clubsport,
                "rally" => CarChoice::RallyCar,
                "kart" => CarChoice::Kart,
                "nascar" => CarChoice::StockCar,
                "extreme_offroad" => CarChoice::SandRail,
                _ => CarChoice::SportsCar,
            },
        }
    } else {
        match module_id {
            "gt" | "gt_challenge" => CarChoice::GT4Clubsport,
            "rally" => CarChoice::RallyCar,
            "kart" => CarChoice::Kart,
            "nascar" => CarChoice::StockCar,
            "extreme_offroad" => CarChoice::SandRail,
            _ => CarChoice::SportsCar,
        }
    }
}

/// Racing game modes supported across single-player practice, time trial, and grid racing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Standard Race: all drivers use the circuit's predefined car.
    StandardRace,
    /// Career Mode: 5-tier GT championship campaign with XP, level progression, and car/circuit unlocks.
    Career,
    /// Experimental Race: all drivers use the car model specified by the user. Allows changing car.
    ExperimentalRace,
    /// Split Screen: 2 simultaneous local players (P1 on Keyboard vs P2 on Gamepad).
    SplitScreen,
    /// Time Trial: race against your personal best time, shown as a shadow / ghost car. Allows changing car.
    TimeTrial,
    /// Free Ride: solo practice to test the circuit or/and the car. Allows changing car.
    FreeRide,
}

pub type GameModeChoice = GameMode;

impl GameMode {
    pub const ALL: [Self; 6] = [
        Self::StandardRace,
        Self::Career,
        Self::ExperimentalRace,
        Self::SplitScreen,
        Self::TimeTrial,
        Self::FreeRide,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::StandardRace => "Standard Race",
            Self::Career => "Career Mode",
            Self::ExperimentalRace => "Experimental Race",
            Self::SplitScreen => "2P Split Screen",
            Self::TimeTrial => "Time Trial",
            Self::FreeRide => "Free Ride",
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            Self::StandardRace => "PREDEFINED CAR • GRID",
            Self::Career => "5-TIER CAMPAIGN • XP & UNLOCKS",
            Self::ExperimentalRace => "CUSTOM CAR SPEC • MULTI-CAR",
            Self::SplitScreen => "LOCAL 2-PLAYER • KEYS VS GAMEPAD",
            Self::TimeTrial => "VS GHOST SHADOW CAR",
            Self::FreeRide => "SOLO PRACTICE & TUNING",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::StandardRace => "All drivers compete using the circuit's official predefined car.",
            Self::Career => "5-tier GT championship campaign. Earn XP across official cups to unlock vehicles and circuits.",
            Self::ExperimentalRace => "All drivers compete using the car model specified by the player.",
            Self::SplitScreen => "Simultaneous 2-player split screen: Player 1 on Keyboard vs Player 2 on Gamepad.",
            Self::TimeTrial => "Race against your personal best time shown as a shadow car.",
            Self::FreeRide => "Solo open practice to freely test the circuit and vehicle handling.",
        }
    }

    pub fn allows_car_change(&self) -> bool {
        match self {
            Self::StandardRace | Self::Career => false,
            Self::ExperimentalRace | Self::SplitScreen | Self::TimeTrial | Self::FreeRide => true,
        }
    }

    pub fn has_bots(&self) -> bool {
        match self {
            Self::StandardRace | Self::Career | Self::ExperimentalRace | Self::SplitScreen => true,
            Self::TimeTrial | Self::FreeRide => false,
        }
    }

    pub fn is_time_attack(&self) -> bool {
        match self {
            Self::TimeTrial | Self::FreeRide => true,
            Self::StandardRace | Self::Career | Self::ExperimentalRace | Self::SplitScreen => false,
        }
    }

    pub fn has_ghost(&self) -> bool {
        matches!(self, Self::TimeTrial)
    }

    pub fn is_split_screen(&self) -> bool {
        matches!(self, Self::SplitScreen)
    }

    pub fn is_career(&self) -> bool {
        matches!(self, Self::Career)
    }

    pub fn next(&self) -> Self {
        match self {
            Self::StandardRace => Self::Career,
            Self::Career => Self::ExperimentalRace,
            Self::ExperimentalRace => Self::SplitScreen,
            Self::SplitScreen => Self::TimeTrial,
            Self::TimeTrial => Self::FreeRide,
            Self::FreeRide => Self::StandardRace,
        }
    }
}

/// Standings entry for results screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResultEntry {
    pub position: usize,
    pub car_name: String,
    pub is_player: bool,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub delta_to_leader: f32,
}

use crate::profile::{ModuleCareerProgress, PlayerProfile, ProfileCareerStats};
use super::profile_ui::render_profile_badge;

/// Selected focus column/panel in the Track & Setup Selection Menu (kept for backwards compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuPanelFocus {
    LeftTracks,
    RightVehicle,
}

/// Catalog filter tabs in the Circuit Selection Menu (Presets or Custom circuits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackCatalogFilter {
    #[default]
    Presets,
    Custom,
}

impl TrackCatalogFilter {
    pub fn next(self) -> Self {
        match self {
            Self::Presets => Self::Custom,
            Self::Custom => Self::Presets,
        }
    }

    pub fn prev(self) -> Self {
        self.next()
    }
}

/// Renders the modern Track & Setup Selection Menu with glass cards and vector typography.
#[allow(clippy::too_many_arguments)]
pub fn render_track_select_menu(
    fonts: &Fonts,
    active_module_id: &str,
    module_title: &str,
    module_subtitle: &str,
    module_accent: Color,
    available_tracks: &[TrackChoice],
    selected_track_idx: usize,
    active_profile: &PlayerProfile,
    active_stats: &ProfileCareerStats,
    active_filter: TrackCatalogFilter,
    filter_counts: (usize, usize),
    career_progress: Option<&ModuleCareerProgress>,
    dev_mode: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep modern motorsport gradient backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.05, 0.06, 0.09, 0.98));

    // Header Title
    fonts.draw_display_centered_with_shadow(
        module_title,
        sw * 0.5,
        scaler.s(32.0),
        scaler.font_s(32.0),
        module_accent,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let sub_str = format!("{} • [ESC] Return to Grand Hub", module_subtitle);
    fonts.draw_ui_regular_centered(
        &sub_str,
        sw * 0.5,
        scaler.s(52.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Columns geometry
    let col_w = (sw * 0.40).clamp(scaler.s(320.0), scaler.s(480.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(16.0)).max(scaler.safe_pad_x);
    let col2_x = (sw * 0.5 + scaler.s(16.0)).min(sw - col_w - scaler.safe_pad_x);

    // Active Profile Badge Banner
    let badge_w = col_w * 2.0 + scaler.s(32.0);
    let badge_x = col1_x;
    let badge_y = scaler.s(62.0);
    let badge_h = scaler.s(48.0);
    render_profile_badge(fonts, &scaler, badge_x, badge_y, badge_w, badge_h, active_profile, active_stats);

    // Optional Career Progression Bar Banner
    let cp_h = if career_progress.is_some() { scaler.s(26.0) } else { 0.0 };
    if let Some(cp) = career_progress {
        let cp_y = badge_y + badge_h + scaler.s(4.0);
        scaler.draw_glass_card(badge_x, cp_y, badge_w, cp_h, Color::new(0.06, 0.08, 0.12, 0.92), Palette::NEON_CYAN, 1.2);

        // Driver Level Tag
        let lvl_str = format!("CAREER TIER {}", cp.level);
        fonts.draw_ui_bold(&lvl_str, badge_x + scaler.s(12.0), cp_y + scaler.s(17.0), scaler.font_s(12.0), Palette::NEON_GOLD);

        // Spendable XP text
        let xp_str = if let Some(target) = cp.next_tier_target_xp() {
            format!("XP: {} / {} (NEXT TIER CAR)", cp.xp, target)
        } else {
            format!("XP: {} (MAX TIER)", cp.xp)
        };
        fonts.draw_ui_regular(&xp_str, badge_x + scaler.s(130.0), cp_y + scaler.s(17.0), scaler.font_s(11.0), Palette::WHITE);

        // Progress bar in center
        let bar_x = badge_x + scaler.s(315.0);
        let bar_y = cp_y + scaler.s(7.0);
        let bar_w = (badge_w - scaler.s(580.0)).clamp(scaler.s(60.0), scaler.s(220.0));
        let bar_h = scaler.s(11.0);
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.08, 0.12, 0.18, 0.95));
        draw_rectangle(bar_x, bar_y, bar_w * cp.level_progress_ratio(), bar_h, Palette::NEON_CYAN);
        draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, Palette::NEON_CYAN);

        // Championship Podiums / Trophies
        let trophy_str = format!("PODIUMS  G:{} S:{} B:{}", cp.trophies_gold, cp.trophies_silver, cp.trophies_bronze);
        fonts.draw_ui_bold(&trophy_str, bar_x + bar_w + scaler.s(12.0), cp_y + scaler.s(17.0), scaler.font_s(10.5), Palette::NEON_GOLD);

        // Mode indicator on right
        let (mode_tag, tag_col) = if dev_mode {
            ("[DEV MODE: ALL UNLOCKED]", Palette::NEON_MAGENTA)
        } else if cp.can_advance_tier() {
            ("★ TIER ADVANCEMENT READY!", Palette::NEON_GOLD)
        } else {
            ("CAREER PROGRESSION ACTIVE", Palette::NEON_GREEN)
        };
        fonts.draw_ui_bold(mode_tag, badge_x + badge_w - scaler.s(210.0), cp_y + scaler.s(17.0), scaler.font_s(10.5), tag_col);
    }

    // Spacing between Profile/Career Panel and Catalog columns
    let menu_content_y = badge_y + badge_h + cp_h + scaler.s(14.0);

    // Left Column: Track Selection Cards & Filter Tabs
    let mut curr_y = menu_content_y;

    fonts.draw_ui_bold(
        "CIRCUIT CATALOG [Up/Down to select]",
        col1_x,
        curr_y + scaler.s(13.0),
        scaler.font_s(15.0),
        module_accent,
    );

    let tm_badge_w = scaler.s(165.0);
    let tm_badge_h = scaler.s(22.0);
    let tm_badge_x = col1_x + col_w - tm_badge_w;
    scaler.draw_glass_card(
        tm_badge_x,
        curr_y - scaler.s(2.0),
        tm_badge_w,
        tm_badge_h,
        Color::new(0.18, 0.08, 0.30, 0.90),
        Palette::NEON_MAGENTA,
        1.2,
    );
    fonts.draw_ui_bold_centered(
        "[T] CIRCUIT MANAGER",
        tm_badge_x + tm_badge_w * 0.5,
        curr_y + scaler.s(13.0),
        scaler.font_s(10.5),
        Palette::NEON_GOLD,
    );

    curr_y += scaler.s(20.0);

    // Filter Tabs: [ OFFICIAL (P) ]  [ CUSTOM (C) ]
    let tab_h = scaler.s(25.0);
    let tab_gap = scaler.s(6.0);
    let filter_tabs = [
        (TrackCatalogFilter::Presets, format!("OFFICIAL [{}]", filter_counts.0)),
        (TrackCatalogFilter::Custom, format!("CUSTOM [{}]", filter_counts.1)),
    ];
    let tab_count = filter_tabs.len() as f32;
    let tab_w = (col_w - tab_gap * (tab_count - 1.0)) / tab_count;

    for (i, (tab_variant, tab_label)) in filter_tabs.iter().enumerate() {
        let tab_x = col1_x + (tab_w + tab_gap) * (i as f32);
        let is_tab_active = *tab_variant == active_filter;
        let tab_bg = if is_tab_active {
            Color::new(0.08, 0.28, 0.40, 0.95)
        } else {
            Palette::UI_CARD_BG
        };
        let tab_border = if is_tab_active {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };
        let text_col = if is_tab_active {
            Palette::WHITE
        } else {
            Palette::UI_TEXT_MUTED
        };
        scaler.draw_glass_card(tab_x, curr_y, tab_w, tab_h, tab_bg, tab_border, if is_tab_active { 1.8 } else { 1.0 });
        fonts.draw_ui_bold_centered(
            tab_label,
            tab_x + tab_w * 0.5,
            curr_y + scaler.s(16.0),
            scaler.font_s(10.5),
            text_col,
        );
    }
    curr_y += tab_h + scaler.s(8.0);

    let total_tracks = available_tracks.len();
    let has_tm_entry = active_filter == TrackCatalogFilter::Custom;
    let total_items = total_tracks + if has_tm_entry { 1 } else { 0 };

    if total_items == 0 {
        let empty_h = scaler.s(150.0);
        scaler.draw_glass_card(col1_x, curr_y, col_w, empty_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);
        fonts.draw_ui_bold_centered(
            "No Official Circuits Found",
            col1_x + col_w * 0.5,
            curr_y + scaler.s(55.0),
            scaler.font_s(15.0),
            Palette::WHITE,
        );
        fonts.draw_ui_regular_centered(
            "Press [Left / Right] to switch category",
            col1_x + col_w * 0.5,
            curr_y + scaler.s(80.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        let btn_h = scaler.s(40.0);
        let btn_y = sh - btn_h - scaler.s(14.0);
        let available_catalog_h = (btn_y - curr_y - scaler.s(24.0)).max(scaler.s(180.0));
        let max_visible = ((available_catalog_h / scaler.s(64.0)).floor() as usize).clamp(4, 7);

        let start_idx = if total_items <= max_visible {
            0
        } else {
            selected_track_idx
                .saturating_sub(max_visible / 2)
                .min(total_items - max_visible)
        };
        let end_idx = (start_idx + max_visible).min(total_items);

        for i in start_idx..end_idx {
            let is_sel = i == selected_track_idx;
            let box_h = scaler.s(58.0);

            if i < total_tracks {
                let track_opt = &available_tracks[i];
                let loaded_track = resolve_track_for_menu(track_opt);
                let is_locked = if let Some(cp) = career_progress {
                    !cp.is_track_unlocked(track_opt.track_id(), dev_mode)
                } else {
                    false
                };

                let bg_col = if is_locked {
                    if is_sel {
                        Color::new(0.20, 0.06, 0.06, 0.95)
                    } else {
                        Color::new(0.08, 0.04, 0.04, 0.85)
                    }
                } else if is_sel {
                    Palette::UI_CARD_BG_HOVER
                } else {
                    Palette::UI_CARD_BG
                };
                let border_col = if is_locked {
                    if is_sel {
                        Palette::RED
                    } else {
                        Color::new(0.45, 0.15, 0.15, 0.70)
                    }
                } else if is_sel {
                    module_accent
                } else {
                    Palette::UI_CARD_BORDER
                };

                scaler.draw_glass_card(col1_x, curr_y, col_w, box_h, bg_col, border_col, if is_sel { 2.2 } else { 1.2 });

                // Small Track Vector Thumbnail on right side of card
                let thumb_w = scaler.s(58.0);
                let thumb_h = scaler.s(44.0);
                let thumb_x = col1_x + col_w - thumb_w - scaler.s(8.0);
                let thumb_y = curr_y + scaler.s(7.0);

                if let Some(ref tr) = loaded_track {
                    super::track_preview::render_track_thumbnail(&scaler, thumb_x, thumb_y, thumb_w, thumb_h, tr, is_sel);
                }

                // Tag pill & metrics badge (Length + Surface breakdown)
                let is_custom = track_opt.is_user_custom();
                let (tag_label, tag_col) = if is_locked {
                    ("🔒 LOCKED • ADVANCE CAREER LEVEL".to_string(), Palette::RED)
                } else if is_custom {
                    let lbl = if let Some(ref tr) = loaded_track {
                        format!("CUSTOM CIRCUIT • {:.0}m • {}", tr.total_length_m(), tr.surface_summary_string())
                    } else {
                        "CUSTOM CIRCUIT".to_string()
                    };
                    (lbl, Palette::NEON_GOLD)
                } else {
                    let tag_prefix = track_opt.tag_for_module(active_module_id);
                    let lbl = if let Some(ref tr) = loaded_track {
                        format!("{} • {:.0}m • {}", tag_prefix, tr.total_length_m(), tr.surface_summary_string())
                    } else {
                        tag_prefix.to_string()
                    };
                    (lbl, if is_sel { module_accent } else { Palette::UI_TEXT_MUTED })
                };
                fonts.draw_ui_bold(
                    &tag_label,
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(16.0),
                    scaler.font_s(10.0),
                    tag_col,
                );

                // Track title
                let (title_str, title_col) = if is_locked {
                    (format!("🔒 {}", track_opt.title()), if is_sel { Color::new(1.0, 0.75, 0.75, 1.0) } else { Color::new(0.70, 0.50, 0.50, 0.85) })
                } else {
                    (track_opt.title().to_string(), if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) })
                };
                fonts.draw_ui_bold(
                    &title_str,
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(34.0),
                    scaler.font_s(15.5),
                    title_col,
                );

                // Description
                fonts.draw_ui_regular(
                    track_opt.description(),
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(49.0),
                    scaler.font_s(10.5),
                    if is_locked { Color::new(0.60, 0.45, 0.45, 0.70) } else { Palette::UI_TEXT_MUTED },
                );
            } else {
                // Dedicated Track Manager Card with distinct purple / magenta theme
                let tm_bg = if is_sel {
                    Color::new(0.32, 0.12, 0.52, 0.95)
                } else {
                    Color::new(0.18, 0.08, 0.30, 0.88)
                };
                let tm_border = if is_sel {
                    Palette::NEON_GOLD
                } else {
                    Palette::NEON_MAGENTA
                };

                scaler.draw_glass_card(col1_x, curr_y, col_w, box_h, tm_bg, tm_border, if is_sel { 2.4 } else { 1.5 });

                fonts.draw_ui_bold(
                    "CIRCUIT MANAGER [T]",
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(16.0),
                    scaler.font_s(10.5),
                    if is_sel { Palette::NEON_GOLD } else { Palette::NEON_MAGENTA },
                );

                fonts.draw_ui_bold(
                    "Circuit Manager",
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(34.0),
                    scaler.font_s(15.5),
                    Palette::WHITE,
                );

                fonts.draw_ui_regular(
                    "Manage circuits, workshop drafts, clone & edit. Press [T]",
                    col1_x + scaler.s(14.0),
                    curr_y + scaler.s(49.0),
                    scaler.font_s(10.5),
                    if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
                );
            }

            curr_y += box_h + scaler.s(6.0);
        }
    }


    // Right Column: Circuit Dossier & Predefined Vehicle Specs
    let mut c2_y = menu_content_y;

    let is_sel_locked = if selected_track_idx < total_tracks {
        let track_opt = &available_tracks[selected_track_idx];
        if let Some(cp) = career_progress {
            !cp.is_track_unlocked(track_opt.track_id(), dev_mode)
        } else {
            false
        }
    } else {
        false
    };

    if selected_track_idx < total_tracks {
        let track_opt = &available_tracks[selected_track_idx];
        let loaded_track = resolve_track_for_menu(track_opt);
        let predefined_car = resolve_predefined_car_for_track(loaded_track.as_ref(), active_module_id);
        let tr_ref = loaded_track.as_ref();

        // Section 1 Header: Circuit Dossier & Vector Telemetry
        fonts.draw_ui_bold(
            "CIRCUIT DOSSIER & TELEMETRY",
            col2_x,
            c2_y + scaler.s(13.0),
            scaler.font_s(15.0),
            if is_sel_locked { Palette::RED } else { module_accent },
        );
        c2_y += scaler.s(22.0);

        if is_sel_locked {
            scaler.draw_glass_card(col2_x, c2_y, col_w, scaler.s(22.0), Color::new(0.30, 0.08, 0.08, 0.90), Palette::RED, 1.2);
            fonts.draw_ui_bold_centered(
                "🔒 CIRCUIT LOCKED — ADVANCE CAREER LEVEL TO UNLOCK",
                col2_x + col_w * 0.5,
                c2_y + scaler.s(15.0),
                scaler.font_s(11.0),
                Palette::WHITE,
            );
            c2_y += scaler.s(26.0);
        }

        // Detailed Vector Map Preview Card
        let preview_h = scaler.s(145.0);
        if let Some(tr) = tr_ref {
            super::track_preview::render_track_detailed_preview(fonts, &scaler, col2_x, c2_y, col_w, preview_h, tr);
        } else {
            scaler.draw_glass_card(col2_x, c2_y, col_w, preview_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);
            fonts.draw_ui_bold_centered(
                "Circuit telemetry loading...",
                col2_x + col_w * 0.5,
                c2_y + preview_h * 0.5,
                scaler.font_s(14.0),
                Palette::UI_TEXT_MUTED,
            );
        }
        c2_y += preview_h + scaler.s(8.0);

        // Circuit Specs & Metrics Glass Card
        let metrics_h = scaler.s(66.0);
        scaler.draw_glass_card(col2_x, c2_y, col_w, metrics_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

        // Circuit Name & Tag
        fonts.draw_ui_bold(
            track_opt.title(),
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(16.0),
            scaler.font_s(15.0),
            Palette::WHITE,
        );
        let right_tag = if track_opt.is_user_custom() {
            "CUSTOM CIRCUIT"
        } else {
            track_opt.tag_for_module(active_module_id)
        };
        let right_tag_col = if track_opt.is_user_custom() {
            Palette::NEON_GOLD
        } else {
            module_accent
        };
        fonts.draw_ui_bold(
            right_tag,
            col2_x + col_w - scaler.s(135.0),
            c2_y + scaler.s(16.0),
            scaler.font_s(10.0),
            right_tag_col,
        );

        // Circuit Description
        fonts.draw_ui_regular(
            track_opt.description(),
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(32.0),
            scaler.font_s(10.5),
            Palette::UI_TEXT_MUTED,
        );

        // Circuit Metrics Summary Row
        let len_str = if let Some(tr) = tr_ref {
            format!("{:.0}m", tr.total_length_m())
        } else {
            "N/A".to_string()
        };
        let laps_str = if let Some(tr) = tr_ref {
            format!("{} Laps", tr.default_laps)
        } else {
            "3 Laps".to_string()
        };
        let cps_str = if let Some(tr) = tr_ref {
            format!("{} Checkpoints", tr.checkpoints.len())
        } else {
            "Checkpoints".to_string()
        };
        let grid_str = if let Some(tr) = tr_ref {
            format!("{} Grid Slots", tr.grid_positions.len().min(8))
        } else {
            "8 Slots".to_string()
        };

        let metrics_summary = format!("{} • {} • {} • {}", len_str, laps_str, cps_str, grid_str);
        fonts.draw_ui_bold(
            &metrics_summary,
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(53.0),
            scaler.font_s(11.0),
            Palette::NEON_CYAN,
        );

        c2_y += metrics_h + scaler.s(12.0);

        // Section 2 Header: Predefined Vehicle Specifications (Informational preview)
        fonts.draw_ui_bold(
            "PREDEFINED VEHICLE SPECIFICATIONS",
            col2_x,
            c2_y + scaler.s(13.0),
            scaler.font_s(15.0),
            module_accent,
        );
        fonts.draw_ui_regular(
            "Circuit default vehicle • Select car & mode in Race Setup [Enter]",
            col2_x,
            c2_y + scaler.s(28.0),
            scaler.font_s(10.5),
            Palette::UI_TEXT_MUTED,
        );
        c2_y += scaler.s(34.0);

        // Predefined Car Glass Card
        let car_card_h = scaler.s(182.0);
        scaler.draw_glass_card(col2_x, c2_y, col_w, car_card_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.3);

        // Car Tag & Title
        fonts.draw_ui_bold(
            predefined_car.tag(),
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(16.0),
            scaler.font_s(10.5),
            Palette::NEON_GOLD,
        );
        fonts.draw_ui_bold(
            predefined_car.title(),
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(34.0),
            scaler.font_s(16.5),
            Palette::WHITE,
        );
        fonts.draw_ui_regular(
            predefined_car.description(),
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(49.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Performance Stat Bars (Speed, Acceleration, Grip, Drift)
        let (spd, acc, grip, drift) = predefined_car.stats();
        let stat_bar_w = col_w - scaler.s(28.0);
        let stat_base_x = col2_x + scaler.s(14.0);

        render_stat_bar_full(scaler, fonts, stat_base_x, c2_y + scaler.s(64.0), stat_bar_w, "SPEED", spd, Palette::NEON_CYAN);
        render_stat_bar_full(scaler, fonts, stat_base_x, c2_y + scaler.s(79.0), stat_bar_w, "ACCEL", acc, Palette::NEON_GOLD);
        render_stat_bar_full(scaler, fonts, stat_base_x, c2_y + scaler.s(94.0), stat_bar_w, "GRIP", grip, Palette::NEON_GREEN);
        render_stat_bar_full(scaler, fonts, stat_base_x, c2_y + scaler.s(109.0), stat_bar_w, "DRIFT", drift, Palette::NEON_MAGENTA);

        // Telemetry / Engineering Specs Badges
        let (spec1, spec2, spec3, spec4) = predefined_car.specs();
        let spec_chip_w = (col_w - scaler.s(36.0)) * 0.5;
        let spec_chip_h = scaler.s(22.0);
        let chip_y1 = c2_y + scaler.s(128.0);
        let chip_y2 = c2_y + scaler.s(153.0);

        // Chip 1 (Drivetrain)
        scaler.draw_glass_card(stat_base_x, chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
        fonts.draw_ui_bold(spec1, stat_base_x + scaler.s(8.0), chip_y1 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_CYAN);

        // Chip 2 (Mass)
        scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
        fonts.draw_ui_bold(spec2, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y1 + scaler.s(15.0), scaler.font_s(10.0), Palette::WHITE);

        // Chip 3 (Top Speed)
        scaler.draw_glass_card(stat_base_x, chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
        fonts.draw_ui_bold(spec3, stat_base_x + scaler.s(8.0), chip_y2 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_GOLD);

        // Chip 4 (Aero / Dynamics)
        scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
        fonts.draw_ui_bold(spec4, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y2 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_GREEN);
        if track_opt.is_user_custom() {
            let footer_btn_y = sh - scaler.s(40.0) - scaler.s(14.0);
            fonts.draw_ui_bold(
                "[T] Circuit Manager",
                col2_x,
                footer_btn_y - scaler.s(26.0),
                scaler.font_s(11.0),
                Palette::NEON_GOLD,
            );
        }
    } else if has_tm_entry && selected_track_idx == total_tracks {
        // Dedicated Circuit Manager / Studio view on right panel
        fonts.draw_ui_bold(
            "CIRCUIT MANAGER & WORKSHOP [T]",
            col2_x,
            c2_y + scaler.s(13.0),
            scaler.font_s(15.0),
            Palette::NEON_MAGENTA,
        );
        c2_y += scaler.s(22.0);

        let studio_h = scaler.s(220.0);
        scaler.draw_glass_card(col2_x, c2_y, col_w, studio_h, Palette::UI_CARD_BG, Palette::NEON_MAGENTA, 1.5);

        fonts.draw_ui_bold(
            "CIRCUIT MANAGER & CAD DESIGNER",
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(20.0),
            scaler.font_s(15.0),
            Palette::NEON_GOLD,
        );
        fonts.draw_ui_regular(
            "Create, edit, organize, clone, and test racing circuits with spline geometry, surface zoning, and module assignments.",
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(40.0),
            scaler.font_s(11.5),
            Palette::WHITE,
        );

        let features = [
            "• Spline CAD editor with elevation & banking bridges",
            "• Multi-surface painting (Asphalt, Dirt, Sand, Water, Ice)",
            "• Jump ramps, obstacles & custom checkpoint gates",
            "• Predefined car assignment & lap balancing",
            "• Press [Enter / Space] or [T] to launch Circuit Manager",
        ];
        let mut feat_y = c2_y + scaler.s(86.0);
        for feat in &features {
            fonts.draw_ui_regular(feat, col2_x + scaler.s(14.0), feat_y, scaler.font_s(11.0), Palette::NEON_CYAN);
            feat_y += scaler.s(18.0);
        }

        c2_y += studio_h + scaler.s(14.0);

        fonts.draw_ui_bold(
            "CIRCUIT MANAGEMENT FEATURES",
            col2_x,
            c2_y + scaler.s(13.0),
            scaler.font_s(15.0),
            Palette::NEON_GOLD,
        );
        c2_y += scaler.s(22.0);

        let actions_h = scaler.s(170.0);
        scaler.draw_glass_card(col2_x, c2_y, col_w, actions_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 1.3);

        fonts.draw_ui_bold(
            "Circuit Manager Overview",
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(20.0),
            scaler.font_s(15.0),
            Palette::WHITE,
        );
        fonts.draw_ui_regular(
            "Press [T] anywhere in the menu or press [Enter] on this card to open Circuit Manager directly.",
            col2_x + scaler.s(14.0),
            c2_y + scaler.s(38.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        let classes = [
            ("[T] Circuit Manager", "Full screen circuit organizer, drafts & file manager"),
            ("CAD Studio", "Direct spline vector circuit layout and surface designer"),
            ("Clone to Drafts", "Safely duplicate any built-in preset or custom circuit"),
            ("Module Distribution", "Assign custom circuits to Classic, Rally, Kart, GT, Nascar"),
        ];
        let mut cl_y = c2_y + scaler.s(70.0);
        for (tag, desc) in &classes {
            fonts.draw_ui_bold(tag, col2_x + scaler.s(14.0), cl_y, scaler.font_s(10.0), Palette::NEON_CYAN);
            fonts.draw_ui_regular(desc, col2_x + scaler.s(130.0), cl_y, scaler.font_s(10.0), Palette::WHITE);
            cl_y += scaler.s(18.0);
        }
    } else {
        // Empty state on right column when no tracks match filter
        fonts.draw_ui_bold(
            "CIRCUIT DOSSIER",
            col2_x,
            c2_y + scaler.s(13.0),
            scaler.font_s(15.0),
            Palette::UI_TEXT_MUTED,
        );
        c2_y += scaler.s(22.0);

        let empty_dossier_h = scaler.s(220.0);
        scaler.draw_glass_card(col2_x, c2_y, col_w, empty_dossier_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);
        fonts.draw_ui_bold_centered(
            "No circuit selected",
            col2_x + col_w * 0.5,
            c2_y + empty_dossier_h * 0.45,
            scaler.font_s(14.0),
            Palette::UI_TEXT_MUTED,
        );
        fonts.draw_ui_regular_centered(
            "Press [T] to manage and design circuits in Circuit Manager",
            col2_x + col_w * 0.5,
            c2_y + empty_dossier_h * 0.55,
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );
    }

    // Footer Launch prompt button
    let is_tm_selected = has_tm_entry && selected_track_idx == total_tracks;
    let (btn_bg, btn_border, start_prompt) = if is_tm_selected {
        (
            Color::new(0.32, 0.12, 0.52, 0.95),
            Palette::NEON_MAGENTA,
            "PRESS [SPACE / ENTER] OR [T] TO OPEN CIRCUIT MANAGER".to_string(),
        )
    } else if is_sel_locked {
        (
            Color::new(0.35, 0.10, 0.10, 0.95),
            Palette::RED,
            "🔒 CIRCUIT LOCKED • REACH REQUIRED CAREER LEVEL TO UNLOCK".to_string(),
        )
    } else if total_tracks > 0 {
        (
            Color::new(0.12, 0.65, 0.32, 0.95),
            Palette::NEON_GREEN,
            "PRESS [SPACE / ENTER] OR GAMEPAD [A / START] TO RACE".to_string(),
        )
    } else {
        (
            Color::new(0.08, 0.28, 0.40, 0.95),
            Palette::NEON_CYAN,
            "PRESS [SPACE / ENTER] OR [T] TO OPEN CIRCUIT MANAGER".to_string(),
        )
    };
    let btn_w = scaler.s(460.0);
    let btn_h = scaler.s(40.0);
    let btn_x = (sw - btn_w) * 0.5;
    let btn_y = sh - btn_h - scaler.s(14.0);

    let footer_text = if crate::storage::is_dev_mode() {
        "[Left / Right] Category  •  [Up / Down] Select Track  •  [T] Circuit Manager  •  [Ctrl+D] Dev Workbench  •  [X] Settings  •  [ESC] Back"
    } else {
        "[Left / Right] Category  •  [Up / Down] Select Track  •  [T] Circuit Manager  •  [X] Settings  •  [K] Controls  •  [ESC] Back"
    };

    fonts.draw_ui_regular_centered(
        footer_text,
        sw * 0.5,
        btn_y - scaler.s(10.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, btn_border);

    fonts.draw_ui_bold_centered(
        &start_prompt,
        sw * 0.5,
        btn_y + scaler.s(25.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

fn render_stat_bar_full(
    scaler: UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    val: f32,
    color: Color,
) {
    fonts.draw_ui_bold(label, x, y + scaler.s(7.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
    let bar_x = x + scaler.s(44.0);
    let bar_w = (w - scaler.s(84.0)).max(30.0);
    let bar_h = scaler.s(6.0);

    draw_rectangle(bar_x, y, bar_w, bar_h, Color::new(0.1, 0.12, 0.16, 0.9));
    draw_rectangle(bar_x, y, bar_w * val.clamp(0.0, 1.0), bar_h, color);

    let pct_str = format!("{:.0}%", (val * 100.0).clamp(0.0, 100.0));
    fonts.draw_ui_bold(&pct_str, bar_x + bar_w + scaler.s(8.0), y + scaler.s(7.0), scaler.font_s(9.5), color);
}

/// Bounding rectangles for interactive Pause Menu action buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PauseMenuButtonLayout {
    pub resume_rect: (f32, f32, f32, f32),
    pub exit_rect: (f32, f32, f32, f32),
}

/// Calculates the layout dimensions and button rectangles for the pause menu overlay.
pub fn pause_menu_layout(sw: f32, sh: f32) -> (f32, f32, f32, f32, PauseMenuButtonLayout) {
    let scaler = UiScaler::new(sw, sh);
    let box_w = scaler.s(500.0);
    let box_h = scaler.s(345.0);
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let pad_x = scaler.s(22.0);
    let gap = scaler.s(16.0);
    let btn_w = (box_w - pad_x * 2.0 - gap) * 0.5;
    let btn_h = scaler.s(48.0);
    let btn_y = box_y + scaler.s(58.0);

    let resume_rect = (box_x + pad_x, btn_y, btn_w, btn_h);
    let exit_rect = (box_x + pad_x + btn_w + gap, btn_y, btn_w, btn_h);

    (
        box_x,
        box_y,
        box_w,
        box_h,
        PauseMenuButtonLayout {
            resume_rect,
            exit_rect,
        },
    )
}

/// Renders the modern Pause overlay with Assist Profile selection, Audio status, and Resume/Exit buttons.
pub fn render_pause_menu(fonts: &Fonts, assist_profile: AssistProfile, audio_settings: &AudioSettings, selected_btn: usize) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark semi-transparent dim
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.70));

    let (box_x, box_y, box_w, box_h, btn_layout) = pause_menu_layout(sw, sh);

    scaler.draw_glass_card(box_x, box_y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

    let title = "RACE PAUSED";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        box_y + scaler.s(38.0),
        scaler.font_s(30.0),
        Palette::WHITE,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let (mx, my) = std::panic::catch_unwind(macroquad::input::mouse_position).unwrap_or((-1000.0, -1000.0));

    // Resume button
    let (rx, ry, rw, rh) = btn_layout.resume_rect;
    let is_resume_hovered = mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh;
    let is_resume_active = selected_btn == 0 || is_resume_hovered;
    let resume_bg = if is_resume_active {
        Color::new(0.14, 0.58, 0.32, 0.98)
    } else {
        Color::new(0.08, 0.36, 0.20, 0.85)
    };
    let resume_border = if is_resume_active {
        Palette::NEON_GREEN
    } else {
        Color::new(0.20, 0.75, 0.40, 0.85)
    };
    draw_rectangle(rx, ry, rw, rh, resume_bg);
    draw_rectangle_lines(
        rx,
        ry,
        rw,
        rh,
        if is_resume_active { 2.6 * scaler.scale } else { 1.4 * scaler.scale },
        resume_border,
    );
    fonts.draw_ui_bold_centered(
        if is_resume_active { "[ENTER] RESUME RACE" } else { "RESUME RACE" },
        rx + rw * 0.5,
        ry + scaler.s(21.0),
        scaler.font_s(14.5),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "Continue | Gamepad Start / A",
        rx + rw * 0.5,
        ry + scaler.s(37.0),
        scaler.font_s(11.0),
        Color::new(0.75, 0.95, 0.80, 0.90),
    );

    // Exit button
    let (ex, ey, ew, eh) = btn_layout.exit_rect;
    let is_exit_hovered = mx >= ex && mx <= ex + ew && my >= ey && my <= ey + eh;
    let is_exit_active = selected_btn == 1 || is_exit_hovered;
    let exit_bg = if is_exit_active {
        Color::new(0.62, 0.14, 0.18, 0.98)
    } else {
        Color::new(0.38, 0.08, 0.12, 0.85)
    };
    let exit_border = if is_exit_active {
        Palette::RED
    } else {
        Color::new(0.85, 0.25, 0.28, 0.85)
    };
    draw_rectangle(ex, ey, ew, eh, exit_bg);
    draw_rectangle_lines(
        ex,
        ey,
        ew,
        eh,
        if is_exit_active { 2.6 * scaler.scale } else { 1.4 * scaler.scale },
        exit_border,
    );
    fonts.draw_ui_bold_centered(
        if is_exit_active { "[ENTER] EXIT RACE" } else { "EXIT RACE" },
        ex + ew * 0.5,
        ey + scaler.s(21.0),
        scaler.font_s(14.5),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "Main Menu | Gamepad B",
        ex + ew * 0.5,
        ey + scaler.s(37.0),
        scaler.font_s(11.0),
        Color::new(0.95, 0.75, 0.75, 0.90),
    );

    // Divider line
    let div_y = ry + rh + scaler.s(14.0);
    draw_rectangle(
        box_x + scaler.s(20.0),
        div_y,
        box_w - scaler.s(40.0),
        scaler.s(1.0),
        Color::new(0.2, 0.3, 0.45, 0.5),
    );

    let assist_item = format!("H / R3 : Toggle Assists [{}]", assist_profile.short_name());
    let music_status = if audio_settings.is_muted || audio_settings.is_music_muted { "MUTED" } else { "ON" };
    let sfx_status = if audio_settings.is_muted || audio_settings.is_sfx_muted { "MUTED" } else { "ON" };
    let vol_pct = (audio_settings.master_volume * 100.0).round() as i32;
    let audio_item = format!("M : Music [{}] | S : Sound [{}] | [ / ] : Vol {}%", music_status, sfx_status, vol_pct);

    let items = [
        assist_item,
        audio_item,
        "O / Y : Arcade Settings & Preferences".to_string(),
        "D : Driver Cards & Opponents Dossier".to_string(),
        "K : Controls Guide | R : Restart Race".to_string(),
        "TAB / Left Stick Click : Camera View".to_string(),
        "Q/A/O/P / Arrows / Stick & Triggers : Drive".to_string(),
        "SPACE / B : Handbrake | Hold Brake at Stop : Reverse".to_string(),
    ];

    let mut item_y = div_y + scaler.s(22.0);
    for item in &items {
        fonts.draw_ui_bold(
            item,
            box_x + scaler.s(24.0),
            item_y,
            scaler.font_s(13.5),
            Color::new(0.85, 0.90, 0.98, 1.0),
        );
        item_y += scaler.s(22.5);
    }
}

/// Renders the Race Results / Podium Standings screen with modern leaderboard cards.
pub fn render_results_screen(
    fonts: &Fonts,
    track_name: &str,
    results: &[RaceResultEntry],
    is_time_attack: bool,
    xp_receipt: Option<&XpAwardReceipt>,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.94));

    let box_w = (sw * 0.80).clamp(scaler.s(520.0), scaler.s(850.0));
    let box_h = (sh * 0.82).clamp(scaler.s(420.0), scaler.s(680.0));
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 2.5);

    let title = if is_time_attack {
        "TIME ATTACK SESSION COMPLETE"
    } else {
        "RACE RESULTS & STANDINGS"
    };
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        y + scaler.s(44.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let track_label = format!("Circuit: {}", track_name);
    fonts.draw_ui_bold(
        &track_label,
        x + scaler.s(28.0),
        y + scaler.s(76.0),
        scaler.font_s(16.0),
        Palette::UI_TEXT_MUTED,
    );

    // Table Header
    let mut row_y = y + scaler.s(108.0);
    let hdr_h = scaler.s(28.0);
    draw_rectangle(x + scaler.s(20.0), row_y - scaler.s(20.0), box_w - scaler.s(40.0), hdr_h, Color::new(0.12, 0.16, 0.25, 0.9));
    fonts.draw_ui_bold("POS", x + scaler.s(32.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("DRIVER / VEHICLE", x + scaler.s(85.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("TOTAL TIME", x + box_w - scaler.s(320.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("BEST LAP", x + box_w - scaler.s(190.0), row_y, scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_bold("GAP", x + box_w - scaler.s(75.0), row_y, scaler.font_s(14.0), Palette::WHITE);

    row_y += scaler.s(24.0);

    for res in results {
        let (row_bg, text_col) = if res.is_player {
            (Color::new(0.18, 0.35, 0.22, 0.90), Palette::NEON_GREEN)
        } else {
            (Color::new(0.09, 0.11, 0.16, 0.70), Color::new(0.85, 0.90, 0.95, 1.0))
        };

        draw_rectangle(x + scaler.s(20.0), row_y - scaler.s(16.0), box_w - scaler.s(40.0), scaler.s(28.0), row_bg);

        // Position medal icon or text
        let pos_str = match res.position {
            1 => "P1".to_string(),
            2 => "P2".to_string(),
            3 => "P3".to_string(),
            _ => format!("P{}", res.position),
        };
        fonts.draw_ui_bold(&pos_str, x + scaler.s(28.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);
        fonts.draw_ui_bold(&res.car_name, x + scaler.s(85.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let total_str = format_lap_time(res.total_time);
        fonts.draw_ui_bold(&total_str, x + box_w - scaler.s(320.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let best_str = format_lap_time(res.best_lap.unwrap_or(0.0));
        fonts.draw_ui_bold(&best_str, x + box_w - scaler.s(190.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        let gap_str = if res.position == 1 {
            "-".to_string()
        } else {
            format!("+{:.2}s", res.delta_to_leader)
        };
        fonts.draw_ui_bold(&gap_str, x + box_w - scaler.s(75.0), row_y + scaler.s(4.0), scaler.font_s(14.0), text_col);

        row_y += scaler.s(32.0);
    }

    // Optional Career XP Receipt Banner
    if let Some(receipt) = xp_receipt {
        let r_h = scaler.s(32.0);
        let r_y = y + box_h - scaler.s(64.0);
        let r_w = box_w - scaler.s(40.0);
        let r_x = x + scaler.s(20.0);
        scaler.draw_glass_card(r_x, r_y, r_w, r_h, Color::new(0.06, 0.10, 0.16, 0.95), Palette::NEON_GOLD, 1.4);

        let first_text = if receipt.is_first_time {
            format!("  •  1ST VISIT BONUS: +{} XP", receipt.first_time_bonus)
        } else {
            String::new()
        };
        let receipt_str = format!(
            "+{} XP EARNED ({} LAPS: {} XP  •  FINISH BONUS: +{} XP{})   |   BALANCE: {} XP",
            receipt.total_xp,
            receipt.completed_laps,
            receipt.lap_xp,
            receipt.completion_bonus,
            first_text,
            receipt.new_balance
        );
        fonts.draw_ui_bold_centered(
            &receipt_str,
            sw * 0.5,
            r_y + scaler.s(21.0),
            scaler.font_s(11.5),
            Palette::NEON_GOLD,
        );
    }

    // Bottom action prompt
    let prompt = "Press [SPACE / ENTER] Hall of Fame | [TAB] Detailed Stats | [R] Restart Race | [ESC] Main Menu";
    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        y + box_h - scaler.s(20.0),
        scaler.font_s(15.0),
        Palette::WHITE,
    );
}

/// Renders the full-screen Controls, Gamepad Mappings, and Assist Settings screen.
pub fn render_controls_screen(
    fonts: &Fonts,
    assist_profile: AssistProfile,
    gamepad_connected: bool,
    gamepad_name: &str,
    input_map: &cabinet::input::InputMap,
    preset_name: &str,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Background backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.05, 0.06, 0.09, 0.98));

    // Header Title
    let title = "CONTROLS & DRIVING ASSISTS";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(44.0),
        scaler.font_s(36.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = "Configurable Controls & Gamepad Mappings | Electronic Vehicle Dynamics Configuration";
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        scaler.s(68.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    // Controller Status Banner
    let banner_w = sw * 0.82;
    let banner_x = (sw - banner_w) * 0.5;
    let banner_y = scaler.s(85.0);
    let banner_h = scaler.s(34.0);

    if gamepad_connected {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.08, 0.22, 0.15, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Palette::NEON_GREEN);
        let text = format!("ACTIVE GAMEPAD DETECTED: {}", gamepad_name);
        fonts.draw_ui_bold(&text, banner_x + scaler.s(16.0), banner_y + scaler.s(22.0), scaler.font_s(14.0), Palette::NEON_GREEN);
    } else {
        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.10, 0.12, 0.18, 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 1.5, Palette::UI_CARD_BORDER);
        let text = "NO GAMEPAD DETECTED — KEYBOARD & TOUCH ACTIVE (PLUG & PLAY READY)";
        fonts.draw_ui_bold(text, banner_x + scaler.s(16.0), banner_y + scaler.s(22.0), scaler.font_s(14.0), Palette::UI_TEXT_MUTED);
    }

    // Left Column: Keyboard Controls
    let col_w = (sw * 0.40).clamp(scaler.s(320.0), scaler.s(480.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(14.0)).max(scaler.safe_pad_x);
    let col_y = scaler.s(130.0);
    let col_h = sh * 0.54;

    scaler.draw_glass_card(col1_x, col_y, col_w, col_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.8);
    fonts.draw_ui_bold("KEYBOARD CONTROLS", col1_x + scaler.s(16.0), col_y + scaler.s(26.0), scaler.font_s(18.0), Palette::NEON_CYAN);

    let throttle_label = input_map.primary_binding_label(cabinet::input::ArcadeAction::Up);
    let brake_label = input_map.primary_binding_label(cabinet::input::ArcadeAction::Down);
    let steer_label = format!(
        "{} / {}",
        input_map.primary_binding_label(cabinet::input::ArcadeAction::Left),
        input_map.primary_binding_label(cabinet::input::ArcadeAction::Right)
    );
    let handbrake_label = input_map.primary_binding_label(cabinet::input::ArcadeAction::Action3);

    let kb_rows = [
        ("Accelerate / Gas", throttle_label.as_str()),
        ("Brake / Reverse (at stop)", brake_label.as_str()),
        ("Steer Left / Right", steer_label.as_str()),
        ("Handbrake & Drift", handbrake_label.as_str()),
        ("Active Key Layout", preset_name),
        ("Cycle Controls Preset", "Tab / C"),
        ("Cycle Assist Profile", "H"),
        ("Controls & Assists Guide", "K"),
        ("Camera Zoom In / Out", "+ / -"),
        ("Instant Session Reset", "R"),
        ("Pause / Resume", "Escape / Pause"),
        ("Audio Mute / Volume", "M / [ and ]"),
    ];

    let mut row_y = col_y + scaler.s(52.0);
    for (action, key) in &kb_rows {
        fonts.draw_ui_regular(action, col1_x + scaler.s(16.0), row_y, scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
        let km = fonts.measure_ui_bold(key, scaler.font_s(13.0));
        fonts.draw_ui_bold(key, col1_x + col_w - km.width - scaler.s(16.0), row_y, scaler.font_s(13.0), Palette::NEON_GOLD);
        row_y += scaler.s(20.0);
    }

    // Right Column: Gamepad Controls
    let col2_x = (sw * 0.5 + scaler.s(14.0)).min(sw - col_w - scaler.safe_pad_x);
    scaler.draw_glass_card(col2_x, col_y, col_w, col_h, Palette::UI_CARD_BG, Palette::NEON_MAGENTA, 1.8);
    fonts.draw_ui_bold("GAMEPAD CONTROLS", col2_x + scaler.s(16.0), col_y + scaler.s(26.0), scaler.font_s(18.0), Palette::NEON_MAGENTA);

    let gp_rows = [
        ("Proportional Steering", "Left Analog Stick / D-Pad"),
        ("Analog Progressive Throttle", "Right Trigger (RT / R2)"),
        ("Analog Brake / Reverse (at stop)", "Left Trigger (LT / L2)"),
        ("Handbrake & Slide Initiation", "A / Cross Button (or RB)"),
        ("Cycle Assist Profile", "Right Stick Click (R3) / Select"),
        ("Cycle Camera Zoom", "Left Stick Click (L3)"),
        ("Pause / Resume Menu", "Start / Menu Button"),
        ("Menu Navigation", "D-Pad / Left Stick"),
        ("Confirm / Start Race", "A / Cross Button (Enter)"),
        ("Back / Cancel", "B / Circle Button (Escape)"),
    ];

    let mut gp_row_y = col_y + scaler.s(52.0);
    for (action, button) in &gp_rows {
        fonts.draw_ui_regular(action, col2_x + scaler.s(16.0), gp_row_y, scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
        let bm = fonts.measure_ui_bold(button, scaler.font_s(13.0));
        fonts.draw_ui_bold(button, col2_x + col_w - bm.width - scaler.s(16.0), gp_row_y, scaler.font_s(13.0), Palette::NEON_GREEN);
        gp_row_y += scaler.s(21.0);
    }

    // Bottom Panel: Active Drive Assists Profile
    let bot_y = col_y + col_h + scaler.s(12.0);
    let bot_h = scaler.s(85.0);
    scaler.draw_glass_card(banner_x, bot_y, banner_w, bot_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    let assist_title = format!("ACTIVE DRIVE ASSIST PROFILE: [H / R3] {}", assist_profile.title());
    let assist_col = match assist_profile {
        AssistProfile::Arcade => Palette::NEON_CYAN,
        AssistProfile::Sport => Palette::NEON_GOLD,
        AssistProfile::Pro => Palette::RED,
    };
    fonts.draw_ui_bold(&assist_title, banner_x + scaler.s(18.0), bot_y + scaler.s(24.0), scaler.font_s(16.0), assist_col);
    fonts.draw_ui_regular(assist_profile.description(), banner_x + scaler.s(18.0), bot_y + scaler.s(48.0), scaler.font_s(13.0), Color::new(0.80, 0.85, 0.92, 1.0));
    fonts.draw_ui_regular("Press [H] on keyboard or [R3 / Select] on Gamepad to switch assist difficulty profile anytime!", banner_x + scaler.s(18.0), bot_y + scaler.s(68.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);

    // Footer Return Prompt
    let back_prompt = "PRESS [TAB / C] CYCLE PRESET  •  [H / R3] ASSISTS  •  [ESC / K / SPACE] RETURN";
    fonts.draw_ui_bold_centered(
        back_prompt,
        sw * 0.5,
        sh - scaler.s(18.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

use crate::tournament::ChampionshipSession;

/// Renders the Motorsport Grand Hub Module Selection Menu.
pub fn render_module_select_menu(
    fonts: &Fonts,
    selected_idx: usize,
    modules: &[(&str, &str, &str, &str, Color)], // (id, title, tag, description, color)
    active_profile: &PlayerProfile,
    active_stats: &ProfileCareerStats,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header
    fonts.draw_display_centered_with_shadow(
        "TDRACE MOTORSPORT GRAND HUB",
        sw * 0.5,
        scaler.s(32.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    fonts.draw_ui_regular_centered(
        "Select a Motorsport Championship Category or Circuit Studio Workshop",
        sw * 0.5,
        scaler.s(52.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    let card_w = (sw * 0.72).clamp(scaler.s(480.0), scaler.s(720.0));
    let card_x = (sw - card_w) * 0.5;

    // Active Profile Badge Banner
    let badge_y = scaler.s(64.0);
    let badge_h = scaler.s(48.0);
    render_profile_badge(fonts, &scaler, card_x, badge_y, card_w, badge_h, active_profile, active_stats);

    let card_gap = scaler.s(8.0);
    let start_y = badge_y + badge_h + scaler.s(10.0);
    let available_h = (sh - start_y - scaler.s(38.0)).max(scaler.s(240.0));
    let card_h = ((available_h - card_gap * (modules.len() as f32 - 1.0)) / modules.len() as f32).clamp(scaler.s(60.0), scaler.s(82.0));
    let mut curr_y = start_y;

    for (i, (_id, title, tag, desc, accent_col)) in modules.iter().enumerate() {
        let is_sel = i == selected_idx;
        let bg_col = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_CARD_BG
        };
        let border_col = if is_sel {
            *accent_col
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(card_x, curr_y, card_w, card_h, bg_col, border_col, if is_sel { 2.4 } else { 1.2 });

        // Left accent bar
        if is_sel {
            draw_rectangle(card_x, curr_y, scaler.s(6.0), card_h, *accent_col);
        }

        // Tag
        fonts.draw_ui_bold(
            tag,
            card_x + scaler.s(20.0),
            curr_y + card_h * 0.25,
            scaler.font_s(11.0),
            if is_sel { *accent_col } else { Palette::UI_TEXT_MUTED },
        );

        // Title
        fonts.draw_display(
            title,
            card_x + scaler.s(20.0),
            curr_y + card_h * 0.54,
            scaler.font_s(18.0),
            if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
        );

        // Description
        fonts.draw_ui_regular(
            desc,
            card_x + scaler.s(20.0),
            curr_y + card_h * 0.82,
            scaler.font_s(12.0),
            if is_sel { Color::new(0.80, 0.85, 0.92, 1.0) } else { Palette::UI_TEXT_MUTED },
        );

        curr_y += card_h + card_gap;
    }

    // Footer prompt
    let prompt = "USE [UP/DOWN] TO SELECT MODULE | [ENTER/SPACE] OPEN MENU | [X] SETTINGS | [P] SWITCH PROFILE | [N] NEW PROFILE | [K] CONTROLS | [ESC] QUIT";
    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        sh - scaler.s(20.0),
        scaler.font_s(13.0),
        Palette::NEON_CYAN,
    );
}

/// Renders the Championship Standings table and round victory screen.
pub fn render_championship_standings_screen(
    fonts: &Fonts,
    champ: &ChampionshipSession,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header
    let title = champ.name.to_uppercase();
    fonts.draw_display_centered_with_shadow(
        &title,
        sw * 0.5,
        scaler.s(45.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = if champ.is_completed {
        "SEASON FINALE — CHAMPIONSHIP DECIDED!".to_string()
    } else {
        format!("ROUND {} OF {} COMPLETED — DRIVER STANDINGS", champ.current_round, champ.total_rounds())
    };
    fonts.draw_ui_bold_centered(
        &subtitle,
        sw * 0.5,
        scaler.s(70.0),
        scaler.font_s(14.0),
        if champ.is_completed { Palette::NEON_GREEN } else { Palette::NEON_CYAN },
    );

    // Standings Table Card
    let table_w = (sw * 0.80).clamp(scaler.s(520.0), scaler.s(850.0));
    let table_x = (sw - table_w) * 0.5;
    let table_y = scaler.s(90.0);
    let table_h = scaler.s(420.0);

    scaler.draw_glass_card(table_x, table_y, table_w, table_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.5);

    // Table Header Row
    let mut row_y = table_y + scaler.s(28.0);
    fonts.draw_ui_bold("POS", table_x + scaler.s(20.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("DRIVER", table_x + scaler.s(70.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("TEAM / CAR", table_x + scaler.s(280.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("WINS", table_x + table_w - scaler.s(160.0), row_y, scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold("POINTS", table_x + table_w - scaler.s(75.0), row_y, scaler.font_s(13.0), Palette::NEON_GOLD);

    draw_rectangle(table_x + scaler.s(15.0), row_y + scaler.s(8.0), table_w - scaler.s(30.0), 1.0, Palette::UI_CARD_BORDER);
    row_y += scaler.s(24.0);

    // Table Rows
    for (i, entry) in champ.standings.iter().enumerate().take(10) {
        let pos_str = format!("#{}", i + 1);
        let pos_col = match i {
            0 => Palette::NEON_GOLD,
            1 => Palette::WHITE,
            2 => Palette::NEON_MAGENTA,
            _ => Palette::UI_TEXT_MUTED,
        };

        fonts.draw_ui_bold(&pos_str, table_x + scaler.s(20.0), row_y, scaler.font_s(14.0), pos_col);
        fonts.draw_ui_bold(&entry.driver_name, table_x + scaler.s(70.0), row_y, scaler.font_s(14.0), Palette::WHITE);
        fonts.draw_ui_regular(&entry.team_name, table_x + scaler.s(280.0), row_y, scaler.font_s(13.0), Color::new(0.75, 0.80, 0.88, 1.0));
        fonts.draw_ui_bold(&entry.wins.to_string(), table_x + table_w - scaler.s(150.0), row_y, scaler.font_s(14.0), Palette::WHITE);
        fonts.draw_display(&format!("{} PTS", entry.points), table_x + table_w - scaler.s(85.0), row_y, scaler.font_s(15.0), Palette::NEON_GOLD);

        row_y += scaler.s(28.0);
    }

    // Bottom Action Prompt
    let next_prompt = if champ.is_completed {
        "SEASON COMPLETE! PRESS [ENTER/SPACE] OR GAMEPAD [A] TO RETURN TO MENU"
    } else {
        "PRESS [ENTER/SPACE] OR GAMEPAD [A] TO START NEXT ROUND | [ESC] EXIT"
    };

    fonts.draw_ui_bold_centered(
        next_prompt,
        sw * 0.5,
        sh - scaler.s(26.0),
        scaler.font_s(15.0),
        Palette::NEON_GREEN,
    );
}

/// Renders the exit confirmation modal overlay when pressing Escape or Gamepad B on the Main Menu.
pub fn render_exit_confirm_modal(fonts: &Fonts) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let theme = CabinetTheme::cyberpunk_neon();
    let gp = GamepadSnapshot::default();
    let modal = UniversalConfirmModal::quit_game();
    let ctx = CabinetContext::new(&scaler, fonts, &theme, &gp, 0.0);
    modal.draw(&ctx);
}

/// Category tab in the Modality Selection screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModalityCategory {
    SinglePlayer,
    Multiplayer,
    Options,
}

impl ModalityCategory {
    pub const ALL: [Self; 3] = [
        Self::SinglePlayer,
        Self::Multiplayer,
        Self::Options,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::SinglePlayer => "SINGLE PLAYER",
            Self::Multiplayer => "MULTIPLAYER",
            Self::Options => "OPTIONS",
        }
    }

    pub fn items(&self) -> &'static [ModalityItem] {
        match self {
            Self::SinglePlayer => &[
                ModalityItem::QuickRace,
                ModalityItem::CustomRace,
                ModalityItem::CareerMode,
                ModalityItem::TimeTrial,
                ModalityItem::FreeRide,
            ],
            Self::Multiplayer => &[
                ModalityItem::SplitScreen,
                ModalityItem::LanPlay,
                ModalityItem::CloudPlay,
            ],
            Self::Options => &[
                ModalityItem::PlayerProfile,
                ModalityItem::Garage,
                ModalityItem::Settings,
            ],
        }
    }
}

/// Distinct race modalities available for selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModalityItem {
    QuickRace,
    CustomRace,
    CareerMode,
    TimeTrial,
    FreeRide,
    SplitScreen,
    LanPlay,
    CloudPlay,
    PlayerProfile,
    Garage,
    Settings,
}

impl ModalityItem {
    pub fn title(&self) -> &'static str {
        match self {
            Self::QuickRace => "Quick Race",
            Self::CustomRace => "Custom Race",
            Self::CareerMode => "Career Mode",
            Self::TimeTrial => "Time Trial",
            Self::FreeRide => "Free Ride",
            Self::SplitScreen => "2P Split Screen",
            Self::LanPlay => "LAN Multiplayer",
            Self::CloudPlay => "Cloud Online",
            Self::PlayerProfile => "Player Profile",
            Self::Garage => "Garage Showroom",
            Self::Settings => "Settings",
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            Self::QuickRace => "PRESET CONFIG • INSTANT ACTION",
            Self::CustomRace => "CUSTOM VEHICLE & GRID OPTIONS",
            Self::CareerMode => "CHAMPIONSHIP CAMPAIGN",
            Self::TimeTrial => "SOLO BENCHMARK VS PB SHADOW",
            Self::FreeRide => "OPEN PRACTICE • NO PRESSURE",
            Self::SplitScreen => "HEAD-TO-HEAD LOCAL RACING",
            Self::LanPlay => "LOCAL NETWORK [COMING SOON]",
            Self::CloudPlay => "WORLDWIDE LOBBIES [COMING SOON]",
            Self::PlayerProfile => "DRIVER RECORDS • CAREER STATS & SLOTS",
            Self::Garage => "360° VEHICLE TURNTABLE & TECHNICAL DOSSIER",
            Self::Settings => "AUDIO • CONTROLS • ASSISTS • DISPLAY CONFIG",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::QuickRace => "Jump straight onto the track with official predefined cars and full opponent grid.",
            Self::CustomRace => "Customize your machine, grid size, AI difficulty, and racing rules freely.",
            Self::CareerMode => "Progress through structured multi-tier championships and unlock elite vehicles.",
            Self::TimeTrial => "Push limits against the clock and chase down your personal best ghost car.",
            Self::FreeRide => "Open practice session without opponents, rules, or lap timers to hone your lines.",
            Self::SplitScreen => "Battle side-by-side with a friend on a single screen using keyboard and gamepad.",
            Self::LanPlay => "Host or join low-latency racing lobbies on your local Wi-Fi or wired network.",
            Self::CloudPlay => "Compete globally in ranked matchmaking, custom public lobbies, and online events.",
            Self::PlayerProfile => "Inspect career statistics, manage driver slots, change nationality and custom car liveries.",
            Self::Garage => "Inspect active motorsport machines in fullscreen 360° turntable, check BHP and weight, and rev engine.",
            Self::Settings => "Configure sound levels, gamepad and keyboard mappings, steering assists, and display settings.",
        }
    }

    pub fn is_available(&self) -> bool {
        !matches!(self, Self::LanPlay | Self::CloudPlay)
    }

    pub fn accent_color(&self) -> Color {
        match self {
            Self::QuickRace => Palette::NEON_CYAN,
            Self::CustomRace => Palette::NEON_GOLD,
            Self::CareerMode => Palette::NEON_GREEN,
            Self::TimeTrial => Palette::NEON_MAGENTA,
            Self::FreeRide => Color::new(0.35, 0.75, 1.0, 1.0),
            Self::SplitScreen => Palette::NEON_ORANGE,
            Self::LanPlay => Color::new(0.60, 0.65, 0.75, 1.0),
            Self::CloudPlay => Color::new(0.60, 0.65, 0.75, 1.0),
            Self::PlayerProfile => Palette::NEON_CYAN,
            Self::Garage => Palette::NEON_GOLD,
            Self::Settings => Palette::NEON_MAGENTA,
        }
    }
}

/// Modal overlay shown when interacting with an in-development modality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalityModal {
    LanComingSoon,
    CloudComingSoon,
    CareerComingSoon,
}

impl ModalityModal {
    pub fn title(&self) -> &'static str {
        match self {
            Self::LanComingSoon => "LAN MULTIPLAYER • IN DEVELOPMENT",
            Self::CloudComingSoon => "CLOUD MULTIPLAYER • IN DEVELOPMENT",
            Self::CareerComingSoon => "CAREER MODE • IN DEVELOPMENT",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::LanComingSoon => "Local Area Network multiplayer is currently under active development.\nDirect IP connection, auto-discovery broadcast, and dedicated headless server support are coming in an upcoming release.",
            Self::CloudComingSoon => "Worldwide online matchmaking and cloud lobbies are currently under active development.\nGlobal leaderboards, ranked matchmaking, and cloud ghost synchronization will debut in Phase 2.",
            Self::CareerComingSoon => "Career campaign progression for this motorsport category is currently under development.\nTier ladders, championship calendars, vehicle unlocking, and trophy progression are coming soon.",
        }
    }
}

/// Renders the Race Modality Selection stage inserted between the Grand Hub and Circuit Selection.
/// Layout: 3 Columns (Col 1: Single Player, Col 2: Multiplayer, Col 3: Options).
pub fn render_modality_select_screen(
    fonts: &Fonts,
    active_module_title: &str,
    _active_module_id: &str,
    active_module_accent: Color,
    category: ModalityCategory,
    selected_idx: usize,
    modal: Option<&ModalityModal>,
    active_profile: &PlayerProfile,
    _active_stats: &ProfileCareerStats,
    _dev_mode: bool,
    _available_tracks: &[TrackChoice],
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header Title & Breadcrumb
    fonts.draw_display_centered_with_shadow(
        &format!("{} • SELECT RACING MODALITY", active_module_title),
        sw * 0.5,
        scaler.s(28.0),
        scaler.font_s(24.0),
        active_module_accent,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    fonts.draw_ui_regular_centered(
        "Choose session format: Single Player vs Multiplayer  •  Or configure options, garage & profile",
        sw * 0.5,
        scaler.s(48.0),
        scaler.font_s(12.5),
        Palette::UI_TEXT_MUTED,
    );

    // 3 Category Tabs at Top
    let tab_w = (sw * 0.28).clamp(scaler.s(160.0), scaler.s(260.0));
    let tab_h = scaler.s(30.0);
    let tab_gap = scaler.s(12.0);
    let total_tabs_w = tab_w * 3.0 + tab_gap * 2.0;
    let tabs_start_x = (sw - total_tabs_w) * 0.5;
    let tab_y = scaler.s(64.0);

    for (cat_idx, cat) in ModalityCategory::ALL.iter().enumerate() {
        let is_cat_active = *cat == category;
        let tx = tabs_start_x + (cat_idx as f32) * (tab_w + tab_gap);
        let tab_bg = if is_cat_active {
            Color::new(0.12, 0.16, 0.26, 0.95)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.60)
        };
        let tab_border = if is_cat_active {
            active_module_accent
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(tx, tab_y, tab_w, tab_h, tab_bg, tab_border, if is_cat_active { 2.0 } else { 1.0 });

        if is_cat_active {
            draw_rectangle(tx, tab_y + tab_h - scaler.s(2.5), tab_w, scaler.s(2.5), active_module_accent);
        }

        let tab_label = match cat {
            ModalityCategory::SinglePlayer => "[ 1. SINGLE PLAYER ]",
            ModalityCategory::Multiplayer => "[ 2. MULTIPLAYER ]",
            ModalityCategory::Options => "[ 3. OPTIONS ]",
        };
        let tab_text_col = if is_cat_active {
            Palette::WHITE
        } else {
            Palette::UI_TEXT_MUTED
        };
        fonts.draw_ui_bold_centered(
            tab_label,
            tx + tab_w * 0.5,
            tab_y + scaler.s(19.0),
            scaler.font_s(11.5),
            tab_text_col,
        );
    }

    // Selected Menu Content Layout (Centered single-menu view)
    let col_w = total_tabs_w;
    let col_x = tabs_start_x;
    let start_y = scaler.s(104.0);
    let available_h = (sh - start_y - scaler.s(36.0)).max(scaler.s(300.0));

    match category {
        ModalityCategory::SinglePlayer => {
            let sp_items = ModalityCategory::SinglePlayer.items();
            let card_gap = scaler.s(8.0);
            let sp_card_h = ((available_h - card_gap * (sp_items.len() as f32 - 1.0)) / sp_items.len() as f32)
                .clamp(scaler.s(54.0), scaler.s(88.0));

            let mut curr_y = start_y;
            for (i, item) in sp_items.iter().enumerate() {
                let is_sel = i == selected_idx;
                let accent = item.accent_color();

                let bg_col = if is_sel {
                    Palette::UI_CARD_BG_HOVER
                } else {
                    Color::new(0.06, 0.08, 0.12, 0.70)
                };
                let border_col = if is_sel {
                    accent
                } else {
                    Palette::UI_CARD_BORDER
                };

                scaler.draw_glass_card(col_x, curr_y, col_w, sp_card_h, bg_col, border_col, if is_sel { 2.4 } else { 1.0 });

                if is_sel {
                    draw_rectangle(col_x, curr_y, scaler.s(6.0), sp_card_h, accent);
                }

                fonts.draw_ui_bold(
                    item.tag(),
                    col_x + scaler.s(18.0),
                    curr_y + sp_card_h * 0.25,
                    scaler.font_s(9.5),
                    if is_sel { accent } else { Palette::UI_TEXT_MUTED },
                );

                let title_str = if is_sel {
                    format!("▶ {}", item.title())
                } else {
                    item.title().to_string()
                };
                fonts.draw_display(
                    &title_str,
                    col_x + scaler.s(18.0),
                    curr_y + sp_card_h * 0.55,
                    scaler.font_s(16.0),
                    if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
                );

                fonts.draw_ui_regular(
                    item.description(),
                    col_x + scaler.s(18.0),
                    curr_y + sp_card_h * 0.84,
                    scaler.font_s(11.0),
                    if is_sel { Color::new(0.80, 0.85, 0.92, 1.0) } else { Palette::UI_TEXT_MUTED },
                );

                if is_sel {
                    fonts.draw_ui_bold(
                        "PRESS [ENTER] TO SELECT ▶",
                        col_x + col_w - scaler.s(190.0),
                        curr_y + sp_card_h * 0.55,
                        scaler.font_s(11.0),
                        accent,
                    );
                }

                curr_y += sp_card_h + card_gap;
            }
        }
        ModalityCategory::Multiplayer => {
            let mp_items = ModalityCategory::Multiplayer.items();
            let card_gap = scaler.s(12.0);
            let mp_card_h = ((available_h - card_gap * (mp_items.len() as f32 - 1.0)) / mp_items.len() as f32)
                .clamp(scaler.s(68.0), scaler.s(110.0));

            let mut curr_y = start_y;
            for (i, item) in mp_items.iter().enumerate() {
                let is_sel = i == selected_idx;
                let accent = item.accent_color();
                let is_avail = item.is_available();

                let bg_col = if is_sel {
                    Palette::UI_CARD_BG_HOVER
                } else {
                    Color::new(0.06, 0.08, 0.12, 0.70)
                };
                let border_col = if is_sel {
                    accent
                } else {
                    Palette::UI_CARD_BORDER
                };

                scaler.draw_glass_card(col_x, curr_y, col_w, mp_card_h, bg_col, border_col, if is_sel { 2.4 } else { 1.0 });

                if is_sel {
                    draw_rectangle(col_x, curr_y, scaler.s(6.0), mp_card_h, accent);
                }

                fonts.draw_ui_bold(
                    item.tag(),
                    col_x + scaler.s(18.0),
                    curr_y + mp_card_h * 0.25,
                    scaler.font_s(9.5),
                    if is_sel { accent } else { Palette::UI_TEXT_MUTED },
                );

                if !is_avail {
                    fonts.draw_ui_bold(
                        "🔒 COMING SOON",
                        col_x + col_w - scaler.s(120.0),
                        curr_y + mp_card_h * 0.25,
                        scaler.font_s(10.0),
                        Palette::NEON_GOLD,
                    );
                } else {
                    fonts.draw_ui_bold(
                        "✓ READY TO PLAY",
                        col_x + col_w - scaler.s(120.0),
                        curr_y + mp_card_h * 0.25,
                        scaler.font_s(10.0),
                        Palette::NEON_GREEN,
                    );
                }

                let title_str = if is_sel {
                    format!("▶ {}", item.title())
                } else {
                    item.title().to_string()
                };
                fonts.draw_display(
                    &title_str,
                    col_x + scaler.s(18.0),
                    curr_y + mp_card_h * 0.55,
                    scaler.font_s(17.0),
                    if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
                );

                fonts.draw_ui_regular(
                    item.description(),
                    col_x + scaler.s(18.0),
                    curr_y + mp_card_h * 0.84,
                    scaler.font_s(11.5),
                    if is_sel { Color::new(0.80, 0.85, 0.92, 1.0) } else { Palette::UI_TEXT_MUTED },
                );

                if is_sel {
                    let prompt_str = if is_avail {
                        "PRESS [ENTER] TO LAUNCH ▶"
                    } else {
                        "PRESS [ENTER] FOR INFO ▶"
                    };
                    fonts.draw_ui_bold(
                        prompt_str,
                        col_x + col_w - scaler.s(200.0),
                        curr_y + mp_card_h * 0.55,
                        scaler.font_s(11.0),
                        accent,
                    );
                }

                curr_y += mp_card_h + card_gap;
            }
        }
        ModalityCategory::Options => {
            let opt_items = ModalityCategory::Options.items();
            let card_gap = scaler.s(12.0);
            let opt_card_h = ((available_h - card_gap * (opt_items.len() as f32 - 1.0)) / opt_items.len() as f32)
                .clamp(scaler.s(68.0), scaler.s(110.0));

            let mut curr_y = start_y;
            for (i, item) in opt_items.iter().enumerate() {
                let is_sel = i == selected_idx;
                let accent = item.accent_color();

                let bg_col = if is_sel {
                    Palette::UI_CARD_BG_HOVER
                } else {
                    Color::new(0.06, 0.08, 0.12, 0.70)
                };
                let border_col = if is_sel {
                    accent
                } else {
                    Palette::UI_CARD_BORDER
                };

                scaler.draw_glass_card(col_x, curr_y, col_w, opt_card_h, bg_col, border_col, if is_sel { 2.4 } else { 1.0 });

                if is_sel {
                    draw_rectangle(col_x, curr_y, scaler.s(6.0), opt_card_h, accent);
                }

                fonts.draw_ui_bold(
                    item.tag(),
                    col_x + scaler.s(18.0),
                    curr_y + opt_card_h * 0.25,
                    scaler.font_s(9.5),
                    if is_sel { accent } else { Palette::UI_TEXT_MUTED },
                );

                // Right-aligned status indicators
                match item {
                    ModalityItem::PlayerProfile => {
                        let profile_str = format!("👤 {}", active_profile.name);
                        fonts.draw_ui_bold(
                            &profile_str,
                            col_x + col_w - scaler.s(180.0),
                            curr_y + opt_card_h * 0.25,
                            scaler.font_s(10.0),
                            Palette::NEON_CYAN,
                        );
                    }
                    ModalityItem::Garage => {
                        fonts.draw_ui_bold(
                            "🏎️ 360° SHOWROOM",
                            col_x + col_w - scaler.s(160.0),
                            curr_y + opt_card_h * 0.25,
                            scaler.font_s(10.0),
                            Palette::NEON_GOLD,
                        );
                    }
                    ModalityItem::Settings => {
                        fonts.draw_ui_bold(
                            "⚙️ ARCADE CONFIG",
                            col_x + col_w - scaler.s(160.0),
                            curr_y + opt_card_h * 0.25,
                            scaler.font_s(10.0),
                            Palette::NEON_MAGENTA,
                        );
                    }
                    _ => {}
                }

                let title_str = if is_sel {
                    format!("▶ {}", item.title())
                } else {
                    item.title().to_string()
                };
                fonts.draw_display(
                    &title_str,
                    col_x + scaler.s(18.0),
                    curr_y + opt_card_h * 0.55,
                    scaler.font_s(17.0),
                    if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) },
                );

                fonts.draw_ui_regular(
                    item.description(),
                    col_x + scaler.s(18.0),
                    curr_y + opt_card_h * 0.84,
                    scaler.font_s(11.5),
                    if is_sel { Color::new(0.80, 0.85, 0.92, 1.0) } else { Palette::UI_TEXT_MUTED },
                );

                if is_sel {
                    let prompt_str = match item {
                        ModalityItem::PlayerProfile => "PRESS [ENTER] OR [P] TO OPEN ▶",
                        ModalityItem::Garage => "PRESS [ENTER] OR [G] TO ENTER ▶",
                        ModalityItem::Settings => "PRESS [ENTER] OR [X] TO CONFIGURE ▶",
                        _ => "PRESS [ENTER] TO SELECT ▶",
                    };
                    fonts.draw_ui_bold(
                        prompt_str,
                        col_x + col_w - scaler.s(220.0),
                        curr_y + opt_card_h * 0.55,
                        scaler.font_s(11.0),
                        accent,
                    );
                }

                curr_y += opt_card_h + card_gap;
            }
        }
    }

    // Bottom Action Prompt / Controller Hints
    fonts.draw_ui_regular_centered(
        "[W/S or UP/DOWN] Navigate Card  •  [A/D or LEFT/RIGHT or TAB / 1/2/3] Switch Menu  •  [G] Garage  •  [P] Profile  •  [X] Settings  •  [ENTER/SPACE] Select  •  [ESC] Hub",
        sw * 0.5,
        sh - scaler.s(14.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    // Modal overlay for in-development features
    if let Some(m) = modal {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.80));

        let mw = (sw * 0.65).clamp(scaler.s(480.0), scaler.s(660.0));
        let mh = scaler.s(220.0);
        let mx = (sw - mw) * 0.5;
        let my = (sh - mh) * 0.5;

        scaler.draw_glass_card(
            mx,
            my,
            mw,
            mh,
            Color::new(0.07, 0.09, 0.14, 0.98),
            Palette::NEON_GOLD,
            2.5,
        );

        fonts.draw_display_centered_with_shadow(
            m.title(),
            sw * 0.5,
            my + scaler.s(36.0),
            scaler.font_s(17.0),
            Palette::NEON_GOLD,
            Color::new(0.0, 0.0, 0.0, 0.7),
            scaler.s(2.0),
        );

        let mut line_y = my + scaler.s(76.0);
        for line in m.message().lines() {
            fonts.draw_ui_regular_centered(
                line,
                sw * 0.5,
                line_y,
                scaler.font_s(12.5),
                Color::new(0.85, 0.90, 0.96, 1.0),
            );
            line_y += scaler.s(24.0);
        }

        fonts.draw_ui_bold_centered(
            "PRESS [ENTER / SPACE / ESC] OR GAMEPAD [A / B] TO DISMISS",
            sw * 0.5,
            my + mh - scaler.s(24.0),
            scaler.font_s(12.0),
            Palette::NEON_CYAN,
        );
    }
}


