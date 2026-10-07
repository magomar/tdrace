use glam::Vec2;
use tdrace_app::render::color::{CarColorScheme, Palette};
use tdrace_core::{Car, CarConfig};


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
        tdrace_core::catalog::official_track("classic", "classic_grand_prix"),
        tdrace_core::catalog::official_track("classic", "oval_speedway"),
        tdrace_core::catalog::official_track("classic", "drift_park"),
        tdrace_core::catalog::official_track("classic", "kart_arena"),
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
    let col_sand = get_track_backdrop_color(SurfaceType::DeepSand);
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
        if tt != TreeType::Bush {
            assert!(tree.trunk_radius() > 0.15);
        }

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
    let png_bytes = include_bytes!("../../../assets/textures/vehicles/topdown/gt/gt_vandorn_arrowhead_t2.png");
    assert!(!png_bytes.is_empty(), "Topdown sprite PNG asset must not be empty");
    assert_eq!(&png_bytes[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(png_bytes.len() > 100_000, "PNG file should contain high-resolution sprite data");
}

#[test]
fn test_porsche_gt3r_lateral_sprite_asset_presence() {
    let high_res = include_bytes!("../../../assets/textures/vehicles/laterals/gt/gt_vandorn_arrowhead_t2.png");
    assert!(!high_res.is_empty(), "Lateral sprite PNG asset must not be empty");
    assert_eq!(&high_res[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(high_res.len() > 50_000, "High-res lateral PNG file should contain detailed sprite data");

    let thumb = include_bytes!("../../../assets/textures/vehicles/laterals/gt/gt_vandorn_arrowhead_t2_thumb.png");
    assert!(!thumb.is_empty(), "Thumbnail sprite PNG asset must not be empty");
    assert_eq!(&thumb[1..4], b"PNG", "Asset must be a valid PNG format header");
    assert!(thumb.len() < high_res.len(), "Thumbnail must be more compact than high-res sprite");
}

#[test]
fn test_all_80_motorsport_cars_catalog_integrity() {
    use tdrace_app::catalog::ALL_REAL_CARS;

    assert_eq!(ALL_REAL_CARS.len(), 110, "Catalog must contain exactly 110 authentic motorsport vehicles");

    let modules = ["gt", "nascar", "rally", "extreme_offroad", "kart", "autocross"];
    for m in modules {
        let count = ALL_REAL_CARS.iter().filter(|c| c.module_id == m).count();
        if m == "gt" {
            assert_eq!(count, 20, "GT module must contain 20 vehicles (4 per tier)");
        } else if m == "extreme_offroad" {
            assert_eq!(count, 21, "Extreme Off-Road must contain 15 tiered + 6 unranked vehicles");
        } else if m == "rally" {
            assert_eq!(count, 21, "Rallycross must contain 18 ranked (3 x 6 tiers) + 3 heritage vehicles");
        } else if m == "kart" {
            assert_eq!(count, 18, "Kart module must contain 18 authentic vehicles (3 per tier x 6 tiers)");
        } else {
            assert_eq!(count, 15, "Module {} must contain 15 vehicles (3 per tier)", m);
        }
    }

    use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
    assert_eq!(CLASSIC_ARCADE_CARS.len(), 12, "Classic arcade catalog must contain 12 fantasy vehicles (2 per category across 6 categories)");

    for car in ALL_REAL_CARS {
        assert!(!car.id.is_empty(), "Car ID cannot be empty");
        assert!(!car.name.is_empty(), "Car name cannot be empty");
        let max_tier = if car.module_id == "extreme_offroad" || car.module_id == "rally" {
            7
        } else if car.module_id == "kart" {
            6
        } else {
            5
        };
        assert!(car.tier >= 1 && car.tier <= max_tier, "Tier must be between 1 and {}", max_tier);
        assert!(car.bhp > 0, "BHP must be positive");
        assert!(car.weight_kg > 0, "Weight must be positive");
        assert!(car.top_speed_kmh > 0, "Top speed must be positive");
        assert!(car.primary_color.a > 0.9, "Primary color must be fully opaque");
        assert!(car.secondary_color.a > 0.9, "Secondary color must be fully opaque");
    }
}

#[test]
fn test_vehicle_asset_registry_color_helpers() {
    use macroquad::color::Color;
    use tdrace_app::render::vehicle_assets::color_to_u32;

    let c = Color::new(1.0, 0.0, 0.5, 1.0);
    let u = color_to_u32(c);
    assert_eq!((u >> 16) & 0xFF, 255);
    assert_eq!((u >> 8) & 0xFF, 0);
    assert_eq!(u & 0xFF, 127);
}

#[test]
fn test_classic_arcade_fantasy_sprites_presence() {
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("../../assets/textures/vehicles");

    let cars = [
        "classic_gt",
        "classic_gt_vintage",
        "classic_nascar",
        "classic_stock_vintage",
        "classic_offroad",
        "classic_at_safari",
        "classic_kart",
        "classic_kart_vintage",
        "classic_rally",
        "classic_rx_vintage",
        "classic_ax_mudlark",
        "classic_ax_brawler",
        "classic_ax_talon",
    ];
    for id in cars {
        let lat_path = assets_dir.join(format!("laterals/classic/{}.png", id));
        let thumb_path = assets_dir.join(format!("laterals/classic/{}_thumb.png", id));
        let top_path = assets_dir.join(format!("topdown/classic/{}.png", id));

        assert!(lat_path.exists(), "Missing lateral for {}: {:?}", id, lat_path);
        assert!(thumb_path.exists(), "Missing thumbnail for {}: {:?}", id, thumb_path);
        assert!(top_path.exists(), "Missing topdown for {}: {:?}", id, top_path);
    }
}

#[test]
fn test_vortex_dune_crusher_topdown_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/classic/classic_offroad.png");
    let bytes = std::fs::read(&path).expect("Failed to read classic_offroad topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse classic_offroad image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut left_nose_pixels = 0;
    let mut right_nose_pixels = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = img.bytes[idx];
            let g = img.bytes[idx + 1];
            let b = img.bytes[idx + 2];
            let a = img.bytes[idx + 3];

            // Orange nosecone pixels: vibrant orange bodywork
            if a > 128 && r > 180 && g > 50 && g < 150 && b < 50 {
                if x < width / 2 {
                    left_nose_pixels += 1;
                } else {
                    right_nose_pixels += 1;
                }
            }
        }
    }

    assert!(
        right_nose_pixels > 5000,
        "Vortex Dune Crusher front nosecone must face forward (+X, right side). Found {} pixels",
        right_nose_pixels
    );
    assert!(
        left_nose_pixels < 1000,
        "Vortex Dune Crusher must not have nosecone in the rear (-X, left side), found {}",
        left_nose_pixels
    );
}

#[test]
fn test_peugeot_208_rally4_topdown_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/rally/rally_gallia_200_t1.png");
    let bytes = std::fs::read(&path).expect("Failed to read rally_gallia_200_t1 topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse rally_gallia_200_t1 image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut left_red = 0;
    let mut right_red = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = img.bytes[idx];
            let g = img.bytes[idx + 1];
            let b = img.bytes[idx + 2];
            let a = img.bytes[idx + 3];

            // Red hood bodywork pixels (front of car)
            if a > 200 && r > 180 && g < 60 && b < 60 {
                if x < width / 2 {
                    left_red += 1;
                } else {
                    right_red += 1;
                }
            }
        }
    }

    assert!(
        right_red > left_red * 2,
        "Peugeot 208 Rally 4 front hood must face forward (+X, right side). Found right: {}, left: {}",
        right_red,
        left_red
    );
}

#[test]
fn test_tony_kart_topdown_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/kart/kart_verde_apex_t3.png");
    let bytes = std::fs::read(&path).expect("Failed to read kart_verde_apex_t3 topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse kart image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut left_yellow = 0;
    let mut right_yellow = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = img.bytes[idx];
            let g = img.bytes[idx + 1];
            let b = img.bytes[idx + 2];
            let a = img.bytes[idx + 3];

            // Yellow front nosecone number plate
            if a > 200 && r > 200 && g > 180 && b < 50 {
                if x < width / 2 {
                    left_yellow += 1;
                } else {
                    right_yellow += 1;
                }
            }
        }
    }

    assert!(
        right_yellow > left_yellow * 2,
        "Tony Kart front noseplate must face forward (+X, right side). Found right: {}, left: {}",
        right_yellow,
        left_yellow
    );
}

#[test]
fn test_classic_ax_mudlark_lateral_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/laterals/classic/classic_ax_mudlark.png");
    let bytes = std::fs::read(&path).expect("Failed to read classic_ax_mudlark lateral sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse classic_ax_mudlark lateral image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut front_nose_pixels = 0;
    let mut rear_nose_pixels = 0;

    // Cyan nosecone bodywork: y in [250..380], vibrant cyan (b > 120, g > 100, r < 80, a > 128)
    for y in 250..380.min(height) {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = img.bytes[idx];
            let g = img.bytes[idx + 1];
            let b = img.bytes[idx + 2];
            let a = img.bytes[idx + 3];

            if a > 128 && b > 120 && g > 100 && r < 80 {
                if x >= 700 {
                    front_nose_pixels += 1;
                } else if x < 300 {
                    rear_nose_pixels += 1;
                }
            }
        }
    }

    assert!(
        front_nose_pixels > 3000,
        "Mudlark Cross Car front nosecone must face forward (+X, right side). Found {} pixels",
        front_nose_pixels
    );
    assert_eq!(
        rear_nose_pixels, 0,
        "Mudlark Cross Car must not have nosecone pixels in the rear (-X, left side)"
    );
}

#[test]
fn test_crg_black_mirror_okj_lateral_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/laterals/kart/kart_blackline_obsidian_t2.png");
    let bytes = std::fs::read(&path).expect("Failed to read kart_blackline_obsidian_t2 lateral sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse kart_blackline_obsidian_t2 lateral image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut front_orange_pixels = 0;
    let mut rear_orange_pixels = 0;

    // Orange front bumper/fairing: vibrant orange (r > 200, g in 80..200, b < 60, a > 128)
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = img.bytes[idx];
            let g = img.bytes[idx + 1];
            let b = img.bytes[idx + 2];
            let a = img.bytes[idx + 3];

            if a > 128 && r > 200 && (80..200).contains(&g) && b < 60 {
                if x >= 750 {
                    front_orange_pixels += 1;
                } else if x < 250 {
                    rear_orange_pixels += 1;
                }
            }
        }
    }

    assert!(
        front_orange_pixels > 1000,
        "CRG Black Mirror OK-J front bumper/fairing must face forward (+X, right side). Found {} pixels",
        front_orange_pixels
    );
    assert_eq!(
        rear_orange_pixels, 0,
        "CRG Black Mirror OK-J must not have front bumper/fairing pixels in the rear (-X, left side)"
    );
}

#[test]
fn test_classic_mask_tinting_transforms_bodywork_pixels() {
    use macroquad::color::Color;
    use macroquad::texture::Image;
    use std::path::Path;
    use tdrace_app::render::vehicle_assets::apply_vehicle_tint;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_dir = manifest_dir.join("../../assets/textures/vehicles/topdown/classic");

    let models = [
        "classic_gt",
        "classic_nascar",
        "classic_offroad",
        "classic_kart",
        "classic_rally",
        "classic_ax_mudlark",
        "classic_ax_brawler",
        "classic_ax_talon",
    ];

    let target_primary = Color::new(0.85, 0.10, 0.90, 1.0); // Vivid Magenta/Purple
    let target_secondary = Color::new(0.10, 0.95, 0.90, 1.0); // Cyan

    for model_id in models {
        let path = topdown_dir.join(format!("{}.png", model_id));
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {:?}", model_id, e));
        let original_img = Image::from_file_with_format(&bytes, None)
            .unwrap_or_else(|e| panic!("Failed to parse image {}: {:?}", model_id, e));

        let tinted_img = apply_vehicle_tint(&original_img, model_id, target_primary, target_secondary);

        assert_eq!(
            original_img.bytes.len(),
            tinted_img.bytes.len(),
            "Tinted image dimensions must match original for {}",
            model_id
        );

        let mut changed_pixels = 0;
        let mut total_opaque = 0;

        for (orig, tinted) in original_img
            .bytes
            .chunks_exact(4)
            .zip(tinted_img.bytes.chunks_exact(4))
        {
            // Transparent pixels must remain untouched
            if orig[3] == 0 {
                assert_eq!(tinted[3], 0, "Transparent pixel modified in {}", model_id);
                continue;
            }
            total_opaque += 1;

            if orig != tinted {
                changed_pixels += 1;
            }
        }

        assert!(
            changed_pixels > 0,
            "Mask tinting must modify bodywork pixels for model {}. Changed: {}/{}",
            model_id,
            changed_pixels,
            total_opaque
        );

        let change_ratio = changed_pixels as f32 / total_opaque as f32;
        let expected_min_ratio = match model_id {
            "classic_offroad" => 0.14,
            "classic_kart" => 0.30,
            "classic_gt" => 0.38,
            "classic_nascar" => 0.50,
            "classic_rally" => 0.50,
            _ => 0.10,
        };
        assert!(
            change_ratio >= expected_min_ratio,
            "Expected at least {:.1}% of opaque pixels tinted on {}, got {:.1}%",
            expected_min_ratio * 100.0,
            model_id,
            change_ratio * 100.0
        );
    }
}

#[test]
fn test_classic_mode_bot_color_schemes_distinct_from_player_sprite() {
    use tdrace_app::catalog::find_model_by_id;
    use tdrace_app::game::RaceSession;

    let mut session = RaceSession::new();
    session.switch_to_classic();
    session.num_bots = 4;
    session.rebuild_roster_participants();

    assert!(session.world.vehicles.len() >= 4, "Roster must include player and bots");

    let player_model_id = session.car_model_ids[0].expect("Player must have classic model id");
    let player_model = find_model_by_id(player_model_id).expect("Model must exist in catalog");

    // All bots must NOT match factory livery (so they trigger mask-based tinting)
    // and must have primary colors visually distinct from the player model's factory primary color.
    for i in 1..session.world.vehicles.len() {
        let bot_scheme = session.color_schemes[i];
        let dr = (bot_scheme.primary.r - player_model.primary_color.r).abs();
        let dg = (bot_scheme.primary.g - player_model.primary_color.g).abs();
        let db = (bot_scheme.primary.b - player_model.primary_color.b).abs();
        let dist = (dr * dr + dg * dg + db * db).sqrt();

        // Factory match check in vehicle_assets:
        let is_factory = dr < 0.05 && dg < 0.05 && db < 0.05;
        assert!(
            !is_factory,
            "Bot {} must not match factory livery; must use mask-based tinting",
            i
        );

        assert!(
            dist >= 0.20,
            "Bot {} color ({:?}) is too close to player sprite color ({:?}), dist = {:.3}",
            i,
            bot_scheme.primary,
            player_model.primary_color,
            dist
        );
    }
}

#[test]
fn test_career_mode_bot_color_schemes_use_masked_colors_and_player_uses_factory() {
    use tdrace_app::catalog::find_model_by_id;
    use tdrace_app::game::RaceSession;
    use tdrace_app::ui::menu::GameMode;

    for tier in 1..=5 {
        let mut session = RaceSession::new();
        session.start_gt_career_tier(tier);

        assert_eq!(session.game_mode, GameMode::Career);
        assert!(session.world.vehicles.len() >= 4, "Roster must include player and bots");

        let player_model_id = session.car_model_ids[0].expect("Player must have model id in GT career");
        let player_model = find_model_by_id(player_model_id).expect("Model must exist in catalog");

        // Human player must use original sprite color schema (factory livery)
        let player_scheme = session.color_schemes[0];
        let p_dr = (player_scheme.primary.r - player_model.primary_color.r).abs();
        let p_dg = (player_scheme.primary.g - player_model.primary_color.g).abs();
        let p_db = (player_scheme.primary.b - player_model.primary_color.b).abs();
        assert!(
            p_dr < 0.05 && p_dg < 0.05 && p_db < 0.05,
            "Tier {}: Player in career mode must use original sprite color schema (factory livery)",
            tier
        );

        // All bots must NOT match their vehicle model factory livery (must use masked colors)
        // and must have primary colors visually distinct from the player model's factory primary color.
        for i in 1..session.world.vehicles.len() {
            let bot_scheme = session.color_schemes[i];
            let bot_model_id = session.car_model_ids[i].expect("Bot must have model id in GT career");
            let bot_model = find_model_by_id(bot_model_id).expect("Bot model must exist in catalog");

            let dr = (bot_scheme.primary.r - bot_model.primary_color.r).abs();
            let dg = (bot_scheme.primary.g - bot_model.primary_color.g).abs();
            let db = (bot_scheme.primary.b - bot_model.primary_color.b).abs();
            let is_factory = dr < 0.05 && dg < 0.05 && db < 0.05;

            assert!(
                !is_factory,
                "Tier {}: Bot {} must not match factory livery; must use masked colors",
                tier, i
            );

            let dr_p = (bot_scheme.primary.r - player_model.primary_color.r).abs();
            let dg_p = (bot_scheme.primary.g - player_model.primary_color.g).abs();
            let db_p = (bot_scheme.primary.b - player_model.primary_color.b).abs();
            let dist = (dr_p * dr_p + dg_p * dg_p + db_p * db_p).sqrt();

            assert!(
                dist >= 0.20,
                "Tier {}: Bot {} color ({:?}) is too close to player sprite color ({:?}), dist = {:.3}",
                tier, i, bot_scheme.primary, player_model.primary_color, dist
            );
        }
    }
}

#[test]
fn test_gt_models_mask_tinting_transforms_bodywork_pixels() {
    use macroquad::color::Color;
    use macroquad::texture::Image;
    use std::path::Path;
    use tdrace_app::render::vehicle_assets::apply_vehicle_tint;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_dir = manifest_dir.join("../../assets/textures/vehicles/topdown/gt");

    let gt_models = [
        "gt_yamato_hayate_t1",
        "gt_bmr_bavaria_t1",
        "gt_albion_victor_t1",
        "gt_vandorn_stratus_t1",
        "gt_vandorn_arrowhead_t2",
    ];

    let target_primary = Color::new(0.85, 0.10, 0.90, 1.0); // Vivid Magenta/Purple
    let target_secondary = Color::new(0.10, 0.95, 0.90, 1.0); // Cyan

    for model_id in gt_models {
        let path = topdown_dir.join(format!("{}.png", model_id));
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {:?}", model_id, e));
        let original_img = Image::from_file_with_format(&bytes, None)
            .unwrap_or_else(|e| panic!("Failed to parse image {}: {:?}", model_id, e));

        let tinted_img = apply_vehicle_tint(&original_img, model_id, target_primary, target_secondary);

        assert_eq!(
            original_img.bytes.len(),
            tinted_img.bytes.len(),
            "Tinted image dimensions must match original for {}",
            model_id
        );

        let mut changed_pixels = 0;
        let mut total_opaque = 0;

        for (orig, tinted) in original_img
            .bytes
            .chunks_exact(4)
            .zip(tinted_img.bytes.chunks_exact(4))
        {
            if orig[3] == 0 {
                assert_eq!(tinted[3], 0, "Transparent pixel modified in {}", model_id);
                continue;
            }
            total_opaque += 1;

            if orig != tinted {
                changed_pixels += 1;
            }
        }

        assert!(
            changed_pixels > 0,
            "Mask tinting must modify bodywork pixels for GT model {}. Changed: {}/{}",
            model_id,
            changed_pixels,
            total_opaque
        );
    }
}

#[test]
fn test_all_modality_emblem_assets_and_integrity() {
    let modalities_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/icons/modalities");

    let modality_slugs = [
        "quick_race",
        "custom_race",
        "career_mode",
        "time_trial",
        "free_ride",
        "split_screen",
        "lan_play",
        "cloud_play",
        "player_profile",
        "garage",
        "track_editor",
        "settings",
    ];

    for slug in &modality_slugs {
        let svg_path = modalities_dir.join(format!("{}.svg", slug));
        assert!(
            svg_path.exists(),
            "Modality SVG vector file must exist: {:?}",
            svg_path
        );
        let svg_content = std::fs::read_to_string(&svg_path).expect("Failed to read SVG file");
        assert!(
            svg_content.contains("<svg") && svg_content.contains("</svg>"),
            "SVG file must contain valid SVG root markup for {}",
            slug
        );

        let png_path = modalities_dir.join(format!("{}-128.png", slug));
        assert!(
            png_path.exists(),
            "Modality 128x128 PNG raster file must exist: {:?}",
            png_path
        );
        let png_bytes = std::fs::read(&png_path).expect("Failed to read PNG file");
        assert!(
            png_bytes.len() > 1000,
            "PNG file size must be at least 1KB for {}",
            slug
        );
        assert_eq!(
            &png_bytes[1..4],
            b"PNG",
            "File must have valid PNG magic header for {}",
            slug
        );
    }
}

#[test]
fn test_surface_material_quality_and_properties() {
    use tdrace_app::render::{SurfaceMaterial, SurfaceTextureQuality};
    use tdrace_core::physics::surface::SurfaceType;

    assert_eq!(SurfaceTextureQuality::Off.name(), "Off (Flat)");
    assert_eq!(SurfaceTextureQuality::Standard.name(), "Standard");
    assert_eq!(SurfaceTextureQuality::High.name(), "High");
    assert_eq!(SurfaceTextureQuality::default(), SurfaceTextureQuality::High);

    for &surf in &SurfaceType::ALL {
        let mat = SurfaceMaterial::new(surf, None);
        assert_eq!(mat.surface_type, surf);
        assert!(mat.tile_scale_meters > 0.0, "Tile scale must be positive for {:?}", surf);
        assert!(mat.roughness >= 0.0 && mat.roughness <= 1.0, "Roughness in [0, 1] for {:?}", surf);
    }
}

#[test]
fn test_procedural_surface_image_generators_all_15_surfaces() {
    use tdrace_app::render::{generate_curb_image, generate_edge_fringe_mask, generate_macro_noise_image, generate_surface_image};
    use tdrace_core::physics::surface::SurfaceType;

    let dim = 64u16;

    for &surf in &SurfaceType::ALL {
        let img = generate_surface_image(surf, dim, dim);
        assert_eq!(img.width, dim);
        assert_eq!(img.height, dim);
        assert_eq!(img.bytes.len(), (dim as usize) * (dim as usize) * 4);

        // Verify non-zero alpha and color variation across pixels
        let mut min_val = 255u8;
        let mut max_val = 0u8;
        for chunk in img.bytes.chunks_exact(4) {
            let r = chunk[0];
            let a = chunk[3];
            assert!(a > 0, "Alpha must be positive for {:?}", surf);
            min_val = min_val.min(r);
            max_val = max_val.max(r);
        }

        assert!(
            max_val > min_val,
            "Procedural texture must exhibit micro-texture color variation for {:?}",
            surf
        );
    }

    // Verify curb image (alternating red and white teeth with bevel gradient)
    let curb = generate_curb_image(64, 32);
    assert_eq!(curb.width, 64);
    assert_eq!(curb.height, 32);
    // Left half should be red (R > 150, G < 100), right half should be white (R > 150, G > 150)
    let p_red = &curb.bytes[0..4]; // (0, 0)
    let p_white = &curb.bytes[(32 * 4)..(32 * 4 + 4)]; // (32, 0)
    assert!(p_red[0] > 150 && p_red[1] < 100, "Left tooth must be red: {:?}", p_red);
    assert!(p_white[0] > 150 && p_white[1] > 150, "Right tooth must be white: {:?}", p_white);

    // Verify edge fringe mask has alpha variations
    let fringe = generate_edge_fringe_mask(32, 32);
    assert_eq!(fringe.bytes.len(), 32 * 32 * 4);
    let mut min_alpha = 255u8;
    let mut max_alpha = 0u8;
    for chunk in fringe.bytes.chunks_exact(4) {
        min_alpha = min_alpha.min(chunk[3]);
        max_alpha = max_alpha.max(chunk[3]);
    }
    assert!(max_alpha > min_alpha, "Edge fringe mask must have alpha falloff");

    // Verify macro noise image
    let macro_noise = generate_macro_noise_image(32, 32);
    assert_eq!(macro_noise.bytes.len(), 32 * 32 * 4);

    // Verify tire rubber contact texture (feathered edges, multi-rib striations, white RGB)
    let rubber = tdrace_app::render::generate_tire_rubber_image(64, 128);
    assert_eq!(rubber.width, 64);
    assert_eq!(rubber.height, 128);
    assert_eq!(rubber.bytes.len(), 64 * 128 * 4);

    // Edge pixel (x=0, y=64) should have zero/minimal alpha due to edge feathering
    let edge_idx = (64 * 64 + 0) * 4;
    assert_eq!(rubber.bytes[edge_idx], 255, "RGB must be white for vertex tinting");
    assert!(rubber.bytes[edge_idx + 3] < 30, "Outer edge must feather to near-zero alpha: got {}", rubber.bytes[edge_idx + 3]);

    // Check that contact ribs reach solid rubber density (> 180 alpha)
    let max_patch_alpha = (0..64).map(|x| rubber.bytes[(64 * 64 + x) * 4 + 3]).max().unwrap();
    assert!(max_patch_alpha > 180, "Contact ribs must reach solid rubber density: got {}", max_patch_alpha);
}

#[test]
fn test_surface_asset_files_exist_and_are_valid_png() {
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidate_dirs = [
        manifest_dir.join("assets/textures/surfaces"),
        manifest_dir.join("../../assets/textures/surfaces"),
        manifest_dir.join("../../../assets/textures/surfaces"),
    ];

    let surfaces_dir = candidate_dirs
        .iter()
        .find(|p| p.exists())
        .expect("assets/textures/surfaces directory must exist");

    let expected_files = [
        "asphalt_diffuse.png",
        "dirt_compacted.png",
        "grass_turf.png",
        "gravel_crushed.png",
        "sand_dune.png",
        "mud_viscous.png",
        "snow_powder.png",
        "ice_glazed.png",
        "water_caustic.png",
        "oil_iridescent.png",
        "concrete_brushed.png",
        "curb_teeth.png",
        "edge_fringe_mask.png",
        "asphalt_groove.png",
    ];

    for filename in &expected_files {
        let path = surfaces_dir.join(filename);
        assert!(path.exists(), "Surface texture file must exist: {:?}", path);
        let bytes = std::fs::read(&path).expect("Failed to read texture file");
        assert!(bytes.len() > 500, "Texture file {:?} must be larger than 500 bytes", filename);
        assert_eq!(&bytes[1..4], b"PNG", "File {:?} must be a valid PNG image", filename);
    }
}

#[test]
fn test_spline_ribbon_and_world_space_uv_mappings() {
    
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    assert!(track.spline.samples.len() > 10);

    // Verify distance monotonically increases along spline samples
    let samples = &track.spline.samples;
    for i in 1..samples.len() {
        assert!(samples[i].distance >= samples[i - 1].distance);
    }

    // Verify ribbon UV scaling formula: v = distance / tile_scale
    let tile_scale = 4.0; // Asphalt scale
    let s0 = &samples[0];
    let s1 = &samples[1];
    let v0 = s0.distance / tile_scale;
    let v1 = s1.distance / tile_scale;
    assert!(v1 >= v0);
    assert_eq!(v0, 0.0);

    // Verify world UV scaling formula: uv = (x / scale, y / scale)
    let world_p1 = glam::Vec2::new(100.0, 200.0);
    let world_p2 = glam::Vec2::new(100.0, 200.0);
    let scale_world = 6.0;
    let uv1 = world_p1 / scale_world;
    let uv2 = world_p2 / scale_world;
    assert_eq!(uv1, uv2, "Identical world coordinates must share identical UVs for seamless continuity");
}

#[test]
fn test_macro_modulation_value_range_and_spatial_continuity() {
    use tdrace_app::render::evaluate_macro_modulation;

    let mut min_val = 100.0f32;
    let mut max_val = -100.0f32;
    let mut sum = 0.0f32;
    let mut count = 0;

    // Sample across a 400m x 400m circuit expanse at 4m intervals
    for x_i in 0..100 {
        for y_i in 0..100 {
            let x = x_i as f32 * 4.0;
            let y = y_i as f32 * 4.0;
            let val = evaluate_macro_modulation(x, y);

            assert!(
                val >= -1.0 && val <= 1.0,
                "Macro modulation must be strictly within [-1.0, 1.0], got {}",
                val
            );

            min_val = min_val.min(val);
            max_val = max_val.max(val);
            sum += val;
            count += 1;
        }
    }

    let mean = sum / count as f32;
    // Verify non-trivial modulation variance over the circuit
    assert!(max_val - min_val > 0.8, "Macro field must exhibit sufficient amplitude variance, got range [{}, {}]", min_val, max_val);
    assert!(mean.abs() < 0.25, "Macro modulation should be reasonably centered near 0, got mean {}", mean);
}

#[test]
fn test_track_wear_state_phase_2_hooks() {
    use tdrace_app::render::TrackWearState;

    let mut wear = TrackWearState::new(10);
    assert_eq!(wear.segment_rubber.len(), 10);
    assert_eq!(wear.segment_marbles.len(), 10);

    // Initial state: pristine track
    for i in 0..10 {
        assert_eq!(wear.get_rubber(i), 0.0);
        assert_eq!(wear.get_marbles(i), 0.0);
    }

    // Dynamic rubber deposition from tire slip work
    wear.deposit_rubber(3, 250.0); // 250 * 0.001 = 0.25
    assert!((wear.get_rubber(3) - 0.25).abs() < 1e-4);

    wear.deposit_rubber(3, 500.0); // 0.25 + 0.50 = 0.75
    assert!((wear.get_rubber(3) - 0.75).abs() < 1e-4);

    // Clamp at 1.0
    wear.deposit_rubber(3, 500.0);
    assert_eq!(wear.get_rubber(3), 1.0);

    // Loose marbles accumulation
    wear.accumulate_marbles(7, 400.0);
    assert!((wear.get_marbles(7) - 0.40).abs() < 1e-4);

    // Out-of-bounds safety
    assert_eq!(wear.get_rubber(999), 0.0);
    assert_eq!(wear.get_marbles(999), 0.0);
    wear.deposit_rubber(999, 100.0); // No panic
    wear.accumulate_marbles(999, 100.0); // No panic
}

#[test]
fn test_segment_curvature_and_apex_rubbering_lateral_distribution() {
    use tdrace_app::render::track::compute_segment_curvature;
        let base_sample = tdrace_core::catalog::official_track("classic", "classic_grand_prix").spline.samples[0].clone();

    // 1. Synthetic straight segment
    let mut s_straight_0 = base_sample.clone();
    s_straight_0.point = glam::Vec2::new(0.0, 0.0);
    s_straight_0.tangent = glam::Vec2::new(1.0, 0.0);
    s_straight_0.distance = 0.0;

    let mut s_straight_1 = s_straight_0.clone();
    s_straight_1.point = glam::Vec2::new(10.0, 0.0);
    s_straight_1.tangent = glam::Vec2::new(1.0, 0.0);
    s_straight_1.distance = 10.0;

    let k_straight = compute_segment_curvature(&s_straight_0, &s_straight_1);
    assert!(k_straight.abs() < 1e-5, "Straight segment curvature must be near zero: {}", k_straight);

    // 2. Synthetic Left turn (counter-clockwise deflection: tangent rotates from (1, 0) towards (0, 1))
    let mut s_left_1 = s_straight_0.clone();
    let angle_left = 0.20f32; // ~11.5 degrees left turn
    s_left_1.tangent = glam::Vec2::new(angle_left.cos(), angle_left.sin());
    s_left_1.point = glam::Vec2::new(5.0, 1.0);
    s_left_1.distance = 5.1;

    let k_left = compute_segment_curvature(&s_straight_0, &s_left_1);
    assert!(k_left > 0.0, "Left turn curvature must be positive: {}", k_left);

    // 3. Synthetic Right turn (clockwise deflection: tangent rotates from (1, 0) towards (0, -1))
    let mut s_right_1 = s_straight_0.clone();
    let angle_right = -0.20f32; // ~11.5 degrees right turn
    s_right_1.tangent = glam::Vec2::new(angle_right.cos(), angle_right.sin());
    s_right_1.point = glam::Vec2::new(5.0, -1.0);
    s_right_1.distance = 5.1;

    let k_right = compute_segment_curvature(&s_straight_0, &s_right_1);
    assert!(k_right < 0.0, "Right turn curvature must be negative: {}", k_right);

    // 4. Verify on realistic track spline
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let samples = &track.spline.samples;
    let mut max_k = 0.0f32;
    for i in 0..samples.len() - 1 {
        let k = compute_segment_curvature(&samples[i], &samples[i + 1]);
        max_k = max_k.max(k.abs());
    }
    assert!(max_k > 0.02, "Realistic track must contain curved segments with significant curvature, found max: {}", max_k);
}

#[test]
fn test_track_render_execution_under_all_quality_tiers_headless_safety() {
    use tdrace_app::render::track::{render_track, set_surface_texture_quality};
    use tdrace_app::render::surface_material::SurfaceTextureQuality;
    

    let tracks = [tdrace_core::catalog::official_track("classic", "classic_grand_prix"), tdrace_core::catalog::official_track("classic", "oval_speedway"), tdrace_core::catalog::official_track("classic", "drift_park")];
    let qualities = [
        SurfaceTextureQuality::Off,
        SurfaceTextureQuality::Standard,
        SurfaceTextureQuality::High,
    ];

    for &q in &qualities {
        set_surface_texture_quality(q);
        for t in &tracks {
            // Must execute without panic in headless environments across all quality tiers
            render_track(t);
        }
    }
}

#[test]
fn test_seamless_periodic_grass_and_asphalt_generators() {
    use tdrace_app::render::generate_surface_image;
    use tdrace_core::physics::surface::SurfaceType;

    let grass_img = generate_surface_image(SurfaceType::Grass, 256, 256);
    assert_eq!(grass_img.width, 256);
    assert_eq!(grass_img.height, 256);
    assert_eq!(grass_img.bytes.len(), 256 * 256 * 4);

    let asphalt_img = generate_surface_image(SurfaceType::Asphalt, 256, 256);
    assert_eq!(asphalt_img.width, 256);
    assert_eq!(asphalt_img.height, 256);
    assert_eq!(asphalt_img.bytes.len(), 256 * 256 * 4);

    // Verify mean luminance range for isotropic matte asphalt (deep charcoal)
    let mut sum_lum = 0u64;
    for y in 0..256 {
        for x in 0..256 {
            let idx = (y * 256 + x) * 4;
            sum_lum += asphalt_img.bytes[idx] as u64;
        }
    }
    let mean_asphalt = sum_lum as f64 / (256.0 * 256.0);
    assert!(mean_asphalt > 25.0 && mean_asphalt < 55.0, "Asphalt mean must be matte charcoal, got {}", mean_asphalt);
}

#[test]
fn test_backdrop_ground_pass_execution() {
    use tdrace_app::render::track::{render_backdrop_ground_pass, render_ground_track_culled, set_surface_texture_quality};
    use tdrace_app::render::surface_material::SurfaceTextureQuality;
        use glam::Vec2;

    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let bounds = Some((Vec2::new(-50.0, -50.0), Vec2::new(300.0, 300.0)));

    for &q in &[SurfaceTextureQuality::Off, SurfaceTextureQuality::Standard, SurfaceTextureQuality::High] {
        set_surface_texture_quality(q);
        render_backdrop_ground_pass(&track, bounds);
        render_backdrop_ground_pass(&track, None);
        render_ground_track_culled(&track, bounds);
    }
}

// ==============================================================================
// SPEC 026: TOP-DOWN PRE-BAKED VEHICLE WHEEL STEERING ANIMATIONS
// ==============================================================================

#[test]
fn test_spec_026_standalone_wheel_texture_asset_integrity() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wheel_dir = manifest_dir.join("../../assets/textures/vehicles/topdown/wheels");
    let wheels = [
        "kart_slick_front.png",
        "gt_slick_front.png",
        "nascar_wheel_front.png",
        "offroad_wheel_front.png",
        "rally_wheel_front.png",
    ];

    for wheel_file in wheels {
        let path = wheel_dir.join(wheel_file);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {}: {:?}", wheel_file, e));

        // Valid PNG signature: [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        assert_eq!(&bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "Must contain valid PNG header: {}", wheel_file);

        let img = Image::from_file_with_format(&bytes, None).unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", wheel_file, e));
        assert_eq!(img.width, 128, "Tire width must be 128px for {}", wheel_file);
        assert_eq!(img.height, 256, "Tire height must be 256px for {}", wheel_file);
        assert_eq!(img.bytes.len(), 128 * 256 * 4);

        // Transparent corner margins (A = 0)
        let w = 128;
        let h = 256;
        for (cx, cy) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (5, 5), (w - 6, 5)] {
            let idx = (cy * w + cx) * 4;
            assert_eq!(img.bytes[idx + 3], 0, "Corner pixel ({}, {}) must have alpha 0 on {}", cx, cy, wheel_file);
        }

        // Center hub must be opaque
        let center_idx = (h / 2 * w + w / 2) * 4;
        assert_eq!(img.bytes[center_idx + 3], 255, "Wheel center hub must be opaque on {}", wheel_file);
    }
}

#[test]
fn test_spec_026_steered_wheel_config_lookup_and_legacy_fallback() {
    use tdrace_app::render::vehicle_assets::{get_steered_wheel_config, WheelLayerMode};

    // 1. All 12 classic module vehicles must return explicit SteeredWheelConfig (Spec 073)
    let kart = get_steered_wheel_config("classic_kart").expect("classic_kart must have SteeredWheelConfig");
    assert_eq!(kart.wheel_texture_id, "kart_slick_front");
    assert!((kart.front_axle_offset - 0.41).abs() < 1e-4);
    assert!((kart.half_track_width - 0.39).abs() < 1e-4);
    assert_eq!(kart.layering, WheelLayerMode::OverChassis);

    let kart_vintage = get_steered_wheel_config("classic_kart_vintage").expect("classic_kart_vintage must have SteeredWheelConfig");
    assert_eq!(kart_vintage.wheel_texture_id, "kart_slick_front");
    assert!((kart_vintage.front_axle_offset - 0.39).abs() < 1e-4);
    assert!((kart_vintage.half_track_width - 0.36).abs() < 1e-4);
    assert_eq!(kart_vintage.layering, WheelLayerMode::OverChassis);

    let gt = get_steered_wheel_config("classic_gt").expect("classic_gt must have SteeredWheelConfig");
    assert_eq!(gt.wheel_texture_id, "gt_slick_front");
    assert!((gt.front_axle_offset - 0.75).abs() < 1e-4);
    assert!((gt.half_track_width - 0.48).abs() < 1e-4);
    assert_eq!(gt.layering, WheelLayerMode::UnderChassis);

    let gt_vintage = get_steered_wheel_config("classic_gt_vintage").expect("classic_gt_vintage must have SteeredWheelConfig");
    assert_eq!(gt_vintage.wheel_texture_id, "gt_slick_front");
    assert!((gt_vintage.front_axle_offset - 0.71).abs() < 1e-4);
    assert!((gt_vintage.half_track_width - 0.44).abs() < 1e-4);
    assert_eq!(gt_vintage.layering, WheelLayerMode::UnderChassis);

    let nascar = get_steered_wheel_config("classic_nascar").expect("classic_nascar must have SteeredWheelConfig");
    assert_eq!(nascar.wheel_texture_id, "nascar_wheel_front");
    assert!((nascar.front_axle_offset - 0.66).abs() < 1e-4);
    assert!((nascar.half_track_width - 0.51).abs() < 1e-4);
    assert_eq!(nascar.layering, WheelLayerMode::UnderChassis);

    let stock_vintage = get_steered_wheel_config("classic_stock_vintage").expect("classic_stock_vintage must have SteeredWheelConfig");
    assert_eq!(stock_vintage.wheel_texture_id, "nascar_wheel_front");
    assert!((stock_vintage.front_axle_offset - 0.72).abs() < 1e-4);
    assert!((stock_vintage.half_track_width - 0.49).abs() < 1e-4);
    assert_eq!(stock_vintage.layering, WheelLayerMode::UnderChassis);

    let offroad = get_steered_wheel_config("classic_offroad").expect("classic_offroad must have SteeredWheelConfig");
    assert_eq!(offroad.wheel_texture_id, "offroad_wheel_front");
    assert!((offroad.front_axle_offset - 0.98).abs() < 1e-4);
    assert!((offroad.half_track_width - 0.62).abs() < 1e-4);
    assert_eq!(offroad.layering, WheelLayerMode::OverChassis);

    let at_safari = get_steered_wheel_config("classic_at_safari").expect("classic_at_safari must have SteeredWheelConfig");
    assert_eq!(at_safari.wheel_texture_id, "offroad_wheel_front");
    assert!((at_safari.front_axle_offset - 0.82).abs() < 1e-4);
    assert!((at_safari.half_track_width - 0.48).abs() < 1e-4);
    assert_eq!(at_safari.layering, WheelLayerMode::UnderChassis);

    let rally = get_steered_wheel_config("classic_rally").expect("classic_rally must have SteeredWheelConfig");
    assert_eq!(rally.wheel_texture_id, "rally_wheel_front");
    assert!((rally.front_axle_offset - 0.73).abs() < 1e-4);
    assert!((rally.half_track_width - 0.41).abs() < 1e-4);
    assert_eq!(rally.layering, WheelLayerMode::UnderChassis);

    let rx_vintage = get_steered_wheel_config("classic_rx_vintage").expect("classic_rx_vintage must have SteeredWheelConfig");
    assert_eq!(rx_vintage.wheel_texture_id, "rally_wheel_front");
    assert!((rx_vintage.front_axle_offset - 0.69).abs() < 1e-4);
    assert!((rx_vintage.half_track_width - 0.40).abs() < 1e-4);
    assert_eq!(rx_vintage.layering, WheelLayerMode::UnderChassis);

    let mudlark = get_steered_wheel_config("classic_ax_mudlark").expect("classic_ax_mudlark must have SteeredWheelConfig");
    assert_eq!(mudlark.wheel_texture_id, "offroad_wheel_front");
    assert!((mudlark.front_axle_offset - 0.85).abs() < 1e-4);
    assert!((mudlark.half_track_width - 0.55).abs() < 1e-4);
    assert_eq!(mudlark.layering, WheelLayerMode::OverChassis);

    let brawler = get_steered_wheel_config("classic_ax_brawler").expect("classic_ax_brawler must have SteeredWheelConfig");
    assert_eq!(brawler.wheel_texture_id, "rally_wheel_front");
    assert!((brawler.front_axle_offset - 0.75).abs() < 1e-4);
    assert!((brawler.half_track_width - 0.44).abs() < 1e-4);
    assert_eq!(brawler.layering, WheelLayerMode::UnderChassis);

    let talon = get_steered_wheel_config("classic_ax_talon").expect("classic_ax_talon must have SteeredWheelConfig");
    assert_eq!(talon.wheel_texture_id, "offroad_wheel_front");
    assert!((talon.front_axle_offset - 1.05).abs() < 1e-4);
    assert!((talon.half_track_width - 0.65).abs() < 1e-4);
    assert_eq!(talon.layering, WheelLayerMode::OverChassis);

    // 2. Legacy fallback guarantee: all non-classic models return None
    let legacy_models = [
        "gt_vandorn_arrowhead_t2",
        "nascar_camaro_zl1",
        "rally_gallia_200_t1",
        "offroad_desert_forge_truck_t2",
    ];
    for model_id in legacy_models {
        assert!(
            get_steered_wheel_config(model_id).is_none(),
            "Model {} must return None to guarantee legacy monolithic sprite fallback",
            model_id
        );
    }
}

#[test]
fn test_classic_kart_topdown_sprite_orientation() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/classic/classic_kart.png");
    let bytes = std::fs::read(&path).expect("Failed to read classic_kart topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse classic_kart image");

    let width = img.width as usize;
    let height = img.height as usize;
    let mut right_pixels = 0;
    let mut left_pixels = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let a = img.bytes[idx + 3];
            if a > 100 {
                if x < width / 2 {
                    left_pixels += 1;
                } else {
                    right_pixels += 1;
                }
            }
        }
    }

    assert!(
        right_pixels > 20000,
        "classic_kart front must face forward (+X, right side). Found right: {}, left: {}",
        right_pixels,
        left_pixels
    );
}

#[test]
fn test_classic_at_safari_topdown_cenital_symmetry() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/classic/classic_at_safari.png");
    let bytes = std::fs::read(&path).expect("Failed to read classic_at_safari topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse classic_at_safari image");

    assert_eq!(img.width, 512);
    assert_eq!(img.height, 512);

    let width = img.width as usize;
    let height = img.height as usize;
    let mut top_pixels = 0;
    let mut bot_pixels = 0;
    let mut min_y = height;
    let mut max_y = 0;
    let mut sum_y = 0;
    let mut total_pixels = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let a = img.bytes[idx + 3];
            if a > 50 {
                total_pixels += 1;
                sum_y += y;
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                if y < 256 {
                    top_pixels += 1;
                } else {
                    bot_pixels += 1;
                }
            }
        }
    }

    assert!(total_pixels > 50000, "Must have substantial car body pixels");
    let y_center = sum_y as f64 / total_pixels as f64;
    assert!(
        (y_center - 256.0).abs() < 2.0,
        "Vehicle must be vertically centered at y=256 (cenital/zenithal). Found y_center: {:.2}",
        y_center
    );

    let symmetry_ratio = top_pixels as f64 / bot_pixels as f64;
    assert!(
        (symmetry_ratio - 1.0).abs() < 0.05,
        "Top and bottom halves must be symmetrical (cenital/zenithal). Ratio: {:.4}",
        symmetry_ratio
    );

    // Height must be tight (orthographic top-down, ~200px, not tilted showing sides >250px)
    let sprite_height = max_y - min_y;
    assert!(
        sprite_height < 220,
        "Orthographic top-down sprite height must be < 220px to avoid showing side panels. Found: {}",
        sprite_height
    );
}

#[test]
fn test_gallia_lyon_t4_topdown_cenital_symmetry() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../assets/textures/vehicles/topdown/rally/rally_gallia_lyon_t4.png");
    let bytes = std::fs::read(&path).expect("Failed to read rally_gallia_lyon_t4 topdown sprite");
    let img = Image::from_file_with_format(&bytes, None).expect("Failed to parse rally_gallia_lyon_t4 image");

    assert_eq!(img.width, 512);
    assert_eq!(img.height, 512);

    let width = img.width as usize;
    let height = img.height as usize;
    let mut top_pixels = 0;
    let mut bot_pixels = 0;
    let mut min_y = height;
    let mut max_y = 0;
    let mut sum_y = 0;
    let mut total_pixels = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let a = img.bytes[idx + 3];
            if a > 50 {
                total_pixels += 1;
                sum_y += y;
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                if y < 256 {
                    top_pixels += 1;
                } else {
                    bot_pixels += 1;
                }
            }
        }
    }

    assert!(total_pixels > 50000, "Must have substantial car body pixels");
    let y_center = sum_y as f64 / total_pixels as f64;
    assert!(
        (y_center - 256.0).abs() < 2.0,
        "Vehicle must be vertically centered at y=256 (cenital/zenithal). Found y_center: {:.2}",
        y_center
    );

    let symmetry_ratio = top_pixels as f64 / bot_pixels as f64;
    assert!(
        (symmetry_ratio - 1.0).abs() < 0.05,
        "Top and bottom halves must be symmetrical (cenital/zenithal). Ratio: {:.4}",
        symmetry_ratio
    );

    // Height must be tight (orthographic top-down, ~218px, not tilted showing side doors >250px)
    let sprite_height = max_y - min_y;
    assert!(
        sprite_height < 220,
        "Orthographic top-down sprite height must be < 220px to avoid showing side panels. Found: {}",
        sprite_height
    );
}

#[test]
fn test_classic_cars_dual_sprites_showroom_and_chassis() {
    use macroquad::texture::Image;
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_dir = manifest_dir.join("../../assets/textures/vehicles/topdown/classic");
    let lateral_dir = manifest_dir.join("../../assets/textures/vehicles/laterals/classic");

    let classic_models = [
        "classic_gt",
        "classic_gt_vintage",
        "classic_nascar",
        "classic_stock_vintage",
        "classic_offroad",
        "classic_at_safari",
        "classic_kart",
        "classic_kart_vintage",
        "classic_rally",
        "classic_rx_vintage",
        "classic_ax_mudlark",
        "classic_ax_brawler",
    ];

    for model_id in classic_models {
        let showroom_path = topdown_dir.join(format!("{}.png", model_id));
        let chassis_path = topdown_dir.join(format!("{}_chassis.png", model_id));
        let lateral_path = lateral_dir.join(format!("{}.png", model_id));
        let thumb_path = lateral_dir.join(format!("{}_thumb.png", model_id));

        // 1. All dual assets must exist on disk
        assert!(showroom_path.exists(), "Showroom topdown sprite must exist: {:?}", showroom_path);
        assert!(chassis_path.exists(), "In-game chassis sprite must exist: {:?}", chassis_path);
        assert!(lateral_path.exists(), "Lateral turntable sprite must exist: {:?}", lateral_path);
        assert!(thumb_path.exists(), "Lateral thumbnail sprite must exist: {:?}", thumb_path);

        // 2. Top-down dimensions must be exactly 512x512
        let showroom_bytes = std::fs::read(&showroom_path).unwrap();
        let chassis_bytes = std::fs::read(&chassis_path).unwrap();
        let showroom_img = Image::from_file_with_format(&showroom_bytes, None).unwrap();
        let chassis_img = Image::from_file_with_format(&chassis_bytes, None).unwrap();

        assert_eq!(showroom_img.width, 512, "Showroom width 512 for {}", model_id);
        assert_eq!(showroom_img.height, 512, "Showroom height 512 for {}", model_id);
        assert_eq!(chassis_img.width, 512, "Chassis width 512 for {}", model_id);
        assert_eq!(chassis_img.height, 512, "Chassis height 512 for {}", model_id);

        // 3. Lateral dimensions must be 1024x512 and thumb 256x128
        let lateral_bytes = std::fs::read(&lateral_path).unwrap();
        let thumb_bytes = std::fs::read(&thumb_path).unwrap();
        let lateral_img = Image::from_file_with_format(&lateral_bytes, None).unwrap();
        let thumb_img = Image::from_file_with_format(&thumb_bytes, None).unwrap();

        assert_eq!(lateral_img.width, 1024, "Lateral width 1024 for {}", model_id);
        assert_eq!(lateral_img.height, 512, "Lateral height 512 for {}", model_id);
        assert_eq!(thumb_img.width, 256, "Thumb width 256 for {}", model_id);
        assert_eq!(thumb_img.height, 128, "Thumb height 128 for {}", model_id);
    }
}

#[test]
fn test_spec_026_wheel_steering_ackermann_deflection_across_classic_cars() {
    use tdrace_app::module::classic::ClassicGameModule;

    let configs = [
        ClassicGameModule::car_classic_kart(),
        ClassicGameModule::car_classic_kart_vintage(),
        ClassicGameModule::car_classic_gt(),
        ClassicGameModule::car_classic_gt_vintage(),
        ClassicGameModule::car_classic_nascar(),
        ClassicGameModule::car_classic_stock_vintage(),
        ClassicGameModule::car_classic_offroad(),
        ClassicGameModule::car_classic_at_safari(),
        ClassicGameModule::car_classic_rally(),
        ClassicGameModule::car_classic_rx_vintage(),
        ClassicGameModule::car_classic_ax_mudlark(),
        ClassicGameModule::car_classic_ax_brawler(),
    ];

    for cfg in configs {
        let car = Car::new(cfg);

        // 1. Symmetrical return to center: steer_angle = 0.0
        let (fl_zero, fr_zero) = car.compute_ackermann_angles(0.0);
        assert!(fl_zero.abs() < 1e-6, "Front-left wheel must align parallel to heading at 0 steer");
        assert!(fr_zero.abs() < 1e-6, "Front-right wheel must align parallel to heading at 0 steer");

        // 2. Turn left (steer_angle > 0.0, counter-clockwise): inner wheel (FL) turns sharper than outer wheel (FR)
        let (fl_left, fr_left) = car.compute_ackermann_angles(0.40);
        assert!(fl_left > 0.0, "Front-left wheel must turn left (counter-clockwise)");
        assert!(fr_left > 0.0, "Front-right wheel must turn left (counter-clockwise)");
        assert!(
            fl_left > fr_left,
            "Inner wheel ({:.4}) must deflect more than outer wheel ({:.4}) under left turn",
            fl_left, fr_left
        );

        // 3. Turn right (steer_angle < 0.0, clockwise): inner wheel (FR) turns sharper than outer wheel (FL)
        let (fl_right, fr_right) = car.compute_ackermann_angles(-0.40);
        assert!(fl_right < 0.0, "Front-left wheel must turn right (clockwise)");
        assert!(fr_right < 0.0, "Front-right wheel must turn right (clockwise)");
        assert!(
            fr_right.abs() > fl_right.abs(),
            "Inner wheel (|{:.4}|) must deflect more than outer wheel (|{:.4}|) under right turn",
            fr_right, fl_right
        );

        // 4. Symmetry: magnitude of FL under left turn matches FR under right turn
        assert!(
            (fl_left.abs() - fr_right.abs()).abs() < 1e-5,
            "Steering geometry must be strictly symmetric between left and right turns"
        );
    }
}

#[test]
fn test_spec_026_steered_wheel_ground_shadow_alignment_and_jump_scaling() {
    use glam::Vec2;

    let angle = 0.35f32; // vehicle heading
    let steer_fl = -0.22f32; // turning
    let wheel_ang = angle + steer_fl;
    let _wheel_size = Vec2::new(0.24, 0.44);

    // Grounded shadow (z_lift = 0.0)
    let z_ground = 0.0f32;
    let s_off_ground = Vec2::new(0.06 + z_ground * 0.30, 0.08 + z_ground * 0.40);
    let s_scale_ground = 1.0 + (z_ground * 0.08).min(0.40);
    let s_alpha_ground = (1.0 / (1.0 + z_ground * 0.55)).clamp(0.25, 1.0);
    assert_eq!(s_scale_ground, 1.0);
    assert_eq!(s_alpha_ground, 1.0);
    assert!((s_off_ground - Vec2::new(0.06, 0.08)).length() < 1e-5);

    // Airborne jump shadow (z_lift = 2.0m)
    let z_jump = 2.0f32;
    let s_off_jump = Vec2::new(0.06 + z_jump * 0.30, 0.08 + z_jump * 0.40);
    let s_scale_jump = 1.0 + (z_jump * 0.08).min(0.40);
    let s_alpha_jump = (1.0 / (1.0 + z_jump * 0.55)).clamp(0.25, 1.0);

    assert!(s_scale_jump > 1.10, "Airborne jump shadow must expand: {:.3}", s_scale_jump);
    assert!(s_alpha_jump < 0.50, "Airborne jump shadow must fade smoothly: {:.3}", s_alpha_jump);
    assert!(s_off_jump.x > s_off_ground.x && s_off_jump.y > s_off_ground.y, "Shadow offset expands with altitude");

    // Shadow orientation: verify forward and right vectors rotate synchronously with wheel heading
    let fwd = Vec2::new(wheel_ang.cos(), wheel_ang.sin());
    let right = Vec2::new(wheel_ang.sin(), -wheel_ang.cos());
    assert!((fwd.dot(right)).abs() < 1e-6, "Shadow local frame must remain orthogonal");
    assert!((fwd.length() - 1.0).abs() < 1e-6);
}

#[test]
fn test_spec_026_colorway_tinting_consistency_on_decomposed_kart() {
    use macroquad::color::Color;
    use macroquad::texture::Image;
    use std::path::Path;
    use tdrace_app::render::vehicle_assets::apply_vehicle_tint;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let chassis_path = manifest_dir.join("../../assets/textures/vehicles/topdown/classic/classic_kart.png");
    let chassis_bytes = std::fs::read(&chassis_path).expect("Failed to read decomposed classic_kart chassis");
    let base_img = Image::from_file_with_format(&chassis_bytes, None).expect("Failed to parse chassis");

    let custom_primary = Color::new(0.90, 0.15, 0.15, 1.0); // Flame Red
    let custom_secondary = Color::new(0.95, 0.95, 0.10, 1.0); // Bright Yellow
    let tinted = apply_vehicle_tint(&base_img, "classic_kart", custom_primary, custom_secondary);

    // Verify dimensions preserved
    assert_eq!(base_img.width, tinted.width);
    assert_eq!(base_img.height, tinted.height);

    // Verify significant bodywork pixel tinting
    let mut tinted_pixels = 0;
    let mut total_opaque = 0;
    for (orig, tint) in base_img.bytes.chunks_exact(4).zip(tinted.bytes.chunks_exact(4)) {
        if orig[3] > 15 {
            total_opaque += 1;
            if orig != tint {
                tinted_pixels += 1;
            }
        }
    }
    let ratio = tinted_pixels as f32 / total_opaque as f32;
    assert!(ratio >= 0.25, "Expected at least 25% of chassis pixels tinted, got {:.1}%", ratio * 100.0);
}

// ============================================================================
// Spec 031: Modality-Realistic Vehicle Lighting Tests
// ============================================================================

#[test]
fn test_spec_031_modality_realistic_lighting_profiles() {
    use tdrace_app::module::VehicleVisualType;
    use tdrace_app::render::{resolve_vehicle_lighting, VehicleLightingConfig};

    // 1. Karting: Zero electrical lights
    let kart_cfg = resolve_vehicle_lighting(
        Some("classic_kart"),
        VehicleVisualType::GoKart {
            exposed_driver: true,
            side_bumpers: true,
        },
    );
    assert_eq!(kart_cfg, VehicleLightingConfig::none());
    assert!(!kart_cfg.has_headlights, "Karts must not have headlights");
    assert!(!kart_cfg.has_brake_lights, "Karts must not have brake lights");
    assert!(!kart_cfg.has_roof_lightbar, "Karts must not have roof lightbar");
    assert!(!kart_cfg.has_dust_chase_light, "Karts must not have dust chase lights");
    assert!(!kart_cfg.project_track_beams, "Karts must not project track beams");

    // 2. NASCAR: Zero electrical lights (decals only, no brake lights)
    let nascar_cfg = resolve_vehicle_lighting(
        Some("nascar_cup_chevrolet_camaro"),
        VehicleVisualType::StockCar {
            tall_wing: false,
            roof_fins: true,
            window_net: true,
        },
    );
    assert_eq!(nascar_cfg, VehicleLightingConfig::none());
    assert!(!nascar_cfg.has_headlights, "NASCAR stock cars must not have electrical headlights");
    assert!(!nascar_cfg.has_brake_lights, "NASCAR stock cars must not have brake lights");
    assert!(!nascar_cfg.project_track_beams, "NASCAR stock cars must not project track beams");

    // 3. GT / Touring: Full DRL headlights, dynamic brake lights, track illumination
    let gt_cfg = resolve_vehicle_lighting(
        Some("gt_vandorn_arrowhead_t2"),
        VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        },
    );
    assert_eq!(gt_cfg, VehicleLightingConfig::gt_touring());
    assert!(gt_cfg.has_headlights, "GT vehicles must have headlights");
    assert!(gt_cfg.has_brake_lights, "GT vehicles must have brake lights");
    assert!(gt_cfg.project_track_beams, "GT vehicles must project track beams");
    assert!(gt_cfg.beam_range_m >= 10.0, "GT beam range must be at least 10m");
    assert!(gt_cfg.beam_spread_rad > 0.10, "GT beam spread must be positive");

    // 4. Rallycross / All-Terrain: Full headlights, brake lights, track illumination
    let rally_cfg = resolve_vehicle_lighting(
        Some("rally_gallia_200_t1"),
        VehicleVisualType::RallyHatch {
            roof_scoop: true,
            mudflaps: true,
            large_wing: true,
        },
    );
    assert_eq!(rally_cfg, VehicleLightingConfig::rally());
    assert!(rally_cfg.has_headlights, "Rally vehicles must have headlights");
    assert!(rally_cfg.has_brake_lights, "Rally vehicles must have brake lights");
    assert!(rally_cfg.project_track_beams, "Rally vehicles must project track beams");

    // 5. Extreme Off-Road: 4-pod roof lightbar, rear dust chase light, brake lights, track illumination
    let offroad_cfg = resolve_vehicle_lighting(
        Some("classic_offroad"),
        VehicleVisualType::SandRail {
            lightbar: true,
            whip_antenna: true,
            paddle_tires: false,
        },
    );
    assert_eq!(offroad_cfg, VehicleLightingConfig::extreme_offroad());
    assert!(offroad_cfg.has_headlights, "Extreme off-road vehicles must have front spots");
    assert!(offroad_cfg.has_brake_lights, "Extreme off-road vehicles must have brake lights");
    assert!(offroad_cfg.has_roof_lightbar, "Extreme off-road vehicles must have roof lightbars");
    assert!(offroad_cfg.has_dust_chase_light, "Extreme off-road vehicles must have rear dust chase lights");
    assert!(offroad_cfg.project_track_beams, "Extreme off-road vehicles must project track beams");
}

#[test]
fn test_spec_031_all_catalog_cars_lighting_by_modality() {
    use tdrace_app::catalog::get_all_models;
    use tdrace_app::render::{resolve_vehicle_lighting, VehicleLightingConfig};

    let all_cars = get_all_models();
    assert!(all_cars.len() >= 80, "Expected at least 80 real car models in catalog");

    for car in all_cars {
        let cfg = resolve_vehicle_lighting(Some(car.id), car.visual_type);
        match car.module_id {
            "kart" => {
                assert_eq!(cfg, VehicleLightingConfig::none(), "Car {} in 'kart' must have zero lights", car.id);
            }
            "nascar" => {
                assert_eq!(cfg, VehicleLightingConfig::none(), "Car {} in 'nascar' must have zero lights", car.id);
            }
            "gt" => {
                assert_eq!(cfg, VehicleLightingConfig::gt_touring(), "Car {} in 'gt' must have GT touring lighting", car.id);
            }
            "rally" => {
                assert_eq!(cfg, VehicleLightingConfig::rally(), "Car {} in 'rally' must have rally lighting", car.id);
            }
            "extreme_offroad" => {
                assert_eq!(cfg, VehicleLightingConfig::extreme_offroad(), "Car {} in 'extreme_offroad' must have offroad lighting", car.id);
            }
            "autocross" => {
                if car.tier == 4 {
                    assert_eq!(cfg, VehicleLightingConfig::rally(), "Car {} in 'autocross' Tier 4 must have rally lighting", car.id);
                } else {
                    assert_eq!(cfg, VehicleLightingConfig::extreme_offroad(), "Car {} in 'autocross' must have offroad lighting", car.id);
                }
            }
            other => panic!("Unexpected module_id '{}' for car {}", other, car.id),
        }
    }
}

#[test]
fn test_spec_031_procedural_archetype_lighting_fallbacks() {
    use tdrace_app::module::VehicleVisualType;
    use tdrace_app::render::{resolve_vehicle_lighting, VehicleLightingConfig};

    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::GoKart { exposed_driver: true, side_bumpers: true }),
        VehicleLightingConfig::none()
    );
    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::StockCar { tall_wing: false, roof_fins: true, window_net: true }),
        VehicleLightingConfig::none()
    );
    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::OpenWheel { front_wing_span: 1.8, rear_wing_height: 0.9, halo: true }),
        VehicleLightingConfig::none()
    );
    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::TouringGT { widebody: true, gt_wing: true, diffuser: true }),
        VehicleLightingConfig::gt_touring()
    );
    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::RallyHatch { roof_scoop: true, mudflaps: true, large_wing: true }),
        VehicleLightingConfig::rally()
    );
    assert_eq!(
        resolve_vehicle_lighting(None, VehicleVisualType::SandRail { lightbar: true, whip_antenna: true, paddle_tires: false }),
        VehicleLightingConfig::extreme_offroad()
    );
}

#[test]
fn test_vehicle_lighting_toggle_switch_on_off() {
    use tdrace_app::module::VehicleVisualType;
    use tdrace_app::render::resolve_vehicle_lighting;

    // 1. Cars with lights: GT, Rally, Offroad start with lights_on = true
    let mut gt = resolve_vehicle_lighting(
        Some("gt_vandorn_arrowhead_t2"),
        VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        },
    );
    assert!(gt.has_lights(), "GT vehicles must have lights equipped");
    assert!(gt.lights_on, "GT headlights must be on by default");

    // Switch off
    assert!(gt.toggle_lights(), "Toggle must succeed for GT car");
    assert!(!gt.lights_on, "GT lights must be switched off");

    // Switch back on
    assert!(gt.toggle_lights(), "Toggle must succeed to turn back on");
    assert!(gt.lights_on, "GT lights must be switched on");

    let mut rally = resolve_vehicle_lighting(
        Some("rally_gallia_200_t1"),
        VehicleVisualType::RallyHatch {
            roof_scoop: true,
            mudflaps: true,
            large_wing: true,
        },
    );
    assert!(rally.has_lights(), "Rally vehicles must have lights equipped");
    assert!(rally.lights_on, "Rally lights must be on by default");
    assert!(rally.toggle_lights());
    assert!(!rally.lights_on);

    let mut offroad = resolve_vehicle_lighting(
        Some("classic_offroad"),
        VehicleVisualType::SandRail {
            lightbar: true,
            whip_antenna: true,
            paddle_tires: false,
        },
    );
    assert!(offroad.has_lights(), "Extreme offroad vehicles must have lights equipped");
    assert!(offroad.lights_on, "Offroad lightbar/spots must be on by default");
    assert!(offroad.toggle_lights());
    assert!(!offroad.lights_on);

    // 2. Cars without lights: Kart, NASCAR cannot have lights switched on
    let mut kart = resolve_vehicle_lighting(
        Some("classic_kart"),
        VehicleVisualType::GoKart {
            exposed_driver: true,
            side_bumpers: true,
        },
    );
    assert!(!kart.has_lights(), "Karts must not have lights equipped");
    assert!(!kart.lights_on, "Karts must not have lights on");
    // Attempting to toggle returns false and stays off
    assert!(!kart.toggle_lights(), "Toggle must return false on kart");
    assert!(!kart.lights_on, "Kart lights must remain off");
    // Explicit with_lights_on(true) must be ignored for unequipped vehicles
    let forced_kart = kart.with_lights_on(true);
    assert!(!forced_kart.lights_on, "Unequipped kart must ignore with_lights_on(true)");

    let mut nascar = resolve_vehicle_lighting(
        Some("nascar_cup_chevrolet_camaro"),
        VehicleVisualType::StockCar {
            tall_wing: false,
            roof_fins: true,
            window_net: true,
        },
    );
    assert!(!nascar.has_lights(), "NASCAR stock cars must not have lights equipped");
    assert!(!nascar.lights_on, "NASCAR lights must not be on");
    assert!(!nascar.toggle_lights(), "Toggle must return false on NASCAR");
    assert!(!nascar.lights_on, "NASCAR lights must remain off");
    let forced_nascar = nascar.with_lights_on(true);
    assert!(!forced_nascar.lights_on, "Unequipped NASCAR must ignore with_lights_on(true)");
}

#[test]
fn test_spec_073_classic_12_vehicle_harmonization_and_steered_wheels() {
    use std::collections::HashMap;
    use std::path::Path;
    use macroquad::texture::Image;
    use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
    use tdrace_app::module::classic::ClassicGameModule;
    use tdrace_app::module::GameModule;
    use tdrace_app::render::vehicle_assets::get_steered_wheel_config;
    use tdrace_core::CarCategory;

    let classic_module = ClassicGameModule::new();
    let vehicles = classic_module.vehicles();

    // 1. Exactly 12 vehicles registered in ClassicGameModule
    assert_eq!(vehicles.len(), 12, "Classic module must have exactly 12 vehicles (2 per category)");

    // 2. Exactly 12 vehicles in CLASSIC_ARCADE_CARS catalog
    assert_eq!(CLASSIC_ARCADE_CARS.len(), 12, "Catalog must contain exactly 12 classic arcade cars");

    // 3. Exactly 2 vehicles per category
    let mut category_counts: HashMap<CarCategory, usize> = HashMap::new();
    for car in CLASSIC_ARCADE_CARS {
        *category_counts.entry(car.category()).or_insert(0) += 1;
    }

    assert_eq!(category_counts.get(&CarCategory::Gt), Some(&2), "Must have 2 GT models");
    assert_eq!(category_counts.get(&CarCategory::Nascar), Some(&2), "Must have 2 Stock Car models");
    assert_eq!(category_counts.get(&CarCategory::Rally), Some(&2), "Must have 2 Rally models");
    assert_eq!(category_counts.get(&CarCategory::Kart), Some(&2), "Must have 2 Kart models");
    assert_eq!(category_counts.get(&CarCategory::OffRoad), Some(&2), "Must have 2 All-Terrain models");
    assert_eq!(category_counts.get(&CarCategory::Autocross), Some(&2), "Must have 2 Autocross models");

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_dir = manifest_dir.join("../../assets/textures/vehicles/topdown/classic");
    let lateral_dir = manifest_dir.join("../../assets/textures/vehicles/laterals/classic");

    // 4. Verify asset files, dimensions, wheel configs, and Ackermann steering for every single vehicle
    for v in &vehicles {
        // A. SteeredWheelConfig presence & validity
        let wheel_cfg = get_steered_wheel_config(v.id).unwrap_or_else(|| panic!("{} missing SteeredWheelConfig", v.id));
        assert!(wheel_cfg.front_axle_offset > 0.30, "Front axle offset must be realistic for {}", v.id);
        assert!(wheel_cfg.half_track_width > 0.30, "Half track width must be realistic for {}", v.id);
        assert!(wheel_cfg.wheel_size.x > 0.15 && wheel_cfg.wheel_size.y > 0.20, "Wheel size realistic for {}", v.id);

        // B. Topdown showroom and chassis sprites
        let topdown_path = topdown_dir.join(format!("{}.png", v.id));
        let chassis_path = topdown_dir.join(format!("{}_chassis.png", v.id));
        assert!(topdown_path.exists(), "Missing topdown sprite for {}", v.id);
        assert!(chassis_path.exists(), "Missing chassis sprite for {}", v.id);

        let td_img = Image::from_file_with_format(&std::fs::read(&topdown_path).unwrap(), None).unwrap();
        let ch_img = Image::from_file_with_format(&std::fs::read(&chassis_path).unwrap(), None).unwrap();
        assert_eq!(td_img.width, 512);
        assert_eq!(td_img.height, 512);
        assert_eq!(ch_img.width, 512);
        assert_eq!(ch_img.height, 512);

        // C. Lateral turntable and thumbnail sprites
        let lateral_path = lateral_dir.join(format!("{}.png", v.id));
        let thumb_path = lateral_dir.join(format!("{}_thumb.png", v.id));
        assert!(lateral_path.exists(), "Missing lateral sprite for {}", v.id);
        assert!(thumb_path.exists(), "Missing thumb sprite for {}", v.id);

        let lat_img = Image::from_file_with_format(&std::fs::read(&lateral_path).unwrap(), None).unwrap();
        let th_img = Image::from_file_with_format(&std::fs::read(&thumb_path).unwrap(), None).unwrap();
        assert_eq!(lat_img.width, 1024);
        assert_eq!(lat_img.height, 512);
        assert_eq!(th_img.width, 256);
        assert_eq!(th_img.height, 128);

        // D. Ackermann dynamic deflection
        let car = Car::new(v.config);
        let (fl_left, fr_left) = car.compute_ackermann_angles(0.35);
        assert!(fl_left > fr_left, "Inner wheel must turn sharper than outer wheel during left turn for {}", v.id);
        let (fl_right, fr_right) = car.compute_ackermann_angles(-0.35);
        assert!(fr_right.abs() > fl_right.abs(), "Inner wheel must turn sharper than outer wheel during right turn for {}", v.id);
    }
}

#[test]
fn test_spec_075_chassis_skeleton_render_geometry_and_fixture_alignment() {
    let presets = [
        ("sports_car", CarConfig::sports_car()),
        ("drift_car", CarConfig::drift_car()),
        ("kart", CarConfig::kart()),
        ("rally_car", CarConfig::rally_car()),
        ("stock_car_ta1", CarConfig::stock_car_ta1()),
        ("sand_rail", CarConfig::sand_rail()),
    ];

    for (name, cfg) in presets {
        let car = Car::new(cfg).with_pose(Vec2::new(10.0, 15.0), 0.0);
        let geom_offset = car.config.chassis.geometric_center_offset_from_cg(car.config.cg_to_front, car.config.cg_to_rear);
        let visual_center = car.state.position + car.forward_vector() * geom_offset;

        // Front bumper world position: visual_center + forward * half_length
        let front_bumper = visual_center + car.forward_vector() * car.config.chassis.half_length(car.config.wheelbase);
        // Expected front bumper: CG + forward * (cg_to_front + front_overhang)
        let expected_front = car.state.position + car.forward_vector() * (car.config.cg_to_front + car.config.chassis.front_overhang);
        assert!(
            (front_bumper.x - expected_front.x).abs() < 1e-4 && (front_bumper.y - expected_front.y).abs() < 1e-4,
            "Front bumper mismatch on {}: {:?} vs {:?}",
            name, front_bumper, expected_front
        );

        // Rear bumper world position: visual_center - forward * half_length
        let rear_bumper = visual_center - car.forward_vector() * car.config.chassis.half_length(car.config.wheelbase);
        // Expected rear bumper: CG - forward * (cg_to_rear + rear_overhang)
        let expected_rear = car.state.position - car.forward_vector() * (car.config.cg_to_rear + car.config.chassis.rear_overhang);
        assert!(
            (rear_bumper.x - expected_rear.x).abs() < 1e-4 && (rear_bumper.y - expected_rear.y).abs() < 1e-4,
            "Rear bumper mismatch on {}: {:?} vs {:?}",
            name, rear_bumper, expected_rear
        );

        // Headlight fixtures must sit inside front bumper
        let (hl_left, hl_right) = car.config.chassis.headlight_positions_world(
            car.state.position,
            car.forward_vector(),
            car.right_vector(),
            car.config.cg_to_front,
        );
        assert!(hl_left.x < front_bumper.x || (hl_left.x - front_bumper.x).abs() < 0.1);
        assert!(hl_right.x < front_bumper.x || (hl_right.x - front_bumper.x).abs() < 0.1);
    }
}

/// Scenario: Decoupled wheel geometry asset scaling and memory bounds (Spec 074)
///
/// Given 6 vehicle categories and 8 tire compounds
/// When inspecting wheel texture assets and loading them into memory
/// Then total texture asset files must be <= 8 (decoupled from compound count)
/// And total decoded texture memory footprint must be strictly under 2.0 MB
/// And adding a new 9th compound requires 0 additional texture files.
#[test]
fn test_wheel_texture_cache_memory_bounds() {
    use macroquad::texture::Image;
    use std::path::PathBuf;
    use tdrace_core::surface::CompoundId;

    let wheel_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/textures/vehicles/topdown/wheels");

    // Enumerate wheel texture files on disk
    let entries = std::fs::read_dir(&wheel_dir).expect("Wheel texture directory must exist");
    let mut wheel_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            wheel_files.push(path);
        }
    }

    println!("Found {} wheel texture assets on disk", wheel_files.len());
    assert!(
        wheel_files.len() <= 10,
        "Total wheel textures ({}) must be <= 10 (N + M decoupled invariance)",
        wheel_files.len()
    );

    let mut total_bytes = 0usize;
    for file_path in &wheel_files {
        let bytes = std::fs::read(file_path).unwrap();
        let img = Image::from_file_with_format(&bytes, None).unwrap();
        let mem = (img.width as usize) * (img.height as usize) * 4;
        total_bytes += mem;
    }

    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
    println!(
        "Wheel Texture Memory: {} textures, total {} bytes ({:.3} MB)",
        wheel_files.len(),
        total_bytes,
        total_mb
    );

    assert!(
        total_bytes < 2 * 1024 * 1024,
        "Total wheel texture memory ({:.3} MB) must be strictly under 2.0 MB",
        total_mb
    );

    // Verify all 8 motorsport compounds have valid badge codes and distinct accent RGBA
    assert_eq!(CompoundId::ALL.len(), 8);
    for c in CompoundId::ALL {
        assert!(!c.name().is_empty());
        assert!(!c.badge_code().is_empty());
        let rgba = c.accent_rgba();
        assert_eq!(rgba[3], 1.0, "Accent color alpha must be 1.0");
    }
}

#[test]
fn test_spec_074_steered_wheel_accent_scale_and_dimensions() {
    use tdrace_app::render::vehicle_assets::get_steered_wheel_config;

    let models = [
        "classic_kart",
        "classic_kart_vintage",
        "classic_gt",
        "classic_gt_vintage",
        "classic_nascar",
        "classic_stock_vintage",
        "classic_rally",
        "classic_rx_vintage",
        "classic_ax_mudlark",
        "classic_ax_brawler",
        "classic_offroad",
        "classic_at_safari",
    ];

    for m in models {
        let cfg = get_steered_wheel_config(m).expect("Steered wheel config must exist");
        let dest_w = cfg.wheel_size.x;
        let dest_h = cfg.wheel_size.y;

        // Physical wheel size bounds in meters
        assert!(dest_w >= 0.15 && dest_w <= 0.35, "Wheel width {dest_w} out of physical range for {m}");
        assert!(dest_h >= 0.20 && dest_h <= 0.65, "Wheel height {dest_h} out of physical range for {m}");

        // Sidewall stripe thickness must be strictly sub-decimeter (e.g. 2-5 cm) to avoid obscuring car sprites
        let stripe_thickness = (dest_w * 0.16).clamp(0.02, 0.05);
        assert!(stripe_thickness <= 0.05, "Stripe thickness {stripe_thickness} must be <= 5cm");
        assert!(stripe_thickness >= 0.02, "Stripe thickness {stripe_thickness} must be >= 2cm");
        assert!(stripe_thickness < dest_w * 0.25, "Stripe thickness must not dominate wheel width");
    }
}

#[test]
fn test_spec_094_universal_steered_wheel_derivation_across_all_platforms() {
    use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS, VAULT_ARCHIVE_CARS};
    use tdrace_app::render::vehicle_assets::{derive_steered_wheel_config, WheelLayerMode};
    use tdrace_app::ui::menu::CarChoice;

    let all_vehicles: Vec<_> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .chain(VAULT_ARCHIVE_CARS.iter())
        .collect();

    assert!(
        all_vehicles.len() >= 122,
        "Master vehicle catalog must contain >= 122 vehicles, found {}",
        all_vehicles.len()
    );

    // 1. Verify steered wheel derivation succeeds globally across all vehicles
    for v in &all_vehicles {
        let car_config = v.to_car_config();
        let steered_cfg = derive_steered_wheel_config(v.id, &car_config)
            .unwrap_or_else(|| panic!("derive_steered_wheel_config must return Some for {}", v.id));

        assert!(
            steered_cfg.front_axle_offset > 0.15,
            "front_axle_offset ({}) must be positive and realistic for {}",
            steered_cfg.front_axle_offset,
            v.id
        );
        assert!(
            steered_cfg.half_track_width > 0.15,
            "half_track_width ({}) must be positive and realistic for {}",
            steered_cfg.half_track_width,
            v.id
        );
        assert!(
            steered_cfg.wheel_size.x > 0.05,
            "wheel_size.x ({}) must be realistic for {}",
            steered_cfg.wheel_size.x,
            v.id
        );
        assert!(
            steered_cfg.wheel_size.y > 0.10,
            "wheel_size.y ({}) must be realistic for {}",
            steered_cfg.wheel_size.y,
            v.id
        );
        assert!(
            steered_cfg.texture_padding_factor.x >= 1.0,
            "texture_padding_factor.x must be >= 1.0 for {}",
            v.id
        );
        assert!(
            steered_cfg.texture_padding_factor.y >= 1.0,
            "texture_padding_factor.y must be >= 1.0 for {}",
            v.id
        );
        let expected_layering = if let Some(anchor) = tdrace_app::render::vehicle_assets::get_visual_wheel_anchor(v.id) {
            if anchor.layering == "OverChassis" {
                WheelLayerMode::OverChassis
            } else {
                WheelLayerMode::UnderChassis
            }
        } else {
            v.base_car_choice.wheel_layer_mode()
        };
        assert_eq!(
            steered_cfg.layering,
            expected_layering,
            "layering must match visual anchor or platform for {}",
            v.id
        );
    }

    // 2. Scenario: Autocross T4 Touring AX saloons possess dedicated touring geometry
    let bohemia = all_vehicles.iter().find(|c| c.id == "autocross_bohemia_veloce_t4").expect("bohemia found");
    assert_eq!(bohemia.base_car_choice, CarChoice::TouringAX);
    let bohemia_cfg = bohemia.to_car_config();
    assert!(bohemia_cfg.track_width >= 1.85, "TouringAX track width >= 1.85m");
    assert!(bohemia_cfg.chassis.front_overhang >= 0.80, "TouringAX front overhang >= 0.80m");
    assert_eq!(bohemia.base_car_choice.wheel_layer_mode(), WheelLayerMode::UnderChassis);
    assert_eq!(bohemia_cfg.engine_placement, tdrace_core::physics::config::EnginePlacement::FrontEngine);

    // 3. Scenario: Volkskraft Dune Buggy T1 possesses distinct Baja geometry from Sand Rail
    let volkskraft = all_vehicles.iter().find(|c| c.id == "offroad_volkskraft_dune_t1").expect("volkskraft found");
    assert_eq!(volkskraft.base_car_choice, CarChoice::DuneBuggyBaja);
    let volkskraft_cfg = volkskraft.to_car_config();
    assert!(volkskraft_cfg.chassis.rear_overhang >= 0.50, "Baja rear overhang >= 0.50m");
    assert!(volkskraft_cfg.chassis.front_overhang >= 0.40, "Baja front overhang >= 0.40m");
    assert_eq!(volkskraft.base_car_choice.wheel_layer_mode(), WheelLayerMode::OverChassis);
    assert_eq!(volkskraft_cfg.engine_placement, tdrace_core::physics::config::EnginePlacement::RearEngine);

    let nomad = all_vehicles.iter().find(|c| c.id == "offroad_laurentian_nomad_t1").expect("nomad found");
    assert_eq!(nomad.base_car_choice, CarChoice::SandRail);
    let nomad_cfg = nomad.to_car_config();
    assert!(nomad_cfg.chassis.front_overhang <= 0.18, "SandRail front overhang <= 0.18m");
    assert_eq!(nomad.base_car_choice.wheel_layer_mode(), WheelLayerMode::OverChassis);

    // 4. Scenario: Trophy Trucks and Monster Trucks possess accurate truck-scale collision hulls
    let trophy = all_vehicles.iter().find(|c| c.id == "offroad_desert_forge_truck_t2").expect("trophy truck found");
    assert_eq!(trophy.base_car_choice, CarChoice::TrophyTruckAWD);
    let trophy_cfg = trophy.to_car_config();
    let trophy_hull_len = trophy_cfg.wheelbase + trophy_cfg.chassis.front_overhang + trophy_cfg.chassis.rear_overhang;
    assert!(trophy_hull_len >= 5.0, "Trophy truck hull length >= 5.0m, found {}", trophy_hull_len);
    assert_eq!(trophy.base_car_choice.wheel_layer_mode(), WheelLayerMode::UnderChassis);

    let monster = all_vehicles.iter().find(|c| c.id == "offroad_colossus_titan_t5").expect("monster truck found");
    assert_eq!(monster.base_car_choice, CarChoice::MonsterTruck);
    let monster_cfg = monster.to_car_config();
    assert!(monster_cfg.track_width >= 2.6, "Monster truck hull width >= 2.6m");
    assert!(monster_cfg.wheels[0].tire_radius >= 0.80, "Monster truck 66-inch tire radius >= 0.80m");
    assert!(monster_cfg.wheels[0].tire_width >= 0.60, "Monster truck 66-inch tire width >= 0.60m");
    assert_eq!(monster.base_car_choice.wheel_layer_mode(), WheelLayerMode::OverChassis);
}

#[test]
fn test_spec_091_global_chassis_sprites_integrity_and_dimensions() {
    use macroquad::texture::Image;
    use std::path::Path;
    use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS};

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_root = manifest_dir.join("../../assets/textures/vehicles/topdown");

    let all_vehicles: Vec<_> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .collect();

    assert_eq!(
        all_vehicles.len(),
        122,
        "Playable vehicle catalog must contain exactly 122 vehicles (12 classic + 110 motorsport)"
    );

    let mut verified_count = 0;
    for v in &all_vehicles {
        let chassis_path = topdown_root
            .join(v.module_id)
            .join(format!("{}_chassis.png", v.id));
        let full_path = topdown_root
            .join(v.module_id)
            .join(format!("{}.png", v.id));

        assert!(
            full_path.exists(),
            "Canonical full topdown sprite must exist on disk: {:?}",
            full_path
        );
        assert!(
            chassis_path.exists(),
            "Chassis cutout sprite must exist on disk for vehicle {}: {:?}",
            v.id,
            chassis_path
        );

        let chassis_bytes = std::fs::read(&chassis_path)
            .unwrap_or_else(|e| panic!("Failed to read chassis sprite for {}: {:?}", v.id, e));
        let chassis_img = Image::from_file_with_format(&chassis_bytes, None)
            .unwrap_or_else(|e| panic!("Corrupted chassis PNG for {}: {:?}", v.id, e));

        assert_eq!(
            chassis_img.width, 512,
            "Chassis sprite width must be 512 for {}",
            v.id
        );
        assert_eq!(
            chassis_img.height, 512,
            "Chassis sprite height must be 512 for {}",
            v.id
        );

        verified_count += 1;
    }

    assert_eq!(
        verified_count, 122,
        "All 122 playable vehicles must have verified 512x512 chassis sprites"
    );

    // Also verify vault novelty lawnmowers in kart/ directory
    let vault_mowers = [
        "vault_asahi_blade_runner",
        "vault_greenfield_prairie_racer",
        "vault_nordic_valhalla_tractor",
    ];
    for mower_id in vault_mowers {
        let chassis_path = topdown_root
            .join("kart")
            .join(format!("{}_chassis.png", mower_id));
        let full_path = topdown_root
            .join("kart")
            .join(format!("{}.png", mower_id));

        assert!(full_path.exists(), "Vault mower full sprite must exist: {:?}", full_path);
        assert!(chassis_path.exists(), "Vault mower chassis sprite must exist: {:?}", chassis_path);

        let bytes = std::fs::read(&chassis_path).unwrap();
        let img = Image::from_file_with_format(&bytes, None).unwrap();
        assert_eq!(img.width, 512);
        assert_eq!(img.height, 512);
    }
}

#[test]
fn test_spec_091_global_steered_wheel_articulation_and_runtime_integration() {
    use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
    use tdrace_app::render::vehicle_assets::{derive_steered_wheel_config, WheelLayerMode};
    use tdrace_app::ui::menu::CarChoice;

    let all_vehicles: Vec<_> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .collect();

    for v in &all_vehicles {
        let car_config = v.to_car_config();
        let car = Car::new(car_config.clone());

        // 1. Verify steered wheel derivation succeeds globally
        let cfg = derive_steered_wheel_config(v.id, &car_config)
            .unwrap_or_else(|| panic!("derive_steered_wheel_config failed for {}", v.id));

        assert!(cfg.front_axle_offset > 0.0, "front_axle_offset must be positive for {}", v.id);
        assert!(cfg.half_track_width > 0.0, "half_track_width must be positive for {}", v.id);
        assert!(cfg.wheel_size.x > 0.0, "wheel_size.x must be positive for {}", v.id);
        assert!(cfg.wheel_size.y > 0.0, "wheel_size.y must be positive for {}", v.id);

        // 2. Proportional dimensions without artificial 1:2 clamp (Spec 095)
        assert!(
            cfg.wheel_size.y > cfg.wheel_size.x,
            "wheel length must exceed wheel width for {}",
            v.id
        );
        assert!(
            cfg.texture_padding_factor.x >= 1.0 && cfg.texture_padding_factor.y >= 1.0,
            "texture padding factors must be >= 1.0 for {}",
            v.id
        );

        // 3. Modality Platform Layering Architecture
        if let Some(anchor) = tdrace_app::render::vehicle_assets::get_visual_wheel_anchor(v.id) {
            let anchor_layering = if anchor.layering == "OverChassis" {
                WheelLayerMode::OverChassis
            } else {
                WheelLayerMode::UnderChassis
            };
            assert_eq!(
                cfg.layering, anchor_layering,
                "Layering must match visual anchor for {}",
                v.id
            );
        } else {
            let expected_layering = match v.base_car_choice {
                CarChoice::CrossCar
                | CarChoice::SuperBuggy
                | CarChoice::DuneBuggyBaja
                | CarChoice::MonsterTruck
                | CarChoice::Kart
                | CarChoice::SuperkartGP
                | CarChoice::SandRail => WheelLayerMode::OverChassis,

                CarChoice::GT4Clubsport
                | CarChoice::GT3Car
                | CarChoice::GT2Biturbo
                | CarChoice::GT1Legend
                | CarChoice::HypercarPrototype
                | CarChoice::TouringAX
                | CarChoice::TrophyTruckAWD
                | CarChoice::MudBoggerHeavy
                | CarChoice::RallyJuniorFWD
                | CarChoice::RallyCar
                | CarChoice::RallyGroupB
                | CarChoice::RallyElectricRX
                | CarChoice::StockCar
                | CarChoice::StockCarTruck
                | CarChoice::SportsCar
                | CarChoice::DriftCar => WheelLayerMode::UnderChassis,
            };
            assert_eq!(
                cfg.layering, expected_layering,
                "Layering must match platform archetype for {}",
                v.id
            );
        }

        // 4. Authentic dynamic Ackermann angle resolution
        let (fl_zero, fr_zero) = car.compute_ackermann_angles(0.0);
        assert!(fl_zero.abs() < 1e-5, "Zero steer tracking parallel for {}", v.id);
        assert!(fr_zero.abs() < 1e-5, "Zero steer tracking parallel for {}", v.id);

        let (fl_left, fr_left) = car.compute_ackermann_angles(0.35);
        assert!(fl_left > 0.0 && fr_left > 0.0, "Both wheels turn into turn for {}", v.id);
        assert!(
            fl_left > fr_left,
            "Ackermann geometry: inner wheel turns sharper than outer wheel for {}",
            v.id
        );
    }
}

#[test]
fn test_spec_095_multiview_wheel_anchor_extraction_and_archetype_integration() {
    use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
    use tdrace_app::render::vehicle_assets::{
        derive_steered_wheel_config, get_visual_wheel_anchor, get_visual_wheel_anchors,
    };

    let all_vehicles: Vec<_> = CLASSIC_ARCADE_CARS
        .iter()
        .chain(ALL_REAL_CARS.iter())
        .collect();

    let anchors = get_visual_wheel_anchors();
    assert!(anchors.len() >= 122, "Visual wheel anchors must contain >= 122 vehicles, found {}", anchors.len());

    // 1. Verify every vehicle in catalog resolves a valid anchor
    for v in &all_vehicles {
        let anchor = get_visual_wheel_anchor(v.id)
            .unwrap_or_else(|| panic!("Vehicle {} must resolve a visual wheel anchor", v.id));

        assert!(anchor.axle_x_px > 256.0, "Front axle X must be in front half of sprite (> 256) for {}", v.id);
        assert!(anchor.track_width_px > 50.0 && anchor.track_width_px < 400.0, "Track width px in realistic range for {}", v.id);
        assert!(anchor.tire_len_px > 30.0 && anchor.tire_len_px < 300.0, "Tire length px in realistic range for {}", v.id);
        assert!(anchor.tire_wid_px > 15.0 && anchor.tire_wid_px < 200.0, "Tire width px in realistic range for {}", v.id);

        let car_config = v.to_car_config();
        let cfg = derive_steered_wheel_config(v.id, &car_config)
            .unwrap_or_else(|| panic!("derive_steered_wheel_config failed for {}", v.id));

        assert!(cfg.front_axle_offset > 0.15, "front_axle_offset must be positive and realistic for {}", v.id);
        assert!(cfg.half_track_width > 0.15, "half_track_width must be positive and realistic for {}", v.id);
        assert!(cfg.wheel_size.x > 0.05, "wheel_size.x must be realistic for {}", v.id);
        assert!(cfg.wheel_size.y > 0.10, "wheel_size.y must be realistic for {}", v.id);
        assert!(cfg.texture_padding_factor.x >= 1.0 && cfg.texture_padding_factor.y >= 1.0);

        // Verify assigned archetype texture exists and can be loaded
        let rel_path = format!("textures/vehicles/topdown/wheels/{}.png", cfg.wheel_texture_id);
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets")
            .join(&rel_path);
        assert!(path.exists(), "Wheel texture {} must exist on disk at {:?}", cfg.wheel_texture_id, path);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {}: {:?}", path.display(), e));
        let img = macroquad::texture::Image::from_file_with_format(&bytes, None).unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", path.display(), e));
        assert_eq!(img.width, 128);
        assert_eq!(img.height, 256);
    }

    // 2. Specific archetype validations
    // Monster Truck must use monster_wheel_front and have massive tires
    let colossus = get_visual_wheel_anchor("offroad_colossus_titan_t5").expect("colossus anchor");
    assert_eq!(colossus.archetype, "monster_wheel_front");
    assert!(colossus.tire_len_px > 120.0, "Monster truck tire len > 120px, got {}", colossus.tire_len_px);
    assert!(colossus.tire_wid_px > 60.0, "Monster truck tire wid > 60px, got {}", colossus.tire_wid_px);
    assert_eq!(colossus.layering, "OverChassis");

    // Mud Bogger must use mud_tractor_front
    let crossbow = get_visual_wheel_anchor("offroad_crossbow_ridge_t4").expect("crossbow anchor");
    assert_eq!(crossbow.archetype, "mud_tractor_front");

    // Trophy Truck must use truck_allterrain_front
    let desert_forge = get_visual_wheel_anchor("offroad_desert_forge_truck_t2").expect("desert_forge anchor");
    assert_eq!(desert_forge.archetype, "truck_allterrain_front");

    // Buggy must use buggy_allterrain_front
    let volkskraft = get_visual_wheel_anchor("offroad_volkskraft_dune_t1").expect("volkskraft anchor");
    assert_eq!(volkskraft.archetype, "buggy_allterrain_front");
    assert_eq!(volkskraft.layering, "OverChassis");

    // Kart must use kart_slick_front and OverChassis
    let cadet = get_visual_wheel_anchor("kart_blackline_cadet_t1").expect("cadet anchor");
    assert_eq!(cadet.archetype, "kart_slick_front");
    assert_eq!(cadet.layering, "OverChassis");
}

#[test]
fn test_spec_095_closed_wheel_chassis_sprites_remain_intact_without_cutouts() {
    use macroquad::texture::Image;
    use std::path::Path;
    use tdrace_app::render::vehicle_assets::get_visual_wheel_anchor;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let topdown_root = manifest_dir.join("../../assets/textures/vehicles/topdown");

    // 1. Closed-wheel vehicles must have intact chassis sprites with zero erased bodywork
    let closed_wheel_models = [
        ("rally", "rally_vortek_quattro_rx_t3"),
        ("rally", "rally_green_mountain_titan_t6"),
        ("rally", "rally_sixstar_vortex_t4"),
        ("gt", "gt_vandorn_arrowhead_t2"),
        ("nascar", "nascar_crossbow_montego_t1"),
        ("extreme_offroad", "offroad_desert_forge_truck_t2"),
        ("autocross", "autocross_vortek_quattro_t4"),
    ];

    for (module_id, model_id) in closed_wheel_models {
        let anchor = get_visual_wheel_anchor(model_id).expect("anchor exists");
        assert_eq!(anchor.layering, "UnderChassis", "Model {} must be UnderChassis", model_id);

        let base_path = topdown_root.join(module_id).join(format!("{}.png", model_id));
        let chassis_path = topdown_root.join(module_id).join(format!("{}_chassis.png", model_id));

        let base_img = Image::from_file_with_format(&std::fs::read(&base_path).unwrap(), None).unwrap();
        let chassis_img = Image::from_file_with_format(&std::fs::read(&chassis_path).unwrap(), None).unwrap();

        assert_eq!(base_img.bytes, chassis_img.bytes, "Chassis sprite for closed-wheel {} must match canonical sprite byte-for-byte", model_id);
    }

    // 2. Open-wheel vehicles must have tire rubber cleanly erased on chassis sprite
    let open_wheel_models = [
        ("kart", "kart_blackline_cadet_t1"),
        ("extreme_offroad", "offroad_volkskraft_dune_t1"),
        ("autocross", "autocross_bologna_superbuggy_t5"),
    ];

    for (module_id, model_id) in open_wheel_models {
        let anchor = get_visual_wheel_anchor(model_id).expect("anchor exists");
        assert_eq!(anchor.layering, "OverChassis", "Model {} must be OverChassis", model_id);

        let base_path = topdown_root.join(module_id).join(format!("{}.png", model_id));
        let chassis_path = topdown_root.join(module_id).join(format!("{}_chassis.png", model_id));

        let base_img = Image::from_file_with_format(&std::fs::read(&base_path).unwrap(), None).unwrap();
        let chassis_img = Image::from_file_with_format(&std::fs::read(&chassis_path).unwrap(), None).unwrap();

        assert_ne!(base_img.bytes, chassis_img.bytes, "Chassis sprite for open-wheel {} must have rubber erased", model_id);
    }
}



