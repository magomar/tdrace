//! Spec 104 (`specs/104_autocross_and_rallycross_tournament_sprint_weekend_format.md`): a weekend through
//! the game session: roster, laps, grids, results screen, bracket and standings.

use std::collections::HashSet;

use race_kit::{DnfCause, FinishState};
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::series::{SeriesDefinition, TournamentStage, DNF_TIME, TOURNAMENT_POINTS_32};

const SUPERBUGGY: &str = include_str!("../../../series/autocross/autocross_superbuggy_world_series.toml");
const WORLD_RX: &str = include_str!("../../../series/rally/rally_world_rallycross_supercars.toml");

fn launch(toml: &str) -> RaceSession {
    let def = SeriesDefinition::from_toml(toml).unwrap();
    let mut session = RaceSession::new();
    session.launch_or_resume_championship(&def);
    session
}

/// Ends the race on the track: the player takes `player_rank`, the other cars follow in grid order.
fn finish_race(session: &mut RaceSession, player_rank: usize) {
    let n = session.world.vehicles.len();
    let mut order: Vec<usize> = (1..n).collect();
    order.insert(player_rank - 1, 0);
    let laps = session.total_laps;
    for (rank, &car) in order.iter().enumerate() {
        let time = 10.0 * laps as f32 + rank as f32 * 1.5;
        session.world.finish[car] = FinishState::Finished { time, position: rank + 1 };
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

/// The drivers on the grid, in grid order.
fn grid_driver_ids(session: &RaceSession) -> Vec<String> {
    session
        .grid_participants
        .iter()
        .map(|p| if p.is_player { "player".to_string() } else { session.opponent_drivers[p.bot_index.unwrap()].id.to_string() })
        .collect()
}

fn next_race(session: &RaceSession) -> tdrace_app::series::TournamentRace {
    session.championship_session.as_ref().unwrap().tournament_race().unwrap().clone()
}

/// Scenario: A weekend runs through the game, one 8-car race per stage
///
/// Given the World SuperBuggy Series launched from the profile
/// When the player races the heat, the semifinal and the Grand Final
/// Then each race has the 8 drivers of the schedule on its grid with the stage laps,
/// the bracket opens between stages and the standings hold the 32-place points at the end
#[test]
fn a_weekend_runs_through_the_game_session() {
    let mut session = launch(SUPERBUGGY);
    assert_eq!(session.state, GameState::TournamentBracket, "the weekend opens on its bracket");
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.standings.len(), 32);
    let heat = next_race(&session);
    assert_eq!((heat.stage, heat.laps), (TournamentStage::QualifyingHeat, 4));

    // Heat: [LAUNCH RACE] on the bracket starts the 3D race.
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!((session.total_laps, session.world.vehicles.len()), (4, 8));
    assert_eq!(grid_driver_ids(&session).into_iter().collect::<HashSet<_>>(), heat.driver_ids.iter().cloned().collect::<HashSet<_>>());
    finish_race(&mut session, 1);
    assert_eq!(session.state, GameState::Finished);
    assert!(session.results.iter().all(|r| r.points_awarded == 0), "a heat scores nothing by itself");
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::TournamentBracket, "the bracket shows the next race");
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.current_round, 0, "the round is scored after the finals");
    assert!(champ.standings.iter().all(|s| s.points == 0));

    // Semifinal: 5 laps, the grid is the semifinal field in heat-time order.
    let semi = next_race(&session);
    assert_eq!((semi.stage, semi.laps), (TournamentStage::Semifinal, 5));
    press_confirm(&mut session);
    assert_eq!((session.total_laps, session.world.vehicles.len()), (5, 8));
    assert_eq!(grid_driver_ids(&session), semi.driver_ids, "the grid follows the seeding");
    finish_race(&mut session, 3);
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::TournamentBracket);

    // Grand Final: 6 laps.
    let grand = next_race(&session);
    assert_eq!((grand.stage, grand.laps), (TournamentStage::GrandFinal, 6));
    press_confirm(&mut session);
    assert_eq!(session.total_laps, 6);
    assert_eq!(grid_driver_ids(&session), grand.driver_ids);
    finish_race(&mut session, 1);
    press_confirm(&mut session);

    assert_eq!(session.state, GameState::ChampionshipStandings, "the finals end the weekend");
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.current_round, 1);
    assert!(champ.tournament.is_none());
    assert_eq!(champ.standings.iter().map(|s| s.points).sum::<u32>(), TOURNAMENT_POINTS_32.iter().sum::<u32>());
    assert_eq!(champ.standings[0].driver_id, "player");
    assert_eq!(champ.standings[0].points, 25);
}

/// Option A through the game: a player who finishes 6th in the heat is on the grid of a consolation
/// semifinal and then of the B-Final, and never lands on a screen without a race to launch.
#[test]
fn a_player_eliminated_in_the_heat_races_on_in_the_game() {
    let mut session = launch(WORLD_RX);
    press_confirm(&mut session);
    finish_race(&mut session, 6);
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::TournamentBracket);
    assert_eq!(next_race(&session).stage, TournamentStage::ConsolationSemifinal);

    press_confirm(&mut session);
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!((session.total_laps, session.world.vehicles.len()), (5, 8));
    assert!(grid_driver_ids(&session).contains(&"player".to_string()));
    finish_race(&mut session, 2);
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::TournamentBracket);
    assert_eq!(next_race(&session).stage, TournamentStage::ConsolationBFinal);

    press_confirm(&mut session);
    assert_eq!(session.state, GameState::StartingGrid);
    finish_race(&mut session, 1);
    press_confirm(&mut session);
    assert_eq!(session.state, GameState::ChampionshipStandings);
    let champ = session.championship_session.as_ref().unwrap();
    let player = champ.standings.iter().find(|s| s.driver_id == "player").unwrap();
    assert_eq!(player.points, TOURNAMENT_POINTS_32[8], "the player won the B-Final: 9th overall");
}

/// A wrecked car is last in its heat whatever its wreck time, in the game as in the engine.
#[test]
fn a_wrecked_car_is_classified_last_in_the_game() {
    let mut session = launch(SUPERBUGGY);
    press_confirm(&mut session);
    let wrecked_car = 1;
    let wrecked_id = session.opponent_drivers[session.grid_participants.iter().find(|p| p.bot_index == Some(wrecked_car - 1)).unwrap().bot_index.unwrap()].id.to_string();
    finish_race(&mut session, 1);
    session.world.finish[wrecked_car] = FinishState::Dnf { time: 1.0, cause: DnfCause::Impact };
    session.check_race_finish();
    let pending = session.pending_championship_results.clone().unwrap();
    let last = pending.last().unwrap();
    assert_eq!((last.driver_id.as_str(), last.finish_position), (wrecked_id.as_str(), 8));
    assert!(last.total_time >= DNF_TIME, "no wreck time can seed a later stage");
    press_confirm(&mut session);
    let weekend = session.championship_session.as_ref().unwrap().tournament.as_ref().unwrap();
    let heat = weekend.heat_results.iter().find(|h| h.iter().any(|r| r.driver_id == wrecked_id)).unwrap();
    assert_eq!(heat.last().unwrap().driver_id, wrecked_id);
}
