use std::fs;
use std::path::Path;
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_app::module::extreme_offroad::ExtremeOffRoadModule;
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::kart::KartGameModule;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_app::module::GameModule;
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::Track;

#[test]
fn test_all_96_tracks_grid_positions_count_and_validation() {
    let modules: Vec<(Box<dyn GameModule>, usize)> = vec![
        (Box::new(ClassicGameModule::new()), 10),
        (Box::new(RallyGameModule::new()), 12),
        (Box::new(ExtremeOffRoadModule::new()), 12),
        (Box::new(KartGameModule::new()), 14),
    ];

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let tracks_dir = root.join("tracks");

    let mut total_verified = 0;

    for (module, target_slots) in modules {
        let mod_id = module.id();

        for track_def in module.tracks() {
            let gen_track = (track_def.generator)();
            assert_eq!(
                gen_track.grid_positions.len(),
                target_slots,
                "Generated track '{}' in module '{}' had {} slots, expected {}",
                track_def.id,
                mod_id,
                gen_track.grid_positions.len(),
                target_slots
            );

            let gen_diags = validate_track(&gen_track);
            let gen_errors: Vec<_> = gen_diags
                .into_iter()
                .filter(|d| d.severity == ValidationSeverity::Error)
                .collect();
            assert!(
                gen_errors.is_empty(),
                "Generated track '{}' had validation errors: {:?}",
                track_def.id,
                gen_errors
            );

            let json_path = tracks_dir.join(mod_id).join(format!("{}.json", track_def.id));
            assert!(json_path.exists(), "Track JSON file must exist: {:?}", json_path);

            let content = fs::read_to_string(&json_path).expect("Read JSON");
            let parsed_track: Track =
                serde_json::from_str(&content).expect("Deserialize track from JSON");
            assert_eq!(
                parsed_track.grid_positions.len(),
                target_slots,
                "JSON track '{}' had {} slots, expected {}",
                track_def.id,
                parsed_track.grid_positions.len(),
                target_slots
            );

            let diags = validate_track(&parsed_track);
            let errors: Vec<_> = diags
                .into_iter()
                .filter(|d| d.severity == ValidationSeverity::Error)
                .collect();
            assert!(
                errors.is_empty(),
                "JSON track '{}' had validation errors: {:?}",
                track_def.id,
                errors
            );

            total_verified += 1;
        }
    }

    // GT World Challenge: all 18 tracks
    let gt_tracks: Vec<(&str, fn() -> Track)> = vec![
        ("monza", GtWorldChallengeModule::track_monza),
        ("spa", GtWorldChallengeModule::track_spa),
        ("silverstone", GtWorldChallengeModule::track_silverstone),
        ("monaco", GtWorldChallengeModule::track_monaco),
        ("suzuka", GtWorldChallengeModule::track_suzuka),
        ("interlagos", GtWorldChallengeModule::track_interlagos),
        ("montreal", GtWorldChallengeModule::track_montreal),
        ("red_bull_ring", GtWorldChallengeModule::track_red_bull_ring),
        ("catalunya", GtWorldChallengeModule::track_catalunya),
        ("zandvoort", GtWorldChallengeModule::track_zandvoort),
        ("bahrain", GtWorldChallengeModule::track_bahrain),
        ("marina_bay", GtWorldChallengeModule::track_marina_bay),
        ("cota", GtWorldChallengeModule::track_cota),
        ("madring", GtWorldChallengeModule::track_madring),
        ("nurburgring_gp", GtWorldChallengeModule::track_nurburgring_gp),
        ("bathurst", GtWorldChallengeModule::track_bathurst),
        ("portimao_gp", GtWorldChallengeModule::track_portimao_gp),
        ("le_mans_sarthe", GtWorldChallengeModule::track_le_mans_sarthe),
    ];

    for (id, generator) in gt_tracks {
        let gen_track = generator();
        assert_eq!(
            gen_track.grid_positions.len(),
            18,
            "GT track '{}' generator had {} slots, expected 18",
            id,
            gen_track.grid_positions.len()
        );

        let gen_diags = validate_track(&gen_track);
        let gen_errors: Vec<_> = gen_diags
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            gen_errors.is_empty(),
            "Generated GT track '{}' had validation errors: {:?}",
            id,
            gen_errors
        );

        let json_path = tracks_dir.join("gt").join(format!("{}.json", id));
        assert!(json_path.exists(), "GT JSON file must exist: {:?}", json_path);

        let content = fs::read_to_string(&json_path).expect("Read JSON");
        let parsed_track: Track =
            serde_json::from_str(&content).expect("Deserialize GT track from JSON");
        assert_eq!(
            parsed_track.grid_positions.len(),
            18,
            "GT JSON track '{}' had {} slots, expected 18",
            id,
            parsed_track.grid_positions.len()
        );

        let diags = validate_track(&parsed_track);
        let errors: Vec<_> = diags
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "GT JSON track '{}' had validation errors: {:?}",
            id,
            errors
        );

        total_verified += 1;
    }

    // NASCAR: verify all 17 tracks have 16 slots and validate cleanly
    let nascar_files = fs::read_dir(tracks_dir.join("nascar")).expect("Read nascar dir");
    for entry in nascar_files {
        let entry = entry.expect("Entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).expect("Read NASCAR json");
            let track: Track = serde_json::from_str(&content).expect("Parse NASCAR track");
            assert_eq!(
                track.grid_positions.len(),
                16,
                "NASCAR track '{:?}' must have 16 slots",
                path.file_name()
            );
            let diags = validate_track(&track);
            let errors: Vec<_> = diags
                .into_iter()
                .filter(|d| d.severity == ValidationSeverity::Error)
                .collect();
            assert!(
                errors.is_empty(),
                "NASCAR track '{:?}' had validation errors: {:?}",
                path.file_name(),
                errors
            );
            total_verified += 1;
        }
    }

    assert_eq!(total_verified, 96, "Must verify all 96 canonical tracks in the game");
}
