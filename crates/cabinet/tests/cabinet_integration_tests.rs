use cabinet::audio::AudioSettings;
use cabinet::fx::{HitStop, ScreenShake};
use cabinet::input::{DigitalInputFilter, GamepadConfig, GamepadSnapshot, NavGrid2D};
use cabinet::profile::{ColorScheme, PlayerProfile, ProfileManager};
use cabinet::records::{HallOfFame, RecordEntry, RecordMetric};
use cabinet::state::{
    format_metric_score, ArcadeSettingsModal, CabinetContext, CabinetScreen, LeaderboardModal,
    ProfileSelectModal, ScreenAction, ScreenStack, UniversalConfirmModal, UniversalPauseModal,
};
use cabinet::ui::{CabinetTheme, DropdownWidget, Fonts, Palette, SliderWidget, TabBar, UiScaler};

#[test]
fn test_ui_scaler_responsive_math() {
    let scaler_desktop = UiScaler::new(1920.0, 1080.0);
    assert!(!scaler_desktop.is_mobile_aspect);
    assert!(scaler_desktop.scale >= 1.0);
    assert!(scaler_desktop.font_s(14.0) >= 14.0);

    let scaler_ultrawide = UiScaler::new(2560.0, 1080.0);
    assert!(scaler_ultrawide.is_mobile_aspect);

    let (cx, cy, cw, ch) = scaler_desktop.centered_rect(400.0, 200.0);
    assert_eq!(cw, 400.0);
    assert_eq!(ch, 200.0);
    assert_eq!(cx, (1920.0 - 400.0) * 0.5);
    assert_eq!(cy, (1080.0 - 200.0) * 0.5);
}

#[test]
fn test_digital_input_filter_progressive_ramp() {
    let mut filter = DigitalInputFilter::default();
    let dt = 1.0 / 60.0;

    let (s, t, b) = filter.update(1.0, 1.0, 0.0, 0.0, dt);
    assert!(s > 0.0 && s < 0.25);
    assert!(t > 0.0 && t < 0.25);
    assert_eq!(b, 0.0);

    // After 60 frames, steer and throttle reach 1.0
    for _ in 0..60 {
        filter.update(1.0, 1.0, 0.0, 0.0, dt);
    }
    assert_eq!(filter.current_steer, 1.0);
    assert_eq!(filter.current_throttle, 1.0);

    // Instant throttle cut
    let (_, t_cut, _) = filter.update(0.0, 0.0, 0.0, 0.0, dt);
    assert_eq!(t_cut, 0.0);
}

#[test]
fn test_nav_grid_2d_orthogonal_navigation() {
    let mut grid = NavGrid2D::new(vec![4, 3, 2]); // 3 columns
    assert_eq!(grid.active_cell(), (0, 0));

    // Nav Down
    assert!(grid.move_down());
    assert_eq!(grid.active_cell(), (0, 1));

    // Nav Right
    assert!(grid.move_right());
    assert_eq!(grid.active_cell(), (1, 0));

    // Nav Up (wraps around column 1)
    assert!(grid.move_up());
    assert_eq!(grid.active_cell(), (1, 2));

    // Nav Left
    assert!(grid.move_left());
    assert_eq!(grid.active_cell(), (0, 1)); // Preserves row 1 in column 0
}

#[test]
fn test_juice_fx_mechanics() {
    // ScreenShake
    let mut shake = ScreenShake::new(10.0, 2.0);
    shake.add_trauma(0.8);
    let (off1, _) = shake.sample_shake();
    assert!(off1.length() >= 0.0);
    shake.update(0.5); // Decay by 1.0 trauma
    assert_eq!(shake.trauma, 0.0);
    let (off2, rot2) = shake.sample_shake();
    assert_eq!(off2, glam::Vec2::ZERO);
    assert_eq!(rot2, 0.0);

    // HitStop
    let mut hitstop = HitStop::new();
    hitstop.freeze(0.1);
    let dt_effective = hitstop.step(0.05);
    assert_eq!(dt_effective, 0.0);
    let dt_resume = hitstop.step(0.06);
    assert!(dt_resume > 0.0);
    assert_eq!(hitstop.time_scale, 1.0);
}

#[test]
fn test_profile_manager_lifecycle() {
    let mut manager = ProfileManager::new();
    assert_eq!(manager.profiles.len(), 1);
    assert_eq!(manager.active_index, 0);

    let p2 = PlayerProfile::new(
        "SpeedDemon",
        "Apex",
        Some("USA"),
        ColorScheme::from_index(1),
    );
    let idx = manager.add_profile(p2);
    assert_eq!(idx, 1);
    assert_eq!(manager.active_profile().name, "SpeedDemon");
    assert_eq!(manager.active_profile().country.as_deref(), Some("USA"));
}

#[test]
fn test_audio_mixer_buses() {
    let mut settings = AudioSettings::default();
    settings.master_volume = 0.8;
    settings.sfx_volume = 0.5;
    assert!((settings.effective_sfx_volume() - 0.4).abs() < 1e-4);

    settings.is_muted = true;
    assert_eq!(settings.effective_sfx_volume(), 0.0);
    assert_eq!(settings.effective_music_volume(), 0.0);
    assert_eq!(settings.effective_ui_volume(), 0.0);
}

struct DummyScreen {
    name: String,
}

impl CabinetScreen for DummyScreen {
    fn name(&self) -> &str {
        &self.name
    }
    fn update(&mut self, _ctx: &mut CabinetContext) -> ScreenAction {
        ScreenAction::None
    }
    fn draw(&self, _ctx: &CabinetContext) {}
}

#[test]
fn test_modal_screen_stack() {
    let root = Box::new(DummyScreen { name: "RootScreen".to_string() });
    let mut stack = ScreenStack::new(root);

    let modal = Box::new(UniversalPauseModal::new("PAUSED"));
    stack.push(modal);

    let popped = stack.pop();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().name(), "UniversalPauseModal");
}

#[test]
fn test_cabinet_settings_widgets_interaction() {
    // 1. SliderWidget
    let mut slider = SliderWidget::new("Sensitivity", 0.5, 2.5, 0.1, 1.0).with_suffix("x");
    assert_eq!(slider.formatted_value(), "1.0x");
    assert_eq!(slider.normalized(), 0.25);

    // Step up and down
    assert!(slider.step_up());
    assert!((slider.value - 1.1).abs() < 1e-4);
    assert!(slider.step_down());
    assert!((slider.value - 1.0).abs() < 1e-4);

    // Ratio assignment
    slider.set_normalized(0.5);
    assert!((slider.value - 1.5).abs() < 1e-4);

    // 2. DropdownWidget
    let opts = vec!["60 FPS".to_string(), "120 FPS".to_string(), "Unlimited".to_string()];
    let mut dropdown = DropdownWidget::new("Framerate", opts, 0);
    assert_eq!(dropdown.selected_option(), "60 FPS");
    assert!(dropdown.cycle_next());
    assert_eq!(dropdown.selected_option(), "120 FPS");
    assert!(dropdown.cycle_next());
    assert_eq!(dropdown.selected_option(), "Unlimited");
    assert!(dropdown.cycle_next());
    assert_eq!(dropdown.selected_option(), "60 FPS"); // Wrap

    dropdown.toggle_open();
    assert!(dropdown.is_open);
    dropdown.close();
    assert!(!dropdown.is_open);

    // 3. TabBar
    let tabs = vec!["AUDIO".to_string(), "VIDEO".to_string(), "GAMEPLAY".to_string()];
    let mut tab_bar = TabBar::new(tabs);
    assert_eq!(tab_bar.active_tab_name(), "AUDIO");
    assert!(tab_bar.next_tab());
    assert_eq!(tab_bar.active_tab_name(), "VIDEO");
    assert!(tab_bar.prev_tab());
    assert_eq!(tab_bar.active_tab_name(), "AUDIO");
    assert!(tab_bar.prev_tab());
    assert_eq!(tab_bar.active_tab_name(), "GAMEPLAY"); // Wrap
}

#[test]
fn test_arcade_settings_modal_lifecycle_and_bindings() {
    let mut audio = AudioSettings {
        master_volume: 0.60,
        music_volume: 0.50,
        sfx_volume: 0.70,
        ui_volume: 0.80,
        is_muted: false,
    };
    let mut gp_config = GamepadConfig {
        stick_deadzone: 0.15,
        trigger_deadzone: 0.08,
        steer_exponent: 1.20,
        steer_scale: 1.10,
    };

    let mut modal = ArcadeSettingsModal::new(&audio, &gp_config);
    assert_eq!(modal.name(), "ArcadeSettingsModal");
    assert!(modal.is_transparent());
    assert_eq!(modal.tab_bar.active_tab_name(), "AUDIO");

    // Check initialized slider values
    assert!((modal.master_slider.normalized() - 0.60).abs() < 1e-4);
    assert!((modal.music_slider.normalized() - 0.50).abs() < 1e-4);
    assert!((modal.stick_deadzone_slider.value - 0.15).abs() < 1e-4);

    // Modify settings via widget API
    modal.master_slider.set_normalized(0.95);
    modal.music_slider.set_normalized(0.35);
    modal.mute_dropdown.set_selected(1); // Muted

    modal.stick_deadzone_slider.set_value(0.22);
    modal.steer_sensitivity_slider.set_value(1.45);

    // Apply to audio and gamepad structs
    modal.apply_to_audio(&mut audio);
    modal.apply_to_gamepad(&mut gp_config);

    assert!((audio.master_volume - 0.95).abs() < 1e-4);
    assert!((audio.music_volume - 0.35).abs() < 1e-4);
    assert!(audio.is_muted);

    assert!((gp_config.stick_deadzone - 0.22).abs() < 1e-4);
    assert!((gp_config.steer_scale - 1.45).abs() < 1e-4);

    // Test restore defaults
    modal.restore_defaults();
    assert_eq!(modal.mute_dropdown.selected_index, 0); // Unmuted default
    assert_eq!(modal.theme_dropdown.selected_index, 0); // Cyberpunk Neon

    // Test on ScreenStack
    let root = Box::new(DummyScreen { name: "GameRoot".to_string() });
    let mut stack = ScreenStack::new(root);
    assert_eq!(stack.len(), 1);

    stack.push(Box::new(modal));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.active_screen_name(), Some("ArcadeSettingsModal"));

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();
    let mut gamepad = GamepadSnapshot::default();
    gamepad.btn_b_pressed = true; // Cancel / Back closes modal

    let mut ctx = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gamepad,
        dt: 1.0 / 60.0,
        audio: None,
    };

    let action = stack.update(&mut ctx);
    assert!(matches!(action, Some(ScreenAction::Pop)));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.active_screen_name(), Some("GameRoot"));
}

#[test]
fn test_universal_confirm_modal_lifecycle() {
    let mut modal = UniversalConfirmModal::new("RESTART RUN", "Are you sure you want to restart?")
        .with_labels("RESTART NOW", "KEEP GOING")
        .with_accent(Palette::NEON_RED);

    assert_eq!(modal.name(), "UniversalConfirmModal");
    assert!(modal.is_transparent());
    assert_eq!(modal.confirm_label, "RESTART NOW");
    assert_eq!(modal.cancel_label, "KEEP GOING");
    assert_eq!(modal.nav.focused_col, 0); // Safety default: Cancel focused

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();

    // 1. Cancel via gamepad B
    let mut gp_cancel = GamepadSnapshot::default();
    gp_cancel.btn_b_pressed = true;
    let mut ctx_cancel = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gp_cancel,
        dt: 1.0 / 60.0,
        audio: None,
    };
    let action = modal.update(&mut ctx_cancel);
    assert!(matches!(action, ScreenAction::Pop));
    assert_eq!(modal.result, Some(false));

    // 2. Confirm: navigate to col 1 (Confirm button) and press A / confirm
    let mut modal_confirm = UniversalConfirmModal::quit_game();
    assert_eq!(modal_confirm.title, "QUIT GAME");
    modal_confirm.nav.set_focus(1, 0); // Move focus to Confirm

    let mut gp_confirm = GamepadSnapshot::default();
    gp_confirm.btn_a_pressed = true;
    let mut ctx_confirm = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gp_confirm,
        dt: 1.0 / 60.0,
        audio: None,
    };

    let action_confirm = modal_confirm.update(&mut ctx_confirm);
    assert!(matches!(action_confirm, ScreenAction::Quit));
    assert_eq!(modal_confirm.result, Some(true));
}

#[test]
fn test_leaderboard_modal_rendering_and_scrolling() {
    // 1. Metric formatting tests
    assert_eq!(format_metric_score(24.582, RecordMetric::LowestTime), "24.582s");
    assert_eq!(format_metric_score(74.238, RecordMetric::LowestTime), "1:14.238");
    assert_eq!(format_metric_score(1250400.0, RecordMetric::HighestScore), "1,250,400 PTS");

    // 2. Create HallOfFame with 20 entries
    let mut hof = HallOfFame::new("arcade_time_attack", RecordMetric::LowestTime, 25);
    for i in 1..=20 {
        hof.insert(RecordEntry {
            player_name: format!("Driver {:02}", i),
            player_alias: format!("D{:02}", i),
            country: Some(if i % 2 == 0 { "ESP".to_string() } else { "USA".to_string() }),
            score: 50.0 + (i as f64 * 1.5),
            detail: format!("Lap {}", i),
            timestamp: "2026-09-10".to_string(),
        });
    }
    assert_eq!(hof.entries.len(), 20);

    let mut modal = LeaderboardModal::new("CIRCUIT RECORD STANDINGS", hof).with_highlight(1);
    assert_eq!(modal.name(), "LeaderboardModal");
    assert!(modal.is_transparent());
    assert_eq!(modal.highlight_rank, Some(1));
    assert_eq!(modal.scroll_offset, 0);

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();

    // Navigate down past visible window (row 16 > 11 visible rows)
    modal.nav.set_focus(0, 16);
    let gp_idle = GamepadSnapshot::default();
    {
        let mut ctx = CabinetContext {
            scaler: &scaler,
            fonts: &fonts,
            theme: &theme,
            gamepad: &gp_idle,
            dt: 1.0 / 60.0,
            audio: None,
        };
        let _ = modal.update(&mut ctx);
        assert!(modal.scroll_offset > 0); // Autoscrolled to keep focused item in view
    }

    // Close modal via B button
    let mut gp_close = GamepadSnapshot::default();
    gp_close.btn_b_pressed = true;
    let mut ctx_close = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gp_close,
        dt: 1.0 / 60.0,
        audio: None,
    };
    let action = modal.update(&mut ctx_close);
    assert!(matches!(action, ScreenAction::Pop));
}

#[test]
fn test_profile_select_modal_slot_and_customization() {
    let mut manager = ProfileManager::new();
    let p2 = PlayerProfile::new("Viper Pilot", "Ghost", Some("USA"), ColorScheme::from_index(2));
    manager.add_profile(p2);
    assert_eq!(manager.profiles.len(), 2);
    assert_eq!(manager.active_index, 1);

    let mut modal = ProfileSelectModal::new(&manager);
    assert_eq!(modal.name(), "ProfileSelectModal");
    assert!(modal.is_transparent());
    assert_eq!(modal.highlighted_slot, 1);

    // Test cycle country on highlighted profile
    let orig_country = modal.manager.profiles[1].country.clone();
    modal.cycle_country(true);
    let new_country = modal.manager.profiles[1].country.clone();
    assert_ne!(orig_country, new_country);

    // Test cycle livery
    let orig_scheme = modal.manager.profiles[1].color_scheme;
    modal.cycle_livery();
    let new_scheme = modal.manager.profiles[1].color_scheme;
    assert_ne!(orig_scheme, new_scheme);

    // Switch active slot to 0
    modal.manager.select_profile(0);
    assert_eq!(modal.manager.active_index, 0);

    // Apply back to target manager
    let mut target_manager = ProfileManager::new();
    modal.apply_to_manager(&mut target_manager);
    assert_eq!(target_manager.profiles.len(), 2);
    assert_eq!(target_manager.active_index, 0);
    assert_eq!(target_manager.profiles[1].country, new_country);
    assert_eq!(target_manager.profiles[1].color_scheme, new_scheme);

    // Close modal via cancel
    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();
    let mut gp = GamepadSnapshot::default();
    gp.btn_b_pressed = true;

    let mut ctx = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gp,
        dt: 1.0 / 60.0,
        audio: None,
    };
    let action = modal.update(&mut ctx);
    assert!(matches!(action, ScreenAction::Pop));
    assert!(modal.is_saved);
}

#[test]
fn test_cabinet_context_audio_wiring_and_tactile_feedback() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use cabinet::audio::{CabinetAudioSink, SoundCue};

    struct TestSink {
        selects: AtomicUsize,
        moves: AtomicUsize,
        cancels: AtomicUsize,
    }

    impl CabinetAudioSink for TestSink {
        fn play_cue(&self, cue: SoundCue) {
            match cue {
                SoundCue::UiSelect => { self.selects.fetch_add(1, Ordering::SeqCst); }
                SoundCue::UiMove => { self.moves.fetch_add(1, Ordering::SeqCst); }
                SoundCue::UiCancel => { self.cancels.fetch_add(1, Ordering::SeqCst); }
                _ => {}
            }
        }
    }

    let sink = TestSink {
        selects: AtomicUsize::new(0),
        moves: AtomicUsize::new(0),
        cancels: AtomicUsize::new(0),
    };


    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();

    // 1. Settings navigation move
    let audio_settings = AudioSettings::default();
    let gamepad_config = GamepadConfig::default();
    let mut settings = ArcadeSettingsModal::new(&audio_settings, &gamepad_config);
    let mut gp_move = GamepadSnapshot::default();

    gp_move.nav_down = true;

    let mut ctx = CabinetContext::new(&scaler, &fonts, &theme, &gp_move, 1.0 / 60.0)
        .with_audio(Some(&sink));

    let _ = settings.update(&mut ctx);
    assert!(sink.moves.load(Ordering::SeqCst) >= 1, "Should trigger ui_move on navigation");

    // 2. Settings cancel
    let mut gp_cancel = GamepadSnapshot::default();
    gp_cancel.btn_b_pressed = true;
    let mut ctx_cancel = CabinetContext::new(&scaler, &fonts, &theme, &gp_cancel, 1.0 / 60.0)
        .with_audio(Some(&sink));

    let action = settings.update(&mut ctx_cancel);
    assert!(matches!(action, ScreenAction::Pop));
    assert_eq!(sink.cancels.load(Ordering::SeqCst), 1, "Should trigger ui_cancel on back/escape");

    // 3. Confirm modal selection
    let mut confirm = UniversalConfirmModal::quit_game();
    confirm.nav.set_focus(1, 0); // Focus confirm button
    let mut gp_confirm = GamepadSnapshot::default();
    gp_confirm.btn_a_pressed = true;
    let mut ctx_confirm = CabinetContext::new(&scaler, &fonts, &theme, &gp_confirm, 1.0 / 60.0)
        .with_audio(Some(&sink));

    let confirm_action = confirm.update(&mut ctx_confirm);
    assert!(matches!(confirm_action, ScreenAction::Quit));
    assert_eq!(sink.selects.load(Ordering::SeqCst), 1, "Should trigger ui_select on confirmation");
}

#[test]
fn test_crt_scanlines_and_settings_integration() {
    use cabinet::fx::{CrtOverlay, ScanlineMode};

    let audio = AudioSettings::default();
    let gp = GamepadConfig::default();
    let mut modal = ArcadeSettingsModal::new(&audio, &gp);

    let mut crt = CrtOverlay::default();
    assert_eq!(crt.config.mode, ScanlineMode::Disabled);

    // Scanlines dropdown selection: 2 is Arcade CRT
    modal.scanlines_dropdown.set_selected(2);
    assert_eq!(modal.scanline_mode(), ScanlineMode::ArcadeCrt);

    modal.apply_to_crt(&mut crt);
    assert_eq!(crt.config.mode, ScanlineMode::ArcadeCrt);
    assert!(crt.is_active());
    assert!((crt.effective_opacity() - ScanlineMode::ArcadeCrt.opacity()).abs() < 1e-4);

    // Check custom configuration and roll animation
    crt.config.roll_speed = 50.0;
    crt.update(0.2);
    assert!((crt.roll_offset - 10.0).abs() < 1e-4);
}

#[test]
fn test_screen_stack_transitions_lifecycle() {
    use cabinet::fx::ScreenTransition;

    let root = Box::new(DummyScreen {
        name: "Stage1".to_string(),
    });
    let mut stack = ScreenStack::new(root);

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts { display: None, ui_bold: None, ui_regular: None };
    let theme = CabinetTheme::cyberpunk_neon();
    let gp = GamepadSnapshot::default();
    let mut ctx = CabinetContext::new(&scaler, &fonts, &theme, &gp, 0.1);

    assert_eq!(stack.active_screen_name(), Some("Stage1"));
    assert!(!stack.is_transitioning());

    // Start transition to switch to Stage2 using a 0.2s fade (0.096s cover, 0.008s hold, 0.096s uncover)
    let next = Box::new(DummyScreen {
        name: "Stage2".to_string(),
    });
    stack.start_transition(ScreenTransition::fade(0.2), ScreenAction::Switch(next));

    assert!(stack.is_transitioning());
    assert_eq!(stack.active_screen_name(), Some("Stage1")); // Still Stage1 during cover

    // Step halfway (0.05s) - still covering Stage1
    ctx.dt = 0.05;
    let _ = stack.update(&mut ctx);
    assert!(stack.is_transitioning());
    assert_eq!(stack.active_screen_name(), Some("Stage1"));

    // Step across midpoint (0.06s) -> hits holding, executes swap to Stage2!
    ctx.dt = 0.06;
    let _ = stack.update(&mut ctx);
    assert_eq!(stack.active_screen_name(), Some("Stage2"));
    assert!(stack.is_transitioning());

    // Step to conclusion
    ctx.dt = 0.15;
    let _ = stack.update(&mut ctx);
    assert_eq!(stack.active_screen_name(), Some("Stage2"));
    assert!(!stack.is_transitioning());
}

#[test]
fn test_input_mapping_action_system() {
    use cabinet::input::{ArcadeAction, ArcadeKey, GamepadButton, InputMap, InputSource};

    let mut map = InputMap::default_arcade();
    let mut gp = GamepadSnapshot::default();

    // Default bindings respond to Gamepad buttons
    assert!(!map.is_down(ArcadeAction::Primary, &gp));
    gp.btn_a_pressed = true;
    assert!(map.is_pressed(ArcadeAction::Primary, &gp));

    // Custom rebinding
    map.set_bindings(
        ArcadeAction::Primary,
        vec![
            InputSource::Key(ArcadeKey::Space),
            InputSource::GamepadBtn(GamepadButton::RightBumper),
        ],
    );

    // Serialization persistence
    let json = map.to_json().expect("InputMap serialize failed");
    let deserialized = InputMap::from_json(&json).expect("InputMap deserialize failed");
    assert_eq!(map, deserialized);
}

#[test]
fn test_floating_text_popups_and_decay() {
    use cabinet::fx::FloatingTextManager;
    use glam::Vec2;

    let mut mgr = FloatingTextManager::new(32);
    assert!(mgr.is_empty());

    mgr.spawn_score(500, Vec2::new(200.0, 300.0));
    mgr.spawn_combo(4, Vec2::new(200.0, 340.0));
    mgr.spawn_alert("PERFECT!", Vec2::new(640.0, 360.0), Palette::NEON_GREEN);

    assert_eq!(mgr.count(), 3);
    assert!(!mgr.is_empty());

    // Step physics
    mgr.update(0.3);
    assert_eq!(mgr.count(), 3);

    // Fade and expiry
    mgr.update(1.2);
    assert_eq!(mgr.count(), 0);
    assert!(mgr.is_empty());
}



