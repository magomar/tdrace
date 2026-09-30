//! # track-bake: fill the derived data of circuit JSON files
//!
//! Reads a "source" circuit (as written by `scripts/osm_importer.py --json`), fills the empty derived parts
//! (spline samples, walls, checkpoints, starting grid, default runoff), validates it, and writes it back in place.
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md` §2.7.
//!
//! ```text
//! cargo run --bin track_bake -- tracks/gt/monza.json [more.json ...]
//!     [--rebuild] [--barrier-offset 4.0] [--barrier-type Steel] [--checkpoints 20] [--sectors 3]
//! ```
//! Options left out keep the file's current setup: wall distance and type (else 4.0 m Steel), checkpoint and
//! sector count (else 20 and 3), and grid size (else the module default).
//! A file with validation errors is not written, and the exit code is 1.

use std::process::ExitCode;

use tdrace_core::track::bake::{bake, BakeOptions};
use tdrace_core::track::geometry::BarrierType;
use tdrace_core::track::{validate_track, Track, ValidationSeverity};

fn usage() -> ExitCode {
    eprintln!(
        "usage: track_bake <file.json>... [--rebuild] [--merge-walls] [--barrier-offset M] \
         [--barrier-type Concrete|Steel|TireWall|CurbWall|Virtual] [--checkpoints N] [--sectors N]"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut opts = BakeOptions::default();
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{} needs a value", name));
        let parsed: Result<(), String> = match arg.as_str() {
            "--rebuild" => {
                opts.rebuild = true;
                Ok(())
            }
            "--merge-walls" => {
                opts.merge_walls = true;
                Ok(())
            }
            "--barrier-offset" => value(&arg).and_then(|v| {
                opts.barrier_offset = Some(v.parse().map_err(|e| format!("--barrier-offset {}: {}", v, e))?);
                Ok(())
            }),
            "--barrier-type" => value(&arg).and_then(|v| {
                opts.barrier_type = Some(
                    serde_json::from_str::<BarrierType>(&format!("\"{}\"", v))
                        .map_err(|_| format!("unknown --barrier-type {}", v))?,
                );
                Ok(())
            }),
            "--checkpoints" => value(&arg).and_then(|v| {
                opts.checkpoint_count = Some(v.parse().map_err(|e| format!("--checkpoints {}: {}", v, e))?);
                Ok(())
            }),
            "--sectors" => value(&arg).and_then(|v| {
                opts.sector_count = Some(v.parse().map_err(|e| format!("--sectors {}: {}", v, e))?);
                Ok(())
            }),
            "-h" | "--help" => return usage(),
            flag if flag.starts_with("--") => Err(format!("unknown option {}", flag)),
            file => {
                files.push(file.to_string());
                Ok(())
            }
        };
        if let Err(e) = parsed {
            eprintln!("track_bake: {}", e);
            return usage();
        }
    }
    if files.is_empty() {
        return usage();
    }

    let mut failed = false;
    for file in &files {
        let mut track = match Track::load_from_file(file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}: {}", file, e);
                failed = true;
                continue;
            }
        };
        let report = match bake(&mut track, &opts) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}: {}", file, e);
                failed = true;
                continue;
            }
        };
        let diagnostics = validate_track(&track);
        for d in &diagnostics {
            if d.severity != ValidationSeverity::Info {
                eprintln!("{}: {:?} {}: {}", file, d.severity, d.code, d.message);
            }
        }
        if diagnostics.iter().any(|d| d.severity == ValidationSeverity::Error) {
            eprintln!("{}: not written (validation errors)", file);
            failed = true;
            continue;
        }
        if let Err(e) = track.save_to_file(file) {
            eprintln!("{}: {}", file, e);
            failed = true;
            continue;
        }
        println!(
            "{}: baked (spline: {}, walls: {}, checkpoints: {}, grid: {}), {:.0} m",
            file,
            report.spline,
            report.walls,
            report.checkpoints,
            report.grid,
            track.spline.total_length()
        );
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
