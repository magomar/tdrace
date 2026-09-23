use tdrace_app::ai::DriverTier;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{GameMode, TrackChoice};

#[test]
fn test_first_race_initialization_uses_grid_capacity_and_tier_one() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.game_mode = GameMode::StandardRace; // Quick Race

    session.init_race();

    let grid_slots = session.max_grid_participants();
    assert!(grid_slots > 1, "Track must have multiple grid slots");
    assert_eq!(session.cars.len(), grid_slots, "Roster must initialize with full grid slots");
    assert_eq!(session.grid_participants.len(), grid_slots);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Rookie, "Difficulty must default to T1 (Rookie)");

    let pref = session.modality_preference(GameMode::StandardRace).expect("Preference should be recorded");
    assert_eq!(pref.racer_count, grid_slots);
    assert_eq!(pref.difficulty, DriverTier::Rookie);
}

#[test]
fn test_preference_memory_across_races_in_same_modality() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.game_mode = GameMode::StandardRace;
    session.init_race();

    // User changes preferences on starting grid: 4 racers (1 player + 3 bots), Pro (T4)
    session.set_num_bots(3);
    session.set_casual_ai_difficulty(DriverTier::Pro);

    assert_eq!(session.cars.len(), 4);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Pro);

    // Switch to another track (e.g. KartArena or OvalSpeedway) in the same modality
    session.track_choice = TrackChoice::OvalSpeedway;
    session.init_race();

    // Next race in the same modality must remember 4 racers and Pro (T4) difficulty
    assert_eq!(session.cars.len(), 4, "Must preserve racer count from last race in same modality");
    assert_eq!(session.num_bots, 3);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Pro, "Must preserve tier difficulty from last race");
}

#[test]
fn test_preference_memory_clamps_when_track_capacity_is_smaller() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.game_mode = GameMode::ExperimentalRace; // Custom race
    session.init_race();

    let full_slots = session.max_grid_participants();
    assert_eq!(session.cars.len(), full_slots);

    // Save preference with 8 racers and Legend (T5)
    session.update_modality_preference(GameMode::ExperimentalRace, 8, DriverTier::Legend);
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Legend);

    // Initialize another track
    session.track_choice = TrackChoice::KartArena;
    session.init_race();

    let kart_slots = session.max_grid_participants();
    let expected_racers = 8.min(kart_slots);
    assert_eq!(session.cars.len(), expected_racers);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Legend);
}

#[test]
fn test_modality_isolation_quick_custom_split() {
    let mut session = RaceSession::new();

    // 1. Quick Race: Configure 5 racers, Amateur (T2)
    session.game_mode = GameMode::StandardRace;
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();
    session.set_num_bots(4); // 1 player + 4 bots = 5 racers
    session.set_casual_ai_difficulty(DriverTier::Amateur);

    // 2. Custom Race: First time initialization must get full grid and T1
    session.game_mode = GameMode::ExperimentalRace;
    session.init_race();

    let full_slots = session.max_grid_participants();
    assert_eq!(session.cars.len(), full_slots, "New modality must initialize with full grid");
    assert_eq!(session.casual_ai_difficulty, DriverTier::Rookie, "New modality must initialize with T1");

    // Customize Custom Race: 3 racers, Contender (T3)
    session.set_num_bots(2);
    session.set_casual_ai_difficulty(DriverTier::Contender);
    assert_eq!(session.cars.len(), 3);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Contender);

    // 3. Split Screen: First time initialization must get full grid and T1
    session.game_mode = GameMode::SplitScreen;
    session.init_race();
    assert_eq!(session.cars.len(), full_slots);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Rookie);

    // Customize Split Screen: 2 racers (1v1 head to head, 0 bots), Pro (T4)
    session.set_num_bots(0);
    session.set_casual_ai_difficulty(DriverTier::Pro);
    assert_eq!(session.cars.len(), 2);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Pro);

    // 4. Return to Quick Race: must still have 5 racers, Amateur (T2)
    session.game_mode = GameMode::StandardRace;
    session.init_race();
    assert_eq!(session.cars.len(), 5);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Amateur);

    // 5. Return to Custom Race: must still have 3 racers, Contender (T3)
    session.game_mode = GameMode::ExperimentalRace;
    session.init_race();
    assert_eq!(session.cars.len(), 3);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Contender);

    // 6. Return to Split Screen: must still have 2 racers, Pro (T4)
    session.game_mode = GameMode::SplitScreen;
    session.init_race();
    assert_eq!(session.cars.len(), 2);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Pro);
}

#[test]
fn test_career_mode_is_exempt_from_casual_preferences() {
    let mut session = RaceSession::new();

    // Set casual preference for standard race
    session.update_modality_preference(GameMode::StandardRace, 4, DriverTier::Legend);

    // Switch to Career Mode
    session.game_mode = GameMode::Career;
    session.num_bots = 7;
    session.init_race();

    // Career mode should not be overwritten by casual preference
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.cars.len(), 8); // 1 player + 7 bots
}
