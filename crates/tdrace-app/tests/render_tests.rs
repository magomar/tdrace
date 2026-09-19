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

#[test]
fn test_sand_rail_visual_archetype_and_liveries() {
    use tdrace_app::module::{EngineAudioProfile, VehicleVisualType};
    use tdrace_app::audio::EngineSoundType;

    // Verify palette constants
    assert_eq!(Palette::DUNE_ORANGE.a, 1.0);
    assert_eq!(Palette::MOJAVE_TAN.a, 1.0);
    assert_eq!(Palette::BAJA_MINT.a, 1.0);
    assert_eq!(Palette::ACID_YELLOW.a, 1.0);
    assert_eq!(Palette::POLAR_WHITE.a, 1.0);
    assert_eq!(Palette::SEDONA_RED.a, 1.0);
    assert_eq!(Palette::MUD.a, 1.0);
    assert_eq!(Palette::SNOW.a, 1.0);

    // Verify color schemes
    let dune_blaze = CarColorScheme::sand_rail_dune_blaze();
    assert_eq!(dune_blaze.primary, Palette::DUNE_ORANGE);
    assert_eq!(dune_blaze.helmet, Palette::ACID_YELLOW);

    let mojave = CarColorScheme::sand_rail_mojave_sand();
    assert_eq!(mojave.primary, Palette::MOJAVE_TAN);

    let baja = CarColorScheme::sand_rail_baja_mint();
    assert_eq!(baja.primary, Palette::BAJA_MINT);

    let arctic = CarColorScheme::sand_rail_arctic_frost();
    assert_eq!(arctic.primary, Palette::BLUE);

    let red_rock = CarColorScheme::sand_rail_red_rock();
    assert_eq!(red_rock.primary, Palette::SEDONA_RED);

    // Verify SandRail visual archetype
    let sand_rail_stunt = VehicleVisualType::SandRail {
        lightbar: true,
        whip_antenna: true,
        paddle_tires: true,
    };

    match sand_rail_stunt {
        VehicleVisualType::SandRail { lightbar, whip_antenna, paddle_tires } => {
            assert!(lightbar);
            assert!(whip_antenna);
            assert!(paddle_tires);
        }
        _ => panic!("Expected SandRail visual type"),
    }

    // Verify engine audio profile
    let audio = EngineAudioProfile::sand_rail_boxer();
    assert_eq!(audio.sound_type, EngineSoundType::SandRailBoxer);
    assert!(audio.turbo_flutter);
    assert!(audio.anti_lag_pops);
}

#[test]
fn test_scenery_culling_and_grandstand_render_geometry() {
    use tdrace_app::render::scenery::{is_grandstand_in_view, is_tree_in_view};
    use tdrace_core::track::scenery::{Grandstand, GrandstandStyle, Tree, TreeType};

    let stand = Grandstand::new(1, Vec2::new(100.0, 100.0), 40.0, 10.0, 0.0)
        .with_style(GrandstandStyle::CoveredStadium);
    let tree = Tree::new(2, Vec2::new(100.0, 100.0), TreeType::Palm).with_scale(1.0);

    // Viewport containing the elements
    let view_in = Some((Vec2::new(50.0, 50.0), Vec2::new(150.0, 150.0)));
    assert!(is_grandstand_in_view(&stand, view_in));
    assert!(is_tree_in_view(&tree, view_in));

    // Viewport far away
    let view_out = Some((Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0)));
    assert!(!is_grandstand_in_view(&stand, view_out));
    assert!(!is_tree_in_view(&tree, view_out));

    // Corners geometry
    let corners = stand.corners();
    assert_eq!(corners.len(), 4);
    // Front edge should be depth * 0.5 away from center along facing normal
    let center_calc = (corners[0] + corners[1] + corners[2] + corners[3]) * 0.25;
    assert!((center_calc.x - 100.0).abs() < 1e-4);
    assert!((center_calc.y - 100.0).abs() < 1e-4);
}

#[test]
fn test_tree_cenital_canopy_and_alpha_modulation() {
    use tdrace_core::track::scenery::{Tree, TreeType};

    for &tt in &TreeType::ALL {
        let tree = Tree::new(1, Vec2::new(0.0, 0.0), tt);
        assert!(tree.canopy_radius() > 1.0);
        assert!(tree.trunk_radius() > 0.15);

        // When car is underneath canopy, car is detected
        let car_under = Vec2::new(0.5, 0.5);
        assert!(tree.contains_canopy(car_under));

        // When car is outside canopy
        let car_far = Vec2::new(20.0, 20.0);
        assert!(!tree.contains_canopy(car_far));
    }
}

#[test]
fn test_porsche_gt3r_topdown_sprite_asset_presence() {
    let png_bytes = include_bytes!("../../../assets/textures/vehicles/topdown/gt/gt_porsche_911_gt3r.png");
    assert!(!png_bytes.is_empty(), "Topdown sprite PNG asset must not be empty");
    assert_eq!(&png_bytes[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(png_bytes.len() > 100_000, "PNG file should contain high-resolution sprite data");
}

#[test]
fn test_porsche_gt3r_lateral_sprite_asset_presence() {
    let high_res = include_bytes!("../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r.png");
    assert!(!high_res.is_empty(), "Lateral sprite PNG asset must not be empty");
    assert_eq!(&high_res[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(high_res.len() > 50_000, "High-res lateral PNG file should contain detailed sprite data");

    let thumb = include_bytes!("../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r_thumb.png");
    assert!(!thumb.is_empty(), "Thumbnail sprite PNG asset must not be empty");
    assert_eq!(&thumb[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(thumb.len() < high_res.len(), "Thumbnail must be more compact than high-res sprite");
}


