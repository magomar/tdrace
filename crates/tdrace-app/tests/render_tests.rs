use glam::Vec2;
use tdrace_app::render::color::{CarColorScheme, Palette};
use tdrace_core::{Car, CarConfig};
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};

#[test]
fn test_palette_and_car_color_schemes() {
    assert_eq!(Palette::CAR_COLORS.len(), 8);

    for i in 0..8 {
        let scheme = CarColorScheme::from_index(i);
        assert!(scheme.primary.a > 0.0);
        assert!(scheme.secondary.a > 0.0);
        assert!(scheme.helmet.a > 0.0);
    }

    // Wrap around
    let scheme_wrap = CarColorScheme::from_index(10);
    let scheme_2 = CarColorScheme::from_index(2);
    assert_eq!(scheme_wrap, scheme_2);
}

#[test]
fn test_track_presets_geometry_for_rendering() {
    let tracks = [
        classic_grand_prix(),
        oval_speedway(),
        drift_park(),
        kart_arena(),
    ];

    for t in &tracks {
        assert!(t.spline.samples.len() >= 10);
        assert!(!t.checkpoints.is_empty());
        assert!(!t.grid_positions.is_empty());
        assert!(t.spline.total_length() > 0.0);
    }
}

#[test]
fn test_car_body_roll_and_geometry() {
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 20.0), 0.5);
    car.state.acceleration_local = Vec2::new(5.0, -8.0); // braking + turning hard

    let roll_lat = (-car.state.acceleration_local.y * 0.015).clamp(-0.18, 0.18);
    let pitch_long = (car.state.acceleration_local.x * 0.012).clamp(-0.15, 0.15);

    assert!(roll_lat > 0.0); // Leaning right
    assert!(pitch_long > 0.0); // Squat/dive offset

    let wheels = car.wheel_positions_world();
    assert_eq!(wheels.len(), 4);
}

#[test]
fn test_track_backdrop_colors() {
    use tdrace_app::render::get_track_backdrop_color;
    use tdrace_core::physics::surface::SurfaceType;

    let col_grass = get_track_backdrop_color(SurfaceType::Grass);
    let col_sand = get_track_backdrop_color(SurfaceType::Sand);
    let col_dirt = get_track_backdrop_color(SurfaceType::Dirt);
    let col_asphalt = get_track_backdrop_color(SurfaceType::Asphalt);

    // Ensure all backdrop colors are opaque and distinct
    assert_eq!(col_grass.a, 1.0);
    assert_eq!(col_sand.a, 1.0);
    assert_eq!(col_dirt.a, 1.0);
    assert_eq!(col_asphalt.a, 1.0);

    assert_ne!(col_grass, col_sand);
    assert_ne!(col_grass, col_dirt);
    assert_ne!(col_grass, col_asphalt);
    assert_ne!(col_sand, col_dirt);
    assert_ne!(col_sand, col_asphalt);
    assert_ne!(col_dirt, col_asphalt);

    // Fallback for non-offtrack types
    let col_fallback = get_track_backdrop_color(SurfaceType::Water);
    assert_eq!(col_fallback, Palette::BACKDROP_GRASS);
}
