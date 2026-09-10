use crate::audio::{CabinetAudioSink, SoundCue};
use crate::fx::ScreenTransition;
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
    pub audio: Option<&'a dyn CabinetAudioSink>,
}

impl<'a> CabinetContext<'a> {
    /// Creates a basic context without audio.
    pub fn new(
        scaler: &'a UiScaler,
        fonts: &'a Fonts,
        theme: &'a CabinetTheme,
        gamepad: &'a GamepadSnapshot,
        dt: f32,
    ) -> Self {
        Self {
            scaler,
            fonts,
            theme,
            gamepad,
            dt,
            audio: None,
        }
    }

    /// Builder method to attach an audio sink.
    pub fn with_audio(mut self, audio: Option<&'a dyn CabinetAudioSink>) -> Self {
        self.audio = audio;
        self
    }

    /// Plays UI select/confirm blip if an audio sink is present.
    #[inline]
    pub fn play_ui_select(&self) {
        if let Some(audio) = self.audio {
            audio.play_ui_select();
        }
    }

    /// Plays UI move/tick if an audio sink is present.
    #[inline]
    pub fn play_ui_move(&self) {
        if let Some(audio) = self.audio {
            audio.play_ui_move();
        }
    }

    /// Plays UI cancel/back sound if an audio sink is present.
    #[inline]
    pub fn play_ui_cancel(&self) {
        if let Some(audio) = self.audio {
            audio.play_ui_cancel();
        }
    }

    /// Plays an arbitrary sound cue if an audio sink is present.
    #[inline]
    pub fn play_cue(&self, cue: SoundCue) {
        if let Some(audio) = self.audio {
            audio.play_cue(cue);
        }
    }
}


/// Action returned by a screen during its frame update.
pub enum ScreenAction {
    /// Keep running this screen.
    None,
    /// Pop this screen off the stack (e.g. closing a pause modal).
    Pop,
    /// Pop this screen with a visual transition.
    PopWith(ScreenTransition),
    /// Push a new modal screen on top of the current screen stack.
    Push(Box<dyn CabinetScreen>),
    /// Push a new modal screen with a visual transition.
    PushWith(Box<dyn CabinetScreen>, ScreenTransition),
    /// Replace the entire screen stack with a new screen (e.g. restarting to Main Menu).
    Switch(Box<dyn CabinetScreen>),
    /// Replace the entire screen stack with a new screen using a visual transition.
    SwitchWith(Box<dyn CabinetScreen>, ScreenTransition),
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

enum PendingTransitionAction {
    Pop,
    Push(Box<dyn CabinetScreen>),
    Switch(Box<dyn CabinetScreen>),
}

/// Dynamic stack of active screens allowing modal dialogs, pause overlays, and menus.
pub struct ScreenStack {
    screens: Vec<Box<dyn CabinetScreen>>,
    active_transition: Option<ScreenTransition>,
    pending_action: Option<PendingTransitionAction>,
}

impl ScreenStack {
    pub fn new(root_screen: Box<dyn CabinetScreen>) -> Self {
        Self {
            screens: vec![root_screen],
            active_transition: None,
            pending_action: None,
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

    /// Starts an arcade screen transition tied to a pending screen action.
    pub fn start_transition(&mut self, trans: ScreenTransition, action: ScreenAction) {
        let pending = match action {
            ScreenAction::Pop | ScreenAction::PopWith(_) => Some(PendingTransitionAction::Pop),
            ScreenAction::Push(next) | ScreenAction::PushWith(next, _) => {
                Some(PendingTransitionAction::Push(next))
            }
            ScreenAction::Switch(next) | ScreenAction::SwitchWith(next, _) => {
                Some(PendingTransitionAction::Switch(next))
            }
            _ => None,
        };

        if let Some(act) = pending {
            self.active_transition = Some(trans);
            self.pending_action = Some(act);
        }
    }

    /// Whether a screen transition is currently active.
    pub fn is_transitioning(&self) -> bool {
        self.active_transition
            .as_ref()
            .map_or(false, |t| t.is_active())
    }

    /// Returns a reference to the active transition if present.
    pub fn transition(&self) -> Option<&ScreenTransition> {
        self.active_transition.as_ref()
    }

    /// Updates the top-most active screen and processes any returned transition action.
    pub fn update(&mut self, ctx: &mut CabinetContext) -> Option<ScreenAction> {
        let mut swap_action = None;
        let mut transition_complete = false;

        if let Some(ref mut trans) = self.active_transition {
            let swap_frame = trans.update(ctx.dt);
            if swap_frame {
                swap_action = self.pending_action.take();
            }
            if trans.is_complete() {
                transition_complete = true;
            }
        }

        if let Some(act) = swap_action {
            match act {
                PendingTransitionAction::Pop => {
                    self.pop();
                }
                PendingTransitionAction::Push(next) => {
                    self.push(next);
                }
                PendingTransitionAction::Switch(next) => {
                    self.switch_root(next);
                }
            }
        }

        if transition_complete {
            self.active_transition = None;
        }

        if let Some(top) = self.screens.last_mut() {
            let action = top.update(ctx);
            match action {
                ScreenAction::Pop => {
                    self.pop();
                    Some(ScreenAction::Pop)
                }
                ScreenAction::PopWith(trans) => {
                    self.start_transition(trans, ScreenAction::Pop);
                    Some(ScreenAction::None)
                }
                ScreenAction::Push(next) => {
                    self.push(next);
                    Some(ScreenAction::None)
                }
                ScreenAction::PushWith(next, trans) => {
                    self.start_transition(trans, ScreenAction::Push(next));
                    Some(ScreenAction::None)
                }
                ScreenAction::Switch(next) => {
                    self.switch_root(next);
                    Some(ScreenAction::None)
                }
                ScreenAction::SwitchWith(next, trans) => {
                    self.start_transition(trans, ScreenAction::Switch(next));
                    Some(ScreenAction::None)
                }
                ScreenAction::Quit => Some(ScreenAction::Quit),
                ScreenAction::None => Some(ScreenAction::None),
            }
        } else {
            None
        }
    }

    /// Renders all visible screens from bottom to top, followed by any active transition overlay.
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

        // Render transition overlay on top of all screens
        if let Some(ref trans) = self.active_transition {
            trans.render(0.0, 0.0, ctx.scaler.screen_w, ctx.scaler.screen_h);
        }
    }
}
