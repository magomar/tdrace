//! Spec 104 (`specs/104_autocross_and_rallycross_tournament_sprint_weekend_format.md`): the tournament
//! sprint weekend of Autocross and Rallycross: field, heats, semifinals, finals, the player's
//! continuous racing and the 32-place points.

use std::collections::HashSet;

use tdrace_app::series::{
    PointSystem, RoundDriverResult, SeriesDefinition, SeriesSession, SeriesStandingEntry, TournamentConfig, TournamentRace,
    TournamentStage, TournamentWeekendState, WeekendFormat, DNF_TIME, TOURNAMENT_POINTS_32, WORLD_RX_FINAL_POINTS_6,
    WORLD_RX_INTERMEDIATE_POINTS_12, WORLD_RX_SEMI_POINTS_6,
};

const SUPERBUGGY: &str = include_str!("../../../series/autocross/autocross_superbuggy_world_series.toml");
const CROSSCAR_AX: &str = include_str!("../../../series/autocross/autocross_crosscar_junior_trophy.toml");
const WORLD_RX: &str = include_str!("../../../series/rally/rally_world_rallycross_supercars.toml");

fn field(n: usize) -> Vec<SeriesStandingEntry> {
    let mut entries = vec![SeriesStandingEntry::new("player", "Player", "Apex")];
    for i in 1..n {
        entries.push(
            SeriesStandingEntry::new(format!("d{:02}", i), format!("Driver {:02}", i), format!("Team {}", i % 7))
                .with_ai_style_and_tier(None, Some(3)),
        );
    }
    entries
}

fn result(id: &str, position: usize, time: f32) -> RoundDriverResult {
    RoundDriverResult {
        driver_id: id.to_string(),
        driver_name: id.to_string(),
        team_name: "T".to_string(),
        finish_position: position,
        total_time: time,
        best_lap: Some(time / 4.0),
        points_awarded: 0,
        has_fastest_lap: false,
    }
}

/// Results of `race` in which the player finishes in `player_pos` and the others follow in grid order.
fn race_results(race: &TournamentRace, player_pos: usize) -> Vec<RoundDriverResult> {
    if race.stage == TournamentStage::SuperPole {
        let time = 38.0 + player_pos as f32 * 0.35;
        return vec![RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Apex".to_string(),
            finish_position: 1,
            total_time: time,
            best_lap: Some(time),
            points_awarded: 0,
            has_fastest_lap: false,
        }];
    }
    let others: Vec<&String> = race.driver_ids.iter().filter(|id| *id != "player").collect();
    let mut out = Vec::new();
    let mut next_other = others.iter();
    for pos in 1..=race.driver_ids.len() {
        let time = 40.0 * race.laps as f32 / 4.0 + pos as f32 * 0.7;
        let id = if pos == player_pos { "player".to_string() } else { next_other.next().unwrap().to_string() };
        out.push(result(&id, pos, time));
    }
    out
}

fn drive(state: &mut TournamentWeekendState, entries: &[SeriesStandingEntry], player_pos: usize) -> TournamentStage {
    let race = state.player_race().expect("the player has a race").clone();
    let stage = race.stage;
    state.submit_player_race(race_results(&race, player_pos), entries).expect("results match the race");
    stage
}

fn ids(race: &TournamentRace) -> HashSet<&str> {
    race.driver_ids.iter().map(|s| s.as_str()).collect()
}

/// Scenario: 32-Driver Autocross Tournament Field Initialization
///
/// Given an Autocross championship configured with `weekend_format = "tournament_sprint"`
/// When Round 1 begins
/// Then a 32-driver roster (Player + 31 AI drivers) should be assembled
/// And drivers should be partitioned into 4 Qualifying Heats of 8 cars each
#[test]
fn the_32_driver_field_is_drawn_into_4_heats_of_8() {
    for toml in [SUPERBUGGY, CROSSCAR_AX] {
        let def = SeriesDefinition::from_toml(toml).unwrap();
        assert_eq!(def.series.weekend_format, WeekendFormat::TournamentSprint);
        assert_eq!(
            def.series.tournament_config(),
            Some(TournamentConfig { total_drivers: 32, heat_laps: 3, semi_laps: 4, final_laps: 5, qualifying_rounds: 1 })
        );
        assert!(def.validate().is_ok());

        let mut session = def.to_session();
        assert!(session.is_tournament());
        assert!(session.standings.len() == 8 || session.standings.len() == 32);
        let pool = tdrace_app::ai::DriverCharacter::all_across_modules();
        session.sync_initial_grid_slots(32, &pool);
        assert_eq!(session.standings.len(), 32);
        assert_eq!(session.standings.iter().filter(|s| s.driver_id == "player").count(), 1, "Player + 31 AI");

        session.ensure_tournament_weekend().unwrap();
        let weekend = session.tournament.as_ref().unwrap();
        assert_eq!(weekend.active_races.len(), 4);
        let mut seen = HashSet::new();
        for (h, heat) in weekend.active_races.iter().enumerate() {
            assert_eq!((heat.stage, heat.group, heat.laps, heat.driver_ids.len()), (TournamentStage::QualifyingHeat, h, 3, 8));
            for id in &heat.driver_ids {
                assert!(seen.insert(id.clone()), "{} is in one heat only", id);
            }
        }
        assert_eq!(seen.len(), 32);
        assert_eq!(session.tournament_race().unwrap().badge().split(' ').next(), Some("HEAT"));
        assert_eq!(session.current_round_laps(), Some(3), "heats run 3 laps whatever the round says");
    }
}

/// Scenario: Qualifying Heat Execution and Semifinal Seeding
///
/// Given Stage 1 (Qualifying Heats) commences
/// When all 4 heats conclude 3-lap sprint races
/// Then the top 4 drivers from each of the 4 heats (16 drivers total) should advance to the Semifinals
/// And drivers should receive seeds for Semifinal 1 and Semifinal 2 based on elapsed heat finish times
#[test]
fn the_top_4_of_each_heat_are_seeded_into_two_semifinals_by_elapsed_time() {
    let entries = field(32);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 7).unwrap();
    assert_eq!(drive(&mut weekend, &entries, 1), TournamentStage::QualifyingHeat);

    assert_eq!(weekend.heat_results.len(), 4);
    assert!(weekend.heat_results.iter().all(|h| h.len() == 8));
    let top4 = |h: usize| -> HashSet<&str> { weekend.heat_results[h].iter().take(4).map(|r| r.driver_id.as_str()).collect() };

    assert_eq!(weekend.active_stage, TournamentStage::Semifinal);
    assert_eq!(weekend.active_races.len(), 2);
    let (semi1, semi2) = (&weekend.active_races[0], &weekend.active_races[1]);
    let expect1: HashSet<&str> = top4(0).union(&top4(2)).copied().collect();
    let expect2: HashSet<&str> = top4(1).union(&top4(3)).copied().collect();
    assert_eq!((semi1.driver_ids.len(), semi2.driver_ids.len()), (8, 8));
    assert_eq!(ids(semi1), expect1, "semifinal 1 takes heats 1 and 3");
    assert_eq!(ids(semi2), expect2, "semifinal 2 takes heats 2 and 4");
    assert!(semi1.laps == 4 && semi2.laps == 4);

    // The grid is the elapsed heat times, fastest first.
    let time_of = |id: &str| weekend.heat_results.iter().flatten().find(|r| r.driver_id == id).unwrap().total_time;
    for semi in [semi1, semi2] {
        let times: Vec<f32> = semi.driver_ids.iter().map(|id| time_of(id)).collect();
        assert!(times.windows(2).all(|w| w[0] <= w[1]), "grid follows heat times: {:?}", times);
    }
}

/// Scenario: Semifinal to Grand Final Progression (Top 4 Advance)
///
/// Given the 16 advancing drivers split into Semifinal 1 (8 cars) and Semifinal 2 (8 cars)
/// When Stage 2 concludes 4-lap sprint races
/// Then the top 4 drivers from Semifinal 1 and the top 4 drivers from Semifinal 2 (8 drivers total) should advance to the Grand Final
/// And drivers finishing 5th through 8th in either semifinal should advance to the Consolation B-Final
#[test]
fn the_top_4_of_each_semifinal_go_to_the_grand_final_and_the_rest_to_the_b_final() {
    let entries = field(32);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 11).unwrap();
    drive(&mut weekend, &entries, 1);
    assert_eq!(drive(&mut weekend, &entries, 2), TournamentStage::Semifinal);

    assert_eq!(weekend.semi_results.len(), 2);
    let take = |range: std::ops::Range<usize>| -> HashSet<&str> {
        weekend.semi_results.iter().flat_map(|s| s[range.clone()].iter().map(|r| r.driver_id.as_str())).collect()
    };
    assert_eq!(weekend.active_races.len(), 2, "Grand Final and B-Final");
    let grand = weekend.active_races.iter().find(|r| r.stage == TournamentStage::GrandFinal).unwrap();
    let b_final = weekend.active_races.iter().find(|r| r.stage == TournamentStage::ConsolationBFinal).unwrap();
    assert_eq!(ids(grand), take(0..4));
    assert_eq!(ids(b_final), take(4..8));
    assert_eq!((grand.driver_ids.len(), b_final.driver_ids.len()), (8, 8));
    assert_eq!((grand.laps, b_final.laps), (5, 4));
    assert_eq!(weekend.active_stage, TournamentStage::GrandFinal, "the player finished 2nd in a semifinal");
}

/// Scenario: Continuous Player Racing (Option A Guarantee)
///
/// Given the player finishes in position 5, 6, 7, or 8 in their Stage 1 Heat or Stage 2 Semifinal
/// When subsequent stages load
/// Then the player should be assigned to an active Consolation race (Consolation Semi or B-Final)
/// And the player should race all 3 stages of the weekend without encountering an elimination screen or spectate-only lock
#[test]
fn a_player_who_misses_the_cut_keeps_racing_in_every_stage() {
    let entries = field(32);
    for heat_pos in 5..=8 {
        let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 3).unwrap();
        let mut stages = vec![drive(&mut weekend, &entries, heat_pos)];

        assert_eq!(weekend.active_stage, TournamentStage::ConsolationSemifinal, "heat place {}", heat_pos);
        let consolation = weekend.player_race().expect("a consolation race with the player").clone();
        assert_eq!(consolation.driver_ids.len(), 8);
        assert_eq!(weekend.active_races.len(), 3, "two semifinals and the player's consolation race");
        stages.push(drive(&mut weekend, &entries, 3));

        assert_eq!(weekend.active_stage, TournamentStage::ConsolationBFinal);
        let b_final = weekend.player_race().expect("the player is in the B-Final").clone();
        assert_eq!(b_final.driver_ids.len(), 8);
        assert_eq!(b_final.driver_ids.last().map(String::as_str), Some("player"), "he starts at the back");
        stages.push(drive(&mut weekend, &entries, 1));

        assert_eq!(stages, vec![TournamentStage::QualifyingHeat, TournamentStage::ConsolationSemifinal, TournamentStage::ConsolationBFinal]);
        assert!(weekend.is_complete);
        assert!(weekend.final_standings().unwrap().iter().any(|r| r.driver_id == "player"));
    }

    for semi_pos in 5..=8 {
        let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 3).unwrap();
        drive(&mut weekend, &entries, 4);
        drive(&mut weekend, &entries, semi_pos);
        assert_eq!(weekend.active_stage, TournamentStage::ConsolationBFinal, "semifinal place {}", semi_pos);
        assert!(weekend.player_race().unwrap().driver_ids.iter().any(|id| id == "player"));
        drive(&mut weekend, &entries, 8);
        assert!(weekend.is_complete);
        assert!(weekend.player_race().is_none(), "nothing is left to race");
    }
}

/// The player is in exactly one race of every stage, whatever he finishes.
#[test]
fn the_player_is_in_exactly_one_scheduled_race_at_every_step() {
    let entries = field(32);
    for path in [[1usize, 1, 1], [8, 8, 8], [4, 5, 5], [5, 1, 8], [2, 4, 3]] {
        let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 99).unwrap();
        for pos in path {
            let with_player = weekend.active_races.iter().filter(|r| r.driver_ids.iter().any(|id| id == "player")).count();
            assert_eq!(with_player, 1, "path {:?}: {:?}", path, weekend.active_stage);
            drive(&mut weekend, &entries, pos);
        }
        assert!(weekend.is_complete, "path {:?}", path);
    }
}

/// Scenario: Championship Points Allocation Across 32 Drivers
///
/// Given Stage 3 concludes with the completion of the Grand Final and B-Final
/// When round results are tabulated
/// Then all 32 drivers should receive designated championship points according to their final finishing rank (1st through 32nd)
/// And the player profile and standings table should update atomically
#[test]
fn all_32_drivers_are_scored_by_final_rank_in_one_update() {
    let def = SeriesDefinition::from_toml(SUPERBUGGY).unwrap();
    let mut session = def.to_session();
    let pool = tdrace_app::ai::DriverCharacter::all_across_modules();
    session.sync_initial_grid_slots(32, &pool);
    assert_eq!(session.point_system, PointSystem::Custom(TOURNAMENT_POINTS_32.to_vec()));
    session.ensure_tournament_weekend().unwrap();

    // Player: 2nd in the heat and the semifinal, wins the Grand Final.
    for pos in [2, 2] {
        let race = session.tournament_race().unwrap().clone();
        assert!(!session.submit_tournament_race("Nova Paka", race_results(&race, pos)).unwrap(), "the weekend goes on");
        assert_eq!(session.current_round, 0, "no points before the last stage");
        assert!(session.standings.iter().all(|s| s.points == 0));
    }
    let race = session.tournament_race().unwrap().clone();
    assert_eq!(race.stage, TournamentStage::GrandFinal);
    assert!(session.submit_tournament_race("Nova Paka", race_results(&race, 1)).unwrap(), "the weekend is over");

    assert_eq!(session.current_round, 1, "one weekend is one round");
    assert!(session.tournament.is_none());
    let round = &session.history[0];
    assert_eq!(round.results.len(), 32);
    let ranks: Vec<usize> = round.results.iter().map(|r| r.finish_position).collect();
    assert_eq!(ranks, (1..=32).collect::<Vec<_>>(), "ranks 1 to 32, each once");
    for r in &round.results {
        assert_eq!(r.points_awarded, TOURNAMENT_POINTS_32[r.finish_position - 1], "{} rank {}", r.driver_id, r.finish_position);
    }
    assert_eq!(round.results[0].driver_id, "player");
    assert_eq!(round.results[31].points_awarded, 1, "32nd place scores 1 point");
    let total: u32 = session.standings.iter().map(|s| s.points).sum();
    assert_eq!(total, TOURNAMENT_POINTS_32.iter().sum::<u32>());
    let player = session.standings.iter().find(|s| s.driver_id == "player").unwrap();
    assert_eq!((player.points, player.wins, player.podiums), (25, 1, 1));
    assert_eq!(session.leader().unwrap().driver_id, "player");
}

/// Ranks 9 to 16 are the B-Final, 17 to 32 the drivers the heats and semifinals dropped.
#[test]
fn final_ranks_follow_grand_final_then_b_final_then_earlier_exits() {
    let entries = field(32);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 5).unwrap();
    drive(&mut weekend, &entries, 6);
    drive(&mut weekend, &entries, 1);
    drive(&mut weekend, &entries, 1);
    let places = weekend.final_standings().unwrap();
    let grand: Vec<&str> = weekend.grand_final_results.as_ref().unwrap().iter().map(|r| r.driver_id.as_str()).collect();
    let b_final: Vec<&str> = weekend.b_final_results.as_ref().unwrap().iter().map(|r| r.driver_id.as_str()).collect();
    assert_eq!(places.iter().take(8).map(|r| r.driver_id.as_str()).collect::<Vec<_>>(), grand);
    assert_eq!(places.iter().skip(8).take(8).map(|r| r.driver_id.as_str()).collect::<Vec<_>>(), b_final);
    assert_eq!(places.iter().map(|r| r.driver_id.as_str()).collect::<HashSet<_>>().len(), 32, "every driver once");
    assert_eq!(places[8].driver_id, "player", "the player won the B-Final");
    // The semifinal driver the player displaced is the first of the rest.
    let semi_out: HashSet<&str> = weekend.semi_results.iter().flat_map(|s| s[4..].iter().map(|r| r.driver_id.as_str())).collect();
    assert!(semi_out.contains(places[16].driver_id.as_str()), "{} was a semifinal drop-out", places[16].driver_id);
}

/// Anti-tamper: wrong fields are refused and the weekend stays as it was.
#[test]
fn results_that_do_not_match_the_race_are_refused() {
    let entries = field(32);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 1).unwrap();
    let before = weekend.clone();
    let race = weekend.player_race().unwrap().clone();

    let mut short = race_results(&race, 1);
    short.pop();
    assert!(weekend.submit_player_race(short, &entries).is_err(), "an aborted heat does not count");
    let mut stranger = race_results(&race, 1);
    stranger[3].driver_id = "nobody".to_string();
    assert!(weekend.submit_player_race(stranger, &entries).is_err());
    let mut twice = race_results(&race, 1);
    twice[4].driver_id = twice[3].driver_id.clone();
    assert!(weekend.submit_player_race(twice, &entries).is_err());
    assert_eq!(weekend, before);
    assert!(weekend.final_standings().is_none(), "no points before both finals");
}

/// DNF drivers receive last place in their heat, whatever time they were wrecked at.
#[test]
fn a_driver_who_did_not_finish_is_classified_last_in_the_heat() {
    let entries = field(32);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::default(), &entries, 2).unwrap();
    let race = weekend.player_race().unwrap().clone();
    let mut results = race_results(&race, 3);
    // The winner wrecks at 2 seconds: a short time that would win the heat if it counted.
    results[0].total_time = DNF_TIME;
    results[0].finish_position = 1;
    let wrecked = results[0].driver_id.clone();
    weekend.submit_player_race(results, &entries).unwrap();
    let heat = weekend.heat_results.iter().find(|h| h.iter().any(|r| r.driver_id == wrecked)).unwrap();
    assert_eq!(heat.last().unwrap().driver_id, wrecked);
    assert_eq!(heat.last().unwrap().finish_position, 8);
    assert_eq!(heat[1].driver_id, "player", "the player moved up from 3rd to 2nd");
}

/// Determinism: the same seed and results give the same bracket, another seed another draw.
#[test]
fn a_weekend_replays_bit_for_bit_from_the_same_seed() {
    let entries = field(32);
    let run = |seed: u64| {
        let mut w = TournamentWeekendState::new(TournamentConfig::default(), &entries, seed).unwrap();
        for pos in [5, 3, 2] {
            drive(&mut w, &entries, pos);
        }
        w
    };
    let (a, b, c) = (run(42), run(42), run(43));
    assert_eq!(a, b);
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    assert_ne!(
        TournamentWeekendState::new(TournamentConfig::default(), &entries, 42).unwrap().active_races,
        TournamentWeekendState::new(TournamentConfig::default(), &entries, 43).unwrap().active_races,
    );
    assert_ne!(a.final_standings(), c.final_standings());
}

/// Scalability: the same engine runs 64 drivers over 4 stages.
#[test]
fn a_64_driver_weekend_runs_8_heats_4_quarterfinals_2_semifinals_and_the_finals() {
    let config = TournamentConfig { total_drivers: 64, ..TournamentConfig::default() };
    let entries = field(64);
    let mut weekend = TournamentWeekendState::new(config, &entries, 9).unwrap();
    assert_eq!(weekend.active_races.len(), 8);
    let mut stages = vec![drive(&mut weekend, &entries, 1)];
    assert_eq!(weekend.active_races.len(), 4, "quarterfinals");
    assert!(weekend.active_races.iter().all(|r| r.stage == TournamentStage::Quarterfinal && r.driver_ids.len() == 8));
    stages.push(drive(&mut weekend, &entries, 1));
    assert_eq!(weekend.active_races.len(), 2, "semifinals");
    stages.push(drive(&mut weekend, &entries, 1));
    assert_eq!(weekend.active_races.len(), 2, "grand final and B-Final");
    stages.push(drive(&mut weekend, &entries, 1));
    assert_eq!(stages, vec![TournamentStage::QualifyingHeat, TournamentStage::Quarterfinal, TournamentStage::Semifinal, TournamentStage::GrandFinal]);
    assert!(weekend.is_complete);
    assert_eq!(weekend.final_standings().unwrap().len(), 64);

    // A player dropped in the heats races a consolation race in every stage until the B-Final.
    let mut weekend = TournamentWeekendState::new(config, &entries, 9).unwrap();
    let mut path = vec![drive(&mut weekend, &entries, 7)];
    path.push(drive(&mut weekend, &entries, 2));
    path.push(drive(&mut weekend, &entries, 2));
    path.push(drive(&mut weekend, &entries, 2));
    assert_eq!(
        path,
        vec![TournamentStage::QualifyingHeat, TournamentStage::ConsolationSemifinal, TournamentStage::ConsolationSemifinal, TournamentStage::ConsolationBFinal]
    );
    assert_eq!(weekend.final_standings().unwrap().len(), 64);
    assert!(TournamentWeekendState::new(TournamentConfig { total_drivers: 20, ..config }, &field(20), 1).is_err());
}

/// A weekend in progress survives a save and a load (career saves hold the session as JSON).
#[test]
fn a_weekend_in_progress_survives_serialization() {
    let def = SeriesDefinition::from_toml(WORLD_RX).unwrap();
    let mut session = def.to_session();
    session.sync_initial_grid_slots(24, &tdrace_app::ai::DriverCharacter::all_across_modules());
    session.ensure_tournament_weekend().unwrap();
    // Run SuperPole first, then finish 6th in the Qualifying Heat so the player goes to ConsolationSemifinal.
    let sp_race = session.tournament_race().unwrap().clone();
    assert_eq!(sp_race.stage, TournamentStage::SuperPole);
    session.submit_tournament_race("Lydden Hill", race_results(&sp_race, 24)).unwrap();
    let race = session.tournament_race().unwrap().clone();
    assert_eq!(race.stage, TournamentStage::QualifyingHeat);
    let mut slow_heat = race_results(&race, 6);
    for r in &mut slow_heat {
        if r.driver_id == "player" {
            r.total_time = 240.0;
        }
    }
    session.submit_tournament_race("Lydden Hill", slow_heat).unwrap();

    let json = serde_json::to_string(&session).unwrap();
    let restored: SeriesSession = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, session);
    assert_eq!(restored.tournament_race().unwrap().stage, TournamentStage::ConsolationSemifinal);
    assert!(restored.tournament_in_progress());

    // A save written before Spec 104 has no weekend fields and stays a single-race series.
    let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
    for key in ["weekend_format", "tournament_config", "tournament"] {
        old.as_object_mut().unwrap().remove(key);
    }
    let legacy: SeriesSession = serde_json::from_value(old).unwrap();
    assert_eq!(legacy.weekend_format, WeekendFormat::StandardSingleRace);
    assert!(!legacy.is_tournament());
    assert!(legacy.tournament_race().is_none());
}

/// A series that runs one race per round keeps its TOML free of the weekend keys, and a bad size is refused.
#[test]
fn weekend_keys_are_declared_only_by_tournament_series() {
    let gt = include_str!("../../../series/gt/gt4_clubman_sprint.toml");
    let def = SeriesDefinition::from_toml(gt).unwrap();
    assert_eq!(def.series.weekend_format, WeekendFormat::StandardSingleRace);
    assert!(def.series.tournament_config().is_none());
    assert!(!def.to_session().is_tournament());
    let toml = def.to_toml().unwrap();
    assert!(!toml.contains("weekend_format") && !toml.contains("total_drivers"), "{}", toml);

    let tournament = SeriesDefinition::from_toml(SUPERBUGGY).unwrap();
    let again = SeriesDefinition::from_toml(&tournament.to_toml().unwrap()).unwrap();
    assert_eq!(again, tournament, "the weekend keys survive a save");
    assert_eq!(SeriesDefinition::from_session(&tournament.to_session(), "autocross", 5).series.tournament_config(), tournament.series.tournament_config());

    let mut bad = tournament.clone();
    bad.series.total_drivers = Some(20);
    assert!(bad.validate().unwrap_err().iter().any(|e| e.contains("16, 24, 32 or 64")));
}

/// Re-running the round of a finished weekend starts a new weekend.
#[test]
fn cancelling_the_latest_round_drops_the_weekend() {
    let mut session = SeriesSession::new("Cup", PointSystem::Custom(TOURNAMENT_POINTS_32.to_vec()), vec!["a".into(), "b".into()], 4, &[("player", "Player", "T")])
        .with_tournament(TournamentConfig::default());
    session.sync_initial_grid_slots(32, &tdrace_app::ai::DriverCharacter::all_across_modules());
    session.ensure_tournament_weekend().unwrap();
    for pos in [1, 1, 1] {
        let race = session.tournament_race().unwrap().clone();
        session.submit_tournament_race("A", race_results(&race, pos)).unwrap();
    }
    assert_eq!(session.current_round, 1);
    session.ensure_tournament_weekend().unwrap();
    assert!(session.tournament.is_some(), "the next round has its own weekend");
    assert_eq!(session.cancel_latest_round().as_deref(), Some("a"));
    assert!(session.tournament.is_none());
    assert_eq!(session.current_round, 0);
    assert!(session.standings.iter().all(|s| s.points == 0));
}

// =============================================================================
// Spec 128: World RX 24-Driver SuperPole, 6-Car Grids, Overall Heat Time
// Classification, Configurable Q1-Q4, Odd/Even Semifinals, Row-by-Row Final,
// and Three-Stage Cumulative Points
// =============================================================================

/// Scenario: Stage 0 SuperPole Solo Run and Snake Seeding into 4 Classificatory Heats
///
/// Given a 24-driver Rallycross championship weekend begins
/// When Stage 0 (`TournamentStage::SuperPole`) starts and concludes after the player's 2-lap chute out-lap + flying lap run and deterministic AI flying-lap simulation
/// Then all 24 drivers should be ranked P1 through P24 by flying lap time and snake-seeded into 4 Classificatory Heats of 6 cars each
#[test]
fn world_rx_24_driver_superpole_and_snake_seeding_into_4_heats_of_6() {
    let def = SeriesDefinition::from_toml(WORLD_RX).unwrap();
    assert_eq!(def.series.weekend_format, WeekendFormat::TournamentSprint);
    assert_eq!(def.series.bot_count, Some(5));
    assert_eq!(
        def.series.tournament_config(),
        Some(TournamentConfig { total_drivers: 24, heat_laps: 3, semi_laps: 4, final_laps: 5, qualifying_rounds: 1 })
    );
    assert!(def.validate().is_ok());

    let entries = field(24);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::world_rx_default(), &entries, 128).unwrap();
    assert_eq!(weekend.active_stage, TournamentStage::SuperPole);
    let sp_race = weekend.player_race().expect("SuperPole scheduled").clone();
    assert_eq!(sp_race.stage, TournamentStage::SuperPole);
    assert_eq!(sp_race.laps, 2, "1 out-lap + 1 flying lap");
    assert_eq!(sp_race.driver_ids, vec!["player".to_string()], "solo player on track");

    // Submit deterministic 24-driver SuperPole flying lap times so P1..P24 order is known exactly.
    let sp_results: Vec<RoundDriverResult> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| result(&e.driver_id, i + 1, 38.0 + i as f32 * 0.25))
        .collect();
    weekend.submit_player_race(sp_results, &entries).unwrap();

    assert_eq!(weekend.superpole_results.len(), 24);
    assert_eq!(weekend.active_stage, TournamentStage::QualifyingHeat);
    assert_eq!(weekend.active_races.len(), 4);
    for h in &weekend.active_races {
        assert_eq!(h.driver_ids.len(), 6);
        assert_eq!(h.laps, 3);
    }

    let p = |rank_1_based: usize| entries[rank_1_based - 1].driver_id.clone();
    assert_eq!(weekend.active_races[0].driver_ids, vec![p(1), p(8), p(9), p(16), p(17), p(24)], "Heat 1 snake seed");
    assert_eq!(weekend.active_races[1].driver_ids, vec![p(2), p(7), p(10), p(15), p(18), p(23)], "Heat 2 snake seed");
    assert_eq!(weekend.active_races[2].driver_ids, vec![p(3), p(6), p(11), p(14), p(19), p(22)], "Heat 3 snake seed");
    assert_eq!(weekend.active_races[3].driver_ids, vec![p(4), p(5), p(12), p(13), p(20), p(21)], "Heat 4 snake seed");
}

/// Scenario: Overall Total Time Intermediate Classification and Odd/Even Semifinal Seeding
///
/// Given all 4 Classificatory Heats of 6 cars conclude their 3-lap sprint races
/// When the Intermediate Classification is computed across all 24 drivers
/// Then drivers should be ranked 1st through 24th strictly by overall total race completion time across all heats
/// (so a fast 4th-place finisher in a rapid heat outranks a slower 2nd-place finisher in a slow heat)
/// And the Top 12 should be seeded into Semifinal 1 (1, 3, 5, 7, 9, 11) and Semifinal 2 (2, 4, 6, 8, 10, 12)
#[test]
fn world_rx_overall_total_time_intermediate_classification_and_odd_even_semifinals() {
    let entries = field(24);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::world_rx_default(), &entries, 77).unwrap();
    // Complete SuperPole solo run
    drive(&mut weekend, &entries, 1);
    assert_eq!(weekend.active_stage, TournamentStage::QualifyingHeat);

    // Player is in Heat 1 and finishes 4th in a rapid heat (114.0s..115.2s), so the 4th-place player (115.2s)
    // is faster than slower drivers in other heats.
    let player_heat = weekend.player_race().unwrap().clone();
    let fast_heat: Vec<RoundDriverResult> = player_heat
        .driver_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| {
            let pos = match idx {
                0 => 4, // player finishes 4th in Heat 1 with a very fast overall time
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 5,
                _ => 6,
            };
            let time = match pos {
                1 => 114.0,
                2 => 114.4,
                3 => 114.8,
                4 => 115.2, // fast 4th place in Heat 1
                5 => 145.0,
                _ => 148.0, // e.g. +30s missed-Joker penalty
            };
            result(id, pos, time)
        })
        .collect();
    weekend.submit_player_race(fast_heat, &entries).unwrap();

    assert_eq!(weekend.intermediate_standings.len(), 24);
    // Verify intermediate_standings is strictly sorted by total_time across all 24 drivers:
    for pair in weekend.intermediate_standings.windows(2) {
        assert!(pair[0].total_time <= pair[1].total_time, "{} ({}) vs {} ({})", pair[0].driver_id, pair[0].total_time, pair[1].driver_id, pair[1].total_time);
    }

    // Even though the player finished 4th in their heat, their fast total time (115.2s) beats slower 2nd/3rd-place finishers in other heats!
    let player_inter = weekend.intermediate_standings.iter().find(|r| r.driver_id == "player").unwrap();
    assert!(player_inter.finish_position <= 12, "fast 4th-place finisher advances in Top 12 (rank {})", player_inter.finish_position);

    // Find a driver who finished 2nd or 3rd in another heat with a slower time than 115.2s:
    let slower_p2 = weekend
        .heat_results
        .iter()
        .skip(1)
        .flatten()
        .find(|r| r.finish_position == 2 && r.total_time > 115.2)
        .expect("a slower 2nd-place heat finisher exists");
    let slower_p2_inter = weekend.intermediate_standings.iter().find(|r| r.driver_id == slower_p2.driver_id).unwrap();
    assert!(
        player_inter.finish_position < slower_p2_inter.finish_position,
        "Fast 4th-place heat finisher (rank {}, {:.2}s) must outrank slower 2nd-place heat finisher (rank {}, {:.2}s)",
        player_inter.finish_position,
        player_inter.total_time,
        slower_p2_inter.finish_position,
        slower_p2_inter.total_time
    );

    // Verify Odd/Even Semifinal seeding:
    // Semi 1: ranks 1, 3, 5, 7, 9, 11 (indices 0, 2, 4, 6, 8, 10)
    // Semi 2: ranks 2, 4, 6, 8, 10, 12 (indices 1, 3, 5, 7, 9, 11)
    assert_eq!(weekend.active_stage, TournamentStage::Semifinal);
    assert_eq!(weekend.active_races.len(), 2);
    let expected_semi1: Vec<String> = [0, 2, 4, 6, 8, 10].into_iter().map(|i| weekend.intermediate_standings[i].driver_id.clone()).collect();
    let expected_semi2: Vec<String> = [1, 3, 5, 7, 9, 11].into_iter().map(|i| weekend.intermediate_standings[i].driver_id.clone()).collect();
    assert_eq!(weekend.active_races[0].driver_ids, expected_semi1);
    assert_eq!(weekend.active_races[1].driver_ids, expected_semi2);
}

/// Scenario: Configurable Multi-Round Qualifying Heats (Q1–Q4) with Default Single Qualifying Round
///
/// Given `TournamentConfig` supports a configurable `qualifying_rounds` setting (`1..=4`, defaulting to `1`)
/// When a Rallycross weekend is configured with `qualifying_rounds = 3` (`Q1, Q2, Q3`)
/// Then the weekend should run 3 sequential Qualifying Heat rounds of 4 heats each before computing
/// the Intermediate Classification by cumulative total time across all 3 Q rounds and advancing Top 12 to Semifinals
#[test]
fn world_rx_configurable_multi_round_qualifying_heats_q1_to_q3() {
    let entries = field(24);
    let config = TournamentConfig { qualifying_rounds: 3, ..TournamentConfig::world_rx_default() };
    assert!(config.validate().is_ok());

    let mut weekend = TournamentWeekendState::new(config, &entries, 303).unwrap();
    assert_eq!(drive(&mut weekend, &entries, 1), TournamentStage::SuperPole);

    // Q1 -> stays in QualifyingHeat for Q2
    assert_eq!((weekend.active_stage, weekend.current_qualifying_round), (TournamentStage::QualifyingHeat, 0));
    drive(&mut weekend, &entries, 1);
    assert_eq!(weekend.qualifying_round_results.len(), 1);
    assert_eq!((weekend.active_stage, weekend.current_qualifying_round), (TournamentStage::QualifyingHeat, 1));

    // Verify Q2 was snake-seeded from Q1's overall time ranking
    let mut q1_ranked: Vec<RoundDriverResult> = weekend.qualifying_round_results[0].iter().flatten().cloned().collect();
    q1_ranked.sort_by(|a, b| a.total_time.total_cmp(&b.total_time));
    assert_eq!(weekend.active_races[0].driver_ids[0], q1_ranked[0].driver_id, "Q2 Heat 1 pole is Q1 overall fastest");
    assert_eq!(weekend.active_races[1].driver_ids[0], q1_ranked[1].driver_id, "Q2 Heat 2 pole is Q1 2nd fastest");

    // Q2 -> stays in QualifyingHeat for Q3
    drive(&mut weekend, &entries, 2);
    assert_eq!(weekend.qualifying_round_results.len(), 2);
    assert_eq!((weekend.active_stage, weekend.current_qualifying_round), (TournamentStage::QualifyingHeat, 2));

    // Q3 -> advances to Semifinal after all 3 Q rounds complete
    drive(&mut weekend, &entries, 1);
    assert_eq!(weekend.qualifying_round_results.len(), 3);
    assert_eq!(weekend.active_stage, TournamentStage::Semifinal);
    assert_eq!(weekend.intermediate_standings.len(), 24);

    // Verify every driver's intermediate_standings total_time equals the sum of their Q1 + Q2 + Q3 heat times
    for row in &weekend.intermediate_standings {
        let sum_q: f32 = weekend
            .qualifying_round_results
            .iter()
            .map(|q| q.iter().flatten().find(|r| r.driver_id == row.driver_id).unwrap().total_time)
            .sum();
        assert!((row.total_time - sum_q).abs() < 1e-3, "{} cumulative {} vs {}", row.driver_id, row.total_time, sum_q);
    }
}

/// Scenario: Position-Only Semifinal Advancement and Row-by-Row Final Grid Pairing
///
/// Given Semifinal 1 (6 cars) and Semifinal 2 (6 cars) conclude their 4-lap sprint races
/// When the Grand Final grid is assembled
/// Then the Top 3 finishers by on-track position from each Semifinal should advance to the 6-car Grand Final
/// paired row by row (Row 1: Semi 1 P1 & Semi 2 P1, Row 2: Semi 1 P2 & Semi 2 P2, Row 3: Semi 1 P3 & Semi 2 P3),
/// with the driver holding the higher Intermediate Classification rank taking the leading slot in each row
#[test]
fn world_rx_position_only_semifinals_and_row_by_row_grand_final_pairing() {
    let entries = field(24);
    let mut weekend = TournamentWeekendState::new(TournamentConfig::world_rx_default(), &entries, 19).unwrap();
    drive(&mut weekend, &entries, 1); // SuperPole
    drive(&mut weekend, &entries, 1); // Q1 Heat
    assert_eq!(weekend.active_stage, TournamentStage::Semifinal);
    drive(&mut weekend, &entries, 2); // Semifinal (player finishes 2nd)

    assert_eq!(weekend.semi_results.len(), 2);
    let semi1 = &weekend.semi_results[0];
    let semi2 = &weekend.semi_results[1];
    assert_eq!((semi1.len(), semi2.len()), (6, 6));

    let grand = weekend.active_races.iter().find(|r| r.stage == TournamentStage::GrandFinal).unwrap();
    let b_final = weekend.active_races.iter().find(|r| r.stage == TournamentStage::ConsolationBFinal).unwrap();
    assert_eq!(grand.driver_ids.len(), 6);
    assert_eq!(b_final.driver_ids.len(), 6);

    let inter_rank = |id: &str| {
        weekend
            .intermediate_standings
            .iter()
            .position(|r| r.driver_id == id)
            .unwrap()
    };

    for row in 0..3 {
        let s1 = &semi1[row].driver_id;
        let s2 = &semi2[row].driver_id;
        let expected_pair = if inter_rank(s1) <= inter_rank(s2) {
            vec![s1.clone(), s2.clone()]
        } else {
            vec![s2.clone(), s1.clone()]
        };
        assert_eq!(
            &grand.driver_ids[row * 2..row * 2 + 2],
            expected_pair.as_slice(),
            "Row {} of Grand Final must pair Semi 1 P{} and Semi 2 P{} ordered by Intermediate Classification rank",
            row + 1,
            row + 1,
            row + 1
        );
    }
}

/// Scenario: Option A Continuous Player Racing Across a 24-Driver RX Weekend
///
/// Given the player finishes 13th–24th in the Intermediate Classification or 4th–6th in a Semifinal
/// When subsequent stages of the Rallycross weekend are scheduled
/// Then the player should be placed into a 6-car ConsolationSemifinal and/or 6-car ConsolationBFinal
#[test]
fn world_rx_continuous_player_racing_across_24_driver_weekend() {
    let entries = field(24);
    // Case 1: Player finishes last in Heat with a slow time -> ranks 13th–24th in Intermediate Classification -> ConsolationSemifinal -> ConsolationBFinal
    let mut weekend = TournamentWeekendState::new(TournamentConfig::world_rx_default(), &entries, 55).unwrap();
    assert_eq!(drive(&mut weekend, &entries, 24), TournamentStage::SuperPole);
    let heat_race = weekend.player_race().unwrap().clone();
    let mut slow_heat = race_results(&heat_race, 6);
    for r in &mut slow_heat {
        if r.driver_id == "player" {
            r.total_time = 240.0;
        }
    }
    weekend.submit_player_race(slow_heat, &entries).unwrap();

    assert_eq!(weekend.active_stage, TournamentStage::ConsolationSemifinal);
    let cons_semi = weekend.player_race().unwrap().clone();
    assert_eq!(cons_semi.driver_ids.len(), 6);
    assert!(cons_semi.driver_ids.contains(&"player".to_string()));
    assert_eq!(drive(&mut weekend, &entries, 2), TournamentStage::ConsolationSemifinal);

    assert_eq!(weekend.active_stage, TournamentStage::ConsolationBFinal);
    let b_final = weekend.player_race().unwrap().clone();
    assert_eq!(b_final.driver_ids.len(), 6);
    assert!(b_final.driver_ids.contains(&"player".to_string()));
    assert_eq!(drive(&mut weekend, &entries, 1), TournamentStage::ConsolationBFinal);
    assert!(weekend.is_complete);
    assert_eq!(weekend.final_standings().unwrap().len(), 24);

    // Case 2: Player qualifies in Top 12, then finishes 4th, 5th, or 6th in Semifinal -> ConsolationBFinal (6 cars)
    for semi_pos in 4..=6 {
        let mut weekend = TournamentWeekendState::new(TournamentConfig::world_rx_default(), &entries, 55).unwrap();
        drive(&mut weekend, &entries, 1); // SuperPole
        drive(&mut weekend, &entries, 1); // Q1 (fast 1st place -> Top 12)
        assert_eq!(weekend.active_stage, TournamentStage::Semifinal);
        drive(&mut weekend, &entries, semi_pos);
        assert_eq!(weekend.active_stage, TournamentStage::ConsolationBFinal);
        let b_final = weekend.player_race().unwrap().clone();
        assert_eq!(b_final.driver_ids.len(), 6);
        assert!(b_final.driver_ids.contains(&"player".to_string()));
        drive(&mut weekend, &entries, 1);
        assert!(weekend.is_complete);
    }
}

/// Scenario: Three-Stage Cumulative Championship Points Allocation
///
/// Given a 24-driver Rallycross weekend completes the Grand Final and B-Final
/// When championship points are awarded for the round
/// Then each driver's weekend points should equal the sum of their Intermediate Classification points (16..=5 for Top 12),
/// Semifinal points (6..=1 for 1st..=6th in Semi 1 and Semi 2), and Grand Final points (8, 5, 4, 3, 2, 1 for 1st..=6th in Grand Final),
/// up to a maximum of 30 points
#[test]
fn world_rx_three_stage_cumulative_championship_points() {
    let def = SeriesDefinition::from_toml(WORLD_RX).unwrap();
    let mut session = def.to_session();
    session.sync_initial_grid_slots(24, &tdrace_app::ai::DriverCharacter::all_across_modules());
    session.ensure_tournament_weekend().unwrap();

    // 1. SuperPole: Player sets fastest flying lap (28.0s)
    let sp_race = session.tournament_race().unwrap().clone();
    assert_eq!(sp_race.stage, TournamentStage::SuperPole);
    let mut sp_res = race_results(&sp_race, 1);
    sp_res[0].total_time = 28.0;
    sp_res[0].best_lap = Some(28.0);
    assert!(!session.submit_tournament_race("Holjes", sp_res).unwrap());

    // 2. Q1 Heat: Player wins Heat 1 with a rapid time (95.0s) so Player is 1st in Intermediate Classification (16 pts)
    let heat_race = session.tournament_race().unwrap().clone();
    assert_eq!(heat_race.stage, TournamentStage::QualifyingHeat);
    let mut heat_res = race_results(&heat_race, 1);
    for (i, r) in heat_res.iter_mut().enumerate() {
        r.total_time = 95.0 + i as f32 * 1.2;
    }
    assert!(!session.submit_tournament_race("Holjes", heat_res).unwrap());

    // 3. Semifinal: Player wins Semifinal 1 (6 pts)
    let semi_race = session.tournament_race().unwrap().clone();
    assert_eq!(semi_race.stage, TournamentStage::Semifinal);
    assert!(!session.submit_tournament_race("Holjes", race_results(&semi_race, 1)).unwrap());

    // Capture the weekend state right before the Grand Final so we can verify every driver's 3-stage breakdown
    let weekend_before_final = session.tournament.as_ref().unwrap().clone();

    // 4. Grand Final: Player wins Grand Final (8 pts) -> 16 + 6 + 8 = 30 pts total!
    let gf_race = session.tournament_race().unwrap().clone();
    assert_eq!(gf_race.stage, TournamentStage::GrandFinal);
    let gf_results = race_results(&gf_race, 1);
    let mut completed_weekend = weekend_before_final;
    completed_weekend.submit_player_race(gf_results.clone(), &session.standings).unwrap();

    assert!(session.submit_tournament_race("Holjes", gf_results).unwrap(), "weekend finishes");
    assert_eq!(session.current_round, 1);
    let round = &session.history[0];
    assert_eq!(round.results.len(), 24);

    for r in &round.results {
        let (inter_pts, semi_pts, final_pts) = completed_weekend.driver_stage_points(&r.driver_id);
        assert_eq!(r.points_awarded, inter_pts + semi_pts + final_pts, "3-stage sum for {}", r.driver_id);
    }

    let player_res = round.results.iter().find(|r| r.driver_id == "player").unwrap();
    assert_eq!(
        player_res.points_awarded,
        WORLD_RX_INTERMEDIATE_POINTS_12[0] + WORLD_RX_SEMI_POINTS_6[0] + WORLD_RX_FINAL_POINTS_6[0]
    );
    assert_eq!(player_res.points_awarded, 30, "perfect weekend earns 16 + 6 + 8 = 30 pts");

    let expected_total_pool: u32 = WORLD_RX_INTERMEDIATE_POINTS_12.iter().sum::<u32>()
        + 2 * WORLD_RX_SEMI_POINTS_6.iter().sum::<u32>()
        + WORLD_RX_FINAL_POINTS_6.iter().sum::<u32>();
    let actual_total_pool: u32 = session.standings.iter().map(|s| s.points).sum();
    assert_eq!(actual_total_pool, expected_total_pool);
}

