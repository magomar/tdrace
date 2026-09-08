use glam::Vec2;
use tdrace_app::render::color::{CarColorScheme, Palette};
use tdrace_core::{Car, CarConfig};
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};

#[test]
fn test_palette_and_car_color_schemes() {
    assert_eq!(Palette::CAR_COLORS.len(), 9);

    for i in 0..9 {
        let scheme = CarColorScheme::from_index(i);
        assert!(scheme.primary.a > 0.0);
        assert!(scheme.secondary.a > 0.0);
        assert!(scheme.helmet.a > 0.0);
    }

    // Wrap around (11 % 9 = 2)
    let scheme_wrap = CarColorScheme::from_index(11);
    let scheme_2 = CarColorScheme::from_index(2);
    assert_eq!(scheme_wrap, scheme_2);
}

#[test]
fn test_cabinet_color_utilities_and_theme_reexport() {
    use tdrace_app::render::color::{color_to_hex, hex_to_color, CabinetPalette, CabinetTheme};

    let col = Palette::NEON_CYAN;
    let hex = color_to_hex(col);
    assert!(hex.starts_with('#'));
    let roundtrip = hex_to_color(&hex);
    assert!((roundtrip.r - col.r).abs() < 0.02);
    assert!((roundtrip.g - col.g).abs() < 0.02);
    assert!((roundtrip.b - col.b).abs() < 0.02);

    // Verify CabinetTheme re-export
    let theme = CabinetTheme::cyberpunk_neon();
    assert_eq!(theme.card_border_glow, CabinetPalette::NEON_CYAN);
    assert_eq!(Palette::WHITE, CabinetPalette::WHITE);
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

#[test]
fn test_stock_car_visual_archetype_and_liveries() {
    use tdrace_app::module::VehicleVisualType;

    // Verify stock car palette constants
    assert_eq!(Palette::DAYTONA_BLUE.a, 1.0);
    assert_eq!(Palette::SUNSET_ORANGE.a, 1.0);
    assert_eq!(Palette::RACING_RED.a, 1.0);
    assert_eq!(Palette::CUP_GOLD.a, 1.0);
    assert_eq!(Palette::INTIMIDATOR_BLACK.a, 1.0);
    assert_eq!(Palette::CAROLINA_BLUE.a, 1.0);

    // Verify stock car livery constructors
    let daytona = CarColorScheme::stock_car_daytona_blue();
    assert_eq!(daytona.primary, Palette::DAYTONA_BLUE);
    assert_eq!(daytona.secondary, Palette::WHITE);

    let sunset = CarColorScheme::stock_car_sunset_orange();
    assert_eq!(sunset.primary, Palette::SUNSET_ORANGE);

    let racing_red = CarColorScheme::stock_car_racing_red();
    assert_eq!(racing_red.primary, Palette::RACING_RED);

    let intimidator = CarColorScheme::stock_car_intimidator_black();
    assert_eq!(intimidator.primary, Palette::INTIMIDATOR_BLACK);

    let petty = CarColorScheme::stock_car_carolina_blue();
    assert_eq!(petty.primary, Palette::CAROLINA_BLUE);

    // Test StockCar visual archetype configs
    let nascar_cup = VehicleVisualType::StockCar {
        tall_wing: false,
        roof_fins: true,
        window_net: true,
    };

    let trans_am_ta1 = VehicleVisualType::StockCar {
        tall_wing: true,
        roof_fins: false,
        window_net: true,
    };

    // Verify pattern matching on variants
    match nascar_cup {
        VehicleVisualType::StockCar { tall_wing, roof_fins, window_net } => {
            assert!(!tall_wing, "NASCAR Cup car uses ducktail blade spoiler");
            assert!(roof_fins, "NASCAR Cup car features roof aerodynamic safety flaps");
            assert!(window_net, "Stock car includes driver window safety net");
        }
        _ => panic!("Expected StockCar visual type"),
    }

    match trans_am_ta1 {
        VehicleVisualType::StockCar { tall_wing, .. } => {
            assert!(tall_wing, "Trans-Am TA1 silhouette uses tall high-mount GT wing");
        }
        _ => panic!("Expected StockCar visual type"),
    }

    // Verify stock car dimensions and physics setup
    let car = Car::new(CarConfig::stock_car_ta1());
    assert!(car.config.top_speed_mps * 3.6 > 310.0);
    assert_eq!(car.config.mass, 1260.0);
}

