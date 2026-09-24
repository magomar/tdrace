use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::RaceSession;
use tdrace_app::profile::{ModuleCareerProgress, PlayerProfile};
use tdrace_app::render::color::CarColorScheme;
use tdrace_app::series::{ChampionshipSession, PointSystem};
use tdrace_app::ui::career_select::build_career_select_cards;

#[test]
fn test_launch_or_resume_preserves_round() {
    let mut session = RaceSession::new();
    let def = session.championship_manager.series.values().next().expect("At least one series def").clone();

    // Create an ongoing championship session with round 2 completed (current_round = 2)
    let mut champ = def.to_session();
    champ.current_round = 2;
    // Add some points to the player
    if let Some(player_standing) = champ.standings.iter_mut().find(|s| s.driver_id == "player") {
        player_standing.points = 35;
    }

    // Set as active championship in module progress
    session.active_career_progress.active_championship = Some(champ.clone());
    session.profile_module_progress.insert(def.series.module_id.clone(), session.active_career_progress.clone());

    // Launch or resume championship
    session.launch_or_resume_championship(&def);

    // Verify session resumed at round 2, NOT 0
    let resumed = session.championship_session.as_ref().expect("Session should be loaded");
    assert_eq!(resumed.current_round, 2, "Championship should resume at round 2, not round 0");

    let player_pts = resumed.standings.iter().find(|s| s.driver_id == "player").map(|s| s.points).unwrap_or(0);
    assert_eq!(player_pts, 35, "Player accumulated points should be preserved");

    // Verify track is round 2 track
    let expected_track = &champ.track_ids[2];
    assert_eq!(session.track_choice.track_id(), expected_track, "Track should match round 2 circuit");
}

#[test]
fn test_parallel_active_championships_across_modalities() {
    let session = RaceSession::new();
    let mut prog_map = std::collections::HashMap::new();

    // GT active championship at round 1
    let mut gt_prog = ModuleCareerProgress::default_for_module(1, "gt");
    let gt_def = session.championship_manager.series.values().find(|s| s.series.module_id == "gt").unwrap();
    let mut gt_champ = gt_def.to_session();
    gt_champ.current_round = 1;
    gt_prog.active_championship = Some(gt_champ);
    prog_map.insert("gt".to_string(), gt_prog.clone());

    // NASCAR active championship at round 2
    let mut nascar_prog = ModuleCareerProgress::default_for_module(1, "nascar");
    let nascar_def = session.championship_manager.series.values().find(|s| s.series.module_id == "nascar").unwrap();
    let mut nascar_champ = nascar_def.to_session();
    nascar_champ.current_round = 2;
    nascar_prog.active_championship = Some(nascar_champ);
    prog_map.insert("nascar".to_string(), nascar_prog);

    // Build career select cards
    let (cards, active_count) = build_career_select_cards(
        &session.championship_manager,
        &prog_map,
        &gt_prog,
        None,
        &[],
    );

    assert_eq!(active_count, 2, "There should be exactly 2 active championship careers");
    assert!(cards.len() >= 5, "Total cards should include active championships plus other disciplines");

    // The first 2 cards must be active
    assert!(cards[0].is_active);
    assert!(cards[1].is_active);

    let active_modules: Vec<&str> = cards[..2].iter().map(|c| c.module_id.as_str()).collect();
    assert!(active_modules.contains(&"gt"));
    assert!(active_modules.contains(&"nascar"));

    // Check round numbers
    for card in &cards[..2] {
        if card.module_id == "gt" {
            assert_eq!(card.current_round, 1);
        } else if card.module_id == "nascar" {
            assert_eq!(card.current_round, 2);
        }
    }
}

#[test]
fn test_db_persistence_of_active_championships() {
    let db = HallOfFameDb::open_in_memory().expect("Should open in-memory DB");
    let profile = PlayerProfile::new("Player One", "P1", None, CarColorScheme::default());
    let profile_id = db.create_profile(&profile).expect("Create profile");

    // Create GT active championship
    let mut gt_prog = ModuleCareerProgress::default_for_module(profile_id, "gt");
    let gt_champ = ChampionshipSession::new(
        "Super GT Challenge",
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "spa".to_string()],
        3,
        &[("player", "Player", "Apex")],
    );
    gt_prog.active_championship = Some(gt_champ);
    db.save_module_progress(&gt_prog).expect("Should save GT progress");

    // Create Rally active championship
    let mut rally_prog = ModuleCareerProgress::default_for_module(profile_id, "rally");
    let mut rally_champ = ChampionshipSession::new(
        "World Rallycross Cup",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["classic_rallycross".to_string(), "supertruck_stadium".to_string()],
        4,
        &[("player", "Player", "DirtKing")],
    );
    rally_champ.current_round = 1;
    rally_prog.active_championship = Some(rally_champ);
    db.save_module_progress(&rally_prog).expect("Should save Rally progress");

    // Load back and verify both coexist independently
    let loaded_all = db.get_all_module_progress(profile_id).expect("Should load all progress");
    assert_eq!(loaded_all.len(), 2);

    let loaded_gt = loaded_all.get("gt").expect("GT progress must exist");
    assert!(loaded_gt.active_championship.is_some());
    assert_eq!(loaded_gt.active_championship.as_ref().unwrap().name, "Super GT Challenge");
    assert_eq!(loaded_gt.active_championship.as_ref().unwrap().current_round, 0);

    let loaded_rally = loaded_all.get("rally").expect("Rally progress must exist");
    assert!(loaded_rally.active_championship.is_some());
    assert_eq!(loaded_rally.active_championship.as_ref().unwrap().name, "World Rallycross Cup");
    assert_eq!(loaded_rally.active_championship.as_ref().unwrap().current_round, 1);

    // Clear GT active championship
    db.clear_active_championship_for_profile(profile_id, "Super GT Challenge", "gt")
        .expect("Should clear GT championship");

    let after_clear = db.get_all_module_progress(profile_id).expect("Should load progress");
    assert!(after_clear.get("gt").unwrap().active_championship.is_none(), "GT active champ must be cleared");
    assert!(after_clear.get("rally").unwrap().active_championship.is_some(), "Rally active champ must remain intact");
}
