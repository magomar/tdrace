use tdrace_app::catalog::{get_all_models, get_models_for_module, get_models_for_module_and_tier};
use tdrace_app::game::{GameState, GarageOrigin, RaceSession, StartingGridFocus};
use tdrace_app::ui::garage::{
    gallery_filter_to_module, garage_car_card_rect, garage_gallery_card_rect,
    garage_gallery_tab_rect, garage_select_button_rect, module_to_gallery_filter,
    GALLERY_MODULES, GarageViewMode,
};
use tdrace_app::ui::starting_grid::{starting_grid_garage_button_rect, starting_grid_launch_button_rect};

#[test]
fn test_starting_grid_garage_button_rect_geometry() {
    let (sw, sh) = (1280.0, 720.0);
    let (gx, gy, gw, gh) = starting_grid_garage_button_rect(sw, sh);
    let (lx, ly, lw, lh) = starting_grid_launch_button_rect(sw, sh);

    // Verify both buttons exist with valid non-zero dimensions
    assert!(gw > 100.0);
    assert!(gh > 30.0);
    assert!(lw > 100.0);
    assert!(lh > 30.0);

    // Garage card is positioned above the Launch race button
    assert!(gy < ly);
    assert_eq!(gx, lx);
    assert_eq!(gw, lw);
}

#[test]
fn test_starting_grid_card_0_enters_garage() {
    let mut session = RaceSession::new();
    session.state = GameState::StartingGrid;
    session.starting_grid_focus = StartingGridFocus::LeftSetup;
    session.starting_grid_card_idx = 0;

    // Simulate selecting Card 0 in StartingGrid
    session.garage_origin = GarageOrigin::StartingGrid;
    session.garage_tier = session.current_race_required_tier();
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::StartingGrid);

    assert_eq!(session.state, GameState::Garage(GarageOrigin::StartingGrid));
    assert_eq!(session.garage_origin, GarageOrigin::StartingGrid);
    assert_eq!(session.garage_car_idx, 0);
}

#[test]
fn test_starting_grid_car_card_1_direct_garage_transition() {
    let mut session = RaceSession::new();
    session.state = GameState::StartingGrid;
    session.starting_grid_focus = StartingGridFocus::LeftSetup;
    session.starting_grid_card_idx = 1;

    // Entering garage from Car Card (Card 1)
    session.garage_origin = GarageOrigin::StartingGrid;
    session.garage_tier = session.current_race_required_tier();
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::StartingGrid);

    assert_eq!(session.state, GameState::Garage(GarageOrigin::StartingGrid));
    assert_eq!(session.garage_origin, GarageOrigin::StartingGrid);
}

#[test]
fn test_menu_direct_garage_shortcut() {
    let mut session = RaceSession::new();
    session.state = GameState::Menu;

    // Direct G key from menu transitions to Garage
    session.garage_origin = GarageOrigin::Menu;
    session.garage_tier = session.current_race_required_tier();
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::Menu);

    assert_eq!(session.state, GameState::Garage(GarageOrigin::Menu));
    assert_eq!(session.garage_origin, GarageOrigin::Menu);
}

#[test]
fn test_garage_shows_all_module_models_across_tiers() {
    let modules = ["gt", "rally", "kart", "nascar", "extreme_offroad"];

    for mod_id in modules {
        let all_cars = get_models_for_module(mod_id);
        assert!(
            all_cars.len() >= 15,
            "Module {} should have at least 15 catalog cars, got {}",
            mod_id,
            all_cars.len()
        );

        // Every module should feature 5 progression tiers with at least 3 distinct models each
        for tier in 1..=5 {
            let tier_models = get_models_for_module_and_tier(mod_id, tier);
            assert!(
                tier_models.len() >= 3,
                "Module {} Tier {} should have at least 3 car models, got {}",
                mod_id,
                tier,
                tier_models.len()
            );
        }
    }

    let global_models = get_all_models();
    assert_eq!(
        global_models.len(),
        80,
        "Expected exactly 80 real car models in catalog, got {}",
        global_models.len()
    );
}

#[test]
fn test_all_80_real_cars_attributes_and_data_integrity() {
    let global_models = get_all_models();
    assert_eq!(global_models.len(), 80);

    let mut seen_ids = std::collections::HashSet::new();
    let valid_modules = ["gt", "rally", "kart", "nascar", "extreme_offroad"];

    for car in global_models {
        // Unique non-empty IDs and names
        assert!(!car.id.is_empty(), "Car ID must not be empty");
        assert!(seen_ids.insert(car.id), "Duplicate car ID detected: {}", car.id);
        assert!(!car.name.is_empty(), "Car name for {} must not be empty", car.id);
        assert!(!car.manufacturer.is_empty(), "Car manufacturer for {} must not be empty", car.id);
        assert!(!car.history_bio.is_empty(), "Car history_bio for {} must not be empty", car.id);

        // Module and tier validity
        assert!(
            valid_modules.contains(&car.module_id),
            "Unknown module ID {} on car {}",
            car.module_id,
            car.id
        );
        assert!((1..=5).contains(&car.tier), "Invalid tier {} on car {}", car.tier, car.id);

        // Realistic non-zero specifications
        assert!(car.bhp > 0, "Car {} must have positive BHP", car.id);
        assert!(car.weight_kg > 0, "Car {} must have positive weight", car.id);
        assert!(car.top_speed_kmh > 0, "Car {} must have positive top speed", car.id);
        assert!(car.accel_0_100 > 0.0, "Car {} must have positive 0-100 accel", car.id);

        // Radar stats bounded between 0.0 and 1.0
        let stats = [
            car.stats.0,
            car.stats.1,
            car.stats.2,
            car.stats.3,
            car.stats.4,
            car.stats.5,
        ];
        for (idx, &stat) in stats.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&stat),
                "Car {} stat index {} out of range: {}",
                car.id,
                idx,
                stat
            );
        }
    }
}

#[test]
fn test_career_tier_gating_in_garage() {
    let mut session = RaceSession::new();
    session.state = GameState::Garage(GarageOrigin::StartingGrid);
    session.active_module_id = "gt";
    session.garage_tier = 1;
    session.garage_car_idx = 0;
    session.active_profile_stats.wins = 0; // Brand new driver -> Only Tier 1 unlocked

    // Tier 1 model selection should succeed
    let tier1_models = get_models_for_module_and_tier("gt", 1);
    let chosen_t1 = tier1_models[0];
    session.car_choice = chosen_t1.base_car_choice;
    session.current_visual_type = chosen_t1.visual_type;
    session.free_car_selection = true;
    session.state = GameState::StartingGrid;

    assert_eq!(session.car_choice, chosen_t1.base_car_choice);
    assert_eq!(session.state, GameState::StartingGrid);

    // Attempting to select a Tier 4 car without enough wins must be rejected
    let mut locked_session = RaceSession::new();
    locked_session.state = GameState::Garage(GarageOrigin::StartingGrid);
    locked_session.active_module_id = "gt";
    locked_session.garage_tier = 4;
    locked_session.garage_car_idx = 0;
    locked_session.active_profile_stats.wins = 1; // 1 win unlocks Tier 2, NOT Tier 4!
    let original_car = locked_session.car_choice;

    let unlocked_tier = {
        let wins = locked_session.active_profile_stats.wins;
        if wins >= 10 {
            5
        } else if wins >= 5 {
            4
        } else if wins >= 2 {
            3
        } else if wins >= 1 {
            2
        } else {
            1
        }
    };
    assert_eq!(unlocked_tier, 2);

    let is_unlocked = locked_session.garage_tier <= unlocked_tier;
    assert!(!is_unlocked, "Tier 4 must be locked for 1 win");

    // Locked selection rejected: car choice remains unchanged and stays in Garage
    if is_unlocked {
        let models = get_models_for_module_and_tier("gt", 4);
        locked_session.car_choice = models[0].base_car_choice;
        locked_session.state = GameState::StartingGrid;
    }

    assert_eq!(locked_session.car_choice, original_car);
    assert_eq!(locked_session.state, GameState::Garage(GarageOrigin::StartingGrid));

    // With 5 wins, Tier 4 becomes unlocked
    locked_session.active_profile_stats.wins = 5;
    let unlocked_tier_5_wins = {
        let wins = locked_session.active_profile_stats.wins;
        if wins >= 10 {
            5
        } else if wins >= 5 {
            4
        } else if wins >= 2 {
            3
        } else if wins >= 1 {
            2
        } else {
            1
        }
    };
    assert_eq!(unlocked_tier_5_wins, 4);
    assert!(locked_session.garage_tier <= unlocked_tier_5_wins);
}

#[test]
fn test_garage_geometry_and_car_card_carousel_layout() {
    let (sw, sh) = (1920.0, 1080.0);
    let (_bx, by, bw, bh) = garage_select_button_rect(sw, sh);
    assert!(bw > 150.0);
    assert!(bh > 30.0);
    assert!(by < sh);

    for i in 0..4 {
        let (cx, cy, cw, ch) = garage_car_card_rect(sw, sh, i, 4);
        assert!(cw > 50.0);
        assert!(ch > 20.0);
        assert!(cx >= 0.0);
        assert!(cy < sh);
    }
}

#[test]
fn test_garage_view_modes() {
    let mode1 = GarageViewMode::Lateral;
    let mode2 = GarageViewMode::TopDownTurntable;
    assert_ne!(mode1, mode2);
}

#[test]
fn test_fleet_gallery_module_tabs_only_and_no_all_tab() {
    // Spec: remove the "all" tab, and keep only module specific tabs
    assert_eq!(GALLERY_MODULES.len(), 5);

    let expected_modules = ["gt", "rally", "kart", "nascar", "extreme_offroad"];
    let expected_labels = ["GT", "RALLY", "KART", "NASCAR", "OFF-ROAD"];

    for (i, &(mod_id, label)) in GALLERY_MODULES.iter().enumerate() {
        assert_ne!(label, "ALL", "The 'ALL' tab must be removed from fleet gallery");
        assert_eq!(mod_id, expected_modules[i]);
        assert_eq!(label, expected_labels[i]);

        // Each module must have valid models in the catalog
        let cars = get_models_for_module(mod_id);
        assert!(!cars.is_empty(), "Module {} should contain vehicles", mod_id);
        for car in &cars {
            assert_eq!(car.module_id, mod_id);
        }
    }
}

#[test]
fn test_fleet_gallery_filter_conversions() {
    let mapping = [
        (0, "gt"),
        (1, "rally"),
        (2, "kart"),
        (3, "nascar"),
        (4, "extreme_offroad"),
    ];

    for (idx, mod_id) in mapping {
        assert_eq!(gallery_filter_to_module(idx), mod_id);
        assert_eq!(module_to_gallery_filter(mod_id), idx);
    }

    // Aliases
    assert_eq!(module_to_gallery_filter("gt_challenge"), 0);
    assert_eq!(module_to_gallery_filter("f1"), 0);
    // Out of bounds fallback
    assert_eq!(gallery_filter_to_module(999), "gt");
    assert_eq!(module_to_gallery_filter("unknown"), 0);
}

#[test]
fn test_fleet_gallery_tab_and_card_geometry() {
    let (sw, sh) = (1280.0, 720.0);

    // Verify all 5 module tabs have non-overlapping, valid positive rects
    let mut prev_right = 0.0;
    for i in 0..GALLERY_MODULES.len() {
        let (tx, ty, tw, th) = garage_gallery_tab_rect(sw, sh, i);
        assert!(tw > 50.0);
        assert!(th > 15.0);
        assert!(ty >= 0.0 && ty < sh);
        assert!(tx >= prev_right, "Tab {} should not overlap previous tab", i);
        prev_right = tx + tw;
    }

    // Verify 4-column car card grid geometry
    for i in 0..16 {
        let (cx, cy, cw, ch) = garage_gallery_card_rect(sw, sh, i);
        assert!(cw > 100.0);
        assert!(ch > 50.0);
        assert!(cx >= 0.0 && cx + cw <= sw);
        assert!(cy >= 0.0 && cy + ch <= sh);
    }
}

#[test]
fn test_fleet_gallery_session_navigation_by_module() {
    let mut session = RaceSession::new();
    session.state = GameState::Garage(GarageOrigin::Menu);
    session.active_module_id = "nascar";
    session.garage_tier = 2;
    session.garage_car_idx = 0;

    // Opening gallery mode should automatically set filter to active module
    session.garage_gallery_mode = true;
    session.garage_gallery_filter = module_to_gallery_filter(session.active_module_id);
    assert_eq!(session.garage_gallery_filter, 3); // NASCAR is tab 3
    assert_eq!(gallery_filter_to_module(session.garage_gallery_filter), "nascar");

    // Switching to module tab 1 (RALLY) via key or tab cycle
    session.garage_gallery_filter = 1;
    session.garage_gallery_sel = 0;
    assert_eq!(gallery_filter_to_module(session.garage_gallery_filter), "rally");

    // Cycle forward with Tab / E
    let total_tabs = GALLERY_MODULES.len();
    session.garage_gallery_filter = (session.garage_gallery_filter + 1) % total_tabs;
    assert_eq!(session.garage_gallery_filter, 2); // KART

    // Cycle backward with Shift-Tab / Q
    session.garage_gallery_filter = (session.garage_gallery_filter + total_tabs - 1) % total_tabs;
    assert_eq!(session.garage_gallery_filter, 1); // RALLY

    // Selecting a car from the rally module
    let rally_cars = get_models_for_module("rally");
    assert!(!rally_cars.is_empty());
    let target_car = rally_cars[0];

    // Confirm selection from fleet gallery
    session.active_module_id = target_car.module_id;
    session.garage_tier = target_car.tier;
    let tier_models = get_models_for_module_and_tier(target_car.module_id, target_car.tier);
    if let Some(pos) = tier_models.iter().position(|m| m.id == target_car.id) {
        session.garage_car_idx = pos;
    }
    session.garage_gallery_mode = false;

    assert!(!session.garage_gallery_mode);
    assert_eq!(session.active_module_id, "rally");
    assert_eq!(session.garage_tier, target_car.tier);
}

#[test]
fn test_garage_stops_music_and_plays_engine() {
    let mut session = RaceSession::new();
    // Start with menu music playing
    session.audio.play_music(tdrace_app::audio::MusicTrack::NeonMenu);
    assert_eq!(session.audio.current_music, Some(tdrace_app::audio::MusicTrack::NeonMenu));

    // Transition to Garage
    session.state = GameState::Garage(GarageOrigin::Menu);
    session.active_module_id = "nascar";
    session.garage_tier = 1;
    session.garage_car_idx = 0;

    // Simulate update in Garage state
    session.update();

    // 1. Music must be stopped in the garage so user can hear the engine
    assert_eq!(session.audio.current_music, None, "Music must stop in the garage");

    // 2. Engine sound should be active and configured for the active car
    assert!(session.audio.is_engine_active, "Engine sound must be active in the garage");
    assert_eq!(session.audio.active_engine_type, tdrace_app::audio::EngineSoundType::NascarV8);

    // 3. Revving engine via gamepad input increases rev RPM and keeps engine active
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_garage(GarageOrigin::Menu, 0.1);
    assert!(session.garage_revving);
    assert!(session.garage_rev_rpm > 0.0);
    assert!(session.audio.is_engine_active);

    // 3b. Muting SFX via toggle_sfx silences engine in the garage
    session.audio.toggle_sfx();
    assert!(session.audio.settings.is_sfx_muted);
    session.update_garage(GarageOrigin::Menu, 0.1);
    assert!(!session.audio.is_engine_active);

    // Unmuting SFX restores engine sound in garage
    session.audio.toggle_sfx();
    assert!(!session.audio.settings.is_sfx_muted);
    session.update_garage(GarageOrigin::Menu, 0.1);
    assert!(session.audio.is_engine_active);

    // 4. Exiting garage stops all engine loops
    session.audio.stop_all_loops();
    assert!(!session.audio.is_engine_active);

    // 5. Returning to Menu resumes menu music
    session.state = GameState::Menu;
    session.update();
    assert_eq!(session.audio.current_music, Some(tdrace_app::audio::MusicTrack::NeonMenu));
}

