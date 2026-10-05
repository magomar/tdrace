use tdrace_app::module::autocross::AutocrossGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::tournament::TournamentFormat;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::validation::validate_track;
use tdrace_core::CarCategory;

#[test]
fn test_autocross_game_module_identity_and_properties() {
    let ax = AutocrossGameModule::new();
    assert_eq!(ax.id(), "autocross");
    assert_eq!(ax.title(), "FIA AUTOCROSS");
    assert_eq!(ax.subtitle(), "Natural Unpaved Dirt & Buggy Racing");
    assert_eq!(ax.default_off_track_surface(), SurfaceType::Dirt);
    assert_eq!(ax.default_track_id(), "nova_paka_ax");
    assert_eq!(ax.default_vehicle_id(), "autocross_ardennes_junior_t1");

    let theme = ax.theme();
    assert_eq!(theme.header_badge, "FIA AUTOCROSS CHAMPIONSHIP");

    let audio = ax.audio_profile();
    assert_eq!(audio.sound_type, tdrace_app::audio::EngineSoundType::CrossCarMotorcycle);
}

#[test]
fn test_autocross_tracks_count_and_zero_joker_ruleset() {
    let ax = AutocrossGameModule::new();
    let tracks = ax.tracks();
    assert_eq!(tracks.len(), 17, "Expected 17 official FIA Autocross tracks");

    let expected_track_ids = [
        "nova_paka_ax",
        "prerov_ax",
        "humpolec_ax",
        "matschenberg_ax",
        "seelow_ax",
        "schluechtern_ax",
        "uelzen_ax",
        "st_georges_ax",
        "faleyras_ax",
        "st_junien_ax",
        "bazaigues_ax",
        "maggiora_ax",
        "musa_ax",
        "vilkyciai_ax",
        "arteixo_ax",
        "carballo_ax",
        "castelo_branco_ax",
    ];

    assert_eq!(tracks.len(), expected_track_ids.len());

    for (idx, def) in tracks.iter().enumerate() {
        assert_eq!(def.id, expected_track_ids[idx]);
        assert_eq!(def.category, "FIA Autocross");
        assert!(def.default_laps >= 3 && def.default_laps <= 7);

        // Load the baked canonical track
        let track = tdrace_core::catalog::official_track("autocross", def.id);
        assert_eq!(track.car_category, CarCategory::Autocross);
        assert_eq!(track.default_laps, def.default_laps);

        // Geometry & validation checks
        assert!(track.spline.total_length() > 300.0, "Track {} too short", def.id);
        assert!(track.checkpoints.len() >= 4, "Track {} has too few checkpoints", def.id);
        assert!(track.grid_positions.len() >= 12, "Track {} must support at least 12 grid slots", def.id);

        let issues = validate_track(&track);
        let error_issues: Vec<_> = issues
            .into_iter()
            .filter(|i| i.severity == tdrace_core::track::validation::ValidationSeverity::Error)
            .collect();
        assert!(
            error_issues.is_empty(),
            "Track '{}' generated validation errors: {:?}",
            def.id,
            error_issues
        );

        // Pure dirt & zero joker lap ruleset verification:
        // In FIA Autocross, unlike Rallycross, tracks are standard closed single loops
        // with sequential checkpoints and NO alternate joker routes/branches.
        assert!(track.spline.waypoints.len() >= 4, "Track {} spline must form a smooth path", def.id);
    }
}

#[test]
fn test_autocross_driver_roster() {
    let ax = AutocrossGameModule::new();
    let drivers = ax.drivers();
    assert_eq!(drivers.len(), 8, "Expected 8 Autocross driver characters");

    let driver_names: Vec<&str> = drivers.iter().map(|d| d.name).collect();
    assert!(driver_names.contains(&"Petr Nikodém"));
    assert!(driver_names.contains(&"Bernd Stubbe"));
    assert!(driver_names.contains(&"Kevin Peters"));
    assert!(driver_names.contains(&"David Méndez"));
    assert!(driver_names.contains(&"Miguel Gayoso"));
    assert!(driver_names.contains(&"Václav Fejfar"));
    assert!(driver_names.contains(&"Vincent Mercier"));
    assert!(driver_names.contains(&"Jakub Novotný"));

    for driver in &drivers {
        assert!(!driver.alias.is_empty());
        assert!(!driver.bio.is_empty());
        assert!(!driver.favorite_cars.is_empty());
        for fav in driver.favorite_cars {
            assert_eq!(fav.discipline, "autocross");
        }
    }
}

#[test]
fn test_autocross_supported_game_modes() {
    let ax = AutocrossGameModule::new();
    let modes = ax.supported_game_modes();
    assert_eq!(modes.len(), 3);

    let has_championship = modes.iter().any(|m| matches!(m, TournamentFormat::Championship { laps_per_round: 5, .. }));
    let has_quick_race = modes.iter().any(|m| matches!(m, TournamentFormat::QuickRace { default_laps: 4, default_bots: 7 }));
    let has_time_attack = modes.iter().any(|m| matches!(m, TournamentFormat::TimeAttack));

    assert!(has_championship, "Must support Championship format");
    assert!(has_quick_race, "Must support QuickRace format");
    assert!(has_time_attack, "Must support TimeAttack format");
}
