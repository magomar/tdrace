//! # export-codex: write the TDRace Codex data files
//!
//! Serialises the game's cars, circuits, surfaces and modules for the Codex web portal.
//! See `specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md`.
//!
//! ```text
//! cargo run -p tdrace-app --bin export_codex -- [--out DIR] [--scope all|launch] [--check]
//! ```
//! `--out` defaults to `portals/shared/data/codex` and `--scope` to `all`.
//! `--check` writes nothing; it exits with code 1 when a file in `--out` differs from a fresh export.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tdrace_app::codex::{export, Scope, DEFAULT_OUT_DIR};

fn usage() -> ExitCode {
    eprintln!("usage: export_codex [--out DIR] [--scope all|launch] [--check]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let repo_root = repo_root.canonicalize().unwrap_or(repo_root);
    let mut out = repo_root.join(DEFAULT_OUT_DIR);
    let mut scope = Scope::All;
    let mut check = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => match args.next() {
                Some(dir) => out = PathBuf::from(dir),
                None => return usage(),
            },
            "--scope" => match args.next().as_deref().and_then(Scope::parse) {
                Some(s) => scope = s,
                None => return usage(),
            },
            "--check" => check = true,
            _ => return usage(),
        }
    }

    let files = match export(scope, &repo_root) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("export_codex: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if check {
        let stale: Vec<&str> = files
            .iter()
            .filter(|(name, json)| std::fs::read_to_string(out.join(name)).ok().as_deref() != Some(json.as_str()))
            .map(|(name, _)| *name)
            .collect();
        if stale.is_empty() {
            println!("export_codex: {} is fresh", out.display());
            return ExitCode::SUCCESS;
        }
        eprintln!("export_codex: stale in {}: {}", out.display(), stale.join(", "));
        eprintln!("run: cargo run -p tdrace-app --bin export_codex");
        return ExitCode::FAILURE;
    }

    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("export_codex: {}: {}", out.display(), e);
        return ExitCode::FAILURE;
    }
    for (name, json) in &files {
        let path = out.join(name);
        if let Err(e) = std::fs::write(&path, json) {
            eprintln!("export_codex: {}: {}", path.display(), e);
            return ExitCode::FAILURE;
        }
        println!("wrote {}", path.display());
    }
    ExitCode::SUCCESS
}
