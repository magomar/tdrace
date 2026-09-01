use tdrace_app::game::RaceSession;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::menu::{resolve_track_for_menu, CarChoice, TrackChoice};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::{
    dirt_figure_eight, hell_rx, holjes_rx, loheac_rx, lydden_hill,
};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

#[test]
fn test_rally_module_tracks_integrity_and_validation() {
    let module = RallyGameModule::new();
    let tracks = module.tracks();

    assert_eq!(tracks.len(), 8, "Rally module should have 8 tracks (5 new + 3 original)");

    let expected_ids = [
        "dirt_figure_eight",
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "oasis_rally",
        "outlaw_pass",
        "sahara_dunes",
    ];

    for id in &expected_ids {
        assert!(
            tracks.iter().any(|t| t.id == *id),
            "Expected track id '{}' in rally module tracks",
            id
        );
    }

    for track_def in &tracks {
        let track = (track_def.generator)();
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
    let fig8 = dirt_figure_eight();
    assert_eq!(fig8.name, "Dirt Figure-8 Arena");

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
fn test_world_rx_tracks_jump_ramps_and_mixed_surfaces() {
    let holjes = holjes_rx();
    assert_eq!(holjes.name, "Höljes Motorstadion (World RX Sweden)");
    assert!(!holjes.geometry.jump_ramps.is_empty(), "Höljes must have the iconic jump ramp");
    let holjes_breakdown = holjes.surface_breakdown();
    assert!(holjes_breakdown.len() >= 2, "Höljes must be mixed surface (asphalt & dirt)");

    let lydden = lydden_hill();
    assert_eq!(lydden.name, "Lydden Hill Circuit (World RX Great Britain)");
    let lydden_breakdown = lydden.surface_breakdown();
    assert!(lydden_breakdown.len() >= 2, "Lydden Hill must be mixed surface");

    let hell = hell_rx();
    assert_eq!(hell.name, "Lånkebanen (World RX Norway)");
    assert!(!hell.geometry.jump_ramps.is_empty(), "Hell RX must have jump ramp");
    let hell_breakdown = hell.surface_breakdown();
    assert!(hell_breakdown.len() >= 2, "Hell RX must be mixed surface");

    let loheac = loheac_rx();
    assert_eq!(loheac.name, "Circuit de Lohéac (World RX France)");
    assert!(!loheac.geometry.jump_ramps.is_empty(), "Lohéac must have jump ramp");
    let loheac_breakdown = loheac.surface_breakdown();
    assert!(loheac_breakdown.len() >= 2, "Lohéac must be mixed surface");
}

#[test]
fn test_famous_rally_tracks_in_track_manager_and_menu_resolution() {
    let tm = TrackManager::default();
    let rally_catalog = tm.module_catalog_tracks("rally");

    assert!(rally_catalog.len() >= 8);

    let rally_ids = [
        "dirt_figure_eight",
        "holjes_rx",
        "lydden_hill",
        "hell_rx",
        "loheac_rx",
        "oasis_rally",
        "outlaw_pass",
        "sahara",
    ];

    for id in &rally_ids {
        assert!(
            rally_catalog.iter().any(|c| c.track_id() == *id),
            "Track manager rally catalog missing '{}'",
            id
        );

        let choice = if *id == "oasis_rally" {
            TrackChoice::OasisRally
        } else if *id == "outlaw_pass" {
            TrackChoice::OutlawPass
        } else {
            TrackChoice::Custom {
                id: id.to_string(),
                title: "".to_string(),
                description: "".to_string(),
                path: format!("rally/{}", id),
            }
        };

        let loaded = tm.load_track(&choice);
        assert!(loaded.is_ok(), "Failed to load track '{}' from TrackManager: {:?}", id, loaded.err());

        let resolved = resolve_track_for_menu(&choice);
        assert!(resolved.is_some(), "Failed to resolve track '{}' for menu preview", id);
    }
}

#[test]
fn test_rally_race_session_simulation_on_new_tracks() {
    let test_tracks = ["dirt_figure_eight", "holjes_rx", "lydden_hill", "hell_rx", "loheac_rx"];

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

        assert_eq!(session.cars.len(), 6, "1 player + 5 bots = 6 rally cars for {}", id);
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
        let track = (track_def.generator)();
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
    use std::path::Path;
    use tdrace_core::track::Track;

    let targets = [
        Path::new("tracks/rally").to_path_buf(),
        Path::new("../../tracks/rally").to_path_buf(),
    ];

    let presets_to_export = [
        ("dirt_figure_eight", dirt_figure_eight()),
        ("holjes_rx", holjes_rx()),
        ("lydden_hill", lydden_hill()),
        ("hell_rx", hell_rx()),
        ("loheac_rx", loheac_rx()),
        ("sahara", tdrace_core::track::presets::sahara_dunes()),
    ];

    for dir in &targets {
        if dir.parent().map_or(false, |p| p.exists()) || dir.exists() {
            let _ = fs::create_dir_all(dir);
            for (slug, track) in &presets_to_export {
                let file_path = dir.join(format!("{}.json", slug));
                track.save_to_file(&file_path).expect("Must save track to file");

                // Verify it can be loaded back
                let loaded = Track::load_from_file(&file_path).expect("Must load track from file");
                assert_eq!(loaded.name, track.name);
                assert_eq!(loaded.spline.waypoints.len(), track.spline.waypoints.len());
                assert_eq!(loaded.checkpoints.len(), track.checkpoints.len());
            }
        }
    }
}
