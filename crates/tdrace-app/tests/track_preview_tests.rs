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
    let monza_choice = TrackChoice::Custom {
        id: "monza".to_string(),
        title: "Monza".to_string(),
        description: "Temple of Speed".to_string(),
        path: "f1/monza".to_string(),
    };
    let monza = resolve_track_for_menu(&monza_choice).expect("Monza must resolve");
    assert!(monza.total_length_m() > 500.0);
    assert_eq!(monza.surface_summary_string(), "100% Asphalt");

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
