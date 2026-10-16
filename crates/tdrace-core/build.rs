//! Embeds the official circuits from `tracks/` (the `tdrace-tracks` submodule) into the crate.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md` and
//! `specs/121_two_stage_lazy_circuit_menu_loading_and_compile_time_card_metadata.md`.
//! Each `tracks/<module>/<id>.json` is re-serialized as compact JSON, DEFLATE-compressed into
//! `OUT_DIR`, and listed in a generated `official_catalog.rs` in `.track_order.json` order
//! along with precomputed card metadata and downsampled thumbnail polylines.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const SUBMODULE_HINT: &str = "run `git submodule update --init tracks`, or set TDRACE_GIT_TRACKS_DIR";

/// Same variable the app uses for dev-mode reads (`storage.rs`, `tracks/README.md`).
/// A relative path is taken from the workspace root.
const TRACKS_DIR_ENV: &str = "TDRACE_GIT_TRACKS_DIR";

const MAX_THUMBNAIL_SAMPLES: usize = 160;
const MAX_BRANCH_THUMBNAIL_SAMPLES: usize = 64;

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

fn parse_surface(name: &str) -> (&'static str, &'static str) {
    match name {
        "Asphalt" => ("SurfaceType::Asphalt", "Asphalt"),
        "Concrete" => ("SurfaceType::Concrete", "Concrete"),
        "Curb" => ("SurfaceType::Curb", "Curb"),
        "Dirt" => ("SurfaceType::Dirt", "Dirt"),
        "PackedGravel" | "Gravel" => ("SurfaceType::PackedGravel", "Packed Gravel"),
        "Grass" => ("SurfaceType::Grass", "Grass"),
        "PackedSand" => ("SurfaceType::PackedSand", "Packed Sand"),
        "DeepSand" => ("SurfaceType::DeepSand", "Deep Sand"),
        "MudTrack" => ("SurfaceType::MudTrack", "Mud Track"),
        "DeepMud" => ("SurfaceType::DeepMud", "Deep Mud"),
        "PackedSnow" => ("SurfaceType::PackedSnow", "Packed Snow"),
        "DeepSnow" => ("SurfaceType::DeepSnow", "Deep Snow"),
        "SheetIce" => ("SurfaceType::SheetIce", "Sheet Ice"),
        "Water" => ("SurfaceType::Water", "Water"),
        "Oil" => ("SurfaceType::Oil", "Oil"),
        "DeepGravel" => ("SurfaceType::DeepGravel", "Deep Gravel"),
        _ => ("SurfaceType::Asphalt", "Asphalt"),
    }
}

fn extract_point_and_surface(sample: &serde_json::Value, fallback_surface: &str) -> Option<([f32; 2], &'static str, &'static str)> {
    let pt = sample.get("point")?.as_array()?;
    if pt.len() < 2 {
        return None;
    }
    let x = pt[0].as_f64()? as f32;
    let y = pt[1].as_f64()? as f32;
    let surf_str = sample.get("surface").and_then(|v| v.as_str()).unwrap_or(fallback_surface);
    let (tok, disp) = parse_surface(surf_str);
    Some(([x, y], tok, disp))
}

fn downsample_points(pts: &[([f32; 2], &'static str, &'static str)], max_len: usize) -> Vec<([f32; 2], &'static str)> {
    let n = pts.len();
    if n == 0 || max_len == 0 {
        return Vec::new();
    }
    if n <= max_len {
        return pts.iter().map(|(p, tok, _)| (*p, *tok)).collect();
    }
    (0..max_len)
        .map(|i| {
            let idx = (i * (n - 1)) / (max_len - 1);
            (pts[idx].0, pts[idx].1)
        })
        .collect()
}

fn compute_surface_summary(samples: &[([f32; 2], &'static str, &'static str)], closed: bool, default_surface_name: &str) -> String {
    if samples.len() < 2 {
        return format!("100% {}", default_surface_name);
    }
    let mut surface_lengths: BTreeMap<&'static str, f32> = BTreeMap::new();
    let mut total_len = 0.0f32;
    let n = if closed { samples.len() } else { samples.len().saturating_sub(1) };
    for i in 0..n {
        let next_i = (i + 1) % samples.len();
        let dx = samples[next_i].0[0] - samples[i].0[0];
        let dy = samples[next_i].0[1] - samples[i].0[1];
        let seg_len = (dx * dx + dy * dy).sqrt();
        let surf_name = samples[i].2;
        *surface_lengths.entry(surf_name).or_insert(0.0) += seg_len;
        total_len += seg_len;
    }
    if total_len <= 1e-4 {
        return format!("100% {}", default_surface_name);
    }
    let mut breakdown: Vec<(&'static str, f32)> = surface_lengths
        .into_iter()
        .map(|(name, len)| (name, (len / total_len) * 100.0))
        .collect();
    breakdown.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    if breakdown.len() == 1 {
        return format!("100% {}", breakdown[0].0);
    }
    breakdown
        .iter()
        .map(|(name, pct)| format!("{:.0}% {}", pct.round(), name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn extract_branch_segments(value: &serde_json::Value, fallback_surface: &str) -> Vec<Vec<([f32; 2], &'static str, &'static str)>> {
    let Some(net) = value.get("network") else {
        return Vec::new();
    };
    let default_layout_id = net.get("default_layout_id").and_then(|v| v.as_str()).unwrap_or("");
    let layouts = net.get("layouts").and_then(|v| v.as_array());
    let main_seq: Vec<u64> = layouts
        .and_then(|arr| {
            arr.iter().find(|l| l.get("id").and_then(|v| v.as_str()) == Some(default_layout_id))
        })
        .and_then(|l| l.get("segment_sequence"))
        .and_then(|v| v.as_array())
        .map(|seq| seq.iter().filter_map(|v| v.as_u64()).collect())
        .unwrap_or_default();

    let chute_id = net
        .get("launch_chute")
        .and_then(|c| c.get("segment_id"))
        .and_then(|v| v.as_u64());

    let Some(segments) = net.get("segments").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut branches = Vec::new();
    for seg in segments {
        let Some(seg_id) = seg.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let is_chute = Some(seg_id) == chute_id;
        let is_joker = !main_seq.contains(&seg_id)
            && layouts.is_some_and(|arr| {
                arr.iter().any(|l| {
                    l.get("segment_sequence")
                        .and_then(|v| v.as_array())
                        .is_some_and(|seq| seq.iter().any(|x| x.as_u64() == Some(seg_id)))
                })
            });
        if is_chute || is_joker {
            if let Some(s_arr) = seg.get("samples").and_then(|v| v.as_array()) {
                let pts: Vec<_> = s_arr
                    .iter()
                    .filter_map(|s| extract_point_and_surface(s, fallback_surface))
                    .collect();
                if !pts.is_empty() {
                    branches.push(pts);
                }
            }
        }
    }
    branches
}

fn format_sample_slice(pts: &[([f32; 2], &'static str)]) -> String {
    let mut out = String::from("&[");
    for (i, (pt, surf_tok)) in pts.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write!(out, "([{:.3}_f32, {:.3}_f32], {})", pt[0], pt[1], surf_tok).unwrap();
    }
    out.push(']');
    out
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
            let default_surf_str = value.get("default_surface").and_then(|v| v.as_str()).unwrap_or("Grass");
            let (default_surf_tok, default_surf_name) = parse_surface(default_surf_str);
            let scale_str = value
                .get("scale")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("1:1")
                .to_string();
            let is_inspired = value.get("is_inspired").and_then(|v| v.as_bool()).unwrap_or(false);

            let spline_obj = value.get("spline");
            let closed = spline_obj
                .and_then(|s| s.get("closed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let total_length_m = spline_obj
                .and_then(|s| s.get("total_length"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            let raw_samples: Vec<([f32; 2], &'static str, &'static str)> = spline_obj
                .and_then(|s| s.get("samples"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| extract_point_and_surface(s, default_surf_str))
                        .collect()
                })
                .unwrap_or_default();

            let surface_summary = compute_surface_summary(&raw_samples, closed, default_surf_name);
            let raw_branches = extract_branch_segments(&value, default_surf_str);

            // For thumbnail points & bounds: use spline.samples if present, else spline.waypoints, else arena boundary_hull
            let fallback_pts: Vec<([f32; 2], &'static str, &'static str)> = if !raw_samples.is_empty() {
                Vec::new()
            } else if let Some(wps) = spline_obj.and_then(|s| s.get("waypoints")).and_then(|v| v.as_array()).filter(|a| !a.is_empty()) {
                wps.iter()
                    .filter_map(|wp| extract_point_and_surface(wp, default_surf_str))
                    .collect()
            } else if let Some(hull) = value.get("kind").and_then(|k| k.get("boundary_hull")).and_then(|v| v.as_array()).filter(|a| !a.is_empty()) {
                let floor_surf = value
                    .get("kind")
                    .and_then(|k| k.get("floor_surface"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(default_surf_str);
                let (tok, disp) = parse_surface(floor_surf);
                hull.iter()
                    .filter_map(|pt| {
                        let arr = pt.as_array()?;
                        if arr.len() < 2 {
                            return None;
                        }
                        Some(([arr[0].as_f64()? as f32, arr[1].as_f64()? as f32], tok, disp))
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let primary_pts = if !raw_samples.is_empty() { &raw_samples } else { &fallback_pts };
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            for (pt, _, _) in primary_pts.iter().chain(raw_branches.iter().flat_map(|b| b.iter())) {
                min_x = min_x.min(pt[0]);
                min_y = min_y.min(pt[1]);
                max_x = max_x.max(pt[0]);
                max_y = max_y.max(pt[1]);
            }
            if !min_x.is_finite() {
                min_x = 0.0;
                min_y = 0.0;
                max_x = 100.0;
                max_y = 100.0;
            }

            let thumb_samples = downsample_points(primary_pts, MAX_THUMBNAIL_SAMPLES);
            let thumb_samples_code = format_sample_slice(&thumb_samples);

            let mut branches_code = String::from("&[");
            for (bi, b) in raw_branches.iter().enumerate() {
                if bi > 0 {
                    branches_code.push_str(", ");
                }
                let ds = downsample_points(b, MAX_BRANCH_THUMBNAIL_SAMPLES);
                branches_code.push_str(&format_sample_slice(&ds));
            }
            branches_code.push(']');

            writeln!(
                generated,
                "    EmbeddedCircuit {{ module: {:?}, id: {:?}, name: {:?}, description: {:?}, tag: {:?}, category_label: {:?}, country_code: {:?}, default_laps: {}, total_length_m: {:.3}_f32, surface_summary: {:?}, scale: {:?}, is_inspired: {}, default_surface: {}, closed: {}, bounds_min: [{:.3}_f32, {:.3}_f32], bounds_max: [{:.3}_f32, {:.3}_f32], thumbnail_samples: {}, thumbnail_branches: {}, data: include_bytes!({:?}) }},",
                module,
                id,
                str_field(&value, "name"),
                str_field(&value, "description"),
                str_field(&value, "tag"),
                str_field(&value, "category_label"),
                str_field(&value, "country_code"),
                default_laps,
                total_length_m,
                surface_summary,
                scale_str,
                is_inspired,
                default_surf_tok,
                closed,
                min_x,
                min_y,
                max_x,
                max_y,
                thumb_samples_code,
                branches_code,
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
