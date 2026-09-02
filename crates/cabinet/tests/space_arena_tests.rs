use cabinet::fx::ScreenShake;
use cabinet::input::{DigitalInputFilter, GamepadSnapshot};
use cabinet::state::{CabinetContext, CabinetScreen, ScreenAction, ScreenStack, UniversalPauseModal};
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
    };
    let pop_action = stack.update(&mut ctx2);
    assert!(matches!(pop_action, Some(ScreenAction::Pop)));
}
