use tdrace_app::{
    championship_standings_table, constructors_standings_table, ChampionshipSession,
    GameState, ModuleCareerProgress, PlayerProfile, PointSystem, RaceSession, RoundDriverResult,
    SeriesSession,
};

#[test]
fn test_scenario_1_team_standings_point_aggregation_across_multi_driver_teams() {
    // Given an active championship session where "Valente GT" fields two drivers ("cedric_laval" and "lawrence_sterling")
    // and "Apex GT Racing" fields one driver ("player")
    let mut champ = SeriesSession::new(
        "GT Constructors Test Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string(), "spa".to_string()],
        3,
        &[
            ("player", "Player", "Apex GT Racing"),
            ("cedric_laval", "Cédric Laval", "Valente GT"),
            ("lawrence_sterling", "Lawrence Sterling", "Valente GT"),
            ("magnus_wolff", "Magnus Wolff", "Bullseye GT"),
        ],
    );

    // When Round 1 concludes with "cedric_laval" finishing P1 (25 pts), "player" finishing P2 (18 pts),
    // and "lawrence_sterling" finishing P3 (15 pts)
    let round_1 = vec![
        RoundDriverResult {
            driver_id: "cedric_laval".to_string(),
            driver_name: "Cédric Laval".to_string(),
            team_name: "Valente GT".to_string(),
            finish_position: 1,
            total_time: 120.0,
            best_lap: Some(39.5),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Apex GT Racing".to_string(),
            finish_position: 2,
            total_time: 121.0,
            best_lap: Some(40.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "lawrence_sterling".to_string(),
            driver_name: "Lawrence Sterling".to_string(),
            team_name: "Valente GT".to_string(),
            finish_position: 3,
            total_time: 122.0,
            best_lap: Some(40.2),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "magnus_wolff".to_string(),
            driver_name: "Magnus Wolff".to_string(),
            team_name: "Bullseye GT".to_string(),
            finish_position: 4,
            total_time: 123.5,
            best_lap: Some(40.8),
            points_awarded: 0,
            has_fastest_lap: false,
        },
    ];
    champ.submit_round_results("Monza", round_1);

    // Then the Constructors' Standings (team_standings) ranks "Valente GT" at P1 with 40 points (25 + 15), 1 win, and 2 podiums
    assert_eq!(champ.team_standings[0].team_name, "Valente GT");
    assert_eq!(champ.team_standings[0].points, 40);
    assert_eq!(champ.team_standings[0].wins, 1);
    assert_eq!(champ.team_standings[0].podiums, 2);

    // And "Apex GT Racing" is ranked at P2 with 18 points, 0 wins, and 1 podium
    assert_eq!(champ.team_standings[1].team_name, "Apex GT Racing");
    assert_eq!(champ.team_standings[1].points, 18);
    assert_eq!(champ.team_standings[1].wins, 0);
    assert_eq!(champ.team_standings[1].podiums, 1);
}

#[test]
fn test_scenario_2_team_points_retention_across_mid_season_driver_substitution() {
    // Given an active championship where "Silberstern GT" has accumulated 30 constructor points
    // via driver "gideon_palmer" (30 pts) and driver "otto_voss" (0 pts)
    let mut champ = SeriesSession::new(
        "GT Substitution Retention Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec![
            "monza".to_string(),
            "spa".to_string(),
            "silverstone".to_string(),
        ],
        3,
        &[
            ("player", "Player", "Apex GT Racing"),
            ("gideon_palmer", "Gideon Palmer", "Silberstern GT"),
            ("otto_voss", "Otto Voss", "Silberstern GT"),
        ],
    );

    // Score Round 1 & Round 2 so Gideon Palmer has 30 pts (15 + 15) and Silberstern GT has 30 pts
    for round_track in ["Monza", "Spa"] {
        champ.submit_round_results(
            round_track,
            vec![
                RoundDriverResult {
                    driver_id: "player".to_string(),
                    driver_name: "Player".to_string(),
                    team_name: "Apex GT Racing".to_string(),
                    finish_position: 1,
                    total_time: 100.0,
                    best_lap: Some(33.0),
                    points_awarded: 0,
                    has_fastest_lap: false,
                },
                RoundDriverResult {
                    driver_id: "gideon_palmer".to_string(),
                    driver_name: "Gideon Palmer".to_string(),
                    team_name: "Silberstern GT".to_string(),
                    finish_position: 3,
                    total_time: 102.0,
                    best_lap: Some(34.0),
                    points_awarded: 0,
                    has_fastest_lap: false,
                },
                RoundDriverResult {
                    driver_id: "otto_voss".to_string(),
                    driver_name: "Otto Voss".to_string(),
                    team_name: "Silberstern GT".to_string(),
                    finish_position: 12,
                    total_time: 110.0,
                    best_lap: Some(36.0),
                    points_awarded: 0,
                    has_fastest_lap: false,
                },
            ],
        );
    }

    let silberstern_before = champ
        .team_standings
        .iter()
        .find(|t| t.team_name == "Silberstern GT")
        .expect("Silberstern GT should exist");
    assert_eq!(silberstern_before.points, 30);

    // When "gideon_palmer" is replaced mid-season by reserve driver "lukas_weber" before Round 3
    let seat_idx = champ
        .standings
        .iter()
        .position(|s| s.driver_id == "gideon_palmer")
        .expect("gideon_palmer seat should exist");
    champ.replace_driver_in_seat(
        seat_idx,
        "lukas_weber",
        "Lukas Weber",
        Some("smooth".to_string()),
        Some(3),
    );

    // Then "lukas_weber" enters the Drivers' Standings with 0 personal points under "Silberstern GT"
    let lukas_entry = champ
        .standings
        .iter()
        .find(|s| s.driver_id == "lukas_weber")
        .expect("Lukas Weber should be in standings");
    assert_eq!(lukas_entry.points, 0);
    assert_eq!(lukas_entry.team_name, "Silberstern GT");

    // And "Silberstern GT" retains all 30 constructor points in team_standings
    let silberstern_mid = champ
        .team_standings
        .iter()
        .find(|t| t.team_name == "Silberstern GT")
        .expect("Silberstern GT should remain in team_standings");
    assert_eq!(silberstern_mid.points, 30);
    assert!(silberstern_mid.driver_ids.contains(&"lukas_weber".to_string()));

    // And when "lukas_weber" scores 12 points in Round 3, "Silberstern GT" advances to 42 constructor points (30 + 12)
    champ.submit_round_results(
        "Silverstone",
        vec![
            RoundDriverResult {
                driver_id: "player".to_string(),
                driver_name: "Player".to_string(),
                team_name: "Apex GT Racing".to_string(),
                finish_position: 1,
                total_time: 100.0,
                best_lap: Some(33.0),
                points_awarded: 0,
                has_fastest_lap: false,
            },
            RoundDriverResult {
                driver_id: "lukas_weber".to_string(),
                driver_name: "Lukas Weber".to_string(),
                team_name: "Silberstern GT".to_string(),
                finish_position: 4, // 12 points in FIA standard
                total_time: 101.5,
                best_lap: Some(33.5),
                points_awarded: 0,
                has_fastest_lap: false,
            },
            RoundDriverResult {
                driver_id: "otto_voss".to_string(),
                driver_name: "Otto Voss".to_string(),
                team_name: "Silberstern GT".to_string(),
                finish_position: 12,
                total_time: 110.0,
                best_lap: Some(36.0),
                points_awarded: 0,
                has_fastest_lap: false,
            },
        ],
    );

    let lukas_after = champ
        .standings
        .iter()
        .find(|s| s.driver_id == "lukas_weber")
        .unwrap();
    assert_eq!(lukas_after.points, 12);

    let silberstern_after = champ
        .team_standings
        .iter()
        .find(|t| t.team_name == "Silberstern GT")
        .unwrap();
    assert_eq!(silberstern_after.points, 42);
}

#[test]
fn test_scenario_8_championship_standings_ui_tab_toggle_between_drivers_and_constructors() {
    // Given an active championship session on the GameState::ChampionshipStandings screen
    let mut session = RaceSession::new();
    let mut champ = ChampionshipSession::new(
        "GT Constructors Championship",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string()],
        3,
        &[
            ("player", "Player", "Apex GT Racing"),
            ("cedric_laval", "Cédric Laval", "Valente GT"),
            ("lawrence_sterling", "Lawrence Sterling", "Valente GT"),
        ],
    );

    // Found a player team so the player team row has is_player_owned = true
    let mut profile = PlayerProfile::default();
    profile.credits = 50_000;
    let mut career = ModuleCareerProgress::default_for_gt(1);
    let quotes = SeriesSession::generate_scout_pool(1, 42);
    champ
        .found_player_team(
            &mut profile,
            &mut career,
            "Apex Racing Team",
            "gt_bmr_bavaria_t1",
            12_000,
            quotes[0].clone(),
        )
        .expect("Team founding should succeed");

    session.championship_session = Some(champ);
    session.state = GameState::ChampionshipStandings;
    assert!(!session.championship_standings_constructors);

    // Drivers' standings table builds cleanly by default
    let drivers_table =
        championship_standings_table(session.championship_session.as_ref().unwrap(), 0);
    assert_eq!(drivers_table.rows.len(), 3);

    // When the player presses [Tab] or gamepad shoulder buttons (LB / RB)
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_championship_standings();
    session.input.gamepad.snapshot.btn_rb_pressed = false;

    // Then the view switches from the Drivers' Standings table to the Constructors' Standings table (constructors_standings_table)
    assert!(session.championship_standings_constructors);
    assert!(
        session
            .championship_session
            .as_ref()
            .unwrap()
            .standings_tab_is_constructors
    );

    let constructors_table =
        constructors_standings_table(session.championship_session.as_ref().unwrap(), 0);
    assert_eq!(constructors_table.rows.len(), 2);

    // And the player's team row is highlighted with an Electric Cyan border (is_player = true) and [PLAYER TEAM] badge
    let player_team_row = constructors_table
        .rows
        .iter()
        .find(|r| r.id == "Apex Racing Team")
        .expect("Player team row should exist in constructors table");
    assert!(
        player_team_row.is_player,
        "Player team row should be flagged as player-owned for Electric Cyan highlighting"
    );
    let team_col = &constructors_table.columns[1];
    let formatted_team_cell = (team_col.extractor)(&player_team_row.data);
    assert!(
        formatted_team_cell.contains("[PLAYER TEAM]"),
        "Expected [PLAYER TEAM] badge in team cell, got: {}",
        formatted_team_cell
    );
}
