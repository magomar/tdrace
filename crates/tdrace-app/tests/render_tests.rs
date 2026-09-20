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

#[test]
fn test_all_80_motorsport_cars_catalog_integrity() {
    use tdrace_app::catalog::ALL_REAL_CARS;

    assert_eq!(ALL_REAL_CARS.len(), 80, "Catalog must contain exactly 80 authentic motorsport vehicles");

    let modules = ["gt", "nascar", "rally", "extreme_offroad", "kart"];
    for m in modules {
        let count = ALL_REAL_CARS.iter().filter(|c| c.module_id == m).count();
        if m == "gt" {
            assert_eq!(count, 20, "GT module must contain 20 vehicles (4 per tier)");
        } else {
            assert_eq!(count, 15, "Module {} must contain 15 vehicles (3 per tier)", m);
        }
    }

    use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
    assert_eq!(CLASSIC_ARCADE_CARS.len(), 5, "Classic arcade catalog must contain 5 fantasy vehicles");

    for car in ALL_REAL_CARS {
        assert!(!car.id.is_empty(), "Car ID cannot be empty");
        assert!(!car.name.is_empty(), "Car name cannot be empty");
        assert!(car.tier >= 1 && car.tier <= 5, "Tier must be between 1 and 5");
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

    let cars = ["classic_gt", "classic_nascar", "classic_offroad", "classic_kart", "classic_rally"];
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

    assert!(session.cars.len() >= 4, "Roster must include player and bots");

    let player_model_id = session.car_model_ids[0].expect("Player must have classic model id");
    let player_model = find_model_by_id(player_model_id).expect("Model must exist in catalog");

    // All bots must NOT match factory livery (so they trigger mask-based tinting)
    // and must have primary colors visually distinct from the player model's factory primary color.
    for i in 1..session.cars.len() {
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
        assert!(session.cars.len() >= 4, "Roster must include player and bots");

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
        for i in 1..session.cars.len() {
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
        "gt_toyota_supra_gt4",
        "gt_bmw_m4_gt4",
        "gt_aston_vantage_gt4",
        "gt_porsche_718_gt4",
        "gt_porsche_911_gt3r",
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




