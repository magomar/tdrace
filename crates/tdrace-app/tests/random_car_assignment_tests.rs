use std::collections::HashSet;
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::CarChoice;
use tdrace_app::module::VehicleVisualType;

#[test]
fn test_eligible_opponent_cars_per_category() {
    let mut session = RaceSession::new();

    // GT World Challenge
    session.switch_to_gt();
    let gt_pool = session.eligible_opponent_cars();
    assert_eq!(
        gt_pool,
        vec![
            CarChoice::GT4Clubsport,
            CarChoice::GT3Car,
            CarChoice::GT2Biturbo,
            CarChoice::GT1Legend,
            CarChoice::HypercarPrototype,
        ]
    );

    // NASCAR Cup
    session.switch_to_nascar();
    let nascar_pool = session.eligible_opponent_cars();
    assert_eq!(nascar_pool, vec![CarChoice::StockCar]);

    // Extreme Off-Road
    session.switch_to_extreme_offroad();
    let offroad_pool = session.eligible_opponent_cars();
    assert_eq!(offroad_pool, vec![CarChoice::SandRail]);

    // Rally
    session.switch_to_rally();
    let rally_pool = session.eligible_opponent_cars();
    assert_eq!(rally_pool, vec![CarChoice::RallyCar]);

    // Kart
    session.switch_to_kart();
    let kart_pool = session.eligible_opponent_cars();
    assert_eq!(kart_pool, vec![CarChoice::Kart]);

    // Classic / Default - constrained to track's CarCategory (single fantasy car per race)
    session.switch_to_classic();
    session.track.car_category = tdrace_core::CarCategory::Gt;
    assert_eq!(session.eligible_opponent_cars(), vec![CarChoice::SportsCar]);

    session.track.car_category = tdrace_core::CarCategory::Nascar;
    assert_eq!(session.eligible_opponent_cars(), vec![CarChoice::StockCar]);

    session.track.car_category = tdrace_core::CarCategory::Rally;
    assert_eq!(session.eligible_opponent_cars(), vec![CarChoice::RallyCar]);

    session.track.car_category = tdrace_core::CarCategory::Kart;
    assert_eq!(session.eligible_opponent_cars(), vec![CarChoice::Kart]);

    session.track.car_category = tdrace_core::CarCategory::OffRoad;
    assert_eq!(session.eligible_opponent_cars(), vec![CarChoice::SandRail]);
}

#[test]
fn test_random_car_assignment_variety_in_gt_race() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.car_visual_types.len(), 8);

    let opponent_cars: Vec<CarChoice> = session
        .grid_participants
        .iter()
        .filter(|p| !p.is_player)
        .map(|p| p.car_choice)
        .collect();

    assert_eq!(opponent_cars.len(), 7);

    let unique_cars: HashSet<CarChoice> = opponent_cars.into_iter().collect();
    // In a 7-bot GT race with 5 car options, there must be variety (> 1 distinct car model)
    assert!(
        unique_cars.len() >= 2,
        "Expected variety among GT opponents, but got: {:?}",
        unique_cars
    );

    // Verify all assigned cars belong to the GT pool
    let gt_pool = session.eligible_opponent_cars();
    for car in &unique_cars {
        assert!(gt_pool.contains(car), "Car {:?} must be in GT pool", car);
    }
}

#[test]
fn test_single_make_disciplines_preserve_uniform_car() {
    let mut session = RaceSession::new();
    session.switch_to_kart();
    session.num_bots = 5;
    session.init_race();

    for p in &session.grid_participants {
        assert_eq!(p.car_choice, CarChoice::Kart);
    }

    session.switch_to_nascar();
    session.num_bots = 5;
    session.init_race();

    for p in &session.grid_participants {
        assert_eq!(p.car_choice, CarChoice::StockCar);
    }
}

#[test]
fn test_random_car_assignment_toggle() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 6;
    session.random_car_assignment = false;
    session.init_race();

    let pred_car = session.resolve_predefined_car();
    for p in &session.grid_participants {
        assert_eq!(
            p.car_choice, pred_car,
            "When random_car_assignment is false, all drivers must use predefined car"
        );
    }
}

#[test]
fn test_car_visual_types_sync_with_grid_participants() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 5;
    session.init_race();

    assert_eq!(session.cars.len(), session.car_visual_types.len());
    for (i, car_visual) in session.car_visual_types.iter().enumerate() {
        match car_visual {
            VehicleVisualType::TouringGT { gt_wing, .. } => {
                assert!(gt_wing, "Car visual at slot {} must have GT wing", i);
            }
            _ => panic!("Expected TouringGT visual type for car {}", i),
        }
    }
}

#[test]
fn test_sample_random_opponent_car_determinism() {
    let session = RaceSession::new();
    let car_a = session.sample_random_opponent_car(2, 123456789);
    let car_b = session.sample_random_opponent_car(2, 123456789);
    assert_eq!(car_a, car_b, "Sampling with same bot_idx and seed must be deterministic");
}
