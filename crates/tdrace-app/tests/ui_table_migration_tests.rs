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

#[test]
fn test_championship_standings_table_32_drivers_pages_16_per_page() {
    use tdrace_app::series::{PointSystem, SeriesSession, SeriesStandingEntry};
    use tdrace_app::ui::menu::{championship_standings_page_rects, championship_standings_table, STANDINGS_PAGE_SIZE};

    assert_eq!(STANDINGS_PAGE_SIZE, 16);

    let mut standings = Vec::new();
    for i in 1..=32 {
        let id = if i == 1 { "player".to_string() } else { format!("bot_{}", i) };
        let name = if i == 1 { "Player One".to_string() } else { format!("Driver {}", i) };
        let mut entry = SeriesStandingEntry::new(id, name, "Apex Racing");
        entry.points = (33 - i) as u32;
        standings.push(entry);
    }

    let mut session = SeriesSession::new("Autocross Cup", PointSystem::FiaStandard { fastest_lap_bonus: false }, vec!["track1".into()], 3, &[]);
    session.standings = standings;

    // Page 0 (drivers 1 to 16)
    let table_p0 = championship_standings_table(&session, 0);
    assert_eq!(table_p0.len(), 16);
    assert_eq!(table_p0.rows[0].rank, Some(1));
    assert!(table_p0.rows[0].is_player);
    assert_eq!(table_p0.rows[15].rank, Some(16));
    assert!(!table_p0.rows[15].is_player);

    let cell_pos = |col: usize, row: usize, t: &cabinet::ui::data_table::DataTable<_>| (t.columns[col].extractor)(&t.rows[row].data);
    assert_eq!(cell_pos(0, 0, &table_p0), "#1");
    assert_eq!(cell_pos(1, 0, &table_p0), "Player One");
    assert_eq!(cell_pos(4, 0, &table_p0), "32 PTS");
    assert_eq!(cell_pos(0, 15, &table_p0), "#16");
    assert_eq!(cell_pos(1, 15, &table_p0), "Driver 16");
    assert_eq!(cell_pos(4, 15, &table_p0), "17 PTS");

    // Page 1 (drivers 17 to 32)
    let table_p1 = championship_standings_table(&session, 1);
    assert_eq!(table_p1.len(), 16);
    assert_eq!(table_p1.rows[0].rank, Some(17));
    assert_eq!(table_p1.rows[15].rank, Some(32));
    assert_eq!(cell_pos(0, 0, &table_p1), "#17");
    assert_eq!(cell_pos(1, 0, &table_p1), "Driver 17");
    assert_eq!(cell_pos(4, 0, &table_p1), "16 PTS");
    assert_eq!(cell_pos(0, 15, &table_p1), "#32");
    assert_eq!(cell_pos(1, 15, &table_p1), "Driver 32");
    assert_eq!(cell_pos(4, 15, &table_p1), "1 PTS");

    // Out of range page clamps to last page
    let table_clamped = championship_standings_table(&session, 99);
    assert_eq!(table_clamped.rows[0].rank, Some(17));

    // Page rects
    let rects_32 = championship_standings_page_rects(1280.0, 720.0, 2);
    assert_eq!(rects_32.len(), 2);
    assert!(rects_32[0].x < rects_32[1].x);
    assert_eq!(rects_32[0].y, rects_32[1].y);

    // 12-driver series has 1 page and no navigation rects
    let mut small_session = SeriesSession::new("Kart Cup", PointSystem::FiaStandard { fastest_lap_bonus: false }, vec!["track1".into()], 3, &[]);
    small_session.standings = session.standings[0..12].to_vec();
    let table_small = championship_standings_table(&small_session, 0);
    assert_eq!(table_small.len(), 12);
    let rects_12 = championship_standings_page_rects(1280.0, 720.0, 1);
    assert!(rects_12.is_empty());
}

#[test]
fn test_championship_standings_page_navigation() {
    use tdrace_app::game::{GameState, RaceSession};
    use tdrace_app::series::{PointSystem, SeriesSession, SeriesStandingEntry};

    let _config = tdrace_app::storage::ScopedTempConfigDir::new("standings_page_nav");
    let mut session = RaceSession::new();
    session.hof_db = None;

    let mut standings = Vec::new();
    for i in 1..=32 {
        standings.push(SeriesStandingEntry::new(format!("d_{}", i), format!("Driver {}", i), "Team"));
    }
    let mut champ = SeriesSession::new("Autocross Series", PointSystem::FiaStandard { fastest_lap_bonus: false }, vec!["track1".into()], 3, &[]);
    champ.standings = standings;
    session.championship_session = Some(champ);
    session.state = GameState::ChampionshipStandings;
    session.championship_standings_page = 0;

    // Simulate Tab / Right arrow forward navigation
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_championship_standings();
    session.input.gamepad.snapshot.btn_rb_pressed = false;
    assert_eq!(session.championship_standings_page, 1, "RB navigates to page 1");

    // Wrap forward
    session.input.gamepad.snapshot.btn_rb_pressed = true;
    session.update_championship_standings();
    session.input.gamepad.snapshot.btn_rb_pressed = false;
    assert_eq!(session.championship_standings_page, 0, "RB wraps back to page 0");

    // Backward navigation
    session.input.gamepad.snapshot.btn_lb_pressed = true;
    session.update_championship_standings();
    session.input.gamepad.snapshot.btn_lb_pressed = false;
    assert_eq!(session.championship_standings_page, 1, "LB wraps backward to page 1");

    session.input.gamepad.snapshot.dpad_left_pressed = true;
    session.update_championship_standings();
    session.input.gamepad.snapshot.dpad_left_pressed = false;
    assert_eq!(session.championship_standings_page, 0, "Dpad Left moves to page 0");
}

