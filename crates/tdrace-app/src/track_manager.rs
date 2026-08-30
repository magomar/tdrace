use std::fs;
use std::path::{Path, PathBuf};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
};
use tdrace_core::track::{Track, TrackCategory};

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
        if self.belongs_to_module("f1") {
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

    /// Scans the tracks directory and its subdirectories (drafts, classic, f1, rally, kart, etc.) for `.json` and `.tdtrack` files.
    pub fn scan_custom_tracks(&mut self) -> Result<usize, String> {
        self.custom_tracks.clear();

        if !self.tracks_dir.exists() {
            let _ = fs::create_dir_all(&self.tracks_dir);
            let _ = fs::create_dir_all(self.tracks_dir.join("drafts"));
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
                    if let Ok(mut track) = Track::load_from_file(&path) {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("custom_track")
                            .to_string();

                        let mut modules = track.modules.clone();
                        // Infer category and module_id from subdirectory if applicable
                        if let Some(ref dir) = subdir {
                            if dir == "drafts" {
                                track.category = TrackCategory::Draft;
                                track.module_id = None;
                                modules.clear();
                            } else {
                                track.category = TrackCategory::Main;
                                track.module_id = Some(dir.clone());
                                modules = vec![dir.clone()];
                            }
                        } else if track.category == TrackCategory::Main {
                            let mod_id = track.module_id.clone().unwrap_or_else(|| "classic".to_string());
                            track.module_id = Some(mod_id.clone());
                            if modules.is_empty() {
                                modules = vec![mod_id];
                            }
                        } else {
                            // Draft track in root
                            modules.clear();
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

    /// Returns all circuits for a specific registered game module or draft collection.
    pub fn module_catalog_tracks(&self, module_id: &str) -> Vec<TrackChoice> {
        match module_id {
            "f1" => {
                let mut list = vec![
                    TrackChoice::Custom {
                        id: "monza".to_string(),
                        title: "Monza Autodromo Nazionale".to_string(),
                        description: "Temple of Speed. 5.79km high-speed DRS straights & Variante del Rettifilo.".to_string(),
                        path: "f1/monza".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "spa".to_string(),
                        title: "Spa-Francorchamps GP".to_string(),
                        description: "Legendary Ardennes circuit featuring Eau Rouge, Raidillon, and Pouhon.".to_string(),
                        path: "f1/spa".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "silverstone".to_string(),
                        title: "Silverstone Circuit".to_string(),
                        description: "High-speed sweeping Maggotts, Becketts & Chapel apex complex.".to_string(),
                        path: "f1/silverstone".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "monaco".to_string(),
                        title: "Circuit de Monaco".to_string(),
                        description: "Prestigious Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.".to_string(),
                        path: "f1/monaco".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "suzuka".to_string(),
                        title: "Suzuka International Racing Course".to_string(),
                        description: "Technical Japanese figure-8 circuit with Esses, Degner, crossover bridge, and 130R.".to_string(),
                        path: "f1/suzuka".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "interlagos".to_string(),
                        title: "Autodromo Jose Carlos Pace (Interlagos)".to_string(),
                        description: "Anti-clockwise Brazilian thriller featuring Senna 'S', Curva do Sol, and Juncao.".to_string(),
                        path: "f1/interlagos".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "montreal".to_string(),
                        title: "Circuit Gilles Villeneuve (Montreal)".to_string(),
                        description: "Canadian island circuit with Virage Senna, L'Epingle hairpin, and Wall of Champions.".to_string(),
                        path: "f1/montreal".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "red_bull_ring".to_string(),
                        title: "Red Bull Ring (Spielberg)".to_string(),
                        description: "Undulating Austrian alpine sprint circuit with steep climbs and heavy braking into Remus.".to_string(),
                        path: "f1/red_bull_ring".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "catalunya".to_string(),
                        title: "Circuit de Barcelona-Catalunya".to_string(),
                        description: "Premier aerodynamic benchmark testing high-speed downforce and technical precision.".to_string(),
                        path: "f1/catalunya".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "zandvoort".to_string(),
                        title: "Circuit Zandvoort".to_string(),
                        description: "Seaside rollercoaster featuring high-banked Hugenholtz and Arie Luyendyk curves.".to_string(),
                        path: "f1/zandvoort".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "bahrain".to_string(),
                        title: "Bahrain International Circuit (Sakhir)".to_string(),
                        description: "Sakhir desert circuit with heavy Turn 1 braking and technical off-camber Turns 9-10.".to_string(),
                        path: "f1/bahrain".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "marina_bay".to_string(),
                        title: "Marina Bay Street Circuit (Singapore)".to_string(),
                        description: "Spectacular floodlit street race navigating tight harbor chicanes and city avenues.".to_string(),
                        path: "f1/marina_bay".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "cota".to_string(),
                        title: "Circuit of the Americas (COTA)".to_string(),
                        description: "Grand Prix venue featuring steep uphill Turn 1 blind crest and high-speed Esses.".to_string(),
                        path: "f1/cota".to_string(),
                    },
                    TrackChoice::ClassicGrandPrix,
                ];
                for custom in self.module_custom_tracks("f1") {
                    if !list.iter().any(|c| c.track_id() == custom.track_id()) {
                        list.push(custom);
                    }
                }
                list
            }
            "rally" => {
                let mut list = vec![
                    TrackChoice::OasisRally,
                    TrackChoice::OutlawPass,
                    TrackChoice::Custom {
                        id: "sahara".to_string(),
                        title: "Sahara Dunes Stage".to_string(),
                        description: "Fast undulating desert sand dunes and high-drift crests.".to_string(),
                        path: "rally/sahara".to_string(),
                    },
                ];
                for custom in self.module_custom_tracks("rally") {
                    if !list.iter().any(|c| c.track_id() == custom.track_id()) {
                        list.push(custom);
                    }
                }
                list
            }
            "kart" => {
                let mut list = vec![
                    TrackChoice::Custom {
                        id: "lonato".to_string(),
                        title: "South Garda Karting (Lonato)".to_string(),
                        description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.".to_string(),
                        path: "kart/lonato".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "sarno".to_string(),
                        title: "Circuito Internazionale Napoli (Sarno)".to_string(),
                        description: "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.".to_string(),
                        path: "kart/sarno".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "genk".to_string(),
                        title: "Karting Genk (Home of Champions)".to_string(),
                        description: "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.".to_string(),
                        path: "kart/genk".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "pfi".to_string(),
                        title: "PF International Kart Circuit".to_string(),
                        description: "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.".to_string(),
                        path: "kart/pfi".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "zuera".to_string(),
                        title: "Circuito Internacional de Zuera".to_string(),
                        description: "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.".to_string(),
                        path: "kart/zuera".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "le_mans_kart".to_string(),
                        title: "Le Mans Karting International".to_string(),
                        description: "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.".to_string(),
                        path: "kart/le_mans_kart".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "portimao_kart".to_string(),
                        title: "Kartodromo Internacional do Algarve".to_string(),
                        description: "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.".to_string(),
                        path: "kart/portimao_kart".to_string(),
                    },
                    TrackChoice::Custom {
                        id: "franciacorta".to_string(),
                        title: "Franciacorta Karting Track".to_string(),
                        description: "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.".to_string(),
                        path: "kart/franciacorta".to_string(),
                    },
                    TrackChoice::KartArena,
                    TrackChoice::DriftPark,
                ];
                for custom in self.module_custom_tracks("kart") {
                    if !list.iter().any(|c| c.track_id() == custom.track_id()) {
                        list.push(custom);
                    }
                }
                list
            }
            "classic" => {
                let mut list = Vec::from(TrackChoice::ALL);
                for custom in self.module_custom_tracks("classic") {
                    if !list.iter().any(|c| c.track_id() == custom.track_id()) {
                        list.push(custom);
                    }
                }
                list
            }
            "drafts" => self.draft_track_choices(),
            _ => {
                // "all"
                let mut list = Vec::new();
                list.extend(TrackChoice::ALL);
                list.push(TrackChoice::Custom {
                    id: "monza".to_string(),
                    title: "Monza Autodromo Nazionale".to_string(),
                    description: "Temple of Speed. 5.79km high-speed DRS straights & Variante del Rettifilo.".to_string(),
                    path: "f1/monza".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "spa".to_string(),
                    title: "Spa-Francorchamps GP".to_string(),
                    description: "Legendary Ardennes circuit featuring Eau Rouge, Raidillon, and Pouhon.".to_string(),
                    path: "f1/spa".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "silverstone".to_string(),
                    title: "Silverstone Circuit".to_string(),
                    description: "High-speed sweeping Maggotts, Becketts & Chapel apex complex.".to_string(),
                    path: "f1/silverstone".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "monaco".to_string(),
                    title: "Circuit de Monaco".to_string(),
                    description: "Prestigious Monte Carlo street circuit with Loews Hairpin, Tunnel, and Swimming Pool.".to_string(),
                    path: "f1/monaco".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "suzuka".to_string(),
                    title: "Suzuka International Racing Course".to_string(),
                    description: "Technical Japanese figure-8 circuit with Esses, Degner, crossover bridge, and 130R.".to_string(),
                    path: "f1/suzuka".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "interlagos".to_string(),
                    title: "Autodromo Jose Carlos Pace (Interlagos)".to_string(),
                    description: "Anti-clockwise Brazilian thriller featuring Senna 'S', Curva do Sol, and Juncao.".to_string(),
                    path: "f1/interlagos".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "montreal".to_string(),
                    title: "Circuit Gilles Villeneuve (Montreal)".to_string(),
                    description: "Canadian island circuit with Virage Senna, L'Epingle hairpin, and Wall of Champions.".to_string(),
                    path: "f1/montreal".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "red_bull_ring".to_string(),
                    title: "Red Bull Ring (Spielberg)".to_string(),
                    description: "Undulating Austrian alpine sprint circuit with steep climbs and heavy braking into Remus.".to_string(),
                    path: "f1/red_bull_ring".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "catalunya".to_string(),
                    title: "Circuit de Barcelona-Catalunya".to_string(),
                    description: "Premier aerodynamic benchmark testing high-speed downforce and technical precision.".to_string(),
                    path: "f1/catalunya".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "zandvoort".to_string(),
                    title: "Circuit Zandvoort".to_string(),
                    description: "Seaside rollercoaster featuring high-banked Hugenholtz and Arie Luyendyk curves.".to_string(),
                    path: "f1/zandvoort".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "bahrain".to_string(),
                    title: "Bahrain International Circuit (Sakhir)".to_string(),
                    description: "Sakhir desert circuit with heavy Turn 1 braking and technical off-camber Turns 9-10.".to_string(),
                    path: "f1/bahrain".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "marina_bay".to_string(),
                    title: "Marina Bay Street Circuit (Singapore)".to_string(),
                    description: "Spectacular floodlit street race navigating tight harbor chicanes and city avenues.".to_string(),
                    path: "f1/marina_bay".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "cota".to_string(),
                    title: "Circuit of the Americas (COTA)".to_string(),
                    description: "Grand Prix venue featuring steep uphill Turn 1 blind crest and high-speed Esses.".to_string(),
                    path: "f1/cota".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "sahara".to_string(),
                    title: "Sahara Dunes Stage".to_string(),
                    description: "Fast undulating desert sand dunes and high-drift crests.".to_string(),
                    path: "rally/sahara".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "lonato".to_string(),
                    title: "South Garda Karting (Lonato)".to_string(),
                    description: "The global Mecca of Karting featuring Curva del Paddock, Pettine hairpin, and Variante Nuova.".to_string(),
                    path: "kart/lonato".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "sarno".to_string(),
                    title: "Circuito Internazionale Napoli (Sarno)".to_string(),
                    description: "The Temple of Speed under Mount Vesuvius with massive full-throttle straights and technical Esses.".to_string(),
                    path: "kart/sarno".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "genk".to_string(),
                    title: "Karting Genk (Home of Champions)".to_string(),
                    description: "Legendary Belgian proving grounds featuring the high-G G-Curve carousel, Europabocht, and Champions Chicane.".to_string(),
                    path: "kart/genk".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "pfi".to_string(),
                    title: "PF International Kart Circuit".to_string(),
                    description: "Britain's premier FIA kart venue featuring the world-famous elevated flyover crossover bridge and underpass.".to_string(),
                    path: "kart/pfi".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "zuera".to_string(),
                    title: "Circuito Internacional de Zuera".to_string(),
                    description: "Ultra-fast Spanish supertrack with enormous drafting straights, Curva del Cierzo, and wide passing sweepers.".to_string(),
                    path: "kart/zuera".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "le_mans_kart".to_string(),
                    title: "Le Mans Karting International".to_string(),
                    description: "Alain Prost circuit at the Le Mans 24 Hours complex with Dunlop chicane, Bugatti Esses, and Courbe des 24H.".to_string(),
                    path: "kart/le_mans_kart".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "portimao_kart".to_string(),
                    title: "Kartodromo Internacional do Algarve".to_string(),
                    description: "Undulating Portuguese rollercoaster circuit with dramatic elevation drops, sweeping downhill turns, and Curva do Sol.".to_string(),
                    path: "kart/portimao_kart".to_string(),
                });
                list.push(TrackChoice::Custom {
                    id: "franciacorta".to_string(),
                    title: "Franciacorta Karting Track".to_string(),
                    description: "Modern premier Italian world championship venue with technical switchback chicanes and trail-braking hairpins.".to_string(),
                    path: "kart/franciacorta".to_string(),
                });
                for custom in &self.custom_tracks {
                    if custom.category == TrackCategory::Main && !list.iter().any(|c| c.track_id() == custom.id) {
                        list.push(TrackChoice::Custom {
                            id: custom.id.clone(),
                            title: custom.title.clone(),
                            description: custom.description.clone(),
                            path: custom.file_path.clone(),
                        });
                    }
                }
                list
            }
        }
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
                    "sahara" => Ok(tdrace_core::track::presets::sahara_dunes()),
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

    /// Returns the target subdirectory for a track based on its category and module assignment.
    pub fn target_dir_for_track(&self, category: TrackCategory, module_id: Option<&str>) -> PathBuf {
        match category {
            TrackCategory::Draft => self.tracks_dir.join("drafts"),
            TrackCategory::Main => {
                let mod_name = module_id.unwrap_or("classic");
                self.tracks_dir.join(mod_name)
            }
        }
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
        let candidates = [
            self.tracks_dir.join("drafts").join(&file_name),
            self.tracks_dir.join("classic").join(&file_name),
            self.tracks_dir.join("f1").join(&file_name),
            self.tracks_dir.join("rally").join(&file_name),
            self.tracks_dir.join("kart").join(&file_name),
            self.tracks_dir.join(&file_name),
        ];
        for cand in &candidates {
            if cand.exists() {
                return cand.clone();
            }
        }
        self.tracks_dir.join("drafts").join(file_name)
    }

    /// Saves a track to disk with overwrite control in the appropriate category/module subdirectory.
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
        // Otherwise, newly created custom tracks land in Drafts by default.
        let existing_path = self.track_path_for_slug(&base_slug);
        if overwrite && existing_path.exists() {
            if let Ok(existing) = Track::load_from_file(&existing_path) {
                track_to_save.category = existing.category;
                track_to_save.module_id = existing.module_id;
                track_to_save.modules = existing.modules;
            }
        } else {
            track_to_save.category = TrackCategory::Draft;
            track_to_save.module_id = None;
            track_to_save.modules.clear();
        }

        // Determine target directory
        let target_dir = self.target_dir_for_track(track_to_save.category, track_to_save.module_id.as_deref());
        let _ = fs::create_dir_all(&target_dir);

        let file_slug = if overwrite {
            base_slug
        } else {
            let mut candidate = base_slug.clone();
            let mut counter = 1;
            while target_dir.join(format!("{}.json", candidate)).exists()
                || self.tracks_dir.join(format!("{}.json", candidate)).exists()
            {
                candidate = format!("{}_{}", base_slug, counter);
                counter += 1;
            }
            candidate
        };

        let file_name = format!("{}.json", file_slug);
        let path = target_dir.join(file_name);

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

    /// Promotes a track from Draft to Main category (Approved circuit) assigned to a specific module,
    /// moving the file from `drafts/` to `<module_id>/`.
    pub fn promote_track_to_module(&mut self, id: &str, module_id: &str) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let old_path_str = self.custom_tracks[pos].file_path.clone();
            let old_path = PathBuf::from(&old_path_str);
            let mut track = Track::load_from_file(&old_path)
                .map_err(|e| format!("Failed to load track to promote: {}", e))?;
            track.category = TrackCategory::Main;
            track.module_id = Some(module_id.to_string());
            track.modules = vec![module_id.to_string()];

            let target_dir = self.tracks_dir.join(module_id);
            let _ = fs::create_dir_all(&target_dir);
            let target_path = target_dir.join(format!("{}.json", id));

            track
                .save_to_file(&target_path)
                .map_err(|e| format!("Failed to save promoted track: {}", e))?;

            if old_path != target_path && old_path.exists() {
                let _ = fs::remove_file(&old_path);
            }

            let _ = self.scan_custom_tracks();
            Ok(())
        } else {
            Err(format!("Custom track '{}' not found", id))
        }
    }

    /// Promotes a track from Draft to Main category with default "classic" module.
    pub fn promote_track(&mut self, id: &str) -> Result<(), String> {
        self.promote_track_to_module(id, "classic")
    }

    /// Demotes a track from Main category back to Draft / Testing, moving the file back to `drafts/`.
    pub fn demote_track(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let old_path_str = self.custom_tracks[pos].file_path.clone();
            let old_path = PathBuf::from(&old_path_str);
            let mut track = Track::load_from_file(&old_path)
                .map_err(|e| format!("Failed to load track to demote: {}", e))?;
            track.category = TrackCategory::Draft;
            track.module_id = None;
            track.modules.clear();

            let target_dir = self.tracks_dir.join("drafts");
            let _ = fs::create_dir_all(&target_dir);
            let target_path = target_dir.join(format!("{}.json", id));

            track
                .save_to_file(&target_path)
                .map_err(|e| format!("Failed to save demoted track: {}", e))?;

            if old_path != target_path && old_path.exists() {
                let _ = fs::remove_file(&old_path);
            }

            let _ = self.scan_custom_tracks();
            Ok(())
        } else {
            Err(format!("Custom track '{}' not found", id))
        }
    }

    /// Updates the display name and description of a custom track and writes changes to disk.
    pub fn update_track_metadata(&mut self, id: &str, new_title: String, new_description: String) -> Result<(), String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let path = PathBuf::from(&path_str);
            let mut track = Track::load_from_file(&path)
                .map_err(|e| format!("Failed to load track for metadata update: {}", e))?;
            track.name = new_title;
            track.description = new_description;

            track
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save updated track metadata: {}", e))?;

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

    /// Deletes a custom track file from disk.
    pub fn delete_custom_track(&mut self, id: &str) -> Result<bool, String> {
        if let Some(pos) = self.custom_tracks.iter().position(|t| t.id == id) {
            let path_str = self.custom_tracks[pos].file_path.clone();
            let path = PathBuf::from(&path_str);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| format!("Failed to delete track file: {}", e))?;
            }
            let _ = self.scan_custom_tracks();
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
        assert_eq!(choices.len(), 29); // 7 classic + 13 f1 + 1 rally unique + 8 famous kart

        let mut gp = classic_grand_prix();
        gp.name = "My Custom GP".to_string();
        gp.description = "A custom testing GP".to_string();
        gp.category = TrackCategory::Draft;

        let saved_path = manager
            .save_custom_track(&gp, Some("test_custom_gp"))
            .expect("Must save custom track");
        assert!(Path::new(&saved_path).exists());

        // Since gp was saved as Draft, main choices is still 29, but draft choices has 1
        assert_eq!(manager.main_track_choices().len(), 29);
        assert_eq!(manager.draft_track_choices().len(), 1);

        let draft_choice = &manager.draft_track_choices()[0];
        assert_eq!(draft_choice.title(), "My Custom GP");
        assert_eq!(draft_choice.description(), "A custom testing GP");

        // Promote track to Main
        manager.promote_track("test_custom_gp").expect("Must promote");
        assert_eq!(manager.main_track_choices().len(), 30);
        assert_eq!(manager.draft_track_choices().len(), 0);

        // Edit metadata
        manager
            .update_track_metadata(
                "test_custom_gp",
                "Renamed Grand Prix".to_string(),
                "Updated description text".to_string(),
            )
            .expect("Must update metadata");
        let loaded = manager.load_track(&manager.main_track_choices()[29]).expect("Must load");
        assert_eq!(loaded.name, "Renamed Grand Prix");
        assert_eq!(loaded.description, "Updated description text");

        // Demote back to draft
        manager.demote_track("test_custom_gp").expect("Must demote");
        assert_eq!(manager.main_track_choices().len(), 29);
        assert_eq!(manager.draft_track_choices().len(), 1);

        // Clean up
        assert!(manager.delete_custom_track("test_custom_gp").unwrap());
        assert_eq!(manager.main_track_choices().len(), 29);
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
        assert_eq!(classic_tracks.len(), 7);

        // F1 tracks
        let f1_tracks = manager.module_catalog_tracks("f1");
        assert_eq!(f1_tracks.len(), 14);
        assert!(f1_tracks.iter().any(|t| t.title().contains("Monza")));
        assert!(f1_tracks.iter().any(|t| t.title().contains("Spa")));
        assert!(f1_tracks.iter().any(|t| t.title().contains("Silverstone")));

        // Rally tracks
        let rally_tracks = manager.module_catalog_tracks("rally");
        assert_eq!(rally_tracks.len(), 3);
        assert!(rally_tracks.iter().any(|t| t.title().contains("Sahara")));

        // Kart tracks
        let kart_tracks = manager.module_catalog_tracks("kart");
        assert_eq!(kart_tracks.len(), 10);
        assert!(kart_tracks.iter().any(|t| t.title().contains("Lonato")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Sarno")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("Genk")));
        assert!(kart_tracks.iter().any(|t| t.title().contains("PFI") || t.title().contains("PF International")));

        // All tracks
        let all_tracks = manager.module_catalog_tracks("all");
        assert_eq!(all_tracks.len(), 29);

        // Save a draft
        let mut draft = classic_grand_prix();
        draft.name = "My Draft Circuit".to_string();
        let _ = manager.save_custom_track_with_options(&draft, Some("my_draft"), false);

        let drafts = manager.module_catalog_tracks("drafts");
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].title(), "My Draft Circuit");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
