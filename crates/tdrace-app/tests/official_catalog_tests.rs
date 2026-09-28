//! Official circuit catalog tests (specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tdrace_app::module::{
    classic::ClassicGameModule, extreme_offroad::ExtremeOffRoadModule, gt::GtWorldChallengeModule,
    kart::KartGameModule, nascar::NascarGameModule, rally::RallyGameModule, GameModule,
};
use tdrace_core::track::Track;

fn tracks_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks")
}

fn modules() -> Vec<Box<dyn GameModule>> {
    vec![
        Box::new(ClassicGameModule::new()),
        Box::new(GtWorldChallengeModule::new()),
        Box::new(RallyGameModule::new()),
        Box::new(KartGameModule::new()),
        Box::new(NascarGameModule::new()),
        Box::new(ExtremeOffRoadModule::new()),
    ]
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let data = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("{}: {} (run `git submodule update --init tracks`)", path.display(), e)
    });
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("{}: {}", path.display(), e))
}

/// Each module's `TrackDefinition` list (read from the embedded catalog) must match its JSON file on disk.
#[test]
fn test_catalog_metadata_matches_json() {
    let root = tracks_dir();
    for module in modules() {
        for def in module.tracks() {
            let path = root.join(module.id()).join(format!("{}.json", def.id));
            let track: Track = read_json(&path);
            let at = format!("{}/{}", module.id(), def.id);
            assert_eq!(track.name, def.title, "name of {}", at);
            assert_eq!(track.description, def.description, "description of {}", at);
            assert_eq!(track.tag, def.tag, "tag of {}", at);
            assert_eq!(track.category_label, def.category, "category_label of {}", at);
            assert_eq!(track.default_laps, def.default_laps, "default_laps of {}", at);
        }
    }
}

#[test]
fn test_track_order_lists_every_official_file_once() {
    let root = tracks_dir();
    let order: BTreeMap<String, Vec<String>> = read_json(&root.join(".track_order.json"));
    for module in modules() {
        let listed = order.get(module.id()).unwrap_or_else(|| panic!("no order for {}", module.id()));
        let rust_order: Vec<String> = module.tracks().iter().map(|d| d.id.to_string()).collect();
        assert_eq!(listed, &rust_order, "order of {}", module.id());

        let on_disk: BTreeSet<String> = std::fs::read_dir(root.join(module.id()))
            .unwrap()
            .filter_map(|e| {
                let p = e.ok()?.path();
                if p.extension()? != "json" {
                    return None;
                }
                Some(p.file_stem()?.to_string_lossy().into_owned())
            })
            .collect();
        let listed_set: BTreeSet<String> = listed.iter().cloned().collect();
        assert_eq!(listed_set.len(), listed.len(), "duplicate in order of {}", module.id());
        assert_eq!(on_disk, listed_set, "files vs order in {}", module.id());
    }
}

#[test]
fn test_aliases_point_to_official_files() {
    let root = tracks_dir();
    let aliases: BTreeMap<String, String> = read_json(&root.join(".aliases.json"));
    let ids: BTreeSet<String> = modules()
        .iter()
        .flat_map(|m| m.tracks().into_iter().map(|d| d.id.to_string()).collect::<Vec<_>>())
        .collect();
    for (alias, id) in &aliases {
        assert!(!ids.contains(alias), "alias {} shadows an official id", alias);
        assert!(ids.contains(id), "alias {} points to unknown id {}", alias, id);
    }
    assert_eq!(aliases.get("daytona").map(String::as_str), Some("daytona_superspeedway"));
}

// --- One resolver for official circuits (spec 042 §2.3, §2.4) ---

use tdrace_app::storage::{ENV_DEV_MODE, ENV_GIT_TRACKS_DIR};
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::menu::{clear_menu_track_cache, resolve_track_for_menu_with_dir, TrackChoice};

/// Serializes tests in this binary that change process-wide environment variables.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct EnvGuard;
impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var(ENV_DEV_MODE);
        std::env::remove_var(ENV_GIT_TRACKS_DIR);
    }
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tdrace_official_{}_{}_{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn monza_choice() -> TrackChoice {
    TrackChoice::Custom {
        id: "monza".to_string(),
        title: "Monza".to_string(),
        description: String::new(),
        path: "gt/monza".to_string(),
    }
}

fn official_monza_name() -> String {
    tdrace_core::catalog::official_track("gt", "monza").name
}

/// A git tracks dir holding a changed copy of Monza.
fn git_dir_with_changed_monza() -> PathBuf {
    let git = temp_dir("git");
    let mut monza = tdrace_core::catalog::official_track("gt", "monza");
    monza.name = "Monza (disk edit)".to_string();
    monza.save_to_file(git.join("gt").join("monza.json")).unwrap();
    git
}

#[test]
fn test_normal_mode_ignores_disk_tracks_dir() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    std::env::remove_var(ENV_DEV_MODE);
    std::env::set_var(ENV_GIT_TRACKS_DIR, git_dir_with_changed_monza());
    clear_menu_track_cache();

    let user = temp_dir("user");
    let manager = TrackManager::new(&user);
    assert_eq!(manager.load_track(&monza_choice()).unwrap().name, official_monza_name());
    assert_eq!(manager.load_track_by_slug("monza").unwrap().name, official_monza_name());
    let menu = resolve_track_for_menu_with_dir(&monza_choice(), &user).unwrap();
    assert_eq!(menu.name, official_monza_name());
}

#[test]
fn test_dev_mode_reads_official_circuit_from_disk() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    std::env::set_var(ENV_DEV_MODE, "1");
    std::env::set_var(ENV_GIT_TRACKS_DIR, git_dir_with_changed_monza());
    clear_menu_track_cache();

    let user = temp_dir("user");
    let manager = TrackManager::new(&user);
    assert_eq!(manager.load_track(&monza_choice()).unwrap().name, "Monza (disk edit)");
    assert_eq!(manager.load_track_by_slug("monza").unwrap().name, "Monza (disk edit)");
    let menu = resolve_track_for_menu_with_dir(&monza_choice(), &user).unwrap();
    assert_eq!(menu.name, "Monza (disk edit)");

    // A circuit that is not on disk still comes from the embedded catalog.
    assert_eq!(
        manager.load_track_by_slug("spa").unwrap().name,
        tdrace_core::catalog::official_track("gt", "spa").name
    );
}

#[test]
fn test_user_file_cannot_shadow_an_official_circuit() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    std::env::remove_var(ENV_DEV_MODE);
    std::env::set_var(ENV_GIT_TRACKS_DIR, temp_dir("empty_git"));
    clear_menu_track_cache();

    let user = temp_dir("user");
    let mut shadow = tdrace_core::catalog::official_track("gt", "monza");
    shadow.name = "Monza (user copy)".to_string();
    shadow.save_to_file(user.join("monza.json")).unwrap();
    shadow.save_to_file(user.join("gt").join("monza.json")).unwrap();

    let manager = TrackManager::new(&user);
    assert_eq!(manager.load_track(&monza_choice()).unwrap().name, official_monza_name());
    let menu = resolve_track_for_menu_with_dir(&monza_choice(), &user).unwrap();
    assert_eq!(menu.name, official_monza_name());
}

#[test]
fn test_old_short_slug_and_module_hint_resolve() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    std::env::remove_var(ENV_DEV_MODE);
    clear_menu_track_cache();

    let user = temp_dir("user");
    let manager = TrackManager::new(&user);
    let daytona = manager.load_track_by_slug("daytona").unwrap();
    assert_eq!(daytona, tdrace_core::catalog::official_track("nascar", "daytona_superspeedway"));

    let offroad_eight = TrackChoice::Custom {
        id: "dirt_figure_eight".to_string(),
        title: String::new(),
        description: String::new(),
        path: "extreme_offroad/dirt_figure_eight".to_string(),
    };
    let expected = tdrace_core::catalog::official_track("extreme_offroad", "dirt_figure_eight");
    assert_eq!(manager.load_track(&offroad_eight).unwrap(), expected);
    assert_eq!(resolve_track_for_menu_with_dir(&offroad_eight, &user).unwrap(), expected);
}

// --- Saving (spec 042 §2.5) ---

fn json_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if p.is_dir() {
            out.extend(json_files_under(&p));
        } else if p.extension().is_some_and(|e| e == "json") {
            out.push(p);
        }
    }
    out
}

#[test]
fn test_dev_save_writes_only_the_official_json_in_every_module() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    let git = temp_dir("git_save");
    std::env::set_var(ENV_DEV_MODE, "1");
    std::env::set_var(ENV_GIT_TRACKS_DIR, &git);

    let user = temp_dir("user_save");
    let mut manager = TrackManager::new(&user);
    for module in ["classic", "gt", "rally", "kart", "nascar", "extreme_offroad"] {
        let circuit = tdrace_core::catalog::module_circuits(module).next().unwrap();
        let mut track = circuit.load().unwrap();
        track.description = format!("dev edit of {}", circuit.id);
        let saved = manager.save_custom_track_with_options(&track, Some(circuit.id), true).unwrap();
        let expected = git.join(module).join(format!("{}.json", circuit.id));
        assert_eq!(Path::new(&saved).canonicalize().unwrap(), expected.canonicalize().unwrap(), "{}", module);
        assert_eq!(Track::load_from_file(&expected).unwrap().description, track.description);
    }
    assert!(json_files_under(&user).is_empty(), "user folder must stay empty: {:?}", json_files_under(&user));
}

#[test]
fn test_promote_uses_the_picked_module_and_removes_the_user_copy() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard;
    let git = temp_dir("git_promote");
    std::env::set_var(ENV_DEV_MODE, "1");
    std::env::set_var(ENV_GIT_TRACKS_DIR, &git);

    let user = temp_dir("user_promote");
    let mut manager = TrackManager::new(&user);
    let mut custom = tdrace_core::track::presets::create_prototypical_track(
        "classic",
        tdrace_core::track::presets::TrackShape::Oval,
        tdrace_core::track::presets::RaceDirection::Right,
    );
    custom.name = "Promo Oval".to_string();
    custom.module_id = Some("classic".to_string());
    custom.modules = vec!["classic".to_string()];
    manager.save_custom_track_with_options(&custom, Some("promo_oval"), true).unwrap();
    assert!(user.join("promo_oval.json").exists());

    let promoted = manager.promote_custom_track_to_git_preset("promo_oval", Some("nascar")).unwrap();
    assert_eq!(promoted.canonicalize().unwrap(), git.join("nascar").join("promo_oval.json").canonicalize().unwrap());
    let on_disk = Track::load_from_file(&promoted).unwrap();
    assert_eq!(on_disk.module_id.as_deref(), Some("nascar"));
    assert!(!user.join("promo_oval.json").exists(), "user copy must be removed");
    assert!(user.join(".backup").join("promo_oval.json").exists(), "user copy must be backed up");
}
