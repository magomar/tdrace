use tdrace_app::game::RaceSession;
use tdrace_app::series::{ChampionshipSession, PointSystem, RoundDriverResult};

#[test]
fn test_new_championship_expands_to_circuit_slots() {
    let mut session = RaceSession::new();
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("driver_1", "Driver One", "Team 1"),
        ("driver_2", "Driver Two", "Team 2"),
    ];
    let champ = ChampionshipSession::new(
        "Monza GT Cup",
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);
    session.init_race();

    let grid_slots = session.max_grid_participants();
    assert_eq!(grid_slots, 18, "Monza circuit has 18 starting grid slots");
    assert_eq!(session.cars.len(), 18, "Cars count must expand to circuit grid slots");
    assert_eq!(session.grid_participants.len(), 18, "Grid participants must expand to circuit grid slots");

    let champ_ref = session.championship_session.as_ref().unwrap();
    assert_eq!(champ_ref.standings.len(), 18, "Championship standings must expand to circuit grid slots");
    assert!(champ_ref.standings.iter().any(|s| s.driver_id == "player"), "Player must be in standings");
}

#[test]
fn test_new_championship_trims_to_circuit_slots_preserving_player() {
    let mut session = RaceSession::new();
    let mut drivers = vec![("player", "Player", "Apex Team")];
    for i in 1..=20 {
        let id: &'static str = Box::leak(format!("bot_{}", i).into_boxed_str());
        let name: &'static str = Box::leak(format!("Bot {}", i).into_boxed_str());
        drivers.push((id, name, "Rival Team"));
    }
    // classic_rallycross has 8 slots
    let champ = ChampionshipSession::new(
        "Rallycross Cup",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["classic_rallycross".to_string()],
        2,
        &drivers,
    );
    session.switch_to_rally();
    session.championship_session = Some(champ);
    session.init_race();

    let grid_slots = session.max_grid_participants();
    assert_eq!(grid_slots, 10, "Classic Rallycross circuit has 10 starting grid slots");
    assert_eq!(session.cars.len(), 10, "Cars count must trim to circuit grid slots");
    assert_eq!(session.grid_participants.len(), 10, "Grid participants must trim to circuit grid slots");

    let champ_ref = session.championship_session.as_ref().unwrap();
    assert_eq!(champ_ref.standings.len(), 10, "Championship standings must trim to circuit grid slots");
    assert!(champ_ref.standings.iter().any(|s| s.driver_id == "player"), "Player must be preserved in standings");
}

#[test]
fn test_round_with_fewer_slots_admits_top_ranked_and_player_without_discarding_others() {
    let mut session = RaceSession::new();
    // 2-round championship: Monza (18 slots) -> Classic Rallycross (8 slots)
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("ai_1", "Alpha One", "Team Alpha"),
        ("ai_2", "Bravo Two", "Team Bravo"),
        ("ai_3", "Charlie Three", "Team Charlie"),
        ("ai_4", "Delta Four", "Team Delta"),
        ("ai_5", "Echo Five", "Team Echo"),
        ("ai_6", "Foxtrot Six", "Team Foxtrot"),
        ("ai_7", "Golf Seven", "Team Golf"),
        ("ai_8", "Hotel Eight", "Team Hotel"),
    ];
    let champ = ChampionshipSession::new(
        "Grand Tour",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["monza".to_string(), "classic_rallycross".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);
    session.init_race();

    // Round 0 (Monza): expanded to 18 drivers
    assert_eq!(session.max_grid_participants(), 18);
    assert_eq!(session.cars.len(), 18);

    // Simulate completion of Round 0 where AI drivers score points
    // Let player finish 4th, ai_1 1st, ai_2 2nd, ai_3 3rd, ai_4 5th, ai_5 6th, ai_6 7th, ai_7 8th, ai_8 9th, etc.
    let standings_before = session.championship_session.as_ref().unwrap().standings.clone();
    let mut round_0_results = Vec::new();
    for (pos, entry) in standings_before.iter().enumerate() {
        round_0_results.push(RoundDriverResult {
            driver_id: entry.driver_id.clone(),
            driver_name: entry.driver_name.clone(),
            team_name: entry.team_name.clone(),
            finish_position: pos + 1,
            total_time: 100.0 + (pos as f32) * 2.0,
            best_lap: Some(60.0 + pos as f32),
            points_awarded: 0,
            has_fastest_lap: pos == 0,
        });
    }

    session.pending_championship_results = Some(round_0_results);
    // Submit round results
    if let Some(res) = session.pending_championship_results.take() {
        session.championship_session.as_mut().unwrap().submit_round_results("Monza", res);
    }

    let champ_after_r0 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ_after_r0.current_round, 1);
    assert_eq!(champ_after_r0.standings.len(), 18, "All 18 drivers remain in standings after round 0");

    // Advance to Round 1 (Classic Rallycross: 10 slots)
    session.init_race();

    assert_eq!(session.max_grid_participants(), 10, "Round 1 has 10 grid slots");
    assert_eq!(session.cars.len(), 10, "Only 10 cars participate in Round 1");
    assert_eq!(session.grid_participants.len(), 10);

    // Human player is in the grid
    let player_participant = session.grid_participants.iter().find(|p| p.is_player);
    assert!(player_participant.is_some(), "Human player must be in Round 1 grid");

    // The other 9 participants must be the top 9 non-player drivers from standings
    let non_player_standings: Vec<String> = session
        .championship_session
        .as_ref()
        .unwrap()
        .standings
        .iter()
        .filter(|s| s.driver_id != "player")
        .take(9)
        .map(|s| s.driver_name.clone())
        .collect();

    let mut bot_participants: Vec<String> = session
        .grid_participants
        .iter()
        .filter(|p| !p.is_player)
        .map(|p| p.name.clone())
        .collect();

    bot_participants.sort();
    let mut expected_drivers = non_player_standings;
    expected_drivers.sort();

    assert_eq!(bot_participants, expected_drivers, "Top 9 ranked AI drivers must be selected");

    // Crucial check: championship standings STILL retains all 18 drivers!
    assert_eq!(
        session.championship_session.as_ref().unwrap().standings.len(),
        18,
        "Non-qualifying drivers must NOT be permanently discarded from championship standings"
    );
}

#[test]
fn test_human_player_always_qualifies_even_when_ranked_last() {
    let mut session = RaceSession::new();
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("ai_1", "Alpha One", "Team Alpha"),
        ("ai_2", "Bravo Two", "Team Bravo"),
        ("ai_3", "Charlie Three", "Team Charlie"),
        ("ai_4", "Delta Four", "Team Delta"),
        ("ai_5", "Echo Five", "Team Echo"),
        ("ai_6", "Foxtrot Six", "Team Foxtrot"),
        ("ai_7", "Golf Seven", "Team Golf"),
        ("ai_8", "Hotel Eight", "Team Hotel"),
    ];
    let champ = ChampionshipSession::new(
        "Grand Tour",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["monza".to_string(), "classic_rallycross".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);
    session.init_race();

    // Round 0 on Monza (18 slots): player finishes dead last (18th with 0 points)
    let standings_before = session.championship_session.as_ref().unwrap().standings.clone();
    let mut round_0_results = Vec::new();
    // Put all AI drivers first (positions 1..17) and player last (position 18)
    let mut non_players: Vec<_> = standings_before.iter().filter(|s| s.driver_id != "player").collect();
    for (pos, entry) in non_players.drain(..).enumerate() {
        round_0_results.push(RoundDriverResult {
            driver_id: entry.driver_id.clone(),
            driver_name: entry.driver_name.clone(),
            team_name: entry.team_name.clone(),
            finish_position: pos + 1,
            total_time: 100.0 + (pos as f32),
            best_lap: Some(60.0),
            points_awarded: 0,
            has_fastest_lap: false,
        });
    }
    // Player finishes 18th
    round_0_results.push(RoundDriverResult {
        driver_id: "player".to_string(),
        driver_name: "Player".to_string(),
        team_name: "Apex GT".to_string(),
        finish_position: 18,
        total_time: 200.0,
        best_lap: Some(90.0),
        points_awarded: 0,
        has_fastest_lap: false,
    });

    session.championship_session.as_mut().unwrap().submit_round_results("Monza", round_0_results);

    // Verify player is ranked 18th (last) in standings
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.standings.last().unwrap().driver_id, "player", "Player must be ranked last in standings");

    // Advance to Round 1 on Classic Rallycross (10 slots)
    session.init_race();

    assert_eq!(session.max_grid_participants(), 10);
    assert_eq!(session.cars.len(), 10);
    // Player MUST still be on the grid in slot 0 despite being 18th in standings!
    let player_participant = session.grid_participants.iter().find(|p| p.is_player);
    assert!(player_participant.is_some(), "Human player must ALWAYS qualify for the round");
    assert_eq!(session.cars.len(), 10);
}

#[test]
fn test_subsequent_round_with_more_slots_allows_all_qualified_drivers_to_race_again() {
    let mut session = RaceSession::new();
    // 3 rounds: Monza (18) -> Classic Rallycross (10) -> Monza (18)
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("ai_1", "Alpha One", "Team Alpha"),
        ("ai_2", "Bravo Two", "Team Bravo"),
        ("ai_3", "Charlie Three", "Team Charlie"),
        ("ai_4", "Delta Four", "Team Delta"),
        ("ai_5", "Echo Five", "Team Echo"),
        ("ai_6", "Foxtrot Six", "Team Foxtrot"),
        ("ai_7", "Golf Seven", "Team Golf"),
        ("ai_8", "Hotel Eight", "Team Hotel"),
    ];
    let champ = ChampionshipSession::new(
        "Three Round Cup",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["monza".to_string(), "classic_rallycross".to_string(), "monza".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);

    // Round 0 (Monza): 18 slots
    session.init_race();
    assert_eq!(session.cars.len(), 18);

    // Finish Round 0
    let r0_results: Vec<RoundDriverResult> = session
        .opponent_drivers
        .iter()
        .enumerate()
        .map(|(idx, d)| RoundDriverResult {
            driver_id: d.id.to_string(),
            driver_name: d.name.to_string(),
            team_name: "Team".to_string(),
            finish_position: idx + 2,
            total_time: 110.0 + idx as f32,
            best_lap: Some(60.0),
            points_awarded: 0,
            has_fastest_lap: false,
        })
        .chain(std::iter::once(RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Apex GT".to_string(),
            finish_position: 1,
            total_time: 100.0,
            best_lap: Some(59.0),
            points_awarded: 0,
            has_fastest_lap: true,
        }))
        .collect();
    session.championship_session.as_mut().unwrap().submit_round_results("Monza", r0_results);

    // Round 1 (Classic Rallycross: 10 slots)
    session.init_race();
    assert_eq!(session.cars.len(), 10, "Round 1 restricted to 10 slots");
    assert_eq!(session.championship_session.as_ref().unwrap().standings.len(), 18);

    // Finish Round 1 with only the 10 participating drivers
    let r1_results: Vec<RoundDriverResult> = (1..=10)
        .map(|pos| RoundDriverResult {
            driver_id: if pos == 1 { "player".to_string() } else { format!("driver_{}", pos) },
            driver_name: format!("Driver {}", pos),
            team_name: "Team".to_string(),
            finish_position: pos,
            total_time: 100.0 + pos as f32,
            best_lap: Some(60.0),
            points_awarded: 0,
            has_fastest_lap: false,
        })
        .collect();
    session.championship_session.as_mut().unwrap().submit_round_results("Classic Rallycross", r1_results);

    // Round 2 (Monza: 18 slots again!)
    session.init_race();
    assert_eq!(session.cars.len(), 18, "Round 2 expands back to 18 slots for all drivers");
    assert_eq!(session.grid_participants.len(), 18);
    assert_eq!(session.championship_session.as_ref().unwrap().standings.len(), 18);
}

#[test]
fn test_qualified_drivers_for_round_helper() {
    let initial_drivers = [
        ("player", "Player", "Apex"),
        ("d1", "Driver 1", "Team 1"),
        ("d2", "Driver 2", "Team 2"),
        ("d3", "Driver 3", "Team 3"),
        ("d4", "Driver 4", "Team 4"),
        ("d5", "Driver 5", "Team 5"),
    ];
    let champ = ChampionshipSession::new(
        "Test Series",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["t1".to_string(), "t2".to_string()],
        2,
        &initial_drivers,
    );

    // If max_slots >= 6, all 6 qualify
    assert_eq!(champ.qualified_drivers_for_round(6).len(), 6);
    assert_eq!(champ.qualified_drivers_for_round(10).len(), 6);

    // If max_slots == 3, player + top 2 bots qualify = 3
    let qualified = champ.qualified_drivers_for_round(3);
    assert_eq!(qualified.len(), 3);
    assert_eq!(qualified[0].driver_id, "player");
    assert_eq!(qualified[1].driver_id, "d1");
    assert_eq!(qualified[2].driver_id, "d2");

    // If max_slots == 0, empty
    assert_eq!(champ.qualified_drivers_for_round(0).len(), 0);
}

#[test]
fn test_successive_round_grid_position_matches_standings_ranking() {
    let mut session = RaceSession::new();
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("bot_alpha", "Bot Alpha", "Team Alpha"),
        ("bot_bravo", "Bot Bravo", "Team Bravo"),
        ("bot_charlie", "Bot Charlie", "Team Charlie"),
        ("bot_delta", "Bot Delta", "Team Delta"),
    ];
    let champ = ChampionshipSession::new(
        "GT Championship",
        PointSystem::FiaStandard { fastest_lap_bonus: false },
        vec!["monza".to_string(), "monza".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);
    session.init_race();

    // Round 0 complete:
    // 1st: Bot Bravo (25 pts)
    // 2nd: Bot Alpha (18 pts)
    // 3rd: Player (15 pts)
    // 4th: Bot Charlie (12 pts)
    // 5th: Bot Delta (10 pts)
    let round_0_results = vec![
        RoundDriverResult {
            driver_id: "bot_bravo".to_string(),
            driver_name: "Bot Bravo".to_string(),
            team_name: "Team Bravo".to_string(),
            finish_position: 1,
            total_time: 100.0,
            best_lap: Some(60.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "bot_alpha".to_string(),
            driver_name: "Bot Alpha".to_string(),
            team_name: "Team Alpha".to_string(),
            finish_position: 2,
            total_time: 102.0,
            best_lap: Some(61.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Apex GT".to_string(),
            finish_position: 3,
            total_time: 104.0,
            best_lap: Some(62.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "bot_charlie".to_string(),
            driver_name: "Bot Charlie".to_string(),
            team_name: "Team Charlie".to_string(),
            finish_position: 4,
            total_time: 106.0,
            best_lap: Some(63.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "bot_delta".to_string(),
            driver_name: "Bot Delta".to_string(),
            team_name: "Team Delta".to_string(),
            finish_position: 5,
            total_time: 108.0,
            best_lap: Some(64.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
    ];

    session.championship_session.as_mut().unwrap().submit_round_results("Monza", round_0_results);

    // Verify standings ranking order:
    // Rank 0: Bot Bravo (25 pts)
    // Rank 1: Bot Alpha (18 pts)
    // Rank 2: Player (15 pts)
    // Rank 3: Bot Charlie (12 pts)
    // Rank 4: Bot Delta (10 pts)
    {
        let champ_after_r0 = session.championship_session.as_ref().unwrap();
        assert_eq!(champ_after_r0.standings[0].driver_id, "bot_bravo");
        assert_eq!(champ_after_r0.standings[1].driver_id, "bot_alpha");
        assert_eq!(champ_after_r0.standings[2].driver_id, "player");
        assert_eq!(champ_after_r0.standings[3].driver_id, "bot_charlie");
        assert_eq!(champ_after_r0.standings[4].driver_id, "bot_delta");
    }

    // Advance to Round 1 (successive round)
    session.advance_championship_round();

    // Verify grid positions match championship standings ranking exactly!
    assert_eq!(session.grid_participants[0].name, "Bot Bravo", "Championship leader must get Pole Position (Grid Slot 0)");
    assert_eq!(session.grid_participants[1].name, "Bot Alpha", "Rank 2 must get Grid Slot 1");
    assert!(session.grid_participants[2].is_player, "Rank 3 (Player) must get Grid Slot 2");
    assert_eq!(session.grid_participants[3].name, "Bot Charlie", "Rank 4 must get Grid Slot 3");
    assert_eq!(session.grid_participants[4].name, "Bot Delta", "Rank 5 must get Grid Slot 4");

    // Verify actual car spawn coordinates:
    // Player car (cars[0]) must be at grid_positions[2]
    let player_expected_pos = session.track.grid_positions[2].position;
    assert!((session.cars[0].state.position - player_expected_pos).length() < 0.01, "Player car must spawn at grid slot 2");

    // Bot Bravo car must be at grid_positions[0]
    let bravo_bot_idx = session.opponent_drivers.iter().position(|d| d.id == "bot_bravo").unwrap();
    let bravo_car_pos = session.cars[1 + bravo_bot_idx].state.position;
    let bravo_expected_pos = session.track.grid_positions[0].position;
    assert!((bravo_car_pos - bravo_expected_pos).length() < 0.01, "Bot Bravo car must spawn at pole (grid slot 0)");
}

#[test]
fn test_successive_round_player_wins_gets_pole_position() {
    let mut session = RaceSession::new();
    let initial_drivers = [
        ("player", "Player", "Apex GT"),
        ("bot_alpha", "Bot Alpha", "Team Alpha"),
        ("bot_bravo", "Bot Bravo", "Team Bravo"),
    ];
    let champ = ChampionshipSession::new(
        "Sprint Cup",
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "monza".to_string()],
        2,
        &initial_drivers,
    );
    session.switch_to_gt();
    session.championship_session = Some(champ);
    session.init_race();

    // Player wins Round 0 with fastest lap (26 pts), Bot Alpha 2nd (18 pts), Bot Bravo 3rd (15 pts)
    let round_0_results = vec![
        RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Apex GT".to_string(),
            finish_position: 1,
            total_time: 98.0,
            best_lap: Some(58.0),
            points_awarded: 0,
            has_fastest_lap: true,
        },
        RoundDriverResult {
            driver_id: "bot_alpha".to_string(),
            driver_name: "Bot Alpha".to_string(),
            team_name: "Team Alpha".to_string(),
            finish_position: 2,
            total_time: 100.0,
            best_lap: Some(59.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "bot_bravo".to_string(),
            driver_name: "Bot Bravo".to_string(),
            team_name: "Team Bravo".to_string(),
            finish_position: 3,
            total_time: 102.0,
            best_lap: Some(60.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
    ];

    session.championship_session.as_mut().unwrap().submit_round_results("Monza", round_0_results);

    // Advance to Round 1 (successive round)
    session.advance_championship_round();

    // Player is Rank 1 (leader) -> must be in Slot 0 (Pole position)
    assert!(session.grid_participants[0].is_player, "Winning player must start on pole (Grid Slot 0)");
    assert_eq!(session.grid_participants[1].name, "Bot Alpha", "2nd in standings must start in Grid Slot 1");
    assert_eq!(session.grid_participants[2].name, "Bot Bravo", "3rd in standings must start in Grid Slot 2");

    let player_expected_pos = session.track.grid_positions[0].position;
    assert!((session.cars[0].state.position - player_expected_pos).length() < 0.01, "Player car must spawn at pole (grid slot 0)");
}
