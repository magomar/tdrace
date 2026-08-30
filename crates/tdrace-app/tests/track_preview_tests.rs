use tdrace_app::ui::menu::{resolve_track_for_menu, TrackChoice};
use tdrace_app::ui::track_preview::{compute_track_bounds, surface_preview_color};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::classic_grand_prix;

#[test]
fn test_surface_preview_colors_coverage() {
    let surfaces = [
        SurfaceType::Asphalt,
        SurfaceType::Dirt,
        SurfaceType::Curb,
        SurfaceType::Grass,
        SurfaceType::Sand,
        SurfaceType::Water,
        SurfaceType::Oil,
        SurfaceType::Ice,
    ];

    for s in surfaces {
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
    let f1_ids = [
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

    for id in f1_ids {
        let choice = TrackChoice::Custom {
            id: id.to_string(),
            title: id.to_string(),
            description: format!("F1 {}", id),
            path: format!("f1/{}", id),
        };
        let track = resolve_track_for_menu(&choice)
            .unwrap_or_else(|| panic!("F1 track '{}' must resolve via menu resolver", id));
        assert!(
            track.total_length_m() > 400.0,
            "F1 track '{}' length must be > 400m",
            id
        );
        assert_eq!(
            track.surface_summary_string(),
            "100% Asphalt",
            "F1 track '{}' surface must be 100% Asphalt",
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
    use tdrace_app::module::f1::F1GameModule;
    let suzuka = F1GameModule::track_suzuka();

    assert_eq!(suzuka.grid_positions.len(), 20);

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

