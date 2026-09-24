use glam::Vec2;
use macroquad::color::Color;
use tdrace_app::ai::DriverTier;
use tdrace_app::camera::{CameraConfig, RaceCamera};
use tdrace_app::config::{DisplayConfig, GameConfig};
use tdrace_app::game::RaceSession;
use tdrace_app::render::{
    compute_proximity_alpha, deconflict_nameplates, render_floating_bot_nameplates,
    PlayerVisibilityOptions, VehicleNameplateItem, NAMEPLATE_DECONFLICT_H_THRESH,
    NAMEPLATE_DECONFLICT_W_THRESH, NAMEPLATE_HEIGHT_CLEARANCE, NAMEPLATE_INNER_RADIUS,
    NAMEPLATE_OUTER_RADIUS, NAMEPLATE_STACK_NUDGE,
};
use tdrace_app::ui::font::Fonts;

#[test]
fn test_constants_calibration() {
    assert_eq!(NAMEPLATE_INNER_RADIUS, 25.0);
    assert_eq!(NAMEPLATE_OUTER_RADIUS, 55.0);
    assert_eq!(NAMEPLATE_HEIGHT_CLEARANCE, 2.4);
    assert_eq!(NAMEPLATE_DECONFLICT_W_THRESH, 75.0);
    assert_eq!(NAMEPLATE_DECONFLICT_H_THRESH, 24.0);
    assert_eq!(NAMEPLATE_STACK_NUDGE, 20.0);
}

#[test]
fn test_compute_proximity_alpha_boundaries_and_linearity() {
    // Within inner radius: full opacity 1.0
    assert_eq!(compute_proximity_alpha(0.0), 1.0);
    assert_eq!(compute_proximity_alpha(10.0), 1.0);
    assert_eq!(compute_proximity_alpha(25.0), 1.0);

    // Beyond outer radius: zero opacity 0.0
    assert_eq!(compute_proximity_alpha(55.0), 0.0);
    assert_eq!(compute_proximity_alpha(60.0), 0.0);
    assert_eq!(compute_proximity_alpha(120.0), 0.0);

    // Midpoint: (55 - 40) / (55 - 25) = 15 / 30 = 0.50
    let mid = compute_proximity_alpha(40.0);
    assert!((mid - 0.50).abs() < 1e-4, "Midpoint must be 0.50, got {}", mid);

    // Quarter points
    let q1 = compute_proximity_alpha(32.5); // 22.5 / 30 = 0.75
    assert!((q1 - 0.75).abs() < 1e-4, "Quarter point must be 0.75, got {}", q1);

    let q3 = compute_proximity_alpha(47.5); // 7.5 / 30 = 0.25
    assert!((q3 - 0.25).abs() < 1e-4, "Three-quarter point must be 0.25, got {}", q3);

    // Strictly monotonically decreasing in transition zone
    let mut prev = 1.0;
    for d in (26..55).map(|x| x as f32) {
        let alpha = compute_proximity_alpha(d);
        assert!(alpha < prev, "Alpha must strictly decrease: d={}, alpha={}, prev={}", d, alpha, prev);
        assert!(alpha > 0.0 && alpha < 1.0);
        prev = alpha;
    }
}

#[test]
fn test_driver_tier_tag() {
    assert_eq!(DriverTier::Rookie.tag(), "T1");
    assert_eq!(DriverTier::Amateur.tag(), "T2");
    assert_eq!(DriverTier::Contender.tag(), "T3");
    assert_eq!(DriverTier::Pro.tag(), "T4");
    assert_eq!(DriverTier::Legend.tag(), "T5");
}

#[test]
fn test_player_visibility_options_bot_nameplates_default_and_toggle() {
    let mut opts = PlayerVisibilityOptions::default();
    assert!(opts.bot_nameplates, "bot_nameplates should be enabled by default");

    // Toggle off
    opts.bot_nameplates = !opts.bot_nameplates;
    assert!(!opts.bot_nameplates);
    // Other options must remain intact
    assert!(opts.overhead_chevron);
    assert!(opts.ground_aura);
    assert!(opts.adaptive_visibility);
    assert!(opts.roof_beacon);
    assert!(opts.curve_helper);

    // Toggle on
    opts.bot_nameplates = !opts.bot_nameplates;
    assert!(opts.bot_nameplates);
}

#[test]
fn test_display_config_serde_roundtrip_and_defaults() {
    let default_cfg = DisplayConfig::default();
    assert!(default_cfg.bot_nameplates);

    // Deserializing without bot_nameplates field defaults to true
    let toml_str = r#"
        fullscreen = false
        target_fps = 60
        vsync = true
    "#;
    let loaded: DisplayConfig = toml::from_str(toml_str).unwrap();
    assert!(loaded.bot_nameplates, "Missing field in TOML must default to true");

    // Deserializing with bot_nameplates = false preserves false
    let toml_false = r#"
        fullscreen = false
        target_fps = 60
        vsync = true
        bot_nameplates = false
    "#;
    let loaded_false: DisplayConfig = toml::from_str(toml_false).unwrap();
    assert!(!loaded_false.bot_nameplates);

    // Roundtrip serialization
    let serialized = toml::to_string(&loaded_false).unwrap();
    assert!(serialized.contains("bot_nameplates = false"));
    let reloaded: DisplayConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(reloaded.bot_nameplates, loaded_false.bot_nameplates);
}

#[test]
fn test_deconflict_nameplates_proximity_culling() {
    let fonts = Fonts::load_embedded();
    let camera = RaceCamera::from_config_with_viewport(&CameraConfig::default(), 1280.0, 720.0);

    let items = vec![
        VehicleNameplateItem {
            car_idx: 1,
            name: "Vortex",
            tier_label: Some("T2"),
            accent_color: Color::new(1.0, 0.2, 0.2, 1.0),
            position: Vec2::new(0.0, 0.0),
            elevation: 0.0,
            distance_to_player: 15.0, // Close: inside R_inner (25m)
        },
        VehicleNameplateItem {
            car_idx: 2,
            name: "Blaze",
            tier_label: Some("T3"),
            accent_color: Color::new(0.2, 1.0, 0.2, 1.0),
            position: Vec2::new(0.0, 20.0),
            elevation: 0.0,
            distance_to_player: 60.0, // Far: outside R_outer (55m) -> culled
        },
    ];

    let deconflicted = deconflict_nameplates(&items, &camera, None, &fonts, 1.0);
    assert_eq!(deconflicted.len(), 1, "Far item outside 55m must be culled");
    assert_eq!(deconflicted[0].item.name, "Vortex");
    assert_eq!(deconflicted[0].alpha, 1.0);
}

#[test]
fn test_deconflict_nameplates_anti_crowding_stacking() {
    let fonts = Fonts::load_embedded();
    let camera = RaceCamera::from_config_with_viewport(&CameraConfig::default(), 1280.0, 720.0);

    // 3 cars situated at identical or nearly identical positions on screen
    let items = vec![
        VehicleNameplateItem {
            car_idx: 1,
            name: "CarA",
            tier_label: Some("T1"),
            accent_color: Color::new(1.0, 0.0, 0.0, 1.0),
            position: Vec2::new(0.0, 0.0),
            elevation: 0.0,
            distance_to_player: 10.0, // Closest
        },
        VehicleNameplateItem {
            car_idx: 2,
            name: "CarB",
            tier_label: Some("T2"),
            accent_color: Color::new(0.0, 1.0, 0.0, 1.0),
            position: Vec2::new(0.0, 0.1),
            elevation: 0.0,
            distance_to_player: 12.0, // Overlaps CarA
        },
        VehicleNameplateItem {
            car_idx: 3,
            name: "CarC",
            tier_label: Some("T3"),
            accent_color: Color::new(0.0, 0.0, 1.0, 1.0),
            position: Vec2::new(0.0, 0.2),
            elevation: 0.0,
            distance_to_player: 14.0, // Overlaps CarA and CarB
        },
    ];

    let deconflicted = deconflict_nameplates(&items, &camera, None, &fonts, 1.0);
    assert_eq!(deconflicted.len(), 3);

    // CarA should be at base screen position (closest)
    let y_a = deconflicted[0].screen_center.y;
    assert_eq!(deconflicted[0].item.name, "CarA");
    assert_eq!(deconflicted[0].alpha, 1.0);

    // CarB should be nudged upward by NAMEPLATE_STACK_NUDGE (20px)
    let y_b = deconflicted[1].screen_center.y;
    assert_eq!(deconflicted[1].item.name, "CarB");
    assert!((y_b - (y_a - NAMEPLATE_STACK_NUDGE)).abs() < 1.0, "CarB y should be nudged: got {}, expected {}", y_b, y_a - NAMEPLATE_STACK_NUDGE);
    assert_eq!(deconflicted[1].alpha, 1.0);

    // CarC is 3rd stack level (stack_level >= 2): nudged further up and alpha dimmed by 0.6
    let y_c = deconflicted[2].screen_center.y;
    assert_eq!(deconflicted[2].item.name, "CarC");
    assert!(y_c < y_b, "CarC must be stacked above CarB");
    assert!((deconflicted[2].alpha - 0.6).abs() < 1e-3, "3rd stack level alpha must be dimmed to 0.6, got {}", deconflicted[2].alpha);
}

#[test]
fn test_deconflict_nameplates_viewport_frustum_culling() {
    let fonts = Fonts::load_embedded();
    let camera = RaceCamera::from_config_with_viewport(&CameraConfig::default(), 1280.0, 720.0);

    // Viewport bounds: x: 100..400, y: 100..400
    let vp = Some((100.0, 100.0, 300.0, 300.0));

    // Car positioned very far in world coords so its screen projection is outside the viewport
    let items = vec![
        VehicleNameplateItem {
            car_idx: 1,
            name: "OutOffscreen",
            tier_label: None,
            accent_color: Color::new(1.0, 1.0, 1.0, 1.0),
            position: Vec2::new(5000.0, 5000.0), // Far outside
            elevation: 0.0,
            distance_to_player: 10.0, // Distance within 25m, but off-screen
        },
    ];

    let deconflicted = deconflict_nameplates(&items, &camera, vp, &fonts, 1.0);
    assert!(deconflicted.is_empty(), "Off-screen item must be culled by viewport frustum bounds");
}

#[test]
fn test_deconflict_nameplates_master_alpha_fade() {
    let fonts = Fonts::load_embedded();
    let camera = RaceCamera::from_config_with_viewport(&CameraConfig::default(), 1280.0, 720.0);

    let items = vec![VehicleNameplateItem {
        car_idx: 1,
        name: "Shadow",
        tier_label: None,
        accent_color: Color::new(0.5, 0.5, 0.5, 1.0),
        position: Vec2::new(0.0, 0.0),
        elevation: 0.0,
        distance_to_player: 15.0,
    }];

    // Near zero master alpha -> empty
    let empty = deconflict_nameplates(&items, &camera, None, &fonts, 0.0);
    assert!(empty.is_empty());

    // 0.5 master alpha -> scaled
    let half = deconflict_nameplates(&items, &camera, None, &fonts, 0.5);
    assert_eq!(half.len(), 1);
    assert!((half[0].alpha - 0.5).abs() < 1e-3);
}

#[test]
fn test_race_session_nameplate_initialization() {
    // Default session
    let session = RaceSession::new();
    assert!(session.visibility_options.bot_nameplates);

    // Custom config with bot_nameplates = false
    let mut config = GameConfig::default();
    config.display.bot_nameplates = false;
    let custom_session = RaceSession::new_with_config(config);
    assert!(!custom_session.visibility_options.bot_nameplates);
}

#[test]
fn test_render_floating_bot_nameplates_headless_execution() {
    let fonts = Fonts::load_embedded();
    let camera = RaceCamera::from_config_with_viewport(&CameraConfig::default(), 1280.0, 720.0);

    let items = vec![
        VehicleNameplateItem {
            car_idx: 1,
            name: "Apex",
            tier_label: Some("T1"),
            accent_color: Color::new(1.0, 0.5, 0.0, 1.0),
            position: Vec2::new(10.0, 10.0),
            elevation: 0.5,
            distance_to_player: 12.0,
        },
        VehicleNameplateItem {
            car_idx: 2,
            name: "Viper",
            tier_label: Some("T5"),
            accent_color: Color::new(0.2, 0.8, 1.0, 1.0),
            position: Vec2::new(15.0, 15.0),
            elevation: 0.0,
            distance_to_player: 35.0,
        },
    ];

    for &alpha in &[0.0f32, 0.5, 1.0] {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_floating_bot_nameplates(&fonts, &camera, None, &items, alpha);
            render_floating_bot_nameplates(&fonts, &camera, Some((0.0, 0.0, 640.0, 720.0)), &items, alpha);
        }));
    }
}
