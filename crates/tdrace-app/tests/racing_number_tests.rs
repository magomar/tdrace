//! Spec 092: racing numbers. Persistence, allocation and the grid.

use tdrace_app::ai::{CareerRivalEntry, DriverTier, DrivingStyle};
use tdrace_app::db::HallOfFameDb;
use tdrace_app::profile::{ModuleCareerProgress, PlayerProfile};
use tdrace_app::series::SeriesStandingEntry;

fn temp_db_path(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tdrace_spec092_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.join("records.db")
}

// ---------------------------------------------------------------------------
// Profile number: default, range, database round trip
// ---------------------------------------------------------------------------

#[test]
fn a_new_profile_has_the_default_number() {
    assert_eq!(PlayerProfile::default().race_number, PlayerProfile::DEFAULT_RACE_NUMBER);
    assert_eq!(PlayerProfile::new_rookie("Rookie").race_number, 7);
    let db = HallOfFameDb::open_in_memory().unwrap();
    assert_eq!(db.get_active_profile().unwrap().race_number, 7);
}

#[test]
fn the_number_survives_a_restart() {
    let path = temp_db_path("restart");
    {
        let db = HallOfFameDb::open(&path).unwrap();
        let mut profile = db.get_active_profile().unwrap();
        profile.race_number = 17;
        db.update_profile(&profile).unwrap();
    }
    let db = HallOfFameDb::open(&path).unwrap();
    let profile = db.get_active_profile().unwrap();
    assert_eq!(profile.race_number, 17);
    assert_eq!(db.get_profile_by_id(profile.id.unwrap()).unwrap().unwrap().race_number, 17);
    assert_eq!(db.get_all_profiles().unwrap()[0].race_number, 17);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn create_profile_stores_the_number() {
    let db = HallOfFameDb::open_in_memory().unwrap();
    let mut profile = PlayerProfile::new_rookie("Second");
    profile.is_active = false;
    profile.race_number = 99;
    let id = db.create_profile(&profile).unwrap();
    assert_eq!(db.get_profile_by_id(id).unwrap().unwrap().race_number, 99);
}

#[test]
fn a_damaged_number_in_the_database_loads_as_the_default() {
    let path = temp_db_path("damaged");
    {
        let db = HallOfFameDb::open(&path).unwrap();
        db.get_active_profile().unwrap();
    }
    for bad in [0_i64, 100, 150, -3, 4_000_000_000] {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute("UPDATE player_profiles SET race_number = ?1", [bad]).unwrap();
        drop(conn);
        let db = HallOfFameDb::open(&path).unwrap();
        assert_eq!(db.get_active_profile().unwrap().race_number, 7, "stored {bad}");
        assert_eq!(db.get_all_profiles().unwrap()[0].race_number, 7, "stored {bad}");
    }
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn an_invalid_number_is_not_written_to_the_database() {
    let path = temp_db_path("write_guard");
    let db = HallOfFameDb::open(&path).unwrap();
    let mut profile = db.get_active_profile().unwrap();
    profile.race_number = 0;
    db.update_profile(&profile).unwrap();
    let raw: i64 = rusqlite::Connection::open(&path)
        .unwrap()
        .query_row("SELECT race_number FROM player_profiles", [], |r| r.get(0))
        .unwrap();
    assert_eq!(raw, 7);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_database_written_before_spec_092_gets_the_default_number() {
    let path = temp_db_path("old_db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE player_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                alias TEXT NOT NULL,
                country TEXT,
                primary_color TEXT NOT NULL,
                secondary_color TEXT NOT NULL,
                helmet_color TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );
            INSERT INTO player_profiles (name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at)
            VALUES ('Old Timer', 'Vintage', 'ESP', '#ff0000', '#00ff00', '#0000ff', 1, '2026-01-01 00:00');",
        )
        .unwrap();
    }
    let db = HallOfFameDb::open(&path).expect("an old database must open without error");
    let profile = db.get_active_profile().unwrap();
    assert_eq!(profile.name, "Old Timer");
    assert_eq!(profile.race_number, 7);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn profile_json_clamps_a_bad_number_and_loads_an_old_profile() {
    let mut value = serde_json::to_value(PlayerProfile::default()).unwrap();
    for bad in [0, 100, 150, -1, 70_000] {
        value["race_number"] = serde_json::json!(bad);
        let profile: PlayerProfile = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(profile.race_number, 7, "stored {bad}");
    }
    value["race_number"] = serde_json::json!(42);
    assert_eq!(serde_json::from_value::<PlayerProfile>(value.clone()).unwrap().race_number, 42);
    value.as_object_mut().unwrap().remove("race_number");
    assert_eq!(serde_json::from_value::<PlayerProfile>(value).unwrap().race_number, 7);
}

#[test]
fn race_number_from_i64_fails_closed() {
    assert_eq!(PlayerProfile::race_number_from_i64(1), 1);
    assert_eq!(PlayerProfile::race_number_from_i64(99), 99);
    for bad in [0, 100, 150, -5, i64::MAX, i64::MIN] {
        assert_eq!(PlayerProfile::race_number_from_i64(bad), PlayerProfile::DEFAULT_RACE_NUMBER);
    }
}

// ---------------------------------------------------------------------------
// Rival numbers saved in the module progress
// ---------------------------------------------------------------------------

fn rival(id: &str, number: Option<u8>) -> CareerRivalEntry {
    CareerRivalEntry {
        driver_id: id.to_string(),
        driver_name: format!("Driver {id}"),
        style: DrivingStyle::Balanced,
        tier: DriverTier::Rookie,
        race_number: number,
    }
}

#[test]
fn career_rival_numbers_survive_a_save_and_reload() {
    let db = HallOfFameDb::open_in_memory().unwrap();
    let profile = db.get_active_profile().unwrap();
    let mut progress = ModuleCareerProgress::default_for_module(profile.id.unwrap(), "kart");
    progress.career_rivals = vec![rival("a", Some(12)), rival("b", None), rival("c", Some(99))];
    db.save_module_progress(&progress).unwrap();
    let loaded = db.get_module_progress(profile.id.unwrap(), "kart").unwrap().unwrap();
    let numbers: Vec<Option<u8>> = loaded.career_rivals.iter().map(|r| r.race_number).collect();
    assert_eq!(numbers, vec![Some(12), None, Some(99)]);
}

#[test]
fn championship_standing_numbers_survive_a_save_and_reload() {
    let db = HallOfFameDb::open_in_memory().unwrap();
    let profile = db.get_active_profile().unwrap();
    let mut progress = ModuleCareerProgress::default_for_module(profile.id.unwrap(), "kart");
    let mut champ = tdrace_app::series::SeriesSession::new(
        "Test Cup",
        tdrace_app::series::PointSystem::ClassicArcade,
        vec!["t1".to_string()],
        3,
        &[("player", "Player", "Team"), ("r1", "Rival 1", "Team"), ("r2", "Rival 2", "Team")],
    );
    champ.standings[1].race_number = Some(31);
    progress.active_championship = Some(champ);
    db.save_module_progress(&progress).unwrap();
    let loaded = db.get_module_progress(profile.id.unwrap(), "kart").unwrap().unwrap();
    let standings = loaded.active_championship.unwrap().standings;
    assert_eq!(standings[0].race_number, None);
    assert_eq!(standings[1].race_number, Some(31));
    assert_eq!(standings[2].race_number, None);
}

#[test]
fn an_old_roster_without_numbers_loads_with_none() {
    let mut value = serde_json::to_value(rival("x", Some(5))).unwrap();
    value.as_object_mut().unwrap().remove("race_number");
    let entry: CareerRivalEntry = serde_json::from_value(value).expect("a rival saved before spec 092 loads");
    assert_eq!(entry.race_number, None);

    let mut value = serde_json::to_value(SeriesStandingEntry::new("x", "X", "Team")).unwrap();
    value.as_object_mut().unwrap().remove("race_number");
    let entry: SeriesStandingEntry = serde_json::from_value(value).expect("a standing saved before spec 092 loads");
    assert_eq!(entry.race_number, None);
}

#[test]
fn a_damaged_rival_number_loads_as_none_instead_of_failing_the_save() {
    let base = serde_json::to_value(rival("x", None)).unwrap();
    for bad in [serde_json::json!(0), serde_json::json!(100), serde_json::json!(300), serde_json::json!(-4), serde_json::Value::Null] {
        let mut value = base.clone();
        value["race_number"] = bad.clone();
        let entry: CareerRivalEntry = serde_json::from_value(value).unwrap_or_else(|e| panic!("{bad}: {e}"));
        assert_eq!(entry.race_number, None, "stored {bad}");

        let mut value = serde_json::to_value(SeriesStandingEntry::new("x", "X", "Team")).unwrap();
        value["race_number"] = bad.clone();
        let entry: SeriesStandingEntry = serde_json::from_value(value).unwrap_or_else(|e| panic!("{bad}: {e}"));
        assert_eq!(entry.race_number, None, "stored {bad}");
    }
    let mut value = base;
    value["race_number"] = serde_json::json!(55);
    let entry: CareerRivalEntry = serde_json::from_value(value).unwrap();
    assert_eq!(entry.race_number, Some(55));
}

// ---------------------------------------------------------------------------
// Grids: casual, career, championship and tournament
// ---------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};

use race_kit::FinishState;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::series::SeriesDefinition;
use tdrace_app::ui::menu::GameMode;

const SUPERBUGGY: &str = include_str!("../../../series/autocross/autocross_superbuggy_world_series.toml");
const WORLD_RX: &str = include_str!("../../../series/rally/rally_world_rallycross_supercars.toml");

fn all_unique(numbers: &[u8]) -> bool {
    numbers.iter().collect::<HashSet<_>>().len() == numbers.len()
}

fn grid_numbers(session: &RaceSession) -> Vec<u8> {
    session.grid_participants.iter().map(|p| p.race_number).collect()
}

/// The number of every driver on the grid, keyed by driver id (`player` for the human).
fn grid_numbers_by_driver(session: &RaceSession) -> HashMap<String, u8> {
    session
        .grid_participants
        .iter()
        .map(|p| {
            let id = if p.is_player { "player".to_string() } else { session.opponent_drivers[p.bot_index.unwrap()].id.to_string() };
            (id, p.race_number)
        })
        .collect()
}

fn launch_in(db: HallOfFameDb, toml: &str, player_number: u8) -> RaceSession {
    let def = SeriesDefinition::from_toml(toml).unwrap();
    let mut session = RaceSession::new();
    let mut profile = db.seed_default_profile_if_empty().unwrap();
    profile.race_number = player_number;
    db.update_profile(&profile).unwrap();
    session.hof_db = Some(db);
    session.refresh_profiles_and_stats();
    session.launch_or_resume_championship(&def);
    session
}

fn finish_race(session: &mut RaceSession) {
    let n = session.world.vehicles.len();
    let laps = session.total_laps;
    for car in 0..n {
        let time = 10.0 * laps as f32 + car as f32 * 1.5;
        session.world.finish[car] = FinishState::Finished { time, position: car + 1 };
        session.world.trackers[car].current_lap = laps + 1;
        session.world.trackers[car].best_lap_time = Some(time / laps as f32);
    }
    session.check_race_finish();
}

fn press_confirm(session: &mut RaceSession) {
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    match session.state {
        GameState::Finished => session.update_finished_screen(),
        GameState::TournamentBracket => session.update_tournament_bracket(),
        _ => panic!("unexpected state {:?}", session.state),
    }
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
}

/// Scenario: Grid generation eliminates duplicate numbers
#[test]
fn an_eight_car_quick_race_grid_has_eight_different_numbers() {
    let mut session = RaceSession::new();
    session.active_profile.race_number = 17;
    session.num_bots = 7;
    session.init_race();

    let numbers = grid_numbers(&session);
    assert_eq!(numbers.len(), 8);
    assert_eq!(numbers.iter().filter(|&&n| n == 17).count(), 1, "exactly one car carries 17");
    let player = session.grid_participants.iter().find(|p| p.is_player).unwrap();
    assert_eq!(player.race_number, 17);
    assert!(numbers.iter().all(|&n| (1..=99).contains(&n)));
    assert!(all_unique(&numbers), "{numbers:?}");

    // The numbers of the cars in the world follow the cars, not the grid order.
    assert_eq!(session.car_race_numbers.len(), session.world.vehicles.len());
    assert_eq!(session.car_race_numbers[0], 17, "car 0 is the player");
    assert_eq!(session.car_race_numbers.iter().copied().collect::<HashSet<_>>(), numbers.iter().copied().collect::<HashSet<_>>());
}

#[test]
fn the_player_number_is_used_even_when_the_profile_value_is_damaged() {
    let mut session = RaceSession::new();
    session.active_profile.race_number = 0;
    session.num_bots = 3;
    session.init_race();
    let numbers = grid_numbers(&session);
    assert_eq!(session.grid_participants.iter().find(|p| p.is_player).unwrap().race_number, 7);
    assert!(all_unique(&numbers));
    assert!(numbers.iter().all(|&n| (1..=99).contains(&n)));
}

#[test]
fn a_split_screen_grid_gives_player_two_its_own_number() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::SplitScreen;
    session.active_profile.race_number = 23;
    session.num_bots = 3;
    session.init_race();
    let numbers = grid_numbers(&session);
    assert_eq!(numbers.len(), 5);
    assert!(all_unique(&numbers), "{numbers:?}");
    assert_eq!(numbers.iter().filter(|&&n| n == 23).count(), 1);
}

#[test]
fn a_career_roster_keeps_its_numbers_between_races() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::Career;
    session.active_profile.race_number = 17;
    session.num_bots = 7;
    session.init_race();

    let first = grid_numbers_by_driver(&session);
    assert!(all_unique(&first.values().copied().collect::<Vec<_>>()));
    let rivals = &session.active_career_progress.career_rivals;
    assert!(!rivals.is_empty());
    assert!(rivals.iter().all(|r| r.race_number.is_some()), "every career rival has a stored number");

    session.init_race();
    assert_eq!(grid_numbers_by_driver(&session), first, "the same rivals keep their numbers in the next race");
}

/// Scenario: Player number change re-allocates only the colliding rival
#[test]
fn changing_the_player_number_moves_only_the_rival_that_has_it() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::Career;
    session.active_profile.race_number = 17;
    session.num_bots = 7;
    session.init_race();
    let before = grid_numbers_by_driver(&session);
    let rival_id = before.iter().find(|(id, _)| *id != "player").map(|(id, _)| id.clone()).unwrap();
    let taken = before[&rival_id];

    session.active_profile.race_number = taken;
    session.rebuild_roster_participants();
    let after = grid_numbers_by_driver(&session);

    assert_eq!(after["player"], taken);
    assert_ne!(after[&rival_id], taken, "the colliding rival got a new number");
    for (id, number) in &before {
        if *id != "player" && *id != rival_id {
            assert_eq!(after[id], *number, "{id} keeps its number");
        }
    }
    assert!(all_unique(&after.values().copied().collect::<Vec<_>>()));
}

/// Scenario: Old save without numbers loads cleanly
#[test]
fn a_roster_saved_without_numbers_gets_unique_numbers() {
    let mut session = launch_in(HallOfFameDb::open_in_memory().unwrap(), WORLD_RX, 17);
    for entry in session.championship_session.as_mut().unwrap().standings.iter_mut() {
        entry.race_number = None;
    }
    session.rebuild_roster_participants();

    let standings = &session.championship_session.as_ref().unwrap().standings;
    let rivals: Vec<u8> = standings.iter().filter(|s| s.driver_id != "player").map(|s| s.race_number.expect("numbered")).collect();
    assert_eq!(rivals.len(), 31);
    assert!(all_unique(&rivals));
    assert!(!rivals.contains(&17));
}

/// Scenario: Tournament roster of 32 drivers is unique
#[test]
fn a_tournament_weekend_numbers_all_32_drivers_and_keeps_them_in_every_race() {
    for toml in [SUPERBUGGY, WORLD_RX] {
        let mut session = launch_in(HallOfFameDb::open_in_memory().unwrap(), toml, 17);
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.standings.len(), 32);

        let roster: HashMap<String, u8> = champ
            .standings
            .iter()
            .map(|s| (s.driver_id.clone(), if s.driver_id == "player" { 17 } else { s.race_number.expect("every rival is numbered") }))
            .collect();
        assert!(all_unique(&roster.values().copied().collect::<Vec<_>>()), "32 drivers, 32 numbers");
        assert_eq!(roster["player"], 17);

        let mut races = 0;
        let mut seen: HashMap<String, u8> = HashMap::new();
        while session.championship_session.as_ref().unwrap().tournament_in_progress() {
            let on_grid = grid_numbers_by_driver(&session);
            assert_eq!(on_grid.len(), 8);
            assert!(all_unique(&on_grid.values().copied().collect::<Vec<_>>()));
            for (id, number) in &on_grid {
                assert_eq!(roster[id], *number, "{id} wears its roster number");
                assert_eq!(*seen.entry(id.clone()).or_insert(*number), *number, "{id} keeps its number between races");
            }
            races += 1;
            finish_race(&mut session);
            press_confirm(&mut session);
            if session.state == GameState::TournamentBracket {
                press_confirm(&mut session);
            }
        }
        assert!(races >= 3, "heat, semifinal and final were all built ({races})");
    }
}

/// Plays the heat, the semifinal and the final of the weekend on the grid.
fn play_weekend(session: &mut RaceSession) {
    while session.championship_session.as_ref().unwrap().tournament_in_progress() {
        finish_race(session);
        press_confirm(session);
        if session.state == GameState::TournamentBracket {
            press_confirm(session);
        }
    }
}

fn standing_numbers(session: &RaceSession) -> HashMap<String, Option<u8>> {
    session.championship_session.as_ref().unwrap().standings.iter().map(|s| (s.driver_id.clone(), s.race_number)).collect()
}

/// Scenario: Rival numbers persist across rounds and saves
#[test]
fn rival_numbers_survive_a_finished_round_a_save_and_a_resume() {
    let path = temp_db_path("resume");
    let round_one = {
        let mut session = launch_in(HallOfFameDb::open(&path).unwrap(), SUPERBUGGY, 17);
        let before = standing_numbers(&session);
        assert_eq!(before.len(), 32);
        assert!(before.iter().filter(|(id, _)| *id != "player").all(|(_, n)| n.is_some()));
        play_weekend(&mut session);
        assert_eq!(session.championship_session.as_ref().unwrap().current_round, 1, "round 1 is scored");
        assert_eq!(standing_numbers(&session), before, "round 1 did not change a number");
        before
    };

    // A restart: a new session on the same database file resumes the saved championship at round 2.
    let session = launch_in(HallOfFameDb::open(&path).unwrap(), SUPERBUGGY, 17);
    assert_eq!(session.championship_session.as_ref().unwrap().current_round, 1);
    assert_eq!(standing_numbers(&session), round_one, "every rival has the number it had in round 1");
    let on_grid = grid_numbers_by_driver(&session);
    assert!(on_grid.iter().all(|(id, n)| *id == "player" || round_one[id] == Some(*n)));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_championship_player_number_change_moves_only_the_colliding_rival() {
    let mut session = launch_in(HallOfFameDb::open_in_memory().unwrap(), SUPERBUGGY, 17);
    let before: HashMap<String, Option<u8>> =
        session.championship_session.as_ref().unwrap().standings.iter().map(|s| (s.driver_id.clone(), s.race_number)).collect();
    let (rival_id, taken) = before.iter().find(|(id, _)| *id != "player").map(|(id, n)| (id.clone(), n.unwrap())).unwrap();

    session.active_profile.race_number = taken;
    session.rebuild_roster_participants();

    let after: HashMap<String, Option<u8>> =
        session.championship_session.as_ref().unwrap().standings.iter().map(|s| (s.driver_id.clone(), s.race_number)).collect();
    assert_ne!(after[&rival_id], Some(taken));
    assert!(after[&rival_id].is_some());
    for (id, number) in &before {
        if *id != rival_id && *id != "player" {
            assert_eq!(after[id], *number, "{id} keeps its number");
        }
    }
    let numbers: Vec<u8> = after.iter().filter(|(id, _)| *id != "player").map(|(_, n)| n.unwrap()).collect();
    assert!(all_unique(&numbers));
    assert!(!numbers.contains(&taken));
}

// ---------------------------------------------------------------------------
// Profile editor: the number field on the create and modify screens
// ---------------------------------------------------------------------------

use macroquad::input::KeyCode;
use tdrace_app::game::inject_key_presses_for_tests;

/// Runs one frame with `keys` held as "pressed this frame".
fn press(session: &mut RaceSession, keys: &[KeyCode]) {
    inject_key_presses_for_tests(keys);
    session.update();
    inject_key_presses_for_tests(&[]);
}

fn press_times(session: &mut RaceSession, key: KeyCode, times: usize) {
    for _ in 0..times {
        press(session, &[key]);
    }
}

fn session_with_profiles(count: usize) -> RaceSession {
    let mut session = RaceSession::new();
    let db = HallOfFameDb::open_in_memory().unwrap();
    db.seed_default_profile_if_empty().unwrap();
    for i in 1..count {
        let mut profile = PlayerProfile::new_rookie(format!("Extra {i}"));
        profile.is_active = false;
        profile.race_number = 20 + i as u8;
        db.create_profile(&profile).unwrap();
    }
    session.hof_db = Some(db);
    session.refresh_profiles_and_stats();
    session
}

/// Scenario: Player defines a preferred racing number (create screen)
#[test]
fn the_create_screen_saves_the_chosen_number() {
    let mut session = session_with_profiles(1);
    session.state = GameState::ModuleSelect { selected_idx: 0 };
    press(&mut session, &[KeyCode::N]);
    match session.state {
        GameState::ProfileCreate { race_number, .. } => assert_eq!(race_number, 7, "a new profile starts on the default"),
        ref other => panic!("expected ProfileCreate, got {other:?}"),
    }

    // Name, alias, country, livery, number.
    press_times(&mut session, KeyCode::Tab, 4);
    press_times(&mut session, KeyCode::Right, 3);
    match session.state {
        GameState::ProfileCreate { race_number, field_idx, .. } => assert_eq!((field_idx, race_number), (4, 10)),
        ref other => panic!("expected ProfileCreate, got {other:?}"),
    }
    press(&mut session, &[KeyCode::Enter]);

    assert!(matches!(session.state, GameState::ProfileManager { .. }));
    assert_eq!(session.active_profile.race_number, 10, "the new profile is active with its number");
    let stored = session.hof_db.as_ref().unwrap().get_active_profile().unwrap();
    assert_eq!(stored.race_number, 10, "the number is in the database");
}

#[test]
fn the_number_field_wraps_between_1_and_99() {
    let mut session = session_with_profiles(1);
    session.state = GameState::ModuleSelect { selected_idx: 0 };
    press(&mut session, &[KeyCode::N]);
    press_times(&mut session, KeyCode::Tab, 4);
    press_times(&mut session, KeyCode::Left, 6);
    match session.state {
        GameState::ProfileCreate { race_number, .. } => assert_eq!(race_number, 1),
        ref other => panic!("expected ProfileCreate, got {other:?}"),
    }
    press(&mut session, &[KeyCode::Left]);
    match session.state {
        GameState::ProfileCreate { race_number, .. } => assert_eq!(race_number, 99, "Left from 1 goes to 99"),
        ref other => panic!("expected ProfileCreate, got {other:?}"),
    }
    press(&mut session, &[KeyCode::Right]);
    match session.state {
        GameState::ProfileCreate { race_number, .. } => assert_eq!(race_number, 1, "Right from 99 goes to 1"),
        ref other => panic!("expected ProfileCreate, got {other:?}"),
    }
}

/// Scenario: Player defines a preferred racing number (modify screen)
#[test]
fn the_roster_manager_edits_and_saves_the_number_of_the_selected_driver() {
    let mut session = session_with_profiles(2);
    let first_id = session.profile_list[0].id.unwrap();
    let second_id = session.profile_list[1].id.unwrap();
    session.open_player_roster_manager(0);
    match session.state {
        GameState::PlayerRosterManager { race_number, .. } => assert_eq!(race_number, session.profile_list[0].race_number),
        ref other => panic!("expected PlayerRosterManager, got {other:?}"),
    }

    // Into the right column, down to the number field (name, alias, country, livery, assists, number).
    press(&mut session, &[KeyCode::Tab]);
    press_times(&mut session, KeyCode::Down, 5);
    match session.state {
        GameState::PlayerRosterManager { field_idx, .. } => assert_eq!(field_idx, 5),
        ref other => panic!("expected PlayerRosterManager, got {other:?}"),
    }
    press_times(&mut session, KeyCode::Right, 2);
    press(&mut session, &[KeyCode::Enter]);

    let db = session.hof_db.as_ref().unwrap();
    let first = db.get_profile_by_id(first_id).unwrap().unwrap();
    assert_eq!(first.race_number, 9, "7 plus two steps");
    assert_eq!(db.get_profile_by_id(second_id).unwrap().unwrap().race_number, 21, "the other driver is untouched");
}

#[test]
fn saving_another_field_in_the_roster_manager_keeps_the_number() {
    let mut session = session_with_profiles(1);
    let id = session.profile_list[0].id.unwrap();
    session.hof_db.as_ref().unwrap().update_profile(&PlayerProfile { race_number: 42, ..session.profile_list[0].clone() }).unwrap();
    session.refresh_profiles_and_stats();
    session.open_player_roster_manager(0);
    press(&mut session, &[KeyCode::Tab]);
    press_times(&mut session, KeyCode::Down, 2); // country
    press(&mut session, &[KeyCode::Right]);
    press(&mut session, &[KeyCode::Enter]);
    let saved = session.hof_db.as_ref().unwrap().get_profile_by_id(id).unwrap().unwrap();
    assert_eq!(saved.race_number, 42);
}

#[test]
fn the_roster_manager_shows_the_number_of_the_driver_it_moves_to() {
    let mut session = session_with_profiles(2);
    session.open_player_roster_manager(0);
    press(&mut session, &[KeyCode::Down]);
    match session.state {
        GameState::PlayerRosterManager { selected_idx, race_number, .. } => {
            assert_eq!(selected_idx, 1);
            assert_eq!(race_number, session.profile_list[1].race_number);
        }
        ref other => panic!("expected PlayerRosterManager, got {other:?}"),
    }
}
