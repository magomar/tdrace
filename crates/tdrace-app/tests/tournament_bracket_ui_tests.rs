//! Spec 104 (`specs/104_autocross_and_rallycross_tournament_sprint_weekend_format.md`): what the tournament
//! bracket screen and the HUD stage badge show. The drawing itself needs a window, so these tests read the
//! view the screen is drawn from.

use tdrace_app::series::{
    PointSystem, RoundDriverResult, SeriesDefinition, SeriesSession, TournamentConfig, TournamentRace, TOURNAMENT_POINTS_32,
};
use tdrace_app::ui::tournament_bracket::{stage_badge, tournament_bracket_view};

const SUPERBUGGY: &str = include_str!("../../../series/autocross/autocross_superbuggy_world_series.toml");

fn weekend_session(config: TournamentConfig) -> SeriesSession {
    let mut session = SeriesSession::new("Cup", PointSystem::Custom(TOURNAMENT_POINTS_32.to_vec()), vec!["a".into()], 4, &[("player", "Player", "Apex")])
        .with_tournament(config);
    session.sync_initial_grid_slots(config.total_drivers, &tdrace_app::ai::DriverCharacter::all_across_modules());
    session.ensure_tournament_weekend().unwrap();
    session
}

fn results_for(race: &TournamentRace, player_pos: usize) -> Vec<RoundDriverResult> {
    let mut others = race.driver_ids.iter().filter(|id| *id != "player");
    (1..=race.driver_ids.len())
        .map(|pos| {
            let id = if pos == player_pos { "player".to_string() } else { others.next().unwrap().clone() };
            RoundDriverResult {
                driver_id: id.clone(),
                driver_name: id,
                team_name: "T".into(),
                finish_position: pos,
                total_time: 40.0 + pos as f32,
                best_lap: Some(10.0),
                points_awarded: 0,
                has_fastest_lap: false,
            }
        })
        .collect()
}

fn play(session: &mut SeriesSession, player_pos: usize) {
    let race = session.tournament_race().unwrap().clone();
    session.submit_tournament_race("Track", results_for(&race, player_pos)).unwrap();
}

/// Scenario: The bracket shows the 32-driver tree with the player's slot highlighted
///
/// Given a drawn weekend of 32 drivers
/// When the bracket screen is built
/// Then it has the heats, semifinals and finals columns, 4 heats of 8 driver cards with name and team,
/// the player's heat is marked as his race and the launch button names the race and its laps
#[test]
fn the_bracket_shows_32_drivers_and_the_players_slot() {
    let session = weekend_session(TournamentConfig::default());
    let view = tournament_bracket_view(&session).expect("a weekend is running");
    assert_eq!(view.columns.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(), vec!["QUALIFYING HEATS", "SEMIFINALS", "FINALS"]);

    let heats = &view.columns[0];
    assert!(heats.is_active && !view.columns[1].is_active);
    assert_eq!(heats.groups.len(), 4);
    assert!(heats.groups.iter().all(|g| g.cards.len() == 8 && g.laps == 4));
    assert_eq!(heats.groups.iter().map(|g| g.label.as_str()).collect::<Vec<_>>(), vec!["HEAT 1", "HEAT 2", "HEAT 3", "HEAT 4"]);
    assert!(heats.groups.iter().flat_map(|g| &g.cards).all(|c| !c.name.is_empty() && !c.team.is_empty()));

    let (col, group) = view.player_slot.expect("the player's slot");
    assert_eq!(col, 0);
    assert!(heats.groups[group].is_player_race, "the player's heat gets the amber border");
    assert!(heats.groups[group].cards.iter().any(|c| c.is_player));
    assert_eq!(heats.groups.iter().filter(|g| g.is_player_race).count(), 1);
    assert!(view.launch_label.contains(&format!("HEAT {}/4", group + 1)) && view.launch_label.contains("4 LAPS"), "{}", view.launch_label);

    assert_eq!(view.columns[1].groups.len(), 2);
    assert!(view.columns[1].groups.iter().all(|g| g.cards.is_empty()), "semifinals are not drawn yet");
    assert_eq!(view.columns[2].groups.iter().map(|g| g.label.as_str()).collect::<Vec<_>>(), vec!["GRAND FINAL", "B-FINAL"]);
}

/// The bracket after a stage lists finishing positions and who went on.
#[test]
fn the_bracket_after_the_heats_shows_positions_and_who_advanced() {
    let mut session = weekend_session(TournamentConfig::default());
    play(&mut session, 2);
    let view = tournament_bracket_view(&session).unwrap();

    let heats = &view.columns[0];
    for group in &heats.groups {
        assert_eq!(group.cards.iter().map(|c| c.position).collect::<Vec<_>>(), (1..=8).map(Some).collect::<Vec<_>>());
        assert!(group.cards.iter().take(4).all(|c| c.advanced == Some(true)));
        assert!(group.cards.iter().skip(4).all(|c| c.advanced == Some(false)));
        assert!(!group.is_player_race);
    }
    assert!(view.columns[1].is_active);
    assert_eq!(view.columns[1].groups.len(), 2);
    assert!(view.columns[1].groups.iter().all(|g| g.cards.len() == 8 && g.cards.iter().all(|c| c.position.is_none())));
    assert_eq!(view.player_slot.map(|(c, _)| c), Some(1));
    assert!(view.launch_label.contains("SEMIFINAL") && view.launch_label.contains("5 LAPS"), "{}", view.launch_label);
}

/// A player out of the heat is shown in a consolation race of the next stage.
#[test]
fn a_player_out_of_the_heat_has_a_consolation_slot() {
    let mut session = weekend_session(TournamentConfig::default());
    play(&mut session, 7);
    let view = tournament_bracket_view(&session).unwrap();
    let semis = &view.columns[1];
    assert_eq!(semis.groups.len(), 3, "two semifinals and the consolation race");
    let (col, group) = view.player_slot.unwrap();
    assert_eq!((col, semis.groups[group].label.as_str()), (1, "CONSOLATION SEMI"));
    assert!(semis.groups[group].cards.iter().any(|c| c.is_player));
    assert!(view.launch_label.contains("CONSOLATION SEMI"));
}

/// Scenario: HUD stage badge and lap counter follow the stage
///
/// Given the player's next race is a heat, a semifinal, a B-Final or the Grand Final
/// When the HUD badge is read
/// Then it reads HEAT x/4, SEMIFINAL x/2, B-FINAL or GRAND FINAL, with the lap count of the stage
#[test]
fn the_hud_badge_names_the_stage_and_the_laps_follow_it() {
    let mut session = weekend_session(TournamentConfig::default());
    let badge = stage_badge(&session).unwrap();
    assert!(badge.starts_with("HEAT ") && badge.ends_with("/4"), "{}", badge);
    assert_eq!(session.current_round_laps(), Some(4));

    play(&mut session, 1);
    let badge = stage_badge(&session).unwrap();
    assert!(badge.starts_with("SEMIFINAL ") && badge.ends_with("/2"), "{}", badge);
    assert_eq!(session.current_round_laps(), Some(5));

    play(&mut session, 6);
    assert_eq!(stage_badge(&session).as_deref(), Some("B-FINAL"));
    assert_eq!(session.current_round_laps(), Some(5), "the B-Final runs the semifinal distance");

    let mut session = weekend_session(TournamentConfig::default());
    play(&mut session, 1);
    play(&mut session, 1);
    assert_eq!(stage_badge(&session).as_deref(), Some("GRAND FINAL"));
    assert_eq!(session.current_round_laps(), Some(6));
    play(&mut session, 1);
    assert_eq!(stage_badge(&session), None, "no badge once the weekend is scored");
}

/// The bracket has a column per stage for 64 drivers and a car name where the series declares one.
#[test]
fn the_bracket_scales_to_64_drivers_and_shows_declared_cars() {
    let session = weekend_session(TournamentConfig { total_drivers: 64, ..TournamentConfig::default() });
    let view = tournament_bracket_view(&session).unwrap();
    assert_eq!(view.columns.iter().map(|c| c.groups.len()).collect::<Vec<_>>(), vec![8, 4, 2, 2]);
    assert_eq!(view.columns[1].title, "QUARTERFINALS");

    let def = SeriesDefinition::from_toml(SUPERBUGGY).unwrap();
    let mut session = def.to_session();
    session.sync_initial_grid_slots(32, &tdrace_app::ai::DriverCharacter::all_across_modules());
    session.ensure_tournament_weekend().unwrap();
    let view = tournament_bracket_view(&session).unwrap();
    let player = view.columns[0].groups.iter().flat_map(|g| &g.cards).find(|c| c.is_player).unwrap();
    assert!(player.car.as_deref().is_some_and(|c| !c.is_empty()), "the player's declared car is named: {:?}", player.car);
    assert_eq!(player.team, "Apex SuperBuggy Racing");

    session.tournament = None;
    assert!(tournament_bracket_view(&session).is_none());
}
