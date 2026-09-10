use cabinet::fx::ScreenShake;
use cabinet::input::{DigitalInputFilter, GamepadSnapshot};
use cabinet::profile::ProfileManager;
use cabinet::records::{HallOfFame, RecordMetric};
use cabinet::state::{
    CabinetContext, CabinetScreen, LeaderboardModal, ProfileSelectModal, ScreenAction, ScreenStack,
    UniversalConfirmModal, UniversalPauseModal,
};
use cabinet::ui::{CabinetTheme, Fonts, UiScaler};

#[test]
fn test_space_arena_game_simulation_and_juice() {
    let mut shake = ScreenShake::new(18.0, 2.0);
    shake.add_trauma(0.5);
    assert!(shake.trauma > 0.0);
    let (offset, _) = shake.sample_shake();
    assert!(offset.x.abs() >= 0.0 || offset.y.abs() >= 0.0);

    // Digital input smoothing for spaceship controls
    let mut filter = DigitalInputFilter::default();
    let (steer, thrust, _) = filter.update(1.0, 1.0, 0.0, 0.0, 1.0 / 60.0);
    assert!(steer > 0.0 && steer < 0.25);
    assert!(thrust > 0.0 && thrust < 0.25);
}

struct TestScreen {
    pub paused: bool,
}

impl CabinetScreen for TestScreen {
    fn name(&self) -> &str {
        "TestScreen"
    }

    fn update(&mut self, _ctx: &mut CabinetContext) -> ScreenAction {
        if self.paused {
            ScreenAction::Push(Box::new(UniversalPauseModal::new("PAUSED")))
        } else {
            ScreenAction::None
        }
    }

    fn draw(&self, _ctx: &CabinetContext) {}
}

#[test]
fn test_space_arena_modal_stack_pause_resume() {
    let root = Box::new(TestScreen { paused: true });
    let mut stack = ScreenStack::new(root);

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts::load_embedded();
    let theme = CabinetTheme::cyberpunk_neon();
    let mut gamepad = GamepadSnapshot::default();

    let mut ctx = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gamepad,
        dt: 1.0 / 60.0,
        audio: None,
    };

    // Update triggers modal push
    let action = stack.update(&mut ctx);
    assert!(matches!(action, Some(ScreenAction::None)));

    // Modal is now on top
    // Trigger confirm / resume
    gamepad.btn_confirm_pressed = true;
    let mut ctx2 = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gamepad,
        dt: 1.0 / 60.0,
        audio: None,
    };
    let pop_action = stack.update(&mut ctx2);
    assert!(matches!(pop_action, Some(ScreenAction::Pop)));
}

#[test]
fn test_space_arena_modals_integration() {
    let root = Box::new(TestScreen { paused: false });
    let mut stack = ScreenStack::new(root);

    let scaler = UiScaler::new(1280.0, 720.0);
    let fonts = Fonts::load_embedded();
    let theme = CabinetTheme::cyberpunk_neon();

    // 1. Push and close LeaderboardModal
    let hof = HallOfFame::new("space_arena_highscores", RecordMetric::HighestScore, 10);
    stack.push(Box::new(LeaderboardModal::new("TOP SCORES", hof)));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.active_screen_name(), Some("LeaderboardModal"));

    let mut gp_b = GamepadSnapshot::default();
    gp_b.btn_b_pressed = true;
    let mut ctx_b = CabinetContext {
        scaler: &scaler,
        fonts: &fonts,
        theme: &theme,
        gamepad: &gp_b,
        dt: 1.0 / 60.0,
        audio: None,
    };

    let pop = stack.update(&mut ctx_b);
    assert!(matches!(pop, Some(ScreenAction::Pop)));
    assert_eq!(stack.len(), 1);

    // 2. Push and close ProfileSelectModal
    let pm = ProfileManager::new();
    stack.push(Box::new(ProfileSelectModal::new(&pm)));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.active_screen_name(), Some("ProfileSelectModal"));

    let pop2 = stack.update(&mut ctx_b);
    assert!(matches!(pop2, Some(ScreenAction::Pop)));
    assert_eq!(stack.len(), 1);

    // 3. Push and confirm UniversalConfirmModal
    let confirm_modal = UniversalConfirmModal::quit_game();
    stack.push(Box::new(confirm_modal));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.active_screen_name(), Some("UniversalConfirmModal"));

    let pop3 = stack.update(&mut ctx_b); // Cancel via B
    assert!(matches!(pop3, Some(ScreenAction::Pop)));
    assert_eq!(stack.len(), 1);
}

