use tdrace_app::ui::menu::{race_results_table, RaceResultEntry};

#[test]
fn results_columns_preserve_order_and_racing_formatting() {
    let results = vec![
        RaceResultEntry { position: 1, car_name: "A very long driver / vehicle name".repeat(8), is_player: true, total_time: 120.0, best_lap: Some(60.0), delta_to_leader: 0.0, car_idx: 7, points_awarded: 25, projected: false, jokers: None, penalty: 0.0 },
        RaceResultEntry { position: 2, car_name: "Opponent".into(), is_player: false, total_time: 123.5, best_lap: None, delta_to_leader: 3.5, car_idx: 2, points_awarded: 18, projected: true, jokers: None, penalty: 0.0 },
    ];
    let table = race_results_table(&results, true);
    assert!(!table.is_focused);
    assert_eq!(table.rows[0].data.car_idx, 7);
    assert_eq!(table.rows[1].data.car_idx, 2);
    assert!(table.rows[0].is_player);
    assert_eq!(table.rows[0].rank, Some(1));
    let cell = |col: usize, row: usize| (table.columns[col].extractor)(&table.rows[row].data);
    assert_eq!(cell(4, 0), "-");
    assert_eq!(cell(4, 1), "~+3.50s");
    assert!(cell(2, 1).starts_with('~'));
    assert_eq!(cell(5, 0), "+25 PTS");
    assert_eq!(table.columns.len(), 6);
    assert_eq!(race_results_table(&results, false).columns.len(), 5);
    assert!(race_results_table(&[], false).is_empty());
}

#[test]
fn pause_focus_rows_and_setting_actions_preserve_paused_state() {
    use tdrace_app::game::{GameState, RaceSession};
    let _config = tdrace_app::storage::ScopedTempConfigDir::new("pause_settings_migration");
    let mut session = RaceSession::new();
    // Exercise UI state without persisting a mode change to the shared test database.
    session.hof_db = None;
    session.state = GameState::Paused;
    session.pause_nav.move_down();
    assert_eq!(session.pause_nav.active_row(), 1);
    let previous = session.assist_profile;
    session.adjust_pause_setting(session.pause_nav.active_row(), false);
    assert_ne!(session.assist_profile, previous);
    assert_eq!(session.state, GameState::Paused);
    session.pause_nav.set_focus(0, 2);
    let muted = session.audio.settings.is_music_muted;
    session.adjust_pause_setting(session.pause_nav.active_row(), false);
    assert_ne!(session.audio.settings.is_music_muted, muted);
    session.pause_nav.set_focus(0, 4);
    session.audio.set_master_volume(0.5);
    session.adjust_pause_setting(session.pause_nav.active_row(), true);
    assert!((session.audio.settings.master_volume - 0.45).abs() < 0.001);
    assert_eq!(session.state, GameState::Paused);
}
