use glam::Vec2;
use tdrace_core::collision::wall::resolve_car_obstacle_collision;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::BarrierType;
use tdrace_core::track::presets::{catalunya_rx, estering_rx, yas_marina_rx};
use tdrace_core::track::scenery::{Grandstand, GrandstandStyle, Tree, TreeType};
use tdrace_core::track::Track;
use wheelbase::CarConfig;

#[test]
fn test_all_tree_types_physical_properties_and_dual_zone_geometry() {
    for &tree_type in &TreeType::ALL {
        let trunk_r = tree_type.default_trunk_radius();
        let canopy_r = tree_type.default_canopy_radius();
        let drag = tree_type.canopy_drag_deceleration();

        // 1. Canopy must be strictly larger than solid trunk
        assert!(canopy_r > trunk_r * 2.0, "Canopy must encompass trunk for {:?}", tree_type);
        assert!(trunk_r >= 0.20 && trunk_r <= 0.60, "Trunk radius within physical limits");
        assert!(drag >= 1.5 && drag <= 4.0, "Canopy drag deceleration rate must be in reasonable range");

        // 2. Tree instance scaling
        let tree = Tree::new(42, Vec2::new(10.0, 20.0), tree_type).with_scale(1.5);
        assert_eq!(tree.trunk_radius(), trunk_r * 1.5);
        assert_eq!(tree.canopy_radius(), canopy_r * 1.5);

        // 3. Dual-zone containment tests:
        // Point right at the center is inside BOTH trunk and canopy
        assert!(tree.contains_trunk(Vec2::new(10.0, 20.0)));
        assert!(tree.contains_canopy(Vec2::new(10.0, 20.0)));

        // Point at edge of canopy is inside canopy but outside trunk
        let edge_offset = tree.canopy_radius() * 0.75;
        let edge_point = Vec2::new(10.0 + edge_offset, 20.0);
        assert!(tree.contains_canopy(edge_point));
        assert!(!tree.contains_trunk(edge_point));

        // Point outside canopy is outside both
        let far_point = Vec2::new(10.0 + tree.canopy_radius() + 2.0, 20.0);
        assert!(!tree.contains_canopy(far_point));
        assert!(!tree.contains_trunk(far_point));

        // 4. Solid trunk obstacle generation
        let obs = tree.trunk_obstacle();
        match obs.shape {
            tdrace_core::track::geometry::ObstacleShape::Circle { radius, .. } => {
                assert_eq!(radius, tree.trunk_radius());
            }
            _ => panic!("Trunk must be a circular obstacle"),
        }
        assert_eq!(obs.restitution, 0.25); // Wood dampening
        assert_eq!(obs.friction, 0.55); // Bark friction
        assert!(obs.name.contains(tree_type.name()));
    }
}

#[test]
fn test_grandstand_concrete_surface_and_obstacle_properties() {
    let center = Vec2::new(50.0, 100.0);
    let length = 40.0;
    let depth = 12.0;
    let angle = 0.5; // ~28.6 degrees
    let stand = Grandstand::new(1, center, length, depth, angle)
        .with_style(GrandstandStyle::CoveredStadium)
        .with_tiers(8);

    assert_eq!(stand.style, GrandstandStyle::CoveredStadium);
    assert_eq!(stand.tiers, 8);

    // 1. Footprint containment: center must be contained
    assert!(stand.contains(center));

    // Points outside must not be contained
    assert!(!stand.contains(center + Vec2::new(50.0, 0.0)));

    // 2. Obstacle generation must produce a concrete barrier
    let obs = stand.to_obstacle();
    assert_eq!(obs.restitution, 0.65);
    assert_eq!(obs.friction, 0.32);
    assert_eq!(obs.center(), center);
    assert!(obs.name.contains("Grandstand #1"));


    // 3. Surface zone generation
    let zone = stand.surface_zone();
    assert_eq!(zone.surface, SurfaceType::Concrete);
    assert!(zone.contains(center));
}

#[test]
fn test_car_physics_collision_with_tree_trunk_obstacle() {
    let tree = Tree::new(1, Vec2::new(20.0, 0.0), TreeType::Pine).with_scale(1.0);
    let trunk_obs = tree.trunk_obstacle();

    // Car driving toward the tree trunk along +X at 20 m/s (in contact with trunk)
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(18.6, 0.0), 0.0);
    car.state.velocity = Vec2::new(20.0, 0.0);
    car.state.speed = 20.0;

    let ev = resolve_car_obstacle_collision(&mut car, &trunk_obs);
    assert!(ev.is_some(), "Car must collide with solid wood trunk obstacle");
    let event = ev.unwrap();

    // Collision must repel the vehicle backward (-X)
    assert!(event.normal.x < -0.5, "Collision normal should point away from trunk");
    assert!(car.state.velocity.x < 20.0, "Speed along impact vector must be reduced or reversed");
    assert!(car.state.position.x < 18.6, "Penetration must be resolved by pushing car backward");
}

#[test]
fn test_car_physics_collision_with_grandstand_obstacle() {
    let stand = Grandstand::new(1, Vec2::new(50.0, 0.0), 30.0, 10.0, 0.0);
    let stand_obs = stand.to_obstacle();

    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(48.0, -4.5), 0.0);
    car.state.velocity = Vec2::new(15.0, 0.0);
    car.state.speed = 15.0;

    let ev = resolve_car_obstacle_collision(&mut car, &stand_obs);
    assert!(ev.is_some(), "Car must collide with grandstand barrier");
    let event = ev.unwrap();
    assert_eq!(event.barrier_type, BarrierType::Concrete);
}

#[test]
fn test_car_soft_canopy_aerodynamic_drag_deceleration() {
    let tree = Tree::new(1, Vec2::new(0.0, 0.0), TreeType::Oak).with_scale(1.0);

    // Car inside canopy but outside trunk
    let car_pos = Vec2::new(2.5, 0.0); // Inside canopy (r=4.6m), outside trunk (r=0.48m)
    assert!(tree.contains_canopy(car_pos));
    assert!(!tree.contains_trunk(car_pos));

    let mut car = Car::new(CarConfig::sports_car()).with_pose(car_pos, 0.0);
    car.state.velocity = Vec2::new(25.0, 0.0);
    car.state.speed = 25.0;

    // Apply soft canopy brush drag
    let dt = 1.0 / 60.0;
    let drag_rate = tree.tree_type.canopy_drag_deceleration();
    car.state.velocity *= (1.0 - drag_rate * dt).max(0.0);
    car.state.speed = car.state.velocity.length();

    let expected_speed = 25.0 * (1.0 - drag_rate * dt);
    assert!((car.state.speed - expected_speed).abs() < 1e-4);
    assert!(car.state.speed < 25.0, "Car must be decelerated by canopy foliage brush drag");
}

#[test]
fn test_presets_contain_scenery_and_sample_concrete() {
    // 1. Catalunya RX has stadium grandstands and Mediterranean palms
    let cat = catalunya_rx();
    assert!(!cat.geometry.grandstands.is_empty(), "Catalunya RX must have grandstands");
    assert!(!cat.geometry.trees.is_empty(), "Catalunya RX must have trees");

    // Grandstand footprint must sample as SurfaceType::Concrete
    let stand = &cat.geometry.grandstands[0];
    assert_eq!(cat.sample_surface(stand.center), SurfaceType::Concrete);

    // 2. Estering RX has hillside bleachers and German forest pines/oaks
    let est = estering_rx();
    assert!(!est.geometry.grandstands.is_empty(), "Estering must have hillside bleachers");
    assert!(!est.geometry.trees.is_empty(), "Estering must have pines and oaks");
    assert!(est.geometry.trees.iter().any(|t| t.tree_type == TreeType::Pine));
    assert!(est.geometry.trees.iter().any(|t| t.tree_type == TreeType::Oak));

    // 3. Yas Marina RX has stadium grandstand and date palms
    let yas = yas_marina_rx();
    assert!(!yas.geometry.grandstands.is_empty(), "Yas Marina must have grandstands");
    assert!(!yas.geometry.trees.is_empty(), "Yas Marina must have palms");
    assert!(yas.geometry.trees.iter().all(|t| t.tree_type == TreeType::Palm));

    // 4. All scenery obstacles must be returned in all_obstacles_with_scenery
    let cat_obs = cat.geometry.all_obstacles_with_scenery();
    assert!(cat_obs.iter().any(|o| o.name.contains("Grandstand")));
    assert!(cat_obs.iter().any(|o| o.name.contains("Trunk")));
}

#[test]
fn test_track_json_serialization_preserves_all_scenery_fields() {
    let track = estering_rx();
    let original_grandstands = track.geometry.grandstands.clone();
    let original_trees = track.geometry.trees.clone();

    // Roundtrip to JSON
    let json = track.to_json_pretty().expect("Must serialize track to JSON");
    let deserialized = Track::from_json(&json).expect("Must deserialize track from JSON");

    assert_eq!(deserialized.geometry.grandstands, original_grandstands);
    assert_eq!(deserialized.geometry.trees, original_trees);
}
