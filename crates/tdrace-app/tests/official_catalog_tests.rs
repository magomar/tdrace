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

/// Until step 6 removes the Rust catalog, each `TrackDefinition` must match its JSON file.
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
