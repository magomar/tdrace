use tdrace_app::game::RaceSession;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::menu::{resolve_track_for_menu, CarChoice, TrackChoice};
use tdrace_core::physics::surface::SurfaceType;

use tdrace_core::track::geometry::JumpRampCarExt;
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

#[test]
fn test_rally_module_tracks_integrity_and_validation() {
    let module = RallyGameModule::new();
    let tracks = module.tracks();

    assert_eq!(tracks.len(), 20, "Rally module should have 20 authentic World RX tracks");

    let expected_ids = [
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "estering_rx",
        "montalegre_rx",
        "nyirad_rx",
        "kouvola_rx",
        "catalunya_rx",
        "mettet_rx",
        "lavare_rx",
        "riga_rx",
        "killarney_rx",
        "lessay_rx",
        "essay_rx",
        "dreux_rx",
        "croft_rx",
        "spa_rx",
        "silverstone_rx",
        "erx_motor_park",
    ];

    for id in &expected_ids {
        assert!(
            tracks.iter().any(|t| t.id == *id),
            "Expected track id '{}' in rally module tracks",
            id
        );
    }

    for track_def in &tracks {
        let track = tdrace_core::catalog::official_track("rally", track_def.id);
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), "Checking track: {}", track_def.id);
        let _ = std::io::stderr().flush();

        if track.name.is_empty() {
            panic!("Track name is empty for {}", track_def.id);
        }
        if track.spline.total_length() <= 200.0 {
            panic!("Track length too short ({}) for {}", track.spline.total_length(), track_def.id);
        }
        if track.checkpoints.len() < 8 {
            panic!("Too few checkpoints ({}) for {}", track.checkpoints.len(), track_def.id);
        }
        if track.grid_positions.len() < 8 {
            panic!("Too few grid positions ({}) for {}", track.grid_positions.len(), track_def.id);
        }
        if track.category != tdrace_core::track::TrackCategory::Main {
            panic!("Category mismatch for {}", track_def.id);
        }
        if !track.modules.iter().any(|m| m == "rally") {
            panic!("Module 'rally' missing in track.modules ({:?}) for {}", track.modules, track_def.id);
        }

        let diags = validate_track(&track);
        for d in &diags {
            let _ = writeln!(std::io::stderr(), "  [{:?}] {}: {}", d.severity, d.code, d.message);
        }
        let _ = std::io::stderr().flush();

        let errors: Vec<_> = diags
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        if !errors.is_empty() {
            panic!("Validation errors on {}: {:?}", track_def.id, errors);
        }
    }
}

#[test]
fn test_dirt_figure_eight_horizontal_flat_dirt_arena() {
    let fig8 = tdrace_core::catalog::official_track("extreme_offroad", "dirt_figure_eight");
    assert_eq!(fig8.name, "Dirt Figure Eight");

    // 1. Verify horizontal orientation (width along X is substantially larger than height along Y)
    let min_x = fig8.spline.samples.iter().map(|s| s.point.x).fold(f32::INFINITY, f32::min);
    let max_x = fig8.spline.samples.iter().map(|s| s.point.x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = fig8.spline.samples.iter().map(|s| s.point.y).fold(f32::INFINITY, f32::min);
    let max_y = fig8.spline.samples.iter().map(|s| s.point.y).fold(f32::NEG_INFINITY, f32::max);
    let width_x = max_x - min_x;
    let height_y = max_y - min_y;
    assert!(
        width_x > height_y * 2.0,
        "Figure-8 circuit must be placed horizontally (width {:.1}m vs height {:.1}m)",
        width_x,
        height_y
    );

    // 2. Verify flat crossover at the same height (0.0m elevation throughout, no bridge overpass)
    let max_elevation = fig8
        .spline
        .samples
        .iter()
        .map(|s| s.elevation)
        .fold(0.0f32, f32::max);
    assert_eq!(
        max_elevation, 0.0,
        "Figure-8 crossover must be at ground level with 0.0m elevation throughout"
    );

    // 3. Verify whole track is 100% dirt surface
    let breakdown = fig8.surface_breakdown();
    assert_eq!(breakdown.len(), 1, "Figure-8 must be single dirt surface type");
    assert_eq!(breakdown[0].0, SurfaceType::Dirt, "Track surface must be 100% Dirt");

    // 4. Ensure there is at least one JumpRamp
    assert!(!fig8.geometry.jump_ramps.is_empty(), "Figure-8 should have jump ramps");
}

#[test]
fn test_dirt_figure_eight_jump_ramps_proportional_trajectory() {
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::CarConfig;

    let fig8 = tdrace_core::catalog::official_track("classic", "dirt_figure_eight");
    assert_eq!(fig8.geometry.jump_ramps.len(), 2);

    for ramp in &fig8.geometry.jump_ramps {
        assert_eq!(ramp.height, 1.8);
        assert!((ramp.ramp_angle_deg - 8.194263).abs() < 1e-4);
        assert_eq!(ramp.launch_speed, 4.0);

        // Simulate a rally car hitting the ramp at 25.0 m/s (~90 km/h)
        let mut car = Car::new(CarConfig::rally_car()).with_pose(ramp.shape.center(), 0.0);
        car.state.velocity = ramp.direction * 25.0;

        let triggered = car.try_trigger_jump_ramp(ramp);
        assert!(triggered, "Car at speed should trigger jump ramp");
        assert!(car.state.is_airborne);

        // Step physics forward while airborne
        let ctrl = CarControls::accelerate();
        let mut apex_elevation = 0.0f32;
        let mut steps_to_landing = 0;

        for _ in 0..120 {
            car.step(&ctrl, SurfaceType::Dirt, 1.0 / 60.0);
            if car.state.elevation > apex_elevation {
                apex_elevation = car.state.elevation;
            }
            if car.state.just_landed {
                break;
            }
            steps_to_landing += 1;
        }

        // Apex elevation should be ~1.4m to 2.5m (proportional to 1.8m height and 8.2° pitch, not >10m-30m!)
        assert!(
            apex_elevation >= 1.4 && apex_elevation <= 2.5,
            "Apex elevation should be proportional to ramp (1.4m - 2.5m), got {:.2}m",
            apex_elevation
        );
        // Air time should be ~0.8s to 1.2s (~45-75 frames), not 4+ seconds
        assert!(
            steps_to_landing >= 40 && steps_to_landing <= 80,
            "Jump should last ~0.8-1.2s (40-80 frames), took {} steps ({:.2}s)",
            steps_to_landing,
            steps_to_landing as f32 / 60.0
        );
        assert!(car.state.just_landed, "Car should have landed");
        assert_eq!(car.state.elevation, 0.0);
        assert_eq!(car.state.vertical_velocity, 0.0);
        assert!(!car.state.is_airborne);
    }
}

#[test]
fn test_world_rx_tracks_jump_ramps_and_mixed_surfaces() {
    let holjes = tdrace_core::catalog::official_track("rally", "holjes_rx");
    assert_eq!(holjes.name, "Höljes Motorstadion");
    assert!(!holjes.geometry.jump_ramps.is_empty(), "Höljes must have the iconic jump ramp");
    let holjes_breakdown = holjes.surface_breakdown();
    assert!(holjes_breakdown.len() >= 2, "Höljes must be mixed surface (asphalt & dirt)");

    let lydden = tdrace_core::catalog::official_track("rally", "lydden_hill");
    assert_eq!(lydden.name, "Lydden Hill Race Circuit");
    let lydden_breakdown = lydden.surface_breakdown();
    assert!(lydden_breakdown.len() >= 2, "Lydden Hill must be mixed surface");

    let hell = tdrace_core::catalog::official_track("rally", "hell_rx");
    assert_eq!(hell.name, "Lånkebanen / Hell RX");
    assert!(!hell.geometry.jump_ramps.is_empty(), "Hell RX must have jump ramp");
    let hell_breakdown = hell.surface_breakdown();
    assert!(hell_breakdown.len() >= 2, "Hell RX must be mixed surface");

    let loheac = tdrace_core::catalog::official_track("rally", "loheac_rx");
    assert_eq!(loheac.name, "Circuit de Lohéac");
    assert!(!loheac.geometry.jump_ramps.is_empty(), "Lohéac must have jump ramp");
    let loheac_breakdown = loheac.surface_breakdown();
    assert!(loheac_breakdown.len() >= 2, "Lohéac must be mixed surface");

    let estering = tdrace_core::catalog::official_track("rally", "estering_rx");
    assert_eq!(estering.name, "Estering Buxtehude");
    let estering_breakdown = estering.surface_breakdown();
    assert!(estering_breakdown.len() >= 2, "Estering must be mixed surface");

    let montalegre = tdrace_core::catalog::official_track("rally", "montalegre_rx");
    assert_eq!(montalegre.name, "Pista de Montalegre");
    assert!(!montalegre.geometry.jump_ramps.is_empty(), "Montalegre must have jump ramp");
    let montalegre_breakdown = montalegre.surface_breakdown();
    assert!(montalegre_breakdown.len() >= 2, "Montalegre must be mixed surface");

    let nyirad = tdrace_core::catalog::official_track("rally", "nyirad_rx");
    assert_eq!(nyirad.name, "Nyirád Racing Center");
    let nyirad_breakdown = nyirad.surface_breakdown();
    assert!(nyirad_breakdown.len() >= 2, "Nyirád must be mixed surface");

    let kouvola = tdrace_core::catalog::official_track("rally", "kouvola_rx");
    assert_eq!(kouvola.name, "Tykkimäen Moottorirata");
    assert!(!kouvola.geometry.jump_ramps.is_empty(), "Kouvola must have jump ramp");
    let kouvola_breakdown = kouvola.surface_breakdown();
    assert!(kouvola_breakdown.len() >= 2, "Kouvola must be mixed surface");

    let catalunya = tdrace_core::catalog::official_track("rally", "catalunya_rx");
    assert_eq!(catalunya.name, "Barcelona-Catalunya RX");
    assert!(!catalunya.geometry.jump_ramps.is_empty(), "Catalunya RX must have jump ramp");
    let catalunya_breakdown = catalunya.surface_breakdown();
    assert!(catalunya_breakdown.len() >= 2, "Catalunya RX must be mixed surface");

    let mettet = tdrace_core::catalog::official_track("rally", "mettet_rx");
    assert_eq!(mettet.name, "Circuit Jules Tacheny Mettet");
    assert!(!mettet.geometry.jump_ramps.is_empty(), "Mettet must have jump ramp");
    let mettet_breakdown = mettet.surface_breakdown();
    assert!(mettet_breakdown.len() >= 2, "Mettet must be mixed surface");

    // Lavaré replaced Silverstone RX (whose loose section is not in OSM); no jump is known there.
    let lavare = tdrace_core::catalog::official_track("rally", "lavare_rx");
    assert_eq!(lavare.name, "Circuit de Lavaré");
    let lavare_breakdown = lavare.surface_breakdown();
    assert!(lavare_breakdown.len() >= 2, "Lavaré must be mixed surface");

    let riga = tdrace_core::catalog::official_track("rally", "riga_rx");
    assert_eq!(riga.name, "Biķernieku Trase / Riga RX");
    assert!(!riga.geometry.jump_ramps.is_empty(), "Riga RX must have jump ramp");
    let riga_breakdown = riga.surface_breakdown();
    assert!(riga_breakdown.len() >= 2, "Riga RX must be mixed surface");

    let killarney = tdrace_core::catalog::official_track("rally", "killarney_rx");
    assert_eq!(killarney.name, "Killarney International Raceway RX");
    assert!(!killarney.geometry.jump_ramps.is_empty(), "Killarney must have jump ramp");
    let killarney_breakdown = killarney.surface_breakdown();
    assert!(killarney_breakdown.len() >= 2, "Killarney must be mixed surface");

    // Lessay replaced Yas Marina RX (whose loose section is not in OSM); no jump is known there.
    let lessay = tdrace_core::catalog::official_track("rally", "lessay_rx");
    assert_eq!(lessay.name, "Circuit de Lessay");
    let lessay_breakdown = lessay.surface_breakdown();
    assert!(lessay_breakdown.len() >= 2, "Lessay must be mixed surface");

    let essay = tdrace_core::catalog::official_track("rally", "essay_rx");
    assert_eq!(essay.name, "Circuit des Ducs / Essay RX");
    assert!(!essay.geometry.jump_ramps.is_empty(), "Essay RX must have jump ramp");
    let essay_breakdown = essay.surface_breakdown();
    assert!(essay_breakdown.len() >= 2, "Essay RX must be mixed surface");
    assert!(!essay.geometry.trees.is_empty(), "Essay RX must have decorative trees");

    // Verify 1:1 scale lengths based on OpenStreetMap & FIA homologation standards
    assert!(
        holjes.spline.total_length() >= 1150.0 && holjes.spline.total_length() <= 1250.0,
        "Höljes 1:1 FIA length expected ~1210m, got {:.1}m",
        holjes.spline.total_length()
    );
    assert!(
        lydden.spline.total_length() >= 1120.0 && lydden.spline.total_length() <= 1220.0,
        "Lydden Hill 1:1 FIA length expected ~1170m, got {:.1}m",
        lydden.spline.total_length()
    );
    assert!(
        hell.spline.total_length() >= 980.0 && hell.spline.total_length() <= 1060.0,
        "Hell RX 1:1 FIA length expected ~1019m, got {:.1}m",
        hell.spline.total_length()
    );
    assert!(
        loheac.spline.total_length() >= 1040.0 && loheac.spline.total_length() <= 1130.0,
        "Lohéac 1:1 FIA length expected ~1088m, got {:.1}m",
        loheac.spline.total_length()
    );
    assert!(
        estering.spline.total_length() >= 910.0 && estering.spline.total_length() <= 990.0,
        "Estering 1:1 FIA length expected ~952m, got {:.1}m",
        estering.spline.total_length()
    );
    assert!(
        montalegre.spline.total_length() >= 1000.0 && montalegre.spline.total_length() <= 1100.0,
        "Montalegre 1:1 FIA length expected ~1050m, got {:.1}m",
        montalegre.spline.total_length()
    );
    // Nyirád: the 1216 m OSM loop at 1:1 with its sharp junction hairpins rounded to 13 m (~1081 m).
    assert!(
        nyirad.spline.total_length() >= 1040.0 && nyirad.spline.total_length() <= 1120.0,
        "Nyirád 1:1 OSM lap with rounded hairpins expected ~1081m, got {:.1}m",
        nyirad.spline.total_length()
    );
    assert!(
        kouvola.spline.total_length() >= 960.0 && kouvola.spline.total_length() <= 1080.0,
        "Kouvola 1:1 FIA length expected ~1060m, got {:.1}m",
        kouvola.spline.total_length()
    );
    assert!(
        catalunya.spline.total_length() >= 1075.0 && catalunya.spline.total_length() <= 1175.0,
        "Catalunya RX 1:1 FIA length expected ~1125m, got {:.1}m",
        catalunya.spline.total_length()
    );
    assert!(
        mettet.spline.total_length() >= 1100.0 && mettet.spline.total_length() <= 1200.0,
        "Mettet 1:1 FIA length expected ~1149m, got {:.1}m",
        mettet.spline.total_length()
    );
    assert!(
        lavare.spline.total_length() >= 1020.0 && lavare.spline.total_length() <= 1100.0,
        "Lavaré 1:1 length expected ~1070m, got {:.1}m",
        lavare.spline.total_length()
    );
    // Riga: the mapped 1071 m OSM loop at 1:1 (the official 1294 m is 17% longer, beyond the 10% rule).
    assert!(
        riga.spline.total_length() >= 1010.0 && riga.spline.total_length() <= 1090.0,
        "Riga RX 1:1 OSM length expected ~1051m, got {:.1}m",
        riga.spline.total_length()
    );
    assert!(
        killarney.spline.total_length() >= 1020.0 && killarney.spline.total_length() <= 1120.0,
        "Killarney 1:1 FIA length expected ~1067m, got {:.1}m",
        killarney.spline.total_length()
    );
    assert!(
        lessay.spline.total_length() >= 850.0 && lessay.spline.total_length() <= 920.0,
        "Lessay 1:1 length expected ~886m, got {:.1}m",
        lessay.spline.total_length()
    );
    assert!(
        essay.spline.total_length() >= 880.0 && essay.spline.total_length() <= 990.0,
        "Essay RX 1:1 FIA length expected ~936m, got {:.1}m",
        essay.spline.total_length()
    );
}

#[test]
fn test_world_rx_jump_ramps_dirt_surface_and_containment_landing() {
    use tdrace_core::collision::wall::resolve_all_wall_collisions;
    use tdrace_core::physics::car::{Car, CarControls};
    use tdrace_core::physics::config::CarConfig;
    use tdrace_core::physics::surface::SurfaceType;
    use tdrace_core::track::geometry::SurfaceShape;

    let tracks = [
        ("holjes_rx", tdrace_core::catalog::official_track("rally", "holjes_rx")),
        ("hell_rx", tdrace_core::catalog::official_track("rally", "hell_rx")),
        ("loheac_rx", tdrace_core::catalog::official_track("rally", "loheac_rx")),
        ("montalegre_rx", tdrace_core::catalog::official_track("rally", "montalegre_rx")),
        ("kouvola_rx", tdrace_core::catalog::official_track("rally", "kouvola_rx")),
        ("catalunya_rx", tdrace_core::catalog::official_track("rally", "catalunya_rx")),
        ("mettet_rx", tdrace_core::catalog::official_track("rally", "mettet_rx")),
        ("riga_rx", tdrace_core::catalog::official_track("rally", "riga_rx")),
        ("killarney_rx", tdrace_core::catalog::official_track("rally", "killarney_rx")),
        ("essay_rx", tdrace_core::catalog::official_track("rally", "essay_rx")),
    ];

    for (slug, track) in &tracks {
        assert_eq!(track.geometry.jump_ramps.len(), 1, "Track {} should have 1 jump ramp", slug);
        let ramp = &track.geometry.jump_ramps[0];

        // 1. Verify surface is Dirt (user request: depicted as dirt ramp, not asphalt)
        assert_eq!(
            ramp.surface,
            SurfaceType::Dirt,
            "Track {} jump ramp must use SurfaceType::Dirt",
            slug
        );

        // 2. Verify ramp dimensions are compact to avoid launching cars out of the circuit
        if let SurfaceShape::OrientedBox { half_extents, .. } = ramp.shape {
            assert!(
                half_extents.x <= 4.0,
                "Track {} ramp half length must be <= 4.0m (compact kicker/crest), got {:.1}m",
                slug,
                half_extents.x
            );
            assert!(
                half_extents.y <= 6.0,
                "Track {} ramp half width must fit track corridor (<= 6.0m), got {:.1}m",
                slug,
                half_extents.y
            );
        } else {
            panic!("Expected OrientedBox shape for jump ramp on {}", slug);
        }

        // 3. Verify launch parameters are realistic and safe
        assert!(ramp.height <= 1.5, "Track {} ramp height must be <= 1.5m, got {:.2}m", slug, ramp.height);
        assert!(
            ramp.ramp_angle_deg <= 7.0,
            "Track {} ramp angle must be <= 7.0 deg, got {:.2} deg",
            slug,
            ramp.ramp_angle_deg
        );
        assert!(
            ramp.launch_speed <= 3.0,
            "Track {} ramp launch speed must be <= 3.0 m/s, got {:.2} m/s",
            slug,
            ramp.launch_speed
        );

        // 4. Test physical jump containment at high rally speeds: 30 m/s (108 km/h) and 42 m/s (151 km/h)
        for &approach_speed in &[30.0f32, 42.0f32] {
            let mut car = Car::new(CarConfig::rally_car()).with_pose(ramp.shape.center(), 0.0);
            car.state.velocity = ramp.direction * approach_speed;

            let triggered = car.try_trigger_jump_ramp(ramp);
            assert!(triggered, "Car at {:.1} m/s should trigger jump ramp on {}", approach_speed, slug);
            assert!(car.state.is_airborne);

            let ctrl = CarControls::accelerate();
            let mut apex_elevation = 0.0f32;
            let mut steps_to_landing = 0;

            for _ in 0..120 {
                car.step(&ctrl, SurfaceType::Dirt, 1.0 / 60.0);
                if car.state.elevation > apex_elevation {
                    apex_elevation = car.state.elevation;
                }
                if car.state.just_landed {
                    break;
                }
                steps_to_landing += 1;
            }

            assert!(car.state.just_landed, "Car at {:.1} m/s should land within 120 frames on {}", approach_speed, slug);
            assert!(!car.state.is_airborne);
            assert_eq!(car.state.elevation, 0.0);

            // Apex elevation must be modest and realistic (~0.4m - 1.8m)
            assert!(
                apex_elevation >= 0.4 && apex_elevation <= 1.8,
                "Track {} apex elevation at {:.1} m/s must be between 0.4m and 1.8m, got {:.2}m",
                slug,
                approach_speed,
                apex_elevation
            );

            // Air time must be snappy (~0.3s to 1.0s)
            assert!(
                steps_to_landing >= 18 && steps_to_landing <= 60,
                "Track {} jump at {:.1} m/s took {} steps ({:.2}s)",
                slug,
                approach_speed,
                steps_to_landing,
                steps_to_landing as f32 / 60.0
            );

            // 5. Containment check: At landing point, car must NOT be inside/over outer or inner barriers
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);
            assert!(
                hit_inner.is_empty() && hit_outer.is_empty(),
                "Track {} car jumped outside circuit or into walls at {:.1} m/s! Landing pos: ({:.1}, {:.1})",
                slug,
                approach_speed,
                car.state.position.x,
                car.state.position.y
            );

            // Distance to track centerline must be well within the track boundaries
            let proj = track.spline.project_point(car.state.position);
            let sample = track.spline.sample_at_distance(proj.progress_distance);
            let lateral_offset = (car.state.position - sample.point).length();
            let allowed_lateral = sample.width * 0.5 + 3.5;
            assert!(
                lateral_offset <= allowed_lateral,
                "Track {} car landed too far laterally from track ribbon ({:.1}m > {:.1}m) at pos ({:.1}, {:.1})",
                slug,
                lateral_offset,
                allowed_lateral,
                car.state.position.x,
                car.state.position.y
            );
        }
    }
}

#[test]
fn test_famous_rally_tracks_in_track_manager_and_menu_resolution() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_rally_tm_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let tm = TrackManager::new(&temp_dir);
    let rally_catalog = tm.module_catalog_tracks("rally");
    assert_eq!(rally_catalog.len(), 20);

    let rally_ids = [
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "estering_rx",
        "montalegre_rx",
        "nyirad_rx",
        "kouvola_rx",
        "catalunya_rx",
        "mettet_rx",
        "lavare_rx",
        "riga_rx",
        "killarney_rx",
        "lessay_rx",
        "essay_rx",
        "dreux_rx",
        "croft_rx",
        "spa_rx",
        "silverstone_rx",
        "erx_motor_park",
    ];

    for id in &rally_ids {
        assert!(
            rally_catalog.iter().any(|c| c.track_id() == *id),
            "Track manager rally catalog missing '{}'",
            id
        );

        let choice = TrackChoice::Custom {
            id: id.to_string(),
            title: "".to_string(),
            description: "".to_string(),
            path: format!("rally/{}", id),
        };

        let loaded = tm.load_track(&choice);
        assert!(loaded.is_ok(), "Failed to load track '{}' from TrackManager: {:?}", id, loaded.err());

        let resolved = resolve_track_for_menu(&choice);
        assert!(resolved.is_some(), "Failed to resolve track '{}' for menu preview", id);
    }
}

#[test]
fn test_rally_race_session_simulation_on_new_tracks() {
    let test_tracks = [
        "essay_rx",
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "estering_rx",
        "montalegre_rx",
        "nyirad_rx",
        "kouvola_rx",
        "catalunya_rx",
        "mettet_rx",
        "lavare_rx",
        "riga_rx",
        "killarney_rx",
        "lessay_rx",
    ];

    for id in &test_tracks {
        let mut session = RaceSession::new();
        session.track_choice = TrackChoice::Custom {
            id: id.to_string(),
            title: "".to_string(),
            description: "".to_string(),
            path: format!("rally/{}", id),
        };
        session.car_choice = CarChoice::RallyCar;
        session.num_bots = 5;
        session.init_race();

        assert_eq!(session.world.vehicles.len(), 6, "1 player + 5 bots = 6 rally cars for {}", id);
        assert_eq!(session.world.trackers.len(), 6);
        assert!(!session.track.name.is_empty());
        assert_eq!(session.track_choice_id(), *id);

        // Step simulation 60 frames to ensure physical stability
        for _ in 0..60 {
            session.update();
        }

        for (i, car) in session.world.vehicles.iter().enumerate() {
            assert!(
                car.state.position.is_finite(),
                "Rally Car #{} position is non-finite on {}: {:?}",
                i,
                id,
                car.state.position
            );
            assert!(
                car.state.velocity.is_finite(),
                "Rally Car #{} velocity is non-finite on {}: {:?}",
                i,
                id,
                car.state.velocity
            );
        }
    }
}

#[test]
fn test_rally_tracks_centerline_driving_and_no_wall_obstructions() {
    use tdrace_core::collision::wall::resolve_all_wall_collisions;
    use tdrace_core::physics::{Car, CarConfig};

    let module = RallyGameModule::new();
    let tracks = module.tracks();

    for track_def in &tracks {
        let track = tdrace_core::catalog::official_track("rally", track_def.id);
        let name = &track.name;

        // 1. Verify that all starting grid slots spawn freely without any barrier collision
        for (slot_idx, grid_pose) in track.grid_positions.iter().enumerate() {
            let mut car = Car::new(CarConfig::sports_car()).with_pose(grid_pose.position, grid_pose.angle);
            car.state.road_elevation = track.spline.sample_at_distance(track.spline.project_point(grid_pose.position).progress_distance).elevation;
            let initial_pos = car.state.position;

            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);

            let displacement = (car.state.position - initial_pos).length();

            assert!(
                hit_inner.is_empty() && hit_outer.is_empty() && displacement < 0.01,
                "Track '{}' ({}) Slot #{} spawned in collision with walls! (hit_inner={}, hit_outer={}, disp={:.3}m)",
                name, track_def.id, slot_idx, !hit_inner.is_empty(), !hit_outer.is_empty(), displacement
            );
        }

        // 2. Drive along the centerline at 1.0m intervals: Ensure no barriers cross the track ribbon
        let total_len = track.spline.total_length();
        let num_samples = (total_len / 1.0) as usize;
        for i in 0..num_samples {
            let dist = i as f32 * 1.0;
            let sample = track.spline.sample_at_distance(dist);
            let heading = sample.tangent.y.atan2(sample.tangent.x);
            let mut car = Car::new(CarConfig::sports_car()).with_pose(sample.point, heading);
            car.state.road_elevation = sample.elevation;

            let initial_pos = car.state.position;
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);

            let displacement = (car.state.position - initial_pos).length();
            assert!(
                hit_inner.is_empty() && hit_outer.is_empty() && displacement < 0.01,
                "Track '{}' ({}) has barrier obstruction at dist={:.1}m / {:.1}m: pos=({:.1}, {:.1}), disp={:.3}m (hit_inner={}, hit_outer={})",
                name, track_def.id, dist, total_len, sample.point.x, sample.point.y, displacement, !hit_inner.is_empty(), !hit_outer.is_empty()
            );
        }
    }
}

#[test]
fn test_export_and_save_rally_tracks_to_disk() {
    use std::fs;
    use tdrace_core::track::Track;

    let temp_dir = std::env::temp_dir().join(format!("tdrace_rally_export_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    let presets_to_export = [
        ("dirt_figure_eight", tdrace_core::catalog::official_track("classic", "dirt_figure_eight")),
        ("holjes_rx", tdrace_core::catalog::official_track("rally", "holjes_rx")),
        ("lydden_hill", tdrace_core::catalog::official_track("rally", "lydden_hill")),
        ("hell_rx", tdrace_core::catalog::official_track("rally", "hell_rx")),
        ("loheac_rx", tdrace_core::catalog::official_track("rally", "loheac_rx")),
        ("essay_rx", tdrace_core::catalog::official_track("rally", "essay_rx")),
    ];

    for (slug, track) in &presets_to_export {
        let file_path = temp_dir.join(format!("{}.json", slug));
        track.save_to_file(&file_path).expect("Must save track to file");

        // Verify it can be loaded back
        let loaded = Track::load_from_file(&file_path).expect("Must load track from file");
        assert_eq!(loaded.name, track.name);
        assert_eq!(loaded.spline.waypoints.len(), track.spline.waypoints.len());
        assert_eq!(loaded.checkpoints.len(), track.checkpoints.len());
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_blyton_ids_resolve_to_croft() {
    // Blyton Park RX (not mapped in OSM) was replaced by Croft; old saves and series keep working.
    for old in ["blyton_rx", "blyton_park", "blyton_park_rx"] {
        assert_eq!(tdrace_core::catalog::canonical_id(old), Some("croft_rx"), "{} must alias croft_rx", old);
    }
    let croft = tdrace_core::catalog::official_track("rally", "blyton_rx");
    assert_eq!(croft.name, "Croft Rallycross Circuit");
    assert_eq!(croft.scale(), "1:1");
    let lap = croft.spline.total_length();
    assert!((lap - 1251.0).abs() / 1251.0 < 0.02, "Croft lap is {:.0} m", lap);
}

#[test]
fn test_silverstone_and_yas_marina_ids_resolve_to_their_replacements() {
    // Yas Marina RX aliases Lessay, while Silverstone RX is restored as an authentic circuit (Spec 051).
    for (old, new) in [("yas_marina_rx", "lessay_rx"), ("yas_marina", "lessay_rx"), ("silverstone_rallycross", "silverstone_rx"), ("spa_rallycross", "spa_rx"), ("erx", "erx_motor_park")] {
        assert_eq!(tdrace_core::catalog::canonical_id(old), Some(new), "{} must alias {}", old, new);
    }
    assert_eq!(tdrace_core::catalog::official_track("rally", "silverstone_rx").name, "Silverstone Circuit RX");
    assert_eq!(tdrace_core::catalog::official_track("rally", "yas_marina_rx").name, "Circuit de Lessay");
}

#[test]
fn test_all_20_world_rx_circuits_have_valid_joker_track_networks() {
    let expected_ids = [
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "estering_rx",
        "montalegre_rx",
        "nyirad_rx",
        "kouvola_rx",
        "catalunya_rx",
        "mettet_rx",
        "lavare_rx",
        "riga_rx",
        "killarney_rx",
        "lessay_rx",
        "essay_rx",
        "dreux_rx",
        "croft_rx",
        "spa_rx",
        "silverstone_rx",
        "erx_motor_park",
    ];

    for id in &expected_ids {
        let track = tdrace_core::catalog::official_track("rally", id);
        let network = track.network.as_ref().unwrap_or_else(|| {
            panic!("{}: missing track.network", id);
        });

        let main_layout = network.get_layout("main").unwrap_or_else(|| {
            panic!("{}: missing main layout in track network", id);
        });
        let joker_layout = network.get_layout("joker").unwrap_or_else(|| {
            panic!("{}: missing joker layout in track network", id);
        });

        // Delta between 30 m and 70 m
        let delta = joker_layout.total_lap_length - main_layout.total_lap_length;
        assert!(
            delta >= 30.0 && delta <= 70.0,
            "{}: joker delta {:.1} m must be between 30 m and 70 m (main: {:.1} m, joker: {:.1} m)",
            id,
            delta,
            main_layout.total_lap_length,
            joker_layout.total_lap_length
        );

        // Verify composite splines can be synthesized for both layouts
        let main_spline = network.build_composite_spline_for_layout("main");
        assert!(main_spline.is_some(), "{}: failed to build composite spline for main layout", id);
        let joker_spline = network.build_composite_spline_for_layout("joker");
        assert!(joker_spline.is_some(), "{}: failed to build composite spline for joker layout", id);

        // Verify split and merge junctions exist
        assert_eq!(network.junctions.len(), 2, "{}: expected 2 junctions (split and merge)", id);

        // Verify split junction has gore config and C1 tangent continuity (< 1e-4 rad)
        let split_j = &network.junctions[0];
        if let tdrace_core::track::network::JunctionKind::Split {
            ingress_socket,
            egress_sockets,
            gore_config,
        } = &split_j.kind {
            assert!(gore_config.is_some(), "{}: split junction must have gore_config", id);
            assert_eq!(egress_sockets.len(), 2, "{}: split junction must have 2 egress sockets", id);

            // C1 tangent continuity at split
            for (idx, egress) in egress_sockets.iter().enumerate() {
                let perp_dot = ingress_socket.tangent.perp_dot(egress.tangent);
                let dot = ingress_socket.tangent.dot(egress.tangent);
                let angle_rad = perp_dot.atan2(dot).abs();
                assert!(
                    angle_rad < 1e-4,
                    "{}: split socket {} tangent divergence {:.6} rad must be < 1e-4 rad",
                    id,
                    idx,
                    angle_rad
                );
            }
        } else {
            panic!("{}: junction 0 must be Split", id);
        }

        // Verify merge junction has merge config and C1 tangent continuity (< 1e-4 rad)
        let merge_j = &network.junctions[1];
        if let tdrace_core::track::network::JunctionKind::Merge {
            ingress_sockets,
            egress_socket,
            merge_config,
        } = &merge_j.kind {
            assert!(merge_config.is_some(), "{}: merge junction must have merge_config", id);
            assert_eq!(ingress_sockets.len(), 2, "{}: merge junction must have 2 ingress sockets", id);

            // C1 tangent continuity at merge
            for (idx, ingress) in ingress_sockets.iter().enumerate() {
                let perp_dot = egress_socket.tangent.perp_dot(ingress.tangent);
                let dot = egress_socket.tangent.dot(ingress.tangent);
                let angle_rad = perp_dot.atan2(dot).abs();
                assert!(
                    angle_rad < 1e-4,
                    "{}: merge socket {} tangent divergence {:.6} rad must be < 1e-4 rad",
                    id,
                    idx,
                    angle_rad
                );
            }
        } else {
            panic!("{}: junction 1 must be Merge", id);
        }

        // Verify joker checkpoints exist
        let has_joker_cp = track.checkpoints.iter().any(|cp| cp.is_joker);
        assert!(has_joker_cp, "{}: track must have at least one joker checkpoint", id);

        // Verify centerline driving on Joker segment has no barrier collisions
        use tdrace_core::collision::wall::resolve_all_wall_collisions;
        use tdrace_core::physics::{Car, CarConfig};
        let seg2 = network.get_segment(tdrace_core::track::network::SegmentId(2)).unwrap();
        for sample in &seg2.samples {
            let heading = sample.tangent.y.atan2(sample.tangent.x);
            let mut car = Car::new(CarConfig::rally_car()).with_pose(sample.point, heading);
            car.state.road_elevation = sample.elevation;
            let initial_pos = car.state.position;
            let hit_inner = resolve_all_wall_collisions(&mut car, &track.geometry.inner_walls, &[]);
            let hit_outer = resolve_all_wall_collisions(&mut car, &track.geometry.outer_walls, &[]);
            let displacement = (car.state.position - initial_pos).length();
            assert!(
                hit_inner.is_empty() && hit_outer.is_empty() && displacement < 0.01,
                "Track '{}' ({}) has wall collision on Joker segment at pos=({:.1}, {:.1}): disp={:.3}m (hit_inner={}, hit_outer={})",
                track.name, id, sample.point.x, sample.point.y, displacement, !hit_inner.is_empty(), !hit_outer.is_empty()
            );
        }
    }
}

#[test]
fn test_holjes_rx_joker_lap_time_delta_simulation() {
    let track = tdrace_core::catalog::official_track("rally", "holjes_rx");
    let network = track.network.as_ref().expect("holjes_rx must have network");

    let seg1 = network.get_segment(tdrace_core::track::network::SegmentId(1)).unwrap();
    let seg2 = network.get_segment(tdrace_core::track::network::SegmentId(2)).unwrap();

    // Racing cruise speed (e.g. 15.0 m/s = ~54 km/h typical cornering speed in technical rallycross sections)
    // Extra distance of 42.0m at 15.0m/s results in ~2.8s delta
    let cruise_speed = 15.0f32;
    let time_main = seg1.length / cruise_speed;
    let time_joker = seg2.length / cruise_speed;
    let time_delta = time_joker - time_main;

    assert!(
        time_delta >= 2.0 && time_delta <= 4.5,
        "Höljes RX time delta {:.2}s must be between 2.0s and 4.5s (main: {:.2}s, joker: {:.2}s)",
        time_delta,
        time_main,
        time_joker
    );
}


/// The 23 official circuits with a joker layout: 3 Classic RX venues and the 20 World RX circuits.
const RX_JOKER_TRACKS: [(&str, &str); 23] = [
    ("classic", "rx_quarry_sprint"),
    ("classic", "rx_hilltop_leap"),
    ("classic", "rx_canyon_flyer"),
    ("rally", "holjes_rx"),
    ("rally", "lydden_hill"),
    ("rally", "hell_rx"),
    ("rally", "loheac_rx"),
    ("rally", "estering_rx"),
    ("rally", "montalegre_rx"),
    ("rally", "nyirad_rx"),
    ("rally", "kouvola_rx"),
    ("rally", "catalunya_rx"),
    ("rally", "mettet_rx"),
    ("rally", "lavare_rx"),
    ("rally", "riga_rx"),
    ("rally", "killarney_rx"),
    ("rally", "lessay_rx"),
    ("rally", "essay_rx"),
    ("rally", "dreux_rx"),
    ("rally", "croft_rx"),
    ("rally", "spa_rx"),
    ("rally", "silverstone_rx"),
    ("rally", "erx_motor_park"),
];

#[test]
fn test_rx_layout_checkpoints_lie_on_their_route_in_driving_order() {
    // The progress tracker expects each layout's checkpoints in the order a car meets them on that route.
    // A checkpoint listed out of order on the joker route makes a car that drives the joker lose its lap.
    for (module, id) in RX_JOKER_TRACKS {
        let track = tdrace_core::catalog::official_track(module, id);
        let network = track.network.as_ref().unwrap_or_else(|| panic!("{}: missing network", id));
        for layout_id in ["main", "joker"] {
            let layout = network.get_layout(layout_id).unwrap_or_else(|| panic!("{}: missing {} layout", id, layout_id));
            let spline = network.build_composite_spline_for_layout(layout_id).expect("composite spline");
            let mut prev = f32::NEG_INFINITY;
            for &cid in &layout.checkpoint_ids {
                let cp = track.checkpoints.iter().find(|c| c.id == cid).unwrap_or_else(|| panic!("{}: no checkpoint {}", id, cid));
                let mid = (cp.gate.start + cp.gate.end) * 0.5;
                let proj = spline.project_point(mid);
                assert!(
                    proj.distance_to_spline <= proj.track_width * 0.5,
                    "{} {}: checkpoint {} is {:.1} m from the route centerline (half width {:.1} m)",
                    id, layout_id, cid, proj.distance_to_spline, proj.track_width * 0.5
                );
                // The finish line sits on the loop seam, so it can project to the end of the lap.
                let mut at = proj.progress_distance;
                if prev == f32::NEG_INFINITY && at > spline.total_length() * 0.5 {
                    at -= spline.total_length();
                }
                assert!(
                    at > prev,
                    "{} {}: checkpoint {} at {:.0} m comes before the previous one at {:.0} m",
                    id, layout_id, cid, at, prev
                );
                prev = at;
            }
        }
        // The tracker moves a car to a checkpoint's segment_id when the car crosses it, so the tag must name
        // a segment the checkpoint lies on.
        for cp in &track.checkpoints {
            let Some(sid) = cp.segment_id else { continue };
            let seg = network.get_segment(sid).unwrap_or_else(|| panic!("{}: checkpoint {} names missing segment {:?}", id, cp.id, sid));
            let proj = seg.project_point((cp.gate.start + cp.gate.end) * 0.5);
            assert!(
                proj.distance_to_spline <= proj.track_width * 0.5,
                "{}: checkpoint {} is tagged segment {:?} but lies {:.1} m from it",
                id, cp.id, sid, proj.distance_to_spline
            );
        }
    }
}

#[test]
fn test_rx_joker_road_reads_as_its_own_surface_not_runoff() {
    // The joker ribbon can run inside the main line's run-off corridor. Wheels on joker road must
    // still get the joker's road surface, or a car on the joker drives on run-off grip.
    for (module, id) in RX_JOKER_TRACKS {
        let track = tdrace_core::catalog::official_track(module, id);
        let network = track.network.as_ref().unwrap_or_else(|| panic!("{}: missing network", id));
        let joker = network.get_layout("joker").expect("joker layout");
        let main = network.get_layout("main").expect("main layout");
        let joker_only = joker.segment_sequence.iter().find(|s| !main.segment_sequence.contains(s)).expect("joker-only segment");
        let seg = network.get_segment(*joker_only).expect("joker segment");
        let mut wrong = Vec::new();
        for pair in seg.samples.windows(2) {
            let sample = &pair[1];
            // Skip the sample where the joker's own surface changes, since either side of that edge is right.
            if pair[0].surface != sample.surface {
                continue;
            }
            // Where other road overlaps the joker (junction throats, a crossing) either road is right, and
            // jump ramps rightly take precedence.
            let main_proj = track.spline.project_point(sample.point);
            let other_road = network.segments.iter().any(|s| s.id != seg.id && s.project_point(sample.point).is_on_track);
            if main_proj.is_on_track || main_proj.is_on_curb || other_road || track.geometry.jump_ramps.iter().any(|r| r.contains(sample.point)) {
                continue;
            }
            let hint = main_proj.progress_distance;
            for (how, got) in [("far", track.sample_surface(sample.point)), ("near", track.sample_surface_near(sample.point, hint))] {
                if got != sample.surface {
                    wrong.push(format!("{} at {:.0} m: {:?} instead of {:?}", how, sample.distance, got, sample.surface));
                }
            }
        }
        assert!(wrong.is_empty(), "{}: joker road reads wrong at {} samples, e.g. {:?}", id, wrong.len(), &wrong[..wrong.len().min(3)]);
    }
}

#[test]
fn test_rx_car_driving_the_joker_route_gets_its_lap_and_its_joker() {
    // End to end through race_kit::RaceWorld::step: lap 1 on the joker route, lap 2 on the main route.
    use race_kit::{DriveControls, RaceFormat, RaceRules, RaceWorld};
    use tdrace_core::physics::{Car, CarConfig};
    use tdrace_core::track::TrackProgressTracker;

    const DT: f32 = 1.0 / 60.0;
    const SPEED: f32 = 15.0;
    let mut failures = Vec::new();
    for (module, id) in RX_JOKER_TRACKS {
        let track = tdrace_core::catalog::official_track(module, id);
        let network = track.network.as_ref().unwrap_or_else(|| panic!("{}: missing network", id));
        let joker = network.build_composite_spline_for_layout("joker").expect("joker spline");
        let main = network.build_composite_spline_for_layout("main").expect("main spline");
        let route = [(&joker, 1.0f32), (&main, 0.0f32)];

        let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(2), ..RaceRules::default() });
        let start = joker.sample_at_distance(1.0);
        world.spawn(
            Car::new(CarConfig::rally_car()).with_pose(start.point, start.tangent.y.atan2(start.tangent.x)),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );

        let mut surface_mismatches = 0;
        let mut wrong_way = None;
        let mut first_mismatch = None;
        'laps: for (spline, from) in route {
            let mut d = from;
            while d < spline.total_length() {
                let s = spline.sample_at_distance(d);
                let car = &mut world.vehicles[0];
                car.state.position = s.point;
                car.state.angle = s.tangent.y.atan2(s.tangent.x);
                car.set_velocity(s.tangent * SPEED);
                let expected = track.sample_car_surfaces(car);
                world.step(&track, &[DriveControls::default()], DT);
                // Where the main curb overlaps branch road near a junction, curb and road are both right.
                let differs = world.last_surfaces[0]
                    .iter()
                    .zip(expected.iter())
                    .any(|(got, want)| got != want && *got != SurfaceType::Curb && *want != SurfaceType::Curb);
                if differs {
                    surface_mismatches += 1;
                    if first_mismatch.is_none() {
                        first_mismatch = Some(format!("{} lap at {:.0} m: {:?} instead of {:?}", if from > 0.0 { "joker" } else { "main" }, d, world.last_surfaces[0], expected));
                    }
                }
                if world.trackers[0].is_wrong_way && wrong_way.is_none() {
                    wrong_way = Some(format!("{} lap at {:.0} m", if from > 0.0 { "joker" } else { "main" }, d));
                }
                if world.is_finished(0) {
                    break 'laps;
                }
                d += SPEED * DT;
            }
        }
        // Cross the line once more if the last step stopped just short of it.
        for k in 1..=8 {
            if world.is_finished(0) {
                break;
            }
            let s = main.sample_at_distance(k as f32 * SPEED * DT);
            world.vehicles[0].state.position = s.point;
            world.step(&track, &[DriveControls::default()], DT);
        }

        let tracker = &world.trackers[0];
        let jokers = tracker.multi_route.as_ref().map_or(0, |m| m.joker_laps_completed);
        if !world.is_finished(0) || tracker.current_lap != 3 || jokers != 1 || surface_mismatches > 0 || wrong_way.is_some() {
            failures.push(format!(
                "{}: finished {}, lap {}, jokers {}, surface mismatches {} (first {:?}), first wrong way {:?}",
                id, world.is_finished(0), tracker.current_lap, jokers, surface_mismatches, first_mismatch, wrong_way
            ));
        }
    }
    assert!(failures.is_empty(), "{} RX tracks failed:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn test_rx_joker_branch_never_turns_tighter_than_3_m() {
    // A joker built as a sideways offset of the main branch folded back on itself where the offset was larger
    // than the bend radius (turn radii 0.0-0.8 m). Points within 2 m of the main branch are skipped: there the
    // joker follows the main road, whose OSM geometry has its own tight kinks.
    let mut failures = Vec::new();
    for (module, id) in RX_JOKER_TRACKS {
        let track = tdrace_core::catalog::official_track(module, id);
        let network = track.network.as_ref().unwrap_or_else(|| panic!("{}: missing network", id));
        let joker = network.get_layout("joker").expect("joker layout");
        let main = network.get_layout("main").expect("main layout");
        let joker_seg = joker.segment_sequence.iter().find(|s| !main.segment_sequence.contains(s)).expect("joker-only segment");
        let main_seg = main.segment_sequence.iter().find(|s| !joker.segment_sequence.contains(s)).expect("main-only segment");
        let seg = network.get_segment(*joker_seg).unwrap();
        let main_branch = network.get_segment(*main_seg).unwrap();
        let tightest = seg
            .samples
            .windows(2)
            .filter(|w| main_branch.project_point(w[0].point).distance_to_spline > 2.0)
            .filter_map(|w| {
                let angle = w[0].tangent.dot(w[1].tangent).clamp(-1.0, 1.0).acos();
                let ds = w[1].distance - w[0].distance;
                (angle > 1e-4 && ds > 0.0).then(|| (ds / angle, w[0].distance))
            })
            .fold((f32::INFINITY, 0.0), |a, b| if b.0 < a.0 { b } else { a });
        if tightest.0 < 3.0 {
            failures.push(format!("{}: joker turns at {:.1} m radius, {:.0} m into the branch", id, tightest.0, tightest.1));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn test_rx_races_get_the_joker_rule_and_other_races_do_not() {
    use race_kit::JokerRule;
    use tdrace_app::game::joker_rule_for;

    let rx_rule = JokerRule { mandatory: 1, penalty_s: 30.0 };
    for (module, id) in RX_JOKER_TRACKS {
        assert_eq!(joker_rule_for(&tdrace_core::catalog::official_track(module, id)), rx_rule, "{}", id);
    }
    // Scenario: Other categories are not affected (Classic GT, Karting, Autocross, all-terrain and stock car).
    for id in ["gt_coastal_grand_prix", "kart_pine_grove", "ax_clay_bowl", "ax_meadow_sprint", "at_dune_sea", "stock_roval"] {
        assert_eq!(joker_rule_for(&tdrace_core::catalog::official_track("classic", id)), JokerRule::default(), "{}", id);
    }
}

#[test]
fn test_rx_race_ignores_the_joker_layout_pick() {
    // Scenario: RX race ignores the joker layout pick
    let mut session = RaceSession::new();
    session.selected_layout_id = Some("joker".to_string());
    let track = session.load_track_for_session(&TrackChoice::ClassicRallycross);
    let network = track.network.as_ref().expect("classic rallycross has a network");
    assert_eq!(network.default_layout_id, "main");
    let main = network.build_composite_spline_for_layout("main").unwrap();
    assert!((track.spline.total_length() - main.total_length()).abs() < 1.0, "the race spline is the main layout");
}
