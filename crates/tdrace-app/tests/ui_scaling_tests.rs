use tdrace_app::ui::scaler::UiScaler;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_ui_scaler_reference_resolution() {
    let scaler = UiScaler::new(1280.0, 720.0);
    assert!((scaler.scale - 1.0).abs() < 1e-4);
    assert_eq!(scaler.s(100.0), 100.0);
    assert_eq!(scaler.font_s(20.0), 20.0);
    assert!(!scaler.is_mobile_aspect);
}

#[test]
fn test_ui_scaler_mobile_aspect_and_safe_padding() {
    // iPhone 14 / modern Android (e.g. 844 x 390, ~2.16 aspect ratio)
    let mobile_scaler = UiScaler::new(844.0, 390.0);
    assert!(mobile_scaler.is_mobile_aspect);
    assert!(mobile_scaler.safe_pad_x >= 16.0);
    assert!(mobile_scaler.safe_pad_y >= 14.0);

    // Font size should never shrink below legible threshold (11.0 minimum)
    assert!(mobile_scaler.font_s(8.0) >= 11.0);

    // Touch targets must satisfy minimum 44dp mobile standard
    assert!(mobile_scaler.touch_target(20.0) >= UiScaler::MIN_TOUCH_SIZE);
}

#[test]
fn test_ui_scaler_4k_and_large_displays() {
    // 4K UHD display (3840 x 2160)
    let scaler_4k = UiScaler::new(3840.0, 2160.0);
    assert!(scaler_4k.scale >= 2.0);
    assert!(scaler_4k.s(50.0) >= 100.0);
}

#[test]
fn test_car_and_track_choices_metadata() {
    for track in &TrackChoice::ALL {
        assert!(!track.title().is_empty());
        assert!(!track.tag().is_empty());
        assert!(!track.description().is_empty());
    }

    for car in &CarChoice::ALL {
        assert!(!car.title().is_empty());
        assert!(!car.tag().is_empty());
        assert!(!car.description().is_empty());

        let (spd, acc, grp, dft) = car.stats();
        assert!(spd > 0.0 && spd <= 1.0);
        assert!(acc > 0.0 && acc <= 1.0);
        assert!(grp > 0.0 && grp <= 1.0);
        assert!(dft > 0.0 && dft <= 1.0);

        let (spec1, spec2, spec3, spec4) = car.specs();
        assert!(!spec1.is_empty());
        assert!(!spec2.is_empty());
        assert!(!spec3.is_empty());
        assert!(!spec4.is_empty());
    }
}

#[test]
fn test_resolve_predefined_car_for_track_and_modules() {
    use tdrace_app::ui::menu::resolve_predefined_car_for_track;
    use tdrace_core::track::presets;

    let f1_track = presets::classic_grand_prix();
    assert_eq!(resolve_predefined_car_for_track(Some(&f1_track), "classic"), CarChoice::SportsCar);

    let oasis = presets::oasis_rally();
    assert_eq!(resolve_predefined_car_for_track(Some(&oasis), "rally"), CarChoice::RallyCar);

    let kart = presets::kart_arena();
    assert_eq!(resolve_predefined_car_for_track(Some(&kart), "kart"), CarChoice::Kart);

    let drift = presets::drift_park();
    assert_eq!(resolve_predefined_car_for_track(Some(&drift), "classic"), CarChoice::DriftCar);

    // Module fallbacks when track is None
    assert_eq!(resolve_predefined_car_for_track(None, "f1"), CarChoice::F1Car);
    assert_eq!(resolve_predefined_car_for_track(None, "rally"), CarChoice::RallyCar);
    assert_eq!(resolve_predefined_car_for_track(None, "kart"), CarChoice::Kart);
    assert_eq!(resolve_predefined_car_for_track(None, "classic"), CarChoice::SportsCar);
}

#[test]
fn test_game_mode_specifications_and_transitions() {
    use tdrace_app::ui::menu::GameMode;

    // 1. TimeTrial
    let tt = GameMode::TimeTrial;
    assert_eq!(tt.title(), "Time Trial");
    assert!(tt.allows_car_change());
    assert!(tt.has_ghost());
    assert!(!tt.has_bots());
    assert!(tt.is_time_attack());
    assert_eq!(tt.next(), GameMode::FreeRide);

    // 2. FreeRide
    let fr = GameMode::FreeRide;
    assert_eq!(fr.title(), "Free Ride");
    assert!(fr.allows_car_change());
    assert!(!fr.has_ghost());
    assert!(!fr.has_bots());
    assert!(fr.is_time_attack());
    assert_eq!(fr.next(), GameMode::StandardRace);

    // 3. StandardRace
    let sr = GameMode::StandardRace;
    assert_eq!(sr.title(), "Standard Race");
    assert!(!sr.allows_car_change());
    assert!(!sr.has_ghost());
    assert!(sr.has_bots());
    assert!(!sr.is_time_attack());
    assert_eq!(sr.next(), GameMode::ExperimentalRace);

    // 4. ExperimentalRace
    let er = GameMode::ExperimentalRace;
    assert_eq!(er.title(), "Experimental Race");
    assert!(er.allows_car_change());
    assert!(!er.has_ghost());
    assert!(er.has_bots());
    assert!(!er.is_time_attack());
    assert_eq!(er.next(), GameMode::TimeTrial);
}

#[test]
fn test_race_session_game_mode_roster_behaviors() {
    use tdrace_app::game::RaceSession;
    use tdrace_app::ui::menu::GameMode;

    let mut session = RaceSession::new();
    session.num_bots = 4;

    // 1. Standard Race: all drivers use track's predefined car
    session.game_mode = GameMode::StandardRace;
    session.free_car_selection = false;
    session.car_choice = CarChoice::DriftCar; // Ignored in StandardRace
    session.init_race();

    assert_eq!(session.cars.len(), 5);
    let pred_car = session.resolve_predefined_car();
    assert_eq!(session.active_player_car_choice(), pred_car);
    for p in &session.grid_participants {
        assert_eq!(p.car_title, pred_car.title());
    }

    // 2. Experimental Race: all drivers use player-selected car
    session.game_mode = GameMode::ExperimentalRace;
    session.free_car_selection = true;
    session.car_choice = CarChoice::DriftCar;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 5);
    assert_eq!(session.active_player_car_choice(), CarChoice::DriftCar);
    for p in &session.grid_participants {
        assert_eq!(p.car_title, CarChoice::DriftCar.title());
    }

    // 3. Time Trial: solo car (plus shadow ghost telemetry)
    session.game_mode = GameMode::TimeTrial;
    session.is_time_attack = true;
    session.free_car_selection = true;
    session.car_choice = CarChoice::F1Car;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 1);
    assert_eq!(session.active_player_car_choice(), CarChoice::F1Car);
    assert_eq!(session.grid_participants.len(), 1);
    assert!(session.grid_participants[0].is_player);

    // 4. Free Ride: solo practice car
    session.game_mode = GameMode::FreeRide;
    session.is_time_attack = true;
    session.free_car_selection = true;
    session.car_choice = CarChoice::RallyCar;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 1);
    assert_eq!(session.active_player_car_choice(), CarChoice::RallyCar);
    assert_eq!(session.grid_participants.len(), 1);
    assert!(session.grid_participants[0].is_player);
}

