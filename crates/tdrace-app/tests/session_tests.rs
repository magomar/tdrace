use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_session_initialization() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    session.init_race();
    let expected_cars = 1 + session.num_bots;
    assert_eq!(session.cars.len(), expected_cars);
    assert_eq!(session.trackers.len(), expected_cars);
    assert_eq!(session.ai_drivers.len(), session.num_bots);
    assert_eq!(session.color_schemes.len(), expected_cars);

    match session.state {
        GameState::StartingGrid => (),
        _ => panic!("Expected StartingGrid state after init_race vs AI"),
    }
}

#[test]
fn test_session_track_and_car_selection() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::KartArena;
    session.car_choice = CarChoice::Kart;
    session.num_bots = 5;
    session.init_race();

    assert_eq!(session.track.name, "Kart Arena");
    assert_eq!(session.cars.len(), 6);
    assert_eq!(session.cars[0].config.mass, 180.0); // Kart mass
}

#[test]
fn test_session_time_attack_mode() {
    let mut session = RaceSession::new();
    session.is_time_attack = true;
    session.init_race();

    assert_eq!(session.cars.len(), 1); // Solo player
    assert_eq!(session.ai_drivers.len(), 0);
}

#[test]
fn test_session_standings_computation() {
    let mut session = RaceSession::new();
    session.num_bots = 3;
    session.init_race();

    // Advance car 2 to lap 2
    session.trackers[2].current_lap = 2;
    session.trackers[2].normalized_progress = 0.35;

    // Advance car 0 to lap 1 with 0.80 progress
    session.trackers[0].current_lap = 1;
    session.trackers[0].normalized_progress = 0.80;

    // Car 1 on lap 1 with 0.40 progress
    session.trackers[1].current_lap = 1;
    session.trackers[1].normalized_progress = 0.40;

    let standings = session.compute_standings();
    assert_eq!(standings[0], 2); // Car 2 is P1
    assert_eq!(standings[1], 0); // Car 0 is P2
    assert_eq!(standings[2], 1); // Car 1 is P3
}

#[test]
fn test_player_lap_tracking_advancement_and_finish() {
    let mut session = RaceSession::new();
    session.init_race();

    assert_eq!(session.prev_player_lap, 1);
    assert_eq!(session.trackers[0].current_lap, 1);

    // Simulate crossing start/finish line for lap 2
    session.trackers[0].current_lap = 2;

    // Run physics step
    session.physics_step(1.0 / 60.0);

    // prev_player_lap must be synchronized to lap 2 so sound triggers only once
    assert_eq!(session.prev_player_lap, 2);

    // Subsequent steps must remain synchronized
    session.physics_step(1.0 / 60.0);
    assert_eq!(session.prev_player_lap, 2);
    assert_eq!(session.trackers[0].current_lap, 2);

    // Complete all laps (e.g., total_laps + 1)
    session.trackers[0].current_lap = session.total_laps + 1;
    session.physics_step(1.0 / 60.0);
    assert_eq!(session.prev_player_lap, session.total_laps + 1);

    // Check race finish transition
    session.check_race_finish();
    assert_eq!(
        session.state,
        GameState::Finished,
        "State should transition to Finished upon race completion"
    );
}

#[test]
fn test_session_update_state_preservation_and_race_start() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // 1. Initializing race puts state into StartingGrid
    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);

    // 2. update() MUST NOT overwrite StartingGrid back to Menu
    session.update();
    assert_eq!(session.state, GameState::StartingGrid);

    // 3. Countdown state must be preserved during update()
    session.state = GameState::Countdown(3.0);
    session.update();
    match session.state {
        GameState::Countdown(rem) => assert!(rem < 3.0, "Countdown should progress with frame dt"),
        _ => panic!("Expected Countdown state to be preserved across update()"),
    }

    // 4. Racing state must be preserved during update()
    session.state = GameState::Racing;
    session.update();
    assert_eq!(session.state, GameState::Racing);

    // 5. Paused state must be preserved during update()
    session.state = GameState::Paused;
    session.update();
    assert_eq!(session.state, GameState::Paused);

    // 6. Finished state must be preserved during update()
    session.state = GameState::Finished;
    session.update();
    assert_eq!(session.state, GameState::Finished);

    // 7. ControlsHelp state must be preserved during update()
    session.state = GameState::ControlsHelp(false);
    session.update();
    assert_eq!(session.state, GameState::ControlsHelp(false));
}

#[test]
fn test_main_menu_exit_confirmation_state() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert!(!session.show_exit_confirm);

    // Triggering exit confirmation modal
    session.show_exit_confirm = true;
    assert!(session.show_exit_confirm, "show_exit_confirm should be true when exit modal is open");
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 }, "State should remain GameState::ModuleSelect");

    // Dismissing exit confirmation modal
    session.show_exit_confirm = false;
    assert!(!session.show_exit_confirm, "show_exit_confirm should be false after dismissal");
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 }, "State should remain GameState::ModuleSelect");
}

#[test]
fn test_all_races_and_modules_default_to_eight_riders() {
    // 1. Default initialization
    let mut session = RaceSession::new();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Default session should have 8 riders (1 player + 7 bots)");
    assert_eq!(session.ai_drivers.len(), 7, "Default session should have 7 AI bots");
    assert_eq!(session.opponent_drivers.len(), 7);

    // 2. Switch to F1
    session.switch_to_f1();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "F1 module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 3. Switch to Rally
    session.switch_to_rally();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Rally module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 4. Switch to Kart
    session.switch_to_kart();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Kart module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 5. Switch to Classic
    session.switch_to_classic();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Classic module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 6. Test all preset tracks in classic mode
    for track_choice in [
        TrackChoice::ClassicGrandPrix,
        TrackChoice::OvalSpeedway,
        TrackChoice::DriftPark,
        TrackChoice::KartArena,
        TrackChoice::RampRaceway,
        TrackChoice::OasisRally,
        TrackChoice::OutlawPass,
    ] {
        session.track_choice = track_choice;
        session.init_race();
        assert_eq!(
            session.cars.len(),
            8,
            "Track {:?} should have 8 riders by default",
            session.track_choice
        );
        assert_eq!(session.ai_drivers.len(), 7);
        assert_eq!(session.opponent_drivers.len(), 7);
    }
}

#[test]
fn test_grid_positioning_fast_lap_earns_pole() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    // Set a fast player personal best on this track
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 18.2);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 55.0);

    session.init_race();

    // Player should be at grid slot 0 (Pole Position)
    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();
    assert_eq!(player_slot, 0, "Player with fastest lap time should start on Pole (slot 0)");

    // Verify car 0 is spawned at grid_positions[0]
    let player_car = &session.cars[0];
    let grid_pose_0 = &session.track.grid_positions[0];
    assert!((player_car.state.position.x - grid_pose_0.position.x).abs() < 1e-3);
    assert!((player_car.state.position.y - grid_pose_0.position.y).abs() < 1e-3);
}

#[test]
fn test_grid_positioning_slower_lap_placed_behind() {
    use tdrace_app::ai::DriverCharacter;
    use tdrace_app::db::HallOfFameEntry;

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;

    // All possible opponents have 20.0s in HoF
    session.hof_entries = DriverCharacter::all()
        .iter()
        .map(|d| HallOfFameEntry {
            id: Some(1),
            track_id: "classic_grand_prix".to_string(),
            player_name: d.name.to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time: 60.0,
            best_lap: Some(20.0),
            laps: 3,
            created_at: "2026-08-27 10:00".to_string(),
        })
        .collect();

    // Player has slower lap time 25.0s
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 25.0);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 75.0);

    // Rebuild grid
    session.rebuild_roster_participants();

    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();

    // Player with 25.0s should be placed behind all opponents with 20.0s
    assert_eq!(player_slot, session.grid_participants.len() - 1, "Player with 25.0s should be at back of grid");
}

#[test]
fn test_grid_positioning_tie_broken_by_circuit_time() {
    use tdrace_app::ai::DriverCharacter;
    use tdrace_app::db::HallOfFameEntry;

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;

    // All opponents have 22.0s lap and 68.0s circuit time
    session.hof_entries = DriverCharacter::all()
        .iter()
        .map(|d| HallOfFameEntry {
            id: Some(1),
            track_id: "classic_grand_prix".to_string(),
            player_name: d.name.to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time: 68.0,
            best_lap: Some(22.0),
            laps: 3,
            created_at: "2026-08-27 10:00".to_string(),
        })
        .collect();

    // Player has identical best lap (22.0s), but FASTER circuit time (64.0s)
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 22.0);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 64.0);

    session.rebuild_roster_participants();

    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();

    assert_eq!(player_slot, 0, "Player with faster circuit time on tied lap should earn P1");
}

#[test]
fn test_grid_positioning_all_slots_unique_and_valid() {
    let mut session = RaceSession::new();
    session.init_race();

    assert_eq!(session.grid_participants.len(), 8);
    // Ensure all cars are placed at valid unique positions
    let mut positions = Vec::new();
    for (i, p) in session.grid_participants.iter().enumerate() {
        let car_pose = if p.is_player {
            session.cars[0].state.position
        } else {
            let bot_idx = p.bot_index.unwrap();
            session.cars[bot_idx + 1].state.position
        };
        let expected_slot_pose = session.track.grid_positions[i].position;
        assert!((car_pose.x - expected_slot_pose.x).abs() < 1e-3);
        assert!((car_pose.y - expected_slot_pose.y).abs() < 1e-3);
        positions.push((car_pose.x.to_bits(), car_pose.y.to_bits()));
    }
    // Verify all 8 car positions are unique
    positions.sort();
    positions.dedup();
    assert_eq!(positions.len(), 8, "All 8 cars must spawn in distinct grid positions");
}

#[test]
fn test_race_session_camera_zoom_in_and_out() {
    let mut session = RaceSession::new();
    session.init_race();
    assert_eq!(session.camera.current_level_idx, 0);
    assert_eq!(session.camera.current_zoom_level().name, "Close");

    // Continuous progressive zoom in/out
    let init_max = session.camera.max_zoom_scale;
    session.camera.zoom_progressive(1.0, 1.0, 0.2);
    assert!(session.camera.max_zoom_scale > init_max);
    session.camera.zoom_progressive(-1.0, 1.0, 0.2);
    assert!((session.camera.max_zoom_scale - init_max).abs() < 0.1);

    // Zoom level cycling with Tab
    let lvl1 = session.camera.cycle_zoom_level();
    assert_eq!(lvl1.name, "Medium");
    assert_eq!(session.camera.current_level_idx, 1);

    let lvl2 = session.camera.cycle_zoom_level();
    assert_eq!(lvl2.name, "Far");
    assert_eq!(session.camera.current_level_idx, 2);

    let lvl3 = session.camera.cycle_zoom_level();
    assert_eq!(lvl3.name, "Very Far");
    assert_eq!(session.camera.current_level_idx, 3);

    let lvl0 = session.camera.cycle_zoom_level();
    assert_eq!(lvl0.name, "Close");
    assert_eq!(session.camera.current_level_idx, 0);
}

#[test]
fn test_race_session_pause_shows_circuit_overview_and_resumes_to_follow() {
    use tdrace_app::camera::CameraMode;
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Racing;

    // Set driving zoom to Far (idx 2)
    session.camera.set_zoom_level(2);
    assert_eq!(session.camera.mode, CameraMode::SmoothFollow);
    assert_eq!(session.camera.current_zoom_level().name, "Far");

    // Pause race
    session.pause_race();
    assert_eq!(session.state, GameState::Paused);
    assert_eq!(session.camera.mode, CameraMode::StaticOverview);
    assert_eq!(session.camera.current_pos, session.camera.overview_center);
    assert_eq!(session.camera.current_zoom, session.camera.overview_zoom);

    // Calling update() while paused preserves overview
    session.update();
    assert_eq!(session.camera.mode, CameraMode::StaticOverview);

    // Resume race
    session.resume_race();
    assert_eq!(session.state, GameState::Racing);
    assert_eq!(session.camera.mode, CameraMode::SmoothFollow);
    assert_eq!(session.camera.current_level_idx, 2);
    assert_eq!(session.camera.current_zoom_level().name, "Far");
}

#[test]
fn test_personal_best_notification_lifecycle() {
    let mut session = RaceSession::new();
    session.init_race();
    let track_id = session.track_choice_id().to_string();

    // Ensure clean baseline for track
    session.active_profile_stats.best_times.clear();
    session.pb_notification = None;

    // 1. Lap 1: Initial record establishes first personal best
    session.trackers[0].last_lap_time = Some(24.5);
    session.trackers[0].current_lap = 2;
    session.physics_step(1.0 / 60.0);

    let pb1 = session.pb_notification.as_ref().expect("Expected PB notification on initial lap");
    assert_eq!(pb1.completed_lap, 1);
    assert_eq!(pb1.lap_time, 24.5);
    assert_eq!(pb1.delta, None);
    assert!(pb1.timer > 3.4);
    assert_eq!(session.active_profile_stats.best_times.get(&track_id), Some(&24.5));
    assert!(session.fx.drift_popups.active_popups().iter().any(|p| p.text.contains("PERSONAL BEST! 00:24.50")));

    // 2. Lap 2: Slower lap does NOT trigger a new PB notification
    let timer_before = session.pb_notification.as_ref().unwrap().timer;
    session.trackers[0].last_lap_time = Some(25.2);
    session.trackers[0].current_lap = 3;
    session.physics_step(1.0 / 60.0);

    let pb_after_slower = session.pb_notification.as_ref().expect("PB notification should persist until expired");
    assert_eq!(pb_after_slower.completed_lap, 1); // Remains lap 1
    assert_eq!(pb_after_slower.lap_time, 24.5);
    assert!(pb_after_slower.timer < timer_before); // Timer ticked down
    assert_eq!(session.active_profile_stats.best_times.get(&track_id), Some(&24.5));

    // 3. Lap 3: Faster lap (23.8s) triggers new PB notification with delta (-0.70s)
    session.trackers[0].last_lap_time = Some(23.8);
    session.trackers[0].current_lap = 4;
    session.physics_step(1.0 / 60.0);

    let pb3 = session.pb_notification.as_ref().expect("Expected PB notification on faster lap");
    assert_eq!(pb3.completed_lap, 3);
    assert_eq!(pb3.lap_time, 23.8);
    let delta = pb3.delta.expect("Expected positive improvement delta");
    assert!((delta - 0.70).abs() < 1e-4);
    assert_eq!(session.active_profile_stats.best_times.get(&track_id), Some(&23.8));
    assert!(session.fx.drift_popups.active_popups().iter().any(|p| p.text.contains("(-0.70s)")));

    // 4. Notification timer expiration
    session.physics_step(3.6);
    assert!(session.pb_notification.is_none(), "Notification must expire after duration");
}

#[test]
fn test_personal_best_notification_struct_and_delta() {
    use tdrace_app::ui::hud::PersonalBestNotification;

    let notif_with_delta = PersonalBestNotification {
        completed_lap: 2,
        lap_time: 21.35,
        delta: Some(0.45),
        timer: 3.5,
        duration: 3.5,
    };

    assert_eq!(notif_with_delta.completed_lap, 2);
    assert_eq!(notif_with_delta.lap_time, 21.35);
    assert_eq!(notif_with_delta.delta, Some(0.45));
    assert_eq!(notif_with_delta.timer, 3.5);

    let notif_initial = PersonalBestNotification {
        completed_lap: 1,
        lap_time: 22.00,
        delta: None,
        timer: 3.5,
        duration: 3.5,
    };

    assert_eq!(notif_initial.completed_lap, 1);
    assert_eq!(notif_initial.delta, None);
}

#[test]
fn test_race_session_input_map_integration_and_rebinding() {
    use cabinet::input::{ArcadeAction, ArcadeKey, InputMap, InputSource};

    let mut session = RaceSession::new();
    session.init_race();

    // Verify initial input map is default racing
    assert_eq!(session.input.input_map, InputMap::default_racing());

    // Switch to WASD layout
    session.input.cycle_control_preset();
    assert_eq!(session.input.input_map, InputMap::wasd_racing());
    assert_eq!(session.input.input_map.primary_binding_label(ArcadeAction::Up), "W");

    // Custom rebinding
    session.input.input_map.set_bindings(
        ArcadeAction::Up,
        vec![InputSource::Key(ArcadeKey::Num1)],
    );
    assert_eq!(session.input.input_map.primary_binding_label(ArcadeAction::Up), "1");

    // Serialization and reload roundtrip
    let json = session.input.input_map.to_json().expect("Serialize input map");
    let loaded = InputMap::from_json(&json).expect("Deserialize input map");
    assert_eq!(session.input.input_map, loaded);
}

#[test]
fn test_race_session_screen_transition_phase_stepping_and_state_swap() {
    use cabinet::fx::transition::{TransitionPhase, TransitionType};

    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert!(!session.is_transitioning());
    assert!(session.transition.is_none());

    // Start iris transition towards Countdown
    session.transition_iris_to(GameState::Countdown(3.5), 0.40);
    assert!(session.is_transitioning());
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert_eq!(session.pending_state, Some(GameState::Countdown(3.5)));

    let trans = session.transition.as_ref().unwrap();
    assert_eq!(trans.config.kind, TransitionType::IrisWipe);
    assert_eq!(trans.phase, TransitionPhase::Covering);

    // Step halfway through covering (0.40 * 0.48 = 0.192s cover duration)
    let swapped = session.update_transition(0.08);
    assert!(!swapped, "State should not swap during covering phase");
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert_eq!(
        session.transition.as_ref().unwrap().phase,
        TransitionPhase::Covering
    );

    // Step enough to cross into Holding phase
    let swapped = session.update_transition(0.15);
    assert!(swapped, "State must swap exactly when transition enters Holding");
    assert_eq!(session.state, GameState::Countdown(3.5));
    assert!(session.pending_state.is_none());
    assert!(session.is_transitioning());

    // Step through uncovering to completion
    let swapped = session.update_transition(0.30);
    assert!(!swapped);
    assert!(!session.is_transitioning());
    assert!(session.transition.is_none());
    assert_eq!(session.state, GameState::Countdown(3.5));
}

#[test]
fn test_screen_transition_presets_and_config() {
    use cabinet::fx::transition::TransitionType;

    let mut session = RaceSession::new();

    // Fade preset
    session.transition_fade_to(GameState::Menu, 0.30);
    assert!(session.is_transitioning());
    assert_eq!(
        session.transition.as_ref().unwrap().config.kind,
        TransitionType::Fade
    );
    session.update_transition(0.50);
    assert!(!session.is_transitioning());
    assert_eq!(session.state, GameState::Menu);

    // Curtain preset
    session.transition_curtain_to(GameState::StartingGrid, 0.40);
    assert!(session.is_transitioning());
    assert_eq!(
        session.transition.as_ref().unwrap().config.kind,
        TransitionType::CurtainWipe
    );
    session.update_transition(0.50);
    assert!(!session.is_transitioning());
    assert_eq!(session.state, GameState::StartingGrid);

    // Scanline preset
    session.transition_scanline_to(GameState::Finished, 0.35);
    assert!(session.is_transitioning());
    assert_eq!(
        session.transition.as_ref().unwrap().config.kind,
        TransitionType::ScanlineWipe
    );
    session.update_transition(0.50);
    assert!(!session.is_transitioning());
    assert_eq!(session.state, GameState::Finished);
}

#[test]
fn test_screen_transition_module_select_switch_to_menu() {
    let mut session = RaceSession::new();
    session.state = GameState::ModuleSelect { selected_idx: 1 }; // Rally module selected

    session.transition_scanline_to(GameState::Menu, 0.35);
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 1 });

    // Advance to Holding: module switch should be automatically applied
    session.update_transition(0.20);
    assert_eq!(session.state, GameState::Menu);
    assert_eq!(session.active_module_id, "rally");

    // Complete transition
    session.update_transition(0.25);
    assert!(!session.is_transitioning());
}

#[test]
fn test_race_session_crt_overlay_settings_and_toggle() {
    use cabinet::fx::crt::ScanlineMode;

    let mut session = RaceSession::new();
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::Disabled);
    assert!(!session.crt_overlay.is_active());

    // Cycle modes
    let m1 = session.cycle_scanline_mode();
    assert_eq!(m1, ScanlineMode::Subtle);
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::Subtle);
    assert_eq!(session.config.display.scanline_mode, "subtle");
    assert!(session.crt_overlay.is_active());

    let m2 = session.cycle_scanline_mode();
    assert_eq!(m2, ScanlineMode::ArcadeCrt);
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::ArcadeCrt);
    assert_eq!(session.config.display.scanline_mode, "arcade_crt");
    assert!(session.crt_overlay.is_active());

    let m3 = session.cycle_scanline_mode();
    assert_eq!(m3, ScanlineMode::RetroGlow);
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::RetroGlow);
    assert_eq!(session.config.display.scanline_mode, "retro_glow");
    assert!(session.crt_overlay.is_active());

    let m4 = session.cycle_scanline_mode();
    assert_eq!(m4, ScanlineMode::Disabled);
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::Disabled);
    assert_eq!(session.config.display.scanline_mode, "disabled");

    // Open settings modal and verify scanlines dropdown synchronization
    session.set_scanline_mode(ScanlineMode::ArcadeCrt);
    session.open_settings_modal();
    assert!(session.is_settings_modal_open());

    let modal = session.settings_modal.as_ref().unwrap();
    assert_eq!(
        modal.scanlines_dropdown.selected_index,
        ScanlineMode::ArcadeCrt.to_index()
    );

    // Modify dropdown to RetroGlow and save
    session
        .settings_modal
        .as_mut()
        .unwrap()
        .scanlines_dropdown
        .set_selected(ScanlineMode::RetroGlow.to_index());
    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());
    assert_eq!(session.crt_overlay.config.mode, ScanlineMode::RetroGlow);
    assert_eq!(session.config.display.scanline_mode, "retro_glow");

    // Modify dropdown and discard (close without save)
    session.open_settings_modal();
    session
        .settings_modal
        .as_mut()
        .unwrap()
        .scanlines_dropdown
        .set_selected(ScanlineMode::Disabled.to_index());
    session.close_settings_modal(false);
    assert_eq!(
        session.crt_overlay.config.mode,
        ScanlineMode::RetroGlow,
        "Mode should remain RetroGlow when discarded"
    );

    // Stepping overlay animation
    session.crt_overlay.config.roll_speed = 50.0;
    session.crt_overlay.config.roll_bar_opacity = 0.1;
    let initial_offset = session.crt_overlay.roll_offset;
    session.crt_overlay.update(0.1);
    assert!(session.crt_overlay.roll_offset > initial_offset);
}


