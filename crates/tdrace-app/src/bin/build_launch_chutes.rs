//! # build-launch-chutes: stamp a launch chute onto Autocross and Rallycross circuit files
//!
//! Spec 103: Autocross and Rallycross Launch Chutes and Templated Track Components.
//!
//! ```text
//! cargo run --bin build_launch_chutes -- tracks/autocross/matschenberg_ax.json [more.json ...] [--verbose]
//! cargo run --bin build_launch_chutes -- --all [--verbose]
//! ```
//! `--all` takes the 17 Autocross circuits (`tracks/autocross`), the 20 Rallycross circuits (`tracks/rally`), the
//! 3 Classic Autocross circuits and the 3 Classic Rallycross circuits. Each file gets the chute of its category
//! (Autocross: concrete pad, 5-3 grid, 8 cars; Rallycross: asphalt pad, 3-2-3-2 grid, 10 cars) at the best waypoint
//! before its finish line, and is written back in place. A file that already has a chute is left alone; one that
//! cannot take a chute is reported and left alone, and the exit code is 1. `--verbose` lists why each candidate
//! waypoint was refused.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::{ChuteSide, LaunchChuteSpec, PackedGridPattern, Track};
use tdrace_core::{CarCategory, SurfaceType};

const CLASSIC_AX: [&str; 3] = ["ax_clay_bowl", "ax_hillside_hammer", "ax_meadow_sprint"];
const CLASSIC_RX: [&str; 3] = ["rx_canyon_flyer", "rx_hilltop_leap", "rx_quarry_sprint"];

/// The circuit files `--all` covers, below `root` (the `tracks` folder).
fn all_circuits(root: &Path) -> Vec<PathBuf> {
    let json_in = |dir: &str| {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .map(|d| d.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect())
            .unwrap_or_default();
        files.sort();
        files
    };
    let mut files = json_in("autocross");
    files.extend(json_in("rally"));
    files.extend(CLASSIC_AX.into_iter().chain(CLASSIC_RX).map(|id| root.join(format!("classic/{id}.json"))));
    files
}

/// The template of a circuit's chute: its category sets the grid and the pad surface.
fn template(track: &Track) -> LaunchChuteSpec {
    let mut spec = LaunchChuteSpec::new(0, ChuteSide::Right);
    if track.car_category == CarCategory::Rally {
        spec.pattern = PackedGridPattern::RallycrossThreeTwoThreeTwo;
        spec.surface = SurfaceType::Asphalt;
    }
    spec
}

fn stamp(path: &Path, verbose: bool) -> Result<String, String> {
    let mut track = Track::load_from_file(path).map_err(|e| e.to_string())?;
    if track.launch_chute().is_some() {
        return Ok("already has a launch chute".to_string());
    }
    if !matches!(track.car_category, CarCategory::Autocross | CarCategory::Rally) {
        return Err(format!("{:?} circuit: launch chutes are for Autocross and Rallycross", track.car_category));
    }
    let template = template(&track);
    let spec = match track.place_launch_chute(&template) {
        Ok(spec) => spec,
        Err(e) => {
            if verbose {
                for k in (0..track.spline.waypoints.len()).filter(|&k| track.can_merge_launch_chute_at(k)) {
                    for side in [ChuteSide::Right, ChuteSide::Left] {
                        let attempt = LaunchChuteSpec { merge_waypoint: k, side, ..template };
                        if let Err(why) = track.clone().stamp_launch_chute(&attempt) {
                            eprintln!("    waypoint {k} {side:?}: {why}");
                        }
                    }
                }
            }
            return Err(e.to_string());
        }
    };
    let errors: Vec<String> = validate_track(&track)
        .into_iter()
        .filter(|d| d.severity == ValidationSeverity::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    if !errors.is_empty() {
        return Err(format!("validation errors after stamping: {errors:?}"));
    }
    track.save_to_file(path).map_err(|e| e.to_string())?;
    let chute = track.launch_chute().expect("stamped");
    Ok(format!(
        "chute at waypoint {} ({:?}), {} slots, {:.0} m pad",
        spec.merge_waypoint,
        spec.side,
        chute.grid_slots.len(),
        chute.pad_width
    ))
}

fn main() -> ExitCode {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut all = false;
    let mut verbose = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--all" => all = true,
            "--verbose" => verbose = true,
            _ => files.push(PathBuf::from(arg)),
        }
    }
    if all {
        files.extend(all_circuits(Path::new("tracks")));
    }
    if files.is_empty() {
        eprintln!("usage: build_launch_chutes (--all | <file.json>...) [--verbose]");
        return ExitCode::from(2);
    }

    let mut failed = 0;
    for path in &files {
        match stamp(path, verbose) {
            Ok(note) => println!("{}: {}", path.display(), note),
            Err(why) => {
                failed += 1;
                eprintln!("{}: FAILED: {}", path.display(), why);
            }
        }
    }
    println!("{} circuits, {} failed", files.len(), failed);
    if failed == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
