use tdrace_app::module::kart::KartGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::menu::{resolve_track_for_menu, CarChoice, TrackChoice};
use tdrace_app::game::RaceSession;
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

#[test]
fn test_kart_module_tracks_integrity_and_validation() {
    let module = KartGameModule::new();
    let tracks = module.tracks();

    assert_eq!(tracks.len(), 10, "Kart module should have 10 tracks (8 famous + 2 presets)");

    let expected_ids = [
        "lonato",
        "sarno",
        "genk",
        "pfi",
        "zuera",
        "le_mans_kart",
        "portimao_kart",
        "franciacorta",
        "kart_arena",
        "drift_park",
    ];

    for id in &expected_ids {
        assert!(
            tracks.iter().any(|t| t.id == *id),
            "Expected track id '{}' in kart module tracks",
            id
        );
    }

    for track_def in &tracks {
        let track = (track_def.generator)();
        assert!(!track.name.is_empty(), "Track name cannot be empty for {}", track_def.id);
        assert!(track.spline.total_length() > 200.0, "Track length too short for {}", track_def.id);
        assert!(track.checkpoints.len() >= 8, "Too few checkpoints for {}", track_def.id);
        assert!(track.grid_positions.len() >= 8, "Too few grid positions for {}", track_def.id);
        assert_eq!(track.category, tdrace_core::track::TrackCategory::Main);
        assert!(track.modules.iter().any(|m| m == "kart"));

        let diags = validate_track(&track);
        let errors: Vec<_> = diags
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "Track '{}' ({}) had validation errors: {:?}",
            track.name,
            track_def.id,
            errors
        );
    }
}

#[test]
fn test_pfi_flyover_bridge_elevation() {
    let pfi = KartGameModule::track_pfi();
    assert_eq!(pfi.name, "PF International Kart Circuit (PFI)");

    // Check that there is a bridge section with elevation >= 4.0m
    let max_elevation = pfi
        .spline
        .samples
        .iter()
        .map(|s| s.elevation)
        .fold(0.0f32, f32::max);
    assert!(
        max_elevation >= 3.9,
        "PFI Flyover Bridge must reach ~4.0m elevation (got {:.2}m)",
        max_elevation
    );

    // Check underpass sample has 0.0 elevation
    let min_elevation = pfi
        .spline
        .samples
        .iter()
        .map(|s| s.elevation)
        .fold(10.0f32, f32::min);
    assert_eq!(min_elevation, 0.0, "PFI Underpass must be at ground level (0.0m)");
}

#[test]
fn test_famous_kart_tracks_in_track_manager_and_menu_resolution() {
    let tm = TrackManager::default();
    let kart_catalog = tm.module_catalog_tracks("kart");

    assert!(kart_catalog.len() >= 10);

    let famous_ids = [
        "lonato",
        "sarno",
        "genk",
        "pfi",
        "zuera",
        "le_mans_kart",
        "portimao_kart",
        "franciacorta",
    ];

    for id in &famous_ids {
        assert!(
            kart_catalog.iter().any(|c| c.track_id() == *id),
            "Track manager kart catalog missing '{}'",
            id
        );

        let choice = TrackChoice::Custom {
            id: id.to_string(),
            title: "".to_string(),
            description: "".to_string(),
            path: format!("kart/{}", id),
        };

        let loaded = tm.load_track(&choice);
        assert!(loaded.is_ok(), "Failed to load track '{}' from TrackManager: {:?}", id, loaded.err());

        let resolved = resolve_track_for_menu(&choice);
        assert!(resolved.is_some(), "Failed to resolve track '{}' for menu preview", id);
    }
}

#[test]
fn test_kart_race_session_simulation_on_lonato_and_sarno() {
    let test_tracks = ["lonato", "sarno", "genk", "pfi", "zuera"];

    for id in &test_tracks {
        let mut session = RaceSession::new();
        session.track_choice = TrackChoice::Custom {
            id: id.to_string(),
            title: "".to_string(),
            description: "".to_string(),
            path: format!("kart/{}", id),
        };
        session.car_choice = CarChoice::Kart;
        session.num_bots = 5;
        session.init_race();

        assert_eq!(session.cars.len(), 6, "1 player + 5 bots = 6 karts for {}", id);
        assert_eq!(session.trackers.len(), 6);
        assert!(!session.track.name.is_empty());
        assert_eq!(session.track_choice_id(), *id);

        // Step simulation 60 frames to ensure physical stability
        for _ in 0..60 {
            session.update();
        }

        for (i, car) in session.cars.iter().enumerate() {
            assert!(
                car.state.position.is_finite(),
                "Kart #{} position is non-finite on {}: {:?}",
                i,
                id,
                car.state.position
            );
            assert!(
                car.state.velocity.is_finite(),
                "Kart #{} velocity is non-finite on {}: {:?}",
                i,
                id,
                car.state.velocity
            );
        }
    }
}
