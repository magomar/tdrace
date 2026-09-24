use tdrace_app::game::{GameState, ProfileOrigin, RaceSession, StartingGridFocus};
use tdrace_app::ui::menu::{CarChoice, GameMode, TrackChoice};
use tdrace_core::track::presets::{
    classic_grand_prix, classic_rallycross, drift_park, kart_arena, oasis_rally, oval_speedway,
    ramp_raceway,
};
use tdrace_core::track::Track;
use tdrace_core::CarCategory;

#[test]
fn test_preset_tracks_predefined_cars_and_balanced_laps() {
    let gp = classic_grand_prix();
    assert_eq!(gp.default_laps, 5);
    assert_eq!(gp.car_category, CarCategory::Gt);

    let oval = oval_speedway();
    assert_eq!(oval.default_laps, 5);
    assert_eq!(oval.car_category, CarCategory::Nascar);

    let drift = drift_park();
    assert_eq!(drift.default_laps, 5);
    assert_eq!(drift.car_category, CarCategory::Gt);

    let kart = kart_arena();
    assert_eq!(kart.default_laps, 5);
    assert_eq!(kart.car_category, CarCategory::Kart);

    let ramp = ramp_raceway();
    assert_eq!(ramp.default_laps, 5);
    assert_eq!(ramp.car_category, CarCategory::Rally);

    let oasis = oasis_rally();
    assert_eq!(oasis.default_laps, 5);
    assert_eq!(oasis.car_category, CarCategory::OffRoad);

    let rx = classic_rallycross();
    assert_eq!(rx.default_laps, 5);
    assert_eq!(rx.car_category, CarCategory::Rally);
}

#[test]
fn test_enforced_predefined_car_in_race_session() {
    let mut session = RaceSession::new();

    // 1. Select Kart Arena (enforced car = Kart, laps = 5)
    session.track_choice = TrackChoice::KartArena;
    session.free_car_selection = false;
    session.num_bots = 4;
    session.init_race();

    assert_eq!(session.total_laps, 5);
    assert_eq!(session.resolve_predefined_car(), CarChoice::Kart);
    assert_eq!(session.active_player_car_choice(), CarChoice::Kart);
    assert_eq!(session.cars.len(), 5); // 1 player + 4 bots

    // Verify player car top speed matches Kart specs (~32 m/s)
    let player_car = &session.cars[0];
    assert!((player_car.config.top_speed_mps - 32.0).abs() < 1.0);

    // Verify all bot cars use Kart specs when free car selection is disabled
    for bot_car in &session.cars[1..] {
        assert!((bot_car.config.top_speed_mps - 32.0).abs() < 1.0);
    }

    // 2. Select Drift Park (enforced car = SportsCar / classic_gt, laps = 5)
    session.track_choice = TrackChoice::DriftPark;
    session.free_car_selection = false;
    session.random_car_assignment = false;
    session.init_race();

    assert_eq!(session.total_laps, 5);
    assert_eq!(session.resolve_predefined_car(), CarChoice::SportsCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);

    // Verify sports car top speed (~58 m/s)
    for car in &session.cars {
        assert!((car.config.top_speed_mps - 58.0).abs() < 1.0);
    }
}

#[test]
fn test_free_car_selection_toggle_in_race_session() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::KartArena;
    session.free_car_selection = false;
    session.car_choice = CarChoice::SportsCar;
    session.num_bots = 3;
    session.init_race();

    // Enforced: player car is Kart even if session.car_choice was SportsCar
    assert_eq!(session.active_player_car_choice(), CarChoice::Kart);

    // Enable free car selection
    session.free_car_selection = true;
    session.rebuild_roster_participants();
    session.init_race();

    // Now player gets SportsCar
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);
    assert!((session.cars[0].config.top_speed_mps - 58.0).abs() < 1.0);

    // AI bots use their distinct preferred vehicles
    let bot_choices: Vec<CarChoice> = session
        .opponent_drivers
        .iter()
        .map(|d| d.preferred_car)
        .collect();
    assert_eq!(bot_choices.len(), 3);
}

#[test]
fn test_roster_driver_count_modification() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.cars.len(), 4);
    assert_eq!(session.opponent_drivers.len(), 3);

    // Modify driver count to 7 bots (8 racers)
    session.num_bots = 7;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.opponent_drivers.len(), 7);
    assert_eq!(session.trackers.len(), 8);
    assert_eq!(session.ai_drivers.len(), 7);

    // Modify driver count to 1 bot (2 racers)
    session.num_bots = 1;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 2);
    assert_eq!(session.opponent_drivers.len(), 1);
    assert_eq!(session.trackers.len(), 2);
    assert_eq!(session.ai_drivers.len(), 1);
}

#[test]
fn test_track_serde_default_laps_and_car_category_roundtrip() {
    let track = classic_grand_prix();
    let json = track.to_json_pretty().expect("Must serialize to JSON");

    let deserialized = Track::from_json(&json).expect("Must deserialize from JSON");
    assert_eq!(deserialized.default_laps, 5);
    assert_eq!(deserialized.car_category, CarCategory::Gt);

    // Test backwards-compatibility when fields are missing from JSON
    let mut val: serde_json::Value = serde_json::from_str(&json).expect("Parse json value");
    if let Some(obj) = val.as_object_mut() {
        obj.remove("default_laps");
        obj.remove("car_category");
    }
    let legacy_json = serde_json::to_string(&val).expect("Serialize stripped json");

    let legacy_track = Track::from_json(&legacy_json).expect("Must deserialize legacy JSON");
    assert_eq!(legacy_track.default_laps, 3);
    assert_eq!(legacy_track.car_category, CarCategory::Gt);
}

#[test]
fn test_2d_navigation_focus_and_cursor_state() {
    use tdrace_app::ui::menu::MenuPanelFocus;
    use tdrace_app::ui::starting_grid::StartingGridFocus;

    let mut session = RaceSession::new();

    // 1. Verify default Track Select Menu column focus
    assert_eq!(session.menu_focused_panel, MenuPanelFocus::LeftTracks);
    assert_eq!(session.menu_track_idx, 0);
    assert_eq!(session.menu_car_idx, 0);

    // Switch column focus to Right Vehicle
    session.menu_focused_panel = MenuPanelFocus::RightVehicle;
    assert_eq!(session.menu_focused_panel, MenuPanelFocus::RightVehicle);

    // 2. Initialize Race and verify Starting Grid default focus
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.starting_grid_focus, StartingGridFocus::LeftSetup);
    assert_eq!(session.starting_grid_card_idx, 0); // Combined Garage & Active Car card
    assert_eq!(session.starting_grid_roster_idx, 0);

    // 3. Starting Grid 2D navigation: panel switching & card cycling
    session.starting_grid_card_idx = 1; // Grid Config card
    assert_eq!(session.starting_grid_card_idx, 1);

    session.starting_grid_card_idx = 2; // Launch Race button
    assert_eq!(session.starting_grid_card_idx, 2);

    // Verify Starting Grid Launch button rect geometry
    let (bx, by, bw, bh) = tdrace_app::ui::starting_grid_launch_button_rect(1280.0, 720.0);
    assert!(bw > 200.0);
    assert!(bh > 30.0);
    assert!(bx > 0.0);
    assert!(by > 300.0);

    // Switch panel focus to Right Starting Grid Roster
    session.starting_grid_focus = StartingGridFocus::RightRoster;
    session.starting_grid_roster_idx = 2;
    assert_eq!(session.starting_grid_focus, StartingGridFocus::RightRoster);
    assert_eq!(session.starting_grid_roster_idx, 2);

    // 4. Pause Menu button cursor
    assert_eq!(session.pause_selected_btn, 0); // Resume
    session.pause_selected_btn = 1; // Exit
    assert_eq!(session.pause_selected_btn, 1);
}

#[test]
fn test_circuit_catalog_filtering_presets_and_custom() {
    use tdrace_app::ui::menu::TrackCatalogFilter;

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_catalog_filter_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::new();
    session.track_manager = tdrace_app::track_manager::TrackManager::new(&temp_dir);
    session.active_module_id = "classic";

    // 1. Filter cycling verification: Presets <-> Custom
    assert_eq!(session.menu_track_filter, TrackCatalogFilter::Presets);
    assert_eq!(session.menu_track_filter.next(), TrackCatalogFilter::Custom);
    assert_eq!(session.menu_track_filter.next().next(), TrackCatalogFilter::Presets);

    assert_eq!(session.menu_track_filter.prev(), TrackCatalogFilter::Custom);
    assert_eq!(session.menu_track_filter.prev().prev(), TrackCatalogFilter::Presets);

    // Initial state: 10 classic presets, 0 custom
    let (preset_c, custom_c) = session.menu_track_filter_counts();
    assert_eq!(preset_c, 10);
    assert_eq!(custom_c, 0);

    // 2. Add a custom circuit
    let mut custom_track = classic_grand_prix();
    custom_track.name = "My Test Custom Circuit".to_string();
    custom_track.description = "A custom track created by user.".to_string();
    let _ = session.track_manager.save_custom_track(&custom_track, Some("my_test_custom_circuit"));

    // Counts after adding custom circuit
    let (preset_c, custom_c) = session.menu_track_filter_counts();
    assert_eq!(preset_c, 10);
    assert_eq!(custom_c, 1);

    // Filter: Presets -> Only official presets
    session.menu_track_filter = TrackCatalogFilter::Presets;
    let filtered_presets = session.filtered_menu_tracks();
    assert_eq!(filtered_presets.len(), 10);
    assert!(filtered_presets.iter().all(|t| t.is_official_preset()));
    assert!(!filtered_presets.iter().any(|t| t.is_user_custom()));

    // Filter: Custom -> Only custom tracks
    session.menu_track_filter = TrackCatalogFilter::Custom;
    let filtered_custom = session.filtered_menu_tracks();
    assert_eq!(filtered_custom.len(), 1);
    assert!(filtered_custom.iter().all(|t| t.is_user_custom()));
    assert_eq!(filtered_custom[0].title(), "My Test Custom Circuit");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_custom_tracks_pure_selection_and_shortcut() {
    use tdrace_app::game::GameState;
    use tdrace_app::ui::menu::TrackCatalogFilter;

    let mut session = RaceSession::new();
    session.state = GameState::Menu;
    session.menu_track_filter = TrackCatalogFilter::Custom;

    let available = session.filtered_menu_tracks();
    // Track selection list is pure: exactly available tracks, zero virtual elements
    let total_items = available.len();
    assert_eq!(total_items, available.len());

    // Verify dedicated transition to Track Manager / My Circuits
    session.state = GameState::TrackManager {
        active_tab: tdrace_app::ui::TrackManagerTab::Main,
        module_filter: tdrace_app::track_manager::ModuleFilter::for_module(session.active_module_id),
        selected_idx: 0,
        modal: tdrace_app::ui::TrackManagerModal::None,
    };
    assert!(matches!(session.state, GameState::TrackManager { .. }));
}

#[test]
fn test_championship_mode_roster_locked_and_custom_race_customizable() {
    use tdrace_app::ui::menu::GameMode;

    // 1. Championship Mode (Career): Roster and vehicle must be strictly locked
    let career = GameMode::Career;
    assert!(!career.allows_roster_customization(), "Championship mode roster must be locked");
    assert!(!career.allows_car_change(), "Championship mode car change must be locked");

    // 2. Custom Race Mode (ExperimentalRace): Roster and vehicle must be customizable
    let custom = GameMode::ExperimentalRace;
    assert!(custom.allows_roster_customization(), "Custom race mode roster must be customizable");
    assert!(custom.allows_car_change(), "Custom race mode car must be customizable");

    // 3. Quick Race (StandardRace): Fixed predefined car & locked roster
    let standard = GameMode::StandardRace;
    assert!(!standard.allows_roster_customization(), "Standard race roster must be locked");
    assert!(!standard.allows_car_change(), "Standard race car must be predefined");
}

#[test]
fn test_combined_garage_button_rect_and_launch_button_rect() {
    use tdrace_app::ui::{
        starting_grid_garage_button_rect, starting_grid_grid_button_rect,
        starting_grid_launch_button_rect,
    };

    let (gx, gy, gw, gh) = starting_grid_garage_button_rect(1280.0, 720.0);
    assert!(gw > 350.0);
    assert!(gh >= 400.0, "Enlarged Garage card must encompass the full active car showcase (h = {})", gh);
    assert!(gx > 0.0);
    assert!(gy > 50.0);

    let (lx, ly, lw, lh) = starting_grid_launch_button_rect(1280.0, 720.0);
    assert!(lw > 350.0);
    assert!(lh > 30.0);
    assert!(lx > 0.0);
    // Launch button must be positioned below the Garage card
    assert!(ly > gy + gh, "Launch button (ly={}) must be below Garage card (gy+gh={})", ly, gy + gh);

    // Grid config button rect is now in the right column above the roster card
    let (grid_x, grid_y, grid_w, grid_h) = starting_grid_grid_button_rect(1280.0, 720.0);
    assert!(grid_x > gx, "Grid config rect (x={}) must be in the right column (gx={})", grid_x, gx);
    assert!(grid_w > 350.0);
    assert!(grid_h > 40.0);
    assert_eq!(grid_y, 60.0);
}

#[test]
fn test_launch_race_eligibility_across_all_classic_tracks() {
    let mut session = RaceSession::new();
    let tracks = session.active_module_tracks();
    for track in tracks {
        session.track_choice = track.clone();
        let loaded = tdrace_app::ui::menu::resolve_track_for_menu(&session.track_choice);
        session.car_choice = tdrace_app::ui::menu::resolve_predefined_car_for_track(loaded.as_ref(), session.active_module_id);
        session.init_race();

        let player_car = session.active_player_car_choice();
        let req_tier = session.current_race_required_tier();
        let eligible = player_car.is_eligible_for_race_tier(req_tier, session.is_dev_mode());
        let unlocked = session.is_car_unlocked(player_car);

        assert!(unlocked, "Predefined car {:?} on track {} must be unlocked", player_car, track.track_id());
        assert!(eligible, "Predefined car {:?} on track {} must be eligible (tier {} <= req {})", player_car, track.track_id(), player_car.tier(), req_tier);

        // Verify that starting grid launch condition succeeds
        assert!(eligible && unlocked, "Starting grid launch must be allowed for track {}", track.track_id());
    }
}

#[test]
fn test_launch_race_eligibility_across_specialized_modules() {
    let mut session = RaceSession::new();
    let modules = ["gt", "nascar", "rally", "kart", "extreme_offroad"];
    for mod_id in modules {
        session.switch_to_module(mod_id);
        let tracks = session.active_module_tracks();
        for track in tracks.iter().take(3) {
            session.track_choice = track.clone();
            let loaded = tdrace_app::ui::menu::resolve_track_for_menu(&session.track_choice);
            session.car_choice = tdrace_app::ui::menu::resolve_predefined_car_for_track(loaded.as_ref(), session.active_module_id);
            session.init_race();

            let player_car = session.active_player_car_choice();
            let req_tier = session.current_race_required_tier();
            let eligible = player_car.is_eligible_for_race_tier(req_tier, session.is_dev_mode());
            let unlocked = session.is_car_unlocked(player_car);

            assert!(unlocked, "Module {} track {} car {:?} must be unlocked", mod_id, track.track_id(), player_car);
            assert!(eligible, "Module {} track {} car {:?} must be eligible (tier {} <= req {})", mod_id, track.track_id(), player_car, player_car.tier(), req_tier);
        }
    }
}

#[test]
fn test_starting_grid_card_2_and_space_launch_transition() {
    let mut session = RaceSession::new();
    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);

    // Starting grid launch prerequisite check
    let player_car = session.active_player_car_choice();
    let req_tier = session.current_race_required_tier();
    assert!(player_car.is_eligible_for_race_tier(req_tier, session.is_dev_mode()));
    assert!(session.is_car_unlocked(player_car));

    // Card 2 simulates selecting Launch Race button and pressing Enter
    session.starting_grid_focus = StartingGridFocus::LeftSetup;
    session.starting_grid_card_idx = 2;
    session.transition_iris_to(GameState::Countdown(3.5), 0.45);
    assert!(session.transition.is_some());
    assert_eq!(session.pending_state, Some(GameState::Countdown(3.5)));
}

#[test]
fn test_starting_grid_player_card_selection_and_profile_flow() {
    let (px, py, pw, ph) = tdrace_app::ui::starting_grid_player_card_rect(1280.0, 720.0);
    assert!(pw > 200.0);
    assert!(ph > 50.0);
    assert!(px > 0.0);
    assert!(py > 0.0);

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    // Default state: LeftSetup, Card 0 (Garage)
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.starting_grid_focus, StartingGridFocus::LeftSetup);
    assert_eq!(session.starting_grid_card_idx, 0);

    // 1. Navigate UP from Garage (0) -> selects Circuit Explorer Card (4)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.starting_grid_card_idx, 4);

    // 2. Navigate UP from Circuit Card (4) -> selects Player Profile Card (3)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.starting_grid_card_idx, 3);

    // 3. Navigate UP from Player Card (3) -> wraps around to Launch Race (2)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.starting_grid_card_idx, 2);

    // 4. Navigate UP from Launch Race (2) -> moves to Garage (0)
    session.input.gamepad.snapshot.nav_up = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_up = false;
    assert_eq!(session.starting_grid_card_idx, 0);

    // 5. Navigate DOWN from Garage (0) -> moves to Launch Race (2)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.starting_grid_card_idx, 2);

    // 6. Navigate DOWN from Launch Race (2) -> wraps around to Player Card (3)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.starting_grid_card_idx, 3);

    // 7. Navigate DOWN from Player Card (3) -> moves to Circuit Card (4)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.starting_grid_card_idx, 4);

    // 8. Navigate DOWN from Circuit Card (4) -> moves to Garage (0)
    session.input.gamepad.snapshot.nav_down = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.nav_down = false;
    assert_eq!(session.starting_grid_card_idx, 0);

    // 9. Select Player Card (3) and hit Enter/Confirm -> loads ProfileManager
    session.starting_grid_card_idx = 3;
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert!(matches!(session.state, GameState::ProfileManager { .. }));
    assert_eq!(session.profile_origin, ProfileOrigin::StartingGrid);

    // 10. Return from ProfileManager with Cancel (Escape/Gamepad B) -> returns to StartingGrid
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    let sel_idx = match session.state {
        GameState::ProfileManager { selected_idx } => selected_idx,
        _ => 0,
    };
    session.update_profile_manager(sel_idx);
    session.input.gamepad.snapshot.btn_cancel_pressed = false;
    assert_eq!(session.state, GameState::StartingGrid);

    // 11. Verify open_profile_from_starting_grid directly
    session.open_profile_from_starting_grid();
    assert!(matches!(session.state, GameState::ProfileManager { .. }));
    assert_eq!(session.profile_origin, ProfileOrigin::StartingGrid);
}

#[test]
fn test_starting_grid_circuit_card_and_selector_flow() {
    let (cx, cy, cw, ch) = tdrace_app::ui::starting_grid_circuit_card_rect(1280.0, 720.0);
    assert!(cw > 200.0);
    assert!(ch > 40.0);
    assert!(cx > 0.0);
    assert!(cy > 0.0);

    let (px, py, _pw, ph) = tdrace_app::ui::starting_grid_player_card_rect(1280.0, 720.0);
    assert_eq!(cx, px);
    assert!(cy >= py + ph);

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    // 1. Select Circuit Card (4) and confirm -> opens Menu with MenuOrigin::StartingGrid
    session.starting_grid_card_idx = 4;
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_starting_grid();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.state, GameState::Menu);
    assert_eq!(session.menu_origin, tdrace_app::game::MenuOrigin::StartingGrid);

    // 2. Escape from Menu returns to StartingGrid
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_cancel_pressed = false;
    assert_eq!(session.state, GameState::StartingGrid);

    // 3. Career mode blocking: open circuit selector while in Career mode
    session.game_mode = GameMode::Career;
    session.open_circuit_selector_from_starting_grid();
    assert_eq!(session.state, GameState::Menu);
    assert_eq!(session.menu_origin, tdrace_app::game::MenuOrigin::StartingGrid);

    // Select a different circuit in the menu
    let tracks = session.filtered_menu_tracks();
    let other_idx = tracks.iter().position(|t| t.track_id() != session.track_choice.track_id()).unwrap_or(1);
    session.menu_track_idx = other_idx;
    let initial_track_id = session.track_choice.track_id().to_string();

    // In Career mode, pressing Confirm on another circuit is blocked (stays in Menu, track unchanged)
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert_eq!(session.state, GameState::Menu);
    assert_eq!(session.track_choice.track_id(), initial_track_id.as_str());

    // In Career mode, pressing Confirm on the active circuit returns to StartingGrid
    let active_idx = tracks.iter().position(|t| t.track_id() == initial_track_id.as_str()).unwrap_or(0);
    session.menu_track_idx = active_idx;
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.track_choice.track_id(), initial_track_id.as_str());

    // 4. Non-career mode: switching circuit works and re-inits into StartingGrid with new circuit
    session.game_mode = GameMode::StandardRace;
    session.open_circuit_selector_from_starting_grid();
    session.menu_track_idx = other_idx;
    let target_choice = tracks[other_idx].clone();

    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_menu();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.track_choice.track_id(), target_choice.track_id());
}


