use tdrace_app::ui::menu::{resolve_track_for_menu, TrackChoice};
use tdrace_app::ui::track_preview::{compute_track_bounds, surface_preview_color};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::classic_grand_prix;

#[test]
fn test_surface_preview_colors_coverage() {
    for &s in &SurfaceType::ALL {
        let col = surface_preview_color(s);
        assert!(col.a > 0.0, "Surface preview color alpha must be > 0");
    }
}

#[test]
fn test_track_bounds_computation() {
    for choice in &TrackChoice::ALL {
        let track = resolve_track_for_menu(choice).expect("All standard choices must resolve");
        let (min, max) = compute_track_bounds(&track);
        assert!(max.x > min.x, "Track bounding width must be positive");
        assert!(max.y > min.y, "Track bounding height must be positive");
    }
}

#[test]
fn test_menu_track_resolver_special_modules() {
    let gt_circuit_ids = [
        "monza",
        "spa",
        "silverstone",
        "monaco",
        "suzuka",
        "interlagos",
        "montreal",
        "red_bull_ring",
        "catalunya",
        "zandvoort",
        "bahrain",
        "marina_bay",
        "cota",
    ];

    for id in gt_circuit_ids {
        let choice = TrackChoice::Custom {
            id: id.to_string(),
            title: id.to_string(),
            description: format!("GT {}", id),
            path: format!("gt/{}", id),
        };
        let track = resolve_track_for_menu(&choice)
            .unwrap_or_else(|| panic!("GT circuit '{}' must resolve via menu resolver", id));
        assert!(
            track.total_length_m() > 400.0,
            "GT circuit '{}' length must be > 400m",
            id
        );
        assert_eq!(
            track.surface_summary_string(),
            "100% Asphalt",
            "GT circuit '{}' surface must be 100% Asphalt",
            id
        );
    }

    let oasis = resolve_track_for_menu(&TrackChoice::OasisRally).expect("Oasis Rally must resolve");
    assert!(oasis.total_length_m() > 400.0);
    assert_eq!(oasis.surface_summary_string(), "100% Dirt");
}

#[test]
fn test_track_preview_surface_breakdown_percentages() {
    let gp = classic_grand_prix();
    let breakdown = gp.surface_breakdown();
    assert_eq!(breakdown.len(), 1);
    assert_eq!(breakdown[0].0, SurfaceType::Asphalt);
    assert!((breakdown[0].1 - 100.0).abs() < 1e-3);
    assert_eq!(gp.surface_summary_string(), "100% Asphalt");
}

#[test]
fn test_suzuka_circuit_grid_and_crossover_geometry() {
    use tdrace_app::module::gt::GtWorldChallengeModule;
    let suzuka = GtWorldChallengeModule::track_suzuka();

    assert_eq!(suzuka.grid_positions.len(), 18);

    // Verify all 20 grid positions are aligned forward along the main straight (tangent ~ (1, 0))
    for (idx, pose) in suzuka.grid_positions.iter().enumerate() {
        assert!(
            pose.angle.abs() < 0.05,
            "Suzuka grid slot {} angle ({:.3} rad) must face forward along main straight",
            idx,
            pose.angle
        );
        assert!(
            pose.position.y.abs() < 5.0,
            "Suzuka grid slot {} Y position ({:.1}) must be on main straight (around Y=0)",
            idx,
            pose.position.y
        );
    }

    // Verify finish line checkpoint is at start line facing forward
    let finish_cp = suzuka.checkpoints.iter().find(|cp| cp.is_finish_line).expect("Finish line checkpoint required");
    assert!(finish_cp.direction.x > 0.95, "Finish line direction must point forward down the straight");

    // Verify overpass bridge elevation exists and exceeds 4.0m
    let max_elev = suzuka.spline.samples.iter().map(|s| s.elevation).fold(0.0f32, f32::max);
    assert!(max_elev >= 4.5, "Suzuka overpass bridge elevation must reach at least 4.5m, got {}", max_elev);
}

#[test]
fn test_validate_all_circuits_and_presets() {
    use tdrace_app::module::gt::GtWorldChallengeModule;
    use tdrace_app::module::kart::KartGameModule;
    use tdrace_app::module::GameModule;
    use tdrace_core::track::presets::{
        classic_grand_prix, drift_park, dune_raid, kart_arena, oasis_rally,
        oval_speedway, ramp_raceway, sahara_dunes,
    };
    use tdrace_core::track::validation::{validate_track, ValidationSeverity};

    let preset_tracks = [
        ("Classic Grand Prix", classic_grand_prix()),
        ("Oval Speedway", oval_speedway()),
        ("Drift Park", drift_park()),
        ("Kart Arena", kart_arena()),
        ("Ramp Raceway", ramp_raceway()),
        ("Oasis Rally", oasis_rally()),
        ("Sahara Dunes", sahara_dunes()),
        ("Dune Raid", dune_raid()),
    ];

    let gt_module = GtWorldChallengeModule::new();
    let gt_tracks = gt_module.tracks();

    let kart_module = KartGameModule::new();
    let kart_tracks = kart_module.tracks();

    println!("\n=== VALIDATING ALL CIRCUITS ===");

    let mut total_errors = 0;

    for (name, track) in &preset_tracks {
        let diags = validate_track(track);
        let errors: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Error).collect();
        let warns: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Warning).collect();
        println!("[PRESET] {:<25} | errors: {:2} | warns: {:2}", name, errors.len(), warns.len());
        total_errors += errors.len();
    }

    for t_def in &gt_tracks {
        let track = (t_def.generator)();
        let diags = validate_track(&track);
        let errors: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Error).collect();
        let warns: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Warning).collect();
        println!("[GT]     {:<25} | errors: {:2} | warns: {:2}", t_def.id, errors.len(), warns.len());
        if !errors.is_empty() {
            for err in errors.iter().take(5) {
                println!("  [{}] {}: {}", t_def.id, err.code, err.message);
            }
            if errors.len() > 5 {
                println!("  ... and {} more errors", errors.len() - 5);
            }
        }
        total_errors += errors.len();
    }

    for t_def in &kart_tracks {
        let track = (t_def.generator)();
        let diags = validate_track(&track);
        let errors: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Error).collect();
        let warns: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Warning).collect();
        println!("[KART]   {:<25} | errors: {:2} | warns: {:2}", t_def.id, errors.len(), warns.len());
        total_errors += errors.len();
    }

    println!("\nTOTAL ERRORS ACROSS ALL TRACKS: {}", total_errors);
    assert_eq!(total_errors, 0, "Total validation errors across all tracks: {}", total_errors);
}

static DEV_MODE_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_menu_track_cache_performance_and_consistency() {
    let _lock = DEV_MODE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    use tdrace_app::ui::menu::clear_menu_track_cache;
    use std::time::Instant;

    clear_menu_track_cache();

    let holjes_choice = TrackChoice::Custom {
        id: "holjes_rx".to_string(),
        title: "Höljes Motorstadion".to_string(),
        description: "World RX Sweden".to_string(),
        path: "rally/holjes_rx".to_string(),
    };

    // First call: initial resolution / cache miss
    let t0 = Instant::now();
    let track1 = resolve_track_for_menu(&holjes_choice).expect("Höljes must resolve");
    let dur_first = t0.elapsed();

    // Second call: cache hit - should be instantaneous
    let t1 = Instant::now();
    let track2 = resolve_track_for_menu(&holjes_choice).expect("Höljes must resolve from cache");
    let dur_second = t1.elapsed();

    assert_eq!(track1.name, track2.name);
    assert_eq!(track1.spline.samples.len(), track2.spline.samples.len());
    // Cache hit should be dramatically faster than the initial generation
    println!("Höljes first resolve: {:?}, second cached resolve: {:?}", dur_first, dur_second);
    assert!(dur_second.as_millis() < 5, "Cached resolve must take < 5ms, took {:?}", dur_second);

    // Test clear_menu_track_cache
    clear_menu_track_cache();
}

struct DevModeTestGuard {
    git_dir_set: bool,
    user_tracks_dir_set: bool,
}
impl DevModeTestGuard {
    #[allow(dead_code)]
    fn enter_with_git_dir(git_dir: &std::path::Path) -> Self {
        std::env::set_var(tdrace_app::storage::ENV_DEV_MODE, "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, git_dir);
        DevModeTestGuard {
            git_dir_set: true,
            user_tracks_dir_set: false,
        }
    }

    fn enter_with_git_and_user_dir(git_dir: &std::path::Path, user_dir: &std::path::Path) -> Self {
        std::env::set_var(tdrace_app::storage::ENV_DEV_MODE, "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, git_dir);
        std::env::set_var(tdrace_app::storage::ENV_USER_TRACKS_DIR, user_dir);
        DevModeTestGuard {
            git_dir_set: true,
            user_tracks_dir_set: true,
        }
    }
}
impl Drop for DevModeTestGuard {
    fn drop(&mut self) {
        std::env::remove_var(tdrace_app::storage::ENV_DEV_MODE);
        if self.git_dir_set {
            std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
        }
        if self.user_tracks_dir_set {
            std::env::remove_var(tdrace_app::storage::ENV_USER_TRACKS_DIR);
        }
    }
}

#[test]
fn test_thumbnail_refresh_when_overwriting_custom_circuit() {
    use std::fs;
    use tdrace_app::track_manager::TrackManager;
    use tdrace_app::ui::menu::{clear_menu_track_cache, resolve_track_for_menu_with_dir};
    use tdrace_core::track::spline::TrackWaypoint;

    clear_menu_track_cache();

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_thumb_refresh_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create and save initial circuit
    let mut initial_track = classic_grand_prix();
    initial_track.name = "Thumbnail Test Circuit".to_string();
    let initial_wp_count = initial_track.spline.waypoints.len();
    let path = manager
        .save_custom_track_with_options(&initial_track, Some("thumb_test_circuit"), true)
        .expect("Initial save must succeed");

    let choice = TrackChoice::Custom {
        id: "thumb_test_circuit".to_string(),
        title: "Thumbnail Test Circuit".to_string(),
        description: "Initial description".to_string(),
        path: path.clone(),
    };

    // 2. Resolve track for menu thumbnail (caches the initial track)
    let loaded1 = resolve_track_for_menu_with_dir(&choice, &temp_dir)
        .expect("Must resolve initial track for thumbnail");
    assert_eq!(loaded1.spline.waypoints.len(), initial_wp_count);
    let (min1, max1) = compute_track_bounds(&loaded1);

    // 3. Modify track geometry (add an extended waypoint that dramatically alters the bounding box)
    let mut modified_track = loaded1;
    modified_track.spline.waypoints.push(TrackWaypoint::new(
        glam::Vec2::new(9999.0, 8888.0),
        16.0,
    ));
    modified_track.rebuild_geometry(2.5, tdrace_core::track::geometry::BarrierType::Steel);
    modified_track.name = "Overwritten Thumbnail Circuit".to_string();

    // 4. Overwrite existing track
    let overwritten_path = manager
        .save_custom_track_with_options(&modified_track, Some("thumb_test_circuit"), true)
        .expect("Overwrite save must succeed");
    assert_eq!(overwritten_path, path);

    // 5. Re-resolve track for menu thumbnail: MUST reflect the overwritten circuit, not stale cache
    let loaded2 = resolve_track_for_menu_with_dir(&choice, &temp_dir)
        .expect("Must resolve overwritten track for thumbnail");
    assert_eq!(
        loaded2.spline.waypoints.len(),
        initial_wp_count + 1,
        "Thumbnail track must include new waypoint after overwrite"
    );
    assert_eq!(loaded2.name, "Overwritten Thumbnail Circuit");

    let (min2, max2) = compute_track_bounds(&loaded2);
    assert!(
        max2.x >= 9999.0,
        "Thumbnail bounding box max X must expand to include new waypoint ({:?} vs {:?})",
        max1,
        max2
    );
    assert_ne!(
        (min1, max1),
        (min2, max2),
        "Thumbnail bounds must update after overwriting circuit"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_thumbnail_refresh_when_overwriting_preset_in_dev_mode() {
    let _lock = DEV_MODE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    use std::fs;
    use tdrace_app::track_manager::TrackManager;
    use tdrace_app::ui::menu::{clear_menu_track_cache, resolve_track_for_menu};
    use tdrace_core::track::presets::classic_grand_prix;
    use tdrace_core::track::spline::TrackWaypoint;

    clear_menu_track_cache();

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_dev_preset_thumb_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mock_git_tracks = temp_dir.join("git_tracks");
    fs::create_dir_all(&mock_git_tracks.join("classic")).unwrap();

    let _dev_guard = DevModeTestGuard::enter_with_git_and_user_dir(&mock_git_tracks, &temp_dir);
    assert!(tdrace_app::storage::is_dev_mode());

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Initial resolution of official preset
    let canonical = classic_grand_prix();
    let initial_wp_count = canonical.spline.waypoints.len();
    let initial_resolved = resolve_track_for_menu(&TrackChoice::ClassicGrandPrix)
        .expect("Canonical preset must resolve");
    let (initial_min, initial_max) = compute_track_bounds(&initial_resolved);

    // 2. Modify preset geometry with distinctive far waypoint
    let mut modified_preset = canonical.clone();
    modified_preset.spline.waypoints.push(TrackWaypoint::new(
        glam::Vec2::new(7777.0, 7777.0),
        20.0,
    ));
    modified_preset.rebuild_geometry(2.5, tdrace_core::track::geometry::BarrierType::Steel);
    modified_preset.description = "Dev Mode Overwritten Circuit Preset".to_string();

    // 3. Overwrite official preset in dev mode
    let save_res = manager.save_custom_track_with_options(&modified_preset, Some("classic_grand_prix"), true);
    assert!(save_res.is_ok(), "Dev mode must allow overwriting git preset");

    // 4. Resolve thumbnail again: MUST reflect overwritten preset from git tracks directory
    let refreshed_resolved = resolve_track_for_menu(&TrackChoice::ClassicGrandPrix)
        .expect("Refreshed preset must resolve");
    assert_eq!(
        refreshed_resolved.spline.waypoints.len(),
        initial_wp_count + 1,
        "Dev mode thumbnail resolution must reflect updated waypoints"
    );
    assert_eq!(
        refreshed_resolved.description,
        "Dev Mode Overwritten Circuit Preset"
    );

    let (new_min, new_max) = compute_track_bounds(&refreshed_resolved);
    assert!(
        new_max.x >= 7777.0,
        "Refreshed preset thumbnail bounds max X must expand"
    );
    assert_ne!((initial_min, initial_max), (new_min, new_max));

    // 5. Restore canonical preset
    let restore_res = manager.save_custom_track_with_options(&canonical, Some("classic_grand_prix"), true);
    assert!(restore_res.is_ok());

    // 6. Verify restored thumbnail resolution
    let restored_resolved = resolve_track_for_menu(&TrackChoice::ClassicGrandPrix)
        .expect("Restored preset must resolve");
    assert_eq!(restored_resolved.spline.waypoints.len(), initial_wp_count);
    assert_eq!(restored_resolved.description, canonical.description);

    let _ = std::fs::remove_dir_all(&temp_dir);
}



