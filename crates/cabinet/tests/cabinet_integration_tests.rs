use cabinet::audio::AudioSettings;
use cabinet::fx::{HitStop, ScreenShake};
use cabinet::input::{DigitalInputFilter, GamepadConfig, GamepadSnapshot, NavGrid2D};
use cabinet::profile::{ColorScheme, PlayerProfile, ProfileManager};
use cabinet::state::{
    ArcadeSettingsModal, CabinetContext, CabinetScreen, ScreenAction, ScreenStack,
    UniversalPauseModal,
};
use cabinet::ui::{CabinetTheme, DropdownWidget, Fonts, SliderWidget, TabBar, UiScaler};

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
    };

    let action = stack.update(&mut ctx);
    assert!(matches!(action, Some(ScreenAction::Pop)));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.active_screen_name(), Some("GameRoot"));
}
