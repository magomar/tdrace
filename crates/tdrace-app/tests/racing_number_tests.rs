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
