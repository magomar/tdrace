//! Spec 082: the joker pill states and the results-screen joker column.

use race_ui::hud::widgets::JokerBadge;
use tdrace_app::ui::menu::{race_results_table, RaceResultEntry};

#[test]
fn joker_pill_is_amber_until_taken_red_on_the_last_lap_and_off_without_the_rule() {
    // 5-lap race, one mandatory joker.
    assert_eq!(JokerBadge::for_driver(1, 0, 1, 5), Some(JokerBadge::Pending));
    assert_eq!(JokerBadge::for_driver(1, 0, 4, 5), Some(JokerBadge::Pending));
    assert_eq!(JokerBadge::for_driver(1, 1, 3, 5), Some(JokerBadge::Done));
    // Scenario: Final-lap warning
    assert_eq!(JokerBadge::for_driver(1, 0, 5, 5), Some(JokerBadge::LastLap));
    assert_eq!(JokerBadge::for_driver(1, 1, 5, 5), Some(JokerBadge::Done));
    // Scenario: Other categories are not affected
    assert_eq!(JokerBadge::for_driver(0, 0, 5, 5), None);
    assert_eq!(JokerBadge::LastLap.label(), "JOKER THIS LAP!");
}

#[test]
fn results_show_the_joker_column_only_in_joker_races() {
    let row = |position: usize, car_idx: usize, total_time: f32, jokers: Option<u32>, penalty: f32| RaceResultEntry {
        position,
        car_name: format!("Driver {}", car_idx),
        is_player: false,
        total_time,
        best_lap: Some(50.0),
        delta_to_leader: 0.0,
        car_idx,
        points_awarded: 0,
        projected: false,
        jokers,
        penalty,
    };
    // Scenario: A missed joker adds 30 s: the penalised car shows "+30.0s NO JOKER".
    let results = vec![row(1, 1, 255.0, Some(1), 0.0), row(2, 0, 280.0, Some(0), 30.0)];
    let table = race_results_table(&results, false);
    let joker_col = table.columns.iter().position(|c| c.id == "joker").expect("joker column in a joker race");
    let cell = |r: usize| (table.columns[joker_col].extractor)(&table.rows[r].data);
    assert_eq!(cell(0), "J");
    assert_eq!(cell(1), "+30.0s NO JOKER");

    let plain = vec![row(1, 0, 250.0, None, 0.0)];
    assert!(race_results_table(&plain, false).columns.iter().all(|c| c.id != "joker"));
}
