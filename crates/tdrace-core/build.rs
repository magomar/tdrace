//! Embeds the official circuits from `tracks/` (the `tdrace-tracks` submodule) into the crate.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md`.
//! Each `tracks/<module>/<id>.json` is re-serialized as compact JSON, DEFLATE-compressed into
//! `OUT_DIR`, and listed in a generated `official_catalog.rs` in `.track_order.json` order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const SUBMODULE_HINT: &str = "run `git submodule update --init tracks`, or set TDRACE_GIT_TRACKS_DIR";

/// Same variable the app uses for dev-mode reads (`storage.rs`, `tracks/README.md`).
/// A relative path is taken from the workspace root.
const TRACKS_DIR_ENV: &str = "TDRACE_GIT_TRACKS_DIR";

fn fail(msg: String) -> ! {
    panic!("\n\ntdrace-core: cannot embed official circuits: {}\n\n", msg);
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let data = fs::read_to_string(path).unwrap_or_else(|e| fail(format!("{}: {} ({})", path.display(), e, SUBMODULE_HINT)));
    serde_json::from_str(&data).unwrap_or_else(|e| fail(format!("{}: {}", path.display(), e)))
}

fn str_field(value: &serde_json::Value, key: &str) -> String {
    value.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.join("../..");
    println!("cargo:rerun-if-env-changed={}", TRACKS_DIR_ENV);
    let tracks_dir = match std::env::var(TRACKS_DIR_ENV) {
        Ok(dir) if !dir.trim().is_empty() => workspace_root.join(dir.trim()),
        _ => workspace_root.join("tracks"),
    };
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", tracks_dir.display());
    let order_path = tracks_dir.join(".track_order.json");
    let aliases_path = tracks_dir.join(".aliases.json");
    println!("cargo:rerun-if-changed={}", order_path.display());
    println!("cargo:rerun-if-changed={}", aliases_path.display());

    if !order_path.exists() {
        fail(format!("{} is missing; {}", order_path.display(), SUBMODULE_HINT));
    }
    let order: BTreeMap<String, Vec<String>> = read_json(&order_path);
    let aliases: BTreeMap<String, String> = read_json(&aliases_path);

    let mut generated = String::from("pub(crate) static CIRCUITS: &[EmbeddedCircuit] = &[\n");
    let mut ids = BTreeSet::new();

    for (module, listed) in &order {
        let module_dir = tracks_dir.join(module);
        println!("cargo:rerun-if-changed={}", module_dir.display());

        let mut on_disk = BTreeSet::new();
        let entries = fs::read_dir(&module_dir).unwrap_or_else(|e| fail(format!("{}: {}", module_dir.display(), e)));
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            if path.extension().is_some_and(|e| e == "json") && !file_name.eq_ignore_ascii_case("manifest.json") {
                on_disk.insert(path.file_stem().unwrap().to_string_lossy().into_owned());
            }
        }
        let listed_set: BTreeSet<String> = listed.iter().cloned().collect();
        if listed_set.len() != listed.len() {
            fail(format!(".track_order.json lists an id twice in module '{}'", module));
        }
        if listed_set != on_disk {
            let missing: Vec<_> = on_disk.difference(&listed_set).collect();
            let extra: Vec<_> = listed_set.difference(&on_disk).collect();
            fail(format!(
                "tracks/{}/ does not match .track_order.json (files not listed: {:?}; listed without file: {:?})",
                module, missing, extra
            ));
        }

        let blob_dir = out_dir.join("tracks").join(module);
        fs::create_dir_all(&blob_dir).unwrap();
        for id in listed {
            let path = module_dir.join(format!("{}.json", id));
            println!("cargo:rerun-if-changed={}", path.display());
            let value: serde_json::Value = read_json(&path);
            let compact = serde_json::to_vec(&value).unwrap();
            let compressed = miniz_oxide::deflate::compress_to_vec(&compact, 6);
            let blob_path = blob_dir.join(format!("{}.deflate", id));
            fs::write(&blob_path, &compressed).unwrap();

            let default_laps = value.get("default_laps").and_then(|v| v.as_u64()).unwrap_or(3);
            writeln!(
                generated,
                "    EmbeddedCircuit {{ module: {:?}, id: {:?}, name: {:?}, description: {:?}, tag: {:?}, category_label: {:?}, country_code: {:?}, default_laps: {}, data: include_bytes!({:?}) }},",
                module,
                id,
                str_field(&value, "name"),
                str_field(&value, "description"),
                str_field(&value, "tag"),
                str_field(&value, "category_label"),
                str_field(&value, "country_code"),
                default_laps,
                blob_path.display().to_string(),
            )
            .unwrap();
            ids.insert(id.clone());
        }
    }
    generated.push_str("];\n\npub(crate) static ALIASES: &[(&str, &str)] = &[\n");
    for (alias, id) in &aliases {
        if !ids.contains(id) {
            fail(format!(".aliases.json maps '{}' to unknown id '{}'", alias, id));
        }
        writeln!(generated, "    ({:?}, {:?}),", alias, id).unwrap();
    }
    generated.push_str("];\n");

    if ids.is_empty() {
        fail(format!("no circuits found in {}; {}", tracks_dir.display(), SUBMODULE_HINT));
    }
    fs::write(out_dir.join("official_catalog.rs"), generated).unwrap();
}
