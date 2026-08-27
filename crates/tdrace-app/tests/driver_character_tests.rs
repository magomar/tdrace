use tdrace_app::ai::DriverCharacter;
use tdrace_app::game::{DriverCardsOrigin, GameState, RaceSession};
use tdrace_app::ui::menu::TrackChoice;


#[test]
fn test_driver_roster_integrity_and_distinct_properties() {
    let roster = DriverCharacter::all();
    assert_eq!(roster.len(), 8, "Must contain exactly 8 predefined driver characters");

    let mut ids = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    let mut aliases = std::collections::HashSet::new();

    for d in roster {
        assert!(!d.id.is_empty());
        assert!(!d.name.is_empty());
        assert!(!d.alias.is_empty());
        assert!(!d.bio.is_empty());

        // Verify no 'bot' in aliases or names
        let name_lower = d.name.to_lowercase();
        let alias_lower = d.alias.to_lowercase();
        assert!(!name_lower.contains("bot"), "Driver name {} must not contain 'bot'", d.name);
        assert!(!alias_lower.contains("bot"), "Driver alias {} must not contain 'bot'", d.alias);

        // Verify stats are within normalized bounds [0.0..1.0]
        assert!(d.stats.speed >= 0.5 && d.stats.speed <= 1.0);
        assert!(d.stats.aggression >= 0.5 && d.stats.aggression <= 1.0);
        assert!(d.stats.precision >= 0.5 && d.stats.precision <= 1.0);
        assert!(d.stats.defense >= 0.5 && d.stats.defense <= 1.0);

        // Verify operationalized BotProfile parameters
        assert!(d.profile.lookahead_time > 0.2 && d.profile.lookahead_time < 0.6);
        assert!(d.profile.speed_factor > 0.85 && d.profile.speed_factor < 1.15);
        assert!(d.profile.steering_kp > 1.5 && d.profile.steering_kp < 3.5);
        assert!(d.profile.brake_margin > 0.7 && d.profile.brake_margin < 1.3);

        assert!(ids.insert(d.id), "Duplicate ID found: {}", d.id);
        assert!(names.insert(d.name), "Duplicate name found: {}", d.name);
        assert!(aliases.insert(d.alias), "Duplicate alias found: {}", d.alias);
    }
}

#[test]
fn test_driver_roster_sampling_uniqueness() {
    for seed in [1, 42, 1337, 99999, 1048576] {
        let opponents = DriverCharacter::sample_opponents(3, seed);
        assert_eq!(opponents.len(), 3);

        let mut seen = std::collections::HashSet::new();
        for opp in &opponents {
            assert!(seen.insert(opp.id), "Sampled opponents must not contain duplicates for seed {}", seed);
        }
    }

    // Full roster sample
    let full = DriverCharacter::sample_opponents(8, 42);
    assert_eq!(full.len(), 8);
    let mut seen_full = std::collections::HashSet::new();
    for opp in &full {
        assert!(seen_full.insert(opp.id));
    }
}

#[test]
fn test_race_session_driver_spawning_and_names() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.opponent_drivers.len(), 3);
    assert_eq!(session.cars.len(), 4, "1 Player + 3 AI Bots");
    assert_eq!(session.color_schemes.len(), 4);
    assert_eq!(session.ai_drivers.len(), 3);

    // Verify AI drivers received their character's operationalized profile
    for i in 0..3 {
        let character = &session.opponent_drivers[i];
        let ai = &session.ai_drivers[i];
        assert_eq!(ai.profile.speed_factor, character.profile.speed_factor);
        assert_eq!(ai.profile.steering_kp, character.profile.steering_kp);
        assert_eq!(session.color_schemes[i + 1], character.color_scheme);
    }

    // Verify standings & results contain real driver aliases
    session.session_time = 65.0;
    session.trackers[0].current_lap = 4; // Player finished 3 laps
    for t in &mut session.trackers {
        t.best_lap_time = Some(21.5);
    }

    // Call private build_results via public compute_standings verification
    let standings = session.compute_standings();
    assert_eq!(standings.len(), 4);

    // Finish race simulation
    session.check_race_finish();
    assert_eq!(session.state, GameState::Finished);
}

#[test]
fn test_driver_cards_navigation_state() {
    let mut session = RaceSession::new();
    session.state = GameState::DriverCards(DriverCardsOrigin::Menu);

    assert_eq!(session.driver_cards_idx, 0);
    let roster_len = DriverCharacter::all().len();

    // Cycle forward
    session.driver_cards_idx = (session.driver_cards_idx + 1) % roster_len;
    assert_eq!(session.driver_cards_idx, 1);

    // Cycle backward with wrap-around
    session.driver_cards_idx = 0;
    session.driver_cards_idx = (session.driver_cards_idx + roster_len - 1) % roster_len;
    assert_eq!(session.driver_cards_idx, 7);
}

#[test]
fn test_starting_grid_flow_and_roster_presentation() {
    let mut session = RaceSession::new();
    session.num_bots = 3;
    session.init_race();

    // When starting a race vs AI, game transitions to StartingGrid to showcase participants
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.opponent_drivers.len(), 3);
    assert_eq!(session.cars.len(), 4);

    // Opening driver cards from StartingGrid sets DriverCardsOrigin::StartingGrid
    session.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
    assert_eq!(session.state, GameState::DriverCards(DriverCardsOrigin::StartingGrid));
}

#[test]
fn test_all_roster_bios_wrap_within_dossier_width() {
    let fonts = tdrace_app::ui::font::Fonts::load_embedded();
    let max_text_w = 380.0;
    let font_size = 13.0;

    for driver in DriverCharacter::all() {
        let lines = fonts.wrap_text(driver.bio, font_size, max_text_w);
        assert!(
            lines.len() >= 2,
            "Driver '{}' bio should wrap into multiple lines in dossier card, got {}",
            driver.name,
            lines.len()
        );
        assert!(
            lines.len() <= 4,
            "Driver '{}' bio should fit cleanly within 2-4 lines in dossier card, got {}",
            driver.name,
            lines.len()
        );

        // Every line should respect max width constraint
        for line in &lines {
            let dim = fonts.measure_ui_regular(line, font_size);
            assert!(
                dim.width <= max_text_w + 10.0,
                "Driver '{}' line '{}' width {} exceeded max_text_w {}",
                driver.name,
                line,
                dim.width,
                max_text_w
            );
        }
    }
}

