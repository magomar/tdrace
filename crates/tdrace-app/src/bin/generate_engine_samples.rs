use std::fs;
use std::path::{Path, PathBuf};
use tdrace_app::audio::manager::EngineSoundType;
use tdrace_app::audio::samples::ArchetypeSampleBank;

fn main() {
    let custom_target = std::env::args().nth(1).map(PathBuf::from);

    let target_dir = if let Some(target) = custom_target {
        target
    } else {
        let candidates = [
            Path::new("assets/audio/engines"),
            Path::new("../assets/audio/engines"),
            Path::new("../../assets/audio/engines"),
        ];
        candidates
            .iter()
            .find(|p| p.parent().map(|par| par.exists()).unwrap_or(false))
            .copied()
            .unwrap_or_else(|| Path::new("assets/audio/engines"))
            .to_path_buf()
    };

    fs::create_dir_all(&target_dir).expect("Failed to create engine audio assets directory");
    println!("Generating canonical engine audio samples into: {:?}", target_dir);

    let archetypes = [
        EngineSoundType::SportGT,
        EngineSoundType::NascarV8,
        EngineSoundType::Kart125cc,
        EngineSoundType::RallyTurbo,
        EngineSoundType::SandRailBoxer,
    ];

    for archetype in archetypes {
        let bank = ArchetypeSampleBank::generate(archetype, 44100);
        bank.save_to_dir(&target_dir)
            .unwrap_or_else(|e| panic!("Failed to save sample bank for {:?}: {}", archetype, e));
        println!("  ✓ Saved sample bank for {:?}", archetype);
    }

    println!("All canonical engine audio samples successfully generated!");
}
