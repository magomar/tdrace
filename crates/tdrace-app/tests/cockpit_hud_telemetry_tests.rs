use race_ui::hud::widgets::{
    compute_ackermann_steer_angles, ChassisHudGeometry, CockpitTelemetryMode,
};
use tdrace_app::config::GameConfig;
use tdrace_app::game::RaceSession;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::{
    CarConfig, ChassisSkeleton, EnginePlacement, SuspensionArchetype,
};

#[test]
fn test_cockpit_telemetry_mode_toggle() {
    let mut mode = CockpitTelemetryMode::default();
    assert_eq!(mode, CockpitTelemetryMode::KinematicDamage);
    assert_eq!(mode.label(), "KINEMATICS");

    mode = CockpitTelemetryMode::DynamicTelemetry;
    assert_eq!(mode.label(), "DYNAMICS");

    // Toggle back
    mode = CockpitTelemetryMode::KinematicDamage;
    assert_eq!(mode.label(), "KINEMATICS");
}

#[test]
fn test_cockpit_telemetry_config_defaults_and_session_init() {
    let mut config = GameConfig::default();
    assert_eq!(
        config.gameplay.default_cockpit_telemetry_mode,
        "kinematic_damage"
    );

    // Initial session should have KinematicDamage
    let session = RaceSession::new_with_config(config.clone());
    assert_eq!(
        session.cockpit_telemetry_mode,
        CockpitTelemetryMode::KinematicDamage
    );

    // Change to dynamic_telemetry
    config.gameplay.default_cockpit_telemetry_mode = "dynamic_telemetry".to_string();
    let session_dynamic = RaceSession::new_with_config(config);
    assert_eq!(
        session_dynamic.cockpit_telemetry_mode,
        CockpitTelemetryMode::DynamicTelemetry
    );
}

#[test]
fn test_proportional_hud_bounds_all_8_chassis() {
    let box_w = 190.0;
    let box_h = 220.0;

    let chassis_roster: [(&str, f32, f32, f32, f32, EnginePlacement, SuspensionArchetype); 8] = [
        ("125cc Shifter Kart", 1.05, 1.40, 0.32, 0.35, EnginePlacement::MidEngine, SuspensionArchetype::RigidKart),
        ("GT4 Sports Coupe", 2.40, 1.80, 0.80, 0.90, EnginePlacement::FrontEngine, SuspensionArchetype::MacPhersonStrut),
        ("GT3 Touring (RR)", 2.51, 2.04, 0.88, 1.15, EnginePlacement::RearEngine, SuspensionArchetype::DoubleWishbone),
        ("NASCAR Stock Car", 2.79, 1.95, 0.98, 1.12, EnginePlacement::FrontEngine, SuspensionArchetype::SolidLiveAxle),
        ("Rallycross Supercar", 2.48, 1.88, 0.75, 0.70, EnginePlacement::FrontEngine, SuspensionArchetype::MacPhersonStrut),
        ("Dakar Sand Rail", 2.40, 1.95, 0.15, 0.42, EnginePlacement::RearEngine, SuspensionArchetype::LongTravelOffRoad),
        ("Autocross CrossCar", 2.10, 1.62, 0.28, 0.38, EnginePlacement::MidEngine, SuspensionArchetype::DoubleWishbone),
        ("Le Mans Hypercar", 3.15, 2.05, 1.05, 0.95, EnginePlacement::MidEngine, SuspensionArchetype::PushrodInboard),
    ];

    for (name, wb, width, fo, ro, placement, arch) in chassis_roster {
        let mut cfg = CarConfig::sports_car();
        cfg.wheelbase = wb;
        cfg.track_width = width * 0.88;
        cfg.engine_placement = placement;
        cfg.suspension.front.archetype = arch;
        cfg.suspension.rear.archetype = arch;
        cfg.chassis = ChassisSkeleton {
            front_overhang: fo,
            rear_overhang: ro,
            body_width: width,
            cabin_start_offset: -0.40,
            cabin_end_offset: 0.30,
            headlight_spread: 0.65,
            taillight_spread: 0.70,
            light_inset: 0.05,
        };

        let car = Car::new(cfg);
        let geo = ChassisHudGeometry::compute(box_w, box_h, 0.0, 0.0, &car, 1.0);

        // Scale must be positive and front axle must sit above rear axle in HUD space
        assert!(geo.scale > 0.0, "{}: scale must be positive", name);
        assert!(
            geo.front_axle_y < geo.rear_axle_y,
            "{}: front axle must be above rear axle in HUD",
            name
        );

        // Verify strictly fits inside 190x220 HUD box
        assert!(
            geo.nose_y >= 0.0,
            "{}: nose_y ({:.1}) clipped above box",
            name,
            geo.nose_y
        );
        assert!(
            geo.tail_y <= box_h,
            "{}: tail_y ({:.1}) clipped below box",
            name,
            geo.tail_y
        );
        assert!(
            geo.outboard_left_x >= 0.0,
            "{}: outboard_left_x ({:.1}) clipped left",
            name,
            geo.outboard_left_x
        );
        assert!(
            geo.outboard_right_x + geo.wheel_w <= box_w,
            "{}: outboard right tire ({:.1}) clipped right of box ({:.1})",
            name,
            geo.outboard_right_x + geo.wheel_w,
            box_w
        );
    }
}

#[test]
fn test_ackermann_hud_steer_differential() {
    // Right turn (+0.35 rad): front-right (inner) must turn sharper than front-left (outer)
    let steer_right = 0.35;
    let (steer_fl_r, steer_fr_r) = compute_ackermann_steer_angles(steer_right);
    assert!(
        steer_fr_r > steer_fl_r,
        "Right turn: inner wheel FR ({}) must be sharper than outer FL ({})",
        steer_fr_r,
        steer_fl_r
    );
    assert!((steer_fr_r - (0.35 * 1.15)).abs() < 1e-4);
    assert!((steer_fl_r - (0.35 * 0.88)).abs() < 1e-4);

    // Left turn (-0.35 rad): front-left (inner) must turn sharper (more negative) than front-right (outer)
    let steer_left = -0.35;
    let (steer_fl_l, steer_fr_l) = compute_ackermann_steer_angles(steer_left);
    assert!(
        steer_fl_l < steer_fr_l,
        "Left turn: inner wheel FL ({}) must be sharper (more negative) than outer FR ({})",
        steer_fl_l,
        steer_fr_l
    );
    assert!((steer_fl_l - (-0.35 * 1.15)).abs() < 1e-4);
    assert!((steer_fr_l - (-0.35 * 0.88)).abs() < 1e-4);

    // Neutral steering
    let (fl_0, fr_0) = compute_ackermann_steer_angles(0.0);
    assert_eq!(fl_0, 0.0);
    assert_eq!(fr_0, 0.0);
}
