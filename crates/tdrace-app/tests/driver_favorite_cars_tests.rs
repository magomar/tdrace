use std::collections::HashSet;
use tdrace_app::ai::DriverCharacter;
use tdrace_app::catalog::{find_model_by_id, get_models_for_module};
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_app::module::extreme_offroad::ExtremeOffRoadModule;
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::kart::KartGameModule;
use tdrace_app::module::nascar::NascarGameModule;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::render::color::CarColorScheme;

#[test]
fn test_all_six_modules_have_exactly_twelve_drivers() {
    let classic = ClassicGameModule::new().drivers();
    let gt = GtWorldChallengeModule::new().drivers();
    let nascar = NascarGameModule::new().drivers();
    let rally = RallyGameModule::new().drivers();
    let kart = KartGameModule::new().drivers();
    let offroad = ExtremeOffRoadModule::new().drivers();

    assert_eq!(classic.len(), 12, "Classic module must have exactly 12 drivers");
    assert_eq!(gt.len(), 12, "GT module must have exactly 12 drivers");
    assert_eq!(nascar.len(), 12, "NASCAR module must have exactly 12 drivers");
    assert_eq!(rally.len(), 12, "Rally module must have exactly 12 drivers");
    assert_eq!(kart.len(), 12, "Kart module must have exactly 12 drivers");
    assert_eq!(offroad.len(), 12, "Extreme Off-Road module must have exactly 12 drivers");
    assert_eq!(DriverCharacter::ROSTER.len(), 12, "Core ROSTER must have exactly 12 drivers");
}

#[test]
fn test_all_seventy_two_driver_ids_are_unique() {
    let modules: Vec<(&str, Vec<DriverCharacter>)> = vec![
        ("classic", ClassicGameModule::new().drivers()),
        ("gt", GtWorldChallengeModule::new().drivers()),
        ("nascar", NascarGameModule::new().drivers()),
        ("rally", RallyGameModule::new().drivers()),
        ("kart", KartGameModule::new().drivers()),
        ("extreme_offroad", ExtremeOffRoadModule::new().drivers()),
    ];

    let mut seen_ids = HashSet::new();
    let mut total_count = 0;

    for (mod_name, drivers) in modules {
        assert_eq!(drivers.len(), 12, "Module {} should have 12 drivers", mod_name);
        for d in drivers {
            assert!(
                seen_ids.insert(d.id),
                "Duplicate driver ID found: '{}' in module '{}'",
                d.id,
                mod_name
            );
            assert!(!d.name.is_empty(), "Driver ID '{}' has empty name", d.id);
            assert!(!d.alias.is_empty(), "Driver ID '{}' has empty alias", d.id);
            assert!(!d.bio.is_empty(), "Driver ID '{}' has empty bio", d.id);
            total_count += 1;
        }
    }

    assert_eq!(total_count, 72, "Total pilots across 6 modules must equal 72");
    assert_eq!(seen_ids.len(), 72, "Must have exactly 72 unique pilot IDs");
}

#[test]
fn test_no_driver_uses_player_default_colors() {
    let player_scheme = CarColorScheme::from_index(0);
    let (p_hex, s_hex, h_hex) = player_scheme.to_hex_strings();

    let modules: Vec<(&str, Vec<DriverCharacter>)> = vec![
        ("classic", ClassicGameModule::new().drivers()),
        ("gt", GtWorldChallengeModule::new().drivers()),
        ("nascar", NascarGameModule::new().drivers()),
        ("rally", RallyGameModule::new().drivers()),
        ("kart", KartGameModule::new().drivers()),
        ("extreme_offroad", ExtremeOffRoadModule::new().drivers()),
    ];

    for (mod_name, drivers) in modules {
        let mut module_liveries = HashSet::new();
        for d in drivers {
            let (p, s, h) = d.color_scheme.to_hex_strings();
            assert_ne!(
                (p.clone(), s.clone(), h.clone()),
                (p_hex.clone(), s_hex.clone(), h_hex.clone()),
                "Driver '{}' in module '{}' must not use human player default colors (index 0)",
                d.name,
                mod_name
            );
            assert!(
                module_liveries.insert((p, s, h)),
                "Duplicate livery within module '{}' for driver '{}'",
                mod_name,
                d.name
            );
        }
    }
}

#[test]
fn test_classic_module_normalizes_all_tiers_to_tier_one() {
    // All classic vehicles in catalog must be Tier 1
    let classic_cars = get_models_for_module("classic");
    assert_eq!(classic_cars.len(), 5, "Classic module has 5 vehicles");
    for car in classic_cars {
        assert_eq!(
            car.tier, 1,
            "Classic car '{}' must have tier == 1, found {}",
            car.id, car.tier
        );
    }

    // Every core driver with a classic favorite car must resolve that car regardless of requested tier
    for driver in DriverCharacter::ROSTER {
        if let Some(t1_car) = driver.favorite_car_for_discipline_and_tier("classic", 1) {
            assert_eq!(
                driver.favorite_car_for_discipline_and_tier("classic", 2),
                Some(t1_car),
                "Driver '{}' should normalize tier 2 to tier 1 in classic",
                driver.name
            );
            assert_eq!(
                driver.favorite_car_for_discipline_and_tier("classic", 5),
                Some(t1_car),
                "Driver '{}' should normalize tier 5 to tier 1 in classic",
                driver.name
            );
        }
    }
}

#[test]
fn test_module_drivers_per_tier_favorite_cars_resolve_in_catalog() {
    let modules: Vec<(&str, Vec<DriverCharacter>)> = vec![
        ("gt", GtWorldChallengeModule::new().drivers()),
        ("nascar", NascarGameModule::new().drivers()),
        ("rally", RallyGameModule::new().drivers()),
        ("kart", KartGameModule::new().drivers()),
        ("extreme_offroad", ExtremeOffRoadModule::new().drivers()),
    ];

    for (mod_name, drivers) in modules {
        for driver in drivers {
            assert_eq!(
                driver.favorite_cars.len(),
                5,
                "Driver '{}' in module '{}' must have 5 favorite cars (one per tier)",
                driver.name,
                mod_name
            );

            for tier in 1..=5u8 {
                let model_id = driver.favorite_car_for_discipline_and_tier(mod_name, tier);
                assert!(
                    model_id.is_some(),
                    "Driver '{}' must have a favorite car for {} tier {}",
                    driver.name,
                    mod_name,
                    tier
                );

                let real_model = driver.favorite_model_for_discipline_and_tier(mod_name, tier);
                assert!(
                    real_model.is_some(),
                    "Favorite model ID '{:?}' for driver '{}' must exist in authentic catalog",
                    model_id,
                    driver.name
                );

                let model = real_model.unwrap();
                assert_eq!(
                    model.tier, tier,
                    "Favorite car '{}' for driver '{}' in {} must be tier {}, found {}",
                    model.id, driver.name, mod_name, tier, model.tier
                );

                // Verify effective_car_choice resolves to a valid CarChoice
                let car_choice = driver.effective_car_choice_for_discipline_and_tier(mod_name, tier);
                assert_eq!(car_choice, model.base_car_choice);
            }
        }
    }
}

#[test]
fn test_gt_tier_starting_grid_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.start_gt_career_tier(1);

    assert_eq!(session.state, GameState::StartingGrid);

    // Opponent participants should be assigned their signature GT4 favorite cars
    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 7);

    for p in &opponents {
        let driver = GtWorldChallengeModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known GT driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("gt", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "GT opponent '{}' should be assigned their signature GT4 car",
            p.name
        );
        let expected_model = find_model_by_id(expected_fav.unwrap()).unwrap();
        assert_eq!(p.car_title, expected_model.name);
    }
}

#[test]
fn test_nascar_starting_grid_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.start_nascar_career_tier(1);

    assert_eq!(session.state, GameState::StartingGrid);

    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 11);

    for p in &opponents {
        let driver = NascarGameModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known NASCAR driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("nascar", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "NASCAR opponent '{}' should be assigned their signature Street Stock car",
            p.name
        );
    }
}

#[test]
fn test_rally_starting_grid_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.start_rally_career_tier(1);

    assert_eq!(session.state, GameState::StartingGrid);

    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 7);

    for p in &opponents {
        let driver = RallyGameModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known Rally driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("rally", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "Rally opponent '{}' should be assigned their signature Junior RX car",
            p.name
        );
    }
}

#[test]
fn test_classic_free_car_selection_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.switch_to_classic();
    session.game_mode = tdrace_app::ui::menu::GameMode::Career;
    session.free_car_selection = true;
    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.state, GameState::StartingGrid);

    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 7);

    for p in &opponents {
        let driver = ClassicGameModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known Classic driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("classic", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "Classic opponent '{}' in free car selection should be assigned signature classic vehicle",
            p.name
        );
    }
}

#[test]
fn test_kart_starting_grid_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.start_kart_career_tier(1);

    assert_eq!(session.state, GameState::StartingGrid);

    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 7);

    for p in &opponents {
        let driver = KartGameModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known Kart driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("kart", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "Kart opponent '{}' should be assigned their signature Cadet 60cc car",
            p.name
        );
    }
}

#[test]
fn test_extreme_offroad_starting_grid_assigns_signature_cars() {
    let mut session = RaceSession::new();
    session.start_extreme_offroad_career_tier(1);

    assert_eq!(session.state, GameState::StartingGrid);

    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    assert_eq!(opponents.len(), 8);

    for p in &opponents {
        let driver = ExtremeOffRoadModule::new()
            .drivers()
            .into_iter()
            .find(|d| d.name == p.name)
            .unwrap_or_else(|| panic!("Opponent '{}' must be a known Off-Road driver", p.name));

        let expected_fav = driver.favorite_car_for_discipline_and_tier("extreme_offroad", 1);
        assert_eq!(
            p.model_id, expected_fav,
            "Extreme Off-Road opponent '{}' should be assigned their signature Sand Rail car",
            p.name
        );
    }
}

