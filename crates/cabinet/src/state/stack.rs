use crate::input::GamepadSnapshot;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::CabinetTheme;

/// Context passed to screen update and draw methods.
pub struct CabinetContext<'a> {
    pub scaler: &'a UiScaler,
    pub fonts: &'a Fonts,
    pub theme: &'a CabinetTheme,
    pub gamepad: &'a GamepadSnapshot,
    pub dt: f32,
}

/// Action returned by a screen during its frame update.
pub enum ScreenAction {
    /// Keep running this screen.
    None,
    /// Pop this screen off the stack (e.g. closing a pause modal).
    Pop,
    /// Push a new modal screen on top of the current screen stack.
    Push(Box<dyn CabinetScreen>),
    /// Replace the entire screen stack with a new screen (e.g. restarting to Main Menu).
    Switch(Box<dyn CabinetScreen>),
    /// Exit the application.
    Quit,
}

/// Trait implemented by all game screens and modal overlays.
pub trait CabinetScreen {
    /// Name/identifier of the screen.
    fn name(&self) -> &str;

    /// Updates screen state and handles inputs.
    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction;

    /// Renders screen visuals.
    fn draw(&self, ctx: &CabinetContext);

    /// Whether screens underneath this modal screen should still be rendered.
    fn is_transparent(&self) -> bool {
        false
    }
}

/// Dynamic stack of active screens allowing modal dialogs, pause overlays, and menus.
pub struct ScreenStack {
    screens: Vec<Box<dyn CabinetScreen>>,
}

impl ScreenStack {
    pub fn new(root_screen: Box<dyn CabinetScreen>) -> Self {
        Self {
            screens: vec![root_screen],
        }
    }

    /// Pushes a new screen or modal onto the stack.
    pub fn push(&mut self, screen: Box<dyn CabinetScreen>) {
        self.screens.push(screen);
    }

    /// Number of active screens on the stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.screens.len()
    }

    /// Whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    /// Name of the top-most active screen, if any.
    pub fn active_screen_name(&self) -> Option<&str> {
        self.screens.last().map(|s| s.name())
    }

    /// Pops the top-most screen off the stack.
    pub fn pop(&mut self) -> Option<Box<dyn CabinetScreen>> {
        if self.screens.len() > 1 {
            self.screens.pop()
        } else {
            None
        }
    }

    /// Replaces the entire stack with a single root screen.
    pub fn switch_root(&mut self, screen: Box<dyn CabinetScreen>) {
        self.screens.clear();
        self.screens.push(screen);
    }

    /// Updates the top-most active screen and processes any returned transition action.
    pub fn update(&mut self, ctx: &mut CabinetContext) -> Option<ScreenAction> {
        if let Some(top) = self.screens.last_mut() {
            let action = top.update(ctx);
            match action {
                ScreenAction::Pop => {
                    self.pop();
                    Some(ScreenAction::Pop)
                }
                ScreenAction::Push(next) => {
                    self.push(next);
                    Some(ScreenAction::None)
                }
                ScreenAction::Switch(next) => {
                    self.switch_root(next);
                    Some(ScreenAction::None)
                }
                ScreenAction::Quit => Some(ScreenAction::Quit),
                ScreenAction::None => Some(ScreenAction::None),
            }
        } else {
            None
        }
    }

    /// Renders all visible screens from bottom to top (rendering transparent overlays over base screens).
    pub fn draw(&self, ctx: &CabinetContext) {
        if self.screens.is_empty() {
            return;
        }

        // Find the lowest visible opaque screen
        let mut start_idx = self.screens.len() - 1;
        while start_idx > 0 && self.screens[start_idx].is_transparent() {
            start_idx -= 1;
        }

        for i in start_idx..self.screens.len() {
            self.screens[i].draw(ctx);
        }
    }
}
