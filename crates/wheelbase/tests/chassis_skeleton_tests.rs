use glam::Vec2;
use wheelbase::config::{CarConfig, ChassisSkeleton};

#[test]
fn test_geometric_center_offset_from_cg() {
    let chassis = ChassisSkeleton::new(0.88, 1.18, 2.04, -0.40, 0.30, 0.65, 0.70, 0.05);
    let lf = 1.30;
    let lr = 1.40;

    let offset = chassis.geometric_center_offset_from_cg(lf, lr);
    // front_extent = 1.30 + 0.88 = 2.18
    // rear_extent = 1.40 + 1.18 = 2.58
    // offset = (2.18 - 2.58) / 2 = -0.20
    assert!((offset - (-0.20)).abs() < 1e-5, "Expected -0.20, got {}", offset);

    let total_len = chassis.total_length(lf + lr);
    assert!((total_len - 4.76).abs() < 1e-5, "Expected 4.76, got {}", total_len);
    assert!((chassis.half_length(lf + lr) - 2.38).abs() < 1e-5);
    assert!((chassis.half_width() - 1.02).abs() < 1e-5);
}

#[test]
fn test_light_positions_world() {
    let chassis = ChassisSkeleton::new(0.80, 0.90, 1.80, -0.40, 0.30, 0.65, 0.70, 0.05);
    let pos = Vec2::new(10.0, 20.0);
    let fwd = Vec2::new(0.0, 1.0); // Facing +Y
    let right = Vec2::new(1.0, 0.0); // Right +X
    let lf = 1.20;
    let lr = 1.20;

    let (hl_left, hl_right) = chassis.headlight_positions_world(pos, fwd, right, lf);
    // front_tip = (10, 20) + (0, 1) * (1.20 + 0.80 - 0.05) = (10, 21.95)
    // half_spread = 0.90 * 0.65 = 0.585
    // hl_left = (10 - 0.585, 21.95) = (9.415, 21.95)
    // hl_right = (10 + 0.585, 21.95) = (10.585, 21.95)
    assert!((hl_left.x - 9.415).abs() < 1e-4);
    assert!((hl_left.y - 21.95).abs() < 1e-4);
    assert!((hl_right.x - 10.585).abs() < 1e-4);
    assert!((hl_right.y - 21.95).abs() < 1e-4);

    let (tl_left, tl_right) = chassis.taillight_positions_world(pos, fwd, right, lr);
    // rear_tip = (10, 20) - (0, 1) * (1.20 + 0.90 - 0.05) = (10, 17.95)
    // half_spread = 0.90 * 0.70 = 0.63
    // tl_left = (10 - 0.63, 17.95) = (9.37, 17.95)
    // tl_right = (10 + 0.63, 17.95) = (10.63, 17.95)
    assert!((tl_left.x - 9.37).abs() < 1e-4);
    assert!((tl_left.y - 17.95).abs() < 1e-4);
    assert!((tl_right.x - 10.63).abs() < 1e-4);
    assert!((tl_right.y - 17.95).abs() < 1e-4);
}

#[test]
fn test_to_body_hull() {
    let chassis = ChassisSkeleton::new(0.80, 0.90, 1.80, -0.40, 0.30, 0.65, 0.70, 0.05);
    let (front, rear, half_w) = chassis.to_body_hull(1.10, 1.30);
    assert!((front - 1.90).abs() < 1e-5);
    assert!((rear - 2.20).abs() < 1e-5);
    assert!((half_w - 0.90).abs() < 1e-5);
}

#[test]
fn test_serde_synthesis_when_chassis_omitted() {
    let sports = CarConfig::sports_car();
    let mut serialized = serde_json::to_value(&sports).expect("serialize sports car");

    // Remove chassis field to simulate legacy payload
    if let serde_json::Value::Object(ref mut map) = serialized {
        map.remove("chassis");
    }

    let deserialized: CarConfig = serde_json::from_value(serialized).expect("deserialize legacy car");
    // Verify synthesized chassis dimensions are proportional to wheelbase (2.40) and track_width (1.40)
    assert!(deserialized.chassis.front_overhang > 0.5);
    assert!(deserialized.chassis.rear_overhang > 0.5);
    assert!((deserialized.chassis.body_width - (1.40 + 0.30)).abs() < 1e-4);
}

#[test]
fn test_all_presets_have_positive_chassis_bounds() {
    let presets = [
        CarConfig::sports_car(),
        CarConfig::drift_car(),
        CarConfig::kart(),
        CarConfig::rally_car(),
        CarConfig::stock_car_ta1(),
        CarConfig::sand_rail(),
        CarConfig::cross_car(),
        CarConfig::touring_ax(),
        CarConfig::super_buggy(),
        CarConfig::dune_buggy_baja(),
        CarConfig::trophy_truck(),
        CarConfig::mud_bogger(),
        CarConfig::monster_truck(),
        CarConfig::rally_junior_fwd(),
        CarConfig::rally_group_b(),
        CarConfig::rally_electric_rx(),
        CarConfig::superkart_gp(),
        CarConfig::stock_car_truck(),
    ];

    for cfg in presets {
        assert!(cfg.chassis.front_overhang > 0.0, "front overhang must be positive");
        assert!(cfg.chassis.rear_overhang > 0.0, "rear overhang must be positive");
        assert!(cfg.chassis.body_width > cfg.track_width * 0.5, "body width must be realistic");
        assert!(cfg.chassis.total_length(cfg.wheelbase) > cfg.wheelbase, "total length must exceed wheelbase");
    }
}

#[test]
fn test_modality_chassis_platforms_spec_094() {
    use wheelbase::config::{EnginePlacement, SuspensionArchetype};

    // Scenario 2: Touring AX geometry
    let touring = CarConfig::touring_ax();
    assert!(touring.track_width >= 1.85, "Touring AX track width must be >= 1.85m");
    assert!(touring.chassis.front_overhang >= 0.80, "Touring AX front overhang must be >= 0.80m");
    assert_eq!(touring.engine_placement, EnginePlacement::FrontEngine);

    // Scenario 3: Volkskraft Dune Buggy vs Sand Rail geometry
    let dune = CarConfig::dune_buggy_baja();
    let rail = CarConfig::sand_rail();
    assert!(dune.chassis.front_overhang >= 0.40, "Dune Buggy Baja front overhang must be >= 0.40m");
    assert!(dune.chassis.rear_overhang >= 0.50, "Dune Buggy Baja rear overhang must be >= 0.50m");
    assert_eq!(dune.engine_placement, EnginePlacement::RearEngine);
    assert!(rail.chassis.front_overhang <= 0.18, "Sand Rail needle nose front overhang must be <= 0.18m");

    // Scenario 4: Trophy Truck and Monster Truck collision hulls
    let trophy = CarConfig::trophy_truck();
    let trophy_hull_len = trophy.chassis.total_length(trophy.wheelbase);
    assert!(trophy_hull_len >= 5.0, "Trophy Truck total length must be >= 5.0m (was {})", trophy_hull_len);

    let monster = CarConfig::monster_truck();
    assert!(monster.chassis.body_width >= 2.6, "Monster Truck body width must be >= 2.6m");
    let sand_rail_len = rail.chassis.total_length(rail.wheelbase);
    assert!(trophy_hull_len > sand_rail_len + 1.5, "Trophy truck must not be constrained by sand rail hull");

    // Other modality archetypes
    let rally_jr = CarConfig::rally_junior_fwd();
    assert_eq!(rally_jr.drive_bias, 1.0);
    assert_eq!(rally_jr.engine_placement, EnginePlacement::FrontEngine);
    assert_eq!(rally_jr.suspension.front.archetype, SuspensionArchetype::MacPhersonStrut);

    let group_b = CarConfig::rally_group_b();
    assert_eq!(group_b.drive_bias, 0.5);
    assert_eq!(group_b.engine_placement, EnginePlacement::MidEngine);

    let superkart = CarConfig::superkart_gp();
    assert!(superkart.downforce_coefficient >= 0.60);
    assert_eq!(superkart.suspension.front.archetype, SuspensionArchetype::RigidKart);

    let stock_truck = CarConfig::stock_car_truck();
    assert!(stock_truck.chassis.front_overhang >= 0.90);
    assert!(stock_truck.chassis.rear_overhang >= 1.20);
    assert_eq!(stock_truck.suspension.rear.archetype, SuspensionArchetype::SolidLiveAxle);
}
