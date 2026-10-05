use tdrace_app::catalog::{get_models_for_module, get_models_for_module_and_tier, find_model_by_id};
use tdrace_core::CarCategory;

#[test]
fn test_autocross_catalog_has_fifteen_vehicles() {
    let cars = get_models_for_module("autocross");
    assert_eq!(cars.len(), 15, "FIA Autocross module must feature exactly 15 authentic vehicles");

    for car in &cars {
        assert_eq!(car.module_id, "autocross");
        assert_eq!(car.category(), CarCategory::Autocross);
        assert!(car.id.starts_with("autocross_"), "ID must start with autocross_: {}", car.id);
        assert!((1..=5).contains(&car.tier), "Tier must be in 1..=5");
        assert!(!car.name.is_empty());
        assert!(!car.manufacturer.is_empty());
        assert!(car.bhp > 0);
        assert!(car.weight_kg > 0);
        assert!(car.top_speed_kmh > 0);
        assert!(car.accel_0_100 > 0.0);

        // Verify car physics configuration generates cleanly
        let config = car.to_car_config();
        assert!(config.mass > 0.0);
    }
}

#[test]
fn test_autocross_catalog_has_three_vehicles_per_tier() {
    let expected_categories = [
        (1, "Cross Car Junior", "RWD", 80),
        (2, "Cross Car Senior", "RWD", 140),
        (3, "Buggy1600", "4WD", 250),
        (4, "TouringAutocross", "4WD", 540),
        (5, "SuperBuggy", "4WD", 650),
    ];

    for (tier, category_name, expected_drivetrain, min_bhp) in expected_categories {
        let tier_cars = get_models_for_module_and_tier("autocross", tier);
        assert_eq!(
            tier_cars.len(),
            3,
            "Autocross Tier {} must contain exactly 3 vehicles",
            tier
        );

        for car in &tier_cars {
            assert_eq!(car.category_name, category_name);
            assert_eq!(car.drivetrain, expected_drivetrain);
            assert!(
                car.bhp >= min_bhp,
                "Car {} in Tier {} has {} BHP, expected >= {}",
                car.id,
                tier,
                car.bhp,
                min_bhp
            );
        }
    }
}

#[test]
fn test_autocross_specific_vehicle_identities() {
    let expected_ids = [
        // Tier 1
        "autocross_ardennes_junior_t1",
        "autocross_iberian_furia_t1",
        "autocross_cosmo_nova_t1",
        // Tier 2
        "autocross_ardennes_pro_t2",
        "autocross_iberian_relampago_t2",
        "autocross_lusitania_bravo_t2",
        // Tier 3
        "autocross_petersen_buggy1600_t3",
        "autocross_bologna_buggy1600_t3",
        "autocross_rapid_buggy1600_t3",
        // Tier 4
        "autocross_bohemia_veloce_t4",
        "autocross_shinano_tsunami_t4",
        "autocross_vortek_quattro_t4",
        // Tier 5
        "autocross_petersen_superbuggy_t5",
        "autocross_bologna_superbuggy_t5",
        "autocross_rapid_superbuggy_t5",
    ];

    for id in expected_ids {
        let model = find_model_by_id(id);
        assert!(model.is_some(), "Expected vehicle {} was not found in catalog", id);
        let m = model.unwrap();
        assert_eq!(m.module_id, "autocross");
    }
}
